/// 持久化服务接口定义和相关数据结构

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use crate::utils::error::AppResult;
use crate::domain::services::{PersistenceService};
use crate::models::structs::*;

/// 持久化服务配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistenceConfig {
    /// 存储根目录
    pub storage_root_dir: PathBuf,
    /// 通道定义存储目录
    pub channel_definitions_dir: String,
    /// 测试实例存储目录  
    pub test_instances_dir: String,
    /// 测试批次存储目录
    pub test_batches_dir: String,
    /// 测试结果存储目录
    pub test_outcomes_dir: String,
    /// 是否启用自动备份
    pub enable_auto_backup: bool,
    /// 备份保留天数
    pub backup_retention_days: u32,
    /// 最大文件大小（MB）
    pub max_file_size_mb: u32,
    /// 是否启用压缩
    pub enable_compression: bool,
}

impl Default for PersistenceConfig {
    fn default() -> Self {
        Self {
            storage_root_dir: PathBuf::from("./data"),
            channel_definitions_dir: "channel_definitions".to_string(),
            test_instances_dir: "test_instances".to_string(),
            test_batches_dir: "test_batches".to_string(),
            test_outcomes_dir: "test_outcomes".to_string(),
            enable_auto_backup: true,
            backup_retention_days: 30,
            max_file_size_mb: 100,
            enable_compression: false,
        }
    }
}

/// 查询条件
#[derive(Debug, Clone, Default)]
pub struct QueryCriteria {
    /// 按创建时间范围过滤
    pub created_after: Option<chrono::DateTime<chrono::Utc>>,
    pub created_before: Option<chrono::DateTime<chrono::Utc>>,
    /// 按更新时间范围过滤
    pub updated_after: Option<chrono::DateTime<chrono::Utc>>,
    pub updated_before: Option<chrono::DateTime<chrono::Utc>>,
    /// 分页参数
    pub limit: Option<usize>,
    pub offset: Option<usize>,
    /// 排序字段和方向
    pub sort_by: Option<String>,
    pub sort_desc: bool,
}

/// 查询结果
#[derive(Debug, Clone)]
pub struct QueryResult<T> {
    /// 查询到的数据
    pub items: Vec<T>,
    /// 总记录数
    pub total_count: usize,
    /// 是否还有更多数据
    pub has_more: bool,
}

/// 持久化服务统计信息
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PersistenceStats {
    /// 通道定义数量
    pub channel_definitions_count: usize,
    /// 测试实例数量
    pub test_instances_count: usize,
    /// 测试批次数量
    pub test_batches_count: usize,
    /// 测试结果数量
    pub test_outcomes_count: usize,
    /// 存储总大小（字节）
    pub total_storage_size_bytes: u64,
    /// 最后备份时间
    pub last_backup_time: Option<chrono::DateTime<chrono::Utc>>,
    /// 数据完整性检查时间
    pub last_integrity_check_time: Option<chrono::DateTime<chrono::Utc>>,
}

/// 扩展的持久化服务接口
/// 在基础PersistenceService的基础上添加了高级功能
#[async_trait]
pub trait ExtendedPersistenceService: PersistenceService {
    // 高级查询功能
    
    /// 条件查询通道定义
    async fn query_channel_definitions(&self, criteria: &QueryCriteria) -> AppResult<QueryResult<ChannelPointDefinition>>;
    
    /// 条件查询测试实例
    async fn query_test_instances(&self, criteria: &QueryCriteria) -> AppResult<QueryResult<ChannelTestInstance>>;
    
    /// 条件查询测试批次
    async fn query_test_batches(&self, criteria: &QueryCriteria) -> AppResult<QueryResult<TestBatchInfo>>;
    
    /// 条件查询测试结果
    async fn query_test_outcomes(&self, criteria: &QueryCriteria) -> AppResult<QueryResult<RawTestOutcome>>;
    
    // 批量操作功能
    
    /// 批量保存通道定义
    async fn batch_save_channel_definitions(&self, definitions: &[ChannelPointDefinition]) -> AppResult<()>;
    
    /// 批量保存测试实例
    async fn batch_save_test_instances(&self, instances: &[ChannelTestInstance]) -> AppResult<()>;
    
    /// 批量保存测试结果
    async fn batch_save_test_outcomes(&self, outcomes: &[RawTestOutcome]) -> AppResult<()>;
    
    /// 批量删除（按ID列表）
    async fn batch_delete_by_ids(&self, entity_type: &str, ids: &[String]) -> AppResult<()>;
    
    // 备份和恢复功能
    
    /// 创建数据备份
    async fn create_backup(&self, backup_name: Option<String>) -> AppResult<PathBuf>;
    
    /// 从备份恢复数据
    async fn restore_from_backup(&self, backup_path: &PathBuf) -> AppResult<()>;
    
    /// 列出所有可用备份
    async fn list_backups(&self) -> AppResult<Vec<BackupInfo>>;
    
    /// 删除旧备份
    async fn cleanup_old_backups(&self) -> AppResult<u32>;
    
    // 数据完整性和统计
    
    /// 验证数据完整性
    async fn verify_data_integrity(&self) -> AppResult<IntegrityReport>;
    
    /// 获取统计信息
    async fn get_statistics(&self) -> AppResult<PersistenceStats>;
    
    /// 获取存储配置
    fn get_config(&self) -> &PersistenceConfig;
    
    /// 更新存储配置
    async fn update_config(&mut self, config: PersistenceConfig) -> AppResult<()>;
    
    // 数据清理和维护
    
    /// 清理过期数据
    async fn cleanup_expired_data(&self, retention_days: u32) -> AppResult<u32>;
    
    /// 压缩存储空间
    async fn compact_storage(&self) -> AppResult<u64>; // 返回释放的字节数
    
    /// 重建索引（如果适用）
    async fn rebuild_indexes(&self) -> AppResult<()>;
}

/// 备份信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupInfo {
    /// 备份名称
    pub name: String,
    /// 备份文件路径
    pub path: PathBuf,
    /// 创建时间
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// 备份大小（字节）
    pub size_bytes: u64,
    /// 备份描述
    pub description: Option<String>,
    /// 是否是自动备份
    pub is_auto_backup: bool,
}

/// 数据完整性报告
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrityReport {
    /// 检查时间
    pub checked_at: chrono::DateTime<chrono::Utc>,
    /// 总体状态
    pub overall_status: IntegrityStatus,
    /// 详细检查结果
    pub details: Vec<IntegrityCheckResult>,
    /// 发现的问题数量
    pub issues_count: u32,
    /// 修复建议
    pub repair_suggestions: Vec<String>,
}

/// 完整性状态
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum IntegrityStatus {
    /// 良好
    Good,
    /// 有警告
    Warning,
    /// 有错误
    Error,
    /// 严重损坏
    Critical,
}

/// 完整性检查结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrityCheckResult {
    /// 检查项名称
    pub check_name: String,
    /// 检查结果状态
    pub status: IntegrityStatus,
    /// 检查消息
    pub message: String,
    /// 检查详情
    pub details: Option<String>,
    /// 受影响的数据项
    pub affected_items: Vec<String>,
}

/// 持久化服务工厂
/// 用于创建不同类型的持久化服务实例
pub struct PersistenceServiceFactory;

impl PersistenceServiceFactory {
    /// 创建JSON文件持久化服务
    pub fn create_json_service(config: PersistenceConfig) -> crate::infrastructure::persistence::JsonPersistenceService {
        crate::infrastructure::persistence::JsonPersistenceService::new(config)
    }
    
    /// 创建默认的JSON文件持久化服务
    pub fn create_default_json_service() -> crate::infrastructure::persistence::JsonPersistenceService {
        Self::create_json_service(PersistenceConfig::default())
    }
}

/// 持久化服务特性助手
pub struct PersistenceServiceHelper;

impl PersistenceServiceHelper {
    /// 验证存储路径
    pub async fn validate_storage_path(path: &PathBuf) -> AppResult<()> {
        if !path.exists() {
            tokio::fs::create_dir_all(path).await
                .map_err(|e| crate::utils::error::AppError::io_error(format!("创建存储目录失败: {}", e)))?;
        }
        
        if !path.is_dir() {
            return Err(crate::utils::error::AppError::validation_error("存储路径必须是目录"));
        }
        
        // 测试写入权限
        let test_file = path.join(".write_test");
        tokio::fs::write(&test_file, "test").await
            .map_err(|e| crate::utils::error::AppError::io_error(format!("存储目录没有写入权限: {}", e)))?;
        tokio::fs::remove_file(&test_file).await.ok();
        
        Ok(())
    }
    
    /// 计算目录大小
    pub fn calculate_directory_size(path: &PathBuf) -> AppResult<u64> {
        use std::fs;
        
        let mut total_size = 0u64;
        
        if path.is_file() {
            return Ok(fs::metadata(path)?.len());
        }
        
        if path.is_dir() {
            for entry in fs::read_dir(path)? {
                let entry = entry?;
                let entry_path = entry.path();
                
                if entry_path.is_file() {
                    total_size += fs::metadata(&entry_path)?.len();
                } else if entry_path.is_dir() {
                    // 递归调用，但现在是同步函数
                    total_size += Self::calculate_directory_size(&entry_path)?;
                }
            }
        }
        
        Ok(total_size)
    }
    
    /// 清理过期文件
    pub async fn cleanup_expired_files(dir: &PathBuf, retention_days: u32) -> AppResult<u32> {
        if !dir.exists() {
            return Ok(0);
        }
        
        let cutoff_time = chrono::Utc::now() - chrono::Duration::days(retention_days as i64);
        let mut deleted_count = 0u32;
        
        let mut dir_entries = tokio::fs::read_dir(dir).await
            .map_err(|e| crate::utils::error::AppError::io_error(format!("读取目录失败: {}", e)))?;
        
        while let Some(entry) = dir_entries.next_entry().await
            .map_err(|e| crate::utils::error::AppError::io_error(format!("遍历目录失败: {}", e)))? {
            
            let metadata = entry.metadata().await
                .map_err(|e| crate::utils::error::AppError::io_error(format!("获取文件元数据失败: {}", e)))?;
            
            if metadata.is_file() {
                if let Ok(modified_time) = metadata.modified() {
                    let file_time: chrono::DateTime<chrono::Utc> = modified_time.into();
                    if file_time < cutoff_time {
                        if tokio::fs::remove_file(entry.path()).await.is_ok() {
                            deleted_count += 1;
                        }
                    }
                }
            }
        }
        
        Ok(deleted_count)
    }
    
    /// 生成备份文件名
    pub fn generate_backup_name(prefix: Option<&str>) -> String {
        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
        match prefix {
            Some(p) => format!("{}_{}", p, timestamp),
            None => format!("backup_{}", timestamp),
        }
    }
    
    /// 验证JSON文件格式
    pub async fn validate_json_file<T>(file_path: &PathBuf) -> AppResult<()> 
    where 
        T: serde::de::DeserializeOwned,
    {
        let content = tokio::fs::read_to_string(file_path).await
            .map_err(|e| crate::utils::error::AppError::io_error(format!("读取文件失败: {}", e)))?;
        
        serde_json::from_str::<T>(&content)
            .map_err(|e| crate::utils::error::AppError::validation_error(format!("JSON格式验证失败: {}", e)))?;
        
        Ok(())
    }
} 
