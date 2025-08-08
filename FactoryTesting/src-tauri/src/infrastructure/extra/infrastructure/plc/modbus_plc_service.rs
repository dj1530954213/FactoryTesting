//! # Modbus PLC服务实现模块
//!
//! ## 业务作用
//! 本模块实现了基于Modbus TCP协议的PLC通信服务，提供：
//! - 完整的Modbus TCP协议实现
//! - 多种数据类型的读写操作（布尔、整数、浮点数等）
//! - 灵活的字节序配置支持
//! - 连接状态管理和自动重连机制
//! - 与全局连接管理器的集成
//!
//! ## 技术特点
//! - **协议支持**: 完整实现Modbus TCP协议规范
//! - **数据转换**: 支持多种字节序的浮点数转换
//! - **异步操作**: 基于tokio的异步I/O操作
//! - **连接管理**: 智能连接池和状态管理
//! - **错误处理**: 详细的错误分类和恢复机制
//!
//! ## Rust知识点
//! - **async/await**: 异步编程模式
//! - **OnceLock**: 线程安全的延迟初始化
//! - **Arc<Mutex<T>>**: 多线程共享状态管理
//! - **trait实现**: 为具体类型实现抽象接口
//! - **字节操作**: 底层字节序转换和位操作

/*use async_trait::async_trait;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::Duration;
use tokio_modbus::client::Context as ModbusClientContext;
use tokio_modbus::prelude::*; // for tcp::connect_slave and Slave
use std::sync::OnceLock;
use std::str::FromStr;

// 导入日志宏
use crate::{log_communication_failure, log_user_operation};

use crate::utils::error::{AppError, AppResult};
use crate::domain::services::BaseService;
use super::plc_communication_service::{
    PlcCommunicationService, PlcConnectionStatus, PlcDataType, PlcTag, PlcCommunicationStats
};
use crate::models::test_plc_config::PlcConnectionConfig;

/// 全局PLC连接管理器注册表
///
/// **业务作用**: 提供全局访问点，用于获取PLC连接管理器实例
/// **线程安全**: OnceLock确保多线程环境下的安全初始化
/// **单例模式**: 全局唯一的管理器实例，避免重复创建
///
/// **Rust知识点**:
/// - `OnceLock<T>`: 线程安全的延迟初始化容器
/// - `static`: 全局静态变量，程序生命周期内存在
/// - `Arc<T>`: 原子引用计数，支持多线程共享所有权
static GLOBAL_PLC_MANAGER: OnceLock<Arc<crate::domain::plc_connection_manager::PlcConnectionManager>> = OnceLock::new();

/// 设置全局PLC连接管理器
///
/// **业务作用**: 在应用启动时注册PLC连接管理器
/// **调用时机**: 通常在应用初始化阶段调用一次
/// **线程安全**: 只能设置一次，后续调用会被忽略
///
/// **参数**: `manager` - PLC连接管理器的Arc智能指针
pub fn set_global_plc_manager(manager: Arc<crate::domain::plc_connection_manager::PlcConnectionManager>) {
    let _ = GLOBAL_PLC_MANAGER.set(manager); // 忽略返回值，设置失败表示已经设置过
}

/// 获取全局PLC连接管理器
///
/// **业务作用**: 为其他模块提供PLC连接管理器的访问接口
/// **返回值**: `Option<Arc<T>>` - 可能为空的管理器引用
/// **使用场景**: 在需要查询连接状态或管理连接时调用
///
/// **Rust知识点**:
/// - `Option<T>`: 表示可能存在或不存在的值
/// - `cloned()`: 对Option内的Arc进行克隆，增加引用计数
pub fn get_global_plc_manager() -> Option<Arc<crate::domain::plc_connection_manager::PlcConnectionManager>> {
    GLOBAL_PLC_MANAGER.get().cloned()
}

// 引入全局 ByteOrder 枚举，替代本文件的临时定义
// **模块化设计**: 使用统一的字节序定义，避免重复代码
use crate::models::ByteOrder;

/// 字节序转换器
///
/// **业务作用**:
/// - 处理不同PLC厂商的字节序差异
/// - 实现浮点数与Modbus寄存器之间的转换
/// - 支持四种常见的字节序格式
///
/// **技术背景**:
/// - Modbus协议使用16位寄存器存储数据
/// - 32位浮点数需要占用2个连续寄存器
/// - 不同厂商对字节序的处理方式不同
///
/// **Rust知识点**:
/// - `struct`: 结构体定义，这里用作命名空间
/// - 位操作：`>>`, `&`, `|` 等位运算符
/// - 类型转换：`as u8` 显式类型转换
struct ByteOrderConverter;

impl ByteOrderConverter {
    /// 将两个Modbus寄存器转换为32位浮点数
    ///
    /// **业务逻辑**:
    /// - 根据指定的字节序重新排列字节
    /// - 将重排后的字节转换为IEEE 754浮点数
    ///
    /// **参数**:
    /// - `reg1`: 第一个16位寄存器值
    /// - `reg2`: 第二个16位寄存器值
    /// - `order`: 字节序类型
    ///
    /// **返回值**: 转换后的32位浮点数
    ///
    /// **字节序说明**:
    /// - ABCD: 高字在前，高字节在前（大端序）
    /// - CDAB: 低字在前，高字节在前
    /// - BADC: 高字在前，低字节在前
    /// - DCBA: 低字在前，低字节在前（小端序）
    fn registers_to_float(reg1: u16, reg2: u16, order: ByteOrder) -> f32 {
        let bytes = match order {
            ByteOrder::ABCD => {
                // 高字在前，高字节在前：[reg1_high, reg1_low, reg2_high, reg2_low]
                // **位操作解释**: reg1 >> 8 提取高字节，reg1 & 0xFF 提取低字节
                [
                    (reg1 >> 8) as u8,    // reg1的高字节
                    (reg1 & 0xFF) as u8,  // reg1的低字节
                    (reg2 >> 8) as u8,    // reg2的高字节
                    (reg2 & 0xFF) as u8,  // reg2的低字节
                ]
            },
            ByteOrder::CDAB => {
                // 低字在前，高字节在前：[reg2_high, reg2_low, reg1_high, reg1_low]
                // **业务含义**: 交换寄存器顺序，但保持字节内顺序
                [
                    (reg2 >> 8) as u8,    // reg2的高字节
                    (reg2 & 0xFF) as u8,  // reg2的低字节
                    (reg1 >> 8) as u8,    // reg1的高字节
                    (reg1 & 0xFF) as u8,  // reg1的低字节
                ]
            },
            ByteOrder::BADC => {
                // 高字在前，低字节在前：[reg1_low, reg1_high, reg2_low, reg2_high]
                // **业务含义**: 保持寄存器顺序，但交换字节内顺序
                [
                    (reg1 & 0xFF) as u8,  // reg1的低字节
                    (reg1 >> 8) as u8,    // reg1的高字节
                    (reg2 & 0xFF) as u8,  // reg2的低字节
                    (reg2 >> 8) as u8,    // reg2的高字节
                ]
            },
            ByteOrder::DCBA => {
                // 低字在前，低字节在前：[reg2_low, reg2_high, reg1_low, reg1_high]
                // **业务含义**: 完全反转字节顺序（小端序）
                [
                    (reg2 & 0xFF) as u8,  // reg2的低字节
                    (reg2 >> 8) as u8,    // reg2的高字节
                    (reg1 & 0xFF) as u8,  // reg1的低字节
                    (reg1 >> 8) as u8,    // reg1的高字节
                ]
            },
        };

        // **IEEE 754转换**: 将字节数组转换为浮点数
        // 使用大端序解释字节数组，因为我们已经按照目标字节序排列了字节
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
    /// 是否使用 0 基地址（true 表示用户输入已是 0 基，不再 -1 偏移）
    pub zero_based_address: bool,
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
            zero_based_address: false,
            connection_timeout_ms: 2000,
            read_timeout_ms: 1000,
            write_timeout_ms: 1000,
        }
    }
}

impl TryFrom<&PlcConnectionConfig> for ModbusConfig {
    type Error = crate::utils::error::AppError;

    fn try_from(conn: &PlcConnectionConfig) -> Result<Self, Self::Error> {
        // 尝试解析字节顺序字符串 → ByteOrder 枚举
        let byte_order = crate::models::ByteOrder::from_str(&conn.byte_order)
            .map_err(|e| crate::utils::error::AppError::configuration_error(e))?;

        // 构造 ModbusConfig
        Ok(ModbusConfig {
            ip_address: conn.ip_address.clone(),
            port: conn.port as u16,
            slave_id: 1, // 目前默认使用 1，后续可扩展至数据库配置
            byte_order,
            zero_based_address: conn.zero_based_address,
            connection_timeout_ms: conn.timeout.max(1000) as u64, // 确保超时时间合理
            read_timeout_ms: conn.timeout.max(1000) as u64,
            write_timeout_ms: conn.timeout.max(1000) as u64,
        })
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

        // 根据配置决定是否需要 -1 偏移
        let final_offset = if self.config.zero_based_address {
            offset
        } else {
            offset - 1
        };

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
        // 优先使用全局连接管理器，并等待其就绪，避免重复建立独立连接
        if let Some(manager) = self.get_plc_connection_manager().await {
            let start_time = std::time::Instant::now();
            loop {
                match self.read_bool_from_manager(&manager, address).await {
                    Ok(v) => return Ok(v),
                    Err(e) => {
                        // 只要管理器仍未建立连接，就继续等待，直到超时
                        if start_time.elapsed().as_millis() as u64 >= self.config.connection_timeout_ms {
                            return Err(e); // 超时后将错误向上抛出
                        }
                        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                    }
                }
            }
        }

        // 如果运行到此，说明当前没有注册全局连接管理器，保持原有独立连接行为
        let (addr_type, reg_offset) = self.parse_modbus_address(address)?;
        let mut client_ctx_guard = self.client_context.lock().await;

        // 🔧 修复：如果没有连接，尝试建立连接
        if client_ctx_guard.is_none() {
            drop(client_ctx_guard);
            // 只在连接失败时记录，不记录正常的连接尝试

            // 建立连接
            let socket_addr = self.get_socket_addr()?;
            let slave = self.get_slave();

            match tokio::time::timeout(
                Duration::from_millis(self.config.connection_timeout_ms),
                tokio_modbus::client::tcp::connect_slave(socket_addr, slave),
            ).await {
                Ok(Ok(ctx)) => {
                    // 连接成功，无需记录日志
                    let mut client_ctx_guard = self.client_context.lock().await;
                    *client_ctx_guard = Some(ctx);
                    let mut status_guard = self.connection_status.lock().await;
                    *status_guard = PlcConnectionStatus::Connected;
                },
                Ok(Err(e)) => {
                    log_communication_failure!("PLC独立连接失败: IP={}, 错误: {}", self.config.ip_address, e);
                    return Err(AppError::PlcCommunicationError {
                        message: format!("连接失败: {}", e)
                    });
                },
                Err(_) => {
                    log_communication_failure!("PLC连接超时: IP={}", self.config.ip_address);
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
        // 优先使用全局连接管理器
        if let Some(manager) = self.get_plc_connection_manager().await {
            let start_time = std::time::Instant::now();
            loop {
                match self.read_float32_from_manager(&manager, address).await {
                    Ok(v) => return Ok(v),
                    Err(e) => {
                        if start_time.elapsed().as_millis() as u64 >= self.config.connection_timeout_ms {
                            return Err(e);
                        }
                        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                    }
                }
            }
        }

        // 如果没有全局连接管理器，则保持原有独立连接逻辑
        let (addr_type, reg_offset) = self.parse_modbus_address(address)?;
        let mut client_ctx_guard = self.client_context.lock().await;

        // 🔧 修复：如果没有连接，尝试建立连接
        if client_ctx_guard.is_none() {
            drop(client_ctx_guard);
            // Float32读取检测到未连接，尝试建立连接

            // 建立连接
            let socket_addr = self.get_socket_addr()?;
            let slave = self.get_slave();

            match tokio::time::timeout(
                Duration::from_millis(self.config.connection_timeout_ms),
                tokio_modbus::client::tcp::connect_slave(socket_addr, slave),
            ).await {
                Ok(Ok(ctx)) => {
                    // Float32读取连接成功
                    let mut client_ctx_guard = self.client_context.lock().await;
                    *client_ctx_guard = Some(ctx);
                    let mut status_guard = self.connection_status.lock().await;
                    *status_guard = PlcConnectionStatus::Connected;
                },
                Ok(Err(e)) => {
                    log_communication_failure!("PLC Float32读取连接失败: IP={}, 错误: {}", self.config.ip_address, e);
                    return Err(AppError::PlcCommunicationError {
                        message: format!("连接失败: {}", e)
                    });
                },
                Err(_) => {
                    log_communication_failure!("PLC Float32读取连接超时: IP={}", self.config.ip_address);
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
        // 优先使用全局连接管理器
        if let Some(manager) = self.get_plc_connection_manager().await {
            let start_time = std::time::Instant::now();
            loop {
                match self.write_bool_to_manager(&manager, address, value).await {
                    Ok(()) => return Ok(()),
                    Err(e) => {
                        if start_time.elapsed().as_millis() as u64 >= self.config.connection_timeout_ms {
                            return Err(e);
                        }
                        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                    }
                }
            }
        }

        // 如果没有全局连接管理器，则继续保留独立连接逻辑
        let (addr_type, reg_offset) = self.parse_modbus_address(address)?;
        let mut client_ctx_guard = self.client_context.lock().await;

        // 🔧 修复：如果没有连接，尝试建立连接
        if client_ctx_guard.is_none() {
            drop(client_ctx_guard);
            // 写入操作检测到未连接，尝试建立连接

            // 建立连接
            let socket_addr = self.get_socket_addr()?;
            let slave = self.get_slave();

            match tokio::time::timeout(
                Duration::from_millis(self.config.connection_timeout_ms),
                tokio_modbus::client::tcp::connect_slave(socket_addr, slave),
            ).await {
                Ok(Ok(ctx)) => {
                    // 写入操作连接成功
                    let mut client_ctx_guard = self.client_context.lock().await;
                    *client_ctx_guard = Some(ctx);
                    let mut status_guard = self.connection_status.lock().await;
                    *status_guard = PlcConnectionStatus::Connected;
                },
                Ok(Err(e)) => {
                    log_communication_failure!("PLC写入操作连接失败: IP={}, 错误: {}", self.config.ip_address, e);
                    return Err(AppError::PlcCommunicationError {
                        message: format!("连接失败: {}", e)
                    });
                },
                Err(_) => {
                    log_communication_failure!("PLC写入操作连接超时: IP={}", self.config.ip_address);
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
            // 连接管理器中无匹配连接，回退到独立连接模式
        }

        // 🔧 修复：确保在独立连接模式下能够自动连接
        let (addr_type, reg_offset) = self.parse_modbus_address(address)?;
        let mut client_ctx_guard = self.client_context.lock().await;

        // 🔧 修复：如果没有连接，尝试建立连接
        if client_ctx_guard.is_none() {
            drop(client_ctx_guard);
            // Float32写入检测到未连接，尝试建立连接

            // 建立连接
            let socket_addr = self.get_socket_addr()?;
            let slave = self.get_slave();

            match tokio::time::timeout(
                Duration::from_millis(self.config.connection_timeout_ms),
                tokio_modbus::client::tcp::connect_slave(socket_addr, slave),
            ).await {
                Ok(Ok(ctx)) => {
                    // Float32写入连接成功
                    let mut client_ctx_guard = self.client_context.lock().await;
                    *client_ctx_guard = Some(ctx);
                    let mut status_guard = self.connection_status.lock().await;
                    *status_guard = PlcConnectionStatus::Connected;
                },
                Ok(Err(e)) => {
                    log_communication_failure!("PLC Float32写入连接失败: IP={}, 错误: {}", self.config.ip_address, e);
                    return Err(AppError::PlcCommunicationError {
                        message: format!("连接失败: {}", e)
                    });
                },
                Err(_) => {
                    log_communication_failure!("PLC Float32写入连接超时: IP={}", self.config.ip_address);
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

        // 字节序转换（仅在必要时记录错误）

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
    async fn get_plc_connection_manager(&self) -> Option<Arc<crate::domain::plc_connection_manager::PlcConnectionManager>> {
        get_global_plc_manager()
    }

    /// 从PLC连接管理器读取布尔值
    async fn read_bool_from_manager(
        &self,
        manager: &Arc<crate::domain::plc_connection_manager::PlcConnectionManager>,
        address: &str,
    ) -> AppResult<bool> {
        use crate::domain::plc_connection_manager::PlcConnectionState;

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
        manager: &Arc<crate::domain::plc_connection_manager::PlcConnectionManager>,
        address: &str,
        value: bool,
    ) -> AppResult<()> {
        use crate::domain::plc_connection_manager::PlcConnectionState;

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
        manager: &Arc<crate::domain::plc_connection_manager::PlcConnectionManager>,
        address: &str,
    ) -> AppResult<f32> {
        use crate::domain::plc_connection_manager::PlcConnectionState;

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
        manager: &Arc<crate::domain::plc_connection_manager::PlcConnectionManager>,
        address: &str,
        value: f32,
    ) -> AppResult<()> {
        use crate::domain::plc_connection_manager::PlcConnectionState;

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

                            // 🔍 详细调试信息：打印写入的寄存器内容 (降级为debug)
                            log::debug!("🔍 [ModbusPlcService] Float32写入调试信息:");
                            log::debug!("   原始值: {}", value);
                            log::debug!("   字节序: {:?}", self.config.byte_order);
                            log::debug!("   转换后寄存器: reg1=0x{:04X}({}), reg2=0x{:04X}({})", reg1, reg1, reg2, reg2);
                            log::debug!("   写入数组: [{}, {}] = [0x{:04X}, 0x{:04X}]", registers_to_write[0], registers_to_write[1], registers_to_write[0], registers_to_write[1]);
                            log::debug!("   目标地址: {}, 偏移: {}", address, reg_offset);

                            // 🔍 将float32转换为字节数组来查看内存布局
                            let bytes = value.to_le_bytes();
                            log::debug!("   Float32字节(小端): [{:02X}, {:02X}, {:02X}, {:02X}]", bytes[0], bytes[1], bytes[2], bytes[3]);
                            let bytes_be = value.to_be_bytes();
                            log::debug!("   Float32字节(大端): [{:02X}, {:02X}, {:02X}, {:02X}]", bytes_be[0], bytes_be[1], bytes_be[2], bytes_be[3]);

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

    */
