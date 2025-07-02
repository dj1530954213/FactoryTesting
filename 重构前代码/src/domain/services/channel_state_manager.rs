use super::*;
use std::collections::HashMap;

/// 通道状态管理器接口
/// 
/// 这是系统中唯一允许修改 ChannelTestInstance 状态的组件
/// 符合 FAT-CSM-001 规则：状态管理唯一入口
#[async_trait]
pub trait IChannelStateManager: BaseService {
    /// 应用原始测试结果到实例状态
    /// 
    /// 这是唯一允许修改通道测试实例状态的方法
    /// 
    /// # 参数
    /// * `instance` - 要更新的测试实例
    /// * `outcome` - 原始测试结果
    /// 
    /// # 返回
    /// * `StateChangeResult` - 状态变更结果
    async fn apply_raw_outcome(
        &self,
        instance: &mut ChannelTestInstance,
        outcome: &RawTestOutcome,
    ) -> AppResult<StateChangeResult>;
    
    /// 批量应用测试结果
    /// 
    /// # 参数
    /// * `updates` - 实例ID到测试结果的映射
    /// 
    /// # 返回
    /// * `Vec<StateChangeResult>` - 所有状态变更结果
    async fn apply_batch_outcomes(
        &self,
        updates: HashMap<String, RawTestOutcome>,
    ) -> AppResult<Vec<StateChangeResult>>;
    
    /// 获取通道当前状态
    /// 
    /// # 参数
    /// * `instance_id` - 实例ID
    /// 
    /// # 返回
    /// * `ChannelState` - 当前状态信息
    async fn get_channel_state(&self, instance_id: &str) -> AppResult<ChannelState>;
    
    /// 获取批次状态摘要
    /// 
    /// # 参数
    /// * `batch_id` - 批次ID
    /// 
    /// # 返回
    /// * `BatchStatistics` - 批次统计信息
    async fn get_batch_statistics(&self, batch_id: &str) -> AppResult<BatchStatistics>;
    
    /// 验证状态转换是否有效
    /// 
    /// # 参数
    /// * `from` - 源状态
    /// * `to` - 目标状态
    /// * `context` - 转换上下文
    /// 
    /// # 返回
    /// * `bool` - 是否为有效转换
    fn is_valid_state_transition(
        &self,
        from: &crate::models::enums::OverallTestStatus,
        to: &crate::models::enums::OverallTestStatus,
        context: &StateTransitionContext,
    ) -> bool;
    
    /// 强制设置通道状态（仅用于管理操作）
    /// 
    /// # 参数
    /// * `instance_id` - 实例ID
    /// * `new_status` - 新状态
    /// * `reason` - 强制设置的原因
    /// 
    /// # 返回
    /// * `StateChangeResult` - 状态变更结果
    async fn force_set_channel_state(
        &self,
        instance_id: &str,
        new_status: crate::models::enums::OverallTestStatus,
        reason: &str,
    ) -> AppResult<StateChangeResult>;
    
    /// 重置通道状态
    /// 
    /// # 参数
    /// * `instance_id` - 实例ID
    /// 
    /// # 返回
    /// * `StateChangeResult` - 状态变更结果
    async fn reset_channel_state(&self, instance_id: &str) -> AppResult<StateChangeResult>;
    
    /// 获取状态变更历史
    /// 
    /// # 参数
    /// * `instance_id` - 实例ID
    /// * `limit` - 返回记录数限制
    /// 
    /// # 返回
    /// * `Vec<StateChangeRecord>` - 状态变更历史
    async fn get_state_change_history(
        &self,
        instance_id: &str,
        limit: Option<usize>,
    ) -> AppResult<Vec<StateChangeRecord>>;

    /// 获取所有缓存的测试实例（如果有缓存实现）
    async fn get_all_cached_test_instances(&self) -> Vec<ChannelTestInstance>;

    /// 清空内部缓存（通道定义 + 测试实例）
    async fn clear_caches(&self);

    /// 从数据库恢复所有批次/实例/定义到内存缓存
    async fn restore_all_batches(&self) -> AppResult<Vec<TestBatchInfo>>;
}

/// 通道状态信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelState {
    /// 实例ID
    pub instance_id: String,
    
    /// 当前状态
    pub current_status: crate::models::enums::OverallTestStatus,
    
    /// 子测试结果
    pub sub_test_results: HashMap<crate::models::enums::SubTestItem, crate::models::structs::SubTestExecutionResult>,
    
    /// 最后更新时间
    pub last_updated: DateTime<Utc>,
    
    /// 状态持续时间（毫秒）
    pub status_duration_ms: u64,
    
    /// 是否可以进行测试
    pub can_test: bool,
    
    /// 错误信息
    pub error_message: Option<String>,
}

/// 状态变更结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateChangeResult {
    /// 实例ID
    pub instance_id: String,
    
    /// 变更前状态
    pub old_status: crate::models::enums::OverallTestStatus,
    
    /// 变更后状态
    pub new_status: crate::models::enums::OverallTestStatus,
    
    /// 是否成功变更
    pub success: bool,
    
    /// 变更时间
    pub timestamp: DateTime<Utc>,
    
    /// 变更原因
    pub reason: String,
    
    /// 错误信息（如果失败）
    pub error_message: Option<String>,
    
    /// 触发的事件
    pub triggered_events: Vec<String>,
}

/// 状态转换上下文
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateTransitionContext {
    /// 触发转换的测试项
    pub test_item: Option<crate::models::enums::SubTestItem>,
    
    /// 测试结果
    pub test_result: Option<TestResult>,
    
    /// 是否为自动转换
    pub is_automatic: bool,
    
    /// 操作用户（如果是手动操作）
    pub operator: Option<String>,
    
    /// 额外上下文信息
    pub context_data: HashMap<String, serde_json::Value>,
}

/// 状态变更记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateChangeRecord {
    /// 记录ID
    pub id: String,
    
    /// 实例ID
    pub instance_id: String,
    
    /// 变更前状态
    pub old_status: crate::models::enums::OverallTestStatus,
    
    /// 变更后状态
    pub new_status: crate::models::enums::OverallTestStatus,
    
    /// 变更时间
    pub timestamp: DateTime<Utc>,
    
    /// 变更原因
    pub reason: String,
    
    /// 操作用户
    pub operator: Option<String>,
    
    /// 转换上下文
    pub context: StateTransitionContext,
}
