#[cfg(test)]
mod tests {
    use crate::models::*;
    use serde_json;

    /// 测试枚举类型的序列化和反序列化
    #[test]
    fn test_enum_serialization() {
        // 测试 OverallTestStatus
        let status = OverallTestStatus::TestCompletedPassed;
        let json_str = serde_json::to_string(&status).unwrap();
        let deserialized: OverallTestStatus = serde_json::from_str(&json_str).unwrap();
        assert_eq!(status, deserialized);

        // 测试 SubTestStatus
        let sub_status = SubTestStatus::Passed;
        let json_str = serde_json::to_string(&sub_status).unwrap();
        let deserialized: SubTestStatus = serde_json::from_str(&json_str).unwrap();
        assert_eq!(sub_status, deserialized);

        // 测试 ModuleType
        let module_type = ModuleType::AI;
        let json_str = serde_json::to_string(&module_type).unwrap();
        let deserialized: ModuleType = serde_json::from_str(&json_str).unwrap();
        assert_eq!(module_type, deserialized);

        // 测试 ModuleType::Other 变体
        let module_type_other = ModuleType::Other("CustomModule".to_string());
        let json_str = serde_json::to_string(&module_type_other).unwrap();
        let deserialized: ModuleType = serde_json::from_str(&json_str).unwrap();
        assert_eq!(module_type_other, deserialized);

        // 测试 PointDataType
        let data_type = PointDataType::Float;
        let json_str = serde_json::to_string(&data_type).unwrap();
        let deserialized: PointDataType = serde_json::from_str(&json_str).unwrap();
        assert_eq!(data_type, deserialized);

        // 测试 SubTestItem
        let sub_test_item = SubTestItem::HardPoint;
        let json_str = serde_json::to_string(&sub_test_item).unwrap();
        let deserialized: SubTestItem = serde_json::from_str(&json_str).unwrap();
        assert_eq!(sub_test_item, deserialized);

        // 测试 SubTestItem::Custom 变体
        let custom_test = SubTestItem::Custom("CustomTest".to_string());
        let json_str = serde_json::to_string(&custom_test).unwrap();
        let deserialized: SubTestItem = serde_json::from_str(&json_str).unwrap();
        assert_eq!(custom_test, deserialized);
    }

    /// 测试 ChannelPointDefinition 的创建和序列化
    #[test]
    fn test_channel_point_definition() {
        let definition = ChannelPointDefinition::new(
            "AI001".to_string(),
            "Temperature_1".to_string(),
            "反应器温度".to_string(),
            "Station1".to_string(),
            "Module1".to_string(),
            ModuleType::AI,
            "CH01".to_string(),
            PointDataType::Float,
            "DB1.DBD0".to_string(),
        );

        // 验证基本字段
        assert_eq!(definition.tag, "AI001");
        assert_eq!(definition.variable_name, "Temperature_1");
        assert_eq!(definition.module_type, ModuleType::AI);
        assert_eq!(definition.data_type, PointDataType::Float);
        assert_eq!(definition.plc_communication_address, "DB1.DBD0");
        assert_eq!(definition.power_supply_type, "有源");
        assert_eq!(definition.wire_system, "4线制");
        assert!(!definition.id.is_empty());

        // 测试序列化和反序列化
        let json_str = serde_json::to_string(&definition).unwrap();
        let deserialized: ChannelPointDefinition = serde_json::from_str(&json_str).unwrap();
        assert_eq!(definition.tag, deserialized.tag);
        assert_eq!(definition.id, deserialized.id);
        assert_eq!(definition.module_type, deserialized.module_type);
    }

    /// 测试 ChannelTestInstance 的创建和操作
    #[test]
    fn test_channel_test_instance() {
        let definition_id = "def_1".to_string();
        let batch_id = "batch_1".to_string();
        let instance = ChannelTestInstance::new(definition_id.clone(), batch_id.clone());

        assert!(!instance.instance_id.is_empty());
        assert_eq!(instance.definition_id, definition_id);
        assert_eq!(instance.test_batch_id, batch_id);
        assert_eq!(instance.overall_status, OverallTestStatus::NotTested);
        assert!(instance.start_time.is_none());
        assert!(instance.sub_test_results.is_empty());
    }

    /// 测试 TestBatchInfo 的创建和操作
    #[test]
    fn test_test_batch_info() {
        let mut batch = TestBatchInfo::new(
            Some("ProductV1.0".to_string()),
            Some("SN123456".to_string()),
        );

        // 验证初始状态
        assert_eq!(batch.product_model, Some("ProductV1.0".to_string()));
        assert_eq!(batch.serial_number, Some("SN123456".to_string()));
        assert_eq!(batch.total_points, 0);
        assert_eq!(batch.tested_points, 0);
        assert_eq!(batch.passed_points, 0);
        assert_eq!(batch.failed_points, 0);
        assert_eq!(batch.skipped_points, 0);
        assert!(batch.custom_data.is_empty());
        assert!(!batch.batch_id.is_empty());

        // 更新统计信息
        batch.total_points = 100;
        batch.tested_points = 80;
        batch.passed_points = 70;
        batch.failed_points = 8;
        batch.skipped_points = 2;
        batch.operator_name = Some("张三".to_string());

        // 添加自定义数据
        batch.custom_data.insert("customer".to_string(), "客户A".to_string());
        batch.custom_data.insert("project".to_string(), "项目X".to_string());

        // 测试序列化和反序列化
        let json_str = serde_json::to_string(&batch).unwrap();
        let deserialized: TestBatchInfo = serde_json::from_str(&json_str).unwrap();
        assert_eq!(batch, deserialized);
    }

    /// 测试 RawTestOutcome 的创建和操作
    #[test]
    fn test_raw_test_outcome() {
        let instance_id = "instance123".to_string();
        let sub_test_item = SubTestItem::HardPoint;

        // 测试成功结果
        let success_outcome = RawTestOutcome::success(instance_id.clone(), sub_test_item.clone());
        assert_eq!(success_outcome.channel_instance_id, instance_id);
        assert_eq!(success_outcome.sub_test_item, sub_test_item);
        assert!(success_outcome.success);
        assert!(success_outcome.message.is_none());
        assert!(success_outcome.readings.as_ref().map_or(true, |v| v.is_empty()), "Readings should be None or empty Vec");

        // 测试失败结果
        let failure_outcome = RawTestOutcome::failure(
            instance_id.clone(),
            SubTestItem::LowAlarm,
            "通信超时".to_string(),
        );
        assert_eq!(failure_outcome.channel_instance_id, instance_id);
        assert_eq!(failure_outcome.sub_test_item, SubTestItem::LowAlarm);
        assert!(!failure_outcome.success);
        assert_eq!(failure_outcome.message, Some("通信超时".to_string()));

        // 测试序列化和反序列化
        let json_str = serde_json::to_string(&success_outcome).unwrap();
        let deserialized: RawTestOutcome = serde_json::from_str(&json_str).unwrap();
        assert_eq!(success_outcome, deserialized);
    }

    /// 测试 SubTestExecutionResult 的工厂方法
    #[test]
    fn test_sub_test_execution_result() {
        // 测试通过状态
        let passed_result = SubTestExecutionResult { status: SubTestStatus::Passed, ..Default::default() };
        assert_eq!(passed_result.status, SubTestStatus::Passed);
        assert!(passed_result.details.is_none());

        // 测试失败状态
        let failed_result = SubTestExecutionResult { status: SubTestStatus::Failed, details: Some("测试失败原因".to_string()), ..Default::default() };
        assert_eq!(failed_result.status, SubTestStatus::Failed);
        assert_eq!(failed_result.details, Some("测试失败原因".to_string()));

        // 测试不适用状态
        let not_applicable_result = SubTestExecutionResult { status: SubTestStatus::NotApplicable, ..Default::default() };
        assert_eq!(not_applicable_result.status, SubTestStatus::NotApplicable);

        // 测试默认状态
        let default_result = SubTestExecutionResult::default();
        assert_eq!(default_result.status, SubTestStatus::NotTested);

        // 测试序列化和反序列化
        let json_str = serde_json::to_string(&passed_result).unwrap();
        let deserialized: SubTestExecutionResult = serde_json::from_str(&json_str).unwrap();
        assert_eq!(passed_result.status, deserialized.status);
    }

    /// 测试 AnalogReadingPoint 的创建和操作
    #[test]
    fn test_analog_reading_point() {
        let mut reading_point = AnalogReadingPoint {
            set_percentage: 0.25,
            set_value_eng: 5.0,
            status: SubTestStatus::NotTested,
            ..Default::default()
        };

        // 验证初始状态
        assert_eq!(reading_point.set_percentage, 0.25);
        assert_eq!(reading_point.set_value_eng, 5.0);
        assert_eq!(reading_point.status, SubTestStatus::NotTested);
        assert!(reading_point.expected_reading_raw.is_none());
        assert!(reading_point.actual_reading_raw.is_none());
        assert!(reading_point.error_percentage.is_none());

        // 更新读数数据
        reading_point.expected_reading_raw = Some(4095.0);
        reading_point.actual_reading_raw = Some(4090.0);
        reading_point.actual_reading_eng = Some(4.99);
        reading_point.error_percentage = Some(0.2);
        reading_point.status = SubTestStatus::Passed;

        // 测试序列化和反序列化
        let json_str = serde_json::to_string(&reading_point).unwrap();
        let deserialized: AnalogReadingPoint = serde_json::from_str(&json_str).unwrap();
        assert_eq!(reading_point.set_percentage, deserialized.set_percentage);
        assert_eq!(reading_point.status, deserialized.status);
        assert_eq!(reading_point.error_percentage, deserialized.error_percentage);
    }

    /// 测试默认值的正确性
    #[test]
    fn test_default_implementations() {
        let _default_enum_status = OverallTestStatus::default();
        let _default_sub_status = SubTestStatus::default();
        let _default_module_type = ModuleType::default();

        let default_instance = ChannelTestInstance::new("def_id".to_string(), "batch_id".to_string());
        assert_eq!(default_instance.overall_status, OverallTestStatus::NotTested);

        let default_batch = TestBatchInfo::new(Some("model_x".to_string()), Some("sn_123".to_string()));
        assert!(default_batch.batch_id.len() > 0);
        assert_eq!(default_batch.total_points, 0);

        let default_outcome = RawTestOutcome::default();
        assert!(!default_outcome.channel_instance_id.is_empty());

        let default_sub_result = SubTestExecutionResult::default();
        assert_eq!(default_sub_result.status, SubTestStatus::NotTested);

        let default_reading_point = AnalogReadingPoint::default();
        assert_eq!(default_reading_point.set_percentage, 0.0);
    }

    /// 测试 UUID 生成的唯一性
    #[test]
    fn test_uuid_uniqueness() {
        let id1 = default_id();
        let id2 = default_id();
        
        // UUID 应该是不同的
        assert_ne!(id1, id2);
        
        // UUID 格式应该正确（36个字符，包含4个连字符）
        assert_eq!(id1.len(), 36);
        assert_eq!(id1.matches('-').count(), 4);
    }

    // /// 测试复杂的嵌套数据结构序列化
    // #[test]
    // fn test_complex_nested_serialization() {
    //     // 这个测试用例之前被注释掉了，现在尝试修复并启用它
    //     // let mut instance = ChannelTestInstance::new("def_complex".to_string(), "batch_complex".to_string());
    //     // instance.overall_status = OverallTestStatus::HardPointTesting;
    //     // ... (其余实现暂时注释) ...
    // }

    // /// 测试 RawTestOutcome 的创建和操作
    // #[test]
    // fn test_raw_test_outcome_with_readings() {
    //     let mut outcome = RawTestOutcome::success("inst_readings".to_string(), SubTestItem::HardPoint);
        
    //     outcome.readings = Some(Vec::new());

    //     if let Some(readings_vec) = outcome.readings.as_mut() {
    //         readings_vec.push(AnalogReadingPoint { set_percentage: 0.0, set_value_eng: 0.0, status: SubTestStatus::Passed, ..Default::default() });
    //         readings_vec.push(AnalogReadingPoint { set_percentage: 0.25, set_value_eng: 5.0, status: SubTestStatus::Passed, ..Default::default() });
    //         readings_vec.push(AnalogReadingPoint { set_percentage: 1.0, set_value_eng: 20.0, status: SubTestStatus::Passed, ..Default::default() });
    //     }

    //     assert!(outcome.readings.is_some());
    //     assert_eq!(outcome.readings.as_ref().unwrap().len(), 3);

    //     let json_string = serde_json::to_string_pretty(&outcome).unwrap();
    //     let deserialized: RawTestOutcome = serde_json::from_str(&json_string).unwrap();
    //     assert_eq!(outcome, deserialized);
    //     assert_eq!(deserialized.readings.as_ref().unwrap().len(), 3);
    // }
} 