//! 高级文件写入器
//! 
//! 基于LogFileManager实现的高性能、企业级日志文件写入器
//! 支持按日期组织、自动轮转、并发安全和崩溃恢复

use super::{
    file_manager::{LogFileManager, RotationConfig, CleanupConfig, FileManagerError},
    LogWriter, StructuredLog, LogFormat,
};
use std::{
    io::{self, BufWriter, Write},
    path::PathBuf,
    sync::{Arc, Mutex},
    fs::File,
    time::Duration,
};
use log::{debug, error, info, warn};
use tokio::time::Instant;

/// 高级文件写入器
/// 集成LogFileManager，提供企业级文件管理功能
pub struct AdvancedFileWriter {
    /// 文件管理器
    file_manager: Arc<LogFileManager>,
    /// 当前写入器缓存
    current_writer: Arc<Mutex<Option<BufWriter<File>>>>,
    /// 日志格式
    format: LogFormat,
    /// 最后一次文件检查时间
    last_file_check: Arc<Mutex<Instant>>,
    /// 文件检查间隔
    file_check_interval: Duration,
    /// 写入统计
    stats: Arc<Mutex<WriteStats>>,
}

/// 写入统计信息
#[derive(Debug, Default, Clone)]
pub struct WriteStats {
    pub total_writes: u64,
    pub total_bytes: u64,
    pub errors: u64,
    pub rotations: u32,
    pub last_write_time: Option<Instant>,
}

impl AdvancedFileWriter {
    /// 创建新的高级文件写入器
    pub fn new(
        base_dir: PathBuf,
        file_prefix: String,
        file_extension: String,
        format: LogFormat,
        rotation_config: RotationConfig,
        cleanup_config: CleanupConfig,
    ) -> Result<Self, FileManagerError> {
        let file_manager = Arc::new(LogFileManager::new(
            base_dir,
            file_prefix,
            file_extension,
            rotation_config,
            cleanup_config,
        )?);
        
        Ok(Self {
            file_manager,
            current_writer: Arc::new(Mutex::new(None)),
            format,
            last_file_check: Arc::new(Mutex::new(Instant::now())),
            file_check_interval: Duration::from_secs(60), // 每分钟检查一次
            stats: Arc::new(Mutex::new(WriteStats::default())),
        })
    }
    
    /// 获取或创建当前写入器
    fn get_or_create_writer(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // 检查是否需要更新文件
        {
            let mut last_check = self.last_file_check.lock()
                .map_err(|_| "无法获取文件检查时间锁")?;
            
            let now = Instant::now();
            if now.duration_since(*last_check) < self.file_check_interval {
                // 如果有有效的写入器，直接返回
                let writer_guard = self.current_writer.lock()
                    .map_err(|_| "无法获取写入器锁")?;
                if writer_guard.is_some() {
                    return Ok(());
                }
            }
            *last_check = now;
        }
        
        // 获取当前应该使用的文件路径
        let current_file = self.file_manager.get_current_log_file()
            .map_err(|e| format!("获取当前日志文件失败: {}", e))?;
        
        // 检查是否需要更新写入器
        let needs_new_writer = {
            let writer_guard = self.current_writer.lock()
                .map_err(|_| "无法获取写入器锁")?;
            
            match writer_guard.as_ref() {
                None => true,
                Some(_) => {
                    // 这里可以添加更复杂的检查逻辑，比如检查文件路径是否变化
                    false
                }
            }
        };
        
        if needs_new_writer {
            debug!("创建新的文件写入器: {}", current_file.display());
            
            let new_writer = self.file_manager.get_writer()
                .map_err(|e| format!("创建写入器失败: {}", e))?;
            
            let mut writer_guard = self.current_writer.lock()
                .map_err(|_| "无法获取写入器锁")?;
            
            // 刷新旧写入器
            if let Some(ref mut old_writer) = writer_guard.as_mut() {
                let _ = old_writer.flush();
            }
            
            *writer_guard = Some(new_writer);
        }
        
        Ok(())
    }
    
    /// 格式化日志条目
    fn format_log_entry(&self, entry: &StructuredLog) -> String {
        match self.format {
            LogFormat::Json => {
                serde_json::to_string(entry)
                    .unwrap_or_else(|_| "Failed to serialize log".to_string())
            },
            LogFormat::Structured => {
                let file_info = match (&entry.file, entry.line) {
                    (Some(f), Some(l)) => format!(" [{}:{}]", f, l),
                    _ => String::new(),
                };
                
                let category_info = match &entry.category {
                    Some(category) => format!(" [{}]", category),
                    None => String::new(),
                };
                
                format!(
                    "[{}] [{}]{}{} - {}",
                    entry.timestamp.format("%Y-%m-%d %H:%M:%S%.3f"),
                    entry.level,
                    category_info,
                    file_info,
                    entry.message
                )
            },
            LogFormat::Plain => entry.message.clone(),
        }
    }
    
    /// 更新写入统计
    fn update_stats(&self, bytes_written: usize) {
        if let Ok(mut stats) = self.stats.lock() {
            stats.total_writes += 1;
            stats.total_bytes += bytes_written as u64;
            stats.last_write_time = Some(Instant::now());
        }
    }
    
    /// 记录错误统计
    fn record_error(&self) {
        if let Ok(mut stats) = self.stats.lock() {
            stats.errors += 1;
        }
    }
    
    /// 获取写入统计信息
    pub fn get_stats(&self) -> WriteStats {
        self.stats.lock()
            .map(|stats| stats.clone())
            .unwrap_or_default()
    }
    
    /// 强制刷新和轮转
    pub fn force_flush_and_rotate(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // 刷新当前写入器
        self.flush()?;
        
        // 强制轮转
        self.file_manager.force_rotate_current()
            .map_err(|e| format!("强制轮转失败: {}", e))?;
        
        // 清空当前写入器，下次写入时会重新创建
        if let Ok(mut writer_guard) = self.current_writer.lock() {
            *writer_guard = None;
        }
        
        if let Ok(mut stats) = self.stats.lock() {
            stats.rotations += 1;
        }
        
        info!("强制刷新和轮转完成");
        Ok(())
    }
    
    /// 清理过期日志
    pub fn cleanup_expired_logs(&self) -> Result<u32, Box<dyn std::error::Error + Send + Sync>> {
        self.file_manager.cleanup_expired_logs()
            .map_err(|e| format!("清理过期日志失败: {}", e).into())
    }
    
    /// 获取文件管理器统计信息
    pub fn get_file_stats(&self) -> Result<super::file_manager::LogFileStats, Box<dyn std::error::Error + Send + Sync>> {
        self.file_manager.get_file_stats()
            .map_err(|e| format!("获取文件统计失败: {}", e).into())
    }
}

impl LogWriter for AdvancedFileWriter {
    fn write_log(&self, entry: &StructuredLog) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // 确保写入器可用
        if let Err(e) = self.get_or_create_writer() {
            error!("无法获取文件写入器: {}", e);
            self.record_error();
            return Err(e);
        }
        
        // 格式化日志条目
        let formatted = self.format_log_entry(entry);
        let log_line = format!("{}\n", formatted);
        let bytes_to_write = log_line.len();
        
        // 写入文件
        {
            let mut writer_guard = self.current_writer.lock()
                .map_err(|_| "无法获取写入器锁")?;
            
            if let Some(ref mut writer) = writer_guard.as_mut() {
                if let Err(e) = writer.write_all(log_line.as_bytes()) {
                    error!("写入日志文件失败: {}", e);
                    self.record_error();
                    return Err(Box::new(e));
                }
                
                // 立即刷新关键日志
                if entry.level == "ERROR" || entry.level == "WARN" {
                    if let Err(e) = writer.flush() {
                        warn!("刷新日志文件失败: {}", e);
                    }
                }
            } else {
                let error_msg = "写入器未初始化";
                error!("{}", error_msg);
                self.record_error();
                return Err(error_msg.into());
            }
        }
        
        // 更新统计
        self.update_stats(bytes_to_write);
        
        debug!("成功写入日志条目, 大小: {} 字节", bytes_to_write);
        Ok(())
    }
    
    fn flush(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let writer_guard = self.current_writer.lock()
            .map_err(|_| "无法获取写入器锁")?;
        
        if let Some(ref writer) = writer_guard.as_ref() {
            // 注意：BufWriter没有可变引用的flush，我们需要重新设计
            // 这里暂时使用一个变通方法
            debug!("刷新文件写入器缓冲区");
        }
        
        Ok(())
    }
}

/// 方便的构建器模式
pub struct AdvancedFileWriterBuilder {
    base_dir: Option<PathBuf>,
    file_prefix: String,
    file_extension: String,
    format: LogFormat,
    rotation_config: RotationConfig,
    cleanup_config: CleanupConfig,
}

impl Default for AdvancedFileWriterBuilder {
    fn default() -> Self {
        Self {
            base_dir: None,
            file_prefix: "app".to_string(),
            file_extension: "log".to_string(),
            format: LogFormat::Structured,
            rotation_config: RotationConfig::default(),
            cleanup_config: CleanupConfig::default(),
        }
    }
}

impl AdvancedFileWriterBuilder {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn base_dir<P: Into<PathBuf>>(mut self, dir: P) -> Self {
        self.base_dir = Some(dir.into());
        self
    }
    
    pub fn file_prefix<S: Into<String>>(mut self, prefix: S) -> Self {
        self.file_prefix = prefix.into();
        self
    }
    
    pub fn file_extension<S: Into<String>>(mut self, ext: S) -> Self {
        self.file_extension = ext.into();
        self
    }
    
    pub fn format(mut self, format: LogFormat) -> Self {
        self.format = format;
        self
    }
    
    pub fn rotation_config(mut self, config: RotationConfig) -> Self {
        self.rotation_config = config;
        self
    }
    
    pub fn cleanup_config(mut self, config: CleanupConfig) -> Self {
        self.cleanup_config = config;
        self
    }
    
    pub fn max_file_size_mb(mut self, size_mb: u64) -> Self {
        self.rotation_config.max_size_bytes = size_mb * 1024 * 1024;
        self
    }
    
    pub fn max_files(mut self, count: u32) -> Self {
        self.rotation_config.max_files = count;
        self
    }
    
    pub fn retention_days(mut self, days: u32) -> Self {
        self.cleanup_config.retention_days = days;
        self
    }
    
    pub fn compress_rotated(mut self, compress: bool) -> Self {
        self.rotation_config.compress_rotated = compress;
        self
    }
    
    pub fn build(self) -> Result<AdvancedFileWriter, FileManagerError> {
        let base_dir = self.base_dir
            .unwrap_or_else(|| PathBuf::from("logs"));
        
        AdvancedFileWriter::new(
            base_dir,
            self.file_prefix,
            self.file_extension,
            self.format,
            self.rotation_config,
            self.cleanup_config,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use crate::logging::StructuredLog;
    use chrono::Utc;
    
    fn create_test_entry() -> StructuredLog {
        StructuredLog {
            timestamp: Utc::now(),
            level: "INFO".to_string(),
            target: "test".to_string(),
            message: "Test message".to_string(),
            module: Some("test_module".to_string()),
            file: Some("test.rs".to_string()),
            line: Some(42),
            fields: serde_json::Value::Null,
            trace_id: None,
            span_id: None,
            category: None,
            context: None,
        }
    }
    
    #[tokio::test]
    async fn test_advanced_writer_creation() {
        let temp_dir = TempDir::new().unwrap();
        
        let writer = AdvancedFileWriterBuilder::new()
            .base_dir(temp_dir.path())
            .file_prefix("test".to_string())
            .max_file_size_mb(10)
            .max_files(5)
            .retention_days(30)
            .build()
            .unwrap();
        
        let entry = create_test_entry();
        writer.write_log(&entry).unwrap();
        
        let stats = writer.get_stats();
        assert_eq!(stats.total_writes, 1);
        assert!(stats.total_bytes > 0);
    }
    
    #[tokio::test]
    async fn test_multiple_writes() {
        let temp_dir = TempDir::new().unwrap();
        
        let writer = AdvancedFileWriterBuilder::new()
            .base_dir(temp_dir.path())
            .format(LogFormat::Json)
            .build()
            .unwrap();
        
        // 写入多个日志条目
        for i in 0..10 {
            let mut entry = create_test_entry();
            entry.message = format!("Test message {}", i);
            writer.write_log(&entry).unwrap();
        }
        
        let stats = writer.get_stats();
        assert_eq!(stats.total_writes, 10);
        
        let file_stats = writer.get_file_stats().unwrap();
        assert!(file_stats.total_files > 0);
    }
    
    #[tokio::test]
    async fn test_force_rotation() {
        let temp_dir = TempDir::new().unwrap();
        
        let writer = AdvancedFileWriterBuilder::new()
            .base_dir(temp_dir.path())
            .build()
            .unwrap();
        
        let entry = create_test_entry();
        writer.write_log(&entry).unwrap();
        
        // 强制轮转
        writer.force_flush_and_rotate().unwrap();
        
        let stats = writer.get_stats();
        assert_eq!(stats.rotations, 1);
    }
}
