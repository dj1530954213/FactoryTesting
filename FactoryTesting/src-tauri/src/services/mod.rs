/// 服务层模块，包含应用层和领域层的服务定义
///
/// 按照清洁架构原则组织：
/// - Application Layer: 应用服务，协调业务流程
/// - Domain Layer: 领域服务，包含核心业务逻辑
/// - Infrastructure Layer: 基础设施服务，处理外部依赖

/// 应用层服务模块
pub mod application;

/// 领域层服务模块
pub mod domain;

/// 基础设施层服务模块
pub mod infrastructure;

/// 通道分配服务模块
pub mod channel_allocation_service;

/// 服务层基础trait定义
pub mod traits;

// 重新导出基础trait
pub use traits::{BaseService, StateManagementService, EventPublisher, ServiceContainer};

// 重新导出应用层服务
pub use application::{
    ITestCoordinationService, TestCoordinationService,
    TestExecutionRequest, TestExecutionResponse, TestProgressUpdate
};

// 重新导出领域层服务
pub use domain::{
    IChannelStateManager, ChannelStateManager,
    ITestExecutionEngine, TestExecutionEngine, TaskStatus, TestTask,
    ISpecificTestStepExecutor, AIHardPointPercentExecutor,
    AIAlarmTestExecutor, DIStateReadExecutor
};

// 重新导出通道分配服务
pub use channel_allocation_service::{
    IChannelAllocationService, ChannelAllocationService,
    ComparisonTable, TestPlcConfig, BatchAllocationResult,
    AllocationSummary, ModuleTypeStats, ValidationResult
};

// 重新导出批次分配服务
pub use application::batch_allocation_service::{
    AllocationResult, AllocationStrategy
};

// 重新导出基础设施层的主要类型（避免冲突）
pub use infrastructure::{
    IPlcCommunicationService,
    PlcTag, PlcDataType, PlcConnectionStatus, PlcCommunicationStats,
    IPersistenceService, SqliteOrmPersistenceService,
    PersistenceConfig, PersistenceStats, QueryCriteria, QueryResult,
    ExtendedPersistenceService, PersistenceServiceFactory,
    BackupInfo, IntegrityReport, IntegrityStatus, PersistenceServiceHelper,
    // 应用配置服务
    AppSettingsService, JsonAppSettingsService, AppSettingsConfig, AppSettingsServiceFactory
};