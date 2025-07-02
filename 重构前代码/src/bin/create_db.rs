// 创建数据库的简单程序
use app_lib::services::infrastructure::persistence::{SqliteOrmPersistenceService, PersistenceConfig};
use app_lib::services::domain::{TestPlcConfigService, ITestPlcConfigService};
use app_lib::services::traits::BaseService;
use std::sync::Arc;
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    env_logger::init();
    
    println!("正在创建SQLite数据库...");
    
    // 直接在当前目录创建数据库
    let db_file_path = PathBuf::from("factory_testing_data.sqlite");
    
    println!("数据库文件路径: {:?}", db_file_path);
    
    // 创建配置
    let mut config = PersistenceConfig::default();
    config.storage_root_dir = PathBuf::from(".");
    
    // 创建持久化服务
    let persistence_service = Arc::new(
        SqliteOrmPersistenceService::new(config, Some(&db_file_path)).await?
    );
    
    // 创建测试PLC配置服务
    let mut test_plc_config_service = TestPlcConfigService::new(persistence_service);
    
    // 初始化服务
    test_plc_config_service.initialize().await?;
    
    println!("数据库创建完成！");
    println!("数据库文件位置: {:?}", db_file_path.canonicalize()?);
    
    // 验证数据
    let channels = test_plc_config_service.get_test_plc_channels(
        app_lib::models::test_plc_config::GetTestPlcChannelsRequest {
            channel_type_filter: None,
            enabled_only: None,
        }
    ).await?;
    
    println!("已初始化 {} 个测试PLC通道配置", channels.len());
    
    // 显示一些示例数据
    for (i, channel) in channels.iter().take(5).enumerate() {
        println!("  {}. {} - {} ({})", 
            i + 1, 
            channel.channel_address, 
            channel.power_supply_type,
            channel.communication_address
        );
    }
    
    Ok(())
} 