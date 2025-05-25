#[cfg(test)]
mod tests {
    use crate::utils::error::{AppError, AppResult};
    use crate::utils::config::{AppConfig, ConfigManager};
    use std::path::PathBuf;
    use tempfile::tempdir;

    /// 测试AppError的创建和错误代码
    #[test]
    fn test_app_error_creation() {
        let error = AppError::generic("测试错误");
        assert_eq!(error.error_code(), "GENERIC");
        assert!(error.to_string().contains("测试错误"));

        let plc_error = AppError::plc_communication_error("PLC连接失败");
        assert_eq!(plc_error.error_code(), "PLC_COMMUNICATION_ERROR");
        assert!(plc_error.to_string().contains("PLC连接失败"));

        let io_error = AppError::io_error("文件读取失败", "Unknown");
        assert_eq!(io_error.error_code(), "IO_ERROR");
        assert!(io_error.to_string().contains("文件读取失败"));
    }

    /// 测试错误转换 (From trait)
    #[test]
    fn test_error_conversion() {
        // 测试从String转换
        let string_error: AppError = "字符串错误".into();
        assert_eq!(string_error.error_code(), "GENERIC");

        // 测试从&str转换
        let str_error: AppError = "字符串错误".into();
        assert_eq!(str_error.error_code(), "GENERIC");

        // 测试serde_json错误转换
        let invalid_json = "{invalid json}";
        let json_error: Result<serde_json::Value, serde_json::Error> = 
            serde_json::from_str(invalid_json);
        match json_error {
            Err(e) => {
                let app_error: AppError = e.into();
                assert_eq!(app_error.error_code(), "JSON_ERROR");
            }
            Ok(_) => panic!("应该产生JSON错误"),
        }
    }

    /// 测试状态转换错误
    #[test]
    fn test_state_transition_error() {
        let error = AppError::state_transition_error(
            "NotTested",
            "TestCompleted",
            "跳过了必要的步骤",
        );
        assert_eq!(error.error_code(), "STATE_TRANSITION_ERROR");
        assert!(error.to_string().contains("从 NotTested 到 TestCompleted"));
        assert!(error.to_string().contains("跳过了必要的步骤"));
    }

    /// 测试应用配置的默认值
    #[test]
    fn test_app_config_defaults() {
        let config = AppConfig::default();
        
        // 检查应用设置默认值
        assert_eq!(config.app_settings.app_name, "FactoryTesting");
        assert_eq!(config.app_settings.environment, "development");
        assert!(config.app_settings.debug_mode);
        assert_eq!(config.app_settings.max_concurrent_tasks, 10);

        // 检查PLC配置默认值
        assert_eq!(config.plc_config.plc_type, "modbus");
        assert_eq!(config.plc_config.host, "127.0.0.1");
        assert_eq!(config.plc_config.port, 502);
        assert!(config.plc_config.mock_mode);

        // 检查测试配置默认值
        assert_eq!(config.test_config.analog_tolerance_percent, 1.0);
        assert!(config.test_config.auto_skip_not_applicable);
        assert_eq!(config.test_config.batch_test_size, 20);

        // 检查日志配置默认值
        assert_eq!(config.logging_config.log_level, "info");
        assert!(config.logging_config.console_output);
        assert!(config.logging_config.file_output);

        // 检查持久化配置默认值
        assert_eq!(config.persistence_config.persistence_type, "json");
        assert!(config.persistence_config.auto_backup);
        assert_eq!(config.persistence_config.max_backups, 7);
    }

    /// 测试配置序列化和反序列化
    #[test]
    fn test_config_serialization() {
        let config = AppConfig::default();
        
        // 序列化
        let json = serde_json::to_string_pretty(&config).unwrap();
        assert!(json.contains("FactoryTesting"));
        assert!(json.contains("modbus"));
        assert!(json.contains("development"));
        
        // 反序列化
        let deserialized: AppConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.app_settings.app_name, config.app_settings.app_name);
        assert_eq!(deserialized.plc_config.host, config.plc_config.host);
        assert_eq!(deserialized.test_config.analog_tolerance_percent, config.test_config.analog_tolerance_percent);
    }

    /// 测试配置管理器基本功能
    #[tokio::test]
    async fn test_config_manager_basic_operations() {
        // 创建临时目录和文件路径
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("test_config.json");
        
        let mut manager = ConfigManager::new(config_path.clone());
        
        // 测试保存和加载
        manager.get_config_mut().app_settings.app_name = "测试应用".to_string();
        manager.get_config_mut().plc_config.host = "192.168.1.100".to_string();
        
        // 保存配置到文件
        manager.save_to_file().await.unwrap();
        assert!(config_path.exists());
        
        // 创建新的管理器并加载配置
        let mut new_manager = ConfigManager::new(config_path);
        new_manager.load_from_file().await.unwrap();
        
        assert_eq!(new_manager.get_config().app_settings.app_name, "测试应用");
        assert_eq!(new_manager.get_config().plc_config.host, "192.168.1.100");
    }

    /// 测试配置验证
    #[test]
    fn test_config_validation() {
        let mut config = AppConfig::default();
        let config_path = PathBuf::from("test_config.json");
        let manager = ConfigManager::new(config_path);
        
        // 默认配置应该通过验证
        assert!(manager.validate_config().is_ok());
        
        // 测试无效的环境配置
        config.app_settings.environment = "invalid_env".to_string();
        let mut manager = ConfigManager::new(PathBuf::from("test.json"));
        *manager.get_config_mut() = config.clone();
        assert!(manager.validate_config().is_err());
        
        // 测试无效的PLC端口
        config.app_settings.environment = "development".to_string();
        config.plc_config.port = 0;
        *manager.get_config_mut() = config.clone();
        assert!(manager.validate_config().is_err());
        
        // 测试空的PLC主机地址
        config.plc_config.port = 502;
        config.plc_config.host = "".to_string();
        *manager.get_config_mut() = config.clone();
        assert!(manager.validate_config().is_err());
    }

    /// 测试环境变量覆盖
    #[test]
    fn test_env_override() {
        // 设置环境变量
        std::env::set_var("PLC_HOST", "10.0.0.1");
        std::env::set_var("PLC_PORT", "1502");
        std::env::set_var("PLC_MOCK_MODE", "false");
        std::env::set_var("APP_ENVIRONMENT", "production");
        std::env::set_var("DEBUG_MODE", "false");
        std::env::set_var("LOG_LEVEL", "error");
        
        let mut manager = ConfigManager::new(PathBuf::from("test.json"));
        manager.override_from_env();
        
        let config = manager.get_config();
        assert_eq!(config.plc_config.host, "10.0.0.1");
        assert_eq!(config.plc_config.port, 1502);
        assert!(!config.plc_config.mock_mode);
        assert_eq!(config.app_settings.environment, "production");
        assert!(!config.app_settings.debug_mode);
        assert_eq!(config.logging_config.log_level, "error");
        
        // 清理环境变量
        std::env::remove_var("PLC_HOST");
        std::env::remove_var("PLC_PORT");
        std::env::remove_var("PLC_MOCK_MODE");
        std::env::remove_var("APP_ENVIRONMENT");
        std::env::remove_var("DEBUG_MODE");
        std::env::remove_var("LOG_LEVEL");
    }

    /// 测试AppResult类型别名
    #[test]
    fn test_app_result() {
        // 测试成功情况
        let success: AppResult<String> = Ok("成功".to_string());
        assert!(success.is_ok());
        
        // 测试错误情况
        let error: AppResult<String> = Err(AppError::generic("测试错误"));
        assert!(error.is_err());
        
        match error {
            Err(e) => assert_eq!(e.error_code(), "GENERIC"),
            Ok(_) => panic!("应该是错误"),
        }
    }

    /// 测试特定业务错误类型
    #[test]
    fn test_business_specific_errors() {
        // 测试超时错误
        let timeout_error = AppError::timeout_error("PLC连接", "连接超过5秒");
        assert_eq!(timeout_error.error_code(), "TIMEOUT_ERROR");
        assert!(timeout_error.to_string().contains("PLC连接"));
        assert!(timeout_error.to_string().contains("连接超过5秒"));
        
        // 测试资源未找到错误
        let not_found_error = AppError::not_found_error("ChannelDefinition", "ID不存在");
        assert_eq!(not_found_error.error_code(), "NOT_FOUND_ERROR");
        assert!(not_found_error.to_string().contains("ChannelDefinition"));
        
        // 测试测试执行错误
        let test_error = AppError::test_execution_error("硬点测试", "信号读取失败");
        assert_eq!(test_error.error_code(), "TEST_EXECUTION_ERROR");
        assert!(test_error.to_string().contains("硬点测试"));
        assert!(test_error.to_string().contains("信号读取失败"));
    }
} 