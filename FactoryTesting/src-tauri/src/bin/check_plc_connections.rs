#![cfg(FALSE)]
use app_lib::services::infrastructure::persistence::{SqliteOrmPersistenceService, PersistenceConfig};
use app_lib::services::domain::test_plc_config_service::{TestPlcConfigService, ITestPlcConfigService};
use app_lib::utils::error::AppError;
use std::sync::Arc;
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<(), AppError> {
    println!("=== 检查PLC连接配置 ===");

    // 初始化数据库连接
    let db_path = PathBuf::from("data/factory_testing_data.sqlite");
    let persistence_config = PersistenceConfig {
        storage_root_dir: PathBuf::from("data"),
        channel_definitions_dir: "channel_definitions".to_string(),
        test_instances_dir: "test_instances".to_string(),
        test_batches_dir: "test_batches".to_string(),
        test_outcomes_dir: "test_outcomes".to_string(),
        enable_auto_backup: true,
        backup_retention_days: 30,
        max_file_size_mb: 100,
        enable_compression: false,
    };

    let persistence_service = Arc::new(SqliteOrmPersistenceService::new(persistence_config, Some(&db_path)).await?);
    let test_plc_config_service = Arc::new(TestPlcConfigService::new(persistence_service.clone()));

    // 获取PLC连接配置
    let plc_connections = test_plc_config_service.get_plc_connections().await?;
    println!("从数据库获取�?{} 个PLC连接配置", plc_connections.len());
    
    if plc_connections.is_empty() {
        println!("�?没有找到任何PLC连接配置�?);
        return Ok(());
    }
    
    println!("\n=== PLC连接配置详情 ===");
    for (i, conn) in plc_connections.iter().enumerate() {
        println!("{}. {} ({})", i + 1, conn.name, if conn.is_test_plc { "测试PLC" } else { "被测PLC" });
        println!("   IP地址: {}:{}", conn.ip_address, conn.port);
        println!("   类型: {:?}", conn.plc_type);
        println!("   启用状�? {}", if conn.is_enabled { "启用" } else { "禁用" });
        println!("   超时时间: {}ms", conn.timeout);
        println!("   重试次数: {}", conn.retry_count);
        if let Some(desc) = &conn.description {
            println!("   描述: {}", desc);
        }
        println!();
    }
    
    // 检查是否有启用的测试PLC和被测PLC
    let test_plc = plc_connections.iter().find(|conn| conn.is_test_plc && conn.is_enabled);
    let target_plc = plc_connections.iter().find(|conn| !conn.is_test_plc && conn.is_enabled);
    
    println!("=== 配置验证 ===");
    match test_plc {
        Some(plc) => println!("�?找到启用的测试PLC: {} ({}:{})", plc.name, plc.ip_address, plc.port),
        None => println!("�?没有找到启用的测试PLC配置�?),
    }
    
    match target_plc {
        Some(plc) => println!("�?找到启用的被测PLC: {} ({}:{})", plc.name, plc.ip_address, plc.port),
        None => println!("�?没有找到启用的被测PLC配置�?),
    }
    
    // 检查是否使用了相同的IP地址
    if let (Some(test), Some(target)) = (test_plc, target_plc) {
        if test.ip_address == target.ip_address && test.port == target.port {
            println!("⚠️  警告：测试PLC和被测PLC使用了相同的IP地址和端口！");
            println!("   这会导致测试失败，因为它们实际上是同一个PLC实例�?);
            println!("   测试PLC: {}:{}", test.ip_address, test.port);
            println!("   被测PLC: {}:{}", target.ip_address, target.port);
        } else {
            println!("�?测试PLC和被测PLC使用了不同的IP地址，配置正确�?);
        }
    }
    
    println!("\n=== 检查完�?===");
    Ok(())
}

