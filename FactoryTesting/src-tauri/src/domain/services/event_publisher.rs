use super::*;

/// 事件发布服务接口
///
/// 负责发布系统事件到前端和其他订阅者
/// 符合 FAT-EVT-001 规则：事件发布规则
#[async_trait]
pub trait IEventPublisher: BaseService {
    /// 发布测试状态变化事件
    ///
    /// # 参数
    /// * `event` - 测试状态变化事件
    async fn publish_test_status_changed(&self, event: TestStatusChangedEvent) -> AppResult<()>;

    /// 发布测试完成事件
    ///
    /// # 参数
    /// * `event` - 测试完成事件
    async fn publish_test_completed(&self, event: TestCompletedEvent) -> AppResult<()>;

    /// 发布批次状态变化事件
    ///
    /// # 参数
    /// * `event` - 批次状态变化事件
    async fn publish_batch_status_changed(&self, event: BatchStatusChangedEvent) -> AppResult<()>;

    /// 发布PLC连接状态变化事件
    ///
    /// # 参数
    /// * `event` - PLC连接状态变化事件
    async fn publish_plc_connection_changed(&self, event: PlcConnectionChangedEvent) -> AppResult<()>;

    /// 发布进度更新事件
    ///
    /// # 参数
    /// * `event` - 进度更新事件
    async fn publish_progress_update(&self, event: ProgressUpdateEvent) -> AppResult<()>;

    /// 发布错误事件
    ///
    /// # 参数
    /// * `event` - 错误事件
    async fn publish_error(&self, event: ErrorEvent) -> AppResult<()>;

    /// 发布警告事件
    ///
    /// # 参数
    /// * `event` - 警告事件
    async fn publish_warning(&self, event: WarningEvent) -> AppResult<()>;

    /// 发布信息事件
    ///
    /// # 参数
    /// * `event` - 信息事件
    async fn publish_info(&self, event: InfoEvent) -> AppResult<()>;

    /// 发布自定义事件
    ///
    /// # 参数
    /// * `event_type` - 事件类型
    /// * `payload` - 事件载荷
    async fn publish_custom(&self, event_type: &str, payload: serde_json::Value) -> AppResult<()>;

    /// 订阅事件
    ///
    /// # 参数
    /// * `event_types` - 要订阅的事件类型列表
    /// * `subscriber` - 订阅者
    async fn subscribe(&self, event_types: Vec<String>, subscriber: Box<dyn EventSubscriber>) -> AppResult<SubscriptionHandle>;

    /// 取消订阅
    ///
    /// # 参数
    /// * `handle` - 订阅句柄
    async fn unsubscribe(&self, handle: SubscriptionHandle) -> AppResult<()>;

    /// 获取事件统计
    ///
    /// # 返回
    /// * `EventStatistics` - 事件统计信息
    async fn get_event_statistics(&self) -> AppResult<EventStatistics>;
}

/// 事件订阅者接口
#[async_trait]
pub trait EventSubscriber: Send + Sync {
    /// 处理事件
    ///
    /// # 参数
    /// * `event` - 事件
    async fn handle_event(&self, event: &SystemEvent) -> AppResult<()>;

    /// 获取订阅者ID
    fn subscriber_id(&self) -> &str;
}

/// 测试状态变化事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestStatusChangedEvent {
    /// 实例ID
    pub instance_id: String,

    /// 批次ID
    pub batch_id: String,

    /// 旧状态
    pub old_status: crate::models::enums::OverallTestStatus,

    /// 新状态
    pub new_status: crate::models::enums::OverallTestStatus,

    /// 变化原因
    pub reason: String,

    /// 时间戳
    pub timestamp: DateTime<Utc>,

    /// 额外数据
    pub metadata: HashMap<String, serde_json::Value>,
}

/// 测试完成事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestCompletedEvent {
    /// 实例ID
    pub instance_id: String,

    /// 批次ID
    pub batch_id: String,

    /// 测试结果
    pub outcome: RawTestOutcome,

    /// 执行时长（毫秒）
    pub duration_ms: u64,

    /// 时间戳
    pub timestamp: DateTime<Utc>,

    /// 额外数据
    pub metadata: HashMap<String, serde_json::Value>,
}

/// 批次状态变化事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchStatusChangedEvent {
    /// 批次ID
    pub batch_id: String,

    /// 批次统计
    pub statistics: BatchStatistics,

    /// 状态变化类型
    pub change_type: BatchChangeType,

    /// 时间戳
    pub timestamp: DateTime<Utc>,

    /// 额外数据
    pub metadata: HashMap<String, serde_json::Value>,
}

/// PLC连接状态变化事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlcConnectionChangedEvent {
    /// 连接ID
    pub connection_id: String,

    /// 连接名称
    pub connection_name: String,

    /// 是否已连接
    pub is_connected: bool,

    /// 错误信息（如果连接失败）
    pub error_message: Option<String>,

    /// 时间戳
    pub timestamp: DateTime<Utc>,

    /// 额外数据
    pub metadata: HashMap<String, serde_json::Value>,
}

/// 进度更新事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressUpdateEvent {
    /// 批次ID
    pub batch_id: String,

    /// 进度信息
    pub progress: TestProgressUpdate,

    /// 时间戳
    pub timestamp: DateTime<Utc>,

    /// 额外数据
    pub metadata: HashMap<String, serde_json::Value>,
}

/// 错误事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorEvent {
    /// 错误ID
    pub error_id: String,

    /// 错误类型
    pub error_type: String,

    /// 错误消息
    pub message: String,

    /// 错误详情
    pub details: Option<String>,

    /// 相关实例ID
    pub instance_id: Option<String>,

    /// 相关批次ID
    pub batch_id: Option<String>,

    /// 严重程度
    pub severity: ErrorSeverity,

    /// 时间戳
    pub timestamp: DateTime<Utc>,

    /// 额外数据
    pub metadata: HashMap<String, serde_json::Value>,
}

/// 警告事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WarningEvent {
    /// 警告ID
    pub warning_id: String,

    /// 警告类型
    pub warning_type: String,

    /// 警告消息
    pub message: String,

    /// 警告详情
    pub details: Option<String>,

    /// 相关实例ID
    pub instance_id: Option<String>,

    /// 相关批次ID
    pub batch_id: Option<String>,

    /// 时间戳
    pub timestamp: DateTime<Utc>,

    /// 额外数据
    pub metadata: HashMap<String, serde_json::Value>,
}

/// 信息事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InfoEvent {
    /// 信息ID
    pub info_id: String,

    /// 信息类型
    pub info_type: String,

    /// 信息消息
    pub message: String,

    /// 信息详情
    pub details: Option<String>,

    /// 相关实例ID
    pub instance_id: Option<String>,

    /// 相关批次ID
    pub batch_id: Option<String>,

    /// 时间戳
    pub timestamp: DateTime<Utc>,

    /// 额外数据
    pub metadata: HashMap<String, serde_json::Value>,
}

/// 系统事件（统一事件类型）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SystemEvent {
    TestStatusChanged(TestStatusChangedEvent),
    TestCompleted(TestCompletedEvent),
    BatchStatusChanged(BatchStatusChangedEvent),
    PlcConnectionChanged(PlcConnectionChangedEvent),
    ProgressUpdate(ProgressUpdateEvent),
    Error(ErrorEvent),
    Warning(WarningEvent),
    Info(InfoEvent),
    Custom {
        event_type: String,
        payload: serde_json::Value,
        timestamp: DateTime<Utc>,
    },
}

/// 批次变化类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BatchChangeType {
    Created,
    Started,
    Paused,
    Resumed,
    Completed,
    Cancelled,
    ProgressUpdate,
}

/// 错误严重程度
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum ErrorSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// 订阅句柄
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscriptionHandle {
    /// 订阅ID
    pub subscription_id: String,

    /// 订阅者ID
    pub subscriber_id: String,

    /// 订阅的事件类型
    pub event_types: Vec<String>,

    /// 创建时间
    pub created_at: DateTime<Utc>,
}

/// 事件统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventStatistics {
    /// 总事件数
    pub total_events: u64,

    /// 按类型统计
    pub events_by_type: HashMap<String, u64>,

    /// 按严重程度统计错误
    pub errors_by_severity: HashMap<ErrorSeverity, u64>,

    /// 活跃订阅者数
    pub active_subscribers: u32,

    /// 最后事件时间
    pub last_event_time: Option<DateTime<Utc>>,

    /// 事件发布速率（事件/秒）
    pub event_rate: f64,

    /// 统计时间范围
    pub time_range: TimeRange,
}
