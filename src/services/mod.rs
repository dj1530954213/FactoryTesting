/// 服务层模块
/// 包含所有业务逻辑和基础设施服务

/// 基础设施层服务 (外部系统交互)
pub mod infrastructure;

/// 领域服务层 (核心业务逻辑)
pub mod domain;

/// 应用层服务 (业务流程协调)
pub mod application;

// 重新导出常用类型
pub use infrastructure::persistence::{IPersistenceService, SqliteOrmPersistenceService};
pub use infrastructure::persistence::app_settings_service::{IAppSettingsService, JsonAppSettingsService, AppSettingsServiceFactory};
pub use infrastructure::plc::{IPlcCommunicationService, MockPlcService};
pub use domain::{
    IChannelStateManager, ChannelStateManager,
    ISpecificTestStepExecutor, AIHardPointPercentExecutor, AIAlarmTestExecutor, DIStateReadExecutor,
    ITestExecutionEngine, TestExecutionEngine, TaskStatus, TestTask
};
pub use application::{
    ITestCoordinationService, TestCoordinationService,
    TestExecutionRequest, TestExecutionResponse, TestProgressUpdate
}; 