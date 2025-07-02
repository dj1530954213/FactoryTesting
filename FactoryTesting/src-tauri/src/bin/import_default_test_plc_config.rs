#![cfg(FALSE)]
// 导入默认的测试PLC配置
use app_lib::services::infrastructure::persistence::{SqliteOrmPersistenceService, PersistenceConfig};
use app_lib::services::domain::test_plc_config_service::TestPlcConfigService;
use app_lib::services::domain::ITestPlcConfigService;
use app_lib::models::test_plc_config::{TestPlcChannelConfig, TestPlcChannelType, GetTestPlcChannelsRequest};
use std::path::PathBuf;
use std::sync::Arc;
use uuid;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    println!("=== 导入默认测试PLC配置 ===");

    // 确保data目录存在
    let data_dir = PathBuf::from("data");
    if !data_dir.exists() {
        std::fs::create_dir_all(&data_dir)?;
        println!("�?创建data目录");
    }

    // 初始化数据库连接
    let db_path = PathBuf::from("data/factory_testing_data.sqlite");
    let persistence_config = PersistenceConfig {
        storage_root_dir: PathBuf::from("data"),
        channel_definitions_dir: "channel_definitions".to_string(),
        test_instances_dir: "test_instances".to_string(),
        test_batches_dir: "test_batches".to_string(),
        test_outcomes_dir: "test_outcomes".to_string(),
        enable_auto_backup: false,
        backup_retention_days: 30,
        max_file_size_mb: 100,
        enable_compression: false,
    };

    let persistence_service = Arc::new(SqliteOrmPersistenceService::new(persistence_config, Some(&db_path)).await?);
    let test_plc_config_service = Arc::new(TestPlcConfigService::new(persistence_service.clone()));

    println!("�?数据库连接成�?);

    // 先清空现有的测试PLC配置数据
    println!("🗑�? 清空现有的测试PLC配置数据...");
    let existing_request = GetTestPlcChannelsRequest {
        channel_type_filter: None,
        enabled_only: None,
    };
    let existing_channels = test_plc_config_service.get_test_plc_channels(existing_request).await?;

    for channel in &existing_channels {
        if let Some(ref id) = channel.id {
            match test_plc_config_service.delete_test_plc_channel(id).await {
                Ok(_) => println!("🗑�? 删除旧通道: {}", channel.channel_address),
                Err(e) => println!("�?删除旧通道失败: {} - {}", channel.channel_address, e),
            }
        }
    }

    println!("�?清空完成，删除了 {} 个旧通道配置", existing_channels.len());

    // 创建与原始数据完全一致的88个测试PLC通道配置
    let default_channels = create_default_test_plc_channels();
    println!("�?创建�?{} 个新的测试PLC通道配置", default_channels.len());

    // 批量保存到数据库
    for channel in &default_channels {
        match test_plc_config_service.save_test_plc_channel(channel.clone()).await {
            Ok(_) => println!("�?保存通道: {} - {}", channel.channel_address, format!("{:?}", channel.channel_type)),
            Err(e) => println!("�?保存通道失败: {} - {}", channel.channel_address, e),
        }
    }

    // 验证导入结果
    let request = GetTestPlcChannelsRequest {
        channel_type_filter: None,
        enabled_only: Some(true),
    };

    let saved_channels = test_plc_config_service.get_test_plc_channels(request).await?;
    println!("\n🎉 导入完成！数据库中现�?{} 个测试PLC通道配置", saved_channels.len());

    // 统计各类型通道数量
    let mut stats = std::collections::HashMap::new();
    for channel in &saved_channels {
        *stats.entry(format!("{:?}", channel.channel_type)).or_insert(0) += 1;
    }

    println!("\n📊 通道类型统计:");
    for (channel_type, count) in stats {
        println!("   {}: {} �?, channel_type, count);
    }

    Ok(())
}

/// 创建与原始数据完全一致的88个测试PLC通道配置
/// 基于原始SQL文件：AI1(8) + AO1(8) + AO2(8) + DI1(16) + DI2(16) + DO1(16) + DO2(16) = 88
fn create_default_test_plc_channels() -> Vec<TestPlcChannelConfig> {
    let mut channels = Vec::new();

    // AI1_1 �?AI1_8 (8个AI有源通道)
    for i in 1..=8 {
        channels.push(TestPlcChannelConfig {
            id: Some(uuid::Uuid::new_v4().to_string()),
            channel_address: format!("AI1_{}", i),
            communication_address: format!("{}", 40101 + (i - 1) * 2), // 40101, 40103, 40105...
            channel_type: TestPlcChannelType::AI,
            power_supply_type: "24V DC".to_string(),
            is_enabled: true,
            description: Some(format!("模拟量输入通道 {}", i)),
            created_at: Some(chrono::Utc::now()),
            updated_at: Some(chrono::Utc::now()),
        });
    }

    // AO1_1 �?AO1_8 (8个AO有源通道)
    for i in 1..=8 {
        channels.push(TestPlcChannelConfig {
            id: Some(uuid::Uuid::new_v4().to_string()),
            channel_address: format!("AO1_{}", i),
            communication_address: format!("{}", 40201 + (i - 1) * 2), // 40201, 40203, 40205...
            channel_type: TestPlcChannelType::AO,
            power_supply_type: "24V DC".to_string(),
            is_enabled: true,
            description: Some(format!("模拟量输出通道 {}", i)),
            created_at: Some(chrono::Utc::now()),
            updated_at: Some(chrono::Utc::now()),
        });
    }

    // AO2_1 �?AO2_8 (8个AO无源通道)
    for i in 1..=8 {
        channels.push(TestPlcChannelConfig {
            id: Some(uuid::Uuid::new_v4().to_string()),
            channel_address: format!("AO2_{}", i),
            communication_address: format!("{}", 40301 + (i - 1) * 2), // 40301, 40303, 40305...
            channel_type: TestPlcChannelType::AONone,
            power_supply_type: "无源".to_string(),
            is_enabled: true,
            description: Some(format!("模拟量输出通道(无源) {}", i)),
            created_at: Some(chrono::Utc::now()),
            updated_at: Some(chrono::Utc::now()),
        });
    }

    // DI1_1 �?DI1_16 (16个DI有源通道)
    for i in 1..=16 {
        channels.push(TestPlcChannelConfig {
            id: Some(uuid::Uuid::new_v4().to_string()),
            channel_address: format!("DI1_{}", i),
            communication_address: format!("{:05}", 101 + i - 1), // 00101, 00102, 00103...
            channel_type: TestPlcChannelType::DI,
            power_supply_type: "24V DC".to_string(),
            is_enabled: true,
            description: Some(format!("数字量输入通道 {}", i)),
            created_at: Some(chrono::Utc::now()),
            updated_at: Some(chrono::Utc::now()),
        });
    }

    // DI2_1 �?DI2_16 (16个DI无源通道)
    for i in 1..=16 {
        channels.push(TestPlcChannelConfig {
            id: Some(uuid::Uuid::new_v4().to_string()),
            channel_address: format!("DI2_{}", i),
            communication_address: format!("{:05}", 201 + i - 1), // 00201, 00202, 00203...
            channel_type: TestPlcChannelType::DINone,
            power_supply_type: "无源".to_string(),
            is_enabled: true,
            description: Some(format!("数字量输入通道(无源) {}", i)),
            created_at: Some(chrono::Utc::now()),
            updated_at: Some(chrono::Utc::now()),
        });
    }

    // DO1_1 �?DO1_16 (16个DO有源通道)
    for i in 1..=16 {
        channels.push(TestPlcChannelConfig {
            id: Some(uuid::Uuid::new_v4().to_string()),
            channel_address: format!("DO1_{}", i),
            communication_address: format!("{:05}", 301 + i - 1), // 00301, 00302, 00303...
            channel_type: TestPlcChannelType::DO,
            power_supply_type: "24V DC".to_string(),
            is_enabled: true,
            description: Some(format!("数字量输出通道 {}", i)),
            created_at: Some(chrono::Utc::now()),
            updated_at: Some(chrono::Utc::now()),
        });
    }

    // DO2_1 �?DO2_16 (16个DO无源通道)
    for i in 1..=16 {
        channels.push(TestPlcChannelConfig {
            id: Some(uuid::Uuid::new_v4().to_string()),
            channel_address: format!("DO2_{}", i),
            communication_address: format!("{:05}", 401 + i - 1), // 00401, 00402, 00403...
            channel_type: TestPlcChannelType::DONone,
            power_supply_type: "无源".to_string(),
            is_enabled: true,
            description: Some(format!("数字量输出通道(无源) {}", i)),
            created_at: Some(chrono::Utc::now()),
            updated_at: Some(chrono::Utc::now()),
        });
    }

    channels
}

