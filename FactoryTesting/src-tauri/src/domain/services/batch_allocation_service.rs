use super::*;

/// 批次分配服务接口
/// 
/// 负责将通道定义分配到测试实例，管理测试批次的创建和分配逻辑
#[async_trait]
pub trait IBatchAllocationService: BaseService {
    /// 分配通道到测试批次
    /// 
    /// # 参数
    /// * `definitions` - 通道定义列表
    /// * `batch_info` - 批次信息
    /// * `strategy` - 分配策略
    /// 
    /// # 返回
    /// * `BatchAllocationResult` - 分配结果
    async fn allocate_channels(
        &self,
        definitions: Vec<ChannelPointDefinition>,
        batch_info: TestBatchInfo,
        strategy: AllocationStrategy,
    ) -> AppResult<BatchAllocationResult>;
    
    /// 验证分配配置
    /// 
    /// # 参数
    /// * `definitions` - 通道定义列表
    /// * `strategy` - 分配策略
    /// 
    /// # 返回
    /// * `ValidationResult` - 验证结果
    async fn validate_allocation(
        &self,
        definitions: &[ChannelPointDefinition],
        strategy: &AllocationStrategy,
    ) -> AppResult<ValidationResult>;
    
    /// 预览分配结果
    /// 
    /// # 参数
    /// * `definitions` - 通道定义列表
    /// * `strategy` - 分配策略
    /// 
    /// # 返回
    /// * `AllocationPreview` - 分配预览
    async fn preview_allocation(
        &self,
        definitions: &[ChannelPointDefinition],
        strategy: &AllocationStrategy,
    ) -> AppResult<AllocationPreview>;
    
    /// 重新分配批次
    /// 
    /// # 参数
    /// * `batch_id` - 批次ID
    /// * `strategy` - 新的分配策略
    /// 
    /// # 返回
    /// * `BatchAllocationResult` - 重新分配结果
    async fn reallocate_batch(
        &self,
        batch_id: &str,
        strategy: AllocationStrategy,
    ) -> AppResult<BatchAllocationResult>;
    
    /// 获取分配历史
    /// 
    /// # 参数
    /// * `batch_id` - 批次ID
    /// 
    /// # 返回
    /// * `Vec<AllocationRecord>` - 分配历史记录
    async fn get_allocation_history(&self, batch_id: &str) -> AppResult<Vec<AllocationRecord>>;
    
    /// 获取分配统计
    /// 
    /// # 参数
    /// * `time_range` - 时间范围
    /// 
    /// # 返回
    /// * `AllocationStatistics` - 分配统计信息
    async fn get_allocation_statistics(&self, time_range: Option<TimeRange>) -> AppResult<AllocationStatistics>;
}

/// 分配策略
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllocationStrategy {
    /// 策略名称
    pub name: String,
    
    /// 分配模式
    pub mode: AllocationMode,
    
    /// 优先级规则
    pub priority_rules: Vec<PriorityRule>,
    
    /// 分组规则
    pub grouping_rules: Vec<GroupingRule>,
    
    /// 过滤规则
    pub filter_rules: Vec<FilterRule>,
    
    /// 最大批次大小
    pub max_batch_size: Option<u32>,
    
    /// 是否允许部分分配
    pub allow_partial_allocation: bool,
    
    /// 分配超时（毫秒）
    pub allocation_timeout_ms: u64,
}

/// 分配模式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AllocationMode {
    /// 顺序分配
    Sequential,
    /// 并行分配
    Parallel,
    /// 优先级分配
    Priority,
    /// 负载均衡分配
    LoadBalanced,
    /// 自定义分配
    Custom,
}

/// 优先级规则
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriorityRule {
    /// 规则名称
    pub name: String,
    
    /// 匹配条件
    pub condition: MatchCondition,
    
    /// 优先级权重
    pub weight: i32,
    
    /// 是否启用
    pub enabled: bool,
}

/// 分组规则
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupingRule {
    /// 规则名称
    pub name: String,
    
    /// 分组字段
    pub group_by: GroupingField,
    
    /// 最大组大小
    pub max_group_size: Option<u32>,
    
    /// 是否启用
    pub enabled: bool,
}

/// 过滤规则
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterRule {
    /// 规则名称
    pub name: String,
    
    /// 过滤条件
    pub condition: MatchCondition,
    
    /// 过滤动作
    pub action: FilterAction,
    
    /// 是否启用
    pub enabled: bool,
}

/// 匹配条件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MatchCondition {
    /// 模块类型匹配
    ModuleType(crate::models::enums::ModuleType),
    /// 标签匹配
    TagPattern(String),
    /// 地址范围匹配
    AddressRange { start: String, end: String },
    /// 自定义条件
    Custom(String),
    /// 复合条件
    Composite {
        operator: LogicalOperator,
        conditions: Vec<MatchCondition>,
    },
}

/// 分组字段
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GroupingField {
    ModuleType,
    PowerSupplyType,
    WireSystem,
    PlcAddress,
    Custom(u32),
}

/// 过滤动作
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FilterAction {
    Include,
    Exclude,
    Skip,
    Priority(i32),
}

/// 逻辑操作符
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LogicalOperator {
    And,
    Or,
    Not,
}

/// 批次分配结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchAllocationResult {
    /// 批次信息
    pub batch_info: TestBatchInfo,
    
    /// 分配的测试实例
    pub test_instances: Vec<ChannelTestInstance>,
    
    /// 分配摘要
    pub allocation_summary: AllocationSummary,
    
    /// 分配时间
    pub allocation_time: DateTime<Utc>,
    
    /// 分配耗时（毫秒）
    pub allocation_duration_ms: u64,
    
    /// 警告信息
    pub warnings: Vec<String>,
    
    /// 跳过的定义
    pub skipped_definitions: Vec<SkippedDefinition>,
}

/// 验证结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    /// 是否有效
    pub is_valid: bool,
    
    /// 错误信息
    pub errors: Vec<ValidationError>,
    
    /// 警告信息
    pub warnings: Vec<ValidationWarning>,
    
    /// 建议信息
    pub suggestions: Vec<ValidationSuggestion>,
}

/// 分配预览
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllocationPreview {
    /// 预计分配数量
    pub estimated_allocations: u32,
    
    /// 预计跳过数量
    pub estimated_skips: u32,
    
    /// 按模块类型统计
    pub module_type_breakdown: HashMap<String, u32>,
    
    /// 预计分配时间（毫秒）
    pub estimated_duration_ms: u64,
    
    /// 资源使用预估
    pub resource_usage: ResourceUsageEstimate,
}

/// 分配记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllocationRecord {
    /// 记录ID
    pub id: String,
    
    /// 批次ID
    pub batch_id: String,
    
    /// 分配策略
    pub strategy: AllocationStrategy,
    
    /// 分配结果
    pub result: BatchAllocationResult,
    
    /// 操作用户
    pub operator: Option<String>,
    
    /// 创建时间
    pub created_at: DateTime<Utc>,
}

/// 分配统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllocationStatistics {
    /// 总分配次数
    pub total_allocations: u64,
    
    /// 成功分配次数
    pub successful_allocations: u64,
    
    /// 平均分配时间（毫秒）
    pub average_allocation_time_ms: f64,
    
    /// 平均批次大小
    pub average_batch_size: f64,
    
    /// 最常用策略
    pub most_used_strategy: Option<String>,
    
    /// 按模块类型统计
    pub module_type_stats: HashMap<String, ModuleTypeAllocationStats>,
    
    /// 时间范围
    pub time_range: TimeRange,
}

/// 跳过的定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkippedDefinition {
    /// 定义ID
    pub definition_id: String,
    
    /// 跳过原因
    pub reason: String,
    
    /// 详细信息
    pub details: Option<String>,
}

/// 验证错误
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationError {
    /// 错误代码
    pub code: String,
    
    /// 错误消息
    pub message: String,
    
    /// 相关定义ID
    pub definition_id: Option<String>,
}

/// 验证警告
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationWarning {
    /// 警告代码
    pub code: String,
    
    /// 警告消息
    pub message: String,
    
    /// 相关定义ID
    pub definition_id: Option<String>,
}

/// 验证建议
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationSuggestion {
    /// 建议类型
    pub suggestion_type: String,
    
    /// 建议消息
    pub message: String,
    
    /// 建议的改进措施
    pub improvement: Option<String>,
}

/// 资源使用预估
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUsageEstimate {
    /// 预计内存使用（字节）
    pub estimated_memory_bytes: u64,
    
    /// 预计CPU使用率
    pub estimated_cpu_usage: f32,
    
    /// 预计网络带宽（字节/秒）
    pub estimated_network_bandwidth: u64,
    
    /// 预计存储空间（字节）
    pub estimated_storage_bytes: u64,
}

/// 模块类型分配统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleTypeAllocationStats {
    /// 模块类型
    pub module_type: String,
    
    /// 总分配数
    pub total_allocated: u64,
    
    /// 成功分配数
    pub successful_allocated: u64,
    
    /// 平均分配时间（毫秒）
    pub average_time_ms: f64,
}

/// 时间范围
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeRange {
    /// 开始时间
    pub start: DateTime<Utc>,
    
    /// 结束时间
    pub end: DateTime<Utc>,
}
