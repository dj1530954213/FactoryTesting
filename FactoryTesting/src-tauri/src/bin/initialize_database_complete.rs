use sea_orm::{Database, DatabaseConnection, Statement, ConnectionTrait};
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔧 完整初始化数据库，包含所有通讯地址字段");
    
    let db_path = PathBuf::from("./factory_testing_data.sqlite");
    let db_url = format!("sqlite://{}?mode=rwc", db_path.display());
    println!("📁 数据库路径: {}", db_url);
    
    let db = Database::connect(&db_url).await?;
    
    // 删除现有表
    println!("🗑️ 删除现有通道定义表...");
    let drop_table_sql = "DROP TABLE IF EXISTS channel_point_definitions";
    db.execute(Statement::from_string(
        sea_orm::DatabaseBackend::Sqlite,
        drop_table_sql.to_string(),
    )).await?;

    // 创建通道定义表，完全匹配点表结构
    println!("🔧 创建通道定义表（完全匹配点表结构）...");
    let create_channel_definitions_sql = r#"
        CREATE TABLE channel_point_definitions (
            id TEXT PRIMARY KEY,
            tag TEXT NOT NULL,
            variable_name TEXT NOT NULL,
            module_type TEXT NOT NULL,
            plc_absolute_address TEXT,
            plc_communication_address TEXT NOT NULL,
            power_supply_type INTEGER NOT NULL,
            description TEXT,

            -- 基本字段
            sequence_number INTEGER,
            module_name TEXT,
            power_type_description TEXT,
            wire_system TEXT,
            channel_position TEXT,
            station_name TEXT,
            variable_description TEXT,
            data_type TEXT,
            read_write_property TEXT,
            save_history TEXT,
            power_off_protection TEXT,
            range_low_limit REAL,
            range_high_limit REAL,

            -- SLL 报警设定
            sll_set_point_value REAL,
            sll_set_point_position TEXT,
            sll_set_point_plc_address TEXT,
            sll_set_point_communication_address TEXT,

            -- SL 报警设定
            sl_set_point_value REAL,
            sl_set_point_position TEXT,
            sl_set_point_plc_address TEXT,
            sl_set_point_communication_address TEXT,

            -- SH 报警设定
            sh_set_point_value REAL,
            sh_set_point_position TEXT,
            sh_set_point_plc_address TEXT,
            sh_set_point_communication_address TEXT,

            -- SHH 报警设定
            shh_set_point_value REAL,
            shh_set_point_position TEXT,
            shh_set_point_plc_address TEXT,
            shh_set_point_communication_address TEXT,

            -- LL/L/H/HH 报警反馈
            ll_alarm_feedback TEXT,
            ll_alarm_plc_address TEXT,
            ll_alarm_communication_address TEXT,
            l_alarm_feedback TEXT,
            l_alarm_plc_address TEXT,
            l_alarm_communication_address TEXT,
            h_alarm_feedback TEXT,
            h_alarm_plc_address TEXT,
            h_alarm_communication_address TEXT,
            hh_alarm_feedback TEXT,
            hh_alarm_plc_address TEXT,
            hh_alarm_communication_address TEXT,

            -- 维护相关
            maintenance_value_setting TEXT,
            maintenance_value_position TEXT,
            maintenance_value_plc_address TEXT,
            maintenance_value_communication_address TEXT,
            maintenance_enable_position TEXT,
            maintenance_enable_plc_address TEXT,
            maintenance_enable_communication_address TEXT,

            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
        )
    "#;
    
    db.execute(Statement::from_string(
        sea_orm::DatabaseBackend::Sqlite,
        create_channel_definitions_sql.to_string(),
    )).await?;
    
    // 创建批次信息表
    println!("🔧 创建批次信息表...");
    let create_batch_info_sql = r#"
        CREATE TABLE IF NOT EXISTS test_batch_info (
            batch_id TEXT PRIMARY KEY,
            batch_name TEXT NOT NULL,
            product_model TEXT,
            serial_number TEXT,
            total_points INTEGER NOT NULL,
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
        )
    "#;
    
    db.execute(Statement::from_string(
        sea_orm::DatabaseBackend::Sqlite,
        create_batch_info_sql.to_string(),
    )).await?;
    
    // 创建测试实例表
    println!("🔧 创建测试实例表...");
    let create_test_instances_sql = r#"
        CREATE TABLE IF NOT EXISTS channel_test_instances (
            instance_id TEXT PRIMARY KEY,
            definition_id TEXT NOT NULL,
            test_batch_id TEXT NOT NULL,
            overall_status TEXT NOT NULL,
            assigned_test_plc_channel TEXT,
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (definition_id) REFERENCES channel_point_definitions(id),
            FOREIGN KEY (test_batch_id) REFERENCES test_batch_info(batch_id)
        )
    "#;
    
    db.execute(Statement::from_string(
        sea_orm::DatabaseBackend::Sqlite,
        create_test_instances_sql.to_string(),
    )).await?;
    
    // 创建测试PLC配置表
    println!("🔧 创建测试PLC配置表...");
    let create_test_plc_channels_sql = r#"
        CREATE TABLE IF NOT EXISTS test_plc_channels (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            channel_name TEXT NOT NULL UNIQUE,
            channel_type TEXT NOT NULL,
            communication_address TEXT NOT NULL,
            power_supply_type INTEGER NOT NULL,
            is_enabled BOOLEAN NOT NULL DEFAULT 1,
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
        )
    "#;
    
    db.execute(Statement::from_string(
        sea_orm::DatabaseBackend::Sqlite,
        create_test_plc_channels_sql.to_string(),
    )).await?;
    
    let create_plc_connections_sql = r#"
        CREATE TABLE IF NOT EXISTS plc_connections (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            connection_name TEXT NOT NULL UNIQUE,
            plc_brand TEXT NOT NULL,
            ip_address TEXT NOT NULL,
            port INTEGER NOT NULL,
            is_enabled BOOLEAN NOT NULL DEFAULT 1,
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
        )
    "#;
    
    db.execute(Statement::from_string(
        sea_orm::DatabaseBackend::Sqlite,
        create_plc_connections_sql.to_string(),
    )).await?;
    
    println!("✅ 数据库表结构创建完成！");
    
    // 验证表结构
    println!("🔍 验证表结构...");
    let tables = vec![
        "channel_point_definitions",
        "test_batch_info", 
        "channel_test_instances",
        "test_plc_channels",
        "plc_connections"
    ];
    
    for table in tables {
        let result = db.execute(Statement::from_string(
            sea_orm::DatabaseBackend::Sqlite,
            format!("PRAGMA table_info({})", table),
        )).await;
        
        match result {
            Ok(_) => println!("✅ 表 {} 创建成功", table),
            Err(e) => println!("❌ 表 {} 创建失败: {}", table, e),
        }
    }
    
    println!("🎉 数据库完整初始化完成！");
    
    Ok(())
}
