/// 手动测试相关的Tauri命令模块
///
/// 业务说明：
/// 本模块提供手动测试相关的所有功能，包括：
/// - 手动子测试执行：支持用户手动执行单个测试项
/// - 通道数据读写：直接与PLC通信，读取或写入通道值
/// - PLC连接管理：建立和维护与测试PLC、被测PLC的连接
/// - 批次自动测试：启动整个批次的自动化测试流程
/// 
/// 架构定位：
/// - 属于接口层，直接对接前端调用
/// - 调用应用层和领域层服务完成具体业务
/// - 处理前端参数验证和结果转换
/// 
/// 调用链：
/// 前端 -> Tauri命令 -> 应用服务 -> 领域服务 -> 基础设施
/// 
/// Rust知识点：
/// - #[tauri::command] 宏用于暴露函数给前端调用
/// - State<T> 用于访问Tauri管理的应用状态
/// - async/await 处理异步操作

// === Tauri相关导入 ===
// Rust知识点：State用于访问Tauri管理的应用状态，Manager用于动态服务管理
use tauri::{State, Manager};

// === 标准库导入 ===
// Rust知识点：Arc<T> 提供线程安全的引用计数，HashMap用于键值对存储
use std::sync::Arc;
use std::collections::HashMap;

// === 应用层服务导入 ===
// 业务说明：量程设置服务用于管理AI/AO通道的量程范围配置
use crate::application::services::range_setting_service::{
    DynamicRangeSettingService,      // 动态服务容器，支持运行时替换
    ChannelRangeSettingService,      // 量程设置服务具体实现
    IChannelRangeSettingService      // 量程设置服务接口
};

// === 领域层服务导入 ===
// 业务说明：这些是核心业务逻辑的抽象接口
use crate::domain::services::IRangeRegisterRepository;                    // 量程寄存器仓储接口
use crate::domain::services::range_value_calculator::{
    DefaultRangeValueCalculator,     // 默认的量程值计算器
    IRangeValueCalculator           // 量程值计算器接口
};
use crate::domain::services::plc_communication_service::IPlcCommunicationService; // PLC通信服务接口

// === 基础设施层导入 ===
// 业务说明：具体的技术实现
use crate::infrastructure::range_register_repository::RangeRegisterRepository;    // 量程寄存器仓储实现

// === 序列化相关导入 ===
// Rust知识点：Serde用于JSON序列化/反序列化
use serde::{Deserialize, Serialize};

// === 数据模型导入 ===
// 业务说明：系统中的核心数据结构
use crate::models::{SubTestItem, PointDataType, RawTestOutcome};
use crate::tauri_commands::AppState;

// === 日志相关导入 ===
// Rust知识点：log crate提供统一的日志接口
use log::{info, error, warn};

// === 异步相关导入 ===
// Rust知识点：tokio提供异步运行时支持
use tokio::time::{sleep, Duration};

/// 执行手动子测试的参数结构体
/// 
/// 业务说明：
/// 前端执行手动测试时传递的参数，包含测试实例标识、测试项类型和附加参数
/// 
/// 字段说明：
/// - instance_id: 通道测试实例的唯一标识符
/// - sub_test_item: 具体的测试项枚举（如AI显示值测试、AO输出测试等）
/// - params: 可选的附加参数，使用JSON Value支持动态结构
/// 
/// Rust知识点：
/// - #[derive(Debug, Deserialize)] 自动派生调试输出和反序列化能力
/// - Option<T> 表示可选值，None表示缺失
/// - HashMap<String, serde_json::Value> 支持动态的键值对参数
#[derive(Debug, Deserialize)]
pub struct ExecuteManualSubTestCmdArgs {
    pub instance_id: String,
    pub sub_test_item: SubTestItem,
    pub params: Option<HashMap<String, serde_json::Value>>,
}

/// 读取通道值的参数结构体
/// 
/// 业务说明：
/// 前端请求读取PLC通道值时的参数，用于指定要读取的具体通道和数据类型
/// 
/// 字段说明：
/// - instance_id: 通道测试实例ID，用于关联测试上下文
/// - plc_address: PLC地址字符串（如"40001"、"DB1.DBD0"）
/// - data_type: 期望的数据类型，用于正确解析读取的原始数据
/// 
/// Rust知识点：
/// - PointDataType 是枚举类型，确保类型安全
/// - String 类型在Rust中是堆分配的可变字符串
#[derive(Debug, Deserialize)]
pub struct ReadChannelValueCmdArgs {
    pub instance_id: String,
    pub plc_address: String,
    pub data_type: PointDataType,
}

/// 写入通道值的参数结构体
/// 
/// 业务说明：
/// 前端请求写入PLC通道值时的参数，包含地址、数据类型和要写入的值
/// 
/// 字段说明：
/// - instance_id: 通道测试实例ID，用于关联测试上下文
/// - plc_address: PLC地址字符串，指定写入位置
/// - data_type: 数据类型，用于验证写入值的类型正确性
/// - value_to_write: 要写入的值，使用JSON Value支持多种数据类型
/// 
/// Rust知识点：
/// - serde_json::Value 是动态JSON类型，可以表示任意JSON值
/// - 类型验证在运行时进行，确保写入值与声明类型匹配
#[derive(Debug, Deserialize)]
pub struct WriteChannelValueCmdArgs {
    pub instance_id: String,
    pub plc_address: String,
    pub data_type: PointDataType,
    pub value_to_write: serde_json::Value,
}

/// 执行手动子测试命令
/// 
/// 业务说明：
/// 执行单个手动测试项，创建测试结果并更新测试实例状态
/// 当前实现返回模拟数据，实际应根据测试类型执行具体测试逻辑
/// 
/// 参数说明：
/// - args: 包含测试实例ID、测试项类型和附加参数
/// - state: 应用状态，提供访问持久化服务和状态管理器
/// 
/// 返回值：
/// - Ok(RawTestOutcome): 测试执行成功，返回测试结果
/// - Err(String): 测试执行失败，返回错误信息
/// 
/// 调用链：
/// 前端手动测试界面 -> execute_manual_sub_test_cmd -> ChannelStateManager -> 更新测试状态
/// 
/// Rust知识点：
/// - async fn 声明异步函数，返回Future
/// - Result<T, E> 用于错误处理
/// - State<'_, T> 是Tauri的状态访问器
#[tauri::command]
pub async fn execute_manual_sub_test_cmd(
    args: ExecuteManualSubTestCmdArgs,
    state: State<'_, AppState>
) -> Result<RawTestOutcome, String> {
    info!("执行手动子测试: 实例ID={}, 测试项={:?}", args.instance_id, args.sub_test_item);
    
    // 获取测试实例
    // 业务说明：从持久化服务加载测试实例，验证实例存在性
    // Rust知识点：match表达式处理Option和Result的多种情况
    let instance = match state.persistence_service.load_test_instance(&args.instance_id).await {
        Ok(Some(inst)) => inst,                                              // 成功找到实例
        Ok(None) => return Err(format!("测试实例不存在: {}", args.instance_id)), // 实例不存在
        Err(e) => {
            error!("获取测试实例失败: {}", e);
            return Err(format!("获取测试实例失败: {}", e));                    // 数据库查询失败
        }
    };
    
    // 创建测试结果
    // 业务说明：构建测试结果数据结构，记录测试执行情况
    // 注意：当前为模拟实现，实际应执行具体的测试逻辑
    let outcome = RawTestOutcome {
        channel_instance_id: args.instance_id.clone(),        // 关联的测试实例ID
        sub_test_item: args.sub_test_item,                   // 测试项类型
        success: true,                                       // 手动测试默认成功，实际应根据用户输入
        raw_value_read: Some("手动测试值".to_string()),      // 原始读取值
        eng_value_calculated: Some("手动工程值".to_string()), // 工程单位转换后的值
        message: Some("手动测试完成".to_string()),          // 测试消息
        start_time: chrono::Utc::now(),                     // 测试开始时间
        end_time: chrono::Utc::now(),                       // 测试结束时间
        readings: None,                                     // AI/AO测试的多点读数
        digital_steps: None,                                // DI/DO测试的步骤记录
        test_result_0_percent: None,                        // 0%量程测试结果
        test_result_25_percent: None,                       // 25%量程测试结果
        test_result_50_percent: None,                       // 50%量程测试结果
        test_result_75_percent: None,                       // 75%量程测试结果
        test_result_100_percent: None,                      // 100%量程测试结果
        details: args.params.unwrap_or_default(),           // 附加参数
    };
    
    // 更新测试实例状态
    // 业务说明：将测试结果保存到状态管理器，更新测试进度
    // Rust知识点：if let 模式匹配用于处理Result的错误情况
    if let Err(e) = state.channel_state_manager.update_test_result(outcome.clone()).await {
        error!("更新测试实例状态失败: {}", e);
        return Err(format!("更新测试实例状态失败: {}", e));
    }
    
    info!("手动子测试执行完成");
    Ok(outcome)  // 返回测试结果供前端显示
}

/// PLC通道数据读取命令
///
/// **业务作用**:
/// - 从PLC设备读取指定通道的实时数据值
/// - 支持多种数据类型的读取操作
/// - 为手动测试和数据监控提供数据源
/// - 实现前端界面的实时数据显示
///
/// **前后端交互**:
/// - **前端调用**: 用户查看通道值或执行手动测试时触发
/// - **参数**: ReadChannelValueCmdArgs包含实例ID、PLC地址、数据类型
/// - **返回值**: serde_json::Value动态类型，适应不同数据类型
/// - **错误处理**: PLC通信失败时返回详细错误信息
///
/// **参数说明**:
/// - `instance_id`: 通道测试实例的唯一标识符
/// - `plc_address`: PLC中的具体地址（如"40001", "DB1.DBD0"）
/// - `data_type`: 期望的数据类型（Bool, Int, Float, String等）
///
/// **数据类型支持**:
/// - **Bool**: 布尔值，用于开关状态、报警信号等
/// - **Int/Int16/Int32**: 整数类型，用于计数值、状态码等
/// - **UInt16/UInt32**: 无符号整数，用于正数范围的数值
/// - **Float/Double**: 浮点数，用于模拟量数值、传感器读数等
/// - **String**: 字符串，用于设备名称、状态描述等
///
/// **当前实现**:
/// - 目前返回模拟数据，便于前端开发和测试
/// - 模拟数据覆盖所有支持的数据类型
/// - 每种类型都有合理的默认值
///
/// **数据类型转换**:
/// - 使用serde_json::Value统一表示不同类型的数据
/// - 浮点数转换时处理精度丢失的情况
/// - 字符串类型支持中文和特殊字符
///
/// **未来扩展**:
/// - 集成PLC通信服务进行真实读取
/// - 添加数据缓存机制提高性能
/// - 支持批量读取操作
/// - 添加数据验证和范围检查
///
/// **Rust知识点**:
/// - `serde_json::Value`: 动态JSON值类型，支持任意JSON数据
/// - `match`表达式: 模式匹配，根据数据类型返回不同值
/// - `unwrap_or()`: 错误处理，提供默认值避免panic
/// - `from_f64()`: 浮点数转换，可能失败需要处理
#[tauri::command]
pub async fn read_channel_value_cmd(
    args: ReadChannelValueCmdArgs,
    state: State<'_, AppState>
) -> Result<serde_json::Value, String> {
    info!("读取通道值: 实例ID={}, 地址={}, 类型={:?}",
          args.instance_id, args.plc_address, args.data_type);

    // 这里应该调用PLC通信服务读取实际值
    // **当前状态**: 返回模拟数据用于前端开发
    // **未来改进**: 集成PLC通信服务进行真实读取
    let mock_value = match args.data_type {
        // 布尔类型 - 用于开关状态、报警信号等
        PointDataType::Bool => serde_json::Value::Bool(true),

        // 整数类型 - 用于计数值、状态码等
        PointDataType::Int => serde_json::Value::Number(serde_json::Number::from(42)),

        // 32位浮点数 - 用于模拟量数值
        PointDataType::Float => serde_json::Value::Number(
            serde_json::Number::from_f64(3.14159).unwrap_or(serde_json::Number::from(0))
        ),

        // 字符串类型 - 用于设备名称、状态描述等
        PointDataType::String => serde_json::Value::String("测试字符串".to_string()),

        // 64位浮点数 - 用于高精度数值
        PointDataType::Double => serde_json::Value::Number(
            serde_json::Number::from_f64(3.14159265359).unwrap_or(serde_json::Number::from(0))
        ),

        // 16位有符号整数
        PointDataType::Int16 => serde_json::Value::Number(serde_json::Number::from(16)),

        // 32位有符号整数
        PointDataType::Int32 => serde_json::Value::Number(serde_json::Number::from(32)),

        // 16位无符号整数
        PointDataType::UInt16 => serde_json::Value::Number(serde_json::Number::from(16)),

        // 32位无符号整数
        PointDataType::UInt32 => serde_json::Value::Number(serde_json::Number::from(32)),
    };

    info!("通道值读取完成: {:?}", mock_value);
    Ok(mock_value) // 返回JSON格式的数据值
}

/// PLC通道数据写入命令
///
/// **业务作用**:
/// - 向PLC设备写入指定通道的数据值
/// - 支持多种数据类型的写入操作
/// - 为手动测试和设备控制提供写入能力
/// - 实现前端界面的设备控制功能
///
/// **前后端交互**:
/// - **前端调用**: 用户设置通道值或执行控制操作时触发
/// - **参数**: WriteChannelValueCmdArgs包含实例ID、PLC地址、数据类型、写入值
/// - **返回值**: Result<(), String>，成功时返回空，失败时返回错误信息
/// - **错误处理**: 类型不匹配或PLC通信失败时返回详细错误
///
/// **参数说明**:
/// - `instance_id`: 通道测试实例的唯一标识符
/// - `plc_address`: PLC中的具体地址（如"40001", "DB1.DBD0"）
/// - `data_type`: 数据类型，用于验证写入值的类型正确性
/// - `value_to_write`: 要写入的JSON格式数据值
///
/// **数据类型验证**:
/// - **Bool**: 验证是否为布尔值类型
/// - **数值类型**: 验证是否为数字类型（Int, Float, Double等）
/// - **String**: 验证是否为字符串类型
/// - **类型安全**: 写入前严格验证数据类型匹配
///
/// **安全考虑**:
/// - **类型验证**: 防止类型不匹配导致的数据错误
/// - **写入确认**: 确保数据正确写入到PLC
/// - **错误处理**: 提供详细的错误信息便于故障排查
/// - **审计日志**: 记录所有写入操作用于审计
///
/// **当前实现**:
/// - 实现了完整的类型验证逻辑
/// - 目前只记录日志，未实际写入PLC
/// - 为真实PLC集成预留了接口
///
/// **数据类型转换**:
/// - 接收JSON格式的动态类型数据
/// - 根据指定的数据类型进行验证
/// - 支持所有常用的PLC数据类型
///
/// **未来扩展**:
/// - 集成PLC通信服务进行真实写入
/// - 添加写入确认和重试机制
/// - 支持批量写入操作
/// - 添加写入权限控制
///
/// **Rust知识点**:
/// - `serde_json::Value`: 动态JSON值，支持类型检查方法
/// - `is_boolean()`, `is_number()`, `is_string()`: JSON值类型检查
/// - `Result<(), String>`: 无返回值的错误处理类型
/// - `format!`: 字符串格式化宏，用于错误信息
#[tauri::command]
pub async fn write_channel_value_cmd(
    args: WriteChannelValueCmdArgs,
    state: State<'_, AppState>
) -> Result<(), String> {
    info!("写入通道值: 实例ID={}, 地址={}, 类型={:?}, 值={:?}",
          args.instance_id, args.plc_address, args.data_type, args.value_to_write);

    // 验证值类型是否匹配
    // **类型安全**: 确保写入的数据类型与期望类型一致
    // **错误预防**: 避免类型不匹配导致的PLC通信错误
    let is_valid = match args.data_type {
        PointDataType::Bool => args.value_to_write.is_boolean(),     // 布尔值验证
        PointDataType::Int => args.value_to_write.is_number(),       // 整数验证
        PointDataType::Float => args.value_to_write.is_number(),     // 浮点数验证
        PointDataType::String => args.value_to_write.is_string(),    // 字符串验证
        PointDataType::Double => args.value_to_write.is_number(),    // 双精度验证
        PointDataType::Int16 => args.value_to_write.is_number(),     // 16位整数验证
        PointDataType::Int32 => args.value_to_write.is_number(),     // 32位整数验证
        PointDataType::UInt16 => args.value_to_write.is_number(),    // 16位无符号整数验证
        PointDataType::UInt32 => args.value_to_write.is_number(),    // 32位无符号整数验证
    };

    // 类型验证失败时返回错误
    if !is_valid {
        return Err(format!("值类型不匹配: 期望{:?}类型", args.data_type));
    }

    // 这里应该调用PLC通信服务写入实际值
    // **当前状态**: 只记录日志，未实际写入PLC
    // **未来改进**: 集成PLC通信服务进行真实写入
    info!("通道值写入完成");
    Ok(()) // 返回成功结果
}

/// PLC连接响应结构体
/// 
/// 业务说明：
/// 返回给前端的PLC连接结果信息，包含连接是否成功和详细消息
/// 
/// 字段说明：
/// - success: 连接是否成功的布尔标志
/// - message: 可选的详细消息，成功时描述连接信息，失败时说明错误原因
/// 
/// Rust知识点：
/// - #[derive(Serialize)] 自动派生序列化能力，用于转换成JSON
/// - Option<String> 表示消息可能不存在
#[derive(Debug, Serialize)]
pub struct PlcConnectionResponse {
    pub success: bool,
    pub message: Option<String>,
}

/// 批次自动测试参数结构体
/// 
/// 业务说明：
/// 启动批次自动测试时的参数，只需要批次ID即可启动整个批次的测试流程
/// 
/// 字段说明：
/// - batch_id: 要测试的批次唯一标识符
/// 
/// Rust知识点：
/// - 简单的参数结构体，体现了单一职责原则
#[derive(Debug, Deserialize)]
pub struct StartBatchAutoTestCmdArgs {
    pub batch_id: String,
}

/// 批次自动测试响应结构体
/// 
/// 业务说明：
/// 返回给前端的批次测试启动结果，告知测试是否成功启动及相关信息
/// 
/// 字段说明：
/// - success: 测试是否成功启动
/// - message: 可选的详细消息，包含测试点位数量等信息
/// 
/// Rust知识点：
/// - 与PlcConnectionResponse结构相同，体现了接口设计的一致性
#[derive(Debug, Serialize)]
pub struct BatchAutoTestResponse {
    pub success: bool,
    pub message: Option<String>,
}

/// PLC连接状态信息结构体
/// 
/// 业务说明：
/// 提供测试PLC和被测PLC的详细连接状态信息，用于前端状态显示和监控
/// 
/// 字段说明：
/// - test_plc_connected: 测试PLC（测试设备）连接状态
/// - target_plc_connected: 被测PLC（待测设备）连接状态
/// - test_plc_name: 测试PLC的配置名称，可能为空
/// - target_plc_name: 被测PLC的配置名称，可能为空
/// - last_check_time: 最后检查时间，格式化为北京时间字符串
/// 
/// Rust知识点：
/// - #[serde(rename_all = "camelCase")] 自动将Rust的snake_case转换为JavaScript的camelCase
/// - 这确保了前后端命名规范的一致性
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlcConnectionStatus {
    pub test_plc_connected: bool,
    pub target_plc_connected: bool,
    pub test_plc_name: Option<String>,
    pub target_plc_name: Option<String>,
    pub last_check_time: String,
}

/// PLC连接命令 - 确认接线
///
/// **业务作用**:
/// - 启动PLC连接管理器，建立与测试PLC和被测PLC的连接
/// - 等待连接建立完成，确保系统可以正常通信
/// - 动态注入量程设置服务，支持实时的量程配置
/// - 为后续的测试操作做好准备
///
/// **前后端交互**:
/// - **前端调用**: 用户点击"确认接线"按钮时触发
/// - **参数**: 无需参数，使用应用状态中的配置
/// - **返回值**: PlcConnectionResponse包含连接状态和详细信息
/// - **错误处理**: 连接失败时返回详细的错误信息
///
/// **业务流程**:
/// 1. 启动PLC连接管理器
/// 2. 等待连接建立（最多3秒）
/// 3. 检查连接状态
/// 4. 动态注入量程设置服务
/// 5. 返回连接结果
///
/// **技术特点**:
/// - **异步操作**: 使用async/await处理连接建立
/// - **超时控制**: 3秒超时避免无限等待
/// - **状态轮询**: 200ms间隔检查连接状态
/// - **动态注入**: 运行时替换服务实例
/// - **错误恢复**: 连接失败时提供详细诊断信息
///
/// **Rust知识点**:
/// - `#[tauri::command]`: Tauri命令宏，暴露给前端
/// - `State<'_, T>`: Tauri状态管理，访问应用状态
/// - `AppHandle`: Tauri应用句柄，用于动态服务管理
/// - `Result<T, String>`: 错误处理，String作为错误类型便于前端处理
#[tauri::command]
pub async fn connect_plc_cmd(
    app: tauri::AppHandle, // 用于动态覆盖 manage 中的服务实例
    state: State<'_, AppState>
) -> Result<PlcConnectionResponse, String> {
    info!("🔗 开始连接PLC - 确认接线");

    let app_state = state.inner(); // 获取应用状态的内部引用
    let plc_connection_manager = app_state.plc_connection_manager.clone(); // 克隆连接管理器

    // 启动PLC连接管理器，建立持久连接
    // **业务逻辑**: 同时启动测试PLC和被测PLC的连接
    match plc_connection_manager.start_connections().await {
        Ok(()) => {
            info!("✅ PLC连接管理器启动成功");

            // 等待PLC实际连上，最多3秒，每200ms检查一次
            // **超时控制**: 避免连接建立过程中的无限等待
            // **轮询机制**: 定期检查连接状态直到成功或超时
            let mut waited_ms = 0;
            let (mut test_plc_connected, mut target_plc_connected, mut test_plc_name, mut target_plc_name) = (false, false, None, None);

            while waited_ms < 3000 { // 最大等待3秒
                let summary = plc_connection_manager.get_plc_status_summary().await;
                test_plc_connected = summary.0;    // 测试PLC连接状态
                target_plc_connected = summary.1;  // 被测PLC连接状态
                test_plc_name = summary.2.clone(); // 测试PLC名称
                target_plc_name = summary.3.clone(); // 被测PLC名称

                // 两个PLC都连接成功时退出等待
                if test_plc_connected && target_plc_connected {
                    break;
                }

                sleep(Duration::from_millis(200)).await; // 等待200ms后重试
                waited_ms += 200; // 累计等待时间
            }
            // 动态替换量程写入服务实现
            {
                // 一定存在，直接获取，就算其中使用的是一个默认的实现我们也可以获取到然后将其替换
                let range_container = app.state::<Arc<DynamicRangeSettingService>>();
                // 构建新的 ChannelRangeSettingService
                let plc_service = crate::infrastructure::plc_communication::global_plc_service();
                //获得默认的plc句柄，并且创建一个新的量程设定服务将原来的服务替换掉
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
            /*
            下方的这部分代码功能重复但这才是真正替换服务的代码
            */
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

/// 开始批次自动测试命令
/// 
/// 业务说明：
/// 启动指定批次的自动测试流程，包括验证批次存在性、加载测试实例、
/// 获取通道定义，最后启动测试协调服务执行自动测试
/// 
/// 执行流程：
/// 1. 验证批次是否存在于数据库中
/// 2. 加载该批次的所有测试实例
/// 3. 获取每个实例对应的通道定义
/// 4. 调用测试协调服务启动批次测试
/// 5. 如果批次不在活动列表中，尝试加载后再启动
/// 
/// 参数说明：
/// - args: 包含批次ID
/// - state: 应用状态，提供访问各种服务
/// 
/// 返回值：
/// - Ok(BatchAutoTestResponse): 测试启动成功，包含成功信息和测试点位数量
/// - Err(String): 启动失败，返回错误原因
/// 
/// 调用链：
/// 前端批次管理界面 -> start_batch_auto_test_cmd -> TestCoordinationService -> 执行测试流程
/// 
/// Rust知识点：
/// - 使用emoji在日志中增加可读性（🚀表示启动）
/// - 多步骤的错误处理，每一步都可能失败
#[tauri::command]
pub async fn start_batch_auto_test_cmd(
    args: StartBatchAutoTestCmdArgs,
    state: State<'_, AppState>
) -> Result<BatchAutoTestResponse, String> {
    info!("🚀 开始批次自动测试: 批次ID={}", args.batch_id);

    // 1. 验证批次存在
    // 业务说明：首先从数据库加载批次信息，确保批次存在且有效
    // Rust知识点：match表达式处理三种情况：找到、未找到、查询错误
    let batch_info = match state.persistence_service.load_batch_info(&args.batch_id).await {
        Ok(Some(info)) => {
            info!("✅ 找到批次信息: {}", info.batch_name);
            info  // 返回批次信息供后续使用
        },
        Ok(None) => {
            // 批次ID无效，返回友好的错误信息
            error!("❌ 批次不存在: {}", args.batch_id);
            return Ok(BatchAutoTestResponse {
                success: false,
                message: Some(format!("批次不存在: {}", args.batch_id)),
            });
        },
        Err(e) => {
            // 数据库查询失败，返回技术错误信息
            error!("❌ 获取批次信息失败: {}", e);
            return Ok(BatchAutoTestResponse {
                success: false,
                message: Some(format!("获取批次信息失败: {}", e)),
            });
        }
    };

    // 2. 获取批次中的所有测试实例
    // 业务说明：加载批次关联的所有测试实例，这些实例代表需要测试的通道
    // 注意：一个批次可能包含多个测试实例，每个实例对应一个通道定义
    let test_instances = match state.persistence_service.load_test_instances_by_batch(&args.batch_id).await {
        Ok(instances) => {
            info!("✅ 获取到 {} 个测试实例", instances.len());
            // 边界情况检查：批次必须至少有一个测试实例
            if instances.is_empty() {
                warn!("⚠️ 批次中没有测试实例");
                return Ok(BatchAutoTestResponse {
                    success: false,
                    message: Some("批次中没有测试实例，请先进行批次分配".to_string()),
                });
            }
            instances  // 返回实例列表
        },
        Err(e) => {
            // 数据库查询失败
            error!("❌ 获取测试实例失败: {}", e);
            return Ok(BatchAutoTestResponse {
                success: false,
                message: Some(format!("获取测试实例失败: {}", e)),
            });
        }
    };

    // 3. 获取通道定义
    // 业务说明：为每个测试实例加载对应的通道定义，通道定义包含测试参数和配置
    // Rust知识点：使用Vec动态收集数据，for循环遍历引用避免所有权转移
    let mut channel_definitions = Vec::new();
    for instance in &test_instances {
        // 从状态管理器获取通道定义（可能来自缓存）
        if let Some(definition) = state.channel_state_manager.get_channel_definition(&instance.definition_id).await {
            channel_definitions.push(definition);
        } else {
            // 通道定义缺失只记录警告，不中断流程
            warn!("⚠️ 未找到通道定义: {}", instance.definition_id);
        }
    }

    // 验证至少有一个有效的通道定义
    if channel_definitions.is_empty() {
        error!("❌ 没有找到任何通道定义");
        return Ok(BatchAutoTestResponse {
            success: false,
            message: Some("没有找到通道定义，请检查数据完整性".to_string()),
        });
    }

    // 4. 直接启动已存在的批次测试
    // 首先检查批次是否已经在活动批次中，如果不在，需要先加载到活动批次
    // 业务说明：测试协调服务管理所有活动的测试批次，批次必须先加载才能启动
    match state.test_coordination_service.start_batch_testing(&args.batch_id).await {
        Ok(()) => {
            // 测试成功启动
            info!("✅ 批次自动测试启动成功: {}", args.batch_id);
            Ok(BatchAutoTestResponse {
                success: true,
                message: Some(format!("批次 '{}' 的硬点通道自动测试已启动，共 {} 个测试点位",
                                    batch_info.batch_name, test_instances.len())),
            })
        },
        Err(e) => {
            // 如果直接启动失败，可能是因为批次不在活动列表中，尝试加载现有批次
            // 业务逻辑：支持恢复之前创建但未完成的批次测试
            warn!("⚠️ 直接启动失败，尝试加载现有批次: {}", e);

            // 使用新的加载现有批次方法
            match state.test_coordination_service.load_existing_batch(&args.batch_id).await {
                Ok(()) => {
                    info!("✅ 批次已加载到活动列表，现在启动测试: {}", args.batch_id);

                    // 再次尝试启动测试
                    // Rust知识点：嵌套的match表达式处理多层异步调用结果
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
                            // 第二次启动失败，说明存在其他问题
                            error!("❌ 启动批次测试失败: {}", e);
                            Ok(BatchAutoTestResponse {
                                success: false,
                                message: Some(format!("启动测试失败: {}", e)),
                            })
                        }
                    }
                },
                Err(e) => {
                    // 加载批次失败，可能是数据问题
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

/// 获取PLC连接状态命令
///
/// **业务作用**:
/// - 实时查询测试PLC和被测PLC的连接状态
/// - 为前端界面提供连接状态显示数据
/// - 支持连接状态的定期刷新和监控
/// - 提供连接诊断和故障排查信息
///
/// **前后端交互**:
/// - **前端调用**: 定期轮询或用户主动查询连接状态
/// - **参数**: 无需参数，直接查询当前状态
/// - **返回值**: PlcConnectionStatus包含详细的连接状态信息
/// - **实时性**: 每次调用都返回最新的连接状态
///
/// **返回数据结构**:
/// - `test_plc_connected`: 测试PLC连接状态（布尔值）
/// - `target_plc_connected`: 被测PLC连接状态（布尔值）
/// - `test_plc_name`: 测试PLC的显示名称
/// - `target_plc_name`: 被测PLC的显示名称
/// - `last_check_time`: 最后检查时间（北京时间格式）
///
/// **使用场景**:
/// - 系统状态页面的连接状态显示
/// - 测试前的连接状态确认
/// - 连接故障的实时监控
/// - 系统健康检查的一部分
///
/// **性能考虑**:
/// - 查询操作轻量级，适合频繁调用
/// - 不会影响实际的PLC通信性能
/// - 时间格式化使用北京时间便于用户理解
///
/// **Rust知识点**:
/// - 异步函数返回Future，支持非阻塞查询
/// - 元组解构赋值，简化多返回值处理
/// - 时间格式化工具的使用
#[tauri::command]
pub async fn get_plc_connection_status_cmd(
    state: State<'_, AppState>
) -> Result<PlcConnectionStatus, String> {
    let app_state = state.inner(); // 获取应用状态引用
    let plc_connection_manager = app_state.plc_connection_manager.clone(); // 克隆连接管理器

    // 从PLC连接管理器获取实时连接状态
    // **实时查询**: 每次调用都获取最新的连接状态
    // **元组解构**: 一次调用获取所有连接状态信息
    let (test_plc_connected, target_plc_connected, test_plc_name, target_plc_name) =
        plc_connection_manager.get_plc_status_summary().await;

    // 构造返回结果
    // **时间格式化**: 使用北京时间格式，便于用户理解
    // **状态封装**: 将所有状态信息封装到统一的结构体中
    Ok(PlcConnectionStatus {
        test_plc_connected,    // 测试PLC连接状态
        target_plc_connected,  // 被测PLC连接状态
        test_plc_name,         // 测试PLC名称
        target_plc_name,       // 被测PLC名称
        last_check_time: crate::utils::time_utils::format_bj(chrono::Utc::now(), "%Y-%m-%d %H:%M:%S"), // 格式化的检查时间
    })
}
