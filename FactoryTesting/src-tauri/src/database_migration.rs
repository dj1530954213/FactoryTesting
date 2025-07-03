use sea_orm::{DatabaseConnection, Statement, ConnectionTrait};
use crate::error::AppError;

/// 数据库迁移管理器
///
/// 负责管理数据库结构的版本升级和迁移
/// 支持从旧版本数据库结构迁移到新的重构后结构
pub struct DatabaseMigration;

impl DatabaseMigration {
    /// 执行所有必要的数据库迁移
    pub async fn migrate(db: &DatabaseConnection) -> Result<(), AppError> {
        log::info!("开始执行数据库迁移...");

        // 阶段一：数据模型重构迁移
        Self::migrate_channel_point_definitions(db).await?;
        Self::migrate_channel_test_instances(db).await?;
        Self::migrate_test_batch_info(db).await?;

        // 阶段二：创建新表（如果不存在）
        Self::migrate_raw_test_outcomes(db).await?;
        Self::migrate_allocation_records(db).await?;
        Self::create_missing_tables(db).await?;

        // 补充：PLC连接配置表新增字节顺序与地址基数列
        Self::add_plc_connection_config_columns(db).await?;

        // 阶段三：数据完整性检查和修复
        Self::verify_data_integrity(db).await?;

        // 🔥 阶段四：数据恢复 - 为没有batch_id的通道定义恢复批次关联
        Self::recover_missing_batch_associations(db).await?;

        log::info!("数据库迁移完成");
        Ok(())
    }

    /// 迁移通道点位定义表
    async fn migrate_channel_point_definitions(db: &DatabaseConnection) -> Result<(), AppError> {
        log::info!("开始迁移channel_point_definitions表...");

        let table_exists = Self::check_table_exists(db, "channel_point_definitions").await?;

        if !table_exists {
            // 表不存在，创建新表
            log::info!("channel_point_definitions表不存在，创建新表");
            Self::create_channel_point_definitions_table(db).await?;
        } else {
            // 表存在，检查并添加缺失的列，保留现有数据
            log::info!("channel_point_definitions表已存在，检查并添加缺失的列");
            Self::add_channel_point_definition_columns(db).await?;

            // 检查数据完整性
            let count_result = db.query_all(Statement::from_string(
                sea_orm::DatabaseBackend::Sqlite,
                "SELECT COUNT(*) as count FROM channel_point_definitions".to_string()
            )).await.map_err(|e| AppError::persistence_error(format!("查询通道定义数量失败: {}", e)))?;

            if let Some(row) = count_result.first() {
                if let Ok(count) = row.try_get::<i64>("", "count") {
                    log::info!("channel_point_definitions表中现有{}条记录，数据已保留", count);
                }
            }
        }

        log::info!("channel_point_definitions表迁移完成");
        Ok(())
    }

    /// 为channel_point_definitions表添加缺失的列
    async fn add_channel_point_definition_columns(db: &DatabaseConnection) -> Result<(), AppError> {
        log::info!("检查并添加channel_point_definitions表的缺失列...");

        let existing_columns = Self::get_existing_columns(db, "channel_point_definitions").await?;

        // 需要添加的新列（包括batch_id）
        let new_columns = vec![
            ("batch_id", "TEXT"), // 🔥 关键修复：添加批次ID字段
            ("sll_set_point_plc_address", "TEXT"),
            ("sll_feedback_plc_address", "TEXT"),
            ("sl_set_point_plc_address", "TEXT"),
            ("sl_feedback_plc_address", "TEXT"),
            ("sh_set_point_plc_address", "TEXT"),
            ("sh_feedback_plc_address", "TEXT"),
            ("shh_set_point_plc_address", "TEXT"),
            ("shh_feedback_plc_address", "TEXT"),
            ("maintenance_value_set_point_plc_address", "TEXT"),
            ("maintenance_enable_switch_point_plc_address", "TEXT"),
            ("created_time", "TEXT"),
            ("updated_time", "TEXT"),
        ];

        for (column_name, column_def) in new_columns {
            if !existing_columns.contains(&column_name.to_string()) {
                log::info!("添加{}列到channel_point_definitions表", column_name);
                let sql = format!("ALTER TABLE channel_point_definitions ADD COLUMN {} {}", column_name, column_def);
                db.execute(Statement::from_string(
                    sea_orm::DatabaseBackend::Sqlite,
                    sql
                )).await.map_err(|e| AppError::persistence_error(format!("添加{}列失败: {}", column_name, e)))?;

                // 为时间戳列设置默认值
                if column_name == "created_time" || column_name == "updated_time" {
                    let update_sql = format!(
                        "UPDATE channel_point_definitions SET {} = datetime('now') WHERE {} IS NULL",
                        column_name, column_name
                    );
                    db.execute(Statement::from_string(
                        sea_orm::DatabaseBackend::Sqlite,
                        update_sql
                    )).await.map_err(|e| AppError::persistence_error(format!("更新{}默认值失败: {}", column_name, e)))?;
                }
            }
        }

        log::info!("✅ channel_point_definitions表列检查和添加完成");
        Ok(())
    }

    /// 迁移通道测试实例表
    async fn migrate_channel_test_instances(db: &DatabaseConnection) -> Result<(), AppError> {
        log::info!("开始迁移channel_test_instances表...");

        let table_exists = Self::check_table_exists(db, "channel_test_instances").await?;

        if !table_exists {
            Self::create_channel_test_instances_table(db).await?;
        } else {
            Self::add_channel_test_instance_columns(db).await?;
            // 修复旧的时间字段问题
            Self::fix_channel_test_instances_time_fields(db).await?;
        }

        log::info!("channel_test_instances表迁移完成");
        Ok(())
    }

    /// 迁移测试批次信息表
    async fn migrate_test_batch_info(db: &DatabaseConnection) -> Result<(), AppError> {
        log::info!("开始迁移test_batch_info表...");

        let table_exists = Self::check_table_exists(db, "test_batch_info").await?;

        if !table_exists {
            Self::create_test_batch_info_table(db).await?;
        } else {
            Self::add_test_batch_info_columns(db).await?;
            // 修复旧的时间字段问题
            Self::fix_test_batch_info_time_fields(db).await?;
        }

        log::info!("test_batch_info表迁移完成");
        Ok(())
    }

    /// 迁移原始测试结果表
    async fn migrate_raw_test_outcomes(db: &DatabaseConnection) -> Result<(), AppError> {
        log::info!("开始迁移raw_test_outcomes表...");

        let table_exists = Self::check_table_exists(db, "raw_test_outcomes").await?;

        if !table_exists {
            Self::create_raw_test_outcomes_table(db).await?;
        } else {
            Self::add_raw_test_outcomes_columns(db).await?;
        }

        log::info!("raw_test_outcomes表迁移完成");
        Ok(())
    }

    /// 迁移批次分配记录表
    async fn migrate_allocation_records(db: &DatabaseConnection) -> Result<(), AppError> {
        log::info!("开始迁移allocation_records表...");

        let table_exists = Self::check_table_exists(db, "allocation_records").await?;

        if !table_exists {
            Self::create_allocation_records_table(db).await?;
        } else {
            // 如需添加新列可在此处实现
        }

        log::info!("allocation_records表迁移完成");
        Ok(())
    }

    /// 创建批次分配记录表
    async fn create_allocation_records_table(db: &DatabaseConnection) -> Result<(), AppError> {
        log::info!("创建allocation_records表");

        let sql = r#"
            CREATE TABLE IF NOT EXISTS allocation_records (
                id TEXT PRIMARY KEY NOT NULL,
                batch_id TEXT NOT NULL,
                strategy TEXT,
                summary_json TEXT,
                operator_name TEXT,
                created_time TEXT NOT NULL
            )
        "#;

        db.execute(Statement::from_string(
            sea_orm::DatabaseBackend::Sqlite,
            sql.to_string()
        )).await.map_err(|e| AppError::persistence_error(format!("创建allocation_records表失败: {}", e)))?;

        Ok(())
    }

    /// 检查表是否存在
    async fn check_table_exists(db: &DatabaseConnection, table_name: &str) -> Result<bool, AppError> {
        let sql = "SELECT name FROM sqlite_master WHERE type='table' AND name=?";
        let result = db.query_all(Statement::from_sql_and_values(
            sea_orm::DatabaseBackend::Sqlite,
            sql,
            vec![table_name.into()]
        )).await.map_err(|e| AppError::persistence_error(format!("检查表是否存在失败: {}", e)))?;

        Ok(!result.is_empty())
    }

    /// 获取表的现有列
    async fn get_existing_columns(db: &DatabaseConnection, table_name: &str) -> Result<std::collections::HashSet<String>, AppError> {
        let sql = format!("PRAGMA table_info({})", table_name);
        let result = db.query_all(Statement::from_string(
            sea_orm::DatabaseBackend::Sqlite,
            sql
        )).await.map_err(|e| AppError::persistence_error(format!("获取表结构失败: {}", e)))?;

        let mut columns = std::collections::HashSet::new();
        for row in result {
            if let Ok(column_name) = row.try_get::<String>("", "name") {
                columns.insert(column_name);
            }
        }

        Ok(columns)
    }

    /// 创建通道点位定义表
    async fn create_channel_point_definitions_table(db: &DatabaseConnection) -> Result<(), AppError> {
        log::info!("创建channel_point_definitions表");

        let sql = r#"
            CREATE TABLE IF NOT EXISTS channel_point_definitions (
                id TEXT PRIMARY KEY NOT NULL,
                batch_id TEXT,
                sequence_number INTEGER,
                module_name TEXT,
                module_type TEXT NOT NULL,
                power_supply_type TEXT NOT NULL,
                wire_system TEXT,
                channel_position TEXT NOT NULL,
                tag TEXT NOT NULL,
                station_name TEXT,
                variable_name TEXT NOT NULL,
                variable_description TEXT,
                data_type TEXT,
                read_write_property TEXT,
                save_history TEXT,
                power_off_protection TEXT,
                range_low_limit REAL,
                range_high_limit REAL,
                sll_set_value REAL,
                sll_set_point TEXT,
                sll_set_point_plc_address TEXT,
                sll_set_point_communication_address TEXT,
                sl_set_value REAL,
                sl_set_point TEXT,
                sl_set_point_plc_address TEXT,
                sl_set_point_communication_address TEXT,
                sh_set_value REAL,
                sh_set_point TEXT,
                sh_set_point_plc_address TEXT,
                sh_set_point_communication_address TEXT,
                shh_set_value REAL,
                shh_set_point TEXT,
                shh_set_point_plc_address TEXT,
                shh_set_point_communication_address TEXT,
                ll_alarm TEXT,
                ll_alarm_plc_address TEXT,
                ll_alarm_communication_address TEXT,
                l_alarm TEXT,
                l_alarm_plc_address TEXT,
                l_alarm_communication_address TEXT,
                h_alarm TEXT,
                h_alarm_plc_address TEXT,
                h_alarm_communication_address TEXT,
                hh_alarm TEXT,
                hh_alarm_plc_address TEXT,
                hh_alarm_communication_address TEXT,
                maintenance_value_setting TEXT,
                maintenance_value_set_point TEXT,
                maintenance_value_set_point_plc_address TEXT,
                maintenance_value_set_point_communication_address TEXT,
                maintenance_enable_switch_point TEXT,
                maintenance_enable_switch_point_plc_address TEXT,
                maintenance_enable_switch_point_communication_address TEXT,
                plc_absolute_address TEXT,
                plc_communication_address TEXT NOT NULL,
                created_time TEXT NOT NULL,
                updated_time TEXT NOT NULL
            )
        "#;

        db.execute(Statement::from_string(
            sea_orm::DatabaseBackend::Sqlite,
            sql.to_string()
        )).await.map_err(|e| AppError::persistence_error(format!("创建channel_point_definitions表失败: {}", e)))?;

        // 兼容旧库：若缺少 sequence_number 列则补充
        let columns = db
            .query_all(Statement::from_string(
                db.get_database_backend(),
                "PRAGMA table_info(channel_point_definitions);".to_string(),
            ))
            .await
            .map_err(|e| AppError::persistence_error(format!("获取表结构失败: {}", e)))?;

        let has_seq_col = columns.iter().any(|column| {
            let name: String = column.try_get("", "name").unwrap_or_default();
            name == "sequence_number"
        });

        if !has_seq_col {
            db.execute(Statement::from_string(
                db.get_database_backend(),
                "ALTER TABLE channel_point_definitions ADD COLUMN sequence_number INTEGER;".to_string(),
            ))
            .await
            .map_err(|e| AppError::persistence_error(format!("添加sequence_number列失败: {}", e)))?;
            log::info!("数据库已添加 sequence_number 列");
        }

        log::info!("成功创建channel_point_definitions表");
        Ok(())
    }

    /// 创建通道测试实例表
    async fn create_channel_test_instances_table(db: &DatabaseConnection) -> Result<(), AppError> {
        log::info!("创建channel_test_instances表");

        let sql = r#"
            CREATE TABLE IF NOT EXISTS channel_test_instances (
                instance_id TEXT PRIMARY KEY NOT NULL,
                definition_id TEXT NOT NULL,
                test_batch_id TEXT NOT NULL,
                test_batch_name TEXT NOT NULL,
                channel_tag TEXT NOT NULL,
                variable_name TEXT NOT NULL,
                variable_description TEXT NOT NULL,
                module_type TEXT NOT NULL,
                data_type TEXT NOT NULL,
                plc_communication_address TEXT NOT NULL,
                overall_status TEXT NOT NULL,
                current_step_details TEXT,
                error_message TEXT,
                created_time TEXT NOT NULL,
                start_time TEXT,
                updated_time TEXT NOT NULL,
                final_test_time TEXT,
                total_test_duration_ms INTEGER,
                hard_point_status INTEGER,
                hard_point_test_result TEXT,
                hard_point_error_detail TEXT,
                actual_value TEXT,
                expected_value TEXT,
                current_value TEXT,
                low_low_alarm_status INTEGER,
                low_alarm_status INTEGER,
                high_alarm_status INTEGER,
                high_high_alarm_status INTEGER,
                maintenance_function INTEGER,
                show_value_status INTEGER,
                test_plc_channel_tag TEXT,
                test_plc_communication_address TEXT,
                test_result_status INTEGER,
                test_result_0_percent REAL,
                test_result_25_percent REAL,
                test_result_50_percent REAL,
                test_result_75_percent REAL,
                test_result_100_percent REAL,
                current_operator TEXT,
                retries_count INTEGER DEFAULT 0,
                sub_test_results_json TEXT,
                hardpoint_readings_json TEXT,
                digital_test_steps_json TEXT,
                transient_data_json TEXT
            )
        "#;

        db.execute(Statement::from_string(
            sea_orm::DatabaseBackend::Sqlite,
            sql.to_string()
        )).await.map_err(|e| AppError::persistence_error(format!("创建channel_test_instances表失败: {}", e)))?;

        log::info!("成功创建channel_test_instances表");
        Ok(())
    }

    /// 添加通道测试实例表的新列
    async fn add_channel_test_instance_columns(db: &DatabaseConnection) -> Result<(), AppError> {
        let existing_columns = Self::get_existing_columns(db, "channel_test_instances").await?;

        // 需要添加的新列（基于重构后的实体结构）
        let new_columns = vec![
            ("test_batch_name", "TEXT NOT NULL DEFAULT ''"),
            ("channel_tag", "TEXT NOT NULL DEFAULT ''"),
            ("variable_name", "TEXT NOT NULL DEFAULT ''"),
            ("variable_description", "TEXT NOT NULL DEFAULT ''"),
            ("module_type", "TEXT NOT NULL DEFAULT ''"),
            ("data_type", "TEXT NOT NULL DEFAULT ''"),
            ("plc_communication_address", "TEXT NOT NULL DEFAULT ''"),
            ("current_step_details", "TEXT"),
            ("error_message", "TEXT"),
            ("start_time", "TEXT"),
            ("final_test_time", "TEXT"),
            ("total_test_duration_ms", "INTEGER"),
            ("hard_point_status", "INTEGER"),
            ("hard_point_test_result", "TEXT"),
            ("hard_point_error_detail", "TEXT"),
            ("actual_value", "TEXT"),
            ("expected_value", "TEXT"),
            ("current_value", "TEXT"),
            ("low_low_alarm_status", "INTEGER"),
            ("low_alarm_status", "INTEGER"),
            ("high_alarm_status", "INTEGER"),
            ("high_high_alarm_status", "INTEGER"),
            ("maintenance_function", "INTEGER"),
            ("show_value_status", "INTEGER"),
            ("test_plc_channel_tag", "TEXT"),
            ("test_plc_communication_address", "TEXT"),
            ("test_result_status", "INTEGER"),
            ("current_operator", "TEXT"),
            ("retries_count", "INTEGER DEFAULT 0"),
            ("test_result_0_percent", "REAL"),
            ("test_result_25_percent", "REAL"),
            ("test_result_50_percent", "REAL"),
            ("test_result_75_percent", "REAL"),
            ("test_result_100_percent", "REAL"),
            ("created_time", "TEXT"),
            ("updated_time", "TEXT"),
            ("sub_test_results_json", "TEXT"),
            ("hardpoint_readings_json", "TEXT"),
            ("digital_test_steps_json", "TEXT"),
            ("transient_data_json", "TEXT"),
        ];

        for (column_name, column_def) in new_columns {
            if !existing_columns.contains(&column_name.to_string()) {
                log::info!("添加{}列到channel_test_instances表", column_name);
                let sql = format!("ALTER TABLE channel_test_instances ADD COLUMN {} {}", column_name, column_def);
                db.execute(Statement::from_string(
                    sea_orm::DatabaseBackend::Sqlite,
                    sql
                )).await.map_err(|e| AppError::persistence_error(format!("添加{}列失败: {}", column_name, e)))?;

                // 为时间戳列设置默认值
                if column_name == "created_time" || column_name == "updated_time" {
                    let update_sql = format!(
                        "UPDATE channel_test_instances SET {} = datetime('now') WHERE {} IS NULL",
                        column_name, column_name
                    );
                    db.execute(Statement::from_string(
                        sea_orm::DatabaseBackend::Sqlite,
                        update_sql
                    )).await.map_err(|e| AppError::persistence_error(format!("更新{}默认值失败: {}", column_name, e)))?;
                }
            }
        }

        // 🚜 移除已废弃的列（trend_check, report_check）
        let obsolete_columns = vec!["trend_check", "report_check"];
        for column in &obsolete_columns {
            if existing_columns.contains(&column.to_string()) {
                log::info!("移除已废弃列{}从channel_test_instances表", column);
                let sql = format!("ALTER TABLE channel_test_instances DROP COLUMN {}", column);
                // 由于SQLite 3.35+才支持DROP COLUMN，如果失败则记录警告并继续
                if let Err(e) = db.execute(Statement::from_string(
                    sea_orm::DatabaseBackend::Sqlite,
                    sql,
                )).await {
                    log::warn!("删除列{}失败: {} (可能SQLite版本过旧，或列已被其他对象依赖)", column, e);
                }
            }
        }

        Ok(())
    }

    /// 创建测试批次信息表
    async fn create_test_batch_info_table(db: &DatabaseConnection) -> Result<(), AppError> {
        log::info!("创建test_batch_info表");

        let sql = r#"
            CREATE TABLE IF NOT EXISTS test_batch_info (
                batch_id TEXT PRIMARY KEY NOT NULL,
                batch_name TEXT NOT NULL,
                product_model TEXT,
                serial_number TEXT,
                customer_name TEXT,
                station_name TEXT,
                created_time TEXT NOT NULL,
                updated_time TEXT NOT NULL,
                start_time TEXT,
                end_time TEXT,
                total_duration_ms INTEGER,
                operator_name TEXT,
                created_by TEXT,
                overall_status TEXT NOT NULL,
                status_summary TEXT,
                error_message TEXT,
                total_points INTEGER DEFAULT 0,
                tested_points INTEGER DEFAULT 0,
                passed_points INTEGER DEFAULT 0,
                failed_points INTEGER DEFAULT 0,
                skipped_points INTEGER DEFAULT 0,
                not_tested_points INTEGER DEFAULT 0,
                progress_percentage REAL DEFAULT 0.0,
                current_testing_channel TEXT,
                test_configuration TEXT,
                import_source TEXT,
                custom_data_json TEXT
            )
        "#;

        db.execute(Statement::from_string(
            sea_orm::DatabaseBackend::Sqlite,
            sql.to_string()
        )).await.map_err(|e| AppError::persistence_error(format!("创建test_batch_info表失败: {}", e)))?;

        log::info!("成功创建test_batch_info表");
        Ok(())
    }

    /// 添加测试批次信息表的新列
    async fn add_test_batch_info_columns(db: &DatabaseConnection) -> Result<(), AppError> {
        let existing_columns = Self::get_existing_columns(db, "test_batch_info").await?;

        // 需要添加的新列（基于重构后的实体结构）
        let new_columns = vec![
            ("batch_name", "TEXT NOT NULL DEFAULT ''"),
            ("customer_name", "TEXT"),
            ("station_name", "TEXT"),
            ("start_time", "TEXT"),
            ("end_time", "TEXT"),
            ("total_duration_ms", "INTEGER"),
            ("operator_name", "TEXT"),  // 添加这个字段
            ("created_by", "TEXT"),
            ("overall_status", "TEXT NOT NULL DEFAULT 'NotTested'"),
            ("status_summary", "TEXT"),  // 添加这个字段
            ("error_message", "TEXT"),
            ("total_points", "INTEGER DEFAULT 0"),  // 添加这个字段
            ("tested_points", "INTEGER DEFAULT 0"),  // 添加这个字段
            ("passed_points", "INTEGER DEFAULT 0"),  // 添加这个字段
            ("failed_points", "INTEGER DEFAULT 0"),  // 添加这个字段
            ("skipped_points", "INTEGER DEFAULT 0"),  // 添加这个字段
            ("not_tested_points", "INTEGER DEFAULT 0"),
            ("progress_percentage", "REAL DEFAULT 0.0"),
            ("current_testing_channel", "TEXT"),
            ("test_configuration", "TEXT"),
            ("import_source", "TEXT"),
            ("custom_data_json", "TEXT"),
            ("created_time", "TEXT"),
            ("updated_time", "TEXT"),
            ("last_updated_time", "TEXT"),  // 添加这个字段以兼容实体模型
        ];

        for (column_name, column_def) in new_columns {
            if !existing_columns.contains(&column_name.to_string()) {
                log::info!("添加{}列到test_batch_info表", column_name);
                let sql = format!("ALTER TABLE test_batch_info ADD COLUMN {} {}", column_name, column_def);
                db.execute(Statement::from_string(
                    sea_orm::DatabaseBackend::Sqlite,
                    sql
                )).await.map_err(|e| AppError::persistence_error(format!("添加{}列失败: {}", column_name, e)))?;

                // 为时间戳列设置默认值
                if column_name == "created_time" || column_name == "updated_time" {
                    let update_sql = format!(
                        "UPDATE test_batch_info SET {} = datetime('now') WHERE {} IS NULL",
                        column_name, column_name
                    );
                    db.execute(Statement::from_string(
                        sea_orm::DatabaseBackend::Sqlite,
                        update_sql
                    )).await.map_err(|e| AppError::persistence_error(format!("更新{}默认值失败: {}", column_name, e)))?;
                }
            }
        }

        Ok(())
    }

    /// 修复测试批次信息表的时间字段问题
    async fn fix_test_batch_info_time_fields(db: &DatabaseConnection) -> Result<(), AppError> {
        log::info!("修复test_batch_info表的时间字段...");

        // 检查是否存在旧的creation_time字段
        let existing_columns = Self::get_existing_columns(db, "test_batch_info").await?;

        if existing_columns.contains(&"creation_time".to_string()) {
            log::info!("发现旧的creation_time字段，开始数据迁移...");

            // 将旧字段的数据复制到新字段
            let migrate_sql = r#"
                UPDATE test_batch_info
                SET created_time = creation_time,
                    updated_time = COALESCE(last_updated_time, creation_time)
                WHERE created_time IS NULL OR updated_time IS NULL
            "#;

            db.execute(Statement::from_string(
                sea_orm::DatabaseBackend::Sqlite,
                migrate_sql.to_string()
            )).await.map_err(|e| AppError::persistence_error(format!("迁移时间字段数据失败: {}", e)))?;

            log::info!("时间字段数据迁移完成");
        }

        Ok(())
    }

    /// 修复通道测试实例表的时间字段问题
    async fn fix_channel_test_instances_time_fields(db: &DatabaseConnection) -> Result<(), AppError> {
        log::info!("修复channel_test_instances表的时间字段...");

        // 检查是否存在旧的creation_time字段
        let existing_columns = Self::get_existing_columns(db, "channel_test_instances").await?;

        if existing_columns.contains(&"creation_time".to_string()) {
            log::info!("发现旧的creation_time字段，开始数据迁移...");

            // 将旧字段的数据复制到新字段
            let migrate_sql = r#"
                UPDATE channel_test_instances
                SET created_time = creation_time,
                    updated_time = COALESCE(last_updated_time, creation_time)
                WHERE created_time IS NULL OR updated_time IS NULL
            "#;

            db.execute(Statement::from_string(
                sea_orm::DatabaseBackend::Sqlite,
                migrate_sql.to_string()
            )).await.map_err(|e| AppError::persistence_error(format!("迁移实例时间字段数据失败: {}", e)))?;

            log::info!("实例时间字段数据迁移完成");
        }

        Ok(())
    }

    /// 创建原始测试结果表
    async fn create_raw_test_outcomes_table(db: &DatabaseConnection) -> Result<(), AppError> {
        log::info!("创建raw_test_outcomes表");

        let sql = r#"
            CREATE TABLE IF NOT EXISTS raw_test_outcomes (
                id TEXT PRIMARY KEY NOT NULL,
                channel_instance_id TEXT NOT NULL,
                sub_test_item TEXT NOT NULL,
                success BOOLEAN NOT NULL,
                raw_value_read TEXT,
                eng_value_calculated TEXT,
                message TEXT,
                start_time TEXT NOT NULL,
                end_time TEXT NOT NULL,
                readings_json TEXT,
                test_result_0_percent REAL,
                test_result_25_percent REAL,
                test_result_50_percent REAL,
                test_result_75_percent REAL,
                test_result_100_percent REAL,
                details_json TEXT
            )
        "#;

        db.execute(Statement::from_string(
            sea_orm::DatabaseBackend::Sqlite,
            sql.to_string()
        )).await.map_err(|e| AppError::persistence_error(format!("创建raw_test_outcomes表失败: {}", e)))?;

        log::info!("成功创建raw_test_outcomes表");
        Ok(())
    }

    /// 添加原始测试结果表的新列
    async fn add_raw_test_outcomes_columns(db: &DatabaseConnection) -> Result<(), AppError> {
        let existing_columns = Self::get_existing_columns(db, "raw_test_outcomes").await?;

        // 需要添加的新列
        let new_columns = vec![
            ("test_result_0_percent", "REAL"),
            ("test_result_25_percent", "REAL"),
            ("test_result_50_percent", "REAL"),
            ("test_result_75_percent", "REAL"),
            ("test_result_100_percent", "REAL"),
        ];

        for (column_name, column_def) in new_columns {
            if !existing_columns.contains(&column_name.to_string()) {
                log::info!("添加{}列到raw_test_outcomes表", column_name);
                let sql = format!("ALTER TABLE raw_test_outcomes ADD COLUMN {} {}", column_name, column_def);
                db.execute(Statement::from_string(
                    sea_orm::DatabaseBackend::Sqlite,
                    sql
                )).await.map_err(|e| AppError::persistence_error(format!("添加{}列失败: {}", column_name, e)))?;
            }
        }

        Ok(())
    }

    /// 创建缺失的表
    async fn create_missing_tables(db: &DatabaseConnection) -> Result<(), AppError> {
        log::info!("检查并创建缺失的表...");

        // 这里可以添加其他需要创建的表
        // 例如：测试配置表、PLC连接配置表等

        Ok(())
    }

    /// 验证数据完整性
    async fn verify_data_integrity(db: &DatabaseConnection) -> Result<(), AppError> {
        log::info!("验证数据完整性...");

        // 检查关键表是否存在
        let tables = vec!["channel_point_definitions", "channel_test_instances", "test_batch_info"];

        for table in tables {
            let exists = Self::check_table_exists(db, table).await?;
            if !exists {
                return Err(AppError::persistence_error(format!("关键表{}不存在", table)));
            }
        }

        log::info!("数据完整性验证通过");
        Ok(())
    }

    /// 🔥 数据恢复：为没有batch_id的通道定义恢复批次关联
    ///
    /// 这个方法会：
    /// 1. 查找所有没有batch_id的通道定义
    /// 2. 尝试通过测试实例找到对应的批次ID
    /// 3. 更新通道定义的batch_id字段
    async fn recover_missing_batch_associations(db: &DatabaseConnection) -> Result<(), AppError> {
        log::info!("🔄 开始数据恢复：为缺失batch_id的通道定义恢复批次关联");

        // 1. 查找所有没有batch_id的通道定义
        let orphaned_definitions_sql = r#"
            SELECT id, tag, station_name
            FROM channel_point_definitions
            WHERE batch_id IS NULL OR batch_id = ''
        "#;

        let orphaned_definitions = db.query_all(Statement::from_string(
            sea_orm::DatabaseBackend::Sqlite,
            orphaned_definitions_sql.to_string()
        )).await.map_err(|e| AppError::persistence_error(format!("查询孤立通道定义失败: {}", e)))?;

        if orphaned_definitions.is_empty() {
            log::info!("✅ 没有发现缺失batch_id的通道定义，跳过数据恢复");
            return Ok(());
        }

        log::info!("🔍 发现{}个缺失batch_id的通道定义，开始恢复", orphaned_definitions.len());

        let mut recovered_count = 0;
        let mut failed_count = 0;

        // 2. 为每个孤立的通道定义尝试恢复批次关联
        for row in orphaned_definitions {
            let definition_id = row.try_get::<String>("", "id")
                .map_err(|e| AppError::persistence_error(format!("获取定义ID失败: {}", e)))?;
            let tag = row.try_get::<String>("", "tag").unwrap_or_default();
            let station_name = row.try_get::<String>("", "station_name").unwrap_or_default();

            // 尝试通过测试实例找到对应的批次ID
            match Self::find_batch_id_for_definition(db, &definition_id).await {
                Ok(Some(batch_id)) => {
                    // 找到了批次ID，更新通道定义
                    match Self::update_definition_batch_id(db, &definition_id, &batch_id).await {
                        Ok(_) => {
                            recovered_count += 1;
                        }
                        Err(e) => {
                            log::warn!("❌ 更新通道定义 {} 的批次ID失败: {}", tag, e);
                            failed_count += 1;
                        }
                    }
                }
                Ok(None) => {
                    // 🔧 修复：不再自动创建默认批次，只记录孤立的通道定义
                    log::debug!("🔍 发现孤立通道定义: {} ({}), 跳过自动批次创建", tag, definition_id);
                    failed_count += 1; // 计入失败数，但不尝试创建
                }
                Err(e) => {
                    log::warn!("❌ 查找通道定义 {} 的批次ID失败: {}", tag, e);
                    failed_count += 1;
                }
            }
        }

        log::info!("🎉 数据恢复完成: 成功恢复{}个，失败{}个", recovered_count, failed_count);
        Ok(())
    }

    /// 通过测试实例查找通道定义对应的批次ID
    async fn find_batch_id_for_definition(db: &DatabaseConnection, definition_id: &str) -> Result<Option<String>, AppError> {
        let sql = r#"
            SELECT test_batch_id
            FROM channel_test_instances
            WHERE definition_id = ?
            LIMIT 1
        "#;

        let result = db.query_all(Statement::from_sql_and_values(
            sea_orm::DatabaseBackend::Sqlite,
            sql,
            vec![definition_id.into()]
        )).await.map_err(|e| AppError::persistence_error(format!("查询测试实例失败: {}", e)))?;

        if let Some(row) = result.first() {
            let batch_id = row.try_get::<String>("", "test_batch_id")
                .map_err(|e| AppError::persistence_error(format!("获取批次ID失败: {}", e)))?;
            Ok(Some(batch_id))
        } else {
            Ok(None)
        }
    }

    /// 更新通道定义的批次ID
    async fn update_definition_batch_id(db: &DatabaseConnection, definition_id: &str, batch_id: &str) -> Result<(), AppError> {
        let sql = r#"
            UPDATE channel_point_definitions
            SET batch_id = ?
            WHERE id = ?
        "#;

        db.execute(Statement::from_sql_and_values(
            sea_orm::DatabaseBackend::Sqlite,
            sql,
            vec![batch_id.into(), definition_id.into()]
        )).await.map_err(|e| AppError::persistence_error(format!("更新批次ID失败: {}", e)))?;

        Ok(())
    }

    /// 为孤立的通道定义创建默认批次
    async fn create_default_batch_for_orphaned_definition(
        db: &DatabaseConnection,
        definition_id: &str,
        tag: &str,
        station_name: &str
    ) -> Result<String, AppError> {
        use uuid::Uuid;
        use chrono::Utc;

        let batch_id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();

        // 创建默认批次名称
        let batch_name = if !station_name.is_empty() {
            format!("历史数据恢复-{}", station_name)
        } else {
            "历史数据恢复-未知站场".to_string()
        };

        // 插入默认批次信息
        let insert_batch_sql = r#"
            INSERT INTO test_batch_info (
                batch_id, batch_name, station_name, created_time, updated_time,
                overall_status, total_points, tested_points, passed_points,
                failed_points, skipped_points
            ) VALUES (?, ?, ?, ?, ?, 'NotTested', 1, 0, 0, 0, 1)
        "#;

        db.execute(Statement::from_sql_and_values(
            sea_orm::DatabaseBackend::Sqlite,
            insert_batch_sql,
            vec![
                batch_id.clone().into(),
                batch_name.into(),
                station_name.into(),
                now.clone().into(),
                now.into(),
            ]
        )).await.map_err(|e| AppError::persistence_error(format!("创建默认批次失败: {}", e)))?;

        // 更新通道定义的批次ID
        Self::update_definition_batch_id(db, definition_id, &batch_id).await?;

        Ok(batch_id)
    }

    /// 为plc_connection_configs表添加缺失列
    async fn add_plc_connection_config_columns(db: &DatabaseConnection) -> Result<(), AppError> {
        log::info!("检查并添加plc_connection_configs表缺失列...");

        let table_exists = Self::check_table_exists(db, "plc_connection_configs").await?;
        if !table_exists {
            // 表不存在时，新建由SeaORM迁移器处理，这里直接返回
            log::warn!("plc_connection_configs表不存在，跳过列检查");
            return Ok(());
        }

        let existing_columns = Self::get_existing_columns(db, "plc_connection_configs").await?;

        let new_columns = vec![
            ("byte_order", "TEXT DEFAULT 'CDAB'"),
            ("zero_based_address", "INTEGER DEFAULT 0"),
        ];

        for (column_name, column_def) in new_columns {
            if !existing_columns.contains(&column_name.to_string()) {
                log::info!("添加{}列到plc_connection_configs表", column_name);
                let sql = format!("ALTER TABLE plc_connection_configs ADD COLUMN {} {}", column_name, column_def);
                db.execute(Statement::from_string(sea_orm::DatabaseBackend::Sqlite, sql))
                    .await
                    .map_err(|e| AppError::persistence_error(format!("添加列{}失败: {}", column_name, e)))?;
            }
        }

        log::info!("✅ plc_connection_configs表列检查完成");
        Ok(())
    }
}
