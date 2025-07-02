use super::*;

/// 事件发布服务接口
///
/// 负责发布系统事件到前端和其他订阅者
/// 符合 FAT-EVT-001 规则：事件发布规则
#[async_trait]
pub trait IEventPublisher: BaseService {
    // === 兼容旧调用方式 ===
    async fn publish_test_status_changed(&self, instance_id: &str, old_status: OverallTestStatus, new_status: OverallTestStatus) -> AppResult<()>;
    async fn publish_test_completed(&self, outcome: &RawTestOutcome) -> AppResult<()>;
    async fn publish_batch_status_changed(&self, batch_id: &str, statistics: &BatchStatistics) -> AppResult<()>;
    async fn publish_plc_connection_changed(&self, connected: bool) -> AppResult<()>;

    // === 扩展接口（可选实现，提供默认空实现） ===
    async fn publish_progress_update(&self, _event: ProgressUpdateEvent) -> AppResult<()> { Ok(()) }
    async fn publish_error(&self, _event: ErrorEvent) -> AppResult<()> { Ok(()) }
    async fn publish_warning(&self, _event: WarningEvent) -> AppResult<()> { Ok(()) }
    async fn publish_info(&self, _event: InfoEvent) -> AppResult<()> { Ok(()) }
    async fn publish_custom(&self, _event_type: &str, _payload: serde_json::Value) -> AppResult<()> { Ok(()) }
    async fn subscribe(&self, _event_types: Vec<String>, _subscriber: Box<dyn EventSubscriber>) -> AppResult<SubscriptionHandle> {
        Ok(SubscriptionHandle {
            subscription_id: "compat".to_string(),
            subscriber_id: "compat".to_string(),
            event_types: vec![],
            created_at: chrono::Utc::now(),
        })
    }
    async fn unsubscribe(&self, _handle: SubscriptionHandle) -> AppResult<()> { Ok(()) }
    async fn get_event_statistics(&self) -> AppResult<EventStatistics> {
        Ok(EventStatistics::default())
    }
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

impl Default for TimeRange {
    fn default() -> Self {
        let now = chrono::Utc::now();
        Self { start: now, end: now }
    }
}

impl Default for EventStatistics {
    fn default() -> Self {
        Self {
            total_events: 0,
            events_by_type: std::collections::HashMap::new(),
            errors_by_severity: std::collections::HashMap::new(),
            active_subscribers: 0,
            last_event_time: None,
            event_rate: 0.0,
            time_range: TimeRange::default(),
        }
    }
}
