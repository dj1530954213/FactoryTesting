/// Tauri命令模块
/// 
/// 业务说明：
/// 这是所有前端可调用的Tauri命令的聚合模块
/// 按功能域组织成不同的子模块，便于维护和扩展
/// 
/// 架构说明：
/// - data_management: 数据管理命令（Excel导入、批次管理等）
/// - manual_testing: 手动测试命令（旧版）
/// - manual_test_commands: 手动测试命令（新版，支持分步测试）
/// - global_function_test_commands: 全局功能测试命令
/// - test_plc_config: PLC配置管理命令
/// - channel_range_setting: 通道量程设置命令
/// 
/// Rust知识点：
/// pub mod 声明公开的子模块
/// pub use 重新导出子模块的内容，简化外部引用路径

pub mod data_management;
pub mod manual_testing;
pub mod manual_test_commands;
pub mod global_function_test_commands;
pub mod test_plc_config;
pub mod channel_range_setting;

// === 数据管理命令重导出 ===
// 业务说明：处理Excel文件解析、批次创建、数据持久化等操作
// 调用链：前端 -> 这些命令 -> DataManagementService -> PersistenceService
pub use data_management::{
    parse_excel_file,                          // 解析Excel文件
    create_test_batch,                         // 创建测试批次
    get_batch_list,                           // 获取批次列表
    get_batch_channel_definitions,             // 获取批次通道定义
    import_excel_and_prepare_batch_cmd,        // 导入Excel并准备批次
    start_tests_for_batch_cmd,                 // 启动批次测试
    get_batch_status_cmd,                      // 获取批次状态
    prepare_test_instances_for_batch_cmd,      // 准备批次测试实例
    import_excel_and_allocate_channels_cmd,    // 导入Excel并分配通道
    parse_excel_and_create_batch_cmd,          // 解析Excel并创建批次
    clear_session_data,                        // 清除会话数据
    parse_excel_without_persistence_cmd,       // 解析Excel但不持久化
    create_batch_and_persist_data_cmd,         // 创建批次并持久化数据
    delete_batch_cmd,                          // 删除批次
    restore_session_cmd                        // 恢复会话
};

// === PLC配置管理命令重导出 ===
// 业务说明：管理PLC连接配置、通道映射、地址配置等
// 调用链：前端 -> 这些命令 -> PlcConfigService -> PersistenceService
pub use test_plc_config::{
    get_test_plc_channels_cmd,                 // 获取测试PLC通道配置
    save_test_plc_channel_cmd,                 // 保存测试PLC通道配置
    delete_test_plc_channel_cmd,               // 删除测试PLC通道配置
    get_plc_connections_cmd,                   // 获取PLC连接列表
    save_plc_connection_cmd,                   // 保存PLC连接配置
    test_plc_connection_cmd,                   // 测试PLC连接
    get_channel_mappings_cmd,                  // 获取通道映射
    generate_channel_mappings_cmd,             // 生成通道映射
    initialize_default_test_plc_channels_cmd,  // 初始化默认测试PLC通道
};

// === 手动测试命令重导出（旧版） ===
// 业务说明：直接执行手动测试的命令，较为简单的测试流程
// 注意：这是旧版API，新项目建议使用manual_test_commands
pub use manual_testing::{
    execute_manual_sub_test_cmd,               // 执行手动子测试
    read_channel_value_cmd,                    // 读取通道值
    write_channel_value_cmd,                   // 写入通道值
    connect_plc_cmd,                          // 连接PLC
    start_batch_auto_test_cmd,                 // 启动批次自动测试
    get_plc_connection_status_cmd              // 获取PLC连接状态
};

// === 手动测试命令重导出（新版） ===
// 业务说明：支持分步骤的手动测试流程，更灵活的测试管理
// 调用链：前端 -> 这些命令 -> ManualTestService -> PlcCommunicationService
pub use manual_test_commands::{
    start_manual_test_cmd,                     // 启动手动测试
    update_manual_test_subitem_cmd,            // 更新手动测试子项状态
    get_manual_test_status_cmd,                // 获取手动测试状态
    start_plc_monitoring_cmd,                  // 启动PLC监控
    stop_plc_monitoring_cmd,                   // 停止PLC监控
    capture_ao_point_cmd                       // 捕获AO点位值
};

// === 通道量程设置命令重导出 ===
// 业务说明：设置AI/AO通道的量程范围
pub use channel_range_setting::apply_channel_range_setting_cmd;

// === 全局功能测试命令重导出 ===
// 业务说明：系统级功能测试，如报警测试、通信测试等
pub use global_function_test_commands::{
    get_global_function_tests_cmd,             // 获取全局功能测试列表
    update_global_function_test_cmd,           // 更新全局功能测试状态
    reset_global_function_tests_cmd,           // 重置全局功能测试
};
