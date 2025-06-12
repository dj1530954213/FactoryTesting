// modbus_plc_service.rs
use async_trait::async_trait;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::Duration;
use tokio_modbus::client::Context as ModbusClientContext;
use tokio_modbus::prelude::*; // for tcp::connect_slave and Slave
use std::sync::OnceLock;

use crate::utils::error::{AppError, AppResult};
use crate::services::traits::BaseService;
use super::plc_communication_service::{
    PlcCommunicationService, PlcConnectionStatus, PlcDataType, PlcTag, PlcCommunicationStats
};

// 全局PLC连接管理器注册表
static GLOBAL_PLC_MANAGER: OnceLock<Arc<crate::services::domain::plc_connection_manager::PlcConnectionManager>> = OnceLock::new();

/// 设置全局PLC连接管理器
pub fn set_global_plc_manager(manager: Arc<crate::services::domain::plc_connection_manager::PlcConnectionManager>) {
    let _ = GLOBAL_PLC_MANAGER.set(manager);
}

/// 获取全局PLC连接管理器
pub fn get_global_plc_manager() -> Option<Arc<crate::services::domain::plc_connection_manager::PlcConnectionManager>> {
    GLOBAL_PLC_MANAGER.get().cloned()
}

// 假设 ByteOrder 和 ByteOrderConverter 存在于项目中
// 如果它们在另一个路径，需要调整 use 语句
// use crate::utils::byte_order::{ByteOrder, ByteOrderConverter};
// 暂时定义一个简单的 ByteOrder enum 以便编译，实际应使用项目中的定义
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ByteOrder {
    ABCD, // Float: AB CD, Int32: AB CD
    CDAB, // Float: CD AB, Int32: CD AB
    BADC, // Float: BA DC, Int32: BA DC
    DCBA, // Float: DC BA, Int32: DC BA
}

impl Default for ByteOrder {
    fn default() -> Self {
        ByteOrder::CDAB // 常见默认值
    }
}

// ByteOrderConverter 实现
struct ByteOrderConverter;
impl ByteOrderConverter {
    /// 将两个寄存器转换为 float32
    fn registers_to_float(reg1: u16, reg2: u16, order: ByteOrder) -> f32 {
        let bytes = match order {
            ByteOrder::ABCD => {
                // 高字在前，高字节在前：[reg1_high, reg1_low, reg2_high, reg2_low]
                [
                    (reg1 >> 8) as u8,
                    (reg1 & 0xFF) as u8,
                    (reg2 >> 8) as u8,
                    (reg2 & 0xFF) as u8,
                ]
            },
            ByteOrder::CDAB => {
                // 低字在前，高字节在前：[reg2_high, reg2_low, reg1_high, reg1_low]
                [
                    (reg2 >> 8) as u8,
                    (reg2 & 0xFF) as u8,
                    (reg1 >> 8) as u8,
                    (reg1 & 0xFF) as u8,
                ]
            },
            ByteOrder::BADC => {
                // 高字在前，低字节在前：[reg1_low, reg1_high, reg2_low, reg2_high]
                [
                    (reg1 & 0xFF) as u8,
                    (reg1 >> 8) as u8,
                    (reg2 & 0xFF) as u8,
                    (reg2 >> 8) as u8,
                ]
            },
            ByteOrder::DCBA => {
                // 低字在前，低字节在前：[reg2_low, reg2_high, reg1_low, reg1_high]
                [
                    (reg2 & 0xFF) as u8,
                    (reg2 >> 8) as u8,
                    (reg1 & 0xFF) as u8,
                    (reg1 >> 8) as u8,
                ]
            },
        };

        f32::from_be_bytes(bytes)
    }

    /// 将 float32 转换为两个寄存器
    fn float_to_registers(value: f32, order: ByteOrder) -> (u16, u16) {
        let bytes = value.to_be_bytes();

        match order {
            ByteOrder::ABCD => {
                // 高字在前，高字节在前：[bytes[0], bytes[1], bytes[2], bytes[3]]
                let reg1 = ((bytes[0] as u16) << 8) | (bytes[1] as u16);
                let reg2 = ((bytes[2] as u16) << 8) | (bytes[3] as u16);
                (reg1, reg2)
            },
            ByteOrder::CDAB => {
                // 低字在前，高字节在前：[bytes[2], bytes[3], bytes[0], bytes[1]]
                let reg1 = ((bytes[2] as u16) << 8) | (bytes[3] as u16);
                let reg2 = ((bytes[0] as u16) << 8) | (bytes[1] as u16);
                (reg1, reg2)
            },
            ByteOrder::BADC => {
                // 高字在前，低字节在前：[bytes[1], bytes[0], bytes[3], bytes[2]]
                let reg1 = ((bytes[1] as u16) << 8) | (bytes[0] as u16);
                let reg2 = ((bytes[3] as u16) << 8) | (bytes[2] as u16);
                (reg1, reg2)
            },
            ByteOrder::DCBA => {
                // 低字在前，低字节在前：[bytes[3], bytes[2], bytes[1], bytes[0]]
                let reg1 = ((bytes[3] as u16) << 8) | (bytes[2] as u16);
                let reg2 = ((bytes[1] as u16) << 8) | (bytes[0] as u16);
                (reg1, reg2)
            },
        }
    }
}


/// Modbus TCP PLC 通信服务配置
#[derive(Debug, Clone)]
pub struct ModbusConfig {
    pub ip_address: String,
    pub port: u16,
    pub slave_id: u8,
    pub byte_order: ByteOrder,
    pub connection_timeout_ms: u64,
    pub read_timeout_ms: u64,
    pub write_timeout_ms: u64,
}

impl Default for ModbusConfig {
    fn default() -> Self {
        Self {
            ip_address: "127.0.0.1".to_string(),
            port: 502,
            slave_id: 1,
            byte_order: ByteOrder::default(),
            connection_timeout_ms: 2000,
            read_timeout_ms: 1000,
            write_timeout_ms: 1000,
        }
    }
}

pub struct ModbusPlcService {
    config: ModbusConfig,
    client_context: Arc<Mutex<Option<ModbusClientContext>>>,
    connection_status: Arc<Mutex<PlcConnectionStatus>>,
    stats: Arc<Mutex<PlcCommunicationStats>>,
}

impl ModbusPlcService {
    pub fn new(config: ModbusConfig) -> Self {
        Self {
            config,
            client_context: Arc::new(Mutex::new(None)),
            connection_status: Arc::new(Mutex::new(PlcConnectionStatus::Disconnected)),
            stats: Arc::new(Mutex::new(PlcCommunicationStats::default())),
        }
    }

    fn get_socket_addr(&self) -> AppResult<SocketAddr> {
        format!("{}:{}", self.config.ip_address, self.config.port)
            .parse::<SocketAddr>()
            .map_err(|e| AppError::ConfigurationError { message: format!("无效的IP地址或端口: {}", e) })
    }

    fn get_slave(&self) -> Slave {
        Slave(self.config.slave_id)
    }

    /// Helper to parse Modbus address string like "40001" or "00001" or "10001"
    /// Returns (register_type_prefix, register_offset)
    /// Register types: 0x = Coils, 1x = Discrete Inputs, 3x = Input Registers, 4x = Holding Registers
    fn parse_modbus_address(&self, address_str: &str) -> AppResult<(char, u16)> {
        if address_str.is_empty() {
            return Err(AppError::PlcCommunicationError { message: "地址不能为空".to_string() });
        }
        let first_char = address_str.chars().next().unwrap();
        let offset_str = &address_str[1..];
        
        let offset = offset_str.parse::<u16>().map_err(|_|
            AppError::PlcCommunicationError { message: format!("无效的地址偏移量: {}", offset_str) }
        )?;

        if offset == 0 {
             return Err(AppError::PlcCommunicationError { message: "Modbus地址偏移量通常从1开始".to_string() });
        }

        // Modbus protocol addresses are 0-indexed. User addresses are 1-indexed.
        let final_offset = offset - 1;

        match first_char {
            '0' | '1' | '3' | '4' => Ok((first_char, final_offset)),
            _ => Err(AppError::PlcCommunicationError { message: format!(
                "不支持的地址类型前缀 '{}' in '{}'. 请使用 0 (线圈), 1 (离散量输入), 3 (输入寄存器), 或 4 (保持寄存器).",
                first_char, address_str
            )}),
        }
    }
}

#[async_trait]
impl BaseService for ModbusPlcService {
    fn service_name(&self) -> &'static str {
        "ModbusPlcService"
    }

    async fn initialize(&mut self) -> AppResult<()> {
        // 可以在这里尝试建立初始连接，或者在第一次操作时按需连接
        // self.connect().await?; // 可选：初始化时即连接
        println!("ModbusPlcService initialized for {}:{}", self.config.ip_address, self.config.port);
        Ok(())
    }

    async fn shutdown(&mut self) -> AppResult<()> {
        self.disconnect().await?;
        println!("ModbusPlcService shutdown.");
        Ok(())
    }

    async fn health_check(&self) -> AppResult<()> {
        match self.test_connection().await {
            Ok(true) => Ok(()),
            Ok(false) => Err(AppError::PlcCommunicationError { message: "健康检查失败: 测试连接返回false".to_string() }),
            Err(e) => Err(AppError::PlcCommunicationError { message: format!("健康检查失败: {}", e) }),
        }
    }
}

#[async_trait]
impl PlcCommunicationService for ModbusPlcService {
    async fn connect(&mut self) -> AppResult<()> {
        let mut status_guard = self.connection_status.lock().await;
        if matches!(*status_guard, PlcConnectionStatus::Connected | PlcConnectionStatus::Connecting) {
            return Ok(()); // 已经是连接或正在连接状态
        }
        *status_guard = PlcConnectionStatus::Connecting;
        drop(status_guard); // Release lock before await

        let socket_addr = self.get_socket_addr()?;
        let slave = self.get_slave();
        
        match tokio::time::timeout(
            Duration::from_millis(self.config.connection_timeout_ms),
            tokio_modbus::client::tcp::connect_slave(socket_addr, slave),
        )
        .await
        {
            Ok(Ok(ctx)) => {
                let mut client_ctx_guard = self.client_context.lock().await;
                *client_ctx_guard = Some(ctx);
                let mut status_guard = self.connection_status.lock().await;
                *status_guard = PlcConnectionStatus::Connected;
                self.stats.lock().await.connection_count += 1;
                Ok(())
            }
            Ok(Err(e)) => {
                let mut status_guard = self.connection_status.lock().await;
                *status_guard = PlcConnectionStatus::Error(format!("Modbus连接失败: {}", e));
                Err(AppError::PlcCommunicationError { message: format!("Modbus连接失败: {}", e) })
            }
            Err(_timeout_err) => {
                let mut status_guard = self.connection_status.lock().await;
                *status_guard = PlcConnectionStatus::Error("Modbus连接超时".to_string());
                Err(AppError::PlcCommunicationError { message: "Modbus连接超时".to_string() })
            }
        }
    }

    async fn disconnect(&mut self) -> AppResult<()> {
        let mut client_ctx_guard = self.client_context.lock().await;
        if let Some(ctx) = client_ctx_guard.take() {
            // The disconnect method in tokio-modbus might not be async or might not exist directly on Context
            // Context is usually dropped to close the connection.
            // Forcing a drop here.
            drop(ctx); 
        }
        let mut status_guard = self.connection_status.lock().await;
        *status_guard = PlcConnectionStatus::Disconnected;
        Ok(())
    }

    fn get_connection_status(&self) -> PlcConnectionStatus {
        // Block on the mutex for this synchronous method.
        // This might not be ideal if called from highly async contexts without spawn_blocking.
        // However, the trait defines it as synchronous.
        self.connection_status.blocking_lock().clone() 
    }

    // is_connected() has a default impl in trait

    async fn test_connection(&self) -> AppResult<bool> {
        // 尝试读取一个无关紧要的寄存器，比如第一个保持寄存器
        // 注意：地址 "40001" 代表保持寄存器地址0
        match self.read_uint16("40001").await {
            Ok(_) => Ok(true),
            Err(_) => Ok(false), // Don't propagate error, just indicate connection test failed
        }
    }

    // --- Basic Data Type Read Methods ---
    async fn read_bool_impl(&self, address: &str) -> AppResult<bool> {
        // 🔧 修复：首先检查是否有活跃的PLC连接管理器连接，如果有且能找到匹配的连接则使用
        if let Some(manager) = self.get_plc_connection_manager().await {
            // 检查是否有匹配的连接
            if let Ok(result) = self.read_bool_from_manager(&manager, address).await {
                return Ok(result);
            }
            // 如果连接管理器中没有匹配的连接，回退到独立连接模式
            // log::debug!("🔄 [ModbusPlcService] 连接管理器中无匹配连接，回退到独立连接模式: IP={}", self.config.ip_address);
        }

        // 🔧 修复：确保在独立连接模式下能够自动连接
        {
            let status = self.connection_status.lock().await;
            if !matches!(*status, PlcConnectionStatus::Connected) {
                drop(status);
                // log::debug!("🔗 [ModbusPlcService] 独立连接模式，尝试自动连接: IP={}", self.config.ip_address);
                // 需要可变引用来连接，但这里是不可变引用，所以我们需要另一种方法
                // 我们将在下面的代码中处理这个问题
            }
        }

        let (addr_type, reg_offset) = self.parse_modbus_address(address)?;
        let mut client_ctx_guard = self.client_context.lock().await;

        // 🔧 修复：如果没有连接，尝试建立连接
        if client_ctx_guard.is_none() {
            drop(client_ctx_guard);
            // log::debug!("🔗 [ModbusPlcService] 检测到未连接，尝试建立连接: IP={}", self.config.ip_address);

            // 建立连接
            let socket_addr = self.get_socket_addr()?;
            let slave = self.get_slave();

            match tokio::time::timeout(
                Duration::from_millis(self.config.connection_timeout_ms),
                tokio_modbus::client::tcp::connect_slave(socket_addr, slave),
            ).await {
                Ok(Ok(ctx)) => {
                    // log::debug!("✅ [ModbusPlcService] 独立连接建立成功: IP={}", self.config.ip_address);
                    let mut client_ctx_guard = self.client_context.lock().await;
                    *client_ctx_guard = Some(ctx);
                    let mut status_guard = self.connection_status.lock().await;
                    *status_guard = PlcConnectionStatus::Connected;
                },
                Ok(Err(e)) => {
                    log::error!("❌ [ModbusPlcService] 独立连接失败: IP={}, 错误={}", self.config.ip_address, e);
                    return Err(AppError::PlcCommunicationError {
                        message: format!("连接失败: {}", e)
                    });
                },
                Err(_) => {
                    log::error!("❌ [ModbusPlcService] 独立连接超时: IP={}", self.config.ip_address);
                    return Err(AppError::PlcCommunicationError {
                        message: "连接超时".to_string()
                    });
                }
            }

            // 重新获取连接
            client_ctx_guard = self.client_context.lock().await;
        }

        let ctx = client_ctx_guard.as_mut().ok_or_else(|| AppError::PlcCommunicationError { message: "连接失败".to_string() })?;
        
        let start_time = chrono::Utc::now();
        
        let modbus_io_result = match addr_type {
            '0' => ctx.read_coils(reg_offset, 1).await,
            '1' => ctx.read_discrete_inputs(reg_offset, 1).await,
            _ => return Err(AppError::PlcCommunicationError { message: format!("地址 {} 不是有效的布尔型 (线圈或离散量输入) 地址", address) }),
        };

        let values: Vec<bool> = match modbus_io_result { // Outer Result for IO errors
            Ok(modbus_protocol_result) => { // Inner Result for Modbus exceptions
                match modbus_protocol_result {
                    Ok(v) => v,
                    Err(e_code) => return Err(AppError::PlcCommunicationError{ 
                        message: format!("Modbus协议错误 (读取布尔值): {:?}", e_code) 
                    }),
                }
            },
            Err(io_err) => return Err(AppError::PlcCommunicationError{ 
                message: format!("Modbus IO错误 (读取布尔值): {}", io_err) 
            }),
        };

        let duration = chrono::Utc::now().signed_duration_since(start_time).num_milliseconds() as u64;
        let mut stats = self.stats.lock().await;
        stats.total_read_time_ms += duration;
        stats.successful_reads += 1;

        values.get(0).copied().ok_or_else(|| AppError::PlcCommunicationError { message: "读取布尔值时返回为空".to_string() })
    }

    async fn read_int8(&self, address: &str) -> AppResult<i8> {
        let value_u16 = self.read_uint16(address).await?;
        Ok((value_u16 & 0xFF) as i8)
    }
    
    async fn read_uint8(&self, address: &str) -> AppResult<u8> {
        let value_u16 = self.read_uint16(address).await?;
        Ok((value_u16 & 0xFF) as u8)
    }

    async fn read_int16(&self, address: &str) -> AppResult<i16> {
        self.read_uint16(address).await.map(|val| val as i16)
    }

    async fn read_uint16(&self, address: &str) -> AppResult<u16> {
        let (addr_type, reg_offset) = self.parse_modbus_address(address)?;
        let mut client_ctx_guard = self.client_context.lock().await;
        let ctx = client_ctx_guard.as_mut().ok_or_else(|| AppError::PlcCommunicationError { message: "未连接".to_string() })?;

        let start_time = chrono::Utc::now();
        let modbus_io_result = match addr_type {
            '4' => ctx.read_holding_registers(reg_offset, 1).await,
            '3' => ctx.read_input_registers(reg_offset, 1).await,
            _ => return Err(AppError::PlcCommunicationError { message: format!("地址 {} 不是有效的16位寄存器 (保持或输入) 地址", address) }),
        };

        let values: Vec<u16> = match modbus_io_result { // Outer Result
            Ok(modbus_protocol_result) => { // Inner Result
                match modbus_protocol_result {
                    Ok(v) => v,
                    Err(e_code) => return Err(AppError::PlcCommunicationError{ 
                        message: format!("Modbus协议错误 (读取u16): {:?}", e_code) 
                    }),
                }
            },
            Err(io_err) => return Err(AppError::PlcCommunicationError{ 
                message: format!("Modbus IO错误 (读取u16): {}", io_err) 
            }),
        };

        let duration = chrono::Utc::now().signed_duration_since(start_time).num_milliseconds() as u64;
        let mut stats = self.stats.lock().await;
        stats.total_read_time_ms += duration;
        stats.successful_reads += 1;
        
        values.get(0).copied().ok_or_else(|| AppError::PlcCommunicationError { message: "读取u16时返回为空".to_string() })
    }
    
    async fn read_float32_impl(&self, address: &str) -> AppResult<f32> {
        // 🔧 修复：首先检查是否有活跃的PLC连接管理器连接，如果有且能找到匹配的连接则使用
        if let Some(manager) = self.get_plc_connection_manager().await {
            // 检查是否有匹配的连接
            if let Ok(result) = self.read_float32_from_manager(&manager, address).await {
                return Ok(result);
            }
            // 如果连接管理器中没有匹配的连接，回退到独立连接模式
            log::debug!("🔄 [ModbusPlcService] 连接管理器中无匹配连接，回退到独立连接模式: IP={}", self.config.ip_address);
        }

        // 🔧 修复：确保在独立连接模式下能够自动连接
        let (addr_type, reg_offset) = self.parse_modbus_address(address)?;
        let mut client_ctx_guard = self.client_context.lock().await;

        // 🔧 修复：如果没有连接，尝试建立连接
        if client_ctx_guard.is_none() {
            drop(client_ctx_guard);
            log::debug!("🔗 [ModbusPlcService] Float32读取检测到未连接，尝试建立连接: IP={}", self.config.ip_address);

            // 建立连接
            let socket_addr = self.get_socket_addr()?;
            let slave = self.get_slave();

            match tokio::time::timeout(
                Duration::from_millis(self.config.connection_timeout_ms),
                tokio_modbus::client::tcp::connect_slave(socket_addr, slave),
            ).await {
                Ok(Ok(ctx)) => {
                    log::debug!("✅ [ModbusPlcService] Float32读取独立连接建立成功: IP={}", self.config.ip_address);
                    let mut client_ctx_guard = self.client_context.lock().await;
                    *client_ctx_guard = Some(ctx);
                    let mut status_guard = self.connection_status.lock().await;
                    *status_guard = PlcConnectionStatus::Connected;
                },
                Ok(Err(e)) => {
                    log::error!("❌ [ModbusPlcService] Float32读取独立连接失败: IP={}, 错误={}", self.config.ip_address, e);
                    return Err(AppError::PlcCommunicationError {
                        message: format!("连接失败: {}", e)
                    });
                },
                Err(_) => {
                    log::error!("❌ [ModbusPlcService] Float32读取独立连接超时: IP={}", self.config.ip_address);
                    return Err(AppError::PlcCommunicationError {
                        message: "连接超时".to_string()
                    });
                }
            }

            // 重新获取连接
            client_ctx_guard = self.client_context.lock().await;
        }

        let ctx = client_ctx_guard.as_mut().ok_or_else(|| AppError::PlcCommunicationError { message: "连接失败".to_string() })?;

        let start_time = chrono::Utc::now();
        let modbus_io_result = match addr_type {
            '4' => ctx.read_holding_registers(reg_offset, 2).await,
            '3' => ctx.read_input_registers(reg_offset, 2).await,
            _ => return Err(AppError::PlcCommunicationError { message: format!("地址 {} 不是有效的32位寄存器 (保持或输入) 地址", address) }),
        };

        let values: Vec<u16> = match modbus_io_result { // Outer Result
            Ok(modbus_protocol_result) => { // Inner Result
                match modbus_protocol_result {
                    Ok(v) => v,
                    Err(e_code) => return Err(AppError::PlcCommunicationError{
                        message: format!("Modbus协议错误 (读取f32的寄存器): {:?}", e_code)
                    }),
                }
            },
            Err(io_err) => return Err(AppError::PlcCommunicationError{
                message: format!("Modbus IO错误 (读取f32的寄存器): {}", io_err)
            }),
        };

        let duration = chrono::Utc::now().signed_duration_since(start_time).num_milliseconds() as u64;
        let mut stats = self.stats.lock().await;
        stats.total_read_time_ms += duration;
        stats.successful_reads += 1;

        if values.len() < 2 {
            return Err(AppError::PlcCommunicationError { message: "读取f32时返回的寄存器数量不足".to_string() });
        }
        Ok(ByteOrderConverter::registers_to_float(values[0], values[1], self.config.byte_order))
    }

    // --- Basic Data Type Write Methods ---
    async fn write_bool_impl(&self, address: &str, value: bool) -> AppResult<()> {
        // 🔧 修复：首先检查是否有活跃的PLC连接管理器连接，如果有且能找到匹配的连接则使用
        if let Some(manager) = self.get_plc_connection_manager().await {
            // 检查是否有匹配的连接
            if let Ok(result) = self.write_bool_to_manager(&manager, address, value).await {
                return Ok(result);
            }
            // 如果连接管理器中没有匹配的连接，回退到独立连接模式
            log::debug!("🔄 [ModbusPlcService] 连接管理器中无匹配连接，回退到独立连接模式: IP={}", self.config.ip_address);
        }

        // 🔧 修复：确保在独立连接模式下能够自动连接
        let (addr_type, reg_offset) = self.parse_modbus_address(address)?;
        let mut client_ctx_guard = self.client_context.lock().await;

        // 🔧 修复：如果没有连接，尝试建立连接
        if client_ctx_guard.is_none() {
            drop(client_ctx_guard);
            log::debug!("🔗 [ModbusPlcService] 写入操作检测到未连接，尝试建立连接: IP={}", self.config.ip_address);

            // 建立连接
            let socket_addr = self.get_socket_addr()?;
            let slave = self.get_slave();

            match tokio::time::timeout(
                Duration::from_millis(self.config.connection_timeout_ms),
                tokio_modbus::client::tcp::connect_slave(socket_addr, slave),
            ).await {
                Ok(Ok(ctx)) => {
                    log::debug!("✅ [ModbusPlcService] 写入操作独立连接建立成功: IP={}", self.config.ip_address);
                    let mut client_ctx_guard = self.client_context.lock().await;
                    *client_ctx_guard = Some(ctx);
                    let mut status_guard = self.connection_status.lock().await;
                    *status_guard = PlcConnectionStatus::Connected;
                },
                Ok(Err(e)) => {
                    log::error!("❌ [ModbusPlcService] 写入操作独立连接失败: IP={}, 错误={}", self.config.ip_address, e);
                    return Err(AppError::PlcCommunicationError {
                        message: format!("连接失败: {}", e)
                    });
                },
                Err(_) => {
                    log::error!("❌ [ModbusPlcService] 写入操作独立连接超时: IP={}", self.config.ip_address);
                    return Err(AppError::PlcCommunicationError {
                        message: "连接超时".to_string()
                    });
                }
            }

            // 重新获取连接
            client_ctx_guard = self.client_context.lock().await;
        }

        let ctx = client_ctx_guard.as_mut().ok_or_else(|| AppError::PlcCommunicationError { message: "连接失败".to_string() })?;

        if addr_type != '0' { 
            return Err(AppError::PlcCommunicationError { message: format!("地址 {} 不是有效的可写线圈地址", address) });
        }
        
        let start_time = chrono::Utc::now();
        let modbus_io_result = ctx.write_single_coil(reg_offset, value).await;
        
        match modbus_io_result { // Outer Result
            Ok(modbus_protocol_result) => { // Inner Result
                match modbus_protocol_result {
                    Ok(_) => {},
                    Err(e_code) => return Err(AppError::PlcCommunicationError{ 
                        message: format!("Modbus协议错误 (写入线圈): {:?}", e_code) 
                    }),
                }
            },
            Err(io_err) => return Err(AppError::PlcCommunicationError{ 
                message: format!("Modbus IO错误 (写入线圈): {}", io_err) 
            }),
        };

        let duration = chrono::Utc::now().signed_duration_since(start_time).num_milliseconds() as u64;
        let mut stats = self.stats.lock().await;
        stats.total_write_time_ms += duration;
        stats.successful_writes += 1;
        Ok(())
    }

    async fn write_int8(&self, address: &str, value: i8) -> AppResult<()> {
        let current_val_u16 = self.read_uint16(address).await.unwrap_or(0);
        let new_val_u16 = (current_val_u16 & 0xFF00) | (value as u16 & 0x00FF);
        self.write_uint16(address, new_val_u16).await
    }

    async fn write_uint8(&self, address: &str, value: u8) -> AppResult<()> {
        let current_val_u16 = self.read_uint16(address).await.unwrap_or(0);
        let new_val_u16 = (current_val_u16 & 0xFF00) | (value as u16);
        self.write_uint16(address, new_val_u16).await
    }
    
    async fn write_int16(&self, address: &str, value: i16) -> AppResult<()> {
        self.write_uint16(address, value as u16).await
    }

    async fn write_uint16(&self, address: &str, value: u16) -> AppResult<()> {
        let (addr_type, reg_offset) = self.parse_modbus_address(address)?;
        let mut client_ctx_guard = self.client_context.lock().await;
        let ctx = client_ctx_guard.as_mut().ok_or_else(|| AppError::PlcCommunicationError { message: "未连接".to_string() })?;

        if addr_type != '4' { 
             return Err(AppError::PlcCommunicationError { message: format!("地址 {} 不是有效的可写保持寄存器地址", address) });
        }

        let start_time = chrono::Utc::now();
        let modbus_io_result = ctx.write_single_register(reg_offset, value).await;

        match modbus_io_result { // Outer Result
            Ok(modbus_protocol_result) => { // Inner Result
                match modbus_protocol_result {
                    Ok(_) => {},
                    Err(e_code) => return Err(AppError::PlcCommunicationError{ 
                        message: format!("Modbus协议错误 (写入u16): {:?}", e_code) 
                    }),
                }
            },
            Err(io_err) => return Err(AppError::PlcCommunicationError{ 
                message: format!("Modbus IO错误 (写入u16): {}", io_err) 
            }),
        };

        let duration = chrono::Utc::now().signed_duration_since(start_time).num_milliseconds() as u64;
        let mut stats = self.stats.lock().await;
        stats.total_write_time_ms += duration;
        stats.successful_writes += 1;
        Ok(())
    }

    async fn write_float32_impl(&self, address: &str, value: f32) -> AppResult<()> {
        // 🔧 修复：首先检查是否有活跃的PLC连接管理器连接，如果有且能找到匹配的连接则使用
        if let Some(manager) = self.get_plc_connection_manager().await {
            // 检查是否有匹配的连接
            if let Ok(result) = self.write_float32_to_manager(&manager, address, value).await {
                return Ok(result);
            }
            // 如果连接管理器中没有匹配的连接，回退到独立连接模式
            log::debug!("🔄 [ModbusPlcService] 连接管理器中无匹配连接，回退到独立连接模式: IP={}", self.config.ip_address);
        }

        // 🔧 修复：确保在独立连接模式下能够自动连接
        let (addr_type, reg_offset) = self.parse_modbus_address(address)?;
        let mut client_ctx_guard = self.client_context.lock().await;

        // 🔧 修复：如果没有连接，尝试建立连接
        if client_ctx_guard.is_none() {
            drop(client_ctx_guard);
            log::debug!("🔗 [ModbusPlcService] Float32写入检测到未连接，尝试建立连接: IP={}", self.config.ip_address);

            // 建立连接
            let socket_addr = self.get_socket_addr()?;
            let slave = self.get_slave();

            match tokio::time::timeout(
                Duration::from_millis(self.config.connection_timeout_ms),
                tokio_modbus::client::tcp::connect_slave(socket_addr, slave),
            ).await {
                Ok(Ok(ctx)) => {
                    log::debug!("✅ [ModbusPlcService] Float32写入独立连接建立成功: IP={}", self.config.ip_address);
                    let mut client_ctx_guard = self.client_context.lock().await;
                    *client_ctx_guard = Some(ctx);
                    let mut status_guard = self.connection_status.lock().await;
                    *status_guard = PlcConnectionStatus::Connected;
                },
                Ok(Err(e)) => {
                    log::error!("❌ [ModbusPlcService] Float32写入独立连接失败: IP={}, 错误={}", self.config.ip_address, e);
                    return Err(AppError::PlcCommunicationError {
                        message: format!("连接失败: {}", e)
                    });
                },
                Err(_) => {
                    log::error!("❌ [ModbusPlcService] Float32写入独立连接超时: IP={}", self.config.ip_address);
                    return Err(AppError::PlcCommunicationError {
                        message: "连接超时".to_string()
                    });
                }
            }

            // 重新获取连接
            client_ctx_guard = self.client_context.lock().await;
        }

        let ctx = client_ctx_guard.as_mut().ok_or_else(|| AppError::PlcCommunicationError { message: "连接失败".to_string() })?;

        if addr_type != '4' {
            return Err(AppError::PlcCommunicationError { message: format!("地址 {} 不是有效的可写保持寄存器地址 (用于f32)", address) });
        }

        let (reg1, reg2) = ByteOrderConverter::float_to_registers(value, self.config.byte_order);
        let registers_to_write = [reg1, reg2];

        // 🔍 详细调试信息：打印写入的寄存器内容
        log::info!("🔍 [ModbusPlcService] Float32写入调试信息:");
        log::info!("   原始值: {}", value);
        log::info!("   字节序: {:?}", self.config.byte_order);
        log::info!("   转换后寄存器: reg1=0x{:04X}({}), reg2=0x{:04X}({})", reg1, reg1, reg2, reg2);
        log::info!("   写入数组: [{}, {}] = [0x{:04X}, 0x{:04X}]", registers_to_write[0], registers_to_write[1], registers_to_write[0], registers_to_write[1]);
        log::info!("   目标地址: {}, 偏移: {}", address, reg_offset);

        // 🔍 将float32转换为字节数组来查看内存布局
        let bytes = value.to_le_bytes();
        log::info!("   Float32字节(小端): [{:02X}, {:02X}, {:02X}, {:02X}]", bytes[0], bytes[1], bytes[2], bytes[3]);
        let bytes_be = value.to_be_bytes();
        log::info!("   Float32字节(大端): [{:02X}, {:02X}, {:02X}, {:02X}]", bytes_be[0], bytes_be[1], bytes_be[2], bytes_be[3]);

        let start_time = chrono::Utc::now();
        let modbus_io_result = ctx.write_multiple_registers(reg_offset, &registers_to_write).await;

        match modbus_io_result { // Outer Result
            Ok(modbus_protocol_result) => { // Inner Result
                match modbus_protocol_result {
                    Ok(_) => {},
                    Err(e_code) => return Err(AppError::PlcCommunicationError{
                        message: format!("Modbus协议错误 (写入f32): {:?}", e_code)
                    }),
                }
            },
            Err(io_err) => return Err(AppError::PlcCommunicationError{
                message: format!("Modbus IO错误 (写入f32): {}", io_err)
            }),
        };

        let duration = chrono::Utc::now().signed_duration_since(start_time).num_milliseconds() as u64;
        let mut stats = self.stats.lock().await;
        stats.total_write_time_ms += duration;
        stats.successful_writes += 1;
        Ok(())
    }

    // --- Placeholder for other read/write methods from the trait ---
    async fn read_int32(&self, address: &str) -> AppResult<i32> { unimplemented!("read_int32 for Modbus at {}", address) }
    async fn read_uint32(&self, address: &str) -> AppResult<u32> { unimplemented!("read_uint32 for Modbus at {}", address) }
    async fn read_int64(&self, address: &str) -> AppResult<i64> { unimplemented!("read_int64 for Modbus at {}", address) }
    async fn read_uint64(&self, address: &str) -> AppResult<u64> { unimplemented!("read_uint64 for Modbus at {}", address) }
    async fn read_float64(&self, address: &str) -> AppResult<f64> { unimplemented!("read_float64 for Modbus at {}", address) }
    async fn read_string(&self, address: &str, max_length: usize) -> AppResult<String> { unimplemented!("read_string for Modbus at {}, len {}", address, max_length) }
    async fn read_bytes(&self, address: &str, length: usize) -> AppResult<Vec<u8>> { unimplemented!("read_bytes for Modbus at {}, len {}", address, length) }

    async fn write_int32(&self, address: &str, value: i32) -> AppResult<()> { unimplemented!("write_int32 for Modbus at {}, value {}", address, value) }
    async fn write_uint32(&self, address: &str, value: u32) -> AppResult<()> { unimplemented!("write_uint32 for Modbus at {}, value {}", address, value) }
    async fn write_int64(&self, address: &str, value: i64) -> AppResult<()> { unimplemented!("write_int64 for Modbus at {}, value {}", address, value) }
    async fn write_uint64(&self, address: &str, value: u64) -> AppResult<()> { unimplemented!("write_uint64 for Modbus at {}, value {}", address, value) }
    async fn write_float64(&self, address: &str, value: f64) -> AppResult<()> { unimplemented!("write_float64 for Modbus at {}, value {}", address, value) }
    async fn write_string(&self, address: &str, value: &str) -> AppResult<()> { unimplemented!("write_string for Modbus at {}, value {}", address, value) }
    async fn write_bytes(&self, address: &str, value: &[u8]) -> AppResult<()> { unimplemented!("write_bytes for Modbus at {}, value len {}", address, value.len()) }

    // --- Advanced methods ---
    async fn batch_read(&self, _addresses: &[String]) -> AppResult<std::collections::HashMap<String, serde_json::Value>> {
        unimplemented!("batch_read for Modbus")
    }
    async fn batch_write(&self, _values: &std::collections::HashMap<String, serde_json::Value>) -> AppResult<()> {
        unimplemented!("batch_write for Modbus")
    }
    async fn read_tag_info(&self, address: &str) -> AppResult<PlcTag> {
        // This would require knowing the data type beforehand or inferring it,
        // or the Modbus server providing such metadata (uncommon for basic Modbus).
        Ok(PlcTag {
            address: address.to_string(),
            data_type: PlcDataType::Float32, // Placeholder
            description: Some("Modbus Tag (info not available from basic Modbus)".to_string()),
            readable: true,
            writable: true, // Assuming, may not be true
            unit: None,
            min_value: None,
            max_value: None,
        })
    }
    async fn list_available_tags(&self) -> AppResult<Vec<PlcTag>> {
        // Standard Modbus doesn't typically support listing all available tags.
        Err(AppError::PlcCommunicationError { message: "Modbus不支持列出所有可用标签".to_string() })
    }
    
    fn get_communication_stats(&self) -> PlcCommunicationStats {
        self.stats.blocking_lock().clone()
    }
    
    fn reset_communication_stats(&mut self) {
        let mut stats_guard = self.stats.blocking_lock(); // Use blocking_lock if self is &mut
        *stats_guard = PlcCommunicationStats::default();
    }
    
    fn set_read_timeout(&mut self, timeout_ms: u32) -> AppResult<()> {
        self.config.read_timeout_ms = timeout_ms as u64;
        // Note: tokio-modbus client context might not support changing timeout on the fly after connection.
        // This might need to be set at connection time or require re-connection.
        // For now, we just update the config.
        Ok(())
    }
    
    fn set_write_timeout(&mut self, timeout_ms: u32) -> AppResult<()> {
        self.config.write_timeout_ms = timeout_ms as u64;
        Ok(())
    }
    
    async fn get_device_info(&self) -> AppResult<std::collections::HashMap<String, String>> {
        // Standard Modbus has MEI (Modbus Encapsulated Interface) Transport (Type 14)
        // for device identification, but it's not universally supported or simple to query.
        Err(AppError::PlcCommunicationError { message: "Modbus不支持获取通用设备信息".to_string() })
    }
}

impl ModbusPlcService {
    /// 获取全局PLC连接管理器实例
    async fn get_plc_connection_manager(&self) -> Option<Arc<crate::services::domain::plc_connection_manager::PlcConnectionManager>> {
        get_global_plc_manager()
    }

    /// 从PLC连接管理器读取布尔值
    async fn read_bool_from_manager(
        &self,
        manager: &Arc<crate::services::domain::plc_connection_manager::PlcConnectionManager>,
        address: &str,
    ) -> AppResult<bool> {
        use crate::services::domain::plc_connection_manager::PlcConnectionState;

        // 获取连接
        let connections = manager.connections.read().await;

        // 🔧 修复：根据当前服务的IP地址查找对应的PLC连接，而不是使用第一个找到的连接
        let target_ip = &self.config.ip_address;
        // 移除冗余的PLC连接查找日志

        for (connection_id, connection) in connections.iter() {
            // 🔧 修复：检查连接的IP地址和端口是否都匹配当前服务的配置
            if connection.config.ip_address == *target_ip &&
               connection.config.port == self.config.port as i32 &&
               connection.state == PlcConnectionState::Connected {
                // 移除冗余的PLC连接匹配日志

                if let Some(context_arc) = &connection.context {
                    let mut context = context_arc.lock().await;

                    // 解析地址并读取
                    let (addr_type, reg_offset) = self.parse_modbus_address(address)?;

                    return match addr_type {
                        '0' => { // 线圈
                            // 移除冗余的线圈读取日志
                            match context.read_coils(reg_offset, 1).await {
                                Ok(Ok(values)) => {
                                    let value = values.first().copied().unwrap_or(false);
                                    // 移除冗余的线圈读取成功日志
                                    Ok(value)
                                },
                                Ok(Err(exception)) => {
                                    log::error!("❌ [ModbusPlcService] Modbus异常: IP={}, 地址={}, 异常={:?}", target_ip, address, exception);
                                    Err(AppError::PlcCommunicationError {
                                        message: format!("Modbus异常: {:?}", exception)
                                    })
                                },
                                Err(e) => {
                                    log::error!("❌ [ModbusPlcService] 读取线圈失败: IP={}, 地址={}, 错误={:?}", target_ip, address, e);
                                    Err(AppError::PlcCommunicationError {
                                        message: format!("读取线圈失败: {:?}", e)
                                    })
                                },
                            }
                        },
                        '1' => { // 离散输入
                            log::debug!("📖 [ModbusPlcService] 读取离散输入: IP={}, 地址={}, 偏移={}", target_ip, address, reg_offset);
                            match context.read_discrete_inputs(reg_offset, 1).await {
                                Ok(Ok(values)) => {
                                    let value = values.first().copied().unwrap_or(false);
                                    log::debug!("✅ [ModbusPlcService] 离散输入读取成功: IP={}, 地址={}, 值={}", target_ip, address, value);
                                    Ok(value)
                                },
                                Ok(Err(exception)) => {
                                    log::error!("❌ [ModbusPlcService] Modbus异常: IP={}, 地址={}, 异常={:?}", target_ip, address, exception);
                                    Err(AppError::PlcCommunicationError {
                                        message: format!("Modbus异常: {:?}", exception)
                                    })
                                },
                                Err(e) => {
                                    log::error!("❌ [ModbusPlcService] 读取离散输入失败: IP={}, 地址={}, 错误={:?}", target_ip, address, e);
                                    Err(AppError::PlcCommunicationError {
                                        message: format!("读取离散输入失败: {:?}", e)
                                    })
                                },
                            }
                        },
                        _ => {
                            log::error!("❌ [ModbusPlcService] 无效的布尔型地址: IP={}, 地址={}", target_ip, address);
                            Err(AppError::PlcCommunicationError {
                                message: format!("地址 {} 不是有效的布尔型地址", address)
                            })
                        },
                    };
                }
            }
        }

        log::error!("❌ [ModbusPlcService] 未找到可用的PLC连接: IP={}", target_ip);
        Err(AppError::PlcCommunicationError {
            message: format!("未找到IP为 {} 的可用PLC连接", target_ip)
        })
    }

    /// 向PLC连接管理器写入布尔值
    async fn write_bool_to_manager(
        &self,
        manager: &Arc<crate::services::domain::plc_connection_manager::PlcConnectionManager>,
        address: &str,
        value: bool,
    ) -> AppResult<()> {
        use crate::services::domain::plc_connection_manager::PlcConnectionState;

        // 获取连接
        let connections = manager.connections.read().await;

        // 🔧 修复：根据当前服务的IP地址查找对应的PLC连接，而不是使用第一个找到的连接
        let target_ip = &self.config.ip_address;
        // 减少冗余日志 - 只在trace模式下显示连接查找信息
        log::trace!("🔍 [ModbusPlcService] 查找PLC连接进行写入: IP={}, 地址={}, 值={}", target_ip, address, value);

        for (connection_id, connection) in connections.iter() {
            // 🔧 修复：检查连接的IP地址和端口是否都匹配当前服务的配置
            if connection.config.ip_address == *target_ip &&
               connection.config.port == self.config.port as i32 &&
               connection.state == PlcConnectionState::Connected {
                log::trace!("✅ [ModbusPlcService] 找到匹配的PLC连接进行写入: ID={}, IP={}", connection_id, target_ip);

                if let Some(context_arc) = &connection.context {
                    let mut context = context_arc.lock().await;

                    // 解析地址并写入
                    let (addr_type, reg_offset) = self.parse_modbus_address(address)?;

                    return match addr_type {
                        '0' => { // 线圈
                            // 移除冗余的线圈写入日志
                            match context.write_single_coil(reg_offset, value).await {
                                Ok(_) => {
                                    // 移除冗余的线圈写入成功日志
                                    Ok(())
                                },
                                Err(e) => {
                                    log::error!("❌ [ModbusPlcService] 写入线圈失败: IP={}, 地址={}, 值={}, 错误={:?}", target_ip, address, value, e);
                                    Err(AppError::PlcCommunicationError {
                                        message: format!("写入线圈失败: {}", e)
                                    })
                                }
                            }
                        },
                        _ => {
                            log::error!("❌ [ModbusPlcService] 无效的可写布尔型地址: IP={}, 地址={}", target_ip, address);
                            Err(AppError::PlcCommunicationError {
                                message: format!("地址 {} 不是有效的可写布尔型地址", address)
                            })
                        },
                    };
                }
            }
        }

        log::error!("❌ [ModbusPlcService] 未找到可用的PLC连接进行写入: IP={}", target_ip);
        Err(AppError::PlcCommunicationError {
            message: format!("未找到IP为 {} 的可用PLC连接", target_ip)
        })
    }

    /// 从PLC连接管理器读取32位浮点数
    async fn read_float32_from_manager(
        &self,
        manager: &Arc<crate::services::domain::plc_connection_manager::PlcConnectionManager>,
        address: &str,
    ) -> AppResult<f32> {
        use crate::services::domain::plc_connection_manager::PlcConnectionState;

        // 获取连接
        let connections = manager.connections.read().await;

        // 🔧 修复：根据当前服务的IP地址查找对应的PLC连接
        let target_ip = &self.config.ip_address;
        // 移除冗余的PLC连接查找日志

        for (connection_id, connection) in connections.iter() {
            // 🔧 修复：检查连接的IP地址和端口是否都匹配当前服务的配置
            if connection.config.ip_address == *target_ip &&
               connection.config.port == self.config.port as i32 &&
               connection.state == PlcConnectionState::Connected {
                // 移除冗余的PLC连接匹配日志

                if let Some(context_arc) = &connection.context {
                    let mut context = context_arc.lock().await;

                    // 解析地址并读取
                    let (addr_type, reg_offset) = self.parse_modbus_address(address)?;

                    return match addr_type {
                        '4' => { // 保持寄存器
                            // log::debug!("📖 [ModbusPlcService] 读取保持寄存器Float32: IP={}, 地址={}, 偏移={}", target_ip, address, reg_offset);
                            // log::debug!("test");
                            match context.read_holding_registers(reg_offset, 2).await {
                                Ok(Ok(values)) => {
                                    if values.len() < 2 {
                                        log::error!("❌ [ModbusPlcService] Float32寄存器数量不足: IP={}, 地址={}", target_ip, address);
                                        return Err(AppError::PlcCommunicationError {
                                            message: "读取f32时返回的寄存器数量不足".to_string()
                                        });
                                    }
                                    let value = ByteOrderConverter::registers_to_float(values[0], values[1], self.config.byte_order);
                                    // log::debug!("✅ [ModbusPlcService] Float32读取成功: IP={}, 地址={}, 值={}", target_ip, address, value);
                                    Ok(value)
                                },
                                Ok(Err(exception)) => {
                                    log::error!("❌ [ModbusPlcService] Modbus异常读取Float32: IP={}, 地址={}, 异常={:?}", target_ip, address, exception);
                                    Err(AppError::PlcCommunicationError {
                                        message: format!("Modbus异常: {:?}", exception)
                                    })
                                },
                                Err(e) => {
                                    log::error!("❌ [ModbusPlcService] 读取保持寄存器Float32失败: IP={}, 地址={}, 错误={:?}", target_ip, address, e);
                                    Err(AppError::PlcCommunicationError {
                                        message: format!("读取保持寄存器失败: {:?}", e)
                                    })
                                },
                            }
                        },
                        '3' => { // 输入寄存器
                            log::debug!("📖 [ModbusPlcService] 读取输入寄存器Float32: IP={}, 地址={}, 偏移={}", target_ip, address, reg_offset);
                            match context.read_input_registers(reg_offset, 2).await {
                                Ok(Ok(values)) => {
                                    if values.len() < 2 {
                                        log::error!("❌ [ModbusPlcService] Float32寄存器数量不足: IP={}, 地址={}", target_ip, address);
                                        return Err(AppError::PlcCommunicationError {
                                            message: "读取f32时返回的寄存器数量不足".to_string()
                                        });
                                    }
                                    let value = ByteOrderConverter::registers_to_float(values[0], values[1], self.config.byte_order);
                                    log::debug!("✅ [ModbusPlcService] Float32读取成功: IP={}, 地址={}, 值={}", target_ip, address, value);
                                    Ok(value)
                                },
                                Ok(Err(exception)) => {
                                    log::error!("❌ [ModbusPlcService] Modbus异常读取Float32: IP={}, 地址={}, 异常={:?}", target_ip, address, exception);
                                    Err(AppError::PlcCommunicationError {
                                        message: format!("Modbus异常: {:?}", exception)
                                    })
                                },
                                Err(e) => {
                                    log::error!("❌ [ModbusPlcService] 读取输入寄存器Float32失败: IP={}, 地址={}, 错误={:?}", target_ip, address, e);
                                    Err(AppError::PlcCommunicationError {
                                        message: format!("读取输入寄存器失败: {:?}", e)
                                    })
                                },
                            }
                        },
                        _ => {
                            log::error!("❌ [ModbusPlcService] 无效的Float32地址: IP={}, 地址={}", target_ip, address);
                            Err(AppError::PlcCommunicationError {
                                message: format!("地址 {} 不是有效的32位寄存器地址", address)
                            })
                        },
                    };
                }
            }
        }

        log::error!("❌ [ModbusPlcService] 未找到可用的PLC连接读取Float32: IP={}", target_ip);
        Err(AppError::PlcCommunicationError {
            message: format!("未找到IP为 {} 的可用PLC连接", target_ip)
        })
    }

    /// 向PLC连接管理器写入32位浮点数
    async fn write_float32_to_manager(
        &self,
        manager: &Arc<crate::services::domain::plc_connection_manager::PlcConnectionManager>,
        address: &str,
        value: f32,
    ) -> AppResult<()> {
        use crate::services::domain::plc_connection_manager::PlcConnectionState;

        // 获取连接
        let connections = manager.connections.read().await;

        // 🔧 修复：根据当前服务的IP地址查找对应的PLC连接
        let target_ip = &self.config.ip_address;
        // 移除冗余的PLC连接查找日志

        for (connection_id, connection) in connections.iter() {
            // 🔧 修复：检查连接的IP地址和端口是否都匹配当前服务的配置
            if connection.config.ip_address == *target_ip &&
               connection.config.port == self.config.port as i32 &&
               connection.state == PlcConnectionState::Connected {
                // 移除冗余的PLC连接匹配日志

                if let Some(context_arc) = &connection.context {
                    let mut context = context_arc.lock().await;

                    // 解析地址并写入
                    let (addr_type, reg_offset) = self.parse_modbus_address(address)?;

                    return match addr_type {
                        '4' => { // 保持寄存器
                            log::debug!("📝 [ModbusPlcService] 写入保持寄存器Float32: IP={}, 地址={}, 偏移={}, 值={}", target_ip, address, reg_offset, value);

                            let (reg1, reg2) = ByteOrderConverter::float_to_registers(value, self.config.byte_order);
                            let registers_to_write = [reg1, reg2];

                            // 🔍 详细调试信息：打印写入的寄存器内容 (管理器方法)
                            log::info!("🔍 [ModbusPlcService] Float32写入调试信息(管理器):");
                            log::info!("   原始值: {}", value);
                            log::info!("   字节序: {:?}", self.config.byte_order);
                            log::info!("   转换后寄存器: reg1=0x{:04X}({}), reg2=0x{:04X}({})", reg1, reg1, reg2, reg2);
                            log::info!("   写入数组: [{}, {}] = [0x{:04X}, 0x{:04X}]", registers_to_write[0], registers_to_write[1], registers_to_write[0], registers_to_write[1]);
                            log::info!("   目标地址: {}, 偏移: {}", address, reg_offset);

                            // 🔍 将float32转换为字节数组来查看内存布局
                            let bytes = value.to_le_bytes();
                            log::info!("   Float32字节(小端): [{:02X}, {:02X}, {:02X}, {:02X}]", bytes[0], bytes[1], bytes[2], bytes[3]);
                            let bytes_be = value.to_be_bytes();
                            log::info!("   Float32字节(大端): [{:02X}, {:02X}, {:02X}, {:02X}]", bytes_be[0], bytes_be[1], bytes_be[2], bytes_be[3]);

                            match context.write_multiple_registers(reg_offset, &registers_to_write).await {
                                Ok(_) => {
                                    log::debug!("✅ [ModbusPlcService] Float32写入成功: IP={}, 地址={}, 值={}", target_ip, address, value);
                                    Ok(())
                                },
                                Err(e) => {
                                    log::error!("❌ [ModbusPlcService] 写入保持寄存器Float32失败: IP={}, 地址={}, 值={}, 错误={:?}", target_ip, address, value, e);
                                    Err(AppError::PlcCommunicationError {
                                        message: format!("写入保持寄存器失败: {}", e)
                                    })
                                }
                            }
                        },
                        _ => {
                            log::error!("❌ [ModbusPlcService] 无效的可写Float32地址: IP={}, 地址={}", target_ip, address);
                            Err(AppError::PlcCommunicationError {
                                message: format!("地址 {} 不是有效的可写保持寄存器地址", address)
                            })
                        },
                    };
                }
            }
        }

        log::error!("❌ [ModbusPlcService] 未找到可用的PLC连接写入Float32: IP={}", target_ip);
        Err(AppError::PlcCommunicationError {
            message: format!("未找到IP为 {} 的可用PLC连接", target_ip)
        })
    }
}