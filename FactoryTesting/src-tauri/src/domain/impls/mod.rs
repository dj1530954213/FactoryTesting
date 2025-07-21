//! # 领域层实现模块 (Domain Implementations)
//!
//! ## 业务说明
//! 本模块包含领域服务接口(trait)的具体实现，提供真实的业务逻辑处理能力
//! 实现与接口定义分离，遵循依赖倒置原则，便于测试和替换
//!
//! ## 实现类型
//! - **真实实现**: 生产环境使用的完整业务逻辑实现
//! - **测试实现**: 用于单元测试的Mock实现
//! - **空实现**: 用于开发阶段的占位实现
//!
//! ## 核心实现
//! - **通道状态管理**: ChannelStateManager - 管理测试通道的状态转换
//! - **测试执行引擎**: TestExecutionEngine - 执行具体的测试任务
//! - **特定测试器**: Specific Test Executors - AI/AO/DI/DO各类型的专用执行器
//! - **PLC连接管理**: PlcConnectionManager - 管理PLC设备连接
//! - **批次分配服务**: RealBatchAllocationService - 处理测试批次的分配逻辑
//! - **测试编排服务**: RealTestOrchestrationService - 协调整个测试流程
//!
//! ## 设计原则
//! - **单一职责**: 每个实现专注于一个特定的业务功能
//! - **依赖注入**: 通过构造函数注入依赖服务
//! - **错误处理**: 统一的错误处理和传播机制
//!
//! ## Rust知识点
//! - **trait实现**: impl Trait for Struct语法
//! - **组合模式**: 通过包含其他服务实现复杂功能
//! - **生命周期管理**: 合理管理资源的生命周期

pub mod channel_state_manager;
pub mod test_execution_engine;
pub mod specific_test_executors;
pub mod plc_connection_manager;
// pub mod stub_test_orchestration_service; // retired after real implementation
pub mod real_test_orchestration_service;
// pub mod stub_batch_allocation_service; // retired after real implementation
pub mod real_batch_allocation_service;
// pub mod noop_services; // 已弃用
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
// pub use stub_batch_allocation_service::StubBatchAllocationService;
pub use real_test_orchestration_service::RealTestOrchestrationService;
pub use real_batch_allocation_service::RealBatchAllocationService;
pub use plc_connection_manager::{PlcConnectionManager, PlcConnectionState}; 
