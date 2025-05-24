// 核心数据模型模块
pub mod models;

// Tauri 应用程序入口点配置
// 在移动平台上使用此宏标记入口点
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  // 构建并运行 Tauri 应用程序
  tauri::Builder::default()
    .setup(|app| {
      // 在调试模式下启用日志插件
      if cfg!(debug_assertions) {
        app.handle().plugin(
          tauri_plugin_log::Builder::default()
            .level(log::LevelFilter::Info) // 设置日志级别为 Info
            .build(),
        )?;
      }
      Ok(())
    })
    .run(tauri::generate_context!()) // 运行应用程序，生成上下文
    .expect("启动 Tauri 应用程序时出错"); // 错误处理
}
