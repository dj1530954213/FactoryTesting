use super::*;

/// Mock通道状态管理器
#[derive(Debug, Clone)]
pub struct MockChannelStateManager {
    base: MockServiceBase,
}

impl MockChannelStateManager {
    pub fn new(config: MockConfig) -> Self {
        Self {
            base: MockServiceBase::new(config),
        }
    }
}

impl MockService for MockChannelStateManager {
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
impl BaseService for MockChannelStateManager {
    fn service_name(&self) -> &'static str {
        "MockChannelStateManager"
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
impl IChannelStateManager for MockChannelStateManager {
    async fn apply_raw_outcome(
        &self,
        _instance: &mut ChannelTestInstance,
        _outcome: &RawTestOutcome,
    ) -> AppResult<StateChangeResult> {
        self.simulate_delay().await;
        
        Ok(StateChangeResult {
            instance_id: "mock_instance".to_string(),
            old_status: crate::models::enums::OverallTestStatus::NotTested,
            new_status: crate::models::enums::OverallTestStatus::WiringConfirmed,
            success: true,
            timestamp: Utc::now(),
            reason: "Mock state change".to_string(),
            error_message: None,
            triggered_events: vec!["test_status_changed".to_string()],
        })
    }
    
    async fn apply_batch_outcomes(
        &self,
        _updates: HashMap<String, RawTestOutcome>,
    ) -> AppResult<Vec<StateChangeResult>> {
        self.simulate_delay().await;
        Ok(vec![])
    }
    
    async fn get_channel_state(&self, _instance_id: &str) -> AppResult<ChannelState> {
        self.simulate_delay().await;
        
        Ok(ChannelState {
            instance_id: "mock_instance".to_string(),
            current_status: crate::models::enums::OverallTestStatus::NotTested,
            sub_test_results: HashMap::new(),
            last_updated: Utc::now(),
            status_duration_ms: 1000,
            can_test: true,
            error_message: None,
        })
    }
    
    async fn get_batch_statistics(&self, _batch_id: &str) -> AppResult<BatchStatistics> {
        self.simulate_delay().await;
        
        Ok(BatchStatistics {
            total_channels: 10,
            tested_channels: 5,
            passed_channels: 4,
            failed_channels: 1,
            skipped_channels: 0,
            in_progress_channels: 5,
            start_time: Some(Utc::now()),
            end_time: None,
            estimated_completion_time: Some(Utc::now() + chrono::Duration::minutes(1)),
        })
    }
    
    fn is_valid_state_transition(
        &self,
        _from: &crate::models::enums::OverallTestStatus,
        _to: &crate::models::enums::OverallTestStatus,
        _context: &StateTransitionContext,
    ) -> bool {
        true
    }
    
    async fn force_set_channel_state(
        &self,
        _instance_id: &str,
        _new_status: crate::models::enums::OverallTestStatus,
        _reason: &str,
    ) -> AppResult<StateChangeResult> {
        self.simulate_delay().await;
        
        Ok(StateChangeResult {
            instance_id: "mock_instance".to_string(),
            old_status: crate::models::enums::OverallTestStatus::NotTested,
            new_status: crate::models::enums::OverallTestStatus::WiringConfirmed,
            success: true,
            timestamp: Utc::now(),
            reason: "Force set".to_string(),
            error_message: None,
            triggered_events: vec!["test_status_changed".to_string()],
        })
    }
    
    async fn reset_channel_state(&self, _instance_id: &str) -> AppResult<StateChangeResult> {
        self.simulate_delay().await;
        
        Ok(StateChangeResult {
            instance_id: "mock_instance".to_string(),
            old_status: crate::models::enums::OverallTestStatus::WiringConfirmed,
            new_status: crate::models::enums::OverallTestStatus::NotTested,
            success: true,
            timestamp: Utc::now(),
            reason: "Reset".to_string(),
            error_message: None,
            triggered_events: vec!["test_status_changed".to_string()],
        })
    }
    
    async fn get_state_change_history(
        &self,
        _instance_id: &str,
        _limit: Option<usize>,
    ) -> AppResult<Vec<StateChangeRecord>> {
        self.simulate_delay().await;
        Ok(vec![])
    }

    async fn get_all_cached_test_instances(&self) -> Vec<ChannelTestInstance> {
        vec![]
    }

    async fn clear_caches(&self) {
        // mock: do nothing
    }

    async fn restore_all_batches(&self) -> AppResult<Vec<crate::models::TestBatchInfo>> {
        Ok(vec![])
    }
}
