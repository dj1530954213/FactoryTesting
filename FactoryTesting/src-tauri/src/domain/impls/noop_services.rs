/*use std::sync::Arc;
use async_trait::async_trait;
use crate::utils::error::{AppResult, AppError};
use crate::models::*;
use crate::domain::services::*;

/// A minimal no-op implementation that fulfils `ITestOrchestrationService`.
pub struct NoopTestOrchestrationService;

impl NoopTestOrchestrationService {
    pub fn new() -> Self { Self }
}

#[async_trait]
impl BaseService for NoopTestOrchestrationService {
    fn service_name(&self) -> &'static str { "NoopTestOrchestrationService" }
    async fn initialize(&mut self) -> AppResult<()> { Ok(()) }
    async fn shutdown(&mut self) -> AppResult<()> { Ok(()) }
    async fn health_check(&self) -> AppResult<()> { Ok(()) }
}

#[async_trait]
impl ITestOrchestrationService for NoopTestOrchestrationService {
    async fn create_test_batch(&self, _batch: TestBatchInfo) -> AppResult<()> { Ok(()) }
    async fn start_batch(&self, _batch_id: &str) -> AppResult<()> { Ok(()) }
    async fn pause_batch(&self, _batch_id: &str) -> AppResult<()> { Ok(()) }
    async fn resume_batch(&self, _batch_id: &str) -> AppResult<()> { Ok(()) }
    async fn cancel_batch(&self, _batch_id: &str) -> AppResult<()> { Ok(()) }
    async fn start_manual_test(&self, _instance_id: &str) -> AppResult<()> { Ok(()) }
    async fn stop_manual_test(&self, _instance_id: &str) -> AppResult<()> { Ok(()) }
}

/// No-op batch allocation service.
pub struct NoopBatchAllocationService;
impl NoopBatchAllocationService { pub fn new() -> Self { Self } }

#[async_trait]
impl BaseService for NoopBatchAllocationService {
    fn service_name(&self) -> &'static str { "NoopBatchAllocationService" }
    async fn initialize(&mut self) -> AppResult<()> { Ok(()) }
    async fn shutdown(&mut self) -> AppResult<()> { Ok(()) }
    async fn health_check(&self) -> AppResult<()> { Ok(()) }
}

#[async_trait]
impl IBatchAllocationService for NoopBatchAllocationService {
    async fn allocate(&self, _strategy: AllocationStrategy, _points: Vec<ChannelPointDefinition>) -> AppResult<AllocationResult> {
        Err(AppError::not_implemented_error("allocate"))
    }
}

/// No-op event publisher
pub struct NoopEventPublisher;
impl NoopEventPublisher { pub fn new() -> Self { Self } }

#[async_trait]
impl BaseService for NoopEventPublisher {
    fn service_name(&self) -> &'static str { "NoopEventPublisher" }
    async fn initialize(&mut self) -> AppResult<()> { Ok(()) }
    async fn shutdown(&mut self) -> AppResult<()> { Ok(()) }
    async fn health_check(&self) -> AppResult<()> { Ok(()) }
}

#[async_trait]
impl IEventPublisher for NoopEventPublisher {
    async fn publish(&self, _event: crate::models::AppEvent) -> AppResult<()> { Ok(()) }
}

/// No-op persistence service implementing every method with Err(not_implemented)
pub struct NoopPersistenceService;
impl NoopPersistenceService { pub fn new() -> Self { Self } }

#[async_trait]
impl BaseService for NoopPersistenceService {
    fn service_name(&self) -> &'static str { "NoopPersistenceService" }
    async fn initialize(&mut self) -> AppResult<()> { Ok(()) }
    async fn shutdown(&mut self) -> AppResult<()> { Ok(()) }
    async fn health_check(&self) -> AppResult<()> { Ok(()) }
}

#[async_trait]
impl IPersistenceService for NoopPersistenceService {
    async fn save_channel_definition(&self, _definition: &ChannelPointDefinition) -> AppResult<()> { Err(AppError::not_implemented_error("save_channel_definition")) }
    async fn save_channel_definitions(&self, _definitions: &[ChannelPointDefinition]) -> AppResult<()> { Err(AppError::not_implemented_error("save_channel_definitions")) }
    async fn load_channel_definition(&self, _id: &str) -> AppResult<Option<ChannelPointDefinition>> { Err(AppError::not_implemented_error("load_channel_definition")) }
    async fn load_all_channel_definitions(&self) -> AppResult<Vec<ChannelPointDefinition>> { Err(AppError::not_implemented_error("load_all_channel_definitions")) }
    async fn query_channel_definitions(&self, _criteria: &QueryCriteria) -> AppResult<Vec<ChannelPointDefinition>> { Err(AppError::not_implemented_error("query_channel_definitions")) }
    async fn delete_channel_definition(&self, _id: &str) -> AppResult<()> { Err(AppError::not_implemented_error("delete_channel_definition")) }
    async fn save_test_instance(&self, _instance: &ChannelTestInstance) -> AppResult<()> { Err(AppError::not_implemented_error("save_test_instance")) }
    async fn save_test_instances(&self, _instances: &[ChannelTestInstance]) -> AppResult<()> { Err(AppError::not_implemented_error("save_test_instances")) }
    async fn load_test_instance(&self, _instance_id: &str) -> AppResult<Option<ChannelTestInstance>> { Err(AppError::not_implemented_error("load_test_instance")) }
    async fn load_all_test_instances(&self) -> AppResult<Vec<ChannelTestInstance>> { Err(AppError::not_implemented_error("load_all_test_instances")) }
    async fn load_test_instances_by_batch(&self, _batch_id: &str) -> AppResult<Vec<ChannelTestInstance>> { Err(AppError::not_implemented_error("load_test_instances_by_batch")) }
    async fn query_test_instances(&self, _criteria: &QueryCriteria) -> AppResult<Vec<ChannelTestInstance>> { Err(AppError::not_implemented_error("query_test_instances")) }
    async fn delete_test_instance(&self, _instance_id: &str) -> AppResult<()> { Err(AppError::not_implemented_error("delete_test_instance")) }
    async fn save_batch_info(&self, _batch: &TestBatchInfo) -> AppResult<()> { Err(AppError::not_implemented_error("save_batch_info")) }
    async fn load_batch_info(&self, _batch_id: &str) -> AppResult<Option<TestBatchInfo>> { Err(AppError::not_implemented_error("load_batch_info")) }
    async fn load_all_batch_info(&self) -> AppResult<Vec<TestBatchInfo>> { Err(AppError::not_implemented_error("load_all_batch_info")) }
}
*/