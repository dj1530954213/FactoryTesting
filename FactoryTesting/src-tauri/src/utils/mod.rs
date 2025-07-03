/// 工具模块，包含错误处理、配置管理等通用功能

/// 统一错误处理模块
pub mod error;

/// 配置管理模块
pub mod config;



// 重新导出常用类型，方便使用
pub use error::{AppError, AppResult};
pub use config::{
    AppConfig, AppSettings, PlcConfig, TestConfig, LoggingConfig, PersistenceConfig,
    ConfigManager, init_global_config, get_global_config, update_global_config,
}; 
