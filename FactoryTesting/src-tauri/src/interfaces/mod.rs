//! 接口适配层 (Interfaces Layer)
//! 负责对接外部框架 (Tauri)、CLI、Web 等适配代码。

pub mod tauri;

// 按需导出
pub use tauri::*; 