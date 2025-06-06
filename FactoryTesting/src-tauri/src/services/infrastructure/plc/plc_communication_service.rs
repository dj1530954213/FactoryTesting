/// PLC通信服务接口定义及相关数据结构

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::utils::error::AppResult;
use crate::services::traits::BaseService;

/// PLC标签信息结构
/// 用于描述PLC中的一个数据点的完整信息
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlcTag {
    /// 标签地址（如：DB1.DBD0, 40001, %MW100等）
    pub address: String,
    /// 数据类型
    pub data_type: PlcDataType,
    /// 标签描述
    pub description: Option<String>,
    /// 是否可读
    pub readable: bool,
    /// 是否可写
    pub writable: bool,
    /// 标签单位（如：mA, bar, °C等）
    pub unit: Option<String>,
    /// 量程最小值
    pub min_value: Option<f64>,
    /// 量程最大值
    pub max_value: Option<f64>,
}

/// PLC数据类型枚举
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PlcDataType {
    /// 布尔类型
    Bool,
    /// 8位有符号整数
    Int8,
    /// 8位无符号整数
    UInt8,
    /// 16位有符号整数
    Int16,
    /// 16位无符号整数
    UInt16,
    /// 32位有符号整数
    Int32,
    /// 32位无符号整数
    UInt32,
    /// 64位有符号整数
    Int64,
    /// 64位无符号整数
    UInt64,
    /// 32位浮点数
    Float32,
    /// 64位浮点数
    Float64,
    /// 字符串类型
    String,
    /// 字节数组
    ByteArray,
}

impl Default for PlcDataType {
    fn default() -> Self {
        Self::Float32
    }
}

/// PLC连接状态
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PlcConnectionStatus {
    /// 已断开
    Disconnected,
    /// 连接中
    Connecting,
    /// 已连接
    Connected,
    /// 连接错误
    Error(String),
}

/// PLC通信统计信息
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PlcCommunicationStats {
    /// 连接建立次数
    pub connection_count: u64,
    /// 成功读取次数
    pub successful_reads: u64,
    /// 失败读取次数
    pub failed_reads: u64,
    /// 成功写入次数
    pub successful_writes: u64,
    /// 失败写入次数
    pub failed_writes: u64,
    /// 成功批量读取次数
    pub successful_batch_reads: u64,
    /// 失败批量读取次数
    pub failed_batch_reads: u64,
    /// 成功批量写入次数
    pub successful_batch_writes: u64,
    /// 失败批量写入次数
    pub failed_batch_writes: u64,
    /// 总的读取耗时（毫秒）
    pub total_read_time_ms: u64,
    /// 总的写入耗时（毫秒）
    pub total_write_time_ms: u64,
    /// 最后一次通信时间
    pub last_communication_time: Option<chrono::DateTime<chrono::Utc>>,
}

/// PLC通信服务接口
/// 这是所有PLC通信实现必须遵循的接口规范
#[async_trait]
pub trait PlcCommunicationService: BaseService {
    /// 连接到PLC
    /// 建立与PLC的通信连接
    async fn connect(&mut self) -> AppResult<()>;
    
    /// 断开PLC连接
    /// 安全关闭与PLC的连接
    async fn disconnect(&mut self) -> AppResult<()>;
    
    /// 检查连接状态
    /// 返回当前的连接状态
    fn get_connection_status(&self) -> PlcConnectionStatus;
    
    /// 检查是否已连接
    /// 简化的连接状态检查
    fn is_connected(&self) -> bool {
        matches!(self.get_connection_status(), PlcConnectionStatus::Connected)
    }
    
    /// 测试连接（ping测试）
    /// 发送一个简单的读取命令来验证连接是否正常
    async fn test_connection(&self) -> AppResult<bool>;
    
    // 基础数据类型读取方法
    
    /// 读取布尔值
    async fn read_bool(&self, address: &str) -> AppResult<bool> {
        log::info!("🔍 [PLC_READ_BOOL] 地址: {}", address);
        let result = self.read_bool_impl(address).await;
        match &result {
            Ok(value) => log::info!("✅ [PLC_READ_BOOL] 地址: {}, 值: {}", address, value),
            Err(e) => log::error!("❌ [PLC_READ_BOOL] 地址: {}, 错误: {}", address, e),
        }
        result
    }

    /// 读取布尔值的实际实现
    async fn read_bool_impl(&self, address: &str) -> AppResult<bool>;
    
    /// 读取8位有符号整数
    async fn read_int8(&self, address: &str) -> AppResult<i8>;
    
    /// 读取8位无符号整数
    async fn read_uint8(&self, address: &str) -> AppResult<u8>;
    
    /// 读取16位有符号整数
    async fn read_int16(&self, address: &str) -> AppResult<i16>;
    
    /// 读取16位无符号整数
    async fn read_uint16(&self, address: &str) -> AppResult<u16>;
    
    /// 读取32位有符号整数
    async fn read_int32(&self, address: &str) -> AppResult<i32>;
    
    /// 读取32位无符号整数
    async fn read_uint32(&self, address: &str) -> AppResult<u32>;
    
    /// 读取64位有符号整数
    async fn read_int64(&self, address: &str) -> AppResult<i64>;
    
    /// 读取64位无符号整数
    async fn read_uint64(&self, address: &str) -> AppResult<u64>;
    
    /// 读取32位浮点数
    async fn read_float32(&self, address: &str) -> AppResult<f32> {
        log::info!("🔍 [PLC_READ_F32] 地址: {}", address);
        let result = self.read_float32_impl(address).await;
        match &result {
            Ok(value) => log::info!("✅ [PLC_READ_F32] 地址: {}, 值: {}", address, value),
            Err(e) => log::error!("❌ [PLC_READ_F32] 地址: {}, 错误: {}", address, e),
        }
        result
    }

    /// 读取32位浮点数的实际实现
    async fn read_float32_impl(&self, address: &str) -> AppResult<f32>;
    
    /// 读取64位浮点数
    async fn read_float64(&self, address: &str) -> AppResult<f64>;
    
    /// 读取字符串
    async fn read_string(&self, address: &str, max_length: usize) -> AppResult<String>;
    
    /// 读取字节数组
    async fn read_bytes(&self, address: &str, length: usize) -> AppResult<Vec<u8>>;
    
    // 基础数据类型写入方法
    
    /// 写入布尔值
    async fn write_bool(&self, address: &str, value: bool) -> AppResult<()> {
        log::info!("📝 [PLC_WRITE_BOOL] 地址: {}, 值: {}", address, value);
        let result = self.write_bool_impl(address, value).await;
        match &result {
            Ok(_) => log::info!("✅ [PLC_WRITE_BOOL] 地址: {}, 值: {} - 写入成功", address, value),
            Err(e) => log::error!("❌ [PLC_WRITE_BOOL] 地址: {}, 值: {}, 错误: {}", address, value, e),
        }
        result
    }

    /// 写入布尔值的实际实现
    async fn write_bool_impl(&self, address: &str, value: bool) -> AppResult<()>;
    
    /// 写入8位有符号整数
    async fn write_int8(&self, address: &str, value: i8) -> AppResult<()>;
    
    /// 写入8位无符号整数
    async fn write_uint8(&self, address: &str, value: u8) -> AppResult<()>;
    
    /// 写入16位有符号整数
    async fn write_int16(&self, address: &str, value: i16) -> AppResult<()>;
    
    /// 写入16位无符号整数
    async fn write_uint16(&self, address: &str, value: u16) -> AppResult<()>;
    
    /// 写入32位有符号整数
    async fn write_int32(&self, address: &str, value: i32) -> AppResult<()>;
    
    /// 写入32位无符号整数
    async fn write_uint32(&self, address: &str, value: u32) -> AppResult<()>;
    
    /// 写入64位有符号整数
    async fn write_int64(&self, address: &str, value: i64) -> AppResult<()>;
    
    /// 写入64位无符号整数
    async fn write_uint64(&self, address: &str, value: u64) -> AppResult<()>;
    
    /// 写入32位浮点数
    async fn write_float32(&self, address: &str, value: f32) -> AppResult<()> {
        log::info!("📝 [PLC_WRITE_F32] 地址: {}, 值: {}", address, value);
        let result = self.write_float32_impl(address, value).await;
        match &result {
            Ok(_) => log::info!("✅ [PLC_WRITE_F32] 地址: {}, 值: {} - 写入成功", address, value),
            Err(e) => log::error!("❌ [PLC_WRITE_F32] 地址: {}, 值: {}, 错误: {}", address, value, e),
        }
        result
    }

    /// 写入32位浮点数的实际实现
    async fn write_float32_impl(&self, address: &str, value: f32) -> AppResult<()>;
    
    /// 写入64位浮点数
    async fn write_float64(&self, address: &str, value: f64) -> AppResult<()>;
    
    /// 写入字符串
    async fn write_string(&self, address: &str, value: &str) -> AppResult<()>;
    
    /// 写入字节数组
    async fn write_bytes(&self, address: &str, value: &[u8]) -> AppResult<()>;
    
    // 高级操作方法
    
    /// 批量读取
    /// 一次性读取多个地址的值，提高通信效率
    async fn batch_read(&self, addresses: &[String]) -> AppResult<HashMap<String, serde_json::Value>>;
    
    /// 批量写入
    /// 一次性写入多个地址的值，提高通信效率
    async fn batch_write(&self, values: &HashMap<String, serde_json::Value>) -> AppResult<()>;
    
    /// 读取标签信息
    /// 获取指定地址的标签元数据
    async fn read_tag_info(&self, address: &str) -> AppResult<PlcTag>;
    
    /// 列出所有可用标签
    /// 获取PLC中所有可访问的标签列表（如果支持）
    async fn list_available_tags(&self) -> AppResult<Vec<PlcTag>>;
    
    /// 获取通信统计信息
    /// 返回PLC通信的统计数据
    fn get_communication_stats(&self) -> PlcCommunicationStats;
    
    /// 重置通信统计信息
    /// 清零所有统计计数器
    fn reset_communication_stats(&mut self);
    
    /// 设置读取超时时间（毫秒）
    fn set_read_timeout(&mut self, timeout_ms: u32) -> AppResult<()>;
    
    /// 设置写入超时时间（毫秒）
    fn set_write_timeout(&mut self, timeout_ms: u32) -> AppResult<()>;
    
    /// 获取PLC设备信息
    /// 返回PLC的基本信息（型号、版本等）
    async fn get_device_info(&self) -> AppResult<HashMap<String, String>>;
    
    // 便捷方法 - 在trait中直接提供默认实现
    
    /// 读取整数（自动选择合适的类型）
    /// 根据值的范围自动选择最合适的整数读取方法
    async fn read_int(&self, address: &str) -> AppResult<i64> {
        // 默认尝试读取32位整数，如果失败则尝试其他类型
        match self.read_int32(address).await {
            Ok(value) => Ok(value as i64),
            Err(_) => self.read_int64(address).await,
        }
    }
    
    /// 写入整数（自动选择合适的类型）
    /// 根据值的范围自动选择最合适的整数写入方法
    async fn write_int(&self, address: &str, value: i64) -> AppResult<()> {
        if value >= i32::MIN as i64 && value <= i32::MAX as i64 {
            self.write_int32(address, value as i32).await
        } else {
            self.write_int64(address, value).await
        }
    }
    
    /// 读取浮点数（自动选择合适的精度）
    /// 默认使用32位浮点数
    async fn read_float(&self, address: &str) -> AppResult<f64> {
        match self.read_float32(address).await {
            Ok(value) => Ok(value as f64),
            Err(_) => self.read_float64(address).await,
        }
    }
    
    /// 写入浮点数（自动选择合适的精度）
    /// 默认使用32位浮点数
    async fn write_float(&self, address: &str, value: f64) -> AppResult<()> {
        if value >= f32::MIN as f64 && value <= f32::MAX as f64 {
            self.write_float32(address, value as f32).await
        } else {
            self.write_float64(address, value).await
        }
    }
} 