// 文件: FactoryTesting/src-tauri/src/models/entities/channel_point_definition.rs
// 详细注释：ChannelPointDefinition实体的SeaORM定义
// 基于原C#项目数据库结构和点表数据重构

use sea_orm::entity::prelude::*;
use sea_orm::ActiveValue::Set;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use crate::models::structs::default_id; // 确保可以访问 default_id
use crate::models::enums::{ModuleType, PointDataType}; // 引入枚举

/// 通道点位定义实体
///
/// 基于原C#项目的ChannelMappings表结构和测试IO.csv点表数据设计
/// 包含了完整的通道配置信息，支持AI/AO/DI/DO四种模块类型
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "channel_point_definitions")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    #[serde(default = "default_id")]
    pub id: String,

    // 基础标识信息
    pub tag: String,                        // 通道标识，如 "1_2_AI_0"
    pub variable_name: String,              // 变量名，如 "PT_2101"
    pub variable_description: String,       // 变量描述，如 "计量撬进口压力"
    pub station_name: String,               // 站点名称，如 "樟洋电厂"
    pub module_name: String,                // 模块名称，如 "8通道模拟量输入模块"

    // 模块类型和数据类型
    #[sea_orm(column_type = "Text")]
    pub module_type: String,                // AI/AO/DI/DO
    pub channel_tag_in_module: String,      // 模块内通道标识
    #[sea_orm(column_type = "Text")]
    pub data_type: String,                  // REAL/BOOL/INT

    // 供电和接线信息
    pub power_supply_type: String,          // 有源/无源
    pub wire_system: String,                // 二线制/四线制

    // PLC地址信息
    #[sea_orm(nullable)]
    pub plc_absolute_address: Option<String>, // PLC绝对地址，如 "%MD100"
    pub plc_communication_address: String,   // 通信地址，如 "40001"

    // 量程信息（模拟量专用）
    #[sea_orm(nullable)]
    pub range_lower_limit: Option<f64>,     // 量程下限
    #[sea_orm(nullable)]
    pub range_upper_limit: Option<f64>,     // 量程上限
    #[sea_orm(nullable)]
    pub engineering_unit: Option<String>,   // 工程单位

    // 报警设定值和地址（AI专用）
    #[sea_orm(nullable)]
    pub sll_set_value: Option<f64>,         // 低低报警设定值
    #[sea_orm(nullable)]
    pub sll_set_point_address: Option<String>, // 低低报警设定点地址
    #[sea_orm(nullable)]
    pub sll_set_point_plc_address: Option<String>, // 低低报警设定点PLC地址
    #[sea_orm(nullable)]
    pub sll_feedback_address: Option<String>, // 低低报警反馈地址
    #[sea_orm(nullable)]
    pub sll_feedback_plc_address: Option<String>, // 低低报警反馈PLC地址

    #[sea_orm(nullable)]
    pub sl_set_value: Option<f64>,          // 低报警设定值
    #[sea_orm(nullable)]
    pub sl_set_point_address: Option<String>, // 低报警设定点地址
    #[sea_orm(nullable)]
    pub sl_set_point_plc_address: Option<String>, // 低报警设定点PLC地址
    #[sea_orm(nullable)]
    pub sl_feedback_address: Option<String>, // 低报警反馈地址
    #[sea_orm(nullable)]
    pub sl_feedback_plc_address: Option<String>, // 低报警反馈PLC地址

    #[sea_orm(nullable)]
    pub sh_set_value: Option<f64>,          // 高报警设定值
    #[sea_orm(nullable)]
    pub sh_set_point_address: Option<String>, // 高报警设定点地址
    #[sea_orm(nullable)]
    pub sh_set_point_plc_address: Option<String>, // 高报警设定点PLC地址
    #[sea_orm(nullable)]
    pub sh_feedback_address: Option<String>, // 高报警反馈地址
    #[sea_orm(nullable)]
    pub sh_feedback_plc_address: Option<String>, // 高报警反馈PLC地址

    #[sea_orm(nullable)]
    pub shh_set_value: Option<f64>,         // 高高报警设定值
    #[sea_orm(nullable)]
    pub shh_set_point_address: Option<String>, // 高高报警设定点地址
    #[sea_orm(nullable)]
    pub shh_set_point_plc_address: Option<String>, // 高高报警设定点PLC地址
    #[sea_orm(nullable)]
    pub shh_feedback_address: Option<String>, // 高高报警反馈地址
    #[sea_orm(nullable)]
    pub shh_feedback_plc_address: Option<String>, // 高高报警反馈PLC地址

    // 维护功能地址
    #[sea_orm(nullable)]
    pub maintenance_value_set_point_address: Option<String>, // 维护值设定点地址
    #[sea_orm(nullable)]
    pub maintenance_value_set_point_plc_address: Option<String>, // 维护值设定点PLC地址
    #[sea_orm(nullable)]
    pub maintenance_enable_switch_point_address: Option<String>, // 维护使能开关点地址
    #[sea_orm(nullable)]
    pub maintenance_enable_switch_point_plc_address: Option<String>, // 维护使能开关点PLC地址

    // 其他属性
    #[sea_orm(nullable)]
    pub access_property: Option<String>,    // 访问属性，如 "R/W"
    #[sea_orm(nullable)]
    pub save_history: Option<bool>,         // 保存历史
    #[sea_orm(nullable)]
    pub power_failure_protection: Option<bool>, // 断电保护

    // 测试PLC地址（用于硬点测试）
    #[sea_orm(nullable)]
    pub test_rig_plc_address: Option<String>, // 测试台PLC地址

    // 时间戳
    pub created_time: DateTime<Utc>,        // 创建时间
    pub updated_time: DateTime<Utc>,        // 更新时间
}

// 定义实体间的关系，目前 ChannelPointDefinition 与其他实体没有直接关系
#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

// 实现 ActiveModelBehavior trait，提供默认的行为
impl ActiveModelBehavior for ActiveModel {
    fn new() -> Self {
        Self {
            id: Set(default_id()),
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

// 从原始结构体到SeaORM Model的转换函数
// 这在将从外部（如Excel导入）获取的原始数据转换为可存入数据库的Model时非常有用
impl From<&crate::models::structs::ChannelPointDefinition> for ActiveModel {
    fn from(original: &crate::models::structs::ChannelPointDefinition) -> Self {
        let now = Utc::now();
        Self {
            id: Set(original.id.clone()),
            tag: Set(original.tag.clone()),
            variable_name: Set(original.variable_name.clone()),
            variable_description: Set(original.variable_description.clone()),
            station_name: Set(original.station_name.clone()),
            module_name: Set(original.module_name.clone()),
            module_type: Set(original.module_type.to_string()),
            channel_tag_in_module: Set(original.channel_tag_in_module.clone()),
            data_type: Set(original.data_type.to_string()),
            power_supply_type: Set(original.power_supply_type.clone()),
            wire_system: Set(original.wire_system.clone()),
            plc_absolute_address: Set(original.plc_absolute_address.clone()),
            plc_communication_address: Set(original.plc_communication_address.clone()),
            range_lower_limit: Set(original.range_lower_limit.map(|v| v as f64)),
            range_upper_limit: Set(original.range_upper_limit.map(|v| v as f64)),
            engineering_unit: Set(original.engineering_unit.clone()),
            sll_set_value: Set(original.sll_set_value.map(|v| v as f64)),
            sll_set_point_address: Set(original.sll_set_point_address.clone()),
            sll_set_point_plc_address: Set(None), // 新字段，原结构体没有
            sll_feedback_address: Set(original.sll_feedback_address.clone()),
            sll_feedback_plc_address: Set(None), // 新字段，原结构体没有
            sl_set_value: Set(original.sl_set_value.map(|v| v as f64)),
            sl_set_point_address: Set(original.sl_set_point_address.clone()),
            sl_set_point_plc_address: Set(None), // 新字段，原结构体没有
            sl_feedback_address: Set(original.sl_feedback_address.clone()),
            sl_feedback_plc_address: Set(None), // 新字段，原结构体没有
            sh_set_value: Set(original.sh_set_value.map(|v| v as f64)),
            sh_set_point_address: Set(original.sh_set_point_address.clone()),
            sh_set_point_plc_address: Set(None), // 新字段，原结构体没有
            sh_feedback_address: Set(original.sh_feedback_address.clone()),
            sh_feedback_plc_address: Set(None), // 新字段，原结构体没有
            shh_set_value: Set(original.shh_set_value.map(|v| v as f64)),
            shh_set_point_address: Set(original.shh_set_point_address.clone()),
            shh_set_point_plc_address: Set(None), // 新字段，原结构体没有
            shh_feedback_address: Set(original.shh_feedback_address.clone()),
            shh_feedback_plc_address: Set(None), // 新字段，原结构体没有
            maintenance_value_set_point_address: Set(original.maintenance_value_set_point_address.clone()),
            maintenance_value_set_point_plc_address: Set(None), // 新字段，原结构体没有
            maintenance_enable_switch_point_address: Set(original.maintenance_enable_switch_point_address.clone()),
            maintenance_enable_switch_point_plc_address: Set(None), // 新字段，原结构体没有
            access_property: Set(original.access_property.clone()),
            save_history: Set(original.save_history),
            power_failure_protection: Set(original.power_failure_protection),
            test_rig_plc_address: Set(original.test_rig_plc_address.clone()),
            created_time: Set(now),
            updated_time: Set(now),
        }
    }
}

// 从SeaORM Model转换回原始结构体
// 这在从数据库加载数据后，需要将其转换为业务逻辑层使用的原始结构体时非常有用
impl From<&Model> for crate::models::structs::ChannelPointDefinition {
    fn from(model: &Model) -> Self {
        crate::models::structs::ChannelPointDefinition {
            id: model.id.clone(),
            tag: model.tag.clone(),
            variable_name: model.variable_name.clone(),
            variable_description: model.variable_description.clone(),
            station_name: model.station_name.clone(),
            module_name: model.module_name.clone(),
            // 将字符串转换回 ModuleType 枚举
            module_type: model.module_type.parse().unwrap_or_default(),
            channel_tag_in_module: model.channel_tag_in_module.clone(),
            // 将字符串转换回 PointDataType 枚举
            data_type: model.data_type.parse().unwrap_or_default(),
            power_supply_type: model.power_supply_type.clone(),
            wire_system: model.wire_system.clone(),
            plc_absolute_address: model.plc_absolute_address.clone(),
            plc_communication_address: model.plc_communication_address.clone(),
            range_lower_limit: model.range_lower_limit.map(|v| v as f32),
            range_upper_limit: model.range_upper_limit.map(|v| v as f32),
            engineering_unit: model.engineering_unit.clone(),
            sll_set_value: model.sll_set_value.map(|v| v as f32),
            sll_set_point_address: model.sll_set_point_address.clone(),
            sll_feedback_address: model.sll_feedback_address.clone(),
            sl_set_value: model.sl_set_value.map(|v| v as f32),
            sl_set_point_address: model.sl_set_point_address.clone(),
            sl_feedback_address: model.sl_feedback_address.clone(),
            sh_set_value: model.sh_set_value.map(|v| v as f32),
            sh_set_point_address: model.sh_set_point_address.clone(),
            sh_feedback_address: model.sh_feedback_address.clone(),
            shh_set_value: model.shh_set_value.map(|v| v as f32),
            shh_set_point_address: model.shh_set_point_address.clone(),
            shh_feedback_address: model.shh_feedback_address.clone(),
            maintenance_value_set_point_address: model.maintenance_value_set_point_address.clone(),
            maintenance_enable_switch_point_address: model.maintenance_enable_switch_point_address.clone(),
            access_property: model.access_property.clone(),
            save_history: model.save_history,
            power_failure_protection: model.power_failure_protection,
            test_rig_plc_address: model.test_rig_plc_address.clone(),
        }
    }
}

// 为 ChannelPointDefinition 实现一些便利方法
impl Model {
    /// 创建新的通道点位定义
    pub fn new(
        tag: String,
        variable_name: String,
        variable_description: String,
        station_name: String,
        module_name: String,
        module_type: ModuleType,
        channel_tag_in_module: String,
        data_type: PointDataType,
        power_supply_type: String,
        wire_system: String,
        plc_communication_address: String,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: default_id(),
            tag,
            variable_name,
            variable_description,
            station_name,
            module_name,
            module_type: module_type.to_string(),
            channel_tag_in_module,
            data_type: data_type.to_string(),
            power_supply_type,
            wire_system,
            plc_absolute_address: None,
            plc_communication_address,
            range_lower_limit: None,
            range_upper_limit: None,
            engineering_unit: None,
            sll_set_value: None,
            sll_set_point_address: None,
            sll_set_point_plc_address: None,
            sll_feedback_address: None,
            sll_feedback_plc_address: None,
            sl_set_value: None,
            sl_set_point_address: None,
            sl_set_point_plc_address: None,
            sl_feedback_address: None,
            sl_feedback_plc_address: None,
            sh_set_value: None,
            sh_set_point_address: None,
            sh_set_point_plc_address: None,
            sh_feedback_address: None,
            sh_feedback_plc_address: None,
            shh_set_value: None,
            shh_set_point_address: None,
            shh_set_point_plc_address: None,
            shh_feedback_address: None,
            shh_feedback_plc_address: None,
            maintenance_value_set_point_address: None,
            maintenance_value_set_point_plc_address: None,
            maintenance_enable_switch_point_address: None,
            maintenance_enable_switch_point_plc_address: None,
            access_property: None,
            save_history: None,
            power_failure_protection: None,
            test_rig_plc_address: None,
            created_time: now,
            updated_time: now,
        }
    }

    /// 获取模块类型枚举
    pub fn get_module_type(&self) -> Result<ModuleType, String> {
        self.module_type.parse()
    }

    /// 获取数据类型枚举
    pub fn get_data_type(&self) -> Result<PointDataType, String> {
        self.data_type.parse()
    }

    /// 判断是否为模拟量输入
    pub fn is_analog_input(&self) -> bool {
        matches!(self.get_module_type(), Ok(ModuleType::AI))
    }

    /// 判断是否为模拟量输出
    pub fn is_analog_output(&self) -> bool {
        matches!(self.get_module_type(), Ok(ModuleType::AO))
    }

    /// 判断是否为数字量输入
    pub fn is_digital_input(&self) -> bool {
        matches!(self.get_module_type(), Ok(ModuleType::DI))
    }

    /// 判断是否为数字量输出
    pub fn is_digital_output(&self) -> bool {
        matches!(self.get_module_type(), Ok(ModuleType::DO))
    }

    /// 判断是否为模拟量（AI或AO）
    pub fn is_analog(&self) -> bool {
        self.is_analog_input() || self.is_analog_output()
    }

    /// 判断是否为数字量（DI或DO）
    pub fn is_digital(&self) -> bool {
        self.is_digital_input() || self.is_digital_output()
    }

    /// 判断是否有报警功能（通常只有AI点有）
    pub fn has_alarm_function(&self) -> bool {
        self.is_analog_input() && (
            self.sll_set_value.is_some() ||
            self.sl_set_value.is_some() ||
            self.sh_set_value.is_some() ||
            self.shh_set_value.is_some()
        )
    }

    /// 判断是否有维护功能
    pub fn has_maintenance_function(&self) -> bool {
        self.maintenance_value_set_point_address.is_some() ||
        self.maintenance_enable_switch_point_address.is_some()
    }

    /// 获取所有报警设定值（用于测试）
    pub fn get_alarm_set_values(&self) -> Vec<(String, f64)> {
        let mut alarms = Vec::new();

        if let Some(value) = self.sll_set_value {
            alarms.push(("SLL".to_string(), value));
        }
        if let Some(value) = self.sl_set_value {
            alarms.push(("SL".to_string(), value));
        }
        if let Some(value) = self.sh_set_value {
            alarms.push(("SH".to_string(), value));
        }
        if let Some(value) = self.shh_set_value {
            alarms.push(("SHH".to_string(), value));
        }

        alarms
    }
}