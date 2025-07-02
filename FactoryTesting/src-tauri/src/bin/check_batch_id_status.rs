#![cfg(FALSE)]
use sea_orm::{Database, DatabaseConnection, Statement, ConnectionTrait};
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 设置数据库路�?- 使用应用程序实际使用的路�?
    let db_path = env::current_dir()?.join("data").join("factory_testing_data.sqlite");
    let db_url = format!("sqlite://{}?mode=rwc", db_path.display());
    
    println!("连接数据�? {}", db_url);
    let db = Database::connect(&db_url).await?;

    // 检�?channel_point_definitions 表的 batch_id 字段状�?
    println!("\n=== 检�?channel_point_definitions 表的 batch_id 字段状�?===");
    
    // 1. 检查表结构
    let schema_sql = "PRAGMA table_info(channel_point_definitions)";
    let schema_result = db.query_all(Statement::from_string(
        sea_orm::DatabaseBackend::Sqlite,
        schema_sql.to_string()
    )).await?;
    
    println!("\n📋 表结�?");
    for row in &schema_result {
        let cid: i32 = row.try_get("", "cid")?;
        let name: String = row.try_get("", "name")?;
        let type_name: String = row.try_get("", "type")?;
        let notnull: i32 = row.try_get("", "notnull")?;
        let dflt_value: Option<String> = row.try_get("", "dflt_value").ok();
        let pk: i32 = row.try_get("", "pk")?;
        
        println!("  {}: {} {} {} {} {}", 
            cid, name, type_name, 
            if notnull == 1 { "NOT NULL" } else { "NULL" },
            dflt_value.unwrap_or("".to_string()),
            if pk == 1 { "PRIMARY KEY" } else { "" }
        );
    }

    // 2. 统计 batch_id 字段的数据状�?
    let count_sql = "SELECT COUNT(*) as total FROM channel_point_definitions";
    let count_result = db.query_one(Statement::from_string(
        sea_orm::DatabaseBackend::Sqlite,
        count_sql.to_string()
    )).await?;
    
    let total_count: i64 = count_result.unwrap().try_get("", "total")?;
    println!("\n📊 数据统计:");
    println!("  总记录数: {}", total_count);

    // 3. 统计�?batch_id 的记�?
    let with_batch_id_sql = "SELECT COUNT(*) as count FROM channel_point_definitions WHERE batch_id IS NOT NULL AND batch_id != ''";
    let with_batch_id_result = db.query_one(Statement::from_string(
        sea_orm::DatabaseBackend::Sqlite,
        with_batch_id_sql.to_string()
    )).await?;
    
    let with_batch_id_count: i64 = with_batch_id_result.unwrap().try_get("", "count")?;
    println!("  �?batch_id 的记�? {}", with_batch_id_count);

    // 4. 统计没有 batch_id 的记�?
    let without_batch_id_sql = "SELECT COUNT(*) as count FROM channel_point_definitions WHERE batch_id IS NULL OR batch_id = ''";
    let without_batch_id_result = db.query_one(Statement::from_string(
        sea_orm::DatabaseBackend::Sqlite,
        without_batch_id_sql.to_string()
    )).await?;
    
    let without_batch_id_count: i64 = without_batch_id_result.unwrap().try_get("", "count")?;
    println!("  没有 batch_id 的记�? {}", without_batch_id_count);

    // 5. 显示一些示例记�?
    let sample_sql = "SELECT id, tag, batch_id, station_name FROM channel_point_definitions LIMIT 10";
    let sample_result = db.query_all(Statement::from_string(
        sea_orm::DatabaseBackend::Sqlite,
        sample_sql.to_string()
    )).await?;
    
    println!("\n📝 示例记录:");
    for (i, row) in sample_result.iter().enumerate() {
        let id: String = row.try_get("", "id")?;
        let tag: String = row.try_get("", "tag")?;
        let batch_id: Option<String> = row.try_get("", "batch_id").ok();
        let station_name: Option<String> = row.try_get("", "station_name").ok();
        
        println!("  {}: ID={}, Tag={}, BatchID={:?}, Station={:?}", 
            i + 1, 
            &id[..8], 
            tag, 
            batch_id, 
            station_name
        );
    }

    // 6. 检�?test_batch_info �?
    println!("\n=== 检�?test_batch_info �?===");
    let batch_info_sql = "SELECT batch_id, batch_name, station_name, total_points FROM test_batch_info";
    let batch_info_result = db.query_all(Statement::from_string(
        sea_orm::DatabaseBackend::Sqlite,
        batch_info_sql.to_string()
    )).await?;
    
    println!("📦 批次信息:");
    for (i, row) in batch_info_result.iter().enumerate() {
        let batch_id: String = row.try_get("", "batch_id")?;
        let batch_name: String = row.try_get("", "batch_name")?;
        let station_name: Option<String> = row.try_get("", "station_name").ok();
        let total_points: Option<i32> = row.try_get("", "total_points").ok();
        
        println!("  {}: BatchID={}, Name={}, Station={:?}, Points={:?}", 
            i + 1, 
            &batch_id[..8], 
            batch_name, 
            station_name,
            total_points
        );
    }

    println!("\n�?数据库检查完�?);
    Ok(())
}

