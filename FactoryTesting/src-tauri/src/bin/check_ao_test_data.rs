use app_lib::services::infrastructure::persistence::{SqliteOrmPersistenceService, PersistenceConfig};
use app_lib::services::traits::PersistenceService;
use app_lib::models::ModuleType;
use std::path::PathBuf;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    
    println!("=== 检查数据库中AO点位的测试数据 ===");
    
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
    
    // 获取所有通道定义
    println!("\n📋 检查通道定义中的AO点位:");
    let all_definitions = persistence_service.load_all_channel_definitions().await?;
    let ao_definitions: Vec<_> = all_definitions.iter()
        .filter(|def| matches!(def.module_type, ModuleType::AO | ModuleType::AONone))
        .collect();
    
    println!("   找到 {} 个AO点位定义", ao_definitions.len());
    for (i, def) in ao_definitions.iter().enumerate() {
        println!("   {}. ID: {}, Tag: {}, 模块类型: {:?}", 
            i + 1, def.id, def.tag, def.module_type);
    }
    
    // 获取所有测试实例
    println!("\n📊 检查测试实例中的AO点位:");
    let all_instances = persistence_service.load_all_test_instances().await?;
    let ao_instances: Vec<_> = all_instances.iter()
        .filter(|instance| {
            // 通过definition_id查找对应的定义
            ao_definitions.iter().any(|def| def.id == instance.definition_id)
        })
        .collect();
    
    println!("   找到 {} 个AO点位测试实例", ao_instances.len());
    for (i, instance) in ao_instances.iter().enumerate() {
        println!("   {}. 实例ID: {}", i + 1, instance.instance_id);
        println!("      定义ID: {}", instance.definition_id);
        println!("      整体状态: {:?}", instance.overall_status);
        
        // 检查百分比测试结果
        let has_percentage_data = instance.transient_data.get("test_result_0_percent").is_some() ||
                                 instance.transient_data.get("test_result_25_percent").is_some() ||
                                 instance.transient_data.get("test_result_50_percent").is_some() ||
                                 instance.transient_data.get("test_result_75_percent").is_some() ||
                                 instance.transient_data.get("test_result_100_percent").is_some();
        
        if has_percentage_data {
            println!("      ✅ 有百分比测试结果:");
            println!("         0%: {:?}", instance.transient_data.get("test_result_0_percent"));
            println!("         25%: {:?}", instance.transient_data.get("test_result_25_percent"));
            println!("         50%: {:?}", instance.transient_data.get("test_result_50_percent"));
            println!("         75%: {:?}", instance.transient_data.get("test_result_75_percent"));
            println!("         100%: {:?}", instance.transient_data.get("test_result_100_percent"));
        } else {
            println!("      ❌ 没有百分比测试结果");
        }
        
        // 检查硬点读数
        if let Some(readings) = &instance.hardpoint_readings {
            println!("      ✅ 有硬点读数数据 ({} 个读数)", readings.len());
            for reading in readings.iter() {
                println!("         {}%: 设定={:.3}, 实际工程量={:.3}", 
                    reading.set_percentage, 
                    reading.set_value_eng,
                    reading.actual_reading_eng.unwrap_or(0.0));
            }
        } else {
            println!("      ❌ 没有硬点读数数据");
        }
        
        println!();
    }
    
    // 检查原始测试结果
    println!("\n📈 检查原始测试结果中的AO点位:");
    let mut all_outcomes = Vec::new();

    // 为每个AO实例获取测试结果
    for instance in &ao_instances {
        match persistence_service.load_test_outcomes_by_instance(&instance.instance_id).await {
            Ok(outcomes) => {
                all_outcomes.extend(outcomes);
            }
            Err(e) => {
                println!("   ⚠️ 获取实例 {} 的测试结果失败: {}", instance.instance_id, e);
            }
        }
    }

    let ao_outcomes: Vec<_> = all_outcomes.iter().collect();
    
    println!("   找到 {} 个AO点位原始测试结果", ao_outcomes.len());
    for (i, outcome) in ao_outcomes.iter().enumerate() {
        println!("   {}. 通道实例ID: {}", i + 1, outcome.channel_instance_id);
        println!("      通道实例ID: {}", outcome.channel_instance_id);
        println!("      测试项目: {:?}", outcome.sub_test_item);
        println!("      成功: {}", outcome.success);
        
        // 检查百分比测试结果
        let has_percentage_data = outcome.test_result_0_percent.is_some() ||
                                 outcome.test_result_25_percent.is_some() ||
                                 outcome.test_result_50_percent.is_some() ||
                                 outcome.test_result_75_percent.is_some() ||
                                 outcome.test_result_100_percent.is_some();
        
        if has_percentage_data {
            println!("      ✅ 有百分比测试结果:");
            println!("         0%: {:?}", outcome.test_result_0_percent);
            println!("         25%: {:?}", outcome.test_result_25_percent);
            println!("         50%: {:?}", outcome.test_result_50_percent);
            println!("         75%: {:?}", outcome.test_result_75_percent);
            println!("         100%: {:?}", outcome.test_result_100_percent);
        } else {
            println!("      ❌ 没有百分比测试结果");
        }
        
        // 检查readings数据
        if let Some(readings) = &outcome.readings {
            println!("      ✅ 有readings数据 ({} 个读数)", readings.len());
            for reading in readings.iter() {
                println!("         {}%: 设定={:.3}, 实际工程量={:.3}", 
                    reading.set_percentage, 
                    reading.set_value_eng,
                    reading.actual_reading_eng.unwrap_or(0.0));
            }
        } else {
            println!("      ❌ 没有readings数据");
        }
        
        println!();
    }
    
    // 总结
    println!("=== 总结 ===");
    println!("AO点位定义数量: {}", ao_definitions.len());
    println!("AO点位测试实例数量: {}", ao_instances.len());
    println!("AO点位原始测试结果数量: {}", ao_outcomes.len());
    
    let instances_with_data = ao_instances.iter().filter(|instance| {
        instance.transient_data.get("test_result_0_percent").is_some()
    }).count();
    
    let outcomes_with_data = ao_outcomes.iter().filter(|outcome| {
        outcome.test_result_0_percent.is_some()
    }).count();
    
    println!("有百分比测试结果的AO实例数量: {}", instances_with_data);
    println!("有百分比测试结果的AO原始结果数量: {}", outcomes_with_data);
    
    if instances_with_data == 0 && outcomes_with_data == 0 {
        println!("❌ 发现问题：AO点位没有百分比测试结果数据！");
    } else {
        println!("✅ AO点位有百分比测试结果数据");
    }
    
    Ok(())
}
