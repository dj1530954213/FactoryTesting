#![cfg(FALSE)]
use sea_orm::{Database, DatabaseConnection, Statement, ConnectionTrait};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== 检查数据库中的实际数据 ===");
    
    // 连接数据库
    let db = Database::connect("sqlite://data/factory_testing_data.sqlite").await?;
    
    // 检查 channel_test_instances 表中的测试结果数据
    println!("\n📊 检查 channel_test_instances 表中的测试结果数据:");
    check_channel_test_instances_data(&db).await?;
    
    // 检查 raw_test_outcomes 表中的测试结果数据
    println!("\n📊 检查 raw_test_outcomes 表中的测试结果数据:");
    check_raw_test_outcomes_data(&db).await?;
    
    println!("\n=== 检查完成 ===");
    
    Ok(())
}

async fn check_channel_test_instances_data(db: &DatabaseConnection) -> Result<(), Box<dyn std::error::Error>> {
    let sql = r#"
        SELECT 
            instance_id,
            channel_tag,
            test_result_0_percent,
            test_result_25_percent,
            test_result_50_percent,
            test_result_75_percent,
            test_result_100_percent,
            created_time,
            updated_time
        FROM channel_test_instances 
        ORDER BY updated_time DESC 
        LIMIT 10
    "#;
    
    let rows = db.query_all(Statement::from_string(
        sea_orm::DatabaseBackend::Sqlite,
        sql.to_string()
    )).await?;
    
    println!("   找到 {} 条测试实例记录", rows.len());
    
    for (i, row) in rows.iter().enumerate() {
        let instance_id: String = row.try_get("", "instance_id").unwrap_or_default();
        let channel_tag: String = row.try_get("", "channel_tag").unwrap_or_default();
        let test_0: Option<f64> = row.try_get("", "test_result_0_percent").ok();
        let test_25: Option<f64> = row.try_get("", "test_result_25_percent").ok();
        let test_50: Option<f64> = row.try_get("", "test_result_50_percent").ok();
        let test_75: Option<f64> = row.try_get("", "test_result_75_percent").ok();
        let test_100: Option<f64> = row.try_get("", "test_result_100_percent").ok();
        let created_time: String = row.try_get("", "created_time").unwrap_or_default();
        let updated_time: String = row.try_get("", "updated_time").unwrap_or_default();
        
        println!("   {}. 实例ID: {}", i + 1, instance_id);
        println!("      通道标签: {}", channel_tag);
        println!("      0%测试结果: {:?}", test_0);
        println!("      25%测试结果: {:?}", test_25);
        println!("      50%测试结果: {:?}", test_50);
        println!("      75%测试结果: {:?}", test_75);
        println!("      100%测试结果: {:?}", test_100);
        println!("      创建时间: {}", created_time);
        println!("      更新时间: {}", updated_time);
        println!();
    }
    
    Ok(())
}

async fn check_raw_test_outcomes_data(db: &DatabaseConnection) -> Result<(), Box<dyn std::error::Error>> {
    let sql = r#"
        SELECT 
            id,
            channel_instance_id,
            sub_test_item,
            success,
            test_result_0_percent,
            test_result_25_percent,
            test_result_50_percent,
            test_result_75_percent,
            test_result_100_percent,
            start_time,
            end_time
        FROM raw_test_outcomes 
        ORDER BY end_time DESC 
        LIMIT 10
    "#;
    
    let rows = db.query_all(Statement::from_string(
        sea_orm::DatabaseBackend::Sqlite,
        sql.to_string()
    )).await?;
    
    println!("   找到 {} 条原始测试结果记录", rows.len());
    
    for (i, row) in rows.iter().enumerate() {
        let id: String = row.try_get("", "id").unwrap_or_default();
        let channel_instance_id: String = row.try_get("", "channel_instance_id").unwrap_or_default();
        let sub_test_item: String = row.try_get("", "sub_test_item").unwrap_or_default();
        let success: bool = row.try_get("", "success").unwrap_or(false);
        let test_0: Option<f64> = row.try_get("", "test_result_0_percent").ok();
        let test_25: Option<f64> = row.try_get("", "test_result_25_percent").ok();
        let test_50: Option<f64> = row.try_get("", "test_result_50_percent").ok();
        let test_75: Option<f64> = row.try_get("", "test_result_75_percent").ok();
        let test_100: Option<f64> = row.try_get("", "test_result_100_percent").ok();
        let start_time: String = row.try_get("", "start_time").unwrap_or_default();
        let end_time: String = row.try_get("", "end_time").unwrap_or_default();
        
        println!("   {}. 结果ID: {}", i + 1, id);
        println!("      实例ID: {}", channel_instance_id);
        println!("      测试项目: {}", sub_test_item);
        println!("      成功: {}", success);
        println!("      0%测试结果: {:?}", test_0);
        println!("      25%测试结果: {:?}", test_25);
        println!("      50%测试结果: {:?}", test_50);
        println!("      75%测试结果: {:?}", test_75);
        println!("      100%测试结果: {:?}", test_100);
        println!("      开始时间: {}", start_time);
        println!("      结束时间: {}", end_time);
        println!();
    }
    
    Ok(())
}
