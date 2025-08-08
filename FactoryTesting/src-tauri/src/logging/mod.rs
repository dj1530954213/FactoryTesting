//! # 日志记录模块 (Logging Module)
//!
//! ## 业务说明
//! 日志记录模块负责系统运行过程中的信息记录，包括用户操作、系统事件、错误信息等
//! 为故障排查、审计追踪和系统分析提供完整的日志支持
//!
//! ## 日志功能
//! - **结构化记录**: 使用标准格式记录各类系统事件
//! - **级别控制**: 支持Debug、Info、Warn、Error等不同级别
//! - **双重输出**: 同时输出到控制台和文件，便于开发和生产使用
//! - **敏感信息过滤**: 自动过滤PLC地址、用户信息等敏感数据
//!
//! ## 日志策略
//! - **业务日志**: 记录测试执行、数据导入等业务操作
//! - **系统日志**: 记录PLC连接、数据库操作等系统事件
//! - **错误日志**: 详细记录异常情况，便于问题定位
//! - **性能日志**: 记录关键操作的执行时间
//!
//! ## Rust知识点
//! - **日志宏**: 使用log crate的宏系统
//! - **环境配置**: 通过env_logger进行环境变量配置
//! - **格式化**: 自定义日志输出格式和时间戳

pub mod logger_config;

pub use logger_config::*;

/// 便捷日志宏 - 记录核心问题日志
/// 只记录4类核心问题，避免日志冗余

/// 记录通讯失败日志
#[macro_export]
macro_rules! log_communication_failure {
    ($msg:expr) => {
        log::error!("[通讯失败] {}", $msg);
    };
    ($msg:expr, $($arg:tt)*) => {
        log::error!("[通讯失败] {}", format!($msg, $($arg)*));
    };
}

/// 记录文件解析失败日志
#[macro_export]
macro_rules! log_file_parsing_failure {
    ($msg:expr) => {
        log::error!("[文件解析失败] {}", $msg);
    };
    ($msg:expr, $($arg:tt)*) => {
        log::error!("[文件解析失败] {}", format!($msg, $($arg)*));
    };
}

/// 记录测试执行失败日志
#[macro_export]
macro_rules! log_test_failure {
    ($msg:expr) => {
        log::error!("[测试执行失败] {}", $msg);
    };
    ($msg:expr, $($arg:tt)*) => {
        log::error!("[测试执行失败] {}", format!($msg, $($arg)*));
    };
}

/// 记录用户操作日志
#[macro_export]
macro_rules! log_user_operation {
    ($msg:expr) => {
        log::info!("[用户操作] {}", $msg);
    };
    ($msg:expr, $($arg:tt)*) => {
        log::info!("[用户操作] {}", format!($msg, $($arg)*));
    };
}

/// 记录用户配置操作警告
#[macro_export]
macro_rules! log_config_warning {
    ($msg:expr) => {
        log::warn!("[配置警告] {}", $msg);
    };
    ($msg:expr, $($arg:tt)*) => {
        log::warn!("[配置警告] {}", format!($msg, $($arg)*));
    };
}

// 重新导出宏
pub use log_communication_failure;
pub use log_file_parsing_failure;
pub use log_test_failure;
pub use log_user_operation;
pub use log_config_warning;
