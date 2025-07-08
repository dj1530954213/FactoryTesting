use tauri::State;
use std::sync::Arc;
use crate::tauri_commands::AppState;
use crate::models::structs::{GlobalFunctionTestStatus, GlobalFunctionKey};
use crate::utils::error::AppResult;
use log::{info, error};
use crate::models::structs::default_id;
use serde::{Serialize, Deserialize};

/// 获取全部全局功能测试状态
#[tauri::command]
pub async fn get_global_function_tests_cmd(
    station_name: String,
    import_time: String,
    app_state: State<'_, AppState>,
) -> Result<Vec<GlobalFunctionTestStatus>, String> {
    // 优先从缓存获取
    {
        let guard = app_state.global_function_tests.lock().await;
        if guard.iter().any(|s| s.station_name == station_name && s.import_time == import_time) {
            return Ok(guard.iter().filter(|s| s.station_name == station_name && s.import_time == import_time).cloned().collect());
        }
    }

    // 缓存没有，尝试从数据库加载
    match app_state.persistence_service.load_global_function_test_statuses_by_station_time(&station_name, &import_time).await {
        Ok(db_statuses) => {
            if !db_statuses.is_empty() {
                let mut guard = app_state.global_function_tests.lock().await;
                guard.extend(db_statuses.clone());
                return Ok(db_statuses);
            }
        }
        Err(e) => {
            error!("加载数据库全局功能测试状态失败: {}", e);
        }
    }
    // 如果按导入时间未找到记录，则尝试回退到该站场最新的一批记录（按 import_time 降序）
    if let Ok(all_by_station) = app_state.persistence_service.load_global_function_test_statuses_by_station(&station_name).await {
        if !all_by_station.is_empty() {
            // 找到最新 import_time
            if let Some(latest_time) = all_by_station.iter().map(|s| s.import_time.clone()).max() {
                let latest_records: Vec<_> = all_by_station.into_iter().filter(|s| s.import_time == latest_time).collect();
                if !latest_records.is_empty() {
                    // 更新内存缓存（追加）
                    let mut guard = app_state.global_function_tests.lock().await;
                    guard.extend(latest_records.clone());
                    return Ok(latest_records);
                }
            }
        }
    }
    Ok(vec![])
}

/// 更新单个全局功能测试状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateGlobalFunctionTestRequest {
    pub station_name: String,
    pub import_time: String,
    pub function_key: GlobalFunctionKey,
    pub status: crate::models::enums::OverallTestStatus,
    pub start_time: Option<String>,
    pub end_time: Option<String>,
}

#[tauri::command]
pub async fn update_global_function_test_cmd(
    request: UpdateGlobalFunctionTestRequest,
    app_state: State<'_, AppState>,
) -> Result<GlobalFunctionTestStatus, String> {
    info!("🔧 [GFT_CMD] 更新全局功能测试状态: station='{}' key={:?} status={:?}", request.station_name, request.function_key, request.status);

    // 找到并更新缓存
    let mut statuses_guard = app_state.global_function_tests.lock().await;
    if let Some(item) = statuses_guard.iter_mut().find(|s| s.station_name == request.station_name && s.import_time == request.import_time && s.function_key == request.function_key) {
        item.status = request.status;
        item.start_time = request.start_time.clone();
        item.end_time = request.end_time.clone();

        // 同步到数据库
        if let Err(e) = app_state.persistence_service.save_global_function_test_status(item).await {
            error!("❌ [GFT_CMD] 保存数据库失败: {}", e);
            return Err(format!("保存失败: {}", e));
        }
        return Ok(item.clone());
    }
    // 如果未找到则创建新记录（防御性）
    let mut new_item = GlobalFunctionTestStatus {
        id: default_id(),
        station_name: request.station_name.clone(),
        import_time: request.import_time.clone(),
        function_key: request.function_key,
        status: request.status,
        start_time: request.start_time.clone(),
        end_time: request.end_time.clone(),
    };
    if let Err(e) = app_state.persistence_service.save_global_function_test_status(&new_item).await {
        error!("❌ [GFT_CMD] 插入数据库失败: {}", e);
    }
    statuses_guard.push(new_item.clone());
    Ok(new_item)
}

/// 重置全部全局功能测试状态
#[tauri::command]
pub async fn reset_global_function_tests_cmd(
    station_name: String,
    import_time: String,
    app_state: State<'_, AppState>,
) -> Result<Vec<GlobalFunctionTestStatus>, String> {
    info!("🔧 [GFT_CMD] 重置全局功能测试状态");

    if let Err(e) = app_state.persistence_service.reset_global_function_test_statuses_by_station(&station_name).await {
        // TODO: 支持按导入时间重置，如有需要可扩展
        error!("❌ [GFT_CMD] 重置数据库失败: {}", e);
        return Err(format!("重置失败: {}", e));
    }

    // 重新生成默认记录
    let mut new_statuses = Vec::new();
    use crate::models::enums::OverallTestStatus;
    for key in [
        GlobalFunctionKey::HistoricalTrend,
        GlobalFunctionKey::RealTimeTrend,
        GlobalFunctionKey::Report,
        GlobalFunctionKey::AlarmLevelSound,
        GlobalFunctionKey::OperationLog,
    ] {
        let status = GlobalFunctionTestStatus {
            station_name: station_name.clone(),
            import_time: import_time.clone(),
            id: default_id(),
            function_key: key,
            start_time: None,
            end_time: None,
            status: OverallTestStatus::NotTested,
        };
        // 保存
        if let Err(e) = app_state.persistence_service.save_global_function_test_status(&status).await {
            error!("❌ [GFT_CMD] 保存默认记录失败: {}", e);
        }
        new_statuses.push(status);
    }

    // 更新缓存
    let mut guard = app_state.global_function_tests.lock().await;
    // 先移除旧站场记录
    guard.retain(|s| !(s.station_name == station_name && s.import_time == import_time));
    guard.extend(new_statuses.clone());

    Ok(new_statuses)
}
