use std::sync::Arc;
use tauri::State;
use log::{info, error, warn};

use crate::models::structs::{
    StartManualTestRequest,
    StartManualTestResponse,
    UpdateManualTestSubItemRequest,
    UpdateManualTestSubItemResponse,
    StartPlcMonitoringRequest,
    StartPlcMonitoringResponse,
    StopPlcMonitoringRequest,
    ManualTestStatus,
};
use crate::services::application::ITestOrchestrationService;
use crate::services::infrastructure::IPlcMonitoringService;
use crate::errors::AppError;

/// 开始手动测试命令
#[tauri::command]
pub async fn start_manual_test_cmd(
    request: StartManualTestRequest,
    orchestration_service: State<'_, Arc<dyn ITestOrchestrationService>>,
) -> Result<StartManualTestResponse, String> {
    info!("🔧 [MANUAL_TEST_CMD] 开始手动测试: {:?}", request);

    match orchestration_service.start_manual_test(request).await {
        Ok(response) => {
            info!("✅ [MANUAL_TEST_CMD] 手动测试启动成功");
            Ok(response)
        }
        Err(e) => {
            error!("❌ [MANUAL_TEST_CMD] 手动测试启动失败: {}", e);
            Err(format!("启动手动测试失败: {}", e))
        }
    }
}

/// 更新手动测试子项状态命令
#[tauri::command]
pub async fn update_manual_test_subitem_cmd(
    request: UpdateManualTestSubItemRequest,
    orchestration_service: State<'_, Arc<dyn ITestOrchestrationService>>,
) -> Result<UpdateManualTestSubItemResponse, String> {
    info!("🔧 [MANUAL_TEST_CMD] 更新手动测试子项: {:?}", request);

    match orchestration_service.update_manual_test_subitem(request).await {
        Ok(response) => {
            info!("✅ [MANUAL_TEST_CMD] 手动测试子项更新成功");
            Ok(response)
        }
        Err(e) => {
            error!("❌ [MANUAL_TEST_CMD] 手动测试子项更新失败: {}", e);
            Err(format!("更新手动测试子项失败: {}", e))
        }
    }
}

/// 获取手动测试状态命令
#[tauri::command]
pub async fn get_manual_test_status_cmd(
    instance_id: String,
    orchestration_service: State<'_, Arc<dyn ITestOrchestrationService>>,
) -> Result<serde_json::Value, String> {
    info!("🔧 [MANUAL_TEST_CMD] 获取手动测试状态: {}", instance_id);

    match orchestration_service.get_manual_test_status(&instance_id).await {
        Ok(status) => {
            info!("✅ [MANUAL_TEST_CMD] 获取手动测试状态成功");
            Ok(serde_json::json!({
                "success": true,
                "testStatus": status
            }))
        }
        Err(e) => {
            error!("❌ [MANUAL_TEST_CMD] 获取手动测试状态失败: {}", e);
            Err(format!("获取手动测试状态失败: {}", e))
        }
    }
}

/// 开始PLC监控命令
#[tauri::command]
pub async fn start_plc_monitoring_cmd(
    request: StartPlcMonitoringRequest,
    monitoring_service: State<'_, Arc<dyn IPlcMonitoringService>>,
) -> Result<StartPlcMonitoringResponse, String> {
    info!("🔧 [MANUAL_TEST_CMD] 开始PLC监控: {:?}", request);

    match monitoring_service.start_monitoring(request).await {
        Ok(response) => {
            info!("✅ [MANUAL_TEST_CMD] PLC监控启动成功");
            Ok(response)
        }
        Err(e) => {
            error!("❌ [MANUAL_TEST_CMD] PLC监控启动失败: {}", e);
            Err(format!("启动PLC监控失败: {}", e))
        }
    }
}

/// 停止PLC监控命令
#[tauri::command]
pub async fn stop_plc_monitoring_cmd(
    request: StopPlcMonitoringRequest,
    monitoring_service: State<'_, Arc<dyn IPlcMonitoringService>>,
) -> Result<serde_json::Value, String> {
    info!("🔧 [MANUAL_TEST_CMD] 停止PLC监控: {:?}", request);

    match monitoring_service.stop_monitoring(request).await {
        Ok(_) => {
            info!("✅ [MANUAL_TEST_CMD] PLC监控停止成功");
            Ok(serde_json::json!({
                "success": true,
                "message": "PLC监控已停止"
            }))
        }
        Err(e) => {
            error!("❌ [MANUAL_TEST_CMD] PLC监控停止失败: {}", e);
            Err(format!("停止PLC监控失败: {}", e))
        }
    }
}
