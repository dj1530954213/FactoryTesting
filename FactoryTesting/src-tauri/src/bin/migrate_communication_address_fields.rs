#![cfg(FALSE)]
use app_lib::database_migration::DatabaseMigration;
use app_lib::utils::error::AppError;
use sea_orm::{Database, DatabaseConnection, Statement, ConnectionTrait};
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日�?
    env_logger::init();
    
    println!("🔧 开始数据库迁移：添加通讯地址字段");
    
    // 连接数据�?
    let db_path = PathBuf::from("./factory_testing_data.sqlite");
    let db_url = format!("sqlite://{}?mode=rwc", db_path.display());
    
    println!("📁 数据库路�? {}", db_url);
    
    let db = Database::connect(&db_url).await?;
    
    // 执行迁移
    migrate_communication_address_fields(&db).await?;
    
    println!("�?数据库迁移完成！");
    
    Ok(())
}

async fn migrate_communication_address_fields(db: &DatabaseConnection) -> Result<(), AppError> {
    println!("🔧 开始添加通讯地址字段�?channel_point_definitions �?..");
    
    // 添加SLL相关通讯地址字段
    let sll_fields = vec![
        "ALTER TABLE channel_point_definitions ADD COLUMN sll_set_point_communication_address TEXT;",
        "ALTER TABLE channel_point_definitions ADD COLUMN sll_feedback_communication_address TEXT;",
    ];
    
    // 添加SL相关通讯地址字段
    let sl_fields = vec![
        "ALTER TABLE channel_point_definitions ADD COLUMN sl_set_point_communication_address TEXT;",
        "ALTER TABLE channel_point_definitions ADD COLUMN sl_feedback_communication_address TEXT;",
    ];
    
    // 添加SH相关通讯地址字段
    let sh_fields = vec![
        "ALTER TABLE channel_point_definitions ADD COLUMN sh_set_point_communication_address TEXT;",
        "ALTER TABLE channel_point_definitions ADD COLUMN sh_feedback_communication_address TEXT;",
    ];
    
    // 添加SHH相关通讯地址字段
    let shh_fields = vec![
        "ALTER TABLE channel_point_definitions ADD COLUMN shh_set_point_communication_address TEXT;",
        "ALTER TABLE channel_point_definitions ADD COLUMN shh_feedback_communication_address TEXT;",
    ];
    
    // 添加维护相关通讯地址字段
    let maintenance_fields = vec![
        "ALTER TABLE channel_point_definitions ADD COLUMN maintenance_value_set_point_communication_address TEXT;",
        "ALTER TABLE channel_point_definitions ADD COLUMN maintenance_enable_switch_point_communication_address TEXT;",
    ];
    
    // 合并所有SQL语句
    let all_statements = [sll_fields, sl_fields, sh_fields, shh_fields, maintenance_fields].concat();
    
    // 执行每个ALTER TABLE语句
    for (index, sql) in all_statements.iter().enumerate() {
        println!("🔧 执行迁移 {}/{}: {}", index + 1, all_statements.len(), sql);
        
        match db.execute(Statement::from_string(
            sea_orm::DatabaseBackend::Sqlite,
            sql.to_string()
        )).await {
            Ok(_) => {
                println!("�?迁移 {}/{} 成功", index + 1, all_statements.len());
            },
            Err(e) => {
                // 检查是否是"列已存在"的错�?
                let error_msg = e.to_string();
                if error_msg.contains("duplicate column name") || error_msg.contains("already exists") {
                    println!("⚠️  迁移 {}/{} 跳过：列已存�?, index + 1, all_statements.len());
                } else {
                    eprintln!("�?迁移 {}/{} 失败: {}", index + 1, all_statements.len(), e);
                    return Err(AppError::persistence_error(format!("迁移失败: {}", e)));
                }
            }
        }
    }
    
    println!("🔍 验证表结�?..");
    
    // 验证表结�?
    let table_info = db.execute(Statement::from_string(
        sea_orm::DatabaseBackend::Sqlite,
        "PRAGMA table_info(channel_point_definitions);".to_string()
    )).await.map_err(|e| AppError::persistence_error(format!("获取表信息失�? {}", e)))?;
    
    println!("📊 当前表结�?");
    println!("{:?}", table_info);
    
    // 检查新字段是否存在
    let check_columns = vec![
        "sll_set_point_communication_address",
        "sll_feedback_communication_address",
        "sl_set_point_communication_address", 
        "sl_feedback_communication_address",
        "sh_set_point_communication_address",
        "sh_feedback_communication_address",
        "shh_set_point_communication_address",
        "shh_feedback_communication_address",
        "maintenance_value_set_point_communication_address",
        "maintenance_enable_switch_point_communication_address",
    ];
    
    for column in check_columns {
        let check_sql = format!(
            "SELECT COUNT(*) as count FROM pragma_table_info('channel_point_definitions') WHERE name = '{}';",
            column
        );
        
        let result = db.execute(Statement::from_string(
            sea_orm::DatabaseBackend::Sqlite,
            check_sql
        )).await.map_err(|e| AppError::persistence_error(format!("检查列失败: {}", e)))?;
        
        println!("🔍 检查列 '{}': {:?}", column, result);
    }
    
    println!("�?所有通讯地址字段迁移完成�?);
    
    Ok(())
}

