/// 检查数据库表结构

use sea_orm::{Database, Statement, ConnectionTrait};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 连接到数据库
    let db = Database::connect("sqlite://./factory_testing_data.sqlite").await?;
    
    println!("🔍 检查数据库表结构...");
    
    // 查询所有表
    let tables_result = db.query_all(Statement::from_string(
        sea_orm::DatabaseBackend::Sqlite,
        "SELECT name FROM sqlite_master WHERE type='table' ORDER BY name;".to_string()
    )).await?;
    
    println!("📊 数据库中的表:");
    for row in &tables_result {
        let table_name: String = row.try_get("", "name")?;
        println!("   - {}", table_name);
    }
    
    // 查询channel_test_instances表结构
    println!("\n🔍 channel_test_instances表结构:");
    let schema_result = db.query_all(Statement::from_string(
        sea_orm::DatabaseBackend::Sqlite,
        "PRAGMA table_info(channel_test_instances);".to_string()
    )).await?;
    
    for row in &schema_result {
        let cid: i32 = row.try_get("", "cid")?;
        let name: String = row.try_get("", "name")?;
        let type_name: String = row.try_get("", "type")?;
        let not_null: i32 = row.try_get("", "notnull")?;
        let default_value: Option<String> = row.try_get("", "dflt_value").ok();
        let pk: i32 = row.try_get("", "pk")?;
        
        println!("   {} | {} | {} | NOT NULL: {} | DEFAULT: {:?} | PK: {}", 
                 cid, name, type_name, not_null == 1, default_value, pk == 1);
    }
    
    // 查询raw_test_outcomes表结构
    println!("\n🔍 raw_test_outcomes表结构:");
    let outcome_schema_result = db.query_all(Statement::from_string(
        sea_orm::DatabaseBackend::Sqlite,
        "PRAGMA table_info(raw_test_outcomes);".to_string()
    )).await?;
    
    for row in &outcome_schema_result {
        let cid: i32 = row.try_get("", "cid")?;
        let name: String = row.try_get("", "name")?;
        let type_name: String = row.try_get("", "type")?;
        let not_null: i32 = row.try_get("", "notnull")?;
        let default_value: Option<String> = row.try_get("", "dflt_value").ok();
        let pk: i32 = row.try_get("", "pk")?;
        
        println!("   {} | {} | {} | NOT NULL: {} | DEFAULT: {:?} | PK: {}", 
                 cid, name, type_name, not_null == 1, default_value, pk == 1);
    }
    
    // 查询数据库中的数据
    println!("\n🔍 channel_test_instances表中的数据:");
    let data_result = db.query_all(Statement::from_string(
        sea_orm::DatabaseBackend::Sqlite,
        "SELECT COUNT(*) as count FROM channel_test_instances;".to_string()
    )).await?;
    
    if let Some(row) = data_result.first() {
        let count: i32 = row.try_get("", "count")?;
        println!("   总记录数: {}", count);
    }
    
    // 查询raw_test_outcomes表中的数据
    println!("\n🔍 raw_test_outcomes表中的数据:");
    let outcome_data_result = db.query_all(Statement::from_string(
        sea_orm::DatabaseBackend::Sqlite,
        "SELECT COUNT(*) as count FROM raw_test_outcomes;".to_string()
    )).await?;
    
    if let Some(row) = outcome_data_result.first() {
        let count: i32 = row.try_get("", "count")?;
        println!("   总记录数: {}", count);
    }
    
    // 如果有数据，显示最新的几条记录
    if let Ok(recent_instances) = db.query_all(Statement::from_string(
        sea_orm::DatabaseBackend::Sqlite,
        "SELECT instance_id, definition_id, test_batch_id FROM channel_test_instances ORDER BY created_at DESC LIMIT 5;".to_string()
    )).await {
        println!("\n🔍 最新的5条测试实例记录:");
        for row in &recent_instances {
            let instance_id: String = row.try_get("", "instance_id")?;
            let definition_id: String = row.try_get("", "definition_id")?;
            let test_batch_id: String = row.try_get("", "test_batch_id")?;

            println!("   实例: {} | 定义: {} | 批次: {}",
                     instance_id, definition_id, test_batch_id);
        }
    }
    
    println!("\n🎉 数据库检查完成！");
    
    Ok(())
}
