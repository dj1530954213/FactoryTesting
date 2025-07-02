#![cfg(FALSE)]
// 检查数据库中的实际数据
use sea_orm::{Database, DatabaseConnection, Statement, ConnectionTrait, QueryResult};
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 检查数据库中的实际数据");

    // 连接数据�?
    let db_path = PathBuf::from("data/factory_testing_data.sqlite");
    let db_url = format!("sqlite://{}?mode=rwc", db_path.display());
    let db = Database::connect(&db_url).await?;
    println!("�?数据库连接成�?);

    // 检�?channel_point_definitions 表的数据
    println!("\n📊 检�?channel_point_definitions �?");
    
    // 统计记录�?
    let count_result = db.query_one(Statement::from_string(
        sea_orm::DatabaseBackend::Sqlite,
        "SELECT COUNT(*) as count FROM channel_point_definitions".to_string(),
    )).await?;
    
    if let Some(row) = count_result {
        let count: i64 = row.try_get("", "count")?;
        println!("   总记录数: {}", count);
        
        if count > 0 {
            // 查看�?条记�?
            let sample_result = db.query_all(Statement::from_string(
                sea_orm::DatabaseBackend::Sqlite,
                "SELECT id, tag, variable_name, module_type, power_supply_type, channel_tag_in_module FROM channel_point_definitions LIMIT 5".to_string(),
            )).await?;
            
            println!("   �?条记�?");
            for (index, row) in sample_result.iter().enumerate() {
                let id: String = row.try_get("", "id")?;
                let tag: String = row.try_get("", "tag")?;
                let variable_name: String = row.try_get("", "variable_name")?;
                let module_type: String = row.try_get("", "module_type")?;
                let power_supply_type: String = row.try_get("", "power_supply_type")?;
                let channel_tag: String = row.try_get("", "channel_tag_in_module")?;
                
                println!("     {}. ID: {}, Tag: {}, 变量�? {}, 模块类型: {}, 供电类型: {}, 通道标签: {}",
                    index + 1, &id[..8], tag, variable_name, module_type, power_supply_type, channel_tag);
            }
        }
    }

    // 检�?test_batch_info 表的数据
    println!("\n📊 检�?test_batch_info �?");
    
    let batch_count_result = db.query_one(Statement::from_string(
        sea_orm::DatabaseBackend::Sqlite,
        "SELECT COUNT(*) as count FROM test_batch_info".to_string(),
    )).await?;
    
    if let Some(row) = batch_count_result {
        let count: i64 = row.try_get("", "count")?;
        println!("   总记录数: {}", count);
        
        if count > 0 {
            // 查看所有批次记�?
            let batch_result = db.query_all(Statement::from_string(
                sea_orm::DatabaseBackend::Sqlite,
                "SELECT batch_id, batch_name, total_points, created_at FROM test_batch_info".to_string(),
            )).await?;
            
            println!("   所有批次记�?");
            for (index, row) in batch_result.iter().enumerate() {
                let batch_id: String = row.try_get("", "batch_id")?;
                let batch_name: String = row.try_get("", "batch_name")?;
                let total_points: i64 = row.try_get("", "total_points")?;
                let created_at: String = row.try_get("", "created_at")?;
                
                println!("     {}. ID: {}, 名称: {}, 总点�? {}, 创建时间: {}",
                    index + 1, &batch_id[..20], batch_name, total_points, created_at);
            }
        }
    }

    // 检�?channel_test_instances 表的数据
    println!("\n📊 检�?channel_test_instances �?");
    
    let instance_count_result = db.query_one(Statement::from_string(
        sea_orm::DatabaseBackend::Sqlite,
        "SELECT COUNT(*) as count FROM channel_test_instances".to_string(),
    )).await?;
    
    if let Some(row) = instance_count_result {
        let count: i64 = row.try_get("", "count")?;
        println!("   总记录数: {}", count);
        
        if count > 0 {
            // 查看�?条测试实例记�?
            let instance_result = db.query_all(Statement::from_string(
                sea_orm::DatabaseBackend::Sqlite,
                "SELECT instance_id, definition_id, test_batch_id, overall_status, assigned_test_plc_channel FROM channel_test_instances LIMIT 5".to_string(),
            )).await?;
            
            println!("   �?条测试实例记�?");
            for (index, row) in instance_result.iter().enumerate() {
                let instance_id: String = row.try_get("", "instance_id")?;
                let definition_id: String = row.try_get("", "definition_id")?;
                let test_batch_id: String = row.try_get("", "test_batch_id")?;
                let overall_status: String = row.try_get("", "overall_status")?;
                let assigned_plc_channel: Option<String> = row.try_get("", "assigned_test_plc_channel").ok();
                
                println!("     {}. 实例ID: {}, 定义ID: {}, 批次ID: {}, 状�? {}, PLC通道: {:?}",
                    index + 1, &instance_id[..8], &definition_id[..8], &test_batch_id[..20], overall_status, assigned_plc_channel);
            }
        }
    }

    println!("\n🎉 数据库数据检查完成！");
    Ok(())
}

