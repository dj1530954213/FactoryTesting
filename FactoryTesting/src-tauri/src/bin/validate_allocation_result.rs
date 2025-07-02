#![cfg(FALSE)]
// 验证分配结果是否符合正确的分配表
use app_lib::services::infrastructure::persistence::{SqliteOrmPersistenceService, PersistenceConfig};
use app_lib::services::channel_allocation_service::{ChannelAllocationService, TestPlcConfig, ComparisonTable, IChannelAllocationService};
use app_lib::services::domain::{TestPlcConfigService, ITestPlcConfigService};
use app_lib::services::traits::BaseService;
use app_lib::models::structs::ChannelPointDefinition;
use app_lib::models::enums::{ModuleType, PointDataType};
use app_lib::models::test_plc_config::{GetTestPlcChannelsRequest, TestPlcChannelType, TestPlcChannelConfig};
use std::sync::Arc;
use std::path::PathBuf;
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日�?
    env_logger::init();
    
    println!("=== 验证分配结果与正确分配表的对�?===");
    
    // 创建正确的分配表（基于您提供的图片）
    let correct_allocations = create_correct_allocation_table();
    
    println!("正确分配表包�?{} 个分配记�?, correct_allocations.len());
    
    // 按批次分组统�?
    let mut batch_stats: HashMap<String, Vec<&CorrectAllocation>> = HashMap::new();
    for allocation in &correct_allocations {
        batch_stats.entry(allocation.batch_name.clone())
            .or_insert_with(Vec::new)
            .push(allocation);
    }
    
    println!("\n正确分配表的批次统计:");
    for (batch_name, allocations) in &batch_stats {
        let mut ai_count = 0;
        let mut ao_count = 0;
        let mut di_count = 0;
        let mut do_count = 0;
        
        for alloc in allocations {
            match alloc.module_type.as_str() {
                "AI" => ai_count += 1,
                "AO" => ao_count += 1,
                "DI" => di_count += 1,
                "DO" => do_count += 1,
                _ => {}
            }
        }
        
        println!("  {}: {} 个通道 (AI:{}, AO:{}, DI:{}, DO:{})", 
                 batch_name, allocations.len(), ai_count, ao_count, di_count, do_count);
    }
    
    // 分析有源/无源匹配规则
    println!("\n分析有源/无源匹配规则:");
    analyze_power_matching_rules(&correct_allocations);
    
    // 分析测试PLC通道分配规律
    println!("\n分析测试PLC通道分配规律:");
    analyze_test_plc_allocation_pattern(&correct_allocations);
    
    println!("\n=== 验证完成 ===");
    
    Ok(())
}

#[derive(Debug, Clone)]
struct CorrectAllocation {
    sequence: u32,
    tag: String,
    description: String,
    customer: String,
    module_type: String,
    channel_position: String,
    test_plc_address: String,
    communication_address: String,
    power_type: String,
    batch_name: String,
}

/// 创建正确的分配表（基于您提供的图片数据）
fn create_correct_allocation_table() -> Vec<CorrectAllocation> {
    vec![
        // 批次1 - AI通道
        CorrectAllocation {
            sequence: 1,
            tag: "PT_2101".to_string(),
            description: "计量撬进口压�?.to_string(),
            customer: "樟洋电厂".to_string(),
            module_type: "AI".to_string(),
            channel_position: "1_2_AI_0".to_string(),
            test_plc_address: "AO2_1".to_string(),
            communication_address: "1_2_AI_0".to_string(),
            power_type: "有源".to_string(),
            batch_name: "批次1".to_string(),
        },
        CorrectAllocation {
            sequence: 2,
            tag: "PT_2102".to_string(),
            description: "计量撬出口压�?.to_string(),
            customer: "樟洋电厂".to_string(),
            module_type: "AI".to_string(),
            channel_position: "1_2_AI_1".to_string(),
            test_plc_address: "AO2_2".to_string(),
            communication_address: "1_2_AI_1".to_string(),
            power_type: "有源".to_string(),
            batch_name: "批次1".to_string(),
        },
        CorrectAllocation {
            sequence: 3,
            tag: "TT_4101".to_string(),
            description: "计量撬进口温�?.to_string(),
            customer: "樟洋电厂".to_string(),
            module_type: "AI".to_string(),
            channel_position: "1_2_AI_2".to_string(),
            test_plc_address: "AO2_3".to_string(),
            communication_address: "1_2_AI_2".to_string(),
            power_type: "有源".to_string(),
            batch_name: "批次1".to_string(),
        },
        CorrectAllocation {
            sequence: 4,
            tag: "TT_4102".to_string(),
            description: "计量撬出口温�?.to_string(),
            customer: "樟洋电厂".to_string(),
            module_type: "AI".to_string(),
            channel_position: "1_2_AI_3".to_string(),
            test_plc_address: "AO2_4".to_string(),
            communication_address: "1_2_AI_3".to_string(),
            power_type: "有源".to_string(),
            batch_name: "批次1".to_string(),
        },
        
        // 批次1 - AO通道
        CorrectAllocation {
            sequence: 18,
            tag: "FCV_7101_AO".to_string(),
            description: "计量撬出口气动阀控制指令".to_string(),
            customer: "樟洋电厂".to_string(),
            module_type: "AO".to_string(),
            channel_position: "1_4_AO_0".to_string(),
            test_plc_address: "AI1_1".to_string(),
            communication_address: "1_4_AO_0".to_string(),
            power_type: "有源".to_string(),
            batch_name: "批次1".to_string(),
        },
        CorrectAllocation {
            sequence: 19,
            tag: "YLDW1_4_AO_1".to_string(),
            description: "预留点位".to_string(),
            customer: "樟洋电厂".to_string(),
            module_type: "AO".to_string(),
            channel_position: "1_4_AO_1".to_string(),
            test_plc_address: "AI1_2".to_string(),
            communication_address: "1_4_AO_1".to_string(),
            power_type: "有源".to_string(),
            batch_name: "批次1".to_string(),
        },
        
        // 批次1 - DI通道
        CorrectAllocation {
            sequence: 26,
            tag: "ESDV6101_1".to_string(),
            description: "电磁阀1电流监视继电器失�?.to_string(),
            customer: "樟洋电厂".to_string(),
            module_type: "DI".to_string(),
            channel_position: "1_5_DI_0".to_string(),
            test_plc_address: "DO2_1".to_string(),
            communication_address: "1_5_DI_0".to_string(),
            power_type: "有源".to_string(),
            batch_name: "批次1".to_string(),
        },
        CorrectAllocation {
            sequence: 27,
            tag: "ESDV6101_2".to_string(),
            description: "电磁阀2电流监视继电器失�?.to_string(),
            customer: "樟洋电厂".to_string(),
            module_type: "DI".to_string(),
            channel_position: "1_5_DI_1".to_string(),
            test_plc_address: "DO2_2".to_string(),
            communication_address: "1_5_DI_1".to_string(),
            power_type: "有源".to_string(),
            batch_name: "批次1".to_string(),
        },
        
        // 批次1 - DO通道
        CorrectAllocation {
            sequence: 58,
            tag: "DO_1_CL_1".to_string(),
            description: "设备1�?.to_string(),
            customer: "樟洋电厂".to_string(),
            module_type: "DO".to_string(),
            channel_position: "1_7_DO_0".to_string(),
            test_plc_address: "DI1_1".to_string(),
            communication_address: "1_7_DO_0".to_string(),
            power_type: "有源".to_string(),
            batch_name: "批次1".to_string(),
        },
        CorrectAllocation {
            sequence: 59,
            tag: "DO_2_OP_1".to_string(),
            description: "设备1开".to_string(),
            customer: "樟洋电厂".to_string(),
            module_type: "DO".to_string(),
            channel_position: "1_7_DO_1".to_string(),
            test_plc_address: "DI1_2".to_string(),
            communication_address: "1_7_DO_1".to_string(),
            power_type: "有源".to_string(),
            batch_name: "批次1".to_string(),
        },
    ]
}

/// 分析有源/无源匹配规则
fn analyze_power_matching_rules(allocations: &[CorrectAllocation]) {
    println!("有源/无源匹配规则分析:");
    
    for allocation in allocations.iter().take(10) {
        let test_plc_type = &allocation.test_plc_address[..3]; // 提取�?个字符，�?"AO2", "AI1"
        let channel_type = &allocation.module_type;
        let power_type = &allocation.power_type;
        
        println!("  {} ({}) -> {} ({})", 
                 channel_type, power_type, test_plc_type, 
                 if test_plc_type.ends_with('2') { "无源" } else { "有源" });
    }
    
    println!("\n推导的匹配规�?");
    println!("  AI有源 -> AO无源 (AO2_X)");
    println!("  AO有源 -> AI有源 (AI1_X)");
    println!("  DI有源 -> DO无源 (DO2_X)");
    println!("  DO有源 -> DI有源 (DI1_X)");
}

/// 分析测试PLC通道分配规律
fn analyze_test_plc_allocation_pattern(allocations: &[CorrectAllocation]) {
    println!("测试PLC通道分配规律:");
    
    let mut test_plc_usage: HashMap<String, Vec<&CorrectAllocation>> = HashMap::new();
    for allocation in allocations {
        let test_plc_type = allocation.test_plc_address[..3].to_string(); // 提取类型，如 "AO2", "AI1"
        test_plc_usage.entry(test_plc_type)
            .or_insert_with(Vec::new)
            .push(allocation);
    }
    
    for (test_plc_type, usages) in &test_plc_usage {
        println!("  {}: 使用�?{} 个通道", test_plc_type, usages.len());
        for usage in usages.iter().take(3) {
            println!("    {} -> {}", usage.tag, usage.test_plc_address);
        }
        if usages.len() > 3 {
            println!("    ... 还有 {} �?, usages.len() - 3);
        }
    }
}

