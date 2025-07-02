// 测试大规模分配，使用更多通道来验证分批逻辑
use app_lib::services::infrastructure::persistence::{SqliteOrmPersistenceService, PersistenceConfig};
use app_lib::services::domain::{TestPlcConfigService, ITestPlcConfigService};
use app_lib::services::channel_allocation_service::{ChannelAllocationService, IChannelAllocationService};
use app_lib::models::test_plc_config::{GetTestPlcChannelsRequest, TestPlcChannelType};
use app_lib::models::{ChannelPointDefinition, ModuleType, PointDataType};
use std::sync::Arc;
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    env_logger::init();

    println!("=== 测试大规模通道分配 ===");

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
    let allocation_service = Arc::new(ChannelAllocationService::new());

    // 获取测试PLC配置
    let request = GetTestPlcChannelsRequest {
        channel_type_filter: None,
        enabled_only: Some(true),
    };

    let test_plc_channels = test_plc_config_service.get_test_plc_channels(request).await?;
    println!("从数据库获取到 {} 个测试PLC通道配置", test_plc_channels.len());

    // 创建测试PLC配置
    let test_plc_config = create_test_plc_config_from_channels(test_plc_channels);

    // 创建更多的通道定义来模拟真实场景
    let channel_definitions = create_large_channel_definitions();
    println!("创建了 {} 个通道点位定义", channel_definitions.len());

    // 统计通道类型
    let mut ai_count = 0;
    let mut ao_count = 0;
    let mut di_count = 0;
    let mut do_count = 0;

    for def in &channel_definitions {
        match def.module_type {
            ModuleType::AI | ModuleType::AINone => ai_count += 1,
            ModuleType::AO | ModuleType::AONone => ao_count += 1,
            ModuleType::DI | ModuleType::DINone => di_count += 1,
            ModuleType::DO | ModuleType::DONone => do_count += 1,
            ModuleType::Communication => {},
            ModuleType::Other(_) => {},
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
        channel_definitions,
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
        let batch_instances: Vec<_> = allocation_result.allocated_instances.iter()
            .filter(|instance| instance.test_batch_id == batch.batch_id)
            .collect();

        println!("批次 {}: {} ({})", i + 1, batch.batch_name, batch.batch_id);
        println!("  实例数量: {}", batch_instances.len());

        // 统计批次中的类型分布
        let mut batch_ai = 0;
        let mut batch_ao = 0;
        let mut batch_di = 0;
        let mut batch_do = 0;

        for instance in &batch_instances {
            // 根据定义ID查找对应的通道定义
            if let Some(def) = allocation_result.allocated_instances.iter()
                .find(|inst| inst.instance_id == instance.instance_id) {
                // 这里我们需要从定义ID推断类型，简化处理
                if instance.definition_id.contains("ai_") {
                    batch_ai += 1;
                } else if instance.definition_id.contains("ao_") {
                    batch_ao += 1;
                } else if instance.definition_id.contains("di_") {
                    batch_di += 1;
                } else if instance.definition_id.contains("do_") {
                    batch_do += 1;
                }
            }
        }

        println!("  类型分布: AI:{}, AO:{}, DI:{}, DO:{}", batch_ai, batch_ao, batch_di, batch_do);

        // 显示前几个实例的详情
        println!("  实例详情:");
        for (j, instance) in batch_instances.iter().take(10).enumerate() {
            let channel_type = if instance.definition_id.contains("ai_") { "AI" }
                              else if instance.definition_id.contains("ao_") { "AO" }
                              else if instance.definition_id.contains("di_") { "DI" }
                              else if instance.definition_id.contains("do_") { "DO" }
                              else { "Unknown" };

            println!("    {}. {} ({}) -> {}",
                     j + 1,
                     instance.definition_id,
                     channel_type,
                     instance.test_plc_channel_tag.as_ref().unwrap_or(&"未分配".to_string()));
        }
        if batch_instances.len() > 10 {
            println!("    ... 还有 {} 个实例", batch_instances.len() - 10);
        }
        println!();
    }

    // 分析未分配的通道
    let total_definitions = ai_count + ao_count + di_count + do_count;
    let unallocated_count = total_definitions - allocation_result.allocated_instances.len();

    if unallocated_count > 0 {
        println!("=== 未分配的通道 ===");
        println!("  未分配数量: {} 个", unallocated_count);

        // 这里可以进一步分析哪些类型的通道没有被分配
        // 由于我们没有保存原始定义列表，这里简化处理
    }

    println!("\n=== 分析完成 ===");

    Ok(())
}

/// 创建大规模的通道定义来测试分批逻辑
fn create_large_channel_definitions() -> Vec<ChannelPointDefinition> {
    let mut definitions = Vec::new();

    // 创建AI有源通道 (20个)
    for i in 1..=20 {
        let mut definition = ChannelPointDefinition::new_with_power_type(
            format!("AI_PWR_{:03}", i),
            format!("AI_Powered_{}", i),
            format!("模拟量输入通道{}", i),
            "测试站点".to_string(),
            "AI模块".to_string(),
            ModuleType::AI,
            format!("CH_{}", i),
            PointDataType::Float,
            format!("DB1.DBD{}", i * 4),
            "有源".to_string(),
        );
        definition.id = format!("ai_powered_{}", i);
        definition.wire_system = "二线制".to_string();
        definitions.push(definition);
    }

    // 创建AI无源通道 (15个)
    for i in 1..=15 {
        let mut definition = ChannelPointDefinition::new_with_power_type(
            format!("AI_UNPWR_{:03}", i),
            format!("AI_Unpowered_{}", i),
            format!("模拟量输入通道(无源){}", i),
            "测试站点".to_string(),
            "AI模块".to_string(),
            ModuleType::AI,
            format!("CH_{}", i),
            PointDataType::Float,
            format!("DB2.DBD{}", i * 4),
            "无源".to_string(),
        );
        definition.id = format!("ai_unpowered_{}", i);
        definition.wire_system = "二线制".to_string();
        definitions.push(definition);
    }

    // 创建AO有源通道 (15个)
    for i in 1..=15 {
        let mut definition = ChannelPointDefinition::new_with_power_type(
            format!("AO_PWR_{:03}", i),
            format!("AO_Powered_{}", i),
            format!("模拟量输出通道{}", i),
            "测试站点".to_string(),
            "AO模块".to_string(),
            ModuleType::AO,
            format!("CH_{}", i),
            PointDataType::Float,
            format!("DB3.DBD{}", i * 4),
            "有源".to_string(),
        );
        definition.id = format!("ao_powered_{}", i);
        definition.wire_system = "二线制".to_string();
        definitions.push(definition);
    }

    // 创建AO无源通道 (20个)
    for i in 1..=20 {
        let mut definition = ChannelPointDefinition::new_with_power_type(
            format!("AO_UNPWR_{:03}", i),
            format!("AO_Unpowered_{}", i),
            format!("模拟量输出通道(无源){}", i),
            "测试站点".to_string(),
            "AO模块".to_string(),
            ModuleType::AO,
            format!("CH_{}", i),
            PointDataType::Float,
            format!("DB4.DBD{}", i * 4),
            "无源".to_string(),
        );
        definition.id = format!("ao_unpowered_{}", i);
        definition.wire_system = "二线制".to_string();
        definitions.push(definition);
    }

    // 创建DI有源通道 (30个)
    for i in 1..=30 {
        let mut definition = ChannelPointDefinition::new_with_power_type(
            format!("DI_PWR_{:03}", i),
            format!("DI_Powered_{}", i),
            format!("数字量输入通道{}", i),
            "测试站点".to_string(),
            "DI模块".to_string(),
            ModuleType::DI,
            format!("CH_{}", i),
            PointDataType::Bool,
            format!("M{}", i),
            "有源".to_string(),
        );
        definition.id = format!("di_powered_{}", i);
        definition.wire_system = "二线制".to_string();
        definitions.push(definition);
    }

    // 创建DI无源通道 (25个)
    for i in 1..=25 {
        let mut definition = ChannelPointDefinition::new_with_power_type(
            format!("DI_UNPWR_{:03}", i),
            format!("DI_Unpowered_{}", i),
            format!("数字量输入通道(无源){}", i),
            "测试站点".to_string(),
            "DI模块".to_string(),
            ModuleType::DI,
            format!("CH_{}", i),
            PointDataType::Bool,
            format!("M{}", i + 100),
            "无源".to_string(),
        );
        definition.id = format!("di_unpowered_{}", i);
        definition.wire_system = "二线制".to_string();
        definitions.push(definition);
    }

    // 创建DO有源通道 (25个)
    for i in 1..=25 {
        let mut definition = ChannelPointDefinition::new_with_power_type(
            format!("DO_PWR_{:03}", i),
            format!("DO_Powered_{}", i),
            format!("数字量输出通道{}", i),
            "测试站点".to_string(),
            "DO模块".to_string(),
            ModuleType::DO,
            format!("CH_{}", i),
            PointDataType::Bool,
            format!("Q{}", i),
            "有源".to_string(),
        );
        definition.id = format!("do_powered_{}", i);
        definition.wire_system = "二线制".to_string();
        definitions.push(definition);
    }

    // 创建DO无源通道 (30个)
    for i in 1..=30 {
        let mut definition = ChannelPointDefinition::new_with_power_type(
            format!("DO_UNPWR_{:03}", i),
            format!("DO_Unpowered_{}", i),
            format!("数字量输出通道(无源){}", i),
            "测试站点".to_string(),
            "DO模块".to_string(),
            ModuleType::DO,
            format!("CH_{}", i),
            PointDataType::Bool,
            format!("Q{}", i + 100),
            "无源".to_string(),
        );
        definition.id = format!("do_unpowered_{}", i);
        definition.wire_system = "二线制".to_string();
        definitions.push(definition);
    }

    definitions
}

// 从测试PLC通道创建配置的辅助函数
use app_lib::{TestPlcConfig, ComparisonTable};

fn create_test_plc_config_from_channels(test_plc_channels: Vec<app_lib::models::test_plc_config::TestPlcChannelConfig>) -> TestPlcConfig {
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
        comparison_tables,
        brand_type: "Siemens".to_string(),
        ip_address: "192.168.1.100".to_string(),
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
