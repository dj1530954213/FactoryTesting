// 文件: FactoryTesting/src-tauri/src/models/entities/raw_test_outcome.rs
// 详细注释：RawTestOutcome实体的SeaORM定义

use sea_orm::entity::prelude::*;
use sea_orm::ActiveValue::Set;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use crate::models::structs::{AnalogReadingPoint}; // 引入所需结构体
use crate::models::enums::SubTestItem; // 修正路径：引入所需枚举

// RawTestOutcome 结构体也需要一个唯一ID作为主键，即使原始结构体没有
// 如果 RawTestOutcome 总是与 ChannelTestInstance 关联，并且其生命周期依赖于此，
// 那么 (channel_instance_id, sub_test_item, timestamp) 的组合可能构成一个自然键，但这在 SeaORM 中更复杂。
// 最简单的方法是添加一个 UUID 主键。
use uuid::Uuid;
fn default_outcome_id() -> String {
    Uuid::new_v4().to_string()
}

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "raw_test_outcomes")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    #[serde(default = "default_outcome_id")]
    pub id: String, // 新增主键

    pub channel_instance_id: String,
    
    #[sea_orm(column_type = "Text")]
    pub sub_test_item: String, // 原为 SubTestItem 枚举
    
    pub success: bool,
    #[sea_orm(nullable)]
    pub raw_value_read: Option<String>,
    #[sea_orm(nullable)]
    pub eng_value_calculated: Option<String>,
    #[sea_orm(nullable)]
    pub message: Option<String>,
    
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,

    // Option<Vec<AnalogReadingPoint>> 序列化为 JSON 字符串存储
    #[sea_orm(column_type = "Text", nullable)]
    pub readings_json: Option<String>,

    // 百分比测试结果字段 - 存储实际工程量
    #[sea_orm(nullable)]
    pub test_result_0_percent: Option<f64>,
    #[sea_orm(nullable)]
    pub test_result_25_percent: Option<f64>,
    #[sea_orm(nullable)]
    pub test_result_50_percent: Option<f64>,
    #[sea_orm(nullable)]
    pub test_result_75_percent: Option<f64>,
    #[sea_orm(nullable)]
    pub test_result_100_percent: Option<f64>,

    // HashMap<String, serde_json::Value> 序列化为 JSON 字符串存储
    #[sea_orm(column_type = "Text", nullable)]
    pub details_json: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    // #[sea_orm(
    //     belongs_to = "super::channel_test_instance::Entity",
    //     from = "Column::ChannelInstanceId",
    //     to = "super::channel_test_instance::Column::InstanceId"
    // )]
    // ChannelTestInstance,
}

impl ActiveModelBehavior for ActiveModel {}

// 注意：原始的 RawTestOutcome 结构体没有 id 字段。
// 在转换为 ActiveModel 时，如果需要创建新的 RawTestOutcome，则应生成一个新的 id。
// 如果是从已有的 RawTestOutcome（例如从其他地方加载而来，且我们认为它应该有 id）转换，则应使用该 id。
// 为简单起见，这里的 From 实现假设总是为 ActiveModel 生成新 id (如果 original 中没有)。
impl From<&crate::models::structs::RawTestOutcome> for ActiveModel {
    fn from(original: &crate::models::structs::RawTestOutcome) -> Self {
        let readings_json = serde_json::to_string(&original.readings).ok();
        let details_json = serde_json::to_string(&original.details).ok();

        Self {
            // 假设原始 RawTestOutcome 没有 id，所以我们在这里生成一个新的
            // 如果业务逻辑要求 RawTestOutcome 自身有持久化的id，则需要调整原始结构体和这里的逻辑
            id: Set(default_outcome_id()), // 生成新的 UUID 作为 ID
            channel_instance_id: Set(original.channel_instance_id.clone()),
            sub_test_item: Set(format!("{:?}", original.sub_test_item)),
            success: Set(original.success),
            raw_value_read: Set(original.raw_value_read.clone()),
            eng_value_calculated: Set(original.eng_value_calculated.clone()),
            message: Set(original.message.clone()),
            start_time: Set(original.start_time),
            end_time: Set(original.end_time),
            readings_json: Set(readings_json),
            test_result_0_percent: Set(original.test_result_0_percent),
            test_result_25_percent: Set(original.test_result_25_percent),
            test_result_50_percent: Set(original.test_result_50_percent),
            test_result_75_percent: Set(original.test_result_75_percent),
            test_result_100_percent: Set(original.test_result_100_percent),
            details_json: Set(details_json),
            ..Default::default()
        }
    }
}

impl From<&Model> for crate::models::structs::RawTestOutcome {
    fn from(model: &Model) -> Self {
        let readings: Option<Vec<AnalogReadingPoint>> = model.readings_json.as_ref()
            .and_then(|json| serde_json::from_str(json).ok());
        let details: HashMap<String, serde_json::Value> = model.details_json.as_ref()
            .and_then(|json| serde_json::from_str(json).ok())
            .unwrap_or_default();

        crate::models::structs::RawTestOutcome {
            // 原始 RawTestOutcome 结构体没有 id 字段，所以这里不映射 model.id
            channel_instance_id: model.channel_instance_id.clone(),
            sub_test_item: match model.sub_test_item.as_str() {
                "HardPoint" => SubTestItem::HardPoint,
                "TrendCheck" => SubTestItem::TrendCheck,
                "ReportCheck" => SubTestItem::ReportCheck,
                "LowLowAlarm" => SubTestItem::LowLowAlarm,
                "LowAlarm" => SubTestItem::LowAlarm,
                "HighAlarm" => SubTestItem::HighAlarm,
                "HighHighAlarm" => SubTestItem::HighHighAlarm,
                "AlarmValueSetting" => SubTestItem::AlarmValueSetting,
                "MaintenanceFunction" => SubTestItem::MaintenanceFunction,
                "StateDisplay" => SubTestItem::StateDisplay,
                _ => SubTestItem::default(), // 或者需要更具体的 SubTestItem::Unknown
            },
            success: model.success,
            raw_value_read: model.raw_value_read.clone(),
            eng_value_calculated: model.eng_value_calculated.clone(),
            message: model.message.clone(),
            start_time: model.start_time,
            end_time: model.end_time,
            readings,
            digital_steps: None, // TODO: 从JSON字段反序列化
            test_result_0_percent: model.test_result_0_percent,
            test_result_25_percent: model.test_result_25_percent,
            test_result_50_percent: model.test_result_50_percent,
            test_result_75_percent: model.test_result_75_percent,
            test_result_100_percent: model.test_result_100_percent,
            details,
        }
    }
} 
