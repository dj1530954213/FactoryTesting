#![cfg(FALSE)]
// 使用真实数据测试批次分配算法
use app_lib::services::infrastructure::persistence::{SqliteOrmPersistenceService, PersistenceConfig};
use app_lib::services::channel_allocation_service::{ChannelAllocationService, TestPlcConfig, ComparisonTable, IChannelAllocationService};
use app_lib::services::domain::{TestPlcConfigService, ITestPlcConfigService};
use app_lib::services::traits::BaseService;
use app_lib::models::structs::ChannelPointDefinition;
use app_lib::models::enums::{ModuleType, PointDataType};
use app_lib::models::test_plc_config::{GetTestPlcChannelsRequest, TestPlcChannelType, TestPlcChannelConfig};
use std::sync::Arc;
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日�?
    env_logger::init();

    println!("=== 使用真实数据测试批次分配算法 ===");

    // 数据库文件路�?
    let db_file_path = PathBuf::from("factory_testing_data.sqlite");

    if !db_file_path.exists() {
        println!("数据库文件不存在: {:?}", db_file_path);
        return Ok(());
    }

    // 创建配置
    let mut config = PersistenceConfig::default();
    config.storage_root_dir = PathBuf::from(".");

    // 创建持久化服�?
    let persistence_service = Arc::new(
        SqliteOrmPersistenceService::new(config, Some(&db_file_path)).await?
    );

    // 创建测试PLC配置服务
    let mut test_plc_config_service = TestPlcConfigService::new(persistence_service.clone());
    test_plc_config_service.initialize().await?;

    // 获取真实的测试PLC通道配置
    let test_plc_channels = test_plc_config_service.get_test_plc_channels(
        GetTestPlcChannelsRequest {
            channel_type_filter: None,
            enabled_only: Some(true),
        }
    ).await?;

    println!("从数据库获取�?{} 个测试PLC通道配置", test_plc_channels.len());

    // 按类型统�?
    let mut ai_count = 0;
    let mut ao_count = 0;
    let mut di_count = 0;
    let mut do_count = 0;

    for channel in &test_plc_channels {
        match channel.channel_type {
            TestPlcChannelType::AI => ai_count += 1,
            TestPlcChannelType::AO => ao_count += 1,
            TestPlcChannelType::AONone => ao_count += 1,  // AO2 (无源)
            TestPlcChannelType::DI => di_count += 1,
            TestPlcChannelType::DINone => di_count += 1,  // DI2 (无源)
            TestPlcChannelType::DO => do_count += 1,
            TestPlcChannelType::DONone => do_count += 1,  // DO2 (无源)
            _ => {}
        }
    }

    println!("测试PLC通道类型统计:");
    println!("  AI: {} �?, ai_count);
    println!("  AO: {} �?, ao_count);
    println!("  DI: {} �?, di_count);
    println!("  DO: {} �?, do_count);

    // 创建真实的通道点位定义（基于测试IO.txt的完�?8个通道�?
    let real_channel_definitions = create_complete_real_channel_definitions();

    println!("\n创建�?{} 个真实通道点位定义", real_channel_definitions.len());

    // 按类型统计通道定义
    let mut def_ai_count = 0;
    let mut def_ao_count = 0;
    let mut def_di_count = 0;
    let mut def_do_count = 0;

    for def in &real_channel_definitions {
        match def.module_type {
            ModuleType::AI => def_ai_count += 1,
            ModuleType::AO => def_ao_count += 1,
            ModuleType::DI => def_di_count += 1,
            ModuleType::DO => def_do_count += 1,
            _ => {}
        }
    }

    println!("通道定义类型统计:");
    println!("  AI: {} �?, def_ai_count);
    println!("  AO: {} �?, def_ao_count);
    println!("  DI: {} �?, def_di_count);
    println!("  DO: {} �?, def_do_count);

    // 创建分配服务
    let allocation_service = ChannelAllocationService::new();

    // 创建测试PLC配置
    let test_plc_config = create_test_plc_config_from_channels(&test_plc_channels);

    // 执行分配测试
    println!("\n=== 开始执行批次分配测�?===");

    let allocation_result = allocation_service.allocate_channels(
        real_channel_definitions,
        test_plc_config,
        Some("测试产品".to_string()),
        Some("SN001".to_string()),
    ).await?;

    println!("分配结果:");
    println!("  生成批次�? {} �?, allocation_result.batches.len());
    println!("  分配实例�? {} �?, allocation_result.allocated_instances.len());

    // 显示批次信息
    if !allocation_result.batches.is_empty() {
        println!("\n批次信息:");
        for (i, batch) in allocation_result.batches.iter().enumerate() {
            println!("  批次 {}: {} ({})",
                     i + 1,
                     batch.batch_name,
                     batch.batch_id);
        }
    }

    // 显示分配实例的详细信息（�?0个）
    if !allocation_result.allocated_instances.is_empty() {
        println!("\n分配实例（前20个）:");
        for (i, instance) in allocation_result.allocated_instances.iter().take(20).enumerate() {
            println!("  {}. {} -> 批次: {}",
                     i + 1,
                     instance.definition_id,
                     instance.test_batch_id);
        }

        if allocation_result.allocated_instances.len() > 20 {
            println!("  ... 还有 {} 个分配实�?, allocation_result.allocated_instances.len() - 20);
        }
    }

    println!("\n=== 测试完成 ===");

    Ok(())
}

/// 从测试PLC通道创建配置
fn create_test_plc_config_from_channels(
    test_plc_channels: &[TestPlcChannelConfig]
) -> TestPlcConfig {
    let mut comparison_tables = Vec::new();

    for channel in test_plc_channels {
        let module_type = match channel.channel_type {
            TestPlcChannelType::AI | TestPlcChannelType::AINone => ModuleType::AI,
            TestPlcChannelType::AO | TestPlcChannelType::AONone => ModuleType::AO,
            TestPlcChannelType::DI | TestPlcChannelType::DINone => ModuleType::DI,
            TestPlcChannelType::DO | TestPlcChannelType::DONone => ModuleType::DO,
        };

        // 根据description判断是否有源：description中没�?无源"字样就是有源
        let is_powered = !channel.description.as_ref()
            .map(|desc| desc.contains("无源"))
            .unwrap_or(false);

        comparison_tables.push(ComparisonTable {
            channel_address: channel.channel_address.clone(),
            communication_address: channel.communication_address.clone(),
            channel_type: module_type,
            is_powered,
        });
    }

    TestPlcConfig {
        brand_type: "测试PLC".to_string(),
        ip_address: "192.168.1.100".to_string(),
        comparison_tables,
    }
}

/// 创建完整�?8个真实通道点位定义（基于测试IO.txt的完整数据）
fn create_complete_real_channel_definitions() -> Vec<ChannelPointDefinition> {
    let mut definitions = Vec::new();

    // AI通道 (1-17) - 模拟量输�?
    for i in 0..17 {
        let channel_address = format!("1_2_AI_{}", i);
        let tag = match i {
            0 => "PT_2101".to_string(),
            1 => "PT_2102".to_string(),
            2 => "TT_4101".to_string(),
            3 => "TT_4102".to_string(),
            4 => "Y1791_2_AI_4".to_string(),
            5 => "Y1791_2_AI_5".to_string(),
            6 => "Y1791_2_AI_6".to_string(),
            7 => "Y1791_2_AI_7".to_string(),
            8 => "FIQ_5702".to_string(),
            9 => "PDI_6301_AI".to_string(),
            10 => "Y1791_3_AI_2".to_string(),
            11 => "Y1791_3_AI_3".to_string(),
            12 => "Y1791_3_AI_4".to_string(),
            13 => "Y1791_3_AI_5".to_string(),
            14 => "Y1791_3_AI_6".to_string(),
            15 => "Y1791_3_AI_7".to_string(),
            16 => "FCV_7101_AO".to_string(),
            _ => format!("AI_CHANNEL_{}", i + 1),
        };

        definitions.push(ChannelPointDefinition::new_with_power_type(
            format!("ai_def_{}", i + 1),
            tag,
            format!("AI通道{}", i + 1),
            "樟洋电厂".to_string(),
            "8通道模拟量输入模�?.to_string(),
            ModuleType::AI,
            channel_address,
            PointDataType::Float,
            format!("{}", 40001 + i * 2),
            "有源".to_string(), // AI通道都是有源
        ));
    }

    // AO通道 (18-25) - 模拟量输�?
    for i in 0..8 {
        let channel_address = format!("1_4_AO_{}", i);
        let tag = match i {
            0 => "FCV_7101_AO".to_string(),
            1 => "YLDW1_4_AO_1".to_string(),
            2 => "Y1791_4_AO_2".to_string(),
            3 => "Y1791_4_AO_3".to_string(),
            4 => "Y1791_4_AO_4".to_string(),
            5 => "Y1791_4_AO_5".to_string(),
            6 => "Y1791_4_AO_6".to_string(),
            7 => "Y1791_4_AO_7".to_string(),
            _ => format!("AO_CHANNEL_{}", i + 1),
        };

        definitions.push(ChannelPointDefinition::new_with_power_type(
            format!("ao_def_{}", i + 1),
            tag,
            format!("AO通道{}", i + 1),
            "樟洋电厂".to_string(),
            "8通道模拟量输出模�?.to_string(),
            ModuleType::AO,
            channel_address,
            PointDataType::Float,
            format!("{}", 40033 + i * 2),
            "有源".to_string(), // AO通道都是有源
        ));
    }

    // DI通道 (26-57) - 数字量输�?
    for i in 0..32 {
        let channel_address = format!("1_5_DI_{}", i);
        let tag = match i {
            0 => "ESDV6101_1".to_string(),
            1 => "ESDV6101_2".to_string(),
            2 => "ESDV6101_Z0".to_string(),
            3 => "ESDV6101_ZC".to_string(),
            4 => "S06101_20".to_string(),
            5 => "S06101_20".to_string(),
            6 => "S06101_20".to_string(),
            7 => "FCV7101_FL".to_string(),
            8 => "S06101_20".to_string(),
            9 => "S06101_20".to_string(),
            10 => "SSV6301_S0".to_string(),
            11 => "SSV6301_SC".to_string(),
            12 => "Y1791_5_DI_12".to_string(),
            13 => "Y1791_5_DI_13".to_string(),
            14 => "Y1791_5_DI_14".to_string(),
            15 => "Y1791_6_DI_15".to_string(),
            _ => format!("DI_CHANNEL_{}", i + 1),
        };

        definitions.push(ChannelPointDefinition::new_with_power_type(
            format!("di_def_{}", i + 1),
            tag,
            format!("DI通道{}", i + 1),
            "樟洋电厂".to_string(),
            "16通道数字量输入模�?.to_string(),
            ModuleType::DI,
            channel_address,
            PointDataType::Bool,
            format!("{:05}", 1 + i),
            "有源".to_string(), // DI通道都是有源
        ));
    }

    // DO通道 (58-88) - 数字量输�?
    for i in 0..31 {
        let channel_address = format!("1_7_DO_{}", i);
        let tag = match i {
            0 => "DO_1_CL_1".to_string(),
            1 => "DO_2_OP_1".to_string(),
            2 => "SQ6103_S0".to_string(),
            3 => "SQ6103_SC".to_string(),
            4 => "S06101_S0".to_string(),
            5 => "S06101_SC".to_string(),
            6 => "S06102_SC".to_string(),
            7 => "S06102_S0".to_string(),
            8 => "S06103_S0".to_string(),
            9 => "S06103_SC".to_string(),
            10 => "S06100_ETH".to_string(),
            11 => "SA1001".to_string(),
            12 => "Y1791_7_DO_12".to_string(),
            13 => "Y1791_7_DO_13".to_string(),
            14 => "Y1791_7_DO_14".to_string(),
            15 => "Y1791_7_DO_15".to_string(),
            _ => format!("DO_CHANNEL_{}", i + 1),
        };

        definitions.push(ChannelPointDefinition::new_with_power_type(
            format!("do_def_{}", i + 1),
            tag,
            format!("DO通道{}", i + 1),
            "樟洋电厂".to_string(),
            "16通道数字量输出模�?.to_string(),
            ModuleType::DO,
            channel_address,
            PointDataType::Bool,
            format!("{:05}", 33 + i),
            "有源".to_string(), // DO通道都是有源
        ));
    }

    definitions
}

/// 创建真实的通道点位定义（基于测试IO.txt的前14个通道�?
fn create_real_channel_definitions() -> Vec<ChannelPointDefinition> {
    vec![
        // AI通道 (1-4)
        ChannelPointDefinition::new(
            "1_2_AI_0".to_string(),
            "PT_2101".to_string(),
            "计量撬进口压�?.to_string(),
            "樟洋电厂".to_string(),
            "8通道模拟量输入模�?.to_string(),
            ModuleType::AI,
            "1_2_AI_0".to_string(),
            PointDataType::Float,
            "40001".to_string(),
        ),
        ChannelPointDefinition::new(
            "1_2_AI_1".to_string(),
            "PT_2102".to_string(),
            "计量撬出口压�?.to_string(),
            "樟洋电厂".to_string(),
            "8通道模拟量输入模�?.to_string(),
            ModuleType::AI,
            "1_2_AI_1".to_string(),
            PointDataType::Float,
            "40003".to_string(),
        ),
        ChannelPointDefinition::new(
            "1_2_AI_2".to_string(),
            "TT_4101".to_string(),
            "计量撬进口温�?.to_string(),
            "樟洋电厂".to_string(),
            "8通道模拟量输入模�?.to_string(),
            ModuleType::AI,
            "1_2_AI_2".to_string(),
            PointDataType::Float,
            "40005".to_string(),
        ),
        ChannelPointDefinition::new(
            "1_2_AI_3".to_string(),
            "TT_4102".to_string(),
            "计量撬出口温�?.to_string(),
            "樟洋电厂".to_string(),
            "8通道模拟量输入模�?.to_string(),
            ModuleType::AI,
            "1_2_AI_3".to_string(),
            PointDataType::Float,
            "40007".to_string(),
        ),

        // AO通道 (1-2)
        ChannelPointDefinition::new(
            "1_4_AO_0".to_string(),
            "FCV_7101_AO".to_string(),
            "计量撬出口气动阀控制指令".to_string(),
            "樟洋电厂".to_string(),
            "8通道模拟量输出模�?.to_string(),
            ModuleType::AO,
            "1_4_AO_0".to_string(),
            PointDataType::Float,
            "40033".to_string(),
        ),
        ChannelPointDefinition::new(
            "1_4_AO_1".to_string(),
            "YLDW1_4_AO_1".to_string(),
            "预留点位".to_string(),
            "樟洋电厂".to_string(),
            "8通道模拟量输出模�?.to_string(),
            ModuleType::AO,
            "1_4_AO_1".to_string(),
            PointDataType::Float,
            "40035".to_string(),
        ),

        // DI通道 (1-4)
        ChannelPointDefinition::new(
            "1_5_DI_0".to_string(),
            "ESDV6101_1".to_string(),
            "电磁阀1电流监视继电器失�?.to_string(),
            "樟洋电厂".to_string(),
            "16通道数字量输入模�?.to_string(),
            ModuleType::DI,
            "1_5_DI_0".to_string(),
            PointDataType::Bool,
            "00001".to_string(),
        ),
        ChannelPointDefinition::new(
            "1_5_DI_1".to_string(),
            "ESDV6101_2".to_string(),
            "电磁阀2电流监视继电器失�?.to_string(),
            "樟洋电厂".to_string(),
            "16通道数字量输入模�?.to_string(),
            ModuleType::DI,
            "1_5_DI_1".to_string(),
            PointDataType::Bool,
            "00002".to_string(),
        ),
        ChannelPointDefinition::new(
            "1_5_DI_2".to_string(),
            "ESDV6101_Z0".to_string(),
            "计量撬进口气动阀全开".to_string(),
            "樟洋电厂".to_string(),
            "16通道数字量输入模�?.to_string(),
            ModuleType::DI,
            "1_5_DI_2".to_string(),
            PointDataType::Bool,
            "00003".to_string(),
        ),
        ChannelPointDefinition::new(
            "1_5_DI_3".to_string(),
            "ESDV6101_ZC".to_string(),
            "计量撬进口气动阀全关".to_string(),
            "樟洋电厂".to_string(),
            "16通道数字量输入模�?.to_string(),
            ModuleType::DI,
            "1_5_DI_3".to_string(),
            PointDataType::Bool,
            "00004".to_string(),
        ),

        // DO通道 (1-4)
        ChannelPointDefinition::new(
            "1_7_DO_0".to_string(),
            "DO_1_CL_1".to_string(),
            "设备1�?.to_string(),
            "樟洋电厂".to_string(),
            "16通道数字量输出模�?.to_string(),
            ModuleType::DO,
            "1_7_DO_0".to_string(),
            PointDataType::Bool,
            "00033".to_string(),
        ),
        ChannelPointDefinition::new(
            "1_7_DO_1".to_string(),
            "DO_2_OP_1".to_string(),
            "设备1开".to_string(),
            "樟洋电厂".to_string(),
            "16通道数字量输出模�?.to_string(),
            ModuleType::DO,
            "1_7_DO_1".to_string(),
            PointDataType::Bool,
            "00034".to_string(),
        ),
        ChannelPointDefinition::new(
            "1_7_DO_2".to_string(),
            "SQ6103_S0".to_string(),
            "A路计量出口气动阀开指令".to_string(),
            "樟洋电厂".to_string(),
            "16通道数字量输出模�?.to_string(),
            ModuleType::DO,
            "1_7_DO_2".to_string(),
            PointDataType::Bool,
            "00035".to_string(),
        ),
        ChannelPointDefinition::new(
            "1_7_DO_3".to_string(),
            "SQ6103_SC".to_string(),
            "A路计量出口气动阀关指�?.to_string(),
            "樟洋电厂".to_string(),
            "16通道数字量输出模�?.to_string(),
            ModuleType::DO,
            "1_7_DO_3".to_string(),
            PointDataType::Bool,
            "00036".to_string(),
        ),
    ]
}

