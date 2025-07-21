//! # 工具模块 (Utils Layer)
//!
//! ## 业务说明
//! 工具模块提供系统中各层都可能用到的通用功能，包括错误处理、配置管理、时间处理等
//! 这些功能与具体业务逻辑无关，但为整个系统提供基础支撑
//!
//! ## 模块功能
//! - **错误处理**: 统一的错误类型定义和错误处理机制
//! - **配置管理**: 应用配置的加载、验证和全局访问
//! - **时间工具**: UTC与北京时间的转换，时间格式化等
//!
//! ## 设计特点
//! - **无副作用**: 大部分函数都是纯函数，便于测试
//! - **类型安全**: 利用Rust类型系统确保配置和错误的正确性
//! - **全局可用**: 通过重新导出提供便捷的访问方式
//!
//! ## Rust知识点
//! - **错误处理**: 使用Result类型和自定义错误类型
//! - **配置模式**: 使用全局单例模式管理配置
//! - **类型别名**: 通过type定义简化复杂类型

/// 统一错误处理模块
pub mod error;

/// 配置管理模块
pub mod config;

/// 时间工具模块（UTC ↔ 北京时间转换）
pub mod time_utils;



// 重新导出常用类型，方便使用
pub use error::{AppError, AppResult};
pub use config::{
    AppConfig, AppSettings, PlcConfig, TestConfig, LoggingConfig, PersistenceConfig,
    ConfigManager, init_global_config, get_global_config, update_global_config,
}; 
