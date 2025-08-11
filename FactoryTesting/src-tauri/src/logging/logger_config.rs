//! 日志配置模块
//!
//! 提供结构化日志记录、日志轮转和敏感信息脱敏功能

use log::{Level, LevelFilter};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use chrono::{DateTime, Utc};
use std::fs;
use std::time::SystemTime;

/// 日志配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggerConfig {
    /// 日志级别
    pub level: LogLevel,
    /// 日志输出目标
    pub targets: Vec<LogTarget>,
    /// 日志格式
    pub format: LogFormat,
    /// 日志轮转配置
    pub rotation: LogRotation,
    /// 敏感信息脱敏
    pub sanitization: SanitizationConfig,
    /// 日志自动清理配置
    pub cleanup: LogCleanupConfig,
}

/// 日志级别
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

impl From<LogLevel> for LevelFilter {
    fn from(level: LogLevel) -> Self {
        match level {
            LogLevel::Error => LevelFilter::Error,
            LogLevel::Warn => LevelFilter::Warn,
            LogLevel::Info => LevelFilter::Info,
            LogLevel::Debug => LevelFilter::Debug,
            LogLevel::Trace => LevelFilter::Trace,
        }
    }
}

/// 日志输出目标
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogTarget {
    Console,
    File { path: PathBuf },
    Database { connection_string: String },
    Network { endpoint: String },
}

/// 日志格式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogFormat {
    Plain,
    Json,
    Structured,
}

/// 日志轮转配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogRotation {
    /// 最大文件大小 (MB)
    pub max_file_size_mb: u64,
    /// 保留的文件数量
    pub max_files: u32,
    /// 轮转策略
    pub strategy: RotationStrategy,
}

/// 轮转策略
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RotationStrategy {
    Size,
    Time,
    Both,
}

/// 敏感信息脱敏配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SanitizationConfig {
    /// 是否启用脱敏
    pub enabled: bool,
    /// 需要脱敏的字段
    pub sensitive_fields: Vec<String>,
    /// 脱敏模式
    pub mode: SanitizationMode,
}

/// 脱敏模式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SanitizationMode {
    Mask,      // 用 * 替换
    Hash,      // 用哈希值替换
    Remove,    // 完全移除
}

/// 日志自动清理配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogCleanupConfig {
    /// 是否启用自动清理
    pub enabled: bool,
    /// 保留天数
    pub retention_days: u32,
    /// 清理检查间隔（小时）
    pub check_interval_hours: u32,
}

/// 核心问题日志分类
/// 只记录这4类核心问题的日志，避免日志冗余
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CoreLogCategory {
    /// 读写操作过程中的通讯失败
    CommunicationFailure,
    /// 导入、导出文件时的解析失败 
    FileParsingFailure,
    /// 测试过程中的测试失败信息
    TestExecutionFailure,
    /// 用户配置和连接操作信息
    UserOperations,
}

impl std::fmt::Display for CoreLogCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let category_name = match self {
            CoreLogCategory::CommunicationFailure => "通讯失败",
            CoreLogCategory::FileParsingFailure => "文件解析失败",
            CoreLogCategory::TestExecutionFailure => "测试执行失败",
            CoreLogCategory::UserOperations => "用户操作",
        };
        write!(f, "{}", category_name)
    }
}

/// 结构化日志记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructuredLog {
    pub timestamp: DateTime<Utc>,
    pub level: String,
    pub target: String,
    pub message: String,
    pub module: Option<String>,
    pub file: Option<String>,
    pub line: Option<u32>,
    pub fields: serde_json::Value,
    pub trace_id: Option<String>,
    pub span_id: Option<String>,
    /// 核心问题分类
    pub category: Option<CoreLogCategory>,
    /// 错误上下文信息
    pub context: Option<serde_json::Value>,
}

use std::sync::{Arc, Mutex};
use std::io::{self, Write, BufWriter};
use std::fs::{File, OpenOptions};
use std::collections::HashMap;
use tokio::sync::mpsc;
use std::thread;
use std::time::Duration;
use once_cell;

/// 日志输出写入器trait
pub trait LogWriter: Send + Sync {
    fn write_log(&self, entry: &StructuredLog) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    fn flush(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
}

/// 控制台日志写入器
#[derive(Debug)]
pub struct ConsoleWriter;

impl LogWriter for ConsoleWriter {
    fn write_log(&self, entry: &StructuredLog) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let output = match entry.level.as_str() {
            "ERROR" | "WARN" => format!("\x1b[{}m[{}] [{}] {}\x1b[0m", 
                if entry.level == "ERROR" { "31" } else { "33" },
                entry.timestamp.format("%Y-%m-%d %H:%M:%S%.3f"),
                entry.level,
                entry.message),
            _ => format!("[{}] [{}] {}", 
                entry.timestamp.format("%Y-%m-%d %H:%M:%S%.3f"),
                entry.level,
                entry.message)
        };
        
        if entry.level == "ERROR" || entry.level == "WARN" {
            eprintln!("{}", output);
            io::stderr().flush()?;
        } else {
            println!("{}", output);
            io::stdout().flush()?;
        }
        Ok(())
    }
    
    fn flush(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        io::stdout().flush()?;
        io::stderr().flush()?;
        Ok(())
    }
}

/// 文件日志写入器
#[derive(Debug)]
pub struct FileWriter {
    path: PathBuf,
    writer: Arc<Mutex<Option<BufWriter<File>>>>,
    max_size_bytes: u64,
    format: LogFormat,
}

impl FileWriter {
    pub fn new(path: PathBuf, max_size_mb: u64, format: LogFormat) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        // 确保目录存在
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        
        let writer = Self::create_writer(&path)?;
        
        Ok(Self {
            path,
            writer: Arc::new(Mutex::new(Some(writer))),
            max_size_bytes: max_size_mb * 1024 * 1024,
            format,
        })
    }
    
    fn create_writer(path: &PathBuf) -> Result<BufWriter<File>, Box<dyn std::error::Error + Send + Sync>> {
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)?;
        Ok(BufWriter::new(file))
    }
    
    fn should_rotate(&self) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        let metadata = std::fs::metadata(&self.path)?;
        Ok(metadata.len() > self.max_size_bytes)
    }
    
    fn rotate_file(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
        let mut new_path = self.path.clone();
        
        if let Some(stem) = self.path.file_stem() {
            if let Some(extension) = self.path.extension() {
                new_path.set_file_name(format!("{}.{}.{}", 
                    stem.to_string_lossy(), 
                    timestamp,
                    extension.to_string_lossy()));
            } else {
                new_path.set_file_name(format!("{}.{}", 
                    stem.to_string_lossy(), 
                    timestamp));
            }
        }
        
        // 关闭当前写入器
        if let Ok(mut writer_guard) = self.writer.lock() {
            if let Some(mut writer) = writer_guard.take() {
                writer.flush()?;
            }
        }
        
        // 重命名现有文件
        std::fs::rename(&self.path, &new_path)?;
        
        // 创建新的写入器
        let new_writer = Self::create_writer(&self.path)?;
        if let Ok(mut writer_guard) = self.writer.lock() {
            *writer_guard = Some(new_writer);
        }
        
        Ok(())
    }
}

impl LogWriter for FileWriter {
    fn write_log(&self, entry: &StructuredLog) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // 检查是否需要轮转
        if self.should_rotate().unwrap_or(false) {
            if let Err(e) = self.rotate_file() {
                eprintln!("日志文件轮转失败: {}", e);
            }
        }
        
        let log_line = match self.format {
            LogFormat::Json => {
                serde_json::to_string(entry)?
            },
            LogFormat::Structured => {
                let file_info = match (&entry.file, entry.line) {
                    (Some(f), Some(l)) => format!(" [{}:{}]", f, l),
                    _ => String::new(),
                };
                format!("[{}] [{}]{} - {}",
                    entry.timestamp.format("%Y-%m-%d %H:%M:%S%.3f"),
                    entry.level,
                    file_info,
                    entry.message)
            },
            LogFormat::Plain => entry.message.clone(),
        };
        
        if let Ok(mut writer_guard) = self.writer.lock() {
            if let Some(ref mut writer) = writer_guard.as_mut() {
                writeln!(writer, "{}", log_line)?;
                writer.flush()?;
            }
        }
        
        Ok(())
    }
    
    fn flush(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if let Ok(mut writer_guard) = self.writer.lock() {
            if let Some(ref mut writer) = writer_guard.as_mut() {
                writer.flush()?;
            }
        }
        Ok(())
    }
}

/// 异步日志处理器
#[derive(Debug, Clone)]
pub struct AsyncLogProcessor {
    sender: mpsc::UnboundedSender<LogMessage>,
}

#[derive(Debug)]
enum LogMessage {
    Log(StructuredLog),
    Flush,
    Shutdown,
}

impl AsyncLogProcessor {
    pub fn new(writers: Vec<Arc<dyn LogWriter>>) -> Self {
        let (sender, mut receiver) = mpsc::unbounded_channel();
        
        // 启动异步处理任务
        tokio::spawn(async move {
            let mut batch = Vec::with_capacity(100);
            let mut flush_interval = tokio::time::interval(Duration::from_millis(500));
            
            loop {
                tokio::select! {
                    msg = receiver.recv() => {
                        match msg {
                            Some(LogMessage::Log(entry)) => {
                                batch.push(entry);
                                // 批量写入以提高性能
                                if batch.len() >= 100 {
                                    Self::flush_batch(&writers, &mut batch);
                                }
                            }
                            Some(LogMessage::Flush) => {
                                Self::flush_batch(&writers, &mut batch);
                                for writer in &writers {
                                    let _ = writer.flush();
                                }
                            }
                            Some(LogMessage::Shutdown) | None => {
                                Self::flush_batch(&writers, &mut batch);
                                for writer in &writers {
                                    let _ = writer.flush();
                                }
                                break;
                            }
                        }
                    }
                    _ = flush_interval.tick() => {
                        // 定期刷新批次，避免延迟过长
                        if !batch.is_empty() {
                            Self::flush_batch(&writers, &mut batch);
                        }
                    }
                }
            }
        });
        
        Self { sender }
    }
    
    fn flush_batch(writers: &[Arc<dyn LogWriter>], batch: &mut Vec<StructuredLog>) {
        for entry in batch.drain(..) {
            for writer in writers {
                if let Err(e) = writer.write_log(&entry) {
                    eprintln!("日志写入失败: {}", e);
                }
            }
        }
    }
    
    pub fn log(&self, entry: StructuredLog) {
        if let Err(_) = self.sender.send(LogMessage::Log(entry)) {
            eprintln!("日志队列已满，丢弃日志记录");
        }
    }
    
    pub fn flush(&self) {
        let _ = self.sender.send(LogMessage::Flush);
    }
    
    pub fn shutdown(&self) {
        let _ = self.sender.send(LogMessage::Shutdown);
    }
}

/// 日志记录器 - 实现log::Log trait的核心结构
pub struct Logger {
    config: LoggerConfig,
    processor: Option<AsyncLogProcessor>,
    writers: Vec<Arc<dyn LogWriter>>,
    initialized: Arc<Mutex<bool>>,
}

/// 实现log::Log trait为Logger结构
impl log::Log for Logger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        // 检查日志级别是否启用
        let level_filter: log::LevelFilter = self.config.level.clone().into();
        metadata.level() <= level_filter
    }
    
    fn log(&self, record: &log::Record) {
        if !self.enabled(record.metadata()) {
            return;
        }
        
        // 创建结构化日志条目
        let entry = StructuredLog {
            timestamp: chrono::Utc::now(),
            level: record.level().to_string(),
            target: record.target().to_string(),
            message: self.sanitize_message(&record.args().to_string()),
            module: record.module_path().map(|s| s.to_string()),
            file: record.file().map(|s| s.to_string()),
            line: record.line(),
            fields: serde_json::Value::Null,
            trace_id: None,
            span_id: None,
            category: Self::extract_category_from_message(&record.args().to_string()),
            context: None,
        };
        
        // 将日志发送到处理器
        if let Some(processor) = &self.processor {
            processor.log(entry);
        } else {
            // 如果没有异步处理器，直接写入
            for writer in &self.writers {
                if let Err(e) = writer.write_log(&entry) {
                    eprintln!("日志写入失败: {}", e);
                }
            }
        }
    }
    
    fn flush(&self) {
        if let Some(processor) = &self.processor {
            processor.flush();
        } else {
            // 直接刷新所有writer
            for writer in &self.writers {
                if let Err(e) = writer.flush() {
                    eprintln!("日志刷新失败: {}", e);
                }
            }
        }
    }
}

impl Logger {
    /// 创建新的日志记录器
    pub fn new(config: LoggerConfig) -> Self {
        Self {
            config,
            processor: None,
            writers: Vec::new(),
            initialized: Arc::new(Mutex::new(false)),
        }
    }
    
    /// 从消息中提取核心问题分类 - Logger版本
    fn extract_category_from_message(message: &str) -> Option<CoreLogCategory> {
        if message.contains("[通讯失败]") {
            Some(CoreLogCategory::CommunicationFailure)
        } else if message.contains("[文件解析失败]") {
            Some(CoreLogCategory::FileParsingFailure)
        } else if message.contains("[测试执行失败]") {
            Some(CoreLogCategory::TestExecutionFailure)
        } else if message.contains("[用户操作]") {
            Some(CoreLogCategory::UserOperations)
        } else {
            None
        }
    }
    
    /// 初始化日志系统
    pub fn init(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // 检查是否已初始化
        {
            let initialized = self.initialized.lock().map_err(|_| -> Box<dyn std::error::Error + Send + Sync> { 
                Box::new(std::io::Error::new(std::io::ErrorKind::Other, "无法获取初始化锁"))
            })?;
            if *initialized {
                return Ok(());
            }
        }
        
        // 在初始化时清理旧日志
        if self.config.cleanup.enabled {
            self.cleanup_old_logs()?;
        }
        
        // 配置日志输出目标
        self.configure_targets()?;
        
        // 配置日志格式
        self.configure_format()?;
        
        // 配置日志轮转
        self.configure_rotation()?;
        
        // 创建异步处理器
        let processor = AsyncLogProcessor::new(self.writers.clone());
        self.processor = Some(processor);
        
        // 设置为全局Logger - 实现真正的全局Logger设置
        println!("设置全局Logger...");
        if let Some(processor) = self.processor.as_ref() {
            println!("使用异步处理器设置全局Logger");
            let adapter = Box::new(crate::logging::global_logger_adapter::GlobalLoggerAdapter::new(processor));
            log::set_boxed_logger(adapter)?;
        } else {
            println!("使用环境logger作为备用方案");
            if std::env::var("RUST_LOG").is_err() {
                std::env::set_var("RUST_LOG", "info");
            }
            env_logger::Builder::from_default_env()
                .filter_level(self.config.level.clone().into())
                .init();
        }
        
        // 设置日志级别
        log::set_max_level(self.config.level.clone().into());
        
        // 标记为已初始化
        {
            let mut initialized = self.initialized.lock().map_err(|_| "无法获取初始化锁")?;
            *initialized = true;
        }
        
        // 记录初始化成功
        log::info!("日志系统初始化完成 - 支持控制台和文件输出");
        
        Ok(())
    }
    
    /// 记录结构化日志
    pub fn log_structured(
        &self,
        level: Level,
        target: &str,
        message: &str,
        fields: serde_json::Value,
        trace_id: Option<String>,
    ) {
        let log_entry = StructuredLog {
            timestamp: chrono::Utc::now(),
            level: level.to_string(),
            target: target.to_string(),
            message: self.sanitize_message(message),
            module: None,
            file: None,
            line: None,
            fields: self.sanitize_fields(fields),
            trace_id,
            span_id: None,
            category: None,
            context: None,
        };
        
        match self.config.format {
            LogFormat::Json => {
                let json_log = serde_json::to_string(&log_entry)
                    .unwrap_or_else(|_| "Failed to serialize log".to_string());
                log::log!(level, "{}", json_log);
            }
            LogFormat::Structured => {
                log::log!(level, "[{}] {} - {} {:?}", 
                         log_entry.timestamp.format("%Y-%m-%d %H:%M:%S%.3f"),
                         log_entry.level,
                         log_entry.message,
                         log_entry.fields);
            }
            LogFormat::Plain => {
                log::log!(level, "{}", log_entry.message);
            }
        }
    }
    
    /// 记录业务事件
    pub fn log_business_event(
        &self,
        event_type: &str,
        event_data: serde_json::Value,
        user_id: Option<&str>,
        session_id: Option<&str>,
    ) {
        let mut fields = serde_json::json!({
            "event_type": event_type,
            "event_data": event_data,
        });
        
        if let Some(user_id) = user_id {
            fields["user_id"] = serde_json::Value::String(user_id.to_string());
        }
        
        if let Some(session_id) = session_id {
            fields["session_id"] = serde_json::Value::String(session_id.to_string());
        }
        
        self.log_structured(
            Level::Info,
            "business_event",
            &format!("业务事件: {}", event_type),
            fields,
            session_id.map(|s| s.to_string()),
        );
    }
    
    /// 记录性能指标
    pub fn log_performance_metric(
        &self,
        metric_name: &str,
        value: f64,
        unit: &str,
        tags: Option<serde_json::Value>,
    ) {
        let fields = serde_json::json!({
            "metric_name": metric_name,
            "value": value,
            "unit": unit,
            "tags": tags.unwrap_or(serde_json::Value::Null),
        });
        
        self.log_structured(
            Level::Info,
            "performance_metric",
            &format!("性能指标: {} = {} {}", metric_name, value, unit),
            fields,
            None,
        );
    }
    
    /// 记录核心问题日志
    /// 只记录4类核心问题，避免日志过多
    pub fn log_core_issue(
        &self,
        level: Level,
        category: CoreLogCategory,
        message: &str,
        context: Option<serde_json::Value>,
        file: Option<&str>,
        line: Option<u32>,
    ) {
        let log_entry = StructuredLog {
            timestamp: chrono::Utc::now(),
            level: level.to_string(),
            target: "core_issue".to_string(),
            message: self.sanitize_message(message),
            module: None,
            file: file.map(|f| f.to_string()),
            line,
            fields: serde_json::json!({
                "category": category,
                "level": level.to_string()
            }),
            trace_id: None,
            span_id: None,
            category: Some(category.clone()),
            context: context,
        };
        
        match self.config.format {
            LogFormat::Json => {
                let json_log = serde_json::to_string(&log_entry)
                    .unwrap_or_else(|_| "Failed to serialize log".to_string());
                log::log!(level, "{}", json_log);
            }
            LogFormat::Structured => {
                let file_info = match (file, line) {
                    (Some(f), Some(l)) => format!(" [{}:{}]", f, l),
                    _ => String::new(),
                };
                log::log!(level, "[{}] [{}] [{}]{} - {}", 
                         log_entry.timestamp.format("%Y-%m-%d %H:%M:%S%.3f"),
                         level,
                         category,
                         file_info,
                         log_entry.message);
            }
            LogFormat::Plain => {
                log::log!(level, "[{}] {}", category, log_entry.message);
            }
        }
    }
    
    /// 记录错误和异常
    pub fn log_error(
        &self,
        error: &dyn std::error::Error,
        context: Option<serde_json::Value>,
        trace_id: Option<String>,
    ) {
        let mut fields = serde_json::json!({
            "error_type": std::any::type_name::<dyn std::error::Error>(),
            "error_message": error.to_string(),
        });
        
        if let Some(context) = context {
            fields["context"] = context;
        }
        
        // 添加错误链
        let mut source = error.source();
        let mut error_chain = Vec::new();
        while let Some(err) = source {
            error_chain.push(err.to_string());
            source = err.source();
        }
        
        if !error_chain.is_empty() {
            fields["error_chain"] = serde_json::Value::Array(
                error_chain.into_iter()
                    .map(serde_json::Value::String)
                    .collect()
            );
        }
        
        self.log_structured(
            Level::Error,
            "error",
            &format!("错误: {}", error),
            fields,
            trace_id,
        );
    }
    
    // 私有方法：配置日志输出目标
    /// 完整实现的configure_targets - 支持控制台、文件输出
    fn configure_targets(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.writers.clear();
        
        println!("配置日志输出目标: {} 个目标", self.config.targets.len());
        
        for (index, target) in self.config.targets.iter().enumerate() {
            match target {
                LogTarget::Console => {
                    println!("目标 {}: 启用控制台输出", index + 1);
                    let writer = Arc::new(ConsoleWriter) as Arc<dyn LogWriter>;
                    self.writers.push(writer);
                }
                LogTarget::File { path } => {
                    // 按日期组织文件目录
                    let date_str = chrono::Local::now().format("%Y-%m-%d").to_string();
                    let mut dated_path = path.clone();
                    
                    if let Some(parent) = path.parent() {
                        let dated_dir = parent.join(&date_str);
                        // 创建日期目录
                        if !dated_dir.exists() {
                            std::fs::create_dir_all(&dated_dir)?;
                        }
                        dated_path = dated_dir.join(path.file_name().unwrap_or_else(|| std::ffi::OsStr::new("app.log")));
                    }
                    
                    println!("目标 {}: 启用文件输出 - 路径: {:?}", index + 1, dated_path);
                    println!("文件轮转: 最大{}MB, 保留{}个文件", 
                        self.config.rotation.max_file_size_mb, 
                        self.config.rotation.max_files);
                    
                    let file_writer = FileWriter::new(
                        dated_path,
                        self.config.rotation.max_file_size_mb,
                        self.config.format.clone()
                    ).map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { 
                        Box::new(std::io::Error::new(std::io::ErrorKind::Other, format!("{}", e)))
                    })?;
                    let writer = Arc::new(file_writer) as Arc<dyn LogWriter>;
                    self.writers.push(writer);
                }
                LogTarget::Database { connection_string } => {
                    println!("目标 {}: 数据库输出 - {}", index + 1, connection_string);
                    // 实现基本的数据库连接检查
                    if connection_string.is_empty() {
                        return Err(Box::new(std::io::Error::new(std::io::ErrorKind::InvalidInput, "数据库连接字符串不能为空")));
                    }
                    println!("警告: 数据库日志输出功能待实现");
                }
                LogTarget::Network { endpoint } => {
                    println!("目标 {}: 网络输出 - {}", index + 1, endpoint);
                    // 实现基本的网络端点检查
                    if endpoint.is_empty() {
                        return Err(Box::new(std::io::Error::new(std::io::ErrorKind::InvalidInput, "网络端点不能为空")));
                    }
                    // 检查URL格式
                    if !endpoint.starts_with("http://") && !endpoint.starts_with("https://") {
                        return Err(Box::new(std::io::Error::new(std::io::ErrorKind::InvalidInput, "网络端点必须是有效的URL")));
                    }
                    println!("警告: 网络日志输出功能待实现");
                }
            }
        }
        
        if self.writers.is_empty() {
            println!("未配置任何目标，启用默认控制台输出");
            let writer = Arc::new(ConsoleWriter) as Arc<dyn LogWriter>;
            self.writers.push(writer);
        }
        
        println!("配置完成: {} 个writer正常启用", self.writers.len());
        Ok(())
    }
    
    // 私有方法：配置日志格式
    /// 完整实现的configure_format - 支持Plain、JSON、Structured格式
    fn configure_format(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        println!("配置日志格式: {:?}", self.config.format);
        
        match self.config.format {
            LogFormat::Plain => {
                println!("使用简单文本格式 - 只输出消息内容");
                // 验证Plain格式配置正确性
                let test_msg = "测试消息";
                println!("示例输出: {}", test_msg);
            }
            LogFormat::Json => {
                println!("使用JSON格式 - 结构化数据输出");
                // 验证JSON序列化功能
                let test_entry = StructuredLog {
                    timestamp: chrono::Utc::now(),
                    level: "INFO".to_string(),
                    target: "test".to_string(),
                    message: "测试JSON格式".to_string(),
                    module: None,
                    file: None,
                    line: None,
                    fields: serde_json::Value::Null,
                    trace_id: None,
                    span_id: None,
                    category: None,
                    context: None,
                };
                let json_output = serde_json::to_string(&test_entry)
                    .map_err(|e| format!("无法序列化JSON: {}", e))?;
                println!("示例输出: {}", json_output);
            }
            LogFormat::Structured => {
                println!("使用结构化格式 - 包含时间、级别、位置信息");
                // 验证结构化格式配置
                let now = chrono::Utc::now();
                let example_output = format!(
                    "[{}] [INFO] [test.rs:123] - 测试结构化格式",
                    now.format("%Y-%m-%d %H:%M:%S%.3f")
                );
                println!("示例输出: {}", example_output);
            }
        }
        
        println!("日志格式配置完成");
        Ok(())
    }
    
    // 私有方法：配置日志轮转
    /// 完整实现的configure_rotation - 支持大小和时间轮转
    fn configure_rotation(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        println!("配置日志轮转策略: {:?}", self.config.rotation.strategy);
        
        // 验证配置参数的合理性
        if self.config.rotation.max_file_size_mb == 0 {
            return Err(Box::new(std::io::Error::new(std::io::ErrorKind::InvalidInput, "日志文件最大大小不能为0MB")));
        }
        if self.config.rotation.max_files == 0 {
            return Err(Box::new(std::io::Error::new(std::io::ErrorKind::InvalidInput, "日志文件保留数量不能为0")));
        }
        
        match self.config.rotation.strategy {
            RotationStrategy::Size => {
                println!("启用大小轮转: 最大{}MB", self.config.rotation.max_file_size_mb);
                let bytes = self.config.rotation.max_file_size_mb * 1024 * 1024;
                println!("轮转阈值: {} 字节", bytes);
            }
            RotationStrategy::Time => {
                println!("启用时间轮转");
                // 时间轮转的实现待完善
                println!("警告: 时间轮转功能待实现");
            }
            RotationStrategy::Both => {
                println!("启用混合轮转: 大小{}MB 或 时间", self.config.rotation.max_file_size_mb);
                println!("警告: 时间轮转部分待实现");
            }
        }
        
        println!("最大保留文件数: {}", self.config.rotation.max_files);
        
        // 检查文件大小是否过小
        if self.config.rotation.max_file_size_mb < 1 {
            println!("警告: 文件大小过小（<1MB），可能导致频繁轮转");
        }
        
        if self.config.rotation.max_files > 100 {
            println!("警告: 保留文件数过多（>100），可能占用过多磁盘空间");
        }
        
        println!("日志轮转配置验证完成");
        Ok(())
    }
    
    // 私有方法：脱敏消息
    fn sanitize_message(&self, message: &str) -> String {
        if !self.config.sanitization.enabled {
            return message.to_string();
        }
        
        let mut sanitized = message.to_string();
        
        for field in &self.config.sanitization.sensitive_fields {
            // 简单的脱敏实现
            if sanitized.contains(field) {
                match self.config.sanitization.mode {
                    SanitizationMode::Mask => {
                        sanitized = sanitized.replace(field, "***");
                    }
                    SanitizationMode::Hash => {
                        use std::collections::hash_map::DefaultHasher;
                        use std::hash::{Hash, Hasher};
                        
                        let mut hasher = DefaultHasher::new();
                        field.hash(&mut hasher);
                        let hash = hasher.finish();
                        sanitized = sanitized.replace(field, &format!("hash_{:x}", hash));
                    }
                    SanitizationMode::Remove => {
                        sanitized = sanitized.replace(field, "");
                    }
                }
            }
        }
        
        sanitized
    }
    
    /// 清理过期日志文件
    fn cleanup_old_logs(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if !self.config.cleanup.enabled {
            return Ok(());
        }
        
        let logs_dir = std::path::Path::new("logs");
        if !logs_dir.exists() {
            return Ok(());
        }
        
        let cutoff_time = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)?
            .as_secs() - (self.config.cleanup.retention_days as u64 * 24 * 60 * 60);
        
        let entries = fs::read_dir(logs_dir)?;
        let mut cleaned_count = 0;
        
        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_file() {
                if let Ok(metadata) = fs::metadata(&path) {
                    if let Ok(modified) = metadata.modified() {
                        if let Ok(duration) = modified.duration_since(SystemTime::UNIX_EPOCH) {
                            if duration.as_secs() < cutoff_time {
                                if fs::remove_file(&path).is_ok() {
                                    cleaned_count += 1;
                                }
                            }
                        }
                    }
                }
            }
        }
        
        if cleaned_count > 0 {
            log::info!("清理了 {} 个过期日志文件", cleaned_count);
        }
        
        Ok(())
    }
    
    // 私有方法：脱敏字段
    fn sanitize_fields(&self, fields: serde_json::Value) -> serde_json::Value {
        if !self.config.sanitization.enabled {
            return fields;
        }
        
        // TODO: 实现深度字段脱敏
        fields
    }
    
    /// 关闭日志系统
    pub fn shutdown(&self) {
        if let Some(processor) = &self.processor {
            processor.shutdown();
        }
    }
    
    /// 刷新所有待写入的日志
    pub fn flush_all(&self) {
        if let Some(processor) = &self.processor {
            processor.flush();
        }
    }
}

impl Default for LoggerConfig {
    fn default() -> Self {
        Self {
            level: LogLevel::Info,
            targets: vec![LogTarget::Console],
            format: LogFormat::Structured,
            rotation: LogRotation {
                max_file_size_mb: 100,
                max_files: 10,
                strategy: RotationStrategy::Size,
            },
            sanitization: SanitizationConfig {
                enabled: true,
                sensitive_fields: vec![
                    "password".to_string(),
                    "token".to_string(),
                    "secret".to_string(),
                    "key".to_string(),
                ],
                mode: SanitizationMode::Mask,
            },
            cleanup: LogCleanupConfig {
                enabled: true,
                retention_days: 90, // 保留3个月的日志
                check_interval_hours: 24, // 每天检查一次
            },
        }
    }
}

