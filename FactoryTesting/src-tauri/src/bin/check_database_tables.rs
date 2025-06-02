use sea_orm::{Database, DatabaseConnection, EntityTrait};
use std::path::PathBuf;
use app_lib::models::entities::{channel_point_definition, test_batch_info, channel_test_instance};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 检查数据库中的所有表数据");
    
    let db_path = PathBuf::from("./factory_testing_data.sqlite");
    let db_url = format!("sqlite://{}?mode=rwc", db_path.display());
    println!("📁 数据库路径: {}", db_url);
    
    let db = Database::connect(&db_url).await?;
    
    // 检查通道定义表
    println!("\n🔍 检查通道定义表 (channel_point_definitions)...");
    let definitions = channel_point_definition::Entity::find().all(&db).await?;
    println!("📊 通道定义表中共有 {} 条记录", definitions.len());
    
    if !definitions.is_empty() {
        println!("✅ 通道定义表有数据");
        for (i, def) in definitions.iter().take(3).enumerate() {
            println!("  定义 {}: 位号={}, 通讯地址={}", i + 1, def.tag, def.plc_communication_address);
        }
    } else {
        println!("❌ 通道定义表为空");
    }
    
    // 检查批次信息表
    println!("\n🔍 检查批次信息表 (test_batch_info)...");
    let batches = test_batch_info::Entity::find().all(&db).await?;
    println!("📊 批次信息表中共有 {} 条记录", batches.len());
    
    if !batches.is_empty() {
        println!("✅ 批次信息表有数据");
        for (i, batch) in batches.iter().take(3).enumerate() {
            println!("  批次 {}: ID={}, 名称={}, 总点位={}", 
                i + 1, batch.batch_id, batch.batch_name, batch.total_points);
        }
    } else {
        println!("❌ 批次信息表为空");
    }
    
    // 检查测试实例表
    println!("\n🔍 检查测试实例表 (channel_test_instances)...");
    let instances = channel_test_instance::Entity::find().all(&db).await?;
    println!("📊 测试实例表中共有 {} 条记录", instances.len());
    
    if !instances.is_empty() {
        println!("✅ 测试实例表有数据");
        for (i, instance) in instances.iter().take(3).enumerate() {
            println!("  实例 {}: ID={}, 批次ID={}, 定义ID={}",
                i + 1, instance.instance_id, instance.test_batch_id, instance.definition_id);
        }
    } else {
        println!("❌ 测试实例表为空");
    }
    
    // 总结
    println!("\n📊 数据库状态总结:");
    println!("  通道定义: {} 条", definitions.len());
    println!("  批次信息: {} 条", batches.len());
    println!("  测试实例: {} 条", instances.len());
    
    if definitions.is_empty() && batches.len() > 0 {
        println!("\n⚠️ 发现问题：有批次信息但没有通道定义！");
        println!("   这说明Excel导入过程中通道定义没有正确保存到数据库");
    }
    
    println!("✅ 数据库数据验证完成！");
    
    Ok(())
}
