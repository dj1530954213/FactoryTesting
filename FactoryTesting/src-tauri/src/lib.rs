/// FAT_TEST 工厂测试系统 - Rust后端核心库
/// 
/// 业务说明：
/// 本文件是整个后端系统的核心入口，采用DDD（领域驱动设计）架构
/// 主要功能包括：PLC通信、批次管理、测试执行、数据导入导出、报告生成等
/// 
/// 架构分层：
/// - models: 数据模型层 - 定义系统中的所有数据结构
/// - domain: 领域层 - 包含核心业务逻辑和领域服务
/// - application: 应用层 - 编排领域服务，实现用例
/// - infrastructure: 基础设施层 - 技术实现细节（数据库、PLC通信等）
/// - interfaces: 接口层 - 对外暴露的API（Tauri命令）
/// - utils: 工具层 - 通用工具函数
pub mod models;//数据模型层
pub mod utils;//工具类层
pub mod tauri_commands;//Tauri命令器
pub mod commands;//命令处理器
pub mod error;//错误处理器
pub mod database_migration;//数据迁移器
pub mod domain;//领域层
pub mod infrastructure;//基础设施层
pub mod monitoring;//监控器
pub mod logging;//日志层
pub mod application;//应用层
pub mod interfaces;//接口适配层

// 重新导出常用类型，方便使用
// Rust知识点：pub use 可以将子模块的内容重新导出到当前模块
// 这样外部使用时可以直接 use app_lib::AppState 而不需要 use app_lib::tauri_commands::AppState
pub use models::*;
pub use utils::{AppConfig};
pub use tauri_commands::{AppState, SystemStatus, init_app_state};
pub use database_migration::DatabaseMigration;

// 导入新的命令函数
// 业务说明：这些命令是前端通过Tauri invoke调用的入口点
// 调用链：前端 -> invoke -> 这些命令 -> 应用层服务 -> 领域层 -> 基础设施层

// 数据管理相关命令 - 处理Excel导入、批次创建、测试数据管理等
use commands::data_management::{
    parse_excel_file, create_test_batch, get_batch_list, get_dashboard_batch_list, get_batch_channel_definitions,
    import_excel_and_prepare_batch_cmd, start_tests_for_batch_cmd, get_batch_status_cmd,
    parse_excel_and_create_batch_cmd, prepare_test_instances_for_batch_cmd,
    import_excel_and_allocate_channels_cmd, clear_session_data,
    parse_excel_without_persistence_cmd, create_batch_and_persist_data_cmd,
    import_excel_and_create_batch_cmd, create_test_batch_with_definitions_cmd, delete_batch_cmd,
    restore_session_cmd
};

// 手动测试相关命令 - 处理手动测试执行、PLC读写、连接管理等
use commands::manual_testing::{
    execute_manual_sub_test_cmd, read_channel_value_cmd, write_channel_value_cmd,
    connect_plc_cmd, start_batch_auto_test_cmd, get_plc_connection_status_cmd
};

// 手动测试命令 - 新版手动测试流程，包含AI/AO/DI/DO等测试类型
use commands::manual_test_commands::{
    start_manual_test_cmd, update_manual_test_subitem_cmd, get_manual_test_status_cmd,
    start_plc_monitoring_cmd, stop_plc_monitoring_cmd,
    // AI手动测试专用命令 - 模拟量输入测试
    generate_random_display_value_cmd, ai_show_value_test_cmd, ai_alarm_test_cmd,
    ai_maintenance_test_cmd, ai_reset_to_display_value_cmd, complete_manual_test_subitem_cmd,
    capture_ao_point_cmd,
    // DI/DO手动测试命令 - 数字量输入/输出测试
    di_signal_test_cmd, capture_do_state_cmd
};

// 全局功能测试命令 - 处理系统级功能测试
use commands::global_function_test_commands::{
    get_global_function_tests_cmd,
    update_global_function_test_cmd,
    reset_global_function_tests_cmd,
};

// 测试PLC配置命令 - 管理PLC连接、通道配置、地址映射等
use commands::test_plc_config::{
    get_test_plc_channels_cmd, save_test_plc_channel_cmd, delete_test_plc_channel_cmd,
    get_plc_connections_cmd, save_plc_connection_cmd, test_plc_connection_cmd, test_temp_plc_connection_cmd,
    test_address_read_cmd, get_channel_mappings_cmd, generate_channel_mappings_cmd, 
    initialize_default_test_plc_channels_cmd, restore_default_test_plc_channels_cmd,
    restore_default_channels_from_sql_cmd
};
// Rust知识点：Arc<T> 是原子引用计数的智能指针，用于在多线程间共享所有权
use std::sync::Arc;

// 量程设置相关导入
use crate::interfaces::tauri::commands::channel_range_setting::apply_channel_range_setting_cmd;
use crate::application::services::range_setting_service::{ChannelRangeSettingService, IChannelRangeSettingService, DynamicRangeSettingService};

// 领域服务接口导入
// Rust知识点：dyn Trait 表示动态分发的trait对象，用于实现依赖倒置原则
use crate::domain::services::plc_communication_service::IPlcCommunicationService;
use crate::domain::services::IRangeRegisterRepository;
use crate::domain::services::range_value_calculator::{IRangeValueCalculator, DefaultRangeValueCalculator};

// 基础设施层实现导入
use crate::infrastructure::range_register_repository::RangeRegisterRepository;

/// 应用程序主要运行函数
///
/// 业务说明：
/// - 初始化日志系统（控制台+文件双重输出）
/// - 初始化应用状态（数据库连接、服务注册等）
/// - 创建并配置量程设置服务
/// - 启动Tauri应用并注册所有命令
/// 
/// 调用链：main.rs -> run() -> init_app_state() -> tauri::Builder
pub fn run() {
    // 创建logging目录
    use std::fs;
    if let Err(e) = fs::create_dir_all("logs") {
        eprintln!("创建logs目录失败: {}", e);
    }

    // 使用 tokio 运行时启动 Tauri 应用
    // Rust知识点：block_on 将异步代码阻塞执行，用于在同步上下文中运行异步代码
    tauri::async_runtime::block_on(async {
        // 初始化日志系统 - 在Tokio运行时环境内
        // 业务说明：只记录4类核心问题，避免日志过多
        use crate::logging::{LoggerConfig, Logger, LogTarget, LogFormat, LogLevel, LogCleanupConfig};
        use std::path::PathBuf;
        
        // 配置日志系统
        let logger_config = LoggerConfig {
            level: LogLevel::Info, // 只记录INFO及ERROR级别的核心问题
            targets: vec![
                LogTarget::Console,
                LogTarget::File { path: PathBuf::from("logs/fat_test.log") }
            ],
            format: LogFormat::Structured,
            rotation: crate::logging::LogRotation {
                max_file_size_mb: 50,
                max_files: 10,
                strategy: crate::logging::RotationStrategy::Size,
            },
            sanitization: crate::logging::SanitizationConfig {
                enabled: true,
                sensitive_fields: vec![
                    "password".to_string(),
                    "token".to_string(),
                    "secret".to_string(),
                    "key".to_string(),
                    "address".to_string(), // PLC地址也需要保护
                ],
                mode: crate::logging::SanitizationMode::Mask,
            },
            cleanup: LogCleanupConfig {
                enabled: true,
                retention_days: 90, // 保留3个月的日志
                check_interval_hours: 24,
            },
        };
        
        let mut logger = Logger::new(logger_config);
        if let Err(e) = logger.init() {
            eprintln!("初始化日志系统失败: {}", e);
            // 备用日志初始化
            let _ = env_logger::Builder::from_default_env()
                .filter_level(log::LevelFilter::Warn)
                .init();
        }

        log::info!("=== FAT_TEST 工厂测试系统启动 ===");
        log::info!("日志系统已初始化，只记录4类核心问题：通讯失败、文件解析失败、测试执行失败、用户配置操作");
        // 初始化应用状态
        // 业务说明：init_app_state会初始化数据库连接、执行数据迁移、创建所有必要的服务
        log::info!("开始初始化应用状态...");
        let app_state = match init_app_state().await {
            Ok(state) => {
                log::info!("应用状态初始化成功");
                state
            },
            Err(e) => {
                // 初始化失败则退出程序
                log::error!("初始化应用状态失败: {}", e);
                eprintln!("初始化应用状态失败: {}", e);
                std::process::exit(1);
            }
        };

        // 创建量程设定服务
        // 业务说明：量程设定服务用于管理PLC通道的量程范围配置
        let plc_service = crate::infrastructure::plc_communication::global_plc_service();

        // 尝试获取目标 PLC 连接句柄（先按 ID，再获取默认）
        // 业务说明：支持多PLC连接，优先使用配置的目标连接，否则使用默认连接
        let plc_handle_opt = match plc_service
            .default_handle_by_id(&app_state.target_connection_id)
            .await
        {
            Some(h) => Some(h),
            None => plc_service.default_handle().await,
        };

        // 创建依赖注入的服务实例
        // Rust知识点：使用 trait object (dyn Trait) 实现依赖倒置原则
        let plc_service_dyn: Arc<dyn IPlcCommunicationService> = plc_service.clone();
        let db_conn = app_state.persistence_service.get_database_connection();
        let range_repo: Arc<dyn IRangeRegisterRepository> = Arc::new(RangeRegisterRepository::new(db_conn));
        let calculator: Arc<dyn IRangeValueCalculator> = Arc::new(DefaultRangeValueCalculator);

        // 提前克隆持久化服务，供后续 .manage 使用
        // 业务说明：持久化服务管理所有数据的存储和查询
        let persistence_service: Arc<dyn crate::domain::services::IPersistenceService> = app_state.persistence_service.as_persistence_service();

        // 根据是否有PLC连接创建相应的量程设置服务实现
        let initial_impl: Arc<dyn IChannelRangeSettingService> = if let Some(handle) = plc_handle_opt {
             // 有PLC连接时使用真实实现
             Arc::new(ChannelRangeSettingService::new(
                 plc_service_dyn,
                 handle,
                 range_repo,
                 calculator,
             ))
         } else {
             // 无PLC连接时使用空实现，避免程序崩溃
             log::warn!("未找到PLC连接句柄，将使用空实现");
             Arc::new(application::services::range_setting_service::NullRangeSettingService::default())
         };

         // 创建动态容器，后续可在运行时替换实现
         // 业务说明：DynamicRangeSettingService允许在运行时切换不同的实现
         let range_setting_service = Arc::new(application::services::range_setting_service::DynamicRangeSettingService::new(initial_impl));

        // 启动 Tauri 应用
        // Rust知识点：Builder模式，通过链式调用构建复杂对象
        tauri::Builder::default()
            // 添加对话框插件，用于文件选择等UI交互
            .plugin(tauri_plugin_dialog::init())
            // 注册应用状态到Tauri管理器，供命令处理函数访问
            // Rust知识点：.manage() 将状态存储在类型映射中，通过类型系统保证唯一性
            .manage(app_state)
            .manage(range_setting_service.clone())
            .manage(persistence_service)
            // 设置钩子函数，在应用启动时执行
            .setup(|app| {
                // 在应用启动后设置AppHandle到事件发布器
                // 业务说明：AppHandle用于在后端主动向前端发送事件
                let app_handle = app.handle();

                // 设置全局AppHandle，用于事件发布
                // 业务说明：事件发布器负责将后端状态变化通知前端
                crate::infrastructure::event_publisher::set_global_app_handle(app_handle.clone());

                log::info!("Tauri应用启动完成，AppHandle已设置到事件发布器");

                Ok(())
            })
            // 注册所有可供前端调用的命令
            // Rust知识点：tauri::generate_handler! 宏自动生成命令分发代码
            .invoke_handler(tauri::generate_handler![
                // === 测试协调相关命令 ===
                // 业务说明：管理批次测试的生命周期（启动、暂停、恢复、停止）
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
                
                // === 数据管理相关命令 ===
                // 业务说明：处理测试数据的导入、存储、查询
                tauri_commands::import_excel_file,
                // tauri_commands::create_test_batch_with_definitions, // 注释掉，使用data_management中的版本
                tauri_commands::get_all_channel_definitions,
                tauri_commands::save_channel_definition,
                tauri_commands::delete_channel_definition,
                tauri_commands::get_all_batch_info,
                tauri_commands::save_batch_info,
                tauri_commands::get_batch_test_instances,
                
                // === 通道状态管理相关命令 ===
                // 业务说明：管理测试通道的状态和测试结果
                tauri_commands::create_test_instance,
                tauri_commands::get_instance_state,
                tauri_commands::update_test_result,
                
                // === 系统信息相关命令 ===
                tauri_commands::get_system_status,
                
                // === 报告生成相关命令 ===
                // 业务说明：生成测试报告（PDF、Excel）、管理报告模板
                tauri_commands::generate_pdf_report,
                tauri_commands::generate_excel_report,
                tauri_commands::get_reports,
                tauri_commands::get_report_templates,
                tauri_commands::create_report_template,
                tauri_commands::update_report_template,
                tauri_commands::delete_report_template,
                tauri_commands::delete_report,
                
                // === 应用配置相关命令 ===
                tauri_commands::load_app_settings_cmd,
                tauri_commands::save_app_settings_cmd,
                // === 数据管理命令 ===
                // 业务说明：批次数据的完整生命周期管理
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
                restore_session_cmd,
                
                // === 手动测试命令 ===
                // 业务说明：手动测试执行、PLC直接读写操作
                execute_manual_sub_test_cmd,
                read_channel_value_cmd,
                write_channel_value_cmd,
                connect_plc_cmd,
                start_batch_auto_test_cmd,
                get_plc_connection_status_cmd,
                
                // === 新版手动测试命令 ===
                // 业务说明：改进的手动测试流程，支持分步骤测试
                start_manual_test_cmd,
                update_manual_test_subitem_cmd,
                get_manual_test_status_cmd,
                start_plc_monitoring_cmd,
                stop_plc_monitoring_cmd,
                
                // === AI/AO手动测试专用命令 ===
                // 业务说明：模拟量输入/输出测试的专用命令
                generate_random_display_value_cmd,
                ai_show_value_test_cmd,
                ai_alarm_test_cmd,
                ai_maintenance_test_cmd,
                ai_reset_to_display_value_cmd,
                complete_manual_test_subitem_cmd,
                capture_ao_point_cmd,
                
                // === DI/DO手动测试命令 ===
                // 业务说明：数字量输入/输出测试
                di_signal_test_cmd,
                capture_do_state_cmd,
                
                // === 全局功能测试命令 ===
                // 业务说明：系统级功能测试管理
                get_global_function_tests_cmd,
                update_global_function_test_cmd,
                reset_global_function_tests_cmd,
                
                // === 测试PLC配置命令 ===
                // 业务说明：PLC连接配置、通道映射管理
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
                restore_default_test_plc_channels_cmd,
                restore_default_channels_from_sql_cmd,
                
                // === 量程设置命令 ===
                apply_channel_range_setting_cmd,
                
                // === 导出相关命令 ===
                // 导出通道分配
                tauri_commands::export_channel_allocation_cmd,
                // 导出测试结果
                tauri_commands::export_test_results_cmd,
                
                // === 错误管理命令 ===
                // 错误备注管理
                tauri_commands::save_error_notes_cmd,
                // 获取测试实例详情
                tauri_commands::get_test_instance_details_cmd
            ])
            // 运行Tauri应用
            // Rust知识点：generate_context! 宏从tauri.conf.json生成应用配置
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
