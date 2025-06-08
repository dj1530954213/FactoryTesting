// 检查数据库中DI/DO点位的digital_test_steps_json数据
use sea_orm::{Database, EntityTrait, ColumnTrait, QueryFilter, QuerySelect};
use app_lib::models::entities::channel_test_instance;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 连接数据库
    let db = Database::connect("sqlite://factory_testing_data.sqlite").await?;
    
    println!("🔍 检查数据库中的 digital_test_steps_json 数据...");
    
    // 首先检查表结构
    let table_info = db.execute_unprepared("PRAGMA table_info(channel_test_instances);").await?;
    println!("📋 channel_test_instances 表结构:");

    // 查询所有记录，看看实际的字段
    let instances = channel_test_instance::Entity::find()
        .limit(5)
        .all(&db)
        .await?;
    
    println!("📊 找到 {} 条有 digital_test_steps_json 数据的记录", instances.len());
    
    for (i, instance) in instances.iter().enumerate() {
        println!("\n--- 记录 {} ---", i + 1);
        println!("实例ID: {}", instance.instance_id);
        println!("状态: {:?}", instance.overall_status);
        println!("digital_test_steps_json 原始值: {:?}", instance.digital_test_steps_json);
        
        if let Some(ref json_str) = instance.digital_test_steps_json {
            println!("JSON 字符串长度: {}", json_str.len());
            println!("JSON 内容前100字符: {}", 
                if json_str.len() > 100 { &json_str[..100] } else { json_str });
            
            // 尝试解析 JSON
            match serde_json::from_str::<serde_json::Value>(json_str) {
                Ok(value) => {
                    println!("✅ JSON 解析成功");
                    if value.is_null() {
                        println!("⚠️  JSON 值是 null");
                    } else if value.is_array() {
                        println!("📋 JSON 是数组，长度: {}", value.as_array().unwrap().len());
                    } else {
                        println!("📄 JSON 类型: {}", value);
                    }
                }
                Err(e) => {
                    println!("❌ JSON 解析失败: {}", e);
                }
            }
        }
    }
    
    // 检查有实际测试数据的记录
    println!("\n🔍 检查有实际测试数据的记录...");
    let tested_instances = channel_test_instance::Entity::find()
        .filter(channel_test_instance::Column::DigitalTestStepsJson.ne("null"))
        .filter(channel_test_instance::Column::DigitalTestStepsJson.ne(""))
        .limit(5)
        .all(&db)
        .await?;
    
    println!("📊 找到 {} 条有实际测试数据的记录", tested_instances.len());
    
    for (i, instance) in tested_instances.iter().enumerate() {
        println!("\n--- 测试数据记录 {} ---", i + 1);
        println!("实例ID: {}", instance.instance_id);
        println!("状态: {:?}", instance.overall_status);
        if let Some(ref json_str) = instance.digital_test_steps_json {
            println!("JSON 内容: {}", json_str);
        }
    }
    
    Ok(())
}
