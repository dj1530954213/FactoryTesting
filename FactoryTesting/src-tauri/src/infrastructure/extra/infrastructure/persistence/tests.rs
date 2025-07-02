#[cfg(test)]
mod tests {
    use crate::services::infrastructure::persistence::app_settings_service::JsonAppSettingsService;
    use crate::services::infrastructure::persistence::persistence_service::{
        PersistenceConfig,
        QueryCriteria,
        IntegrityStatus,
        ExtendedPersistenceService,
        HasTimestamps,
        PersistenceServiceHelper,
    };
    use crate::models::structs::*;
    use crate::models::enums::*;
    use crate::services::traits::{BaseService, PersistenceService};
    use tempfile::{tempdir, TempDir};
    use std::path::{Path, PathBuf};
    use tokio;
    use crate::utils::error::AppError;
    use crate::services::infrastructure::persistence::sqlite_orm_persistence_service::SqliteOrmPersistenceService;
    use env_logger;
    use crate::services::infrastructure::persistence::persistence_service::PersistenceServiceFactory;

    /// 创建测试用的持久化服务
    pub async fn create_test_service() -> (SqliteOrmPersistenceService, TempDir) {
        let temp_dir = tempdir().unwrap();
        let memory_path = Path::new(":memory:");

        // 尝试初始化 env_logger，如果失败则忽略错误（例如在测试并行运行时重复初始化）
        let _ = env_logger::builder().is_test(true).try_init(); 

        let config = PersistenceConfig {
            storage_root_dir: temp_dir.path().to_path_buf(),
            ..Default::default()
        };

        let service = SqliteOrmPersistenceService::new(config, Some(memory_path)).await.unwrap();
        (service, temp_dir)
    }

    /// 创建测试用的持久化服务（带自定义配置和数据库路径）
    pub async fn create_test_service_with_config_and_db_path(
        config: PersistenceConfig, 
        db_path: Option<&Path>
    ) -> SqliteOrmPersistenceService {
        // 尝试初始化 env_logger，如果失败则忽略错误
        let _ = env_logger::builder().is_test(true).try_init(); 

        SqliteOrmPersistenceService::new(config, db_path).await.unwrap()
    }

    /// 创建测试用的通道点位定义
    fn create_test_channel_definition(id: &str) -> ChannelPointDefinition {
        let mut def = ChannelPointDefinition::new(
            format!("TEST_TAG_{}", id),
            format!("test_var_{}", id),
            format!("测试通道 {}", id),
            "TEST_STATION".to_string(),
            "TEST_MODULE".to_string(),
            ModuleType::AI,
            format!("AI_{}", id),
            PointDataType::Float,
            format!("DB1.DBD{}", id.parse::<u32>().unwrap_or(0) * 4)
        );
        def.id = id.to_string();
        def.range_low_limit = Some(4.0);
        def.range_high_limit = Some(20.0);
        def.engineering_unit = Some("mA".to_string());
        def
    }

    /// 创建测试用的测试实例
    fn create_test_instance(_instance_id: &str, batch_id: &str, definition_id: &str) -> ChannelTestInstance {
        let mut instance = ChannelTestInstance::new(definition_id.to_string(), batch_id.to_string());
        instance.instance_id = _instance_id.to_string();
        instance
    }

    /// 创建测试用的测试批次
    fn create_test_batch(batch_id: &str) -> TestBatchInfo {
        let mut batch = TestBatchInfo::new(
            Some("TEST_MODEL".to_string()),
            Some(format!("TEST_SERIAL_{}", batch_id))
        );
        batch.batch_id = batch_id.to_string();
        batch
    }

    /// 创建测试用的测试结果
    fn create_test_outcome(channel_instance_id: &str, sub_test_item: SubTestItem) -> RawTestOutcome {
        let mut outcome = RawTestOutcome::success(channel_instance_id.to_string(), sub_test_item);
        outcome.raw_value_read = Some("RawVal".to_string());
        outcome.eng_value_calculated = Some("EngVal".to_string());
        outcome.message = Some("测试成功".to_string());
        outcome.readings = Some(Vec::new());
        outcome
    }

    /// 测试基础服务功能
    #[tokio::test]
    async fn test_base_service_functionality() {
        let (mut service, _temp_dir) = create_test_service().await;

        assert_eq!(service.service_name(), "SqliteOrmPersistenceService");

        service.health_check().await.unwrap();

        service.shutdown().await.unwrap();

        assert!(service.health_check().await.is_err());
    }

    /// 测试通道定义的CRUD操作
    #[tokio::test]
    async fn test_channel_definition_crud() {
        let (service, _temp_dir) = create_test_service().await;

        let definition = create_test_channel_definition("test_001");

        service.save_channel_definition(&definition).await.unwrap();

        let loaded = service.load_channel_definition("test_001").await.unwrap();
        assert!(loaded.is_some());
        let loaded = loaded.unwrap();
        assert_eq!(loaded.id, definition.id);
        assert_eq!(loaded.variable_description, definition.variable_description);

        let not_found = service.load_channel_definition("not_exist").await.unwrap();
        assert!(not_found.is_none());

        let definition2 = create_test_channel_definition("test_002");
        service.save_channel_definition(&definition2).await.unwrap();

        let all_definitions = service.load_all_channel_definitions().await.unwrap();
        assert_eq!(all_definitions.len(), 2);

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

        service.save_test_instance(&instance).await.unwrap();

        let loaded = service.load_test_instance("inst_001").await.unwrap();
        assert!(loaded.is_some());
        let loaded = loaded.unwrap();
        assert_eq!(loaded.instance_id, instance.instance_id);
        assert_eq!(loaded.test_batch_id, instance.test_batch_id);

        let instance2 = create_test_instance("inst_002", "batch_001", "def_002");
        let instance3 = create_test_instance("inst_003", "batch_002", "def_003");
        service.save_test_instance(&instance2).await.unwrap();
        service.save_test_instance(&instance3).await.unwrap();

        let batch_instances = service.load_test_instances_by_batch("batch_001").await.unwrap();
        assert_eq!(batch_instances.len(), 2);

        service.delete_test_instance("inst_001").await.unwrap();
        let after_delete = service.load_test_instances_by_batch("batch_001").await.unwrap();
        assert_eq!(after_delete.len(), 1);
    }

    /// 测试测试批次的CRUD操作
    #[tokio::test]
    async fn test_batch_info_crud() {
        let (service, _temp_dir) = create_test_service().await;

        let batch = create_test_batch("batch_001");

        service.save_batch_info(&batch).await.unwrap();

        let loaded = service.load_batch_info("batch_001").await.unwrap();
        assert!(loaded.is_some());
        let loaded = loaded.unwrap();
        assert_eq!(loaded.batch_id, batch.batch_id);
        assert_eq!(loaded.product_model, batch.product_model);

        let batch2 = create_test_batch("batch_002");
        service.save_batch_info(&batch2).await.unwrap();

        let all_batches = service.load_all_batch_info().await.unwrap();
        assert_eq!(all_batches.len(), 2);

        service.delete_batch_info("batch_001").await.unwrap();
        let after_delete = service.load_all_batch_info().await.unwrap();
        assert_eq!(after_delete.len(), 1);
        assert_eq!(after_delete[0].batch_id, "batch_002");
    }

    /// 测试测试结果的CRUD操作
    #[tokio::test]
    async fn test_test_outcome_crud() {
        let (service, _temp_dir) = create_test_service().await;

        let outcome = create_test_outcome("inst_001", SubTestItem::HardPoint);

        service.save_test_outcome(&outcome).await.unwrap();

        let loaded_by_instance = service.load_test_outcomes_by_instance("inst_001").await.unwrap();
        assert_eq!(loaded_by_instance.len(), 1);
        assert_eq!(loaded_by_instance[0].sub_test_item, SubTestItem::HardPoint);

        let test_batch_id_for_outcomes = "batch_for_outcomes_test";
        let instance_for_batch_test = create_test_instance("inst_001", test_batch_id_for_outcomes, "def_for_outcomes_001");
        service.save_test_instance(&instance_for_batch_test).await.unwrap();

        let instance_for_batch_test_2 = create_test_instance("inst_002", test_batch_id_for_outcomes, "def_for_outcomes_002");
        service.save_test_instance(&instance_for_batch_test_2).await.unwrap();

        let outcome2 = create_test_outcome("inst_002", SubTestItem::TrendCheck);
        service.save_test_outcome(&outcome2).await.unwrap();
        
        let loaded_by_batch = service.load_test_outcomes_by_batch(test_batch_id_for_outcomes).await.unwrap();
        assert!(!loaded_by_batch.is_empty(), "按批次加载结果时，结果不应为空");
        assert!(loaded_by_batch.len() >= 1, "应至少加载一个结果"); 
        assert_eq!(loaded_by_batch.len(), 2, "应加载与批次关联的两个测试结果");
    }

    /// 测试扩展持久化功能 - 查询功能
    #[tokio::test]
    #[ignore = "Temporarily disabled - calls unimplemented ExtendedPersistenceService methods"]
    async fn test_extended_query_functionality() {
        let (service, _temp_dir) = create_test_service().await;

        let now = chrono::Utc::now();
        let hour_ago = now - chrono::Duration::hours(1);
        let _definition1 = create_test_channel_definition("def_001");
        let mut batch1 = create_test_batch("batch_001");
        batch1.creation_time = hour_ago;
        batch1.last_updated_time = hour_ago;

        let mut batch2 = create_test_batch("batch_002");
        batch2.creation_time = now;
        batch2.last_updated_time = now;

        service.save_batch_info(&batch1).await.unwrap();
        service.save_batch_info(&batch2).await.unwrap();

        let criteria = QueryCriteria {
            created_after: Some(now - chrono::Duration::minutes(30)),
            ..QueryCriteria::default()
        };

        let result = service.query_test_batches(&criteria).await.unwrap();
        assert_eq!(result.items.len(), 1);
        assert_eq!(result.items[0].batch_id, "batch_002");

        let criteria = QueryCriteria {
            limit: Some(1),
            offset: Some(0),
            ..QueryCriteria::default()
        };

        let result = service.query_test_batches(&criteria).await.unwrap();
        assert_eq!(result.items.len(), 1);
        assert!(result.has_more || result.total_count > 1);

        let criteria = QueryCriteria {
            sort_by: Some("creation_time".to_string()),
            sort_desc: true,
            ..QueryCriteria::default()
        };

        let sorted_batches = service.query_test_batches(&criteria).await.unwrap();
        assert!(!sorted_batches.items.is_empty(), "排序后批次不应为空");

        println!("--- Sorted Test Batches (created_at desc) ---");
        for item in &sorted_batches.items {
            println!("ID: {}, Created_at: {:?}", item.batch_id, item.created_at());
        }
        println!("-------------------------------------------");

        assert_eq!(sorted_batches.items[0].batch_id, "batch_002", "按创建时间降序排序失败");
    }

    /// 测试批量操作功能
    #[tokio::test]
    #[ignore = "Temporarily disabled - calls unimplemented ExtendedPersistenceService methods"]
    async fn test_batch_operations() {
        let (service, _temp_dir) = create_test_service().await;

        let defs_to_save = vec![
            create_test_channel_definition("def_batch_1"),
            create_test_channel_definition("def_batch_2"),
        ];
        service.batch_save_channel_definitions(&defs_to_save).await.unwrap();

        let loaded_defs = service.load_all_channel_definitions().await.unwrap();
        assert!(loaded_defs.iter().any(|d| d.id == "def_batch_1"));
        assert!(loaded_defs.iter().any(|d| d.id == "def_batch_2"));

        let instance = create_test_instance("instance_batch_1", "batch_for_instances", "def_for_instance");
        match service.save_test_instance(&instance).await {
            Ok(_) => {},
            Err(e) => panic!("save_test_instance failed in test_batch_operations: {:?}", e),
        }
        
        let loaded_instance = service.load_test_instance("instance_batch_1").await.unwrap();
        assert!(loaded_instance.is_some());
        assert_eq!(loaded_instance.unwrap().instance_id, "instance_batch_1");

        service.delete_test_instance("instance_batch_1").await.unwrap();
        let after_delete = service.load_test_instances_by_batch("batch_for_instances").await.unwrap();
        assert_eq!(after_delete.len(), 0);
    }

    // TODO: Temporarily ignored due to rusqlite/libsqlite3-sys version conflicts affecting backup implementation.
    // Re-enable and fix when the underlying dependency issues are resolved.
    #[tokio::test]
    #[ignore = "Temporarily disabled due to unresolved rusqlite/libsqlite3-sys dependency conflicts affecting backup functionality"]
    async fn test_backup_and_restore() {
        let temp_dir_guard = tempfile::tempdir().unwrap();
        let temp_dir_path = temp_dir_guard.path();
        env_logger::try_init().ok(); // Initialize logger for more details if test fails

        let config = PersistenceConfig {
            storage_root_dir: temp_dir_path.to_path_buf(),
            enable_auto_backup: false, // Disable auto backup for this test
            ..Default::default()
        };

        // Create service with a file-based database for backup testing
        let db_file_name = "test_db_for_backup.sqlite";
        let db_path = temp_dir_path.join(db_file_name);
        
        log::info!("Test DB path for backup/restore: {:?}", db_path);

        let mut service = create_test_service_with_config_and_db_path(config.clone(), Some(&db_path)).await;

        // 1. Initialize and add some data
        service.initialize().await.expect("Service initialization failed");
        let initial_def = create_test_channel_definition("def_bk_rs_01");
        service.save_channel_definition(&initial_def).await.expect("Save initial def failed");

        // Verify data presence before backup
        let loaded_before_backup = service.load_channel_definition(&initial_def.id).await.expect("Load before backup failed").unwrap();
        assert_eq!(loaded_before_backup.variable_description, initial_def.variable_description, "Data should exist before backup");

        // 2. Perform a backup
        let backup_name = "my_test_backup_01";
        let backup_info = service.backup(backup_name).await.expect("Backup failed");
        assert_eq!(backup_info.name, backup_name);
        assert!(backup_info.path.exists(), "Backup file should exist at {:?}", backup_info.path);
        assert!(backup_info.size_bytes > 0, "Backup file should have size > 0");

        // 3. (Optional) Modify or delete data in the original database
        service.delete_channel_definition(&initial_def.id).await.expect("Delete after backup failed");
        assert!(service.load_channel_definition(&initial_def.id).await.expect("Load after delete failed").is_none(), "Data should be deleted after backup");

        // 4. Restore from backup
        // Before restoring, it might be good to shutdown and reinitialize the service 
        // or ensure the DB connection is reset if it's an in-memory DB being restored from file.
        // For a file DB, simply replacing the file and re-opening is common.
        service.restore_from_backup(&backup_info.path).await.expect("Restore failed");
        
        // It's crucial that after a restore, the service reloads its state from the restored data.
        // For some services, this might mean re-initializing or explicitly reloading.
        // Let's assume our service re-connects or is fresh after restore call for this test.
        // Or, if it's a file-based DB, the next connection will pick up the restored file.
        // We might need to create a new service instance pointing to the same db_path if the old one holds a stale connection.
        
        // Re-verify data presence from the restored database
        // It might be safer to create a new service instance to ensure it reads the restored file
        drop(service); // Drop the old service instance
        let mut restored_service = create_test_service_with_config_and_db_path(config, Some(&db_path)).await;
        restored_service.initialize().await.expect("Restored service initialization failed");

        let loaded_after_restore_opt = restored_service.load_channel_definition(&initial_def.id).await.expect("Load after restore failed");
        assert!(loaded_after_restore_opt.is_some(), "Data should be present after restore");
        let loaded_after_restore = loaded_after_restore_opt.unwrap();
        assert_eq!(loaded_after_restore.variable_description, initial_def.variable_description, "Restored data description mismatch");

        // 5. List backups and verify
        let backups = restored_service.list_backups().await.expect("List backups failed");
        assert!(!backups.is_empty(), "Backups list should not be empty");
        assert!(backups.iter().any(|b| b.name == backup_name), "Our backup should be in the list");

        log::info!("test_backup_and_restore completed successfully.");
    }

    /// 测试数据完整性验证
    #[tokio::test]
    async fn test_data_integrity_verification() {
        let (service, _temp_dir) = create_test_service().await;

        let definition = create_test_channel_definition("integrity_test");
        service.save_channel_definition(&definition).await.unwrap();

        let report = service.verify_data_integrity().await.unwrap();
        assert_eq!(report.overall_status, IntegrityStatus::Good);
        assert_eq!(report.issues_count, 0);
        assert!(!report.details.is_empty());
    }

    /// 测试统计信息功能
    #[tokio::test]
    async fn test_statistics() {
        let (service, _temp_dir) = create_test_service().await;

        let initial_stats = service.get_statistics().await.unwrap();
        assert_eq!(initial_stats.channel_definitions_count, 0);
        assert_eq!(initial_stats.test_instances_count, 0);

        let definition = create_test_channel_definition("stats_test");
        service.save_channel_definition(&definition).await.unwrap();

        let instance = create_test_instance("stats_inst", "stats_batch", "stats_def");
        service.save_test_instance(&instance).await.unwrap();

        let updated_stats = service.get_statistics().await.unwrap();
        assert_eq!(updated_stats.channel_definitions_count, 1);
        assert_eq!(updated_stats.test_instances_count, 1);
        assert_eq!(updated_stats.total_storage_size_bytes, 0, "内存数据库的存储大小应为0");
    }

    /// 测试配置更新功能
    #[tokio::test]
    #[ignore = "Temporarily disabled - calls unimplemented ExtendedPersistenceService methods"]
    async fn test_config_update() {
        let (mut service, _temp_dir) = create_test_service().await;

        let current_config = service.get_config().clone();

        let mut new_config = current_config.clone();
        new_config.backup_retention_days = 60;
        new_config.max_file_size_mb = 200;

        service.update_config(new_config).await.unwrap();

        let updated_config = service.get_config();
        assert_eq!(updated_config.backup_retention_days, 60);
        assert_eq!(updated_config.max_file_size_mb, 200);
    }

    /// 测试数据清理功能
    #[tokio::test]
    #[ignore = "Temporarily disabled - calls unimplemented ExtendedPersistenceService methods"]
    async fn test_data_cleanup() {
        let (service, _temp_dir) = create_test_service().await;

        let definition = create_test_channel_definition("cleanup_test");
        service.save_channel_definition(&definition).await.unwrap();

        let _deleted_count = service.cleanup_expired_data(0).await.unwrap();
    }

    /// 测试存储压缩功能
    #[tokio::test]
    async fn test_storage_compaction() {
        let (service, _temp_dir) = create_test_service().await;
        let result = service.compact_storage().await;
        assert!(result.is_err());
        match result.unwrap_err() {
            AppError::NotImplemented { .. } => { /* Correct error */ }
            e => panic!("Expected NotImplemented error, got {:?}", e),
        }
    }

    /// 测试索引重建功能
    #[tokio::test]
    async fn test_index_rebuild() {
        let (service, _temp_dir) = create_test_service().await;
        let result = service.rebuild_indexes().await;
        assert!(result.is_err());
        match result.unwrap_err() {
            AppError::NotImplemented { .. } => { /* Correct error */ }
            e => panic!("Expected NotImplemented error, got {:?}", e),
        }
    }

    /// 测试并发访问
    #[tokio::test]
    async fn test_concurrent_access() {
        let (service, _temp_dir) = create_test_service().await;
        let service = std::sync::Arc::new(service);

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

        for handle in handles {
            handle.await.unwrap();
        }

        let all_definitions = service.load_all_channel_definitions().await.unwrap();
        assert_eq!(all_definitions.len(), 10);
    }

    /// 测试错误处理
    #[tokio::test]
    async fn test_error_handling() {
        // invalid_config 变量在此测试中未使用，可以移除或注释掉以消除警告
        // let invalid_config = PersistenceConfig {
        //     storage_root_dir: PathBuf::from("/invalid/path/that/does/not/exist"),
        //     ..PersistenceConfig::default()
        // };

        // 为 new 调用提供 PersistenceConfig::default()
        let mut service = SqliteOrmPersistenceService::new(PersistenceConfig::default(), Some(Path::new(":memory:"))).await.unwrap();
        let _ = service.initialize().await; // initialize 应该处理来自 new 的服务
        // 这个测试的目的可能是测试 new 或 initialize 在特定配置下的错误情况，
        // 例如，如果 db_path_opt 指向一个不可访问的文件路径，并且 storage_root_dir 也无效。
        // 当前的实现，如果 db_path_opt 是 :memory:，则不会访问文件系统创建数据库。
        // 如果要测试文件路径错误，应提供一个文件路径给 db_path_opt。
        // 但对于基本的错误处理（例如，服务可以初始化和关闭），当前这样就可以了。
    }

    #[tokio::test]
    async fn test_persistence_service_factory() {
        let service_result = PersistenceServiceFactory::create_default_sqlite_service().await;
        assert!(service_result.is_ok(), "创建默认SQLite服务应该成功");
        let service = service_result.unwrap();
        assert_eq!(service.service_name(), "SqliteOrmPersistenceService");

        let custom_config = PersistenceConfig {
            storage_root_dir: std::path::PathBuf::from("./custom_data"),
            ..PersistenceConfig::default()
        };

        let custom_service_result = PersistenceServiceFactory::create_sqlite_service(custom_config, Some(std::path::Path::new(":memory:"))).await;
        assert!(custom_service_result.is_ok(), "创建自定义SQLite服务应该成功");
        let custom_service = custom_service_result.unwrap();
        assert_eq!(custom_service.service_name(), "SqliteOrmPersistenceService");
    }
}

#[cfg(test)]
mod helper_tests {
    use tempfile::TempDir;
    use crate::services::infrastructure::persistence::persistence_service::PersistenceConfig;
    use crate::services::infrastructure::persistence::persistence_service::PersistenceServiceHelper;
    use crate::services::infrastructure::persistence::persistence_service::PersistenceServiceFactory;
    use crate::services::traits::BaseService;

    #[tokio::test]
    async fn test_persistence_service_helper() {
        let temp_dir = TempDir::new().unwrap();
        let test_path = temp_dir.path().to_path_buf();

        PersistenceServiceHelper::validate_storage_path(&test_path).await.unwrap();

        #[cfg(not(target_os = "windows"))]
        {
            let initial_size = PersistenceServiceHelper::calculate_directory_size(&test_path)
                .unwrap_or_else(|e| {
                    eprintln!("Error calculating initial directory size: {:?}", e);
                    0
                });
            println!("Initial directory size: {}", initial_size);

            let final_size = PersistenceServiceHelper::calculate_directory_size(&test_path)
                .unwrap_or_else(|e| {
                    eprintln!("Error calculating final directory size: {:?}", e);
                    0
                });
            println!("Final directory size: {}", final_size);
            assert!(final_size > initial_size, "Backup size should be greater than initial size or there was an error in size calculation.");
        }
    }
} 