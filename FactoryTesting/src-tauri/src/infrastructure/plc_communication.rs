//! PLC通信服务实现
//!
//! 基于第二阶段定义的接口实现完整的PLC通信功能

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{Mutex, RwLock};
use tokio::time::timeout;
use tokio_modbus::prelude::*;
use std::str::FromStr;
use uuid::Uuid;
use chrono::{DateTime, Utc};

use crate::domain::services::{
    BaseService,
    PlcConnectionConfig, PlcProtocol, ConnectionHandle,
    ReadRequest, WriteRequest, ReadResult, WriteResult,
    PlcDataType, PlcValue, ConnectionStats, ConnectionTestResult
};
use crate::utils::error::{AppError, AppResult};

// 复用领域层定义的通信服务接口，避免重复定义造成类型不一致
pub use crate::domain::services::plc_communication_service::IPlcCommunicationService;

use once_cell::sync::OnceCell;

/// 全局唯一的 ModbusTcpPlcService 实例
static GLOBAL_PLC_SERVICE: OnceCell<Arc<ModbusTcpPlcService>> = OnceCell::new();

/// 获取全局 PLC 服务单例
pub fn global_plc_service() -> Arc<ModbusTcpPlcService> {
    GLOBAL_PLC_SERVICE
        .get_or_init(|| Arc::new(ModbusTcpPlcService::default()))
        .clone()
}

/// Modbus TCP连接池管理器
#[derive(Debug)]
pub struct ModbusTcpConnectionPool {
    /// 活动连接
    connections: Arc<RwLock<HashMap<String, ModbusTcpConnection>>>,

    /// 连接配置
    configs: Arc<RwLock<HashMap<String, PlcConnectionConfig>>>,

    /// 全局统计信息
    global_stats: Arc<Mutex<GlobalConnectionStats>>,
}

/// 单个Modbus TCP连接
#[derive(Debug, Clone)]
struct ModbusTcpConnection {
    /// 字节顺序
    byte_order: crate::models::ByteOrder,
    /// 地址是否从0开始
    zero_based_address: bool,
    /// 连接句柄
    handle: ConnectionHandle,

    /// Modbus客户端上下文
    context: Arc<Mutex<Option<tokio_modbus::client::Context>>>,

    /// 连接状态
    is_connected: Arc<Mutex<bool>>,

    /// 连接统计
    stats: Arc<Mutex<ConnectionStats>>,

    /// 最后心跳时间
    last_heartbeat: Arc<Mutex<DateTime<Utc>>>,

}

/// 全局连接统计
#[derive(Debug, Default)]
struct GlobalConnectionStats {
    total_connections: u64,
    active_connections: u64,
    failed_connections: u64,
    total_operations: u64,
    successful_operations: u64,
}

impl ModbusTcpConnectionPool {
    /// 创建新的连接池
    pub fn new() -> Self {
        Self {
            connections: Arc::new(RwLock::new(HashMap::new())),
            configs: Arc::new(RwLock::new(HashMap::new())),
            global_stats: Arc::new(Mutex::new(GlobalConnectionStats::default())),
        }
    }

    /// 获取或创建连接
    async fn get_or_create_connection(&self, config: &PlcConnectionConfig) -> AppResult<Arc<ModbusTcpConnection>> {
        let connections = self.connections.read().await;

        // 检查是否已有连接
        if let Some(conn) = connections.get(&config.id) {
            // 检查连接是否仍然有效
            if *conn.is_connected.lock().await {
                return Ok(Arc::new(conn.clone()));
            }
        }

        drop(connections);

        // 创建新连接
        self.create_new_connection(config).await
    }

    /// 创建新的Modbus TCP连接
    async fn create_new_connection(&self, config: &PlcConnectionConfig) -> AppResult<Arc<ModbusTcpConnection>> {
        if config.protocol != PlcProtocol::ModbusTcp {
            return Err(AppError::configuration_error(
                format!("不支持的协议类型: {:?}", config.protocol)
            ));
        }

        // 解析地址
        let socket_addr = format!("{}:{}", config.host, config.port)
            .parse::<std::net::SocketAddr>()
            .map_err(|e| AppError::configuration_error(
                format!("无效的地址格式: {}:{}, 错误: {}", config.host, config.port, e)
            ))?;

        // 获取从站ID
        let slave_id = config.protocol_params
            .get("slave_id")
            .and_then(|v| v.as_u64())
            .unwrap_or(1) as u8;

        let slave = Slave(slave_id);

        // 建立连接
        let context = timeout(
            Duration::from_millis(config.timeout_ms),
            tcp::connect_slave(socket_addr, slave)
        ).await
        .map_err(|_| AppError::timeout_error("PLC连接", "连接超时"))?
        .map_err(|e| AppError::plc_communication_error(
            format!("Modbus连接失败: {}", e)
        ))?;

        // 创建连接句柄
        let handle = ConnectionHandle {
            connection_id: config.id.clone(),
            handle_id: Uuid::new_v4().to_string(),
            protocol: config.protocol,
            created_at: Utc::now(),
            last_activity: Utc::now(),
        };

        // 创建连接统计
        let stats = ConnectionStats {
            connection_id: config.id.clone(),
            total_reads: 0,
            total_writes: 0,
            successful_reads: 0,
            successful_writes: 0,
            average_read_time_ms: 0.0,
            average_write_time_ms: 0.0,
            connection_established_at: Utc::now(),
            last_communication: Utc::now(),
            connection_errors: 0,
        };

        // 创建连接对象
        let byte_order_enum = crate::models::ByteOrder::from_str(&config.byte_order).unwrap_or_default();
        let connection = ModbusTcpConnection {
            handle: handle.clone(),
            context: Arc::new(Mutex::new(Some(context))),
            is_connected: Arc::new(Mutex::new(true)),
            stats: Arc::new(Mutex::new(stats)),
            last_heartbeat: Arc::new(Mutex::new(Utc::now())),
            byte_order: byte_order_enum,
            zero_based_address: config.zero_based_address,
        };

        // 存储连接和配置
        let mut connections = self.connections.write().await;
        let mut configs = self.configs.write().await;

        connections.insert(config.id.clone(), connection.clone());
        configs.insert(config.id.clone(), config.clone());

        // 更新全局统计
        let mut global_stats = self.global_stats.lock().await;
        global_stats.total_connections += 1;
        global_stats.active_connections += 1;

        log::info!("成功创建Modbus TCP连接: {} -> {}:{}", config.id, config.host, config.port);

        Ok(Arc::new(connection))
    }

    /// 移除连接
    async fn remove_connection(&self, connection_id: &str) -> AppResult<()> {
        let mut connections = self.connections.write().await;
        let mut configs = self.configs.write().await;

        if let Some(conn) = connections.remove(connection_id) {
            // 标记为断开
            *conn.is_connected.lock().await = false;

            // 关闭上下文
            let mut context = conn.context.lock().await;
            if let Some(ctx) = context.take() {
                drop(ctx); // 关闭连接
            }

            // 更新全局统计
            let mut global_stats = self.global_stats.lock().await;
            global_stats.active_connections = global_stats.active_connections.saturating_sub(1);

            log::info!("已移除Modbus TCP连接: {}", connection_id);
        }

        configs.remove(connection_id);

        Ok(())
    }

    /// 获取连接
    async fn get_connection(&self, handle: &ConnectionHandle) -> AppResult<Arc<ModbusTcpConnection>> {
        let connections = self.connections.read().await;

        connections.get(&handle.connection_id)
            .cloned()
            .map(Arc::new)
            .ok_or_else(|| AppError::not_found_error(
                "PLC连接",
                format!("连接不存在: {}", handle.connection_id)
            ))
    }
}

/// Modbus TCP PLC通信服务
#[derive(Debug)]
pub struct ModbusTcpPlcService {
    /// 连接池
    pool: ModbusTcpConnectionPool,

    /// 服务状态
    is_initialized: Arc<Mutex<bool>>,
    /// 多默认连接句柄映射，key 为连接ID
    default_handles: Arc<Mutex<HashMap<String, ConnectionHandle>>>,
    /// 向后兼容的最后一次默认连接句柄
    default_handle: Arc<Mutex<Option<ConnectionHandle>>>,
    /// 最后一次成功建立的默认连接配置（用于日志）
    last_default_config: Arc<Mutex<Option<PlcConnectionConfig>>>,
}

impl ModbusTcpPlcService {
    /// 返回最后一次成功建立的默认连接地址，如 127.0.0.1:502
    pub async fn last_default_address(&self) -> Option<String> {
        let guard = self.last_default_config.lock().await;
        guard.as_ref().map(|c| format!("{}:{}", c.host, c.port))
    }
    /// 创建新的服务实例
    pub fn new() -> Self {
        Self {
            pool: ModbusTcpConnectionPool::new(),
            is_initialized: Arc::new(Mutex::new(false)),
            default_handles: Arc::new(Mutex::new(HashMap::new())),
            default_handle: Arc::new(Mutex::new(None)),
            last_default_config: Arc::new(Mutex::new(None)),
        }
    }
}

impl Default for ModbusTcpPlcService {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl BaseService for ModbusTcpPlcService {
    fn service_name(&self) -> &'static str {
        "ModbusTcpPlcService"
    }

    async fn initialize(&mut self) -> AppResult<()> {
        let mut is_initialized = self.is_initialized.lock().await;
        if *is_initialized {
            return Ok(());
        }

        log::info!("初始化Modbus TCP PLC通信服务");

        *is_initialized = true;
        Ok(())
    }

    async fn shutdown(&mut self) -> AppResult<()> {
        let mut is_initialized = self.is_initialized.lock().await;
        if !*is_initialized {
            return Ok(());
        }

        log::info!("关闭Modbus TCP PLC通信服务");

        // 关闭所有连接
        let connections = self.pool.connections.read().await;
        let connection_ids: Vec<String> = connections.keys().cloned().collect();
        drop(connections);

        for connection_id in connection_ids {
            if let Err(e) = self.pool.remove_connection(&connection_id).await {
                log::warn!("关闭连接时出错 {}: {}", connection_id, e);
            }
        }

        *is_initialized = false;
        Ok(())
    }

    async fn health_check(&self) -> AppResult<()> {
        let is_initialized = self.is_initialized.lock().await;
        if !*is_initialized {
            return Err(AppError::service_initialization_error("ModbusTcpPlcService", "服务未初始化"));
        }

        // 检查活动连接数
        let global_stats = self.pool.global_stats.lock().await;
        log::debug!("PLC服务健康检查: 活动连接数 = {}", global_stats.active_connections);

        Ok(())
    }
}

#[async_trait::async_trait]
impl IPlcCommunicationService for ModbusTcpPlcService {
    async fn connect(&self, config: &PlcConnectionConfig) -> AppResult<ConnectionHandle> {
        log::info!("连接到PLC: {} ({}:{})", config.name, config.host, config.port);

        let connection = self.pool.get_or_create_connection(config).await?;

        // 保存默认句柄
        {
            // 向后兼容：更新单一默认句柄
            let mut guard = self.default_handle.lock().await;
            *guard = Some(connection.handle.clone());

            // 更新多连接句柄映射
            let mut map = self.default_handles.lock().await;
            map.insert(config.id.clone(), connection.handle.clone());

            // 记录最后成功连接的配置，便于后续日志输出
            let mut cfg_guard = self.last_default_config.lock().await;
            *cfg_guard = Some(config.clone());
        }

        Ok(connection.handle.clone())
    }

    async fn disconnect(&self, handle: &ConnectionHandle) -> AppResult<()> {
        log::info!("断开PLC连接: {}", handle.connection_id);

        self.pool.remove_connection(&handle.connection_id).await
    }

    async fn is_connected(&self, handle: &ConnectionHandle) -> AppResult<bool> {
        let connection = self.pool.get_connection(handle).await?;
        let is_connected = *connection.is_connected.lock().await;
        Ok(is_connected)
    }

    async fn read_bool(&self, handle: &ConnectionHandle, address: &str) -> AppResult<bool> {
        let connection = self.pool.get_connection(handle).await?;
        let start_time = Utc::now();

        // 获取配置信息
        let (plc_name, plc_host, plc_port) = {
            let configs = self.pool.configs.read().await;
            let config = configs.get(&handle.connection_id)
                .ok_or_else(|| AppError::not_found_error("PLC配置", &handle.connection_id))?;
            (config.name.clone(), config.host.clone(), config.port)
        };

        // 解析Modbus地址
        let (register_type, offset) = parse_modbus_address_ex(address, connection.zero_based_address)?;

        log::info!("🔍 [PLC_READ_BOOL] 开始读取布尔值: PLC={}({}:{}), 地址={}, 类型={:?}, 偏移={}",
                   plc_name, plc_host, plc_port, address, register_type, offset);

        let mut context_guard = connection.context.lock().await;
        let context = context_guard.as_mut()
            .ok_or_else(|| {
                log::error!("❌ [PLC_READ_BOOL] PLC连接已断开: PLC={}, 地址={}",
                           plc_name, address);
                AppError::plc_communication_error("连接已断开".to_string())
            })?;

        let result = match register_type {
            ModbusRegisterType::Coil => {
                match context.read_coils(offset, 1).await {
                    Ok(Ok(values)) => {
                        let value = values.first().copied().unwrap_or(false);
                        log::info!("✅ [PLC_READ_BOOL] 读取线圈成功: PLC={}, 地址={}, 值={}",
                                  plc_name, address, value);
                        value
                    },
                    Ok(Err(exception)) => {
                        log::error!("❌ [PLC_READ_BOOL] Modbus异常: PLC={}, 地址={}, 异常={:?}",
                                   plc_name, address, exception);
                        return Err(AppError::plc_communication_error(format!("Modbus异常: {:?}", exception)));
                    },
                    Err(e) => {
                        log::error!("❌ [PLC_READ_BOOL] 读取线圈失败: PLC={}, 地址={}, 错误={:?}",
                                   plc_name, address, e);
                        return Err(AppError::plc_communication_error(format!("读取线圈失败: {:?}", e)));
                    },
                }
            },
            ModbusRegisterType::DiscreteInput => {
                match context.read_discrete_inputs(offset, 1).await {
                    Ok(Ok(values)) => {
                        let value = values.first().copied().unwrap_or(false);
                        log::info!("✅ [PLC_READ_BOOL] 读取离散输入成功: PLC={}, 地址={}, 值={}",
                                  plc_name, address, value);
                        value
                    },
                    Ok(Err(exception)) => {
                        log::error!("❌ [PLC_READ_BOOL] Modbus异常: PLC={}, 地址={}, 异常={:?}",
                                   plc_name, address, exception);
                        return Err(AppError::plc_communication_error(format!("Modbus异常: {:?}", exception)));
                    },
                    Err(e) => {
                        log::error!("❌ [PLC_READ_BOOL] 读取离散输入失败: PLC={}, 地址={}, 错误={:?}",
                                   plc_name, address, e);
                        return Err(AppError::plc_communication_error(format!("读取离散输入失败: {:?}", e)));
                    },
                }
            },
            _ => {
                log::error!("❌ [PLC_READ_BOOL] 无效的布尔型地址: PLC={}, 地址={}, 类型={:?}",
                           plc_name, address, register_type);
                return Err(AppError::plc_communication_error(
                    format!("地址 {} 不是有效的布尔型地址", address)
                ));
            },
        };

        // 更新统计信息
        update_read_stats(&connection.stats, start_time).await;

        Ok(result)
    }

    async fn write_bool(&self, handle: &ConnectionHandle, address: &str, value: bool) -> AppResult<()> {
        let connection = self.pool.get_connection(handle).await?;
        let start_time = Utc::now();

        // 获取配置信息
        let plc_name = {
            let configs = self.pool.configs.read().await;
            let config = configs.get(&handle.connection_id)
                .ok_or_else(|| AppError::not_found_error("PLC配置", &handle.connection_id))?;
            config.name.clone()
        };

        // 解析Modbus地址
        let (register_type, offset) = parse_modbus_address_ex(address, connection.zero_based_address)?;

        log::info!("🔍 [PLC_WRITE_BOOL] 开始写入布尔值: PLC={}, 地址={}, 类型={:?}, 偏移={}, 值={}",
                   plc_name, address, register_type, offset, value);

        let mut context_guard = connection.context.lock().await;
        let context = context_guard.as_mut()
            .ok_or_else(|| {
                log::error!("❌ [PLC_WRITE_BOOL] PLC连接已断开: PLC={}, 地址={}",
                           plc_name, address);
                AppError::plc_communication_error("连接已断开".to_string())
            })?;

        match register_type {
            ModbusRegisterType::Coil => {
                match context.write_single_coil(offset, value).await {
                    Ok(_) => {
                        log::info!("✅ [PLC_WRITE_BOOL] 写入线圈成功: PLC={}, 地址={}, 值={}",
                                  plc_name, address, value);
                    },
                    Err(e) => {
                        log::error!("❌ [PLC_WRITE_BOOL] 写入线圈失败: PLC={}, 地址={}, 值={}, 错误={}",
                                   plc_name, address, value, e);
                        return Err(AppError::plc_communication_error(format!("写入线圈失败: {}", e)));
                    }
                }
            },
            _ => {
                log::error!("❌ [PLC_WRITE_BOOL] 无效的可写布尔型地址: PLC={}, 地址={}, 类型={:?}",
                           plc_name, address, register_type);
                return Err(AppError::plc_communication_error(
                    format!("地址 {} 不是有效的可写布尔型地址", address)
                ));
            },
        }

        // 更新统计信息
        update_write_stats(&connection.stats, start_time).await;

        Ok(())
    }

    async fn read_f32(&self, handle: &ConnectionHandle, address: &str) -> AppResult<f32> {
        let connection = self.pool.get_connection(handle).await?;
        let start_time = Utc::now();

        // 解析Modbus地址
        let (register_type, offset) = parse_modbus_address_ex(address, connection.zero_based_address)?;

        let mut context_guard = connection.context.lock().await;
        let context = context_guard.as_mut()
            .ok_or_else(|| AppError::plc_communication_error("连接已断开".to_string()))?;

        let registers = match register_type {
            ModbusRegisterType::HoldingRegister => {
                match context.read_holding_registers(offset, 2).await {
                    Ok(Ok(regs)) => regs,
                    Ok(Err(exception)) => return Err(AppError::plc_communication_error(format!("Modbus异常: {:?}", exception))),
                    Err(e) => return Err(AppError::plc_communication_error(format!("读取保持寄存器失败: {:?}", e))),
                }
            },
            ModbusRegisterType::InputRegister => {
                match context.read_input_registers(offset, 2).await {
                    Ok(Ok(regs)) => regs,
                    Ok(Err(exception)) => return Err(AppError::plc_communication_error(format!("Modbus异常: {:?}", exception))),
                    Err(e) => return Err(AppError::plc_communication_error(format!("读取输入寄存器失败: {:?}", e))),
                }
            },
            _ => return Err(AppError::plc_communication_error(
                format!("地址 {} 不是有效的32位寄存器地址", address)
            )),
        };

        if registers.len() < 2 {
            return Err(AppError::plc_communication_error(
                "读取的寄存器数量不足".to_string()
            ));
        }

        // 转换为f32 (使用大端字节序)
        let result = ByteOrderConverter::registers_to_float(registers[0], registers[1], connection.byte_order);

        // 更新统计信息
        update_read_stats(&connection.stats, start_time).await;

        Ok(result)
    }

    async fn write_f32(&self, handle: &ConnectionHandle, address: &str, value: f32) -> AppResult<()> {
        let connection = self.pool.get_connection(handle).await?;
        let start_time = Utc::now();

        // 解析Modbus地址
        let (register_type, offset) = parse_modbus_address_ex(address, connection.zero_based_address)?;

        let mut context_guard = connection.context.lock().await;
        let context = context_guard.as_mut()
            .ok_or_else(|| AppError::plc_communication_error("连接已断开".to_string()))?;

        match register_type {
            ModbusRegisterType::HoldingRegister => {
                let (reg1, reg2) = ByteOrderConverter::float_to_registers(value, connection.byte_order);
                let registers = [reg1, reg2];
                context.write_multiple_registers(offset, &registers).await
                    .map_err(|e| AppError::plc_communication_error(format!("写入保持寄存器失败: {}", e)))?;
            },
            _ => return Err(AppError::plc_communication_error(
                format!("地址 {} 不是有效的可写32位寄存器地址", address)
            )),
        }

        // 更新统计信息
        update_write_stats(&connection.stats, start_time).await;

        Ok(())
    }

    async fn read_i32(&self, handle: &ConnectionHandle, address: &str) -> AppResult<i32> {
        let connection = self.pool.get_connection(handle).await?;
        let start_time = Utc::now();

        // 解析Modbus地址
        let (register_type, offset) = parse_modbus_address_ex(address, connection.zero_based_address)?;

        let mut context_guard = connection.context.lock().await;
        let context = context_guard.as_mut()
            .ok_or_else(|| AppError::plc_communication_error("连接已断开".to_string()))?;

        let registers = match register_type {
            ModbusRegisterType::HoldingRegister => {
                match context.read_holding_registers(offset, 2).await {
                    Ok(Ok(regs)) => regs,
                    Ok(Err(exception)) => return Err(AppError::plc_communication_error(format!("Modbus异常: {:?}", exception))),
                    Err(e) => return Err(AppError::plc_communication_error(format!("读取保持寄存器失败: {:?}", e))),
                }
            },
            ModbusRegisterType::InputRegister => {
                match context.read_input_registers(offset, 2).await {
                    Ok(Ok(regs)) => regs,
                    Ok(Err(exception)) => return Err(AppError::plc_communication_error(format!("Modbus异常: {:?}", exception))),
                    Err(e) => return Err(AppError::plc_communication_error(format!("读取输入寄存器失败: {:?}", e))),
                }
            },
            _ => return Err(AppError::plc_communication_error(
                format!("地址 {} 不是有效的32位寄存器地址", address)
            )),
        };

        if registers.len() < 2 {
            return Err(AppError::plc_communication_error(
                "读取的寄存器数量不足".to_string()
            ));
        }

        // 根据连接字节序转换为 i32
        let result = ByteOrderConverter::registers_to_i32(registers[0], registers[1], connection.byte_order);

        // 更新统计信息
        update_read_stats(&connection.stats, start_time).await;

        Ok(result)
    }

    async fn write_i32(&self, handle: &ConnectionHandle, address: &str, value: i32) -> AppResult<()> {
        let connection = self.pool.get_connection(handle).await?;
        let start_time = Utc::now();

        // 解析Modbus地址
        let (register_type, offset) = parse_modbus_address_ex(address, connection.zero_based_address)?;

        let mut context_guard = connection.context.lock().await;
        let context = context_guard.as_mut()
            .ok_or_else(|| AppError::plc_communication_error("连接已断开".to_string()))?;

        match register_type {
            ModbusRegisterType::HoldingRegister => {
                let (reg1, reg2) = ByteOrderConverter::i32_to_registers(value, connection.byte_order);
                let registers = [reg1, reg2];
                context.write_multiple_registers(offset, &registers).await
                    .map_err(|e| AppError::plc_communication_error(format!("写入保持寄存器失败: {}", e)))?;
            },
            _ => return Err(AppError::plc_communication_error(
                format!("地址 {} 不是有效的可写32位寄存器地址", address)
            )),
        }

        // 更新统计信息
        update_write_stats(&connection.stats, start_time).await;

        Ok(())
    }

    async fn batch_read(&self, handle: &ConnectionHandle, requests: &[ReadRequest]) -> AppResult<Vec<ReadResult>> {
        let connection = self.pool.get_connection(handle).await?;
        let mut results = Vec::with_capacity(requests.len());

        for request in requests {
            let start_time = Utc::now();
            let result = match request.data_type {
                PlcDataType::Bool => {
                    match self.read_bool(handle, &request.address).await {
                        Ok(value) => ReadResult {
                            request_id: request.id.clone(),
                            success: true,
                            value: Some(PlcValue::Bool(value)),
                            error_message: None,
                            execution_time_ms: Utc::now().signed_duration_since(start_time).num_milliseconds() as u64,
                        },
                        Err(e) => ReadResult {
                            request_id: request.id.clone(),
                            success: false,
                            value: None,
                            error_message: Some(e.to_string()),
                            execution_time_ms: Utc::now().signed_duration_since(start_time).num_milliseconds() as u64,
                        },
                    }
                },
                PlcDataType::Float32 => {
                    match self.read_f32(handle, &request.address).await {
                        Ok(value) => ReadResult {
                            request_id: request.id.clone(),
                            success: true,
                            value: Some(PlcValue::Float32(value)),
                            error_message: None,
                            execution_time_ms: Utc::now().signed_duration_since(start_time).num_milliseconds() as u64,
                        },
                        Err(e) => ReadResult {
                            request_id: request.id.clone(),
                            success: false,
                            value: None,
                            error_message: Some(e.to_string()),
                            execution_time_ms: Utc::now().signed_duration_since(start_time).num_milliseconds() as u64,
                        },
                    }
                },
                PlcDataType::Int32 => {
                    match self.read_i32(handle, &request.address).await {
                        Ok(value) => ReadResult {
                            request_id: request.id.clone(),
                            success: true,
                            value: Some(PlcValue::Int32(value)),
                            error_message: None,
                            execution_time_ms: Utc::now().signed_duration_since(start_time).num_milliseconds() as u64,
                        },
                        Err(e) => ReadResult {
                            request_id: request.id.clone(),
                            success: false,
                            value: None,
                            error_message: Some(e.to_string()),
                            execution_time_ms: Utc::now().signed_duration_since(start_time).num_milliseconds() as u64,
                        },
                    }
                },
                _ => ReadResult {
                    request_id: request.id.clone(),
                    success: false,
                    value: None,
                    error_message: Some(format!("不支持的数据类型: {:?}", request.data_type)),
                    execution_time_ms: 0,
                },
            };

            results.push(result);
        }

        Ok(results)
    }

    async fn batch_write(&self, handle: &ConnectionHandle, requests: &[WriteRequest]) -> AppResult<Vec<WriteResult>> {
        let connection = self.pool.get_connection(handle).await?;
        let mut results = Vec::with_capacity(requests.len());

        for request in requests {
            let start_time = Utc::now();
            let result = match &request.value {
                PlcValue::Bool(value) => {
                    match self.write_bool(handle, &request.address, *value).await {
                        Ok(_) => WriteResult {
                            request_id: request.id.clone(),
                            success: true,
                            error_message: None,
                            execution_time_ms: Utc::now().signed_duration_since(start_time).num_milliseconds() as u64,
                        },
                        Err(e) => WriteResult {
                            request_id: request.id.clone(),
                            success: false,
                            error_message: Some(e.to_string()),
                            execution_time_ms: Utc::now().signed_duration_since(start_time).num_milliseconds() as u64,
                        },
                    }
                },
                PlcValue::Float32(value) => {
                    match self.write_f32(handle, &request.address, *value).await {
                        Ok(_) => WriteResult {
                            request_id: request.id.clone(),
                            success: true,
                            error_message: None,
                            execution_time_ms: Utc::now().signed_duration_since(start_time).num_milliseconds() as u64,
                        },
                        Err(e) => WriteResult {
                            request_id: request.id.clone(),
                            success: false,
                            error_message: Some(e.to_string()),
                            execution_time_ms: Utc::now().signed_duration_since(start_time).num_milliseconds() as u64,
                        },
                    }
                },
                PlcValue::Int32(value) => {
                    match self.write_i32(handle, &request.address, *value).await {
                        Ok(_) => WriteResult {
                            request_id: request.id.clone(),
                            success: true,
                            error_message: None,
                            execution_time_ms: Utc::now().signed_duration_since(start_time).num_milliseconds() as u64,
                        },
                        Err(e) => WriteResult {
                            request_id: request.id.clone(),
                            success: false,
                            error_message: Some(e.to_string()),
                            execution_time_ms: Utc::now().signed_duration_since(start_time).num_milliseconds() as u64,
                        },
                    }
                },
                _ => WriteResult {
                    request_id: request.id.clone(),
                    success: false,
                    error_message: Some(format!("不支持的数据类型: {:?}", request.value)),
                    execution_time_ms: 0,
                },
            };

            results.push(result);
        }

        Ok(results)
    }

    async fn get_connection_stats(&self, handle: &ConnectionHandle) -> AppResult<ConnectionStats> {
        let connection = self.pool.get_connection(handle).await?;
        let stats = connection.stats.lock().await;
        Ok(stats.clone())
    }

    async fn default_handle_by_id(&self, connection_id: &str) -> Option<ConnectionHandle> {
        let guard = self.default_handles.lock().await;
        guard.get(connection_id).cloned()
    }

    async fn default_handle(&self) -> Option<ConnectionHandle> {
        let guard = self.default_handle.lock().await;
        guard.clone()
    }

    async fn test_connection(&self, config: &PlcConnectionConfig) -> AppResult<ConnectionTestResult> {
        let start_time = Utc::now();

        // 尝试建立临时连接进行测试
        match self.pool.get_or_create_connection(config).await {
            Ok(connection) => {
                let connection_time = Utc::now().signed_duration_since(start_time).num_milliseconds() as u64;

                // 尝试读取一个测试地址
                let test_result = {
                    let mut context_guard = connection.context.lock().await;
                    if let Some(context) = context_guard.as_mut() {
                        // 尝试读取第一个保持寄存器
                        context.read_holding_registers(0, 1).await.is_ok()
                    } else {
                        false
                    }
                };

                Ok(ConnectionTestResult {
                    success: test_result,
                    connection_time_ms: connection_time,
                    error_message: if test_result { None } else { Some("测试读取失败".to_string()) },
                    protocol_info: Some("Modbus TCP".to_string()),
                    device_info: Some(format!("{}:{}", config.host, config.port)),
                })
            },
            Err(e) => {
                let connection_time = Utc::now().signed_duration_since(start_time).num_milliseconds() as u64;
                Ok(ConnectionTestResult {
                    success: false,
                    connection_time_ms: connection_time,
                    error_message: Some(e.to_string()),
                    protocol_info: Some("Modbus TCP".to_string()),
                    device_info: Some(format!("{}:{}", config.host, config.port)),
                })
            }
        }
    }
}

/// Modbus寄存器类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModbusRegisterType {
    Coil,           // 0x 线圈
    DiscreteInput,  // 1x 离散输入
    InputRegister,  // 3x 输入寄存器
    HoldingRegister,// 4x 保持寄存器
}

/// 解析Modbus地址
///
/// 支持的格式：
/// - 0xxxx: 线圈 (Coil)
/// - 1xxxx: 离散输入 (Discrete Input)
/// - 3xxxx: 输入寄存器 (Input Register)
/// - 4xxxx: 保持寄存器 (Holding Register)
pub fn parse_modbus_address_ex(address: &str, zero_based: bool) -> AppResult<(ModbusRegisterType, u16)> {
    if address.is_empty() {
        return Err(AppError::validation_error("地址不能为空".to_string()));
    }

    if address.len() < 2 {
        return Err(AppError::validation_error(
            format!("地址格式无效: {}", address)
        ));
    }

    let first_char = address.chars().next().unwrap();
    let offset_str = &address[1..];

    let offset = offset_str.parse::<u16>()
        .map_err(|_| AppError::validation_error(
            format!("无效的地址偏移量: {}", offset_str)
        ))?;

    let protocol_offset = if zero_based { offset } else { if offset > 0 { offset - 1 } else { 0 } };

    let register_type = match first_char {
        '0' => ModbusRegisterType::Coil,
        '1' => ModbusRegisterType::DiscreteInput,
        '3' => ModbusRegisterType::InputRegister,
        '4' => ModbusRegisterType::HoldingRegister,
        _ => return Err(AppError::validation_error(
            format!("不支持的地址类型前缀: '{}' in '{}'", first_char, address)
        )),
    };

    Ok((register_type, protocol_offset))
}

/// 兼容旧代码的单参数版本，默认按1基地址（zero_based = false）
pub fn parse_modbus_address(address: &str) -> AppResult<(ModbusRegisterType, u16)> {
    parse_modbus_address_ex(address, false)
}

/// 字节序转换工具
struct ByteOrderConverter;
impl ByteOrderConverter {
    fn registers_to_float(reg1: u16, reg2: u16, order: crate::models::ByteOrder) -> f32 {
        let bytes = match order {
            crate::models::ByteOrder::ABCD => [
                (reg1 >> 8) as u8,
                (reg1 & 0xFF) as u8,
                (reg2 >> 8) as u8,
                (reg2 & 0xFF) as u8,
            ],
            crate::models::ByteOrder::CDAB => [
                (reg2 >> 8) as u8,
                (reg2 & 0xFF) as u8,
                (reg1 >> 8) as u8,
                (reg1 & 0xFF) as u8,
            ],
            crate::models::ByteOrder::BADC => [
                (reg1 & 0xFF) as u8,
                (reg1 >> 8) as u8,
                (reg2 & 0xFF) as u8,
                (reg2 >> 8) as u8,
            ],
            crate::models::ByteOrder::DCBA => [
                (reg2 & 0xFF) as u8,
                (reg2 >> 8) as u8,
                (reg1 & 0xFF) as u8,
                (reg1 >> 8) as u8,
            ],
        };
        f32::from_be_bytes(bytes)
    }

    fn float_to_registers(value: f32, order: crate::models::ByteOrder) -> (u16, u16) {
        let bytes = value.to_be_bytes();
        match order {
            crate::models::ByteOrder::ABCD => {
                let reg1 = ((bytes[0] as u16) << 8) | (bytes[1] as u16);
                let reg2 = ((bytes[2] as u16) << 8) | (bytes[3] as u16);
                (reg1, reg2)
            }
            crate::models::ByteOrder::CDAB => {
                let reg1 = ((bytes[2] as u16) << 8) | (bytes[3] as u16);
                let reg2 = ((bytes[0] as u16) << 8) | (bytes[1] as u16);
                (reg1, reg2)
            }
            crate::models::ByteOrder::BADC => {
                let reg1 = ((bytes[1] as u16) << 8) | (bytes[0] as u16);
                let reg2 = ((bytes[3] as u16) << 8) | (bytes[2] as u16);
                (reg1, reg2)
            }
            crate::models::ByteOrder::DCBA => {
                let reg1 = ((bytes[3] as u16) << 8) | (bytes[2] as u16);
                let reg2 = ((bytes[1] as u16) << 8) | (bytes[0] as u16);
                (reg1, reg2)
            }
        }
    }

    // i32 <-> registers
    fn registers_to_i32(reg1: u16, reg2: u16, order: crate::models::ByteOrder) -> i32 {
        let bytes = match order {
            crate::models::ByteOrder::ABCD => [
                (reg1 >> 8) as u8,
                (reg1 & 0xFF) as u8,
                (reg2 >> 8) as u8,
                (reg2 & 0xFF) as u8,
            ],
            crate::models::ByteOrder::CDAB => [
                (reg2 >> 8) as u8,
                (reg2 & 0xFF) as u8,
                (reg1 >> 8) as u8,
                (reg1 & 0xFF) as u8,
            ],
            crate::models::ByteOrder::BADC => [
                (reg1 & 0xFF) as u8,
                (reg1 >> 8) as u8,
                (reg2 & 0xFF) as u8,
                (reg2 >> 8) as u8,
            ],
            crate::models::ByteOrder::DCBA => [
                (reg2 & 0xFF) as u8,
                (reg2 >> 8) as u8,
                (reg1 & 0xFF) as u8,
                (reg1 >> 8) as u8,
            ],
        };
        i32::from_be_bytes(bytes)
    }

    fn i32_to_registers(value: i32, order: crate::models::ByteOrder) -> (u16, u16) {
        let bytes = value.to_be_bytes();
        match order {
            crate::models::ByteOrder::ABCD => {
                let reg1 = ((bytes[0] as u16) << 8) | (bytes[1] as u16);
                let reg2 = ((bytes[2] as u16) << 8) | (bytes[3] as u16);
                (reg1, reg2)
            }
            crate::models::ByteOrder::CDAB => {
                let reg1 = ((bytes[2] as u16) << 8) | (bytes[3] as u16);
                let reg2 = ((bytes[0] as u16) << 8) | (bytes[1] as u16);
                (reg1, reg2)
            }
            crate::models::ByteOrder::BADC => {
                let reg1 = ((bytes[1] as u16) << 8) | (bytes[0] as u16);
                let reg2 = ((bytes[3] as u16) << 8) | (bytes[2] as u16);
                (reg1, reg2)
            }
            crate::models::ByteOrder::DCBA => {
                let reg1 = ((bytes[3] as u16) << 8) | (bytes[2] as u16);
                let reg2 = ((bytes[1] as u16) << 8) | (bytes[0] as u16);
                (reg1, reg2)
            }
        }
    }

}

/// 将两个16位寄存器转换为32位浮点数 (大端字节序)
pub fn registers_to_f32(registers: &[u16]) -> f32 {
    if registers.len() < 2 {
        return 0.0;
    }

    // 大端字节序: 高位在前
    let bytes = [
        (registers[0] >> 8) as u8,
        (registers[0] & 0xFF) as u8,
        (registers[1] >> 8) as u8,
        (registers[1] & 0xFF) as u8,
    ];

    f32::from_be_bytes(bytes)
}


/// 更新读取统计信息
async fn update_read_stats(stats: &Arc<Mutex<ConnectionStats>>, start_time: DateTime<Utc>) {
    let mut stats = stats.lock().await;
    let duration = Utc::now().signed_duration_since(start_time).num_milliseconds() as u64;

    stats.total_reads += 1;
    stats.successful_reads += 1;
    stats.last_communication = Utc::now();

    // 更新平均读取时间
    let total_time = stats.average_read_time_ms * (stats.successful_reads - 1) as f64 + duration as f64;
    stats.average_read_time_ms = total_time / stats.successful_reads as f64;
}

/// 更新写入统计信息
async fn update_write_stats(stats: &Arc<Mutex<ConnectionStats>>, start_time: DateTime<Utc>) {
    let mut stats = stats.lock().await;
    let duration = Utc::now().signed_duration_since(start_time).num_milliseconds() as u64;

    stats.total_writes += 1;
    stats.successful_writes += 1;
    stats.last_communication = Utc::now();

    // 更新平均写入时间
    let total_time = stats.average_write_time_ms * (stats.successful_writes - 1) as f64 + duration as f64;
    stats.average_write_time_ms = total_time / stats.successful_writes as f64;
}


