// 文件: FactoryTesting/src-tauri/src/models/entities/channel_test_instance.rs
// 详细注释：ChannelTestInstance实体的SeaORM定义

use sea_orm::entity::prelude::*;
use sea_orm::ActiveValue::Set;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use crate::models::structs::{default_id, SubTestExecutionResult, AnalogReadingPoint}; // 引入所需结构体
use crate::models::enums::{OverallTestStatus, SubTestItem}; // 引入所需枚举

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "channel_test_instances")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    #[serde(default = "default_id")]
    pub instance_id: String,

    pub definition_id: String, // 关联到 ChannelPointDefinition 的 ID
    pub test_batch_id: String, // 关联到 TestBatchInfo 的 ID
    pub test_batch_name: String, // 测试批次名称

    #[sea_orm(column_type = "Text")]
    pub overall_status: String, // 原为 OverallTestStatus 枚举

    #[sea_orm(nullable)]
    pub current_step_details: Option<String>,
    #[sea_orm(nullable)]
    pub error_message: Option<String>,

    pub creation_time: DateTime<Utc>,
    #[sea_orm(nullable)]
    pub start_time: Option<DateTime<Utc>>,
    pub last_updated_time: DateTime<Utc>,
    #[sea_orm(nullable)]
    pub final_test_time: Option<DateTime<Utc>>,
    #[sea_orm(nullable)]
    pub total_test_duration_ms: Option<i64>,

    // HashMap<SubTestItem, SubTestExecutionResult> 序列化为 JSON 字符串存储
    #[sea_orm(column_type = "Text", nullable)]
    pub sub_test_results_json: Option<String>,

    // Option<Vec<AnalogReadingPoint>> 序列化为 JSON 字符串存储
    #[sea_orm(column_type = "Text", nullable)]
    pub hardpoint_readings_json: Option<String>,

    #[sea_orm(nullable)]
    pub manual_test_current_value_input: Option<String>,
    #[sea_orm(nullable)]
    pub manual_test_current_value_output: Option<String>,

    // 分配的测试PLC通道信息
    #[sea_orm(nullable)]
    pub test_plc_channel_tag: Option<String>,
    #[sea_orm(nullable)]
    pub test_plc_communication_address: Option<String>,

    #[sea_orm(nullable)]
    pub current_operator: Option<String>,
    pub retries_count: u32,

    // HashMap<String, serde_json::Value> 序列化为 JSON 字符串存储
    #[sea_orm(column_type = "Text", nullable)]
    pub transient_data_json: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    // #[sea_orm(
    //     belongs_to = "super::channel_point_definition::Entity",
    //     from = "Column::DefinitionId",
    //     to = "super::channel_point_definition::Column::Id"
    // )]
    // ChannelPointDefinition,

    // #[sea_orm(
    //     belongs_to = "super::test_batch_info::Entity",
    //     from = "Column::TestBatchId",
    //     to = "super::test_batch_info::Column::BatchId"
    // )]
    // TestBatchInfo,
}

impl ActiveModelBehavior for ActiveModel {}

impl From<&crate::models::structs::ChannelTestInstance> for ActiveModel {
    fn from(original: &crate::models::structs::ChannelTestInstance) -> Self {
        let sub_test_results_json = serde_json::to_string(&original.sub_test_results).ok();
        let hardpoint_readings_json = serde_json::to_string(&original.hardpoint_readings).ok();
        let transient_data_json = serde_json::to_string(&original.transient_data).ok();

        Self {
            instance_id: Set(original.instance_id.clone()),
            definition_id: Set(original.definition_id.clone()),
            test_batch_id: Set(original.test_batch_id.clone()),
            test_batch_name: Set(original.test_batch_name.clone()),
            overall_status: Set(format!("{:?}", original.overall_status)),
            current_step_details: Set(original.current_step_details.clone()),
            error_message: Set(original.error_message.clone()),
            creation_time: Set(original.creation_time),
            start_time: Set(original.start_time),
            last_updated_time: Set(original.last_updated_time),
            final_test_time: Set(original.final_test_time),
            total_test_duration_ms: Set(original.total_test_duration_ms),
            sub_test_results_json: Set(sub_test_results_json),
            hardpoint_readings_json: Set(hardpoint_readings_json),
            manual_test_current_value_input: Set(original.manual_test_current_value_input.clone()),
            manual_test_current_value_output: Set(original.manual_test_current_value_output.clone()),
            current_operator: Set(original.current_operator.clone()),
            retries_count: Set(original.retries_count),
            transient_data_json: Set(transient_data_json),
            test_plc_channel_tag: Set(original.test_plc_channel_tag.clone()),
            test_plc_communication_address: Set(original.test_plc_communication_address.clone()),
            ..Default::default()
        }
    }
}

impl From<&Model> for crate::models::structs::ChannelTestInstance {
    fn from(model: &Model) -> Self {
        let sub_test_results: HashMap<SubTestItem, SubTestExecutionResult> = model.sub_test_results_json.as_ref()
            .and_then(|json| serde_json::from_str(json).ok())
            .unwrap_or_default();
        let hardpoint_readings: Option<Vec<AnalogReadingPoint>> = model.hardpoint_readings_json.as_ref()
            .and_then(|json| serde_json::from_str(json).ok());
        let transient_data: HashMap<String, serde_json::Value> = model.transient_data_json.as_ref()
            .and_then(|json| serde_json::from_str(json).ok())
            .unwrap_or_default();
            
        crate::models::structs::ChannelTestInstance {
            instance_id: model.instance_id.clone(),
            definition_id: model.definition_id.clone(),
            test_batch_id: model.test_batch_id.clone(),
            test_batch_name: model.test_batch_name.clone(),
            overall_status: match model.overall_status.as_str() {
                "NotTested" => OverallTestStatus::NotTested,
                "Skipped" => OverallTestStatus::Skipped,
                "WiringConfirmed" => OverallTestStatus::WiringConfirmed,
                "HardPointTesting" => OverallTestStatus::HardPointTesting,
                "HardPointTestCompleted" => OverallTestStatus::HardPointTestCompleted,
                "ManualTesting" => OverallTestStatus::ManualTesting,
                "TestCompletedPassed" => OverallTestStatus::TestCompletedPassed,
                "TestCompletedFailed" => OverallTestStatus::TestCompletedFailed,
                "Retesting" => OverallTestStatus::Retesting,
                _ => OverallTestStatus::default(),
            },
            current_step_details: model.current_step_details.clone(),
            error_message: model.error_message.clone(),
            creation_time: model.creation_time,
            start_time: model.start_time,
            last_updated_time: model.last_updated_time,
            final_test_time: model.final_test_time,
            total_test_duration_ms: model.total_test_duration_ms,
            sub_test_results,
            hardpoint_readings,
            manual_test_current_value_input: model.manual_test_current_value_input.clone(),
            manual_test_current_value_output: model.manual_test_current_value_output.clone(),
            current_operator: model.current_operator.clone(),
            retries_count: model.retries_count,
            transient_data,
            test_plc_channel_tag: model.test_plc_channel_tag.clone(),
            test_plc_communication_address: model.test_plc_communication_address.clone(),
        }
    }
} 