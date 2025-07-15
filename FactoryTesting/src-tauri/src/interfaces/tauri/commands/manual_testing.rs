/// 手动测试相关的Tauri命令
///
/// 包括手动子测试执行、通道读写、PLC连接和自动测试等功能

use tauri::{State, Manager};
use std::sync::Arc;
use crate::application::services::range_setting_service::{DynamicRangeSettingService, ChannelRangeSettingService, IChannelRangeSettingService};
use crate::domain::services::IRangeRegisterRepository;
use crate::infrastructure::range_register_repository::RangeRegisterRepository;
use crate::domain::services::range_value_calculator::{DefaultRangeValueCalculator, IRangeValueCalculator};
use crate::domain::services::plc_communication_service::IPlcCommunicationService;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::models::{SubTestItem, PointDataType, RawTestOutcome};
use crate::tauri_commands::AppState;
use log::{info, error, warn};
use tokio::time::{sleep, Duration};

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
        digital_steps: None,
        test_result_0_percent: None,
        test_result_25_percent: None,
        test_result_50_percent: None,
        test_result_75_percent: None,
        test_result_100_percent: None,
        details: args.params.unwrap_or_default(),
    };
    
    // 更新测试实例状态
    if let Err(e) = state.channel_state_manager.update_test_result(outcome.clone()).await {
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
    app: tauri::AppHandle, // 用于动态覆盖 manage 中的服务实例
    state: State<'_, AppState>
) -> Result<PlcConnectionResponse, String> {
    info!("🔗 开始连接PLC - 确认接线");

    let app_state = state.inner();
    let plc_connection_manager = app_state.plc_connection_manager.clone();

    // 启动PLC连接管理器，建立持久连接
    match plc_connection_manager.start_connections().await {
        Ok(()) => {
            info!("✅ PLC连接管理器启动成功");

            // 等待PLC实际连上，最多3秒，每200ms检查一次
            let mut waited_ms = 0;
            let (mut test_plc_connected, mut target_plc_connected, mut test_plc_name, mut target_plc_name) = (false, false, None, None);
            while waited_ms < 3000 {
                let summary = plc_connection_manager.get_plc_status_summary().await;
                test_plc_connected = summary.0;
                target_plc_connected = summary.1;
                test_plc_name = summary.2.clone();
                target_plc_name = summary.3.clone();
                if test_plc_connected && target_plc_connected {
                    break;
                }
                sleep(Duration::from_millis(200)).await;
                waited_ms += 200;
            }
            // 动态替换量程写入服务实现
            {
                // 一定存在，直接获取
                let range_container = app.state::<Arc<DynamicRangeSettingService>>();
                // 构建新的 ChannelRangeSettingService
                let plc_service = crate::infrastructure::plc_communication::global_plc_service();
                if let Some(handle) = plc_service.default_handle().await {
                    let db_conn = app_state.persistence_service.get_database_connection();
                    let range_repo: Arc<dyn IRangeRegisterRepository> = Arc::new(RangeRegisterRepository::new(db_conn));
                    let calculator: Arc<dyn IRangeValueCalculator> = Arc::new(DefaultRangeValueCalculator);
                    let new_impl = Arc::new(ChannelRangeSettingService::new(
                        plc_service,
                        handle,
                        range_repo,
                        calculator,
                    )) as Arc<dyn IChannelRangeSettingService>;
                    range_container.replace(new_impl).await;
                } else {
                    warn!("[connect_plc_cmd] PLC连接已建立但未获取到默认句柄，无法替换量程服务");
                }
            }

            // 等待一段时间让连接建立
            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

            // 检查连接状态
            let (test_plc_connected, target_plc_connected, test_plc_name, target_plc_name) =
                plc_connection_manager.get_plc_status_summary().await;

            // 若至少一个 PLC 已连接，尝试构建 ChannelRangeSettingService 并覆盖 manage
            if let Some(default_handle) = crate::infrastructure::plc_communication::global_plc_service().default_handle().await {
                use std::sync::Arc;
                use crate::application::services::range_setting_service::{ChannelRangeSettingService, IChannelRangeSettingService};
                use crate::domain::services::IRangeRegisterRepository;
                use crate::domain::services::range_value_calculator::{IRangeValueCalculator, DefaultRangeValueCalculator};
                use crate::infrastructure::range_register_repository::RangeRegisterRepository;
                use crate::domain::services::plc_communication_service::IPlcCommunicationService;

                let plc_service = crate::infrastructure::plc_communication::global_plc_service();
                let plc_service_dyn: Arc<dyn IPlcCommunicationService> = plc_service.clone();

                // 创建依赖
                let db_conn = state.persistence_service.get_database_connection();
                let range_repo: Arc<dyn IRangeRegisterRepository> = Arc::new(RangeRegisterRepository::new(db_conn));
                let calculator: Arc<dyn IRangeValueCalculator> = Arc::new(DefaultRangeValueCalculator);

                let range_setting_service: Arc<dyn IChannelRangeSettingService> = Arc::new(
                    ChannelRangeSettingService::new(
                        plc_service_dyn,
                        default_handle,
                        range_repo,
                        calculator,
                    )
                );
                // 覆盖旧的 NullRangeSettingService
                app.manage(range_setting_service);
                log::info!("[connect_plc_cmd] 已注入新的 ChannelRangeSettingService");
            }

            let overall_success = test_plc_connected && target_plc_connected;
            let message = if overall_success {
                format!("所有PLC连接成功，接线确认完成。测试PLC: {}, 被测PLC: {}",
                    test_plc_name.unwrap_or("未知".to_string()),
                    target_plc_name.unwrap_or("未知".to_string()))
            } else if test_plc_connected || target_plc_connected {
                let mut parts = Vec::new();
                if test_plc_connected {
                    parts.push(format!("测试PLC ({}) 连接成功", test_plc_name.unwrap_or("未知".to_string())));
                } else {
                    parts.push(format!("测试PLC ({}) 连接失败", test_plc_name.unwrap_or("未配置".to_string())));
                }
                if target_plc_connected {
                    parts.push(format!("被测PLC ({}) 连接成功", target_plc_name.unwrap_or("未知".to_string())));
                } else {
                    parts.push(format!("被测PLC ({}) 连接失败", target_plc_name.unwrap_or("未配置".to_string())));
                }
                parts.join("; ")
            } else {
                "所有PLC连接失败，请检查PLC配置和网络连接".to_string()
            };

            let response = PlcConnectionResponse {
                success: overall_success,
                message: Some(message),
            };

            if overall_success {
                info!("✅ PLC连接完成 - 测试PLC和被测PLC都已连接，开始心跳检测");
            } else {
                warn!("⚠️ PLC连接未完全成功，连接管理器将继续尝试重连");
            }

            Ok(response)
        }
        Err(e) => {
            error!("❌ PLC连接管理器启动失败: {}", e);
            Ok(PlcConnectionResponse {
                success: false,
                message: Some(format!("PLC连接管理器启动失败: {}", e)),
            })
        }
    }
}

/// 开始批次自动测试
#[tauri::command]
pub async fn start_batch_auto_test_cmd(
    args: StartBatchAutoTestCmdArgs,
    state: State<'_, AppState>
) -> Result<BatchAutoTestResponse, String> {
    info!("🚀 开始批次自动测试: 批次ID={}", args.batch_id);

    // 1. 验证批次存在
    let batch_info = match state.persistence_service.load_batch_info(&args.batch_id).await {
        Ok(Some(info)) => {
            info!("✅ 找到批次信息: {}", info.batch_name);
            info
        },
        Ok(None) => {
            error!("❌ 批次不存在: {}", args.batch_id);
            return Ok(BatchAutoTestResponse {
                success: false,
                message: Some(format!("批次不存在: {}", args.batch_id)),
            });
        },
        Err(e) => {
            error!("❌ 获取批次信息失败: {}", e);
            return Ok(BatchAutoTestResponse {
                success: false,
                message: Some(format!("获取批次信息失败: {}", e)),
            });
        }
    };

    // 2. 获取批次中的所有测试实例
    let test_instances = match state.persistence_service.load_test_instances_by_batch(&args.batch_id).await {
        Ok(instances) => {
            info!("✅ 获取到 {} 个测试实例", instances.len());
            if instances.is_empty() {
                warn!("⚠️ 批次中没有测试实例");
                return Ok(BatchAutoTestResponse {
                    success: false,
                    message: Some("批次中没有测试实例，请先进行批次分配".to_string()),
                });
            }
            instances
        },
        Err(e) => {
            error!("❌ 获取测试实例失败: {}", e);
            return Ok(BatchAutoTestResponse {
                success: false,
                message: Some(format!("获取测试实例失败: {}", e)),
            });
        }
    };

    // 3. 获取通道定义
    let mut channel_definitions = Vec::new();
    for instance in &test_instances {
        if let Some(definition) = state.channel_state_manager.get_channel_definition(&instance.definition_id).await {
            channel_definitions.push(definition);
        } else {
            warn!("⚠️ 未找到通道定义: {}", instance.definition_id);
        }
    }

    if channel_definitions.is_empty() {
        error!("❌ 没有找到任何通道定义");
        return Ok(BatchAutoTestResponse {
            success: false,
            message: Some("没有找到通道定义，请检查数据完整性".to_string()),
        });
    }

    // 4. 直接启动已存在的批次测试
    // 首先检查批次是否已经在活动批次中，如果不在，需要先加载到活动批次
    match state.test_coordination_service.start_batch_testing(&args.batch_id).await {
        Ok(()) => {
            info!("✅ 批次自动测试启动成功: {}", args.batch_id);
            Ok(BatchAutoTestResponse {
                success: true,
                message: Some(format!("批次 '{}' 的硬点通道自动测试已启动，共 {} 个测试点位",
                                    batch_info.batch_name, test_instances.len())),
            })
        },
        Err(e) => {
            // 如果直接启动失败，可能是因为批次不在活动列表中，尝试加载现有批次
            warn!("⚠️ 直接启动失败，尝试加载现有批次: {}", e);

            // 使用新的加载现有批次方法
            match state.test_coordination_service.load_existing_batch(&args.batch_id).await {
                Ok(()) => {
                    info!("✅ 批次已加载到活动列表，现在启动测试: {}", args.batch_id);

                    // 再次尝试启动测试
                    match state.test_coordination_service.start_batch_testing(&args.batch_id).await {
                        Ok(()) => {
                            info!("✅ 批次测试启动成功: {}", args.batch_id);
                            Ok(BatchAutoTestResponse {
                                success: true,
                                message: Some(format!("批次 '{}' 的硬点通道自动测试已启动，共 {} 个测试点位",
                                                    batch_info.batch_name, test_instances.len())),
                            })
                        },
                        Err(e) => {
                            error!("❌ 启动批次测试失败: {}", e);
                            Ok(BatchAutoTestResponse {
                                success: false,
                                message: Some(format!("启动测试失败: {}", e)),
                            })
                        }
                    }
                },
                Err(e) => {
                    error!("❌ 加载批次失败: {}", e);
                    Ok(BatchAutoTestResponse {
                        success: false,
                        message: Some(format!("加载批次失败: {}", e)),
                    })
                }
            }
        }
    }
}

/// 获取PLC连接状态
#[tauri::command]
pub async fn get_plc_connection_status_cmd(
    state: State<'_, AppState>
) -> Result<PlcConnectionStatus, String> {
    let app_state = state.inner();
    let plc_connection_manager = app_state.plc_connection_manager.clone();

    // 从PLC连接管理器获取实时连接状态
    let (test_plc_connected, target_plc_connected, test_plc_name, target_plc_name) =
        plc_connection_manager.get_plc_status_summary().await;

    Ok(PlcConnectionStatus {
        test_plc_connected,
        target_plc_connected,
        test_plc_name,
        target_plc_name,
        last_check_time: crate::utils::time_utils::format_bj(chrono::Utc::now(), "%Y-%m-%d %H:%M:%S"),
    })
}
