// 文件: FactoryTesting/src-tauri/src/models/entities/test_batch_info.rs
// 详细注释：TestBatchInfo实体的SeaORM定义
// 基于原C#项目数据库结构重构

use sea_orm::entity::prelude::*;
use sea_orm::ActiveValue::Set;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use crate::models::structs::default_id;
use crate::models::enums::OverallTestStatus;

/// 测试批次信息实体
///
/// 管理测试批次的基本信息和统计数据
/// 一个批次包含多个通道测试实例
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "test_batch_info")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    #[serde(default = "default_id")]
    pub batch_id: String,                   // 批次ID

    // 产品信息
    pub batch_name: String,                 // 批次名称
    #[sea_orm(nullable)]
    pub product_model: Option<String>,      // 产品型号
    #[sea_orm(nullable)]
    pub serial_number: Option<String>,      // 序列号
    #[sea_orm(nullable)]
    pub customer_name: Option<String>,      // 客户名称
    #[sea_orm(nullable)]
    pub station_name: Option<String>,       // 站点名称

    // 时间信息
    pub created_time: DateTime<Utc>,        // 创建时间
    pub updated_time: DateTime<Utc>,        // 最后更新时间
    #[sea_orm(nullable)]
    pub start_time: Option<DateTime<Utc>>,  // 开始测试时间
    #[sea_orm(nullable)]
    pub end_time: Option<DateTime<Utc>>,    // 结束测试时间
    #[sea_orm(nullable)]
    pub total_duration_ms: Option<i64>,     // 总测试时长（毫秒）

    // 操作信息
    #[sea_orm(nullable)]
    pub operator_name: Option<String>,      // 操作员名称
    #[sea_orm(nullable)]
    pub created_by: Option<String>,         // 创建者

    // 状态信息
    #[sea_orm(column_type = "Text")]
    pub overall_status: String,             // 整体状态
    #[sea_orm(nullable)]
    pub status_summary: Option<String>,     // 状态摘要
    #[sea_orm(nullable)]
    pub error_message: Option<String>,      // 错误消息

    // 统计信息
    pub total_points: u32,                  // 总点数
    pub tested_points: u32,                 // 已测试点数
    pub passed_points: u32,                 // 通过点数
    pub failed_points: u32,                 // 失败点数
    pub skipped_points: u32,                // 跳过点数
    pub not_tested_points: u32,             // 未测试点数

    // 进度信息
    pub progress_percentage: f32,           // 进度百分比 (0.0 - 100.0)
    #[sea_orm(nullable)]
    pub current_testing_channel: Option<String>, // 当前测试通道

    // 配置信息
    #[sea_orm(nullable)]
    pub test_configuration: Option<String>, // 测试配置（JSON）
    #[sea_orm(nullable)]
    pub import_source: Option<String>,      // 导入源（如Excel文件名）

    // 自定义数据（JSON存储）
    #[sea_orm(column_type = "Text", nullable)]
    pub custom_data_json: Option<String>,   // 自定义数据
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    // 关联关系暂时注释，等其他实体完善后再启用
    // #[sea_orm(has_many = "super::channel_test_instance::Entity")]
    // ChannelTestInstance,
}

impl ActiveModelBehavior for ActiveModel {
    fn new() -> Self {
        Self {
            batch_id: Set(default_id()),
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

impl From<&crate::models::structs::TestBatchInfo> for ActiveModel {
    fn from(original: &crate::models::structs::TestBatchInfo) -> Self {
        let custom_data_json = serde_json::to_string(&original.custom_data)
            .unwrap_or_else(|_| "{}".to_string());
        let now = Utc::now();

        Self {
            batch_id: Set(original.batch_id.clone()),
            batch_name: Set(original.batch_name.clone()),
            product_model: Set(original.product_model.clone()),
            serial_number: Set(original.serial_number.clone()),
            customer_name: Set(original.customer_name.clone()),
            station_name: Set(original.station_name.clone()), // 🔧 修复：正确映射station_name字段
            created_time: Set(original.creation_time),
            updated_time: Set(original.last_updated_time),
            start_time: Set(None), // 新字段，原结构体没有
            end_time: Set(None), // 新字段，原结构体没有
            total_duration_ms: Set(None), // 新字段，原结构体没有
            operator_name: Set(original.operator_name.clone()),
            created_by: Set(None), // 新字段，原结构体没有
            overall_status: Set(original.overall_status.to_string()),
            status_summary: Set(original.status_summary.clone()),
            error_message: Set(None), // 新字段，原结构体没有
            total_points: Set(original.total_points),
            tested_points: Set(original.tested_points),
            passed_points: Set(original.passed_points),
            failed_points: Set(original.failed_points),
            skipped_points: Set(original.skipped_points),
            not_tested_points: Set(0), // 新字段，计算得出
            progress_percentage: Set(0.0), // 新字段，计算得出
            current_testing_channel: Set(None), // 新字段，原结构体没有
            test_configuration: Set(None), // 新字段，原结构体没有
            import_source: Set(None), // 新字段，原结构体没有
            custom_data_json: Set(Some(custom_data_json)),
        }
    }
}

impl From<&Model> for crate::models::structs::TestBatchInfo {
    fn from(model: &Model) -> Self {
        let custom_data_map: HashMap<String, String> = model.custom_data_json.as_ref()
            .and_then(|json_str| serde_json::from_str(json_str).ok())
            .unwrap_or_default();

        crate::models::structs::TestBatchInfo {
            batch_id: model.batch_id.clone(),
            batch_name: model.batch_name.clone(),
            product_model: model.product_model.clone(),
            serial_number: model.serial_number.clone(),
            customer_name: model.customer_name.clone(),
            station_name: model.station_name.clone(),
            creation_time: model.created_time,
            last_updated_time: model.updated_time,
            operator_name: model.operator_name.clone(),
            status_summary: model.status_summary.clone(),
            total_points: model.total_points,
            tested_points: model.tested_points,
            passed_points: model.passed_points,
            failed_points: model.failed_points,
            skipped_points: model.skipped_points,
            overall_status: model.overall_status.parse().unwrap_or_default(),
            custom_data: custom_data_map,
        }
    }
}

// 为 TestBatchInfo 实体添加便利方法
impl Model {
    /// 创建新的测试批次
    pub fn new(
        batch_name: String,
        product_model: Option<String>,
        operator_name: Option<String>,
    ) -> Self {
        let now = Utc::now();
        Self {
            batch_id: default_id(),
            batch_name,
            product_model,
            serial_number: None,
            customer_name: None,
            station_name: None,
            created_time: now,
            updated_time: now,
            start_time: None,
            end_time: None,
            total_duration_ms: None,
            operator_name,
            created_by: None,
            overall_status: OverallTestStatus::NotTested.to_string(),
            status_summary: None,
            error_message: None,
            total_points: 0,
            tested_points: 0,
            passed_points: 0,
            failed_points: 0,
            skipped_points: 0,
            not_tested_points: 0,
            progress_percentage: 0.0,
            current_testing_channel: None,
            test_configuration: None,
            import_source: None,
            custom_data_json: None,
        }
    }

    /// 获取整体状态枚举
    pub fn get_overall_status(&self) -> Result<OverallTestStatus, String> {
        self.overall_status.parse()
    }

    /// 更新统计信息
    pub fn update_statistics(&mut self, total: u32, tested: u32, passed: u32, failed: u32, skipped: u32) {
        self.total_points = total;
        self.tested_points = tested;
        self.passed_points = passed;
        self.failed_points = failed;
        self.skipped_points = skipped;
        self.not_tested_points = total.saturating_sub(tested);

        // 计算进度百分比
        if total > 0 {
            self.progress_percentage = (tested as f32 / total as f32) * 100.0;
        } else {
            self.progress_percentage = 0.0;
        }
    }

    /// 判断批次是否完成
    pub fn is_completed(&self) -> bool {
        self.tested_points + self.skipped_points >= self.total_points
    }

    /// 判断批次是否全部通过
    pub fn is_all_passed(&self) -> bool {
        self.is_completed() && self.failed_points == 0
    }

    /// 获取成功率
    pub fn get_success_rate(&self) -> f32 {
        if self.tested_points > 0 {
            (self.passed_points as f32 / self.tested_points as f32) * 100.0
        } else {
            0.0
        }
    }

    /// 开始测试
    pub fn start_testing(&mut self) {
        if self.start_time.is_none() {
            self.start_time = Some(Utc::now());
        }
    }

    /// 结束测试
    pub fn finish_testing(&mut self) {
        self.end_time = Some(Utc::now());
        if let Some(start_time) = self.start_time {
            self.total_duration_ms = Some((Utc::now() - start_time).num_milliseconds());
        }
    }

    /// 设置当前测试通道
    pub fn set_current_testing_channel(&mut self, channel: Option<String>) {
        self.current_testing_channel = channel;
    }
}