use sea_orm::{Database, DatabaseConnection, Statement, ConnectionTrait};
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== 检查数据库表结构 ===");
    
    let db_path = PathBuf::from("data/factory_testing_data.sqlite");
    println!("📁 数据库文件: {:?}", db_path);
    
    if !db_path.exists() {
        println!("❌ 数据库文件不存在！");
        return Ok(());
    }
    
    let db_url = format!("sqlite://{}?mode=rwc", db_path.display());
    let db = Database::connect(&db_url).await?;
    
    // 检查channel_point_definitions表结构
    println!("\n🔍 检查channel_point_definitions表结构...");
    let pragma_sql = "PRAGMA table_info(channel_point_definitions)";
    let result = db.execute(Statement::from_string(
        sea_orm::DatabaseBackend::Sqlite,
        pragma_sql.to_string(),
    )).await;
    
    match result {
        Ok(_) => {
            println!("✅ 成功获取表结构信息");
            
            // 获取列信息
            let columns_sql = "SELECT name, type, [notnull], dflt_value FROM pragma_table_info('channel_point_definitions')";
            let columns_result = db.execute(Statement::from_string(
                sea_orm::DatabaseBackend::Sqlite,
                columns_sql.to_string(),
            )).await;
            
            match columns_result {
                Ok(_) => println!("✅ 表结构查询成功"),
                Err(e) => println!("❌ 查询列信息失败: {}", e),
            }
        },
        Err(e) => {
            println!("❌ 获取表结构失败: {}", e);
        }
    }
    
    // 检查是否有description字段
    println!("\n🔍 检查description字段...");
    let desc_check_sql = "SELECT sql FROM sqlite_master WHERE type='table' AND name='channel_point_definitions'";
    let desc_result = db.execute(Statement::from_string(
        sea_orm::DatabaseBackend::Sqlite,
        desc_check_sql.to_string(),
    )).await;
    
    match desc_result {
        Ok(_) => println!("✅ 成功获取表创建SQL"),
        Err(e) => println!("❌ 获取表创建SQL失败: {}", e),
    }
    
    // 尝试查询一条记录看字段
    println!("\n🔍 查询示例记录...");
    let sample_sql = "SELECT * FROM channel_point_definitions LIMIT 1";
    let sample_result = db.execute(Statement::from_string(
        sea_orm::DatabaseBackend::Sqlite,
        sample_sql.to_string(),
    )).await;
    
    match sample_result {
        Ok(_) => println!("✅ 成功查询示例记录"),
        Err(e) => println!("❌ 查询示例记录失败: {}", e),
    }
    
    println!("\n🎉 表结构检查完成！");
    
    Ok(())
}
