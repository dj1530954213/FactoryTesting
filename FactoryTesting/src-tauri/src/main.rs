// 防止在 Windows 发布版本中显示额外的控制台窗口，请勿删除！
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

// 应用程序主入口函数
fn main() {
    // 在调试模式下运行示例
    #[cfg(debug_assertions)]
    run_example();
    
    // 启动Tauri应用
    // 在 cargo check 阶段，我们不实际运行Tauri应用，
    // 而是依赖 app_lib::run() 中的打印语句（如果有）或其能被编译。
    // 如果 app_lib::run() 做了很多事，可能会减慢检查速度或有其他副作用。
    // 对于纯粹的库检查，这行可能不是必需的，但对于Tauri项目结构，main.rs通常会调用它。
    app_lib::run(); // 保留，因为它在Tauri模板中常见
}

#[cfg(debug_assertions)]
fn run_example() {
    println!("=== FAT_TEST 核心数据模型示例 ===");
    
    // 导入我们的模型
    // 注意：现在 app_lib 重新导出了 models::*，所以可以直接用
    use app_lib::models::{ChannelPointDefinition, ModuleType, PointDataType, TestBatchInfo, ChannelTestInstance, SubTestItem, SubTestStatus, SubTestExecutionResult, RawTestOutcome, AnalogReadingPoint};
    
    // 1. 创建通道点位定义示例
    println!("\n1. 创建通道点位定义:");
    let definition = ChannelPointDefinition::new(
        "AI001".to_string(),
        "Temperature_1".to_string(),
        "反应器温度".to_string(),
        "Station1".to_string(),
        "Module1".to_string(),
        ModuleType::AI,
        "CH01".to_string(),
        PointDataType::Float,
        "DB1.DBD0".to_string(),
    );
    println!("   位号: {}", definition.tag);
    println!("   变量名: {}", definition.variable_name);
    println!("   模块类型: {:?}", definition.module_type);
    println!("   PLC地址: {}", definition.plc_communication_address);
    
    // 2. 创建测试批次信息
    println!("\n2. 创建测试批次信息:");
    let mut batch = TestBatchInfo::new(
        Some("ProductV1.0".to_string()),
        Some("SN123456".to_string()),
    );
    batch.total_points = 120;
    batch.operator_name = Some("张三".to_string());
    println!("   批次ID: {}", batch.batch_id);
    println!("   产品型号: {:?}", batch.product_model);
    println!("   序列号: {:?}", batch.serial_number);
    println!("   总点数: {}", batch.total_points);
    
    // 3. 创建通道测试实例
    println!("\n3. 创建通道测试实例:");
    let mut instance = ChannelTestInstance::new(
        definition.id.clone(),
        batch.batch_id.clone(),
    );
    
    // 添加一些子测试结果
    instance.sub_test_results.insert(
        SubTestItem::HardPoint,
        // Corrected: Use new() method
        SubTestExecutionResult::new(SubTestStatus::Passed, None, None, None),
    );
    instance.sub_test_results.insert(
        SubTestItem::LowAlarm,
        // Corrected: Use new() method
        SubTestExecutionResult::new(SubTestStatus::Failed, Some("报警值设置失败".to_string()), None, None),
    );
    
    println!("   实例ID: {}", instance.instance_id);
    println!("   整体状态: {:?}", instance.overall_status);
    println!("   子测试结果数量: {}", instance.sub_test_results.len());
    
    // 4. 创建原始测试结果
    println!("\n4. 创建原始测试结果:");
    // Corrected: RawTestOutcome::success and ::failure were removed. Use ::new directly.
    let mut outcome = RawTestOutcome::new(
        instance.instance_id.clone(),
        SubTestItem::HardPoint,
        true, // success status
    );
    outcome.raw_value_read = Some("16384".to_string());
    outcome.eng_value_calculated = Some("20.0".to_string());
    
    // 添加一些模拟量读数点
    // Corrected: Initialize AnalogReadingPoint fields directly and handle Option for readings
    let readings_vec = outcome.readings.get_or_insert_with(Vec::new);
    readings_vec.push(AnalogReadingPoint {
        set_percentage: 0.0,
        set_value_eng: 0.0,
        actual_reading_raw: Some(0.0),
        actual_reading_eng: Some(0.0),
        status: SubTestStatus::Passed,
        ..Default::default()
    });
    readings_vec.push(AnalogReadingPoint {
        set_percentage: 0.25,
        set_value_eng: 5.0,
        actual_reading_raw: Some(8192.0),
        actual_reading_eng: Some(4.98),
        status: SubTestStatus::Passed,
        ..Default::default()
    });
    readings_vec.push(AnalogReadingPoint {
        set_percentage: 1.0,
        set_value_eng: 20.0,
        actual_reading_raw: Some(32767.0),
        actual_reading_eng: Some(19.95),
        status: SubTestStatus::Passed,
        ..Default::default()
    });
    
    println!("   测试成功: {}", outcome.success);
    println!("   子测试项: {:?}", outcome.sub_test_item);
    // Corrected: Handle Option for len()
    println!("   读数点数量: {}", outcome.readings.as_ref().map_or(0, |v| v.len()));
    
    // 5. 演示序列化功能
    println!("\n5. 演示JSON序列化:");
    match serde_json::to_string_pretty(&instance) {
        Ok(json) => {
            println!("   通道测试实例JSON长度: {} 字符", json.len());
            // 只显示前200个字符作为示例
            let preview = if json.len() > 200 {
                format!("{}...", &json[..200])
            } else {
                json
            };
            println!("   JSON预览: {}", preview);
        }
        Err(e) => println!("   序列化失败: {}", e),
    }
    
    println!("\n=== 示例运行完成 ===\n");
}
