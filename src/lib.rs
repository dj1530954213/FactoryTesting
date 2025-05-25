/// FAT_TEST 工厂测试系统 - Rust后端核心库
pub mod models;
pub mod utils;
pub mod services;
pub mod tauri_commands;

// 重新导出常用类型，方便使用
pub use models::*;
pub use utils::{AppError, AppResult, AppConfig};
pub use services::*;
pub use tauri_commands::{AppState, SystemStatus, init_app_state, get_tauri_commands};

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
            .invoke_handler(get_tauri_commands())
            .run(tauri::generate_context!())
            .expect("启动 Tauri 应用失败");
    });
} 