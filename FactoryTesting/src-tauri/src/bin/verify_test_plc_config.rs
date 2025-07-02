#![cfg(FALSE)]
// 验证测试PLC配置中的有源/无源设置
use app_lib::services::infrastructure::persistence::{SqliteOrmPersistenceService, PersistenceConfig};
use app_lib::services::domain::{TestPlcConfigService, ITestPlcConfigService};
use app_lib::models::test_plc_config::{GetTestPlcChannelsRequest, TestPlcChannelType};
use std::sync::Arc;
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日�?
    env_logger::init();

    println!("=== 验证测试PLC配置中的有源/无源设置 ===");

    // 初始化服�?
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

    // 获取测试PLC配置
    let request = GetTestPlcChannelsRequest {
        channel_type_filter: None,
        enabled_only: Some(true),
    };

    let test_plc_channels = test_plc_config_service.get_test_plc_channels(request).await?;
    println!("从数据库获取�?{} 个测试PLC通道配置", test_plc_channels.len());

    // 按类型和有源/无源分组统计
    let mut ai_powered = Vec::new();
    let mut ai_unpowered = Vec::new();
    let mut ao_powered = Vec::new();
    let mut ao_unpowered = Vec::new();
    let mut di_powered = Vec::new();
    let mut di_unpowered = Vec::new();
    let mut do_powered = Vec::new();
    let mut do_unpowered = Vec::new();

    for channel in &test_plc_channels {
        // 根据channel_type枚举值判断是否有�?
        let is_powered = match channel.channel_type {
            TestPlcChannelType::AI | TestPlcChannelType::AO |
            TestPlcChannelType::DI | TestPlcChannelType::DO => true,
            TestPlcChannelType::AINone | TestPlcChannelType::AONone |
            TestPlcChannelType::DINone | TestPlcChannelType::DONone => false,
        };

        match channel.channel_type {
            TestPlcChannelType::AI => {
                ai_powered.push(channel);
            },
            TestPlcChannelType::AINone => {
                ai_unpowered.push(channel);
            },
            TestPlcChannelType::AO => {
                ao_powered.push(channel);
            },
            TestPlcChannelType::AONone => {
                ao_unpowered.push(channel);
            },
            TestPlcChannelType::DI => {
                di_powered.push(channel);
            },
            TestPlcChannelType::DINone => {
                di_unpowered.push(channel);
            },
            TestPlcChannelType::DO => {
                do_powered.push(channel);
            },
            TestPlcChannelType::DONone => {
                do_unpowered.push(channel);
            },
        }
    }

    println!("\n=== 测试PLC通道配置统计 ===");
    println!("AI有源: {} �?, ai_powered.len());
    println!("AI无源: {} �?, ai_unpowered.len());
    println!("AO有源: {} �?, ao_powered.len());
    println!("AO无源: {} �?, ao_unpowered.len());
    println!("DI有源: {} �?, di_powered.len());
    println!("DI无源: {} �?, di_unpowered.len());
    println!("DO有源: {} �?, do_powered.len());
    println!("DO无源: {} �?, do_unpowered.len());

    // 详细显示每种类型的前几个通道
    println!("\n=== 详细通道信息 ===");

    if !ai_powered.is_empty() {
        println!("\nAI有源通道:");
        for (i, channel) in ai_powered.iter().take(5).enumerate() {
            println!("  {}. {} - {} ({})",
                     i + 1,
                     channel.channel_address,
                     channel.power_supply_type,
                     channel.description.as_ref().unwrap_or(&"无描�?.to_string()));
        }
        if ai_powered.len() > 5 {
            println!("  ... 还有 {} �?, ai_powered.len() - 5);
        }
    }

    if !ai_unpowered.is_empty() {
        println!("\nAI无源通道:");
        for (i, channel) in ai_unpowered.iter().take(5).enumerate() {
            println!("  {}. {} - {} ({})",
                     i + 1,
                     channel.channel_address,
                     channel.power_supply_type,
                     channel.description.as_ref().unwrap_or(&"无描�?.to_string()));
        }
        if ai_unpowered.len() > 5 {
            println!("  ... 还有 {} �?, ai_unpowered.len() - 5);
        }
    }

    if !ao_powered.is_empty() {
        println!("\nAO有源通道:");
        for (i, channel) in ao_powered.iter().take(5).enumerate() {
            println!("  {}. {} - {} ({})",
                     i + 1,
                     channel.channel_address,
                     channel.power_supply_type,
                     channel.description.as_ref().unwrap_or(&"无描�?.to_string()));
        }
        if ao_powered.len() > 5 {
            println!("  ... 还有 {} �?, ao_powered.len() - 5);
        }
    }

    if !ao_unpowered.is_empty() {
        println!("\nAO无源通道:");
        for (i, channel) in ao_unpowered.iter().take(5).enumerate() {
            println!("  {}. {} - {} ({})",
                     i + 1,
                     channel.channel_address,
                     channel.power_supply_type,
                     channel.description.as_ref().unwrap_or(&"无描�?.to_string()));
        }
        if ao_unpowered.len() > 5 {
            println!("  ... 还有 {} �?, ao_unpowered.len() - 5);
        }
    }

    if !di_powered.is_empty() {
        println!("\nDI有源通道:");
        for (i, channel) in di_powered.iter().take(5).enumerate() {
            println!("  {}. {} - {} ({})",
                     i + 1,
                     channel.channel_address,
                     channel.power_supply_type,
                     channel.description.as_ref().unwrap_or(&"无描�?.to_string()));
        }
        if di_powered.len() > 5 {
            println!("  ... 还有 {} �?, di_powered.len() - 5);
        }
    }

    if !di_unpowered.is_empty() {
        println!("\nDI无源通道:");
        for (i, channel) in di_unpowered.iter().take(5).enumerate() {
            println!("  {}. {} - {} ({})",
                     i + 1,
                     channel.channel_address,
                     channel.power_supply_type,
                     channel.description.as_ref().unwrap_or(&"无描�?.to_string()));
        }
        if di_unpowered.len() > 5 {
            println!("  ... 还有 {} �?, di_unpowered.len() - 5);
        }
    }

    if !do_powered.is_empty() {
        println!("\nDO有源通道:");
        for (i, channel) in do_powered.iter().take(5).enumerate() {
            println!("  {}. {} - {} ({})",
                     i + 1,
                     channel.channel_address,
                     channel.power_supply_type,
                     channel.description.as_ref().unwrap_or(&"无描�?.to_string()));
        }
        if do_powered.len() > 5 {
            println!("  ... 还有 {} �?, do_powered.len() - 5);
        }
    }

    if !do_unpowered.is_empty() {
        println!("\nDO无源通道:");
        for (i, channel) in do_unpowered.iter().take(5).enumerate() {
            println!("  {}. {} - {} ({})",
                     i + 1,
                     channel.channel_address,
                     channel.power_supply_type,
                     channel.description.as_ref().unwrap_or(&"无描�?.to_string()));
        }
        if do_unpowered.len() > 5 {
            println!("  ... 还有 {} �?, do_unpowered.len() - 5);
        }
    }

    // 验证是否符合正确分配表的要求
    println!("\n=== 验证有源/无源匹配要求 ===");
    println!("根据正确分配表，需�?");
    println!("  AI有源 �?AO无源 (需要AO无源通道)");
    println!("  AO有源 �?AI有源 (需要AI有源通道)");
    println!("  DI有源 �?DO无源 (需要DO无源通道)");
    println!("  DO有源 �?DI有源 (需要DI有源通道)");

    println!("\n当前测试PLC配置:");
    println!("  AI有源: {} �? AO无源: {} �?�?AI有源测试需�? {}",
             ai_powered.len(), ao_unpowered.len(),
             if ao_unpowered.len() >= 4 { "�?满足" } else { "�?不足" });
    println!("  AO有源: {} �? AI有源: {} �?�?AO有源测试需�? {}",
             ao_powered.len(), ai_powered.len(),
             if ai_powered.len() >= 2 { "�?满足" } else { "�?不足" });
    println!("  DI有源: {} �? DO无源: {} �?�?DI有源测试需�? {}",
             di_powered.len(), do_unpowered.len(),
             if do_unpowered.len() >= 4 { "�?满足" } else { "�?不足" });
    println!("  DO有源: {} �? DI有源: {} �?�?DO有源测试需�? {}",
             do_powered.len(), di_powered.len(),
             if di_powered.len() >= 4 { "�?满足" } else { "�?不足" });

    println!("\n=== 验证完成 ===");

    Ok(())
}

