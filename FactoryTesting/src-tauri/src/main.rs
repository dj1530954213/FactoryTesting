// 防止在 Windows 发布版本中显示额外的控制台窗口，请勿删除！
// Rust知识点：cfg_attr 条件编译属性，仅在release模式下生效
// windows_subsystem = "windows" 表示使用Windows图形界面子系统而非控制台子系统
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

/// 应用程序主入口函数
/// 
/// 业务说明：
/// - FAT_TEST工厂测试系统的程序入口
/// - 直接委托给lib.rs中的run函数执行实际启动逻辑
/// 
/// Rust知识点：
/// - main函数是Rust程序的入口点
/// - app_lib是crate名称，对应Cargo.toml中的[lib] name = "app_lib"
fn main() {
    // 启动 Tauri 应用
    // 调用链：main.rs -> lib.rs::run() -> tauri::Builder启动
    app_lib::run();
}
