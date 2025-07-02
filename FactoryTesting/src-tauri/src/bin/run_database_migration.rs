#![cfg(FALSE)]
/// 运行数据库迁移工�?
/// 更新数据库表结构，添加缺失的字段

use sea_orm::Database;
use app_lib::database_migration::DatabaseMigration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日�?
    env_logger::init();
    
    println!("🔧 开始运行数据库迁移...");
    
    // 连接到数据库
    let db = Database::connect("sqlite://./factory_testing_data.sqlite").await?;
    
    println!("�?已连接到数据�?);
    
    // 运行迁移
    DatabaseMigration::migrate(&db).await?;
    
    println!("🎉 数据库迁移完成！");
    
    Ok(())
}

