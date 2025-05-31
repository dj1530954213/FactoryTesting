use super::*;

/// Mock测试编排服务
#[derive(Debug, Clone)]
pub struct MockTestOrchestrationService {
    base: MockServiceBase,
}

impl MockTestOrchestrationService {
    pub fn new(config: MockConfig) -> Self {
        Self {
            base: MockServiceBase::new(config),
        }
    }
}

impl MockService for MockTestOrchestrationService {
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
impl BaseService for MockTestOrchestrationService {
    fn service_name(&self) -> &'static str {
        "MockTestOrchestrationService"
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
impl ITestOrchestrationService for MockTestOrchestrationService {
    async fn create_test_batch(&self, _request: TestExecutionRequest) -> AppResult<TestExecutionResponse> {
        self.simulate_delay().await;
        
        if self.should_inject_error() {
            return Err(crate::utils::error::AppError::MockError("Injected error".to_string()));
        }
        
        // 创建Mock响应
        let batch_info = TestBatchInfo::new(
            Some("MockProduct".to_string()),
            Some("MockSerial".to_string()),
        );
        
        Ok(TestExecutionResponse {
            batch_info,
            test_instances: vec![],
            allocation_summary: AllocationSummary {
                total_channels: 0,
                allocated_channels: 0,
                skipped_channels: 0,
                error_channels: 0,
                module_type_stats: HashMap::new(),
                allocation_time: Utc::now(),
                allocation_duration_ms: 100,
            },
            timestamp: Utc::now(),
        })
    }
    
    async fn start_batch_test(&self, _batch_id: &str) -> AppResult<()> {
        self.simulate_delay().await;
        Ok(())
    }
    
    async fn pause_batch_test(&self, _batch_id: &str) -> AppResult<()> {
        self.simulate_delay().await;
        Ok(())
    }
    
    async fn resume_batch_test(&self, _batch_id: &str) -> AppResult<()> {
        self.simulate_delay().await;
        Ok(())
    }
    
    async fn cancel_batch_test(&self, _batch_id: &str) -> AppResult<()> {
        self.simulate_delay().await;
        Ok(())
    }
    
    async fn get_batch_progress(&self, _batch_id: &str) -> AppResult<TestProgressUpdate> {
        self.simulate_delay().await;
        
        Ok(TestProgressUpdate {
            batch_id: "mock_batch".to_string(),
            instance_id: "mock_instance".to_string(),
            progress_percentage: 50.0,
            current_step: "Mock testing".to_string(),
            estimated_remaining_time_ms: Some(30000),
            statistics: BatchStatistics {
                total_channels: 10,
                tested_channels: 5,
                passed_channels: 4,
                failed_channels: 1,
                skipped_channels: 0,
                in_progress_channels: 5,
                start_time: Some(Utc::now()),
                end_time: None,
                estimated_completion_time: Some(Utc::now() + chrono::Duration::minutes(1)),
            },
            timestamp: Utc::now(),
        })
    }
    
    async fn get_active_batches(&self) -> AppResult<Vec<TestBatchInfo>> {
        self.simulate_delay().await;
        Ok(vec![])
    }
    
    async fn get_batch_details(&self, _batch_id: &str) -> AppResult<TestBatchInfo> {
        self.simulate_delay().await;
        Ok(TestBatchInfo::new(
            Some("MockProduct".to_string()),
            Some("MockSerial".to_string()),
        ))
    }
    
    async fn delete_batch(&self, _batch_id: &str) -> AppResult<()> {
        self.simulate_delay().await;
        Ok(())
    }
}
