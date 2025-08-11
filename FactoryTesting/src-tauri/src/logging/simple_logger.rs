/// 简化版Logger实现
/// 专注于核心功能验证，避免复杂的异步处理

use super::*;
use log::{Log, Metadata, Record};
use std::sync::{Arc, Mutex};
use std::fs::OpenOptions;
use std::io::{Write as IoWrite, BufWriter};
use chrono::Local;

/// 简化版Logger - 实现log::Log trait
pub struct SimpleLogger {
    config: LoggerConfig,
    file_writer: Arc<Mutex<Option<BufWriter<std::fs::File>>>>,
}

impl SimpleLogger {
    pub fn new(config: LoggerConfig) -> Self {
        Self {
            config,
            file_writer: Arc::new(Mutex::new(None)),
        }
    }

    pub fn init(&self) -> Result<(), Box<dyn std::error::Error>> {
        // 配置文件输出
        for target in &self.config.targets {
            if let LogTarget::File { path } = target {
                if let Some(parent) = path.parent() {
                    std::fs::create_dir_all(parent)?;
                }
                
                let file = OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(path)?;
                    
                let mut writer_guard = self.file_writer.lock().map_err(|_| "文件写入器锁定失败")?;
                *writer_guard = Some(BufWriter::new(file));
                break; // 只处理第一个文件目标
            }
        }

        // 设置为全局logger
        let logger = SimpleLogger {
            config: self.config.clone(),
            file_writer: self.file_writer.clone(),
        };
        
        log::set_boxed_logger(Box::new(logger))?;
        log::set_max_level(self.config.level.clone().into());
        
        Ok(())
    }
    
    fn write_to_console(&self, record: &Record) {
        let message = format!(
            "[{}] [{}] {}",
            Local::now().format("%Y-%m-%d %H:%M:%S%.3f"),
            record.level(),
            record.args()
        );
        
        match record.level() {
            log::Level::Error => {
                eprintln!("\x1b[31m{}\x1b[0m", message);
                let _ = std::io::stderr().flush();
            }
            log::Level::Warn => {
                eprintln!("\x1b[33m{}\x1b[0m", message);
                let _ = std::io::stderr().flush();
            }
            _ => {
                println!("{}", message);
                let _ = std::io::stdout().flush();
            }
        }
    }
    
    fn write_to_file(&self, record: &Record) {
        if let Ok(mut writer_guard) = self.file_writer.lock() {
            if let Some(ref mut writer) = writer_guard.as_mut() {
                let message = format!(
                    "[{}] [{}] [{}] - {}\n",
                    Local::now().format("%Y-%m-%d %H:%M:%S%.3f"),
                    record.level(),
                    record.target(),
                    record.args()
                );
                
                if let Err(e) = writer.write_all(message.as_bytes()) {
                    eprintln!("写入日志文件失败: {}", e);
                } else {
                    let _ = writer.flush();
                }
            }
        }
    }
}

impl Log for SimpleLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= log::max_level()
    }

    fn log(&self, record: &Record) {
        if !self.enabled(record.metadata()) {
            return;
        }

        // 写入控制台
        for target in &self.config.targets {
            match target {
                LogTarget::Console => self.write_to_console(record),
                LogTarget::File { .. } => self.write_to_file(record),
                _ => {} // 其他目标暂未实现
            }
        }
    }

    fn flush(&self) {
        let _ = std::io::stdout().flush();
        let _ = std::io::stderr().flush();
        
        if let Ok(mut writer_guard) = self.file_writer.lock() {
            if let Some(ref mut writer) = writer_guard.as_mut() {
                let _ = writer.flush();
            }
        }
    }
}

/// 便捷初始化函数
pub fn init_simple_logger() -> Result<(), Box<dyn std::error::Error>> {
    let config = LoggerConfig {
        level: LogLevel::Info,
        targets: vec![
            LogTarget::Console,
            LogTarget::File { path: std::path::PathBuf::from("logs/simple_test.log") }
        ],
        format: LogFormat::Structured,
        rotation: LogRotation {
            max_file_size_mb: 10,
            max_files: 5,
            strategy: RotationStrategy::Size,
        },
        sanitization: SanitizationConfig {
            enabled: false,
            sensitive_fields: vec![],
            mode: SanitizationMode::Mask,
        },
        cleanup: LogCleanupConfig {
            enabled: false,
            retention_days: 30,
            check_interval_hours: 24,
        },
    };
    
    let logger = SimpleLogger::new(config);
    logger.init()?;
    
    Ok(())
}