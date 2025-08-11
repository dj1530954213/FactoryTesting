//! 日志清理任务调度器
//!
//! 负责在后台定期执行日志清理任务，包括:
//! - 过期日志文件清理
//! - 空目录删除
//! - 崩溃恢复检查
//! - 锁文件清理

use super::file_manager::{LogFileManager, CleanupConfig, FileManagerError};
use std::{
    sync::{Arc, atomic::{AtomicBool, Ordering}},
    time::Duration,
    path::Path,
    fs,
};
use log::{debug, error, info, warn};
use tokio::{
    task::JoinHandle,
    time::{interval, Instant, sleep},
    select,
};
use chrono::Local;

/// 清理任务调度器
pub struct CleanupScheduler {
    /// 文件管理器
    file_manager: Arc<LogFileManager>,
    /// 运行状态
    running: Arc<AtomicBool>,
    /// 后台任务句柄
    task_handle: Option<JoinHandle<()>>,
    /// 配置
    config: CleanupSchedulerConfig,
}

/// 清理调度器配置
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CleanupSchedulerConfig {
    /// 清理间隔（小时）
    pub cleanup_interval_hours: u32,
    /// 健康检查间隔（分钟）
    pub health_check_interval_minutes: u32,
    /// 锁文件检查间隔（分钟）
    pub lock_cleanup_interval_minutes: u32,
    /// 空目录清理间隔（小时）
    pub empty_dir_cleanup_interval_hours: u32,
    /// 是否启用自动清理
    pub auto_cleanup_enabled: bool,
    /// 错误后重试间隔（秒）
    pub error_retry_interval_seconds: u64,
}

struct CleanupStats {
    last_cleanup: Option<Instant>,
    last_health_check: Option<Instant>,
    last_lock_cleanup: Option<Instant>,
    last_empty_dir_cleanup: Option<Instant>,
    total_cleanups: u64,
    total_files_cleaned: u64,
    total_errors: u64,
}

impl Default for CleanupSchedulerConfig {
    fn default() -> Self {
        Self {
            cleanup_interval_hours: 24,        // 每24小时清理一次
            health_check_interval_minutes: 30,  // 每30分钟健康检查
            lock_cleanup_interval_minutes: 60,  // 每60分钟清理锁文件
            empty_dir_cleanup_interval_hours: 6, // 每6小时清理空目录
            auto_cleanup_enabled: true,
            error_retry_interval_seconds: 300,  // 5分钟后重试
        }
    }
}

impl CleanupScheduler {
    /// 创建新的清理调度器
    pub fn new(
        file_manager: Arc<LogFileManager>,
        config: CleanupSchedulerConfig,
    ) -> Self {
        Self {
            file_manager,
            running: Arc::new(AtomicBool::new(false)),
            task_handle: None,
            config,
        }
    }
    
    /// 启动清理调度器
    pub fn start(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if self.running.load(Ordering::Relaxed) {
            warn!("清理调度器已经在运行");
            return Ok(());
        }
        
        info!("启动日志清理调度器...");
        
        self.running.store(true, Ordering::Relaxed);
        
        let file_manager = Arc::clone(&self.file_manager);
        let running = Arc::clone(&self.running);
        let config = self.config.clone();
        
        let task_handle = tokio::spawn(async move {
            Self::run_scheduler(file_manager, running, config).await;
        });
        
        self.task_handle = Some(task_handle);
        info!("日志清理调度器已启动");
        
        Ok(())
    }
    
    /// 停止清理调度器
    pub async fn stop(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if !self.running.load(Ordering::Relaxed) {
            return Ok(());
        }
        
        info!("停止日志清理调度器...");
        
        self.running.store(false, Ordering::Relaxed);
        
        if let Some(handle) = self.task_handle.take() {
            // 给一个合理的超时时间
            let timeout = Duration::from_secs(10);
            
            match tokio::time::timeout(timeout, handle).await {
                Ok(result) => {
                    if let Err(e) = result {
                        warn!("清理任务结束时发生错误: {:?}", e);
                    }
                }
                Err(_) => {
                    warn!("清理任务停止超时，强制结束");
                }
            }
        }
        
        info!("日志清理调度器已停止");
        Ok(())
    }
    
    /// 主调度循环
    async fn run_scheduler(
        file_manager: Arc<LogFileManager>,
        running: Arc<AtomicBool>,
        config: CleanupSchedulerConfig,
    ) {
        let mut stats = CleanupStats {
            last_cleanup: None,
            last_health_check: None,
            last_lock_cleanup: None,
            last_empty_dir_cleanup: None,
            total_cleanups: 0,
            total_files_cleaned: 0,
            total_errors: 0,
        };
        
        // 初始化各种间隔定时器
        let mut cleanup_interval = interval(Duration::from_secs(config.cleanup_interval_hours as u64 * 3600));
        let mut health_check_interval = interval(Duration::from_secs(config.health_check_interval_minutes as u64 * 60));
        let mut lock_cleanup_interval = interval(Duration::from_secs(config.lock_cleanup_interval_minutes as u64 * 60));
        let mut empty_dir_cleanup_interval = interval(Duration::from_secs(config.empty_dir_cleanup_interval_hours as u64 * 3600));
        
        info!("清理调度器主循环已启动");
        
        // 初始化时执行一次清理
        if config.auto_cleanup_enabled {
            Self::perform_cleanup(&file_manager, &mut stats, &config).await;
        }
        
        while running.load(Ordering::Relaxed) {
            select! {
                _ = cleanup_interval.tick() => {
                    if config.auto_cleanup_enabled {
                        Self::perform_cleanup(&file_manager, &mut stats, &config).await;
                    }
                }
                _ = health_check_interval.tick() => {
                    Self::perform_health_check(&file_manager, &mut stats).await;
                }
                _ = lock_cleanup_interval.tick() => {
                    Self::perform_lock_cleanup(&file_manager, &mut stats).await;
                }
                _ = empty_dir_cleanup_interval.tick() => {
                    Self::perform_empty_dir_cleanup(&file_manager, &mut stats).await;
                }
            }
            
            // 等待片刻再检查，避免过度消耗CPU
            sleep(Duration::from_millis(100)).await;
        }
        
        info!("清理调度器主循环已退出");
    }
    
    /// 执行日志清理
    async fn perform_cleanup(
        file_manager: &Arc<LogFileManager>,
        stats: &mut CleanupStats,
        config: &CleanupSchedulerConfig,
    ) {
        let start_time = Instant::now();
        debug!("开始执行日志清理...");
        
        match file_manager.cleanup_expired_logs() {
            Ok(cleaned_count) => {
                stats.last_cleanup = Some(start_time);
                stats.total_cleanups += 1;
                stats.total_files_cleaned += cleaned_count as u64;
                
                if cleaned_count > 0 {
                    info!(
                        "日志清理完成，清理了 {} 个文件，耗时 {:.2}秒",
                        cleaned_count,
                        start_time.elapsed().as_secs_f32()
                    );
                } else {
                    debug!("清理检查完成，无需清理的文件");
                }
            }
            Err(e) => {
                stats.total_errors += 1;
                error!("清理日志文件失败: {}", e);
                
                // 错误后等待一段时间再重试
                sleep(Duration::from_secs(config.error_retry_interval_seconds)).await;
            }
        }
    }
    
    /// 执行健康检查
    async fn perform_health_check(
        file_manager: &Arc<LogFileManager>,
        stats: &mut CleanupStats,
    ) {
        let start_time = Instant::now();
        
        match file_manager.get_file_stats() {
            Ok(file_stats) => {
                stats.last_health_check = Some(start_time);
                
                debug!(
                    "健康检查: {} 个文件，总大小 {} 字节，{} 个活跃文件",
                    file_stats.total_files,
                    file_stats.total_size,
                    file_stats.active_files
                );
                
                // 检查空间使用情况
                const GB: u64 = 1024 * 1024 * 1024;
                if file_stats.total_size > 10 * GB {
                    warn!(
                        "日志文件占用空间较大: {:.2} GB",
                        file_stats.total_size as f64 / GB as f64
                    );
                }
                
                if file_stats.total_files > 1000 {
                    warn!(
                        "日志文件数量较多: {} 个文件",
                        file_stats.total_files
                    );
                }
            }
            Err(e) => {
                stats.total_errors += 1;
                error!("健康检查失败: {}", e);
            }
        }
    }
    
    /// 执行锁文件清理
    async fn perform_lock_cleanup(
        file_manager: &Arc<LogFileManager>,
        stats: &mut CleanupStats,
    ) {
        let start_time = Instant::now();
        debug!("执行锁文件清理...");
        
        // 获取日志基础目录
        // 这里需要从 file_manager 中获取基础目录，但是 LogFileManager 没有提供公开接口
        // 暂时使用硬编码路径
        let logs_base_path = Path::new("logs");
        
        if !logs_base_path.exists() {
            return;
        }
        
        let mut cleaned_locks = 0;
        
        match Self::cleanup_stale_lock_files(logs_base_path) {
            Ok(count) => {
                cleaned_locks = count;
                if count > 0 {
                    info!("清理了 {} 个过期锁文件", count);
                }
            }
            Err(e) => {
                stats.total_errors += 1;
                error!("锁文件清理失败: {}", e);
                return;
            }
        }
        
        stats.last_lock_cleanup = Some(start_time);
        
        if cleaned_locks > 0 {
            info!(
                "锁文件清理完成，清理了 {} 个文件，耗时 {:.2}秒",
                cleaned_locks,
                start_time.elapsed().as_secs_f32()
            );
        } else {
            debug!("锁文件检查完成，无需清理的文件");
        }
    }
    
    /// 执行空目录清理
    async fn perform_empty_dir_cleanup(
        file_manager: &Arc<LogFileManager>,
        stats: &mut CleanupStats,
    ) {
        let start_time = Instant::now();
        debug!("执行空目录清理...");
        
        let logs_base_path = Path::new("logs");
        
        if !logs_base_path.exists() {
            return;
        }
        
        let mut cleaned_dirs = 0;
        
        match Self::cleanup_empty_directories(logs_base_path) {
            Ok(count) => {
                cleaned_dirs = count;
                if count > 0 {
                    info!("清理了 {} 个空目录", count);
                }
            }
            Err(e) => {
                stats.total_errors += 1;
                error!("空目录清理失败: {}", e);
                return;
            }
        }
        
        stats.last_empty_dir_cleanup = Some(start_time);
        
        if cleaned_dirs > 0 {
            info!(
                "空目录清理完成，清理了 {} 个目录，耗时 {:.2}秒",
                cleaned_dirs,
                start_time.elapsed().as_secs_f32()
            );
        } else {
            debug!("空目录检查完成，无需清理的目录");
        }
    }
    
    /// 清理过期的锁文件
    fn cleanup_stale_lock_files(base_path: &Path) -> Result<u32, Box<dyn std::error::Error + Send + Sync>> {
        let mut cleaned_count = 0;
        let cutoff_time = std::time::SystemTime::now() - Duration::from_secs(3600); // 1小时前
        
        Self::cleanup_locks_recursive(base_path, cutoff_time, &mut cleaned_count)?;
        
        Ok(cleaned_count)
    }
    
    /// 递归清理锁文件
    fn cleanup_locks_recursive(
        dir: &Path,
        cutoff_time: std::time::SystemTime,
        cleaned_count: &mut u32,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let entries = fs::read_dir(dir)?;
        
        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_file() {
                if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                    if file_name.ends_with(".lock") {
                        if let Ok(metadata) = fs::metadata(&path) {
                            if let Ok(modified) = metadata.modified() {
                                if modified < cutoff_time {
                                    if let Err(e) = fs::remove_file(&path) {
                                        warn!("删除过期锁文件失败 {}: {}", path.display(), e);
                                    } else {
                                        debug!("删除过期锁文件: {}", path.display());
                                        *cleaned_count += 1;
                                    }
                                }
                            }
                        }
                    }
                }
            } else if path.is_dir() {
                Self::cleanup_locks_recursive(&path, cutoff_time, cleaned_count)?;
            }
        }
        
        Ok(())
    }
    
    /// 清理空目录
    fn cleanup_empty_directories(base_path: &Path) -> Result<u32, Box<dyn std::error::Error + Send + Sync>> {
        let mut cleaned_count = 0;
        Self::cleanup_empty_dirs_recursive(base_path, base_path, &mut cleaned_count)?;
        Ok(cleaned_count)
    }
    
    /// 递归清理空目录
    fn cleanup_empty_dirs_recursive(
        dir: &Path,
        base_path: &Path,
        cleaned_count: &mut u32,
    ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        let entries = match fs::read_dir(dir) {
            Ok(entries) => entries,
            Err(_) => return Ok(false), // 无法读取的目录不处理
        };
        
        let mut has_files = false;
        let mut subdirs = Vec::new();
        
        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_file() {
                has_files = true;
            } else if path.is_dir() {
                subdirs.push(path);
            }
        }
        
        // 先处理子目录
        for subdir in subdirs {
            let subdir_empty = Self::cleanup_empty_dirs_recursive(&subdir, base_path, cleaned_count)?;
            if !subdir_empty {
                has_files = true; // 子目录不为空，所以当前目录也不为空
            }
        }
        
        // 如果当前目录为空且不是基础目录，尝试删除
        if !has_files && dir != base_path {
            match fs::remove_dir(dir) {
                Ok(()) => {
                    debug!("删除空目录: {}", dir.display());
                    *cleaned_count += 1;
                    return Ok(true); // 目录已被删除
                }
                Err(e) => {
                    warn!("删除空目录失败 {}: {}", dir.display(), e);
                }
            }
        }
        
        Ok(!has_files)
    }
    
    /// 立即执行一次完整清理
    pub async fn run_cleanup_now(&self) -> Result<CleanupResult, Box<dyn std::error::Error + Send + Sync>> {
        info!("手动触发完整清理任务...");
        
        let start_time = Instant::now();
        let mut result = CleanupResult::default();
        
        // 1. 清理过期日志文件
        match self.file_manager.cleanup_expired_logs() {
            Ok(count) => {
                result.cleaned_files = count;
                info!("清理了 {} 个过期日志文件", count);
            }
            Err(e) => {
                result.errors.push(format!("清理过期日志失败: {}", e));
            }
        }
        
        // 2. 清理锁文件
        match Self::cleanup_stale_lock_files(Path::new("logs")) {
            Ok(count) => {
                result.cleaned_lock_files = count;
                if count > 0 {
                    info!("清理了 {} 个过期锁文件", count);
                }
            }
            Err(e) => {
                result.errors.push(format!("清理锁文件失败: {}", e));
            }
        }
        
        // 3. 清理空目录
        match Self::cleanup_empty_directories(Path::new("logs")) {
            Ok(count) => {
                result.cleaned_directories = count;
                if count > 0 {
                    info!("清理了 {} 个空目录", count);
                }
            }
            Err(e) => {
                result.errors.push(format!("清理空目录失败: {}", e));
            }
        }
        
        result.duration = start_time.elapsed();
        
        if result.errors.is_empty() {
            info!(
                "手动清理任务完成，清理了 {} 个文件、{} 个锁文件、{} 个目录，耗时 {:.2}秒",
                result.cleaned_files,
                result.cleaned_lock_files,
                result.cleaned_directories,
                result.duration.as_secs_f32()
            );
        } else {
            warn!(
                "手动清理任务完成，但有 {} 个错误",
                result.errors.len()
            );
        }
        
        Ok(result)
    }
    
    /// 检查调度器是否正在运行
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::Relaxed)
    }
}

/// 清理结果
#[derive(Debug, Default)]
pub struct CleanupResult {
    pub cleaned_files: u32,
    pub cleaned_lock_files: u32,
    pub cleaned_directories: u32,
    pub duration: Duration,
    pub errors: Vec<String>,
}

impl Drop for CleanupScheduler {
    fn drop(&mut self) {
        self.running.store(false, Ordering::Relaxed);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::io::Write;
    use crate::logging::file_manager::{RotationConfig, CleanupConfig};
    
    async fn create_test_scheduler() -> (CleanupScheduler, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        
        let file_manager = Arc::new(
            LogFileManager::new(
                temp_dir.path(),
                "test".to_string(),
                "log".to_string(),
                RotationConfig::default(),
                CleanupConfig::default(),
            ).unwrap()
        );
        
        let config = CleanupSchedulerConfig {
            cleanup_interval_hours: 1,
            health_check_interval_minutes: 5,
            lock_cleanup_interval_minutes: 10,
            empty_dir_cleanup_interval_hours: 2,
            auto_cleanup_enabled: true,
            error_retry_interval_seconds: 5,
        };
        
        let scheduler = CleanupScheduler::new(file_manager, config);
        (scheduler, temp_dir)
    }
    
    #[tokio::test]
    async fn test_scheduler_start_stop() {
        let (mut scheduler, _temp_dir) = create_test_scheduler().await;
        
        assert!(!scheduler.is_running());
        
        scheduler.start().unwrap();
        assert!(scheduler.is_running());
        
        scheduler.stop().await.unwrap();
        assert!(!scheduler.is_running());
    }
    
    #[tokio::test]
    async fn test_manual_cleanup() {
        let (scheduler, temp_dir) = create_test_scheduler().await;
        
        // 创建一些测试文件
        let test_file = temp_dir.path().join("test.log");
        let mut file = std::fs::File::create(&test_file).unwrap();
        file.write_all(b"test content").unwrap();
        
        // 创建一个锁文件
        let lock_file = temp_dir.path().join("test.log.lock");
        std::fs::File::create(&lock_file).unwrap();
        
        let result = scheduler.run_cleanup_now().await.unwrap();
        
        // 验证清理结果
        assert!(result.errors.is_empty());
    }
    
    #[tokio::test]
    async fn test_empty_directory_cleanup() {
        let (scheduler, temp_dir) = create_test_scheduler().await;
        
        // 创建一个空目录
        let empty_dir = temp_dir.path().join("empty");
        fs::create_dir(&empty_dir).unwrap();
        
        let cleaned = CleanupScheduler::cleanup_empty_directories(temp_dir.path()).unwrap();
        assert!(cleaned > 0);
        
        // 验证目录已被删除
        assert!(!empty_dir.exists());
    }
    
    #[tokio::test]
    async fn test_stale_lock_cleanup() {
        let (scheduler, temp_dir) = create_test_scheduler().await;
        
        // 创建一个过期锁文件
        let lock_file = temp_dir.path().join("old.log.lock");
        std::fs::File::create(&lock_file).unwrap();
        
        // 设置文件时间为过去
        let old_time = std::time::SystemTime::now() - Duration::from_secs(7200);
        if let Ok(file) = std::fs::File::open(&lock_file) {
            let _ = file.set_times(
                filetime::FileTime::from_system_time(old_time), 
                filetime::FileTime::from_system_time(old_time)
            );
        }
        
        let cleaned = CleanupScheduler::cleanup_stale_lock_files(temp_dir.path()).unwrap();
        assert!(cleaned > 0);
        
        // 验证锁文件已被删除
        assert!(!lock_file.exists());
    }
}
