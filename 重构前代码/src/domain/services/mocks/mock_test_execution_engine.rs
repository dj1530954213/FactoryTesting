use super::*;

/// Mock测试执行引擎
#[derive(Debug, Clone)]
pub struct MockTestExecutionEngine {
    base: MockServiceBase,
}

impl MockTestExecutionEngine {
    pub fn new(config: MockConfig) -> Self {
        Self {
            base: MockServiceBase::new(config),
        }
    }
}

impl MockService for MockTestExecutionEngine {
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
impl BaseService for MockTestExecutionEngine {
    fn service_name(&self) -> &'static str {
        "MockTestExecutionEngine"
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
impl ITestExecutionEngine for MockTestExecutionEngine {
    async fn submit_batch_test(
        &self,
        batch_id: &str,
        _test_instances: Vec<ChannelTestInstance>,
        _config: TestExecutionConfig,
    ) -> AppResult<BatchExecutionHandle> {
        self.simulate_delay().await;

        Ok(BatchExecutionHandle {
            batch_id: batch_id.to_string(),
            execution_id: uuid::Uuid::new_v4().to_string(),
            task_ids: vec![],
            created_at: Utc::now(),
            cancellation_token: None,
        })
    }

    async fn submit_test_task(
        &self,
        instance: ChannelTestInstance,
        priority: TaskPriority,
    ) -> AppResult<TaskExecutionHandle> {
        self.simulate_delay().await;

        Ok(TaskExecutionHandle {
            task_id: uuid::Uuid::new_v4().to_string(),
            instance_id: instance.instance_id,
            batch_id: Some(instance.test_batch_id),
            priority,
            created_at: Utc::now(),
            cancellation_token: None,
        })
    }

    async fn pause_batch(&self, _batch_id: &str) -> AppResult<()> {
        self.simulate_delay().await;
        Ok(())
    }

    async fn resume_batch(&self, _batch_id: &str) -> AppResult<()> {
        self.simulate_delay().await;
        Ok(())
    }

    async fn cancel_batch(&self, _batch_id: &str) -> AppResult<()> {
        self.simulate_delay().await;
        Ok(())
    }

    async fn cancel_task(&self, _task_id: &str) -> AppResult<()> {
        self.simulate_delay().await;
        Ok(())
    }

    async fn get_batch_status(&self, batch_id: &str) -> AppResult<BatchExecutionStatus> {
        self.simulate_delay().await;

        Ok(BatchExecutionStatus {
            batch_id: batch_id.to_string(),
            status: ExecutionStatus::Running,
            total_tasks: 10,
            completed_tasks: 5,
            failed_tasks: 1,
            running_tasks: 2,
            pending_tasks: 2,
            start_time: Some(Utc::now()),
            end_time: None,
            estimated_completion: Some(Utc::now() + chrono::Duration::minutes(5)),
            progress_percentage: 50.0,
        })
    }

    async fn get_task_status(&self, task_id: &str) -> AppResult<TaskExecutionStatus> {
        self.simulate_delay().await;

        Ok(TaskExecutionStatus {
            task_id: task_id.to_string(),
            instance_id: "mock_instance".to_string(),
            status: ExecutionStatus::Running,
            current_step: Some("Mock testing".to_string()),
            start_time: Some(Utc::now()),
            end_time: None,
            duration_ms: None,
            error_message: None,
            retry_count: 0,
        })
    }

    async fn get_engine_stats(&self) -> AppResult<ExecutionEngineStats> {
        self.simulate_delay().await;

        Ok(ExecutionEngineStats {
            max_concurrent: 88,
            current_concurrent: 5,
            total_tasks: 100,
            completed_tasks: 80,
            failed_tasks: 5,
            average_execution_time_ms: 2500.0,
            engine_start_time: Utc::now(),
            last_activity: Utc::now(),
            queued_tasks: 10,
        })
    }

    async fn set_max_concurrent(&self, _max_concurrent: usize) -> AppResult<()> {
        self.simulate_delay().await;
        Ok(())
    }

    async fn get_current_concurrent(&self) -> AppResult<usize> {
        self.simulate_delay().await;
        Ok(5)
    }
}
