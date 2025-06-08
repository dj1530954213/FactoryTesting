use super::*;

/// 测试数据生成器
pub struct TestDataGenerator;

impl TestDataGenerator {
    /// 生成通道点位定义
    pub fn generate_channel_definitions(count: usize) -> Vec<ChannelPointDefinition> {
        (0..count)
            .map(|i| {
                ChannelPointDefinition::new(
                    format!("TEST_{:03}", i),
                    format!("Variable_{:03}", i),
                    format!("Test Channel {}", i),
                    format!("Station_{}", i % 5 + 1),
                    format!("Module_{}", i % 10 + 1),
                    match i % 4 {
                        0 => crate::models::enums::ModuleType::AI,
                        1 => crate::models::enums::ModuleType::AO,
                        2 => crate::models::enums::ModuleType::DI,
                        _ => crate::models::enums::ModuleType::DO,
                    },
                    format!("CH{:02}", i % 16 + 1),
                    crate::models::enums::PointDataType::Float,
                    format!("DB1.DBD{}", i * 4),
                )
            })
            .collect()
    }

    /// 生成测试批次信息
    pub fn generate_batch_info() -> TestBatchInfo {
        let mut batch = TestBatchInfo::new(
            Some("TestProduct_V1.0".to_string()),
            Some(format!("SN{:08}", rand::random::<u32>())),
        );
        batch.total_points = 88;
        batch.operator_name = Some("Test Operator".to_string());
        batch
    }

    /// 生成测试实例
    pub fn generate_test_instances(
        definitions: &[ChannelPointDefinition],
        batch_id: &str,
    ) -> Vec<ChannelTestInstance> {
        definitions
            .iter()
            .map(|def| ChannelTestInstance::new(def.id.clone(), batch_id.to_string()))
            .collect()
    }

    /// 生成测试结果
    pub fn generate_test_outcome(
        instance_id: &str,
        test_item: crate::models::enums::SubTestItem,
        success: bool,
    ) -> RawTestOutcome {
        let now = chrono::Utc::now();
        RawTestOutcome {
            channel_instance_id: instance_id.to_string(),
            sub_test_item: test_item,
            success,
            raw_value_read: Some(format!("{:.2}", rand::random::<f32>() * 100.0)),
            eng_value_calculated: Some(format!("{:.2}", rand::random::<f32>() * 100.0)),
            message: if success {
                Some("Test completed successfully".to_string())
            } else {
                Some("Test failed due to value out of range".to_string())
            },
            start_time: now,
            end_time: now,
            readings: None,
            digital_steps: None,
            test_result_0_percent: None,
            test_result_25_percent: None,
            test_result_50_percent: None,
            test_result_75_percent: None,
            test_result_100_percent: None,
            details: HashMap::new(),
        }
    }

    /// 生成PLC连接配置
    pub fn generate_plc_config() -> PlcConnectionConfig {
        PlcConnectionConfig {
            id: uuid::Uuid::new_v4().to_string(),
            name: "Test PLC Connection".to_string(),
            protocol: PlcProtocol::ModbusTcp,
            host: "192.168.1.100".to_string(),
            port: 502,
            timeout_ms: 5000,
            read_timeout_ms: 3000,
            write_timeout_ms: 3000,
            retry_count: 3,
            retry_interval_ms: 1000,
            protocol_params: HashMap::new(),
        }
    }

    /// 生成分配策略
    pub fn generate_allocation_strategy() -> AllocationStrategy {
        AllocationStrategy {
            name: "Test Strategy".to_string(),
            mode: AllocationMode::Sequential,
            priority_rules: vec![],
            grouping_rules: vec![],
            filter_rules: vec![],
            max_batch_size: Some(88),
            allow_partial_allocation: true,
            allocation_timeout_ms: 30000,
        }
    }

    /// 生成测试执行配置
    pub fn generate_test_execution_config() -> TestExecutionConfig {
        TestExecutionConfig {
            max_concurrent_tests: 88,
            test_timeout_ms: 30000,
            enable_auto_retry: true,
            max_retry_count: 3,
            retry_interval_ms: 1000,
            stop_on_error: false,
            priority: TestPriority::Normal,
        }
    }

    /// 生成完整的测试场景
    pub fn generate_test_scenario(channel_count: usize) -> TestScenario {
        let definitions = Self::generate_channel_definitions(channel_count);
        let batch_info = Self::generate_batch_info();
        let test_instances = Self::generate_test_instances(&definitions, &batch_info.batch_id);
        let plc_config = Self::generate_plc_config();
        let allocation_strategy = Self::generate_allocation_strategy();
        let execution_config = Self::generate_test_execution_config();

        TestScenario {
            name: format!("Test Scenario with {} channels", channel_count),
            description: "Generated test scenario".to_string(),
            channel_definitions: definitions,
            batch_info,
            test_instances,
            plc_config,
            allocation_strategy,
            execution_config,
        }
    }
}

/// 测试场景
#[derive(Debug, Clone)]
pub struct TestScenario {
    pub name: String,
    pub description: String,
    pub channel_definitions: Vec<ChannelPointDefinition>,
    pub batch_info: TestBatchInfo,
    pub test_instances: Vec<ChannelTestInstance>,
    pub plc_config: PlcConnectionConfig,
    pub allocation_strategy: AllocationStrategy,
    pub execution_config: TestExecutionConfig,
}

impl TestScenario {
    /// 创建小型测试场景（10个通道）
    pub fn small() -> Self {
        TestDataGenerator::generate_test_scenario(10)
    }

    /// 创建中型测试场景（50个通道）
    pub fn medium() -> Self {
        TestDataGenerator::generate_test_scenario(50)
    }

    /// 创建大型测试场景（88个通道）
    pub fn large() -> Self {
        TestDataGenerator::generate_test_scenario(88)
    }

    /// 创建自定义大小的测试场景
    pub fn custom(channel_count: usize) -> Self {
        TestDataGenerator::generate_test_scenario(channel_count)
    }
}
