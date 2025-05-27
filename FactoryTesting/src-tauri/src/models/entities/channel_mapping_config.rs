// 文件: FactoryTesting/src-tauri/src/models/entities/channel_mapping_config.rs
// 详细注释：通道映射配置实体的SeaORM定义

use sea_orm::entity::prelude::*;
use sea_orm::ActiveValue::Set;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use crate::models::structs::default_id;

/// 通道映射配置实体
/// 对应数据库中的 channel_mapping_configs 表
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "channel_mapping_configs")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    #[serde(default = "default_id")]
    pub id: String,
    
    /// 被测通道ID (ChannelPointDefinition.id)
    pub target_channel_id: String,
    
    /// 测试PLC通道ID (TestPlcChannelConfig.id)
    pub test_plc_channel_id: String,
    
    /// 映射类型 (Direct, Inverse, Scaled, Custom)
    pub mapping_type: String,
    
    /// 是否激活
    pub is_active: bool,
    
    /// 备注信息
    #[sea_orm(nullable)]
    pub notes: Option<String>,
    
    /// 创建时间
    pub created_at: DateTime<Utc>,
    
    /// 更新时间
    pub updated_at: DateTime<Utc>,
}

/// 定义实体间的关系
#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    // 可以定义与 ChannelPointDefinition 和 TestPlcChannelConfig 的关系
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
impl From<&crate::models::test_plc_config::ChannelMappingConfig> for ActiveModel {
    fn from(config: &crate::models::test_plc_config::ChannelMappingConfig) -> Self {
        Self {
            id: Set(config.id.clone()),
            target_channel_id: Set(config.target_channel_id.clone()),
            test_plc_channel_id: Set(config.test_plc_channel_id.clone()),
            mapping_type: Set(format!("{:?}", config.mapping_type)),
            is_active: Set(config.is_active),
            notes: Set(config.notes.clone()),
            created_at: Set(config.created_at),
            updated_at: Set(Utc::now()),
        }
    }
}

/// 从数据库实体转换为业务模型
impl From<&Model> for crate::models::test_plc_config::ChannelMappingConfig {
    fn from(model: &Model) -> Self {
        Self {
            id: model.id.clone(),
            target_channel_id: model.target_channel_id.clone(),
            test_plc_channel_id: model.test_plc_channel_id.clone(),
            mapping_type: match model.mapping_type.as_str() {
                "Direct" => crate::models::test_plc_config::MappingType::Direct,
                "Inverse" => crate::models::test_plc_config::MappingType::Inverse,
                "Scaled" => crate::models::test_plc_config::MappingType::Scaled,
                "Custom" => crate::models::test_plc_config::MappingType::Custom,
                _ => crate::models::test_plc_config::MappingType::Direct, // 默认值
            },
            is_active: model.is_active,
            notes: model.notes.clone(),
            created_at: model.created_at,
        }
    }
} 