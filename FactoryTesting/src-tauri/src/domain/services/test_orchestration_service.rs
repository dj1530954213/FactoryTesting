use super::*;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// 测试编排服务接口
/// 
/// 负责协调整个测试流程，包括批次创建、测试执行、进度监控等
#[async_trait]
pub trait ITestOrchestrationService: BaseService {
    /// 创建测试批次
    /// 
    /// # 参数
    /// * `request` - 测试执行请求
    /// 
    /// # 返回
    /// * `TestExecutionResponse` - 包含批次信息和分配结果
    async fn create_test_batch(&self, request: TestExecutionRequest) -> AppResult<TestExecutionResponse>;
    
    /// 开始批次测试
    /// 
    /// # 参数
    /// * `batch_id` - 批次ID
    /// 
    /// # 返回
    /// * `()` - 成功启动测试
    async fn start_batch_test(&self, batch_id: &str) -> AppResult<()>;
    
    /// 暂停批次测试
    /// 
    /// # 参数
    /// * `batch_id` - 批次ID
    async fn pause_batch_test(&self, batch_id: &str) -> AppResult<()>;
    
    /// 恢复批次测试
    /// 
    /// # 参数
    /// * `batch_id` - 批次ID
    async fn resume_batch_test(&self, batch_id: &str) -> AppResult<()>;
    
    /// 取消批次测试
    /// 
    /// # 参数
    /// * `batch_id` - 批次ID
    async fn cancel_batch_test(&self, batch_id: &str) -> AppResult<()>;
    
    /// 获取批次测试进度
    /// 
    /// # 参数
    /// * `batch_id` - 批次ID
    /// 
    /// # 返回
    /// * `TestProgressUpdate` - 当前进度信息
    async fn get_batch_progress(&self, batch_id: &str) -> AppResult<TestProgressUpdate>;
    
    /// 获取所有活跃批次
    /// 
    /// # 返回
    /// * `Vec<TestBatchInfo>` - 活跃批次列表
    async fn get_active_batches(&self) -> AppResult<Vec<TestBatchInfo>>;
    
    /// 获取批次详细信息
    /// 
    /// # 参数
    /// * `batch_id` - 批次ID
    /// 
    /// # 返回
    /// * `TestBatchInfo` - 批次详细信息
    async fn get_batch_details(&self, batch_id: &str) -> AppResult<TestBatchInfo>;
    
    /// 删除批次
    /// 
    /// # 参数
    /// * `batch_id` - 批次ID
    async fn delete_batch(&self, batch_id: &str) -> AppResult<()>;
}

/// 测试执行请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestExecutionRequest {
    /// 通道定义列表
    pub channel_definitions: Vec<ChannelPointDefinition>,
    
    /// 批次信息
    pub batch_info: TestBatchInfo,
    
    /// 测试配置
    pub test_config: TestExecutionConfig,
    
    /// 请求时间戳
    pub timestamp: DateTime<Utc>,
}

/// 测试执行响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestExecutionResponse {
    /// 批次信息
    pub batch_info: TestBatchInfo,
    
    /// 分配的测试实例
    pub test_instances: Vec<ChannelTestInstance>,
    
    /// 分配摘要
    pub allocation_summary: AllocationSummary,
    
    /// 响应时间戳
    pub timestamp: DateTime<Utc>,
}

/// 测试执行配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestExecutionConfig {
    /// 最大并发测试数
    pub max_concurrent_tests: usize,
    
    /// 测试超时时间（毫秒）
    pub test_timeout_ms: u64,
    
    /// 是否启用自动重试
    pub enable_auto_retry: bool,
    
    /// 最大重试次数
    pub max_retry_count: u32,
    
    /// 重试间隔（毫秒）
    pub retry_interval_ms: u64,
    
    /// 是否在错误时停止批次
    pub stop_on_error: bool,
    
    /// 测试优先级
    pub priority: TestPriority,
}

/// 测试优先级
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TestPriority {
    Low,
    Normal,
    High,
    Critical,
}

/// 分配摘要
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllocationSummary {
    /// 总通道数
    pub total_channels: u32,
    
    /// 成功分配的通道数
    pub allocated_channels: u32,
    
    /// 跳过的通道数
    pub skipped_channels: u32,
    
    /// 错误的通道数
    pub error_channels: u32,
    
    /// 按模块类型统计
    pub module_type_stats: HashMap<String, ModuleTypeStats>,
    
    /// 分配时间
    pub allocation_time: DateTime<Utc>,
    
    /// 分配耗时（毫秒）
    pub allocation_duration_ms: u64,
}

/// 模块类型统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleTypeStats {
    /// 模块类型
    pub module_type: String,
    
    /// 总数
    pub total_count: u32,
    
    /// 分配数
    pub allocated_count: u32,
    
    /// 跳过数
    pub skipped_count: u32,
    
    /// 错误数
    pub error_count: u32,
}
