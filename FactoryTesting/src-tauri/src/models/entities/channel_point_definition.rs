// 文件: FactoryTesting/src-tauri/src/models/entities/channel_point_definition.rs
// 详细注释：ChannelPointDefinition实体的SeaORM定义

use sea_orm::entity::prelude::*;
use sea_orm::ActiveValue::Set;
use serde::{Deserialize, Serialize};
use crate::models::structs::default_id; // 确保可以访问 default_id
use crate::models::enums::{ModuleType, PointDataType}; // 引入枚举

// 将原始的 ChannelPointDefinition 结构体转换为 SeaORM 实体
// 大部分字段直接映射，Option<T> 映射为可空列
// 枚举类型 ModuleType 和 PointDataType 暂时映射为字符串存储
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "channel_point_definitions")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)] // id 不是自增的，由应用生成
    #[serde(default = "default_id")]
    pub id: String,
    pub tag: String,
    pub variable_name: String,
    pub variable_description: String,
    pub station_name: String,
    pub module_name: String,
    
    // ModuleType 枚举将存储为字符串
    // 在更复杂的场景下，可以考虑将其映射为整数或使用单独的查找表
    #[sea_orm(column_type = "Text")]
    pub module_type: String, // 原为 ModuleType
    
    pub channel_tag_in_module: String,
    
    // PointDataType 枚举将存储为字符串
    #[sea_orm(column_type = "Text")]
    pub data_type: String, // 原为 PointDataType
    
    pub power_supply_type: String,
    pub wire_system: String,
    
    #[sea_orm(nullable)]
    pub plc_absolute_address: Option<String>,
    pub plc_communication_address: String,
    
    #[sea_orm(nullable)]
    pub range_lower_limit: Option<f32>,
    #[sea_orm(nullable)]
    pub range_upper_limit: Option<f32>,
    #[sea_orm(nullable)]
    pub engineering_unit: Option<String>,
    
    #[sea_orm(nullable)]
    pub sll_set_value: Option<f32>,
    #[sea_orm(nullable)]
    pub sll_set_point_address: Option<String>,
    #[sea_orm(nullable)]
    pub sll_feedback_address: Option<String>,
    
    #[sea_orm(nullable)]
    pub sl_set_value: Option<f32>,
    #[sea_orm(nullable)]
    pub sl_set_point_address: Option<String>,
    #[sea_orm(nullable)]
    pub sl_feedback_address: Option<String>,
    
    #[sea_orm(nullable)]
    pub sh_set_value: Option<f32>,
    #[sea_orm(nullable)]
    pub sh_set_point_address: Option<String>,
    #[sea_orm(nullable)]
    pub sh_feedback_address: Option<String>,
    
    #[sea_orm(nullable)]
    pub shh_set_value: Option<f32>,
    #[sea_orm(nullable)]
    pub shh_set_point_address: Option<String>,
    #[sea_orm(nullable)]
    pub shh_feedback_address: Option<String>,
    
    #[sea_orm(nullable)]
    pub maintenance_value_set_point_address: Option<String>,
    #[sea_orm(nullable)]
    pub maintenance_enable_switch_point_address: Option<String>,
    
    #[sea_orm(nullable)]
    pub access_property: Option<String>,
    #[sea_orm(nullable)]
    pub save_history: Option<bool>,
    #[sea_orm(nullable)]
    pub power_failure_protection: Option<bool>,
    
    #[sea_orm(nullable)]
    pub test_rig_plc_address: Option<String>,

    // SeaORM 的 ActiveModelBehavior trait 提供了默认的创建/更新时间戳行为
    // 但如果 ChannelPointDefinition 本身不包含这些字段，则不需要在此处显式添加
    // 如果原始结构体有创建/更新时间字段，需要在这里也定义它们
    // pub created_at: DateTimeUtc,
    // pub updated_at: DateTimeUtc,
}

// 定义实体间的关系，目前 ChannelPointDefinition 与其他实体没有直接关系
#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

// 为实体实现 ActiveModelBehavior trait
// 这允许我们在保存或更新实体时自动处理一些行为，例如时间戳更新
// 但对于 ChannelPointDefinition，目前没有复杂的自定义行为
impl ActiveModelBehavior for ActiveModel {}

// 可选：从原始结构体到SeaORM Model的转换函数
// 这在将从外部（如Excel导入）获取的原始数据转换为可存入数据库的Model时非常有用
impl From<&crate::models::structs::ChannelPointDefinition> for ActiveModel {
    fn from(original: &crate::models::structs::ChannelPointDefinition) -> Self {
        Self {
            id: Set(original.id.clone()),
            tag: Set(original.tag.clone()),
            variable_name: Set(original.variable_name.clone()),
            variable_description: Set(original.variable_description.clone()),
            station_name: Set(original.station_name.clone()),
            module_name: Set(original.module_name.clone()),
            module_type: Set(format!("{:?}", original.module_type)), // 将枚举转换为字符串 (移除不正确的转义)
            channel_tag_in_module: Set(original.channel_tag_in_module.clone()),
            data_type: Set(format!("{:?}", original.data_type)), // 将枚举转换为字符串 (移除不正确的转义)
            power_supply_type: Set(original.power_supply_type.clone()),
            wire_system: Set(original.wire_system.clone()),
            plc_absolute_address: Set(original.plc_absolute_address.clone()),
            plc_communication_address: Set(original.plc_communication_address.clone()),
            range_lower_limit: Set(original.range_lower_limit),
            range_upper_limit: Set(original.range_upper_limit),
            engineering_unit: Set(original.engineering_unit.clone()),
            sll_set_value: Set(original.sll_set_value),
            sll_set_point_address: Set(original.sll_set_point_address.clone()),
            sll_feedback_address: Set(original.sll_feedback_address.clone()),
            sl_set_value: Set(original.sl_set_value),
            sl_set_point_address: Set(original.sl_set_point_address.clone()),
            sl_feedback_address: Set(original.sl_feedback_address.clone()),
            sh_set_value: Set(original.sh_set_value),
            sh_set_point_address: Set(original.sh_set_point_address.clone()),
            sh_feedback_address: Set(original.sh_feedback_address.clone()),
            shh_set_value: Set(original.shh_set_value),
            shh_set_point_address: Set(original.shh_set_point_address.clone()),
            shh_feedback_address: Set(original.shh_feedback_address.clone()),
            maintenance_value_set_point_address: Set(original.maintenance_value_set_point_address.clone()),
            maintenance_enable_switch_point_address: Set(original.maintenance_enable_switch_point_address.clone()),
            access_property: Set(original.access_property.clone()),
            save_history: Set(original.save_history),
            power_failure_protection: Set(original.power_failure_protection),
            test_rig_plc_address: Set(original.test_rig_plc_address.clone()),
            // created_at 和 updated_at 如果 ChannelPointDefinition 结构体没有，则不需要设置
            ..Default::default() // 对于其他未在From中处理的字段（例如主键的自增行为或默认值），使用Default
        }
    }
}

// 可选：从SeaORM Model转换回原始结构体
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
            // 将字符串转换回 ModuleType 枚举，这里需要一个 robust 的解析逻辑
            // 简单的示例是使用 match，但实际中可能需要更复杂的解析
            module_type: match model.module_type.as_str() {
                "AI" => ModuleType::AI,
                "AO" => ModuleType::AO,
                "DI" => ModuleType::DI,
                "DO" => ModuleType::DO,
                "AINone" => ModuleType::AINone,
                "AONone" => ModuleType::AONone,
                "DINone" => ModuleType::DINone,
                "DONone" => ModuleType::DONone,
                _ => ModuleType::default(), // 或者抛出错误
            },
            channel_tag_in_module: model.channel_tag_in_module.clone(),
            // 将字符串转换回 PointDataType 枚举
            data_type: match model.data_type.as_str() {
                "Bool" => PointDataType::Bool,
                "Float" => PointDataType::Float,
                "Int" => PointDataType::Int,
                _ => PointDataType::default(), // 或者抛出错误
            },
            power_supply_type: model.power_supply_type.clone(),
            wire_system: model.wire_system.clone(),
            plc_absolute_address: model.plc_absolute_address.clone(),
            plc_communication_address: model.plc_communication_address.clone(),
            range_lower_limit: model.range_lower_limit,
            range_upper_limit: model.range_upper_limit,
            engineering_unit: model.engineering_unit.clone(),
            sll_set_value: model.sll_set_value,
            sll_set_point_address: model.sll_set_point_address.clone(),
            sll_feedback_address: model.sll_feedback_address.clone(),
            sl_set_value: model.sl_set_value,
            sl_set_point_address: model.sl_set_point_address.clone(),
            sl_feedback_address: model.sl_feedback_address.clone(),
            sh_set_value: model.sh_set_value,
            sh_set_point_address: model.sh_set_point_address.clone(),
            sh_feedback_address: model.sh_feedback_address.clone(),
            shh_set_value: model.shh_set_value,
            shh_set_point_address: model.shh_set_point_address.clone(),
            shh_feedback_address: model.shh_feedback_address.clone(),
            maintenance_value_set_point_address: model.maintenance_value_set_point_address.clone(),
            maintenance_enable_switch_point_address: model.maintenance_enable_switch_point_address.clone(),
            access_property: model.access_property.clone(),
            save_history: model.save_history,
            power_failure_protection: model.power_failure_protection,
            test_rig_plc_address: model.test_rig_plc_address.clone(),
            // ChannelPointDefinition 结构体中没有创建/更新时间字段
        }
    }
}

// 确保crate::models::structs::default_id() 可用
// 并且 crate::models::enums::{ModuleType, PointDataType} 已正确导入
// 如果 ModuleType 或 PointDataType 的 Debug 输出不是其变体名称（例如 "AI", "Bool"），
// 则 From<&crate::models::structs::ChannelPointDefinition> for ActiveModel 中的 format!("{:?}", ...)
// 以及 From<&Model> for crate::models::structs::ChannelPointDefinition 中的 match 语句需要相应调整。
// 推荐为枚举实现 Display 和 FromStr trait 以进行更稳健的字符串转换。 