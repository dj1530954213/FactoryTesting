#![cfg(FALSE)]
// 检查数据库表结�?
use app_lib::services::infrastructure::persistence::{SqliteOrmPersistenceService, PersistenceConfig};
use sea_orm::{DatabaseConnection, Statement, ConnectionTrait};
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    
    println!("=== 检查数据库表结�?===");
    
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
    
    // 检�?test_batch_info 表结�?
    println!("\n🔍 检�?test_batch_info 表结�?");
    check_table_schema(db_conn, "test_batch_info").await?;
    
    // 检�?channel_test_instances 表结�?
    println!("\n🔍 检�?channel_test_instances 表结�?");
    check_table_schema(db_conn, "channel_test_instances").await?;
    
    // 检�?channel_point_definitions 表结�?
    println!("\n🔍 检�?channel_point_definitions 表结�?");
    check_table_schema(db_conn, "channel_point_definitions").await?;

    // 专门检�?channel_position �?
    println!("\n🔧 专门检�?channel_position �?");
    check_and_fix_channel_position_column(db_conn).await?;

    // 检�?raw_test_outcomes 表结�?
    println!("\n📋 raw_test_outcomes 表结�?");
    check_table_schema(db_conn, "raw_test_outcomes").await?;

    // 专门检�?test_result_0_percent �?
    println!("\n🔧 专门检�?raw_test_outcomes 表的测试结果�?");
    check_and_fix_raw_test_outcomes_columns(db_conn).await?;

    println!("\n=== 检查完�?===");
    
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
        println!("�?�?{} 不存�?, table_name);
        return Ok(());
    }
    
    println!("�?�?{} 存在", table_name);
    
    // 获取表结�?
    let schema_sql = format!("PRAGMA table_info({})", table_name);
    let schema_result = db.query_all(Statement::from_string(
        sea_orm::DatabaseBackend::Sqlite,
        schema_sql
    )).await?;
    
    println!("   列信�?");
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

async fn check_and_fix_channel_position_column(db: &DatabaseConnection) -> Result<(), Box<dyn std::error::Error>> {
    // 获取 channel_point_definitions 表结�?
    let schema_sql = "PRAGMA table_info(channel_point_definitions)";
    let schema_result = db.query_all(Statement::from_string(
        sea_orm::DatabaseBackend::Sqlite,
        schema_sql.to_string()
    )).await?;

    // 检查是否存�?channel_position �?
    let has_channel_position = schema_result.iter().any(|row| {
        row.try_get::<String>("", "name").unwrap_or_default() == "channel_position"
    });

    if has_channel_position {
        println!("�?channel_position 列存�?);

        // 测试一个简单的查询
        println!("🧪 测试查询 channel_position �?..");
        let test_sql = "SELECT id, channel_position FROM channel_point_definitions LIMIT 1";
        match db.query_all(Statement::from_string(
            sea_orm::DatabaseBackend::Sqlite,
            test_sql.to_string()
        )).await {
            Ok(rows) => {
                println!("�?查询 channel_position 列成功，返回 {} �?, rows.len());
                for row in rows {
                    let id: String = row.try_get("", "id").unwrap_or_default();
                    let channel_position: String = row.try_get("", "channel_position").unwrap_or_default();
                    println!("   - ID: {}, channel_position: {}", id, channel_position);
                }
            },
            Err(e) => {
                println!("�?查询 channel_position 列失�? {}", e);
            }
        }
    } else {
        println!("�?channel_position 列不存在");

        // 尝试手动添加 channel_position �?
        println!("🔧 尝试手动添加 channel_position �?..");
        let add_column_sql = "ALTER TABLE channel_point_definitions ADD COLUMN channel_position TEXT NOT NULL DEFAULT ''";
        match db.execute(Statement::from_string(
            sea_orm::DatabaseBackend::Sqlite,
            add_column_sql.to_string()
        )).await {
            Ok(_) => println!("�?成功添加 channel_position �?),
            Err(e) => println!("�?添加 channel_position 列失�? {}", e),
        }
    }

    Ok(())
}

async fn check_and_fix_raw_test_outcomes_columns(db: &DatabaseConnection) -> Result<(), Box<dyn std::error::Error>> {
    // 获取 raw_test_outcomes 表结�?
    let schema_sql = "PRAGMA table_info(raw_test_outcomes)";
    let schema_result = db.query_all(Statement::from_string(
        sea_orm::DatabaseBackend::Sqlite,
        schema_sql.to_string()
    )).await?;

    // 检查需要的测试结果�?
    let required_columns = vec![
        "test_result_0_percent",
        "test_result_25_percent",
        "test_result_50_percent",
        "test_result_75_percent",
        "test_result_100_percent"
    ];

    let existing_columns: Vec<String> = schema_result.iter()
        .map(|row| row.try_get::<String>("", "name").unwrap_or_default())
        .collect();

    println!("   现有�? {:?}", existing_columns);

    for column in &required_columns {
        if existing_columns.contains(&column.to_string()) {
            println!("�?{} 列存�?, column);
        } else {
            println!("�?{} 列不存在", column);

            // 尝试手动添加�?
            println!("🔧 尝试手动添加 {} �?..", column);
            let add_column_sql = format!("ALTER TABLE raw_test_outcomes ADD COLUMN {} REAL", column);
            match db.execute(Statement::from_string(
                sea_orm::DatabaseBackend::Sqlite,
                add_column_sql
            )).await {
                Ok(_) => println!("�?成功添加 {} �?, column),
                Err(e) => println!("�?添加 {} 列失�? {}", column, e),
            }
        }
    }

    Ok(())
}

