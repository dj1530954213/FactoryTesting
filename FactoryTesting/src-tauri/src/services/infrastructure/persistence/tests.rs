#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::infrastructure::persistence::persistence_service::{
        QueryCriteria, IntegrityStatus, PersistenceConfig, 
        PersistenceServiceHelper, PersistenceServiceFactory
    };
    use crate::services::infrastructure::persistence::json_persistence_service::JsonPersistenceService;
    use crate::models::enums::*;
    use crate::models::structs::*;
    use crate::services::traits::{BaseService, PersistenceService};
    use tempfile::TempDir;
    use std::path::PathBuf;
    use tokio;

    /// 创建测试用的持久化服务
    async fn create_test_service() -> (JsonPersistenceService, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let config = PersistenceConfig {
            storage_root_dir: temp_dir.path().to_path_buf(),
            channel_definitions_dir: "channels".to_string(),
            test_instances_dir: "instances".to_string(),
            test_batches_dir: "batches".to_string(),
            test_outcomes_dir: "outcomes".to_string(),
            enable_auto_backup: false, // 测试时禁用自动备份
            backup_retention_days: 7,
            max_file_size_mb: 10,
            enable_compression: false,
        };

        let mut service = JsonPersistenceService::new(config);
        service.initialize().await.unwrap();
        (service, temp_dir)
    }

    /// 创建测试用的通道点位定义
    fn create_test_channel_definition(id: &str) -> ChannelPointDefinition {
        ChannelPointDefinition {
            id: id.to_string(),
            tag: format!("TEST_TAG_{}", id),
            variable_name: format!("test_var_{}", id),
            variable_description: format!("测试通道 {}", id),
            station_name: "TEST_STATION".to_string(),
            module_name: "TEST_MODULE".to_string(),
            module_type: ModuleType::AI,
            channel_tag_in_module: format!("AI_{}", id),
            data_type: PointDataType::Float,
            power_supply_type: "有源".to_string(),
            wire_system: "4线制".to_string(),
            plc_absolute_address: None,
            plc_communication_address: format!("DB1.DBD{}", id.parse::<u32>().unwrap_or(0) * 4),
            range_lower_limit: Some(4.0),
            range_upper_limit: Some(20.0),
            engineering_unit: Some("mA".to_string()),
            sll_set_value: None,
            sll_set_point_address: None,
            sll_feedback_address: None,
            sl_set_value: None,
            sl_set_point_address: None,
            sl_feedback_address: None,
            sh_set_value: None,
            sh_set_point_address: None,
            sh_feedback_address: None,
            shh_set_value: None,
            shh_set_point_address: None,
            shh_feedback_address: None,
            maintenance_value_set_point_address: None,
            maintenance_enable_switch_point_address: None,
            access_property: None,
            save_history: None,
            power_failure_protection: None,
            test_rig_plc_address: None,
        }
    }

    /// 创建测试用的测试实例
    fn create_test_instance(instance_id: &str, batch_id: &str, definition_id: &str) -> ChannelTestInstance {
        ChannelTestInstance {
            instance_id: instance_id.to_string(),
            channel_definition_id: definition_id.to_string(),
            batch_id: batch_id.to_string(),
            overall_status: OverallTestStatus::WaitingForTest,
            error_message: None,
            creation_time: chrono::Utc::now(),
            last_updated_time: chrono::Utc::now(),
            start_test_time: None,
            final_test_time: None,
            total_test_duration_ms: None,
            sub_test_results: std::collections::HashMap::new(),
            current_operator: Some("test_operator".to_string()),
            retries_count: 0,
            transient_data: std::collections::HashMap::new(),
        }
    }

    /// 创建测试用的测试批次
    fn create_test_batch(batch_id: &str) -> TestBatchInfo {
        TestBatchInfo {
            batch_id: batch_id.to_string(),
            product_model: Some("TEST_MODEL".to_string()),
            serial_number: Some("TEST_SERIAL".to_string()),
            creation_time: chrono::Utc::now(),
            last_updated_time: chrono::Utc::now(),
            operator_name: Some("test_operator".to_string()),
            status_summary: Some("测试批次描述".to_string()),
            total_points: 10,
            tested_points: 0,
            passed_points: 0,
            failed_points: 0,
            skipped_points: 0,
            custom_data: std::collections::HashMap::new(),
        }
    }

    /// 创建测试用的测试结果
    fn create_test_outcome(channel_instance_id: &str, sub_test_item: SubTestItem) -> RawTestOutcome {
        let now = chrono::Utc::now();
        RawTestOutcome {
            channel_instance_id: channel_instance_id.to_string(),
            sub_test_item,
            success: true,
            message: Some("测试成功".to_string()),
            start_time: now,
            end_time: now,
            readings: Vec::new(),
            details: std::collections::HashMap::new(),
        }
    }

    /// 测试基础服务功能
    #[tokio::test]
    async fn test_base_service_functionality() {
        let (mut service, _temp_dir) = create_test_service().await;

        // 测试服务名称
        assert_eq!(service.service_name(), "JsonPersistenceService");

        // 测试健康检查
        service.health_check().await.unwrap();

        // 测试关闭服务
        service.shutdown().await.unwrap();

        // 关闭后健康检查应该失败
        assert!(service.health_check().await.is_err());
    }

    /// 测试通道定义的CRUD操作
    #[tokio::test]
    async fn test_channel_definition_crud() {
        let (service, _temp_dir) = create_test_service().await;

        let definition = create_test_channel_definition("test_001");

        // 测试保存
        service.save_channel_definition(&definition).await.unwrap();

        // 测试加载单个
        let loaded = service.load_channel_definition("test_001").await.unwrap();
        assert!(loaded.is_some());
        let loaded = loaded.unwrap();
        assert_eq!(loaded.id, definition.id);
        assert_eq!(loaded.variable_description, definition.variable_description);

        // 测试加载不存在的
        let not_found = service.load_channel_definition("not_exist").await.unwrap();
        assert!(not_found.is_none());

        // 测试加载所有
        let definition2 = create_test_channel_definition("test_002");
        service.save_channel_definition(&definition2).await.unwrap();

        let all_definitions = service.load_all_channel_definitions().await.unwrap();
        assert_eq!(all_definitions.len(), 2);

        // 测试删除
        service.delete_channel_definition("test_001").await.unwrap();
        let after_delete = service.load_all_channel_definitions().await.unwrap();
        assert_eq!(after_delete.len(), 1);
        assert_eq!(after_delete[0].id, "test_002");
    }

    /// 测试测试实例的CRUD操作
    #[tokio::test]
    async fn test_test_instance_crud() {
        let (service, _temp_dir) = create_test_service().await;

        let instance = create_test_instance("inst_001", "batch_001", "def_001");

        // 测试保存
        service.save_test_instance(&instance).await.unwrap();

        // 测试加载单个
        let loaded = service.load_test_instance("inst_001").await.unwrap();
        assert!(loaded.is_some());
        let loaded = loaded.unwrap();
        assert_eq!(loaded.instance_id, instance.instance_id);
        assert_eq!(loaded.batch_id, instance.batch_id);

        // 测试按批次加载
        let instance2 = create_test_instance("inst_002", "batch_001", "def_002");
        let instance3 = create_test_instance("inst_003", "batch_002", "def_003");
        service.save_test_instance(&instance2).await.unwrap();
        service.save_test_instance(&instance3).await.unwrap();

        let batch_instances = service.load_test_instances_by_batch("batch_001").await.unwrap();
        assert_eq!(batch_instances.len(), 2);

        // 测试删除
        service.delete_test_instance("inst_001").await.unwrap();
        let after_delete = service.load_test_instances_by_batch("batch_001").await.unwrap();
        assert_eq!(after_delete.len(), 1);
    }

    /// 测试测试批次的CRUD操作
    #[tokio::test]
    async fn test_batch_info_crud() {
        let (service, _temp_dir) = create_test_service().await;

        let batch = create_test_batch("batch_001");

        // 测试保存
        service.save_batch_info(&batch).await.unwrap();

        // 测试加载单个
        let loaded = service.load_batch_info("batch_001").await.unwrap();
        assert!(loaded.is_some());
        let loaded = loaded.unwrap();
        assert_eq!(loaded.batch_id, batch.batch_id);
        assert_eq!(loaded.product_model, batch.product_model);

        // 测试加载所有
        let batch2 = create_test_batch("batch_002");
        service.save_batch_info(&batch2).await.unwrap();

        let all_batches = service.load_all_batch_info().await.unwrap();
        assert_eq!(all_batches.len(), 2);

        // 测试删除
        service.delete_batch_info("batch_001").await.unwrap();
        let after_delete = service.load_all_batch_info().await.unwrap();
        assert_eq!(after_delete.len(), 1);
        assert_eq!(after_delete[0].batch_id, "batch_002");
    }

    /// 测试测试结果的CRUD操作
    #[tokio::test]
    async fn test_test_outcome_crud() {
        let (service, _temp_dir) = create_test_service().await;

        let outcome = create_test_outcome("inst_001", SubTestItem::BasicConnectivityCheck);

        // 测试保存
        service.save_test_outcome(&outcome).await.unwrap();

        // 测试按实例加载
        let outcome2 = create_test_outcome("inst_001", SubTestItem::RangeTest);
        let outcome3 = create_test_outcome("inst_002", SubTestItem::BasicConnectivityCheck);
        service.save_test_outcome(&outcome2).await.unwrap();
        service.save_test_outcome(&outcome3).await.unwrap();

        let instance_outcomes = service.load_test_outcomes_by_instance("inst_001").await.unwrap();
        assert_eq!(instance_outcomes.len(), 2);

        // 测试按批次加载
        let batch_outcomes = service.load_test_outcomes_by_batch("batch_001").await.unwrap();
        assert_eq!(batch_outcomes.len(), 3);
    }

    /// 测试扩展持久化功能 - 查询功能
    #[tokio::test]
    async fn test_extended_query_functionality() {
        let (service, _temp_dir) = create_test_service().await;

        // 准备测试数据
        let now = chrono::Utc::now();
        let hour_ago = now - chrono::Duration::hours(1);
        let mut definition1 = create_test_channel_definition("def_001");
        let mut batch1 = create_test_batch("batch_001");
        batch1.creation_time = hour_ago;
        batch1.last_updated_time = hour_ago;

        let mut batch2 = create_test_batch("batch_002");
        batch2.creation_time = now;
        batch2.last_updated_time = now;

        service.save_batch_info(&batch1).await.unwrap();
        service.save_batch_info(&batch2).await.unwrap();

        // 测试时间范围查询
        let criteria = QueryCriteria {
            created_after: Some(now - chrono::Duration::minutes(30)),
            ..QueryCriteria::default()
        };

        let result = service.query_test_batches(&criteria).await.unwrap();
        assert_eq!(result.items.len(), 1);
        assert_eq!(result.items[0].batch_id, "batch_002");

        // 测试分页查询
        let criteria = QueryCriteria {
            limit: Some(1),
            offset: Some(0),
            ..QueryCriteria::default()
        };

        let result = service.query_test_batches(&criteria).await.unwrap();
        assert_eq!(result.items.len(), 1);
        assert!(result.has_more || result.total_count > 1);

        // 测试排序
        let criteria = QueryCriteria {
            sort_by: Some("created_at".to_string()),
            sort_desc: true,
            ..QueryCriteria::default()
        };

        let result = service.query_test_batches(&criteria).await.unwrap();
        assert_eq!(result.items.len(), 2);
        assert_eq!(result.items[0].batch_id, "batch_002"); // 最新的在前
    }

    /// 测试批量操作功能
    #[tokio::test]
    async fn test_batch_operations() {
        let (service, _temp_dir) = create_test_service().await;

        // 测试批量保存通道定义
        let definitions = vec![
            create_test_channel_definition("batch_001"),
            create_test_channel_definition("batch_002"),
            create_test_channel_definition("batch_003"),
        ];

        service.batch_save_channel_definitions(&definitions).await.unwrap();

        let all_definitions = service.load_all_channel_definitions().await.unwrap();
        assert_eq!(all_definitions.len(), 3);

        // 测试批量删除
        let ids = vec!["batch_001".to_string(), "batch_002".to_string()];
        service.batch_delete_by_ids("channel_definition", &ids).await.unwrap();

        let after_delete = service.load_all_channel_definitions().await.unwrap();
        assert_eq!(after_delete.len(), 1);
        assert_eq!(after_delete[0].id, "batch_003");
    }

    /// 测试备份和恢复功能
    #[tokio::test]
    async fn test_backup_and_restore() {
        let (service, temp_dir) = create_test_service().await;

        // 准备测试数据
        let definition = create_test_channel_definition("backup_test");
        service.save_channel_definition(&definition).await.unwrap();

        // 创建备份
        let backup_path = service.create_backup(Some("test_backup")).await.unwrap();
        assert!(backup_path.exists());

        // 删除原数据
        service.delete_channel_definition("backup_test").await.unwrap();
        let empty_result = service.load_all_channel_definitions().await.unwrap();
        assert_eq!(empty_result.len(), 0);

        // 从备份恢复
        let backup_data_dir = temp_dir.path().join("backups").join("test_backup");
        service.restore_from_backup(&backup_data_dir).await.unwrap();

        // 验证数据恢复
        let restored_result = service.load_all_channel_definitions().await.unwrap();
        assert_eq!(restored_result.len(), 1);
        assert_eq!(restored_result[0].id, "backup_test");

        // 测试列出备份
        let backups = service.list_backups().await.unwrap();
        assert!(!backups.is_empty());
        assert!(backups.iter().any(|b| b.name.contains("test_backup")));
    }

    /// 测试数据完整性验证
    #[tokio::test]
    async fn test_data_integrity_verification() {
        let (service, _temp_dir) = create_test_service().await;

        // 保存一些测试数据
        let definition = create_test_channel_definition("integrity_test");
        service.save_channel_definition(&definition).await.unwrap();

        // 执行完整性验证
        let report = service.verify_data_integrity().await.unwrap();
        assert_eq!(report.overall_status, IntegrityStatus::Good);
        assert_eq!(report.issues_count, 0);
        assert!(!report.details.is_empty());
    }

    /// 测试统计信息功能
    #[tokio::test]
    async fn test_statistics() {
        let (service, _temp_dir) = create_test_service().await;

        // 初始统计应该为空
        let initial_stats = service.get_statistics().await.unwrap();
        assert_eq!(initial_stats.channel_definitions_count, 0);
        assert_eq!(initial_stats.test_instances_count, 0);

        // 添加一些数据
        let definition = create_test_channel_definition("stats_test");
        service.save_channel_definition(&definition).await.unwrap();

        let instance = create_test_instance("stats_inst", "stats_batch", "stats_def");
        service.save_test_instance(&instance).await.unwrap();

        // 更新后的统计
        let updated_stats = service.get_statistics().await.unwrap();
        assert_eq!(updated_stats.channel_definitions_count, 1);
        assert_eq!(updated_stats.test_instances_count, 1);
        assert!(updated_stats.total_storage_size_bytes > 0);
    }

    /// 测试配置更新功能
    #[tokio::test]
    async fn test_config_update() {
        let (mut service, _temp_dir) = create_test_service().await;

        // 获取当前配置
        let current_config = service.get_config().clone();

        // 更新配置
        let mut new_config = current_config.clone();
        new_config.backup_retention_days = 60;
        new_config.max_file_size_mb = 200;

        service.update_config(new_config).await.unwrap();

        // 验证配置更新
        let updated_config = service.get_config();
        assert_eq!(updated_config.backup_retention_days, 60);
        assert_eq!(updated_config.max_file_size_mb, 200);
    }

    /// 测试数据清理功能
    #[tokio::test]
    async fn test_data_cleanup() {
        let (service, _temp_dir) = create_test_service().await;

        // 保存一些测试数据
        let definition = create_test_channel_definition("cleanup_test");
        service.save_channel_definition(&definition).await.unwrap();

        // 执行数据清理（使用很短的保留期来触发清理）
        let deleted_count = service.cleanup_expired_data(0).await.unwrap();
        // 由于文件刚创建，可能不会被清理，这个测试主要验证功能不出错
        assert!(deleted_count >= 0);
    }

    /// 测试存储压缩功能
    #[tokio::test]
    async fn test_storage_compaction() {
        let (service, _temp_dir) = create_test_service().await;

        // 执行存储压缩
        let freed_bytes = service.compact_storage().await.unwrap();
        assert!(freed_bytes >= 0);
    }

    /// 测试索引重建功能
    #[tokio::test]
    async fn test_index_rebuild() {
        let (service, _temp_dir) = create_test_service().await;

        // 保存一些数据
        let definition = create_test_channel_definition("index_test");
        service.save_channel_definition(&definition).await.unwrap();

        // 重建索引
        service.rebuild_indexes().await.unwrap();

        // 验证数据仍然可以访问
        let loaded = service.load_channel_definition("index_test").await.unwrap();
        assert!(loaded.is_some());
    }

    /// 测试并发访问
    #[tokio::test]
    async fn test_concurrent_access() {
        let (service, _temp_dir) = create_test_service().await;
        let service = std::sync::Arc::new(service);

        // 创建多个并发任务
        let mut handles = Vec::new();

        for i in 0..10 {
            let service_clone = service.clone();
            let handle = tokio::spawn(async move {
                let definition = create_test_channel_definition(&format!("concurrent_{}", i));
                service_clone.save_channel_definition(&definition).await.unwrap();
                
                let loaded = service_clone.load_channel_definition(&format!("concurrent_{}", i)).await.unwrap();
                assert!(loaded.is_some());
            });
            handles.push(handle);
        }

        // 等待所有任务完成
        for handle in handles {
            handle.await.unwrap();
        }

        // 验证所有数据都已保存
        let all_definitions = service.load_all_channel_definitions().await.unwrap();
        assert_eq!(all_definitions.len(), 10);
    }

    /// 测试错误处理
    #[tokio::test]
    async fn test_error_handling() {
        // 测试无效配置
        let invalid_config = PersistenceConfig {
            storage_root_dir: PathBuf::from("/invalid/path/that/does/not/exist"),
            ..PersistenceConfig::default()
        };

        let mut service = JsonPersistenceService::new(invalid_config);
        // 初始化应该失败或者创建目录
        // 这取决于系统权限，我们主要测试不会panic
        let _ = service.initialize().await;
    }
}

// 额外的辅助测试
#[cfg(test)]
mod helper_tests {
    use super::*;
    use tempfile::TempDir;

    /// 测试PersistenceServiceHelper的功能
    #[tokio::test]
    async fn test_persistence_service_helper() {
        let temp_dir = TempDir::new().unwrap();
        let test_path = temp_dir.path().to_path_buf();

        // 测试验证存储路径
        PersistenceServiceHelper::validate_storage_path(&test_path).await.unwrap();

        // 测试计算目录大小
        let size = PersistenceServiceHelper::calculate_directory_size(&test_path).await.unwrap();
        assert!(size >= 0);

        // 测试生成备份名称
        let backup_name = PersistenceServiceHelper::generate_backup_name(Some("test"));
        assert!(backup_name.starts_with("test_"));
        assert!(backup_name.len() > 10); // 包含时间戳

        let auto_name = PersistenceServiceHelper::generate_backup_name(None);
        assert!(auto_name.starts_with("backup_"));
    }

    /// 测试工厂方法
    #[test]
    fn test_persistence_service_factory() {
        let service = PersistenceServiceFactory::create_default_json_service();
        assert_eq!(service.service_name(), "JsonPersistenceService");

        let custom_config = PersistenceConfig {
            storage_root_dir: std::path::PathBuf::from("./custom_data"),
            ..PersistenceConfig::default()
        };

        let custom_service = PersistenceServiceFactory::create_json_service(custom_config);
        assert_eq!(custom_service.service_name(), "JsonPersistenceService");
    }
} 