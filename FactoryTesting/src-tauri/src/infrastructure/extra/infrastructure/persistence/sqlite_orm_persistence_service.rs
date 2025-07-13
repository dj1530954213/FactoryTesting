// 文件: FactoryTesting/src-tauri/src/services/infrastructure/persistence/sqlite_orm_persistence_service.rs
// 详细注释：使用SeaORM和SQLite实现数据持久化服务

use async_trait::async_trait;
use sea_orm::{Database, DatabaseConnection, Schema, ConnectionTrait, EntityTrait, QueryFilter, ColumnTrait, PaginatorTrait, ActiveModelTrait, Set, ConnectOptions};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex}; // 使用 Mutex
use chrono::Utc;
use std::time::Duration;
// 确保导入 rusqlite (如果直接使用其类型)
// use rusqlite; // 如果下面只用 rusqlite::*, 则这个可能不需要
// use sea_orm::sqlx::SqliteConnection; // 通过 sea_orm::sqlx 引用
// use sea_orm::sqlx::Executor; // 如果 Executor 未被使用，可以注释或移除以避免警告

use crate::models::{ChannelPointDefinition, TestBatchInfo, ChannelTestInstance, RawTestOutcome, GlobalFunctionTestStatus};
use crate::models::entities; // 导入实体模块
use crate::domain::services::{BaseService, IPersistenceService as PersistenceService};
use std::any::Any;
// 导入 ExtendedPersistenceService 和相关结构体
use crate::infrastructure::persistence::persistence_service::{
    ExtendedPersistenceService,
    BackupInfo,
    QueryCriteria,
    QueryResult,
    PersistenceStats,
    PersistenceConfig,
    IntegrityReport,
    IntegrityStatus, // 导入 IntegrityStatus
    IntegrityCheckResult // 导入 IntegrityCheckResult
};
use crate::utils::error::{AppError, AppResult};
use log::{info, warn, error, debug, trace};
use uuid::Uuid;

// 定义常量
const DEFAULT_DB_FILE: &str = "factory_testing_data.sqlite";
const SQLITE_URL_PREFIX: &str = "sqlite://";
const BACKUPS_DIR_NAME: &str = "_backups"; // 修改常量名并统一为 _backups

/// 基于SeaORM和SQLite的持久化服务实现
#[derive(Clone)]
pub struct SqliteOrmPersistenceService {
    db_conn: Arc<DatabaseConnection>, // 使用Arc以便在多处共享连接
    db_file_path: PathBuf, // 存储数据库文件的实际路径
    is_active: Arc<Mutex<bool>>, // 新增状态标志
    config: PersistenceConfig, // 添加 config 字段
}

impl SqliteOrmPersistenceService {
    /// 删除 station_name 为空的全局功能测试状态脏数据
    pub async fn delete_blank_station_global_function_tests(&self) -> AppResult<u64> {
        use entities::global_function_test_status;
        use sea_orm::{EntityTrait, ColumnTrait, QueryFilter, Condition};

        let cond = Condition::any()
            .add(global_function_test_status::Column::StationName.is_null())
            .add(global_function_test_status::Column::StationName.eq(""));

        let result = global_function_test_status::Entity::delete_many()
            .filter(cond)
            .exec(self.db_conn.as_ref())
            .await
            .map_err(|e| AppError::persistence_error(format!("删除空 station_name 全局功能测试记录失败: {}", e)))?;
        let affected = result.rows_affected;
        if affected > 0 {
            info!("[PERSIST] 已删除 {} 条空 station_name 的全局功能测试记录", affected);
        }
        Ok(affected)
    }
    /// 创建新的 SqliteOrmPersistenceService 实例
    ///
    /// # Arguments
    ///
    /// * `config` - 持久化服务的配置
    /// * `db_path_opt` - SQLite数据库文件的可选路径。如果为None，则使用默认路径。
    pub async fn new(config: PersistenceConfig, db_path_opt: Option<&Path>) -> AppResult<Self> {
        let determined_db_file_path = db_path_opt
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| {
                // 如果没有提供特定路径，则使用 config 中的 storage_root_dir
                if db_path_opt.and_then(|p| p.to_str()) == Some(":memory:") {
                    return PathBuf::from(":memory:");
                }
                // 默认数据库文件路径基于 config.storage_root_dir
                config.storage_root_dir.join(DEFAULT_DB_FILE)
            });

        info!("🗄️ [PERSIST] 数据库文件路径: {:?}", determined_db_file_path);
        let db_url = if determined_db_file_path.to_str() == Some(":memory:") {
            "sqlite::memory:".to_string()
        } else {
            // 确保使用绝对路径，并正确处理Windows路径
            let absolute_path = if determined_db_file_path.is_absolute() {
                determined_db_file_path.clone()
            } else {
                std::env::current_dir()
                    .map_err(|e| AppError::io_error("获取当前目录失败".to_string(), e.kind().to_string()))?
                    .join(&determined_db_file_path)
            };

            // 在Windows上，需要使用正确的路径格式
            #[cfg(windows)]
            {
                format!("sqlite:///{}", absolute_path.to_string_lossy().replace('\\', "/"))
            }
            #[cfg(not(windows))]
            {
                format!("sqlite://{}", absolute_path.to_string_lossy())
            }
        };

        if determined_db_file_path.to_str() != Some(":memory:") {
            let parent_dir = determined_db_file_path.parent().unwrap_or_else(|| &config.storage_root_dir);
            if !parent_dir.exists() {
                tokio::fs::create_dir_all(parent_dir).await.map_err(|e|
                    AppError::io_error(
                        format!("创建数据库目录失败: {:?}", parent_dir),
                        e.kind().to_string()
                    )
                )?;
            }
        }

        // 使用 ConnectOptions 以自定义连接池参数，避免并发超时
        let mut connect_opts = ConnectOptions::new(db_url.clone());
        connect_opts
            .max_connections(20)
            .min_connections(2)
            .connect_timeout(Duration::from_secs(30))
            .sqlx_logging(false); // 关闭底层 sqlx 日志，减少噪声

        let conn = Database::connect(connect_opts)
            .await
            .map_err(|db_err| AppError::persistence_error(db_err.to_string()))?;

        Self::setup_schema(&conn).await?;

        Ok(Self {
            db_conn: Arc::new(conn),
            db_file_path: determined_db_file_path,
            is_active: Arc::new(Mutex::new(true)),
            config, // 存储 config
        })
    }

    /// 初始化数据库表结构
    /// 此函数应该负责创建所有必要的表 (如果它们不存在)
    async fn setup_schema(db: &DatabaseConnection) -> AppResult<()> {
        // 使用 ConnectionTrait::execute 和 DatabaseBackend::build 创建表
        let backend = db.get_database_backend();
        use sea_orm::sea_query::Index;
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

        // 创建测试PLC配置相关表
        let stmt_test_plc_channel_configs = schema.create_table_from_entity(entities::test_plc_channel_config::Entity).if_not_exists().to_owned();
        db.execute(backend.build(&stmt_test_plc_channel_configs))
            .await.map_err(|e| AppError::persistence_error(format!("创建 test_plc_channel_configs 表失败: {}", e)))?;

        let stmt_plc_connection_configs = schema.create_table_from_entity(entities::plc_connection_config::Entity).if_not_exists().to_owned();
        db.execute(backend.build(&stmt_plc_connection_configs))
            .await.map_err(|e| AppError::persistence_error(format!("创建 plc_connection_configs 表失败: {}", e)))?;

        let stmt_channel_mapping_configs = schema.create_table_from_entity(entities::channel_mapping_config::Entity).if_not_exists().to_owned();
        db.execute(backend.build(&stmt_channel_mapping_configs))
            .await.map_err(|e| AppError::persistence_error(format!("创建 channel_mapping_configs 表失败: {}", e)))?;

        // 全局功能测试状态表
        let stmt_global_function_tests = schema.create_table_from_entity(entities::global_function_test_status::Entity).if_not_exists().to_owned();
        db.execute(backend.build(&stmt_global_function_tests))
            .await.map_err(|e| AppError::persistence_error(format!("创建 global_function_test_statuses 表失败: {}", e)))?;

        // 确保 global_function_test_statuses 表包含 station_name 列 (向后兼容旧版本)
    {
        use sea_orm::{Statement, TryGetable, QueryTrait, ConnectionTrait};
        let col_rows = db
            .query_all(Statement::from_sql_and_values(backend, "PRAGMA table_info(global_function_test_statuses)", vec![]))
            .await
            .map_err(|e| AppError::persistence_error(e.to_string()))?;
        let mut has_station = false;
        let mut has_import_time = false;
        for row in col_rows {
            let name: String = row.try_get("", "name").unwrap_or_default();
            match name.as_str() {
                "station_name" => { has_station = true; },
                "import_time" => { has_import_time = true; },
                _ => {}
            }
        }

        if !has_station {
            let alter_sql = "ALTER TABLE global_function_test_statuses ADD COLUMN station_name TEXT NOT NULL DEFAULT ''";
            if let Err(e) = db.execute_unprepared(alter_sql).await {
                log::error!("添加 station_name 列失败: {}", e);
            } else {
                log::info!("已向 global_function_test_statuses 表添加 station_name 列");
            }
        }

        if !has_import_time {
             let alter_sql = "ALTER TABLE global_function_test_statuses ADD COLUMN import_time TEXT NOT NULL DEFAULT ''";
             if let Err(e) = db.execute_unprepared(alter_sql).await {
                 log::error!("添加 import_time 列失败: {}", e);
             } else {
                 log::info!("已向 global_function_test_statuses 表添加 import_time 列");
             }
         }
     }

     // 检查并删除旧的 station_name+function_key 唯一索引（不含 import_time）
     {
         use sea_orm::{Statement, TryGetable, QueryTrait, ConnectionTrait};
         let idx_rows = db
             .query_all(Statement::from_sql_and_values(backend, "PRAGMA index_list('global_function_test_statuses')", vec![]))
             .await
             .unwrap_or_default();
         for row in idx_rows {
             let idx_name: String = row.try_get("", "name").unwrap_or_default();
             let is_unique: i64 = row.try_get("", "unique").unwrap_or(0);
             if is_unique == 1 {
                 let info_rows = db
                     .query_all(Statement::from_sql_and_values(backend, &format!("PRAGMA index_info('{}')", idx_name), vec![]))
                     .await
                     .unwrap_or_default();
                 let mut cols: Vec<String> = Vec::new();
                 for info in info_rows {
                     let col_name: String = info.try_get("", "name").unwrap_or_default();
                     cols.push(col_name);
                 }
                 if cols == ["station_name", "function_key"] {
                     let drop_sql = format!("DROP INDEX IF EXISTS {}", idx_name);
                     if let Err(e) = db.execute_unprepared(&drop_sql).await {
                         log::error!("删除旧索引 {} 失败: {}", idx_name, e);
                     } else {
                         log::info!("已删除旧唯一索引 {}", idx_name);
                     }
                 }
             }
         }
     }

     // 为 station_name + import_time + function_key 创建唯一索引
        let idx_stmt = Index::create()
            .name("idx_gfts_station_time_function")
            .table(entities::global_function_test_status::Entity)
            .col(entities::global_function_test_status::Column::StationName)
            .col(entities::global_function_test_status::Column::ImportTime)
            .col(entities::global_function_test_status::Column::FunctionKey)
            .if_not_exists()
            .unique()
            .to_owned();
        db.execute(backend.build(&idx_stmt))
            .await
            .map_err(|e| AppError::persistence_error(format!("创建 global_function_test_statuses 索引失败: {}", e)))?;

        log::info!("数据库表结构设置完成或已存在。");
        Ok(())
    }

    /// 获取数据库连接（用于迁移等操作）
    pub fn get_database_connection(&self) -> &DatabaseConnection {
        self.db_conn.as_ref()
    }
}

#[async_trait]
impl BaseService for SqliteOrmPersistenceService {
    fn service_name(&self) -> &'static str {
        "SqliteOrmPersistenceService"
    }

    async fn initialize(&mut self) -> AppResult<()> {
        let mut active_guard = self.is_active.lock().unwrap();
        if !*active_guard { // 如果服务之前被关闭，则重新激活
            *active_guard = true;
            log::info!("{} 已重新初始化并激活。", self.service_name());
        } else {
            log::info!("{} 已初始化或已处于激活状态。", self.service_name());
        }
        // 实际的数据库和模式初始化已在 new() 中完成
        Ok(())
    }

    async fn shutdown(&mut self) -> AppResult<()> {
        let mut active_guard = self.is_active.lock().unwrap();
        *active_guard = false;
        log::info!("{} 服务已关闭。实际数据库连接将在Arc释放时关闭。", self.service_name());
        Ok(())
    }

    async fn health_check(&self) -> AppResult<()> {
        {
            let active_guard = self.is_active.lock().unwrap();
            if !*active_guard {
                return Err(AppError::service_health_check_error(
                    self.service_name().to_string(),
                    "服务已被关闭".to_string() // 更清晰的错误消息
                ));
            }
        }
        // 仅在服务激活时才 ping 数据库
        self.db_conn.ping().await.map_err(|db_err| {
            AppError::persistence_error(format!("数据库健康检查失败 (ping): {}", db_err))
        })?;
        Ok(())
    }
}

#[async_trait]
impl PersistenceService for SqliteOrmPersistenceService {
    async fn delete_blank_station_global_function_tests(&self) -> AppResult<()> {
        let _ = self.delete_blank_station_global_function_tests().await?;
        Ok(())
    }
    // --- ChannelPointDefinition ---
    async fn save_channel_definition(&self, definition: &ChannelPointDefinition) -> AppResult<()> {
        // 验证UUID格式
        if definition.id.is_empty() || definition.id.len() < 36 {
            let error_msg = format!("无效的UUID格式: '{}'", definition.id);
            log::error!("❌ [SAVE_DEFINITION] {}", error_msg);
            return Err(AppError::validation_error(error_msg));
        }

        // 检查是否已存在相同ID的记录
        let existing = entities::channel_point_definition::Entity::find_by_id(definition.id.clone())
            .one(self.db_conn.as_ref())
            .await
            .map_err(|e| {
                let error_msg = format!("查询通道点位定义失败: {}", e);
                log::error!("❌ [SAVE_DEFINITION] {}", error_msg);
                AppError::persistence_error(error_msg)
            })?;

        if existing.is_some() {
            // 记录已存在，执行更新操作
            let mut active_model: entities::channel_point_definition::ActiveModel = definition.into();
            // 确保ID不变
            active_model.id = Set(definition.id.clone());
            active_model.updated_time = Set(chrono::Utc::now().to_rfc3339());

            active_model.update(self.db_conn.as_ref())
                .await
                .map_err(|e| {
                    let error_msg = format!("更新通道点位定义失败: {} - {}", definition.tag, e);
                    log::error!("❌ [SAVE_DEFINITION] {}", error_msg);
                    AppError::persistence_error(error_msg)
                })?;
        } else {
            // 记录不存在，执行插入操作
            let active_model: entities::channel_point_definition::ActiveModel = definition.into();

            entities::channel_point_definition::Entity::insert(active_model)
                .exec(self.db_conn.as_ref())
                .await
                .map_err(|e| {
                    let error_msg = format!("插入通道点位定义失败: {} - 详细错误: {}", definition.tag, e);
                    log::error!("❌ [SAVE_DEFINITION] {}", error_msg);
                    log::error!("❌ [SAVE_DEFINITION] 失败的定义详情: ID={}, Tag={}, ModuleType={:?}",
                        definition.id, definition.tag, definition.module_type);
                    AppError::persistence_error(error_msg)
                })?;
        }

        Ok(())
    }

    /// 批量保存通道点位定义
    async fn save_channel_definitions(&self, definitions: &[ChannelPointDefinition]) -> AppResult<()> {
        if definitions.is_empty() { return Ok(()); }
        for def in definitions {
            self.save_channel_definition(def).await?;
        }
        Ok(())
    }

    async fn load_channel_definition(&self, id: &str) -> AppResult<Option<ChannelPointDefinition>> {
        let model = entities::channel_point_definition::Entity::find_by_id(id.to_string())
            .one(self.db_conn.as_ref())
            .await
            .map_err(|e| {
                let error_msg = format!("加载通道点位定义失败: ID={} - {}", id, e);
                log::error!("❌ [LOAD_DEFINITION] {}", error_msg);
                AppError::persistence_error(error_msg)
            })?;

        Ok(model.map(|m| (&m).into())) // 使用 From trait 转换
    }

    async fn load_all_channel_definitions(&self) -> AppResult<Vec<ChannelPointDefinition>> {
        let models = entities::channel_point_definition::Entity::find()
            .all(self.db_conn.as_ref())
            .await
            .map_err(|e| {
                let error_msg = format!("加载所有通道点位定义失败: {}", e);
                log::error!("❌ [LOAD_ALL_DEFINITIONS] {}", error_msg);
                AppError::persistence_error(error_msg)
            })?;

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

    // --- TestBatchInfo ---
    async fn save_batch_info(&self, batch: &TestBatchInfo) -> AppResult<()> {
        // 检查是否已存在相同ID的记录
        let existing = entities::test_batch_info::Entity::find_by_id(batch.batch_id.clone())
            .one(self.db_conn.as_ref())
            .await
            .map_err(|e| AppError::persistence_error(format!("查询测试批次失败: {}", e)))?;

        if existing.is_some() {
            // 记录已存在，执行更新操作
            let mut active_model: entities::test_batch_info::ActiveModel = batch.into();
            // 确保ID不变
            active_model.batch_id = Set(batch.batch_id.clone());
            active_model.updated_time = Set(chrono::Utc::now());

            active_model.update(self.db_conn.as_ref())
                .await
                .map_err(|e| AppError::persistence_error(format!("更新测试批次失败: {}", e)))?;
        } else {
            // 记录不存在，执行插入操作
            let active_model: entities::test_batch_info::ActiveModel = batch.into();
            entities::test_batch_info::Entity::insert(active_model)
                .exec(self.db_conn.as_ref())
                .await
                .map_err(|e| AppError::persistence_error(format!("插入测试批次失败: {}", e)))?;
        }

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
        // 检查是否已存在相同ID的记录
        let existing = entities::channel_test_instance::Entity::find_by_id(instance.instance_id.clone())
            .one(self.db_conn.as_ref())
            .await
            .map_err(|e| AppError::persistence_error(format!("查询测试实例失败: {}", e)))?;

        if existing.is_some() {
            // 记录已存在，执行更新操作
            let mut active_model: entities::channel_test_instance::ActiveModel = instance.into();
            // 确保ID不变
            active_model.instance_id = Set(instance.instance_id.clone());
            active_model.updated_time = Set(chrono::Utc::now());

            active_model.update(self.db_conn.as_ref())
                .await
                .map_err(|e| AppError::persistence_error(format!("更新测试实例失败: {}", e)))?;

            // 🔧 移除 [PERSISTENCE] 日志
        } else {
            // 记录不存在，执行插入操作
            let active_model: entities::channel_test_instance::ActiveModel = instance.into();
            entities::channel_test_instance::Entity::insert(active_model)
                .exec(self.db_conn.as_ref())
                .await
                .map_err(|e| AppError::persistence_error(format!("插入测试实例失败: {}", e)))?;

            // 🔧 移除 [PERSISTENCE] 日志
        }

        Ok(())
    }

    async fn load_test_instance(&self, instance_id: &str) -> AppResult<Option<ChannelTestInstance>> {
        // 🔧 性能优化：移除详细调试日志，只保留关键错误信息
        let model = entities::channel_test_instance::Entity::find_by_id(instance_id.to_string())
            .one(self.db_conn.as_ref())
            .await
            .map_err(|e| AppError::persistence_error(format!("加载测试实例失败: {}", e)))?;

        Ok(model.map(|m| (&m).into()))
    }

    async fn load_all_test_instances(&self) -> AppResult<Vec<ChannelTestInstance>> {
        let models = entities::channel_test_instance::Entity::find()
            .all(self.db_conn.as_ref())
            .await
            .map_err(|e| AppError::persistence_error(format!("加载所有测试实例失败: {}", e)))?;

        // 🔧 性能优化：移除详细调试日志
        Ok(models.iter().map(|m| m.into()).collect())
    }

    async fn load_test_instances_by_batch(&self, batch_id: &str) -> AppResult<Vec<ChannelTestInstance>> {
        // 🔧 修复：强制从数据库重新查询，避免 ORM 缓存问题
        // 使用 fresh() 方法确保获取最新数据
        let models = entities::channel_test_instance::Entity::find()
            .filter(entities::channel_test_instance::Column::TestBatchId.eq(batch_id.to_string()))
            .all(self.db_conn.as_ref())
            .await
            .map_err(|e| AppError::persistence_error(format!("按批次加载测试实例失败: {}", e)))?;

        // 🔧 添加数据验证日志
        // 🔧 性能优化：移除持久化详细日志

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
        trace!("🛢️ [PERSIST] 准备插入 RawTestOutcome, instance_id={}, sub_test_item={:?}", outcome.channel_instance_id, outcome.sub_test_item);

        let active_model: entities::raw_test_outcome::ActiveModel = outcome.into();
        let insert_result = entities::raw_test_outcome::Entity::insert(active_model)
            .exec(self.db_conn.as_ref())
            .await
            .map_err(|e| {
                error!("❌ [PERSIST] 插入 RawTestOutcome 失败: {}", e);
                AppError::persistence_error(format!("保存测试结果失败: {}", e))
            })?;
        trace!("✅ [PERSIST] RawTestOutcome 插入成功");
        Ok(())
    }

    async fn load_test_outcomes_by_instance(&self, instance_id: &str) -> AppResult<Vec<RawTestOutcome>> {
        let models = entities::raw_test_outcome::Entity::find()
            .filter(entities::raw_test_outcome::Column::ChannelInstanceId.eq(instance_id))
            .all(self.db_conn.as_ref())
            .await
            .map_err(|e| AppError::persistence_error(format!("加载实例 {} 的测试结果失败: {}", instance_id, e)))?;
        Ok(models.iter().map(|m| m.into()).collect())
    }

    /// 按批次ID查询测试结果
    async fn load_test_outcomes_by_batch(&self, batch_id: &str) -> AppResult<Vec<RawTestOutcome>> {
        // 由于 raw_test_outcome 表中没有直接的 test_batch_id 字段，
        // 我们需要通过 channel_test_instance 表来关联查询
        // 这里先简化实现，返回所有测试结果
        // TODO: 实现正确的关联查询
        let models = entities::raw_test_outcome::Entity::find()
            .all(self.db_conn.as_ref())
            .await
            .map_err(|e| AppError::persistence_error(format!("按批次ID查询测试结果失败: {}", e)))?;

        // 过滤属于指定批次的结果
        // 这需要通过 channel_instance_id 关联到 channel_test_instance 表
        // 暂时返回所有结果，后续可以优化为正确的关联查询
        Ok(models.iter().map(|m| m.into()).collect())
    }

    // 测试PLC配置相关方法

    /// 保存测试PLC通道配置
    async fn save_test_plc_channel(&self, channel: &crate::models::test_plc_config::TestPlcChannelConfig) -> AppResult<()> {
        use sea_orm::{ActiveModelTrait, Set};

        let active_model: entities::test_plc_channel_config::ActiveModel = channel.into();

        // 检查是否有ID，如果有ID则尝试更新，否则插入
        if let Some(id) = &channel.id {
            // 检查记录是否存在
            let existing = entities::test_plc_channel_config::Entity::find_by_id(id.clone())
                .one(self.db_conn.as_ref())
                .await
                .map_err(|e| {
                    error!("检查测试PLC通道配置是否存在失败: {}", e);
                    AppError::persistence_error(format!("检查测试PLC通道配置是否存在失败: {}", e))
                })?;

            if existing.is_some() {
                // 记录存在，执行更新
                active_model.update(self.db_conn.as_ref())
                    .await
                    .map_err(|e| {
                        error!("更新测试PLC通道配置失败: {}", e);
                        AppError::persistence_error(format!("更新测试PLC通道配置失败: {}", e))
                    })?;
            } else {
                // 记录不存在，执行插入
                active_model.insert(self.db_conn.as_ref())
                    .await
                    .map_err(|e| {
                        error!("插入测试PLC通道配置失败: {}", e);
                        AppError::persistence_error(format!("插入测试PLC通道配置失败: {}", e))
                    })?;
            }
        } else {
            // 没有ID，执行插入
            active_model.insert(self.db_conn.as_ref())
                .await
                .map_err(|e| {
                    error!("插入新测试PLC通道配置失败: {}", e);
                    AppError::persistence_error(format!("插入新测试PLC通道配置失败: {}", e))
                })?;
        }

        Ok(())
    }

    /// 加载测试PLC通道配置
    async fn load_test_plc_channel(&self, id: &str) -> AppResult<Option<crate::models::test_plc_config::TestPlcChannelConfig>> {
        let model = entities::test_plc_channel_config::Entity::find_by_id(id.to_string())
            .one(self.db_conn.as_ref())
            .await
            .map_err(|e| AppError::persistence_error(format!("加载测试PLC通道配置失败: {}", e)))?;
        Ok(model.map(|m| (&m).into()))
    }

    /// 加载所有测试PLC通道配置
    async fn load_all_test_plc_channels(&self) -> AppResult<Vec<crate::models::test_plc_config::TestPlcChannelConfig>> {
        let models = entities::test_plc_channel_config::Entity::find()
            .all(self.db_conn.as_ref())
            .await
            .map_err(|e| AppError::persistence_error(format!("加载所有测试PLC通道配置失败: {}", e)))?;
        Ok(models.iter().map(|m| m.into()).collect())
    }

    /// 删除测试PLC通道配置
    async fn delete_test_plc_channel(&self, id: &str) -> AppResult<()> {
        let delete_result = entities::test_plc_channel_config::Entity::delete_by_id(id.to_string())
            .exec(self.db_conn.as_ref())
            .await
            .map_err(|e| AppError::persistence_error(format!("删除测试PLC通道配置失败: {}", e)))?;
        if delete_result.rows_affected == 0 {
            Err(AppError::not_found_error("TestPlcChannelConfig", format!("未找到ID为 {} 的测试PLC通道配置进行删除", id)))
        } else {
            Ok(())
        }
    }

    /// 保存PLC连接配置
    async fn save_plc_connection(&self, connection: &crate::models::test_plc_config::PlcConnectionConfig) -> AppResult<()> {
        // 检查是否已存在相同ID的记录
        let existing = entities::plc_connection_config::Entity::find_by_id(connection.id.clone())
            .one(self.db_conn.as_ref())
            .await
            .map_err(|e| AppError::persistence_error(format!("检查PLC连接配置是否存在失败: {}", e)))?;

        if existing.is_some() {
            // 更新现有记录
            let mut active_model: entities::plc_connection_config::ActiveModel = connection.into();
            // 确保ID不被重新设置
            active_model.id = sea_orm::ActiveValue::Unchanged(connection.id.clone());
            // 更新时间
            active_model.updated_at = sea_orm::ActiveValue::Set(chrono::Utc::now());

            entities::plc_connection_config::Entity::update(active_model)
                .exec(self.db_conn.as_ref())
                .await
                .map_err(|e| AppError::persistence_error(format!("更新PLC连接配置失败: {}", e)))?;
        } else {
            // 插入新记录
            let active_model: entities::plc_connection_config::ActiveModel = connection.into();
            entities::plc_connection_config::Entity::insert(active_model)
                .exec(self.db_conn.as_ref())
                .await
                .map_err(|e| AppError::persistence_error(format!("保存PLC连接配置失败: {}", e)))?;
        }

        Ok(())
    }

    /// 加载PLC连接配置
    async fn load_plc_connection(&self, id: &str) -> AppResult<Option<crate::models::test_plc_config::PlcConnectionConfig>> {
        let model = entities::plc_connection_config::Entity::find_by_id(id.to_string())
            .one(self.db_conn.as_ref())
            .await
            .map_err(|e| AppError::persistence_error(format!("加载PLC连接配置失败: {}", e)))?;
        Ok(model.map(|m| (&m).into()))
    }

    /// 加载所有PLC连接配置
    async fn load_all_plc_connections(&self) -> AppResult<Vec<crate::models::test_plc_config::PlcConnectionConfig>> {
        let models = entities::plc_connection_config::Entity::find()
            .all(self.db_conn.as_ref())
            .await
            .map_err(|e| AppError::persistence_error(format!("加载所有PLC连接配置失败: {}", e)))?;
        Ok(models.iter().map(|m| m.into()).collect())
    }

    /// 删除PLC连接配置
    async fn delete_plc_connection(&self, id: &str) -> AppResult<()> {
        let delete_result = entities::plc_connection_config::Entity::delete_by_id(id.to_string())
            .exec(self.db_conn.as_ref())
            .await
            .map_err(|e| AppError::persistence_error(format!("删除PLC连接配置失败: {}", e)))?;
        if delete_result.rows_affected == 0 {
            Err(AppError::not_found_error("PlcConnectionConfig", format!("未找到ID为 {} 的PLC连接配置进行删除", id)))
        } else {
            Ok(())
        }
    }

    /// 保存通道映射配置
    async fn save_channel_mapping(&self, mapping: &crate::models::test_plc_config::ChannelMappingConfig) -> AppResult<()> {
        let active_model: entities::channel_mapping_config::ActiveModel = mapping.into();
        entities::channel_mapping_config::Entity::insert(active_model)
            .exec(self.db_conn.as_ref())
            .await
            .map_err(|e| AppError::persistence_error(format!("保存通道映射配置失败: {}", e)))?;
        Ok(())
    }

    /// 加载通道映射配置
    async fn load_channel_mapping(&self, id: &str) -> AppResult<Option<crate::models::test_plc_config::ChannelMappingConfig>> {
        let model = entities::channel_mapping_config::Entity::find_by_id(id.to_string())
            .one(self.db_conn.as_ref())
            .await
            .map_err(|e| AppError::persistence_error(format!("加载通道映射配置失败: {}", e)))?;
        Ok(model.map(|m| (&m).into()))
    }

    /// 加载所有通道映射配置
    async fn load_all_channel_mappings(&self) -> AppResult<Vec<crate::models::test_plc_config::ChannelMappingConfig>> {
        let models = entities::channel_mapping_config::Entity::find()
            .all(self.db_conn.as_ref())
            .await
            .map_err(|e| AppError::persistence_error(format!("加载所有通道映射配置失败: {}", e)))?;
        Ok(models.iter().map(|m| m.into()).collect())
    }

    /// 删除通道映射配置
    async fn delete_channel_mapping(&self, id: &str) -> AppResult<()> {
        let delete_result = entities::channel_mapping_config::Entity::delete_by_id(id.to_string())
            .exec(self.db_conn.as_ref())
            .await
            .map_err(|e| AppError::persistence_error(format!("删除通道映射配置失败: {}", e)))?;
        if delete_result.rows_affected == 0 {
            Err(AppError::not_found_error("ChannelMappingConfig", format!("未找到ID为 {} 的通道映射配置进行删除", id)))
        } else {
            Ok(())
        }
    }

    // ===== 全局功能测试状态 =====
    async fn save_global_function_test_status(&self, status: &GlobalFunctionTestStatus) -> AppResult<()> {
        if status.import_time.trim().is_empty() {
            return Err(AppError::validation_error("import_time 不能为空，请确保前端正确传递导入时间"));
        }
        log::info!("[PERSIST] save_global_function_test_status station='{}' key={:?} status={:?}", status.station_name, status.function_key, status.status);
        use sea_orm::{ActiveModelTrait, Set};
        // 通过 function_key 查找是否已存在
        let existing = entities::global_function_test_status::Entity::find()
            .filter(entities::global_function_test_status::Column::StationName.eq(status.station_name.clone()))
            .filter(entities::global_function_test_status::Column::ImportTime.eq(status.import_time.clone()))
            .filter(entities::global_function_test_status::Column::FunctionKey.eq(status.function_key.to_string()))
            .one(self.db_conn.as_ref())
            .await
            .map_err(|e| AppError::persistence_error(format!("查询 GlobalFunctionTestStatus 失败: {}", e)))?;

        if let Some(model) = existing {
        log::debug!("[PERSIST] existing record found, performing UPDATE");
            // 更新
            let mut am: entities::global_function_test_status::ActiveModel = model.clone().into();
            am.import_time = Set(status.import_time.clone());
            am.start_time = Set(status.start_time.clone());
            am.end_time = Set(status.end_time.clone());
            am.status = Set(status.status.to_string());
            am.update(self.db_conn.as_ref())
                .await
                .map_err(|e| AppError::persistence_error(format!("更新 GlobalFunctionTestStatus 失败: {}", e)))?;
        } else {
            // 未找到精确匹配记录，尝试回退匹配旧数据（import_time 为空或 NULL）
            let legacy_existing = entities::global_function_test_status::Entity::find()
                .filter(entities::global_function_test_status::Column::StationName.eq(status.station_name.clone()))
                .filter(entities::global_function_test_status::Column::FunctionKey.eq(status.function_key.to_string()))
                .filter(
                    entities::global_function_test_status::Column::ImportTime
                        .eq("")
                        .or(entities::global_function_test_status::Column::ImportTime.is_null()),
                )
                .one(self.db_conn.as_ref())
                .await
                .map_err(|e| AppError::persistence_error(format!("查询 legacy GlobalFunctionTestStatus 失败: {}", e)))?;

            if let Some(model) = legacy_existing {
                log::info!("[PERSIST] legacy record found, updating and migrating import_time");
                let mut am: entities::global_function_test_status::ActiveModel = model.into();
                am.import_time = Set(status.import_time.clone());
                am.start_time = Set(status.start_time.clone());
                am.end_time = Set(status.end_time.clone());
                am.status = Set(status.status.to_string());
                am.update(self.db_conn.as_ref())
                    .await
                    .map_err(|e| AppError::persistence_error(format!("更新 legacy GlobalFunctionTestStatus 失败: {}", e)))?;
            } else {
                // 仍未找到，则执行插入
                let am: entities::global_function_test_status::ActiveModel = status.into();
                entities::global_function_test_status::Entity::insert(am)
                    .exec(self.db_conn.as_ref())
                    .await
                    .map_err(|e| AppError::persistence_error(format!("插入 GlobalFunctionTestStatus 失败: {}", e)))?;
            }
        }
        Ok(())
    }

    async fn load_all_global_function_test_statuses(&self) -> AppResult<Vec<GlobalFunctionTestStatus>> {
        let models = entities::global_function_test_status::Entity::find()
            .all(self.db_conn.as_ref())
            .await
            .map_err(|e| AppError::persistence_error(format!("加载 GlobalFunctionTestStatus 失败: {}", e)))?;
        Ok(models.iter().map(|m| m.into()).collect())
    }

    async fn reset_global_function_test_statuses(&self) -> AppResult<()> {
        entities::global_function_test_status::Entity::delete_many()
            .exec(self.db_conn.as_ref())
            .await
            .map_err(|e| AppError::persistence_error(format!("重置 GlobalFunctionTestStatus 失败: {}", e)))?;
        Ok(())
    }

    async fn load_global_function_test_statuses_by_station(&self, station_name: &str) -> AppResult<Vec<GlobalFunctionTestStatus>> {
        let models = entities::global_function_test_status::Entity::find()
            .filter(entities::global_function_test_status::Column::StationName.eq(station_name))
            .all(self.db_conn.as_ref())
            .await
            .map_err(|e| AppError::persistence_error(format!("加载 GlobalFunctionTestStatus by station 失败: {}", e)))?;
        Ok(models.iter().map(|m| m.into()).collect())
    }

    async fn load_global_function_test_statuses_by_station_time(&self, station_name: &str, import_time: &str) -> AppResult<Vec<GlobalFunctionTestStatus>> {
        let models = entities::global_function_test_status::Entity::find()
            .filter(entities::global_function_test_status::Column::StationName.eq(station_name))
            .filter(entities::global_function_test_status::Column::ImportTime.eq(import_time))
            .all(self.db_conn.as_ref())
            .await
            .map_err(|e| AppError::persistence_error(format!("加载 GlobalFunctionTestStatus by station+time 失败: {}", e)))?;
        Ok(models.iter().map(|m| m.into()).collect())
    }

    async fn ensure_global_function_tests(&self, station_name: &str, import_time: &str) -> AppResult<()> {
        if import_time.trim().is_empty() {
            return Err(AppError::validation_error("import_time 不能为空"));
        }
        use crate::models::{GlobalFunctionKey, GlobalFunctionTestStatus};
        use sea_orm::ActiveModelTrait;
        // 查询是否已有记录
        let existing = entities::global_function_test_status::Entity::find()
            .filter(entities::global_function_test_status::Column::StationName.eq(station_name))
            .filter(entities::global_function_test_status::Column::ImportTime.eq(import_time))
            .all(self.db_conn.as_ref())
            .await
            .map_err(|e| AppError::persistence_error(format!("查询 GlobalFunctionTestStatus 失败: {}", e)))?;
        if !existing.is_empty() {
            return Ok(());
        }
        // 批量生成 5 条默认记录
        let mut default_items = Vec::new();
        for key in [
            GlobalFunctionKey::HistoricalTrend,
            GlobalFunctionKey::RealTimeTrend,
            GlobalFunctionKey::Report,
            GlobalFunctionKey::AlarmLevelSound,
            GlobalFunctionKey::OperationLog,
        ] {
            default_items.push(GlobalFunctionTestStatus {
                station_name: station_name.to_string(),
                id: crate::models::structs::default_id(),
                function_key: key,
                import_time: import_time.to_string(),
                start_time: None,
                end_time: None,
                status: crate::models::enums::OverallTestStatus::NotTested,
            });
        }
        for item in default_items {
            let am: entities::global_function_test_status::ActiveModel = (&item).into();
            entities::global_function_test_status::Entity::insert(am)
                .exec(self.db_conn.as_ref())
                .await
                .map_err(|e| AppError::persistence_error(format!("插入默认 GlobalFunctionTestStatus 失败: {}", e)))?;
        }
        Ok(())
    }

    async fn reset_global_function_test_statuses_by_station(&self, station_name: &str) -> AppResult<()> {
        entities::global_function_test_status::Entity::delete_many()
            .filter(entities::global_function_test_status::Column::StationName.eq(station_name))
            .exec(self.db_conn.as_ref())
            .await
            .map_err(|e| AppError::persistence_error(format!("按站场重置 GlobalFunctionTestStatus 失败: {}", e)))?;
        Ok(())
    }

    async fn query_channel_definitions(&self, _criteria: &crate::domain::services::persistence_service::QueryCriteria) -> AppResult<Vec<ChannelPointDefinition>> {
        Err(AppError::not_implemented_error("query_channel_definitions"))
    }

    async fn save_test_instances(&self, _instances: &[ChannelTestInstance]) -> AppResult<()> {
        Err(AppError::not_implemented_error("save_test_instances (bulk)"))
    }

    async fn query_test_instances(&self, _criteria: &crate::domain::services::persistence_service::QueryCriteria) -> AppResult<Vec<ChannelTestInstance>> {
        Err(AppError::not_implemented_error("query_test_instances"))
    }

    async fn query_batch_info(&self, _criteria: &crate::domain::services::persistence_service::QueryCriteria) -> AppResult<Vec<TestBatchInfo>> {
        Err(AppError::not_implemented_error("query_batch_info"))
    }

    async fn save_test_outcomes(&self, outcomes: &[RawTestOutcome]) -> AppResult<()> {
        trace!("🛢️ [PERSIST] 批量保存 {} 条 RawTestOutcome", outcomes.len());
        for outcome in outcomes {
            self.save_test_outcome(outcome).await?;
        }
        Ok(())
    }

    async fn execute_transaction(&self, _operations: Vec<crate::domain::services::persistence_service::TransactionOperation>) -> AppResult<crate::domain::services::persistence_service::TransactionResult> {
        Err(AppError::not_implemented_error("execute_transaction"))
    }

    async fn create_backup(&self, _backup_name: &str) -> AppResult<crate::domain::services::persistence_service::BackupInfo> {
        Err(AppError::not_implemented_error("create_backup"))
    }

    async fn restore_backup(&self, _backup_id: &str) -> AppResult<()> {
        Err(AppError::not_implemented_error("restore_backup"))
    }

    async fn get_storage_statistics(&self) -> AppResult<crate::domain::services::persistence_service::StorageStatistics> {
        Err(AppError::not_implemented_error("get_storage_statistics"))
    }

    async fn cleanup_expired_data(&self, _retention_policy: &crate::domain::services::persistence_service::RetentionPolicy) -> AppResult<crate::domain::services::persistence_service::CleanupResult> {
        Err(AppError::not_implemented_error("cleanup_expired_data"))
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[async_trait]
impl ExtendedPersistenceService for SqliteOrmPersistenceService {
    async fn backup(&self, backup_name: &str) -> AppResult<BackupInfo> {
        log::warn!("SqliteOrmPersistenceService::backup is temporarily not implemented.");
        // 暂时移除具体实现，直到依赖问题解决
        Err(AppError::not_implemented_error(format!("Backup functionality for '{}' is temporarily disabled due to dependency issues.", backup_name)))
    }

    async fn restore_from_backup(&self, backup_path: &PathBuf) -> AppResult<()> {
        log::warn!("SqliteOrmPersistenceService::restore_from_backup is temporarily not implemented.");
        Err(AppError::not_implemented_error(format!("Restore from backup functionality for '{:?}' is temporarily disabled.", backup_path)))
    }

    async fn list_backups(&self) -> AppResult<Vec<BackupInfo>> {
        log::warn!("SqliteOrmPersistenceService::list_backups is temporarily not implemented.");
        Err(AppError::not_implemented_error("List backups functionality is temporarily disabled.".to_string()))
        // 之前的实现：
        // let backup_dir = self.config.storage_root_dir.join(BACKUPS_DIR_NAME);
        // if !backup_dir.exists() {
        //     return Ok(Vec::new());
        // }
        // let mut backups = Vec::new();
        // let mut entries = tokio::fs::read_dir(backup_dir).await.map_err(|e|
        //     AppError::io_error("读取备份目录失败".to_string(), e.kind().to_string())
        // )?;
        // while let Some(entry) = entries.next_entry().await.map_err(|e|
        //     AppError::io_error("读取备份目录条目失败".to_string(), e.kind().to_string()))? {
        //     let path = entry.path();
        //     if path.is_file() && path.extension().map_or(false, |ext| ext == "sqlite") {
        //         let metadata = tokio::fs::metadata(&path).await.map_err(|e|
        //             AppError::io_error(format!("获取备份文件 {:?} 元数据失败", path), e.kind().to_string()))?;
        //         let name_cow = path.file_stem().unwrap_or_default().to_string_lossy();
        //         let is_auto = name_cow.starts_with("auto_");
        //         let name_owned = name_cow.into_owned();
        //         backups.push(BackupInfo {
        //             name: name_owned,
        //             path,
        //             size_bytes: metadata.len(),
        //             created_at: metadata.created().map(DateTime::from).unwrap_or_else(|_| Utc::now()),
        //             description: Some(format!("SQLite backup created on {}", Utc::now().to_rfc2822())),
        //             is_auto_backup: is_auto,
        //         });
        //     }
        // }
        // backups.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        // Ok(backups)
    }

    // --- Placeholder implementations for other ExtendedPersistenceService methods ---
    async fn query_channel_definitions(&self, _criteria: &QueryCriteria) -> AppResult<QueryResult<ChannelPointDefinition>> {
        Err(AppError::not_implemented_error("query_channel_definitions not implemented for SqliteOrmPersistenceService".to_string()))
    }
    async fn query_test_instances(&self, _criteria: &QueryCriteria) -> AppResult<QueryResult<ChannelTestInstance>> {
        Err(AppError::not_implemented_error("query_test_instances not implemented for SqliteOrmPersistenceService".to_string()))
    }



    


    async fn query_test_batches(&self, _criteria: &QueryCriteria) -> AppResult<QueryResult<TestBatchInfo>> {
        Err(AppError::not_implemented_error("query_test_batches not implemented for SqliteOrmPersistenceService".to_string()))
    }
    async fn query_test_outcomes(&self, _criteria: &QueryCriteria) -> AppResult<QueryResult<RawTestOutcome>> {
        Err(AppError::not_implemented_error("query_test_outcomes not implemented for SqliteOrmPersistenceService".to_string()))
    }
    async fn batch_save_channel_definitions(&self, _definitions: &[ChannelPointDefinition]) -> AppResult<()> {
        Err(AppError::not_implemented_error("batch_save_channel_definitions not implemented for SqliteOrmPersistenceService".to_string()))
    }
    async fn batch_save_test_instances(&self, instances: &[ChannelTestInstance]) -> AppResult<()> {
        if instances.is_empty() {
            return Ok(());
        }

        // 逐个保存，使用 save_test_instance 的 upsert 逻辑
        for instance in instances {
            self.save_test_instance(instance).await?;
        }

        Ok(())
    }
    async fn batch_save_test_outcomes(&self, _outcomes: &[RawTestOutcome]) -> AppResult<()> {
        Err(AppError::not_implemented_error("batch_save_test_outcomes not implemented for SqliteOrmPersistenceService".to_string()))
    }
    async fn batch_delete_by_ids(&self, _entity_type: &str, _ids: &[String]) -> AppResult<()> {
        Err(AppError::not_implemented_error("batch_delete_by_ids not implemented for SqliteOrmPersistenceService".to_string()))
    }
    async fn cleanup_old_backups(&self) -> AppResult<u32> {
        Err(AppError::not_implemented_error("cleanup_old_backups not implemented for SqliteOrmPersistenceService".to_string()))
    }
    async fn verify_data_integrity(&self) -> AppResult<IntegrityReport> {
        // 可以提供一个简单的实现，例如检查数据库连接是否健康
        self.health_check().await?; // 复用基础健康检查
        Ok(IntegrityReport {
            checked_at: Utc::now(),
            overall_status: IntegrityStatus::Good, // 假设连接健康则数据良好
            details: vec![IntegrityCheckResult {
                check_name: "Database Connection".to_string(),
                status: IntegrityStatus::Good,
                message: "数据库连接健康".to_string(),
                details: None,
                affected_items: Vec::new(),
            }],
            issues_count: 0,
            repair_suggestions: Vec::new(),
        })
    }
    async fn get_statistics(&self) -> AppResult<PersistenceStats> {
        let db = self.db_conn.as_ref();

        let channel_definitions_count = entities::channel_point_definition::Entity::find()
            .count(db)
            .await
            .map_err(|e| AppError::persistence_error(format!("统计通道定义失败: {}", e)))? as usize;

        let test_instances_count = entities::channel_test_instance::Entity::find()
            .count(db)
            .await
            .map_err(|e| AppError::persistence_error(format!("统计测试实例失败: {}", e)))? as usize;

        let test_batches_count = entities::test_batch_info::Entity::find()
            .count(db)
            .await
            .map_err(|e| AppError::persistence_error(format!("统计测试批次失败: {}", e)))? as usize;

        let test_outcomes_count = entities::raw_test_outcome::Entity::find()
            .count(db)
            .await
            .map_err(|e| AppError::persistence_error(format!("统计测试结果失败: {}", e)))? as usize;

        // 对于内存数据库，total_storage_size_bytes 通常为 0 或难以精确计算。
        // 如果是文件数据库，可以通过 self.db_file_path 获取文件大小。
        let total_storage_size_bytes = if self.db_file_path.to_str() == Some(":memory:") {
            0
        } else {
            match tokio::fs::metadata(&self.db_file_path).await {
                Ok(meta) => meta.len(),
                Err(e) => {
                    log::warn!("获取数据库文件大小失败 {:?}: {}", self.db_file_path, e);
                    0 // 或者返回一个错误？但统计信息通常不应因此失败
                }
            }
        };

        // last_backup_time 和 last_integrity_check_time 暂时为 None
        Ok(PersistenceStats {
            channel_definitions_count,
            test_instances_count,
            test_batches_count,
            test_outcomes_count,
            total_storage_size_bytes,
            last_backup_time: None,
            last_integrity_check_time: None,
        })
    }
    fn get_config(&self) -> &PersistenceConfig {
        &self.config
    }
    async fn update_config(&mut self, _config: PersistenceConfig) -> AppResult<()> {
        Err(AppError::not_implemented_error("update_config not implemented for SqliteOrmPersistenceService".to_string()))
    }
    async fn cleanup_expired_data(&self, _retention_days: u32) -> AppResult<u32> {
        Err(AppError::not_implemented_error("cleanup_expired_data not implemented for SqliteOrmPersistenceService".to_string()))
    }
    async fn compact_storage(&self) -> AppResult<u64> {
        // 对于 SQLite，可以使用 VACUUM 命令
        // self.db_conn.execute_unprepared("VACUUM;").await.map_err(|e| AppError::db_error(e.to_string()))?;
        // Ok(0) // VACUUM 不直接返回释放的空间
        Err(AppError::not_implemented_error("compact_storage (VACUUM) not fully implemented for SqliteOrmPersistenceService".to_string()))
    }
    async fn rebuild_indexes(&self) -> AppResult<()> {
        Err(AppError::not_implemented_error("rebuild_indexes not implemented for SqliteOrmPersistenceService".to_string()))
    }

    fn get_database_connection(&self) -> sea_orm::DatabaseConnection {
        (*self.db_conn).clone()
    }

            fn as_persistence_service(&self) -> Arc<dyn PersistenceService> {
        Arc::new(self.clone())
    }
}
