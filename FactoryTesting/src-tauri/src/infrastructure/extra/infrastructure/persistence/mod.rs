/// 持久化服务模块
/// 提供数据持久化功能，支持JSON文件和数据库存储

/// 持久化服务接口定义和基础类型
pub mod persistence_service;

/// 应用配置服务（专门处理应用设置）
pub mod app_settings_service;

/// SQLite ORM 持久化实现
pub mod sqlite_orm_persistence_service;

/// 单元测试模块
#[cfg(test)]
pub mod tests;

// 为后续步骤准备的数据库实现（暂时注释）
// pub mod database_persistence_service;
// pub mod sqlite_persistence_service;

// 重新导出主要接口和实现
pub use persistence_service::*;
pub use app_settings_service::*;
pub use sqlite_orm_persistence_service::*; 
