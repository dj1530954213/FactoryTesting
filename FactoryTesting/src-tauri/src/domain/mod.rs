//! 领域层模块
//!
//! 包含业务逻辑和领域服务接口定义

pub mod services;
pub mod impls;

// 重新导出领域服务
pub use services::*;
pub use impls::*;
