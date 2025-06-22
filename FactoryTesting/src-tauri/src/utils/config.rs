use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use crate::utils::error::{AppError, AppResult};

/// 提供给 serde 的默认字节顺序（CDAB）
fn default_byte_order() -> String {
    "CDAB".to_string()
}

/// 应用程序主配置结构
/// 包含应用程序运行所需的所有配置信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// 应用程序基本设置
    pub app_settings: AppSettings,
    /// PLC连接配置
    pub plc_config: PlcConfig,
    /// 测试配置
    pub test_config: TestConfig,
    /// 日志配置
    pub logging_config: LoggingConfig,
    /// 数据存储配置
    pub persistence_config: PersistenceConfig,
}

/// 应用程序基本设置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    /// 应用程序名称
    pub app_name: String,
    /// 应用程序版本
    pub app_version: String,
    /// 运行环境 (development, testing, production)
    pub environment: String,
    /// 是否启用调试模式
    pub debug_mode: bool,
    /// 工作目录
    pub work_directory: Option<PathBuf>,
    /// 最大并发任务数
    pub max_concurrent_tasks: usize,
    /// 操作超时时间（毫秒）
    pub default_timeout_ms: u64,
}

/// PLC连接配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlcConfig {
    /// PLC类型 (modbus, s7, opcua)
    pub plc_type: String,
    /// PLC IP地址
    pub host: String,
    /// PLC端口
    pub port: u16,
    /// 连接超时时间（毫秒）
    pub connection_timeout_ms: u64,
    /// 读取超时时间（毫秒）
    pub read_timeout_ms: u64,
    /// 写入超时时间（毫秒）
    pub write_timeout_ms: u64,
    /// 重试次数
    pub retry_count: u32,
    /// 重试间隔（毫秒）
    pub retry_interval_ms: u64,
    /// 字节顺序配置 (ABCD / CDAB / BADC / DCBA)
    #[serde(default="default_byte_order")]
    pub byte_order: String,
    /// Modbus 地址是否使用 0 基
    #[serde(default)]
    pub zero_based_address: bool,
}

/// 测试配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestConfig {
    /// 测试点位默认稳定时间（毫秒）
    pub default_stabilization_time_ms: u64,
    /// 模拟量测试精度容差
    pub analog_tolerance_percent: f32,
    /// 数字量测试稳定时间（毫秒）
    pub digital_stabilization_time_ms: u64,
    /// 报警测试等待时间（毫秒）
    pub alarm_test_wait_time_ms: u64,
    /// 是否自动跳过不适用的测试
    pub auto_skip_not_applicable: bool,
    /// 测试失败时是否自动重试
    pub auto_retry_on_failure: bool,
    /// 自动重试次数
    pub auto_retry_count: u32,
    /// 批量测试的批次大小
    pub batch_test_size: usize,
}

/// 日志配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// 日志级别 (debug, info, warn, error)
    pub log_level: String,
    /// 日志文件路径
    pub log_file_path: Option<PathBuf>,
    /// 是否启用控制台输出
    pub console_output: bool,
    /// 是否启用文件输出
    pub file_output: bool,
    /// 日志文件最大大小（MB）
    pub max_file_size_mb: u64,
    /// 保留的日志文件数量
    pub max_files: usize,
}

/// 数据持久化配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistenceConfig {
    /// 持久化类型 (json, sqlite, memory)
    pub persistence_type: String,
    /// 数据文件路径
    pub data_path: PathBuf,
    /// 是否启用自动备份
    pub auto_backup: bool,
    /// 备份间隔（小时）
    pub backup_interval_hours: u64,
    /// 保留的备份文件数量
    pub max_backups: usize,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            app_settings: AppSettings::default(),
            plc_config: PlcConfig::default(),
            test_config: TestConfig::default(),
            logging_config: LoggingConfig::default(),
            persistence_config: PersistenceConfig::default(),
        }
    }
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            app_name: "FactoryTesting".to_string(),
            app_version: "1.0.0".to_string(),
            environment: "development".to_string(),
            debug_mode: true,
            work_directory: None,
            max_concurrent_tasks: 10,
            default_timeout_ms: 30000,
        }
    }
}

impl Default for PlcConfig {
    fn default() -> Self {
        Self {
            plc_type: "modbus".to_string(),
            host: "127.0.0.1".to_string(),
            port: 502,
            connection_timeout_ms: 5000,
            read_timeout_ms: 3000,
            write_timeout_ms: 3000,
            retry_count: 3,
            retry_interval_ms: 1000,
            byte_order: "CDAB".to_string(),
            zero_based_address: false,
        }
    }
}

impl Default for TestConfig {
    fn default() -> Self {
        Self {
            default_stabilization_time_ms: 2000,
            analog_tolerance_percent: 1.0,
            digital_stabilization_time_ms: 500,
            alarm_test_wait_time_ms: 5000,
            auto_skip_not_applicable: true,
            auto_retry_on_failure: false,
            auto_retry_count: 1,
            batch_test_size: 20,
        }
    }
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            log_level: "info".to_string(),
            log_file_path: Some(PathBuf::from("logs/app.log")),
            console_output: true,
            file_output: true,
            max_file_size_mb: 10,
            max_files: 5,
        }
    }
}

impl Default for PersistenceConfig {
    fn default() -> Self {
        Self {
            persistence_type: "json".to_string(),
            data_path: PathBuf::from("data"),
            auto_backup: true,
            backup_interval_hours: 24,
            max_backups: 7,
        }
    }
}

/// 配置管理器
/// 负责加载、保存和管理应用程序配置
pub struct ConfigManager {
    config: AppConfig,
    config_file_path: PathBuf,
}

impl ConfigManager {
    /// 创建新的配置管理器
    pub fn new(config_file_path: PathBuf) -> Self {
        Self {
            config: AppConfig::default(),
            config_file_path,
        }
    }

    /// 从文件加载配置
    pub async fn load_from_file(&mut self) -> AppResult<()> {
        if !self.config_file_path.exists() {
            // 如果配置文件不存在，创建默认配置文件
            self.save_to_file().await?;
            return Ok(());
        }

        let content = tokio::fs::read_to_string(&self.config_file_path)
            .await
            .map_err(|e| AppError::io_error(format!("读取配置文件失败: {}", e), e.kind().to_string()))?;

        self.config = serde_json::from_str(&content)
            .map_err(|e| AppError::configuration_error(format!("解析配置文件失败: {}", e)))?;

        Ok(())
    }

    /// 将配置保存到文件
    pub async fn save_to_file(&self) -> AppResult<()> {
        // 确保目录存在
        if let Some(parent) = self.config_file_path.parent() {
            tokio::fs::create_dir_all(parent).await
                .map_err(|e| AppError::io_error(format!("创建配置目录失败: {}", e), e.kind().to_string()))?;
        }

        let content = serde_json::to_string_pretty(&self.config)
            .map_err(|e| AppError::json_error(format!("序列化配置失败: {}", e)))?;

        tokio::fs::write(&self.config_file_path, content)
            .await
            .map_err(|e| AppError::io_error(format!("写入配置文件失败: {}", e), e.kind().to_string()))?;

        Ok(())
    }

    /// 从环境变量覆盖配置
    pub fn override_from_env(&mut self) {
        // PLC 配置
        if let Ok(host) = std::env::var("PLC_HOST") {
            self.config.plc_config.host = host;
        }
        if let Ok(port) = std::env::var("PLC_PORT") {
            if let Ok(port) = port.parse::<u16>() {
                self.config.plc_config.port = port;
            }
        }
        if let Ok(plc_type) = std::env::var("PLC_TYPE") {
            self.config.plc_config.plc_type = plc_type;
        }


        // 应用程序设置
        if let Ok(env) = std::env::var("APP_ENVIRONMENT") {
            self.config.app_settings.environment = env;
        }
        if let Ok(debug) = std::env::var("DEBUG_MODE") {
            self.config.app_settings.debug_mode = debug.to_lowercase() == "true";
        }
        if let Ok(log_level) = std::env::var("LOG_LEVEL") {
            self.config.logging_config.log_level = log_level;
        }

        // 数据路径
        if let Ok(data_path) = std::env::var("DATA_PATH") {
            self.config.persistence_config.data_path = PathBuf::from(data_path);
        }
    }

    /// 获取配置的只读引用
    pub fn get_config(&self) -> &AppConfig {
        &self.config
    }

    /// 获取配置的可变引用
    pub fn get_config_mut(&mut self) -> &mut AppConfig {
        &mut self.config
    }

    /// 验证配置的有效性
    pub fn validate_config(&self) -> AppResult<()> {
        // 验证PLC配置
        if self.config.plc_config.host.is_empty() {
            return Err(AppError::configuration_error("PLC主机地址不能为空"));
        }

        if self.config.plc_config.port == 0 {
            return Err(AppError::configuration_error("PLC端口号不能为0"));
        }

        // 验证环境配置
        let valid_environments = ["development", "testing", "production"];
        if !valid_environments.contains(&self.config.app_settings.environment.as_str()) {
            return Err(AppError::configuration_error(format!(
                "无效的环境配置: {}，有效值: {:?}",
                self.config.app_settings.environment, valid_environments
            )));
        }

        // 验证日志级别
        let valid_log_levels = ["debug", "info", "warn", "error"];
        if !valid_log_levels.contains(&self.config.logging_config.log_level.as_str()) {
            return Err(AppError::configuration_error(format!(
                "无效的日志级别: {}，有效值: {:?}",
                self.config.logging_config.log_level, valid_log_levels
            )));
        }

        // 验证PLC类型
        let valid_plc_types = ["modbus", "s7", "opcua"];
        if !valid_plc_types.contains(&self.config.plc_config.plc_type.as_str()) {
            return Err(AppError::configuration_error(format!(
                "无效的PLC类型: {}，有效值: {:?}",
                self.config.plc_config.plc_type, valid_plc_types
            )));
        }

        Ok(())
    }

    /// 重置为默认配置
    pub fn reset_to_default(&mut self) {
        self.config = AppConfig::default();
    }
}

/// 全局配置管理器实例
/// 使用 lazy_static 确保全局唯一性
use std::sync::Mutex;
use std::sync::OnceLock;

static GLOBAL_CONFIG: OnceLock<Mutex<ConfigManager>> = OnceLock::new();

/// 初始化全局配置管理器
pub async fn init_global_config(config_path: Option<PathBuf>) -> AppResult<()> {
    let config_path = config_path.unwrap_or_else(|| PathBuf::from("config/app_config.json"));
    let mut config_manager = ConfigManager::new(config_path);
    
    // 从文件加载配置
    config_manager.load_from_file().await?;
    
    // 从环境变量覆盖配置
    config_manager.override_from_env();
    
    // 验证配置
    config_manager.validate_config()?;
    
    // 设置全局配置
    GLOBAL_CONFIG
        .set(Mutex::new(config_manager))
        .map_err(|_| AppError::configuration_error("全局配置已经初始化"))?;
    
    Ok(())
}

/// 获取全局配置的只读访问
pub fn get_global_config() -> AppResult<AppConfig> {
    let config_manager = GLOBAL_CONFIG
        .get()
        .ok_or_else(|| AppError::configuration_error("全局配置未初始化"))?
        .lock()
        .map_err(|_| AppError::concurrency_error("获取全局配置锁失败"))?;
    
    Ok(config_manager.get_config().clone())
}

/// 更新全局配置
pub async fn update_global_config<F>(updater: F) -> AppResult<()>
where
    F: FnOnce(&mut AppConfig),
{
    let config_manager = GLOBAL_CONFIG
        .get()
        .ok_or_else(|| AppError::configuration_error("全局配置未初始化"))?;
    
    {
        let mut manager = config_manager
            .lock()
            .map_err(|_| AppError::concurrency_error("获取全局配置锁失败"))?;
        
        updater(manager.get_config_mut());
        manager.validate_config()?;
        manager.save_to_file().await?;
    }
    
    Ok(())
} 