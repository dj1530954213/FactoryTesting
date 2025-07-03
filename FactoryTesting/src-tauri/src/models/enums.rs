use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::str::FromStr;

// Re-export PlcProtocol for compatibility with older import paths
pub use crate::domain::services::plc_communication_service::PlcProtocol;

/// 整体测试状态枚举
/// 表示一个通道测试实例的总体状态
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum OverallTestStatus {
    /// 未测试
    NotTested,
    /// 跳过测试
    Skipped,
    /// 接线确认需要
    WiringConfirmationRequired,
    /// 接线已确认，等待开始硬点或手动测试
    WiringConfirmed,
    /// 硬点测试进行中
    HardPointTestInProgress,
    //TODO:是否与下面的HardPointTesting重复？
    /// 硬点测试进行中
    HardPointTesting,
    /// 硬点测试已完成
    HardPointTestCompleted,
    /// 手动测试进行中
    ManualTestInProgress,
    //TODO:是否与下面的HardPointTesting重复？
    /// 手动测试进行中
    ManualTesting,
    /// 测试完成且通过
    TestCompletedPassed,
    /// 测试完成但失败
    TestCompletedFailed,
    /// 重新测试中
    Retesting,
}

impl Default for OverallTestStatus {
    fn default() -> Self {
        Self::NotTested
    }
}

impl Display for OverallTestStatus {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            OverallTestStatus::NotTested => "NotTested",
            OverallTestStatus::Skipped => "Skipped",
            OverallTestStatus::WiringConfirmationRequired => "WiringConfirmationRequired",
            OverallTestStatus::WiringConfirmed => "WiringConfirmed",
            OverallTestStatus::HardPointTestInProgress => "HardPointTestInProgress",
            OverallTestStatus::HardPointTesting => "HardPointTesting",
            OverallTestStatus::HardPointTestCompleted => "HardPointTestCompleted",
            OverallTestStatus::ManualTestInProgress => "ManualTestInProgress",
            OverallTestStatus::ManualTesting => "ManualTesting",
            OverallTestStatus::TestCompletedPassed => "TestCompletedPassed",
            OverallTestStatus::TestCompletedFailed => "TestCompletedFailed",
            OverallTestStatus::Retesting => "Retesting",
        };
        write!(f, "{}", s)
    }
}

impl FromStr for OverallTestStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "NotTested" => Ok(OverallTestStatus::NotTested),
            "Skipped" => Ok(OverallTestStatus::Skipped),
            "WiringConfirmationRequired" => Ok(OverallTestStatus::WiringConfirmationRequired),
            "WiringConfirmed" => Ok(OverallTestStatus::WiringConfirmed),
            "HardPointTestInProgress" => Ok(OverallTestStatus::HardPointTestInProgress),
            "HardPointTesting" => Ok(OverallTestStatus::HardPointTesting),
            "HardPointTestCompleted" => Ok(OverallTestStatus::HardPointTestCompleted),
            "ManualTestInProgress" => Ok(OverallTestStatus::ManualTestInProgress),
            "ManualTesting" => Ok(OverallTestStatus::ManualTesting),
            "TestCompletedPassed" => Ok(OverallTestStatus::TestCompletedPassed),
            "TestCompletedFailed" => Ok(OverallTestStatus::TestCompletedFailed),
            "Retesting" => Ok(OverallTestStatus::Retesting),
            _ => Err(format!("Invalid OverallTestStatus: {}", s)),
        }
    }
}

/// 子测试状态枚举
/// 表示单个子测试项的状态
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SubTestStatus {
    /// 未测试
    NotTested,
    /// 测试中
    Testing,
    /// 测试通过
    Passed,
    /// 测试失败
    Failed,
    /// 不适用（该测试项对当前模块类型不适用）
    NotApplicable,
    /// 跳过测试
    Skipped,
}

impl Default for SubTestStatus {
    fn default() -> Self {
        Self::NotTested
    }
}

/// 模块类型枚举
/// 表示不同类型的PLC模块
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ModuleType {
    /// 模拟量输入
    AI,
    /// 模拟量输出
    AO,
    /// 数字量输入
    DI,
    /// 数字量输出
    DO,
    /// 模拟量输入（无源，特殊处理逻辑）
    AINone,
    /// 模拟量输出（无源）
    AONone,
    /// 数字量输入（无源）
    DINone,
    /// 数字量输出（无源）
    DONone,
    /// 通信模块
    Communication,
    /// 其他特殊模块类型
    Other(String),
}

impl Default for ModuleType {
    fn default() -> Self {
        Self::AI
    }
}

impl Display for ModuleType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            ModuleType::AI => "AI",
            ModuleType::AO => "AO",
            ModuleType::DI => "DI",
            ModuleType::DO => "DO",
            ModuleType::AINone => "AINone",
            ModuleType::AONone => "AONone",
            ModuleType::DINone => "DINone",
            ModuleType::DONone => "DONone",
            ModuleType::Communication => "Communication",
            ModuleType::Other(s) => s,
        };
        write!(f, "{}", s)
    }
}

impl FromStr for ModuleType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "AI" => Ok(ModuleType::AI),
            "AO" => Ok(ModuleType::AO),
            "DI" => Ok(ModuleType::DI),
            "DO" => Ok(ModuleType::DO),
            "AINone" => Ok(ModuleType::AINone),
            "AONone" => Ok(ModuleType::AONone),
            "DINone" => Ok(ModuleType::DINone),
            "DONone" => Ok(ModuleType::DONone),
            "Communication" => Ok(ModuleType::Communication),
            _ => Ok(ModuleType::Other(s.to_string())),
        }
    }
}

/// 点位数据类型枚举
/// 表示PLC点位的数据类型
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PointDataType {
    /// 布尔类型
    Bool,
    /// 浮点数类型
    Float,
    /// 整数类型
    Int,
    /// 字符串类型
    String,
    /// 双精度浮点数类型
    Double,
    /// 16位整数
    Int16,
    /// 32位整数
    Int32,
    /// 无符号16位整数
    UInt16,
    /// 无符号32位整数
    UInt32,
}

impl Default for PointDataType {
    fn default() -> Self {
        Self::Float
    }
}

impl Display for PointDataType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            PointDataType::Bool => "Bool",
            PointDataType::Float => "Float",
            PointDataType::Int => "Int",
            PointDataType::String => "String",
            PointDataType::Double => "Double",
            PointDataType::Int16 => "Int16",
            PointDataType::Int32 => "Int32",
            PointDataType::UInt16 => "UInt16",
            PointDataType::UInt32 => "UInt32",
        };
        write!(f, "{}", s)
    }
}

impl FromStr for PointDataType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Bool" => Ok(PointDataType::Bool),
            "Float" | "Real" => Ok(PointDataType::Float),
            "Int" => Ok(PointDataType::Int),
            "String" => Ok(PointDataType::String),
            "Double" => Ok(PointDataType::Double),
            "Int16" => Ok(PointDataType::Int16),
            "Int32" => Ok(PointDataType::Int32),
            "UInt16" => Ok(PointDataType::UInt16),
            "UInt32" => Ok(PointDataType::UInt32),
            _ => Err(format!("Invalid PointDataType: {}", s)),
        }
    }
}

/// 子测试项枚举
/// 对应原ChannelMapping.cs中的各种子测试项
/// 使用Eq和Hash特征以便作为HashMap的键使用
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SubTestItem {
    // 通用测试项
    /// 硬点回路测试（核心测试项）
    HardPoint,
    /// 趋势检查（AI/AO模块）
    TrendCheck,
    /// 趋势检查（简化名称）
    Trend,
    /// 报表检查（AI/AO模块）
    ReportCheck,
    /// 报表检查（简化名称）
    Report,
    /// 维护功能测试（AI/AO模块）
    Maintenance,

    // AI模块特有测试项
    /// 低低报警测试
    LowLowAlarm,
    /// 低报警测试
    LowAlarm,
    /// 高报警测试
    HighAlarm,
    /// 高高报警测试
    HighHighAlarm,
    /// 报警值设定整体状态（AI模块）
    AlarmValueSetting,
    /// 维护功能测试（AI/AO模块）
    MaintenanceFunction,

    // DI/DO模块特有测试项
    /// 状态显示/回读测试（DI/DO模块）
    StateDisplay,

    // AO模块特有测试项（可选，可能包含在HardPoint内）
    /// 输出0%测试
    Output0Percent,
    /// 输出25%测试
    Output25Percent,
    /// 输出50%测试
    Output50Percent,
    /// 输出75%测试
    Output75Percent,
    /// 输出100%测试
    Output100Percent,

    // 通信测试项
    /// 通信连接测试
    CommunicationTest,

    // 自定义测试项（支持扩展）
    Custom(String),
}

impl Default for SubTestItem {
    fn default() -> Self {
        SubTestItem::HardPoint
    }
}

/// 日志级别枚举
/// 用于系统日志记录
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LogLevel {
    /// 调试级别
    Debug,
    /// 信息级别
    Info,
    /// 警告级别
    Warning,
    /// 错误级别
    Error,
    /// 致命错误级别
    Fatal,
}

impl Default for LogLevel {
    fn default() -> Self {
        Self::Info
    }
}

// ==================== 字节顺序 ====================
/// PLC 字节顺序
/// 与 Modbus/TCP 的寄存器组合方式对应
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ByteOrder {
    /// 寄存器高字节在前，高字在前 (AB CD)
    ABCD,
    /// 寄存器低字在前，高字节在前 (CD AB)
    CDAB,
    /// 寄存器高字节在前，低字在前 (BA DC)
    BADC,
    /// 寄存器低字节在前，低字节在前 (DC BA)
    DCBA,
}

impl Default for ByteOrder {
    fn default() -> Self {
        ByteOrder::CDAB
    }
}

impl Display for ByteOrder {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            ByteOrder::ABCD => "ABCD",
            ByteOrder::CDAB => "CDAB",
            ByteOrder::BADC => "BADC",
            ByteOrder::DCBA => "DCBA",
        };
        write!(f, "{}", s)
    }
}

impl FromStr for ByteOrder {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "ABCD" => Ok(ByteOrder::ABCD),
            "CDAB" => Ok(ByteOrder::CDAB),
            "BADC" => Ok(ByteOrder::BADC),
            "DCBA" => Ok(ByteOrder::DCBA),
            _ => Err(format!("Unsupported ByteOrder: {}", s)),
        }
    }
}
