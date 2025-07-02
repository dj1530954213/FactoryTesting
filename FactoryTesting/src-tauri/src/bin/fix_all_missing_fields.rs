#![cfg(FALSE)]
use sea_orm::{Database, DatabaseConnection, Statement, ConnectionTrait};
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== 修复所有缺失的数据库字�?===");
    
    let db_path = PathBuf::from("data/factory_testing_data.sqlite");
    println!("📁 数据库文�? {:?}", db_path);
    
    if !db_path.exists() {
        println!("�?数据库文件不存在�?);
        return Ok(());
    }
    
    let db_url = format!("sqlite://{}?mode=rwc", db_path.display());
    let db = Database::connect(&db_url).await?;
    
    // 定义所有需要检查和添加的字�?
    let fields_to_check = vec![
        ("description", "TEXT"),
        ("sequence_number", "INTEGER"),
        ("power_type_description", "TEXT"),
        ("channel_position", "TEXT"),
        ("read_write_property", "TEXT"),
        ("module_name", "TEXT"),
        ("wire_system", "TEXT"),
        ("station_name", "TEXT"),
        ("variable_description", "TEXT"),
        ("data_type", "TEXT"),
        ("save_history", "TEXT"),
        ("power_off_protection", "TEXT"),
        ("range_low_limit", "REAL"),
        ("range_high_limit", "REAL"),
        ("sll_set_point_value", "REAL"),
        ("sll_set_point_position", "TEXT"),
        ("sll_set_point_plc_address", "TEXT"),
        ("sll_set_point_communication_address", "TEXT"),
        ("sl_set_point_value", "REAL"),
        ("sl_set_point_position", "TEXT"),
        ("sl_set_point_plc_address", "TEXT"),
        ("sl_set_point_communication_address", "TEXT"),
        ("sh_set_point_value", "REAL"),
        ("sh_set_point_position", "TEXT"),
        ("sh_set_point_plc_address", "TEXT"),
        ("sh_set_point_communication_address", "TEXT"),
        ("shh_set_point_value", "REAL"),
        ("shh_set_point_position", "TEXT"),
        ("shh_set_point_plc_address", "TEXT"),
        ("shh_set_point_communication_address", "TEXT"),
        ("sll_feedback_position", "TEXT"),
        ("sll_feedback_plc_address", "TEXT"),
        ("sll_feedback_communication_address", "TEXT"),
        ("sl_feedback_position", "TEXT"),
        ("sl_feedback_plc_address", "TEXT"),
        ("sl_feedback_communication_address", "TEXT"),
        ("sh_feedback_position", "TEXT"),
        ("sh_feedback_plc_address", "TEXT"),
        ("sh_feedback_communication_address", "TEXT"),
        ("shh_feedback_position", "TEXT"),
        ("shh_feedback_plc_address", "TEXT"),
        ("shh_feedback_communication_address", "TEXT"),
        ("ll_alarm_position", "TEXT"),
        ("ll_alarm_plc_address", "TEXT"),
        ("ll_alarm_communication_address", "TEXT"),
        ("l_alarm_position", "TEXT"),
        ("l_alarm_plc_address", "TEXT"),
        ("l_alarm_communication_address", "TEXT"),
        ("h_alarm_position", "TEXT"),
        ("h_alarm_plc_address", "TEXT"),
        ("h_alarm_communication_address", "TEXT"),
        ("hh_alarm_position", "TEXT"),
        ("hh_alarm_plc_address", "TEXT"),
        ("hh_alarm_communication_address", "TEXT"),
        ("maintenance_value_setting", "TEXT"),
        ("maintenance_value_position", "TEXT"),
        ("maintenance_value_plc_address", "TEXT"),
        ("maintenance_value_communication_address", "TEXT"),
        ("maintenance_enable_position", "TEXT"),
        ("maintenance_enable_plc_address", "TEXT"),
        ("maintenance_enable_communication_address", "TEXT"),
        ("ll_alarm_feedback", "TEXT"),
        ("l_alarm_feedback", "TEXT"),
        ("h_alarm_feedback", "TEXT"),
        ("hh_alarm_feedback", "TEXT"),
        ("created_at", "TEXT"),
        ("updated_at", "TEXT"),
    ];
    
    println!("\n🔍 检查并添加缺失的字�?..");
    
    let mut added_count = 0;
    let mut existing_count = 0;
    
    for (field_name, field_type) in &fields_to_check {
        let check_sql = format!("SELECT {} FROM channel_point_definitions LIMIT 1", field_name);
        let check_result = db.execute(Statement::from_string(
            sea_orm::DatabaseBackend::Sqlite,
            check_sql,
        )).await;
        
        match check_result {
            Ok(_) => {
                existing_count += 1;
                println!("�?字段 {} 已存�?, field_name);
            },
            Err(_) => {
                println!("⚠️  字段 {} 不存在，正在添加...", field_name);
                
                let add_sql = format!("ALTER TABLE channel_point_definitions ADD COLUMN {} {}", field_name, field_type);
                let add_result = db.execute(Statement::from_string(
                    sea_orm::DatabaseBackend::Sqlite,
                    add_sql,
                )).await;
                
                match add_result {
                    Ok(_) => {
                        added_count += 1;
                        println!("�?成功添加字段 {}", field_name);
                    },
                    Err(e) => {
                        println!("�?添加字段 {} 失败: {}", field_name, e);
                    }
                }
            }
        }
    }
    
    println!("\n📊 修复结果统计:");
    println!("  已存在字�? {} �?, existing_count);
    println!("  新添加字�? {} �?, added_count);
    println!("  总检查字�? {} �?, fields_to_check.len());
    
    // 验证修复结果
    println!("\n🔍 验证修复结果...");
    let verify_sql = "SELECT id, tag, variable_name, description, sequence_number, power_type_description, channel_position, read_write_property FROM channel_point_definitions LIMIT 1";
    let verify_result = db.execute(Statement::from_string(
        sea_orm::DatabaseBackend::Sqlite,
        verify_sql.to_string(),
    )).await;
    
    match verify_result {
        Ok(_) => println!("�?表结构修复成功！"),
        Err(e) => println!("�?表结构仍有问�? {}", e),
    }
    
    println!("\n🎉 数据库字段修复完成！");
    
    Ok(())
}

