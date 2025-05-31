use super::*;

/// Mock批次分配服务
#[derive(Debug, Clone)]
pub struct MockBatchAllocationService {
    base: MockServiceBase,
}

impl MockBatchAllocationService {
    pub fn new(config: MockConfig) -> Self {
        Self {
            base: MockServiceBase::new(config),
        }
    }
}

impl MockService for MockBatchAllocationService {
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
impl BaseService for MockBatchAllocationService {
    fn service_name(&self) -> &'static str {
        "MockBatchAllocationService"
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
impl IBatchAllocationService for MockBatchAllocationService {
    async fn allocate_channels(
        &self,
        definitions: Vec<ChannelPointDefinition>,
        batch_info: TestBatchInfo,
        _strategy: AllocationStrategy,
    ) -> AppResult<BatchAllocationResult> {
        self.simulate_delay().await;
        
        let test_instances: Vec<ChannelTestInstance> = definitions
            .iter()
            .map(|def| ChannelTestInstance::new(def.id.clone(), batch_info.batch_id.clone()))
            .collect();
        
        let allocation_summary = AllocationSummary {
            total_channels: definitions.len() as u32,
            allocated_channels: definitions.len() as u32,
            skipped_channels: 0,
            error_channels: 0,
            module_type_stats: HashMap::new(),
            allocation_time: Utc::now(),
            allocation_duration_ms: 100,
        };
        
        Ok(BatchAllocationResult {
            batch_info,
            test_instances,
            allocation_summary,
            allocation_time: Utc::now(),
            allocation_duration_ms: 100,
            warnings: vec![],
            skipped_definitions: vec![],
        })
    }
    
    async fn validate_allocation(
        &self,
        _definitions: &[ChannelPointDefinition],
        _strategy: &AllocationStrategy,
    ) -> AppResult<ValidationResult> {
        self.simulate_delay().await;
        
        Ok(ValidationResult {
            is_valid: true,
            errors: vec![],
            warnings: vec![],
            suggestions: vec![],
        })
    }
    
    async fn preview_allocation(
        &self,
        definitions: &[ChannelPointDefinition],
        _strategy: &AllocationStrategy,
    ) -> AppResult<AllocationPreview> {
        self.simulate_delay().await;
        
        Ok(AllocationPreview {
            estimated_allocations: definitions.len() as u32,
            estimated_skips: 0,
            module_type_breakdown: HashMap::new(),
            estimated_duration_ms: 100,
            resource_usage: ResourceUsageEstimate {
                estimated_memory_bytes: 1024 * 1024,
                estimated_cpu_usage: 10.0,
                estimated_network_bandwidth: 1024,
                estimated_storage_bytes: 512 * 1024,
            },
        })
    }
    
    async fn reallocate_batch(
        &self,
        batch_id: &str,
        _strategy: AllocationStrategy,
    ) -> AppResult<BatchAllocationResult> {
        self.simulate_delay().await;
        
        let batch_info = TestBatchInfo::new(
            Some("MockProduct".to_string()),
            Some("MockSerial".to_string()),
        );
        
        Ok(BatchAllocationResult {
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
            allocation_time: Utc::now(),
            allocation_duration_ms: 100,
            warnings: vec![],
            skipped_definitions: vec![],
        })
    }
    
    async fn get_allocation_history(&self, _batch_id: &str) -> AppResult<Vec<AllocationRecord>> {
        self.simulate_delay().await;
        Ok(vec![])
    }
    
    async fn get_allocation_statistics(&self, _time_range: Option<TimeRange>) -> AppResult<AllocationStatistics> {
        self.simulate_delay().await;
        
        Ok(AllocationStatistics {
            total_allocations: 100,
            successful_allocations: 95,
            average_allocation_time_ms: 150.0,
            average_batch_size: 50.0,
            most_used_strategy: Some("Sequential".to_string()),
            module_type_stats: HashMap::new(),
            time_range: TimeRange {
                start: Utc::now() - chrono::Duration::days(30),
                end: Utc::now(),
            },
        })
    }
}
