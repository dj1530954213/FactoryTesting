// 文件: FactoryTesting/src-tauri/src/models/entities/channel_point_definition.rs
// 详细注释：ChannelPointDefinition实体的SeaORM定义
// 完全匹配点表结构，包含所有字段

use sea_orm::entity::prelude::*;
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
    #[sea_orm(nullable)]
    pub channel_position: Option<String>,
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
