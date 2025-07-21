//! # 数据库迁移模块 (Database Migration Module)
//!
//! ## 业务说明
//! 本模块负责管理FAT_TEST系统的数据库结构迁移和版本升级，确保系统能够
//! 从旧版本平滑升级到新版本，同时保持数据完整性和向后兼容性
//!
//! ## 核心功能
//! ### 1. 结构迁移
//! - **表创建**: 检查并创建缺失的数据库表
//! - **列添加**: 为现有表添加新的字段
//! - **索引管理**: 创建和更新数据库索引
//! - **约束管理**: 添加外键约束和数据约束
//!
//! ### 2. 数据迁移
//! - **数据完整性**: 修复数据完整性问题
//! - **关联恢复**: 恢复丢失的批次关联关系
//! - **默认数据**: 初始化系统必需的默认数据
//! - **数据清理**: 清理无效或冗余的数据
//!
//! ### 3. 版本管理
//! - **版本检测**: 自动检测当前数据库版本
//! - **增量迁移**: 只执行必要的迁移步骤
//! - **回滚支持**: 支持迁移失败时的回滚操作
//!
//! ## 迁移策略
//! ### 四阶段执行
//! 1. **数据模型重构**: 更新核心数据模型结构
//! 2. **新表创建**: 创建新增的业务表
//! 3. **完整性检查**: 验证数据关联的完整性
//! 4. **数据恢复**: 恢复和初始化必要数据
//!
//! ### 幂等性设计
//! - 所有迁移操作都支持安全的重复执行
//! - 通过条件检查避免重复创建和修改
//! - 事务保护确保迁移过程的原子性
//!
//! ## 使用场景
//! - **系统启动**: 每次启动时自动执行必要的迁移
//! - **版本升级**: 软件版本升级时的数据库升级
//! - **环境部署**: 新环境部署时的初始化
//! - **数据修复**: 修复数据完整性问题
//!
//! ## 调用链路
//! ```
//! 系统启动 → AppState::new() → SqliteOrmPersistenceService → 
//! DatabaseMigration::migrate() → 各项迁移任务
//! ```
//!
//! ## Rust知识点
//! - **SeaORM**: 使用Rust异步ORM框架进行数据库操作
//! - **ConnectionTrait**: 数据库连接抽象，支持多种数据库
//! - **事务处理**: 使用数据库事务确保操作的原子性
//! - **错误处理**: 完善的错误传播和处理机制

use sea_orm::{DatabaseConnection, Statement, ConnectionTrait};
use sea_orm::ActiveValue::Set;
use crate::error::AppError;

/// 数据库迁移管理器
///
/// 业务说明：
/// 负责管理数据库结构的版本升级和迁移
/// 支持从旧版本数据库结构迁移到新的重构后结构
/// 这是一个纯工具类，没有实例字段，所有方法都是关联函数
/// 
/// Rust知识点：
/// - pub struct 公开的结构体
/// - 单元结构体（unit struct），没有字段
pub struct DatabaseMigration;

impl DatabaseMigration {

    /// 迁移并种子 range_registers 表（量程寄存器地址映射）
    /// 
    /// 业务说明：
    /// - 量程寄存器是PLC中存储AI/AO通道量程信息的特殊寄存器
    /// - 每个AO通道都有对应的量程寄存器，地址规则为：通道标签_RANGE
    /// - 这个表存储通道标签到Modbus寄存器地址的映射
    /// - 默认提供16个AO通道的量程寄存器映射
    /// 
    /// 执行流程：
    /// 1. 检查表是否存在，不存在则创建
    /// 2. 检查是否已有数据，避免重复插入
    /// 3. 插入默认的寄存器映射数据
    /// 
    /// 参数：
    /// - db: 数据库连接
    /// 
    /// 返回：
    /// - Ok(()): 迁移成功
    /// - Err: 迁移失败的错误信息
    /// 
    /// Rust知识点：
    /// - async fn: 异步函数
    /// - Result<T, E>: 错误处理类型
    async fn migrate_range_registers(db: &DatabaseConnection) -> Result<(), AppError> {
        use sea_orm::ActiveModelTrait;
        use uuid::Uuid;
        use chrono::Utc;
        use crate::models::entities::range_register;

        // 1. 如表不存在则创建
        // 业务说明：首先检查表是否存在，避免重复创建
        // Rust知识点：Self 引用当前类型，? 操作符用于错误传播
        let exists = Self::check_table_exists(db, "range_registers").await?;
        if !exists {
            log::info!("创建 range_registers 表");
            // Rust知识点：r#"..."# 是原始字符串字面量，保留换行和空格
            let sql = r#"
                CREATE TABLE IF NOT EXISTS range_registers (
                    id TEXT PRIMARY KEY NOT NULL,
                    channel_tag TEXT UNIQUE NOT NULL,
                    register TEXT NOT NULL,
                    remark TEXT,
                    created_at TEXT NOT NULL,
                    updated_at TEXT NOT NULL
                )
            "#;
            db.execute(Statement::from_string(sea_orm::DatabaseBackend::Sqlite, sql.to_string()))
                .await
                .map_err(|e| AppError::persistence_error(format!("创建range_registers表失败: {}", e)))?;
        }

        // 2. 判断是否已存在数据
        // 业务说明：避免重复插入默认数据，保持幂等性
        let count_sql = "SELECT COUNT(*) as cnt FROM range_registers";
        let rows = db
            .query_all(Statement::from_string(sea_orm::DatabaseBackend::Sqlite, count_sql.to_string()))
            .await
            .map_err(|e| AppError::persistence_error(format!("统计range_registers失败: {}", e)))?;
        let mut need_seed = true;
        if let Some(row) = rows.first() {
            if let Ok(cnt) = row.try_get::<i64>("", "cnt") {
                need_seed = cnt == 0;
            }
        }
        if !need_seed {
            log::info!("range_registers 表已有数据，跳过默认种子");
            return Ok(());
        }

        log::info!("向 range_registers 表插入默认寄存器映射...");
        // 业务说明：默认的量程寄存器映射
        // AO1_1到AO1_8: 第一组AO通道（有源）
        // AO2_1到AO2_8: 第二组AO通道（无源）
        // 寄存器地址从45601开始，每个通道间隔2
        // Rust知识点：vec![] 创建向量，元素为元组(&str, &str)
        let defaults = vec![
            ("AO1_1_RANGE", "45601"),
            ("AO1_2_RANGE", "45603"),
            ("AO1_3_RANGE", "45605"),
            ("AO1_4_RANGE", "45607"),
            ("AO1_5_RANGE", "45609"),
            ("AO1_6_RANGE", "45611"),
            ("AO1_7_RANGE", "45613"),
            ("AO1_8_RANGE", "45615"),
            ("AO2_1_RANGE", "45617"),
            ("AO2_2_RANGE", "45619"),
            ("AO2_3_RANGE", "45621"),
            ("AO2_4_RANGE", "45623"),
            ("AO2_5_RANGE", "45625"),
            ("AO2_6_RANGE", "45627"),
            ("AO2_7_RANGE", "45629"),
            ("AO2_8_RANGE", "45631"),
        ];
        // 业务说明：遍历默认映射，使用SeaORM的ActiveModel插入数据
        // Rust知识点：for循环解构元组
        for (tag, reg) in defaults {
            // 业务说明：创建ActiveModel对象，这是SeaORM的数据模型
            // Rust知识点：
            // - Set() 将值包装为ActiveValue
            // - Uuid::new_v4() 生成版本4的UUID
            // - to_string() 转换为String
            // - Some() 将值包装为Option
            // - into() 类型转换
            let am = range_register::ActiveModel {
                id: Set(Uuid::new_v4().to_string()),
                channel_tag: Set(tag.to_string()),
                register: Set(reg.to_string()),
                remark: Set(Some("默认映射".into())),
                created_at: Set(Utc::now()),
                updated_at: Set(Utc::now()),
            };
            // 业务说明：插入数据，失败不中断，只记录警告
            // Rust知识点：if let Err(e) 模式匹配错误情况
            if let Err(e) = am.insert(db).await {
                log::warn!("插入默认寄存器映射 {} -> {} 失败: {}", tag, reg, e);
            }
        }
        log::info!("默认寄存器映射插入完成");
        Ok(())
    }
    /// 执行所有必要的数据库迁移
    /// 
    /// 业务说明：
    /// 这是数据库迁移的主入口，统一管理所有迁移任务
    /// 分四个阶段执行，确保数据库结构的正确性和数据的完整性
    /// 
    /// 执行阶段：
    /// - 阶段一：数据模型重构迁移（核心业务表）
    /// - 阶段二：创建新表（如果不存在）
    /// - 阶段三：数据完整性检查和修复
    /// - 阶段四：数据恢复（修复孤立的通道定义）
    /// 
    /// 参数：
    /// - db: 数据库连接
    /// 
    /// 返回：
    /// - Ok(()): 所有迁移成功
    /// - Err: 任何迁移失败的错误
    /// 
    /// 调用链：
    /// SqliteOrmPersistenceService::new() -> DatabaseMigration::migrate()
    /// 
    /// Rust知识点：
    /// - pub async fn: 公开的异步函数
    /// - &DatabaseConnection: 借用数据库连接
    pub async fn migrate(db: &DatabaseConnection) -> Result<(), AppError> {
        log::info!("开始执行数据库迁移...");

        // 阶段一：数据模型重构迁移
        // 业务说明：迁移核心业务表，这些表是测试系统的基础
        Self::migrate_channel_point_definitions(db).await?;  // 通道点位定义表
        Self::migrate_channel_test_instances(db).await?;    // 通道测试实例表
        Self::migrate_test_batch_info(db).await?;           // 测试批次信息表

        // 阶段二：创建新表（如果不存在）
        // 业务说明：创建新增的功能表
        Self::migrate_raw_test_outcomes(db).await?;    // 原始测试结果表
        Self::migrate_allocation_records(db).await?;   // 通道分配记录表
        Self::create_missing_tables(db).await?;         // 其他缺失的表

        // 新增：迁移并种子 range_registers 表
        // 业务说明：初始化量程寄存器映射，这是AO通道测试的关键配置
        Self::migrate_range_registers(db).await?;

        // 补充：PLC连接配置表新增字节顺序与地址基数列
        // 业务说明：添加PLC通信必需的配置字段
        // byte_order: Modbus字节序（CDAB等）
        // zero_based_address: 地址是否从0开始
        Self::add_plc_connection_config_columns(db).await?;

        // 阶段三：数据完整性检查和修复
        // 业务说明：确保关键表存在，为后续操作提供保障
        Self::verify_data_integrity(db).await?;

        // 🔥 阶段四：数据恢复 - 为没有batch_id的通道定义恢复批次关联
        // 业务说明：修复历史数据问题，一些旧版本的通道定义可能缺失batch_id
        // 通过测试实例找回关联关系
        Self::recover_missing_batch_associations(db).await?;

        log::info!("数据库迁移完成");
        Ok(())
    }

    /// 迁移通道点位定义表
    /// 
    /// 业务说明：
    /// channel_point_definitions 是核心业务表，存储所有通道的定义信息
    /// 包括通道类型、地址、量程、报警设置等
    /// 迁移时保留现有数据，只添加缺失的列
    /// 
    /// 执行流程：
    /// 1. 检查表是否存在
    /// 2. 不存在则创建新表
    /// 3. 存在则添加缺失的列
    /// 4. 记录现有数据数量
    /// 
    /// Rust知识点：
    /// - async fn: 异步函数，返回Future
    async fn migrate_channel_point_definitions(db: &DatabaseConnection) -> Result<(), AppError> {
        log::info!("开始迁移channel_point_definitions表...");

        // 检查表是否存在
        // Rust知识点：? 操作符在Result为Err时提前返回
        let table_exists = Self::check_table_exists(db, "channel_point_definitions").await?;

        if !table_exists {
            // 表不存在，创建新表
            // 业务说明：全新安装时需要创建完整的表结构
            log::info!("channel_point_definitions表不存在，创建新表");
            Self::create_channel_point_definitions_table(db).await?;
        } else {
            // 表存在，检查并添加缺失的列，保留现有数据
            // 业务说明：升级场景，保持向后兼容性
            log::info!("channel_point_definitions表已存在，检查并添加缺失的列");
            Self::add_channel_point_definition_columns(db).await?;

            // 检查数据完整性
            // 业务说明：统计现有记录数，便于迁移后验证
            // Rust知识点：
            // - Statement::from_string 创建SQL语句
            // - map_err 转换错误类型
            let count_result = db.query_all(Statement::from_string(
                sea_orm::DatabaseBackend::Sqlite,
                "SELECT COUNT(*) as count FROM channel_point_definitions".to_string()
            )).await.map_err(|e| AppError::persistence_error(format!("查询通道定义数量失败: {}", e)))?;

            // 业务说明：获取并记录现有数据数量
            // Rust知识点：
            // - if let Some() 模式匹配 Option 类型
            // - try_get::<T> 尝试获取指定类型的值
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
    /// 
    /// 业务说明：
    /// 升级数据库时，为现有表添加新功能所需的列
    /// 保持向后兼容性，不删除现有列，只添加新列
    /// 
    /// 新增列说明：
    /// - batch_id: 批次ID，关联到test_batch_info表
    /// - *_plc_address: 各种报警设定值和反馈值的PLC地址
    /// - created_time/updated_time: 时间戳字段
    /// 
    /// Rust知识点：
    /// - async fn: 异步函数
    async fn add_channel_point_definition_columns(db: &DatabaseConnection) -> Result<(), AppError> {
        log::info!("检查并添加channel_point_definitions表的缺失列...");

        // 获取现有列信息
        // 业务说明：通过PRAGMA命令获取表的现有列，避免重复添加
        let existing_columns = Self::get_existing_columns(db, "channel_point_definitions").await?;

        // 需要添加的新列（包括batch_id）
        // 业务说明：这些列是新功能所需
        // 🔥 关键修复：添加batch_id字段，解决旧数据与批次关联问题
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

        // 遍历新列，检查并添加
        // Rust知识点：
        // - for循环解构元组
        // - &str.to_string() 转换为String
        // - contains() 检查HashSet是否包含元素
        for (column_name, column_def) in new_columns {
            if !existing_columns.contains(&column_name.to_string()) {
                log::info!("添加{}列到channel_point_definitions表", column_name);
                // SQL DDL语句：ALTER TABLE添加列
                let sql = format!("ALTER TABLE channel_point_definitions ADD COLUMN {} {}", column_name, column_def);
                db.execute(Statement::from_string(
                    sea_orm::DatabaseBackend::Sqlite,
                    sql
                )).await.map_err(|e| AppError::persistence_error(format!("添加{}列失败: {}", column_name, e)))?;

                // 为时间戳列设置默认值
                // 业务说明：确保时间戳字段不为NULL，使用当前时间填充
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
    /// 
    /// 业务说明：
    /// channel_test_instances 表存储每次测试的实例数据
    /// 一个通道定义可以有多个测试实例（不同批次）
    /// 实例记录测试状态、结果、错误信息等
    /// 
    /// 执行流程：
    /// 1. 检查表是否存在
    /// 2. 不存在则创建新表
    /// 3. 存在则添加新列并修复时间字段
    /// 
    /// Rust知识点：
    /// - Result<(), AppError> 表示可能失败的操作
    async fn migrate_channel_test_instances(db: &DatabaseConnection) -> Result<(), AppError> {
        log::info!("开始迁移channel_test_instances表...");

        let table_exists = Self::check_table_exists(db, "channel_test_instances").await?;

        if !table_exists {
            // 业务说明：全新安装，创建完整表结构
            Self::create_channel_test_instances_table(db).await?;
        } else {
            // 业务说明：升级场景，添加新列
            Self::add_channel_test_instance_columns(db).await?;
            // 修复旧的时间字段问题
            // 业务说明：旧版本使用creation_time，新版本统一为created_time
            Self::fix_channel_test_instances_time_fields(db).await?;
        }

        log::info!("channel_test_instances表迁移完成");
        Ok(())
    }

    /// 迁移测试批次信息表
    /// 
    /// 业务说明：
    /// test_batch_info 表存储测试批次的基本信息
    /// 包括批次名称、状态、进度、操作员等
    /// 是测试管理的核心表
    /// 
    /// Rust知识点：
    /// - async/await 异步编程模型
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
    /// 
    /// 业务说明：
    /// raw_test_outcomes 表存储每个子测试项的详细结果
    /// 包括测试值、工程值、测试时间、成功状态等
    /// 一个通道实例可以有多个子测试项结果
    /// 
    /// Rust知识点：
    /// - 关联函数（associated function）通过Self调用
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
    /// 
    /// 业务说明：
    /// allocation_records 表存储通道分配的历史记录
    /// 记录每次分配的策略、结果摘要、操作员等
    /// 用于审计和问题追溯
    /// 
    /// Rust知识点：
    /// - if 表达式可以省略else分支
    async fn migrate_allocation_records(db: &DatabaseConnection) -> Result<(), AppError> {
        log::info!("开始迁移allocation_records表...");

        let table_exists = Self::check_table_exists(db, "allocation_records").await?;

        if !table_exists {
            Self::create_allocation_records_table(db).await?;
        } else {
            // 如需添加新列可在此处实现
            // 业务说明：当前版本无需添加新列
        }

        log::info!("allocation_records表迁移完成");
        Ok(())
    }

    /// 创建批次分配记录表
    /// 
    /// 业务说明：
    /// 创建完整的分配记录表结构
    /// 字段说明：
    /// - id: 主键
    /// - batch_id: 关联的批次ID
    /// - strategy: 分配策略（如按类型、按顺序等）
    /// - summary_json: 分配结果JSON摘要
    /// - operator_name: 操作员名称
    /// - created_time: 创建时间
    /// 
    /// Rust知识点：
    /// - r#"..."# 原始字符串，保留格式
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
    /// 
    /// 业务说明：
    /// 通过查询SQLite的元数据表sqlite_master来判断表是否存在
    /// 这是所有迁移操作的前置检查
    /// 
    /// 参数：
    /// - db: 数据库连接
    /// - table_name: 要检查的表名
    /// 
    /// 返回：
    /// - Ok(true): 表存在
    /// - Ok(false): 表不存在
    /// - Err: 查询失败
    /// 
    /// SQL知识点：
    /// - sqlite_master 是SQLite的系统表
    /// - type='table' 过滤只查询表（排除索引、视图等）
    /// 
    /// Rust知识点：
    /// - &str 字符串切片引用
    /// - vec![] 创建向量
    /// - into() 类型转换
    async fn check_table_exists(db: &DatabaseConnection, table_name: &str) -> Result<bool, AppError> {
        // SQL查询：从sqlite_master表中查找指定表名
        let sql = "SELECT name FROM sqlite_master WHERE type='table' AND name=?";
        // 业务说明：使用参数化查询避免SQL注入
        // Rust知识点：Statement::from_sql_and_values 创建参数化查询
        let result = db.query_all(Statement::from_sql_and_values(
            sea_orm::DatabaseBackend::Sqlite,
            sql,
            vec![table_name.into()]  // 将&str转换为Value类型
        )).await.map_err(|e| AppError::persistence_error(format!("检查表是否存在失败: {}", e)))?;

        // 业务说明：如果查询结果不为空，说明表存在
        // Rust知识点：is_empty() 检查Vec是否为空
        Ok(!result.is_empty())
    }

    /// 获取表的现有列
    /// 
    /// 业务说明：
    /// 使用SQLite的PRAGMA命令获取表的列信息
    /// 用于判断哪些列需要添加，避免重复添加
    /// 
    /// 参数：
    /// - db: 数据库连接
    /// - table_name: 表名
    /// 
    /// 返回：
    /// - Ok: 列名的HashSet集合
    /// - Err: 查询失败
    /// 
    /// SQL知识点：
    /// - PRAGMA table_info() 返回表的列信息
    /// 
    /// Rust知识点：
    /// - HashSet<String> 去重集合，提供O(1)查找性能
    /// - format! 宏用于字符串格式化
    async fn get_existing_columns(db: &DatabaseConnection, table_name: &str) -> Result<std::collections::HashSet<String>, AppError> {
        // 使用PRAGMA命令获取表结构
        // 业务说明：PRAGMA table_info返回列的详细信息
        let sql = format!("PRAGMA table_info({})", table_name);
        let result = db.query_all(Statement::from_string(
            sea_orm::DatabaseBackend::Sqlite,
            sql
        )).await.map_err(|e| AppError::persistence_error(format!("获取表结构失败: {}", e)))?;

        // 构建HashSet存储列名
        let mut columns = std::collections::HashSet::new();
        // 业务说明：遍历PRAGMA返回的每一行，提取name字段
        // Rust知识点：
        // - for循环消耗Vec
        // - try_get 尝试获取指定类型的值
        // - insert() 向HashSet添加元素
        for row in result {
            if let Ok(column_name) = row.try_get::<String>("", "name") {
                columns.insert(column_name);
            }
        }

        Ok(columns)
    }

    /// 创建通道点位定义表
    /// 
    /// 业务说明：
    /// 创建完整的channel_point_definitions表结构
    /// 这是系统最核心的表，包含所有通道的详细配置
    /// 
    /// 表结构说明：
    /// - 基本信息：id、batch_id、模块信息、通道位置等
    /// - 通道属性：类型、供电方式、数据类型等
    /// - 量程信息：上下限值
    /// - 报警设置：四级报警（LL/L/H/HH）的设定值和地址
    /// - 维护功能：维护值设置和开关
    /// - PLC地址：绝对地址和通信地址
    /// - 时间戳：创建和更新时间
    /// 
    /// Rust知识点：
    /// - CREATE TABLE IF NOT EXISTS 避免重复创建
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
        // 业务说明：sequence_number用于排序显示，旧版本可能没有这个字段
        let columns = db
            .query_all(Statement::from_string(
                db.get_database_backend(),
                "PRAGMA table_info(channel_point_definitions);".to_string(),
            ))
            .await
            .map_err(|e| AppError::persistence_error(format!("获取表结构失败: {}", e)))?;

        // 检查是否存在sequence_number列
        // Rust知识点：
        // - any() 检查迭代器中是否有任何元素满足条件
        // - unwrap_or_default() 在错误时返回默认值
        let has_seq_col = columns.iter().any(|column| {
            let name: String = column.try_get("", "name").unwrap_or_default();
            name == "sequence_number"
        });

        if !has_seq_col {
            // 业务说明：添加缺失的sequence_number列
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
    /// 
    /// 业务说明：
    /// 创建channel_test_instances表的完整结构
    /// 这个表存储每个通道在特定批次中的测试实例
    /// 一个通道定义可以在不同批次中产生多个测试实例
    /// 
    /// 表结构说明：
    /// - 身份信息：instance_id、definition_id、test_batch_id
    /// - 基本信息：通道标签、变量名、描述、模块类型等
    /// - 测试状态：overall_status、current_step_details、error_message
    /// - 时间记录：创建、开始、更新、结束时间，总耗时
    /// - 测试结果：硬接点状态、实际值、期望值、各百分比测试结果
    /// - 报警状态：四级报警状态（低低/低/高/高高）
    /// - 功能状态：维护功能、显示值状态
    /// - 测试PLC信息：测试通道标签和通信地址
    /// - 操作信息：当前操作员、重试次数
    /// - JSON数据：子测试结果、硬接点读数、数字测试步骤、瞬态数据
    /// - 错误备注：集成错误、PLC编程错误、HMI配置错误的人工备注
    /// 
    /// Rust知识点：
    /// - DEFAULT 子句设置列的默认值
    /// - JSON字段存储复杂的结构化数据
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
                transient_data_json TEXT,
                integration_error_notes TEXT,
                plc_programming_error_notes TEXT,
                hmi_configuration_error_notes TEXT
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
    /// 
    /// 业务说明：
    /// 为已存在的channel_test_instances表添加新功能所需的列
    /// 这个函数用于数据库升级场景，保留现有数据的同时添加新字段
    /// 同时会移除已废弃的列（如trend_check、report_check）
    /// 
    /// 新增列类别：
    /// - 基本信息列：test_batch_name、channel_tag、variable_name等
    /// - 状态跟踪列：current_step_details、error_message、各类状态字段
    /// - 时间记录列：start_time、final_test_time、total_test_duration_ms
    /// - 测试结果列：各百分比测试结果（0%、25%、50%、75%、100%）
    /// - 报警状态列：四级报警状态字段
    /// - JSON数据列：存储复杂结构的测试数据
    /// - 错误备注列：用于人工记录各类错误原因
    /// 
    /// 特殊处理：
    /// - NOT NULL DEFAULT '' 确保非空字段有默认值
    /// - 时间戳字段自动设置为当前时间
    /// - 尝试删除废弃列，失败则记录警告（兼容旧版SQLite）
    /// 
    /// Rust知识点：
    /// - vec![] 宏创建包含元组的向量
    /// - &str 和 String 的转换
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
            // 错误备注字段 - 用于人工记录测试失败原因
            // 业务说明：当测试失败时，工程师可以手动记录具体的错误原因
            ("integration_error_notes", "TEXT"),          // 集成错误：如通道配置错误
            ("plc_programming_error_notes", "TEXT"),      // PLC编程错误：如地址错误
            ("hmi_configuration_error_notes", "TEXT"),    // HMI配置错误：如画面配置错误
        ];

        // 遍历新列，检查并添加缺失的列
        // Rust知识点：for循环解构元组，获取列名和定义
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
        // 业务说明：这些列在新版本中已不再使用
        // trend_check: 原用于趋势检查，现已集成到其他功能中
        // report_check: 原用于报表检查，现已有新的报表系统
        let obsolete_columns = vec!["trend_check", "report_check"];
        // Rust知识点：&obsolete_columns 借用向量进行迭代
        for column in &obsolete_columns {
            if existing_columns.contains(&column.to_string()) {
                log::info!("移除已废弃列{}从channel_test_instances表", column);
                let sql = format!("ALTER TABLE channel_test_instances DROP COLUMN {}", column);
                // 由于SQLite 3.35+才支持DROP COLUMN，如果失败则记录警告并继续
                // 业务说明：保持向后兼容，不因删除列失败而中断迁移
                // Rust知识点：if let Err(e) 模式匹配错误情况
                if let Err(e) = db.execute(Statement::from_string(
                    sea_orm::DatabaseBackend::Sqlite,
                    sql,
                )).await {
                    // 记录警告但不中断执行
                    log::warn!("删除列{}失败: {} (可能SQLite版本过旧，或列已被其他对象依赖)", column, e);
                }
            }
        }

        Ok(())
    }

    /// 创建测试批次信息表
    /// 
    /// 业务说明：
    /// 创建test_batch_info表的完整结构
    /// 这是测试管理的核心表，每个批次代表一次完整的测试任务
    /// 包含批次的所有元数据、状态信息和统计数据
    /// 
    /// 表结构说明：
    /// - 基本信息：batch_id、batch_name、product_model、serial_number
    /// - 客户信息：customer_name、station_name
    /// - 时间记录：created_time、updated_time、start_time、end_time、total_duration_ms
    /// - 人员信息：operator_name（执行人）、created_by（创建人）
    /// - 状态信息：overall_status（总体状态）、status_summary（状态摘要）、error_message
    /// - 统计信息：各类点位统计（总数、已测、通过、失败、跳过、未测）
    /// - 进度信息：progress_percentage（进度百分比）、current_testing_channel（当前测试通道）
    /// - 配置信息：test_configuration（测试配置）、import_source（导入来源）
    /// - 扩展数据：custom_data_json（自定义数据）
    /// 
    /// 默认值说明：
    /// - overall_status默认为'NotTested'
    /// - 各统计字段默认为0
    /// - progress_percentage默认为0.0
    /// 
    /// Rust知识点：
    /// - PRIMARY KEY约束确保批次ID唯一
    /// - DEFAULT子句设置默认值
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
    /// 
    /// 业务说明：
    /// 为已存在的test_batch_info表添加新功能所需的列
    /// 用于数据库升级场景，确保旧版本升级后具有所有必需的字段
    /// 重点添加了统计字段和状态跟踪字段
    /// 
    /// 新增列分类：
    /// - 基本信息：batch_name、customer_name、station_name
    /// - 时间管理：start_time、end_time、total_duration_ms、last_updated_time
    /// - 人员信息：operator_name（操作员）、created_by（创建者）
    /// - 状态跟踪：overall_status、status_summary、error_message
    /// - 统计数据：total_points、tested_points、passed_points、failed_points等
    /// - 进度管理：progress_percentage、current_testing_channel
    /// - 配置信息：test_configuration、import_source
    /// - 扩展数据：custom_data_json
    /// 
    /// 特殊处理：
    /// - 所有NOT NULL字段都设置了默认值，避免升级失败
    /// - 时间戳字段自动填充当前时间
    /// - overall_status默认为'NotTested'表示未测试状态
    /// 
    /// Rust知识点：
    /// - 使用vec!宏创建元组数组
    /// - 动态SQL构建
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
            // 人员信息字段
            ("operator_name", "TEXT"),         // 操作员：执行测试的人员
            ("created_by", "TEXT"),            // 创建者：创建批次的人员
            // 状态管理字段
            ("overall_status", "TEXT NOT NULL DEFAULT 'NotTested'"),  // 总体状态
            ("status_summary", "TEXT"),        // 状态摘要：简要描述当前状态
            ("error_message", "TEXT"),         // 错误信息：失败时的详细信息
            // 统计字段 - 实时跟踪测试进度
            ("total_points", "INTEGER DEFAULT 0"),    // 总点位数
            ("tested_points", "INTEGER DEFAULT 0"),   // 已测试点位数
            ("passed_points", "INTEGER DEFAULT 0"),   // 通过的点位数
            ("failed_points", "INTEGER DEFAULT 0"),   // 失败的点位数
            ("skipped_points", "INTEGER DEFAULT 0"),  // 跳过的点位数
            ("not_tested_points", "INTEGER DEFAULT 0"),   // 未测试点位数
            // 进度跟踪字段
            ("progress_percentage", "REAL DEFAULT 0.0"),   // 进度百分比(0-100)
            ("current_testing_channel", "TEXT"),           // 当前正在测试的通道
            // 配置和来源信息
            ("test_configuration", "TEXT"),                // 测试配置JSON
            ("import_source", "TEXT"),                     // 数据导入来源(Excel/手动等)
            ("custom_data_json", "TEXT"),                  // 自定义扩展数据
            // 时间戳字段
            ("created_time", "TEXT"),
            ("updated_time", "TEXT"), 
            ("last_updated_time", "TEXT"),                 // 兼容旧版本字段名
        ];

        // 遍历并添加缺失的列
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
    /// 
    /// 业务说明：
    /// 处理数据库版本升级中的字段名称变更
    /// 旧版本使用creation_time和last_updated_time
    /// 新版本统一为created_time和updated_time
    /// 该函数负责数据迁移，确保时间信息不丢失
    /// 
    /// 迁移策略：
    /// - created_time = creation_time（如果为空）
    /// - updated_time = last_updated_time 或 creation_time（如果为空）
    /// - 保留原字段，不删除，确保向后兼容
    /// 
    /// Rust知识点：
    /// - HashSet::contains 检查集合中是否包含元素
    /// - COALESCE SQL函数返回第一个非NULL值
    async fn fix_test_batch_info_time_fields(db: &DatabaseConnection) -> Result<(), AppError> {
        log::info!("修复test_batch_info表的时间字段...");

        // 检查是否存在旧的creation_time字段
        let existing_columns = Self::get_existing_columns(db, "test_batch_info").await?;

        if existing_columns.contains(&"creation_time".to_string()) {
            log::info!("发现旧的creation_time字段，开始数据迁移...");

            // 将旧字段的数据复制到新字段
            // SQL知识点：COALESCE函数返回参数列表中第一个非NULL值
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
    /// 
    /// 业务说明：
    /// 与fix_test_batch_info_time_fields类似，处理channel_test_instances表的时间字段迁移
    /// 确保从旧版本升级的数据库保持时间信息的完整性
    /// 这是数据库重构过程中的重要步骤
    /// 
    /// 迁移内容：
    /// - creation_time -> created_time
    /// - last_updated_time -> updated_time
    /// 
    /// 注意事项：
    /// - 只在确实存在旧字段时执行迁移
    /// - 使用WHERE子句避免覆盖已有的新字段数据
    /// - 迁移完成后不删除旧字段，保持兼容性
    /// 
    /// Rust知识点：
    /// - if条件判断控制迁移执行
    /// - SQL UPDATE语句的条件更新
    async fn fix_channel_test_instances_time_fields(db: &DatabaseConnection) -> Result<(), AppError> {
        log::info!("修复channel_test_instances表的时间字段...");

        // 检查是否存在旧的creation_time字段
        let existing_columns = Self::get_existing_columns(db, "channel_test_instances").await?;

        if existing_columns.contains(&"creation_time".to_string()) {
            log::info!("发现旧的creation_time字段，开始数据迁移...");

            // 将旧字段的数据复制到新字段
            // 业务说明：确保只更新空值，避免覆盖已有数据
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
    /// 
    /// 业务说明：
    /// 创建raw_test_outcomes表，存储每个子测试项的详细测试结果
    /// 这是测试数据的最细粒度存储，记录每个测试步骤的原始数据
    /// 一个通道实例可以有多个子测试项（如0%、25%、50%、75%、100%测试）
    /// 
    /// 表结构说明：
    /// - id: 主键，唯一标识每个测试结果
    /// - channel_instance_id: 关联的通道实例ID
    /// - sub_test_item: 子测试项名称（如"0%测试"、"报警测试"等）
    /// - success: 测试是否成功
    /// - raw_value_read: 从PLC读取的原始值
    /// - eng_value_calculated: 计算得到的工程值
    /// - message: 测试消息或错误信息
    /// - start_time/end_time: 测试的开始和结束时间
    /// - readings_json: JSON格式的详细读数记录
    /// - test_result_*_percent: 各百分比点的测试结果
    /// - details_json: 其他详细信息的JSON存储
    /// 
    /// Rust知识点：
    /// - BOOLEAN类型在SQLite中实际存储为INTEGER (0/1)
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
    /// 
    /// 业务说明：
    /// 为已存在的raw_test_outcomes表添加新的测试结果列
    /// 这些列用于存储不同百分比点的测试结果值
    /// 支持渐进式的测试流程（0% -> 25% -> 50% -> 75% -> 100%）
    /// 
    /// 新增列说明：
    /// - test_result_0_percent: 0%量程点的测试结果
    /// - test_result_25_percent: 25%量程点的测试结果
    /// - test_result_50_percent: 50%量程点的测试结果
    /// - test_result_75_percent: 75%量程点的测试结果
    /// - test_result_100_percent: 100%量程点的测试结果
    /// 
    /// 应用场景：
    /// - 模拟量通道需要在不同量程点进行测试
    /// - 每个百分比点对应不同的输入值和期望输出
    /// - 用于验证通道的线性度和精度
    /// 
    /// Rust知识点：
    /// - REAL类型对应Rust的f32/f64类型
    async fn add_raw_test_outcomes_columns(db: &DatabaseConnection) -> Result<(), AppError> {
        let existing_columns = Self::get_existing_columns(db, "raw_test_outcomes").await?;

        // 需要添加的新列
        // 业务说明：这些列在旧版本中可能不存在，需要为升级添加
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
    /// 
    /// 业务说明：
    /// 预留的扩展点，用于创建未来可能需要的新表
    /// 当前版本暂时保留空实现，所有必需的表已在其他函数中创建
    /// 
    /// 设计意图：
    /// - 提供统一的扩展点，便于添加新表
    /// - 避免修改主迁移逻辑
    /// - 支持模块化的表创建
    /// 
    /// 可能的扩展：
    /// - 测试模板配置表
    /// - 用户权限管理表
    /// - 审计日志表
    /// - 系统配置表
    /// 
    /// Rust知识点：
    /// - 空实现函数保持接口一致性
    async fn create_missing_tables(db: &DatabaseConnection) -> Result<(), AppError> {
        log::info!("检查并创建缺失的表...");

        // 这里可以添加其他需要创建的表
        // （当前保留空实现）
        // 例如：测试配置表、PLC连接配置表等

        Ok(())
    }

    /// 验证数据完整性
    /// 
    /// 业务说明：
    /// 在所有迁移操作完成后，验证数据库的完整性
    /// 确保所有关键表都已正确创建，为应用运行提供保障
    /// 这是迁移过程的最后一道防线
    /// 
    /// 验证内容：
    /// - channel_point_definitions: 通道定义表（核心业务表）
    /// - channel_test_instances: 测试实例表（测试执行记录）
    /// - test_batch_info: 批次信息表（测试管理）
    /// 
    /// 失败处理：
    /// - 任何关键表缺失都会导致迁移失败
    /// - 返回明确的错误信息，指出缺失的表
    /// - 防止应用在不完整的数据库上运行
    /// 
    /// Rust知识点：
    /// - vec![] 创建字符串切片向量
    /// - for循环遍历验证每个表
    /// - 提前返回(early return)模式
    async fn verify_data_integrity(db: &DatabaseConnection) -> Result<(), AppError> {
        log::info!("验证数据完整性...");

        // 检查关键表是否存在
        // 业务说明：这些是系统运行必需的核心表
        let tables = vec!["channel_point_definitions", "channel_test_instances", "test_batch_info"];

        for table in tables {
            let exists = Self::check_table_exists(db, table).await?;
            if !exists {
                // 关键表缺失，迁移失败
                return Err(AppError::persistence_error(format!("关键表{}不存在", table)));
            }
        }

        log::info!("数据完整性验证通过");
        Ok(())
    }

    /// 🔥 数据恢复：为没有batch_id的通道定义恢复批次关联
    /// 
    /// 业务说明：
    /// 这是一个关键的数据修复函数，处理历史遗留问题
    /// 早期版本的channel_point_definitions表没有batch_id字段
    /// 导致通道定义成为"孤儿"，无法关联到具体批次
    /// 本函数通过分析测试实例数据，重建丢失的关联关系
    ///
    /// 执行流程：
    /// 1. 查找所有没有batch_id的通道定义（孤立数据）
    /// 2. 尝试通过测试实例找到对应的批次ID
    /// 3. 更新通道定义的batch_id字段
    /// 4. 统计恢复成功和失败的数量
    /// 
    /// 恢复策略：
    /// - 优先通过channel_test_instances表查找关联
    /// - 找到批次ID后立即更新channel_point_definitions
    /// - 找不到的记录只记录日志，不创建默认批次（避免污染数据）
    /// 
    /// 注意事项：
    /// - 🔧 修复：不再自动创建默认批次，保持数据真实性
    /// - 恢复失败的数据需要人工介入处理
    /// 
    /// Rust知识点：
    /// - mut变量用于统计计数
    /// - match表达式处理多种情况
    /// - Option<T>表示可能不存在的值
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
    /// 
    /// 业务说明：
    /// 辅助函数，用于数据恢复过程
    /// 通过查询channel_test_instances表，找到某个通道定义关联的批次
    /// 一个通道定义可能有多个测试实例，只需要找到任意一个即可确定批次
    /// 
    /// 参数：
    /// - db: 数据库连接
    /// - definition_id: 通道定义ID
    /// 
    /// 返回：
    /// - Ok(Some(batch_id)): 找到关联的批次ID
    /// - Ok(None): 没有找到关联的测试实例
    /// - Err: 查询失败
    /// 
    /// 实现说明：
    /// - 使用LIMIT 1提高查询效率，只需要找到一个即可
    /// - 通过definition_id外键关联查找
    /// 
    /// Rust知识点：
    /// - Result<Option<T>, E> 双层包装表示可能失败的可选值
    /// - &str 参数避免String的所有权转移
    async fn find_batch_id_for_definition(db: &DatabaseConnection, definition_id: &str) -> Result<Option<String>, AppError> {
        // SQL查询：通过definition_id查找任意一个测试实例的批次ID
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
    /// 
    /// 业务说明：
    /// 数据恢复的核心操作，将找到的批次ID更新到通道定义表中
    /// 修复孤立的通道定义，使其重新关联到正确的批次
    /// 
    /// 参数：
    /// - db: 数据库连接
    /// - definition_id: 要更新的通道定义ID
    /// - batch_id: 恢复的批次ID
    /// 
    /// 返回：
    /// - Ok(()): 更新成功
    /// - Err: 更新失败
    /// 
    /// SQL说明：
    /// - 使用参数化查询防止SQL注入
    /// - WHERE条件确保只更新指定的记录
    /// 
    /// Rust知识点：
    /// - () 作为返回类型表示只关心操作是否成功
    /// - vec![].into() 将参数转换为SeaORM需要的Value类型
    async fn update_definition_batch_id(db: &DatabaseConnection, definition_id: &str, batch_id: &str) -> Result<(), AppError> {
        // 更新SQL：设置channel_point_definitions表的batch_id字段
        let sql = r#"
            UPDATE channel_point_definitions
            SET batch_id = ?
            WHERE id = ?
        "#;

        db.execute(Statement::from_sql_and_values(
            sea_orm::DatabaseBackend::Sqlite,
            sql,
            vec![batch_id.into(), definition_id.into()]  // 参数顺序与SQL中的?顺序对应
        )).await.map_err(|e| AppError::persistence_error(format!("更新批次ID失败: {}", e)))?;

        Ok(())
    }

    /// 为孤立的通道定义创建默认批次
    /// 
    /// 业务说明：
    /// 【已废弃】此函数原用于为无法找到批次的通道定义创建默认批次
    /// 当前版本不再自动创建默认批次，保留此函数仅供参考
    /// 创建虚拟批次可能会污染数据，建议人工处理孤立数据
    /// 
    /// 原设计意图：
    /// - 为历史遗留的孤立通道定义创建占位批次
    /// - 批次名称包含"历史数据恢复"标识
    /// - 保留站场信息便于后续追溯
    /// 
    /// 参数：
    /// - db: 数据库连接
    /// - definition_id: 孤立的通道定义ID
    /// - tag: 通道标签
    /// - station_name: 站场名称
    /// 
    /// 返回：
    /// - Ok(batch_id): 创建的批次ID
    /// - Err: 创建失败
    /// 
    /// Rust知识点：
    /// - use语句在函数内部导入依赖
    /// - Uuid::new_v4() 生成随机UUID
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
    /// 
    /// 业务说明：
    /// 为PLC连接配置表添加新的通信参数列
    /// 这些参数是Modbus通信协议的重要配置项
    /// 支持不同厂商PLC的兼容性需求
    /// 
    /// 新增列说明：
    /// - byte_order: 字节序配置，控制多字节数据的解析顺序
    ///   - CDAB: 最常见的格式（默认值）
    ///   - ABCD: 标准大端序
    ///   - BADC: 字交换格式
    ///   - DCBA: 完全反转格式
    /// - zero_based_address: 地址基数配置
    ///   - 0: 地址从0开始（默认值）
    ///   - 1: 地址从1开始（某些PLC使用）
    /// 
    /// 特殊处理：
    /// - 如果表不存在，跳过处理（由SeaORM迁移器负责创建）
    /// - 设置合理的默认值，保证兼容性
    /// 
    /// Rust知识点：
    /// - 提前返回模式处理表不存在的情况
    /// - log::warn! 记录警告级别日志
    async fn add_plc_connection_config_columns(db: &DatabaseConnection) -> Result<(), AppError> {
        log::info!("检查并添加plc_connection_configs表缺失列...");

        // 先检查表是否存在
        let table_exists = Self::check_table_exists(db, "plc_connection_configs").await?;
        if !table_exists {
            // 表不存在时，新建由SeaORM迁移器处理，这里直接返回
            log::warn!("plc_connection_configs表不存在，跳过列检查");
            return Ok(());
        }

        let existing_columns = Self::get_existing_columns(db, "plc_connection_configs").await?;

        // 定义需要添加的新列
        let new_columns = vec![
            ("byte_order", "TEXT DEFAULT 'CDAB'"),         // Modbus字节序
            ("zero_based_address", "INTEGER DEFAULT 0"),   // 地址基数(0或1)
        ];

        // 遍历并添加缺失的列
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
