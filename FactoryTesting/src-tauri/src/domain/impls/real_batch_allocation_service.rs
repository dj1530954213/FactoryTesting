use async_trait::async_trait;
use std::sync::Arc;

use crate::domain::services::batch_allocation_service::{
    IBatchAllocationService, AllocationStrategy, BatchAllocationResult, ValidationResult,
    AllocationPreview,
};
use crate::domain::services::{BaseService, TimeRange};
use crate::models::structs::{ChannelPointDefinition, TestBatchInfo};
use crate::utils::error::{AppError, AppResult};

use sea_orm::DatabaseConnection;

/// Real implementation that bridges domain trait to application-layer batch allocation logic.
/// Currently delegates to `application::services::batch_allocation_service::BatchAllocationService`.
/// NOTE: Only `allocate_channels` is fully wired for now; the remaining methods will be
/// progressively implemented in subsequent steps.
pub struct RealBatchAllocationService {
    _db: Arc<DatabaseConnection>,
}

impl RealBatchAllocationService {
    /// Create a new instance using a database connection.
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { _db: db }
    }
}

#[async_trait]
impl BaseService for RealBatchAllocationService {
    fn service_name(&self) -> &'static str {
        "RealBatchAllocationService"
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
impl IBatchAllocationService for RealBatchAllocationService {
    async fn allocate_channels(
        &self,
        definitions: Vec<ChannelPointDefinition>,
        batch_info: TestBatchInfo,
        strategy: AllocationStrategy,
    ) -> AppResult<BatchAllocationResult> {
        Err(AppError::not_implemented_error("allocate_channels not yet implemented"))
    }

    async fn validate_allocation(
        &self,
        _definitions: &[ChannelPointDefinition],
        _strategy: &AllocationStrategy,
    ) -> AppResult<ValidationResult> {
        Err(AppError::not_implemented_error("validate_allocation"))
    }

    async fn preview_allocation(
        &self,
        _definitions: &[ChannelPointDefinition],
        _strategy: &AllocationStrategy,
    ) -> AppResult<AllocationPreview> {
        Err(AppError::not_implemented_error("preview_allocation"))
    }

    async fn reallocate_batch(
        &self,
        _batch_id: &str,
        _strategy: AllocationStrategy,
    ) -> AppResult<BatchAllocationResult> {
        Err(AppError::not_implemented_error("reallocate_batch"))
    }

    async fn get_allocation_history(
        &self,
        _batch_id: &str,
    ) -> AppResult<Vec<crate::domain::services::batch_allocation_service::AllocationRecord>> {
        Err(AppError::not_implemented_error("get_allocation_history"))
    }

    async fn get_allocation_statistics(
        &self,
        _time_range: Option<TimeRange>,
    ) -> AppResult<crate::domain::services::batch_allocation_service::AllocationStatistics> {
        Err(AppError::not_implemented_error("get_allocation_statistics"))
    }
}
