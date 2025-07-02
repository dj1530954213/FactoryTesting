use async_trait::async_trait;
use std::sync::Arc;
use crate::{
    domain::services::{
        ITestOrchestrationService, TestExecutionRequest, TestExecutionResponse, TestProgressUpdate,
    },
    models::structs::{
        TestBatchInfo, StartManualTestRequest, StartManualTestResponse, UpdateManualTestSubItemRequest,
        UpdateManualTestSubItemResponse, ManualTestStatus,
    },
    utils::error::{AppError, AppResult},
};

/// A minimal placeholder implementation that fulfils `ITestOrchestrationService`.
///
/// It intentionally returns `AppError::not_implemented_error` for every method so that
/// production code compiles while the real orchestration logic is developed.
#[derive(Default)]
pub struct StubTestOrchestrationService;

#[async_trait]
impl crate::domain::services::BaseService for StubTestOrchestrationService {
    fn service_name(&self) -> &'static str {
        "StubTestOrchestrationService"
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
impl ITestOrchestrationService for StubTestOrchestrationService {
    async fn create_test_batch(&self, _request: TestExecutionRequest) -> AppResult<TestExecutionResponse> {
        Err(AppError::not_implemented_error("create_test_batch"))
    }

    async fn start_batch_test(&self, _batch_id: &str) -> AppResult<()> {
        Err(AppError::not_implemented_error("start_batch_test"))
    }

    async fn pause_batch_test(&self, _batch_id: &str) -> AppResult<()> {
        Err(AppError::not_implemented_error("pause_batch_test"))
    }

    async fn resume_batch_test(&self, _batch_id: &str) -> AppResult<()> {
        Err(AppError::not_implemented_error("resume_batch_test"))
    }

    async fn cancel_batch_test(&self, _batch_id: &str) -> AppResult<()> {
        Err(AppError::not_implemented_error("cancel_batch_test"))
    }

    async fn get_batch_progress(&self, _batch_id: &str) -> AppResult<TestProgressUpdate> {
        Err(AppError::not_implemented_error("get_batch_progress"))
    }

    async fn get_active_batches(&self) -> AppResult<Vec<TestBatchInfo>> {
        Err(AppError::not_implemented_error("get_active_batches"))
    }

    async fn get_batch_details(&self, _batch_id: &str) -> AppResult<TestBatchInfo> {
        Err(AppError::not_implemented_error("get_batch_details"))
    }

    async fn delete_batch(&self, _batch_id: &str) -> AppResult<()> {
        Err(AppError::not_implemented_error("delete_batch"))
    }

    async fn start_manual_test(&self, _request: StartManualTestRequest) -> AppResult<StartManualTestResponse> {
        Err(AppError::not_implemented_error("start_manual_test"))
    }

    async fn update_manual_test_subitem(
        &self,
        _request: UpdateManualTestSubItemRequest,
    ) -> AppResult<UpdateManualTestSubItemResponse> {
        Err(AppError::not_implemented_error("update_manual_test_subitem"))
    }

    async fn get_manual_test_status(&self, _instance_id: &str) -> AppResult<Option<ManualTestStatus>> {
        Err(AppError::not_implemented_error("get_manual_test_status"))
    }
}
