use super::*;
use std::collections::HashMap;

/// PLC通信服务接口
/// 
/// 负责与PLC设备的通信，支持Modbus TCP、Siemens S7、OPC UA等协议
/// 符合 FAT-CTK-001 规则：通信任务规则
#[async_trait]
pub trait IPlcCommunicationService: BaseService {
    /// 连接到PLC
    /// 
    /// # 参数
    /// * `config` - 连接配置
    /// 
    /// # 返回
    /// * `ConnectionHandle` - 连接句柄
    async fn connect(&self, config: &PlcConnectionConfig) -> AppResult<ConnectionHandle>;
    
    /// 断开PLC连接
    /// 
    /// # 参数
    /// * `handle` - 连接句柄
    async fn disconnect(&self, handle: &ConnectionHandle) -> AppResult<()>;
    
    /// 检查连接状态
    /// 
    /// # 参数
    /// * `handle` - 连接句柄
    /// 
    /// # 返回
    /// * `bool` - 是否已连接
    async fn is_connected(&self, handle: &ConnectionHandle) -> AppResult<bool>;
    
    /// 读取布尔值
    /// 
    /// # 参数
    /// * `handle` - 连接句柄
    /// * `address` - PLC地址
    /// 
    /// # 返回
    /// * `bool` - 读取的布尔值
    async fn read_bool(&self, handle: &ConnectionHandle, address: &str) -> AppResult<bool>;
    
    /// 写入布尔值
    /// 
    /// # 参数
    /// * `handle` - 连接句柄
    /// * `address` - PLC地址
    /// * `value` - 要写入的值
    async fn write_bool(&self, handle: &ConnectionHandle, address: &str, value: bool) -> AppResult<()>;
    
    /// 读取32位浮点数
    /// 
    /// # 参数
    /// * `handle` - 连接句柄
    /// * `address` - PLC地址
    /// 
    /// # 返回
    /// * `f32` - 读取的浮点数
    async fn read_f32(&self, handle: &ConnectionHandle, address: &str) -> AppResult<f32>;
    
    /// 写入32位浮点数
    /// 
    /// # 参数
    /// * `handle` - 连接句柄
    /// * `address` - PLC地址
    /// * `value` - 要写入的值
    async fn write_f32(&self, handle: &ConnectionHandle, address: &str, value: f32) -> AppResult<()>;
    
    /// 读取32位整数
    /// 
    /// # 参数
    /// * `handle` - 连接句柄
    /// * `address` - PLC地址
    /// 
    /// # 返回
    /// * `i32` - 读取的整数
    async fn read_i32(&self, handle: &ConnectionHandle, address: &str) -> AppResult<i32>;
    
    /// 写入32位整数
    /// 
    /// # 参数
    /// * `handle` - 连接句柄
    /// * `address` - PLC地址
    /// * `value` - 要写入的值
    async fn write_i32(&self, handle: &ConnectionHandle, address: &str, value: i32) -> AppResult<()>;
    
    /// 批量读取操作
    /// 
    /// # 参数
    /// * `handle` - 连接句柄
    /// * `requests` - 读取请求列表
    /// 
    /// # 返回
    /// * `Vec<ReadResult>` - 读取结果列表
    async fn batch_read(&self, handle: &ConnectionHandle, requests: &[ReadRequest]) -> AppResult<Vec<ReadResult>>;
    
    /// 批量写入操作
    /// 
    /// # 参数
    /// * `handle` - 连接句柄
    /// * `requests` - 写入请求列表
    /// 
    /// # 返回
    /// * `Vec<WriteResult>` - 写入结果列表
    async fn batch_write(&self, handle: &ConnectionHandle, requests: &[WriteRequest]) -> AppResult<Vec<WriteResult>>;
    
    /// 获取连接统计信息
    /// 
    /// # 参数
    /// * `handle` - 连接句柄
    /// 
    /// # 返回
    /// * `ConnectionStats` - 连接统计信息
    async fn get_connection_stats(&self, handle: &ConnectionHandle) -> AppResult<ConnectionStats>;
    
    /// 测试连接
    /// 
    /// # 参数
    /// * `config` - 连接配置
    /// 
    /// # 返回
    /// * `ConnectionTestResult` - 连接测试结果
    async fn test_connection(&self, config: &PlcConnectionConfig) -> AppResult<ConnectionTestResult>;

    /// 获取指定连接ID的默认连接句柄（若尚未连接则返回 None）
    async fn default_handle_by_id(&self, connection_id: &str) -> Option<ConnectionHandle>;

    /// 获取最后一次连接的默认连接句柄（向后兼容）
    async fn default_handle(&self) -> Option<ConnectionHandle>;
}

/// PLC连接配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlcConnectionConfig {
    /// 连接ID
    pub id: String,
    
    /// 连接名称
    pub name: String,
    
    /// 协议类型
    pub protocol: PlcProtocol,
    
    /// 主机地址
    pub host: String,
    
    /// 端口号
    pub port: u16,
    
    /// 连接超时（毫秒）
    pub timeout_ms: u64,
    
    /// 读取超时（毫秒）
    pub read_timeout_ms: u64,
    
    /// 写入超时（毫秒）
    pub write_timeout_ms: u64,

    /// 字节顺序，如 "ABCD" "CDAB" "BADC" "DCBA"
    pub byte_order: String,

    /// 地址是否从0开始（Modbus中有的PLC地址0基）
    pub zero_based_address: bool,
    
    /// 重试次数
    pub retry_count: u32,
    
    /// 重试间隔（毫秒）
    pub retry_interval_ms: u64,
    
    /// 协议特定参数
    pub protocol_params: HashMap<String, serde_json::Value>,
}

/// PLC协议类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlcProtocol {
    ModbusTcp,
    SiemensS7,
    OpcUa,
    EthernetIp,
}

/// 连接句柄
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionHandle {
    /// 连接ID
    pub connection_id: String,
    
    /// 句柄ID
    pub handle_id: String,
    
    /// 协议类型
    pub protocol: PlcProtocol,
    
    /// 创建时间
    pub created_at: DateTime<Utc>,
    
    /// 最后活动时间
    pub last_activity: DateTime<Utc>,
}

/// 读取请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadRequest {
    /// 请求ID
    pub id: String,
    
    /// PLC地址
    pub address: String,
    
    /// 数据类型
    pub data_type: PlcDataType,
    
    /// 数组长度（对于数组类型）
    pub array_length: Option<u32>,
}

/// 写入请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WriteRequest {
    /// 请求ID
    pub id: String,
    
    /// PLC地址
    pub address: String,
    
    /// 要写入的值
    pub value: PlcValue,
}

/// 读取结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadResult {
    /// 请求ID
    pub request_id: String,
    
    /// 是否成功
    pub success: bool,
    
    /// 读取的值
    pub value: Option<PlcValue>,
    
    /// 错误信息
    pub error_message: Option<String>,
    
    /// 执行时间（毫秒）
    pub execution_time_ms: u64,
}

/// 写入结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WriteResult {
    /// 请求ID
    pub request_id: String,
    
    /// 是否成功
    pub success: bool,
    
    /// 错误信息
    pub error_message: Option<String>,
    
    /// 执行时间（毫秒）
    pub execution_time_ms: u64,
}

/// PLC数据类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlcDataType {
    Bool,
    Int16,
    Int32,
    Float32,
    String,
    ByteArray,
}

/// PLC值
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PlcValue {
    Bool(bool),
    Int16(i16),
    Int32(i32),
    Float32(f32),
    String(String),
    ByteArray(Vec<u8>),
    Array(Vec<PlcValue>),
}

/// 连接统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionStats {
    /// 连接ID
    pub connection_id: String,
    
    /// 总读取次数
    pub total_reads: u64,
    
    /// 总写入次数
    pub total_writes: u64,
    
    /// 成功读取次数
    pub successful_reads: u64,
    
    /// 成功写入次数
    pub successful_writes: u64,
    
    /// 平均读取时间（毫秒）
    pub average_read_time_ms: f64,
    
    /// 平均写入时间（毫秒）
    pub average_write_time_ms: f64,
    
    /// 连接建立时间
    pub connection_established_at: DateTime<Utc>,
    
    /// 最后通信时间
    pub last_communication: DateTime<Utc>,
    
    /// 连接错误次数
    pub connection_errors: u64,
}

/// 连接测试结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionTestResult {
    /// 是否成功
    pub success: bool,
    
    /// 连接时间（毫秒）
    pub connection_time_ms: u64,
    
    /// 错误信息
    pub error_message: Option<String>,
    
    /// 协议版本信息
    pub protocol_info: Option<String>,
    
    /// 设备信息
    pub device_info: Option<String>,
}
