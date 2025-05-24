// modbus_plc_service.rs
use async_trait::async_trait;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::Duration;
use tokio_modbus::client::Context as ModbusClientContext;
use tokio_modbus::prelude::*; // for tcp::connect_slave and Slave

use crate::utils::error::{AppError, AppResult};
use crate::services::traits::BaseService;
use super::plc_communication_service::{
    PlcCommunicationService, PlcConnectionStatus, PlcDataType, PlcTag, PlcCommunicationStats
};

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

// 临时的 ByteOrderConverter 桩，实际应使用项目中的实现
struct ByteOrderConverter;
impl ByteOrderConverter {
    #[allow(dead_code)]
    fn registers_to_float(_reg1: u16, _reg2: u16, _order: ByteOrder) -> f32 {
        // TODO: 实现实际的字节序转换逻辑
        0.0
    }
    #[allow(dead_code)]
    fn float_to_registers(_value: f32, _order: ByteOrder) -> (u16, u16) {
        // TODO: 实现实际的字节序转换逻辑
        (0, 0)
    }
    // 其他转换方法...
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
    async fn read_bool(&self, address: &str) -> AppResult<bool> {
        let (addr_type, reg_offset) = self.parse_modbus_address(address)?;
        let mut client_ctx_guard = self.client_context.lock().await;
        let ctx = client_ctx_guard.as_mut().ok_or_else(|| AppError::PlcCommunicationError { message: "未连接".to_string() })?;
        
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
    
    async fn read_float32(&self, address: &str) -> AppResult<f32> {
        let (addr_type, reg_offset) = self.parse_modbus_address(address)?;
        let mut client_ctx_guard = self.client_context.lock().await;
        let ctx = client_ctx_guard.as_mut().ok_or_else(|| AppError::PlcCommunicationError { message: "未连接".to_string() })?;

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
    async fn write_bool(&self, address: &str, value: bool) -> AppResult<()> {
        let (addr_type, reg_offset) = self.parse_modbus_address(address)?;
        let mut client_ctx_guard = self.client_context.lock().await;
        let ctx = client_ctx_guard.as_mut().ok_or_else(|| AppError::PlcCommunicationError { message: "未连接".to_string() })?;

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

    async fn write_float32(&self, address: &str, value: f32) -> AppResult<()> {
        let (addr_type, reg_offset) = self.parse_modbus_address(address)?;
        let mut client_ctx_guard = self.client_context.lock().await;
        let ctx = client_ctx_guard.as_mut().ok_or_else(|| AppError::PlcCommunicationError { message: "未连接".to_string() })?;

        if addr_type != '4' {
            return Err(AppError::PlcCommunicationError { message: format!("地址 {} 不是有效的可写保持寄存器地址 (用于f32)", address) });
        }
        
        let (reg1, reg2) = ByteOrderConverter::float_to_registers(value, self.config.byte_order);
        let registers_to_write = [reg1, reg2];

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