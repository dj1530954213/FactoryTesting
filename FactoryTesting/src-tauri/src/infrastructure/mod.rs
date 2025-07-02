//! 基础设施层模块
//!
//! 包含依赖注入容器和基础设施服务

pub mod di_container;
pub mod plc_communication;
pub mod plc_compat;
pub mod extra; // 临时迁移的基础设施代码，后续合并重构

#[cfg(test)]
mod plc_communication_tests;

// 重新导出基础设施组件
pub use di_container::*;
pub use plc_communication::*;
pub use plc_compat::*;
pub use extra::*;
