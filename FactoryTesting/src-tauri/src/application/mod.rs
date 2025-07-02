//! 应用层模块 (Application Layer)
//! 负责编排领域服务、对外提供用例级服务接口。

pub mod services;

// 重新导出应用层常用类型，方便上层调用
pub use services::*; 