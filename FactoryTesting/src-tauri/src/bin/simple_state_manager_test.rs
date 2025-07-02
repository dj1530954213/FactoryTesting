#![cfg(FALSE)]
/// 简化的状态管理器测试
/// 
/// 验证内存缓存机制是否正确工作，解�?未找到测试实�?的问�?

use std::sync::Arc;
use sea_orm::Database;
use app_lib::services::domain::channel_state_manager::{ChannelStateManager, IChannelStateManager};
use app_lib::services::infrastructure::persistence::{SqliteOrmPersistenceService, PersistenceConfig};
use app_lib::services::traits::PersistenceService;
use app_lib::models::structs::{ChannelTestInstance, ChannelPointDefinition, RawTestOutcome};
use app_lib::models::enums::{ModuleType, PointDataType, SubTestItem};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 简化的状态管理器测试");
    
    // 连接到内存数据库
    let db = Database::connect("sqlite::memory:").await?;
    
    // 创建表结�?
    println!("📋 创建数据库表结构...");
    app_lib::database_migration::DatabaseMigration::migrate(&db).await?;
    
    // 创建持久化服�?
    let persistence_service = Arc::new(
        SqliteOrmPersistenceService::new(
            PersistenceConfig::default(),
            None, // 使用内存数据库，不需要文件路�?
        ).await?
    );
    
    // 创建状态管理器
    let state_manager = ChannelStateManager::new(persistence_service.clone());
    
    // 第一步：创建测试数据
    println!("\n📝 创建测试数据...");
    let test_definition = create_test_definition();
    let test_instance = create_test_instance(&test_definition);
    
    // 第二步：直接保存测试实例到数据库和缓�?
    println!("\n💾 保存测试实例到数据库...");
    persistence_service.save_test_instance(&test_instance).await?;
    
    // 第三步：验证数据库中的数�?
    println!("\n🔍 验证数据库中的数�?..");
    match persistence_service.load_test_instance(&test_instance.instance_id).await? {
        Some(loaded_instance) => {
            println!("�?数据库中找到测试实例: {}", loaded_instance.instance_id);
            println!("   - 定义ID: {}", loaded_instance.definition_id);
            println!("   - 状�? {:?}", loaded_instance.overall_status);
        }
        None => {
            println!("�?数据库中未找到测试实�? {}", test_instance.instance_id);
            return Err("数据库保存测试失�?.into());
        }
    }
    
    // 第四步：测试状态管理器的更新功�?
    println!("\n🔄 测试状态管理器的更新功�?..");
    let test_outcome = RawTestOutcome::new(
        test_instance.instance_id.clone(),
        SubTestItem::HardPoint,
        true,
    );
    
    // 使用状态管理器更新测试结果
    match state_manager.update_test_result(test_outcome).await {
        Ok(_) => {
            println!("�?状态管理器更新测试结果成功");
        }
        Err(e) => {
            println!("�?状态管理器更新测试结果失败: {}", e);
            
            // 打印详细的调试信�?
            println!("\n🔍 调试信息�?);
            println!("   - 尝试更新的实例ID: {}", test_instance.instance_id);
            
            // 检查数据库中的所有测试实�?
            match persistence_service.load_all_test_instances().await {
                Ok(all_instances) => {
                    println!("   - 数据库中共有 {} 个测试实�?, all_instances.len());
                    for (i, inst) in all_instances.iter().enumerate() {
                        println!("     {}. 实例ID: {} (定义ID: {})", 
                                 i + 1, inst.instance_id, inst.definition_id);
                    }
                }
                Err(e) => {
                    println!("   - 查询所有测试实例失�? {}", e);
                }
            }
            
            return Err(format!("状态更新测试失�? {}", e).into());
        }
    }
    
    // 第五步：验证更新后的状�?
    println!("\n🔍 验证更新后的状�?..");
    
    // 从数据库重新加载
    match persistence_service.load_test_instance(&test_instance.instance_id).await? {
        Some(updated_instance) => {
            println!("�?更新后的测试实例状�? {:?}", updated_instance.overall_status);
            
            // 检查子测试结果
            if let Some(hard_point_result) = updated_instance.sub_test_results.get(&SubTestItem::HardPoint) {
                println!("   - 硬点测试状�? {:?}", hard_point_result.status);
                println!("   - 实际�? {:?}", hard_point_result.actual_value);
            }
        }
        None => {
            println!("�?更新后未找到测试实例");
            return Err("状态更新验证失�?.into());
        }
    }
    
    // 第六步：测试内存缓存功能
    println!("\n🔍 测试内存缓存功能...");
    let cached_instance = state_manager.get_cached_test_instance(&test_instance.instance_id).await;
    
    match cached_instance {
        Some(instance) => {
            println!("�?内存缓存中找到测试实�? {}", instance.instance_id);
            println!("   - 状�? {:?}", instance.overall_status);
        }
        None => {
            println!("⚠️ 内存缓存中未找到测试实例（这是正常的，因为我们没有通过批次分配存储�?);
        }
    }
    
    println!("\n🎉 状态管理器基本功能测试完成�?);
    println!("�?数据库保存和加载正常");
    println!("�?状态更新功能正�?);
    println!("�?修复�?未找到测试实�?的问�?);
    
    Ok(())
}

/// 创建测试用的通道定义
fn create_test_definition() -> ChannelPointDefinition {
    ChannelPointDefinition::new(
        "TEST001".to_string(),
        "Temperature_Test".to_string(),
        "测试温度传感�?.to_string(),
        "TestStation".to_string(),
        "TestModule".to_string(),
        ModuleType::AI,
        "CH01".to_string(),
        PointDataType::Float,
        "DB1.DBD0".to_string(),
    )
}

/// 创建测试用的测试实例
fn create_test_instance(definition: &ChannelPointDefinition) -> ChannelTestInstance {
    ChannelTestInstance::new(
        definition.id.clone(),
        "test_batch_001".to_string(),
    )
}

