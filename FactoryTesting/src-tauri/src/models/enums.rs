//! # 模型枚举类型模块
//!
//! ## 业务作用
//! 本模块定义了系统中使用的各种枚举类型，包括：
//! - **测试状态枚举**: 表示测试流程中的各种状态
//! - **PLC相关枚举**: PLC协议、数据类型、连接状态等
//! - **字节序枚举**: 处理不同PLC厂商的字节序差异
//! - **业务流程枚举**: 测试流程、状态转换等业务逻辑
//!
//! ## 设计原则
//! - **类型安全**: 使用强类型枚举避免魔法数字和字符串
//! - **序列化支持**: 所有枚举都支持JSON序列化
//! - **字符串转换**: 提供与字符串的双向转换能力
//! - **默认值**: 为枚举提供合理的默认值
//! - **向后兼容**: 通过重新导出保持API稳定性
//!
//! ## Rust知识点
//! - **枚举类型**: Rust的代数数据类型
//! - **trait实现**: Display、FromStr、Default等trait
//! - **derive宏**: 自动实现常用trait
//! - **模块重导出**: pub use语句的使用

use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::str::FromStr;

// ==================== PLC协议类型重导出 ====================

/// PLC协议类型重导出
///
/// **业务作用**: 为了保持向后兼容性，从领域服务层重新导出PlcProtocol枚举
/// **使用场景**: 允许旧代码继续使用models::enums::PlcProtocol路径
///
/// **协议支持**:
/// - `ModbusTcp`: Modbus TCP/IP协议，最常用的工业通信协议
/// - `SiemensS7`: 西门子S7通信协议，用于西门子PLC
/// - `OpcUa`: OPC统一架构协议，现代工业4.0标准
/// - `EthernetIp`: 以太网/IP协议，主要用于罗克韦尔设备
///
/// **设计考虑**:
/// - 协议定义在领域层，确保业务逻辑的一致性
/// - 通过重导出提供便捷的访问路径
/// - 支持未来添加新的协议类型
///
/// **Rust知识点**:
/// - `pub use`: 重新导出，使外部可以通过当前模块访问
/// - 模块路径：通过完整路径引用其他模块的类型
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

// ==================== PLC字节顺序枚举 ====================

/// PLC数据字节顺序枚举
///
/// **业务作用**:
/// - 定义多字节数据在PLC寄存器中的存储顺序
/// - 解决不同PLC厂商字节序差异的兼容性问题
/// - 确保浮点数和多字节整数的正确解析
///
/// **技术背景**:
/// - Modbus协议使用16位寄存器存储数据
/// - 32位数据需要占用2个连续寄存器
/// - 不同厂商对字节在寄存器中的排列方式不同
/// - 错误的字节序会导致数据解析完全错误
///
/// **字节序说明**:
/// 以32位浮点数1.0为例，其IEEE 754表示为0x3F800000
/// - ABCD: 3F80 0000 (大端序，高字在前，高字节在前)
/// - CDAB: 0000 3F80 (低字在前，高字节在前) - 最常见
/// - BADC: 803F 0000 (高字在前，低字节在前)
/// - DCBA: 0000 803F (小端序，低字在前，低字节在前)
///
/// **厂商差异**:
/// - 西门子PLC通常使用ABCD或BADC
/// - 施耐德PLC通常使用CDAB
/// - 三菱PLC通常使用DCBA
/// - Modbus RTU设备多数使用CDAB
///
/// **Rust知识点**:
/// - `#[derive(...)]`: 自动实现常用trait
/// - `Copy`: 支持按位复制，性能更好
/// - `PartialEq, Eq`: 支持相等性比较
/// - `Serialize/Deserialize`: 支持JSON序列化
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ByteOrder {
    /// 大端序：高字在前，高字节在前 (AB CD)
    /// **适用设备**: 西门子S7-300/400系列PLC
    /// **字节排列**: [高字高字节, 高字低字节, 低字高字节, 低字低字节]
    /// **示例**: 浮点数1.0存储为 [3F, 80, 00, 00]
    ABCD,

    /// 混合序：低字在前，高字节在前 (CD AB) - 默认值
    /// **适用设备**: 大多数Modbus RTU设备、施耐德PLC
    /// **字节排列**: [低字高字节, 低字低字节, 高字高字节, 高字低字节]
    /// **示例**: 浮点数1.0存储为 [00, 00, 3F, 80]
    /// **常见性**: 这是最常见的字节序，因此设为默认值
    CDAB,

    /// 混合序：高字在前，低字节在前 (BA DC)
    /// **适用设备**: 部分西门子PLC、ABB设备
    /// **字节排列**: [高字低字节, 高字高字节, 低字低字节, 低字高字节]
    /// **示例**: 浮点数1.0存储为 [80, 3F, 00, 00]
    BADC,

    /// 小端序：低字在前，低字节在前 (DC BA)
    /// **适用设备**: 三菱PLC、部分欧姆龙PLC
    /// **字节排列**: [低字低字节, 低字高字节, 高字低字节, 高字高字节]
    /// **示例**: 浮点数1.0存储为 [00, 00, 80, 3F]
    DCBA,
}

/// 默认字节序实现
///
/// **业务考虑**: 选择CDAB作为默认值的原因：
/// 1. 这是Modbus协议中最常见的字节序
/// 2. 大多数工业设备都支持这种格式
/// 3. 与大部分PLC厂商的默认设置一致
///
/// **Rust知识点**:
/// - `Default` trait提供类型的默认值
/// - 在结构体初始化时可以使用`..Default::default()`
impl Default for ByteOrder {
    fn default() -> Self {
        ByteOrder::CDAB // 最常见的字节序作为默认值
    }
}

/// 字节序的字符串显示实现
///
/// **业务用途**:
/// - 用于配置文件的可读性
/// - 日志输出和错误信息
/// - 用户界面的显示
///
/// **格式标准**: 使用4个字母的标准表示法
impl Display for ByteOrder {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            ByteOrder::ABCD => "ABCD", // 大端序标准表示
            ByteOrder::CDAB => "CDAB", // 混合序标准表示
            ByteOrder::BADC => "BADC", // 混合序标准表示
            ByteOrder::DCBA => "DCBA", // 小端序标准表示
        };
        write!(f, "{}", s)
    }
}

/// 从字符串解析字节序
///
/// **业务用途**:
/// - 从配置文件读取字节序设置
/// - 解析用户输入的字节序参数
/// - 支持API参数的字符串格式
///
/// **容错处理**:
/// - 支持大小写不敏感的解析
/// - 提供详细的错误信息
/// - 验证输入的有效性
///
/// **Rust知识点**:
/// - `FromStr` trait支持字符串解析
/// - `to_uppercase()`: 转换为大写进行匹配
/// - `Result<T, E>`: 错误处理类型
impl FromStr for ByteOrder {
    type Err = String; // 错误类型为字符串，便于错误信息传递

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() { // 大小写不敏感匹配
            "ABCD" => Ok(ByteOrder::ABCD),
            "CDAB" => Ok(ByteOrder::CDAB),
            "BADC" => Ok(ByteOrder::BADC),
            "DCBA" => Ok(ByteOrder::DCBA),
            _ => Err(format!("不支持的字节序格式: {}，支持的格式: ABCD, CDAB, BADC, DCBA", s)),
        }
    }
}
