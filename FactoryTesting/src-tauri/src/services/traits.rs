//! 兼容旧路径的 Trait 导出
//! ⚠️ 重构结束后请统一改为 `crate::domain::services::*` 并删除本文件

pub use crate::domain::services::*;

// === 为兼容旧代码提供无 I 前缀别名 ===
pub use crate::domain::services::{
    IEventPublisher as EventPublisher,
    IPersistenceService as PersistenceService,
};

// 导出 BaseService 供旧代码引用
pub use crate::domain::services::BaseService; 