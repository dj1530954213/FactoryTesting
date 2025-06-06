//! PLC通信服务实现
//!
//! 基于第二阶段定义的接口实现完整的PLC通信功能

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{Mutex, RwLock};
use tokio::time::timeout;
use tokio_modbus::prelude::*;
use uuid::Uuid;
use chrono::{DateTime, Utc};

use crate::domain::services::{
    IPlcCommunicationService, BaseService,
    PlcConnectionConfig, PlcProtocol, ConnectionHandle,
    ReadRequest, WriteRequest, ReadResult, WriteResult,
    PlcDataType, PlcValue, ConnectionStats, ConnectionTestResult
};
use crate::utils::error::{AppError, AppResult};

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
        let connection = ModbusTcpConnection {
            handle: handle.clone(),
            context: Arc::new(Mutex::new(Some(context))),
            is_connected: Arc::new(Mutex::new(true)),
            stats: Arc::new(Mutex::new(stats)),
            last_heartbeat: Arc::new(Mutex::new(Utc::now())),
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
}

impl ModbusTcpPlcService {
    /// 创建新的服务实例
    pub fn new() -> Self {
        Self {
            pool: ModbusTcpConnectionPool::new(),
            is_initialized: Arc::new(Mutex::new(false)),
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

        // 解析Modbus地址
        let (register_type, offset) = parse_modbus_address(address)?;

        let mut context_guard = connection.context.lock().await;
        let context = context_guard.as_mut()
            .ok_or_else(|| AppError::plc_communication_error("连接已断开".to_string()))?;

        let result = match register_type {
            ModbusRegisterType::Coil => {
                match context.read_coils(offset, 1).await {
                    Ok(Ok(values)) => values.first().copied().unwrap_or(false),
                    Ok(Err(exception)) => return Err(AppError::plc_communication_error(format!("Modbus异常: {:?}", exception))),
                    Err(e) => return Err(AppError::plc_communication_error(format!("读取线圈失败: {:?}", e))),
                }
            },
            ModbusRegisterType::DiscreteInput => {
                match context.read_discrete_inputs(offset, 1).await {
                    Ok(Ok(values)) => values.first().copied().unwrap_or(false),
                    Ok(Err(exception)) => return Err(AppError::plc_communication_error(format!("Modbus异常: {:?}", exception))),
                    Err(e) => return Err(AppError::plc_communication_error(format!("读取离散输入失败: {:?}", e))),
                }
            },
            _ => return Err(AppError::plc_communication_error(
                format!("地址 {} 不是有效的布尔型地址", address)
            )),
        };

        // 更新统计信息
        update_read_stats(&connection.stats, start_time).await;

        Ok(result)
    }

    async fn write_bool(&self, handle: &ConnectionHandle, address: &str, value: bool) -> AppResult<()> {
        let connection = self.pool.get_connection(handle).await?;
        let start_time = Utc::now();

        // 解析Modbus地址
        let (register_type, offset) = parse_modbus_address(address)?;

        let mut context_guard = connection.context.lock().await;
        let context = context_guard.as_mut()
            .ok_or_else(|| AppError::plc_communication_error("连接已断开".to_string()))?;

        match register_type {
            ModbusRegisterType::Coil => {
                context.write_single_coil(offset, value).await
                    .map_err(|e| AppError::plc_communication_error(format!("写入线圈失败: {}", e)))?;
            },
            _ => return Err(AppError::plc_communication_error(
                format!("地址 {} 不是有效的可写布尔型地址", address)
            )),
        }

        // 更新统计信息
        update_write_stats(&connection.stats, start_time).await;

        Ok(())
    }

    async fn read_f32(&self, handle: &ConnectionHandle, address: &str) -> AppResult<f32> {
        let connection = self.pool.get_connection(handle).await?;
        let start_time = Utc::now();

        // 解析Modbus地址
        let (register_type, offset) = parse_modbus_address(address)?;

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
        let result = registers_to_f32(&registers[0..2]);

        // 更新统计信息
        update_read_stats(&connection.stats, start_time).await;

        Ok(result)
    }

    async fn write_f32(&self, handle: &ConnectionHandle, address: &str, value: f32) -> AppResult<()> {
        let connection = self.pool.get_connection(handle).await?;
        let start_time = Utc::now();

        // 解析Modbus地址
        let (register_type, offset) = parse_modbus_address(address)?;

        let mut context_guard = connection.context.lock().await;
        let context = context_guard.as_mut()
            .ok_or_else(|| AppError::plc_communication_error("连接已断开".to_string()))?;

        match register_type {
            ModbusRegisterType::HoldingRegister => {
                let registers = f32_to_registers(value);
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
        let (register_type, offset) = parse_modbus_address(address)?;

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

        // 转换为i32 (使用大端字节序)
        let result = registers_to_i32(&registers[0..2]);

        // 更新统计信息
        update_read_stats(&connection.stats, start_time).await;

        Ok(result)
    }

    async fn write_i32(&self, handle: &ConnectionHandle, address: &str, value: i32) -> AppResult<()> {
        let connection = self.pool.get_connection(handle).await?;
        let start_time = Utc::now();

        // 解析Modbus地址
        let (register_type, offset) = parse_modbus_address(address)?;

        let mut context_guard = connection.context.lock().await;
        let context = context_guard.as_mut()
            .ok_or_else(|| AppError::plc_communication_error("连接已断开".to_string()))?;

        match register_type {
            ModbusRegisterType::HoldingRegister => {
                let registers = i32_to_registers(value);
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
pub fn parse_modbus_address(address: &str) -> AppResult<(ModbusRegisterType, u16)> {
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

    // Modbus地址通常从1开始，但协议中是从0开始
    let protocol_offset = if offset > 0 { offset - 1 } else { 0 };

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

/// 将32位浮点数转换为两个16位寄存器 (大端字节序)
pub fn f32_to_registers(value: f32) -> [u16; 2] {
    let bytes = value.to_be_bytes();
    [
        ((bytes[0] as u16) << 8) | (bytes[1] as u16),
        ((bytes[2] as u16) << 8) | (bytes[3] as u16),
    ]
}

/// 将两个16位寄存器转换为32位整数 (大端字节序)
pub fn registers_to_i32(registers: &[u16]) -> i32 {
    if registers.len() < 2 {
        return 0;
    }

    // 大端字节序: 高位在前
    let bytes = [
        (registers[0] >> 8) as u8,
        (registers[0] & 0xFF) as u8,
        (registers[1] >> 8) as u8,
        (registers[1] & 0xFF) as u8,
    ];

    i32::from_be_bytes(bytes)
}

/// 将32位整数转换为两个16位寄存器 (大端字节序)
pub fn i32_to_registers(value: i32) -> [u16; 2] {
    let bytes = value.to_be_bytes();
    [
        ((bytes[0] as u16) << 8) | (bytes[1] as u16),
        ((bytes[2] as u16) << 8) | (bytes[3] as u16),
    ]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_modbus_address() {
        // 测试线圈地址
        let (reg_type, offset) = parse_modbus_address("00001").unwrap();
        assert_eq!(reg_type, ModbusRegisterType::Coil);
        assert_eq!(offset, 0);

        // 测试保持寄存器地址
        let (reg_type, offset) = parse_modbus_address("40001").unwrap();
        assert_eq!(reg_type, ModbusRegisterType::HoldingRegister);
        assert_eq!(offset, 0);

        // 测试输入寄存器地址
        let (reg_type, offset) = parse_modbus_address("30100").unwrap();
        assert_eq!(reg_type, ModbusRegisterType::InputRegister);
        assert_eq!(offset, 99);

        // 测试无效地址
        assert!(parse_modbus_address("").is_err());
        assert!(parse_modbus_address("5001").is_err());
        assert!(parse_modbus_address("4abc").is_err());
    }

    #[test]
    fn test_f32_conversion() {
        let test_value = 123.456f32;
        let registers = f32_to_registers(test_value);
        let converted_back = registers_to_f32(&registers);

        // 由于浮点数精度问题，使用近似比较
        assert!((test_value - converted_back).abs() < 0.001);
    }

    #[test]
    fn test_i32_conversion() {
        let test_value = 123456i32;
        let registers = i32_to_registers(test_value);
        let converted_back = registers_to_i32(&registers);

        assert_eq!(test_value, converted_back);
    }
}

