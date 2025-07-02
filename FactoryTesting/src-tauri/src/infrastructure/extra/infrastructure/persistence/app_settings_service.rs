/// 应用配置服务接口定义和实现
/// 专门负责应用程序配置的持久化，与通用数据持久化分离

use async_trait::async_trait;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use crate::utils::error::{AppError, AppResult};
use crate::services::traits::BaseService;
use crate::models::structs::AppSettings;

/// 应用配置服务接口
/// 专门用于管理应用程序配置的持久化
#[async_trait]
pub trait AppSettingsService: BaseService {
    /// 保存应用配置
    async fn save_settings(&self, settings: &AppSettings) -> AppResult<()>;
    
    /// 加载应用配置
    async fn load_settings(&self) -> AppResult<Option<AppSettings>>;
    
    /// 重置为默认配置
    async fn reset_to_defaults(&self) -> AppResult<AppSettings>;
    
    /// 验证配置文件完整性
    async fn validate_settings_file(&self) -> AppResult<bool>;
}

/// 应用配置服务配置
#[derive(Debug, Clone)]
pub struct AppSettingsConfig {
    /// 配置文件存储根目录
    pub storage_root_dir: PathBuf,
    /// 配置文件名
    pub config_file_name: String,
}

impl Default for AppSettingsConfig {
    fn default() -> Self {
        Self {
            storage_root_dir: PathBuf::from("./config"),
            config_file_name: "app_settings.json".to_string(),
        }
    }
}

/// JSON文件应用配置服务实现
#[derive(Debug)]
pub struct JsonAppSettingsService {
    config: AppSettingsConfig,
    is_active: Arc<Mutex<bool>>,
}

impl JsonAppSettingsService {
    /// 创建新的JSON应用配置服务
    pub fn new(config: AppSettingsConfig) -> Self {
        Self {
            config,
            is_active: Arc::new(Mutex::new(false)),
        }
    }
    
    /// 创建默认配置的服务
    pub fn new_default() -> Self {
        Self::new(AppSettingsConfig::default())
    }
    
    /// 获取配置文件完整路径
    fn get_settings_file_path(&self) -> PathBuf {
        self.config.storage_root_dir.join(&self.config.config_file_name)
    }
    
    /// 确保配置目录存在
    async fn ensure_config_directory_exists(&self) -> AppResult<()> {
        if !self.config.storage_root_dir.exists() {
            tokio::fs::create_dir_all(&self.config.storage_root_dir).await
                .map_err(|e| AppError::io_error(
                    format!("创建配置目录 {:?} 失败", self.config.storage_root_dir),
                    e.kind().to_string()
                ))?;
        }
        Ok(())
    }
}

#[async_trait]
impl BaseService for JsonAppSettingsService {
    fn service_name(&self) -> &'static str {
        "JsonAppSettingsService"
    }

    async fn initialize(&mut self) -> AppResult<()> {
        self.ensure_config_directory_exists().await?;
        let mut active_guard = self.is_active.lock().unwrap();
        *active_guard = true;
        log::info!("{} 服务已初始化并激活", self.service_name());
        Ok(())
    }

    async fn shutdown(&mut self) -> AppResult<()> {
        let mut active_guard = self.is_active.lock().unwrap();
        *active_guard = false;
        log::info!("{} 服务已关闭", self.service_name());
        Ok(())
    }

    async fn health_check(&self) -> AppResult<()> {
        {
            let active_guard = self.is_active.lock().unwrap();
            if !*active_guard {
                return Err(AppError::service_health_check_error(
                    self.service_name().to_string(),
                    "服务未激活".to_string()
                ));
            }
        }
        
        // 检查配置目录是否可访问
        if !self.config.storage_root_dir.exists() || !self.config.storage_root_dir.is_dir() {
            return Err(AppError::service_health_check_error(
                self.service_name().to_string(),
                format!("配置目录 {:?} 不可访问", self.config.storage_root_dir)
            ));
        }
        
        Ok(())
    }
}

#[async_trait]
impl AppSettingsService for JsonAppSettingsService {
    async fn save_settings(&self, settings: &AppSettings) -> AppResult<()> {
        self.ensure_config_directory_exists().await?;
        
        let file_path = self.get_settings_file_path();
        let json_content = serde_json::to_string_pretty(settings)
            .map_err(|e| AppError::json_error(format!("序列化应用配置失败: {}", e)))?;
        
        tokio::fs::write(&file_path, json_content).await
            .map_err(|e| AppError::io_error(
                format!("写入配置文件 {:?} 失败", file_path),
                e.kind().to_string()
            ))?;
        Ok(())
    }

    async fn load_settings(&self) -> AppResult<Option<AppSettings>> {
        let file_path = self.get_settings_file_path();
        
        if !file_path.exists() {
            return Ok(None);
        }
        
        let json_content = tokio::fs::read_to_string(&file_path).await
            .map_err(|e| AppError::io_error(
                format!("读取配置文件 {:?} 失败", file_path),
                e.kind().to_string()
            ))?;
        
        let settings: AppSettings = serde_json::from_str(&json_content)
            .map_err(|e| AppError::json_error(format!("反序列化配置文件失败: {}", e)))?;
        Ok(Some(settings))
    }

    async fn reset_to_defaults(&self) -> AppResult<AppSettings> {
        let default_settings = AppSettings::default();
        self.save_settings(&default_settings).await?;
        log::info!("应用配置已重置为默认值");
        Ok(default_settings)
    }

    async fn validate_settings_file(&self) -> AppResult<bool> {
        let file_path = self.get_settings_file_path();
        
        if !file_path.exists() {
            return Ok(false);
        }
        
        match self.load_settings().await {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }
}

/// 应用配置服务工厂
pub struct AppSettingsServiceFactory;

impl AppSettingsServiceFactory {
    /// 创建JSON应用配置服务
    pub fn create_json_service(config: AppSettingsConfig) -> JsonAppSettingsService {
        JsonAppSettingsService::new(config)
    }
    
    /// 创建默认的JSON应用配置服务
    pub fn create_default_json_service() -> JsonAppSettingsService {
        JsonAppSettingsService::new_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    
    async fn create_test_service() -> (JsonAppSettingsService, tempfile::TempDir) {
        let temp_dir = tempdir().unwrap();
        let config = AppSettingsConfig {
            storage_root_dir: temp_dir.path().to_path_buf(),
            config_file_name: "test_app_settings.json".to_string(),
        };
        let service = JsonAppSettingsService::new(config);
        (service, temp_dir)
    }
    
    #[tokio::test]
    async fn test_app_settings_service_basic_operations() {
        let (mut service, _temp_dir) = create_test_service().await;
        
        // 初始化服务
        service.initialize().await.unwrap();
        assert_eq!(service.service_name(), "JsonAppSettingsService");
        
        // 健康检查
        service.health_check().await.unwrap();
        
        // 测试加载不存在的配置
        let loaded = service.load_settings().await.unwrap();
        assert!(loaded.is_none());
        
        // 测试保存和加载配置
        let mut settings = AppSettings::default();
        settings.theme = "dark".to_string();
        settings.plc_ip_address = Some("192.168.1.100".to_string());
        
        service.save_settings(&settings).await.unwrap();
        
        let loaded = service.load_settings().await.unwrap();
        assert!(loaded.is_some());
        let loaded_settings = loaded.unwrap();
        assert_eq!(loaded_settings.theme, "dark");
        assert_eq!(loaded_settings.plc_ip_address, Some("192.168.1.100".to_string()));
        
        // 测试配置文件验证
        assert!(service.validate_settings_file().await.unwrap());
        
        // 测试重置为默认值
        let default_settings = service.reset_to_defaults().await.unwrap();
        assert_eq!(default_settings.theme, AppSettings::default().theme);
        
        // 关闭服务
        service.shutdown().await.unwrap();
        
        // 关闭后健康检查应该失败
        assert!(service.health_check().await.is_err());
    }
    
    #[tokio::test]
    async fn test_app_settings_service_factory() {
        let service = AppSettingsServiceFactory::create_default_json_service();
        assert_eq!(service.service_name(), "JsonAppSettingsService");
        
        let custom_config = AppSettingsConfig {
            storage_root_dir: PathBuf::from("./custom_config"),
            config_file_name: "custom_settings.json".to_string(),
        };
        let custom_service = AppSettingsServiceFactory::create_json_service(custom_config);
        assert_eq!(custom_service.service_name(), "JsonAppSettingsService");
    }
} 