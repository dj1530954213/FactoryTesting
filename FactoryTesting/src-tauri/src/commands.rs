//! 兼容旧路径的 Commands 模块
//! 
//! 业务说明：
//! 这是一个重导出模块，用于保持向后兼容性
//! 实际的命令实现已迁移到 interfaces::tauri::commands 目录
//! 
//! Rust知识点：
//! pub use 语句用于重新导出其他模块的内容
//! 这样可以在不破坏现有代码的情况下重构模块结构
//! 
//! 调用链：前端 -> lib.rs引用此模块 -> 实际调用interfaces::tauri::commands中的实现

pub use crate::interfaces::tauri::commands::*; 
