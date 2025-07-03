//! 领域服务接口定义
//!
//! 这个模块包含了所有核心业务服务的接口定义，
//! 遵循依赖倒置原则，具体实现在infrastructure层

pub mod test_orchestration_service;
pub mod channel_state_manager;
pub mod test_execution_engine;
pub mod plc_communication_service;
pub mod batch_allocation_service;
pub mod event_publisher;
pub mod persistence_service;


// 重新导出所有服务接口
pub use test_orchestration_service::*;
pub use channel_state_manager::*;
pub use test_execution_engine::*;
pub use plc_communication_service::*;
pub use batch_allocation_service::*;
pub use event_publisher::*;
pub use persistence_service::*;

// ---------------------------------------------------------------------------
// 兼容旧路径 / 命名的重新导出（Backward-compatibility Aliases）
// 在 Phase 3 清理时可删除。
// ---------------------------------------------------------------------------
// 将具体实现结构体重新导出到 `services` 命名空间
pub use crate::domain::impls::{
    ChannelStateManager,
    TestExecutionEngine,
    TestPlcConfigService,
    PlcConnectionManager,
};
// 为旧的不带 `I` 前缀的 trait 名称提供别名
pub use event_publisher::IEventPublisher as EventPublisher;
pub use persistence_service::IPersistenceService as PersistenceService;
// 将 ITestPlcConfigService 重新导出到 services 命名空间
pub use crate::domain::impls::test_plc_config_service::ITestPlcConfigService;

// Re-export enums for easy access within domain services
pub use crate::models::enums::*;

// 重新导出基础类型
use crate::utils::error::AppResult;
use crate::models::structs::*;
use async_trait::async_trait;
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// 基础服务trait，所有服务都应实现
#[async_trait]
pub trait BaseService: Send + Sync {
    /// 服务名称
    fn service_name(&self) -> &'static str;
    
    /// 初始化服务
    async fn initialize(&mut self) -> AppResult<()>;
    
    /// 关闭服务
    async fn shutdown(&mut self) -> AppResult<()>;
    
    /// 健康检查
    async fn health_check(&self) -> AppResult<()>;
}

/// 测试值类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TestValue {
    Bool(bool),
    Int(i32),
    Float(f32),
    String(String),
}

/// 测试结果类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestResult {
    pub success: bool,
    pub actual_value: Option<TestValue>,
    pub expected_value: Option<TestValue>,
    pub error_message: Option<String>,
    pub execution_time_ms: u64,
    pub timestamp: DateTime<Utc>,
}

/// 批次统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchStatistics {
    pub total_channels: u32,
    pub tested_channels: u32,
    pub passed_channels: u32,
    pub failed_channels: u32,
    pub skipped_channels: u32,
    pub in_progress_channels: u32,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
    pub estimated_completion_time: Option<DateTime<Utc>>,
}

/// 测试进度更新信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestProgressUpdate {
    pub batch_id: String,
    pub instance_id: String,
    pub progress_percentage: f32,
    pub current_step: String,
    pub estimated_remaining_time_ms: Option<u64>,
    pub statistics: BatchStatistics,
    pub timestamp: DateTime<Utc>,
}

/// 服务健康状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceHealth {
    pub service_name: String,
    pub is_healthy: bool,
    pub last_check: DateTime<Utc>,
    pub error_message: Option<String>,
    pub uptime_seconds: u64,
}
