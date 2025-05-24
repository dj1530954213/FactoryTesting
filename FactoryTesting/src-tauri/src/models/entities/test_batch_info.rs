// 文件: FactoryTesting/src-tauri/src/models/entities/test_batch_info.rs
// 详细注释：TestBatchInfo实体的SeaORM定义

use sea_orm::entity::prelude::*;
use sea_orm::ActiveValue::Set;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::collections::HashMap; // 确保 HashMap 已导入
use crate::models::structs::{default_id}; // 确保可以访问 default_id

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "test_batch_info")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    #[serde(default = "default_id")]
    pub batch_id: String, // 原为 id，但在 TestBatchInfo 中是 batch_id

    #[sea_orm(nullable)]
    pub product_model: Option<String>,
    #[sea_orm(nullable)]
    pub serial_number: Option<String>,
    #[sea_orm(nullable)]
    pub customer_name: Option<String>,

    pub creation_time: DateTime<Utc>,
    pub last_updated_time: DateTime<Utc>, // TestBatchInfo 中有此字段

    #[sea_orm(nullable)]
    pub operator_name: Option<String>, // TestBatchInfo 中有此字段
    #[sea_orm(nullable)]
    pub status_summary: Option<String>,
    
    pub total_points: u32,
    pub tested_points: u32,
    pub passed_points: u32,
    pub failed_points: u32,
    pub skipped_points: u32, // TestBatchInfo 中有此字段

    // HashMap<String, String> 需要序列化为 Text/JSONB
    // 为了简化，这里先假设其能被 SeaORM 作为 Text 处理（通过 serde 的 Serialize/Deserialize）
    #[sea_orm(column_type = "Text", nullable)] // 假设 custom_data 可以为 null
    pub custom_data: Option<String>, // 原为 HashMap<String, String>
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    // 未来可以定义与 ChannelTestInstance 的关系
    // #[sea_orm(has_many = "super::channel_test_instance::Entity")]
    // ChannelTestInstance,
}

impl ActiveModelBehavior for ActiveModel {}

impl From<&crate::models::structs::TestBatchInfo> for ActiveModel {
    fn from(original: &crate::models::structs::TestBatchInfo) -> Self {
        let custom_data_json = serde_json::to_string(&original.custom_data)
            .unwrap_or_else(|_| "{}".to_string()); // 序列化 HashMap 为 JSON 字符串

        Self {
            batch_id: Set(original.batch_id.clone()),
            product_model: Set(original.product_model.clone()),
            serial_number: Set(original.serial_number.clone()),
            customer_name: Set(original.customer_name.clone()),
            creation_time: Set(original.creation_time),
            last_updated_time: Set(original.last_updated_time),
            operator_name: Set(original.operator_name.clone()),
            status_summary: Set(original.status_summary.clone()),
            total_points: Set(original.total_points),
            tested_points: Set(original.tested_points),
            passed_points: Set(original.passed_points),
            failed_points: Set(original.failed_points),
            skipped_points: Set(original.skipped_points),
            custom_data: Set(Some(custom_data_json)), // 存储 JSON 字符串
            ..Default::default()
        }
    }
}

impl From<&Model> for crate::models::structs::TestBatchInfo {
    fn from(model: &Model) -> Self {
        let custom_data_map: HashMap<String, String> = model.custom_data.as_ref()
            .and_then(|json_str| serde_json::from_str(json_str).ok())
            .unwrap_or_default(); // 从 JSON 字符串反序列化回 HashMap

        crate::models::structs::TestBatchInfo {
            batch_id: model.batch_id.clone(),
            product_model: model.product_model.clone(),
            serial_number: model.serial_number.clone(),
            customer_name: model.customer_name.clone(),
            creation_time: model.creation_time,
            last_updated_time: model.last_updated_time,
            operator_name: model.operator_name.clone(),
            status_summary: model.status_summary.clone(),
            total_points: model.total_points,
            tested_points: model.tested_points,
            passed_points: model.passed_points,
            failed_points: model.failed_points,
            skipped_points: model.skipped_points,
            custom_data: custom_data_map,
        }
    }
} 