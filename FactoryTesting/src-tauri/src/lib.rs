/// FAT_TEST 工厂测试系统 - Rust后端核心库
pub mod models;
pub mod utils;
pub mod services;
pub mod tauri_commands;
pub mod commands;

// 重新导出常用类型，方便使用
pub use models::*;
pub use utils::{AppError, AppResult, AppConfig};
pub use services::*;
pub use tauri_commands::{AppState, SystemStatus, init_app_state};

// 导入新的命令函数
use commands::data_management::{
    parse_excel_file, create_test_batch, get_batch_list, get_batch_channel_definitions
};

/// 应用程序主要运行函数
/// 
/// 这个函数现在会启动 Tauri 应用程序
pub fn run() {
    // 使用 tokio 运行时启动 Tauri 应用
    tauri::async_runtime::block_on(async {
        // 初始化应用状态
        let app_state = match init_app_state().await {
            Ok(state) => state,
            Err(e) => {
                eprintln!("初始化应用状态失败: {}", e);
                std::process::exit(1);
            }
        };

        // 启动 Tauri 应用
        tauri::Builder::default()
            .manage(app_state)
            .invoke_handler(tauri::generate_handler![
                tauri_commands::submit_test_execution,
                tauri_commands::start_batch_testing,
                tauri_commands::pause_batch_testing,
                tauri_commands::resume_batch_testing,
                tauri_commands::stop_batch_testing,
                tauri_commands::get_batch_progress,
                tauri_commands::get_batch_results,
                tauri_commands::cleanup_completed_batch,
                tauri_commands::get_all_channel_definitions,
                tauri_commands::save_channel_definition,
                tauri_commands::delete_channel_definition,
                tauri_commands::get_all_batch_info,
                tauri_commands::save_batch_info,
                tauri_commands::get_batch_test_instances,
                tauri_commands::create_test_instance,
                tauri_commands::get_instance_state,
                tauri_commands::update_test_result,
                tauri_commands::get_system_status,
                parse_excel_file,
                create_test_batch,
                get_batch_list,
                get_batch_channel_definitions
            ])
            .run(tauri::generate_context!())
            .expect("启动 Tauri 应用失败");
    });
}

#[cfg(debug_assertions)]
fn run_example() {
    println!("=== FAT_TEST 核心数据模型示例 ===");
    
    // 导入我们的模型
    use crate::models::*;
    
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
    
    // 3. 创建测试实例
    println!("\n3. 创建测试实例:");
    let test_instance = ChannelTestInstance::new(
        definition.id.clone(),
        batch.batch_id.clone(),
    );
    println!("   实例ID: {}", test_instance.instance_id);
    println!("   定义ID: {}", test_instance.definition_id);
    println!("   整体状态: {:?}", test_instance.overall_status);
    
    // 4. 演示错误处理
    println!("\n4. 错误处理示例:");
    let error = AppError::plc_communication_error("连接超时");
    println!("   错误代码: {}", error.error_code());
    println!("   错误信息: {}", error);
    
    // 5. 序列化示例
    println!("\n5. JSON序列化示例:");
    match serde_json::to_string_pretty(&definition) {
        Ok(json) => {
            println!("   通道定义JSON（前200字符）: {}", 
                     &json.chars().take(200).collect::<String>());
        }
        Err(e) => println!("   序列化失败: {}", e),
    }
    
    println!("\n=== 示例运行完成 ===");
}
