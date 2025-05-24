/// JSON文件持久化服务实现
/// 使用JSON文件作为数据存储后端，提供完整的持久化功能

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::sync::RwLock; // 用于异步环境下的读写锁
use chrono::{Utc, DateTime};
use std::fs; // 用于同步文件操作，如copy_directory

use crate::utils::error::{AppError, AppResult};
use crate::services::traits::{BaseService, PersistenceService}; // 确保导入基础trait
use crate::services::infrastructure::persistence::persistence_service::{ // 导入相关配置和数据结构
    PersistenceConfig,
    QueryCriteria,
    QueryResult,
    IntegrityCheckResult,
    IntegrityStatus,
    HasTimestamps, // 用于 apply_query_criteria
    ExtendedPersistenceService,
    BackupInfo,
    PersistenceStats,
    IntegrityReport,
};
use crate::models::structs::*; // 导入所有模型结构体

const CHANNEL_DEFINITIONS_DIR: &str = "channel_definitions";
const TEST_INSTANCES_DIR: &str = "test_instances";
const TEST_BATCHES_DIR: &str = "batch_info"; // 修正了之前的 "batches"
const TEST_OUTCOMES_DIR: &str = "test_outcomes";
const CONFIG_DIR: &str = "config"; // 用于 AppSettings

/// JSON文件持久化服务
/// 使用文件系统存储数据，每个实体类型使用单独的目录
#[derive(Debug)] // 添加 Debug trait
pub struct JsonPersistenceService {
    /// 配置信息
    config: PersistenceConfig,
    /// 内存缓存，提高读取性能 (可选，这里暂时简化，不实现复杂缓存逻辑)
    /// cache: RwLock<HashMap<String, String>>, // 暂时移除缓存以简化
}

impl JsonPersistenceService {
    /// 创建新的JSON持久化服务
    pub fn new(config: PersistenceConfig) -> Self {
        Self {
            config,
            // cache: RwLock::new(HashMap::new()), // 暂时移除缓存
        }
    }

    /// 获取实体类型的存储目录
    fn get_entity_dir_path(&self, entity_type: &str) -> PathBuf {
        self.config.storage_root_dir.join(entity_type)
    }

    /// 获取实体文件路径
    fn get_entity_file_path(&self, entity_type: &str, id: &str) -> PathBuf {
        self.get_entity_dir_path(entity_type).join(format!("{}.json", id))
    }

    /// 确保目录存在
    async fn ensure_directory_exists(&self, dir: &PathBuf) -> AppResult<()> {
        if !dir.exists() {
            tokio::fs::create_dir_all(dir).await
                .map_err(|e| AppError::io_error(format!("创建目录 {:?} 失败: {}", dir, e), e.kind().to_string()))?;
        }
        Ok(())
    }

    /// 保存数据到文件
    async fn save_to_file<T>(&self, entity_type: &str, id: &str, data: &T) -> AppResult<()>
    where
        T: Serialize + Send + Sync, // Send + Sync for async
    {
        let dir = self.get_entity_dir_path(entity_type);
        self.ensure_directory_exists(&dir).await?;

        let file_path = self.get_entity_file_path(entity_type, id);
        let json_content = serde_json::to_string_pretty(data)
            .map_err(|e| AppError::json_error(format!("序列化数据 (ID: {}) 到JSON失败: {}", id, e)))?;

        tokio::fs::write(&file_path, json_content).await
            .map_err(|e| AppError::io_error(format!("写入文件 {:?} 失败: {}", file_path, e), e.kind().to_string()))?;
        Ok(())
    }

    /// 从文件加载数据
    async fn load_from_file<T>(&self, entity_type: &str, id: &str) -> AppResult<Option<T>>
    where
        T: for<'de> Deserialize<'de> + Send + Sync, // Send + Sync for async
    {
        let file_path = self.get_entity_file_path(entity_type, id);
        
        if !file_path.exists() {
            return Ok(None);
        }

        let json_content = tokio::fs::read_to_string(&file_path).await
            .map_err(|e| AppError::io_error(format!("读取文件 {:?} 失败: {}", file_path, e), e.kind().to_string()))?;

        let data: T = serde_json::from_str(&json_content)
            .map_err(|e| AppError::json_error(format!("反序列化文件 {:?} 内容失败: {}", file_path, e)))?;

        Ok(Some(data))
    }

    /// 加载所有指定类型的数据
    async fn load_all_from_dir<T>(&self, entity_type: &str) -> AppResult<Vec<T>>
    where
        T: for<'de> Deserialize<'de> + Send + Sync, // Send + Sync for async
    {
        let dir = self.get_entity_dir_path(entity_type);
        
        if !dir.exists() {
            return Ok(Vec::new());
        }

        let mut results = Vec::new();
        let mut entries = tokio::fs::read_dir(&dir).await
            .map_err(|e| AppError::io_error(format!("读取目录 {:?} 失败: {}", dir, e), e.kind().to_string()))?;

        while let Some(entry) = entries.next_entry().await
            .map_err(|e| AppError::io_error(format!("遍历目录项 {:?} 失败: {}", dir, e), e.kind().to_string()))? {
            
            let path = entry.path();
            if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("json") {
                let json_content = tokio::fs::read_to_string(&path).await
                    .map_err(|e| AppError::io_error(format!("读取文件 {:?} 失败: {}", path, e), e.kind().to_string()))?;

                match serde_json::from_str::<T>(&json_content) {
                    Ok(data) => results.push(data),
                    Err(e) => {
                        // 记录错误但继续处理其他文件
                        log::error!("反序列化文件 {:?} 失败: {}. 跳过此文件.", path, e);
                    }
                }
            }
        }
        Ok(results)
    }

    /// 删除文件
    async fn delete_entity_file(&self, entity_type: &str, id: &str) -> AppResult<()> {
        let file_path = self.get_entity_file_path(entity_type, id);
        
        if file_path.exists() {
            tokio::fs::remove_file(&file_path).await
                .map_err(|e| AppError::io_error(format!("删除文件 {:?} 失败: {}", file_path, e), e.kind().to_string()))?;
        }
        Ok(())
    }
    
    /// 复制目录内容 (同步操作, 用于备份/恢复)
    fn copy_directory_contents(&self, source: &PathBuf, target: &PathBuf) -> AppResult<()> {
        if !source.exists() {
            return Ok(()); // 源目录不存在，无需操作
        }
        if !target.exists() {
            fs::create_dir_all(target)
                .map_err(|e| AppError::io_error(format!("创建目标目录 {:?} 失败: {}", target, e), e.kind().to_string()))?;
        }

        for entry_result in fs::read_dir(source)
            .map_err(|e| AppError::io_error(format!("读取源目录 {:?} 失败: {}", source, e), e.kind().to_string()))? {
            let entry = entry_result.map_err(|e| AppError::io_error(format!("读取源目录项 {:?} 失败: {}", source, e), e.kind().to_string()))?;
            let source_path = entry.path();
            let target_path = target.join(entry.file_name());

            if source_path.is_dir() {
                self.copy_directory_contents(&source_path, &target_path)?;
            } else if source_path.is_file() {
                fs::copy(&source_path, &target_path)
                    .map_err(|e| AppError::io_error(format!("复制文件 {:?} 到 {:?} 失败: {}", source_path, target_path, e), e.kind().to_string()))?;
            }
        }
        Ok(())
    }

    /// 应用查询条件过滤数据 (同步操作)
    fn apply_query_criteria<T>(&self, mut items: Vec<T>, criteria: &QueryCriteria) -> QueryResult<T>
    where
        T: HasTimestamps, 
    {
        let total_items_before_filter = items.len();

        // 时间范围过滤
        if let Some(created_after) = criteria.created_after {
            items.retain(|item| item.created_at() >= created_after);
        }
        if let Some(created_before) = criteria.created_before {
            items.retain(|item| item.created_at() <= created_before);
        }
        if let Some(updated_after) = criteria.updated_after {
            items.retain(|item| item.updated_at() >= updated_after);
        }
        if let Some(updated_before) = criteria.updated_before {
            items.retain(|item| item.updated_at() <= updated_before);
        }

        // 排序 (仅支持 created_at 和 updated_at)
        if let Some(sort_by) = &criteria.sort_by {
            match sort_by.as_str() {
                "created_at" => {
                    if criteria.sort_desc { items.sort_by(|a, b| b.created_at().cmp(&a.created_at())); } 
                    else { items.sort_by(|a, b| a.created_at().cmp(&b.created_at())); }
                }
                "updated_at" => {
                    if criteria.sort_desc { items.sort_by(|a, b| b.updated_at().cmp(&a.updated_at())); } 
                    else { items.sort_by(|a, b| a.updated_at().cmp(&b.updated_at())); }
                }
                _ => { log::warn!("Unsupported sort_by field: {}", sort_by); }
            }
        }

        let total_items_after_filter_before_paging = items.len();
        
        // 分页
        let offset = criteria.offset.unwrap_or(0);
        let limit = criteria.limit.unwrap_or(usize::MAX);
        
        let paged_items: Vec<T> = items.into_iter().skip(offset).take(limit).collect();
        let has_more = offset + paged_items.len() < total_items_after_filter_before_paging;

        QueryResult {
            items: paged_items,
            total_count: total_items_before_filter, // 返回过滤前的总数，或过滤后的总数，取决于业务需求
            has_more,
        }
    }

    // 将 verify_entity_integrity 移到这里作为 JsonPersistenceService 的一个方法
    async fn verify_entity_integrity(&self, entity_type: &str) -> AppResult<IntegrityCheckResult> {
        let dir = self.get_entity_dir_path(entity_type);
        let mut checked_files_count = 0;
        let mut corrupted_files = Vec::new();
        let mut status = IntegrityStatus::Good;

        if !dir.exists() {
            return Ok(IntegrityCheckResult {
                check_name: entity_type.to_string(),
                status: IntegrityStatus::Warning,
                message: format!("实体目录 {:?} 不存在.", dir),
                details: None,
                affected_items: Vec::new(),
            });
        }

        let mut entries = tokio::fs::read_dir(&dir).await
            .map_err(|e| AppError::io_error(format!("读取实体目录 {:?} 失败: {}", dir, e), e.kind().to_string()))?;
        
        while let Some(entry) = entries.next_entry().await
            .map_err(|e| AppError::io_error(format!("遍历实体目录项 {:?} 失败: {}", dir, e), e.kind().to_string()))? {
            let path = entry.path();
            if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("json") {
                checked_files_count += 1;
                match tokio::fs::read_to_string(&path).await {
                    Ok(content_string) => {
                        if serde_json::from_str::<serde_json::Value>(&content_string).is_err() {
                            corrupted_files.push(path.to_string_lossy().into_owned());
                            status = IntegrityStatus::Error;
                        }
                    }
                    Err(e) => {
                        corrupted_files.push(format!("{} (读取错误: {})", path.to_string_lossy(), e));
                        status = IntegrityStatus::Error;
                    }
                }
            }
        }

        let message = if corrupted_files.is_empty() {
            format!("检查了 {} 个文件，未发现损坏.", checked_files_count)
        } else {
            format!("检查了 {} 个文件，发现 {} 个可能已损坏的文件.", checked_files_count, corrupted_files.len())
        };

        Ok(IntegrityCheckResult {
            check_name: entity_type.to_string(),
            status,
            message,
            details: if corrupted_files.is_empty() { None } else { Some(format!("损坏文件列表: {:?}", corrupted_files)) },
            affected_items: corrupted_files,
        })
    }
}

#[async_trait]
impl BaseService for JsonPersistenceService {
    fn service_name(&self) -> &'static str {
        "JsonPersistenceService"
    }

    async fn initialize(&mut self) -> AppResult<()> {
        self.ensure_directory_exists(&self.config.storage_root_dir).await?;
        log::info!("{} initialized. Storage root: {:?}", self.service_name(), self.config.storage_root_dir);
        Ok(())
    }

    async fn shutdown(&mut self) -> AppResult<()> {
        log::info!("{} shutting down.", self.service_name());
        // 对于JSON服务，关闭时可能不需要太多操作
        Ok(())
    }

    async fn health_check(&self) -> AppResult<()> {
        if self.config.storage_root_dir.exists() && self.config.storage_root_dir.is_dir() {
            Ok(())
        } else {
            Err(AppError::PersistenceError{ message: format!("存储根目录 {:?} 不可访问或不是目录", self.config.storage_root_dir) })
        }
    }
}

#[async_trait]
impl PersistenceService for JsonPersistenceService {
    async fn save_channel_definition(&self, definition: &ChannelPointDefinition) -> AppResult<()> {
        self.save_to_file(CHANNEL_DEFINITIONS_DIR, &definition.id, definition).await
    }

    async fn load_channel_definition(&self, id: &str) -> AppResult<Option<ChannelPointDefinition>> {
        self.load_from_file(CHANNEL_DEFINITIONS_DIR, id).await
    }

    async fn load_all_channel_definitions(&self) -> AppResult<Vec<ChannelPointDefinition>> {
        self.load_all_from_dir(CHANNEL_DEFINITIONS_DIR).await
    }

    async fn delete_channel_definition(&self, id: &str) -> AppResult<()> {
        self.delete_entity_file(CHANNEL_DEFINITIONS_DIR, id).await
    }

    async fn save_test_instance(&self, instance: &ChannelTestInstance) -> AppResult<()> {
        self.save_to_file(TEST_INSTANCES_DIR, &instance.instance_id, instance).await
    }

    async fn load_test_instance(&self, instance_id: &str) -> AppResult<Option<ChannelTestInstance>> {
        self.load_from_file(TEST_INSTANCES_DIR, instance_id).await
    }

    async fn load_test_instances_by_batch(&self, batch_id: &str) -> AppResult<Vec<ChannelTestInstance>> {
        let all_instances = self.load_all_from_dir::<ChannelTestInstance>(TEST_INSTANCES_DIR).await?;
        Ok(all_instances.into_iter().filter(|inst| inst.test_batch_id == batch_id).collect())
    }

    async fn delete_test_instance(&self, instance_id: &str) -> AppResult<()> {
        self.delete_entity_file(TEST_INSTANCES_DIR, instance_id).await
    }

    async fn save_batch_info(&self, batch: &TestBatchInfo) -> AppResult<()> {
        self.save_to_file(TEST_BATCHES_DIR, &batch.batch_id, batch).await
    }

    async fn load_batch_info(&self, batch_id: &str) -> AppResult<Option<TestBatchInfo>> {
        self.load_from_file(TEST_BATCHES_DIR, batch_id).await
    }

    async fn load_all_batch_info(&self) -> AppResult<Vec<TestBatchInfo>> {
        self.load_all_from_dir(TEST_BATCHES_DIR).await
    }

    async fn delete_batch_info(&self, batch_id: &str) -> AppResult<()> {
        self.delete_entity_file(TEST_BATCHES_DIR, batch_id).await
    }

    async fn save_test_outcome(&self, outcome: &RawTestOutcome) -> AppResult<()> {
        // Outcomes might be numerous; consider a naming strategy if they need individual retrieval by a unique ID.
        // For now, using instance_id + sub_test_item + timestamp for some uniqueness if saved as separate files.
        // However, they are often queried by instance_id. Storing them in a sub-folder per instance might be better.
        // Current trait implies saving one by one.
        let outcome_file_id = format!("{}_{:?}_{}", outcome.channel_instance_id, outcome.sub_test_item, outcome.end_time.timestamp_millis());
        self.save_to_file(TEST_OUTCOMES_DIR, &outcome_file_id, outcome).await
    }

    async fn load_test_outcomes_by_instance(&self, instance_id: &str) -> AppResult<Vec<RawTestOutcome>> {
        let all_outcomes = self.load_all_from_dir::<RawTestOutcome>(TEST_OUTCOMES_DIR).await?;
        Ok(all_outcomes.into_iter().filter(|o| o.channel_instance_id == instance_id).collect())
    }

    async fn load_test_outcomes_by_batch(&self, batch_id: &str) -> AppResult<Vec<RawTestOutcome>> {
        let instances = self.load_test_instances_by_batch(batch_id).await?;
        let mut results = Vec::new();
        for instance in instances {
            results.extend(self.load_test_outcomes_by_instance(&instance.instance_id).await?);
        }
        Ok(results)
    }
    
    async fn save_app_settings(&self, settings: &AppSettings) -> AppResult<()> {
        self.save_to_file(CONFIG_DIR, "app_settings", settings).await
    }

    async fn load_app_settings(&self) -> AppResult<Option<AppSettings>> {
        self.load_from_file(CONFIG_DIR, "app_settings").await
    }

    async fn backup_data(&self, target_backup_dir: &PathBuf) -> AppResult<()> {
        log::info!("Backing up data from {:?} to {:?}", &self.config.storage_root_dir, target_backup_dir);
        self.copy_directory_contents(&self.config.storage_root_dir, target_backup_dir)
    }

    async fn restore_data(&self, source_backup_dir: &PathBuf) -> AppResult<()> {
        if !source_backup_dir.exists() {
            return Err(AppError::io_error(format!("备份源 {:?} 不存在", source_backup_dir), "NotFound".to_string()));
        }
        log::info!("Restoring data from {:?} to {:?}", source_backup_dir, &self.config.storage_root_dir);
        // Potentially dangerous: clear current data first? Or merge?
        // For a simple restore, we might clear and copy.
        // Ensure target root exists
        self.ensure_directory_exists(&self.config.storage_root_dir).await?;
        self.copy_directory_contents(source_backup_dir, &self.config.storage_root_dir)
    }
}

#[async_trait]
impl ExtendedPersistenceService for JsonPersistenceService {
    async fn query_channel_definitions(&self, criteria: &QueryCriteria) -> AppResult<QueryResult<ChannelPointDefinition>> {
        let all_items = self.load_all_channel_definitions().await?;
        Ok(self.apply_query_criteria(all_items, criteria))
    }

    async fn query_test_instances(&self, criteria: &QueryCriteria) -> AppResult<QueryResult<ChannelTestInstance>> {
        // This is inefficient for JSON files unless criteria include batch_id or similar to narrow down.
        // A full scan and filter is very costly.
        // For now, we load ALL instances and then filter.
        let all_items = self.load_all_from_dir::<ChannelTestInstance>(TEST_INSTANCES_DIR).await?;
        Ok(self.apply_query_criteria(all_items, criteria))
    }

    async fn query_test_batches(&self, criteria: &QueryCriteria) -> AppResult<QueryResult<TestBatchInfo>> {
        let all_items = self.load_all_batch_info().await?;
        Ok(self.apply_query_criteria(all_items, criteria))
    }

    async fn query_test_outcomes(&self, criteria: &QueryCriteria) -> AppResult<QueryResult<RawTestOutcome>> {
        // Similar to instances, querying all outcomes is very inefficient.
        let all_items = self.load_all_from_dir::<RawTestOutcome>(TEST_OUTCOMES_DIR).await?;
        Ok(self.apply_query_criteria(all_items, criteria))
    }

    async fn batch_save_channel_definitions(&self, definitions: &[ChannelPointDefinition]) -> AppResult<()> {
        for definition in definitions {
            self.save_channel_definition(definition).await?;
        }
        Ok(())
    }

    async fn batch_save_test_instances(&self, instances: &[ChannelTestInstance]) -> AppResult<()> {
        for instance in instances {
            self.save_test_instance(instance).await?;
        }
        Ok(())
    }

    async fn batch_save_test_outcomes(&self, outcomes: &[RawTestOutcome]) -> AppResult<()> {
        for outcome in outcomes {
            self.save_test_outcome(outcome).await?;
        }
        Ok(())
    }

    async fn batch_delete_by_ids(&self, entity_type_str: &str, ids: &[String]) -> AppResult<()> {
        for id in ids {
            // Map string to actual directory constant to avoid typos
            let entity_dir_const = match entity_type_str {
                "channel_definitions" => CHANNEL_DEFINITIONS_DIR,
                "test_instances" => TEST_INSTANCES_DIR,
                "batch_info" => TEST_BATCHES_DIR,
                // Deleting outcomes by ID might not be primary use case, usually by instance/batch
                _ => return Err(AppError::validation_error(format!("Unsupported entity type for batch delete: {}", entity_type_str)))
            };
            self.delete_entity_file(entity_dir_const, id).await?;
        }
        Ok(())
    }

    async fn create_backup(&self, backup_name_opt: Option<String>) -> AppResult<PathBuf> {
        let timestamp = Utc::now().format("%Y%m%d_%H%M%S").to_string();
        let name = backup_name_opt.unwrap_or_else(|| format!("backup_{}", timestamp));
        
        let backup_root_dir = self.config.storage_root_dir.join("_backups");
        self.ensure_directory_exists(&backup_root_dir).await?;
        let specific_backup_path = backup_root_dir.join(name);
        self.ensure_directory_exists(&specific_backup_path).await?;

        self.copy_directory_contents(&self.config.storage_root_dir, &specific_backup_path)?;
        Ok(specific_backup_path)
    }

    async fn restore_from_backup(&self, backup_path: &PathBuf) -> AppResult<()> {
        self.restore_data(backup_path).await // Uses the method from PersistenceService
    }

    async fn list_backups(&self) -> AppResult<Vec<BackupInfo>> {
        let backup_root_dir = self.config.storage_root_dir.join("_backups");
        if !backup_root_dir.exists() { return Ok(Vec::new()); }

        let mut backups = Vec::new();
        let mut entries = tokio::fs::read_dir(backup_root_dir).await
            .map_err(|e| AppError::io_error(format!("读取备份目录失败: {}", e), e.kind().to_string()))?;

        while let Some(entry) = entries.next_entry().await.map_err(|e| AppError::io_error(format!("读取备份条目失败: {}", e),e.kind().to_string()))? {
            let path = entry.path();
            if path.is_dir() { // Each backup is a directory
                let metadata = fs::metadata(&path) // fs::metadata is sync
                    .map_err(|e| AppError::io_error(format!("读取备份元数据 {:?} 失败: {}", path, e), e.kind().to_string()))?;
                let created_at: DateTime<Utc> = metadata.created().map(DateTime::from).unwrap_or_else(|_| Utc::now());
                
                // Simplistic size calculation (sum of file sizes in backup dir, non-recursive for now)
                let mut size_bytes = 0;
                // if let Ok(mut backup_entries) = tokio::fs::read_dir(&path).await {
                //     while let Ok(Some(file_entry)) = backup_entries.next_entry().await {
                //         if let Ok(file_meta) = file_entry.metadata().await {
                //             if file_meta.is_file() {
                //                 size_bytes += file_meta.len();
                //             }
                //         }
                //     }
                // }
                 // For simplicity, size calculation is deferred or approximated. A full recursive sum is complex.

                backups.push(BackupInfo {
                    name: path.file_name().unwrap_or_default().to_string_lossy().into_owned(),
                    path: path.clone(),
                    created_at,
                    size_bytes, // Placeholder
                    description: None, // Could store a meta file inside backup for description
                    is_auto_backup: path.file_name().unwrap_or_default().to_string_lossy().starts_with("auto_backup_"),
                });
            }
        }
        Ok(backups)
    }

    async fn cleanup_old_backups(&self) -> AppResult<u32> {
        if !self.config.enable_auto_backup || self.config.backup_retention_days == 0 {
            return Ok(0);
        }
        let backups = self.list_backups().await?;
        let mut deleted_count = 0;
        let now = Utc::now();
        let retention_duration = chrono::Duration::days(self.config.backup_retention_days as i64);

        for backup in backups {
            if backup.is_auto_backup && (now - backup.created_at > retention_duration) {
                if fs::remove_dir_all(&backup.path).is_ok() { // fs::remove_dir_all is sync
                    deleted_count += 1;
                    log::info!("Deleted old backup: {:?}", backup.path);
                } else {
                    log::warn!("Failed to delete old backup: {:?}", backup.path);
                }
            }
        }
        Ok(deleted_count)
    }
    
    async fn verify_data_integrity(&self) -> AppResult<IntegrityReport> {
        let entity_types = [
            CHANNEL_DEFINITIONS_DIR, 
            TEST_INSTANCES_DIR, 
            TEST_BATCHES_DIR, 
            TEST_OUTCOMES_DIR,
            CONFIG_DIR,
        ];
        let mut all_details = Vec::new();
        let mut overall_status = IntegrityStatus::Good;
        let mut total_issues_count = 0;

        for entity_type_str in &entity_types {
            // 现在调用 JsonPersistenceService 自身的 verify_entity_integrity 方法
            let result = self.verify_entity_integrity(entity_type_str).await?;
            if result.status != IntegrityStatus::Good {
                total_issues_count += result.affected_items.len() as u32;
                if result.status == IntegrityStatus::Critical { overall_status = IntegrityStatus::Critical; }
                else if result.status == IntegrityStatus::Error && overall_status != IntegrityStatus::Critical { overall_status = IntegrityStatus::Error; }
                else if result.status == IntegrityStatus::Warning && overall_status == IntegrityStatus::Good { overall_status = IntegrityStatus::Warning; }
            }
            all_details.push(result);
        }
        
        Ok(IntegrityReport {
            checked_at: Utc::now(),
            overall_status,
            details: all_details,
            issues_count: total_issues_count,
            repair_suggestions: Vec::new(),
        })
    }

    async fn get_statistics(&self) -> AppResult<PersistenceStats> {
        let defs = self.load_all_channel_definitions().await?.len();
        let insts = self.load_all_from_dir::<ChannelTestInstance>(TEST_INSTANCES_DIR).await?.len();
        let batches = self.load_all_batch_info().await?.len();
        let outcomes = self.load_all_from_dir::<RawTestOutcome>(TEST_OUTCOMES_DIR).await?.len();
        
        // Simplistic total_storage_size_bytes - sum of sizes of top-level entity dirs
        // A more accurate calculation would involve recursing through all files.
        let mut total_size = 0;
        // let root_dir_entries = tokio::fs::read_dir(&self.config.storage_root_dir).await.ok();
        // if let Some(mut entries) = root_dir_entries {
        //     while let Ok(Some(entry)) = entries.next_entry().await {
        //         if let Ok(meta) = entry.metadata().await {
        //            // This is not recursive, just size of entries in root.
        //            // total_size += meta.len();
        //         }
        //     }
        // }
        // For now, returning 0 or a placeholder
        
        Ok(PersistenceStats {
            channel_definitions_count: defs,
            test_instances_count: insts,
            test_batches_count: batches,
            test_outcomes_count: outcomes,
            total_storage_size_bytes: total_size, 
            last_backup_time: None, // To implement: find latest backup from list_backups
            last_integrity_check_time: None, // To implement: store timestamp after verify_data_integrity
        })
    }

    fn get_config(&self) -> &PersistenceConfig {
        &self.config
    }

    async fn update_config(&mut self, config: PersistenceConfig) -> AppResult<()> {
        self.config = config;
        // May require re-initialization or other actions if paths change, etc.
        log::info!("Persistence config updated.");
        Ok(())
    }

    async fn cleanup_expired_data(&self, retention_days: u32) -> AppResult<u32> {
        log::warn!("cleanup_expired_data is not fully implemented for JsonPersistenceService. It might be slow or incomplete.");
        // This is a complex operation for JSON file storage.
        // It would involve iterating through all files, parsing their timestamps, and deleting if expired.
        // For simplicity, returning 0 deleted items.
        // A proper implementation would need to define what "expired" means for each entity type.
        // For example, TestOutcomes or TestInstances older than X days.
        
        let mut deleted_count = 0;
        let now = Utc::now();
        let retention_duration = chrono::Duration::days(retention_days as i64);

        // Example for TestInstances (assuming they have `last_updated_time`)
        let instances_dir = self.get_entity_dir_path(TEST_INSTANCES_DIR);
        if instances_dir.exists() {
            let mut entries = tokio::fs::read_dir(instances_dir).await
                .map_err(|e| AppError::io_error(format!("Failed to read instances dir for cleanup: {}", e), e.kind().to_string()))?;
            while let Some(entry) = entries.next_entry().await.map_err(|e| AppError::io_error(format!("Failed to read instance entry for cleanup: {}", e),e.kind().to_string()))? {
                let path = entry.path();
                if path.is_file() {
                    if let Ok(instance_content) = tokio::fs::read_to_string(&path).await {
                        if let Ok(instance) = serde_json::from_str::<ChannelTestInstance>(&instance_content) {
                            if (now - instance.last_updated_time) > retention_duration {
                                if tokio::fs::remove_file(&path).await.is_ok() {
                                    deleted_count += 1;
                                }
                            }
                        }
                    }
                }
            }
        }
        // Similar logic would be needed for other relevant entity types.
        Ok(deleted_count)
    }

    async fn compact_storage(&self) -> AppResult<u64> {
        Err(AppError::not_implemented_error("compact_storage not meaningfully implemented for JsonPersistenceService".to_string()))
    }

    async fn rebuild_indexes(&self) -> AppResult<()> {
        Err(AppError::not_implemented_error("rebuild_indexes not implemented for JsonPersistenceService".to_string()))
    }
} 