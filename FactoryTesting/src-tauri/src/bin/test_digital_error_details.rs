// 测试DI/DO错误详情显示功能
// 验证数字量测试步骤数据的存储和前端显示

use app_lib::models::structs::{ChannelTestInstance, ChannelPointDefinition, RawTestOutcome, DigitalTestStep};
use app_lib::models::enums::{ModuleType, OverallTestStatus, SubTestItem, SubTestStatus, PointDataType};
use app_lib::services::domain::channel_state_manager::ChannelStateManager;
use app_lib::services::infrastructure::persistence::sqlite_orm_persistence_service::SqliteOrmPersistenceService;
use app_lib::services::infrastructure::persistence::PersistenceConfig;
use app_lib::IChannelStateManager;
use chrono::Utc;
use std::collections::HashMap;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 开始测试DI/DO错误详情显示功能...");

    // 初始化数据库连接
    let config = PersistenceConfig::default();

    let persistence_service = Arc::new(SqliteOrmPersistenceService::new(config, Some(std::path::Path::new("./test_digital_error.db"))).await?);
    let state_manager = Arc::new(ChannelStateManager::new(persistence_service.clone()));

    // 创建测试用的DI点位定义
    let di_definition = ChannelPointDefinition {
        id: "test_di_001".to_string(),
        batch_id: "test_batch_001".to_string(),
        station_name: "测试站".to_string(),
        module_name: "DI模块".to_string(),
        channel_tag_in_module: "DI_TEST_001".to_string(),
        power_supply_type: 1, // 有源
        variable_name: "TEST_DI_VALVE".to_string(),
        variable_description: "测试阀门状态".to_string(),
        module_type: ModuleType::DI,
        data_type: PointDataType::Bool,
        plc_communication_address: "40101".to_string(),
        plc_absolute_address: Some("%MD101".to_string()),
        range_high_limit: None,
        range_low_limit: None,
        engineering_unit: None,
        sll_set_point_plc_address: None,
        sll_set_point_communication_address: None,
        sl_set_point_plc_address: None,
        sl_set_point_communication_address: None,
        sh_set_point_plc_address: None,
        sh_set_point_communication_address: None,
        shh_set_point_plc_address: None,
        shh_set_point_communication_address: None,
        tag: "DI_TEST_001".to_string(),
        channel_position: "1".to_string(),
        customer: "测试客户".to_string(),
        sequence: 1,
        description: "测试阀门状态".to_string(),
        communication_address: "40101".to_string(),
        test_plc_address: "50001".to_string(),
    };

    // 创建测试用的DO点位定义
    let do_definition = ChannelPointDefinition {
        id: "test_do_001".to_string(),
        batch_id: "test_batch_001".to_string(),
        station_name: "测试站".to_string(),
        module_name: "DO模块".to_string(),
        channel_tag_in_module: "DO_TEST_001".to_string(),
        power_supply_type: 1, // 有源
        variable_name: "TEST_DO_PUMP".to_string(),
        variable_description: "测试泵控制".to_string(),
        module_type: ModuleType::DO,
        data_type: PointDataType::Bool,
        plc_communication_address: "40201".to_string(),
        plc_absolute_address: Some("%MD201".to_string()),
        range_high_limit: None,
        range_low_limit: None,
        engineering_unit: None,
        sll_set_point_plc_address: None,
        sll_set_point_communication_address: None,
        sl_set_point_plc_address: None,
        sl_set_point_communication_address: None,
        sh_set_point_plc_address: None,
        sh_set_point_communication_address: None,
        shh_set_point_plc_address: None,
        shh_set_point_communication_address: None,
        tag: "DO_TEST_001".to_string(),
        channel_position: "1".to_string(),
        customer: "测试客户".to_string(),
        sequence: 2,
        description: "测试泵控制".to_string(),
        communication_address: "40201".to_string(),
        test_plc_address: "50101".to_string(),
    };

    // 创建DI测试实例
    let mut di_instance = ChannelTestInstance {
        instance_id: "di_test_instance_001".to_string(),
        definition_id: di_definition.id.clone(),
        test_batch_id: "test_batch_001".to_string(),
        test_batch_name: "数字量测试批次".to_string(),
        overall_status: OverallTestStatus::HardPointTesting,
        sub_test_results: HashMap::new(),
        test_plc_channel_tag: Some("TEST_PLC_DO_001".to_string()),
        test_plc_communication_address: Some("50001".to_string()),
        error_message: None,
        current_step_details: None,
        creation_time: Utc::now(),
        start_time: Some(Utc::now()),
        last_updated_time: Utc::now(),
        final_test_time: None,
        total_test_duration_ms: None,
        current_operator: Some("测试员".to_string()),
        retries_count: 0,
        transient_data: HashMap::new(),
        hardpoint_readings: None,
        digital_test_steps: None,
        manual_test_current_value_input: None,
        manual_test_current_value_output: None,
    };

    // 创建DO测试实例
    let mut do_instance = ChannelTestInstance {
        instance_id: "do_test_instance_001".to_string(),
        definition_id: do_definition.id.clone(),
        test_batch_id: "test_batch_001".to_string(),
        test_batch_name: "数字量测试批次".to_string(),
        overall_status: OverallTestStatus::HardPointTesting,
        sub_test_results: HashMap::new(),
        test_plc_channel_tag: Some("TEST_PLC_DI_001".to_string()),
        test_plc_communication_address: Some("50101".to_string()),
        error_message: None,
        current_step_details: None,
        creation_time: Utc::now(),
        start_time: Some(Utc::now()),
        last_updated_time: Utc::now(),
        final_test_time: None,
        total_test_duration_ms: None,
        current_operator: Some("测试员".to_string()),
        retries_count: 0,
        transient_data: HashMap::new(),
        hardpoint_readings: None,
        digital_test_steps: None,
        manual_test_current_value_input: None,
        manual_test_current_value_output: None,
    };

    println!("✅ 创建了测试实例");

    // 模拟DI测试失败的情况 - 创建详细的测试步骤数据
    let di_test_steps = vec![
        DigitalTestStep {
            step_number: 1,
            step_description: "测试PLC DO输出低电平，检查被测PLC DI显示断开".to_string(),
            set_value: false,
            expected_reading: false,
            actual_reading: false,
            status: SubTestStatus::Passed,
            timestamp: Utc::now(),
        },
        DigitalTestStep {
            step_number: 2,
            step_description: "测试PLC DO输出高电平，检查被测PLC DI显示接通".to_string(),
            set_value: true,
            expected_reading: true,
            actual_reading: false, // 模拟失败：期望true但实际读到false
            status: SubTestStatus::Failed,
            timestamp: Utc::now(),
        },
        DigitalTestStep {
            step_number: 3,
            step_description: "测试PLC DO复位低电平，检查被测PLC DI显示断开".to_string(),
            set_value: false,
            expected_reading: false,
            actual_reading: false,
            status: SubTestStatus::Passed,
            timestamp: Utc::now(),
        },
    ];

    // 创建DI测试结果
    let di_outcome = RawTestOutcome {
        channel_instance_id: di_instance.instance_id.clone(),
        sub_test_item: SubTestItem::HardPoint,
        success: false, // 测试失败
        raw_value_read: Some("数字量测试".to_string()),
        eng_value_calculated: Some("DI硬点测试失败".to_string()),
        message: Some("❌ DI硬点测试失败: DO高电平时DI应为true，实际为false".to_string()),
        start_time: Utc::now(),
        end_time: Utc::now(),
        readings: None,
        digital_steps: Some(di_test_steps),
        test_result_0_percent: None,
        test_result_25_percent: None,
        test_result_50_percent: None,
        test_result_75_percent: None,
        test_result_100_percent: None,
        details: HashMap::new(),
    };

    println!("📝 创建了DI测试失败结果，包含3个测试步骤");

    // 应用DI测试结果
    state_manager.apply_raw_outcome(&mut di_instance, di_outcome).await?;
    println!("✅ 应用了DI测试结果到状态管理器");

    // 模拟DO测试成功的情况
    let do_test_steps = vec![
        DigitalTestStep {
            step_number: 1,
            step_description: "被测PLC DO输出低电平，检查测试PLC DI显示断开".to_string(),
            set_value: false,
            expected_reading: false,
            actual_reading: false,
            status: SubTestStatus::Passed,
            timestamp: Utc::now(),
        },
        DigitalTestStep {
            step_number: 2,
            step_description: "被测PLC DO输出高电平，检查测试PLC DI显示接通".to_string(),
            set_value: true,
            expected_reading: true,
            actual_reading: true,
            status: SubTestStatus::Passed,
            timestamp: Utc::now(),
        },
        DigitalTestStep {
            step_number: 3,
            step_description: "被测PLC DO复位低电平，检查测试PLC DI显示断开".to_string(),
            set_value: false,
            expected_reading: false,
            actual_reading: false,
            status: SubTestStatus::Passed,
            timestamp: Utc::now(),
        },
    ];

    // 创建DO测试结果
    let do_outcome = RawTestOutcome {
        channel_instance_id: do_instance.instance_id.clone(),
        sub_test_item: SubTestItem::HardPoint,
        success: true, // 测试成功
        raw_value_read: Some("数字量测试".to_string()),
        eng_value_calculated: Some("DO硬点测试成功".to_string()),
        message: Some("✅ DO硬点测试成功: 低→高→低电平切换，测试PLC DI状态正确响应".to_string()),
        start_time: Utc::now(),
        end_time: Utc::now(),
        readings: None,
        digital_steps: Some(do_test_steps),
        test_result_0_percent: None,
        test_result_25_percent: None,
        test_result_50_percent: None,
        test_result_75_percent: None,
        test_result_100_percent: None,
        details: HashMap::new(),
    };

    println!("📝 创建了DO测试成功结果，包含3个测试步骤");

    // 应用DO测试结果
    state_manager.apply_raw_outcome(&mut do_instance, do_outcome).await?;
    println!("✅ 应用了DO测试结果到状态管理器");

    // 验证数据是否正确存储
    println!("\n🔍 验证测试结果存储:");
    
    if let Some(di_steps) = &di_instance.digital_test_steps {
        println!("DI测试步骤数量: {}", di_steps.len());
        for step in di_steps {
            println!("  步骤{}: {} - 状态: {:?}", 
                step.step_number, step.step_description, step.status);
            println!("    设定值: {}, 期望: {}, 实际: {}", 
                step.set_value, step.expected_reading, step.actual_reading);
        }
    }

    if let Some(do_steps) = &do_instance.digital_test_steps {
        println!("DO测试步骤数量: {}", do_steps.len());
        for step in do_steps {
            println!("  步骤{}: {} - 状态: {:?}", 
                step.step_number, step.step_description, step.status);
            println!("    设定值: {}, 期望: {}, 实际: {}", 
                step.set_value, step.expected_reading, step.actual_reading);
        }
    }

    println!("\n🎯 测试完成！");
    println!("现在可以在前端测试区域查看这些点位的错误详情，应该能看到详细的测试步骤信息。");
    
    Ok(())
}
