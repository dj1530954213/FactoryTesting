//! 依赖注入容器
//!
//! 负责管理所有服务的生命周期和依赖关系

use std::sync::Arc;
use crate::domain::services::*;
use crate::infrastructure::plc_communication::IPlcCommunicationService;
use crate::infrastructure::plc_communication::ModbusTcpPlcService;
use crate::domain::impls::real_test_orchestration_service::RealTestOrchestrationService;
use crate::application::services::test_coordination_service::TestCoordinationService;
use crate::application::services::channel_allocation_service::ChannelAllocationService;
use crate::domain::impls::test_plc_config_service::TestPlcConfigService;
use crate::infrastructure::extra::infrastructure::event_publisher::SimpleEventPublisher;
use crate::domain::impls::real_batch_allocation_service::RealBatchAllocationService;
use sea_orm::DatabaseConnection;
use crate::infrastructure::extra::infrastructure::{
    IPersistenceService, SqliteOrmPersistenceService, PersistenceConfig,
};
use crate::domain::impls::channel_state_manager::ChannelStateManager;
use crate::domain::impls::test_execution_engine::TestExecutionEngine;
use crate::utils::error::AppError;

/// 应用服务容器接口
pub trait ServiceContainer: Send + Sync {
    /// 获取测试编排服务
    fn get_test_orchestration_service(&self) -> Arc<dyn ITestOrchestrationService>;

    /// 获取通道状态管理器
    fn get_channel_state_manager(&self) -> Arc<dyn IChannelStateManager>;

    /// 获取测试执行引擎
    fn get_test_execution_engine(&self) -> Arc<dyn ITestExecutionEngine>;

    /// 获取PLC通信服务
    fn get_plc_communication_service(&self) -> Arc<dyn IPlcCommunicationService>;

    /// 获取批次分配服务
    fn get_batch_allocation_service(&self) -> Arc<dyn IBatchAllocationService>;

    /// 获取事件发布服务
    fn get_event_publisher(&self) -> Arc<dyn IEventPublisher>;

    /// 获取持久化服务
    fn get_persistence_service(&self) -> Arc<dyn IPersistenceService>;
}

/// 应用配置接口
pub trait AppConfig: Send + Sync {
    /// 获取最大并发测试数
    fn max_concurrent_tests(&self) -> usize;

    /// 获取PLC连接超时时间（毫秒）
    fn plc_timeout_ms(&self) -> u64;

    /// 获取数据库连接字符串
    fn database_url(&self) -> &str;

    /// 获取日志级别
    fn log_level(&self) -> &str;



    /// 获取测试数据目录
    fn test_data_directory(&self) -> &str;

    /// 获取备份目录
    fn backup_directory(&self) -> &str;
}

/// 默认应用配置
#[derive(Debug, Clone)]
pub struct DefaultAppConfig {
    max_concurrent_tests: usize,
    plc_timeout_ms: u64,
    database_url: String,
    log_level: String,

    test_data_directory: String,
    backup_directory: String,
}

impl Default for DefaultAppConfig {
    fn default() -> Self {
        Self {
            max_concurrent_tests: 88,
            plc_timeout_ms: 5000,
            database_url: "sqlite://fat_test.db".to_string(),
            log_level: "info".to_string(),

            test_data_directory: "./test_data".to_string(),
            backup_directory: "./backups".to_string(),
        }
    }
}

impl AppConfig for DefaultAppConfig {
    fn max_concurrent_tests(&self) -> usize {
        self.max_concurrent_tests
    }

    fn plc_timeout_ms(&self) -> u64 {
        self.plc_timeout_ms
    }

    fn database_url(&self) -> &str {
        &self.database_url
    }

    fn log_level(&self) -> &str {
        &self.log_level
    }



    fn test_data_directory(&self) -> &str {
        &self.test_data_directory
    }

    fn backup_directory(&self) -> &str {
        &self.backup_directory
    }
}

/// 基于配置文件的应用配置
pub struct ConfigBasedAppConfig {
    settings: config::Config,
}

impl ConfigBasedAppConfig {
    pub fn new(settings: config::Config) -> Self {
        Self { settings }
    }
}

impl AppConfig for ConfigBasedAppConfig {
    fn max_concurrent_tests(&self) -> usize {
        self.settings.get("max_concurrent_tests").unwrap_or(88)
    }

    fn plc_timeout_ms(&self) -> u64 {
        self.settings.get("plc_timeout_ms").unwrap_or(5000)
    }

    fn database_url(&self) -> &str {
        // 注意：这里需要处理生命周期问题，实际实现中可能需要调整
        "sqlite://fat_test.db"
    }

    fn log_level(&self) -> &str {
        "info"
    }



    fn test_data_directory(&self) -> &str {
        "./test_data"
    }

    fn backup_directory(&self) -> &str {
        "./backups"
    }
}

/// 应用服务容器实现
pub struct AppServiceContainer {
    config: Arc<dyn AppConfig>,
    persistence_service: Arc<dyn IPersistenceService>,
    /// 全局共享的 PLC 通信服务实例（同时用于测试PLC与被测PLC）
    plc_service: Arc<ModbusTcpPlcService>,
}

impl AppServiceContainer {
    /// 创建新的服务容器
    pub fn new() -> Result<Self, AppError> {
        let config = Arc::new(DefaultAppConfig::default());
        // 创建同步 runtime 以便在同步上下文中等待 async new()
        let rt = tokio::runtime::Runtime::new().map_err(|e| AppError::generic(e.to_string()))?;
        let persistence_service = rt.block_on(async {
            let cfg = PersistenceConfig::default();
            // 使用默认存储根目录 (inside cfg)
            SqliteOrmPersistenceService::new(cfg, None).await
        })?;
        let persistence_service: Arc<dyn IPersistenceService> = Arc::new(persistence_service);
        let plc_service = crate::infrastructure::plc_communication::global_plc_service();
        Ok(Self { config, persistence_service, plc_service })
    }

    /// 从配置文件创建服务容器
    pub fn from_config(config_path: &str) -> Result<Self, AppError> {
        // 加载配置文件
        let settings = config::Config::builder()
            .add_source(config::File::with_name(config_path))
            .build()
            .map_err(|e| AppError::configuration_error(e.to_string()))?;

        // 使用配置创建容器
        let config = Arc::new(ConfigBasedAppConfig::new(settings));
        // 为简化演示，暂复用默认构造逻辑
Self::new()
    }


    }

impl ServiceContainer for AppServiceContainer {
    fn get_test_orchestration_service(&self) -> Arc<dyn ITestOrchestrationService> {
        // 在实际实现中，这里应该创建真实的服务实例
        // 目前返回Mock实现作为占位符
        {
            // build dependencies
            let channel_state_manager = self.get_channel_state_manager();
            let test_execution_engine = self.get_test_execution_engine();
            let persistence_service = self.get_persistence_service();
            let event_publisher = self.get_event_publisher();
            let channel_allocation_service: Arc<dyn crate::application::services::channel_allocation_service::IChannelAllocationService> = Arc::new(ChannelAllocationService::new());
            let plc_config_service = Arc::new(TestPlcConfigService::new(persistence_service.clone()));

            let tc_service = TestCoordinationService::new(
                channel_state_manager,
                test_execution_engine,
                persistence_service.clone(),
                event_publisher,
                channel_allocation_service,
                plc_config_service,
            );
            Arc::new(RealTestOrchestrationService::new(tc_service))
        }
    }

    fn get_channel_state_manager(&self) -> Arc<dyn IChannelStateManager> {
        Arc::new(ChannelStateManager::new(self.persistence_service.clone()))
    }

    fn get_test_execution_engine(&self) -> Arc<dyn ITestExecutionEngine> {
        Arc::new(TestExecutionEngine::new(
             4,
             self.plc_service.clone(),
             self.plc_service.clone(),
             String::new(),
             String::new(),
         ))
    }

    fn get_plc_communication_service(&self) -> Arc<dyn IPlcCommunicationService> {
        self.plc_service.clone()
    }

    fn get_batch_allocation_service(&self) -> Arc<dyn IBatchAllocationService> {
        let db = Arc::new(self.persistence_service.get_database_connection());
        Arc::new(RealBatchAllocationService::new(db))
    }

    fn get_event_publisher(&self) -> Arc<dyn IEventPublisher> {
        Arc::new(SimpleEventPublisher::new())
    }

    fn get_persistence_service(&self) -> Arc<dyn IPersistenceService> {
        self.persistence_service.clone()
    }
}

/* MockServiceContainer and related code removed to eliminate mocks
// MockServiceContainer removed
    mock_suite: crate::domain::services::mocks::MockServiceSuite,
}

// impl MockServiceContainer removed
    pub fn new() -> Result<Self, AppError> {
        let mock_suite = crate::domain::services::mocks::MockFactory::create_full_mock_suite(None);
        Ok(Self { mock_suite })
    }

    pub fn with_config(config: crate::domain::services::mocks::MockConfig) -> Result<Self, AppError> {
        let mock_suite = crate::domain::services::mocks::MockFactory::create_full_mock_suite(Some(config));
        Ok(Self { mock_suite })
    }
}

// impl ServiceContainer for MockServiceContainer removed
    fn get_test_orchestration_service(&self) -> Arc<dyn ITestOrchestrationService> {
        Arc::new(self.mock_suite.test_orchestration.clone())
    }

    fn get_channel_state_manager(&self) -> Arc<dyn IChannelStateManager> {
        Arc::new(self.mock_suite.channel_state_manager.clone())
    }

    fn get_test_execution_engine(&self) -> Arc<dyn ITestExecutionEngine> {
        Arc::new(self.mock_suite.test_execution_engine.clone())
    }

    fn get_plc_communication_service(&self) -> Arc<dyn IPlcCommunicationService> {
        // 在Mock容器中也使用真实的Modbus PLC服务
        crate::infrastructure::plc_communication::global_plc_service()
    }

    fn get_batch_allocation_service(&self) -> Arc<dyn IBatchAllocationService> {
        Arc::new(self.mock_suite.batch_allocation.clone())
    }

    fn get_event_publisher(&self) -> Arc<dyn IEventPublisher> {
        Arc::new(self.mock_suite.event_publisher.clone())
    }

    */

/// 容器工厂
pub struct ContainerFactory;

impl ContainerFactory {
    /// 创建生产环境容器
    pub fn create_production_container() -> Result<Box<dyn ServiceContainer>, AppError> {
        let container = AppServiceContainer::new()?;
        Ok(Box::new(container))
    }

    /// 创建测试环境容器
    pub fn create_test_container() -> Result<Box<dyn ServiceContainer>, AppError> {
        Self::create_production_container()
    }

    /// 从配置创建容器
    pub fn create_from_config(config_path: &str) -> Result<Box<dyn ServiceContainer>, AppError> {
        let container = AppServiceContainer::from_config(config_path)?;
        Ok(Box::new(container))
    }

    /// 根据环境变量创建容器
    pub fn create_from_environment() -> Result<Box<dyn ServiceContainer>, AppError> {
        let env = std::env::var("RUST_ENV").unwrap_or_else(|_| "development".to_string());

        match env.as_str() {
            "production" => Self::create_production_container(),
            "test" => Self::create_test_container(),
            _ => {
                // 开发环境，尝试从配置文件加载
                if std::path::Path::new("config/development.toml").exists() {
                    Self::create_from_config("config/development")
                } else {
                    Self::create_production_container()
                }
            }
        }
    }
}
