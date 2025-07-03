/// Tauri 命令模块
///
/// 这个模块定义了所有前端可以调用的 Tauri 命令
/// 将后端服务暴露给前端 Angular 应用

use crate::models::{
    ChannelPointDefinition, TestBatchInfo, ChannelTestInstance, RawTestOutcome,
    TestReport, ReportTemplate, ReportGenerationRequest, AppSettings
};
use crate::application::services::{
    ITestCoordinationService, TestCoordinationService,
    TestExecutionRequest, TestExecutionResponse, TestProgressUpdate,
    IReportGenerationService, ReportGenerationService
};
use crate::domain::services::{
    IChannelStateManager, ChannelStateManager,
    ITestExecutionEngine, TestExecutionEngine,
    ITestPlcConfigService, TestPlcConfigService,
    PlcConnectionManager
};
use crate::infrastructure::{
    IPersistenceService, SqliteOrmPersistenceService,
    excel::ExcelImporter,
    persistence::{AppSettingsService, JsonAppSettingsService, AppSettingsConfig},
    SimpleEventPublisher
};
use crate::infrastructure::plc_communication::IPlcCommunicationService;
use crate::application::services::channel_allocation_service::{IChannelAllocationService, ChannelAllocationService};
use crate::utils::error::{AppError, AppResult};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::State;
use tokio::sync::Mutex;
use std::collections::HashSet;
use chrono::{DateTime, Utc};
use std::convert::TryFrom;
use std::path::PathBuf;

// ============================================================================
// 应用状态管理
// ============================================================================

/// 应用状态，包含所有服务实例
pub struct AppState {
    //测试流程协调服务
    pub test_coordination_service: Arc<dyn ITestCoordinationService>,
    pub channel_state_manager: Arc<dyn IChannelStateManager>,
    pub test_execution_engine: Arc<dyn ITestExecutionEngine>,
    pub persistence_service: Arc<dyn IPersistenceService>,
    pub report_generation_service: Arc<dyn IReportGenerationService>,
    pub app_settings_service: Arc<dyn AppSettingsService>,
    pub test_plc_config_service: Arc<dyn ITestPlcConfigService>,
    pub channel_allocation_service: Arc<dyn IChannelAllocationService>,
    pub plc_connection_manager: Arc<PlcConnectionManager>,
    pub plc_monitoring_service: Arc<dyn crate::infrastructure::IPlcMonitoringService>,

    // 会话管理：跟踪当前会话中创建的批次
    pub session_batch_ids: Arc<Mutex<HashSet<String>>>,
    pub session_start_time: DateTime<Utc>,
}

/// 系统状态信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemStatus {
    pub active_test_tasks: usize,
    pub system_health: String,
    pub version: String,
}

impl AppState {
    /// 创建新的应用状态
    pub async fn new() -> AppResult<Self> {
        // 创建数据库配置
        let config = crate::infrastructure::persistence::PersistenceConfig::default();

        // 创建持久化服务 - 使用实际的SQLite文件而不是内存数据库
        let db_file_path = config.storage_root_dir.join("factory_testing_data.sqlite");

        // 确保数据库目录存在
        if let Some(parent_dir) = db_file_path.parent() {
            tokio::fs::create_dir_all(parent_dir).await.map_err(|e|
                AppError::io_error(
                    format!("创建数据库目录失败: {:?}", parent_dir),
                    e.kind().to_string()
                )
            )?;
        }

        // 如果数据库文件不存在，创建一个空文件
        if !db_file_path.exists() {
            tokio::fs::write(&db_file_path, "").await.map_err(|e|
                AppError::io_error(
                    format!("创建数据库文件失败: {:?}", db_file_path),
                    e.kind().to_string()
                )
            )?;
        }

        let sqlite_persistence_service = SqliteOrmPersistenceService::new(config.clone(), Some(&db_file_path)).await?;

        // 执行数据库迁移
        let db_conn = sqlite_persistence_service.get_database_connection();
        if let Err(e) = crate::database_migration::DatabaseMigration::migrate(db_conn).await {
            log::error!("数据库迁移失败: {}", e);
            return Err(e);
        }

        let persistence_service: Arc<dyn IPersistenceService> = Arc::new(sqlite_persistence_service);

        // 创建应用配置服务
        let app_settings_config = AppSettingsConfig::default();
        let mut app_settings_service: Arc<dyn AppSettingsService> = Arc::new(
            JsonAppSettingsService::new(app_settings_config)
        );

        // 初始化应用配置服务
        if let Some(service) = Arc::get_mut(&mut app_settings_service) {
            service.initialize().await?;
        }

        // 创建测试PLC配置服务（需要先创建，因为后面要用到）
        let test_plc_config_service: Arc<dyn ITestPlcConfigService> = Arc::new(
            TestPlcConfigService::new(persistence_service.clone())
        );

        // 创建通道状态管理器
        let channel_state_manager: Arc<dyn IChannelStateManager> = Arc::new(
            ChannelStateManager::new(persistence_service.clone())
        );

        // 🔧 修复：从数据库读取PLC连接配置，不使用硬编码IP
        let plc_connections = test_plc_config_service.get_plc_connections().await
            .map_err(|e| format!("获取PLC连接配置失败: {}", e))?;

        let test_plc_connection = plc_connections.iter()
            .find(|conn| conn.is_test_plc && conn.is_enabled)
            .ok_or_else(|| "没有找到启用的测试PLC连接配置".to_string())?;

        let target_plc_connection = plc_connections.iter()
            .find(|conn| !conn.is_test_plc && conn.is_enabled)
            .ok_or_else(|| "没有找到启用的被测PLC连接配置".to_string())?;

        log::info!("🔗 使用数据库PLC配置 - 测试PLC: {}:{}, 被测PLC: {}:{}",
            test_plc_connection.ip_address, test_plc_connection.port,
            target_plc_connection.ip_address, target_plc_connection.port);

        // 使用 PLC 连接配置中的字节顺序与地址基准构造 ModbusConfig
        let test_rig_config = crate::infrastructure::plc::modbus_plc_service::ModbusConfig::try_from(test_plc_connection)
            .map_err(|e| e.to_string())?;

        let target_config = crate::infrastructure::plc::modbus_plc_service::ModbusConfig::try_from(target_plc_connection)
            .map_err(|e| e.to_string())?;

        let plc_service_test_rig: Arc<dyn IPlcCommunicationService> = Arc::new(
            crate::infrastructure::ModbusTcpPlcService::default()
        );
        let plc_service_target: Arc<dyn IPlcCommunicationService> = Arc::new(
            crate::infrastructure::ModbusTcpPlcService::default()
        );

        // 创建测试执行引擎
        let test_execution_engine: Arc<dyn ITestExecutionEngine> = Arc::new(
            TestExecutionEngine::new(
                88, // 最大并发测试数，和PLC通道数量一致
                plc_service_test_rig,
                plc_service_target.clone(),
            )
        );

        // 创建事件发布器
        let event_publisher: Arc<dyn crate::domain::services::EventPublisher> = Arc::new(
            SimpleEventPublisher::new()
        );

        // 创建通道分配服务
        let channel_allocation_service: Arc<dyn crate::application::services::channel_allocation_service::IChannelAllocationService> = Arc::new(
            ChannelAllocationService::new()
        );

        // test_plc_config_service 已在上面创建



        // 创建测试协调服务
        let test_coordination_service: Arc<dyn ITestCoordinationService> = Arc::new(
            TestCoordinationService::new(
                channel_state_manager.clone(),
                test_execution_engine.clone(),
                persistence_service.clone(),
                event_publisher,
                channel_allocation_service.clone(),
                test_plc_config_service.clone(),
            )
        );

        // 创建报告生成服务
        let reports_dir = std::path::PathBuf::from("reports");
        let report_generation_service: Arc<dyn IReportGenerationService> = Arc::new(
            ReportGenerationService::new(
                persistence_service.clone(),
                reports_dir,
            )?
        );

        // 创建PLC连接管理器
        let plc_connection_manager = Arc::new(PlcConnectionManager::new(
            test_plc_config_service.clone(),
        ));

        // 设置全局PLC连接管理器，让ModbusPlcService能够访问
        crate::infrastructure::plc::modbus_plc_service::set_global_plc_manager(plc_connection_manager.clone());

        // 创建PLC监控服务 - 使用真实的PLC监控服务
        let plc_monitoring_service: Arc<dyn crate::infrastructure::IPlcMonitoringService> = Arc::new(
            crate::infrastructure::plc_monitoring_service::PlcMonitoringService::new(
                plc_service_target.clone(),
                Arc::new(crate::infrastructure::event_publisher::SimpleEventPublisher::new()),
            )
        );



        Ok(Self {
            test_coordination_service,
            channel_state_manager,
            test_execution_engine,
            persistence_service,
            report_generation_service,
            app_settings_service,
            test_plc_config_service,
            channel_allocation_service,
            plc_connection_manager,
            plc_monitoring_service,

            // 会话管理：跟踪当前会话中创建的批次
            session_batch_ids: Arc::new(Mutex::new(HashSet::new())),
            session_start_time: Utc::now(),
        })
    }
}

// ============================================================================
// 测试协调相关命令
// ============================================================================

/// 提交测试执行请求
#[tauri::command]
pub async fn submit_test_execution(
    state: State<'_, AppState>,
    request: TestExecutionRequest,
) -> Result<TestExecutionResponse, String> {
    let response = state.test_coordination_service
        .submit_test_execution(request)
        .await
        .map_err(|e| e.to_string())?;

    // 将生成的所有批次ID添加到会话跟踪中
    {
        let mut session_batch_ids = state.session_batch_ids.lock().await;
        for batch in &response.all_batches {
            session_batch_ids.insert(batch.batch_id.clone());
        }
    }

    Ok(response)
}

/// 开始批次测试
#[tauri::command]
pub async fn start_batch_testing(
    state: State<'_, AppState>,
    batch_id: String,
) -> Result<(), String> {
    state.test_coordination_service
        .start_batch_testing(&batch_id)
        .await
        .map_err(|e| e.to_string())
}

/// 暂停批次测试
#[tauri::command]
pub async fn pause_batch_testing(
    state: State<'_, AppState>,
    batch_id: String,
) -> Result<(), String> {
    state.test_coordination_service
        .pause_batch_testing(&batch_id)
        .await
        .map_err(|e| e.to_string())
}

/// 恢复批次测试
#[tauri::command]
pub async fn resume_batch_testing(
    state: State<'_, AppState>,
    batch_id: String,
) -> Result<(), String> {
    state.test_coordination_service
        .resume_batch_testing(&batch_id)
        .await
        .map_err(|e| e.to_string())
}

/// 停止批次测试
#[tauri::command]
pub async fn stop_batch_testing(
    state: State<'_, AppState>,
    batch_id: String,
) -> Result<(), String> {
    state.test_coordination_service
        .stop_batch_testing(&batch_id)
        .await
        .map_err(|e| e.to_string())
}

/// 获取批次测试进度
#[tauri::command]
pub async fn get_batch_progress(
    state: State<'_, AppState>,
    batch_id: String,
) -> Result<Vec<TestProgressUpdate>, String> {
    state.test_coordination_service
        .get_batch_progress(&batch_id)
        .await
        .map_err(|e| e.to_string())
}

/// 获取批次测试结果
#[tauri::command]
pub async fn get_batch_results(
    state: State<'_, AppState>,
    batch_id: String,
) -> Result<Vec<RawTestOutcome>, String> {
    state.test_coordination_service
        .get_batch_results(&batch_id)
        .await
        .map_err(|e| e.to_string())
}

/// 获取当前会话的所有批次信息
#[tauri::command]
pub async fn get_session_batches(
    state: State<'_, AppState>,
) -> Result<Vec<TestBatchInfo>, String> {
    // 获取会话中跟踪的批次ID
    let session_batch_ids = {
        let batch_ids = state.session_batch_ids.lock().await;
        batch_ids.clone()
    };

    // 如果没有跟踪的批次，返回最近的所有批次
    if session_batch_ids.is_empty() {
        state.persistence_service
            .load_all_batch_info()
            .await
            .map_err(|e| e.to_string())
    } else {
        // 获取所有批次，然后筛选出会话中的批次
        let all_batches = state.persistence_service
            .load_all_batch_info()
            .await
            .map_err(|e| e.to_string())?;

        let session_batches = all_batches
            .into_iter()
            .filter(|batch| session_batch_ids.contains(&batch.batch_id))
            .collect();

        Ok(session_batches)
    }
}

/// 清理完成的批次
#[tauri::command]
pub async fn cleanup_completed_batch(
    state: State<'_, AppState>,
    batch_id: String,
) -> Result<(), String> {
    state.test_coordination_service
        .cleanup_completed_batch(&batch_id)
        .await
        .map_err(|e| e.to_string())
}

/// 开始单个通道的硬点测试
#[tauri::command]
pub async fn start_single_channel_test(
    state: State<'_, AppState>,
    instance_id: String,
) -> Result<(), String> {
    state.test_coordination_service
        .start_single_channel_test(&instance_id)
        .await
        .map_err(|e| e.to_string())
}

/// 创建测试数据 - 用于调试批次分配功能
#[tauri::command]
pub async fn create_test_data(
    state: State<'_, AppState>,
) -> Result<Vec<ChannelPointDefinition>, String> {
    use crate::models::{ModuleType, PointDataType};

    log::info!("[CreateTestData] 开始创建测试数据");

    let mut definitions = Vec::new();

    // 创建AI有源通道（4个）
    for i in 1..=4 {
        let mut def = ChannelPointDefinition::new(
            format!("AI{:03}_有源", i),
            format!("Temperature_{}", i),
            format!("温度传感器{} (有源)", i),
            "Station1".to_string(),
            "Module1".to_string(),
            ModuleType::AI,
            format!("CH{:02}", i),
            PointDataType::Float,
            format!("DB1.DBD{}", i * 4),
        );
        def.power_supply_type = "有源".to_string();
        def.range_low_limit = Some(0.0);
        def.range_high_limit = Some(100.0);
        // 不再生成虚拟地址
        def.test_rig_plc_address = None;
        definitions.push(def);
    }

    // 创建AO无源通道（4个）
    for i in 1..=4 {
        let mut def = ChannelPointDefinition::new(
            format!("AO{:03}_无源", i),
            format!("Output_Signal_{}", i),
            format!("输出信号{} (无源)", i),
            "Station1".to_string(),
            "Module2".to_string(),
            ModuleType::AO,
            format!("CH{:02}", i),
            PointDataType::Float,
            format!("DB1.DBD{}", 100 + i * 4),
        );
        def.power_supply_type = "无源".to_string();
        def.range_low_limit = Some(4.0);
        def.range_high_limit = Some(20.0);
        definitions.push(def);
    }

    // 创建DI有源通道（4个）
    for i in 1..=4 {
        let mut def = ChannelPointDefinition::new(
            format!("DI{:03}_有源", i),
            format!("Digital_Input_{}", i),
            format!("数字输入{} (有源)", i),
            "Station2".to_string(),
            "Module3".to_string(),
            ModuleType::DI,
            format!("CH{:02}", i),
            PointDataType::Bool,
            format!("DB3.DBX{}.{}", i / 8, i % 8),
        );
        def.power_supply_type = "有源".to_string();
        def.test_rig_plc_address = Some(format!("DB4.DBX{}.{}", i / 8, i % 8));
        definitions.push(def);
    }

    // 创建DO无源通道（4个）
    for i in 1..=4 {
        let mut def = ChannelPointDefinition::new(
            format!("DO{:03}_无源", i),
            format!("Digital_Output_{}", i),
            format!("数字输出{} (无源)", i),
            "Station2".to_string(),
            "Module4".to_string(),
            ModuleType::DO,
            format!("CH{:02}", i),
            PointDataType::Bool,
            format!("DB5.DBX{}.{}", i / 8, i % 8),
        );
        def.power_supply_type = "无源".to_string();
        definitions.push(def);
    }

    // 创建AI无源通道（4个）
    for i in 1..=4 {
        let mut def = ChannelPointDefinition::new(
            format!("AI{:03}_无源", i + 4),
            format!("Pressure_{}", i),
            format!("压力传感器{} (无源)", i),
            "Station3".to_string(),
            "Module5".to_string(),
            ModuleType::AINone,
            format!("CH{:02}", i),
            PointDataType::Float,
            format!("DB6.DBD{}", i * 4),
        );
        def.power_supply_type = "无源".to_string();
        def.range_low_limit = Some(0.0);
        def.range_high_limit = Some(10.0);
        definitions.push(def);
    }

    log::info!("[CreateTestData] 创建了 {} 个测试通道定义", definitions.len());

    // 保存到数据库
    for def in &definitions {
        if let Err(e) = state.persistence_service.save_channel_definition(def).await {
            log::error!("[CreateTestData] 保存通道定义失败: {} - {}", def.id, e);
        } else {
            log::debug!("[CreateTestData] 保存通道定义成功: {} - {}", def.id, def.tag);
        }
    }

    log::info!("[CreateTestData] 所有测试数据创建完成");
    Ok(definitions)
}

// ============================================================================
// 数据管理相关命令
// ============================================================================

/// 导入Excel文件并解析通道定义
#[tauri::command]
pub async fn import_excel_file(
    file_path: String,
) -> Result<Vec<ChannelPointDefinition>, String> {
    ExcelImporter::parse_excel_file(&file_path)
        .await
        .map_err(|e| e.to_string())
}

/// 创建测试批次并保存通道定义
#[tauri::command]
pub async fn create_test_batch_with_definitions(
    state: State<'_, AppState>,
    batch_info: TestBatchInfo,
    definitions: Vec<ChannelPointDefinition>,
) -> Result<String, String> {
    // 保存批次信息
    state.persistence_service
        .save_batch_info(&batch_info)
        .await
        .map_err(|e| e.to_string())?;

    // 🔥 保存通道定义（设置批次ID）
    let mut updated_definitions = definitions;
    for definition in &mut updated_definitions {
        definition.batch_id = Some(batch_info.batch_id.clone());
    }

    for definition in &updated_definitions {
        state.persistence_service
            .save_channel_definition(definition)
            .await
            .map_err(|e| e.to_string())?;

        // 为每个定义创建测试实例
        let instance = ChannelTestInstance::new(
            definition.id.clone(),
            batch_info.batch_id.clone(),
        );

        state.persistence_service
            .save_test_instance(&instance)
            .await
            .map_err(|e| e.to_string())?;
    }

    Ok(batch_info.batch_id)
}

/// 获取所有通道定义
#[tauri::command]
pub async fn get_all_channel_definitions(
    state: State<'_, AppState>,
) -> Result<Vec<ChannelPointDefinition>, String> {
    state.persistence_service
        .load_all_channel_definitions()
        .await
        .map_err(|e| e.to_string())
}

/// 保存通道定义
#[tauri::command]
pub async fn save_channel_definition(
    state: State<'_, AppState>,
    definition: ChannelPointDefinition,
) -> Result<(), String> {
    state.persistence_service
        .save_channel_definition(&definition)
        .await
        .map_err(|e| e.to_string())
}

/// 删除通道定义
#[tauri::command]
pub async fn delete_channel_definition(
    state: State<'_, AppState>,
    definition_id: String,
) -> Result<(), String> {
    state.persistence_service
        .delete_channel_definition(&definition_id)
        .await
        .map_err(|e| e.to_string())
}

/// 获取所有批次信息
#[tauri::command]
pub async fn get_all_batch_info(
    state: State<'_, AppState>,
) -> Result<Vec<TestBatchInfo>, String> {
    state.persistence_service
        .load_all_batch_info()
        .await
        .map_err(|e| e.to_string())
}

/// 保存批次信息
#[tauri::command]
pub async fn save_batch_info(
    state: State<'_, AppState>,
    batch_info: TestBatchInfo,
) -> Result<(), String> {
    state.persistence_service
        .save_batch_info(&batch_info)
        .await
        .map_err(|e| e.to_string())
}

/// 获取批次测试实例
#[tauri::command]
pub async fn get_batch_test_instances(
    _state: State<'_, AppState>,
    _batch_id: String,
) -> Result<Vec<ChannelTestInstance>, String> {
    // TODO: 实现获取批次测试实例的逻辑
    // 目前返回空列表
    Ok(vec![])
}

// ============================================================================
// 通道状态管理相关命令
// ============================================================================

/// 创建测试实例
#[tauri::command]
pub async fn create_test_instance(
    state: State<'_, AppState>,
    definition_id: String,
    batch_id: String,
) -> Result<ChannelTestInstance, String> {
    state.channel_state_manager
        .create_test_instance(&definition_id, &batch_id)
        .await
        .map_err(|e| e.to_string())
}

/// 获取实例状态
#[tauri::command]
pub async fn get_instance_state(
    state: State<'_, AppState>,
    instance_id: String,
) -> Result<ChannelTestInstance, String> {
    state.channel_state_manager
        .get_instance_state(&instance_id)
        .await
        .map_err(|e| e.to_string())
}

/// 更新测试结果
#[tauri::command]
pub async fn update_test_result(
    state: State<'_, AppState>,
    outcome: RawTestOutcome,
) -> Result<(), String> {
    state.channel_state_manager
        .update_test_result(outcome)
        .await
        .map_err(|e| e.to_string())
}

// ============================================================================
// 系统信息相关命令
// ============================================================================

/// 获取系统状态
#[tauri::command]
pub async fn get_system_status(
    _state: State<'_, AppState>,
) -> Result<SystemStatus, String> {
    Ok(SystemStatus {
        active_test_tasks: 0, // TODO: 从测试执行引擎获取
        system_health: "healthy".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

/// 初始化应用状态
pub async fn init_app_state() -> AppResult<AppState> {
    AppState::new().await
}

// ============================================================================
// 报告生成相关命令
// ============================================================================

/// 生成PDF报告
#[tauri::command]
pub async fn generate_pdf_report(
    state: State<'_, AppState>,
    request: ReportGenerationRequest,
) -> Result<TestReport, String> {
    state.report_generation_service
        .generate_pdf_report(request, "system") // TODO: 从认证系统获取用户ID
        .await
        .map_err(|e| e.to_string())
}

/// 生成Excel报告
#[tauri::command]
pub async fn generate_excel_report(
    state: State<'_, AppState>,
    request: ReportGenerationRequest,
) -> Result<TestReport, String> {
    state.report_generation_service
        .generate_excel_report(request, "system") // TODO: 从认证系统获取用户ID
        .await
        .map_err(|e| e.to_string())
}

/// 获取报告列表
#[tauri::command]
pub async fn get_reports(
    state: State<'_, AppState>,
    batch_id: Option<String>,
) -> Result<Vec<TestReport>, String> {
    state.report_generation_service
        .get_reports(batch_id.as_deref())
        .await
        .map_err(|e| e.to_string())
}

/// 获取报告模板列表
#[tauri::command]
pub async fn get_report_templates(
    state: State<'_, AppState>,
) -> Result<Vec<ReportTemplate>, String> {
    state.report_generation_service
        .get_templates()
        .await
        .map_err(|e| e.to_string())
}

/// 创建报告模板
#[tauri::command]
pub async fn create_report_template(
    state: State<'_, AppState>,
    template: ReportTemplate,
) -> Result<(), String> {
    state.report_generation_service
        .create_template(template)
        .await
        .map_err(|e| e.to_string())
}

/// 更新报告模板
#[tauri::command]
pub async fn update_report_template(
    state: State<'_, AppState>,
    template: ReportTemplate,
) -> Result<(), String> {
    state.report_generation_service
        .update_template(template)
        .await
        .map_err(|e| e.to_string())
}

/// 删除报告模板
#[tauri::command]
pub async fn delete_report_template(
    state: State<'_, AppState>,
    template_id: String,
) -> Result<(), String> {
    state.report_generation_service
        .delete_template(&template_id)
        .await
        .map_err(|e| e.to_string())
}

/// 删除报告
#[tauri::command]
pub async fn delete_report(
    state: State<'_, AppState>,
    report_id: String,
) -> Result<(), String> {
    state.report_generation_service
        .delete_report(&report_id)
        .await
        .map_err(|e| e.to_string())
}

// ============================================================================
// 应用配置相关命令
// ============================================================================

/// 加载应用配置
#[tauri::command]
pub async fn load_app_settings_cmd(
    state: State<'_, AppState>,
) -> Result<AppSettings, String> {
    match state.app_settings_service.load_settings().await {
        Ok(Some(settings)) => Ok(settings),
        Ok(None) => {
            // 如果没有配置文件，返回默认配置
            let default_settings = AppSettings::default();
            // 保存默认配置到文件
            if let Err(e) = state.app_settings_service.save_settings(&default_settings).await {
                log::warn!("保存默认应用配置失败: {}", e);
            }
            Ok(default_settings)
        },
        Err(e) => {
            log::error!("加载应用配置失败: {}", e);
            // 发生错误时也返回默认配置
            Ok(AppSettings::default())
        }
    }
}

/// 保存应用配置
#[tauri::command]
pub async fn save_app_settings_cmd(
    state: State<'_, AppState>,
    settings: AppSettings,
) -> Result<(), String> {
    state.app_settings_service
        .save_settings(&settings)
        .await
        .map_err(|e| {
            log::error!("保存应用配置失败: {}", e);
            format!("保存应用配置失败: {}", e)
        })
}

/// 导出通道分配表
#[derive(Deserialize)]
pub struct ExportChannelAllocationArgs {
    pub target_path: Option<String>,
    pub batch_ids: Option<Vec<String>>, // 可选: 指定导出哪些批次
}

#[tauri::command]
pub async fn export_channel_allocation_cmd(
    state: State<'_, AppState>,
    target_path: Option<String>, // 向后兼容旧调用
    args: Option<ExportChannelAllocationArgs>,
) -> Result<String, String> {
    // 兼容逻辑: 如果前端新版本使用 args 结构体，则覆盖 target_path
    let (real_path_opt, batch_ids_opt) = if let Some(a) = args {
        (a.target_path.or(target_path.clone()), a.batch_ids)
    } else {
        (target_path.clone(), None)
    };

    log::info!("📤 [CMD] 收到导出通道分配表请求, target_path={:?}, batch_ids={:?}", real_path_opt, batch_ids_opt);

    // 计算需要导出的批次ID集合
    let allowed_batch_ids: Option<Vec<String>> = if let Some(list) = batch_ids_opt {
        Some(list)
    } else {
        // 使用当前会话的批次
        let set = state.session_batch_ids.lock().await.clone();
        if set.is_empty() { None } else { Some(set.into_iter().collect()) }
    };

    let service = crate::infrastructure::excel_export_service::ExcelExportService::new(
        state.persistence_service.clone(),
        state.channel_state_manager.clone(),
    );

    let path_buf = real_path_opt.map(PathBuf::from);
    match service.export_channel_allocation_with_filter(path_buf, allowed_batch_ids).await {
        Ok(result_path) => {
            log::info!("✅ [CMD] 通道分配表导出成功: {}", result_path);
            Ok(result_path)
        },
        Err(e) => {
            log::error!("❌ [CMD] 通道分配表导出失败: {}", e);
            Err(e.to_string())
        }
    }
}
