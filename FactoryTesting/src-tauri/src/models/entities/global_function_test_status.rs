use sea_orm::entity::prelude::*;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};

/// SeaORM Entity - GlobalFunctionTestStatus
/// 用于保存 5 个上位机全局功能测试项的状态
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "global_function_test_statuses")]
pub struct Model {
    /// 主键 UUID
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,

    /// 站场名称
    pub station_name: String,

    /// 功能键（固定 5 个值之一）
    pub function_key: String,

    /// 点表导入时间（UTC ISO8601 字符串）
    pub import_time: String,

    /// 开始时间（UTC ISO8601）
    pub start_time: Option<String>,

    /// 结束时间（UTC ISO8601）
    pub end_time: Option<String>,

    /// 当前状态（枚举字符串：NotTested / Testing / Passed / Failed）
    pub status: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
