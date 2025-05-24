// 文件: src-tauri/src/services/domain/mod.rs
// 详细注释：声明和导出领域服务模块

// pub mod test_execution_engine; // 示例：测试执行引擎
// pub mod specific_test_executors; // 示例：具体测试执行器
// pub mod statistics_service; // 示例：统计服务
// pub mod test_record_service; // 示例：测试记录服务

pub mod channel_state_manager;

pub use channel_state_manager::{ChannelStateManager, ChannelStateManagerService}; 