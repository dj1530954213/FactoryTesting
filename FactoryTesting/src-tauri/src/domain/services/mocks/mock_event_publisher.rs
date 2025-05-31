use super::*;

/// Mock事件发布服务
#[derive(Debug, Clone)]
pub struct MockEventPublisher {
    base: MockServiceBase,
}

impl MockEventPublisher {
    pub fn new(config: MockConfig) -> Self {
        Self {
            base: MockServiceBase::new(config),
        }
    }
}

impl MockService for MockEventPublisher {
    fn get_mock_config(&self) -> &MockConfig {
        self.base.get_mock_config()
    }
    
    fn set_mock_config(&mut self, config: MockConfig) {
        self.base.set_mock_config(config);
    }
    
    fn get_call_history(&self) -> Vec<CallRecord> {
        self.base.get_call_history()
    }
    
    fn clear_call_history(&mut self) {
        self.base.clear_call_history();
    }
    
    fn record_call(&mut self, method_name: &str, parameters: serde_json::Value, success: bool, duration_ms: u64, error_message: Option<String>) {
        self.base.record_call(method_name, parameters, success, duration_ms, error_message);
    }
}

#[async_trait]
impl BaseService for MockEventPublisher {
    fn service_name(&self) -> &'static str {
        "MockEventPublisher"
    }
    
    async fn initialize(&mut self) -> AppResult<()> {
        Ok(())
    }
    
    async fn shutdown(&mut self) -> AppResult<()> {
        Ok(())
    }
    
    async fn health_check(&self) -> AppResult<()> {
        Ok(())
    }
}

#[async_trait]
impl IEventPublisher for MockEventPublisher {
    async fn publish_test_status_changed(&self, _event: TestStatusChangedEvent) -> AppResult<()> {
        self.simulate_delay().await;
        Ok(())
    }
    
    async fn publish_test_completed(&self, _event: TestCompletedEvent) -> AppResult<()> {
        self.simulate_delay().await;
        Ok(())
    }
    
    async fn publish_batch_status_changed(&self, _event: BatchStatusChangedEvent) -> AppResult<()> {
        self.simulate_delay().await;
        Ok(())
    }
    
    async fn publish_plc_connection_changed(&self, _event: PlcConnectionChangedEvent) -> AppResult<()> {
        self.simulate_delay().await;
        Ok(())
    }
    
    async fn publish_progress_update(&self, _event: ProgressUpdateEvent) -> AppResult<()> {
        self.simulate_delay().await;
        Ok(())
    }
    
    async fn publish_error(&self, _event: ErrorEvent) -> AppResult<()> {
        self.simulate_delay().await;
        Ok(())
    }
    
    async fn publish_warning(&self, _event: WarningEvent) -> AppResult<()> {
        self.simulate_delay().await;
        Ok(())
    }
    
    async fn publish_info(&self, _event: InfoEvent) -> AppResult<()> {
        self.simulate_delay().await;
        Ok(())
    }
    
    async fn publish_custom(&self, _event_type: &str, _payload: serde_json::Value) -> AppResult<()> {
        self.simulate_delay().await;
        Ok(())
    }
    
    async fn subscribe(&self, _event_types: Vec<String>, _subscriber: Box<dyn EventSubscriber>) -> AppResult<SubscriptionHandle> {
        self.simulate_delay().await;
        
        Ok(SubscriptionHandle {
            subscription_id: uuid::Uuid::new_v4().to_string(),
            subscriber_id: "mock_subscriber".to_string(),
            event_types: vec!["test_status_changed".to_string()],
            created_at: Utc::now(),
        })
    }
    
    async fn unsubscribe(&self, _handle: SubscriptionHandle) -> AppResult<()> {
        self.simulate_delay().await;
        Ok(())
    }
    
    async fn get_event_statistics(&self) -> AppResult<EventStatistics> {
        self.simulate_delay().await;
        
        Ok(EventStatistics {
            total_events: 1000,
            events_by_type: HashMap::new(),
            errors_by_severity: HashMap::new(),
            active_subscribers: 5,
            last_event_time: Some(Utc::now()),
            event_rate: 10.5,
            time_range: TimeRange {
                start: Utc::now() - chrono::Duration::hours(1),
                end: Utc::now(),
            },
        })
    }
}
