#![cfg(FALSE)]
use sea_orm::{Database, DatabaseConnection, Statement, ConnectionTrait};
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== 检查数据库内容 ===");
    
    // 检查主应用使用的数据库文件
    let db_path = PathBuf::from("data/factory_testing_data.sqlite");
    println!("📁 检查数据库文件: {:?}", db_path);
    
    if !db_path.exists() {
        println!("�?数据库文件不存在�?);
        return Ok(());
    }
    
    let db_url = format!("sqlite://{}?mode=rwc", db_path.display());
    let db = Database::connect(&db_url).await?;
    
    // 检查表是否存在
    println!("\n🔍 检查表结构...");
    let tables = vec![
        "channel_point_definitions",
        "test_batch_info", 
        "channel_test_instances",
        "test_plc_channels",
        "plc_connections"
    ];
    
    for table in &tables {
        let result = db.execute(Statement::from_string(
            sea_orm::DatabaseBackend::Sqlite,
            format!("SELECT name FROM sqlite_master WHERE type='table' AND name='{}'", table),
        )).await;
        
        match result {
            Ok(_) => {
                // 检查表中的记录�?
                let count_result = db.execute(Statement::from_string(
                    sea_orm::DatabaseBackend::Sqlite,
                    format!("SELECT COUNT(*) as count FROM {}", table),
                )).await;
                
                match count_result {
                    Ok(_) => println!("�?�?{} 存在", table),
                    Err(e) => println!("�?�?{} 不存在或查询失败: {}", table, e),
                }
            },
            Err(e) => println!("�?�?{} 检查失�? {}", table, e),
        }
    }
    
    // 检查channel_point_definitions表的内容
    println!("\n📊 检查channel_point_definitions表内�?..");
    let count_sql = "SELECT COUNT(*) as count FROM channel_point_definitions";
    let count_result = db.execute(Statement::from_string(
        sea_orm::DatabaseBackend::Sqlite,
        count_sql.to_string(),
    )).await;
    
    match count_result {
        Ok(_) => {
            // 获取�?条记�?
            let sample_sql = "SELECT id, tag, variable_name, module_type, power_supply_type FROM channel_point_definitions LIMIT 5";
            let sample_result = db.execute(Statement::from_string(
                sea_orm::DatabaseBackend::Sqlite,
                sample_sql.to_string(),
            )).await;
            
            match sample_result {
                Ok(_) => println!("�?成功查询channel_point_definitions�?),
                Err(e) => println!("�?查询channel_point_definitions表失�? {}", e),
            }
        },
        Err(e) => println!("�?统计channel_point_definitions表记录数失败: {}", e),
    }
    
    // 检查test_batch_info表的内容
    println!("\n📊 检查test_batch_info表内�?..");
    let batch_count_sql = "SELECT COUNT(*) as count FROM test_batch_info";
    let batch_count_result = db.execute(Statement::from_string(
        sea_orm::DatabaseBackend::Sqlite,
        batch_count_sql.to_string(),
    )).await;
    
    match batch_count_result {
        Ok(_) => println!("�?成功查询test_batch_info�?),
        Err(e) => println!("�?查询test_batch_info表失�? {}", e),
    }
    
    println!("\n🎉 数据库内容检查完成！");
    
    Ok(())
}

