//! 企业级日志系统集成模块
//!
//! 集成所有文件管理组件，提供完整的企业级日志解决方案
//! 包括：文件管理、轮转、清理、并发安全和崩溃恢复

use super::{
    file_manager::{LogFileManager, RotationConfig, CleanupConfig, LogFileStats},
    advanced_file_writer::{AdvancedFileWriter, AdvancedFileWriterBuilder},
    cleanup_scheduler::{CleanupScheduler, CleanupSchedulerConfig, CleanupResult},
    logger_config::{LoggerConfig, LogTarget, LogFormat, LogLevel, StructuredLog, AsyncLogProcessor},
    global_logger_adapter::GlobalLoggerAdapter,
    ConsoleWriter, LogWriter,
};
use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{Arc, Mutex, RwLock},
    time::Duration,
};
use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex as AsyncMutex;

/// 企业级日志系统配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnterpriseLoggerConfig {
    /// 日志级别
    pub level: LogLevel,
    /// 基础目录
    pub base_dir: PathBuf,
    /// 文件前缀
    pub file_prefix: String,
    /// 文件格式
    pub format: LogFormat,
    /// 是否启用控制台输出
    pub console_enabled: bool,
    /// 是否启用文件输出
    pub file_enabled: bool,
    /// 轮转配置
    pub rotation: RotationConfig,
    /// 清理配置
    pub cleanup: CleanupConfig,
    /// 清理调度器配置
    pub scheduler: CleanupSchedulerConfig,
    /// 异步处理是否启用
    pub async_processing: bool,
    /// 异步批量大小
    pub async_batch_size: usize,
}

/// 企业级日志系统状态
#[derive(Debug, Clone)]
pub struct LoggerStatus {
    pub initialized: bool,
    pub file_writer_active: bool,
    pub scheduler_running: bool,
    pub total_logs_written: u64,
    pub total_files_rotated: u32,
    pub last_cleanup_time: Option<std::time::Instant>,
    pub file_stats: Option<LogFileStats>,
}

/// 企业级日志系统主类
pub struct EnterpriseLogger {
    config: EnterpriseLoggerConfig,
    file_manager: Option<Arc<LogFileManager>>,
    file_writer: Option<Arc<AdvancedFileWriter>>,
    console_writer: Option<Arc<ConsoleWriter>>,
    async_processor: Option<AsyncLogProcessor>,
    cleanup_scheduler: AsyncMutex<Option<CleanupScheduler>>,
    status: Arc<RwLock<LoggerStatus>>,
    initialized: Arc<Mutex<bool>>,
}

impl EnterpriseLogger {
    /// 创建新的企业级日志系统
    pub fn new(config: EnterpriseLoggerConfig) -> Self {
        Self {
            config,
            file_manager: None,
            file_writer: None,
            console_writer: None,
            async_processor: None,
            cleanup_scheduler: AsyncMutex::new(None),
            status: Arc::new(RwLock::new(LoggerStatus {
                initialized: false,
                file_writer_active: false,
                scheduler_running: false,
                total_logs_written: 0,
                total_files_rotated: 0,
                last_cleanup_time: None,
                file_stats: None,
            })),
            initialized: Arc::new(Mutex::new(false)),
        }
    }
    
    /// 初始化日志系统
    pub async fn initialize(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut initialized_guard = self.initialized.lock()
            .map_err(|_| "无法获取初始化锁")?;
        
        if *initialized_guard {
            info!("企业日志系统已经初始化");
            return Ok(());
        }
        
        info!("初始化企业级日志系统...");
        
        // 1. 初始化文件管理器
        if self.config.file_enabled {
            let file_manager = Arc::new(LogFileManager::new(
                self.config.base_dir.clone(),
                self.config.file_prefix.clone(),
                "log".to_string(),
                self.config.rotation.clone(),
                self.config.cleanup.clone(),
            )?);
            
            self.file_manager = Some(Arc::clone(&file_manager));
            
            // 创建高级文件写入器
            let file_writer = Arc::new(AdvancedFileWriterBuilder::new()
                .base_dir(self.config.base_dir.clone())
                .file_prefix(self.config.file_prefix.clone())
                .format(self.config.format.clone())
                .rotation_config(self.config.rotation.clone())
                .cleanup_config(self.config.cleanup.clone())
                .build()?);
            
            self.file_writer = Some(file_writer);
        }
        
        // 2. 初始化控制台写入器
        if self.config.console_enabled {
            self.console_writer = Some(Arc::new(ConsoleWriter));
        }
        
        // 3. 初始化异步处理器
        if self.config.async_processing {
            let mut writers: Vec<Arc<dyn LogWriter>> = Vec::new();
            
            if let Some(console_writer) = &self.console_writer {
                writers.push(Arc::clone(console_writer) as Arc<dyn LogWriter>);
            }
            
            if let Some(file_writer) = &self.file_writer {
                writers.push(Arc::clone(file_writer) as Arc<dyn LogWriter>);
            }
            
            self.async_processor = Some(AsyncLogProcessor::new(writers));
        }
        
        // 4. 初始化清理调度器
        if let Some(file_manager) = &self.file_manager {
            let mut scheduler = CleanupScheduler::new(
                Arc::clone(file_manager),
                self.config.scheduler.clone(),
            );
            
            scheduler.start()?;
            
            let mut scheduler_guard = self.cleanup_scheduler.lock().await;
            *scheduler_guard = Some(scheduler);
        }
        
        // 5. 设置为全局logger
        self.setup_global_logger()?;
        
        // 6. 更新状态
        {
            let mut status = self.status.write()
                .map_err(|_| "无法获取状态写锁")?;
            
            status.initialized = true;
            status.file_writer_active = self.file_writer.is_some();
            status.scheduler_running = self.cleanup_scheduler.try_lock()
                .map(|guard| guard.as_ref().map_or(false, |s| s.is_running()))
                .unwrap_or(false);
        }
        
        *initialized_guard = true;
        
        info!("企业级日志系统初始化完成");
        info!("配置: 控制台={}, 文件={}, 异步={}, 清理调度器={}",
              self.config.console_enabled,
              self.config.file_enabled,
              self.config.async_processing,
              self.config.scheduler.auto_cleanup_enabled);
        
        // 记录初始化成功的测试日志
        log::info!("企业级日志系统工作正常 - 支持按日期组织、自动轮转、定期清理");
        
        Ok(())
    }
    
    /// 设置全局logger
    fn setup_global_logger(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if let Some(processor) = &self.async_processor {
            let adapter = Box::new(GlobalLoggerAdapter::new(processor));
            log::set_boxed_logger(adapter)?;
            log::set_max_level(self.config.level.clone().into());
            return Ok(());
        }
        
        // 如果没有异步处理器，使用简单logger
        if self.config.console_enabled {
            env_logger::Builder::from_default_env()
                .filter_level(self.config.level.clone().into())
                .init();
        }
        
        Ok(())
    }
    
    /// 获取当前系统状态
    pub fn get_status(&self) -> Result<LoggerStatus, Box<dyn std::error::Error + Send + Sync>> {
        let mut status = self.status.read()
            .map_err(|_| "无法获取状态读锁")?
            .clone();
        
        // 更新文件统计
        if let Some(file_writer) = &self.file_writer {
            status.file_stats = file_writer.get_file_stats().ok();
            let write_stats = file_writer.get_stats();
            status.total_logs_written = write_stats.total_writes;
        }
        
        Ok(status)
    }
    
    /// 手动执行日志清理
    pub async fn run_cleanup(&self) -> Result<CleanupResult, Box<dyn std::error::Error + Send + Sync>> {
        let scheduler_guard = self.cleanup_scheduler.lock().await;
        
        match scheduler_guard.as_ref() {
            Some(scheduler) => {
                let result = scheduler.run_cleanup_now().await?;
                
                // 更新最后清理时间
                if let Ok(mut status) = self.status.write() {
                    status.last_cleanup_time = Some(std::time::Instant::now());
                }
                
                Ok(result)
            }
            None => Err("清理调度器未初始化".into()),
        }
    }
    
    /// 强制轮转当前日志文件
    pub fn rotate_logs(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if let Some(file_writer) = &self.file_writer {
            file_writer.force_flush_and_rotate()?;
            
            // 更新轮转统计
            if let Ok(mut status) = self.status.write() {
                status.total_files_rotated += 1;
            }
            
            info!("手动轮转日志文件完成");
        }
        
        Ok(())
    }
    
    /// 获取文件系统统计信息
    pub fn get_file_stats(&self) -> Result<LogFileStats, Box<dyn std::error::Error + Send + Sync>> {
        match &self.file_writer {
            Some(file_writer) => file_writer.get_file_stats(),
            None => Err("文件写入器未初始化".into()),
        }
    }
    
    /// 刷新所有日志
    pub fn flush_all(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if let Some(processor) = &self.async_processor {
            processor.flush();
        }
        
        if let Some(file_writer) = &self.file_writer {
            file_writer.flush()?;
        }
        
        if let Some(console_writer) = &self.console_writer {
            console_writer.flush()?;
        }
        
        debug!("所有日志写入器已刷新");
        Ok(())
    }
    
    /// 关闭日志系统
    pub async fn shutdown(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!("关闭企业级日志系统...");
        
        // 1. 先刷新所有待处理的日志
        let _ = self.flush_all();
        
        // 2. 关闭清理调度器
        {
            let mut scheduler_guard = self.cleanup_scheduler.lock().await;
            if let Some(mut scheduler) = scheduler_guard.take() {
                scheduler.stop().await?;
            }
        }
        
        // 3. 关闭异步处理器
        if let Some(processor) = &self.async_processor {
            processor.shutdown();
        }
        
        // 4. 更新状态
        {
            let mut status = self.status.write()
                .map_err(|_| "无法获取状态写锁")?;
            status.initialized = false;
            status.file_writer_active = false;
            status.scheduler_running = false;
        }
        
        {
            let mut initialized = self.initialized.lock()
                .map_err(|_| "无法获取初始化锁")?;
            *initialized = false;
        }
        
        info!("企业级日志系统已关闭");
        Ok(())
    }
    
    /// 获取配置信息
    pub fn get_config(&self) -> &EnterpriseLoggerConfig {
        &self.config
    }
    
    /// 更新配置（需要重新初始化）
    pub fn update_config(&mut self, new_config: EnterpriseLoggerConfig) {
        warn!("更新日志配置，需要重新初始化系统");
        self.config = new_config;
        
        // 清空初始化状态
        if let Ok(mut initialized) = self.initialized.lock() {
            *initialized = false;
        }
    }
}

/// 企业级日志系统构建器
pub struct EnterpriseLoggerBuilder {
    config: EnterpriseLoggerConfig,
}

impl Default for EnterpriseLoggerBuilder {
    fn default() -> Self {
        Self {
            config: EnterpriseLoggerConfig {
                level: LogLevel::Info,
                base_dir: PathBuf::from("logs"),
                file_prefix: "app".to_string(),
                format: LogFormat::Structured,
                console_enabled: true,
                file_enabled: true,
                rotation: RotationConfig::default(),
                cleanup: CleanupConfig::default(),
                scheduler: CleanupSchedulerConfig::default(),
                async_processing: true,
                async_batch_size: 100,
            },
        }
    }
}

impl EnterpriseLoggerBuilder {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn level(mut self, level: LogLevel) -> Self {
        self.config.level = level;
        self
    }
    
    pub fn base_dir<P: Into<PathBuf>>(mut self, dir: P) -> Self {
        self.config.base_dir = dir.into();
        self
    }
    
    pub fn file_prefix<S: Into<String>>(mut self, prefix: S) -> Self {
        self.config.file_prefix = prefix.into();
        self
    }
    
    pub fn format(mut self, format: LogFormat) -> Self {
        self.config.format = format;
        self
    }
    
    pub fn console_enabled(mut self, enabled: bool) -> Self {
        self.config.console_enabled = enabled;
        self
    }
    
    pub fn file_enabled(mut self, enabled: bool) -> Self {
        self.config.file_enabled = enabled;
        self
    }
    
    pub fn max_file_size_mb(mut self, size_mb: u64) -> Self {
        self.config.rotation.max_size_bytes = size_mb * 1024 * 1024;
        self
    }
    
    pub fn max_files(mut self, count: u32) -> Self {
        self.config.rotation.max_files = count;
        self
    }
    
    pub fn retention_days(mut self, days: u32) -> Self {
        self.config.cleanup.retention_days = days;
        self
    }
    
    pub fn compress_rotated(mut self, compress: bool) -> Self {
        self.config.rotation.compress_rotated = compress;
        self
    }
    
    pub fn async_processing(mut self, enabled: bool) -> Self {
        self.config.async_processing = enabled;
        self
    }
    
    pub fn cleanup_interval_hours(mut self, hours: u32) -> Self {
        self.config.scheduler.cleanup_interval_hours = hours;
        self
    }
    
    pub fn auto_cleanup(mut self, enabled: bool) -> Self {
        self.config.scheduler.auto_cleanup_enabled = enabled;
        self
    }
    
    pub fn build(self) -> EnterpriseLogger {
        EnterpriseLogger::new(self.config)
    }
}

static ENTERPRISE_LOGGER: once_cell::sync::Lazy<Arc<Mutex<Option<EnterpriseLogger>>>> = 
    once_cell::sync::Lazy::new(|| Arc::new(Mutex::new(None)));

/// 全局企业日志系统管理
/// 
/// 提供一个全局的单例管理器，便于在整个应用中使用
pub struct GlobalEnterpriseLogger;

impl GlobalEnterpriseLogger {
    /// 初始化全局日志系统
    pub async fn initialize(config: EnterpriseLoggerConfig) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut logger = EnterpriseLogger::new(config);
        logger.initialize().await?;
        
        let mut global_logger = ENTERPRISE_LOGGER.lock()
            .map_err(|_| "无法获取全局logger锁")?;
        *global_logger = Some(logger);
        
        Ok(())
    }
    
    /// 获取全局日志系统状态
    pub fn get_status() -> Result<LoggerStatus, Box<dyn std::error::Error + Send + Sync>> {
        let global_logger = ENTERPRISE_LOGGER.lock()
            .map_err(|_| "无法获取全局logger锁")?;
        
        match global_logger.as_ref() {
            Some(logger) => logger.get_status(),
            None => Err("全局日志系统未初始化".into()),
        }
    }
    
    /// 手动执行清理
    pub async fn run_cleanup() -> Result<CleanupResult, Box<dyn std::error::Error + Send + Sync>> {
        let global_logger = ENTERPRISE_LOGGER.lock()
            .map_err(|_| "无法获取全局logger锁")?;
        
        match global_logger.as_ref() {
            Some(logger) => logger.run_cleanup().await,
            None => Err("全局日志系统未初始化".into()),
        }
    }
    
    /// 强制轮转日志文件
    pub fn rotate_logs() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let global_logger = ENTERPRISE_LOGGER.lock()
            .map_err(|_| "无法获取全局logger锁")?;
        
        match global_logger.as_ref() {
            Some(logger) => logger.rotate_logs(),
            None => Err("全局日志系统未初始化".into()),
        }
    }
    
    /// 获取文件统计
    pub fn get_file_stats() -> Result<LogFileStats, Box<dyn std::error::Error + Send + Sync>> {
        let global_logger = ENTERPRISE_LOGGER.lock()
            .map_err(|_| "无法获取全局logger锁")?;
        
        match global_logger.as_ref() {
            Some(logger) => logger.get_file_stats(),
            None => Err("全局日志系统未初始化".into()),
        }
    }
    
    /// 关闭全局日志系统
    pub async fn shutdown() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut global_logger = ENTERPRISE_LOGGER.lock()
            .map_err(|_| "无法获取全局logger锁")?;
        
        if let Some(mut logger) = global_logger.take() {
            logger.shutdown().await?;
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use tokio::time::{sleep, Duration};
    
    #[tokio::test]
    async fn test_enterprise_logger_initialization() {
        let temp_dir = TempDir::new().unwrap();
        
        let mut logger = EnterpriseLoggerBuilder::new()
            .base_dir(temp_dir.path())
            .level(LogLevel::Debug)
            .max_file_size_mb(1)
            .max_files(3)
            .retention_days(7)
            .build();
        
        assert!(logger.initialize().await.is_ok());
        
        let status = logger.get_status().unwrap();
        assert!(status.initialized);
        assert!(status.file_writer_active);
        
        logger.shutdown().await.unwrap();
    }
    
    #[tokio::test]
    async fn test_logging_and_rotation() {
        let temp_dir = TempDir::new().unwrap();
        
        let mut logger = EnterpriseLoggerBuilder::new()
            .base_dir(temp_dir.path())
            .max_file_size_mb(1) // 很小的文件大小便于测试
            .build();
        
        logger.initialize().await.unwrap();
        
        // 写入一些日志
        for i in 0..100 {
            log::info!("测试日志条目 {}", i);
        }
        
        logger.flush_all().unwrap();
        
        // 等待片刻让异步处理器处理
        sleep(Duration::from_millis(100)).await;
        
        let stats = logger.get_file_stats().unwrap();
        assert!(stats.total_files > 0);
        
        logger.shutdown().await.unwrap();
    }
    
    #[tokio::test]
    async fn test_manual_cleanup() {
        let temp_dir = TempDir::new().unwrap();
        
        let mut logger = EnterpriseLoggerBuilder::new()
            .base_dir(temp_dir.path())
            .retention_days(0) // 立即清理
            .build();
        
        logger.initialize().await.unwrap();
        
        // 写入一些日志
        log::info!("测试日志");
        logger.flush_all().unwrap();
        
        // 等待片刻
        sleep(Duration::from_millis(50)).await;
        
        // 手动执行清理
        let cleanup_result = logger.run_cleanup().await.unwrap();
        
        // 验证清理结果
        assert!(cleanup_result.errors.is_empty());
        
        logger.shutdown().await.unwrap();
    }
    
    #[tokio::test]
    async fn test_global_logger() {
        let temp_dir = TempDir::new().unwrap();
        
        let config = EnterpriseLoggerBuilder::new()
            .base_dir(temp_dir.path())
            .build()
            .get_config()
            .clone();
        
        // 初始化全局logger
        GlobalEnterpriseLogger::initialize(config).await.unwrap();
        
        // 测试状态获取
        let status = GlobalEnterpriseLogger::get_status().unwrap();
        assert!(status.initialized);
        
        // 写入日志
        log::info!("全局logger测试");
        
        // 测试轮转
        GlobalEnterpriseLogger::rotate_logs().unwrap();
        
        // 测试清理
        let cleanup_result = GlobalEnterpriseLogger::run_cleanup().await.unwrap();
        assert!(cleanup_result.errors.is_empty());
        
        // 关闭
        GlobalEnterpriseLogger::shutdown().await.unwrap();
    }
}
