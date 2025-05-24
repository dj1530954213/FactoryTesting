/// 服务层模块，包含应用层和领域层的服务定义
/// 
/// 按照清洁架构原则组织：
/// - Application Layer: 应用服务，协调业务流程
/// - Domain Layer: 领域服务，包含核心业务逻辑
/// - Infrastructure Layer: 基础设施服务，处理外部依赖

/// 应用层服务模块（将在后续步骤中实现）
// pub mod application;

/// 领域层服务模块（将在后续步骤中实现）
// pub mod domain;

/// 基础设施层服务模块
pub mod infrastructure;

/// 服务层基础trait定义
pub mod traits;

// 重新导出基础trait
pub use traits::{BaseService, StateManagementService, EventPublisher, ServiceContainer};

// 重新导出基础设施层的主要类型（避免冲突）
pub use infrastructure::plc::{
    PlcTag, PlcDataType, PlcConnectionStatus, PlcCommunicationStats, MockPlcService
};

// 重新导出持久化服务的主要类型
pub use infrastructure::persistence::{
    PersistenceConfig, PersistenceStats, QueryCriteria, QueryResult,
    ExtendedPersistenceService, JsonPersistenceService, PersistenceServiceFactory,
    BackupInfo, IntegrityReport, IntegrityStatus, PersistenceServiceHelper
}; 