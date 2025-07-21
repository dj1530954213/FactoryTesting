//! # 持久化服务接口 (Persistence Service Interface)
//!
//! ## 业务说明
//! 持久化服务是系统数据持久化的统一抽象接口，定义了所有数据存储和检索操作
//! 通过这个接口，上层业务逻辑与具体的存储实现（SQLite、MySQL等）解耦
//!
//! ## 核心功能
//! ### 1. 通道定义管理
//! - **保存操作**: 单个和批量保存通道点位定义
//! - **查询操作**: 按ID、条件查询通道定义
//! - **删除操作**: 安全删除通道定义数据
//!
//! ### 2. 测试实例管理
//! - **实例存储**: 保存测试实例和状态信息
//! - **状态更新**: 更新测试进度和结果
//! - **错误备注**: 管理测试过程中的错误记录
//!
//! ### 3. 批次数据管理
//! - **批次操作**: 创建、查询、删除测试批次
//! - **关联管理**: 维护批次与测试实例的关联关系
//! - **状态跟踪**: 跟踪批次执行状态
//!
//! ### 4. 配置数据管理
//! - **PLC配置**: 管理PLC连接和通道配置
//! - **映射配置**: 管理通道映射关系
//! - **系统配置**: 管理应用程序配置参数
//!
//! ## 设计特点
//! - **接口抽象**: 通过trait定义统一的数据访问接口
//! - **异步操作**: 所有数据库操作都是异步的，提高性能
//! - **事务支持**: 支持数据库事务，确保数据一致性
//! - **批量操作**: 支持批量数据操作，提高效率
//! - **类型安全**: 使用强类型确保数据操作的正确性
//!
//! ## 实现策略
//! - **SqliteOrmPersistenceService**: 基于SQLite的生产实现
//! - **MockPersistenceService**: 用于测试的Mock实现
//! - **扩展性**: 支持未来添加其他数据库实现
//!
//! ## Rust知识点
//! - **trait对象**: 使用dyn trait实现运行时多态
//! - **async trait**: 支持异步方法的trait定义
//! - **类型转换**: 通过as_any支持向下类型转换

use std::any::Any;
use super::*;
use crate::models::test_plc_config::{TestPlcChannelConfig, PlcConnectionConfig, ChannelMappingConfig};
use crate::utils::error::AppError;

/// 持久化服务接口
/// 
/// 业务说明：
/// 定义了系统所有数据持久化操作的统一接口
/// 负责数据的存储、检索、更新和删除，支持事务和批量操作
/// 
/// 设计原则：
/// - 接口隔离：不同类型的数据操作分组定义
/// - 异步优先：所有操作都是异步的
/// - 错误处理：统一的错误处理机制
/// 
/// Rust知识点：
/// - async_trait：允许trait中定义异步方法
/// - BaseService：继承基础服务接口
#[async_trait]


pub trait IPersistenceService: BaseService {
    /// 将 trait 对象转换为 Any，以支持向下转型
    fn as_any(&self) -> &dyn Any;
    /// 保存通道点位定义
    /// 
    /// # 参数
    /// * `definition` - 通道点位定义
    async fn save_channel_definition(&self, definition: &ChannelPointDefinition) -> AppResult<()>;
    
    /// 批量保存通道点位定义
    /// 
    /// # 参数
    /// * `definitions` - 通道点位定义列表
    async fn save_channel_definitions(&self, definitions: &[ChannelPointDefinition]) -> AppResult<()>;
    
    /// 加载通道点位定义
    /// 
    /// # 参数
    /// * `id` - 定义ID
    /// 
    /// # 返回
    /// * `Option<ChannelPointDefinition>` - 通道点位定义
    async fn load_channel_definition(&self, id: &str) -> AppResult<Option<ChannelPointDefinition>>;
    
    /// 加载所有通道点位定义
    /// 
    /// # 返回
    /// * `Vec<ChannelPointDefinition>` - 所有通道点位定义
    async fn load_all_channel_definitions(&self) -> AppResult<Vec<ChannelPointDefinition>>;
    
    /// 按条件查询通道点位定义
    /// 
    /// # 参数
    /// * `criteria` - 查询条件
    /// 
    /// # 返回
    /// * `Vec<ChannelPointDefinition>` - 匹配的通道点位定义
    async fn query_channel_definitions(&self, criteria: &QueryCriteria) -> AppResult<Vec<ChannelPointDefinition>>;
    
    /// 删除通道点位定义
    /// 
    /// # 参数
    /// * `id` - 定义ID
    async fn delete_channel_definition(&self, id: &str) -> AppResult<()>;
    
    /// 保存测试实例
    /// 
    /// # 参数
    /// * `instance` - 测试实例
    async fn save_test_instance(&self, instance: &ChannelTestInstance) -> AppResult<()>;
    
    /// 批量保存测试实例
    /// 
    /// # 参数
    /// * `instances` - 测试实例列表
    async fn save_test_instances(&self, instances: &[ChannelTestInstance]) -> AppResult<()>;
    
    /// 更新测试实例的错误备注
    /// 
    /// # 参数
    /// * `instance_id` - 实例ID
    /// * `integration_error_notes` - 集成错误备注
    /// * `plc_programming_error_notes` - PLC编程错误备注
    /// * `hmi_configuration_error_notes` - 上位机组态错误备注
    async fn update_instance_error_notes(
        &self,
        instance_id: &str,
        integration_error_notes: Option<&str>,
        plc_programming_error_notes: Option<&str>,
        hmi_configuration_error_notes: Option<&str>,
    ) -> AppResult<()>;
    
    /// 加载测试实例
    /// 
    /// # 参数
    /// * `instance_id` - 实例ID
    /// 
    /// # 返回
    /// * `Option<ChannelTestInstance>` - 测试实例
    async fn load_test_instance(&self, instance_id: &str) -> AppResult<Option<ChannelTestInstance>>;
    
    /// 加载所有测试实例
    ///
    /// # 返回
    /// * `Vec<ChannelTestInstance>` - 所有测试实例列表
    async fn load_all_test_instances(&self) -> AppResult<Vec<ChannelTestInstance>>;

    /// 加载批次的所有测试实例
    ///
    /// # 参数
    /// * `batch_id` - 批次ID
    ///
    /// # 返回
    /// * `Vec<ChannelTestInstance>` - 测试实例列表
    async fn load_test_instances_by_batch(&self, batch_id: &str) -> AppResult<Vec<ChannelTestInstance>>;
    
    /// 按条件查询测试实例
    /// 
    /// # 参数
    /// * `criteria` - 查询条件
    /// 
    /// # 返回
    /// * `Vec<ChannelTestInstance>` - 匹配的测试实例
    async fn query_test_instances(&self, criteria: &QueryCriteria) -> AppResult<Vec<ChannelTestInstance>>;

    // ================== Global Function Test Status ==================
    /// 删除 station_name 为空的全局功能测试状态脏数据
    async fn delete_blank_station_global_function_tests(&self) -> AppResult<()>;
    
    /// 删除测试实例
    /// 
    /// # 参数
    /// * `instance_id` - 实例ID
    async fn delete_test_instance(&self, instance_id: &str) -> AppResult<()>;
    
    /// 保存测试批次信息
    /// 
    /// # 参数
    /// * `batch` - 测试批次信息
    async fn save_batch_info(&self, batch: &TestBatchInfo) -> AppResult<()>;
    
    /// 加载测试批次信息
    /// 
    /// # 参数
    /// * `batch_id` - 批次ID
    /// 
    /// # 返回
    /// * `Option<TestBatchInfo>` - 测试批次信息
    async fn load_batch_info(&self, batch_id: &str) -> AppResult<Option<TestBatchInfo>>;
    
    /// 加载所有测试批次信息
    /// 
    /// # 返回
    /// * `Vec<TestBatchInfo>` - 所有测试批次信息
    async fn load_all_batch_info(&self) -> AppResult<Vec<TestBatchInfo>>;
    
    /// 按条件查询测试批次
    /// 
    /// # 参数
    /// * `criteria` - 查询条件
    /// 
    /// # 返回
    /// * `Vec<TestBatchInfo>` - 匹配的测试批次
    async fn query_batch_info(&self, criteria: &QueryCriteria) -> AppResult<Vec<TestBatchInfo>>;
    
    /// 删除测试批次信息
    /// 
    /// # 参数
    /// * `batch_id` - 批次ID
    async fn delete_batch_info(&self, batch_id: &str) -> AppResult<()>;
    
    /// 保存测试结果
    /// 
    /// # 参数
    /// * `outcome` - 测试结果
    async fn save_test_outcome(&self, outcome: &RawTestOutcome) -> AppResult<()>;
    
    /// 批量保存测试结果
    /// 
    /// # 参数
    /// * `outcomes` - 测试结果列表
    async fn save_test_outcomes(&self, outcomes: &[RawTestOutcome]) -> AppResult<()>;
    
    /// 按测试实例ID查询测试结果
    /// 
    /// # 参数
    /// * `instance_id` - 实例ID
    /// 
    /// # 返回
    /// * `Vec<RawTestOutcome>` - 测试结果列表
    async fn load_test_outcomes_by_instance(&self, instance_id: &str) -> AppResult<Vec<RawTestOutcome>>;
    
    /// 按批次ID查询测试结果
    /// 
    /// # 参数
    /// * `batch_id` - 批次ID
    /// 
    /// # 返回
    /// * `Vec<RawTestOutcome>` - 测试结果列表
    async fn load_test_outcomes_by_batch(&self, batch_id: &str) -> AppResult<Vec<RawTestOutcome>>;
    
    /// 执行事务
    /// 
    /// # 参数
    /// * `operations` - 事务操作列表
    /// 
    /// # 返回
    /// * `TransactionResult` - 事务执行结果
    async fn execute_transaction(&self, operations: Vec<TransactionOperation>) -> AppResult<TransactionResult>;
    
    /// 创建数据备份
    /// 
    /// # 参数
    /// * `backup_name` - 备份名称
    /// 
    /// # 返回
    /// * `BackupInfo` - 备份信息
    async fn create_backup(&self, backup_name: &str) -> AppResult<BackupInfo>;
    
    /// 恢复数据备份
    /// 
    /// # 参数
    /// * `backup_id` - 备份ID
    async fn restore_backup(&self, backup_id: &str) -> AppResult<()>;
    
    /// 获取存储统计信息
    /// 
    /// # 返回
    /// * `StorageStatistics` - 存储统计信息
    async fn get_storage_statistics(&self) -> AppResult<StorageStatistics>;
    
    /// 清理过期数据
    /// 
    /// # 参数
    /// * `retention_policy` - 保留策略
    /// 
    /// # 返回
    /// * `CleanupResult` - 清理结果
    async fn cleanup_expired_data(&self, retention_policy: &RetentionPolicy) -> AppResult<CleanupResult>;

    // ======== 全局功能测试项 ========
    async fn save_global_function_test_status(&self, _status: &crate::models::GlobalFunctionTestStatus) -> AppResult<()> {
        Err(AppError::not_implemented_error("save_global_function_test_status"))
    }
    async fn load_all_global_function_test_statuses(&self) -> AppResult<Vec<crate::models::GlobalFunctionTestStatus>> {
        Err(AppError::not_implemented_error("load_all_global_function_test_statuses"))
    }
    async fn reset_global_function_test_statuses(&self) -> AppResult<()> {
        Err(AppError::not_implemented_error("reset_global_function_test_statuses"))
    }

    /// 按站场加载全局功能测试状态
    async fn load_global_function_test_statuses_by_station(&self, _station_name: &str) -> AppResult<Vec<crate::models::GlobalFunctionTestStatus>> {
        Err(AppError::not_implemented_error("load_global_function_test_statuses_by_station"))
    }
    /// 按站场+导入时间加载全局功能测试状态
    async fn load_global_function_test_statuses_by_station_time(&self, _station_name: &str, _import_time: &str) -> AppResult<Vec<crate::models::GlobalFunctionTestStatus>> {
        Err(AppError::not_implemented_error("load_global_function_test_statuses_by_station_time"))
    }
    /// 确保指定站场+导入时间存在 5 条默认全局功能测试状态
    async fn ensure_global_function_tests(&self, _station_name: &str, _import_time: &str) -> AppResult<()> {
        Err(AppError::not_implemented_error("ensure_global_function_tests"))
    }

    /// 按站场重置全局功能测试状态
    async fn reset_global_function_test_statuses_by_station(&self, _station_name: &str) -> AppResult<()> {
        Err(AppError::not_implemented_error("reset_global_function_test_statuses_by_station"))
    }

    // ======== PLC 测试配置相关 ========
    /// 保存测试 PLC 通道配置
    async fn save_test_plc_channel(&self, _channel: &TestPlcChannelConfig) -> AppResult<()> {
        Err(AppError::not_implemented_error("save_test_plc_channel"))
    }

    /// 加载单个测试 PLC 通道配置
    async fn load_test_plc_channel(&self, _id: &str) -> AppResult<Option<TestPlcChannelConfig>> {
        Err(AppError::not_implemented_error("load_test_plc_channel"))
    }

    /// 加载所有测试 PLC 通道配置
    async fn load_all_test_plc_channels(&self) -> AppResult<Vec<TestPlcChannelConfig>> {
        Err(AppError::not_implemented_error("load_all_test_plc_channels"))
    }

    /// 删除测试 PLC 通道配置
    async fn delete_test_plc_channel(&self, _id: &str) -> AppResult<()> {
        Err(AppError::not_implemented_error("delete_test_plc_channel"))
    }

    // ======== PLC 连接配置相关 ========
    /// 保存 PLC 连接配置
    async fn save_plc_connection(&self, _connection: &PlcConnectionConfig) -> AppResult<()> {
        Err(AppError::not_implemented_error("save_plc_connection"))
    }

    /// 加载 PLC 连接配置
    async fn load_plc_connection(&self, _id: &str) -> AppResult<Option<PlcConnectionConfig>> {
        Err(AppError::not_implemented_error("load_plc_connection"))
    }

    /// 加载所有 PLC 连接配置
    async fn load_all_plc_connections(&self) -> AppResult<Vec<PlcConnectionConfig>> {
        Err(AppError::not_implemented_error("load_all_plc_connections"))
    }

    /// 删除 PLC 连接配置
    async fn delete_plc_connection(&self, _id: &str) -> AppResult<()> {
        Err(AppError::not_implemented_error("delete_plc_connection"))
    }

    // ======== 通道映射相关 ========
    /// 保存通道映射配置
    async fn save_channel_mapping(&self, _mapping: &ChannelMappingConfig) -> AppResult<()> {
        Err(AppError::not_implemented_error("save_channel_mapping"))
    }

    /// 加载通道映射配置
    async fn load_channel_mapping(&self, _id: &str) -> AppResult<Option<ChannelMappingConfig>> {
        Err(AppError::not_implemented_error("load_channel_mapping"))
    }

    /// 加载所有通道映射配置
    async fn load_all_channel_mappings(&self) -> AppResult<Vec<ChannelMappingConfig>> {
        Err(AppError::not_implemented_error("load_all_channel_mappings"))
    }

    /// 删除通道映射配置
    async fn delete_channel_mapping(&self, _id: &str) -> AppResult<()> {
        Err(AppError::not_implemented_error("delete_channel_mapping"))
    }
}

/// 查询条件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryCriteria {
    /// 过滤条件
    pub filters: Vec<FilterCondition>,
    
    /// 排序条件
    pub sort_by: Vec<SortCondition>,
    
    /// 分页信息
    pub pagination: Option<PaginationInfo>,
    
    /// 包含的字段
    pub include_fields: Option<Vec<String>>,
    
    /// 排除的字段
    pub exclude_fields: Option<Vec<String>>,
}

/// 过滤条件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterCondition {
    /// 字段名
    pub field: String,
    
    /// 操作符
    pub operator: FilterOperator,
    
    /// 值
    pub value: serde_json::Value,
}

/// 过滤操作符
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FilterOperator {
    Equal,
    NotEqual,
    GreaterThan,
    GreaterThanOrEqual,
    LessThan,
    LessThanOrEqual,
    Like,
    NotLike,
    In,
    NotIn,
    IsNull,
    IsNotNull,
    Between,
}

/// 排序条件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SortCondition {
    /// 字段名
    pub field: String,
    
    /// 排序方向
    pub direction: SortDirection,
}

/// 排序方向
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SortDirection {
    Ascending,
    Descending,
}

/// 分页信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginationInfo {
    /// 页码（从1开始）
    pub page: u32,
    
    /// 每页大小
    pub page_size: u32,
    
    /// 偏移量
    pub offset: Option<u32>,
}

/// 事务操作
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransactionOperation {
    SaveChannelDefinition(ChannelPointDefinition),
    SaveTestInstance(ChannelTestInstance),
    SaveBatchInfo(TestBatchInfo),
    SaveTestOutcome(RawTestOutcome),
    DeleteChannelDefinition(String),
    DeleteTestInstance(String),
    DeleteBatchInfo(String),
    Custom {
        operation_type: String,
        data: serde_json::Value,
    },
}

/// 事务结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionResult {
    /// 是否成功
    pub success: bool,
    
    /// 执行的操作数
    pub operations_executed: u32,
    
    /// 失败的操作数
    pub operations_failed: u32,
    
    /// 错误信息
    pub errors: Vec<String>,
    
    /// 执行时长（毫秒）
    pub duration_ms: u64,
    
    /// 事务ID
    pub transaction_id: String,
}

/// 备份信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupInfo {
    /// 备份ID
    pub backup_id: String,
    
    /// 备份名称
    pub backup_name: String,
    
    /// 备份文件路径
    pub file_path: String,
    
    /// 备份大小（字节）
    pub size_bytes: u64,
    
    /// 创建时间
    pub created_at: DateTime<Utc>,
    
    /// 备份类型
    pub backup_type: BackupType,
    
    /// 压缩比
    pub compression_ratio: Option<f32>,
}

/// 备份类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BackupType {
    Full,
    Incremental,
    Differential,
}

/// 存储统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageStatistics {
    /// 总记录数
    pub total_records: u64,
    
    /// 按表统计
    pub records_by_table: HashMap<String, u64>,
    
    /// 数据库大小（字节）
    pub database_size_bytes: u64,
    
    /// 索引大小（字节）
    pub index_size_bytes: u64,
    
    /// 可用空间（字节）
    pub available_space_bytes: u64,
    
    /// 最后更新时间
    pub last_updated: DateTime<Utc>,
    
    /// 数据增长率（记录/天）
    pub growth_rate_records_per_day: f64,
}

/// 保留策略
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetentionPolicy {
    /// 测试结果保留天数
    pub test_outcomes_retention_days: u32,
    
    /// 批次信息保留天数
    pub batch_info_retention_days: u32,
    
    /// 日志保留天数
    pub logs_retention_days: u32,
    
    /// 是否保留失败的测试结果
    pub keep_failed_tests: bool,
    
    /// 最大存储大小（字节）
    pub max_storage_size_bytes: Option<u64>,
}

/// 清理结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanupResult {
    /// 删除的记录数
    pub deleted_records: u64,
    
    /// 按表统计删除数
    pub deleted_by_table: HashMap<String, u64>,
    
    /// 释放的空间（字节）
    pub freed_space_bytes: u64,
    
    /// 清理时长（毫秒）
    pub duration_ms: u64,
    
    /// 清理时间
    pub cleanup_time: DateTime<Utc>,
}
