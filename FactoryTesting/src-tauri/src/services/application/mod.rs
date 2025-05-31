/// 应用层服务模块
///
/// 应用层负责协调领域服务和基础设施服务，实现完整的业务流程
/// 提供面向用户的高级API

/// 测试协调服务 - 协调整个测试流程
pub mod test_coordination_service;
pub mod report_generation_service;
pub mod data_import_service;
pub mod batch_allocation_service;

// 重新导出主要的服务
pub use data_import_service::{DataImportService, ImportResult};
pub use batch_allocation_service::{BatchAllocationService, AllocationResult, AllocationStrategy};

// 重新导出常用类型
pub use test_coordination_service::{
    ITestCoordinationService,
    TestCoordinationService,
    TestExecutionRequest,
    TestExecutionResponse,
    TestProgressUpdate,
};

// 重新导出服务接口和实现
pub use report_generation_service::*;