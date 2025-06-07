// 文件: FactoryTesting/src-tauri/src/bin/test_and_verify_storage.rs
// 测试硬点测试结果存储并立即验证数据库

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use chrono::Utc;
use uuid::Uuid;
use app_lib::models::structs::{ChannelTestInstance, RawTestOutcome, AnalogReadingPoint, ChannelPointDefinition, TestBatchInfo};
use app_lib::models::enums::{OverallTestStatus, SubTestItem, SubTestStatus, ModuleType, PointDataType};
use app_lib::services::domain::channel_state_manager::ChannelStateManager;
use app_lib::services::infrastructure::persistence::sqlite_orm_persistence_service::SqliteOrmPersistenceService;
use app_lib::utils::error::AppError;
use app_lib::services::infrastructure::persistence::persistence_service::PersistenceConfig;
use app_lib::IChannelStateManager;
use app_lib::traits::PersistenceService;
use sea_orm::{EntityTrait, QueryFilter, ColumnTrait};
use app_lib::models::entities::{channel_test_instance, raw_test_outcome};

#[tokio::main]
async fn main() -> Result<(), AppError> {
    env_logger::init();
    
    println!("🧪 开始测试硬点测试结果存储并立即验证");
    
    // 创建服务
    let config = PersistenceConfig {
        storage_root_dir: PathBuf::from("data"),
        channel_definitions_dir: "channel_definitions".to_string(),
        test_instances_dir: "test_instances".to_string(),
        test_batches_dir: "test_batches".to_string(),
        test_outcomes_dir: "test_outcomes".to_string(),
        enable_auto_backup: false,
        backup_retention_days: 30,
        max_file_size_mb: 100,
        enable_compression: false,
    };

    // 使用与检查工具相同的数据库路径
    let db_path = PathBuf::from("data/factory_testing_data.sqlite");
    let persistence_service = SqliteOrmPersistenceService::new(config.clone(), Some(&db_path)).await?;

    // 为状态管理器创建另一个持久化服务实例
    let persistence_service_for_state = SqliteOrmPersistenceService::new(config, Some(&db_path)).await?;
    let state_manager = ChannelStateManager::new(Arc::new(persistence_service_for_state));

    // 首先创建测试批次
    let batch_id = Uuid::new_v4().to_string();
    let test_batch = TestBatchInfo {
        batch_id: batch_id.clone(),
        batch_name: "测试批次001".to_string(),
        product_model: Some("测试产品".to_string()),
        serial_number: Some("SN001".to_string()),
        customer_name: None,
        station_name: None,
        creation_time: Utc::now(),
        last_updated_time: Utc::now(),
        operator_name: Some("测试员".to_string()),
        status_summary: None,
        total_points: 1,
        tested_points: 0,
        passed_points: 0,
        failed_points: 0,
        skipped_points: 0,
        overall_status: OverallTestStatus::NotTested,
        custom_data: HashMap::new(),
    };

    persistence_service.save_batch_info(&test_batch).await?;
    println!("✅ 创建测试批次: {}", test_batch.batch_id);

    // 创建通道定义
    let definition_id = Uuid::new_v4().to_string();
    let channel_definition = ChannelPointDefinition {
        id: definition_id.clone(),
        batch_id: Some(batch_id.clone()),
        tag: "TEST_AI_01".to_string(),
        variable_name: "测试AI变量".to_string(),
        variable_description: "测试AI变量描述".to_string(),
        station_name: "测试站".to_string(),
        module_name: "AI模块".to_string(),
        module_type: ModuleType::AI,
        channel_tag_in_module: "1".to_string(),
        data_type: PointDataType::Float,
        power_supply_type: "有源".to_string(),
        wire_system: "4线制".to_string(),
        plc_absolute_address: Some("%MD100".to_string()),
        plc_communication_address: "40001".to_string(),
        range_low_limit: Some(0.0),
        range_high_limit: Some(100.0),
        engineering_unit: Some("℃".to_string()),
        sll_set_value: None,
        sll_set_point_address: None,
        sll_set_point_plc_address: None,
        sll_set_point_communication_address: None,
        sll_feedback_address: None,
        sll_feedback_plc_address: None,
        sll_feedback_communication_address: None,
        sl_set_value: None,
        sl_set_point_address: None,
        sl_set_point_plc_address: None,
        sl_set_point_communication_address: None,
        sl_feedback_address: None,
        sl_feedback_plc_address: None,
        sl_feedback_communication_address: None,
        sh_set_value: None,
        sh_set_point_address: None,
        sh_set_point_plc_address: None,
        sh_set_point_communication_address: None,
        sh_feedback_address: None,
        sh_feedback_plc_address: None,
        sh_feedback_communication_address: None,
        shh_set_value: None,
        shh_set_point_address: None,
        shh_set_point_plc_address: None,
        shh_set_point_communication_address: None,
        shh_feedback_address: None,
        shh_feedback_plc_address: None,
        shh_feedback_communication_address: None,
        maintenance_value_set_point_address: None,
        maintenance_value_set_point_plc_address: None,
        maintenance_value_set_point_communication_address: None,
        maintenance_enable_switch_point_address: None,
        maintenance_enable_switch_point_plc_address: None,
        maintenance_enable_switch_point_communication_address: None,
        access_property: None,
        save_history: None,
        power_failure_protection: None,
        test_rig_plc_address: None,
    };

    persistence_service.save_channel_definition(&channel_definition).await?;
    println!("✅ 创建通道定义: {}", channel_definition.id);
    
    // 创建测试实例
    let mut test_instance = ChannelTestInstance {
        instance_id: Uuid::new_v4().to_string(),
        definition_id: definition_id.clone(),
        test_batch_id: batch_id.clone(),
        test_batch_name: "测试批次001".to_string(),
        overall_status: OverallTestStatus::HardPointTesting,
        current_step_details: None,
        error_message: None,
        creation_time: Utc::now(),
        start_time: Some(Utc::now()),
        last_updated_time: Utc::now(),
        final_test_time: None,
        total_test_duration_ms: None,
        sub_test_results: HashMap::new(),
        hardpoint_readings: None,
        manual_test_current_value_input: None,
        manual_test_current_value_output: None,
        current_operator: None,
        retries_count: 0,
        transient_data: HashMap::new(),
        test_plc_channel_tag: Some("TEST_AI_01".to_string()),
        test_plc_communication_address: Some("40001".to_string()),
    };
    
    println!("✅ 创建测试实例: {}", test_instance.instance_id);
    
    // 创建硬点测试结果
    let hardpoint_outcome = RawTestOutcome {
        channel_instance_id: test_instance.instance_id.clone(),
        sub_test_item: SubTestItem::HardPoint,
        success: true,
        raw_value_read: Some("多点测试".to_string()),
        eng_value_calculated: Some("0.0-100.0".to_string()),
        message: Some("AI硬点5点测试全部通过".to_string()),
        start_time: Utc::now(),
        end_time: Utc::now(),
        readings: Some(vec![
            AnalogReadingPoint {
                set_percentage: 0.0,
                set_value_eng: 0.0,
                expected_reading_raw: Some(0.0),
                actual_reading_raw: Some(0.1),
                actual_reading_eng: Some(0.1),
                status: SubTestStatus::Passed,
                error_percentage: Some(0.1),
            },
            AnalogReadingPoint {
                set_percentage: 0.25,
                set_value_eng: 25.0,
                expected_reading_raw: Some(25.0),
                actual_reading_raw: Some(25.2),
                actual_reading_eng: Some(25.2),
                status: SubTestStatus::Passed,
                error_percentage: Some(0.8),
            },
        ]),
        test_result_0_percent: Some(0.1),
        test_result_25_percent: Some(25.2),
        test_result_50_percent: Some(49.8),
        test_result_75_percent: Some(75.1),
        test_result_100_percent: Some(99.9),
        details: HashMap::new(),
    };
    
    println!("✅ 创建硬点测试结果");
    
    // 应用测试结果
    state_manager.apply_raw_outcome(&mut test_instance, hardpoint_outcome.clone()).await?;
    println!("✅ 应用测试结果到测试实例");
    
    // 保存到数据库
    persistence_service.save_test_instance(&test_instance).await?;
    println!("✅ 测试实例已保存到数据库");

    // 保存原始测试结果
    persistence_service.save_test_outcome(&hardpoint_outcome).await?;
    println!("✅ 原始测试结果已保存到数据库");

    // 立即验证数据库中的数据
    println!("\n🔍 立即验证数据库中的数据...");

    // 获取数据库连接
    let db = persistence_service.get_database_connection();

    // 查询测试实例
    let instances = channel_test_instance::Entity::find()
        .filter(channel_test_instance::Column::InstanceId.eq(&test_instance.instance_id))
        .all(db)
        .await
        .map_err(|e| AppError::persistence_error(format!("查询测试实例失败: {}", e)))?;
    
    println!("📊 找到 {} 个测试实例", instances.len());
    
    if let Some(instance) = instances.first() {
        println!("✅ 测试实例存在: {}", instance.instance_id);
        println!("   - test_result_0_percent: {:?}", instance.test_result_0_percent);
        println!("   - test_result_25_percent: {:?}", instance.test_result_25_percent);
        println!("   - test_result_50_percent: {:?}", instance.test_result_50_percent);
        println!("   - test_result_75_percent: {:?}", instance.test_result_75_percent);
        println!("   - test_result_100_percent: {:?}", instance.test_result_100_percent);
    } else {
        println!("❌ 测试实例未找到！");
    }
    
    // 查询原始测试结果
    let outcomes = raw_test_outcome::Entity::find()
        .filter(raw_test_outcome::Column::ChannelInstanceId.eq(&test_instance.instance_id))
        .all(db)
        .await
        .map_err(|e| AppError::persistence_error(format!("查询原始测试结果失败: {}", e)))?;
    
    println!("📊 找到 {} 个原始测试结果", outcomes.len());
    
    if let Some(outcome) = outcomes.first() {
        println!("✅ 原始测试结果存在: {}", outcome.id);
        println!("   - test_result_0_percent: {:?}", outcome.test_result_0_percent);
        println!("   - test_result_25_percent: {:?}", outcome.test_result_25_percent);
        println!("   - test_result_50_percent: {:?}", outcome.test_result_50_percent);
        println!("   - test_result_75_percent: {:?}", outcome.test_result_75_percent);
        println!("   - test_result_100_percent: {:?}", outcome.test_result_100_percent);
    } else {
        println!("❌ 原始测试结果未找到！");
    }
    
    println!("🎉 测试和验证完成！");
    
    Ok(())
}
