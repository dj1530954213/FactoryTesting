use sea_orm::{Database, Statement, ConnectionTrait};

#[tokio::test]
async fn test_allocation_records_migrated() {
    // 使用内存 SQLite 数据库执行迁移，验证 allocation_records 表被正确创建
    let db = Database::connect("sqlite::memory:").await.expect("connect in-memory db");

    // 调用应用中的数据库迁移逻辑
    crate::database_migration::DatabaseMigration::migrate(&db)
        .await
        .expect("migrate should succeed");

    // 查询 sqlite_master 验证表存在
    let backend = db.get_database_backend();
    let stmt = Statement::from_string(
        backend,
        "SELECT name FROM sqlite_master WHERE type='table' AND name='allocation_records';",
    );

    let result = db.query_one(stmt).await.expect("query sqlite_master");
    assert!(result.is_some(), "allocation_records table should exist after migration");
}
