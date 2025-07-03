// 文件: FactoryTesting/src-tauri/src/models/entities/test_plc_channel_config.rs
// 详细注释：测试PLC通道配置实体的SeaORM定义

use sea_orm::entity::prelude::*;
use sea_orm::ActiveValue::Set;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use crate::models::structs::default_id;

/// 测试PLC通道配置实体
/// 对应数据库中的 test_plc_channel_configs 表
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "test_plc_channel_configs")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    #[serde(default = "default_id")]
    pub id: String,
    
    /// 通道位号 (如: AI1_1, AO1_2)
    pub channel_address: String,
    
    /// 通道类型 (0-7: AI, AO, DI, DO, AINone, AONone, DINone, DONone)
    pub channel_type: i32,
    
    /// 通讯地址 (如: 40101, 00101)
    pub communication_address: String,
    
    /// 供电类型 (必填项)
    pub power_supply_type: String,
    
    /// 描述信息
    #[sea_orm(nullable)]
    pub description: Option<String>,
    
    /// 是否启用
    pub is_enabled: bool,
    
    /// 创建时间
    pub created_at: DateTime<Utc>,
    
    /// 更新时间
    pub updated_at: DateTime<Utc>,
}

/// 定义实体间的关系
#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    // 未来可以定义与通道映射配置的关系
}

/// 实现 ActiveModelBehavior trait
impl ActiveModelBehavior for ActiveModel {
    fn new() -> Self {
        Self {
            id: Set(default_id()),
            created_at: Set(Utc::now()),
            updated_at: Set(Utc::now()),
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
impl From<&crate::models::test_plc_config::TestPlcChannelConfig> for ActiveModel {
    fn from(config: &crate::models::test_plc_config::TestPlcChannelConfig) -> Self {
        Self {
            id: Set(config.id.clone().unwrap_or_else(|| default_id())),
            channel_address: Set(config.channel_address.clone()),
            channel_type: Set(config.channel_type as i32),
            communication_address: Set(config.communication_address.clone()),
            power_supply_type: Set(config.power_supply_type.clone()),
            description: Set(config.description.clone()),
            is_enabled: Set(config.is_enabled),
            created_at: Set(config.created_at.unwrap_or_else(|| Utc::now())),
            updated_at: Set(config.updated_at.unwrap_or_else(|| Utc::now())),
        }
    }
}

/// 从数据库实体转换为业务模型
impl From<&Model> for crate::models::test_plc_config::TestPlcChannelConfig {
    fn from(model: &Model) -> Self {
        Self {
            id: Some(model.id.clone()),
            channel_address: model.channel_address.clone(),
            channel_type: match model.channel_type {
                0 => crate::models::test_plc_config::TestPlcChannelType::AI,
                1 => crate::models::test_plc_config::TestPlcChannelType::AO,
                2 => crate::models::test_plc_config::TestPlcChannelType::DI,
                3 => crate::models::test_plc_config::TestPlcChannelType::DO,
                4 => crate::models::test_plc_config::TestPlcChannelType::AINone,
                5 => crate::models::test_plc_config::TestPlcChannelType::AONone,
                6 => crate::models::test_plc_config::TestPlcChannelType::DINone,
                7 => crate::models::test_plc_config::TestPlcChannelType::DONone,
                _ => crate::models::test_plc_config::TestPlcChannelType::AI, // 默认值
            },
            communication_address: model.communication_address.clone(),
            power_supply_type: model.power_supply_type.clone(),
            description: model.description.clone(),
            is_enabled: model.is_enabled,
            created_at: Some(model.created_at),
            updated_at: Some(model.updated_at),
        }
    }
} 
