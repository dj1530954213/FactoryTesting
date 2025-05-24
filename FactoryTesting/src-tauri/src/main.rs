// 防止在 Windows 发布版本中显示额外的控制台窗口，请勿删除！
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

// 应用程序主入口函数
fn main() {
    // 在调试模式下运行示例
    #[cfg(debug_assertions)]
    run_example();
    
    // 启动Tauri应用
    app_lib::run()
}

#[cfg(debug_assertions)]
fn run_example() {
    println!("=== FAT_TEST 核心数据模型示例 ===");
    
    // 导入我们的模型
    use app_lib::models::*;
    
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
        SubTestExecutionResult::passed(),
    );
    instance.sub_test_results.insert(
        SubTestItem::LowAlarm,
        SubTestExecutionResult::failed("报警值设置失败".to_string()),
    );
    
    println!("   实例ID: {}", instance.instance_id);
    println!("   整体状态: {:?}", instance.overall_status);
    println!("   子测试结果数量: {}", instance.sub_test_results.len());
    
    // 4. 创建原始测试结果
    println!("\n4. 创建原始测试结果:");
    let mut outcome = RawTestOutcome::success(
        instance.instance_id.clone(),
        SubTestItem::HardPoint,
    );
    
    // 添加一些模拟量读数点
    outcome.readings.push(AnalogReadingPoint::new(0.0, 0.0));
    outcome.readings.push(AnalogReadingPoint::new(0.25, 5.0));
    outcome.readings.push(AnalogReadingPoint::new(1.0, 20.0));
    
    println!("   测试成功: {}", outcome.success);
    println!("   子测试项: {:?}", outcome.sub_test_item);
    println!("   读数点数量: {}", outcome.readings.len());
    
    // 5. 演示序列化功能
    println!("\n5. 演示JSON序列化:");
    match serde_json::to_string_pretty(&instance) {
        Ok(json) => {
            println!("   通道测试实例JSON长度: {} 字符", json.len());
            // 只显示前100个字符作为示例
            let preview = if json.len() > 100 {
                format!("{}...", &json[..100])
            } else {
                json
            };
            println!("   JSON预览: {}", preview);
        }
        Err(e) => println!("   序列化失败: {}", e),
    }
    
    println!("\n=== 示例运行完成 ===\n");
}
