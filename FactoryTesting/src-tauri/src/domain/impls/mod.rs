//! 领域层实现（ChannelStateManager、TestExecutionEngine 等）
//! 与 traits 分离，放置在 domain::impls 便于与接口解耦。

pub mod channel_state_manager;
pub mod test_execution_engine;
pub mod specific_test_executors;
pub mod plc_connection_manager;
pub mod stub_test_orchestration_service;
pub mod stub_batch_allocation_service;
pub mod real_batch_allocation_service;
#[cfg(FALSE)]
pub mod noop_services;
pub mod test_plc_config_service;

// 仅导出实现结构体及关联枚举
pub use channel_state_manager::ChannelStateManager;
pub use specific_test_executors::{
    AIHardPointPercentExecutor,
    AIAlarmTestExecutor,
    DIHardPointTestExecutor,
    DOHardPointTestExecutor,
    AOHardPointTestExecutor,
};
pub use test_execution_engine::{
    TestExecutionEngine,
    TaskStatus,
    TestTask,
};
pub use test_plc_config_service::TestPlcConfigService;
pub use stub_batch_allocation_service::StubBatchAllocationService;
pub use real_batch_allocation_service::RealBatchAllocationService;
pub use plc_connection_manager::{PlcConnectionManager, PlcConnectionState}; 