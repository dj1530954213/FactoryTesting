use super::*;

/// Mock PLC通信服务
#[derive(Debug, Clone)]
pub struct MockPlcCommunicationService {
    base: MockServiceBase,
}

impl MockPlcCommunicationService {
    pub fn new(config: MockConfig) -> Self {
        Self {
            base: MockServiceBase::new(config),
        }
    }
}

impl MockService for MockPlcCommunicationService {
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
impl BaseService for MockPlcCommunicationService {
    fn service_name(&self) -> &'static str {
        "MockPlcCommunicationService"
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
impl IPlcCommunicationService for MockPlcCommunicationService {
    async fn connect(&self, config: &PlcConnectionConfig) -> AppResult<ConnectionHandle> {
        self.simulate_delay().await;
        
        Ok(ConnectionHandle {
            connection_id: config.id.clone(),
            handle_id: uuid::Uuid::new_v4().to_string(),
            protocol: config.protocol,
            created_at: Utc::now(),
            last_activity: Utc::now(),
        })
    }
    
    async fn disconnect(&self, _handle: &ConnectionHandle) -> AppResult<()> {
        self.simulate_delay().await;
        Ok(())
    }
    
    async fn is_connected(&self, _handle: &ConnectionHandle) -> AppResult<bool> {
        self.simulate_delay().await;
        Ok(true)
    }
    
    async fn read_bool(&self, _handle: &ConnectionHandle, _address: &str) -> AppResult<bool> {
        self.simulate_delay().await;
        Ok(rand::random())
    }
    
    async fn write_bool(&self, _handle: &ConnectionHandle, _address: &str, _value: bool) -> AppResult<()> {
        self.simulate_delay().await;
        Ok(())
    }
    
    async fn read_f32(&self, _handle: &ConnectionHandle, _address: &str) -> AppResult<f32> {
        self.simulate_delay().await;
        Ok(rand::random::<f32>() * 100.0)
    }
    
    async fn write_f32(&self, _handle: &ConnectionHandle, _address: &str, _value: f32) -> AppResult<()> {
        self.simulate_delay().await;
        Ok(())
    }
    
    async fn read_i32(&self, _handle: &ConnectionHandle, _address: &str) -> AppResult<i32> {
        self.simulate_delay().await;
        Ok(rand::random::<i32>() % 1000)
    }
    
    async fn write_i32(&self, _handle: &ConnectionHandle, _address: &str, _value: i32) -> AppResult<()> {
        self.simulate_delay().await;
        Ok(())
    }
    
    async fn batch_read(&self, _handle: &ConnectionHandle, requests: &[ReadRequest]) -> AppResult<Vec<ReadResult>> {
        self.simulate_delay().await;
        
        let results = requests.iter().map(|req| ReadResult {
            request_id: req.id.clone(),
            success: true,
            value: Some(PlcValue::Float32(rand::random::<f32>() * 100.0)),
            error_message: None,
            execution_time_ms: 10,
        }).collect();
        
        Ok(results)
    }
    
    async fn batch_write(&self, _handle: &ConnectionHandle, requests: &[WriteRequest]) -> AppResult<Vec<WriteResult>> {
        self.simulate_delay().await;
        
        let results = requests.iter().map(|req| WriteResult {
            request_id: req.id.clone(),
            success: true,
            error_message: None,
            execution_time_ms: 10,
        }).collect();
        
        Ok(results)
    }
    
    async fn get_connection_stats(&self, handle: &ConnectionHandle) -> AppResult<ConnectionStats> {
        self.simulate_delay().await;
        
        Ok(ConnectionStats {
            connection_id: handle.connection_id.clone(),
            total_reads: 100,
            total_writes: 50,
            successful_reads: 98,
            successful_writes: 49,
            average_read_time_ms: 15.5,
            average_write_time_ms: 12.3,
            connection_established_at: Utc::now(),
            last_communication: Utc::now(),
            connection_errors: 2,
        })
    }
    
    async fn test_connection(&self, config: &PlcConnectionConfig) -> AppResult<ConnectionTestResult> {
        self.simulate_delay().await;
        
        Ok(ConnectionTestResult {
            success: true,
            connection_time_ms: 150,
            error_message: None,
            protocol_info: Some(format!("{:?} v1.0", config.protocol)),
            device_info: Some("Mock PLC Device".to_string()),
        })
    }
}
