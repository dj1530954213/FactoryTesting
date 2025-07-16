//! # PLC连接配置实体模块
//!
//! ## 业务作用
//! 本模块定义了PLC连接配置的数据模型，用于：
//! - **配置管理**: 存储和管理PLC设备的连接参数
//! - **数据持久化**: 通过SeaORM实现数据库操作
//! - **序列化支持**: 支持JSON序列化用于API交互
//! - **类型安全**: 使用强类型确保配置数据的正确性
//! - **默认值处理**: 提供合理的默认配置值
//!
//! ## 数据模型设计
//! - **实体映射**: 对应数据库中的plc_connection_configs表
//! - **字段完整性**: 包含PLC连接所需的所有参数
//! - **状态管理**: 记录连接状态和时间戳信息
//! - **扩展性**: 支持多种PLC协议类型
//!
//! ## 技术特点
//! - **ORM集成**: 基于SeaORM的实体定义
//! - **序列化**: 支持serde的序列化和反序列化
//! - **时间处理**: 使用chrono处理时间相关字段
//! - **验证机制**: 内置数据验证和默认值处理
//!
//! ## Rust知识点
//! - **derive宏**: 自动实现常用trait
//! - **属性宏**: SeaORM和serde的属性配置
//! - **Option类型**: 处理可空字段
//! - **生命周期**: 异步trait的生命周期管理

use sea_orm::entity::prelude::*;
use sea_orm::ActiveValue::Set;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use crate::models::structs::default_id;

/// PLC连接配置实体模型
///
/// **业务含义**: 表示一个PLC设备的完整连接配置信息
/// **数据库映射**: 对应plc_connection_configs表
/// **使用场景**: PLC连接管理、配置存储、状态跟踪
///
/// **设计原则**:
/// - **完整性**: 包含PLC连接所需的所有参数
/// - **可扩展性**: 支持多种PLC协议和厂商
/// - **状态跟踪**: 记录连接状态和历史信息
/// - **类型安全**: 使用强类型避免配置错误
///
/// **Rust知识点**:
/// - `#[derive(...)]`: 自动实现多个trait
/// - `Clone`: 支持值的克隆
/// - `Debug`: 支持调试输出
/// - `PartialEq`: 支持相等性比较
/// - `DeriveEntityModel`: SeaORM实体模型宏
/// - `Serialize/Deserialize`: serde序列化支持
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "plc_connection_configs")]
pub struct Model {
    /// 连接配置的唯一标识符
    /// **业务含义**: 用于唯一标识一个PLC连接配置
    /// **数据库约束**: 主键，非自增
    /// **默认值**: 通过default_id()函数生成UUID
    /// **序列化**: serde默认值处理
    #[sea_orm(primary_key, auto_increment = false)]
    #[serde(default = "default_id")]
    pub id: String,

    /// PLC连接的显示名称
    /// **业务含义**: 用户友好的连接名称，便于识别
    /// **使用场景**: UI显示、日志记录、错误报告
    /// **数据验证**: 应该是非空且有意义的名称
    pub name: String,

    /// PLC设备类型
    /// **业务含义**: 指定PLC的协议类型
    /// **支持类型**: ModbusTcp, SiemensS7, OpcUa, Mock
    /// **扩展性**: 可以添加新的PLC协议类型
    /// **验证**: 应该是预定义的有效类型之一
    pub plc_type: String,

    /// PLC设备的IP地址
    /// **业务含义**: PLC设备在网络中的地址
    /// **格式要求**: 标准的IPv4地址格式（如192.168.1.100）
    /// **网络配置**: 必须与系统网络配置兼容
    pub ip_address: String,

    /// PLC设备的端口号
    /// **业务含义**: PLC服务监听的网络端口
    /// **默认端口**: Modbus TCP通常使用502端口
    /// **范围限制**: 1-65535的有效端口范围
    /// **数据类型**: i32用于数据库兼容性
    pub port: i32,

    /// 连接超时时间（毫秒）
    /// **业务含义**: 建立连接的最大等待时间
    /// **性能影响**: 影响连接建立的响应速度
    /// **推荐值**: 通常设置为3000-10000毫秒
    /// **故障处理**: 超时后触发重连机制
    pub timeout: i32,

    /// 连接失败时的重试次数
    /// **业务含义**: 连接失败后的自动重试次数
    /// **可靠性**: 提高连接成功率
    /// **避免过载**: 防止对PLC设备造成过大压力
    /// **推荐值**: 通常设置为3-5次
    pub retry_count: i32,

    /// 数据字节顺序
    /// **业务含义**: 多字节数据的存储顺序
    /// **支持格式**: ABCD(大端), CDAB, BADC, DCBA(小端)
    /// **厂商差异**: 不同PLC厂商可能使用不同的字节序
    /// **数据正确性**: 错误的字节序会导致数据解析错误
    pub byte_order: String,

    /// 地址编码是否从0开始
    /// **业务含义**: PLC地址的起始编号方式
    /// **厂商差异**: 有些PLC地址从0开始，有些从1开始
    /// **地址转换**: 影响地址的内部转换逻辑
    /// **兼容性**: 确保与PLC设备的地址编码一致
    pub zero_based_address: bool,

    /// 是否为测试用PLC
    /// **业务含义**: 标识是否为测试环境的PLC
    /// **环境隔离**: 区分生产环境和测试环境
    /// **安全考虑**: 测试PLC可能有不同的安全策略
    pub is_test_plc: bool,

    /// 连接配置的描述信息
    /// **业务含义**: 详细描述连接的用途和特点
    /// **可选字段**: 可以为空，用于提供额外信息
    /// **文档价值**: 帮助理解配置的业务背景
    #[sea_orm(nullable)]
    pub description: Option<String>,

    /// 连接是否启用
    /// **业务含义**: 控制连接是否参与自动连接
    /// **运维控制**: 可以临时禁用某个连接
    /// **系统管理**: 支持连接的动态启停
    pub is_enabled: bool,

    /// 最后成功连接的时间
    /// **业务含义**: 记录最近一次成功连接的时间戳
    /// **可选字段**: 从未连接过时为None
    /// **监控价值**: 用于连接状态监控和故障分析
    /// **时区处理**: 使用UTC时间避免时区问题
    #[sea_orm(nullable)]
    pub last_connected: Option<DateTime<Utc>>,

    /// 当前连接状态
    /// **业务含义**: 表示连接的实时状态
    /// **状态类型**: Disconnected, Connecting, Connected, Error, Timeout
    /// **状态同步**: 与实际连接状态保持同步
    /// **监控依据**: 系统监控的重要指标
    pub connection_status: String,

    /// 配置创建时间
    /// **业务含义**: 记录配置首次创建的时间
    /// **审计价值**: 用于配置变更的审计跟踪
    /// **不可变**: 创建后不会改变
    pub created_at: DateTime<Utc>,

    /// 配置最后更新时间
    /// **业务含义**: 记录配置最后修改的时间
    /// **变更跟踪**: 每次更新时自动更新此字段
    /// **版本控制**: 可用于实现简单的版本控制
    pub updated_at: DateTime<Utc>,
}

/// 实体关系定义
///
/// **业务作用**: 定义PLC连接配置与其他实体之间的关系
/// **扩展性**: 为未来的关系定义预留空间
///
/// **潜在关系**:
/// - 与测试PLC通道配置的一对多关系
/// - 与连接日志的一对多关系
/// - 与测试任务的多对多关系
///
/// **Rust知识点**:
/// - `#[derive(...)]`: 自动实现必要的trait
/// - `Copy`: 支持按位复制
/// - `Clone`: 支持克隆
/// - `Debug`: 支持调试输出
/// - `EnumIter`: 支持枚举迭代
/// - `DeriveRelation`: SeaORM关系派生宏
#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    // 未来可以定义与测试PLC通道配置的关系
    // 例如: TestPlcChannelConfig -> HasMany(super::test_plc_channel_config::Entity)
}

/// ActiveModel行为实现
///
/// **业务作用**: 定义实体的活动模型行为，包括创建和保存时的逻辑
/// **生命周期管理**: 自动处理时间戳和默认值
///
/// **主要功能**:
/// - **创建时**: 设置默认值和初始时间戳
/// - **保存前**: 更新时间戳和执行验证
/// - **数据一致性**: 确保数据的完整性和正确性
///
/// **Rust知识点**:
/// - `ActiveModelBehavior`: SeaORM的活动模型行为trait
/// - `Set()`: SeaORM的活动值设置
/// - `..`: 结构体更新语法
/// - `ActiveModelTrait::default()`: 获取默认的活动模型
impl ActiveModelBehavior for ActiveModel {
    /// 创建新的活动模型实例
    ///
    /// **业务逻辑**:
    /// 1. 生成唯一的ID
    /// 2. 设置创建和更新时间为当前时间
    /// 3. 设置合理的默认值
    /// 4. 其他字段使用默认值
    ///
    /// **默认值说明**:
    /// - `id`: 生成UUID作为唯一标识
    /// - `created_at/updated_at`: 当前UTC时间
    /// - `byte_order`: "CDAB" - 常见的Modbus字节序
    /// - `zero_based_address`: false - 大多数PLC从1开始编址
    ///
    /// **返回值**: 配置了默认值的活动模型实例
    fn new() -> Self {
        Self {
            id: Set(default_id()),                    // 生成唯一ID
            created_at: Set(Utc::now()),             // 设置创建时间
            updated_at: Set(Utc::now()),             // 设置更新时间
            byte_order: Set("CDAB".to_string()),     // 默认字节序
            zero_based_address: Set(false),          // 默认地址模式
            ..ActiveModelTrait::default()            // 其他字段使用默认值
        }
    }

    fn before_save<'life0, 'async_trait, C>(
        mut self,
        _db: &'life0 C,
        insert: bool,
    ) -> core::pin::Pin<Box<dyn core::future::Future<Output = Result<Self, DbErr>> + core::marker::Send + 'async_trait>>
    where
        C: 'async_trait + ConnectionTrait,
        'life0: 'async_trait,
        Self: 'async_trait,
    {
        Box::pin(async move {
            if !insert {
                self.updated_at = Set(Utc::now());
            }
            Ok(self)
        })
    }
}

/// 从业务模型转换为数据库实体
impl From<&crate::models::test_plc_config::PlcConnectionConfig> for ActiveModel {
    fn from(config: &crate::models::test_plc_config::PlcConnectionConfig) -> Self {
        Self {
            id: Set(config.id.clone()),
            name: Set(config.name.clone()),
            plc_type: Set(format!("{:?}", config.plc_type)),
            ip_address: Set(config.ip_address.clone()),
            port: Set(config.port),
            timeout: Set(config.timeout),
            retry_count: Set(config.retry_count),
            byte_order: Set(config.byte_order.clone()),
            zero_based_address: Set(config.zero_based_address),
            is_test_plc: Set(config.is_test_plc),
            description: Set(config.description.clone()),
            is_enabled: Set(config.is_enabled),
            last_connected: Set(config.last_connected),
            connection_status: Set(format!("{:?}", config.connection_status)),
            created_at: Set(Utc::now()),
            updated_at: Set(Utc::now()),
        }
    }
}

/// 从数据库实体转换为业务模型
impl From<&Model> for crate::models::test_plc_config::PlcConnectionConfig {
    fn from(model: &Model) -> Self {
        Self {
            id: model.id.clone(),
            name: model.name.clone(),
            plc_type: match model.plc_type.as_str() {
                "ModbusTcp" => crate::models::test_plc_config::PlcType::ModbusTcp,
                "SiemensS7" => crate::models::test_plc_config::PlcType::SiemensS7,
                "OpcUa" => crate::models::test_plc_config::PlcType::OpcUa,
                _ => crate::models::test_plc_config::PlcType::ModbusTcp, // 默认值
            },
            ip_address: model.ip_address.clone(),
            port: model.port,
            timeout: model.timeout,
            retry_count: model.retry_count,
            byte_order: model.byte_order.clone(),
            zero_based_address: model.zero_based_address,
            is_test_plc: model.is_test_plc,
            description: model.description.clone(),
            is_enabled: model.is_enabled,
            last_connected: model.last_connected,
            connection_status: match model.connection_status.as_str() {
                "Disconnected" => crate::models::test_plc_config::ConnectionStatus::Disconnected,
                "Connecting" => crate::models::test_plc_config::ConnectionStatus::Connecting,
                "Connected" => crate::models::test_plc_config::ConnectionStatus::Connected,
                "Error" => crate::models::test_plc_config::ConnectionStatus::Error,
                "Timeout" => crate::models::test_plc_config::ConnectionStatus::Timeout,
                _ => crate::models::test_plc_config::ConnectionStatus::Disconnected, // 默认值
            },
        }
    }
} 
