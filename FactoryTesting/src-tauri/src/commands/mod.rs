/// Tauri命令模块
/// 
/// 包含所有前端可调用的Tauri命令

pub mod data_management;
pub mod manual_testing;
pub mod test_plc_config;

// 重新导出命令
pub use data_management::{
    parse_excel_file,
    create_test_batch,
    get_batch_list,
    get_batch_channel_definitions,
};

pub use test_plc_config::{
    get_test_plc_channels_cmd,
    save_test_plc_channel_cmd,
    delete_test_plc_channel_cmd,
    get_plc_connections_cmd,
    save_plc_connection_cmd,
    test_plc_connection_cmd,
    get_channel_mappings_cmd,
    generate_channel_mappings_cmd,
    initialize_default_test_plc_channels_cmd,
}; 