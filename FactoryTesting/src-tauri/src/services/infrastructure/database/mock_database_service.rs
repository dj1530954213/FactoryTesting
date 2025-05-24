/// JSON文件持久化服务实现
/// 使用JSON文件作为数据存储后端，提供完整的持久化功能

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::sync::RwLock;
use std::sync::Arc;
use uuid::Uuid;
use chrono::{DateTime, Utc};

use crate::utils::error::{AppError, AppResult};
use crate::services::traits::{BaseService, PersistenceService};
use crate::services::infrastructure::persistence::persistence_service::*;
use crate::models::structs::*;
use crate::models::enums::*;

/// JSON文件持久化服务
/// 使用文件系统存储数据，每个实体类型使用单独的目录
pub struct JsonPersistenceService {
    /// 配置信息
    config: PersistenceConfig,
    /// 内存缓存，提高读取性能
    cache: RwLock<HashMap<String, String>>,
}

impl JsonPersistenceService {
    /// 创建新的JSON持久化服务
    pub fn new(config: PersistenceConfig) -> Self {
        Self {
            config,
            cache: RwLock::new(HashMap::new()),
        }
    }

    /// 获取实体类型的存储目录
    fn get_entity_dir(&self, entity_type: &str) -> PathBuf {
        self.config.storage_root_dir.join(entity_type)
    }

    /// 获取实体文件路径
    fn get_entity_file_path(&self, entity_type: &str, id: &str) -> PathBuf {
        self.get_entity_dir(entity_type).join(format!("{}.json", id))
    }

    /// 确保目录存在
    async fn ensure_directory_exists(&self, dir: &PathBuf) -> AppResult<()> {
        if !dir.exists() {
            tokio::fs::create_dir_all(dir).await
                .map_err(|e| crate::utils::error::AppError::io_error(format!("创建目录失败: {}", e)))?;
        }
        Ok(())
    }

    /// 保存数据到文件
    async fn save_to_file<T>(&self, entity_type: &str, id: &str, data: &T) -> AppResult<()>
    where
        T: Serialize,
    {
        let dir = self.get_entity_dir(entity_type);
        self.ensure_directory_exists(&dir).await?;

        let file_path = self.get_entity_file_path(entity_type, id);
        let json_content = serde_json::to_string_pretty(data)
            .map_err(|e| crate::utils::error::AppError::serialization_error(format!("序列化失败: {}", e)))?;

        tokio::fs::write(&file_path, json_content).await
            .map_err(|e| crate::utils::error::AppError::io_error(format!("写入文件失败: {}", e)))?;

        // 更新缓存
        let cache_key = format!("{}:{}", entity_type, id);
        if let Ok(mut cache) = self.cache.write() {
            cache.insert(cache_key, file_path.to_string_lossy().to_string());
        }

        Ok(())
    }

    /// 从文件加载数据
    async fn load_from_file<T>(&self, entity_type: &str, id: &str) -> AppResult<Option<T>>
    where
        T: for<'de> Deserialize<'de>,
    {
        let file_path = self.get_entity_file_path(entity_type, id);
        
        if !file_path.exists() {
            return Ok(None);
        }

        let json_content = tokio::fs::read_to_string(&file_path).await
            .map_err(|e| crate::utils::error::AppError::io_error(format!("读取文件失败: {}", e)))?;

        let data: T = serde_json::from_str(&json_content)
            .map_err(|e| crate::utils::error::AppError::serialization_error(format!("反序列化失败: {}", e)))?;

        Ok(Some(data))
    }

    /// 加载所有指定类型的数据
    async fn load_all<T>(&self, entity_type: &str) -> AppResult<Vec<T>>
    where
        T: for<'de> Deserialize<'de>,
    {
        let dir = self.get_entity_dir(entity_type);
        
        if !dir.exists() {
            return Ok(Vec::new());
        }

        let mut results = Vec::new();
        let mut entries = tokio::fs::read_dir(&dir).await
            .map_err(|e| crate::utils::error::AppError::io_error(format!("读取目录失败: {}", e)))?;

        while let Some(entry) = entries.next_entry().await
            .map_err(|e| crate::utils::error::AppError::io_error(format!("遍历目录失败: {}", e)))? {
            
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                let json_content = tokio::fs::read_to_string(&path).await
                    .map_err(|e| crate::utils::error::AppError::io_error(format!("读取文件失败: {}", e)))?;

                match serde_json::from_str::<T>(&json_content) {
                    Ok(data) => results.push(data),
                    Err(e) => {
                        // 记录错误但继续处理其他文件
                        eprintln!("反序列化文件 {:?} 失败: {}", path, e);
                    }
                }
            }
        }

        Ok(results)
    }

    /// 删除文件
    async fn delete_file(&self, entity_type: &str, id: &str) -> AppResult<()> {
        let file_path = self.get_entity_file_path(entity_type, id);
        
        if !file_path.exists() {
            return Ok(());
        }

        tokio::fs::remove_file(&file_path).await
            .map_err(|e| crate::utils::error::AppError::io_error(format!("删除文件失败: {}", e)))?;

        // 从缓存中移除
        let cache_key = format!("{}:{}", entity_type, id);
        if let Ok(mut cache) = self.cache.write() {
            cache.remove(&cache_key);
        }

        Ok(())
    }

    /// 复制目录
    fn copy_directory(&self, source: &PathBuf, target: &PathBuf) -> AppResult<()> {
        use std::fs;
        
        if !source.exists() {
            return Ok(());
        }

        // 创建目标目录
        fs::create_dir_all(target)
            .map_err(|e| crate::utils::error::AppError::io_error(format!("创建目录失败: {}", e)))?;

        // 遍历源目录
        for entry in fs::read_dir(source)? {
            let entry = entry?;
            let source_path = entry.path();
            let file_name = entry.file_name();
            let target_path = target.join(&file_name);

            if source_path.is_file() {
                // 复制文件
                fs::copy(&source_path, &target_path)
                    .map_err(|e| crate::utils::error::AppError::io_error(format!("复制文件失败: {}", e)))?;
            } else if source_path.is_dir() {
                // 递归复制子目录
                self.copy_directory(&source_path, &target_path)?;
            }
        }

        Ok(())
    }

    /// 应用查询条件过滤数据
    fn apply_query_criteria<T>(&self, mut items: Vec<T>, criteria: &QueryCriteria) -> QueryResult<T>
    where
        T: HasTimestamps,
    {
        let total_count = items.len();

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

        // 排序
        if let Some(sort_by) = &criteria.sort_by {
            match sort_by.as_str() {
                "created_at" => {
                    if criteria.sort_desc {
                        items.sort_by(|a, b| b.created_at().cmp(&a.created_at()));
                    } else {
                        items.sort_by(|a, b| a.created_at().cmp(&b.created_at()));
                    }
                }
                "updated_at" => {
                    if criteria.sort_desc {
                        items.sort_by(|a, b| b.updated_at().cmp(&a.updated_at()));
                    } else {
                        items.sort_by(|a, b| a.updated_at().cmp(&b.updated_at()));
                    }
                }
                _ => {} // 不支持的排序字段
            }
        }

        // 分页
        let offset = criteria.offset.unwrap_or(0);
        let limit = criteria.limit.unwrap_or(usize::MAX);
        
        let total_filtered = items.len();
        let items: Vec<T> = items.into_iter()
            .skip(offset)
            .take(limit)
            .collect();

        let has_more = offset + items.len() < total_filtered;

        QueryResult {
            items,
            total_count,
            has_more,
        }
    }
}

#[async_trait]
impl BaseService for JsonPersistenceService {
    fn service_name(&self) -> &'static str {
        "JsonPersistenceService"
    }

    async fn initialize(&mut self) -> AppResult<()> {
        // 确保根目录存在
        self.ensure_directory_exists(&self.config.storage_root_dir).await?;
        Ok(())
    }

    async fn shutdown(&mut self) -> AppResult<()> {
        // 清理缓存
        if let Ok(mut cache) = self.cache.write() {
            cache.clear();
        }
        Ok(())
    }

    async fn health_check(&self) -> AppResult<()> {
        // 检查存储目录是否可访问
        if self.config.storage_root_dir.exists() && self.config.storage_root_dir.is_dir() {
            Ok(())
        } else {
            Err(crate::utils::error::AppError::generic("存储目录不可访问"))
        }
    }
}

#[async_trait]
impl PersistenceService for JsonPersistenceService {
    async fn save_channel_definition(&self, definition: &ChannelPointDefinition) -> AppResult<()> {
        self.save_to_file("channel_definition", &definition.id, definition).await
    }

    async fn load_channel_definition(&self, id: &str) -> AppResult<Option<ChannelPointDefinition>> {
        self.load_from_file("channel_definition", id).await
    }

    async fn load_all_channel_definitions(&self) -> AppResult<Vec<ChannelPointDefinition>> {
        self.load_all("channel_definition").await
    }

    async fn delete_channel_definition(&self, id: &str) -> AppResult<()> {
        self.delete_file("channel_definition", id).await
    }

    async fn save_test_instance(&self, instance: &ChannelTestInstance) -> AppResult<()> {
        self.save_to_file("test_instance", &instance.instance_id, instance).await
    }

    async fn load_test_instance(&self, instance_id: &str) -> AppResult<Option<ChannelTestInstance>> {
        self.load_from_file("test_instance", instance_id).await
    }

    async fn load_test_instances_by_batch(&self, batch_id: &str) -> AppResult<Vec<ChannelTestInstance>> {
        let all_instances: Vec<ChannelTestInstance> = self.load_all("test_instance").await?;
        let filtered_instances = all_instances.into_iter()
            .filter(|instance| instance.batch_id == batch_id)
            .collect();
        Ok(filtered_instances)
    }

    async fn delete_test_instance(&self, instance_id: &str) -> AppResult<()> {
        self.delete_file("test_instance", instance_id).await
    }

    async fn save_batch_info(&self, batch: &TestBatchInfo) -> AppResult<()> {
        self.save_to_file("test_batch", &batch.batch_id, batch).await
    }

    async fn load_batch_info(&self, batch_id: &str) -> AppResult<Option<TestBatchInfo>> {
        self.load_from_file("test_batch", batch_id).await
    }

    async fn load_all_batch_info(&self) -> AppResult<Vec<TestBatchInfo>> {
        self.load_all("test_batch").await
    }

    async fn delete_batch_info(&self, batch_id: &str) -> AppResult<()> {
        self.delete_file("test_batch", batch_id).await
    }

    async fn save_test_outcome(&self, outcome: &RawTestOutcome) -> AppResult<()> {
        // 使用channel_instance_id作为文件名
        self.save_to_file("test_outcome", &outcome.channel_instance_id, outcome).await
    }

    async fn load_test_outcomes_by_instance(&self, instance_id: &str) -> AppResult<Vec<RawTestOutcome>> {
        let all_outcomes: Vec<RawTestOutcome> = self.load_all("test_outcome").await?;
        let filtered_outcomes = all_outcomes.into_iter()
            .filter(|outcome| outcome.channel_instance_id == instance_id)
            .collect();
        Ok(filtered_outcomes)
    }

    async fn load_test_outcomes_by_batch(&self, batch_id: &str) -> AppResult<Vec<RawTestOutcome>> {
        // 通过测试实例查找测试结果
        let batch_instances = self.load_test_instances_by_batch(batch_id).await?;
        let instance_ids: Vec<String> = batch_instances.iter()
            .map(|instance| instance.instance_id.clone())
            .collect();
        
        let all_outcomes: Vec<RawTestOutcome> = self.load_all("test_outcome").await?;
        let filtered_outcomes = all_outcomes.into_iter()
            .filter(|outcome| instance_ids.contains(&outcome.channel_instance_id))
            .collect();
        Ok(filtered_outcomes)
    }
}

impl JsonPersistenceService {
    /// 验证单个实体类型的数据完整性
    async fn verify_entity_integrity(&self, entity_type: &str) -> AppResult<IntegrityCheckResult> {
        let entity_dir = self.get_entity_dir(entity_type);
        
        let mut affected_items = Vec::new();

        if !entity_dir.exists() {
            return Ok(IntegrityCheckResult {
                check_name: format!("{}_存在性检查", entity_type),
                status: IntegrityStatus::Warning,
                message: "实体目录不存在".to_string(),
                details: Some(format!("目录路径: {:?}", entity_dir)),
                affected_items,
            });
        }

        let mut entries = tokio::fs::read_dir(&entity_dir).await
            .map_err(|e| crate::utils::error::AppError::io_error(format!("读取目录失败: {}", e)))?;

        while let Some(entry) = entries.next_entry().await
            .map_err(|e| crate::utils::error::AppError::io_error(format!("遍历目录失败: {}", e)))? {
            
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                // 尝试读取和解析文件
                match tokio::fs::read_to_string(&path).await {
                    Ok(content) => {
                        if let Err(_) = serde_json::from_str::<serde_json::Value>(&content) {
                            if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                                affected_items.push(file_name.to_string());
                            }
                        }
                    }
                    Err(_) => {
                        if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                            affected_items.push(file_name.to_string());
                        }
                    }
                }
            }
        }

        let status = if affected_items.is_empty() {
            IntegrityStatus::Good
        } else {
            IntegrityStatus::Error
        };

        Ok(IntegrityCheckResult {
            check_name: format!("{}_格式检查", entity_type),
            status,
            message: if affected_items.is_empty() {
                "所有文件格式正确".to_string()
            } else {
                format!("发现 {} 个损坏的文件", affected_items.len())
            },
            details: None,
            affected_items,
        })
    }
}

/// 时间戳访问trait
trait HasTimestamps {
    fn created_at(&self) -> chrono::DateTime<chrono::Utc>;
    fn updated_at(&self) -> chrono::DateTime<chrono::Utc>;
}

impl HasTimestamps for ChannelPointDefinition {
    fn created_at(&self) -> chrono::DateTime<chrono::Utc> {
        self.creation_time
    }
    
    fn updated_at(&self) -> chrono::DateTime<chrono::Utc> {
        self.last_updated_time
    }
}

impl HasTimestamps for ChannelTestInstance {
    fn created_at(&self) -> chrono::DateTime<chrono::Utc> {
        self.creation_time
    }
    
    fn updated_at(&self) -> chrono::DateTime<chrono::Utc> {
        self.last_updated_time
    }
}

impl HasTimestamps for TestBatchInfo {
    fn created_at(&self) -> chrono::DateTime<chrono::Utc> {
        self.creation_time
    }
    
    fn updated_at(&self) -> chrono::DateTime<chrono::Utc> {
        self.last_updated_time
    }
}

impl HasTimestamps for RawTestOutcome {
    fn created_at(&self) -> chrono::DateTime<chrono::Utc> {
        self.start_time
    }
    
    fn updated_at(&self) -> chrono::DateTime<chrono::Utc> {
        self.end_time
    }
}

impl JsonPersistenceService {
    // 这里可以添加JsonPersistenceService特有的方法
    // 目前暂时为空
} 