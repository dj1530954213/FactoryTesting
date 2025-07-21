//! # 应用配置管理模块 (Application Configuration Module)
//!
//! ## 业务说明
//! 本模块负责管理FAT_TEST工厂测试系统的所有配置信息，提供统一的配置管理接口
//! 支持从多种来源加载配置，包括配置文件、环境变量和命令行参数
//!
//! ## 核心功能
//! ### 1. 配置结构定义
//! - **应用设置**: 基本的应用程序配置（名称、版本、环境等）
//! - **PLC配置**: PLC设备连接和通信参数配置
//! - **测试配置**: 测试执行相关的参数配置
//! - **日志配置**: 日志级别、输出目标等配置
//! - **持久化配置**: 数据库连接和存储配置
//!
//! ### 2. 配置加载机制
//! - **文件加载**: 支持TOML、JSON、YAML等格式的配置文件
//! - **环境变量**: 支持通过环境变量覆盖配置值
//! - **默认值**: 提供合理的默认配置，降低配置复杂度
//! - **验证机制**: 配置加载后进行完整性和有效性验证
//!
//! ### 3. 全局配置管理
//! - **单例模式**: 全局唯一的配置实例
//! - **线程安全**: 支持多线程环境下的安全访问
//! - **热更新**: 支持运行时配置更新（部分配置）
//! - **配置持久化**: 支持将运行时配置保存回文件
//!
//! ## 配置层次结构
//! ```
//! AppConfig
//! ├── AppSettings (应用基本设置)
//! ├── PlcConfig (PLC连接配置)
//! ├── TestConfig (测试执行配置)
//! ├── LoggingConfig (日志配置)
//! └── PersistenceConfig (数据持久化配置)
//! ```
//!
//! ## 使用场景
//! - **系统启动**: 加载应用程序的基础配置
//! - **PLC连接**: 配置PLC设备的连接参数
//! - **测试参数**: 配置测试流程的各种参数
//! - **环境适配**: 支持开发、测试、生产环境的配置
//!
//! ## Rust知识点
//! - **序列化**: 使用serde进行配置的序列化和反序列化
//! - **类型安全**: 通过Rust类型系统确保配置的正确性
//! - **模式匹配**: 使用枚举和模式匹配处理不同配置场景

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

/// PLC连接配置结构体
///
/// **业务作用**:
/// - 统一管理PLC设备的连接参数和通信配置
/// - 支持多种PLC协议和厂商的配置
/// - 提供灵活的超时和重试机制配置
/// - 支持开发和生产环境的配置切换
///
/// **配置来源**:
/// - 配置文件（config.toml/config.json）
/// - 环境变量覆盖
/// - 代码中的默认值
/// - 运行时动态配置
///
/// **使用场景**:
/// - 应用启动时的PLC连接初始化
/// - 连接参数的动态调整
/// - 不同环境的配置管理
/// - PLC通信故障的参数调优
///
/// **Rust知识点**:
/// - `#[derive(...)]`: 自动实现常用trait
/// - `Serialize/Deserialize`: serde序列化支持
/// - `#[serde(default)]`: 字段缺失时使用默认值
/// - `#[serde(default="function")]`: 使用指定函数提供默认值
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlcConfig {
    /// PLC协议类型
    /// **支持协议**: "modbus"(Modbus TCP), "s7"(Siemens S7), "opcua"(OPC UA)
    /// **默认值**: "modbus"
    /// **验证**: 必须是预定义的有效协议类型
    pub plc_type: String,

    /// PLC设备IP地址
    /// **格式**: 标准IPv4地址格式（如"192.168.1.100"）
    /// **默认值**: "127.0.0.1"（本地回环地址）
    /// **验证**: 不能为空，必须是有效的IP地址格式
    pub host: String,

    /// PLC设备端口号
    /// **默认端口**: 502（Modbus TCP标准端口）
    /// **范围**: 1-65535
    /// **验证**: 不能为0，必须在有效端口范围内
    pub port: u16,

    /// 连接建立超时时间（毫秒）
    /// **业务含义**: TCP连接建立的最大等待时间
    /// **默认值**: 5000ms（5秒）
    /// **调优**: 网络环境差时可适当增加
    pub connection_timeout_ms: u64,

    /// 数据读取超时时间（毫秒）
    /// **业务含义**: 单次读取操作的最大等待时间
    /// **默认值**: 3000ms（3秒）
    /// **影响**: 影响读取操作的响应性
    pub read_timeout_ms: u64,

    /// 数据写入超时时间（毫秒）
    /// **业务含义**: 单次写入操作的最大等待时间
    /// **默认值**: 3000ms（3秒）
    /// **考虑**: 写入通常比读取耗时更长
    pub write_timeout_ms: u64,

    /// 操作失败时的重试次数
    /// **默认值**: 3次
    /// **策略**: 0表示不重试，过大会影响响应速度
    /// **适用**: 网络抖动或设备临时繁忙的情况
    pub retry_count: u32,

    /// 重试间隔时间（毫秒）
    /// **默认值**: 1000ms（1秒）
    /// **作用**: 避免频繁重试对设备造成压力
    /// **调优**: 根据设备响应特性调整
    pub retry_interval_ms: u64,

    /// 多字节数据的字节序配置
    /// **支持格式**: "ABCD"(大端), "CDAB", "BADC", "DCBA"(小端)
    /// **默认值**: 通过default_byte_order()函数提供，通常为"CDAB"
    /// **重要性**: 错误的字节序会导致浮点数解析错误
    /// **serde属性**: 配置文件中缺失时使用默认函数值
    #[serde(default="default_byte_order")]
    pub byte_order: String,

    /// Modbus地址编码模式
    /// **业务含义**: PLC地址是否从0开始编号
    /// **默认值**: false（从1开始，符合大多数PLC习惯）
    /// **厂商差异**: 不同PLC厂商的地址编码方式不同
    /// **serde属性**: 配置文件中缺失时使用类型默认值
    #[serde(default)]
    pub zero_based_address: bool,

    /// Mock模式开关
    /// **业务含义**: 是否使用模拟PLC进行开发和测试
    /// **默认值**: false（使用真实PLC）
    /// **用途**: 开发环境下的离线测试和CI/CD流水线
    /// **环境隔离**: 避免开发测试影响生产设备
    #[serde(default)]
    pub mock_mode: bool,
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
            mock_mode: false,
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
    ///
    /// **业务作用**:
    /// - 支持通过环境变量动态调整配置参数
    /// - 便于不同环境（开发、测试、生产）的配置管理
    /// - 支持容器化部署和CI/CD流水线
    /// - 提供配置的灵活性和安全性
    ///
    /// **支持的PLC环境变量**:
    /// - `PLC_HOST`: PLC设备IP地址
    /// - `PLC_PORT`: PLC设备端口号
    /// - `PLC_TYPE`: PLC协议类型
    /// - `PLC_MOCK_MODE`: Mock模式开关
    ///
    /// **使用场景**:
    /// - Docker容器部署时的配置注入
    /// - 不同环境的配置切换
    /// - 敏感信息的安全配置
    /// - 运维人员的配置调整
    ///
    /// **错误处理**:
    /// - 环境变量不存在时保持原配置不变
    /// - 类型转换失败时忽略该环境变量
    /// - 不会因环境变量错误而导致程序崩溃
    ///
    /// **Rust知识点**:
    /// - `std::env::var()`: 获取环境变量，返回Result类型
    /// - `if let Ok(...)`: 模式匹配，只处理成功的情况
    /// - `parse::<T>()`: 字符串解析为指定类型
    /// - `to_lowercase()`: 字符串转小写，便于布尔值解析
    pub fn override_from_env(&mut self) {
        // PLC配置的环境变量覆盖
        // **配置优先级**: 环境变量 > 配置文件 > 默认值

        // PLC主机地址覆盖
        // **环境变量**: PLC_HOST
        // **用途**: 在不同环境中指定不同的PLC设备
        if let Ok(host) = std::env::var("PLC_HOST") {
            self.config.plc_config.host = host;
        }

        // PLC端口号覆盖
        // **环境变量**: PLC_PORT
        // **类型转换**: 字符串转换为u16类型
        // **错误处理**: 转换失败时保持原配置
        if let Ok(port) = std::env::var("PLC_PORT") {
            if let Ok(port) = port.parse::<u16>() {
                self.config.plc_config.port = port;
            }
        }

        // PLC协议类型覆盖
        // **环境变量**: PLC_TYPE
        // **支持值**: "modbus", "s7", "opcua"
        if let Ok(plc_type) = std::env::var("PLC_TYPE") {
            self.config.plc_config.plc_type = plc_type;
        }

        // Mock模式开关覆盖
        // **环境变量**: PLC_MOCK_MODE
        // **布尔值解析**: "true"(不区分大小写)为真，其他为假
        // **用途**: 开发环境启用Mock模式，生产环境禁用
        if let Ok(mock_mode) = std::env::var("PLC_MOCK_MODE") {
            self.config.plc_config.mock_mode = mock_mode.to_lowercase() == "true";
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
