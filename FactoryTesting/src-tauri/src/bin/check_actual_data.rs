#![cfg(FALSE)]
use app_lib::services::infrastructure::persistence::{SqliteOrmPersistenceService, PersistenceConfig};
use app_lib::services::traits::PersistenceService;
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== 检查数据库实际数据内容 ===");
    
    // 使用与主应用相同的配�?
    let config = PersistenceConfig::default();
    let db_file_path = config.storage_root_dir.join("factory_testing_data.sqlite");
    
    println!("📁 数据库文件路�? {:?}", db_file_path);
    
    if !db_file_path.exists() {
        println!("�?数据库文件不存在�?);
        return Ok(());
    }
    
    // 创建持久化服�?
    let persistence_service = SqliteOrmPersistenceService::new(config, Some(&db_file_path)).await?;
    
    // 检查通道定义数据
    println!("\n📊 检查通道定义数据...");
    match persistence_service.load_all_channel_definitions().await {
        Ok(definitions) => {
            println!("�?成功加载通道定义，共 {} 条记�?, definitions.len());
            
            if definitions.is_empty() {
                println!("⚠️  数据库中没有通道定义数据�?);
            } else {
                println!("📋 �?条记�?");
                for (i, def) in definitions.iter().take(5).enumerate() {
                    println!("  {}. ID: {}, 位号: {}, 变量�? {}, 模块类型: {}", 
                        i + 1, def.id, def.tag, def.variable_name, def.module_type);
                }
                
                // 统计模块类型
                let mut ai_count = 0;
                let mut ao_count = 0;
                let mut di_count = 0;
                let mut do_count = 0;
                
                for def in &definitions {
                    match def.module_type.to_string().as_str() {
                        "AI" => ai_count += 1,
                        "AO" => ao_count += 1,
                        "DI" => di_count += 1,
                        "DO" => do_count += 1,
                        _ => {}
                    }
                }
                
                println!("📈 模块类型统计:");
                println!("  AI: {} �?, ai_count);
                println!("  AO: {} �?, ao_count);
                println!("  DI: {} �?, di_count);
                println!("  DO: {} �?, do_count);
            }
        },
        Err(e) => {
            println!("�?加载通道定义失败: {}", e);
        }
    }
    
    // 检查批次信息数�?
    println!("\n📊 检查批次信息数�?..");
    match persistence_service.load_all_batch_info().await {
        Ok(batches) => {
            println!("�?成功加载批次信息，共 {} 条记�?, batches.len());
            
            if batches.is_empty() {
                println!("⚠️  数据库中没有批次信息数据�?);
            } else {
                println!("📋 批次列表:");
                for batch in &batches {
                    println!("  批次ID: {}, 名称: {}, 总点�? {}", 
                        batch.batch_id, batch.batch_name, batch.total_points);
                }
            }
        },
        Err(e) => {
            println!("�?加载批次信息失败: {}", e);
        }
    }
    
    // 检查测试实例数据（通过批次加载�?
    println!("\n📊 检查测试实例数�?..");

    // 首先获取所有批次，然后加载每个批次的测试实�?
    match persistence_service.load_all_batch_info().await {
        Ok(batches) => {
            if batches.is_empty() {
                println!("⚠️  没有批次数据，无法检查测试实例！");
            } else {
                let mut total_instances = 0;
                for batch in batches.iter().take(3) { // 只检查前3个批�?
                    match persistence_service.load_test_instances_by_batch(&batch.batch_id).await {
                        Ok(instances) => {
                            total_instances += instances.len();
                            if !instances.is_empty() {
                                println!("📋 批次 {} 的测试实�?({} �?:", batch.batch_id, instances.len());
                                for (i, instance) in instances.iter().take(3).enumerate() {
                                    println!("  {}. 实例ID: {}, 定义ID: {}, 状�? {:?}",
                                        i + 1, instance.instance_id, instance.definition_id, instance.overall_status);
                                }
                            }
                        },
                        Err(e) => {
                            println!("�?加载批次 {} 的测试实例失�? {}", batch.batch_id, e);
                        }
                    }
                }
                println!("�?总共检查了 {} 个测试实�?, total_instances);
            }
        },
        Err(e) => {
            println!("�?加载批次信息失败，无法检查测试实�? {}", e);
        }
    }
    
    println!("\n🎉 数据检查完成！");
    
    Ok(())
}

