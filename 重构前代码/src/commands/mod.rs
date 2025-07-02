/// Tauri命令模块
/// 
/// 包含所有前端可调用的Tauri命令

pub mod data_management;
pub mod manual_testing;
pub mod manual_test_commands;
pub mod test_plc_config;

// 重新导出命令
pub use data_management::{
    parse_excel_file,
    create_test_batch,
    get_batch_list,
    get_batch_channel_definitions,
    import_excel_and_prepare_batch_cmd,
    start_tests_for_batch_cmd,
    get_batch_status_cmd,
    prepare_test_instances_for_batch_cmd,
    import_excel_and_allocate_channels_cmd,
    parse_excel_and_create_batch_cmd,
    clear_session_data,
    parse_excel_without_persistence_cmd,
    create_batch_and_persist_data_cmd,
    delete_batch_cmd,
    restore_session_cmd
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

pub use manual_testing::{
    execute_manual_sub_test_cmd,
    read_channel_value_cmd,
    write_channel_value_cmd,
    connect_plc_cmd,
    start_batch_auto_test_cmd,
    get_plc_connection_status_cmd
};

pub use manual_test_commands::{
    start_manual_test_cmd,
    update_manual_test_subitem_cmd,
    get_manual_test_status_cmd,
    start_plc_monitoring_cmd,
    stop_plc_monitoring_cmd
};