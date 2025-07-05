use std::sync::Arc;
use tauri::State;
use log::{info, error, warn};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use rand::Rng;

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
// 注意：ManualTestSubItem 需要在 models 中定义
// 暂时使用字符串代替，后续需要定义正确的枚举
use crate::application::services::ITestCoordinationService;
use crate::infrastructure::IPlcMonitoringService;
use crate::domain::services::plc_comm_extension::PlcServiceLegacyExt;
use crate::infrastructure::plc_communication::IPlcCommunicationService;
use crate::domain::services::plc_communication_service::{PlcConnectionConfig, PlcProtocol};
use crate::infrastructure::plc_communication::global_plc_service;
//use crate::infrastructure::extra::infrastructure::plc::plc_communication_service::PlcCommunicationService;

/// 开始手动测试命令
#[tauri::command]
pub async fn start_manual_test_cmd(
    request: StartManualTestRequest,
    app_state: State<'_, crate::tauri_commands::AppState>,
) -> Result<StartManualTestResponse, String> {
    info!("🔧 [MANUAL_TEST_CMD] 开始手动测试: {:?}", request);

    match app_state.test_coordination_service.start_manual_test(request).await {
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
    app_state: State<'_, crate::tauri_commands::AppState>,
) -> Result<UpdateManualTestSubItemResponse, String> {
    info!("🔧 [MANUAL_TEST_CMD] 更新手动测试子项: {:?}", request);

    match app_state.test_coordination_service.update_manual_test_subitem(request).await {
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
    app_state: State<'_, crate::tauri_commands::AppState>,
) -> Result<serde_json::Value, String> {
    info!("🔧 [MANUAL_TEST_CMD] 获取手动测试状态: {}", instance_id);

    match app_state.test_coordination_service.get_manual_test_status(&instance_id).await {
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
    mut request: StartPlcMonitoringRequest,
    app_state: State<'_, crate::tauri_commands::AppState>,
) -> Result<StartPlcMonitoringResponse, String> {
    // 根据模块类型补充 connection_id
    if request.connection_id.is_none() {
        let conn_id = match request.module_type {
            crate::models::enums::ModuleType::AI | crate::models::enums::ModuleType::DI |
            crate::models::enums::ModuleType::DINone | crate::models::enums::ModuleType::AINone => app_state.target_connection_id.clone(),
            _ => app_state.test_rig_connection_id.clone(),
        };
        request.connection_id = Some(conn_id);
    }

    info!("🔧 [MANUAL_TEST_CMD] 开始PLC监控: {:?}", request);

    match app_state.plc_monitoring_service.start_monitoring(request).await {
        Ok(response) => {
            // 获取当前默认PLC连接地址（IP:PORT），便于排查使用了哪台PLC
            if let Some(addr) = global_plc_service().last_default_address().await {
                info!("✅ [MANUAL_TEST_CMD] PLC监控启动成功: {}", addr);
            } else {
                info!("✅ [MANUAL_TEST_CMD] PLC监控启动成功 (当前无活动PLC连接)");
            }
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
    app_state: State<'_, crate::tauri_commands::AppState>,
) -> Result<serde_json::Value, String> {
    info!("🔧 [MANUAL_TEST_CMD] 停止PLC监控: {:?}", request);

    match app_state.plc_monitoring_service.stop_monitoring(request).await {
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

// ==================== AI手动测试专用命令 ====================

/// AI手动测试显示值核对请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiShowValueTestRequest {
    pub instance_id: String,
    pub test_value: f64,  // 用户输入或随机生成的测试值（工程值）
}

/// AI手动测试显示值核对响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiShowValueTestResponse {
    pub success: bool,
    pub message: String,
    pub sent_percentage: f64,  // 发送到测试PLC的百分比值
    pub test_plc_address: String,  // 测试PLC通信地址
}

/// AI手动测试报警测试请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiAlarmTestRequest {
    pub instance_id: String,
    pub alarm_type: String,  // "LL", "L", "H", "HH"
}

/// AI手动测试报警测试响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiAlarmTestResponse {
    pub success: bool,
    pub message: String,
    pub sent_value: f64,      // 发送的工程值
    pub sent_percentage: f64, // 发送到测试PLC的百分比值
    pub test_plc_address: String,
}

/// AI手动测试维护功能请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiMaintenanceTestRequest {
    pub instance_id: String,
    pub enable: bool,  // true=启用维护, false=复位
}

/// AI手动测试维护功能响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiMaintenanceTestResponse {
    pub success: bool,
    pub message: String,
    pub maintenance_address: String,  // 维护使能开关地址
}

/// 生成随机显示值请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateRandomValueRequest {
    pub instance_id: String,
}

/// 生成随机显示值响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateRandomValueResponse {
    pub success: bool,
    pub message: String,
    pub random_value: f64,  // 生成的随机工程值
    pub low_limit: f64,     // 低限
    pub high_limit: f64,    // 高限
}

/// AI手动测试 - 生成随机显示值
#[tauri::command]
pub async fn generate_random_display_value_cmd(
    request: GenerateRandomValueRequest,
    app_state: State<'_, crate::tauri_commands::AppState>,
) -> Result<GenerateRandomValueResponse, String> {
    info!("🎯 [AI_MANUAL_TEST] 点击生成随机显示值按钮: {}", request.instance_id);

    // 获取通道定义信息
    let instance = match app_state.persistence_service.load_test_instance(&request.instance_id).await {
        Ok(Some(instance)) => instance,
        Ok(None) => {
            error!("❌ [AI_MANUAL_TEST] 未找到测试实例: {}", request.instance_id);
            return Err("未找到指定的测试实例".to_string());
        }
        Err(e) => {
            error!("❌ [AI_MANUAL_TEST] 加载测试实例失败: {}", e);
            return Err(format!("加载测试实例失败: {}", e));
        }
    };

    let definition = match app_state.persistence_service.load_channel_definition(&instance.definition_id).await {
        Ok(Some(definition)) => definition,
        Ok(None) => {
            error!("❌ [AI_MANUAL_TEST] 未找到通道定义: {}", instance.definition_id);
            return Err("未找到通道定义".to_string());
        }
        Err(e) => {
            error!("❌ [AI_MANUAL_TEST] 加载通道定义失败: {}", e);
            return Err(format!("加载通道定义失败: {}", e));
        }
    };

    // 生成随机值（在低限和高限之间）
    let low_limit = definition.range_low_limit.unwrap_or(0.0) as f64;
    let high_limit = definition.range_high_limit.unwrap_or(100.0) as f64;

    if high_limit <= low_limit {
        error!("❌ [AI_MANUAL_TEST] 无效的限值范围: 低限={}, 高限={}", low_limit, high_limit);
        return Err("无效的限值范围".to_string());
    }

    let range = high_limit - low_limit;
    let mut rng = rand::thread_rng();
    let random_value = low_limit + (rng.gen::<f64>() * range);

    info!("✅ [AI_MANUAL_TEST] 生成随机值成功: {:.2} (范围: {:.2} - {:.2})",
          random_value, low_limit, high_limit);

    Ok(GenerateRandomValueResponse {
        success: true,
        message: "随机值生成成功".to_string(),
        random_value,
        low_limit,
        high_limit,
    })
}

/// AI手动测试 - 显示值核对测试
#[tauri::command]
pub async fn ai_show_value_test_cmd(
    request: AiShowValueTestRequest,
    app_state: State<'_, crate::tauri_commands::AppState>,
) -> Result<AiShowValueTestResponse, String> {
    info!("🎯 [AI_MANUAL_TEST] 点击显示值核对测试按钮: {} -> {:.2}",
          request.instance_id, request.test_value);

    // 获取测试实例和通道定义
    let (instance, definition) = match get_instance_and_definition(&app_state, &request.instance_id).await {
        Ok((instance, definition)) => (instance, definition),
        Err(e) => return Err(e),
    };

    // 获取测试PLC通道地址
    let test_plc_address = match get_test_plc_address(&app_state, &instance).await {
        Ok(address) => address,
        Err(e) => return Err(e),
    };

    // 将工程值转换为百分比 (0.0-100.0)
    let percentage = convert_engineering_to_percentage(
        request.test_value,
        definition.range_low_limit.unwrap_or(0.0) as f64,
        definition.range_high_limit.unwrap_or(100.0) as f64,
    );

    // 实际执行PLC写入操作
    match write_to_test_plc(&app_state, &test_plc_address, percentage).await {
        Ok(_) => {
            info!("✅ [AI_MANUAL_TEST] 显示值下发成功: {:.2} -> {:.2}% -> {}",
                  request.test_value, percentage, test_plc_address);

            Ok(AiShowValueTestResponse {
                success: true,
                message: format!("测试值下发成功: {:.2} ({:.2}%)", request.test_value, percentage),
                sent_percentage: percentage,
                test_plc_address,
            })
        }
        Err(e) => {
            error!("❌ [AI_MANUAL_TEST] 显示值下发失败: {}", e);
            Err(format!("显示值下发失败: {}", e))
        }
    }
}

/// AI手动测试 - 报警测试
#[tauri::command]
pub async fn ai_alarm_test_cmd(
    request: AiAlarmTestRequest,
    app_state: State<'_, crate::tauri_commands::AppState>,
) -> Result<AiAlarmTestResponse, String> {
    info!("🎯 [AI_MANUAL_TEST] 点击{}报警测试按钮: {}",
          request.alarm_type, request.instance_id);

    // 获取测试实例和通道定义
    let (instance, definition) = match get_instance_and_definition(&app_state, &request.instance_id).await {
        Ok((instance, definition)) => (instance, definition),
        Err(e) => return Err(e),
    };

    // 获取测试PLC通道地址
    let test_plc_address = match get_test_plc_address(&app_state, &instance).await {
        Ok(address) => address,
        Err(e) => return Err(e),
    };

    // 根据报警类型计算测试值（量程的1%偏移）
    let range = definition.range_high_limit.unwrap_or(100.0) - definition.range_low_limit.unwrap_or(0.0);
    let offset = range * 0.01; // 1%偏移

    let test_value = match request.alarm_type.as_str() {
        "LL" => definition.sll_set_value.unwrap_or(0.0) as f64 - offset as f64,
        "L" => definition.sl_set_value.unwrap_or(10.0) as f64 - offset as f64,
        "H" => definition.sh_set_value.unwrap_or(90.0) as f64 + offset as f64,
        "HH" => definition.shh_set_value.unwrap_or(100.0) as f64 + offset as f64,
        _ => {
            error!("❌ [AI_MANUAL_TEST] 无效的报警类型: {}", request.alarm_type);
            return Err("无效的报警类型".to_string());
        }
    };

    // 将工程值转换为百分比
    let percentage = convert_engineering_to_percentage(
        test_value,
        definition.range_low_limit.unwrap_or(0.0) as f64,
        definition.range_high_limit.unwrap_or(100.0) as f64,
    );

    // 实际执行PLC写入操作
    match write_to_test_plc(&app_state, &test_plc_address, percentage).await {
        Ok(_) => {
            info!("✅ [AI_MANUAL_TEST] {}报警测试值下发成功: {:.2} -> {:.2}% -> {}",
                  request.alarm_type, test_value, percentage, test_plc_address);

            Ok(AiAlarmTestResponse {
                success: true,
                message: format!("{}报警测试值下发成功: {:.2} ({:.2}%)",
                               request.alarm_type, test_value, percentage),
                sent_value: test_value,
                sent_percentage: percentage,
                test_plc_address,
            })
        }
        Err(e) => {
            error!("❌ [AI_MANUAL_TEST] {}报警测试值下发失败: {}", request.alarm_type, e);
            Err(format!("{}报警测试值下发失败: {}", request.alarm_type, e))
        }
    }
}

/// AI手动测试 - 维护功能测试
#[tauri::command]
pub async fn ai_maintenance_test_cmd(
    request: AiMaintenanceTestRequest,
    app_state: State<'_, crate::tauri_commands::AppState>,
) -> Result<AiMaintenanceTestResponse, String> {
    info!("🎯 [AI_MANUAL_TEST] 点击维护功能{}按钮: {}",
          if request.enable { "启用" } else { "复位" }, request.instance_id);

    // 获取测试实例和通道定义
    let (_instance, definition) = match get_instance_and_definition(&app_state, &request.instance_id).await {
        Ok((instance, definition)) => (instance, definition),
        Err(e) => return Err(e),
    };

    // 获取维护使能开关地址，并进行规范化（长度不足 5 位时左补 0）
    let mut maintenance_address = match definition.maintenance_enable_switch_point_communication_address {
        Some(addr) => normalize_modbus_address(&addr),
        None => {
            error!("❌ [AI_MANUAL_TEST] 未配置维护使能开关地址: {}", request.instance_id);
            return Err("未配置维护使能开关地址".to_string());
        }
    };

    // 实际执行PLC写入操作（维护功能使用布尔值）
    match write_bool_to_target_plc(&app_state, &maintenance_address, request.enable).await {
        Ok(_) => {
            let action = if request.enable { "启用" } else { "复位" };
            info!("✅ [AI_MANUAL_TEST] 维护功能{}成功: {} -> {}",
                  action, maintenance_address, request.enable);

            Ok(AiMaintenanceTestResponse {
                success: true,
                message: format!("维护功能{}成功", action),
                maintenance_address,
            })
        }
        Err(e) => {
            let action = if request.enable { "启用" } else { "复位" };
            error!("❌ [AI_MANUAL_TEST] 维护功能{}失败: {}", action, e);
            Err(format!("维护功能{}失败: {}", action, e))
        }
    }
}

/// AI手动测试 - 复位到显示值
#[tauri::command]
pub async fn ai_reset_to_display_value_cmd(
    request: AiShowValueTestRequest,
    app_state: State<'_, crate::tauri_commands::AppState>,
) -> Result<AiShowValueTestResponse, String> {
    info!("🎯 [AI_MANUAL_TEST] 点击复位到显示值按钮: {} -> {:.2}",
          request.instance_id, request.test_value);

    // 复用显示值测试的逻辑
    ai_show_value_test_cmd(request, app_state).await
}

/// 手动测试子项完成确认
#[tauri::command]
pub async fn complete_manual_test_subitem_cmd(
    instance_id: String,
    sub_item: String, // 暂时使用字符串，后续可以定义枚举
    app_state: State<'_, crate::tauri_commands::AppState>,
) -> Result<serde_json::Value, String> {
    info!("🔧 [AI_MANUAL_TEST] 完成手动测试子项: {} -> {}", instance_id, sub_item);

    // 将字符串转换为SubTestItem
    let sub_test_item = match sub_item.as_str() {
        "ShowValueCheck" => crate::models::enums::SubTestItem::HardPoint, // 暂时映射到硬点测试
        "LowLowAlarmTest" => crate::models::enums::SubTestItem::LowLowAlarm,
        "LowAlarmTest" => crate::models::enums::SubTestItem::LowAlarm,
        "HighAlarmTest" => crate::models::enums::SubTestItem::HighAlarm,
        "HighHighAlarmTest" => crate::models::enums::SubTestItem::HighHighAlarm,
        "TrendCheck" => crate::models::enums::SubTestItem::Trend,
        "ReportCheck" => crate::models::enums::SubTestItem::Report,
        "MaintenanceFunction" => crate::models::enums::SubTestItem::Maintenance,
        _ => crate::models::enums::SubTestItem::HardPoint, // 默认值
    };

    // 创建一个成功的测试结果
    let mut outcome = crate::models::RawTestOutcome::success(
        instance_id.clone(),
        sub_test_item,
    );
    outcome.message = Some(format!("手动测试子项完成: {}", sub_item));

    // 通过状态管理器更新测试结果
    match app_state.channel_state_manager.update_test_result(outcome).await {
        Ok(_) => {
            info!("✅ [AI_MANUAL_TEST] 手动测试子项完成: {:?}", sub_item);
            Ok(serde_json::json!({
                "success": true,
                "message": "测试项完成确认成功"
            }))
        }
        Err(e) => {
            error!("❌ [AI_MANUAL_TEST] 手动测试子项完成失败: {}", e);
            Err(format!("测试项完成确认失败: {}", e))
        }
    }
}

// ==================== 辅助函数 ====================

/// 获取测试实例和通道定义
async fn get_instance_and_definition(
    app_state: &State<'_, crate::tauri_commands::AppState>,
    instance_id: &str,
) -> Result<(crate::models::ChannelTestInstance, crate::models::ChannelPointDefinition), String> {
    // 获取测试实例
    let instance = match app_state.persistence_service.load_test_instance(instance_id).await {
        Ok(Some(instance)) => instance,
        Ok(None) => {
            error!("❌ [AI_MANUAL_TEST] 未找到测试实例: {}", instance_id);
            return Err("未找到指定的测试实例".to_string());
        }
        Err(e) => {
            error!("❌ [AI_MANUAL_TEST] 加载测试实例失败: {}", e);
            return Err(format!("加载测试实例失败: {}", e));
        }
    };

    // 获取通道定义
    let definition = match app_state.persistence_service.load_channel_definition(&instance.definition_id).await {
        Ok(Some(definition)) => definition,
        Ok(None) => {
            error!("❌ [AI_MANUAL_TEST] 未找到通道定义: {}", instance.definition_id);
            return Err("未找到通道定义".to_string());
        }
        Err(e) => {
            error!("❌ [AI_MANUAL_TEST] 加载通道定义失败: {}", e);
            return Err(format!("加载通道定义失败: {}", e));
        }
    };

    Ok((instance, definition))
}

/// 获取测试PLC通道通信地址（直接查询数据库）
async fn get_test_plc_address(
    app_state: &State<'_, crate::tauri_commands::AppState>,
    instance: &crate::models::ChannelTestInstance,
) -> Result<String, String> {
    use crate::models::test_plc_config::GetTestPlcChannelsRequest;

    let tag = match &instance.test_plc_channel_tag {
        Some(t) => t.clone(),
        None => {
            error!("❌ [AI_MANUAL_TEST] 测试实例未分配测试PLC通道: {}", instance.instance_id);
            return Err("测试实例未分配测试PLC通道".to_string());
        }
    };

    // 查询数据库获取所有启用的通道配置
    let channels = match app_state
        .test_plc_config_service
        .get_test_plc_channels(GetTestPlcChannelsRequest {
            channel_type_filter: None,
            enabled_only: Some(true),
        })
        .await
    {
        Ok(cs) => cs,
        Err(e) => {
            error!("❌ [AI_MANUAL_TEST] 加载测试PLC通道配置失败: {}", e);
            return Err(format!("加载测试PLC通道配置失败: {}", e));
        }
    };

    if let Some(cfg) = channels.iter().find(|c| c.channel_address == tag) {
        Ok(cfg.communication_address.clone())
    } else {
        error!("❌ [AI_MANUAL_TEST] 通道标签未找到对应配置: {}", tag);
        Err("未找到测试PLC通道通信地址".to_string())
    }
}

/// 将工程值转换为百分比 (0.0-100.0)
fn convert_engineering_to_percentage(engineering_value: f64, range_low: f64, range_high: f64) -> f64 {
    if range_high <= range_low {
        warn!("⚠️ [AI_MANUAL_TEST] 无效的量程范围: {} - {}", range_low, range_high);
        return 0.0;
    }

    let percentage = (engineering_value - range_low) / (range_high - range_low) * 100.0;

    // 限制在0-100范围内
    percentage.max(0.0).min(100.0)
}

/// 写入浮点值到测试PLC
async fn write_to_test_plc(
    app_state: &State<'_, crate::tauri_commands::AppState>,
    address: &str,
    percentage: f64,
) -> Result<(), String> {
    info!("📝 [AI_MANUAL_TEST] 写入测试PLC [{}]: {:.2}%", address, percentage);

    // 获取测试 PLC 配置（仅需 IP）
    let test_plc_config = match app_state
        .test_plc_config_service
        .get_test_plc_config()
        .await
    {
        Ok(cfg) => cfg,
        Err(e) => {
            error!("❌ [AI_MANUAL_TEST] 获取测试PLC配置失败: {}", e);
            return Err(format!("获取测试PLC配置失败: {}", e));
        }
    };

    // --- 新版全局 PLC 服务 ---
    let plc_service: std::sync::Arc<dyn IPlcCommunicationService + Send + Sync> = global_plc_service();
    // 使用自动测试阶段已建立的测试PLC连接ID
    let connection_id = app_state.test_rig_connection_id.clone();

    // 连接配置
    use std::collections::HashMap;
    let plc_config = PlcConnectionConfig {
        id: connection_id.clone(),
        name: "ManualTestPLC".to_string(),
        protocol: PlcProtocol::ModbusTcp,
        host: test_plc_config.ip_address.clone(),
        port: 502,
        timeout_ms: 5000,
        read_timeout_ms: 3000,
        write_timeout_ms: 3000,
        byte_order: crate::models::ByteOrder::default().to_string(),
        zero_based_address: false,
        retry_count: 0,
        retry_interval_ms: 500,
        protocol_params: HashMap::new(),
    };

    // 必须已存在连接
    match plc_service.default_handle_by_id(&connection_id).await {
        Some(h) => {
            if !plc_service.is_connected(&h).await.unwrap_or(false) {
                error!("❌ [AI_MANUAL_TEST] 测试PLC连接已断开: {}", connection_id);
                return Err("测试PLC连接已断开".to_string());
            }
        }
        None => {
            error!("❌ [AI_MANUAL_TEST] 未找到测试PLC连接: {}", connection_id);
            return Err("测试PLC未连接".to_string());
        }
    }

    // 写入百分比
    match plc_service
        .write_float32_by_id(&connection_id, address, percentage as f32)
        .await
    {
        Ok(_) => {
            info!(
                "✅ [AI_MANUAL_TEST] 测试PLC写入成功: [{}] = {:.2}%",
                address, percentage
            );
            Ok(())
        }
        Err(e) => {
            error!(
                "❌ [AI_MANUAL_TEST] 测试PLC写入失败: [{}] = {:.2}%, 错误: {}",
                address, percentage, e
            );
            Err(format!("PLC写入失败: {}", e))
        }
    }
}

/// 写入布尔值到被测PLC（用于维护功能）
async fn write_bool_to_target_plc(
    app_state: &State<'_, crate::tauri_commands::AppState>,
    address: &str,
    value: bool,
) -> Result<(), String> {
    // 对地址执行规范化，确保 0X 开头 4 位对齐等
    let fixed_address = normalize_modbus_address(address);
    info!("📝 [AI_MANUAL_TEST] 写入被测PLC [{}]: {}", fixed_address, value);

    // 获取被测 PLC 配置（目前仍与测试 PLC 同一个配置）
    let test_plc_config = match app_state
        .test_plc_config_service
        .get_test_plc_config()
        .await
    {
        Ok(cfg) => cfg,
        Err(e) => {
            error!("❌ [AI_MANUAL_TEST] 获取PLC配置失败: {}", e);
            return Err(format!("获取PLC配置失败: {}", e));
        }
    };

    // --- 新版全局 PLC 服务 ---
    let plc_service: std::sync::Arc<dyn IPlcCommunicationService + Send + Sync> = global_plc_service();
    // 使用自动测试阶段已建立的被测PLC连接ID
    let connection_id = app_state.target_connection_id.clone();

    use std::collections::HashMap;
    let plc_config = PlcConnectionConfig {
        id: connection_id.clone(),
        name: "ManualTargetPLC".to_string(),
        protocol: PlcProtocol::ModbusTcp,
        host: test_plc_config.ip_address.clone(),
        port: 502,
        timeout_ms: 5000,
        read_timeout_ms: 3000,
        write_timeout_ms: 3000,
        byte_order: crate::models::ByteOrder::default().to_string(),
        zero_based_address: false,
        retry_count: 0,
        retry_interval_ms: 500,
        protocol_params: HashMap::new(),
    };

    // 检查连接是否存在且在线
    match plc_service.default_handle_by_id(&connection_id).await {
        Some(h) => {
            if !plc_service.is_connected(&h).await.unwrap_or(false) {
                error!("❌ [AI_MANUAL_TEST] 被测PLC连接已断开: {}", connection_id);
                return Err("被测PLC连接已断开".to_string());
            }
        }
        None => {
            error!("❌ [AI_MANUAL_TEST] 未找到被测PLC连接: {}", connection_id);
            return Err("被测PLC未连接".to_string());
        }
    }

    // 写入布尔值
    match plc_service
        .write_bool_by_id(&connection_id, &fixed_address, value)
        .await
    {
        Ok(_) => {
            info!(
                "✅ [AI_MANUAL_TEST] 被测PLC写入成功: [{}] = {}",
                fixed_address, value
            );
            Ok(())
        }
        Err(e) => {
            error!(
                "❌ [AI_MANUAL_TEST] 被测PLC写入失败: [{}] = {}, 错误: {}",
                fixed_address, value, e
            );
            Err(format!("PLC写入失败: {}", e))
        }
    }
}
/// 规范化 Modbus 地址：不足 5 位时在左侧补零至 5 位
fn normalize_modbus_address(address: &str) -> String {
    // 仅保留数字字符
    let digits: String = address.chars().filter(|c| c.is_ascii_digit()).collect();
    if digits.is_empty() {
        return address.to_string(); // 返回原样，后续写入会报错指出
    }
    format!("{:0>5}", digits)
}

/// ==================== DI 手动测试专用命令 ====================

/// DI 信号下发请求（将测试 PLC DO 通道置位或复位）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiSignalTestRequest {
    pub instance_id: String,
    pub enable: bool, // true = 置位 (ON), false = 复位 (OFF)
}

/// DI 信号下发响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiSignalTestResponse {
    pub success: bool,
    pub message: String,
    pub test_plc_address: String,
}

/// DI 手动测试 - 信号下发
#[tauri::command]
pub async fn di_signal_test_cmd(
    request: DiSignalTestRequest,
    app_state: State<'_, crate::tauri_commands::AppState>,
) -> Result<DiSignalTestResponse, String> {
    info!("🎯 [DI_MANUAL_TEST] 点击{}按钮: {}",
          if request.enable { "置位" } else { "复位" }, request.instance_id);

    // 获取测试实例
    let instance = match app_state.persistence_service.load_test_instance(&request.instance_id).await {
        Ok(Some(inst)) => inst,
        Ok(None) => {
            error!("❌ [DI_MANUAL_TEST] 未找到测试实例: {}", request.instance_id);
            return Err("未找到指定的测试实例".to_string());
        },
        Err(e) => {
            error!("❌ [DI_MANUAL_TEST] 加载测试实例失败: {}", e);
            return Err(format!("加载测试实例失败: {}", e));
        }
    };

    // 使用测试 PLC 通讯地址 (保持寄存器/线圈地址)，再进行数字化处理
    let test_plc_address = match &instance.test_plc_communication_address {
        Some(addr) => normalize_modbus_address(addr),
        None => {
            error!("❌ [DI_MANUAL_TEST] 测试实例未分配测试PLC通道: {}", request.instance_id);
            return Err("测试实例未分配测试PLC通道".to_string());
        }
    };

    // 下发布尔值到测试 PLC
    match write_bool_to_test_plc(&app_state, &test_plc_address, request.enable).await {
        Ok(_) => {
            let action = if request.enable { "置位" } else { "复位" };
            info!("✅ [DI_MANUAL_TEST] {}成功: {} -> {}", action, test_plc_address, request.enable);
            Ok(DiSignalTestResponse {
                success: true,
                message: format!("{}成功", action),
                test_plc_address,
            })
        }
        Err(e) => {
            let action = if request.enable { "置位" } else { "复位" };
            error!("❌ [DI_MANUAL_TEST] {}失败: {}", action, e);
            Err(format!("{}失败: {}", action, e))
        }
    }
}

/// 写入布尔值到测试 PLC（用于 DI 点位手动测试）
async fn write_bool_to_test_plc(
    app_state: &State<'_, crate::tauri_commands::AppState>,
    address: &str,
    value: bool,
) -> Result<(), String> {
    let fixed_address = normalize_modbus_address(address);
    info!("📝 [DI_MANUAL_TEST] 写入测试PLC [{}]: {}", fixed_address, value);

    // 获取测试 PLC 配置
    let test_plc_config = match app_state
        .test_plc_config_service
        .get_test_plc_config()
        .await
    {
        Ok(cfg) => cfg,
        Err(e) => {
            error!("❌ [DI_MANUAL_TEST] 获取测试PLC配置失败: {}", e);
            return Err(format!("获取测试PLC配置失败: {}", e));
        }
    };

    // --- 新版全局 PLC 服务 ---
    let plc_service: std::sync::Arc<dyn IPlcCommunicationService + Send + Sync> = global_plc_service();
    let connection_id = "manual_test_plc_bool".to_string();

    use std::collections::HashMap;
    let plc_config = PlcConnectionConfig {
        id: connection_id.clone(),
        name: "ManualTestPLC_BOOL".to_string(),
        protocol: PlcProtocol::ModbusTcp,
        host: test_plc_config.ip_address.clone(),
        port: 502,
        timeout_ms: 5000,
        read_timeout_ms: 3000,
        write_timeout_ms: 3000,
        byte_order: crate::models::ByteOrder::default().to_string(),
        zero_based_address: false,
        retry_count: 0,
        retry_interval_ms: 500,
        protocol_params: HashMap::new(),
    };

    if let Err(e) = plc_service.connect(&plc_config).await {
        error!("❌ [DI_MANUAL_TEST] PLC连接失败: {}", e);
        return Err(format!("PLC连接失败: {}", e));
    }

    match plc_service
        .write_bool_by_id(&connection_id, &fixed_address, value)
        .await
    {
        Ok(_) => {
            info!(
                "✅ [DI_MANUAL_TEST] 测试PLC写入成功: [{}] = {}",
                fixed_address, value
            );
            Ok(())
        }
        Err(e) => {
            error!(
                "❌ [DI_MANUAL_TEST] 测试PLC写入失败: [{}] = {}, 错误: {}",
                fixed_address, value, e
            );
            Err(format!("PLC写入失败: {}", e))
        }
    }
}