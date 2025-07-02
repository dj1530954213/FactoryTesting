// 文件: FactoryTesting/src-tauri/src/models/entities/plc_connection_config.rs
// 详细注释：PLC连接配置实体的SeaORM定义

use sea_orm::entity::prelude::*;
use sea_orm::ActiveValue::Set;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use crate::models::structs::default_id;

/// PLC连接配置实体
/// 对应数据库中的 plc_connection_configs 表
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "plc_connection_configs")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    #[serde(default = "default_id")]
    pub id: String,
    
    /// 连接名称
    pub name: String,
    
    /// PLC类型 (ModbusTcp, SiemensS7, OpcUa, Mock)
    pub plc_type: String,
    
    /// IP地址
    pub ip_address: String,
    
    /// 端口号
    pub port: i32,
    
    /// 超时时间(ms)
    pub timeout: i32,
    
    /// 重试次数
    pub retry_count: i32,
    
    /// 字节顺序 (ABCD / CDAB / BADC / DCBA)
    pub byte_order: String,
    
    /// 地址是否从0开始
    pub zero_based_address: bool,
    
    /// 是否为测试PLC
    pub is_test_plc: bool,
    
    /// 描述信息
    #[sea_orm(nullable)]
    pub description: Option<String>,
    
    /// 是否启用
    pub is_enabled: bool,
    
    /// 最后连接时间
    #[sea_orm(nullable)]
    pub last_connected: Option<DateTime<Utc>>,
    
    /// 连接状态 (Disconnected, Connecting, Connected, Error, Timeout)
    pub connection_status: String,
    
    /// 创建时间
    pub created_at: DateTime<Utc>,
    
    /// 更新时间
    pub updated_at: DateTime<Utc>,
}

/// 定义实体间的关系
#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    // 未来可以定义与测试PLC通道配置的关系
}

/// 实现 ActiveModelBehavior trait
impl ActiveModelBehavior for ActiveModel {
    fn new() -> Self {
        Self {
            id: Set(default_id()),
            created_at: Set(Utc::now()),
            updated_at: Set(Utc::now()),
            byte_order: Set("CDAB".to_string()),
            zero_based_address: Set(false),
            ..ActiveModelTrait::default()
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