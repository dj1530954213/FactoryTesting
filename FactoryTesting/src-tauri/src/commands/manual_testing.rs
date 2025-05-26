/// 手动测试相关的Tauri命令
/// 
/// 包括手动子测试执行、通道读写等功能

use tauri::State;
use serde::Deserialize;
use std::collections::HashMap;
use crate::models::{SubTestItem, PointDataType, RawTestOutcome};
use crate::tauri_commands::AppState;
use log::{info, error};

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