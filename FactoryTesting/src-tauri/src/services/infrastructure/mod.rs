/// 基础设施层服务模块
/// 负责与外部系统的交互，如PLC通信、数据持久化等

/// PLC通信相关模块
pub mod plc;

/// 数据持久化相关模块
pub mod persistence;

/// Excel文件处理相关模块
pub mod excel;

/// 事件发布相关模块
pub mod event_publisher;

// 为后续步骤准备的模块（暂时注释）
// pub mod excel;

// 重新导出PLC相关接口和实现
pub use plc::{
    PlcCommunicationService as IPlcCommunicationService,
    PlcTag,
    PlcDataType,
    PlcConnectionStatus,
    PlcCommunicationStats,
};

// 重新导出持久化相关接口和实现
pub use persistence::{
    ExtendedPersistenceService as IPersistenceService,
    SqliteOrmPersistenceService,
    PersistenceConfig,
    PersistenceStats,
    QueryCriteria,
    QueryResult,
    ExtendedPersistenceService,
    PersistenceServiceFactory,
    BackupInfo,
    IntegrityReport,
    IntegrityStatus,
    PersistenceServiceHelper,
    AppSettingsService,
    JsonAppSettingsService,
    AppSettingsConfig,
    AppSettingsServiceFactory,
};

// 重新导出Excel相关服务
pub use excel::{
    ExcelImporter,
};

// 重新导出事件发布相关服务
pub use event_publisher::{
    SimpleEventPublisher,
};

// 或者更明确地导出需要的类型，例如：
// pub use plc::plc_communication_service::IPlcCommunicationService;
// pub use plc::mock_plc_service::MockPlcService;
// pub use persistence::persistence_service::IPersistenceService;
// pub use persistence::sqlite_orm_persistence_service::SqliteOrmPersistenceService;
// pub use persistence::json_persistence_service::JsonPersistenceService;

// 暂时先不 re-export，让调用方使用完整路径，或者只导出模块
// pub use plc; // 移除或改为 pub use plc::SpecificType;
// pub use persistence; // 移除或改为 pub use persistence::SpecificType;