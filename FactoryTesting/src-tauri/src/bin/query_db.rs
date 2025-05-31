// 查询数据库中的通道定义数据
use app_lib::services::infrastructure::persistence::{SqliteOrmPersistenceService, PersistenceConfig};
use app_lib::models::entities::channel_point_definition;
use sea_orm::EntityTrait;
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    env_logger::init();

    println!("正在查询数据库中的通道定义数据...");

    // 数据库文件路径
    let db_file_path = PathBuf::from("factory_testing_data.sqlite");

    if !db_file_path.exists() {
        println!("数据库文件不存在: {:?}", db_file_path);
        return Ok(());
    }

    println!("数据库文件路径: {:?}", db_file_path);

    // 创建配置
    let mut config = PersistenceConfig::default();
    config.storage_root_dir = PathBuf::from(".");

    // 创建持久化服务
    let persistence_service = SqliteOrmPersistenceService::new(config, Some(&db_file_path)).await?;
    let db = persistence_service.get_database_connection();

    // 查询所有通道定义
    let channel_definitions = channel_point_definition::Entity::find()
        .all(db)
        .await?;

    println!("数据库中共有 {} 个通道定义", channel_definitions.len());

    if channel_definitions.is_empty() {
        println!("数据库中没有通道定义数据");
        return Ok(());
    }

    // 按模块类型分组统计
    let mut ai_count = 0;
    let mut ao_count = 0;
    let mut di_count = 0;
    let mut do_count = 0;

    for def in &channel_definitions {
        match def.module_type.as_str() {
            "AI" => ai_count += 1,
            "AO" => ao_count += 1,
            "DI" => di_count += 1,
            "DO" => do_count += 1,
            _ => {}
        }
    }

    println!("模块类型统计:");
    println!("  AI: {} 个", ai_count);
    println!("  AO: {} 个", ao_count);
    println!("  DI: {} 个", di_count);
    println!("  DO: {} 个", do_count);

    // 显示前20个通道的详细信息
    println!("\n前20个通道的详细信息:");
    for (i, def) in channel_definitions.iter().take(20).enumerate() {
        println!("{}. tag={}, module_type={}, plc_address={}, variable_name={}",
                 i + 1,
                 def.tag,
                 def.module_type,
                 def.plc_communication_address,
                 def.variable_name);
    }

    if channel_definitions.len() > 20 {
        println!("... 还有 {} 个通道", channel_definitions.len() - 20);
    }

    Ok(())
}
