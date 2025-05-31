// 检查数据库表结构
use app_lib::services::infrastructure::persistence::{SqliteOrmPersistenceService, PersistenceConfig};
use sea_orm::{DatabaseConnection, Statement, ConnectionTrait};
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    
    println!("=== 检查数据库表结构 ===");
    
    // 初始化数据库连接
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
    
    let persistence_service = SqliteOrmPersistenceService::new(persistence_config, Some(&db_path)).await?;
    let db_conn = persistence_service.get_database_connection();
    
    // 检查 test_batch_info 表结构
    println!("\n🔍 检查 test_batch_info 表结构:");
    check_table_schema(db_conn, "test_batch_info").await?;
    
    // 检查 channel_test_instances 表结构
    println!("\n🔍 检查 channel_test_instances 表结构:");
    check_table_schema(db_conn, "channel_test_instances").await?;
    
    // 检查 channel_point_definitions 表结构
    println!("\n🔍 检查 channel_point_definitions 表结构:");
    check_table_schema(db_conn, "channel_point_definitions").await?;
    
    println!("\n=== 检查完成 ===");
    
    Ok(())
}

async fn check_table_schema(db: &DatabaseConnection, table_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    // 检查表是否存在
    let table_exists_sql = "SELECT name FROM sqlite_master WHERE type='table' AND name=?";
    let result = db.query_all(Statement::from_sql_and_values(
        sea_orm::DatabaseBackend::Sqlite,
        table_exists_sql,
        vec![table_name.into()]
    )).await?;
    
    if result.is_empty() {
        println!("❌ 表 {} 不存在", table_name);
        return Ok(());
    }
    
    println!("✅ 表 {} 存在", table_name);
    
    // 获取表结构
    let schema_sql = format!("PRAGMA table_info({})", table_name);
    let schema_result = db.query_all(Statement::from_string(
        sea_orm::DatabaseBackend::Sqlite,
        schema_sql
    )).await?;
    
    println!("   列信息:");
    for row in schema_result {
        let column_name: String = row.try_get("", "name")?;
        let column_type: String = row.try_get("", "type")?;
        let not_null: i32 = row.try_get("", "notnull")?;
        let default_value: Option<String> = row.try_get("", "dflt_value").ok();
        let is_pk: i32 = row.try_get("", "pk")?;
        
        println!("     - {} {} {} {} {}", 
                 column_name,
                 column_type,
                 if not_null == 1 { "NOT NULL" } else { "NULL" },
                 if let Some(def) = default_value { format!("DEFAULT {}", def) } else { "".to_string() },
                 if is_pk == 1 { "PRIMARY KEY" } else { "" });
    }
    
    Ok(())
}
