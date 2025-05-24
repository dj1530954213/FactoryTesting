// 文件: FactoryTesting/src-tauri/src/services/infrastructure/persistence/sqlite_orm_persistence_service.rs
// 详细注释：使用SeaORM和SQLite实现数据持久化服务

use async_trait::async_trait;
use sea_orm::{Database, DatabaseConnection, Schema, ConnectionTrait, ActiveModelTrait, EntityTrait, DbErr, QueryFilter, ColumnTrait};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock; // 如果需要内部状态的并发控制

use crate::models::structs::*;
use crate::models::entities; // 导入实体模块
use crate::services::traits::{BaseService, PersistenceService};
use crate::utils::error::{AppError, AppResult};

// 默认的SQLite数据库文件名
const DEFAULT_DB_FILE: &str = "factory_testing_data.sqlite";
// 数据库URL前缀
const SQLITE_URL_PREFIX: &str = "sqlite://";

/// 基于SeaORM和SQLite的持久化服务实现
pub struct SqliteOrmPersistenceService {
    db_conn: Arc<DatabaseConnection>, // 使用Arc以便在多处共享连接
    db_file_path: PathBuf, // 存储数据库文件的实际路径
    // config: PersistenceConfig, // 如果有特定的配置，可以从这里传入
    // is_initialized: Arc<RwLock<bool>>, // 跟踪初始化状态
}

impl SqliteOrmPersistenceService {
    /// 创建新的 SqliteOrmPersistenceService 实例
    /// 
    /// # Arguments
    /// 
    /// * `db_path` - SQLite数据库文件的可选路径。如果为None，则使用默认路径。
    pub async fn new(db_path_opt: Option<&Path>) -> AppResult<Self> {
        let determined_db_file_path = db_path_opt
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| {
                // 默认情况下，可以考虑将数据库放在应用的数据目录中
                // 这里为了简单，先放在当前工作目录下
                std::env::current_dir()
                    .unwrap_or_default()
                    .join(DEFAULT_DB_FILE)
            });

        let db_url = format!("{}{}", SQLITE_URL_PREFIX, determined_db_file_path.to_string_lossy());

        // 确保数据库文件的父目录存在
        if let Some(parent_dir) = determined_db_file_path.parent() {
            if !parent_dir.exists() {
                tokio::fs::create_dir_all(parent_dir).await.map_err(|e| 
                    AppError::io_error(
                        format!("创建数据库目录失败: {:?}", parent_dir),
                        e.kind().to_string()
                    )
                )?;
            }
        }

        // 连接到数据库，如果数据库文件不存在，SeaORM (sqlx) 会尝试创建它
        let conn = Database::connect(&db_url)
            .await
            .map_err(|db_err| AppError::persistence_error(db_err.to_string()))?;
        
        // 初始化表结构 (如果需要)
        Self::setup_schema(&conn).await?;

        Ok(Self {
            db_conn: Arc::new(conn),
            db_file_path: determined_db_file_path, // 存储路径
            // is_initialized: Arc::new(RwLock::new(true)), // 假设初始化成功
        })
    }

    /// 初始化数据库表结构
    /// 此函数应该负责创建所有必要的表 (如果它们不存在)
    async fn setup_schema(db: &DatabaseConnection) -> AppResult<()> {
        // 使用 ConnectionTrait::execute 和 DatabaseBackend::build 创建表
        let backend = db.get_database_backend();
        let schema = Schema::new(backend);

        let stmt_channel_point_definitions = schema.create_table_from_entity(entities::channel_point_definition::Entity).if_not_exists().to_owned();
        db.execute(backend.build(&stmt_channel_point_definitions))
            .await.map_err(|e| AppError::persistence_error(format!("创建 channel_point_definitions 表失败: {}", e)))?;
        
        let stmt_test_batch_info = schema.create_table_from_entity(entities::test_batch_info::Entity).if_not_exists().to_owned();
        db.execute(backend.build(&stmt_test_batch_info))
            .await.map_err(|e| AppError::persistence_error(format!("创建 test_batch_info 表失败: {}", e)))?;

        let stmt_channel_test_instances = schema.create_table_from_entity(entities::channel_test_instance::Entity).if_not_exists().to_owned();
        db.execute(backend.build(&stmt_channel_test_instances))
            .await.map_err(|e| AppError::persistence_error(format!("创建 channel_test_instances 表失败: {}", e)))?;

        let stmt_raw_test_outcomes = schema.create_table_from_entity(entities::raw_test_outcome::Entity).if_not_exists().to_owned();
        db.execute(backend.build(&stmt_raw_test_outcomes))
            .await.map_err(|e| AppError::persistence_error(format!("创建 raw_test_outcomes 表失败: {}", e)))?;

        // 可以添加更多的表创建逻辑...
        log::info!("数据库表结构设置完成或已存在。");
        Ok(())
    }
}

#[async_trait]
impl BaseService for SqliteOrmPersistenceService {
    fn service_name(&self) -> &'static str {
        "SqliteOrmPersistenceService"
    }

    async fn initialize(&mut self) -> AppResult<()> {
        // 在 new 方法中已经处理了大部分初始化逻辑
        // 这里可以添加额外的初始化步骤，或者简单地返回 Ok
        // let mut init_guard = self.is_initialized.write().await;
        // if *init_guard {
        //     return Ok(());
        // }
        // // ... 执行初始化 ...
        // *init_guard = true;
        log::info!("{} 已初始化。", self.service_name());
        Ok(())
    }

    async fn shutdown(&mut self) -> AppResult<()> {
        // SeaORM 的 DatabaseConnection 在 Drop 时会自动关闭
        // 如果有其他资源需要清理，可以在这里处理
        log::info!("{} 已关闭。", self.service_name());
        Ok(())
    }

    async fn health_check(&self) -> AppResult<()> {
        // 简单的健康检查：尝试ping数据库
        self.db_conn.ping().await.map_err(|db_err| {
            AppError::persistence_error(format!("数据库健康检查失败: {}", db_err))
        })?;
        log::debug!("数据库连接健康。");
        Ok(())
    }
}

#[async_trait]
impl PersistenceService for SqliteOrmPersistenceService {
    // --- ChannelPointDefinition --- 
    async fn save_channel_definition(&self, definition: &ChannelPointDefinition) -> AppResult<()> {
        let active_model: entities::channel_point_definition::ActiveModel = definition.into(); // 使用 From trait 转换
        active_model.save(self.db_conn.as_ref()).await.map_err(|e| AppError::persistence_error(format!("保存通道点位定义失败: {}", e)))?;
        Ok(())
    }

    async fn load_channel_definition(&self, id: &str) -> AppResult<Option<ChannelPointDefinition>> {
        let model = entities::channel_point_definition::Entity::find_by_id(id.to_string())
            .one(self.db_conn.as_ref())
            .await
            .map_err(|e| AppError::persistence_error(format!("加载通道点位定义失败: {}", e)))?;
        Ok(model.map(|m| (&m).into())) // 使用 From trait 转换
    }

    async fn load_all_channel_definitions(&self) -> AppResult<Vec<ChannelPointDefinition>> {
        let models = entities::channel_point_definition::Entity::find()
            .all(self.db_conn.as_ref())
            .await
            .map_err(|e| AppError::persistence_error(format!("加载所有通道点位定义失败: {}", e)))?;
        Ok(models.iter().map(|m| m.into()).collect()) // 使用 From trait 转换
    }

    async fn delete_channel_definition(&self, id: &str) -> AppResult<()> {
        let delete_result = entities::channel_point_definition::Entity::delete_by_id(id.to_string())
            .exec(self.db_conn.as_ref())
            .await
            .map_err(|e| AppError::persistence_error(format!("删除通道点位定义失败: {}", e)))?;
        if delete_result.rows_affected == 0 {
            Err(AppError::not_found_error("ChannelPointDefinition", format!("未找到ID为 {} 的通道点位定义进行删除", id)))
        } else {
            Ok(())
        }
    }

    // --- AppSettings (通过JSON文件处理，与JsonPersistenceService类似) ---
    async fn save_app_settings(&self, settings: &AppSettings) -> AppResult<()> {
        // 实际路径应从配置或固定位置获取
        let path = Path::new("app_settings.json"); 
        let json_data = serde_json::to_string_pretty(settings)
            .map_err(|e| AppError::serialization_error(format!("序列化AppSettings失败: {}", e)))?;
        tokio::fs::write(path, json_data).await.map_err(|e| 
            AppError::io_error(
                format!("保存AppSettings到JSON文件失败: {:?}", path),
                e.kind().to_string()
        ))?;
        Ok(())
    }

    async fn load_app_settings(&self) -> AppResult<Option<AppSettings>> {
        let path = Path::new("app_settings.json");
        if !path.exists() {
            return Ok(None); // 配置文件不存在，返回None
        }
        let json_data = tokio::fs::read_to_string(path).await.map_err(|e| 
            AppError::io_error(
                format!("从JSON文件加载AppSettings失败: {:?}", path),
                e.kind().to_string()
        ))?;
        let settings: AppSettings = serde_json::from_str(&json_data)
            .map_err(|e| AppError::serialization_error(format!("反序列化AppSettings失败: {}", e)))?;
        Ok(Some(settings))
    }

    // --- TestBatchInfo --- 
    async fn save_batch_info(&self, batch: &TestBatchInfo) -> AppResult<()> {
        let active_model: entities::test_batch_info::ActiveModel = batch.into();
        active_model.save(self.db_conn.as_ref()).await.map_err(|e| AppError::persistence_error(format!("保存测试批次信息失败: {}", e)))?;
        Ok(())
    }

    async fn load_batch_info(&self, batch_id: &str) -> AppResult<Option<TestBatchInfo>> {
        let model = entities::test_batch_info::Entity::find_by_id(batch_id.to_string())
            .one(self.db_conn.as_ref())
            .await
            .map_err(|e| AppError::persistence_error(format!("加载测试批次信息失败: {}", e)))?;
        Ok(model.map(|m| (&m).into()))
    }

    async fn load_all_batch_info(&self) -> AppResult<Vec<TestBatchInfo>> {
        let models = entities::test_batch_info::Entity::find()
            .all(self.db_conn.as_ref())
            .await
            .map_err(|e| AppError::persistence_error(format!("加载所有测试批次信息失败: {}", e)))?;
        Ok(models.iter().map(|m| m.into()).collect())
    }

    async fn delete_batch_info(&self, batch_id: &str) -> AppResult<()> {
        let delete_result = entities::test_batch_info::Entity::delete_by_id(batch_id.to_string())
            .exec(self.db_conn.as_ref())
            .await
            .map_err(|e| AppError::persistence_error(format!("删除测试批次信息失败: {}", e)))?;
        if delete_result.rows_affected == 0 {
            Err(AppError::not_found_error("TestBatchInfo", format!("未找到ID为 {} 的测试批次信息进行删除", batch_id)))
        } else {
            Ok(())
        }
    }

    // --- ChannelTestInstance --- 
    async fn save_test_instance(&self, instance: &ChannelTestInstance) -> AppResult<()> {
        let active_model: entities::channel_test_instance::ActiveModel = instance.into();
        active_model.save(self.db_conn.as_ref()).await.map_err(|e| AppError::persistence_error(format!("保存测试实例失败: {}", e)))?;
        Ok(())
    }

    async fn load_test_instance(&self, instance_id: &str) -> AppResult<Option<ChannelTestInstance>> {
        let model = entities::channel_test_instance::Entity::find_by_id(instance_id.to_string())
            .one(self.db_conn.as_ref())
            .await
            .map_err(|e| AppError::persistence_error(format!("加载测试实例失败: {}", e)))?;
        Ok(model.map(|m| (&m).into()))
    }

    async fn load_test_instances_by_batch(&self, batch_id: &str) -> AppResult<Vec<ChannelTestInstance>> {
        let models = entities::channel_test_instance::Entity::find()
            .filter(entities::channel_test_instance::Column::TestBatchId.eq(batch_id.to_string()))
            .all(self.db_conn.as_ref())
            .await
            .map_err(|e| AppError::persistence_error(format!("按批次加载测试实例失败: {}", e)))?;
        Ok(models.iter().map(|m| m.into()).collect())
    }

    async fn delete_test_instance(&self, instance_id: &str) -> AppResult<()> {
        let delete_result = entities::channel_test_instance::Entity::delete_by_id(instance_id.to_string())
            .exec(self.db_conn.as_ref())
            .await
            .map_err(|e| AppError::persistence_error(format!("删除测试实例失败: {}", e)))?;
        if delete_result.rows_affected == 0 {
            Err(AppError::not_found_error("ChannelTestInstance", format!("未找到ID为 {} 的测试实例进行删除", instance_id)))
        } else {
            Ok(())
        }
    }

    // --- RawTestOutcome --- 
    async fn save_test_outcome(&self, outcome: &RawTestOutcome) -> AppResult<()> {
        let active_model: entities::raw_test_outcome::ActiveModel = outcome.into();
        active_model.save(self.db_conn.as_ref()).await.map_err(|e| AppError::persistence_error(format!("保存测试结果失败: {}", e)))?;
        Ok(())
    }

    async fn load_test_outcomes_by_instance(&self, instance_id: &str) -> AppResult<Vec<RawTestOutcome>> {
        let models = entities::raw_test_outcome::Entity::find()
            .filter(entities::raw_test_outcome::Column::ChannelInstanceId.eq(instance_id.to_string()))
            .all(self.db_conn.as_ref())
            .await
            .map_err(|e| AppError::persistence_error(format!("按实例ID加载测试结果失败: {}", e)))?;
        Ok(models.iter().map(|m| m.into()).collect())
    }

    async fn load_test_outcomes_by_batch(&self, _batch_id: &str) -> AppResult<Vec<RawTestOutcome>> {
        // TODO: 当前 RawTestOutcome 没有直接的 batch_id。实现此方法需要修改数据模型或进行更复杂的查询。
        Err(AppError::not_implemented_error("load_test_outcomes_by_batch for SqliteOrmPersistenceService due to data model limitations"))
    }
    
    // --- Backup/Restore (SQLite specific) ---
    async fn backup_data(&self, backup_dir: &std::path::PathBuf) -> AppResult<()> {
        // 简单实现：直接复制SQLite文件
        // 注意：这种方式在数据库正在写入时可能不安全，理想情况下应使用SQLite的在线备份API
        let current_db_path = &self.db_file_path;
        if !current_db_path.exists() {
            return Err(AppError::persistence_error(format!("数据库文件 {:?} 不存在，无法备份", current_db_path)));
        }

        if !backup_dir.exists() {
            tokio::fs::create_dir_all(backup_dir).await.map_err(|e| AppError::io_error(format!("创建备份目录失败: {:?}", backup_dir), e.kind().to_string()))?;
        }
        let backup_file_name = current_db_path.file_name().unwrap_or_else(|| std::ffi::OsStr::new(DEFAULT_DB_FILE));
        let backup_db_path = backup_dir.join(backup_file_name);

        tokio::fs::copy(current_db_path, &backup_db_path).await.map_err(|e| AppError::io_error(format!("备份数据库文件失败从 {:?} 到 {:?}", current_db_path, backup_db_path), e.kind().to_string()))?;
        log::info!("数据库已备份到: {:?}", backup_db_path);
        Ok(())
    }

    async fn restore_data(&self, backup_dir: &std::path::PathBuf) -> AppResult<()> {
        // 简单实现：直接用备份文件覆盖当前SQLite文件
        // 注意：这将丢失当前所有数据！恢复前应有明确的用户确认。
        // 同样，数据库在线时直接覆盖文件可能不安全。
        let current_db_path = &self.db_file_path;
        let backup_file_name = current_db_path.file_name().unwrap_or_else(|| std::ffi::OsStr::new(DEFAULT_DB_FILE));
        let backup_db_path = backup_dir.join(backup_file_name);

        if !backup_db_path.exists() {
            return Err(AppError::persistence_error(format!("备份文件 {:?} 不存在，无法恢复", backup_db_path)));
        }

        // 在实际应用中，恢复前应该关闭当前连接，复制文件，然后重新打开连接。
        // 这里为了简化，我们假设服务在恢复后会重新初始化或用户会重启应用。
        // 直接复制文件可能会导致连接上的操作失败或数据损坏，这只是一个非常基础的实现。
        log::warn!("正在从 {:?} 恢复数据库到 {:?}，当前数据将被覆盖！", backup_db_path, current_db_path);
        tokio::fs::copy(&backup_db_path, current_db_path).await.map_err(|e| AppError::io_error(format!("恢复数据库文件失败从 {:?} 到 {:?}", backup_db_path, current_db_path), e.kind().to_string()))?;
        log::info!("数据库已从 {:?} 恢复。建议重启应用以确保连接一致性。", backup_db_path);
        Ok(())
    }
} 