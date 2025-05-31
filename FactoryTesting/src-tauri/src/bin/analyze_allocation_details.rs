// 详细分析分配结果
use app_lib::services::infrastructure::persistence::{SqliteOrmPersistenceService, PersistenceConfig};
use app_lib::services::channel_allocation_service::{ChannelAllocationService, TestPlcConfig, ComparisonTable, IChannelAllocationService};
use app_lib::services::domain::{TestPlcConfigService, ITestPlcConfigService};
use app_lib::models::structs::ChannelPointDefinition;
use app_lib::models::enums::{ModuleType, PointDataType};
use app_lib::models::test_plc_config::{GetTestPlcChannelsRequest, TestPlcChannelType, TestPlcChannelConfig};
use std::sync::Arc;
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    env_logger::init();

    println!("=== 详细分析分配结果 ===");

    // 初始化服务
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
    let allocation_service = ChannelAllocationService::new();

    // 获取测试PLC配置
    let request = GetTestPlcChannelsRequest {
        channel_type_filter: None,
        enabled_only: Some(true),
    };

    let test_plc_channels = test_plc_config_service.get_test_plc_channels(request).await?;
    println!("从数据库获取到 {} 个测试PLC通道配置", test_plc_channels.len());

    // 创建测试PLC配置
    let test_plc_config = create_test_plc_config_from_channels(&test_plc_channels);

    // 创建真实的通道点位定义（只创建前14个，模拟正确分配表的情况）
    let real_channel_definitions = create_limited_real_channel_definitions();

    println!("创建了 {} 个通道点位定义", real_channel_definitions.len());

    // 按类型统计通道定义
    let mut ai_count = 0;
    let mut ao_count = 0;
    let mut di_count = 0;
    let mut do_count = 0;

    for def in &real_channel_definitions {
        match def.module_type {
            ModuleType::AI | ModuleType::AINone => ai_count += 1,
            ModuleType::AO | ModuleType::AONone => ao_count += 1,
            ModuleType::DI | ModuleType::DINone => di_count += 1,
            ModuleType::DO | ModuleType::DONone => do_count += 1,
            ModuleType::Communication => {}, // 忽略通信模块
            ModuleType::Other(_) => {}, // 忽略其他类型
        }
    }

    println!("通道定义类型统计:");
    println!("  AI: {} 个", ai_count);
    println!("  AO: {} 个", ao_count);
    println!("  DI: {} 个", di_count);
    println!("  DO: {} 个", do_count);

    // 执行分配
    println!("\n=== 开始执行批次分配测试 ===");
    let allocation_result = allocation_service.allocate_channels(
        real_channel_definitions.clone(),
        test_plc_config,
        None,
        None,
    ).await?;

    println!("分配结果:");
    println!("  生成批次数: {} 个", allocation_result.batches.len());
    println!("  分配实例数: {} 个", allocation_result.allocated_instances.len());

    // 详细分析每个批次
    println!("\n=== 详细批次分析 ===");
    for (i, batch) in allocation_result.batches.iter().enumerate() {
        println!("批次 {}: {} ({})", i + 1, batch.batch_name, batch.batch_id);

        // 统计该批次中的实例
        let batch_instances: Vec<_> = allocation_result.allocated_instances.iter()
            .filter(|instance| instance.test_batch_id == batch.batch_id)
            .collect();

        println!("  实例数量: {}", batch_instances.len());

        // 按类型统计
        let mut batch_ai = 0;
        let mut batch_ao = 0;
        let mut batch_di = 0;
        let mut batch_do = 0;

        for instance in &batch_instances {
            // 根据定义ID查找对应的通道定义
            if let Some(def) = real_channel_definitions.iter().find(|d| d.id == instance.definition_id) {
                match def.module_type {
                    ModuleType::AI | ModuleType::AINone => batch_ai += 1,
                    ModuleType::AO | ModuleType::AONone => batch_ao += 1,
                    ModuleType::DI | ModuleType::DINone => batch_di += 1,
                    ModuleType::DO | ModuleType::DONone => batch_do += 1,
                    ModuleType::Communication => {}, // 忽略通信模块
                    ModuleType::Other(_) => {}, // 忽略其他类型
                }
            }
        }

        println!("  类型分布: AI:{}, AO:{}, DI:{}, DO:{}", batch_ai, batch_ao, batch_di, batch_do);

        // 显示前几个实例的详细信息
        println!("  实例详情:");
        for (j, instance) in batch_instances.iter().take(5).enumerate() {
            if let Some(def) = real_channel_definitions.iter().find(|d| d.id == instance.definition_id) {
                println!("    {}. {} ({}) -> {}",
                         j + 1,
                         def.tag,
                         format!("{:?}", def.module_type),
                         instance.test_plc_channel_tag.as_ref().unwrap_or(&"未分配".to_string()));
            }
        }
        if batch_instances.len() > 5 {
            println!("    ... 还有 {} 个实例", batch_instances.len() - 5);
        }
        println!();
    }

    // 分析未分配的通道
    let allocated_definition_ids: std::collections::HashSet<_> = allocation_result.allocated_instances.iter()
        .map(|instance| &instance.definition_id)
        .collect();

    let unallocated_definitions: Vec<_> = real_channel_definitions.iter()
        .filter(|def| !allocated_definition_ids.contains(&def.id))
        .collect();

    if !unallocated_definitions.is_empty() {
        println!("=== 未分配的通道 ===");
        for def in &unallocated_definitions {
            println!("  {} ({:?}) - {}", def.tag, def.module_type, def.power_supply_type);
        }
    }

    println!("\n=== 分析完成 ===");

    Ok(())
}

/// 创建有限的真实通道点位定义（模拟正确分配表的14个通道）
fn create_limited_real_channel_definitions() -> Vec<ChannelPointDefinition> {
    let mut definitions = Vec::new();

    // AI通道 (4个) - 有源
    let ai_tags = ["PT_2101", "PT_2102", "TT_4101", "TT_4102"];
    for (i, tag) in ai_tags.iter().enumerate() {
        definitions.push(ChannelPointDefinition::new_with_power_type(
            format!("ai_def_{}", i + 1),
            tag.to_string(),
            format!("AI通道{}", i + 1),
            "樟洋电厂".to_string(),
            "8通道模拟量输入模块".to_string(),
            ModuleType::AI,
            format!("1_2_AI_{}", i),
            PointDataType::Float,
            format!("{}", 40001 + i * 2),
            "有源".to_string(),
        ));
    }

    // AO通道 (2个) - 有源
    let ao_tags = ["FCV_7101_AO", "YLDW1_4_AO_1"];
    for (i, tag) in ao_tags.iter().enumerate() {
        definitions.push(ChannelPointDefinition::new_with_power_type(
            format!("ao_def_{}", i + 1),
            tag.to_string(),
            format!("AO通道{}", i + 1),
            "樟洋电厂".to_string(),
            "8通道模拟量输出模块".to_string(),
            ModuleType::AO,
            format!("1_4_AO_{}", i),
            PointDataType::Float,
            format!("{}", 40033 + i * 2),
            "有源".to_string(),
        ));
    }

    // DI通道 (4个) - 有源
    let di_tags = ["ESDV6101_1", "ESDV6101_2", "ESDV6101_Z0", "ESDV6101_ZC"];
    for (i, tag) in di_tags.iter().enumerate() {
        definitions.push(ChannelPointDefinition::new_with_power_type(
            format!("di_def_{}", i + 1),
            tag.to_string(),
            format!("DI通道{}", i + 1),
            "樟洋电厂".to_string(),
            "16通道数字量输入模块".to_string(),
            ModuleType::DI,
            format!("1_5_DI_{}", i),
            PointDataType::Bool,
            format!("{:05}", 1 + i),
            "有源".to_string(),
        ));
    }

    // DO通道 (4个) - 有源
    let do_tags = ["DO_1_CL_1", "DO_2_OP_1", "SQ6103_S0", "SQ6103_SC"];
    for (i, tag) in do_tags.iter().enumerate() {
        definitions.push(ChannelPointDefinition::new_with_power_type(
            format!("do_def_{}", i + 1),
            tag.to_string(),
            format!("DO通道{}", i + 1),
            "樟洋电厂".to_string(),
            "16通道数字量输出模块".to_string(),
            ModuleType::DO,
            format!("1_7_DO_{}", i),
            PointDataType::Bool,
            format!("{:05}", 33 + i),
            "有源".to_string(),
        ));
    }

    definitions
}

/// 从测试PLC通道创建配置
fn create_test_plc_config_from_channels(
    test_plc_channels: &[TestPlcChannelConfig]
) -> TestPlcConfig {
    let mut comparison_tables = Vec::new();

    for channel in test_plc_channels {
        // 根据channel_type枚举值判断是否有源
        let is_powered = match channel.channel_type {
            TestPlcChannelType::AI | TestPlcChannelType::AO |
            TestPlcChannelType::DI | TestPlcChannelType::DO => true,
            TestPlcChannelType::AINone | TestPlcChannelType::AONone |
            TestPlcChannelType::DINone | TestPlcChannelType::DONone => false,
        };

        comparison_tables.push(ComparisonTable {
            channel_address: channel.channel_address.clone(),
            communication_address: channel.communication_address.clone(),
            channel_type: convert_test_plc_channel_type_to_module_type(&channel.channel_type),
            is_powered,
        });
    }

    TestPlcConfig {
        brand_type: "Siemens".to_string(),
        ip_address: "192.168.1.100".to_string(),
        comparison_tables,
    }
}

/// 转换TestPlcChannelType到ModuleType
fn convert_test_plc_channel_type_to_module_type(channel_type: &TestPlcChannelType) -> ModuleType {
    match channel_type {
        TestPlcChannelType::AI => ModuleType::AI,
        TestPlcChannelType::AO => ModuleType::AO,
        TestPlcChannelType::DI => ModuleType::DI,
        TestPlcChannelType::DO => ModuleType::DO,
        TestPlcChannelType::AINone => ModuleType::AI,
        TestPlcChannelType::AONone => ModuleType::AO,
        TestPlcChannelType::DINone => ModuleType::DI,
        TestPlcChannelType::DONone => ModuleType::DO,
    }
}
