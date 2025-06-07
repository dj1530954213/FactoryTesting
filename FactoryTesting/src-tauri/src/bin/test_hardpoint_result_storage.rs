/// 测试硬点测试结果存储功能
/// 验证百分比测试结果和硬点状态是否正确存储到数据库

use std::collections::HashMap;
use std::sync::Arc;
use chrono::Utc;
use app_lib::models::structs::{ChannelTestInstance, RawTestOutcome, AnalogReadingPoint, SubTestExecutionResult};
use app_lib::models::enums::{SubTestItem, SubTestStatus};
use app_lib::services::domain::channel_state_manager::{ChannelStateManager, IChannelStateManager};
use app_lib::services::infrastructure::persistence::persistence_service::PersistenceServiceFactory;
use app_lib::traits::PersistenceService;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    env_logger::init();
    
    println!("🧪 开始测试硬点测试结果存储功能");
    
    // 创建内存数据库
    let persistence_service = Arc::new(PersistenceServiceFactory::create_default_sqlite_service().await?);
    let state_manager = ChannelStateManager::new(persistence_service.clone());
    
    // 创建测试实例
    let mut test_instance = ChannelTestInstance::new(
        "test_def_001".to_string(),
        "test_batch_001".to_string(),
    );
    
    // 初始化硬点测试项
    test_instance.sub_test_results.insert(
        SubTestItem::HardPoint,
        SubTestExecutionResult::new(
            SubTestStatus::NotTested,
            None,
            None,
            None,
        ),
    );
    
    println!("✅ 创建测试实例: {}", test_instance.instance_id);
    
    // 创建模拟的硬点测试结果
    let test_readings = vec![
        AnalogReadingPoint {
            set_percentage: 0.0,
            set_value_eng: 0.0,
            expected_reading_raw: Some(0.0),
            actual_reading_raw: Some(0.1),
            actual_reading_eng: Some(0.1),
            status: SubTestStatus::Passed,
            error_percentage: Some(0.1),
        },
        AnalogReadingPoint {
            set_percentage: 0.25,
            set_value_eng: 25.0,
            expected_reading_raw: Some(25.0),
            actual_reading_raw: Some(25.2),
            actual_reading_eng: Some(25.2),
            status: SubTestStatus::Passed,
            error_percentage: Some(0.8),
        },
        AnalogReadingPoint {
            set_percentage: 0.5,
            set_value_eng: 50.0,
            expected_reading_raw: Some(50.0),
            actual_reading_raw: Some(49.8),
            actual_reading_eng: Some(49.8),
            status: SubTestStatus::Passed,
            error_percentage: Some(0.4),
        },
        AnalogReadingPoint {
            set_percentage: 0.75,
            set_value_eng: 75.0,
            expected_reading_raw: Some(75.0),
            actual_reading_raw: Some(75.1),
            actual_reading_eng: Some(75.1),
            status: SubTestStatus::Passed,
            error_percentage: Some(0.13),
        },
        AnalogReadingPoint {
            set_percentage: 1.0,
            set_value_eng: 100.0,
            expected_reading_raw: Some(100.0),
            actual_reading_raw: Some(99.9),
            actual_reading_eng: Some(99.9),
            status: SubTestStatus::Passed,
            error_percentage: Some(0.1),
        },
    ];
    
    // 创建硬点测试结果
    let hardpoint_outcome = RawTestOutcome {
        channel_instance_id: test_instance.instance_id.clone(),
        sub_test_item: SubTestItem::HardPoint,
        success: true,
        raw_value_read: Some("多点测试".to_string()),
        eng_value_calculated: Some("0.0-100.0".to_string()),
        message: Some("AI硬点5点测试全部通过".to_string()),
        start_time: Utc::now(),
        end_time: Utc::now(),
        readings: Some(test_readings.clone()),
        test_result_0_percent: Some(0.1),
        test_result_25_percent: Some(25.2),
        test_result_50_percent: Some(49.8),
        test_result_75_percent: Some(75.1),
        test_result_100_percent: Some(99.9),
        details: HashMap::new(),
    };
    
    println!("✅ 创建硬点测试结果，包含5个百分比测试点");
    
    // 应用测试结果
    state_manager.apply_raw_outcome(&mut test_instance, hardpoint_outcome).await?;
    
    println!("✅ 应用测试结果到测试实例");
    
    // 验证测试实例状态
    if let Some(hardpoint_result) = test_instance.sub_test_results.get(&SubTestItem::HardPoint) {
        println!("🔍 硬点测试状态: {:?}", hardpoint_result.status);
        println!("🔍 硬点测试实际值: {:?}", hardpoint_result.actual_value);
        println!("🔍 硬点测试期望值: {:?}", hardpoint_result.expected_value);
        println!("🔍 硬点测试详情: {:?}", hardpoint_result.details);
        
        assert_eq!(hardpoint_result.status, SubTestStatus::Passed);
    } else {
        panic!("❌ 硬点测试结果未找到");
    }
    
    // 验证百分比测试结果是否存储到临时数据中
    println!("🔍 检查临时数据中的百分比测试结果:");
    for (key, value) in &test_instance.transient_data {
        if key.contains("test_result_") {
            println!("   {}: {:?}", key, value);
        }
    }
    
    // 验证硬点读数是否存储
    if let Some(readings) = &test_instance.hardpoint_readings {
        println!("🔍 硬点读数数量: {}", readings.len());
        assert_eq!(readings.len(), 5);
    } else {
        panic!("❌ 硬点读数未存储");
    }
    
    // 保存到数据库
    persistence_service.save_test_instance(&test_instance).await?;
    println!("✅ 测试实例已保存到数据库");

    // 从数据库重新加载并验证
    let loaded_instance = persistence_service
        .load_test_instance(&test_instance.instance_id)
        .await?
        .expect("测试实例应该存在");
    
    println!("✅ 从数据库重新加载测试实例");
    
    // 验证硬点测试状态是否正确存储
    if let Some(hardpoint_result) = loaded_instance.sub_test_results.get(&SubTestItem::HardPoint) {
        println!("🔍 重新加载后的硬点测试状态: {:?}", hardpoint_result.status);
        assert_eq!(hardpoint_result.status, SubTestStatus::Passed);
    }
    
    // 验证百分比测试结果是否正确存储
    println!("🔍 重新加载后的百分比测试结果:");
    for (key, value) in &loaded_instance.transient_data {
        if key.contains("test_result_") {
            println!("   {}: {:?}", key, value);
        }
    }
    
    // 验证百分比数据是否正确
    let expected_results = vec![
        ("test_result_0_percent", 0.1),
        ("test_result_25_percent", 25.2),
        ("test_result_50_percent", 49.8),
        ("test_result_75_percent", 75.1),
        ("test_result_100_percent", 99.9),
    ];
    
    for (key, expected_value) in expected_results {
        if let Some(actual_value) = loaded_instance.transient_data.get(key) {
            let actual_f64 = actual_value.as_f64().expect("应该是数字");
            assert!((actual_f64 - expected_value).abs() < 0.01, 
                "{}的值不匹配: 期望{}, 实际{}", key, expected_value, actual_f64);
            println!("✅ {} 验证通过: {}", key, actual_f64);
        } else {
            panic!("❌ {} 未找到", key);
        }
    }
    
    println!("🎉 所有测试通过！硬点测试结果存储功能正常工作");
    
    Ok(())
}
