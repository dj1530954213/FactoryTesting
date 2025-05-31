//! Mock服务实现
//!
//! 用于单元测试和集成测试的Mock服务

pub mod mock_test_orchestration_service;
pub mod mock_channel_state_manager;
pub mod mock_test_execution_engine;
pub mod mock_plc_communication_service;
pub mod mock_batch_allocation_service;
pub mod mock_event_publisher;
pub mod mock_persistence_service;
pub mod test_data_generator;

// 重新导出所有Mock实现
pub use mock_test_orchestration_service::*;
pub use mock_channel_state_manager::*;
pub use mock_test_execution_engine::*;
pub use mock_plc_communication_service::*;
pub use mock_batch_allocation_service::*;
pub use mock_event_publisher::*;
pub use mock_persistence_service::*;
pub use test_data_generator::*;

use super::*;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

/// Mock配置
#[derive(Debug, Clone)]
pub struct MockConfig {
    /// 是否启用延迟模拟
    pub enable_delay_simulation: bool,

    /// 基础延迟时间（毫秒）
    pub base_delay_ms: u64,

    /// 随机延迟范围（毫秒）
    pub random_delay_range_ms: u64,

    /// 错误注入概率（0.0-1.0）
    pub error_injection_probability: f64,

    /// 是否记录调用历史
    pub record_call_history: bool,

    /// 最大调用历史记录数
    pub max_call_history: usize,
}

impl Default for MockConfig {
    fn default() -> Self {
        Self {
            enable_delay_simulation: false,
            base_delay_ms: 10,
            random_delay_range_ms: 50,
            error_injection_probability: 0.0,
            record_call_history: true,
            max_call_history: 1000,
        }
    }
}

/// 调用记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallRecord {
    /// 调用ID
    pub call_id: String,

    /// 方法名
    pub method_name: String,

    /// 参数（序列化为JSON）
    pub parameters: serde_json::Value,

    /// 调用时间
    pub timestamp: DateTime<Utc>,

    /// 执行时长（毫秒）
    pub duration_ms: u64,

    /// 是否成功
    pub success: bool,

    /// 错误信息（如果失败）
    pub error_message: Option<String>,
}

/// Mock服务基础trait
pub trait MockService {
    /// 获取Mock配置
    fn get_mock_config(&self) -> &MockConfig;

    /// 设置Mock配置
    fn set_mock_config(&mut self, config: MockConfig);

    /// 获取调用历史
    fn get_call_history(&self) -> Vec<CallRecord>;

    /// 清除调用历史
    fn clear_call_history(&mut self);

    /// 记录方法调用
    fn record_call(&mut self, method_name: &str, parameters: serde_json::Value, success: bool, duration_ms: u64, error_message: Option<String>);

    /// 模拟延迟
    async fn simulate_delay(&self) {
        if self.get_mock_config().enable_delay_simulation {
            let base_delay = self.get_mock_config().base_delay_ms;
            let random_range = self.get_mock_config().random_delay_range_ms;

            let delay = if random_range > 0 {
                base_delay + (rand::random::<u64>() % random_range)
            } else {
                base_delay
            };

            tokio::time::sleep(tokio::time::Duration::from_millis(delay)).await;
        }
    }

    /// 检查是否应该注入错误
    fn should_inject_error(&self) -> bool {
        let probability = self.get_mock_config().error_injection_probability;
        if probability <= 0.0 {
            false
        } else if probability >= 1.0 {
            true
        } else {
            rand::random::<f64>() < probability
        }
    }
}

/// Mock服务基础实现
#[derive(Debug, Clone)]
pub struct MockServiceBase {
    config: MockConfig,
    call_history: Arc<Mutex<Vec<CallRecord>>>,
}

impl MockServiceBase {
    pub fn new(config: MockConfig) -> Self {
        Self {
            config,
            call_history: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn with_default_config() -> Self {
        Self::new(MockConfig::default())
    }
}

impl MockService for MockServiceBase {
    fn get_mock_config(&self) -> &MockConfig {
        &self.config
    }

    fn set_mock_config(&mut self, config: MockConfig) {
        self.config = config;
    }

    fn get_call_history(&self) -> Vec<CallRecord> {
        self.call_history.lock().unwrap().clone()
    }

    fn clear_call_history(&mut self) {
        self.call_history.lock().unwrap().clear();
    }

    fn record_call(&mut self, method_name: &str, parameters: serde_json::Value, success: bool, duration_ms: u64, error_message: Option<String>) {
        if self.config.record_call_history {
            let mut history = self.call_history.lock().unwrap();

            let record = CallRecord {
                call_id: uuid::Uuid::new_v4().to_string(),
                method_name: method_name.to_string(),
                parameters,
                timestamp: Utc::now(),
                duration_ms,
                success,
                error_message,
            };

            history.push(record);

            // 限制历史记录数量
            if history.len() > self.config.max_call_history {
                history.remove(0);
            }
        }
    }
}

/// Mock工厂
pub struct MockFactory;

impl MockFactory {
    /// 创建测试编排服务Mock
    pub fn create_test_orchestration_service(config: Option<MockConfig>) -> MockTestOrchestrationService {
        MockTestOrchestrationService::new(config.unwrap_or_default())
    }

    /// 创建通道状态管理器Mock
    pub fn create_channel_state_manager(config: Option<MockConfig>) -> MockChannelStateManager {
        MockChannelStateManager::new(config.unwrap_or_default())
    }

    /// 创建测试执行引擎Mock
    pub fn create_test_execution_engine(config: Option<MockConfig>) -> MockTestExecutionEngine {
        MockTestExecutionEngine::new(config.unwrap_or_default())
    }

    /// 创建PLC通信服务Mock
    pub fn create_plc_communication_service(config: Option<MockConfig>) -> MockPlcCommunicationService {
        MockPlcCommunicationService::new(config.unwrap_or_default())
    }

    /// 创建批次分配服务Mock
    pub fn create_batch_allocation_service(config: Option<MockConfig>) -> MockBatchAllocationService {
        MockBatchAllocationService::new(config.unwrap_or_default())
    }

    /// 创建事件发布服务Mock
    pub fn create_event_publisher(config: Option<MockConfig>) -> MockEventPublisher {
        MockEventPublisher::new(config.unwrap_or_default())
    }

    /// 创建持久化服务Mock
    pub fn create_persistence_service(config: Option<MockConfig>) -> MockPersistenceService {
        MockPersistenceService::new(config.unwrap_or_default())
    }

    /// 创建完整的Mock服务集合
    pub fn create_full_mock_suite(config: Option<MockConfig>) -> MockServiceSuite {
        let config = config.unwrap_or_default();

        MockServiceSuite {
            test_orchestration: Self::create_test_orchestration_service(Some(config.clone())),
            channel_state_manager: Self::create_channel_state_manager(Some(config.clone())),
            test_execution_engine: Self::create_test_execution_engine(Some(config.clone())),
            plc_communication: Self::create_plc_communication_service(Some(config.clone())),
            batch_allocation: Self::create_batch_allocation_service(Some(config.clone())),
            event_publisher: Self::create_event_publisher(Some(config.clone())),
            persistence: Self::create_persistence_service(Some(config)),
        }
    }
}

/// Mock服务套件
pub struct MockServiceSuite {
    pub test_orchestration: MockTestOrchestrationService,
    pub channel_state_manager: MockChannelStateManager,
    pub test_execution_engine: MockTestExecutionEngine,
    pub plc_communication: MockPlcCommunicationService,
    pub batch_allocation: MockBatchAllocationService,
    pub event_publisher: MockEventPublisher,
    pub persistence: MockPersistenceService,
}

/// 测试场景配置
#[derive(Debug, Clone)]
pub struct TestScenarioConfig {
    /// 场景名称
    pub name: String,

    /// 场景描述
    pub description: String,

    /// Mock配置
    pub mock_config: MockConfig,

    /// 预设数据
    pub preset_data: HashMap<String, serde_json::Value>,

    /// 预期行为
    pub expected_behaviors: Vec<ExpectedBehavior>,
}

/// 预期行为
#[derive(Debug, Clone)]
pub struct ExpectedBehavior {
    /// 方法名
    pub method_name: String,

    /// 调用次数
    pub call_count: u32,

    /// 预期参数
    pub expected_parameters: Option<serde_json::Value>,

    /// 返回值
    pub return_value: Option<serde_json::Value>,

    /// 是否应该失败
    pub should_fail: bool,

    /// 错误消息
    pub error_message: Option<String>,
}
