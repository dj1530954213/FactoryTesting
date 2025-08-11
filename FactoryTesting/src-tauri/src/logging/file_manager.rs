//! 日志文件管理系统
//!
//! 负责日志文件的创建、轮转、清理和并发安全管理
//! 实现企业级日志文件管理策略

use std::{
    collections::HashMap,
    fs::{self, File, OpenOptions},
    io::{self, BufWriter, Write},
    path::{Path, PathBuf},
    sync::{Arc, Mutex, RwLock},
    time::{SystemTime, UNIX_EPOCH},
};
use chrono::{DateTime, Local, Utc};
use serde::{Deserialize, Serialize};
use log::{debug, error, info, warn};

/// 文件管理器错误类型
#[derive(Debug, thiserror::Error)]
pub enum FileManagerError {
    #[error("文件I/O错误: {0}")]
    IoError(#[from] io::Error),
    #[error("路径错误: {message}")]
    PathError { message: String },
    #[error("锁文件错误: {message}")]
    LockError { message: String },
    #[error("配置错误: {message}")]
    ConfigError { message: String },
}

/// 文件轮转策略配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RotationConfig {
    /// 最大文件大小（字节）
    pub max_size_bytes: u64,
    /// 最大文件数量
    pub max_files: u32,
    /// 时间轮转间隔（小时）
    pub time_interval_hours: Option<u32>,
    /// 是否启用压缩
    pub compress_rotated: bool,
}

/// 清理策略配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanupConfig {
    /// 保留天数
    pub retention_days: u32,
    /// 清理检查间隔（小时）
    pub check_interval_hours: u32,
    /// 是否清理空目录
    pub remove_empty_dirs: bool,
    /// 是否只清理压缩文件
    pub cleanup_compressed_only: bool,
}

/// 文件元数据信息
#[derive(Debug, Clone)]
pub struct LogFileInfo {
    pub path: PathBuf,
    pub size: u64,
    pub created: SystemTime,
    pub modified: SystemTime,
    pub is_current: bool,
    pub sequence_number: u32,
}

/// 日志文件管理器
/// 负责按日期组织文件、轮转、清理和并发安全
pub struct LogFileManager {
    /// 基础日志目录
    base_dir: PathBuf,
    /// 文件名前缀
    file_prefix: String,
    /// 文件扩展名
    file_extension: String,
    /// 轮转配置
    rotation_config: RotationConfig,
    /// 清理配置
    cleanup_config: CleanupConfig,
    /// 当前活跃文件映射 (日期 -> 文件信息)
    active_files: Arc<RwLock<HashMap<String, LogFileInfo>>>,
    /// 锁文件管理
    lock_files: Arc<Mutex<HashMap<String, File>>>,
    /// 最后清理时间
    last_cleanup: Arc<Mutex<SystemTime>>,
}

impl LogFileManager {
    /// 创建新的文件管理器
    pub fn new<P: AsRef<Path>>(
        base_dir: P,
        file_prefix: String,
        file_extension: String,
        rotation_config: RotationConfig,
        cleanup_config: CleanupConfig,
    ) -> Result<Self, FileManagerError> {
        let base_dir = base_dir.as_ref().to_path_buf();
        
        // 创建基础目录
        if !base_dir.exists() {
            fs::create_dir_all(&base_dir)
                .map_err(|e| FileManagerError::PathError { 
                    message: format!("无法创建基础目录 {}: {}", base_dir.display(), e)
                })?;
        }
        
        // 验证配置
        Self::validate_config(&rotation_config, &cleanup_config)?;
        
        Ok(Self {
            base_dir,
            file_prefix,
            file_extension,
            rotation_config,
            cleanup_config,
            active_files: Arc::new(RwLock::new(HashMap::new())),
            lock_files: Arc::new(Mutex::new(HashMap::new())),
            last_cleanup: Arc::new(Mutex::new(SystemTime::now())),
        })
    }
    
    /// 验证配置参数
    fn validate_config(
        rotation: &RotationConfig,
        cleanup: &CleanupConfig,
    ) -> Result<(), FileManagerError> {
        if rotation.max_size_bytes == 0 {
            return Err(FileManagerError::ConfigError {
                message: "最大文件大小不能为0".to_string(),
            });
        }
        
        if rotation.max_files == 0 {
            return Err(FileManagerError::ConfigError {
                message: "最大文件数量不能为0".to_string(),
            });
        }
        
        if cleanup.retention_days == 0 {
            return Err(FileManagerError::ConfigError {
                message: "保留天数不能为0".to_string(),
            });
        }
        
        Ok(())
    }
    
    /// 获取当前日期的日志文件路径
    /// 按照 logs/2024-01-15/ 的格式组织目录
    pub fn get_current_log_file(&self) -> Result<PathBuf, FileManagerError> {
        let date_str = Local::now().format("%Y-%m-%d").to_string();
        let date_dir = self.base_dir.join(&date_str);
        
        // 确保日期目录存在
        if !date_dir.exists() {
            fs::create_dir_all(&date_dir)
                .map_err(|e| FileManagerError::PathError {
                    message: format!("无法创建日期目录 {}: {}", date_dir.display(), e),
                })?;
            
            info!("创建日期目录: {}", date_dir.display());
        }
        
        // 生成文件路径
        let file_path = date_dir.join(format!(
            "{}.{}",
            self.file_prefix, self.file_extension
        ));
        
        // 检查是否需要轮转
        if self.should_rotate(&file_path)? {
            self.rotate_file(&file_path)?;
        }
        
        // 更新活跃文件信息
        self.update_active_file_info(&date_str, &file_path)?;
        
        Ok(file_path)
    }
    
    /// 检查文件是否需要轮转
    fn should_rotate(&self, file_path: &Path) -> Result<bool, FileManagerError> {
        if !file_path.exists() {
            return Ok(false);
        }
        
        let metadata = fs::metadata(file_path)?;
        let file_size = metadata.len();
        
        // 检查文件大小
        if file_size >= self.rotation_config.max_size_bytes {
            debug!("文件 {} 达到大小限制，需要轮转", file_path.display());
            return Ok(true);
        }
        
        // 检查时间间隔（如果配置了）
        if let Some(interval_hours) = self.rotation_config.time_interval_hours {
            if let Ok(created) = metadata.created() {
                let elapsed = created.elapsed().unwrap_or_default();
                let threshold = std::time::Duration::from_secs(interval_hours as u64 * 3600);
                
                if elapsed >= threshold {
                    debug!("文件 {} 达到时间限制，需要轮转", file_path.display());
                    return Ok(true);
                }
            }
        }
        
        Ok(false)
    }
    
    /// 轮转日志文件
    fn rotate_file(&self, current_path: &Path) -> Result<(), FileManagerError> {
        if !current_path.exists() {
            return Ok(());
        }
        
        // 获取锁以确保轮转的原子性
        let path_str = current_path.to_string_lossy().to_string();
        let _lock = self.acquire_file_lock(&path_str)?;
        
        // 生成轮转后的文件名
        let rotated_path = self.generate_rotated_filename(current_path)?;
        
        // 执行轮转
        fs::rename(current_path, &rotated_path)
            .map_err(|e| FileManagerError::IoError(e))?;
        
        info!("文件轮转完成: {} -> {}", 
              current_path.display(), rotated_path.display());
        
        // 如果启用了压缩，压缩轮转后的文件
        if self.rotation_config.compress_rotated {
            if let Err(e) = self.compress_file(&rotated_path) {
                warn!("压缩文件失败 {}: {}", rotated_path.display(), e);
            }
        }
        
        // 清理旧文件
        self.cleanup_old_rotated_files(current_path)?;
        
        Ok(())
    }
    
    /// 生成轮转后的文件名
    /// 格式: filename.20240115_143022.001.log
    fn generate_rotated_filename(&self, original_path: &Path) -> Result<PathBuf, FileManagerError> {
        let timestamp = Local::now().format("%Y%m%d_%H%M%S").to_string();
        let parent = original_path.parent().unwrap_or_else(|| Path::new("."));
        
        // 查找已存在的相同时间戳文件，生成序号
        let mut sequence = 1;
        loop {
            let rotated_name = format!(
                "{}.{}.{:03}.{}",
                self.file_prefix,
                timestamp,
                sequence,
                self.file_extension
            );
            
            let rotated_path = parent.join(rotated_name);
            if !rotated_path.exists() {
                return Ok(rotated_path);
            }
            
            sequence += 1;
            if sequence > 999 {
                return Err(FileManagerError::PathError {
                    message: "无法生成唯一的轮转文件名".to_string(),
                });
            }
        }
    }
    
    /// 压缩文件
    fn compress_file(&self, file_path: &Path) -> Result<(), FileManagerError> {
        use std::process::Command;
        
        // 使用系统的gzip命令压缩文件
        let output = Command::new("gzip")
            .arg(file_path)
            .output();
        
        match output {
            Ok(result) if result.status.success() => {
                info!("文件压缩成功: {}.gz", file_path.display());
                Ok(())
            }
            Ok(result) => {
                let error_msg = String::from_utf8_lossy(&result.stderr);
                Err(FileManagerError::IoError(io::Error::new(
                    io::ErrorKind::Other,
                    format!("压缩失败: {}", error_msg),
                )))
            }
            Err(e) => {
                // 如果gzip不可用，跳过压缩
                warn!("无法执行gzip压缩，跳过: {}", e);
                Ok(())
            }
        }
    }
    
    /// 清理旧的轮转文件
    fn cleanup_old_rotated_files(&self, base_path: &Path) -> Result<(), FileManagerError> {
        let parent = base_path.parent().unwrap_or_else(|| Path::new("."));
        
        // 收集所有轮转文件
        let mut rotated_files = Vec::new();
        
        let entries = fs::read_dir(parent)?;
        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            let file_name = path.file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("");
            
            // 匹配轮转文件模式: prefix.timestamp.seq.ext 或 prefix.timestamp.seq.ext.gz
            if file_name.starts_with(&self.file_prefix) &&
               (file_name.ends_with(&self.file_extension) ||
                file_name.ends_with(&format!("{}.gz", self.file_extension))) &&
               file_name.contains(".") {
                
                if let Ok(metadata) = entry.metadata() {
                    rotated_files.push((path, metadata.created().unwrap_or(UNIX_EPOCH)));
                }
            }
        }
        
        // 按创建时间排序，删除多余的文件
        rotated_files.sort_by_key(|(_, created)| *created);
        
        if rotated_files.len() > self.rotation_config.max_files as usize {
            let files_to_remove = rotated_files.len() - self.rotation_config.max_files as usize;
            
            for (path, _) in rotated_files.iter().take(files_to_remove) {
                if let Err(e) = fs::remove_file(path) {
                    warn!("删除旧轮转文件失败 {}: {}", path.display(), e);
                } else {
                    debug!("删除旧轮转文件: {}", path.display());
                }
            }
        }
        
        Ok(())
    }
    
    /// 更新活跃文件信息
    fn update_active_file_info(&self, date: &str, file_path: &Path) -> Result<(), FileManagerError> {
        let file_info = if file_path.exists() {
            let metadata = fs::metadata(file_path)?;
            LogFileInfo {
                path: file_path.to_path_buf(),
                size: metadata.len(),
                created: metadata.created().unwrap_or(UNIX_EPOCH),
                modified: metadata.modified().unwrap_or(UNIX_EPOCH),
                is_current: true,
                sequence_number: 0,
            }
        } else {
            LogFileInfo {
                path: file_path.to_path_buf(),
                size: 0,
                created: SystemTime::now(),
                modified: SystemTime::now(),
                is_current: true,
                sequence_number: 0,
            }
        };
        
        if let Ok(mut active_files) = self.active_files.write() {
            active_files.insert(date.to_string(), file_info);
        }
        
        Ok(())
    }
    
    /// 获取文件锁
    fn acquire_file_lock(&self, path: &str) -> Result<(), FileManagerError> {
        let lock_file_path = format!("{}.lock", path);
        
        let mut lock_files = self.lock_files.lock()
            .map_err(|_| FileManagerError::LockError {
                message: "无法获取锁文件映射的锁".to_string(),
            })?;
        
        if !lock_files.contains_key(&lock_file_path) {
            let lock_file = OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(true)
                .open(&lock_file_path)?;
            
            lock_files.insert(lock_file_path, lock_file);
        }
        
        Ok(())
    }
    
    /// 清理过期的日志文件
    /// 根据配置的保留天数清理旧文件
    pub fn cleanup_expired_logs(&self) -> Result<u32, FileManagerError> {
        // 检查是否需要执行清理
        {
            let last_cleanup = self.last_cleanup.lock()
                .map_err(|_| FileManagerError::LockError {
                    message: "无法获取清理时间锁".to_string(),
                })?;
            
            let elapsed = last_cleanup.elapsed().unwrap_or_default();
            let check_interval = std::time::Duration::from_secs(
                self.cleanup_config.check_interval_hours as u64 * 3600
            );
            
            if elapsed < check_interval {
                return Ok(0);
            }
        }
        
        info!("开始清理过期日志文件...");
        
        let cutoff_time = SystemTime::now() - 
            std::time::Duration::from_secs(
                self.cleanup_config.retention_days as u64 * 24 * 3600
            );
        
        let mut cleaned_count = 0;
        let mut empty_dirs = Vec::new();
        
        // 递归清理日志目录
        cleaned_count += self.cleanup_directory(&self.base_dir, cutoff_time, &mut empty_dirs)?;
        
        // 清理空目录
        if self.cleanup_config.remove_empty_dirs {
            for dir in empty_dirs {
                if self.is_directory_empty(&dir)? {
                    if let Err(e) = fs::remove_dir(&dir) {
                        warn!("删除空目录失败 {}: {}", dir.display(), e);
                    } else {
                        debug!("删除空目录: {}", dir.display());
                    }
                }
            }
        }
        
        // 更新最后清理时间
        {
            let mut last_cleanup = self.last_cleanup.lock()
                .map_err(|_| FileManagerError::LockError {
                    message: "无法获取清理时间锁".to_string(),
                })?;
            *last_cleanup = SystemTime::now();
        }
        
        if cleaned_count > 0 {
            info!("清理完成，删除了 {} 个过期日志文件", cleaned_count);
        }
        
        Ok(cleaned_count)
    }
    
    /// 清理单个目录
    fn cleanup_directory(
        &self,
        dir: &Path,
        cutoff_time: SystemTime,
        empty_dirs: &mut Vec<PathBuf>,
    ) -> Result<u32, FileManagerError> {
        let mut cleaned_count = 0;
        let mut has_files = false;
        
        let entries = match fs::read_dir(dir) {
            Ok(entries) => entries,
            Err(e) => {
                warn!("无法读取目录 {}: {}", dir.display(), e);
                return Ok(0);
            }
        };
        
        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_file() {
                has_files = true;
                
                // 检查是否是日志文件
                if self.is_log_file(&path) {
                    if let Ok(metadata) = fs::metadata(&path) {
                        let modified = metadata.modified().unwrap_or(UNIX_EPOCH);
                        
                        if modified < cutoff_time {
                            // 检查是否只清理压缩文件
                            if self.cleanup_config.cleanup_compressed_only {
                                if !path.extension()
                                    .and_then(|ext| ext.to_str())
                                    .unwrap_or("").eq("gz") {
                                    continue;
                                }
                            }
                            
                            if let Err(e) = fs::remove_file(&path) {
                                warn!("删除过期文件失败 {}: {}", path.display(), e);
                            } else {
                                debug!("删除过期文件: {}", path.display());
                                cleaned_count += 1;
                            }
                        }
                    }
                }
            } else if path.is_dir() {
                cleaned_count += self.cleanup_directory(&path, cutoff_time, empty_dirs)?;
            }
        }
        
        // 如果目录可能为空，添加到清理列表
        if !has_files {
            empty_dirs.push(dir.to_path_buf());
        }
        
        Ok(cleaned_count)
    }
    
    /// 检查是否是日志文件
    fn is_log_file(&self, path: &Path) -> bool {
        let file_name = path.file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("");
        
        file_name.starts_with(&self.file_prefix) &&
        (file_name.ends_with(&self.file_extension) ||
         file_name.ends_with(&format!("{}.gz", self.file_extension))) ||
        file_name.ends_with(".lock")
    }
    
    /// 检查目录是否为空
    fn is_directory_empty(&self, dir: &Path) -> Result<bool, FileManagerError> {
        let entries = fs::read_dir(dir)?;
        Ok(entries.count() == 0)
    }
    
    /// 获取日志文件统计信息
    pub fn get_file_stats(&self) -> Result<LogFileStats, FileManagerError> {
        let mut total_files = 0;
        let mut total_size = 0;
        let mut oldest_file: Option<SystemTime> = None;
        let mut newest_file: Option<SystemTime> = None;
        
        self.collect_stats(&self.base_dir, &mut total_files, &mut total_size, &mut oldest_file, &mut newest_file)?;
        
        Ok(LogFileStats {
            total_files,
            total_size,
            oldest_file,
            newest_file,
            active_files: self.active_files.read()
                .map(|files| files.len() as u32)
                .unwrap_or(0),
        })
    }
    
    /// 收集统计信息
    fn collect_stats(
        &self,
        dir: &Path,
        total_files: &mut u32,
        total_size: &mut u64,
        oldest_file: &mut Option<SystemTime>,
        newest_file: &mut Option<SystemTime>,
    ) -> Result<(), FileManagerError> {
        let entries = fs::read_dir(dir)?;
        
        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_file() && self.is_log_file(&path) {
                if let Ok(metadata) = fs::metadata(&path) {
                    *total_files += 1;
                    *total_size += metadata.len();
                    
                    let modified = metadata.modified().unwrap_or(UNIX_EPOCH);
                    
                    match oldest_file {
                        None => *oldest_file = Some(modified),
                        Some(current) if modified < *current => *oldest_file = Some(modified),
                        _ => {}
                    }
                    
                    match newest_file {
                        None => *newest_file = Some(modified),
                        Some(current) if modified > *current => *newest_file = Some(modified),
                        _ => {}
                    }
                }
            } else if path.is_dir() {
                self.collect_stats(&path, total_files, total_size, oldest_file, newest_file)?;
            }
        }
        
        Ok(())
    }
    
    /// 强制轮转当前文件
    pub fn force_rotate_current(&self) -> Result<(), FileManagerError> {
        let current_file = self.get_current_log_file()?;
        if current_file.exists() {
            self.rotate_file(&current_file)?;
            info!("强制轮转文件完成: {}", current_file.display());
        }
        Ok(())
    }
    
    /// 获取当前活跃文件的写入器
    pub fn get_writer(&self) -> Result<BufWriter<File>, FileManagerError> {
        let current_file = self.get_current_log_file()?;
        
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&current_file)?;
        
        Ok(BufWriter::new(file))
    }
}

/// 日志文件统计信息
#[derive(Debug, Clone)]
pub struct LogFileStats {
    pub total_files: u32,
    pub total_size: u64,
    pub oldest_file: Option<SystemTime>,
    pub newest_file: Option<SystemTime>,
    pub active_files: u32,
}

/// 默认配置
impl Default for RotationConfig {
    fn default() -> Self {
        Self {
            max_size_bytes: 100 * 1024 * 1024, // 100MB
            max_files: 10,
            time_interval_hours: Some(24), // 24小时轮转
            compress_rotated: true,
        }
    }
}

impl Default for CleanupConfig {
    fn default() -> Self {
        Self {
            retention_days: 90, // 保留90天
            check_interval_hours: 24, // 每24小时检查一次
            remove_empty_dirs: true,
            cleanup_compressed_only: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;
    
    fn create_test_file_manager() -> (LogFileManager, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let manager = LogFileManager::new(
            temp_dir.path(),
            "test_log".to_string(),
            "log".to_string(),
            RotationConfig::default(),
            CleanupConfig::default(),
        ).unwrap();
        
        (manager, temp_dir)
    }
    
    #[test]
    fn test_file_creation() {
        let (manager, _temp_dir) = create_test_file_manager();
        
        let file_path = manager.get_current_log_file().unwrap();
        assert!(file_path.to_string_lossy().contains(&Local::now().format("%Y-%m-%d").to_string()));
    }
    
    #[test]
    fn test_rotation_by_size() {
        let (manager, _temp_dir) = create_test_file_manager();
        
        // 创建一个大文件来触发轮转
        let file_path = manager.get_current_log_file().unwrap();
        {
            let mut writer = manager.get_writer().unwrap();
            let large_content = "x".repeat(manager.rotation_config.max_size_bytes as usize + 1);
            writer.write_all(large_content.as_bytes()).unwrap();
            writer.flush().unwrap();
        }
        
        // 下次获取文件时应该触发轮转
        let new_file_path = manager.get_current_log_file().unwrap();
        assert_eq!(file_path, new_file_path);
        
        // 检查文件大小
        let metadata = fs::metadata(&file_path).unwrap();
        assert!(metadata.len() < manager.rotation_config.max_size_bytes);
    }
    
    #[test]
    fn test_cleanup_expired_logs() {
        let (manager, temp_dir) = create_test_file_manager();
        
        // 创建一些旧文件
        let old_date = Local::now() - chrono::Duration::days(100);
        let old_dir = temp_dir.path().join(old_date.format("%Y-%m-%d").to_string());
        fs::create_dir_all(&old_dir).unwrap();
        
        let old_file = old_dir.join("test_log.log");
        fs::write(&old_file, "old content").unwrap();
        
        // 设置文件时间为过去
        let old_time = SystemTime::now() - std::time::Duration::from_secs(100 * 24 * 3600);
        if let Ok(file) = File::open(&old_file) {
            let _ = file.set_times(filetime::FileTime::from_system_time(old_time), 
                                   filetime::FileTime::from_system_time(old_time));
        }
        
        // 执行清理
        let cleaned_count = manager.cleanup_expired_logs().unwrap();
        assert!(cleaned_count > 0);
        
        // 验证文件已被删除
        assert!(!old_file.exists());
    }
    
    #[test]
    fn test_file_stats() {
        let (manager, _temp_dir) = create_test_file_manager();
        
        // 创建一些文件
        let _file_path = manager.get_current_log_file().unwrap();
        let mut writer = manager.get_writer().unwrap();
        writer.write_all(b"test content").unwrap();
        writer.flush().unwrap();
        
        let stats = manager.get_file_stats().unwrap();
        assert!(stats.total_files > 0);
        assert!(stats.total_size > 0);
    }
}
