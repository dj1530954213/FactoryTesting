/// Tauri 命令模块
/// 
/// 这个模块定义了所有前端可以调用的 Tauri 命令
/// 将后端服务暴露给前端 Angular 应用

use crate::models::{
    ChannelPointDefinition, TestBatchInfo, ChannelTestInstance, RawTestOutcome,
    TestReport, ReportTemplate, ReportGenerationRequest, AppSettings
};
use crate::services::application::{
    ITestCoordinationService, TestCoordinationService,
    TestExecutionRequest, TestExecutionResponse, TestProgressUpdate,
    IReportGenerationService, ReportGenerationService
};
use crate::services::domain::{
    IChannelStateManager, ChannelStateManager,
    ITestExecutionEngine, TestExecutionEngine,
    ITestPlcConfigService, TestPlcConfigService
};
use crate::services::infrastructure::{
    IPersistenceService, SqliteOrmPersistenceService,
    IPlcCommunicationService, MockPlcService,
    excel::ExcelImporter,
    persistence::{AppSettingsService, JsonAppSettingsService, AppSettingsConfig}
};
use crate::utils::error::{AppError, AppResult};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::State;
use tokio::sync::Mutex;

// ============================================================================
// 应用状态管理
// ============================================================================

/// 应用状态，包含所有服务实例
pub struct AppState {
    pub test_coordination_service: Arc<dyn ITestCoordinationService>,
    pub channel_state_manager: Arc<dyn IChannelStateManager>,
    pub test_execution_engine: Arc<dyn ITestExecutionEngine>,
    pub persistence_service: Arc<dyn IPersistenceService>,
    pub report_generation_service: Arc<dyn IReportGenerationService>,
    pub app_settings_service: Arc<dyn AppSettingsService>,
    pub test_plc_config_service: Arc<dyn ITestPlcConfigService>,
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
        let config = crate::services::infrastructure::persistence::PersistenceConfig::default();

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

        // 创建通道状态管理器
        let channel_state_manager: Arc<dyn IChannelStateManager> = Arc::new(
            ChannelStateManager::new(persistence_service.clone())
        );

        // 创建PLC服务
        let plc_service_test_rig: Arc<dyn IPlcCommunicationService> = Arc::new(MockPlcService::new("TestRig"));
        let plc_service_target: Arc<dyn IPlcCommunicationService> = Arc::new(MockPlcService::new("Target"));

        // 创建测试执行引擎
        let test_execution_engine: Arc<dyn ITestExecutionEngine> = Arc::new(
            TestExecutionEngine::new(
                10, // 最大并发测试数
                plc_service_test_rig,
                plc_service_target,
            )
        );

        // 创建测试协调服务
        let test_coordination_service: Arc<dyn ITestCoordinationService> = Arc::new(
            TestCoordinationService::new(
                channel_state_manager.clone(),
                test_execution_engine.clone(),
                persistence_service.clone(),
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

        // 创建测试PLC配置服务
        let test_plc_config_service: Arc<dyn ITestPlcConfigService> = Arc::new(
            TestPlcConfigService::new(persistence_service.clone())
        );

        Ok(Self {
            test_coordination_service,
            channel_state_manager,
            test_execution_engine,
            persistence_service,
            report_generation_service,
            app_settings_service,
            test_plc_config_service,
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
    state.test_coordination_service
        .submit_test_execution(request)
        .await
        .map_err(|e| e.to_string())
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
    
    // 保存通道定义
    for definition in definitions {
        state.persistence_service
            .save_channel_definition(&definition)
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
    instance_id: String,
    outcome: RawTestOutcome,
) -> Result<(), String> {
    state.channel_state_manager
        .update_test_result(&instance_id, outcome)
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