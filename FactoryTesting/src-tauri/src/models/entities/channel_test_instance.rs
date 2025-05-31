// 文件: FactoryTesting/src-tauri/src/models/entities/channel_test_instance.rs
// 详细注释：ChannelTestInstance实体的SeaORM定义
// 基于原C#项目数据库结构重构

use sea_orm::entity::prelude::*;
use sea_orm::ActiveValue::Set;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use crate::models::structs::{default_id, SubTestExecutionResult, AnalogReadingPoint}; // 引入所需结构体
use crate::models::enums::{OverallTestStatus, SubTestItem}; // 引入所需枚举

/// 通道测试实例实体
///
/// 基于原C#项目的ChannelMappings表结构设计
/// 包含了完整的测试状态和结果信息
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "channel_test_instances")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    #[serde(default = "default_id")]
    pub instance_id: String,

    // 关联信息
    pub definition_id: String,              // 关联到 ChannelPointDefinition 的 ID
    pub test_batch_id: String,              // 关联到 TestBatchInfo 的 ID
    pub test_batch_name: String,            // 测试批次名称

    // 基础信息（从ChannelPointDefinition复制过来便于查询）
    pub channel_tag: String,                // 通道标识，如 "1_2_AI_0"
    pub variable_name: String,              // 变量名，如 "PT_2101"
    pub variable_description: String,       // 变量描述，如 "计量撬进口压力"
    pub module_type: String,                // AI/AO/DI/DO
    pub data_type: String,                  // REAL/BOOL/INT
    pub plc_communication_address: String,  // 通信地址，如 "40001"

    // 测试状态
    #[sea_orm(column_type = "Text")]
    pub overall_status: String,             // 整体测试状态
    #[sea_orm(nullable)]
    pub current_step_details: Option<String>, // 当前步骤详情
    #[sea_orm(nullable)]
    pub error_message: Option<String>,      // 错误消息

    // 时间信息
    pub created_time: DateTime<Utc>,        // 创建时间
    #[sea_orm(nullable)]
    pub start_time: Option<DateTime<Utc>>,  // 开始测试时间
    pub updated_time: DateTime<Utc>,        // 最后更新时间
    #[sea_orm(nullable)]
    pub final_test_time: Option<DateTime<Utc>>, // 测试完成时间
    #[sea_orm(nullable)]
    pub total_test_duration_ms: Option<i64>, // 总测试时长（毫秒）

    // 测试结果（基于原C#项目的字段结构）
    #[sea_orm(nullable)]
    pub hard_point_status: Option<i32>,     // 硬点测试状态
    #[sea_orm(nullable)]
    pub hard_point_test_result: Option<String>, // 硬点测试结果
    #[sea_orm(nullable)]
    pub hard_point_error_detail: Option<String>, // 硬点测试错误详情
    #[sea_orm(nullable)]
    pub actual_value: Option<String>,       // 实际值
    #[sea_orm(nullable)]
    pub expected_value: Option<String>,     // 期望值
    #[sea_orm(nullable)]
    pub current_value: Option<String>,      // 当前值

    // 报警测试状态（AI专用）
    #[sea_orm(nullable)]
    pub low_low_alarm_status: Option<i32>,  // 低低报警状态
    #[sea_orm(nullable)]
    pub low_alarm_status: Option<i32>,      // 低报警状态
    #[sea_orm(nullable)]
    pub high_alarm_status: Option<i32>,     // 高报警状态
    #[sea_orm(nullable)]
    pub high_high_alarm_status: Option<i32>, // 高高报警状态

    // 功能测试状态
    #[sea_orm(nullable)]
    pub maintenance_function: Option<i32>,   // 维护功能状态
    #[sea_orm(nullable)]
    pub trend_check: Option<i32>,           // 趋势检查状态
    #[sea_orm(nullable)]
    pub report_check: Option<i32>,          // 报表检查状态
    #[sea_orm(nullable)]
    pub show_value_status: Option<i32>,     // 显示值状态

    // 分配的测试PLC通道信息
    #[sea_orm(nullable)]
    pub test_plc_channel_tag: Option<String>, // 测试PLC通道标识
    #[sea_orm(nullable)]
    pub test_plc_communication_address: Option<String>, // 测试PLC通信地址
    #[sea_orm(nullable)]
    pub test_result_status: Option<i32>,     // 测试结果状态

    // 操作信息
    #[sea_orm(nullable)]
    pub current_operator: Option<String>,   // 当前操作员
    pub retries_count: u32,                 // 重试次数

    // 复杂数据结构（JSON存储）
    #[sea_orm(column_type = "Text", nullable)]
    pub sub_test_results_json: Option<String>, // 子测试结果JSON
    #[sea_orm(column_type = "Text", nullable)]
    pub hardpoint_readings_json: Option<String>, // 硬点读数JSON
    #[sea_orm(column_type = "Text", nullable)]
    pub transient_data_json: Option<String>, // 临时数据JSON
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    // 关联关系暂时注释，等其他实体完善后再启用
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

impl ActiveModelBehavior for ActiveModel {
    fn new() -> Self {
        Self {
            instance_id: Set(default_id()),
            created_time: Set(Utc::now()),
            updated_time: Set(Utc::now()),
            ..ActiveModelTrait::default()
        }
    }

    fn before_save<'life0, 'async_trait, C>(
        mut self,
        _db: &'life0 C,
        _insert: bool,
    ) -> core::pin::Pin<Box<dyn core::future::Future<Output = Result<Self, DbErr>> + core::marker::Send + 'async_trait>>
    where
        'life0: 'async_trait,
        C: 'async_trait + ConnectionTrait,
        Self: 'async_trait,
    {
        Box::pin(async move {
            self.updated_time = Set(Utc::now());
            Ok(self)
        })
    }
}

impl From<&crate::models::structs::ChannelTestInstance> for ActiveModel {
    fn from(original: &crate::models::structs::ChannelTestInstance) -> Self {
        let sub_test_results_json = serde_json::to_string(&original.sub_test_results).ok();
        let hardpoint_readings_json = serde_json::to_string(&original.hardpoint_readings).ok();
        let transient_data_json = serde_json::to_string(&original.transient_data).ok();
        let now = Utc::now();

        Self {
            instance_id: Set(original.instance_id.clone()),
            definition_id: Set(original.definition_id.clone()),
            test_batch_id: Set(original.test_batch_id.clone()),
            test_batch_name: Set(original.test_batch_name.clone()),

            // 基础信息（需要从definition中获取，这里先设置默认值）
            channel_tag: Set("".to_string()),
            variable_name: Set("".to_string()),
            variable_description: Set("".to_string()),
            module_type: Set("".to_string()),
            data_type: Set("".to_string()),
            plc_communication_address: Set("".to_string()),

            overall_status: Set(original.overall_status.to_string()),
            current_step_details: Set(original.current_step_details.clone()),
            error_message: Set(original.error_message.clone()),

            created_time: Set(original.creation_time),
            start_time: Set(original.start_time),
            updated_time: Set(original.last_updated_time),
            final_test_time: Set(original.final_test_time),
            total_test_duration_ms: Set(original.total_test_duration_ms),

            // 测试结果字段（从sub_test_results中提取）
            hard_point_status: Set(None),
            hard_point_test_result: Set(None),
            hard_point_error_detail: Set(None),
            actual_value: Set(None),
            expected_value: Set(None),
            current_value: Set(None),

            // 报警状态
            low_low_alarm_status: Set(None),
            low_alarm_status: Set(None),
            high_alarm_status: Set(None),
            high_high_alarm_status: Set(None),

            // 功能测试状态
            maintenance_function: Set(None),
            trend_check: Set(None),
            report_check: Set(None),
            show_value_status: Set(None),

            test_plc_channel_tag: Set(original.test_plc_channel_tag.clone()),
            test_plc_communication_address: Set(original.test_plc_communication_address.clone()),
            test_result_status: Set(None),

            current_operator: Set(original.current_operator.clone()),
            retries_count: Set(original.retries_count),

            sub_test_results_json: Set(sub_test_results_json),
            hardpoint_readings_json: Set(hardpoint_readings_json),
            transient_data_json: Set(transient_data_json),
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
            overall_status: model.overall_status.parse().unwrap_or_default(),
            current_step_details: model.current_step_details.clone(),
            error_message: model.error_message.clone(),
            creation_time: model.created_time,
            start_time: model.start_time,
            last_updated_time: model.updated_time,
            final_test_time: model.final_test_time,
            total_test_duration_ms: model.total_test_duration_ms,
            sub_test_results,
            hardpoint_readings,
            manual_test_current_value_input: None, // 新实体结构中没有这个字段
            manual_test_current_value_output: None, // 新实体结构中没有这个字段
            current_operator: model.current_operator.clone(),
            retries_count: model.retries_count,
            transient_data,
            test_plc_channel_tag: model.test_plc_channel_tag.clone(),
            test_plc_communication_address: model.test_plc_communication_address.clone(),
        }
    }
}

// 为 ChannelTestInstance 实体添加便利方法
impl Model {
    /// 创建新的测试实例
    pub fn new(
        definition_id: String,
        test_batch_id: String,
        test_batch_name: String,
        channel_tag: String,
        variable_name: String,
        variable_description: String,
        module_type: String,
        data_type: String,
        plc_communication_address: String,
    ) -> Self {
        let now = Utc::now();
        Self {
            instance_id: default_id(),
            definition_id,
            test_batch_id,
            test_batch_name,
            channel_tag,
            variable_name,
            variable_description,
            module_type,
            data_type,
            plc_communication_address,
            overall_status: OverallTestStatus::NotTested.to_string(),
            current_step_details: None,
            error_message: None,
            created_time: now,
            start_time: None,
            updated_time: now,
            final_test_time: None,
            total_test_duration_ms: None,
            hard_point_status: None,
            hard_point_test_result: None,
            hard_point_error_detail: None,
            actual_value: None,
            expected_value: None,
            current_value: None,
            low_low_alarm_status: None,
            low_alarm_status: None,
            high_alarm_status: None,
            high_high_alarm_status: None,
            maintenance_function: None,
            trend_check: None,
            report_check: None,
            show_value_status: None,
            test_plc_channel_tag: None,
            test_plc_communication_address: None,
            test_result_status: None,
            current_operator: None,
            retries_count: 0,
            sub_test_results_json: None,
            hardpoint_readings_json: None,
            transient_data_json: None,
        }
    }

    /// 获取整体测试状态枚举
    pub fn get_overall_status(&self) -> Result<OverallTestStatus, String> {
        self.overall_status.parse()
    }

    /// 判断是否为模拟量输入
    pub fn is_analog_input(&self) -> bool {
        self.module_type == "AI"
    }

    /// 判断是否为模拟量输出
    pub fn is_analog_output(&self) -> bool {
        self.module_type == "AO"
    }

    /// 判断是否为数字量输入
    pub fn is_digital_input(&self) -> bool {
        self.module_type == "DI"
    }

    /// 判断是否为数字量输出
    pub fn is_digital_output(&self) -> bool {
        self.module_type == "DO"
    }

    /// 判断测试是否完成
    pub fn is_test_completed(&self) -> bool {
        matches!(self.get_overall_status(),
            Ok(OverallTestStatus::TestCompletedPassed) |
            Ok(OverallTestStatus::TestCompletedFailed)
        )
    }

    /// 判断测试是否通过
    pub fn is_test_passed(&self) -> bool {
        matches!(self.get_overall_status(), Ok(OverallTestStatus::TestCompletedPassed))
    }

    /// 判断测试是否失败
    pub fn is_test_failed(&self) -> bool {
        matches!(self.get_overall_status(), Ok(OverallTestStatus::TestCompletedFailed))
    }
}