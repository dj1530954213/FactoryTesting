/// 全局功能测试命令模块
/// 
/// 业务说明：
/// 全局功能测试是指系统级别的功能测试，区别于单个通道的测试
/// 主要包括：历史趋势、实时趋势、报表、报警声音、操作日志等功能
/// 这些功能通常需要整个系统协同工作，无法通过单个通道测试验证
/// 
/// 架构定位：
/// - 位于接口层(interfaces)，负责处理前端请求
/// - 通过应用状态(AppState)访问缓存和持久化服务
/// - 支持按站场名称和导入时间管理测试状态
/// 
/// 调用链：
/// 前端全局功能测试界面 -> 这些命令 -> PersistenceService -> 数据库

use tauri::State;
use std::sync::Arc;
use crate::tauri_commands::AppState;
use crate::models::structs::{GlobalFunctionTestStatus, GlobalFunctionKey};
use crate::utils::error::AppResult;
use log::{info, error};
use crate::models::structs::default_id;
use serde::{Serialize, Deserialize};

/// 获取全部全局功能测试状态
/// 
/// 业务说明：
/// - 获取指定站场和导入时间的所有全局功能测试状态
/// - 优先从内存缓存获取，提高性能
/// - 缓存未命中时从数据库加载
/// - 支持回退到站场最新一批记录（容错机制）
/// 
/// 参数：
/// - station_name: 站场名称
/// - import_time: 导入时间，格式为 ISO 8601
/// - app_state: 应用状态，包含缓存和持久化服务
/// 
/// 返回：
/// - Ok: 全局功能测试状态列表
/// - Err: 错误信息
/// 
/// 调用链：
/// 前端测试界面 -> get_global_function_tests_cmd -> 内存缓存/数据库
/// 
/// Rust知识点：
/// - #[tauri::command] 标记为Tauri命令
/// - async/await 异步编程
/// - 作用域块 {} 用于限制锁的生命周期，避免死锁
#[tauri::command]
pub async fn get_global_function_tests_cmd(
    station_name: String,
    import_time: String,
    app_state: State<'_, AppState>,
) -> Result<Vec<GlobalFunctionTestStatus>, String> {
    // 优先从缓存获取
    // Rust知识点：使用作用域块限制锁的生命周期
    {
        let guard = app_state.global_function_tests.lock().await;
        // 检查缓存中是否有匹配的记录
        if guard.iter().any(|s| s.station_name == station_name && s.import_time == import_time) {
            // Rust知识点：filter + cloned + collect 组合用于筛选并克隆元素
            return Ok(guard.iter().filter(|s| s.station_name == station_name && s.import_time == import_time).cloned().collect());
        }
    }

    // 缓存没有，尝试从数据库加载
    // 业务说明：按站场名称和导入时间精确查询
    match app_state.persistence_service.load_global_function_test_statuses_by_station_time(&station_name, &import_time).await {
        Ok(db_statuses) => {
            if !db_statuses.is_empty() {
                // 将数据库记录加载到缓存
                let mut guard = app_state.global_function_tests.lock().await;
                // Rust知识点：extend 方法用于批量添加元素到集合
                guard.extend(db_statuses.clone());
                return Ok(db_statuses);
            }
        }
        Err(e) => {
            error!("加载数据库全局功能测试状态失败: {}", e);
        }
    }
    
    // 如果按导入时间未找到记录，则尝试回退到该站场最新的一批记录（按 import_time 降序）
    // 业务说明：这是一个容错机制，当精确时间匹配失败时，使用最新的测试状态
    if let Ok(all_by_station) = app_state.persistence_service.load_global_function_test_statuses_by_station(&station_name).await {
        if !all_by_station.is_empty() {
            // 找到最新 import_time
            // Rust知识点：map + max 组合找出最大值
            if let Some(latest_time) = all_by_station.iter().map(|s| s.import_time.clone()).max() {
                // 筛选出最新时间的所有记录
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
    
    // 返回空列表表示没有找到任何记录
    Ok(vec![])
}

/// 更新单个全局功能测试状态
/// 
/// 业务说明：
/// 用于更新单个全局功能的测试状态
/// 
/// Rust知识点：
/// - #[derive] 自动实现指定的trait
/// - Debug: 用于调试输出
/// - Clone: 允许值被克隆
/// - Serialize/Deserialize: 支持JSON序列化
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateGlobalFunctionTestRequest {
    pub station_name: String,                                // 站场名称
    pub import_time: String,                                 // 导入时间
    pub function_key: GlobalFunctionKey,                     // 功能键（如历史趋势、实时趋势等）
    pub status: crate::models::enums::OverallTestStatus,    // 测试状态
    pub start_time: Option<String>,                          // 开始时间（可选）
    pub end_time: Option<String>,                            // 结束时间（可选）
}

/// 更新全局功能测试状态命令
/// 
/// 业务说明：
/// - 更新指定全局功能的测试状态（通过、失败、未测试等）
/// - 同时更新内存缓存和数据库
/// - 如果记录不存在则创建新记录（防御性编程）
/// 
/// 参数：
/// - request: 更新请求，包含站场、时间、功能键和新状态
/// - app_state: 应用状态
/// 
/// 返回：
/// - Ok: 更新后的全局功能测试状态
/// - Err: 错误信息
/// 
/// 调用链：
/// 前端测试执行界面 -> update_global_function_test_cmd -> 内存缓存 -> 数据库
#[tauri::command]
pub async fn update_global_function_test_cmd(
    request: UpdateGlobalFunctionTestRequest,
    app_state: State<'_, AppState>,
) -> Result<GlobalFunctionTestStatus, String> {
    info!("🔧 [GFT_CMD] 更新全局功能测试状态: station='{}' key={:?} status={:?}", request.station_name, request.function_key, request.status);

    // 找到并更新缓存
    let mut statuses_guard = app_state.global_function_tests.lock().await;
    // Rust知识点：iter_mut() 获取可变迭代器，允许修改元素
    if let Some(item) = statuses_guard.iter_mut().find(|s| s.station_name == request.station_name && s.import_time == request.import_time && s.function_key == request.function_key) {
        // 更新状态和时间戳
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
    // 业务说明：这种情况不应该发生，但为了健壮性还是处理
    let mut new_item = GlobalFunctionTestStatus {
        id: default_id(),                             // 生成新的UUID
        station_name: request.station_name.clone(),
        import_time: request.import_time.clone(),
        function_key: request.function_key,
        status: request.status,
        start_time: request.start_time.clone(),
        end_time: request.end_time.clone(),
    };
    
    // 保存到数据库
    if let Err(e) = app_state.persistence_service.save_global_function_test_status(&new_item).await {
        error!("❌ [GFT_CMD] 插入数据库失败: {}", e);
    }
    
    // 添加到缓存
    statuses_guard.push(new_item.clone());
    Ok(new_item)
}

/// 重置全部全局功能测试状态
/// 
/// 业务说明：
/// - 将指定站场的所有全局功能测试状态重置为"未测试"
/// - 清除旧记录并创建新的默认记录
/// - 用于重新开始测试或清理测试数据
/// 
/// 参数：
/// - station_name: 站场名称
/// - import_time: 导入时间
/// - app_state: 应用状态
/// 
/// 返回：
/// - Ok: 重置后的全局功能测试状态列表
/// - Err: 错误信息
/// 
/// 调用链：
/// 前端重置按钮 -> reset_global_function_tests_cmd -> 数据库 -> 重新生成默认记录
/// 
/// Rust知识点：
/// - Vec<T> 动态数组
/// - for 循环遍历数组
#[tauri::command]
pub async fn reset_global_function_tests_cmd(
    station_name: String,
    import_time: String,
    app_state: State<'_, AppState>,
) -> Result<Vec<GlobalFunctionTestStatus>, String> {
    info!("🔧 [GFT_CMD] 重置全局功能测试状态");

    // 重置数据库中的记录
    if let Err(e) = app_state.persistence_service.reset_global_function_test_statuses_by_station(&station_name).await {
        // TODO: 支持按导入时间重置，如有需要可扩展
        error!("❌ [GFT_CMD] 重置数据库失败: {}", e);
        return Err(format!("重置失败: {}", e));
    }

    // 重新生成默认记录
    let mut new_statuses = Vec::new();
    use crate::models::enums::OverallTestStatus;
    
    // 遍历所有全局功能类型，为每个功能创建默认记录
    // 业务说明：这5个功能是系统必须测试的全局功能
    for key in [
        GlobalFunctionKey::HistoricalTrend,    // 历史趋势
        GlobalFunctionKey::RealTimeTrend,      // 实时趋势
        GlobalFunctionKey::Report,             // 报表功能
        GlobalFunctionKey::AlarmLevelSound,    // 报警声音
        GlobalFunctionKey::OperationLog,       // 操作日志
    ] {
        let status = GlobalFunctionTestStatus {
            station_name: station_name.clone(),
            import_time: import_time.clone(),
            id: default_id(),                      // 生成唯一ID
            function_key: key,
            start_time: None,
            end_time: None,
            status: OverallTestStatus::NotTested,  // 初始状态为未测试
        };
        
        // 保存到数据库
        if let Err(e) = app_state.persistence_service.save_global_function_test_status(&status).await {
            error!("❌ [GFT_CMD] 保存默认记录失败: {}", e);
        }
        new_statuses.push(status);
    }

    // 更新缓存
    let mut guard = app_state.global_function_tests.lock().await;
    // 先移除旧站场记录
    // Rust知识点：retain 方法保留满足条件的元素，移除不满足条件的元素
    guard.retain(|s| !(s.station_name == station_name && s.import_time == import_time));
    // 添加新记录
    guard.extend(new_statuses.clone());

    Ok(new_statuses)
}
