/// 简单的事件发布器实现
///
/// 用于发布系统事件到前端或其他订阅者
use async_trait::async_trait;
use crate::services::traits::{EventPublisher, BaseService};
use crate::models::structs::RawTestOutcome;
use crate::models::enums::OverallTestStatus;
use crate::error::AppError;
use tauri::{AppHandle, Manager, Emitter};
use serde_json::json;
use once_cell::sync::OnceCell;
// BatchStatistics 需要从正确的位置导入，暂时使用一个简单的结构体
#[derive(Debug, Clone)]
pub struct BatchStatistics {
    pub total: u32,
    pub passed: u32,
    pub failed: u32,
}
use log::{info, debug, warn, error, trace};

type AppResult<T> = Result<T, AppError>;

// 全局AppHandle存储，用于事件发布
static GLOBAL_APP_HANDLE: once_cell::sync::OnceCell<AppHandle> = once_cell::sync::OnceCell::new();

/// 设置全局AppHandle
pub fn set_global_app_handle(handle: AppHandle) {
    if let Err(_) = GLOBAL_APP_HANDLE.set(handle) {
        // 🔧 移除 [EventPublisher] 日志
    } else {
        // 🔧 移除 [EventPublisher] 日志
    }
}

/// 简单的事件发布器实现
/// 支持通过Tauri发布事件到前端
pub struct SimpleEventPublisher {
    service_name: String,
}

impl SimpleEventPublisher {
    pub fn new() -> Self {
        Self {
            service_name: "SimpleEventPublisher".to_string(),
        }
    }

    /// 发布事件到前端
    async fn emit_to_frontend(&self, event_name: &str, payload: serde_json::Value) -> AppResult<()> {
        if let Some(handle) = GLOBAL_APP_HANDLE.get() {
            if let Err(e) = handle.emit(event_name, payload) {
                // 🔧 移除 [EventPublisher] 日志
                return Err(AppError::generic(format!("发布事件失败: {}", e)));
            } else {
                // 完全移除事件发布成功的冗余日志
            }
        } else {
            // 🔧 移除 [EventPublisher] 日志
        }
        Ok(())
    }
}

#[async_trait]
impl BaseService for SimpleEventPublisher {
    fn service_name(&self) -> &'static str {
        "SimpleEventPublisher"
    }

    async fn initialize(&mut self) -> AppResult<()> {
        Ok(())
    }

    async fn shutdown(&mut self) -> AppResult<()> {
        Ok(())
    }

    async fn health_check(&self) -> AppResult<()> {
        Ok(())
    }
}

#[async_trait]
impl EventPublisher for SimpleEventPublisher {
    /// 发布测试状态变化事件
    async fn publish_test_status_changed(
        &self,
        instance_id: &str,
        old_status: OverallTestStatus,
        new_status: OverallTestStatus,
    ) -> AppResult<()> {
        // 🔧 移除 [EventPublisher] 日志

        // 发布状态变化事件到前端
        let payload = json!({
            "instanceId": instance_id,
            "oldStatus": old_status,
            "newStatus": new_status,
            "timestamp": chrono::Utc::now().to_rfc3339()
        });

        self.emit_to_frontend("test-status-changed", payload).await?;

        // 🔧 移除 [EventPublisher] 日志

        Ok(())
    }

    /// 发布测试完成事件
    async fn publish_test_completed(&self, outcome: &RawTestOutcome) -> AppResult<()> {
        // 移除冗余的测试完成DEBUG日志

        // 发布事件到前端
        let payload = json!({
            "instanceId": outcome.channel_instance_id,
            "success": outcome.success,
            "subTestItem": outcome.sub_test_item,
            "message": outcome.message.as_deref().unwrap_or(""),
            "rawValue": outcome.raw_value_read,
            "engValue": outcome.eng_value_calculated
        });

        self.emit_to_frontend("test-completed", payload).await?;

        // 🔧 移除 [EventPublisher] 日志

        Ok(())
    }

    /// 发布批次状态变化事件
    async fn publish_batch_status_changed(&self, batch_id: &str, statistics: &crate::services::traits::BatchStatistics) -> AppResult<()> {
        // 🔧 移除 [EventPublisher] 日志

        // 判断批次状态
        let status = if statistics.tested_channels >= statistics.total_channels {
            "completed"
        } else if statistics.tested_channels > 0 {
            "running"
        } else {
            "pending"
        };

        let progress_percentage = if statistics.total_channels > 0 {
            (statistics.tested_channels as f32 / statistics.total_channels as f32) * 100.0
        } else {
            0.0
        };

        // 发布批次状态变化事件到前端
        let batch_payload = json!({
            "batchId": batch_id,
            "status": status,
            "statistics": {
                "totalChannels": statistics.total_channels,
                "testedChannels": statistics.tested_channels,
                "passedChannels": statistics.passed_channels,
                "failedChannels": statistics.failed_channels,
                "skippedChannels": statistics.skipped_channels,
                "inProgressChannels": statistics.in_progress_channels,
                "progressPercentage": progress_percentage,
                "startTime": None::<String>,
                "endTime": None::<String>,
                "estimatedCompletionTime": None::<String>
            }
        });

        self.emit_to_frontend("batch-status-changed", batch_payload).await?;

        // 同时发布测试进度更新事件（专门用于进度模态框）
        let progress_payload = json!({
            "batchId": batch_id,
            "totalPoints": statistics.total_channels,
            "completedPoints": statistics.tested_channels,
            "successPoints": statistics.passed_channels,
            "failedPoints": statistics.failed_channels,
            "progressPercentage": progress_percentage,
            "currentPoint": serde_json::Value::Null
        });

        self.emit_to_frontend("test-progress-update", progress_payload).await?;

        // 🔧 移除 [EventPublisher] 日志

        Ok(())
    }

    /// 发布PLC连接状态变化事件
    async fn publish_plc_connection_changed(&self, connected: bool) -> AppResult<()> {
        if connected {
            // 🔧 移除 [EventPublisher] 日志
        } else {
            // 🔧 移除 [EventPublisher] 日志
        }

        // TODO: 实际的事件发布逻辑

        Ok(())
    }

    /// 发布错误事件
    async fn publish_error(&self, error: &AppError) -> AppResult<()> {
        // 🔧 移除 [EventPublisher] 日志

        // TODO: 实际的事件发布逻辑

        Ok(())
    }
}
