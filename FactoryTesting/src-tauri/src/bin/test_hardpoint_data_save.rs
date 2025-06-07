use app_lib::services::infrastructure::persistence::{SqliteOrmPersistenceService, PersistenceConfig};
use app_lib::services::traits::PersistenceService;
use app_lib::models::{ChannelTestInstance, RawTestOutcome, SubTestItem, OverallTestStatus, SubTestExecutionResult, SubTestStatus, AnalogReadingPoint};
use chrono::Utc;
use std::path::PathBuf;
use std::collections::HashMap;
use serde_json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    
    println!("=== 测试硬点数据保存功能 ===");
    
    // 初始化持久化服务
    let db_path = PathBuf::from("data/factory_testing_data.sqlite");
    let persistence_config = PersistenceConfig {
        storage_root_dir: PathBuf::from("data"),
        channel_definitions_dir: "channel_definitions".to_string(),
        test_instances_dir: "test_instances".to_string(),
        test_batches_dir: "test_batches".to_string(),
        test_outcomes_dir: "test_outcomes".to_string(),
        enable_auto_backup: false,
        backup_retention_days: 30,
        max_file_size_mb: 100,
        enable_compression: false,
    };
    
    let persistence_service = SqliteOrmPersistenceService::new(persistence_config, Some(&db_path)).await?;
    
    // 创建一个测试实例
    let mut test_instance = ChannelTestInstance::new(
        "test_definition_123".to_string(),
        "test_batch_456".to_string(),
    );
    
    // 注意：ChannelTestInstance结构体没有channel_tag字段，这个信息在definition中
    test_instance.overall_status = OverallTestStatus::HardPointTestInProgress;
    
    // 初始化子测试结果
    test_instance.sub_test_results.insert(
        SubTestItem::HardPoint,
        SubTestExecutionResult::new(SubTestStatus::NotTested, None, None, None)
    );
    
    println!("📝 创建测试实例: {}", test_instance.instance_id);
    
    // 创建硬点测试结果
    let readings = vec![
        AnalogReadingPoint {
            set_percentage: 0.0,
            set_value_eng: 0.0,
            expected_reading_raw: Some(0.0),
            actual_reading_raw: Some(0.1),
            actual_reading_eng: Some(0.05),
            status: SubTestStatus::Passed,
            error_percentage: Some(0.5),
        },
        AnalogReadingPoint {
            set_percentage: 25.0,
            set_value_eng: 1.0,
            expected_reading_raw: Some(25.0),
            actual_reading_raw: Some(25.2),
            actual_reading_eng: Some(1.008),
            status: SubTestStatus::Passed,
            error_percentage: Some(0.8),
        },
        AnalogReadingPoint {
            set_percentage: 50.0,
            set_value_eng: 2.0,
            expected_reading_raw: Some(50.0),
            actual_reading_raw: Some(49.8),
            actual_reading_eng: Some(1.992),
            status: SubTestStatus::Passed,
            error_percentage: Some(0.4),
        },
        AnalogReadingPoint {
            set_percentage: 75.0,
            set_value_eng: 3.0,
            expected_reading_raw: Some(75.0),
            actual_reading_raw: Some(75.1),
            actual_reading_eng: Some(3.004),
            status: SubTestStatus::Passed,
            error_percentage: Some(0.13),
        },
        AnalogReadingPoint {
            set_percentage: 100.0,
            set_value_eng: 4.0,
            expected_reading_raw: Some(100.0),
            actual_reading_raw: Some(99.9),
            actual_reading_eng: Some(3.996),
            status: SubTestStatus::Passed,
            error_percentage: Some(0.1),
        },
    ];
    
    // 创建RawTestOutcome，包含百分比测试结果
    let outcome = RawTestOutcome {
        channel_instance_id: test_instance.instance_id.clone(),
        sub_test_item: SubTestItem::HardPoint,
        success: true,
        raw_value_read: Some("多点测试".to_string()),
        eng_value_calculated: Some("0.00-4.00".to_string()),
        message: Some("AI硬点5点测试全部通过".to_string()),
        start_time: Utc::now(),
        end_time: Utc::now(),
        readings: Some(readings.clone()),
        test_result_0_percent: Some(0.05),
        test_result_25_percent: Some(1.008),
        test_result_50_percent: Some(1.992),
        test_result_75_percent: Some(3.004),
        test_result_100_percent: Some(3.996),
        details: HashMap::new(),
    };
    
    println!("📊 创建硬点测试结果:");
    println!("   0%: {:?}", outcome.test_result_0_percent);
    println!("   25%: {:?}", outcome.test_result_25_percent);
    println!("   50%: {:?}", outcome.test_result_50_percent);
    println!("   75%: {:?}", outcome.test_result_75_percent);
    println!("   100%: {:?}", outcome.test_result_100_percent);
    
    // 手动应用测试结果到实例（模拟状态管理器的逻辑）
    if let Some(sub_result) = test_instance.sub_test_results.get_mut(&SubTestItem::HardPoint) {
        sub_result.status = SubTestStatus::Passed;
        sub_result.timestamp = outcome.end_time;
        sub_result.actual_value = outcome.raw_value_read.clone();
        sub_result.expected_value = outcome.eng_value_calculated.clone();
        sub_result.details = outcome.message.clone();
    }
    
    // 存储硬点读数
    test_instance.hardpoint_readings = outcome.readings.clone();
    
    // 存储百分比测试结果到transient_data
    test_instance.transient_data.insert("test_result_0_percent".to_string(),
        serde_json::json!(outcome.test_result_0_percent));
    test_instance.transient_data.insert("test_result_25_percent".to_string(),
        serde_json::json!(outcome.test_result_25_percent));
    test_instance.transient_data.insert("test_result_50_percent".to_string(),
        serde_json::json!(outcome.test_result_50_percent));
    test_instance.transient_data.insert("test_result_75_percent".to_string(),
        serde_json::json!(outcome.test_result_75_percent));
    test_instance.transient_data.insert("test_result_100_percent".to_string(),
        serde_json::json!(outcome.test_result_100_percent));
    
    test_instance.overall_status = OverallTestStatus::HardPointTestCompleted;
    
    println!("💾 保存测试实例到数据库...");
    
    // 保存测试实例
    persistence_service.save_test_instance(&test_instance).await?;
    
    println!("✅ 测试实例已保存");
    
    // 保存原始测试结果
    persistence_service.save_test_outcome(&outcome).await?;
    
    println!("✅ 原始测试结果已保存");
    
    // 立即验证数据是否正确保存
    println!("\n🔍 验证数据是否正确保存...");
    
    // 从数据库重新加载测试实例
    match persistence_service.load_test_instance(&test_instance.instance_id).await? {
        Some(loaded_instance) => {
            println!("✅ 成功从数据库加载测试实例");
            println!("   实例ID: {}", loaded_instance.instance_id);
            println!("   定义ID: {}", loaded_instance.definition_id);
            println!("   整体状态: {:?}", loaded_instance.overall_status);
            
            // 检查百分比测试结果
            println!("\n📊 百分比测试结果验证:");
            println!("   0%: {:?}", loaded_instance.transient_data.get("test_result_0_percent"));
            println!("   25%: {:?}", loaded_instance.transient_data.get("test_result_25_percent"));
            println!("   50%: {:?}", loaded_instance.transient_data.get("test_result_50_percent"));
            println!("   75%: {:?}", loaded_instance.transient_data.get("test_result_75_percent"));
            println!("   100%: {:?}", loaded_instance.transient_data.get("test_result_100_percent"));
            
            // 检查硬点读数
            if let Some(readings) = &loaded_instance.hardpoint_readings {
                println!("\n📈 硬点读数验证:");
                for reading in readings.iter() {
                    println!("   {}%: 设定={:.3}, 实际原始={:.3}, 实际工程量={:.3}",
                        reading.set_percentage,
                        reading.set_value_eng,
                        reading.actual_reading_raw.unwrap_or(0.0),
                        reading.actual_reading_eng.unwrap_or(0.0));
                }
            } else {
                println!("❌ 硬点读数数据丢失");
            }
        }
        None => {
            println!("❌ 无法从数据库加载测试实例");
        }
    }
    
    // 验证原始测试结果 - 注意：持久化服务没有load_test_outcome方法
    // 我们可以通过其他方式验证，比如查看数据库中的raw_test_outcomes表
    println!("\n📊 原始测试结果已保存到数据库");
    
    println!("\n🎉 硬点数据保存测试完成！");
    
    Ok(())
}
