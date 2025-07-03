// 文件: FactoryTesting/src-tauri/src/models/test_plc_config.rs
// 详细注释：测试PLC配置相关的业务模型定义

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// 通道类型枚举 - 对应数据库中的ChannelType字段
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(into = "i32", from = "i32")]
#[repr(i32)]
pub enum TestPlcChannelType {
    AI = 0,      // 模拟量输入
    AO = 1,      // 模拟量输出
    DI = 2,      // 数字量输入
    DO = 3,      // 数字量输出
    AINone = 4,  // 模拟量输入(无源)
    AONone = 5,  // 模拟量输出(无源)
    DINone = 6,  // 数字量输入(无源)
    DONone = 7   // 数字量输出(无源)
}

/// 测试PLC通道配置 - 对应数据库ComparisonTables表
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestPlcChannelConfig {
    pub id: Option<String>,                    // 主键ID
    #[serde(rename = "channelAddress")]
    pub channel_address: String,               // 通道位号 (如: AI1_1, AO1_2)
    #[serde(rename = "channelType")]
    pub channel_type: TestPlcChannelType,      // 通道类型 (0-7)
    #[serde(rename = "communicationAddress")]
    pub communication_address: String,         // 通讯地址 (如: 40101, 00101)
    #[serde(rename = "powerSupplyType")]
    pub power_supply_type: String,             // 供电类型 (必填项)
    pub description: Option<String>,           // 描述信息
    #[serde(rename = "isEnabled")]
    pub is_enabled: bool,                      // 是否启用
    #[serde(rename = "createdAt")]
    pub created_at: Option<DateTime<Utc>>,     // 创建时间
    #[serde(rename = "updatedAt")]
    pub updated_at: Option<DateTime<Utc>>,     // 更新时间
}

/// PLC类型枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlcType {
    ModbusTcp,
    SiemensS7,
    OpcUa,

}

/// 连接状态枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConnectionStatus {
    Disconnected,        // 未连接
    Connecting,          // 连接中
    Connected,           // 已连接
    Error,               // 连接错误
    Timeout,             // 连接超时
}

/// PLC连接配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlcConnectionConfig {
    pub id: String,                            // 配置ID
    pub name: String,                          // 配置名称
    #[serde(rename = "plcType")]
    pub plc_type: PlcType,                     // PLC类型
    #[serde(rename = "ipAddress")]
    pub ip_address: String,                    // IP地址
    pub port: i32,                             // 端口号
    pub timeout: i32,                          // 超时时间(ms)
    #[serde(rename = "retryCount")]
    pub retry_count: i32,                      // 重试次数
    #[serde(rename = "byteOrder", default = "default_byte_order")]  // 字节顺序
    pub byte_order: String,                    // 保存为字符串，方便前后端兼容
    #[serde(rename = "zeroBasedAddress", default)]
    pub zero_based_address: bool,              // 地址是否从0开始
    #[serde(rename = "isTestPlc")]
    pub is_test_plc: bool,                     // 是否为测试PLC
    pub description: Option<String>,           // 描述
    #[serde(rename = "isEnabled")]
    pub is_enabled: bool,                      // 是否启用
    #[serde(rename = "lastConnected")]
    pub last_connected: Option<DateTime<Utc>>, // 最后连接时间
    #[serde(rename = "connectionStatus")]
    pub connection_status: ConnectionStatus,   // 连接状态
}

/// 映射类型枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MappingType {
    Direct,                     // 直接映射
    Inverse,                    // 反向映射
    Scaled,                     // 比例映射
    Custom,                     // 自定义映射
}

/// 通道映射配置 - 用于将被测PLC通道映射到测试PLC通道
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelMappingConfig {
    pub id: String,                            // 映射ID
    #[serde(rename = "targetChannelId")]
    pub target_channel_id: String,             // 被测通道ID (ChannelPointDefinition.id)
    #[serde(rename = "testPlcChannelId")]
    pub test_plc_channel_id: String,           // 测试PLC通道ID (TestPlcChannelConfig.id)
    #[serde(rename = "mappingType")]
    pub mapping_type: MappingType,             // 映射类型
    #[serde(rename = "isActive")]
    pub is_active: bool,                       // 是否激活
    pub notes: Option<String>,                 // 备注
    #[serde(rename = "createdAt")]
    pub created_at: DateTime<Utc>,             // 创建时间
}

/// 测试PLC配置服务请求/响应结构体

/// 获取测试PLC通道配置请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetTestPlcChannelsRequest {
    pub channel_type_filter: Option<TestPlcChannelType>,
    pub enabled_only: Option<bool>,
}

/// 保存测试PLC通道配置请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveTestPlcChannelRequest {
    pub channel: TestPlcChannelConfig,
}

/// 测试PLC连接请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestPlcConnectionRequest {
    pub connection_id: String,
}

/// 测试PLC连接响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestPlcConnectionResponse {
    pub success: bool,
    pub message: String,
    pub connection_time_ms: Option<u64>,
}

/// 地址读取测试响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddressReadTestResponse {
    pub success: bool,
    pub value: Option<serde_json::Value>,
    pub error: Option<String>,
    pub read_time_ms: Option<u64>,
}

/// 自动生成通道映射请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateChannelMappingsRequest {
    pub target_channel_ids: Vec<String>,
    pub strategy: MappingStrategy,
}

/// 映射策略枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MappingStrategy {
    ByChannelType,      // 按通道类型匹配
    Sequential,         // 顺序分配
    LoadBalanced,       // 负载均衡
}

/// 自动生成通道映射响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateChannelMappingsResponse {
    pub success: bool,
    pub message: String,
    pub mappings: Vec<ChannelMappingConfig>,
    pub conflicts: Vec<String>,
}

/// 实现默认值
impl Default for TestPlcChannelConfig {
    fn default() -> Self {
        Self {
            id: None,
            channel_address: String::new(),
            channel_type: TestPlcChannelType::AI,
            communication_address: String::new(),
            power_supply_type: String::new(),
            description: None,
            is_enabled: true,
            created_at: None,
            updated_at: None,
        }
    }
}

impl Default for PlcConnectionConfig {
    fn default() -> Self {
        Self {
            id: String::new(),
            name: String::new(),
            plc_type: PlcType::ModbusTcp,
            ip_address: "192.168.1.100".to_string(),
            port: 502,
            timeout: 5000,
            retry_count: 3,
            byte_order: "CDAB".to_string(),
            zero_based_address: false,
            is_test_plc: true,
            description: None,
            is_enabled: true,
            last_connected: None,
            connection_status: ConnectionStatus::Disconnected,
        }
    }
}

/// 辅助函数：获取通道类型的字符串表示
impl TestPlcChannelType {
    pub fn as_str(&self) -> &'static str {
        match self {
            TestPlcChannelType::AI => "AI",
            TestPlcChannelType::AO => "AO",
            TestPlcChannelType::DI => "DI",
            TestPlcChannelType::DO => "DO",
            TestPlcChannelType::AINone => "AINone",
            TestPlcChannelType::AONone => "AONone",
            TestPlcChannelType::DINone => "DINone",
            TestPlcChannelType::DONone => "DONone",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "AI" => Some(TestPlcChannelType::AI),
            "AO" => Some(TestPlcChannelType::AO),
            "DI" => Some(TestPlcChannelType::DI),
            "DO" => Some(TestPlcChannelType::DO),
            "AINone" => Some(TestPlcChannelType::AINone),
            "AONone" => Some(TestPlcChannelType::AONone),
            "DINone" => Some(TestPlcChannelType::DINone),
            "DONone" => Some(TestPlcChannelType::DONone),
            _ => None,
        }
    }
}

// 实现 From<TestPlcChannelType> for i32
impl From<TestPlcChannelType> for i32 {
    fn from(channel_type: TestPlcChannelType) -> Self {
        channel_type as i32
    }
}

// 实现 From<i32> for TestPlcChannelType
impl From<i32> for TestPlcChannelType {
    fn from(value: i32) -> Self {
        match value {
            0 => TestPlcChannelType::AI,
            1 => TestPlcChannelType::AO,
            2 => TestPlcChannelType::DI,
            3 => TestPlcChannelType::DO,
            4 => TestPlcChannelType::AINone,
            5 => TestPlcChannelType::AONone,
            6 => TestPlcChannelType::DINone,
            7 => TestPlcChannelType::DONone,
            _ => TestPlcChannelType::AI, // 默认值
        }
    }
}

fn default_byte_order() -> String { "CDAB".to_string() } 
