/// 测试重复数据插入防护机制
/// 
/// 这个测试验证 BatchAllocationService 是否能正确防止重复插入测试实例

use std::sync::Arc;
use sea_orm::{Database, EntityTrait, PaginatorTrait};
use app_lib::services::application::batch_allocation_service::{BatchAllocationService, AllocationStrategy};
use app_lib::models::entities::{channel_point_definition, channel_test_instance};
use app_lib::models::structs::ChannelPointDefinition;
use app_lib::models::enums::{ModuleType, PointDataType};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 测试重复数据插入防护机制");
    
    // 连接到内存数据库
    let db = Database::connect("sqlite::memory:").await?;
    
    // 创建表结构
    println!("📋 创建数据库表结构...");
    app_lib::database_migration::DatabaseMigration::migrate(&db).await?;
    
    // 创建测试用的通道定义
    println!("📝 创建测试通道定义...");
    let test_definitions = create_test_definitions();
    
    // 保存通道定义到数据库
    for definition in &test_definitions {
        let active_model: channel_point_definition::ActiveModel = definition.into();
        channel_point_definition::Entity::insert(active_model).exec(&db).await?;
    }
    
    println!("✅ 已保存 {} 个通道定义到数据库", test_definitions.len());
    
    // 创建批次分配服务
    let allocation_service = BatchAllocationService::new(Arc::new(db.clone()));
    
    // 第一次分配 - 应该创建新的测试实例
    println!("\n🔄 第一次批次分配...");
    let result1 = allocation_service.create_test_batch(
        "测试批次1".to_string(),
        Some("TEST_MODEL".to_string()),
        Some("操作员1".to_string()),
        AllocationStrategy::Smart,
        None,
    ).await?;
    
    println!("✅ 第一次分配完成: 批次ID={}, 实例数量={}", 
             result1.batch_info.batch_id, result1.test_instances.len());
    
    // 检查数据库中的测试实例数量
    let instances_count_1 = channel_test_instance::Entity::find().count(&db).await?;
    println!("📊 数据库中测试实例数量: {}", instances_count_1);
    
    // 第二次分配 - 应该检测到重复并跳过创建
    println!("\n🔄 第二次批次分配（相同数据）...");
    let result2 = allocation_service.create_test_batch(
        "测试批次2".to_string(),
        Some("TEST_MODEL".to_string()),
        Some("操作员2".to_string()),
        AllocationStrategy::Smart,
        None,
    ).await?;
    
    println!("✅ 第二次分配完成: 批次ID={}, 实例数量={}", 
             result2.batch_info.batch_id, result2.test_instances.len());
    
    // 检查数据库中的测试实例数量
    let instances_count_2 = channel_test_instance::Entity::find().count(&db).await?;
    println!("📊 数据库中测试实例数量: {}", instances_count_2);
    
    // 验证结果
    println!("\n🔍 验证结果:");
    if instances_count_2 > instances_count_1 {
        println!("❌ 测试失败: 检测到重复数据插入!");
        println!("   第一次分配后: {} 个实例", instances_count_1);
        println!("   第二次分配后: {} 个实例", instances_count_2);
        println!("   增加了: {} 个实例", instances_count_2 - instances_count_1);
    } else {
        println!("✅ 测试成功: 重复数据插入防护机制工作正常!");
        println!("   两次分配后数据库中都有 {} 个实例", instances_count_2);
    }
    
    // 显示详细的实例信息
    let all_instances = channel_test_instance::Entity::find().all(&db).await?;
    println!("\n📋 数据库中的所有测试实例:");
    for (i, instance) in all_instances.iter().enumerate() {
        println!("  {}. 实例ID: {}, 批次ID: {}, 定义ID: {}", 
                 i + 1, instance.instance_id, instance.test_batch_id, instance.definition_id);
    }
    
    Ok(())
}

/// 创建测试用的通道定义
fn create_test_definitions() -> Vec<ChannelPointDefinition> {
    vec![
        ChannelPointDefinition::new(
            "AI001".to_string(),
            "Temperature_1".to_string(),
            "温度传感器1".to_string(),
            "Station1".to_string(),
            "Module1".to_string(),
            ModuleType::AI,
            "CH01".to_string(),
            PointDataType::Float,
            "DB1.DBD0".to_string(),
        ),
        ChannelPointDefinition::new(
            "AI002".to_string(),
            "Pressure_1".to_string(),
            "压力传感器1".to_string(),
            "Station1".to_string(),
            "Module1".to_string(),
            ModuleType::AI,
            "CH02".to_string(),
            PointDataType::Float,
            "DB1.DBD4".to_string(),
        ),
        ChannelPointDefinition::new(
            "DI001".to_string(),
            "Switch_1".to_string(),
            "开关1".to_string(),
            "Station1".to_string(),
            "Module2".to_string(),
            ModuleType::DI,
            "CH01".to_string(),
            PointDataType::Bool,
            "DB1.DBX0.0".to_string(),
        ),
    ]
}
