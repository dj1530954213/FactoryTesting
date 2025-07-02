// 防止在 Windows 发布版本中显示额外的控制台窗口，请勿删除！
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

// 应用程序主入口函数
fn main() {
    // 启动 Tauri 应用
    app_lib::run();
}
