// 检查数据库中的报警设定值地址配置
use sea_orm::{Database, Statement, ConnectionTrait};
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 检查数据库中的报警设定值地址配置");

    // 连接数据库
    let db_path = PathBuf::from("data/factory_testing_data.sqlite");
    let db_url = format!("sqlite://{}?mode=rwc", db_path.display());
    let db = Database::connect(&db_url).await?;
    println!("✅ 数据库连接成功");

    // 查询PT_2102点位的报警设定值地址
    println!("\n📊 查询PT_2102点位的报警设定值地址:");
    
    let query_sql = r#"
        SELECT 
            tag,
            plc_communication_address,
            sll_set_point_communication_address,
            sl_set_point_communication_address,
            sh_set_point_communication_address,
            shh_set_point_communication_address
        FROM channel_point_definitions 
        WHERE tag = 'PT_2102'
        LIMIT 1
    "#;
    
    let result = db.query_all(Statement::from_string(
        sea_orm::DatabaseBackend::Sqlite,
        query_sql.to_string(),
    )).await?;
    
    if let Some(row) = result.first() {
        let tag: String = row.try_get("", "tag")?;
        let main_addr: String = row.try_get("", "plc_communication_address")?;
        let sll_addr: Option<String> = row.try_get("", "sll_set_point_communication_address").ok();
        let sl_addr: Option<String> = row.try_get("", "sl_set_point_communication_address").ok();
        let sh_addr: Option<String> = row.try_get("", "sh_set_point_communication_address").ok();
        let shh_addr: Option<String> = row.try_get("", "shh_set_point_communication_address").ok();
        
        println!("   点位标识: {}", tag);
        println!("   主地址: {}", main_addr);
        println!("   SLL设定值地址: {:?}", sll_addr);
        println!("   SL设定值地址: {:?}", sl_addr);
        println!("   SH设定值地址: {:?}", sh_addr);
        println!("   SHH设定值地址: {:?}", shh_addr);
    } else {
        println!("❌ 未找到PT_2102点位");
    }

    // 查询所有AI点位的报警设定值地址配置情况
    println!("\n📊 查询所有AI点位的报警设定值地址配置情况:");
    
    let summary_sql = r#"
        SELECT 
            COUNT(*) as total_ai_points,
            COUNT(sll_set_point_communication_address) as sll_configured,
            COUNT(sl_set_point_communication_address) as sl_configured,
            COUNT(sh_set_point_communication_address) as sh_configured,
            COUNT(shh_set_point_communication_address) as shh_configured
        FROM channel_point_definitions 
        WHERE module_type = 'AI'
    "#;
    
    let summary_result = db.query_all(Statement::from_string(
        sea_orm::DatabaseBackend::Sqlite,
        summary_sql.to_string(),
    )).await?;
    
    if let Some(row) = summary_result.first() {
        let total: i64 = row.try_get("", "total_ai_points")?;
        let sll_count: i64 = row.try_get("", "sll_configured")?;
        let sl_count: i64 = row.try_get("", "sl_configured")?;
        let sh_count: i64 = row.try_get("", "sh_configured")?;
        let shh_count: i64 = row.try_get("", "shh_configured")?;
        
        println!("   总AI点位数: {}", total);
        println!("   配置SLL地址的点位数: {}", sll_count);
        println!("   配置SL地址的点位数: {}", sl_count);
        println!("   配置SH地址的点位数: {}", sh_count);
        println!("   配置SHH地址的点位数: {}", shh_count);
    }

    // 查询前5个AI点位的详细地址信息
    println!("\n📊 查询前5个AI点位的详细地址信息:");
    
    let detail_sql = r#"
        SELECT 
            tag,
            plc_communication_address,
            sll_set_point_communication_address,
            sl_set_point_communication_address,
            sh_set_point_communication_address,
            shh_set_point_communication_address
        FROM channel_point_definitions 
        WHERE module_type = 'AI'
        LIMIT 5
    "#;
    
    let detail_result = db.query_all(Statement::from_string(
        sea_orm::DatabaseBackend::Sqlite,
        detail_sql.to_string(),
    )).await?;
    
    for (i, row) in detail_result.iter().enumerate() {
        let tag: String = row.try_get("", "tag")?;
        let main_addr: String = row.try_get("", "plc_communication_address")?;
        let sll_addr: Option<String> = row.try_get("", "sll_set_point_communication_address").ok();
        let sl_addr: Option<String> = row.try_get("", "sl_set_point_communication_address").ok();
        let sh_addr: Option<String> = row.try_get("", "sh_set_point_communication_address").ok();
        let shh_addr: Option<String> = row.try_get("", "shh_set_point_communication_address").ok();
        
        println!("   {}. {} (主:{}) SLL:{:?} SL:{:?} SH:{:?} SHH:{:?}", 
                 i+1, tag, main_addr, sll_addr, sl_addr, sh_addr, shh_addr);
    }

    Ok(())
}
