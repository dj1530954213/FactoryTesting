// Thin wrapper: re-export canonical trait
pub use crate::domain::impls::test_execution_engine::ITestExecutionEngine;

#[cfg(any())]
mod legacy {


#[cfg(any())]
/// 测试执行引擎接口
/// 
/// 负责管理和并发执行测试任务，支持88个通道的并发测试
/// 符合 FAT-TTM-001 规则：测试任务管理
#[async_trait]
#[cfg(any())]
pub trait ITestExecutionEngine: BaseService {
    /// 提交批次测试任务
    /// 
    /// # 参数
    /// * `batch_id` - 批次ID
    /// * `test_instances` - 测试实例列表
    /// * `config` - 执行配置
    /// 
    /// # 返回
    /// * `BatchExecutionHandle` - 批次执行句柄
    async fn submit_batch_test(
        &self,
        batch_id: &str,
        test_instances: Vec<ChannelTestInstance>,
        config: TestExecutionConfig,
    ) -> AppResult<BatchExecutionHandle>;
    
    /// 提交单个测试任务
    /// 
    /// # 参数
    /// * `instance` - 测试实例
    /// * `priority` - 任务优先级
    /// 
    /// # 返回
    /// * `TaskExecutionHandle` - 任务执行句柄
    async fn submit_test_task(
        &self,
        instance: ChannelTestInstance,
        priority: TaskPriority,
    ) -> AppResult<TaskExecutionHandle>;
    
    /// 暂停批次执行
    /// 
    /// # 参数
    /// * `batch_id` - 批次ID
    async fn pause_batch(&self, batch_id: &str) -> AppResult<()>;
    
    /// 恢复批次执行
    /// 
    /// # 参数
    /// * `batch_id` - 批次ID
    async fn resume_batch(&self, batch_id: &str) -> AppResult<()>;
    
    /// 取消批次执行
    /// 
    /// # 参数
    /// * `batch_id` - 批次ID
    async fn cancel_batch(&self, batch_id: &str) -> AppResult<()>;
    
    /// 取消单个任务
    /// 
    /// # 参数
    /// * `task_id` - 任务ID
    async fn cancel_task(&self, task_id: &str) -> AppResult<()>;
    
    /// 获取批次执行状态
    /// 
    /// # 参数
    /// * `batch_id` - 批次ID
    /// 
    /// # 返回
    /// * `BatchExecutionStatus` - 批次执行状态
    async fn get_batch_status(&self, batch_id: &str) -> AppResult<BatchExecutionStatus>;
    
    /// 获取任务执行状态
    /// 
    /// # 参数
    /// * `task_id` - 任务ID
    /// 
    /// # 返回
    /// * `TaskExecutionStatus` - 任务执行状态
    async fn get_task_status(&self, task_id: &str) -> AppResult<TaskExecutionStatus>;
    
    /// 获取引擎统计信息
    /// 
    /// # 返回
    /// * `ExecutionEngineStats` - 引擎统计信息
    async fn get_engine_stats(&self) -> AppResult<ExecutionEngineStats>;
    
    /// 设置最大并发数
    /// 
    /// # 参数
    /// * `max_concurrent` - 最大并发数
    async fn set_max_concurrent(&self, max_concurrent: usize) -> AppResult<()>;
    
    /// 获取当前并发数
    /// 
    /// # 返回
    /// * `usize` - 当前并发数
    async fn get_current_concurrent(&self) -> AppResult<usize>;
}

/// 批次执行句柄
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchExecutionHandle {
    /// 批次ID
    pub batch_id: String,
    
    /// 执行ID
    pub execution_id: String,
    
    /// 任务ID列表
    pub task_ids: Vec<String>,
    
    /// 创建时间
    pub created_at: DateTime<Utc>,
    
    /// 取消令牌
    #[serde(skip)]
    pub cancellation_token: Option<CancellationToken>,
}

/// 任务执行句柄
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskExecutionHandle {
    /// 任务ID
    pub task_id: String,
    
    /// 实例ID
    pub instance_id: String,
    
    /// 批次ID
    pub batch_id: Option<String>,
    
    /// 优先级
    pub priority: TaskPriority,
    
    /// 创建时间
    pub created_at: DateTime<Utc>,
    
    /// 取消令牌
    #[serde(skip)]
    pub cancellation_token: Option<CancellationToken>,
}

/// 任务优先级
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum TaskPriority {
    Low = 1,
    Normal = 2,
    High = 3,
    Critical = 4,
}

/// 批次执行状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchExecutionStatus {
    /// 批次ID
    pub batch_id: String,
    
    /// 执行状态
    pub status: ExecutionStatus,
    
    /// 总任务数
    pub total_tasks: u32,
    
    /// 已完成任务数
    pub completed_tasks: u32,
    
    /// 失败任务数
    pub failed_tasks: u32,
    
    /// 正在执行任务数
    pub running_tasks: u32,
    
    /// 等待执行任务数
    pub pending_tasks: u32,
    
    /// 开始时间
    pub start_time: Option<DateTime<Utc>>,
    
    /// 结束时间
    pub end_time: Option<DateTime<Utc>>,
    
    /// 预计完成时间
    pub estimated_completion: Option<DateTime<Utc>>,
    
    /// 进度百分比
    pub progress_percentage: f32,
}

/// 任务执行状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskExecutionStatus {
    /// 任务ID
    pub task_id: String,
    
    /// 实例ID
    pub instance_id: String,
    
    /// 执行状态
    pub status: ExecutionStatus,
    
    /// 当前步骤
    pub current_step: Option<String>,
    
    /// 开始时间
    pub start_time: Option<DateTime<Utc>>,
    
    /// 结束时间
    pub end_time: Option<DateTime<Utc>>,
    
    /// 执行时长（毫秒）
    pub duration_ms: Option<u64>,
    
    /// 错误信息
    pub error_message: Option<String>,
    
    /// 重试次数
    pub retry_count: u32,
}

/// 执行状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExecutionStatus {
    Pending,
    Running,
    Paused,
    Completed,
    Failed,
    Cancelled,
}

/// 执行引擎统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionEngineStats {
    /// 最大并发数
    pub max_concurrent: usize,
    
    /// 当前并发数
    pub current_concurrent: usize,
    
    /// 总任务数
    pub total_tasks: u64,
    
    /// 已完成任务数
    pub completed_tasks: u64,
    
    /// 失败任务数
    pub failed_tasks: u64,
    
    /// 平均执行时间（毫秒）
    pub average_execution_time_ms: f64,
    
    /// 引擎启动时间
    pub engine_start_time: DateTime<Utc>,
    
    /// 最后活动时间
    pub last_activity: DateTime<Utc>,
    
    /// 队列中等待的任务数
    pub queued_tasks: u32,
}
}
