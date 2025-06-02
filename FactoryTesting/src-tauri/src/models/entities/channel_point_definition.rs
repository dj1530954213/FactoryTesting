// 文件: FactoryTesting/src-tauri/src/models/entities/channel_point_definition.rs
// 详细注释：ChannelPointDefinition实体的SeaORM定义
// 完全匹配点表结构，包含所有字段

use sea_orm::entity::prelude::*;
use sea_orm::ActiveValue::Set;
use serde::{Deserialize, Serialize};
use crate::models::structs::default_id; // 确保可以访问 default_id

/// 通道点位定义实体 - 完全匹配点表结构
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "channel_point_definitions")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    #[serde(default = "default_id")]
    pub id: String,

    // 核心字段
    pub tag: String,
    pub variable_name: String,
    pub module_type: String,
    #[sea_orm(nullable)]
    pub plc_absolute_address: Option<String>,
    pub plc_communication_address: String,
    pub power_supply_type: i32,
    #[sea_orm(nullable)]
    pub description: Option<String>,

    // 基本字段（匹配点表）
    #[sea_orm(nullable)]
    pub sequence_number: Option<i32>,
    #[sea_orm(nullable)]
    pub module_name: Option<String>,
    #[sea_orm(nullable)]
    pub power_type_description: Option<String>,
    #[sea_orm(nullable)]
    pub wire_system: Option<String>,
    pub channel_tag_in_module: String,
    #[sea_orm(nullable)]
    pub station_name: Option<String>,
    #[sea_orm(nullable)]
    pub variable_description: Option<String>,
    #[sea_orm(nullable)]
    pub data_type: Option<String>,
    #[sea_orm(nullable)]
    pub read_write_property: Option<String>,
    #[sea_orm(nullable)]
    pub save_history: Option<String>,
    #[sea_orm(nullable)]
    pub power_off_protection: Option<String>,
    #[sea_orm(nullable)]
    pub range_low_limit: Option<f64>,
    #[sea_orm(nullable)]
    pub range_high_limit: Option<f64>,

    // SLL 报警设定
    #[sea_orm(nullable)]
    pub sll_set_point_value: Option<f64>,
    #[sea_orm(nullable)]
    pub sll_set_point_position: Option<String>,
    #[sea_orm(nullable)]
    pub sll_set_point_plc_address: Option<String>,
    #[sea_orm(nullable)]
    pub sll_set_point_communication_address: Option<String>,

    // SL 报警设定
    #[sea_orm(nullable)]
    pub sl_set_point_value: Option<f64>,
    #[sea_orm(nullable)]
    pub sl_set_point_position: Option<String>,
    #[sea_orm(nullable)]
    pub sl_set_point_plc_address: Option<String>,
    #[sea_orm(nullable)]
    pub sl_set_point_communication_address: Option<String>,

    // SH 报警设定
    #[sea_orm(nullable)]
    pub sh_set_point_value: Option<f64>,
    #[sea_orm(nullable)]
    pub sh_set_point_position: Option<String>,
    #[sea_orm(nullable)]
    pub sh_set_point_plc_address: Option<String>,
    #[sea_orm(nullable)]
    pub sh_set_point_communication_address: Option<String>,

    // SHH 报警设定
    #[sea_orm(nullable)]
    pub shh_set_point_value: Option<f64>,
    #[sea_orm(nullable)]
    pub shh_set_point_position: Option<String>,
    #[sea_orm(nullable)]
    pub shh_set_point_plc_address: Option<String>,
    #[sea_orm(nullable)]
    pub shh_set_point_communication_address: Option<String>,

    // LL/L/H/HH 报警反馈
    #[sea_orm(nullable)]
    pub ll_alarm_feedback: Option<String>,
    #[sea_orm(nullable)]
    pub ll_alarm_plc_address: Option<String>,
    #[sea_orm(nullable)]
    pub ll_alarm_communication_address: Option<String>,
    #[sea_orm(nullable)]
    pub l_alarm_feedback: Option<String>,
    #[sea_orm(nullable)]
    pub l_alarm_plc_address: Option<String>,
    #[sea_orm(nullable)]
    pub l_alarm_communication_address: Option<String>,
    #[sea_orm(nullable)]
    pub h_alarm_feedback: Option<String>,
    #[sea_orm(nullable)]
    pub h_alarm_plc_address: Option<String>,
    #[sea_orm(nullable)]
    pub h_alarm_communication_address: Option<String>,
    #[sea_orm(nullable)]
    pub hh_alarm_feedback: Option<String>,
    #[sea_orm(nullable)]
    pub hh_alarm_plc_address: Option<String>,
    #[sea_orm(nullable)]
    pub hh_alarm_communication_address: Option<String>,

    // 维护相关
    #[sea_orm(nullable)]
    pub maintenance_value_setting: Option<String>,
    #[sea_orm(nullable)]
    pub maintenance_value_position: Option<String>,
    #[sea_orm(nullable)]
    pub maintenance_value_plc_address: Option<String>,
    #[sea_orm(nullable)]
    pub maintenance_value_communication_address: Option<String>,
    #[sea_orm(nullable)]
    pub maintenance_enable_position: Option<String>,
    #[sea_orm(nullable)]
    pub maintenance_enable_plc_address: Option<String>,
    #[sea_orm(nullable)]
    pub maintenance_enable_communication_address: Option<String>,

    // 时间戳
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

// 从ChannelPointDefinition结构体转换为SeaORM ActiveModel
impl From<&crate::models::structs::ChannelPointDefinition> for ActiveModel {
    fn from(definition: &crate::models::structs::ChannelPointDefinition) -> Self {
        use chrono::Utc;

        let now = Utc::now().to_rfc3339();

        Self {
            id: Set(definition.id.clone()),
            tag: Set(definition.tag.clone()),
            variable_name: Set(definition.variable_name.clone()),
            module_type: Set(definition.module_type.to_string()),
            plc_absolute_address: Set(definition.plc_absolute_address.clone()),
            plc_communication_address: Set(definition.plc_communication_address.clone()),
            power_supply_type: Set(match definition.power_supply_type.as_str() {
                "有源" => 1,
                "无源" => 0,
                _ => 1, // 默认有源
            }),
            description: Set(Some(definition.variable_description.clone())),

            // 基本字段
            sequence_number: Set(None),
            module_name: Set(Some(definition.module_name.clone())),
            power_type_description: Set(Some(definition.power_supply_type.clone())),
            wire_system: Set(Some(definition.wire_system.clone())),
            channel_tag_in_module: Set(definition.channel_tag_in_module.clone()),
            station_name: Set(Some(definition.station_name.clone())),
            variable_description: Set(Some(definition.variable_description.clone())),
            data_type: Set(Some(definition.data_type.to_string())),
            read_write_property: Set(definition.access_property.clone()),
            save_history: Set(definition.save_history.map(|b| if b { "是".to_string() } else { "否".to_string() })),
            power_off_protection: Set(definition.power_failure_protection.map(|b| if b { "是".to_string() } else { "否".to_string() })),
            range_low_limit: Set(definition.range_lower_limit.map(|f| f as f64)),
            range_high_limit: Set(definition.range_upper_limit.map(|f| f as f64)),

            // SLL 报警设定
            sll_set_point_value: Set(definition.sll_set_value.map(|f| f as f64)),
            sll_set_point_position: Set(definition.sll_set_point_address.clone()),
            sll_set_point_plc_address: Set(definition.sll_set_point_plc_address.clone()),
            sll_set_point_communication_address: Set(definition.sll_set_point_communication_address.clone()),

            // SL 报警设定
            sl_set_point_value: Set(definition.sl_set_value.map(|f| f as f64)),
            sl_set_point_position: Set(definition.sl_set_point_address.clone()),
            sl_set_point_plc_address: Set(definition.sl_set_point_plc_address.clone()),
            sl_set_point_communication_address: Set(definition.sl_set_point_communication_address.clone()),

            // SH 报警设定
            sh_set_point_value: Set(definition.sh_set_value.map(|f| f as f64)),
            sh_set_point_position: Set(definition.sh_set_point_address.clone()),
            sh_set_point_plc_address: Set(definition.sh_set_point_plc_address.clone()),
            sh_set_point_communication_address: Set(definition.sh_set_point_communication_address.clone()),

            // SHH 报警设定
            shh_set_point_value: Set(definition.shh_set_value.map(|f| f as f64)),
            shh_set_point_position: Set(definition.shh_set_point_address.clone()),
            shh_set_point_plc_address: Set(definition.shh_set_point_plc_address.clone()),
            shh_set_point_communication_address: Set(definition.shh_set_point_communication_address.clone()),

            // LL/L/H/HH 报警反馈
            ll_alarm_feedback: Set(definition.sll_feedback_address.clone()),
            ll_alarm_plc_address: Set(definition.sll_feedback_plc_address.clone()),
            ll_alarm_communication_address: Set(definition.sll_feedback_communication_address.clone()),
            l_alarm_feedback: Set(definition.sl_feedback_address.clone()),
            l_alarm_plc_address: Set(definition.sl_feedback_plc_address.clone()),
            l_alarm_communication_address: Set(definition.sl_feedback_communication_address.clone()),
            h_alarm_feedback: Set(definition.sh_feedback_address.clone()),
            h_alarm_plc_address: Set(definition.sh_feedback_plc_address.clone()),
            h_alarm_communication_address: Set(definition.sh_feedback_communication_address.clone()),
            hh_alarm_feedback: Set(definition.shh_feedback_address.clone()),
            hh_alarm_plc_address: Set(definition.shh_feedback_plc_address.clone()),
            hh_alarm_communication_address: Set(definition.shh_feedback_communication_address.clone()),

            // 维护相关
            maintenance_value_setting: Set(None),
            maintenance_value_position: Set(definition.maintenance_value_set_point_address.clone()),
            maintenance_value_plc_address: Set(definition.maintenance_value_set_point_plc_address.clone()),
            maintenance_value_communication_address: Set(definition.maintenance_value_set_point_communication_address.clone()),
            maintenance_enable_position: Set(definition.maintenance_enable_switch_point_address.clone()),
            maintenance_enable_plc_address: Set(definition.maintenance_enable_switch_point_plc_address.clone()),
            maintenance_enable_communication_address: Set(definition.maintenance_enable_switch_point_communication_address.clone()),

            // 时间戳
            created_at: Set(now.clone()),
            updated_at: Set(now),
        }
    }
}

// 从SeaORM Model转换回ChannelPointDefinition结构体
impl From<&Model> for crate::models::structs::ChannelPointDefinition {
    fn from(model: &Model) -> Self {
        use crate::models::enums::{ModuleType, PointDataType};

        crate::models::structs::ChannelPointDefinition {
            id: model.id.clone(),
            tag: model.tag.clone(),
            variable_name: model.variable_name.clone(),
            variable_description: model.variable_description.clone().unwrap_or_default(),
            station_name: model.station_name.clone().unwrap_or_default(),
            module_name: model.module_name.clone().unwrap_or_default(),
            module_type: model.module_type.parse().unwrap_or(ModuleType::AI),
            channel_tag_in_module: model.channel_tag_in_module.clone(),
            data_type: model.data_type.clone().unwrap_or_default().parse().unwrap_or(PointDataType::Float),
            power_supply_type: match model.power_supply_type {
                1 => "有源".to_string(),
                0 => "无源".to_string(),
                _ => "有源".to_string(),
            },
            wire_system: model.wire_system.clone().unwrap_or_default(),
            plc_absolute_address: model.plc_absolute_address.clone(),
            plc_communication_address: model.plc_communication_address.clone(),
            range_lower_limit: model.range_low_limit.map(|v| v as f32),
            range_upper_limit: model.range_high_limit.map(|v| v as f32),
            engineering_unit: None,

            // SLL 报警设定
            sll_set_value: model.sll_set_point_value.map(|v| v as f32),
            sll_set_point_address: model.sll_set_point_position.clone(),
            sll_set_point_plc_address: model.sll_set_point_plc_address.clone(),
            sll_set_point_communication_address: model.sll_set_point_communication_address.clone(),

            // SL 报警设定
            sl_set_value: model.sl_set_point_value.map(|v| v as f32),
            sl_set_point_address: model.sl_set_point_position.clone(),
            sl_set_point_plc_address: model.sl_set_point_plc_address.clone(),
            sl_set_point_communication_address: model.sl_set_point_communication_address.clone(),

            // SH 报警设定
            sh_set_value: model.sh_set_point_value.map(|v| v as f32),
            sh_set_point_address: model.sh_set_point_position.clone(),
            sh_set_point_plc_address: model.sh_set_point_plc_address.clone(),
            sh_set_point_communication_address: model.sh_set_point_communication_address.clone(),

            // SHH 报警设定
            shh_set_value: model.shh_set_point_value.map(|v| v as f32),
            shh_set_point_address: model.shh_set_point_position.clone(),
            shh_set_point_plc_address: model.shh_set_point_plc_address.clone(),
            shh_set_point_communication_address: model.shh_set_point_communication_address.clone(),

            // 报警反馈
            sll_feedback_address: model.ll_alarm_feedback.clone(),
            sll_feedback_plc_address: model.ll_alarm_plc_address.clone(),
            sll_feedback_communication_address: model.ll_alarm_communication_address.clone(),
            sl_feedback_address: model.l_alarm_feedback.clone(),
            sl_feedback_plc_address: model.l_alarm_plc_address.clone(),
            sl_feedback_communication_address: model.l_alarm_communication_address.clone(),
            sh_feedback_address: model.h_alarm_feedback.clone(),
            sh_feedback_plc_address: model.h_alarm_plc_address.clone(),
            sh_feedback_communication_address: model.h_alarm_communication_address.clone(),
            shh_feedback_address: model.hh_alarm_feedback.clone(),
            shh_feedback_plc_address: model.hh_alarm_plc_address.clone(),
            shh_feedback_communication_address: model.hh_alarm_communication_address.clone(),

            // 维护相关
            maintenance_value_set_point_address: model.maintenance_value_position.clone(),
            maintenance_value_set_point_plc_address: model.maintenance_value_plc_address.clone(),
            maintenance_value_set_point_communication_address: model.maintenance_value_communication_address.clone(),
            maintenance_enable_switch_point_address: model.maintenance_enable_position.clone(),
            maintenance_enable_switch_point_plc_address: model.maintenance_enable_plc_address.clone(),
            maintenance_enable_switch_point_communication_address: model.maintenance_enable_communication_address.clone(),

            // 其他字段
            access_property: model.read_write_property.clone(),
            save_history: model.save_history.clone().map(|s| s == "是"),
            power_failure_protection: model.power_off_protection.clone().map(|s| s == "是"),
            test_rig_plc_address: None,
        }
    }
}