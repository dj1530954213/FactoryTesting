/// 验证数据库中的硬点测试结果存储
/// 检查channel_test_instances表中的硬点状态和百分比字段

use sea_orm::{Database, EntityTrait, QueryFilter, ColumnTrait};
use app_lib::models::entities::{channel_test_instance, raw_test_outcome};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 连接到数据库
    let db = Database::connect("sqlite://./factory_testing_data.sqlite").await?;
    
    println!("🔍 查询数据库中的硬点测试结果...");
    
    // 查询所有测试实例
    let instances = channel_test_instance::Entity::find()
        .filter(channel_test_instance::Column::HardPointStatus.is_not_null())
        .all(&db)
        .await?;
    
    println!("📊 找到 {} 个包含硬点测试结果的实例", instances.len());
    
    for instance in instances {
        println!("\n🔍 测试实例: {}", instance.instance_id);
        println!("   📋 定义ID: {}", instance.definition_id);
        println!("   📦 批次ID: {}", instance.test_batch_id);
        
        // 硬点测试状态
        if let Some(status) = instance.hard_point_status {
            let status_text = match status {
                0 => "未测试",
                1 => "通过",
                2 => "失败",
                3 => "不适用",
                4 => "测试中",
                5 => "跳过",
                _ => "未知状态",
            };
            println!("   🎯 硬点测试状态: {} ({})", status, status_text);
        }
        
        // 硬点测试结果
        if let Some(result) = &instance.hard_point_test_result {
            println!("   ✅ 硬点测试结果: {}", result);
        }
        
        // 硬点测试错误详情
        if let Some(error_detail) = &instance.hard_point_error_detail {
            println!("   ❌ 硬点测试错误: {}", error_detail);
        }
        
        // 实际值和期望值
        if let Some(actual) = &instance.actual_value {
            println!("   📊 实际值: {}", actual);
        }
        if let Some(expected) = &instance.expected_value {
            println!("   🎯 期望值: {}", expected);
        }
        
        // 百分比测试结果
        println!("   📈 百分比测试结果:");
        if let Some(val) = instance.test_result_0_percent {
            println!("      0%: {}", val);
        }
        if let Some(val) = instance.test_result_25_percent {
            println!("      25%: {}", val);
        }
        if let Some(val) = instance.test_result_50_percent {
            println!("      50%: {}", val);
        }
        if let Some(val) = instance.test_result_75_percent {
            println!("      75%: {}", val);
        }
        if let Some(val) = instance.test_result_100_percent {
            println!("      100%: {}", val);
        }
        
        // 临时数据JSON
        if let Some(transient_data) = &instance.transient_data_json {
            if !transient_data.is_empty() && transient_data != "{}" {
                println!("   💾 临时数据: {}", transient_data);
            }
        }
        
        // 子测试结果JSON
        if let Some(sub_results) = &instance.sub_test_results_json {
            if !sub_results.is_empty() && sub_results != "{}" {
                println!("   🧪 子测试结果: {}", sub_results);
            }
        }
    }
    
    // 查询原始测试结果表
    println!("\n🔍 查询原始测试结果表...");

    let outcomes = raw_test_outcome::Entity::find()
        .filter(raw_test_outcome::Column::SubTestItem.eq("HardPoint"))
        .all(&db)
        .await?;
    
    println!("📊 找到 {} 个硬点测试原始结果", outcomes.len());
    
    for outcome in outcomes {
        println!("\n🔍 原始测试结果: {}", outcome.id);
        println!("   📋 通道实例ID: {}", outcome.channel_instance_id);
        println!("   🎯 测试项: {}", outcome.sub_test_item);
        println!("   ✅ 成功: {}", outcome.success);
        
        if let Some(message) = &outcome.message {
            println!("   💬 消息: {}", message);
        }
        
        // 百分比测试结果
        println!("   📈 百分比测试结果:");
        if let Some(val) = outcome.test_result_0_percent {
            println!("      0%: {}", val);
        }
        if let Some(val) = outcome.test_result_25_percent {
            println!("      25%: {}", val);
        }
        if let Some(val) = outcome.test_result_50_percent {
            println!("      50%: {}", val);
        }
        if let Some(val) = outcome.test_result_75_percent {
            println!("      75%: {}", val);
        }
        if let Some(val) = outcome.test_result_100_percent {
            println!("      100%: {}", val);
        }
        
        // 读数JSON
        if let Some(readings) = &outcome.readings_json {
            if !readings.is_empty() && readings != "null" {
                println!("   📊 读数数据: {}", readings);
            }
        }
    }
    
    println!("\n🎉 数据库查询完成！");
    
    Ok(())
}
