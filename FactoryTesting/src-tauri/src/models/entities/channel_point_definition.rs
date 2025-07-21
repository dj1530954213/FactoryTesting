//! # 通道点位定义实体 (Channel Point Definition Entity)
//!
//! ## 业务说明
//! 通道点位定义实体对应数据库中的channel_point_definitions表
//! 存储工厂测试系统中所有测试点位的完整配置信息，是系统的核心数据结构
//!
//! ## 数据结构特点
//! - **53个字段**: 完整描述一个测试点位的所有属性
//! - **层次化组织**: 按功能分为基础信息、量程设定、报警配置等
//! - **可选字段**: 大部分字段为可选，适应不同类型通道的需求
//! - **类型兼容**: 支持AI/AO/DI/DO四种通道类型
//!
//! ## 字段分类
//! ### 基础信息字段（14个）
//! - 序号、模块信息、通道位置、变量名称等基本属性
//! - 供电类型、线制、数据类型等技术参数
//!
//! ### 量程字段（2个）
//! - 量程上下限，用于模拟量通道的工程量转换
//!
//! ### 报警设定字段（20个）
//! - SLL/SL/SH/SHH四级报警的设定值和地址配置
//! - 每级报警包含设定值、设定点位、PLC地址、通讯地址等
//!
//! ### PLC通信字段（12个）
//! - PLC绝对地址、通讯地址等通信相关配置
//! - 支持多种PLC协议和地址格式
//!
//! ## 使用场景
//! - **数据导入**: 从Excel文件导入点位配置
//! - **测试执行**: 为测试实例提供配置数据
//! - **报告生成**: 测试报告中的点位信息来源
//! - **配置管理**: 系统配置的持久化存储
//!
//! ## Rust知识点
//! - **SeaORM实体**: 使用DeriveEntityModel宏自动生成实体代码
//! - **可选字段**: 大量使用Option<T>表示数据库中的nullable字段
//! - **序列化**: 支持与前端的JSON数据交换

use sea_orm::entity::prelude::*;
use sea_orm::ActiveValue::Set;
use serde::{Deserialize, Serialize};
use crate::models::structs::default_id; // 确保可以访问 default_id

/// 通道点位定义实体 - 完全匹配点表结构（53个字段）
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "channel_point_definitions")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    #[serde(default = "default_id")]
    pub id: String,

    // === 批次关联字段 ===
    #[sea_orm(nullable)]
    pub batch_id: Option<String>,                        // 批次ID

    // === 基础信息字段（14个）===
    #[sea_orm(nullable)]
    pub sequence_number: Option<i32>,                    // 序号
    #[sea_orm(nullable)]
    pub module_name: Option<String>,                     // 模块名称
    pub module_type: String,                             // 模块类型 (AI/AO/DI/DO)
    pub power_supply_type: String,                       // 供电类型（有源/无源）
    #[sea_orm(nullable)]
    pub wire_system: Option<String>,                     // 线制
    pub channel_position: String,                        // 通道位号
    pub tag: String,                                     // 位号
    #[sea_orm(nullable)]
    pub station_name: Option<String>,                    // 场站名
    pub variable_name: String,                           // 变量名称（HMI）
    #[sea_orm(nullable)]
    pub variable_description: Option<String>,            // 变量描述
    #[sea_orm(nullable)]
    pub data_type: Option<String>,                       // 数据类型
    #[sea_orm(nullable)]
    pub read_write_property: Option<String>,             // 读写属性
    #[sea_orm(nullable)]
    pub save_history: Option<String>,                    // 保存历史
    #[sea_orm(nullable)]
    pub power_off_protection: Option<String>,            // 掉电保护

    // === 量程字段（2个）===
    #[sea_orm(nullable)]
    pub range_low_limit: Option<f64>,                    // 量程低限
    #[sea_orm(nullable)]
    pub range_high_limit: Option<f64>,                   // 量程高限

    // === SLL设定字段（4个）===
    #[sea_orm(nullable)]
    pub sll_set_value: Option<f64>,                      // SLL设定值
    #[sea_orm(nullable)]
    pub sll_set_point: Option<String>,                   // SLL设定点位
    #[sea_orm(nullable)]
    pub sll_set_point_plc_address: Option<String>,       // SLL设定点位_PLC地址
    #[sea_orm(nullable)]
    pub sll_set_point_communication_address: Option<String>, // SLL设定点位_通讯地址

    // === SL设定字段（4个）===
    #[sea_orm(nullable)]
    pub sl_set_value: Option<f64>,                       // SL设定值
    #[sea_orm(nullable)]
    pub sl_set_point: Option<String>,                    // SL设定点位
    #[sea_orm(nullable)]
    pub sl_set_point_plc_address: Option<String>,        // SL设定点位_PLC地址
    #[sea_orm(nullable)]
    pub sl_set_point_communication_address: Option<String>, // SL设定点位_通讯地址

    // === SH设定字段（4个）===
    #[sea_orm(nullable)]
    pub sh_set_value: Option<f64>,                       // SH设定值
    #[sea_orm(nullable)]
    pub sh_set_point: Option<String>,                    // SH设定点位
    #[sea_orm(nullable)]
    pub sh_set_point_plc_address: Option<String>,        // SH设定点位_PLC地址
    #[sea_orm(nullable)]
    pub sh_set_point_communication_address: Option<String>, // SH设定点位_通讯地址

    // === SHH设定字段（4个）===
    #[sea_orm(nullable)]
    pub shh_set_value: Option<f64>,                      // SHH设定值
    #[sea_orm(nullable)]
    pub shh_set_point: Option<String>,                   // SHH设定点位
    #[sea_orm(nullable)]
    pub shh_set_point_plc_address: Option<String>,       // SHH设定点位_PLC地址
    #[sea_orm(nullable)]
    pub shh_set_point_communication_address: Option<String>, // SHH设定点位_通讯地址

    // === LL报警字段（3个）===
    #[sea_orm(nullable)]
    pub ll_alarm: Option<String>,                        // LL报警
    #[sea_orm(nullable)]
    pub ll_alarm_plc_address: Option<String>,            // LL报警_PLC地址
    #[sea_orm(nullable)]
    pub ll_alarm_communication_address: Option<String>, // LL报警_通讯地址

    // === L报警字段（3个）===
    #[sea_orm(nullable)]
    pub l_alarm: Option<String>,                         // L报警
    #[sea_orm(nullable)]
    pub l_alarm_plc_address: Option<String>,             // L报警_PLC地址
    #[sea_orm(nullable)]
    pub l_alarm_communication_address: Option<String>,  // L报警_通讯地址

    // === H报警字段（3个）===
    #[sea_orm(nullable)]
    pub h_alarm: Option<String>,                         // H报警
    #[sea_orm(nullable)]
    pub h_alarm_plc_address: Option<String>,             // H报警_PLC地址
    #[sea_orm(nullable)]
    pub h_alarm_communication_address: Option<String>,  // H报警_通讯地址

    // === HH报警字段（3个）===
    #[sea_orm(nullable)]
    pub hh_alarm: Option<String>,                        // HH报警
    #[sea_orm(nullable)]
    pub hh_alarm_plc_address: Option<String>,            // HH报警_PLC地址
    #[sea_orm(nullable)]
    pub hh_alarm_communication_address: Option<String>, // HH报警_通讯地址

    // === 维护字段（6个）===
    #[sea_orm(nullable)]
    pub maintenance_value_setting: Option<String>,       // 维护值设定
    #[sea_orm(nullable)]
    pub maintenance_value_set_point: Option<String>,     // 维护值设定点位
    #[sea_orm(nullable)]
    pub maintenance_value_set_point_plc_address: Option<String>, // 维护值设定点位_PLC地址
    #[sea_orm(nullable)]
    pub maintenance_value_set_point_communication_address: Option<String>, // 维护值设定点位_通讯地址
    #[sea_orm(nullable)]
    pub maintenance_enable_switch_point: Option<String>, // 维护使能开关点位
    #[sea_orm(nullable)]
    pub maintenance_enable_switch_point_plc_address: Option<String>, // 维护使能开关点位_PLC地址
    #[sea_orm(nullable)]
    pub maintenance_enable_switch_point_communication_address: Option<String>, // 维护使能开关点位_通讯地址

    // === 地址字段（2个）===
    #[sea_orm(nullable)]
    pub plc_absolute_address: Option<String>,            // PLC绝对地址
    pub plc_communication_address: String,               // 上位机通讯地址

    // === 时间戳字段（2个）===
    pub created_time: String,
    pub updated_time: String,
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
            batch_id: Set(definition.batch_id.clone()),
            sequence_number: Set(definition.sequence_number.map(|n| n as i32)),
            tag: Set(definition.tag.clone()),
            module_name: Set(Some(definition.module_name.clone())),
            module_type: Set(definition.module_type.to_string()),
            power_supply_type: Set(definition.power_supply_type.clone()),
            wire_system: Set(Some(definition.wire_system.clone())),
            channel_position: Set(definition.channel_tag_in_module.clone()),
            station_name: Set(Some(definition.station_name.clone())),
            variable_name: Set(definition.variable_name.clone()),
            variable_description: Set(Some(definition.variable_description.clone())),
            data_type: Set(Some(definition.data_type.to_string())),
            read_write_property: Set(definition.access_property.clone()),
            save_history: Set(definition.save_history.map(|b| if b { "是".to_string() } else { "否".to_string() })),
            power_off_protection: Set(definition.power_failure_protection.map(|b| if b { "是".to_string() } else { "否".to_string() })),

            // === 量程字段 ===
            range_low_limit: Set(definition.range_low_limit.map(|f| f as f64)),
            range_high_limit: Set(definition.range_high_limit.map(|f| f as f64)),

            // === SLL设定字段 ===
            sll_set_value: Set(definition.sll_set_value.map(|f| f as f64)),
            sll_set_point: Set(definition.sll_set_point_address.clone()),
            sll_set_point_plc_address: Set(definition.sll_set_point_plc_address.clone()),
            sll_set_point_communication_address: Set(definition.sll_set_point_communication_address.clone()),

            // === SL设定字段 ===
            sl_set_value: Set(definition.sl_set_value.map(|f| f as f64)),
            sl_set_point: Set(definition.sl_set_point_address.clone()),
            sl_set_point_plc_address: Set(definition.sl_set_point_plc_address.clone()),
            sl_set_point_communication_address: Set(definition.sl_set_point_communication_address.clone()),

            // === SH设定字段 ===
            sh_set_value: Set(definition.sh_set_value.map(|f| f as f64)),
            sh_set_point: Set(definition.sh_set_point_address.clone()),
            sh_set_point_plc_address: Set(definition.sh_set_point_plc_address.clone()),
            sh_set_point_communication_address: Set(definition.sh_set_point_communication_address.clone()),

            // === SHH设定字段 ===
            shh_set_value: Set(definition.shh_set_value.map(|f| f as f64)),
            shh_set_point: Set(definition.shh_set_point_address.clone()),
            shh_set_point_plc_address: Set(definition.shh_set_point_plc_address.clone()),
            shh_set_point_communication_address: Set(definition.shh_set_point_communication_address.clone()),

            // === 报警字段 ===
            ll_alarm: Set(definition.sll_feedback_address.clone()),
            ll_alarm_plc_address: Set(definition.sll_feedback_plc_address.clone()),
            ll_alarm_communication_address: Set(definition.sll_feedback_communication_address.clone()),
            l_alarm: Set(definition.sl_feedback_address.clone()),
            l_alarm_plc_address: Set(definition.sl_feedback_plc_address.clone()),
            l_alarm_communication_address: Set(definition.sl_feedback_communication_address.clone()),
            h_alarm: Set(definition.sh_feedback_address.clone()),
            h_alarm_plc_address: Set(definition.sh_feedback_plc_address.clone()),
            h_alarm_communication_address: Set(definition.sh_feedback_communication_address.clone()),
            hh_alarm: Set(definition.shh_feedback_address.clone()),
            hh_alarm_plc_address: Set(definition.shh_feedback_plc_address.clone()),
            hh_alarm_communication_address: Set(definition.shh_feedback_communication_address.clone()),

            // === 维护字段 ===
            maintenance_value_setting: Set(None),
            maintenance_value_set_point: Set(definition.maintenance_value_set_point_address.clone()),
            maintenance_value_set_point_plc_address: Set(definition.maintenance_value_set_point_plc_address.clone()),
            maintenance_value_set_point_communication_address: Set(definition.maintenance_value_set_point_communication_address.clone()),
            maintenance_enable_switch_point: Set(definition.maintenance_enable_switch_point_address.clone()),
            maintenance_enable_switch_point_plc_address: Set(definition.maintenance_enable_switch_point_plc_address.clone()),
            maintenance_enable_switch_point_communication_address: Set(definition.maintenance_enable_switch_point_communication_address.clone()),

            // === 地址字段 ===
            plc_absolute_address: Set(definition.plc_absolute_address.clone()),
            plc_communication_address: Set(definition.plc_communication_address.clone()),

            // === 时间戳字段 ===
            created_time: Set(now.clone()),
            updated_time: Set(now),
        }
    }
}

// 从SeaORM Model转换回ChannelPointDefinition结构体
impl From<&Model> for crate::models::structs::ChannelPointDefinition {
    fn from(model: &Model) -> Self {
        use crate::models::enums::{ModuleType, PointDataType};

        crate::models::structs::ChannelPointDefinition {
            id: model.id.clone(),
            batch_id: model.batch_id.clone(),
            sequence_number: model.sequence_number.map(|n| n as u32),
            tag: model.tag.clone(),
            variable_name: model.variable_name.clone(),
            variable_description: model.variable_description.clone().unwrap_or_default(),
            station_name: model.station_name.clone().unwrap_or_default(),
            module_name: model.module_name.clone().unwrap_or_default(),
            module_type: model.module_type.parse().unwrap_or(ModuleType::AI),
            channel_tag_in_module: model.channel_position.clone(),
            data_type: model.data_type.clone().unwrap_or_default().parse().unwrap_or(PointDataType::Float),
            power_supply_type: model.power_supply_type.clone(),
            wire_system: model.wire_system.clone().unwrap_or_default(),
            plc_absolute_address: model.plc_absolute_address.clone(),
            plc_communication_address: model.plc_communication_address.clone(),
            range_low_limit: model.range_low_limit.map(|v| v as f32),
            range_high_limit: model.range_high_limit.map(|v| v as f32),
            engineering_unit: None,

            // SLL 报警设定
            sll_set_value: model.sll_set_value.map(|v| v as f32),
            sll_set_point_address: model.sll_set_point.clone(),
            sll_set_point_plc_address: model.sll_set_point_plc_address.clone(),
            sll_set_point_communication_address: model.sll_set_point_communication_address.clone(),

            // SL 报警设定
            sl_set_value: model.sl_set_value.map(|v| v as f32),
            sl_set_point_address: model.sl_set_point.clone(),
            sl_set_point_plc_address: model.sl_set_point_plc_address.clone(),
            sl_set_point_communication_address: model.sl_set_point_communication_address.clone(),

            // SH 报警设定
            sh_set_value: model.sh_set_value.map(|v| v as f32),
            sh_set_point_address: model.sh_set_point.clone(),
            sh_set_point_plc_address: model.sh_set_point_plc_address.clone(),
            sh_set_point_communication_address: model.sh_set_point_communication_address.clone(),

            // SHH 报警设定
            shh_set_value: model.shh_set_value.map(|v| v as f32),
            shh_set_point_address: model.shh_set_point.clone(),
            shh_set_point_plc_address: model.shh_set_point_plc_address.clone(),
            shh_set_point_communication_address: model.shh_set_point_communication_address.clone(),

            // 报警反馈
            sll_feedback_address: model.ll_alarm.clone(),
            sll_feedback_plc_address: model.ll_alarm_plc_address.clone(),
            sll_feedback_communication_address: model.ll_alarm_communication_address.clone(),
            sl_feedback_address: model.l_alarm.clone(),
            sl_feedback_plc_address: model.l_alarm_plc_address.clone(),
            sl_feedback_communication_address: model.l_alarm_communication_address.clone(),
            sh_feedback_address: model.h_alarm.clone(),
            sh_feedback_plc_address: model.h_alarm_plc_address.clone(),
            sh_feedback_communication_address: model.h_alarm_communication_address.clone(),
            shh_feedback_address: model.hh_alarm.clone(),
            shh_feedback_plc_address: model.hh_alarm_plc_address.clone(),
            shh_feedback_communication_address: model.hh_alarm_communication_address.clone(),

            // 维护相关
            maintenance_value_set_point_address: model.maintenance_value_set_point.clone(),
            maintenance_value_set_point_plc_address: model.maintenance_value_set_point_plc_address.clone(),
            maintenance_value_set_point_communication_address: model.maintenance_value_set_point_communication_address.clone(),
            maintenance_enable_switch_point_address: model.maintenance_enable_switch_point.clone(),
            maintenance_enable_switch_point_plc_address: model.maintenance_enable_switch_point_plc_address.clone(),
            maintenance_enable_switch_point_communication_address: model.maintenance_enable_switch_point_communication_address.clone(),

            // 其他字段
            access_property: model.read_write_property.clone(),
            save_history: model.save_history.clone().map(|s| s == "是"),
            power_failure_protection: model.power_off_protection.clone().map(|s| s == "是"),

            // 不再自动生成虚拟测试台架地址，将通过通道分配时从测试PLC配置表获取真实地址
            test_rig_plc_address: None,
        }
    }
}
