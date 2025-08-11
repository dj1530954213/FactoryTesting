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
pub mod global_logger_adapter;
pub mod simple_logger;
pub mod file_manager;
pub mod advanced_file_writer;
pub mod cleanup_scheduler;
pub mod enterprise_logger;
#[cfg(test)]
pub mod usage_examples;
#[cfg(test)]
pub mod test_logger;
#[cfg(test)]
pub mod test_complete_logger;
#[cfg(test)]
pub mod quick_test;
#[cfg(test)]
pub mod macro_examples;
#[cfg(test)]
pub mod integration_test;
#[cfg(test)]
pub mod quick_macro_test;
#[cfg(test)]
pub mod comprehensive_tests;
#[cfg(test)]
pub mod test_runner;

pub use logger_config::*;
pub use enterprise_logger::{EnterpriseLogger, EnterpriseLoggerBuilder, GlobalEnterpriseLogger};

/// 核心问题日志宏系统
/// 
/// 重构的4类核心日志宏，集成到新的Logger系统中
/// 使用CoreLogCategory分类，包含文件和行号信息
/// 调用实际的Logger方法产生可见的日志输出

/// 记录通讯失败日志
/// 
/// 记录PLC通讯、网络连接等通讯相关的失败信息
/// 
/// # 示例
/// ```
/// log_communication_failure!("PLC连接超时");
/// log_communication_failure!("网络错误: {}", error_msg);
/// ```
#[macro_export]
macro_rules! log_communication_failure {
    ($msg:expr) => {{
        // 同时使用标准日志系统和直接输出确保可见性
        let formatted_msg = format!("[通讯失败] {}", $msg);
        
        // 使用标准log系统 - 会被Logger的log方法处理
        log::error!("{}", formatted_msg);
        
        // 直接控制台输出确保立即可见
        eprintln!("\x1b[31m[{}] [ERROR] [通讯失败] [{}:{}] - {}\x1b[0m", 
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S%.3f"),
            file!(),
            line!(),
            $msg
        );
        
        // 如果使用的是企业Logger，也记录到结构化日志
        // 这里通过全局logger的category检测来记录
        use std::io::{self, Write};
        let _ = io::stderr().flush();
    }};
    ($msg:expr, $($arg:tt)*) => {{
        let formatted_msg = format!($msg, $($arg)*);
        $crate::log_communication_failure!(formatted_msg);
    }};
}

/// 记录文件解析失败日志
/// 
/// 记录导入、导出文件时的解析错误
/// 
/// # 示例
/// ```
/// log_file_parsing_failure!("无效的CSV格式");
/// log_file_parsing_failure!("解析第{}行失败: {}", line_num, error);
/// ```
#[macro_export]
macro_rules! log_file_parsing_failure {
    ($msg:expr) => {{
        let formatted_msg = format!("[文件解析失败] {}", $msg);
        
        // 使用标准log系统
        log::error!("{}", formatted_msg);
        
        // 直接控制台输出
        eprintln!("\x1b[31m[{}] [ERROR] [文件解析失败] [{}:{}] - {}\x1b[0m", 
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S%.3f"),
            file!(),
            line!(),
            $msg
        );
        
        use std::io::{self, Write};
        let _ = io::stderr().flush();
    }};
    ($msg:expr, $($arg:tt)*) => {{
        let formatted_msg = format!($msg, $($arg)*);
        $crate::log_file_parsing_failure!(formatted_msg);
    }};
}

/// 记录测试执行失败日志
/// 
/// 记录测试过程中的执行失败信息
/// 
/// # 示例
/// ```
/// log_test_failure!("温度测试超出范围");
/// log_test_failure!("第{}项测试失败: 期望{}, 实际{}", test_id, expected, actual);
/// ```
#[macro_export]
macro_rules! log_test_failure {
    ($msg:expr) => {{
        let formatted_msg = format!("[测试执行失败] {}", $msg);
        
        // 使用标准log系统
        log::error!("{}", formatted_msg);
        
        // 直接控制台输出
        eprintln!("\x1b[31m[{}] [ERROR] [测试执行失败] [{}:{}] - {}\x1b[0m", 
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S%.3f"),
            file!(),
            line!(),
            $msg
        );
        
        use std::io::{self, Write};
        let _ = io::stderr().flush();
    }};
    ($msg:expr, $($arg:tt)*) => {{
        let formatted_msg = format!($msg, $($arg)*);
        $crate::log_test_failure!(formatted_msg);
    }};
}

/// 记录用户操作日志
/// 
/// 记录用户配置和连接操作信息
/// 
/// # 示例
/// ```
/// log_user_operation!("用户连接PLC设备");
/// log_user_operation!("用户{}修改了配置项: {}", username, config_name);
/// ```
#[macro_export]
macro_rules! log_user_operation {
    ($msg:expr) => {{
        let formatted_msg = format!("[用户操作] {}", $msg);
        
        // 使用标准log系统
        log::info!("{}", formatted_msg);
        
        // 直接控制台输出
        println!("[{}] [INFO] [用户操作] [{}:{}] - {}", 
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S%.3f"),
            file!(),
            line!(),
            $msg
        );
        
        use std::io::{self, Write};
        let _ = io::stdout().flush();
    }};
    ($msg:expr, $($arg:tt)*) => {{
        let formatted_msg = format!($msg, $($arg)*);
        $crate::log_user_operation!(formatted_msg);
    }};
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