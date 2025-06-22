//! 依赖注入容器
//!
//! 负责管理所有服务的生命周期和依赖关系

use std::sync::Arc;
use sea_orm::{Database, DatabaseConnection};

use crate::services::application::batch_allocation_service::BatchAllocationService;
use crate::services::application::test_orchestration_service::TestOrchestrationService;
use crate::services::domain::test_execution_engine::{ITestExecutionEngine, TestExecutionEngine};
use crate::services::domain::test_plc_config_service::{ITestPlcConfigService, TestPlcConfigService};
use crate::services::domain::channel_state_manager::{IChannelStateManager, ChannelStateManager};
use crate::services::infrastructure::persistence::{SqliteOrmPersistenceService, PersistenceConfig};
use crate::domain::services::*;
use crate::services::infrastructure::IPlcCommunicationService;
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
            database_url: "sqlite:../fat_test.db".to_string(), // Adjusted path for being in src-tauri
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
        "sqlite:../fat_test.db" // Adjusted path
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
    db_connection: Arc<DatabaseConnection>,
    persistence_service: Arc<dyn IPersistenceService>,
    test_plc_config_service: Arc<dyn ITestPlcConfigService>,
    channel_state_manager: Arc<dyn IChannelStateManager>,
    test_execution_engine: Arc<dyn ITestExecutionEngine>,
    batch_allocation_service: Arc<dyn IBatchAllocationService>,
    test_orchestration_service: Arc<dyn ITestOrchestrationService>,
}

impl AppServiceContainer {
    /// 创建新的服务容器
    pub async fn new() -> Result<Self, AppError> {
        let config = Arc::new(DefaultAppConfig::default());
        
        // 创建数据库连接
        let conn = Database::connect(config.database_url()).await
            .map_err(|e| AppError::database_error(format!("Failed to connect to database: {}", e)))?;
        let db_connection = Arc::new(conn);
        
        // 实例化持久化服务
        let persistence_service = Arc::new(SqliteOrmPersistenceService::with_connection(db_connection.clone(), PersistenceConfig::default()));

        // 实例化领域服务
        let test_plc_config_service = Arc::new(TestPlcConfigService::new(persistence_service.clone()));
        let channel_state_manager = Arc::new(ChannelStateManager::new(persistence_service.clone()));
        let test_execution_engine = Arc::new(TestExecutionEngine::new(
            config.max_concurrent_tests(),
            test_plc_config_service.clone(),
        ));
        let batch_allocation_service = Arc::new(BatchAllocationService::new(db_connection.clone()));

        // EventPublisher和PersistenceService暂时使用Mock
        let event_publisher_mock = Arc::new(crate::domain::services::mocks::MockEventPublisher::new(Default::default()));
        let persistence_service_mock = Arc::new(crate::domain::services::mocks::MockPersistenceService::new(Default::default()));

        let test_orchestration_service = Arc::new(TestOrchestrationService::new(
            channel_state_manager.clone(),
            test_execution_engine.clone(),
            batch_allocation_service.clone(),
            event_publisher_mock,
            persistence_service_mock,
        ));

        Ok(Self { 
            config,
            db_connection,
            persistence_service,
            test_plc_config_service,
            channel_state_manager,
            test_execution_engine,
            batch_allocation_service,
            test_orchestration_service,
        })
    }

    /// 从配置文件创建服务容器
    pub async fn from_config(config_path: &str) -> Result<Self, AppError> {
        let settings = config::Config::builder()
            .add_source(config::File::with_name(config_path))
            .build()
            .map_err(|e| AppError::configuration_error(e.to_string()))?;

        let config = Arc::new(ConfigBasedAppConfig::new(settings));
        
        // 创建数据库连接
        let conn = Database::connect(config.database_url()).await
            .map_err(|e| AppError::database_error(format!("Failed to connect to database: {}", e)))?;
        let db_connection = Arc::new(conn);
        
        // 实例化持久化服务
        let persistence_service = Arc::new(SqliteOrmPersistenceService::with_connection(db_connection.clone(), PersistenceConfig::default()));

        // 实例化领域服务
        let test_plc_config_service = Arc::new(TestPlcConfigService::new(persistence_service.clone()));
        let channel_state_manager = Arc::new(ChannelStateManager::new(persistence_service.clone()));
        let test_execution_engine = Arc::new(TestExecutionEngine::new(
            config.max_concurrent_tests(),
            test_plc_config_service.clone(),
        ));
        let batch_allocation_service = Arc::new(BatchAllocationService::new(db_connection.clone()));
        
        // EventPublisher和PersistenceService暂时使用Mock
        let event_publisher_mock = Arc::new(crate::domain::services::mocks::MockEventPublisher::new(Default::default()));
        let persistence_service_mock = Arc::new(crate::domain::services::mocks::MockPersistenceService::new(Default::default()));

        let test_orchestration_service = Arc::new(TestOrchestrationService::new(
            channel_state_manager.clone(),
            test_execution_engine.clone(),
            batch_allocation_service.clone(),
            event_publisher_mock,
            persistence_service_mock,
        ));
        
        Ok(Self { 
            config,
            db_connection,
            persistence_service,
            test_plc_config_service,
            channel_state_manager,
            test_execution_engine,
            batch_allocation_service,
            test_orchestration_service,
        })
    }

    /// 创建Mock模式的服务容器
    pub fn create_mock_container() -> Result<MockServiceContainer, AppError> {
        MockServiceContainer::new()
    }
}

impl ServiceContainer for AppServiceContainer {
    fn get_test_orchestration_service(&self) -> Arc<dyn ITestOrchestrationService> {
        self.test_orchestration_service.clone()
    }

    fn get_channel_state_manager(&self) -> Arc<dyn IChannelStateManager> {
        self.channel_state_manager.clone()
    }

    fn get_test_execution_engine(&self) -> Arc<dyn ITestExecutionEngine> {
        self.test_execution_engine.clone()
    }

    fn get_plc_communication_service(&self) -> Arc<dyn IPlcCommunicationService> {
        // 使用默认配置创建Modbus PLC服务
        let config = crate::services::infrastructure::plc::modbus_plc_service::ModbusConfig {
            ip_address: "192.168.1.100".to_string(),
            port: 502,
            slave_id: 1,
            byte_order: crate::models::ByteOrder::default(),
            zero_based_address: false,
            connection_timeout_ms: 5000,
            read_timeout_ms: 3000,
            write_timeout_ms: 3000,
        };
        Arc::new(crate::services::infrastructure::plc::modbus_plc_service::ModbusPlcService::new(config))
    }

    fn get_batch_allocation_service(&self) -> Arc<dyn IBatchAllocationService> {
        self.batch_allocation_service.clone()
    }

    fn get_event_publisher(&self) -> Arc<dyn IEventPublisher> {
        Arc::new(crate::domain::services::mocks::MockEventPublisher::new(
            crate::domain::services::mocks::MockConfig::default()
        ))
    }

    fn get_persistence_service(&self) -> Arc<dyn IPersistenceService> {
        Arc::new(crate::domain::services::mocks::MockPersistenceService::new(
            crate::domain::services::mocks::MockConfig::default()
        ))
    }
}

/// Mock服务容器
pub struct MockServiceContainer {
    mock_suite: crate::domain::services::mocks::MockServiceSuite,
}

impl MockServiceContainer {
    pub fn new() -> Result<Self, AppError> {
        let mock_suite = crate::domain::services::mocks::MockFactory::create_full_mock_suite(None);
        Ok(Self { mock_suite })
    }

    pub fn with_config(config: crate::domain::services::mocks::MockConfig) -> Result<Self, AppError> {
        let mock_suite = crate::domain::services::mocks::MockFactory::create_full_mock_suite(Some(config));
        Ok(Self { mock_suite })
    }
}

impl ServiceContainer for MockServiceContainer {
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
        let config = crate::services::infrastructure::plc::modbus_plc_service::ModbusConfig {
            ip_address: "192.168.1.100".to_string(),
            port: 502,
            slave_id: 1,
            byte_order: crate::models::ByteOrder::default(),
            zero_based_address: false,
            connection_timeout_ms: 5000,
            read_timeout_ms: 3000,
            write_timeout_ms: 3000,
        };
        Arc::new(crate::services::infrastructure::plc::modbus_plc_service::ModbusPlcService::new(config))
    }

    fn get_batch_allocation_service(&self) -> Arc<dyn IBatchAllocationService> {
        Arc::new(self.mock_suite.batch_allocation.clone())
    }

    fn get_event_publisher(&self) -> Arc<dyn IEventPublisher> {
        Arc::new(self.mock_suite.event_publisher.clone())
    }

    fn get_persistence_service(&self) -> Arc<dyn IPersistenceService> {
        Arc::new(self.mock_suite.persistence.clone())
    }
}

/// 容器工厂
pub struct ContainerFactory;

impl ContainerFactory {
    /// 创建生产环境容器
    pub async fn create_production_container() -> Result<Box<dyn ServiceContainer>, AppError> {
        let container = AppServiceContainer::new().await?;
        Ok(Box::new(container))
    }

    /// 创建测试环境容器
    pub fn create_test_container() -> Result<Box<dyn ServiceContainer>, AppError> {
        let container = MockServiceContainer::new()?;
        Ok(Box::new(container))
    }

    /// 从配置创建容器
    pub async fn create_from_config(config_path: &str) -> Result<Box<dyn ServiceContainer>, AppError> {
        let container = AppServiceContainer::from_config(config_path).await?;
        Ok(Box::new(container))
    }

    /// 根据环境变量创建容器
    pub async fn create_from_environment() -> Result<Box<dyn ServiceContainer>, AppError> {
        let env = std::env::var("RUST_ENV").unwrap_or_else(|_| "development".to_string());

        match env.as_str() {
            "production" => Self::create_production_container().await,
            "test" => Self::create_test_container(),
            _ => {
                // 开发环境，尝试从配置文件加载
                if std::path::Path::new("config/development.toml").exists() {
                    Self::create_from_config("config/development").await
                } else {
                    Self::create_production_container().await
                }
            }
        }
    }
}
