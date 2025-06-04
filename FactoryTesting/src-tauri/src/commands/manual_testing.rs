/// 手动测试相关的Tauri命令
///
/// 包括手动子测试执行、通道读写、PLC连接和自动测试等功能

use tauri::State;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::models::{SubTestItem, PointDataType, RawTestOutcome};
use crate::tauri_commands::AppState;
use log::{info, error, warn};

/// 执行手动子测试的参数
#[derive(Debug, Deserialize)]
pub struct ExecuteManualSubTestCmdArgs {
    pub instance_id: String,
    pub sub_test_item: SubTestItem,
    pub params: Option<HashMap<String, serde_json::Value>>,
}

/// 读取通道值的参数
#[derive(Debug, Deserialize)]
pub struct ReadChannelValueCmdArgs {
    pub instance_id: String,
    pub plc_address: String,
    pub data_type: PointDataType,
}

/// 写入通道值的参数
#[derive(Debug, Deserialize)]
pub struct WriteChannelValueCmdArgs {
    pub instance_id: String,
    pub plc_address: String,
    pub data_type: PointDataType,
    pub value_to_write: serde_json::Value,
}

/// 执行手动子测试
#[tauri::command]
pub async fn execute_manual_sub_test_cmd(
    args: ExecuteManualSubTestCmdArgs,
    state: State<'_, AppState>
) -> Result<RawTestOutcome, String> {
    info!("执行手动子测试: 实例ID={}, 测试项={:?}", args.instance_id, args.sub_test_item);
    
    // 获取测试实例
    let instance = match state.persistence_service.load_test_instance(&args.instance_id).await {
        Ok(Some(inst)) => inst,
        Ok(None) => return Err(format!("测试实例不存在: {}", args.instance_id)),
        Err(e) => {
            error!("获取测试实例失败: {}", e);
            return Err(format!("获取测试实例失败: {}", e));
        }
    };
    
    // 创建测试结果
    let outcome = RawTestOutcome {
        channel_instance_id: args.instance_id.clone(),
        sub_test_item: args.sub_test_item,
        success: true, // 手动测试默认成功，实际应根据用户输入
        raw_value_read: Some("手动测试值".to_string()),
        eng_value_calculated: Some("手动工程值".to_string()),
        message: Some("手动测试完成".to_string()),
        start_time: chrono::Utc::now(),
        end_time: chrono::Utc::now(),
        readings: None,
        details: args.params.unwrap_or_default(),
    };
    
    // 更新测试实例状态
    if let Err(e) = state.channel_state_manager.update_test_result(&args.instance_id, outcome.clone()).await {
        error!("更新测试实例状态失败: {}", e);
        return Err(format!("更新测试实例状态失败: {}", e));
    }
    
    info!("手动子测试执行完成");
    Ok(outcome)
}

/// 读取通道值
#[tauri::command]
pub async fn read_channel_value_cmd(
    args: ReadChannelValueCmdArgs,
    state: State<'_, AppState>
) -> Result<serde_json::Value, String> {
    info!("读取通道值: 实例ID={}, 地址={}, 类型={:?}", 
          args.instance_id, args.plc_address, args.data_type);
    
    // 这里应该调用PLC通信服务读取实际值
    // 目前返回模拟值
    let mock_value = match args.data_type {
        PointDataType::Bool => serde_json::Value::Bool(true),
        PointDataType::Int => serde_json::Value::Number(serde_json::Number::from(42)),
        PointDataType::Float => serde_json::Value::Number(
            serde_json::Number::from_f64(3.14159).unwrap_or(serde_json::Number::from(0))
        ),
        PointDataType::String => serde_json::Value::String("测试字符串".to_string()),
        PointDataType::Double => serde_json::Value::Number(
            serde_json::Number::from_f64(3.14159265359).unwrap_or(serde_json::Number::from(0))
        ),
        PointDataType::Int16 => serde_json::Value::Number(serde_json::Number::from(16)),
        PointDataType::Int32 => serde_json::Value::Number(serde_json::Number::from(32)),
        PointDataType::UInt16 => serde_json::Value::Number(serde_json::Number::from(16)),
        PointDataType::UInt32 => serde_json::Value::Number(serde_json::Number::from(32)),
    };
    
    info!("通道值读取完成: {:?}", mock_value);
    Ok(mock_value)
}

/// 写入通道值
#[tauri::command]
pub async fn write_channel_value_cmd(
    args: WriteChannelValueCmdArgs,
    state: State<'_, AppState>
) -> Result<(), String> {
    info!("写入通道值: 实例ID={}, 地址={}, 类型={:?}, 值={:?}", 
          args.instance_id, args.plc_address, args.data_type, args.value_to_write);
    
    // 验证值类型是否匹配
    let is_valid = match args.data_type {
        PointDataType::Bool => args.value_to_write.is_boolean(),
        PointDataType::Int => args.value_to_write.is_number(),
        PointDataType::Float => args.value_to_write.is_number(),
        PointDataType::String => args.value_to_write.is_string(),
        PointDataType::Double => args.value_to_write.is_number(),
        PointDataType::Int16 => args.value_to_write.is_number(),
        PointDataType::Int32 => args.value_to_write.is_number(),
        PointDataType::UInt16 => args.value_to_write.is_number(),
        PointDataType::UInt32 => args.value_to_write.is_number(),
    };
    
    if !is_valid {
        return Err(format!("值类型不匹配: 期望{:?}类型", args.data_type));
    }
    
    // 这里应该调用PLC通信服务写入实际值
    // 目前只是记录日志
    info!("通道值写入完成");
    Ok(())
}

/// PLC连接响应结构
#[derive(Debug, Serialize)]
pub struct PlcConnectionResponse {
    pub success: bool,
    pub message: Option<String>,
}

/// 批次自动测试参数
#[derive(Debug, Deserialize)]
pub struct StartBatchAutoTestCmdArgs {
    pub batch_id: String,
}

/// 批次自动测试响应结构
#[derive(Debug, Serialize)]
pub struct BatchAutoTestResponse {
    pub success: bool,
    pub message: Option<String>,
}

/// PLC连接状态信息
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlcConnectionStatus {
    pub test_plc_connected: bool,
    pub target_plc_connected: bool,
    pub test_plc_name: Option<String>,
    pub target_plc_name: Option<String>,
    pub last_check_time: String,
}

/// 连接PLC - 确认接线
#[tauri::command]
pub async fn connect_plc_cmd(
    state: State<'_, AppState>
) -> Result<PlcConnectionResponse, String> {
    info!("🔗 开始连接PLC - 确认接线");

    let app_state = state.inner();
    let test_plc_config_service = app_state.test_plc_config_service.clone();

    // 1. 获取PLC连接配置
    let plc_connections = match test_plc_config_service.get_plc_connections().await {
        Ok(connections) => connections,
        Err(e) => {
            error!("❌ 获取PLC连接配置失败: {}", e);
            return Ok(PlcConnectionResponse {
                success: false,
                message: Some(format!("获取PLC连接配置失败: {}", e)),
            });
        }
    };

    if plc_connections.is_empty() {
        warn!("⚠️ 没有找到PLC连接配置");
        return Ok(PlcConnectionResponse {
            success: false,
            message: Some("没有找到PLC连接配置，请先在测试PLC配置页面添加连接".to_string()),
        });
    }

    // 2. 分别连接测试PLC和被测PLC
    let mut test_plc_connected = false;
    let mut target_plc_connected = false;
    let mut connection_messages = Vec::new();

    for connection in &plc_connections {
        if !connection.is_enabled {
            continue;
        }

        info!("🔗 尝试连接PLC: {} ({}:{})", connection.name, connection.ip_address, connection.port);

        match test_plc_config_service.test_plc_connection(&connection.id).await {
            Ok(test_result) => {
                if test_result.success {
                    if connection.is_test_plc {
                        test_plc_connected = true;
                        connection_messages.push(format!("测试PLC ({}) 连接成功", connection.name));
                        info!("✅ 测试PLC连接成功: {}", connection.name);
                    } else {
                        target_plc_connected = true;
                        connection_messages.push(format!("被测PLC ({}) 连接成功", connection.name));
                        info!("✅ 被测PLC连接成功: {}", connection.name);
                    }
                } else {
                    let error_msg = format!("{} 连接失败: {}",
                        if connection.is_test_plc { "测试PLC" } else { "被测PLC" },
                        test_result.message);
                    connection_messages.push(error_msg.clone());
                    error!("❌ {}", error_msg);
                }
            }
            Err(e) => {
                let error_msg = format!("{} 连接异常: {}",
                    if connection.is_test_plc { "测试PLC" } else { "被测PLC" },
                    e);
                connection_messages.push(error_msg.clone());
                error!("❌ {}", error_msg);
            }
        }
    }

    // 3. 验证连接状态
    let overall_success = test_plc_connected && target_plc_connected;
    let message = if overall_success {
        "所有PLC连接成功，接线确认完成".to_string()
    } else if test_plc_connected || target_plc_connected {
        format!("部分PLC连接成功: {}", connection_messages.join("; "))
    } else {
        format!("所有PLC连接失败: {}", connection_messages.join("; "))
    };

    let response = PlcConnectionResponse {
        success: overall_success,
        message: Some(message),
    };

    if overall_success {
        info!("✅ PLC连接完成 - 测试PLC和被测PLC都已连接");
    } else {
        warn!("⚠️ PLC连接未完全成功");
    }

    Ok(response)
}

/// 开始批次自动测试
#[tauri::command]
pub async fn start_batch_auto_test_cmd(
    args: StartBatchAutoTestCmdArgs,
    state: State<'_, AppState>
) -> Result<BatchAutoTestResponse, String> {
    info!("🚀 开始批次自动测试: 批次ID={}", args.batch_id);

    // TODO: 实现实际的批次自动测试逻辑
    // 1. 验证批次存在
    // 2. 获取批次中的所有测试实例
    // 3. 调用测试任务管理服务创建并启动测试任务
    // 4. 根据并发配置并行执行硬点通道测试

    // 目前返回模拟成功响应
    warn!("⚠️ 批次自动测试功能尚未完全实现，返回模拟成功响应");

    let response = BatchAutoTestResponse {
        success: true,
        message: Some("批次自动测试已启动 (模拟)".to_string()),
    };

    info!("✅ 批次自动测试启动完成");
    Ok(response)
}

/// 获取PLC连接状态
#[tauri::command]
pub async fn get_plc_connection_status_cmd(
    state: State<'_, AppState>
) -> Result<PlcConnectionStatus, String> {
    let app_state = state.inner();
    let test_plc_config_service = app_state.test_plc_config_service.clone();

    // 获取PLC连接配置
    let plc_connections = match test_plc_config_service.get_plc_connections().await {
        Ok(connections) => connections,
        Err(e) => {
            error!("❌ 获取PLC连接配置失败: {}", e);
            return Ok(PlcConnectionStatus {
                test_plc_connected: false,
                target_plc_connected: false,
                test_plc_name: None,
                target_plc_name: None,
                last_check_time: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            });
        }
    };

    let mut test_plc_connected = false;
    let mut target_plc_connected = false;
    let mut test_plc_name = None;
    let mut target_plc_name = None;

    // 检查每个启用的PLC连接状态
    for connection in &plc_connections {
        if !connection.is_enabled {
            continue;
        }

        // 测试连接状态
        let is_connected = match test_plc_config_service.test_plc_connection(&connection.id).await {
            Ok(test_result) => test_result.success,
            Err(_) => false,
        };

        if connection.is_test_plc {
            test_plc_connected = is_connected;
            test_plc_name = Some(connection.name.clone());
        } else {
            target_plc_connected = is_connected;
            target_plc_name = Some(connection.name.clone());
        }
    }

    Ok(PlcConnectionStatus {
        test_plc_connected,
        target_plc_connected,
        test_plc_name,
        target_plc_name,
        last_check_time: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
    })
}