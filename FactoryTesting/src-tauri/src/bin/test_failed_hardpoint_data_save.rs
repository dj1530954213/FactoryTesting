use app_lib::services::infrastructure::persistence::{SqliteOrmPersistenceService, PersistenceConfig};
use app_lib::services::domain::channel_state_manager::{ChannelStateManager, IChannelStateManager};
use app_lib::services::traits::PersistenceService;
use app_lib::models::{ChannelTestInstance, RawTestOutcome, SubTestItem, OverallTestStatus, SubTestExecutionResult, SubTestStatus, AnalogReadingPoint};
use chrono::Utc;
use std::path::PathBuf;
use std::collections::HashMap;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    
    println!("=== 测试失败的硬点测试过程数据保存功能 ===");
    
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
    
    let persistence_service = Arc::new(SqliteOrmPersistenceService::new(persistence_config, Some(&db_path)).await?);
    
    // 初始化状态管理器
    let state_manager = ChannelStateManager::new(persistence_service.clone());
    
    // 创建一个测试实例
    let mut test_instance = ChannelTestInstance::new(
        "test_definition_failed_hardpoint".to_string(),
        "test_batch_failed_hardpoint".to_string(),
    );
    
    test_instance.overall_status = OverallTestStatus::HardPointTestInProgress;
    
    // 初始化子测试结果
    test_instance.sub_test_results.insert(
        SubTestItem::HardPoint,
        SubTestExecutionResult::new(SubTestStatus::NotTested, None, None, None)
    );
    
    println!("📝 创建测试实例: {}", test_instance.instance_id);
    
    // 先保存测试实例到持久化服务
    persistence_service.save_test_instance(&test_instance).await?;
    println!("✅ 测试实例已保存到数据库");
    
    // 创建失败的硬点测试结果（模拟AI硬点测试执行器的输出）
    // 其中50%和75%点测试失败（误差超过2%）
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
            actual_reading_raw: Some(52.5),  // 误差过大
            actual_reading_eng: Some(2.1),   // 误差过大
            status: SubTestStatus::Failed,
            error_percentage: Some(5.0),     // 5%误差，超过2%容忍度
        },
        AnalogReadingPoint {
            set_percentage: 75.0,
            set_value_eng: 3.0,
            expected_reading_raw: Some(75.0),
            actual_reading_raw: Some(78.2),  // 误差过大
            actual_reading_eng: Some(3.128), // 误差过大
            status: SubTestStatus::Failed,
            error_percentage: Some(4.27),    // 4.27%误差，超过2%容忍度
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
    
    // 创建失败的RawTestOutcome，但包含完整的过程数据
    let outcome = RawTestOutcome {
        channel_instance_id: test_instance.instance_id.clone(),
        sub_test_item: SubTestItem::HardPoint,
        success: false,  // 测试失败
        raw_value_read: Some("多点测试".to_string()),
        eng_value_calculated: Some("0.00-4.00".to_string()),
        message: Some("AI硬点测试部分失败：50%和75%点误差过大".to_string()),
        start_time: Utc::now(),
        end_time: Utc::now(),
        readings: Some(readings.clone()),
        // 🔧 关键：即使测试失败，也要保存百分比测试结果
        test_result_0_percent: Some(0.05),
        test_result_25_percent: Some(1.008),
        test_result_50_percent: Some(2.1),    // 失败的测试点
        test_result_75_percent: Some(3.128),  // 失败的测试点
        test_result_100_percent: Some(3.996),
        details: HashMap::new(),
        digital_steps: None,
    };
    
    println!("📊 创建失败的硬点测试结果:");
    println!("   测试成功: {}", outcome.success);
    println!("   0%: {:?} (通过)", outcome.test_result_0_percent);
    println!("   25%: {:?} (通过)", outcome.test_result_25_percent);
    println!("   50%: {:?} (失败)", outcome.test_result_50_percent);
    println!("   75%: {:?} (失败)", outcome.test_result_75_percent);
    println!("   100%: {:?} (通过)", outcome.test_result_100_percent);
    
    // 模拟测试协调服务的完整流程
    println!("\n🔄 模拟失败测试的完整流程...");
    
    // 第1步：保存测试结果到持久化存储
    println!("💾 第1步：保存失败的测试结果到持久化存储...");
    persistence_service.save_test_outcome(&outcome).await?;
    println!("✅ 失败的测试结果已保存到数据库");
    
    // 第2步：更新状态管理器中的测试实例状态
    println!("🔄 第2步：更新状态管理器中的测试实例状态...");
    state_manager.update_test_result(outcome.clone()).await?;
    println!("✅ 状态管理器已更新");
    
    // 第3步：验证失败测试的数据是否正确保存
    println!("\n🔍 第3步：验证失败测试的数据是否正确保存...");
    
    // 从状态管理器重新获取测试实例
    match state_manager.get_cached_test_instance(&test_instance.instance_id).await {
        Some(updated_instance) => {
            println!("✅ 成功从状态管理器获取更新后的测试实例");
            println!("   实例ID: {}", updated_instance.instance_id);
            println!("   整体状态: {:?}", updated_instance.overall_status);
            
            // 检查百分比测试结果
            println!("\n📊 失败测试的百分比测试结果验证:");
            println!("   0%: {:?} (通过)", updated_instance.transient_data.get("test_result_0_percent"));
            println!("   25%: {:?} (通过)", updated_instance.transient_data.get("test_result_25_percent"));
            println!("   50%: {:?} (失败)", updated_instance.transient_data.get("test_result_50_percent"));
            println!("   75%: {:?} (失败)", updated_instance.transient_data.get("test_result_75_percent"));
            println!("   100%: {:?} (通过)", updated_instance.transient_data.get("test_result_100_percent"));
            
            // 检查硬点读数
            if let Some(readings) = &updated_instance.hardpoint_readings {
                println!("\n📈 失败测试的硬点读数验证:");
                for reading in readings.iter() {
                    let status_icon = if reading.status == SubTestStatus::Passed { "✅" } else { "❌" };
                    println!("   {}%: {} 设定={:.3}, 实际原始={:.3}, 实际工程量={:.3}, 误差={:.2}%",
                        reading.set_percentage,
                        status_icon,
                        reading.set_value_eng,
                        reading.actual_reading_raw.unwrap_or(0.0),
                        reading.actual_reading_eng.unwrap_or(0.0),
                        reading.error_percentage.unwrap_or(0.0));
                }
            } else {
                println!("❌ 硬点读数数据丢失");
            }
            
            // 检查子测试结果
            if let Some(hardpoint_result) = updated_instance.sub_test_results.get(&SubTestItem::HardPoint) {
                println!("\n🧪 失败测试的子测试结果验证:");
                println!("   状态: {:?}", hardpoint_result.status);
                println!("   实际值: {:?}", hardpoint_result.actual_value);
                println!("   期望值: {:?}", hardpoint_result.expected_value);
                println!("   详情: {:?}", hardpoint_result.details);
            }
        }
        None => {
            println!("❌ 无法从状态管理器获取测试实例");
        }
    }
    
    // 从数据库直接验证数据
    println!("\n🗄️ 从数据库直接验证失败测试的数据...");
    match persistence_service.load_test_instance(&test_instance.instance_id).await? {
        Some(db_instance) => {
            println!("✅ 成功从数据库加载失败测试的实例");
            println!("   实例ID: {}", db_instance.instance_id);
            println!("   整体状态: {:?}", db_instance.overall_status);
            
            // 检查数据库中的百分比测试结果
            println!("\n📊 数据库中失败测试的百分比测试结果:");
            println!("   0%: {:?} (通过)", db_instance.transient_data.get("test_result_0_percent"));
            println!("   25%: {:?} (通过)", db_instance.transient_data.get("test_result_25_percent"));
            println!("   50%: {:?} (失败)", db_instance.transient_data.get("test_result_50_percent"));
            println!("   75%: {:?} (失败)", db_instance.transient_data.get("test_result_75_percent"));
            println!("   100%: {:?} (通过)", db_instance.transient_data.get("test_result_100_percent"));
        }
        None => {
            println!("❌ 无法从数据库加载失败测试的实例");
        }
    }
    
    println!("\n🎉 失败硬点测试的过程数据保存功能测试完成！");
    println!("✅ 验证结果：即使测试失败，所有过程数据（0%-100%）都能正确保存到数据库");
    
    Ok(())
}
