use app_lib::utils::error::AppError;
use sea_orm::{Database, DatabaseConnection, EntityTrait, ColumnTrait, QueryFilter};
use std::path::PathBuf;
use app_lib::models::entities::channel_point_definition::{Entity as ChannelPointDefinition, Column};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    env_logger::init();
    
    println!("🔍 验证数据库中的通讯地址字段数据");
    
    // 连接数据库
    let db_path = PathBuf::from("./factory_testing_data.sqlite");
    let db_url = format!("sqlite://{}?mode=rwc", db_path.display());
    
    println!("📁 数据库路径: {}", db_url);
    
    let db = Database::connect(&db_url).await?;
    
    // 验证数据库中的数据
    verify_database_data(&db).await?;
    
    println!("✅ 数据库数据验证完成！");
    
    Ok(())
}

async fn verify_database_data(db: &DatabaseConnection) -> Result<(), AppError> {
    println!("🔍 查询数据库中的通道定义数据...");
    
    // 查询所有通道定义
    let definitions = ChannelPointDefinition::find()
        .all(db)
        .await
        .map_err(|e| AppError::persistence_error(format!("查询通道定义失败: {}", e)))?;
    
    println!("📊 数据库中共有 {} 个通道定义", definitions.len());
    
    if definitions.is_empty() {
        println!("⚠️  数据库中没有通道定义数据！");
        return Ok(());
    }
    
    // 统计各种字段的数据
    let mut plc_absolute_address_count = 0;
    let mut plc_communication_address_count = 0;
    let mut sll_plc_address_count = 0;
    let mut sll_communication_address_count = 0;
    let mut maintenance_plc_address_count = 0;
    let mut maintenance_communication_address_count = 0;
    
    println!("\n🔍 验证前5个定义的字段数据:");
    
    for (index, definition) in definitions.iter().take(5).enumerate() {
        println!("\n--- 数据库定义 {} ---", index + 1);
        println!("位号: {:?}", definition.tag);
        println!("PLC绝对地址: {:?}", definition.plc_absolute_address);
        println!("上位机通讯地址: {:?}", definition.plc_communication_address);
        println!("SLL设定点位_PLC地址: {:?}", definition.sll_set_point_plc_address);
        println!("SLL设定点位_通讯地址: {:?}", definition.sll_set_point_communication_address);
        println!("维护值设定点位_PLC地址: {:?}", definition.maintenance_value_set_point_plc_address);
        println!("维护值设定点位_通讯地址: {:?}", definition.maintenance_value_set_point_communication_address);
    }
    
    // 统计所有定义
    for definition in &definitions {
        if definition.plc_absolute_address.is_some() {
            plc_absolute_address_count += 1;
        }
        if !definition.plc_communication_address.is_empty() {
            plc_communication_address_count += 1;
        }
        if definition.sll_set_point_plc_address.is_some() {
            sll_plc_address_count += 1;
        }
        if definition.sll_set_point_communication_address.is_some() {
            sll_communication_address_count += 1;
        }
        if definition.maintenance_value_set_point_plc_address.is_some() {
            maintenance_plc_address_count += 1;
        }
        if definition.maintenance_value_set_point_communication_address.is_some() {
            maintenance_communication_address_count += 1;
        }
    }
    
    println!("\n📊 数据库字段统计:");
    println!("总定义数: {}", definitions.len());
    println!("包含PLC绝对地址的定义: {}", plc_absolute_address_count);
    println!("包含上位机通讯地址的定义: {}", plc_communication_address_count);
    println!("包含SLL报警PLC地址的定义: {}", sll_plc_address_count);
    println!("包含SLL报警通讯地址的定义: {}", sll_communication_address_count);
    println!("包含维护PLC地址的定义: {}", maintenance_plc_address_count);
    println!("包含维护通讯地址的定义: {}", maintenance_communication_address_count);
    
    // 验证数据完整性
    if plc_communication_address_count == definitions.len() {
        println!("✅ 上位机通讯地址字段数据完整");
    } else {
        println!("❌ 上位机通讯地址字段数据不完整");
    }
    
    if sll_communication_address_count > 0 {
        println!("✅ SLL报警通讯地址字段有数据");
    } else {
        println!("❌ SLL报警通讯地址字段没有数据");
    }
    
    if maintenance_communication_address_count > 0 {
        println!("✅ 维护通讯地址字段有数据");
    } else {
        println!("❌ 维护通讯地址字段没有数据");
    }
    
    Ok(())
}
