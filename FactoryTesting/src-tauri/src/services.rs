//! (已废弃) 原 services 兼容层已移除。
//! 
//! ⚠️ 重构完成后请删除本文件，并将所有 `use crate::services::...` 改为新层级路径。

// === traits 文件模块声明 ===
pub mod traits;
// 重新导出 traits 中的内容，供旧路径使用
pub use traits::*;

// === application 层 ===
pub mod application {
    pub use crate::application::services::*;
}

// === channel_allocation_service 兼容模块 ===
pub mod channel_allocation_service {
    // 重新导出通道分配相关核心类型，保持旧路径兼容
    pub use crate::application::services::channel_allocation_service::{
        ChannelAllocationService,
        IChannelAllocationService,
        TestPlcConfig,
        ComparisonTable,
    };
}

// === domain  traits / 实现 ===
pub mod domain {
    // 精确 re-export 常用领域层 trait，保持旧路径兼容
    pub use crate::domain::services::{
        IPersistenceService,
        IEventPublisher,
    };
    // 使用 impls 目录中的完整 Trait 版本，避免方法缺失
    pub use crate::domain::impls::channel_state_manager::IChannelStateManager;
    pub use crate::domain::impls::test_execution_engine::ITestExecutionEngine;
    // 领域层实现 (ChannelStateManager, TestExecutionEngine, etc.)
    pub use crate::domain::impls::*;
    pub use crate::domain::impls::specific_test_executors::ISpecificTestStepExecutor;
    pub use crate::domain::impls::test_plc_config_service::ITestPlcConfigService;
}

// === infrastructure ===
pub mod infrastructure {
    pub use crate::infrastructure::*;
}

// === 旧顶层 helpers ===
// 旧路径 helpers：直接在顶层重新导出常用类型
pub use crate::application::services::channel_allocation_service::{
    ChannelAllocationService,
    IChannelAllocationService,
    TestPlcConfig,
    ComparisonTable,
};
