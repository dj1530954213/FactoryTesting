/// 领域服务层模块
/// 包含核心业务逻辑和领域对象

/// 通道状态管理器 - 唯一负责修改ChannelTestInstance核心状态的地方
pub mod channel_state_manager;

/// 特定测试步骤执行器 - 执行具体的测试步骤
pub mod specific_test_executors;

/// 测试执行引擎 - 管理和并发执行测试序列
pub mod test_execution_engine;

/// 测试PLC配置服务 - 管理测试PLC配置
pub mod test_plc_config_service;

/// PLC连接管理器 - 管理PLC持久连接和心跳检测
pub mod plc_connection_manager;

// 重新导出常用类型
pub use channel_state_manager::{IChannelStateManager, ChannelStateManager};
pub use specific_test_executors::{
    ISpecificTestStepExecutor,
    AIHardPointPercentExecutor,
    AIAlarmTestExecutor,
    DIHardPointTestExecutor,
    DOHardPointTestExecutor,
    AOHardPointTestExecutor
};
pub use test_execution_engine::{
    ITestExecutionEngine,
    TestExecutionEngine,
    TaskStatus,
    TestTask
};
pub use test_plc_config_service::{ITestPlcConfigService, TestPlcConfigService};
pub use plc_connection_manager::{PlcConnectionManager, PlcConnectionState};