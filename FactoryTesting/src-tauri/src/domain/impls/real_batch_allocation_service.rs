use async_trait::async_trait;
use std::sync::Arc;

use crate::domain::services::batch_allocation_service::{
    IBatchAllocationService, AllocationStrategy, BatchAllocationResult, ValidationResult,
    AllocationPreview, ValidationError, ValidationWarning, ValidationSuggestion,
};
use crate::domain::services::{BaseService, TimeRange};
use crate::models::structs::{ChannelPointDefinition, TestBatchInfo};
use crate::models::structs::ChannelTestInstance;
use crate::domain::services::test_orchestration_service::AllocationSummary;
use chrono::{Utc, DateTime};
use std::collections::HashMap;
use crate::utils::error::{AppError, AppResult};

use sea_orm::{DatabaseConnection, Statement, DatabaseBackend, ConnectionTrait, TryGetable, EntityTrait};
use crate::application::services::batch_allocation_service as app_service;
use crate::models::entities::test_batch_info;

/// Real implementation that bridges domain trait to application-layer batch allocation logic.
/// Currently delegates to `application::services::batch_allocation_service::BatchAllocationService`.
/// NOTE: Only `allocate_channels` is fully wired for now; the remaining methods will be
/// progressively implemented in subsequent steps.
pub struct RealBatchAllocationService {
    db: Arc<DatabaseConnection>,
}

impl RealBatchAllocationService {
    /// 将领域层策略转换为应用层策略
    fn convert_strategy(domain: &crate::domain::services::batch_allocation_service::AllocationStrategy) -> app_service::AllocationStrategy {
        use app_service::AllocationStrategy as AS;
        match domain.name.as_str() {
            "ByModuleType" => AS::ByModuleType,
            "ByStation" => AS::ByStation,
            "ByProductModel" => AS::ByProductModel,
            _ => AS::Smart,
        }
    }

    /// 将应用层分配摘要转换为领域层分配摘要（简化版）
    fn convert_summary(app: &app_service::AllocationSummary) -> AllocationSummary {
        AllocationSummary {
            total_channels: app.total_channels as u32,
            allocated_channels: 0,
            skipped_channels: 0,
            error_channels: 0,
            module_type_stats: HashMap::new(),
            allocation_time: Utc::now(),
            allocation_duration_ms: 0,
        }
    }
    /// Create a new instance using a database connection.
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
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
        let _ = definitions; // 目前忽略显式定义列表，后续可扩展

        // 构建应用层服务
        let app_service = app_service::BatchAllocationService::new(self.db.clone());
        let app_strategy = Self::convert_strategy(&strategy);

        // 调用应用层逻辑
        let app_result = app_service
            .create_test_batch_with_full_info(batch_info.clone(), app_strategy, None)
            .await
            .map_err(|e| AppError::persistence_error(format!("创建批次失败: {}", e)))?;

        // 转换摘要
        let domain_summary = Self::convert_summary(&app_result.allocation_summary);

        Ok(BatchAllocationResult {
            batch_info: app_result.batch_info,
            test_instances: app_result.test_instances,
            allocation_summary: domain_summary.clone(),
            allocation_time: domain_summary.allocation_time,
            allocation_duration_ms: domain_summary.allocation_duration_ms,
            warnings: Vec::new(),
            skipped_definitions: Vec::new(),
        })
    }

    async fn validate_allocation(
        &self,
        definitions: &[ChannelPointDefinition],
        strategy: &AllocationStrategy,
    ) -> AppResult<ValidationResult> {
        // Very lightweight validation just to enable real flow.
        // 1. Ensure we have at least one definition.
        let mut errors = Vec::<ValidationError>::new();
        let mut warnings = Vec::<ValidationWarning>::new();
        let suggestions = Vec::<ValidationSuggestion>::new();

        if definitions.is_empty() {
            errors.push(ValidationError {
                code: "EMPTY_DEFINITIONS".into(),
                message: "未提供任何通道定义".into(),
                definition_id: None,
            });
        }

        // Example rule: if max_batch_size is set but smaller than definitions count, warn.
        if let Some(max) = strategy.max_batch_size {
            if definitions.len() as u32 > max {
                warnings.push(ValidationWarning {
                    code: "EXCEEDS_MAX_BATCH".into(),
                    message: format!("定义数量({}) 超过了策略限定的最大批次大小({})", definitions.len(), max),
                    definition_id: None,
                });
            }
        }

        let is_valid = errors.is_empty();

        Ok(ValidationResult {
            is_valid,
            errors,
            warnings,
            suggestions,
        })
    }

    async fn preview_allocation(
        &self,
        definitions: &[ChannelPointDefinition],
        _strategy: &AllocationStrategy,
    ) -> AppResult<AllocationPreview> {
        use std::collections::HashMap;
        // Compute simple breakdown by module type string representation.
        let mut module_type_breakdown: HashMap<String, u32> = HashMap::new();
        for def in definitions {
            let key = format!("{:?}", def.module_type);
            *module_type_breakdown.entry(key).or_insert(0) += 1;
        }

        Ok(AllocationPreview {
            estimated_allocations: definitions.len() as u32,
            estimated_skips: 0,
            module_type_breakdown,
            estimated_duration_ms: 0,
            resource_usage: crate::domain::services::batch_allocation_service::ResourceUsageEstimate {
                estimated_memory_bytes: 0,
                estimated_cpu_usage: 0.0,
                estimated_network_bandwidth: 0,
                estimated_storage_bytes: 0,
            },
        })
    }

    async fn reallocate_batch(
        &self,
        batch_id: &str,
        strategy: AllocationStrategy,
    ) -> AppResult<BatchAllocationResult> {
        // 1. 读取批次信息
        let model = test_batch_info::Entity::find_by_id(batch_id.to_string())
            .one(&*self.db)
            .await
            .map_err(|e| AppError::persistence_error(format!("查询批次信息失败: {}", e)))?;
        let batch_info_model = model.ok_or_else(|| AppError::not_found_error("TestBatch", "批次不存在"))?;
        let batch_info: TestBatchInfo = (&batch_info_model).into();

        // 2. 构建应用层分配服务
        let app_service = app_service::BatchAllocationService::new(self.db.clone());

        // 3. 策略转换
        let app_strategy = Self::convert_strategy(&strategy);

        // 4. 调用应用层重新创建批次（按照完整信息路径）
        let app_result = app_service
            .create_test_batch_with_full_info(batch_info.clone(), app_strategy, None)
            .await
            .map_err(|e| AppError::persistence_error(format!("应用层重新分配失败: {}", e)))?;

        // 5. 转换分配摘要
        let domain_summary = Self::convert_summary(&app_result.allocation_summary);

        // 6. 组装领域层结果
        let domain_result = BatchAllocationResult {
            batch_info: app_result.batch_info,
            test_instances: app_result.test_instances,
            allocation_summary: domain_summary.clone(),
            allocation_time: domain_summary.allocation_time,
            allocation_duration_ms: domain_summary.allocation_duration_ms,
            warnings: Vec::new(),
            skipped_definitions: Vec::new(),
        };

        Ok(domain_result)
    }

    async fn get_allocation_history(
        &self,
        batch_id: &str,
    ) -> AppResult<Vec<crate::domain::services::batch_allocation_service::AllocationRecord>> {
        use crate::domain::services::{
    batch_allocation_service::{AllocationRecord, AllocationStrategy as DomainStrategy, AllocationMode, BatchAllocationResult, SkippedDefinition, ResourceUsageEstimate},
    test_orchestration_service::AllocationSummary as DomainAllocationSummary,
};
        use serde_json;
        use chrono::TimeZone;
        use std::collections::HashMap;

        let sql = "SELECT id, batch_id, strategy, summary_json, operator_name, created_time FROM allocation_records WHERE batch_id = ? ORDER BY created_time DESC";
        let rows = self.db.query_all(Statement::from_sql_and_values(
            DatabaseBackend::Sqlite,
            sql,
            vec![batch_id.into()],
        ))
        .await
        .map_err(|e| AppError::persistence_error(format!("查询分配记录失败: {}", e)))?;

        let mut records = Vec::new();
        for row in rows {
            let id: String = row.try_get("", "id").unwrap_or_default();
            let batch_id_val: String = row.try_get("", "batch_id").unwrap_or_default();
            let strategy_str: String = row.try_get("", "strategy").unwrap_or_default();
            let summary_json: String = row.try_get("", "summary_json").unwrap_or_default();
            let operator_name: String = row.try_get("", "operator_name").unwrap_or_default();
            let created_time_str: String = row.try_get("", "created_time").unwrap_or_default();

            // 反序列化分配摘要
            let allocation_summary: DomainAllocationSummary = serde_json::from_str(&summary_json).unwrap_or_else(|_| DomainAllocationSummary {
                total_channels: 0,
                allocated_channels: 0,
                skipped_channels: 0,
                error_channels: 0,
                module_type_stats: HashMap::new(),
                allocation_time: Utc::now(),
                allocation_duration_ms: 0,
            });

            // 构造占位 BatchAllocationResult （没有测试实例等详细信息）
            let mut placeholder_batch = TestBatchInfo::new(None, None);
            placeholder_batch.batch_id = batch_id_val.clone();
            let batch_allocation_result = BatchAllocationResult {
                batch_info: placeholder_batch,
                test_instances: Vec::<ChannelTestInstance>::new(),
                allocation_summary: allocation_summary.clone(),
                allocation_time: allocation_summary.allocation_time,
                allocation_duration_ms: allocation_summary.allocation_duration_ms,
                warnings: Vec::new(),
                skipped_definitions: Vec::<SkippedDefinition>::new(),
            };

            // 构造简化策略对象
            let strategy = DomainStrategy {
                name: strategy_str.clone(),
                mode: AllocationMode::Sequential,
                priority_rules: Vec::new(),
                grouping_rules: Vec::new(),
                filter_rules: Vec::new(),
                max_batch_size: None,
                allow_partial_allocation: true,
                allocation_timeout_ms: 0,
            };

            // 解析创建时间
            let created_at: DateTime<Utc> = DateTime::parse_from_rfc3339(&created_time_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now());

            records.push(AllocationRecord {
                id,
                batch_id: batch_id_val,
                strategy,
                result: batch_allocation_result,
                operator: if operator_name.is_empty() { None } else { Some(operator_name) },
                created_at,
            });
        }
        Ok(records)
    }

    async fn get_allocation_statistics(
        &self,
        time_range: Option<TimeRange>,
    ) -> AppResult<crate::domain::services::batch_allocation_service::AllocationStatistics> {
        use crate::domain::services::batch_allocation_service::{AllocationStatistics, TimeRange as DomainTimeRange};
        use serde_json;
        use std::collections::HashMap;

        // 动态构造 SQL
        let mut sql = "SELECT strategy, summary_json FROM allocation_records".to_string();
        let mut values: Vec<sea_orm::Value> = Vec::new();
        if let Some(ref tr) = time_range {
            sql.push_str(" WHERE created_time >= ? AND created_time <= ?");
            values.push(tr.start.to_rfc3339().into());
            values.push(tr.end.to_rfc3339().into());
        }

        let rows = self.db.query_all(Statement::from_sql_and_values(
            DatabaseBackend::Sqlite,
            sql,
            values,
        ))
        .await
        .map_err(|e| AppError::persistence_error(format!("查询分配统计失败: {}", e)))?;

        let mut total_allocations: u64 = 0;
        let mut total_channels: u64 = 0;
        let mut strategy_count: HashMap<String, u64> = HashMap::new();
        // 模块类型聚合统计 (module_type -> (total, successful))
        let mut module_type_agg: HashMap<String, (u64, u64)> = HashMap::new();

        for row in rows {
            total_allocations += 1;
            let strategy_str: String = row.try_get("", "strategy").unwrap_or_default();
            *strategy_count.entry(strategy_str).or_insert(0) += 1;

            let summary_json: String = row.try_get("", "summary_json").unwrap_or_default();
            if let Ok(summary) = serde_json::from_str::<crate::application::services::batch_allocation_service::AllocationSummary>(&summary_json) {
                total_channels += summary.total_channels as u64;

                // 聚合各模块类型计数
                let update_stat = |map: &mut HashMap<String, (u64, u64)>, key: &str, count: usize| {
                    let entry = map.entry(key.to_string()).or_insert((0, 0));
                    entry.0 += count as u64; // total allocated
                    entry.1 += count as u64; // 成功计数同 total（暂不区分失败）
                };

                update_stat(&mut module_type_agg, "AI", summary.ai_channels);
                update_stat(&mut module_type_agg, "AO", summary.ao_channels);
                update_stat(&mut module_type_agg, "DI", summary.di_channels);
                update_stat(&mut module_type_agg, "DO", summary.do_channels);
            }
        }

        let average_batch_size = if total_allocations > 0 {
            total_channels as f64 / total_allocations as f64
        } else {
            0.0
        };

        let most_used_strategy = strategy_count.into_iter().max_by_key(|(_, c)| *c).map(|(s, _)| s);

        let stats = AllocationStatistics {
            total_allocations,
            successful_allocations: total_allocations, // 目前默认全部成功
            average_allocation_time_ms: 0.0,
            average_batch_size,
            most_used_strategy,
            module_type_stats: module_type_agg.into_iter().map(|(module_type, (total, successful))| {
                (
                    module_type.clone(),
                    crate::domain::services::batch_allocation_service::ModuleTypeAllocationStats {
                        module_type,
                        total_allocated: total,
                        successful_allocated: successful,
                        average_time_ms: 0.0,
                    },
                )
            }).collect(),
            time_range: time_range.unwrap_or_else(|| DomainTimeRange { start: Utc::now(), end: Utc::now() }),
        };

        Ok(stats)
    }
}
