/// Tauri 命令模块
/// 
/// 这个模块定义了所有前端可以调用的 Tauri 命令
/// 将后端服务暴露给前端 Angular 应用

use crate::models::{
    ChannelPointDefinition, ChannelTestInstance, TestBatchInfo, RawTestOutcome
};
use crate::services::application::{
    ITestCoordinationService, TestCoordinationService,
    TestExecutionRequest, TestExecutionResponse, TestProgressUpdate
};
use crate::services::domain::{
    IChannelStateManager, ChannelStateManager,
    ITestExecutionEngine, TestExecutionEngine
};
use crate::services::infrastructure::{
    IPersistenceService, SqliteOrmPersistenceService,
    IPlcCommunicationService, MockPlcService
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
}

impl AppState {
    /// 创建新的应用状态
    pub async fn new() -> AppResult<Self> {
        // 创建基础设施服务
        let persistence_service = Arc::new(
            SqliteOrmPersistenceService::new("./data/factory_testing.db").await?
        );
        
        let plc_service_test_rig = Arc::new(MockPlcService::new());
        let plc_service_target = Arc::new(MockPlcService::new());
        
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
        
        Ok(Self {
            test_coordination_service,
            channel_state_manager,
            test_execution_engine,
            persistence_service,
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

/// 获取所有通道定义
#[tauri::command]
pub async fn get_all_channel_definitions(
    state: State<'_, AppState>,
) -> Result<Vec<ChannelPointDefinition>, String> {
    state.persistence_service
        .get_all_channel_definitions()
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
        .get_all_batch_info()
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

/// 获取批次的所有测试实例
#[tauri::command]
pub async fn get_batch_test_instances(
    state: State<'_, AppState>,
    batch_id: String,
) -> Result<Vec<ChannelTestInstance>, String> {
    state.persistence_service
        .get_test_instances_by_batch(&batch_id)
        .await
        .map_err(|e| e.to_string())
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
        .create_test_instance(definition_id, batch_id)
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
    state: State<'_, AppState>,
) -> Result<SystemStatus, String> {
    let active_tasks = state.test_execution_engine.get_active_task_count().await;
    
    Ok(SystemStatus {
        active_test_tasks: active_tasks,
        system_health: "healthy".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

/// 系统状态信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemStatus {
    pub active_test_tasks: usize,
    pub system_health: String,
    pub version: String,
}

// ============================================================================
// 工具函数
// ============================================================================

/// 初始化应用状态
pub async fn init_app_state() -> AppResult<AppState> {
    AppState::new().await
}

/// 获取所有可用的 Tauri 命令
pub fn get_tauri_commands() -> impl Fn(tauri::Invoke) + Send + Sync + 'static {
    tauri::generate_handler![
        // 测试协调命令
        submit_test_execution,
        start_batch_testing,
        pause_batch_testing,
        resume_batch_testing,
        stop_batch_testing,
        get_batch_progress,
        get_batch_results,
        cleanup_completed_batch,
        // 数据管理命令
        get_all_channel_definitions,
        save_channel_definition,
        delete_channel_definition,
        get_all_batch_info,
        save_batch_info,
        get_batch_test_instances,
        // 通道状态管理命令
        create_test_instance,
        get_instance_state,
        update_test_result,
        // 系统信息命令
        get_system_status,
    ]
} 