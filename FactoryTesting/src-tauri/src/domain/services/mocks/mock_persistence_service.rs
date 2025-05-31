use super::*;

/// Mock持久化服务
#[derive(Debug, Clone)]
pub struct MockPersistenceService {
    base: MockServiceBase,
}

impl MockPersistenceService {
    pub fn new(config: MockConfig) -> Self {
        Self {
            base: MockServiceBase::new(config),
        }
    }
}

impl MockService for MockPersistenceService {
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
impl BaseService for MockPersistenceService {
    fn service_name(&self) -> &'static str {
        "MockPersistenceService"
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
impl IPersistenceService for MockPersistenceService {
    async fn save_channel_definition(&self, _definition: &ChannelPointDefinition) -> AppResult<()> {
        self.simulate_delay().await;
        Ok(())
    }
    
    async fn save_channel_definitions(&self, _definitions: &[ChannelPointDefinition]) -> AppResult<()> {
        self.simulate_delay().await;
        Ok(())
    }
    
    async fn load_channel_definition(&self, _id: &str) -> AppResult<Option<ChannelPointDefinition>> {
        self.simulate_delay().await;
        Ok(None)
    }
    
    async fn load_all_channel_definitions(&self) -> AppResult<Vec<ChannelPointDefinition>> {
        self.simulate_delay().await;
        Ok(vec![])
    }
    
    async fn query_channel_definitions(&self, _criteria: &QueryCriteria) -> AppResult<Vec<ChannelPointDefinition>> {
        self.simulate_delay().await;
        Ok(vec![])
    }
    
    async fn delete_channel_definition(&self, _id: &str) -> AppResult<()> {
        self.simulate_delay().await;
        Ok(())
    }
    
    async fn save_test_instance(&self, _instance: &ChannelTestInstance) -> AppResult<()> {
        self.simulate_delay().await;
        Ok(())
    }
    
    async fn save_test_instances(&self, _instances: &[ChannelTestInstance]) -> AppResult<()> {
        self.simulate_delay().await;
        Ok(())
    }
    
    async fn load_test_instance(&self, _instance_id: &str) -> AppResult<Option<ChannelTestInstance>> {
        self.simulate_delay().await;
        Ok(None)
    }
    
    async fn load_test_instances_by_batch(&self, _batch_id: &str) -> AppResult<Vec<ChannelTestInstance>> {
        self.simulate_delay().await;
        Ok(vec![])
    }
    
    async fn query_test_instances(&self, _criteria: &QueryCriteria) -> AppResult<Vec<ChannelTestInstance>> {
        self.simulate_delay().await;
        Ok(vec![])
    }
    
    async fn delete_test_instance(&self, _instance_id: &str) -> AppResult<()> {
        self.simulate_delay().await;
        Ok(())
    }
    
    async fn save_batch_info(&self, _batch: &TestBatchInfo) -> AppResult<()> {
        self.simulate_delay().await;
        Ok(())
    }
    
    async fn load_batch_info(&self, _batch_id: &str) -> AppResult<Option<TestBatchInfo>> {
        self.simulate_delay().await;
        Ok(None)
    }
    
    async fn load_all_batch_info(&self) -> AppResult<Vec<TestBatchInfo>> {
        self.simulate_delay().await;
        Ok(vec![])
    }
    
    async fn query_batch_info(&self, _criteria: &QueryCriteria) -> AppResult<Vec<TestBatchInfo>> {
        self.simulate_delay().await;
        Ok(vec![])
    }
    
    async fn delete_batch_info(&self, _batch_id: &str) -> AppResult<()> {
        self.simulate_delay().await;
        Ok(())
    }
    
    async fn save_test_outcome(&self, _outcome: &RawTestOutcome) -> AppResult<()> {
        self.simulate_delay().await;
        Ok(())
    }
    
    async fn save_test_outcomes(&self, _outcomes: &[RawTestOutcome]) -> AppResult<()> {
        self.simulate_delay().await;
        Ok(())
    }
    
    async fn load_test_outcomes_by_instance(&self, _instance_id: &str) -> AppResult<Vec<RawTestOutcome>> {
        self.simulate_delay().await;
        Ok(vec![])
    }
    
    async fn load_test_outcomes_by_batch(&self, _batch_id: &str) -> AppResult<Vec<RawTestOutcome>> {
        self.simulate_delay().await;
        Ok(vec![])
    }
    
    async fn execute_transaction(&self, _operations: Vec<TransactionOperation>) -> AppResult<TransactionResult> {
        self.simulate_delay().await;
        
        Ok(TransactionResult {
            success: true,
            operations_executed: 1,
            operations_failed: 0,
            errors: vec![],
            duration_ms: 50,
            transaction_id: uuid::Uuid::new_v4().to_string(),
        })
    }
    
    async fn create_backup(&self, backup_name: &str) -> AppResult<BackupInfo> {
        self.simulate_delay().await;
        
        Ok(BackupInfo {
            backup_id: uuid::Uuid::new_v4().to_string(),
            backup_name: backup_name.to_string(),
            file_path: format!("./backups/{}.db", backup_name),
            size_bytes: 1024 * 1024,
            created_at: Utc::now(),
            backup_type: BackupType::Full,
            compression_ratio: Some(0.7),
        })
    }
    
    async fn restore_backup(&self, _backup_id: &str) -> AppResult<()> {
        self.simulate_delay().await;
        Ok(())
    }
    
    async fn get_storage_statistics(&self) -> AppResult<StorageStatistics> {
        self.simulate_delay().await;
        
        Ok(StorageStatistics {
            total_records: 10000,
            records_by_table: HashMap::new(),
            database_size_bytes: 10 * 1024 * 1024,
            index_size_bytes: 1024 * 1024,
            available_space_bytes: 100 * 1024 * 1024,
            last_updated: Utc::now(),
            growth_rate_records_per_day: 100.0,
        })
    }
    
    async fn cleanup_expired_data(&self, _retention_policy: &RetentionPolicy) -> AppResult<CleanupResult> {
        self.simulate_delay().await;
        
        Ok(CleanupResult {
            deleted_records: 100,
            deleted_by_table: HashMap::new(),
            freed_space_bytes: 1024 * 1024,
            duration_ms: 1000,
            cleanup_time: Utc::now(),
        })
    }
}
