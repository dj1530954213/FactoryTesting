// 防止在 Windows 发布版本中显示额外的控制台窗口，请勿删除！
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

/// 应用程序主入口函数
/// 
/// 启动 FAT_TEST 工厂测试系统
fn main() {
    // 初始化日志系统
    env_logger::init();
    
    println!("=== FAT_TEST 工厂测试系统启动 ===");
    
    // 启动 Tauri 应用
    app_lib::run();
} 