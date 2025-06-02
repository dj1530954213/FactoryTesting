use sea_orm::{Database, DatabaseConnection, Statement, ConnectionTrait};
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== 修复数据库表结构问题 ===");
    
    let db_path = PathBuf::from("data/factory_testing_data.sqlite");
    println!("📁 数据库文件: {:?}", db_path);
    
    if !db_path.exists() {
        println!("❌ 数据库文件不存在！");
        return Ok(());
    }
    
    let db_url = format!("sqlite://{}?mode=rwc", db_path.display());
    let db = Database::connect(&db_url).await?;
    
    // 检查当前表结构
    println!("\n🔍 检查当前表结构...");
    let pragma_sql = "PRAGMA table_info(channel_point_definitions)";
    let result = db.execute(Statement::from_string(
        sea_orm::DatabaseBackend::Sqlite,
        pragma_sql.to_string(),
    )).await;
    
    match result {
        Ok(_) => println!("✅ 表存在"),
        Err(e) => {
            println!("❌ 表不存在或查询失败: {}", e);
            return Ok(());
        }
    }
    
    // 检查是否有description字段
    println!("\n🔍 检查description字段是否存在...");
    let check_desc_sql = "SELECT description FROM channel_point_definitions LIMIT 1";
    let desc_result = db.execute(Statement::from_string(
        sea_orm::DatabaseBackend::Sqlite,
        check_desc_sql.to_string(),
    )).await;

    match desc_result {
        Ok(_) => {
            println!("✅ description字段已存在");
        },
        Err(e) => {
            println!("⚠️  description字段不存在: {}", e);
            println!("🔧 添加description字段...");

            let add_desc_sql = "ALTER TABLE channel_point_definitions ADD COLUMN description TEXT";
            let add_result = db.execute(Statement::from_string(
                sea_orm::DatabaseBackend::Sqlite,
                add_desc_sql.to_string(),
            )).await;

            match add_result {
                Ok(_) => println!("✅ 成功添加description字段"),
                Err(e) => println!("❌ 添加description字段失败: {}", e),
            }
        }
    }

    // 检查是否有sequence_number字段
    println!("\n🔍 检查sequence_number字段是否存在...");
    let check_seq_sql = "SELECT sequence_number FROM channel_point_definitions LIMIT 1";
    let seq_result = db.execute(Statement::from_string(
        sea_orm::DatabaseBackend::Sqlite,
        check_seq_sql.to_string(),
    )).await;

    match seq_result {
        Ok(_) => {
            println!("✅ sequence_number字段已存在");
        },
        Err(e) => {
            println!("⚠️  sequence_number字段不存在: {}", e);
            println!("🔧 添加sequence_number字段...");

            let add_seq_sql = "ALTER TABLE channel_point_definitions ADD COLUMN sequence_number INTEGER";
            let add_seq_result = db.execute(Statement::from_string(
                sea_orm::DatabaseBackend::Sqlite,
                add_seq_sql.to_string(),
            )).await;

            match add_seq_result {
                Ok(_) => println!("✅ 成功添加sequence_number字段"),
                Err(e) => println!("❌ 添加sequence_number字段失败: {}", e),
            }
        }
    }

    // 检查是否有power_type_description字段
    println!("\n🔍 检查power_type_description字段是否存在...");
    let check_power_desc_sql = "SELECT power_type_description FROM channel_point_definitions LIMIT 1";
    let power_desc_result = db.execute(Statement::from_string(
        sea_orm::DatabaseBackend::Sqlite,
        check_power_desc_sql.to_string(),
    )).await;

    match power_desc_result {
        Ok(_) => {
            println!("✅ power_type_description字段已存在");
        },
        Err(e) => {
            println!("⚠️  power_type_description字段不存在: {}", e);
            println!("🔧 添加power_type_description字段...");

            let add_power_desc_sql = "ALTER TABLE channel_point_definitions ADD COLUMN power_type_description TEXT";
            let add_power_desc_result = db.execute(Statement::from_string(
                sea_orm::DatabaseBackend::Sqlite,
                add_power_desc_sql.to_string(),
            )).await;

            match add_power_desc_result {
                Ok(_) => println!("✅ 成功添加power_type_description字段"),
                Err(e) => println!("❌ 添加power_type_description字段失败: {}", e),
            }
        }
    }

    // 检查是否有channel_position字段
    println!("\n🔍 检查channel_position字段是否存在...");
    let check_pos_sql = "SELECT channel_position FROM channel_point_definitions LIMIT 1";
    let pos_result = db.execute(Statement::from_string(
        sea_orm::DatabaseBackend::Sqlite,
        check_pos_sql.to_string(),
    )).await;

    match pos_result {
        Ok(_) => {
            println!("✅ channel_position字段已存在");
        },
        Err(e) => {
            println!("⚠️  channel_position字段不存在: {}", e);
            println!("🔧 添加channel_position字段...");

            let add_pos_sql = "ALTER TABLE channel_point_definitions ADD COLUMN channel_position TEXT";
            let add_pos_result = db.execute(Statement::from_string(
                sea_orm::DatabaseBackend::Sqlite,
                add_pos_sql.to_string(),
            )).await;

            match add_pos_result {
                Ok(_) => println!("✅ 成功添加channel_position字段"),
                Err(e) => println!("❌ 添加channel_position字段失败: {}", e),
            }
        }
    }

    // 验证修复结果
    println!("\n🔍 验证修复结果...");
    let verify_sql = "SELECT id, tag, variable_name, description, sequence_number, power_type_description, channel_position FROM channel_point_definitions LIMIT 1";
    let verify_result = db.execute(Statement::from_string(
        sea_orm::DatabaseBackend::Sqlite,
        verify_sql.to_string(),
    )).await;
    
    match verify_result {
        Ok(_) => println!("✅ 表结构修复成功！"),
        Err(e) => println!("❌ 表结构仍有问题: {}", e),
    }
    
    println!("\n🎉 数据库表结构修复完成！");
    
    Ok(())
}
