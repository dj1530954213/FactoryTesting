/// Tauri 命令模块
/// 
/// 这个模块定义了所有前端可以调用的 Tauri 命令
/// 将后端服务暴露给前端 Angular 应用

use crate::models::{
    ChannelPointDefinition, ChannelTestInstance, TestBatchInfo, RawTestOutcome,
    TestReport, ReportTemplate, ReportGenerationRequest
};
use crate::services::application::{
    ITestCoordinationService, TestCoordinationService,
    TestExecutionRequest, TestExecutionResponse, TestProgressUpdate,
    IReportGenerationService, ReportGenerationService
};
use crate::services::domain::{
    IChannelStateManager, ChannelStateManager,
    ITestExecutionEngine, TestExecutionEngine
};
use crate::services::infrastructure::{
    IPersistenceService, SqliteOrmPersistenceService,
    IPlcCommunicationService, MockPlcService,
    excel::ExcelImporter
};
use crate::utils::error::{AppError, AppResult};
use std::sync::Arc;
use tokio::sync::Mutex;
use serde::{Serialize, Deserialize};
use tauri::State;

/// 应用状态，包含所有服务实例
pub struct AppState {
    pub test_coordination_service: Arc<dyn ITestCoordinationService>,
    pub channel_state_manager: Arc<dyn IChannelStateManager>,
    pub test_execution_engine: Arc<dyn ITestExecutionEngine>,
    pub persistence_service: Arc<dyn IPersistenceService>,
    pub report_generation_service: Arc<dyn IReportGenerationService>,
}

impl AppState {
    /// 创建新的应用状态
    pub async fn new() -> AppResult<Self> {
        // 获取当前工作目录并构建数据库路径
        let current_dir = std::env::current_dir()
            .map_err(|e| AppError::io_error("获取当前目录失败".to_string(), e.to_string()))?;
        
        // 构建数据目录路径 (在 src-tauri 的上级目录)
        let data_dir = current_dir.parent()
            .ok_or_else(|| AppError::io_error("无法获取父目录".to_string(), "路径错误".to_string()))?
            .join("data");
        
        // 确保数据目录存在
        if !data_dir.exists() {
            std::fs::create_dir_all(&data_dir)
                .map_err(|e| AppError::io_error(
                    format!("创建数据目录失败: {:?}", data_dir), 
                    e.to_string()
                ))?;
        }
        
        // 构建数据库文件路径
        let db_path = data_dir.join("factory_testing.db");
        
        println!("数据库路径: {:?}", db_path); // 调试信息
        println!("数据目录存在: {}", data_dir.exists());
        println!("数据库文件存在: {}", db_path.exists());
        
        // 创建基础设施服务
        let config = crate::services::infrastructure::persistence::PersistenceConfig::default();
        
        // 暂时使用内存数据库进行测试
        println!("使用内存数据库进行测试...");
        let persistence_service = Arc::new(
            SqliteOrmPersistenceService::new(config, Some(std::path::Path::new(":memory:"))).await?
        );
        
        let plc_service_test_rig = Arc::new(MockPlcService::new("TestRig"));
        let plc_service_target = Arc::new(MockPlcService::new("Target"));
        
        // 创建领域服务
        let channel_state_manager = Arc::new(
            ChannelStateManager::new(persistence_service.clone())
        );
        
        let test_execution_engine = Arc::new(
            TestExecutionEngine::new(
                5, // 最大并发测试数
                plc_service_test_rig,
                plc_service_target,
            )
        );
        
        // 创建应用服务
        let test_coordination_service = Arc::new(
            TestCoordinationService::new(
                channel_state_manager.clone(),
                test_execution_engine.clone(),
                persistence_service.clone(),
            )
        );
        
        // 创建报告目录
        let reports_dir = data_dir.join("reports");
        let report_generation_service = Arc::new(
            ReportGenerationService::new(
                persistence_service.clone(),
                reports_dir,
            )?
        );
        
        Ok(Self {
            test_coordination_service,
            channel_state_manager,
            test_execution_engine,
            persistence_service,
            report_generation_service,
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
    state: State<'_, AppState>,
    batch_id: String,
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

/// 系统状态信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemStatus {
    pub active_test_tasks: usize,
    pub system_health: String,
    pub version: String,
}

/// 获取系统状态
#[tauri::command]
pub async fn get_system_status(
    state: State<'_, AppState>,
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