use sea_orm::{Database, DatabaseConnection, Statement, ConnectionTrait};
use crate::error::AppError;

/// 数据库迁移管理器
pub struct DatabaseMigration;

impl DatabaseMigration {
    /// 执行所有必要的数据库迁移
    pub async fn migrate(db: &DatabaseConnection) -> Result<(), AppError> {
        log::info!("开始执行数据库迁移...");
        
        // 迁移1: 添加test_batch_name列到channel_test_instances表
        Self::add_test_batch_name_column(db).await?;
        
        // 迁移2: 添加test_plc_channel_tag和test_plc_communication_address列
        Self::add_test_plc_columns(db).await?;
        
        // 迁移3: 添加batch_name列到test_batch_info表
        Self::add_batch_name_column(db).await?;
        
        // 迁移4: 添加缺失的字段到channel_test_instances表
        Self::add_missing_instance_columns(db).await?;
        
        // 迁移5: 添加overall_status和custom_data列到test_batch_info表
        Self::add_test_batch_info_missing_columns(db).await?;
        
        log::info!("数据库迁移完成");
        Ok(())
    }
    
    /// 添加test_batch_name列到channel_test_instances表
    async fn add_test_batch_name_column(db: &DatabaseConnection) -> Result<(), AppError> {
        // 检查列是否已存在
        let check_sql = "PRAGMA table_info(channel_test_instances)";
        let result = db.query_all(Statement::from_string(
            sea_orm::DatabaseBackend::Sqlite,
            check_sql.to_string()
        )).await.map_err(|e| AppError::persistence_error(format!("检查表结构失败: {}", e)))?;
        
        let mut has_test_batch_name = false;
        for row in result {
            if let Ok(column_name) = row.try_get::<String>("", "name") {
                if column_name == "test_batch_name" {
                    has_test_batch_name = true;
                    break;
                }
            }
        }
        
        if !has_test_batch_name {
            log::info!("添加test_batch_name列到channel_test_instances表");
            let sql = "ALTER TABLE channel_test_instances ADD COLUMN test_batch_name TEXT DEFAULT ''";
            db.execute(Statement::from_string(
                sea_orm::DatabaseBackend::Sqlite,
                sql.to_string()
            )).await.map_err(|e| AppError::persistence_error(format!("添加test_batch_name列失败: {}", e)))?;
            log::info!("成功添加test_batch_name列");
        } else {
            log::info!("test_batch_name列已存在，跳过迁移");
        }
        
        Ok(())
    }
    
    /// 添加batch_name列到test_batch_info表
    async fn add_batch_name_column(db: &DatabaseConnection) -> Result<(), AppError> {
        // 检查列是否已存在
        let check_sql = "PRAGMA table_info(test_batch_info)";
        let result = db.query_all(Statement::from_string(
            sea_orm::DatabaseBackend::Sqlite,
            check_sql.to_string()
        )).await.map_err(|e| AppError::persistence_error(format!("检查test_batch_info表结构失败: {}", e)))?;
        
        let mut has_batch_name = false;
        for row in result {
            if let Ok(column_name) = row.try_get::<String>("", "name") {
                if column_name == "batch_name" {
                    has_batch_name = true;
                    break;
                }
            }
        }
        
        if !has_batch_name {
            log::info!("添加batch_name列到test_batch_info表");
            let sql = "ALTER TABLE test_batch_info ADD COLUMN batch_name TEXT DEFAULT ''";
            db.execute(Statement::from_string(
                sea_orm::DatabaseBackend::Sqlite,
                sql.to_string()
            )).await.map_err(|e| AppError::persistence_error(format!("添加batch_name列失败: {}", e)))?;
            log::info!("成功添加batch_name列到test_batch_info表");
        } else {
            log::info!("batch_name列已存在，跳过迁移");
        }
        
        Ok(())
    }
    
    /// 添加测试PLC相关列
    async fn add_test_plc_columns(db: &DatabaseConnection) -> Result<(), AppError> {
        // 检查test_plc_channel_tag列是否已存在
        let check_sql = "PRAGMA table_info(channel_test_instances)";
        let result = db.query_all(Statement::from_string(
            sea_orm::DatabaseBackend::Sqlite,
            check_sql.to_string()
        )).await.map_err(|e| AppError::persistence_error(format!("检查表结构失败: {}", e)))?;
        
        let mut has_test_plc_channel_tag = false;
        let mut has_test_plc_communication_address = false;
        
        for row in result {
            if let Ok(column_name) = row.try_get::<String>("", "name") {
                if column_name == "test_plc_channel_tag" {
                    has_test_plc_channel_tag = true;
                } else if column_name == "test_plc_communication_address" {
                    has_test_plc_communication_address = true;
                }
            }
        }
        
        if !has_test_plc_channel_tag {
            log::info!("添加test_plc_channel_tag列到channel_test_instances表");
            let sql = "ALTER TABLE channel_test_instances ADD COLUMN test_plc_channel_tag TEXT";
            db.execute(Statement::from_string(
                sea_orm::DatabaseBackend::Sqlite,
                sql.to_string()
            )).await.map_err(|e| AppError::persistence_error(format!("添加test_plc_channel_tag列失败: {}", e)))?;
            log::info!("成功添加test_plc_channel_tag列");
        }
        
        if !has_test_plc_communication_address {
            log::info!("添加test_plc_communication_address列到channel_test_instances表");
            let sql = "ALTER TABLE channel_test_instances ADD COLUMN test_plc_communication_address TEXT";
            db.execute(Statement::from_string(
                sea_orm::DatabaseBackend::Sqlite,
                sql.to_string()
            )).await.map_err(|e| AppError::persistence_error(format!("添加test_plc_communication_address列失败: {}", e)))?;
            log::info!("成功添加test_plc_communication_address列");
        }
        
        Ok(())
    }
    
    /// 添加缺失的字段到channel_test_instances表
    async fn add_missing_instance_columns(db: &DatabaseConnection) -> Result<(), AppError> {
        let check_sql = "PRAGMA table_info(channel_test_instances)";
        let result = db.query_all(Statement::from_string(
            sea_orm::DatabaseBackend::Sqlite,
            check_sql.to_string()
        )).await.map_err(|e| AppError::persistence_error(format!("检查表结构失败: {}", e)))?;
        
        let mut existing_columns = std::collections::HashSet::new();
        for row in result {
            if let Ok(column_name) = row.try_get::<String>("", "name") {
                existing_columns.insert(column_name);
            }
        }
        
        // 需要添加的列定义
        let required_columns = vec![
            ("current_operator", "TEXT"),
            ("retries_count", "INTEGER DEFAULT 0"),
            ("transient_data", "TEXT DEFAULT '{}'"),
        ];
        
        for (column_name, column_def) in required_columns {
            if !existing_columns.contains(column_name) {
                log::info!("添加{}列到channel_test_instances表", column_name);
                let sql = format!("ALTER TABLE channel_test_instances ADD COLUMN {} {}", column_name, column_def);
                db.execute(Statement::from_string(
                    sea_orm::DatabaseBackend::Sqlite,
                    sql
                )).await.map_err(|e| AppError::persistence_error(format!("添加{}列失败: {}", column_name, e)))?;
                log::info!("成功添加{}列", column_name);
            }
        }
        
        Ok(())
    }
    
    /// 添加缺失的字段到test_batch_info表
    async fn add_test_batch_info_missing_columns(db: &DatabaseConnection) -> Result<(), AppError> {
        let check_sql = "PRAGMA table_info(test_batch_info)";
        let result = db.query_all(Statement::from_string(
            sea_orm::DatabaseBackend::Sqlite,
            check_sql.to_string()
        )).await.map_err(|e| AppError::persistence_error(format!("检查test_batch_info表结构失败: {}", e)))?;
        
        let mut existing_columns = std::collections::HashSet::new();
        for row in result {
            if let Ok(column_name) = row.try_get::<String>("", "name") {
                existing_columns.insert(column_name);
            }
        }
        
        // 需要添加的列定义
        let required_columns = vec![
            ("overall_status", "TEXT DEFAULT 'NotTested'"),
            ("custom_data", "TEXT DEFAULT '{}'"),
        ];
        
        for (column_name, column_def) in required_columns {
            if !existing_columns.contains(column_name) {
                log::info!("添加{}列到test_batch_info表", column_name);
                let sql = format!("ALTER TABLE test_batch_info ADD COLUMN {} {}", column_name, column_def);
                db.execute(Statement::from_string(
                    sea_orm::DatabaseBackend::Sqlite,
                    sql
                )).await.map_err(|e| AppError::persistence_error(format!("添加{}列失败: {}", column_name, e)))?;
                log::info!("成功添加{}列到test_batch_info表", column_name);
            }
        }
        
        Ok(())
    }
} 