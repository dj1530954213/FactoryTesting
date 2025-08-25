//! # 手动测试命令模块（新版）(Manual Test Commands V2)
//!
//! ## 业务说明
//! 本模块提供更加细粒度的手动测试功能，支持分步骤的测试流程和实时监控
//! 相比旧版手动测试，提供了更灵活的测试控制和更好的用户体验
//!
//! ## 核心功能
//! ### 1. 手动测试生命周期管理
//! - **测试启动**: 初始化测试任务和状态
//! - **步骤控制**: 支持测试流程的分步执行
//! - **状态查询**: 实时查询测试进度和结果
//! - **测试完成**: 自动收集和保存测试结果
//!
//! ### 2. PLC实时监控功能
//! - **数据监控**: 实时监控通道数据变化
//! - **事件推送**: 向前端推送监控事件
//! - **监控控制**: 支持启动/停止监控
//!
//! ### 3. 专业测试功能
//! - **AI测试**: 显示值核对、报警测试、维护功能测试
//! - **AO测试**: 五点采集测试（0%、25%、50%、75%、100%）
//! - **DI测试**: 信号下发和状态确认
//! - **DO测试**: 输出控制和反馈验证
//!
//! ## 架构设计
//! - **分层设计**: 接口层 → 应用层 → 领域层 → 基础设施层
//! - **事件驱动**: 基于事件的异步通信模式
//! - **状态管理**: 维护完整的测试状态信息
//! - **错误恢复**: 支持测试异常的恢复处理
//!
//! ## 调用链路
//! ```
//! 前端测试界面 → Tauri命令 → 测试协调服务 → PLC监控服务 → 
//! PLC通信服务 → 硬件设备 → 状态反馈 → 前端更新
//! ```
//!
//! ## Rust知识点
//! - **异步编程**: 大量使用async/await处理PLC通信
//! - **状态管理**: 使用Arc<Mutex>管理共享状态
//! - **错误处理**: 完善的Result错误处理链

// === 标准库导入 ===
use std::sync::Arc;
use std::collections::HashMap;

// === Tauri相关导入 ===
use tauri::State;

// === 日志相关导入 ===
use log::{info, error, warn};

// === 序列化相关导入 ===
use serde::{Deserialize, Serialize};

// === 随机数生成器 ===
use rand::Rng;

// === 领域服务导入 ===
use crate::domain::services::plc_comm_extension::PlcServiceLegacyExt;  // PLC服务遗留扩展
use crate::domain::services::plc_communication_service::IPlcCommunicationService; // PLC通信服务接口

// ==================== 常量 ====================
/// AO 手动采集允许的百分比偏差
/// 
/// 业务说明：
/// AO测试时，测试PLC读取的实际值与期望值之间的偏差不得超过3%
/// 超出此范围则认为AO输出不准确，测试失败
/// 
/// Rust知识点：
/// const 声明编译时常量，必须指定类型
const AO_TOLERANCE_PERCENT: f64 = 5.0;

// === 数据模型导入 ===
// 业务说明：这些结构体定义了前后端交互的数据格式
use crate::models::structs::{
    StartManualTestRequest,          // 启动手动测试请求
    StartManualTestResponse,         // 启动手动测试响应
    UpdateManualTestSubItemRequest,  // 更新测试子项请求
    UpdateManualTestSubItemResponse, // 更新测试子项响应
    StartPlcMonitoringRequest,       // 启动PLC监控请求
    StartPlcMonitoringResponse,      // 启动PLC监控响应
    StopPlcMonitoringRequest,        // 停止PLC监控请求
    ManualTestStatus,                // 手动测试状态
};
// 注意：ManualTestSubItem 需要在 models 中定义
// 暂时使用字符串代替，后续需要定义正确的枚举

// === 应用层服务导入 ===
use crate::application::services::ITestCoordinationService;  // 测试协调服务接口

// === 基础设施层服务导入 ===
use crate::infrastructure::IPlcMonitoringService;            // PLC监控服务接口
use crate::infrastructure::plc_communication::global_plc_service; // 全局PLC服务实例

// === PLC配置相关导入 ===
use crate::domain::services::plc_communication_service::{PlcConnectionConfig, PlcProtocol};

/// 开始手动测试命令
/// 
/// 业务说明：
/// 启动指定通道的手动测试流程，初始化测试状态并创建测试任务
/// 这是手动测试的入口点，前端点击“开始手动测试”时调用
/// 
/// 参数说明：
/// - request: 包含测试实例ID、模块类型等信息
/// - app_state: 应用状态，提供访问测试协调服务
/// 
/// 返回值：
/// - Ok(StartManualTestResponse): 测试启动成功，返回测试状态
/// - Err(String): 启动失败，返回错误信息
/// 
/// 调用链：
/// 前端手动测试界面 -> start_manual_test_cmd -> TestCoordinationService -> 初始化测试状态
/// 
/// Rust知识点：
/// - #[tauri::command] 宏标记为Tauri命令
/// - {:?} 使用Debug trait输出结构体信息
#[tauri::command]
pub async fn start_manual_test_cmd(
    request: StartManualTestRequest,
    app_state: State<'_, crate::tauri_commands::AppState>,
) -> Result<StartManualTestResponse, String> {
    info!("🔧 [MANUAL_TEST_CMD] 开始手动测试: {:?}", request);

    // 调用测试协调服务启动手动测试
    // 业务说明：测试协调服务负责管理所有测试任务的生命周期
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
/// 
/// 业务说明：
/// 更新具体测试子项的状态（如显示值核对、报警测试等）
/// 每个手动测试可能包含多个子项，每个子项完成后调用此命令更新状态
/// 
/// 参数说明：
/// - request: 包含实例ID、子项名称、状态等信息
/// - app_state: 应用状态
/// 
/// 返回值：
/// - Ok(UpdateManualTestSubItemResponse): 更新成功，返回新状态
/// - Err(String): 更新失败，返回错误信息
/// 
/// 调用链：
/// 前端完成子项测试 -> update_manual_test_subitem_cmd -> TestCoordinationService -> 更新状态
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
/// 
/// 业务说明：
/// 查询指定测试实例的当前手动测试状态
/// 前端可以定期调用此命令获取最新状态，更新测试进度显示
/// 
/// 参数说明：
/// - instance_id: 测试实例的唯一标识符
/// - app_state: 应用状态
/// 
/// 返回值：
/// - Ok(serde_json::Value): 返回JSON格式的测试状态
/// - Err(String): 查询失败，返回错误信息
/// 
/// 调用链：
/// 前端定时查询 -> get_manual_test_status_cmd -> TestCoordinationService -> 返回状态
/// 
/// Rust知识点：
/// - serde_json::Value 可以表示任意JSON结构
/// - serde_json::json! 宏便于构建JSON对象
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
/// 
/// 业务说明：
/// 启动对指定PLC地址的实时监控，用于在手动测试过程中实时显示通道数据
/// 根据模块类型自动选择监控的PLC：
/// - AI/DI模块：监控被测PLC（因为要看被测设备的输入信号）
/// - AO/DO模块：监控测试PLC（因为要看测试台接收到的输出信号）
/// 
/// 参数说明：
/// - request: 监控请求，包含模块类型、监控地址等
/// - app_state: 应用状态
/// 
/// 特殊处理：
/// 1. 自动根据模块类型选择PLC连接
/// 2. 如果前端未提供监控地址，会从测试实例中获取
/// 
/// 调用链：
/// 前端手动测试界面 -> start_plc_monitoring_cmd -> PlcMonitoringService -> PLC实时读取
/// 
/// Rust知识点：
/// - mut request 允许修改参数
/// - match 语句的多模式匹配使用 |
#[tauri::command]
pub async fn start_plc_monitoring_cmd(
    mut request: StartPlcMonitoringRequest,
    app_state: State<'_, crate::tauri_commands::AppState>,
) -> Result<StartPlcMonitoringResponse, String> {
    // 根据模块类型补充 connection_id
    // 业务逻辑：不同类型的模块需要监控不同的PLC
    if request.connection_id.is_none() {
        let conn_id = match request.module_type {
            // DI/AI 模块监控被测对象，使用 target_connection_id
            crate::models::enums::ModuleType::AI | crate::models::enums::ModuleType::DI |
            crate::models::enums::ModuleType::DINone | crate::models::enums::ModuleType::AINone => app_state.target_connection_id.clone(),
            // DO/AO 模块监控测试台，使用 test_rig_connection_id
            crate::models::enums::ModuleType::DO | crate::models::enums::ModuleType::DONone |
            crate::models::enums::ModuleType::AO | crate::models::enums::ModuleType::AONone => app_state.test_rig_connection_id.clone(),
            // 其他未明确指定的模块类型，也默认使用测试台连接
            _ => app_state.test_rig_connection_id.clone(),
        };
        request.connection_id = Some(conn_id);
    }

    // ===== 兜底：若前端未提供监控地址，根据模块类型智能填充 =====
    // 业务说明：为了提高前端使用便利性，如果前端未提供监控地址，
    // 系统会根据模块类型自动从测试实例中获取合适的地址
    if request.monitoring_addresses.is_empty() {
        use crate::models::enums::ModuleType;
        match app_state.persistence_service.load_test_instance(&request.instance_id).await {
            Ok(Some(inst)) => {
                // 仅 DO/AO 等需要测试 PLC 的模块才兜底使用 test_plc_communication_address
                // 业务逻辑：DO/AO输出模块需要监控测试PLC的接收情况
                let need_test_plc_addr = matches!(
                    request.module_type,
                    ModuleType::DO | ModuleType::AO | ModuleType::DONone | ModuleType::AONone
                );

                if need_test_plc_addr {
                    if let Some(addr) = inst.test_plc_communication_address {
                        request.monitoring_addresses.push(addr.clone());

                        // 若未提供 address_key_map，则自动生成
                        // Rust知识点：使用Option的is_none()方法判断是否为None
                        if request.address_key_map.is_none() {
                            // 根据模块类型选择合适的键名
                            let key = if matches!(request.module_type, ModuleType::AO | ModuleType::AONone) {
                                "currentOutput"  // AO模块使用“当前输出”
                            } else {
                                "currentState"   // DO模块使用“当前状态”
                            };
                            let mut map = std::collections::HashMap::new();
                            map.insert(addr.clone(), key.to_string());
                            request.address_key_map = Some(map);
                        }
                        info!("🔧 [MANUAL_TEST_CMD] 监控地址为空，已使用测试PLC地址兜底: {}", addr);
                    } else {
                        warn!("⚠️ [MANUAL_TEST_CMD] 实例缺少 test_plc_communication_address，无法填充监控地址");
                    }
                } else {
                    // DI/AI模块不应使用测试PLC地址
                    warn!("⚠️ [MANUAL_TEST_CMD] DI 等模块未提供监控地址，且不应使用测试PLC地址兜底");
                }
            }
            Ok(None) => {
                warn!("⚠️ [MANUAL_TEST_CMD] 未找到测试实例: {}", request.instance_id);
            }
            Err(e) => {
                error!("❌ [MANUAL_TEST_CMD] 加载测试实例失败: {}", e);
            }
        }
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
/// 
/// 业务说明：
/// 停止指定测试实例的PLC实时监控
/// 通常在手动测试完成或切换到其他测试项时调用
/// 
/// 参数说明：
/// - request: 停止监控请求，包含实例ID
/// - app_state: 应用状态
/// 
/// 返回值：
/// - Ok(serde_json::Value): 停止成功，返回成功消息
/// - Err(String): 停止失败，返回错误信息
/// 
/// 调用链：
/// 前端结束监控 -> stop_plc_monitoring_cmd -> PlcMonitoringService -> 停止监控任务
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
// 业务说明：AI（模拟量输入）模块的手动测试需要特殊的功能：
// 1. 显示值核对：下发测试值到测试PLC，验证被测设备显示正确
// 2. 报警测试：下发触发报警的值，验证报警功能
// 3. 维护功能：启用/禁用维护模式

/// AI手动测试显示值核对请求结构体
/// 
/// 业务说明：
/// 用于验证AI通道的显示值是否准确
/// 测试流程：测试PLC模拟传感器信号 -> 被测设备读取 -> 显示在HMI上
/// 
/// 字段说明：
/// - instance_id: 测试实例标识符
/// - test_value: 要测试的工程值（如温度、压力等实际物理量）
/// 
/// Rust知识点：
/// - #[derive(Debug, Clone, Serialize, Deserialize)] 自动派生常用trait
/// - f64 双精度浮点数，用于精确表示模拟量
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiShowValueTestRequest {
    pub instance_id: String,
    pub test_value: f64,  // 用户输入或随机生成的测试值（工程值）
}

/// AI手动测试显示值核对响应结构体
/// 
/// 业务说明：
/// 返回显示值测试的执行结果
/// 
/// 字段说明：
/// - success: 测试是否成功
/// - message: 结果消息
/// - sent_percentage: 发送到测试PLC的百分比值（0-100%）
/// - test_plc_address: 使用的测试PLC地址，方便排查问题
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

/// AI手动测试 - 生成随机显示值命令
/// 
/// 业务说明：
/// 为AI测试生成一个在量程范围内的随机值
/// 方便测试人员快速选择不同的测试值，避免手动输入
/// 
/// 参数说明：
/// - request: 包含测试实例ID
/// - app_state: 应用状态
/// 
/// 返回值：
/// - Ok(GenerateRandomValueResponse): 生成成功，返回随机值及量程范围
/// - Err(String): 生成失败，返回错误信息
/// 
/// 算法说明：
/// 随机值 = 低限 + rand() * (高限 - 低限)
/// 
/// 调用链：
/// 前端点击随机值按钮 -> generate_random_display_value_cmd -> 获取通道定义 -> 计算随机值
/// 
/// Rust知识点：
/// - rand::thread_rng() 创建线程局部的随机数生成器
/// - gen::<f64>() 生成[0, 1)范围内的浮点数
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
    // 业务说明：从通道定义中获取量程范围，确保随机值在有效范围内
    // Rust知识点：unwrap_or() 提供默认值，避免Option为None时程序崩溃
    let low_limit = definition.range_low_limit.unwrap_or(0.0) as f64;
    let high_limit = definition.range_high_limit.unwrap_or(100.0) as f64;

    // 验证量程范围的有效性
    if high_limit <= low_limit {
        error!("❌ [AI_MANUAL_TEST] 无效的限值范围: 低限={}, 高限={}", low_limit, high_limit);
        return Err("无效的限值范围".to_string());
    }

    // 计算随机值
    let range = high_limit - low_limit;
    let mut rng = rand::thread_rng();  // 创建随机数生成器
    let random_value = low_limit + (rng.gen::<f64>() * range);  // 生成在范围内的随机值

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

/// AI手动测试 - 显示值核对测试命令
/// 
/// 业务说明：
/// 将指定的工程值转换为百分比并下发到测试PLC
/// 测试PLC模拟传感器信号，被测设备应显示对应的工程值
/// 
/// 测试流程：
/// 1. 获取测试实例和通道定义
/// 2. 查询测试PLC地址
/// 3. 将工程值转换为百分比
/// 4. 写入到测试PLC
/// 
/// 参数说明：
/// - request: 包含实例ID和测试值
/// - app_state: 应用状态
/// 
/// 调用链：
/// 前端输入测试值 -> ai_show_value_test_cmd -> 获取PLC地址 -> 写入PLC
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
    // 业务说明：测试PLC使用百分比表示模拟量，需要将工程值转换
    // 转换公式：百分比 = (工程值 - 低限) / (高限 - 低限) * 100
    let percentage = convert_engineering_to_percentage(
        request.test_value,
        definition.range_low_limit.unwrap_or(0.0) as f64,
        definition.range_high_limit.unwrap_or(100.0) as f64,
    );

    // 实际执行PLC写入操作
    // 业务说明：调用辅助函数将百分比值写入到测试PLC
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

/// AI手动测试 - 报警测试命令
/// 
/// 业务说明：
/// 下发触发各类报警的测试值，验证被测设备的报警功能
/// 支持四种报警类型：LL（低低）、L（低）、H（高）、HH（高高）
/// 
/// 测试策略：
/// - 低低报警：SLL设定值 - 1%量程
/// - 低报警：SL设定值 - 1%量程
/// - 高报警：SH设定值 + 1%量程
/// - 高高报警：SHH设定值 + 1%量程
/// 
/// 参数说明：
/// - request: 包含实例ID和报警类型
/// - app_state: 应用状态
/// 
/// 调用链：
/// 前端选择报警类型 -> ai_alarm_test_cmd -> 计算测试值 -> 写入PLC
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

    // ===== 生成测试值 =====
    // 新测试策略：
    // LL : SLL - 1% 量程
    // L  : 随机值 ∈ (SLL , SL)
    // H  : 随机值 ∈ (SH , SHH)
    // HH : SHH + 1% 量程
    let range = definition.range_high_limit.unwrap_or(100.0) as f64
        - definition.range_low_limit.unwrap_or(0.0) as f64;
    let offset = range * 0.01;

    // 生成测试值，根据报警类型采用不同策略
    let mut test_value = match request.alarm_type.as_str() {
        // 低低报警：固定偏移
        "LL" => definition.sll_set_value.unwrap_or(0.0) as f64 - offset,
        // 低报警：在 SLL 与 SL 之间随机
        "L"  => {
            let ll = definition.sll_set_value.unwrap_or(0.0) as f64;
            let l  = definition.sl_set_value.unwrap_or(10.0) as f64;
            if l > ll { rand::thread_rng().gen_range(ll..l) } else { l - offset }
        }
        // 高报警：在 SH 与 SHH 之间随机
        "H"  => {
            let h  = definition.sh_set_value.unwrap_or(90.0) as f64;
            let hh = definition.shh_set_value.unwrap_or(100.0) as f64;
            if hh > h { rand::thread_rng().gen_range(h..hh) } else { h + offset }
        }
        // 高高报警：固定偏移
        "HH" => definition.shh_set_value.unwrap_or(100.0) as f64 + offset,
        _ => {
            error!("❌ [AI_MANUAL_TEST] 无效的报警类型: {}", request.alarm_type);
            return Err("无效的报警类型".to_string());
        }
    };

    // 若生成值超出量程，进行夹紧处理
    let low_limit  = definition.range_low_limit.unwrap_or(0.0) as f64;
    let high_limit = definition.range_high_limit.unwrap_or(100.0) as f64;
    if test_value < low_limit {
        test_value = low_limit;
    }
    if test_value > high_limit {
        test_value = high_limit;
    }

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

/// AI手动测试 - 维护功能测试命令
/// 
/// 业务说明：
/// 测试AI通道的维护功能，包括启用和复位维护模式
/// 维护模式通常用于临时禁用报警或锁定输出值
/// 
/// 注意事项：
/// - 维护功能写入到被测PLC，不是测试PLC
/// - 使用布尔值控制维护状态
/// 
/// 参数说明：
/// - request: 包含实例ID和启用/禁用标志
/// - app_state: 应用状态
/// 
/// 调用链：
/// 前端点击维护按钮 -> ai_maintenance_test_cmd -> 获取维护地址 -> 写入被测PLC
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
    // 业务说明：维护开关地址从通道定义中获取
    // Modbus地址规范化：确保地址格式一致，避免通信错误
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

/// AI手动测试 - 复位到显示值命令
/// 
/// 业务说明：
/// 在报警测试后，将AI通道复位到正常显示值
/// 本质上与显示值核对测试相同，只是用途不同
/// 
/// 参数说明：
/// - request: 与显示值测试相同的请求结构
/// - app_state: 应用状态
/// 
/// Rust知识点：
/// - 函数复用：直接调用现有函数避免代码重复
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

/// 手动测试子项完成确认命令
/// 
/// 业务说明：
/// 当测试人员完成某个手动测试子项后，调用此命令记录测试结果
/// 会创建一个成功的测试结果并更新到状态管理器
/// 
/// 参数说明：
/// - instance_id: 测试实例标识符
/// - sub_item: 子项名称字符串（如"ShowValueCheck"、"LowAlarmTest"等）
/// - app_state: 应用状态
/// 
/// 字符串与枚举映射：
/// - ShowValueCheck -> HardPoint
/// - LowLowAlarmTest -> LowLowAlarm
/// - LowAlarmTest -> LowAlarm
/// - HighAlarmTest -> HighAlarm
/// - HighHighAlarmTest -> HighHighAlarm
/// - TrendCheck -> Trend
/// - ReportCheck -> Report
/// - MaintenanceFunction -> Maintenance
/// 
/// 调用链：
/// 前端确认完成 -> complete_manual_test_subitem_cmd -> 创建RawTestOutcome -> ChannelStateManager
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
// 业务说明：以下是一系列辅助函数，用于支持上述命令的实现
// 这些函数封装了常用的业务逻辑，提高代码复用性

/// 将采集百分比映射到 SubTestItem 枚举
/// 
/// 业务说明：
/// AO测试需要在五个固定点进行采集（0%、25%、50%、75%、100%）
/// 每个采集点对应一个特定的测试子项
/// 
/// 参数：
/// - percent: 采集点百分比（只能是0、25、50、75、100）
/// 
/// 返回值：
/// 对应的SubTestItem枚举值
/// 
/// Rust知识点：
/// - use SubTestItem::* 可以省略枚举前缀
/// - match 必须覆盖所有情况，_ 处理其他情况
fn percent_to_sub_test(percent: u8) -> crate::models::enums::SubTestItem {
    use crate::models::enums::SubTestItem::*;
    match percent {
        0 => Output0Percent,
        25 => Output25Percent,
        50 => Output50Percent,
        75 => Output75Percent,
        100 => Output100Percent,
        _ => HardPoint, // 不应发生，但需要处理以满足编译器要求
    }
}

/// AO 手动采集命令
#[tauri::command]
pub async fn capture_ao_point_cmd(
    instance_id: String,
    checkpoint_percent: u8, // 0 / 25 / 50 / 75 / 100
    app_state: State<'_, crate::tauri_commands::AppState>,
) -> Result<serde_json::Value, String> {
    info!("📥 [AO_CMD] 收到采集请求: instance={} percent={}", instance_id, checkpoint_percent);
    if ![0u8, 25, 50, 75, 100].contains(&checkpoint_percent) {
        return Err("不支持的采集点百分比".to_string());
    }

    // 获取实例与定义
    let (instance, definition) = get_instance_and_definition(&app_state, &instance_id).await?;

    // 获取测试PLC AI 地址
    let test_plc_address = get_test_plc_address(&app_state, &instance).await?;

    // 量程
    let range_low = definition.range_low_limit.unwrap_or(0.0) as f64;
    let range_high = definition.range_high_limit.unwrap_or(100.0) as f64;
    if range_high <= range_low {
        return Err("无效的量程范围".into());
    }

    let expected_value = range_low + (range_high - range_low) * checkpoint_percent as f64 / 100.0;

    // 读取当前值
    let plc_service_arc = crate::infrastructure::plc_communication::global_plc_service();
    let plc_service: std::sync::Arc<dyn IPlcCommunicationService + Send + Sync> = plc_service_arc;
    let conn_id = &app_state.test_rig_connection_id;
    info!("🔌 [AO_CMD] 读取 PLC 地址 {}", test_plc_address);
    let actual_value = plc_service
        .read_float32_by_id(conn_id, &test_plc_address)
        .await
        .map_err(|e| format!("读取测试PLC失败: {}", e))? as f64;

    // 偏差
    let deviation = ((actual_value - expected_value) / (range_high - range_low) * 100.0).abs();
    if deviation > AO_TOLERANCE_PERCENT {
        return Err(format!(
            "偏差 {:.2}% 超过允许值 {:.1}% (当前值 {:.3}, 期望 {:.3})",
            deviation, AO_TOLERANCE_PERCENT, actual_value, expected_value
        ));
    }

    // 写入 RawTestOutcome
    let mut outcome = crate::models::RawTestOutcome::success(
        instance_id.clone(),
        percent_to_sub_test(checkpoint_percent),
    );
    outcome.raw_value_read = Some(format!("{:.3}", actual_value));
    outcome.eng_value_calculated = Some(format!("{:.3}", actual_value));
    outcome.message = Some(format!("AO 手动采集 {}%", checkpoint_percent));
    // 百分比结果写入对应字段
    info!("📊 [AO_CMD] 偏差 {:.2}% , 保存 RawTestOutcome", deviation);
    match checkpoint_percent {
        0 => outcome.test_result_0_percent = Some(actual_value),
        25 => outcome.test_result_25_percent = Some(actual_value),
        50 => outcome.test_result_50_percent = Some(actual_value),
        75 => outcome.test_result_75_percent = Some(actual_value),
        100 => outcome.test_result_100_percent = Some(actual_value),
        _ => {}
    }

    info!("💾 [AO_CMD] 调用 ChannelStateManager 更新结果");
    app_state
        .channel_state_manager
        .update_test_result(outcome)
        .await
        .map_err(|e| format!("保存测试结果失败: {}", e))?;

    Ok(serde_json::json!({
        "success": true,
        "actual_value": actual_value,
        "deviation_percent": deviation
    }))
}

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
/// 
/// 业务说明：
/// - 用于AI测试的维护功能，写入布尔值到被测PLC
/// - 主要用于触发报警、维护等状态
/// - 使用已建立的被测PLC连接
/// 
/// 参数：
/// - app_state: 应用状态，包含PLC服务
/// - address: Modbus地址（会自动规范化）
/// - value: 要写入的布尔值
/// 
/// Rust知识点：
/// - async fn 声明异步函数，返回Future
/// - &State<'_, T> 是Tauri的状态管理类型
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
/// 
/// 业务说明：
/// - Modbus地址需要规范化为5位数字格式
/// - 例如："123" -> "00123"，"0X456" -> "00456"
/// - 仅保留数字部分，忽略前缀
/// 
/// 参数：
/// - address: 原始地址字符串
/// 
/// 返回：
/// - 规范化后的5位地址字符串
/// 
/// Rust知识点：
/// - chars() 将字符串转换为字符迭代器
/// - filter() 过滤满足条件的元素
/// - collect() 将迭代器收集为指定类型
/// - format! 宏用于格式化字符串，{:0>5} 表示右对齐，左侧补0，总宽度5
fn normalize_modbus_address(address: &str) -> String {
    // 仅保留数字字符
    let digits: String = address.chars().filter(|c| c.is_ascii_digit()).collect();
    if digits.is_empty() {
        return address.to_string(); // 返回原样，后续写入会报错指出
    }
    format!("{:0>5}", digits)
}

/// ==================== DI 手动测试专用命令 ====================
/// 
/// 业务说明：
/// DI（数字量输入）测试需要测试PLC的DO（数字量输出）来模拟信号
/// 测试流程：测试PLC DO -> 被测PLC DI -> 验证被测系统响应

/// DI 信号下发请求（将测试 PLC DO 通道置位或复位）
/// 
/// 业务说明：
/// - 用于触发DI测试信号
/// - 通过测试PLC的DO输出来模拟DI输入
/// 
/// Rust知识点：
/// - #[derive] 自动实现指定的trait
/// - Serialize/Deserialize 用于JSON序列化
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiSignalTestRequest {
    pub instance_id: String,      // 测试实例ID
    pub enable: bool,             // true = 置位 (ON), false = 复位 (OFF)
}

/// DI 信号下发响应
/// 
/// 业务说明：
/// - 返回信号下发的执行结果
/// - 包含实际使用的PLC地址，便于调试
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiSignalTestResponse {
    pub success: bool,            // 操作是否成功
    pub message: String,          // 结果消息
    pub test_plc_address: String, // 实际使用的测试PLC地址
}

/// DI 手动测试 - 信号下发
/// 
/// 业务说明：
/// - 前端调用此命令进行DI测试的信号下发
/// - 通过测试PLC的DO输出来触发被测PLC的DI输入
/// - 支持置位（ON）和复位（OFF）操作
/// 
/// 参数：
/// - request: DI信号测试请求，包含实例ID和信号状态
/// - app_state: 应用状态
/// 
/// 返回：
/// - Ok: 信号下发响应，包含操作结果
/// - Err: 错误信息
/// 
/// 调用链：
/// 前端DI测试界面 -> di_signal_test_cmd -> write_bool_to_test_plc -> 测试PLC
/// 
/// Rust知识点：
/// - #[tauri::command] 标记为Tauri命令
/// - State<'_, T> 是Tauri的状态管理类型
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
/// 
/// 业务说明：
/// - 用于DI测试时控制测试PLC的DO输出
/// - 测试PLC的DO连接到被测PLC的DI，实现信号模拟
/// - 使用独立的连接ID避免与其他测试冲突
/// 
/// 参数：
/// - app_state: 应用状态，包含PLC配置服务
/// - address: Modbus地址（会自动规范化为5位）
/// - value: 要写入的布尔值（true=ON, false=OFF）
/// 
/// 返回：
/// - Ok(()): 写入成功
/// - Err: 错误信息
/// 
/// Rust知识点：
/// - async/await 异步编程
/// - Result<T, E> 错误处理
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

// ==================== DO 手动测试专用命令 ====================
// 业务说明：DO（数字量输出）模块的手动测试需要特殊的功能：
// 1. 数字状态采集：按照低-高-低的序列采集DO输出状态
// 2. 状态验证：验证测试PLC接收到的状态与预期一致

/// DO 数字状态采集请求结构体
/// 
/// 业务说明：
/// 用于采集DO通道的数字状态（低电平/高电平）
/// 测试流程：被测设备DO输出 -> 测试PLC DI输入 -> 验证状态一致性
/// 
/// 字段说明：
/// - instance_id: 测试实例标识符
/// - step_number: 步骤号（1=第1次低电平, 2=高电平, 3=第2次低电平）
/// - expected_state: 期望状态（true=高电平, false=低电平）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DoStateTestRequest {
    pub instance_id: String,
    pub step_number: u8,     // 1, 2, 3
    pub expected_state: bool, // true = 高电平, false = 低电平
}

/// DO 数字状态采集响应结构体
/// 
/// 业务说明：
/// 返回数字状态采集的执行结果
/// 
/// 字段说明：
/// - success: 采集是否成功
/// - message: 结果消息
/// - actual_value: 实际读取到的状态
/// - test_plc_address: 使用的测试PLC地址
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DoStateTestResponse {
    pub success: bool,
    pub message: String,
    pub actual_value: bool,       // 实际读取的状态值
    pub test_plc_address: String, // 测试PLC通信地址
}

/// DO手动测试 - 数字状态采集命令
/// 
/// 业务说明：
/// 采集DO通道的输出状态，按照低-高-低的序列进行
/// 读取测试PLC的DI输入来验证被测设备DO输出的正确性
/// 
/// 参数说明：
/// - instanceId: 实例ID（驼峰命名匹配前端）
/// - stepNumber: 步骤号（驼峰命名匹配前端）
/// - expectedState: 期望状态（驼峰命名匹配前端）
/// - app_state: 应用状态
/// 
/// 返回值：
/// - Ok(DoStateTestResponse): 采集成功，返回实际状态
/// - Err(String): 采集失败，返回错误信息
/// 
/// 调用链：
/// 前端DO测试界面 -> capture_do_state_cmd -> 读取测试PLC状态 -> 保存到digital_test_steps_json
#[tauri::command]
pub async fn capture_do_state_cmd(
    instanceId: String,
    stepNumber: u8,
    expectedState: bool,
    app_state: State<'_, crate::tauri_commands::AppState>,
) -> Result<DoStateTestResponse, String> {
    info!("📥 [DO_CMD] 收到状态采集请求: instance={} step={} expected={}",
          instanceId, stepNumber, expectedState);

    // 验证步骤号有效性
    if ![1u8, 2, 3].contains(&stepNumber) {
        return Err("不支持的采集步骤号，仅支持1、2、3".to_string());
    }

    // 获取实例与定义
    let (instance, _definition) = get_instance_and_definition(&app_state, &instanceId).await?;

    // 获取测试PLC DI地址（用于读取DO输出状态）
    let test_plc_address = get_test_plc_address(&app_state, &instance).await?;

    // 读取当前数字状态
    let plc_service_arc = crate::infrastructure::plc_communication::global_plc_service();
    let plc_service: std::sync::Arc<dyn IPlcCommunicationService + Send + Sync> = plc_service_arc;
    let conn_id = &app_state.test_rig_connection_id;
    
    info!("🔌 [DO_CMD] 读取测试PLC DI地址 {}", test_plc_address);
    let actual_state = plc_service
        .read_bool_by_id(conn_id, &test_plc_address)
        .await
        .map_err(|e| format!("读取测试PLC失败: {}", e))?;

    info!("📊 [DO_CMD] 步骤{}：期望={}, 实际={}", 
          stepNumber, expectedState, actual_state);

    // 校验实际状态是否与期望状态一致（DO测试要求严格相等）
    if actual_state != expectedState {
        return Err(format!(
            "状态校验失败: 期望{}, 实际{} (步骤{})",
            if expectedState { "高电平" } else { "低电平" },
            if actual_state { "高电平" } else { "低电平" },
            stepNumber
        ));
    }

    // 创建数字测试步骤结构
    let digital_step = crate::models::structs::DigitalTestStep {
        step_number: stepNumber as u32,
        step_description: format!("DO手动采集步骤{}", stepNumber),
        set_value: expectedState,
        expected_reading: expectedState,
        actual_reading: actual_state,
        status: crate::models::enums::SubTestStatus::Passed,
        timestamp: chrono::Utc::now(),
    };

    // 更新或创建RawTestOutcome，保存到digital_test_steps字段
    let mut outcome = crate::models::RawTestOutcome::success(
        instanceId.clone(),
        crate::models::enums::SubTestItem::HardPoint, // DO测试使用HardPoint
    );
    outcome.message = Some(format!("DO 手动状态采集 步骤{}", stepNumber));
    outcome.raw_value_read = Some(format!("{}", actual_state));
    outcome.digital_steps = Some(vec![digital_step]);

    info!("💾 [DO_CMD] 调用 ChannelStateManager 更新数字测试步骤");
    app_state
        .channel_state_manager
        .update_test_result(outcome)
        .await
        .map_err(|e| format!("保存测试结果失败: {}", e))?;

    Ok(DoStateTestResponse {
        success: true,
        message: format!("步骤{}状态采集成功", stepNumber),
        actual_value: actual_state,
        test_plc_address,
    })
}
