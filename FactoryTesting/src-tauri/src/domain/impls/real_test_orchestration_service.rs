use async_trait::async_trait;
use std::sync::Arc;

use crate::{
    domain::services::{
        ITestOrchestrationService, TestExecutionRequest, TestExecutionResponse, TestProgressUpdate,
        AllocationSummary, BaseService,
    },
    models::structs::{
        TestBatchInfo, StartManualTestRequest, StartManualTestResponse, UpdateManualTestSubItemRequest,
        UpdateManualTestSubItemResponse, ManualTestStatus,
    },
    utils::error::{AppError, AppResult},
};

use crate::application::services::test_coordination_service::{TestCoordinationService, ITestCoordinationService, TestExecutionRequest as AppExecutionRequest, TestExecutionResponse as AppExecutionResponse};
use crate::domain::services::TestProgressUpdate as DomainProgress;
use serde_json;

/// A thin wrapper around `application::services::test_coordination_service::TestCoordinationService`
/// that adapts it to the domain-layer `ITestOrchestrationService` trait.
///
/// For the current incremental step we only wire the struct; the methods will be
/// gradually delegated. At this stage they still return `not_implemented_error` to
/// keep compilation working while allowing DI to inject the real skeleton.
#[derive(Clone)]
pub struct RealTestOrchestrationService {
    inner: Arc<TestCoordinationService>,
}

impl RealTestOrchestrationService {
    pub fn new(inner: TestCoordinationService) -> Self {
        Self {
            inner: Arc::new(inner),
        }
    }
}

#[async_trait]
impl BaseService for RealTestOrchestrationService {
    fn service_name(&self) -> &'static str {
        "RealTestOrchestrationService"
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
impl ITestOrchestrationService for RealTestOrchestrationService {
    async fn create_test_batch(&self, request: TestExecutionRequest) -> AppResult<TestExecutionResponse> {
        // Map domain-layer request to application-layer request
        let app_req = AppExecutionRequest {
            batch_info: request.batch_info.clone(),
            channel_definitions: request.channel_definitions.clone(),
            max_concurrent_tests: Some(request.test_config.max_concurrent_tests as usize),
            auto_start: true,
        };

        // Delegate to the underlying TestCoordinationService
        let app_resp: AppExecutionResponse = self.inner.submit_test_execution(app_req).await?;

        // Convert application-layer response back to domain-layer response.
        // At this incremental stage we only fill the minimally required fields.
        use chrono::Utc;
        use std::collections::HashMap;
        let allocation_summary = AllocationSummary {
            total_channels: 0,
            allocated_channels: 0,
            skipped_channels: 0,
            error_channels: 0,
            module_type_stats: HashMap::new(),
            allocation_time: Utc::now(),
            allocation_duration_ms: 0,
        };

        Ok(TestExecutionResponse {
            batch_info: app_resp.all_batches.first().cloned().unwrap_or_else(|| request.batch_info),
            test_instances: Vec::new(),
            allocation_summary,
            timestamp: Utc::now(),
        })
    }

    async fn start_batch_test(&self, batch_id: &str) -> AppResult<()> {
        self.inner.start_batch_testing(batch_id).await
    }

    async fn pause_batch_test(&self, batch_id: &str) -> AppResult<()> {
        self.inner.pause_batch_testing(batch_id).await
    }

    async fn resume_batch_test(&self, batch_id: &str) -> AppResult<()> {
        self.inner.resume_batch_testing(batch_id).await
    }

    async fn cancel_batch_test(&self, batch_id: &str) -> AppResult<()> {
        self.inner.stop_batch_testing(batch_id).await
    }

    async fn get_batch_progress(&self, batch_id: &str) -> AppResult<TestProgressUpdate> {
        let updates = self.inner.get_batch_progress(batch_id).await?;
        let latest = updates.last().cloned().ok_or_else(|| AppError::not_found_error("batch_progress", batch_id))?;
        // Convert application progress to domain progress via JSON roundtrip (types are structurally identical)
        let value = serde_json::to_value(&latest).map_err(|e| AppError::generic(e.to_string()))?;
        let domain_progress: DomainProgress = serde_json::from_value(value).map_err(|e| AppError::generic(e.to_string()))?;
        Ok(domain_progress)
    }

    async fn get_active_batches(&self) -> AppResult<Vec<TestBatchInfo>> {
        // 保持与之前 Stub 行为一致，暂未实现
        Err(AppError::not_implemented_error("get_active_batches"))
    }

    async fn get_batch_details(&self, batch_id: &str) -> AppResult<TestBatchInfo> {
        Err(AppError::not_implemented_error(&format!("get_batch_details for {}", batch_id)))
    }

    async fn delete_batch(&self, batch_id: &str) -> AppResult<()> {
        self.inner.cleanup_completed_batch(batch_id).await
    }

    async fn start_manual_test(&self, request: StartManualTestRequest) -> AppResult<StartManualTestResponse> {
        self.inner.start_manual_test(request).await
    }

    async fn update_manual_test_subitem(
        &self,
        request: UpdateManualTestSubItemRequest,
    ) -> AppResult<UpdateManualTestSubItemResponse> {
        self.inner.update_manual_test_subitem(request).await
    }

    async fn get_manual_test_status(&self, instance_id: &str) -> AppResult<Option<ManualTestStatus>> {
        self.inner.get_manual_test_status(instance_id).await
    }
}
