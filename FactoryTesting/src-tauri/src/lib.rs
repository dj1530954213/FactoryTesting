/// FAT_TEST 工厂测试系统 - Rust后端核心库
pub mod models;
pub mod utils;
pub mod services;
pub mod tauri_commands;
pub mod commands;
pub mod error;
pub mod database_migration;
pub mod domain;
pub mod infrastructure;
pub mod monitoring;
pub mod logging;

// 重新导出常用类型，方便使用
pub use models::*;
pub use utils::{AppConfig};
pub use services::*;
pub use tauri_commands::{AppState, SystemStatus, init_app_state};
pub use database_migration::DatabaseMigration;

// 导入新的命令函数
use commands::data_management::{
    parse_excel_file, create_test_batch, get_batch_list, get_dashboard_batch_list, get_batch_channel_definitions,
    import_excel_and_prepare_batch_cmd, start_tests_for_batch_cmd, get_batch_status_cmd,
    parse_excel_and_create_batch_cmd, prepare_test_instances_for_batch_cmd,
    import_excel_and_allocate_channels_cmd, clear_session_data,
    parse_excel_without_persistence_cmd, create_batch_and_persist_data_cmd,
    import_excel_and_create_batch_cmd, create_test_batch_with_definitions_cmd, delete_batch_cmd
};
use commands::manual_testing::{
    execute_manual_sub_test_cmd, read_channel_value_cmd, write_channel_value_cmd,
    connect_plc_cmd, start_batch_auto_test_cmd, get_plc_connection_status_cmd
};
use commands::manual_test_commands::{
    start_manual_test_cmd, update_manual_test_subitem_cmd, get_manual_test_status_cmd,
    start_plc_monitoring_cmd, stop_plc_monitoring_cmd,
    // AI手动测试专用命令
    generate_random_display_value_cmd, ai_show_value_test_cmd, ai_alarm_test_cmd,
    ai_maintenance_test_cmd, ai_reset_to_display_value_cmd, complete_manual_test_subitem_cmd,
    // DI手动测试命令
    di_signal_test_cmd
};
use commands::test_plc_config::{
    get_test_plc_channels_cmd, save_test_plc_channel_cmd, delete_test_plc_channel_cmd,
    get_plc_connections_cmd, save_plc_connection_cmd, test_plc_connection_cmd, test_temp_plc_connection_cmd,
    test_address_read_cmd, get_channel_mappings_cmd, generate_channel_mappings_cmd, 
    initialize_default_test_plc_channels_cmd, restore_default_test_plc_channels_cmd
};

/// 应用程序主要运行函数
///
/// 这个函数现在会启动 Tauri 应用程序
pub fn run() {
    // 初始化日志系统 - 同时输出到控制台和文件
    use std::fs::OpenOptions;
    use std::io::{Write, BufWriter};
    use std::sync::{Arc, Mutex};

    // 创建日志文件
    let log_file = OpenOptions::new()
        .create(true)
        .append(true)
        .open("fat_test_debug.log")
        .expect("无法创建日志文件");

    let file_writer = Arc::new(Mutex::new(BufWriter::new(log_file)));

    // 使用简单的控制台日志，同时写入文件
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Debug) // 应用默认级别
        .filter_module("sqlx", log::LevelFilter::Warn) // 过滤sqlx查询日志
        .filter_module("sea_orm", log::LevelFilter::Warn) // 过滤sea_orm查询日志
        .filter_module("sea_orm_migration", log::LevelFilter::Warn) // 过滤迁移日志
        .filter_module("sqlx::query", log::LevelFilter::Off) // 完全关闭sqlx查询日志
        .filter_module("tokio_modbus", log::LevelFilter::Warn) // 过滤tokio_modbus的DEBUG日志
        .filter_module("app::services", log::LevelFilter::Debug) // 确保业务服务日志显示
        .filter_module("app::tauri_commands", log::LevelFilter::Debug) // 确保命令日志显示
        .filter_module("app::models", log::LevelFilter::Debug) // 确保模型转换日志显示
        .format_timestamp_secs() // 添加时间戳
        .format_module_path(false) // 简化模块路径显示
        .format(move |buf, record| {
            use std::io::Write;
            let formatted = format!("[{}] [{}] {}\n",
                chrono::Utc::now().format("%Y-%m-%d %H:%M:%S"),
                record.level(),
                record.args()
            );

            // 写入控制台
            write!(buf, "{}", formatted)?;

            // 同时写入文件
            if let Ok(mut writer) = file_writer.lock() {
                let _ = writer.write_all(formatted.as_bytes());
                let _ = writer.flush();
            }

            Ok(())
        })
        .init();

    log::info!("=== FAT_TEST 工厂测试系统启动 ===");
    log::info!("日志系统已初始化，级别: DEBUG (数据库查询日志已过滤)");

    // 使用 tokio 运行时启动 Tauri 应用
    tauri::async_runtime::block_on(async {
        // 初始化应用状态
        log::info!("开始初始化应用状态...");
        let app_state = match init_app_state().await {
            Ok(state) => {
                log::info!("应用状态初始化成功");
                state
            },
            Err(e) => {
                log::error!("初始化应用状态失败: {}", e);
                eprintln!("初始化应用状态失败: {}", e);
                std::process::exit(1);
            }
        };

        // 启动 Tauri 应用
        tauri::Builder::default()
            .plugin(tauri_plugin_dialog::init())
            .manage(app_state)
            .setup(|app| {
                // 在应用启动后设置AppHandle到事件发布器
                let app_handle = app.handle();

                // 设置全局AppHandle，用于事件发布
                crate::services::infrastructure::event_publisher::set_global_app_handle(app_handle.clone());

                log::info!("Tauri应用启动完成，AppHandle已设置到事件发布器");

                Ok(())
            })
            .invoke_handler(tauri::generate_handler![
                // 测试协调相关命令
                tauri_commands::submit_test_execution,
                tauri_commands::start_batch_testing,
                tauri_commands::pause_batch_testing,
                tauri_commands::resume_batch_testing,
                tauri_commands::stop_batch_testing,
                tauri_commands::get_batch_progress,
                tauri_commands::get_batch_results,
                tauri_commands::get_session_batches,
                tauri_commands::cleanup_completed_batch,
                tauri_commands::start_single_channel_test,
                tauri_commands::create_test_data,
                // 数据管理相关命令
                tauri_commands::import_excel_file,
                // tauri_commands::create_test_batch_with_definitions, // 注释掉，使用data_management中的版本
                tauri_commands::get_all_channel_definitions,
                tauri_commands::save_channel_definition,
                tauri_commands::delete_channel_definition,
                tauri_commands::get_all_batch_info,
                tauri_commands::save_batch_info,
                tauri_commands::get_batch_test_instances,
                // 通道状态管理相关命令
                tauri_commands::create_test_instance,
                tauri_commands::get_instance_state,
                tauri_commands::update_test_result,
                // 系统信息相关命令
                tauri_commands::get_system_status,
                // 报告生成相关命令
                tauri_commands::generate_pdf_report,
                tauri_commands::generate_excel_report,
                tauri_commands::get_reports,
                tauri_commands::get_report_templates,
                tauri_commands::create_report_template,
                tauri_commands::update_report_template,
                tauri_commands::delete_report_template,
                tauri_commands::delete_report,
                // 应用配置相关命令
                tauri_commands::load_app_settings_cmd,
                tauri_commands::save_app_settings_cmd,
                // 数据管理命令
                parse_excel_file,
                create_test_batch,
                get_batch_list,
                get_dashboard_batch_list,
                get_batch_channel_definitions,
                parse_excel_and_create_batch_cmd,
                import_excel_and_prepare_batch_cmd,
                start_tests_for_batch_cmd,
                get_batch_status_cmd,
                prepare_test_instances_for_batch_cmd,
                import_excel_and_allocate_channels_cmd,
                clear_session_data,
                parse_excel_without_persistence_cmd,
                create_batch_and_persist_data_cmd,
                import_excel_and_create_batch_cmd,
                create_test_batch_with_definitions_cmd,
                delete_batch_cmd,
                // 手动测试命令
                execute_manual_sub_test_cmd,
                read_channel_value_cmd,
                write_channel_value_cmd,
                connect_plc_cmd,
                start_batch_auto_test_cmd,
                get_plc_connection_status_cmd,
                // 新的手动测试命令
                start_manual_test_cmd,
                update_manual_test_subitem_cmd,
                get_manual_test_status_cmd,
                start_plc_monitoring_cmd,
                stop_plc_monitoring_cmd,
                // AI手动测试专用命令
                generate_random_display_value_cmd,
                ai_show_value_test_cmd,
                ai_alarm_test_cmd,
                ai_maintenance_test_cmd,
                ai_reset_to_display_value_cmd,
                complete_manual_test_subitem_cmd,
                di_signal_test_cmd,
                // 测试PLC配置命令
                get_test_plc_channels_cmd,
                save_test_plc_channel_cmd,
                delete_test_plc_channel_cmd,
                get_plc_connections_cmd,
                save_plc_connection_cmd,
                test_plc_connection_cmd,
                test_temp_plc_connection_cmd,
                test_address_read_cmd,
                get_channel_mappings_cmd,
                generate_channel_mappings_cmd,
                initialize_default_test_plc_channels_cmd,
                restore_default_test_plc_channels_cmd
            ])
            .run(tauri::generate_context!())
            .expect("启动 Tauri 应用失败");
    });
}


//TODD:需要删除
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
    let error = error::AppError::PlcCommunicationError { message: "连接超时".to_string() };
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
