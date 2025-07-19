/// Tauri 命令模块 - 系统核心命令和状态管理
///
/// 业务说明：
/// 本模块是前后端交互的核心桥梁，定义了所有前端可调用的Tauri命令
/// 负责管理应用全局状态，协调各层服务的创建和初始化
/// 
/// 架构定位：
/// - 位于接口层最顶层，直接响应前端请求
/// - 管理应用状态(AppState)，包含所有服务实例
/// - 协调DDD各层服务的依赖注入和生命周期
/// 
/// 主要职责：
/// 1. 应用状态管理 - 创建和维护全局服务实例
/// 2. 命令定义 - 暴露给前端的所有API接口
/// 3. 会话管理 - 跟踪用户会话和批次信息
/// 4. 服务编排 - 协调多个服务完成复杂业务流程

use crate::models::{
    ChannelPointDefinition, TestBatchInfo, ChannelTestInstance, RawTestOutcome,
    TestReport, ReportTemplate, ReportGenerationRequest, AppSettings
};
use crate::application::services::{
    ITestCoordinationService, TestCoordinationService,
    TestExecutionRequest, TestExecutionResponse, TestProgressUpdate,
    IReportGenerationService, ReportGenerationService
};
use crate::domain::services::{
    IChannelStateManager, ChannelStateManager,
    ITestExecutionEngine, TestExecutionEngine,
    ITestPlcConfigService, TestPlcConfigService,
    PlcConnectionManager
};
use crate::infrastructure::{
    IPersistenceService, SqliteOrmPersistenceService,
    excel::ExcelImporter,
    persistence::{AppSettingsService, JsonAppSettingsService, AppSettingsConfig},
    SimpleEventPublisher
};
use crate::infrastructure::plc_communication::IPlcCommunicationService;
use crate::application::services::channel_allocation_service::{IChannelAllocationService, ChannelAllocationService};
use crate::utils::error::{AppError, AppResult};
use serde::{Deserialize, Serialize};
use crate::models::structs::{GlobalFunctionTestStatus, GlobalFunctionKey, default_id};
use std::sync::Arc;
use tauri::State;
use tokio::sync::Mutex;
use std::collections::HashSet;
use chrono::{DateTime, Utc};
use std::path::PathBuf;

// ============================================================================
// 应用状态管理
// ============================================================================

/// 应用状态 - 系统全局状态容器
/// 
/// 业务说明：
/// AppState是整个应用的核心状态容器，管理所有服务实例的生命周期
/// 通过Tauri的状态管理机制，在所有命令处理函数间共享
/// 
/// 设计原则：
/// - 所有服务都使用Arc<dyn Trait>实现依赖倒置和线程安全共享
/// - 使用trait对象支持运行时多态和测试替换
/// - 通过Arc确保服务可以在多个异步任务间安全共享
/// 
/// Rust知识点：
/// - Arc<T>: 原子引用计数智能指针，用于多线程共享所有权
/// - dyn Trait: 动态分发的trait对象，实现运行时多态
/// - Mutex<T>: 互斥锁，保护共享可变状态
pub struct AppState {
    // === 核心业务服务 ===
    
    /// 测试流程协调服务 - 编排整个测试流程
    pub test_coordination_service: Arc<dyn ITestCoordinationService>,
    
    /// 通道状态管理器 - 管理测试通道的状态转换
    pub channel_state_manager: Arc<dyn IChannelStateManager>,
    
    /// 测试执行引擎 - 并发执行具体的测试任务
    pub test_execution_engine: Arc<dyn ITestExecutionEngine>,
    
    /// 持久化服务 - 数据存储和查询
    pub persistence_service: Arc<dyn IPersistenceService>,
    
    /// 报告生成服务 - 生成测试报告
    pub report_generation_service: Arc<dyn IReportGenerationService>,
    
    /// 应用设置服务 - 管理应用配置
    pub app_settings_service: Arc<dyn AppSettingsService>,
    
    /// 测试PLC配置服务 - 管理PLC连接和通道配置
    pub test_plc_config_service: Arc<dyn ITestPlcConfigService>,
    
    /// 通道分配服务 - 自动分配测试通道
    pub channel_allocation_service: Arc<dyn IChannelAllocationService>,
    
    // === PLC连接管理 ===
    
    /// 测试台架PLC的连接ID
    pub test_rig_connection_id: String,
    
    /// 被测PLC的连接ID
    pub target_connection_id: String,
    
    /// PLC连接管理器 - 管理多个PLC连接
    pub plc_connection_manager: Arc<PlcConnectionManager>,
    
    /// PLC监控服务 - 实时监控PLC通道值
    pub plc_monitoring_service: Arc<dyn crate::infrastructure::IPlcMonitoringService>,

    // === 状态缓存 ===
    
    /// 全局功能测试状态缓存
    /// 业务说明：缓存系统级功能测试的状态，避免频繁查询数据库
    pub global_function_tests: Arc<Mutex<Vec<GlobalFunctionTestStatus>>>,

    // === 会话管理 ===
    
    /// 会话批次ID集合
    /// 业务说明：跟踪当前会话中创建的所有批次，用于会话清理
    pub session_batch_ids: Arc<Mutex<HashSet<String>>>,
    
    /// 会话开始时间
    pub session_start_time: DateTime<Utc>,
}

/// 系统状态信息
/// 
/// 业务说明：
/// 提供系统运行状态的快照信息，供前端显示系统健康状况
/// 
/// Rust知识点：
/// - #[derive] 自动实现指定的trait
/// - Serialize/Deserialize 支持JSON序列化
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemStatus {
    /// 活跃的测试任务数量
    pub active_test_tasks: usize,
    
    /// 系统健康状态描述
    pub system_health: String,
    
    /// 系统版本号
    pub version: String,
}

impl AppState {
    /// 创建新的应用状态
    /// 
    /// 业务说明：
    /// 这是系统初始化的核心方法，负责：
    /// 1. 创建数据库连接和执行迁移
    /// 2. 初始化所有服务实例
    /// 3. 建立服务间的依赖关系
    /// 4. 加载缓存数据
    /// 
    /// 执行流程：
    /// 1. 设置数据库（创建文件、执行迁移）
    /// 2. 创建持久化服务
    /// 3. 加载应用配置和PLC连接配置
    /// 4. 创建各层服务（领域层->应用层->基础设施层）
    /// 5. 初始化会话管理
    /// 
    /// Rust知识点：
    /// - async fn 声明异步函数
    /// - AppResult<T> 是统一的错误处理类型
    pub async fn new() -> AppResult<Self> {
        // 创建数据库配置
        // 业务说明：使用默认配置，数据库文件存储在系统数据目录
        let config = crate::infrastructure::persistence::PersistenceConfig::default();

        // 创建持久化服务 - 使用实际的SQLite文件而不是内存数据库
        let db_file_path = config.storage_root_dir.join("factory_testing_data.sqlite");

        // 确保数据库目录存在
        if let Some(parent_dir) = db_file_path.parent() {
            tokio::fs::create_dir_all(parent_dir).await.map_err(|e|
                AppError::io_error(
                    format!("创建数据库目录失败: {:?}", parent_dir),
                    e.kind().to_string()
                )
            )?;
        }

        // 如果数据库文件不存在，创建一个空文件
        if !db_file_path.exists() {
            tokio::fs::write(&db_file_path, "").await.map_err(|e|
                AppError::io_error(
                    format!("创建数据库文件失败: {:?}", db_file_path),
                    e.kind().to_string()
                )
            )?;
        }

        let sqlite_persistence_service = SqliteOrmPersistenceService::new(config.clone(), Some(&db_file_path)).await?;

        // 执行数据库迁移
        let db_conn = sqlite_persistence_service.get_database_connection();
        if let Err(e) = crate::database_migration::DatabaseMigration::migrate(db_conn).await {
            log::error!("数据库迁移失败: {}", e);
            return Err(e);
        }

        let persistence_service: Arc<dyn IPersistenceService> = Arc::new(sqlite_persistence_service);

        // 加载全部全局功能测试状态
        let mut gft_statuses = persistence_service.load_all_global_function_test_statuses().await.unwrap_or_default();
        // 清理 station_name 为空的旧记录，避免干扰
        if gft_statuses.iter().any(|s| s.station_name.is_empty()) {
            log::info!("[INIT] 清理 station_name 为空的全局功能测试记录");
            gft_statuses.retain(|s| !s.station_name.is_empty());
        }

        // 创建应用配置服务
        let app_settings_config = AppSettingsConfig::default();
        let mut app_settings_service: Arc<dyn AppSettingsService> = Arc::new(
            JsonAppSettingsService::new(app_settings_config)
        );

        // 初始化应用配置服务
        if let Some(service) = Arc::get_mut(&mut app_settings_service) {
            service.initialize().await?;
        }

        // 创建测试PLC配置服务（需要先创建，因为后面要用到）
        let test_plc_config_service: Arc<dyn ITestPlcConfigService> = Arc::new(
            TestPlcConfigService::new(persistence_service.clone())
        );

        // 创建通道状态管理器
        let channel_state_manager: Arc<dyn IChannelStateManager> = Arc::new(
            ChannelStateManager::new(persistence_service.clone())
        );

        // 🔧 修复：从数据库读取PLC连接配置，不使用硬编码IP
        let plc_connections = test_plc_config_service.get_plc_connections().await
            .map_err(|e| format!("获取PLC连接配置失败: {}", e))?;

        let test_plc_connection = plc_connections.iter()
            .find(|conn| conn.is_test_plc && conn.is_enabled)
            .ok_or_else(|| "没有找到启用的测试PLC连接配置".to_string())?;

        let target_plc_connection = plc_connections.iter()
            .find(|conn| !conn.is_test_plc && conn.is_enabled)
            .ok_or_else(|| "没有找到启用的被测PLC连接配置".to_string())?;

        log::info!("🔗 使用数据库PLC配置 - 测试PLC: {}:{}, 被测PLC: {}:{}",
            test_plc_connection.ip_address, test_plc_connection.port,
            target_plc_connection.ip_address, target_plc_connection.port);

        // 构造统一的 PlcConnectionConfig 供 PLC 服务使用
        use std::collections::HashMap;
        use crate::domain::services::plc_communication_service::{PlcConnectionConfig, PlcProtocol};

        let test_rig_conn_cfg = PlcConnectionConfig {
            id: test_plc_connection.id.clone(),
            name: test_plc_connection.name.clone(),
            protocol: PlcProtocol::ModbusTcp,
            host: test_plc_connection.ip_address.clone(),
            port: test_plc_connection.port as u16,
            timeout_ms: test_plc_connection.timeout as u64,
            read_timeout_ms: test_plc_connection.timeout as u64,
            write_timeout_ms: test_plc_connection.timeout as u64,
            retry_count: test_plc_connection.retry_count as u32,
            retry_interval_ms: 500,
            protocol_params: HashMap::new(),
            byte_order: String::from("CDAB"),
            zero_based_address: false,
        };

        let target_conn_cfg = PlcConnectionConfig {
            id: target_plc_connection.id.clone(),
            name: target_plc_connection.name.clone(),
            protocol: PlcProtocol::ModbusTcp,
            host: target_plc_connection.ip_address.clone(),
            port: target_plc_connection.port as u16,
            timeout_ms: target_plc_connection.timeout as u64,
            read_timeout_ms: target_plc_connection.timeout as u64,
            write_timeout_ms: target_plc_connection.timeout as u64,
            retry_count: target_plc_connection.retry_count as u32,
            retry_interval_ms: 500,
            protocol_params: HashMap::new(),
            byte_order: String::from("CDAB"),
            zero_based_address: false,
        };

        // 全局共享的 PLC 服务实例，测试PLC与被测PLC 共用同一个实例
        let plc_service: Arc<crate::infrastructure::ModbusTcpPlcService> = crate::infrastructure::plc_communication::global_plc_service();

        // 启动阶段不再直接建立PLC连接，改为延迟到用户确认接线后再连接
        let test_rig_connection_id = test_rig_conn_cfg.id.clone();
        let target_connection_id = target_conn_cfg.id.clone();

        let plc_service_test_rig: Arc<dyn IPlcCommunicationService> = plc_service.clone();
        let plc_service_target: Arc<dyn IPlcCommunicationService> = plc_service.clone();

        // 创建测试执行引擎
        let test_execution_engine: Arc<dyn ITestExecutionEngine> = Arc::new(
            TestExecutionEngine::new(
                88, // 最大并发测试数，和PLC通道数量一致
                plc_service_test_rig.clone(),
                plc_service_target.clone(),
                test_rig_connection_id.clone(),
                target_connection_id.clone(),
            )
        );

        // 创建事件发布器
        let event_publisher: Arc<dyn crate::domain::services::EventPublisher> = Arc::new(
            SimpleEventPublisher::new()
        );

        // 创建通道分配服务
        let channel_allocation_service: Arc<dyn crate::application::services::channel_allocation_service::IChannelAllocationService> = Arc::new(
            ChannelAllocationService::new()
        );

        // test_plc_config_service 已在上面创建



        // 创建测试协调服务
        let test_coordination_service: Arc<dyn ITestCoordinationService> = Arc::new(
            TestCoordinationService::new(
                channel_state_manager.clone(),
                test_execution_engine.clone(),
                persistence_service.clone(),
                event_publisher,
                channel_allocation_service.clone(),
                test_plc_config_service.clone(),
            )
        );

        // 创建报告生成服务
        let reports_dir = std::path::PathBuf::from("reports");
        let report_generation_service: Arc<dyn IReportGenerationService> = Arc::new(
            ReportGenerationService::new(
                persistence_service.clone(),
                reports_dir,
            )?
        );

        // 创建PLC连接管理器
        let plc_connection_manager = Arc::new(PlcConnectionManager::new(
            test_plc_config_service.clone(),
        ));

        // 设置全局PLC连接管理器，让ModbusPlcService能够访问
        //crate::infrastructure::plc_communication::set_global_plc_manager(plc_connection_manager.clone());
        crate::domain::services::plc_communication_service::set_global_plc_manager(plc_connection_manager.clone());

        // 创建PLC监控服务 - 使用真实的PLC监控服务
        let plc_monitoring_service: Arc<dyn crate::infrastructure::IPlcMonitoringService> = Arc::new(
            crate::infrastructure::plc_monitoring_service::PlcMonitoringService::new(
                plc_service_target.clone(),
                Arc::new(crate::infrastructure::event_publisher::SimpleEventPublisher::new()),
            )
        );



        Ok(Self {
            test_coordination_service,
            channel_state_manager,
            test_execution_engine,
            persistence_service,
            report_generation_service,
            app_settings_service,
            test_plc_config_service,
            channel_allocation_service,
            plc_connection_manager,
            plc_monitoring_service,

            // 新增连接ID
            test_rig_connection_id,
            target_connection_id,

            // 会话管理：跟踪当前会话中创建的批次
            session_batch_ids: Arc::new(Mutex::new(HashSet::new())),
            session_start_time: Utc::now(),
            global_function_tests: Arc::new(Mutex::new(gft_statuses)),
        })
    }
}

// ============================================================================
// 测试协调相关命令
// ============================================================================

/// 提交测试执行请求
/// 
/// 业务说明：
/// - 系统核心测试流程的入口点
/// - 接收前端的测试请求，创建测试批次和测试实例
/// - 自动分配测试通道，建立被测通道与测试通道的映射关系
/// 
/// 执行流程：
/// 1. 调用测试协调服务处理请求
/// 2. 将生成的批次ID记录到会话中
/// 3. 返回测试执行响应，包含批次信息
/// 
/// 参数：
/// - state: 应用状态
/// - request: 测试执行请求，包含批次信息和通道定义
/// 
/// 返回：
/// - Ok: 测试执行响应，包含所有生成的批次
/// - Err: 错误信息
/// 
/// 调用链：
/// 前端测试页面 -> submit_test_execution -> TestCoordinationService -> 通道分配/持久化
/// 
/// Rust知识点：
/// - State<'_, T> 是Tauri的状态管理类型
/// - map_err 用于转换错误类型
/// - {} 作用域块限制锁的生命周期
#[tauri::command]
pub async fn submit_test_execution(
    state: State<'_, AppState>,
    request: TestExecutionRequest,
) -> Result<TestExecutionResponse, String> {
    // 调用测试协调服务处理请求
    let response = state.test_coordination_service
        .submit_test_execution(request)
        .await
        .map_err(|e| e.to_string())?;

    // 将生成的所有批次ID添加到会话跟踪中
    // 业务说明：会话跟踪用于管理用户在一个会话中创建的所有批次
    {
        let mut session_batch_ids = state.session_batch_ids.lock().await;
        for batch in &response.all_batches {
            session_batch_ids.insert(batch.batch_id.clone());
        }
    }

    Ok(response)
}

/// 开始批次测试
/// 
/// 业务说明：
/// - 启动指定批次的自动测试流程
/// - 将批次状态从"准备就绪"更改为"运行中"
/// - 按照通道分配结果开始执行测试任务
/// 
/// 参数：
/// - state: 应用状态
/// - batch_id: 要启动的批次ID
/// 
/// 返回：
/// - Ok(()): 成功启动测试
/// - Err: 错误信息（如批次不存在、状态不正确等）
/// 
/// 调用链：
/// 前端开始测试按钮 -> start_batch_testing -> TestCoordinationService -> TestExecutionEngine
/// 
/// Rust知识点：
/// - 单元类型 () 表示无返回值
/// - map_err 转换错误类型为String
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
/// 
/// 业务说明：
/// - 暂停正在运行的批次测试
/// - 保持测试状态，可以随时恢复
/// - 不会丢失已完成的测试结果
/// 
/// 参数：
/// - state: 应用状态
/// - batch_id: 要暂停的批次ID
/// 
/// 返回：
/// - Ok(()): 成功暂停测试
/// - Err: 错误信息
/// 
/// 调用链：
/// 前端暂停按钮 -> pause_batch_testing -> TestCoordinationService -> TestExecutionEngine
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
/// 
/// 业务说明：
/// - 恢复之前暂停的批次测试
/// - 从上次暂停的位置继续执行
/// - 保持测试的连续性
/// 
/// 参数：
/// - state: 应用状态
/// - batch_id: 要恢复的批次ID
/// 
/// 返回：
/// - Ok(()): 成功恢复测试
/// - Err: 错误信息
/// 
/// 调用链：
/// 前端恢复按钮 -> resume_batch_testing -> TestCoordinationService -> TestExecutionEngine
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
/// 
/// 业务说明：
/// - 强制停止正在运行的批次测试
/// - 将批次状态更改为"已停止"
/// - 保存已完成的测试结果
/// - 与暂停不同，停止后不能恢复
/// 
/// 参数：
/// - state: 应用状态
/// - batch_id: 要停止的批次ID
/// 
/// 返回：
/// - Ok(()): 成功停止测试
/// - Err: 错误信息
/// 
/// 调用链：
/// 前端停止按钮 -> stop_batch_testing -> TestCoordinationService -> TestExecutionEngine
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
/// 
/// 业务说明：
/// - 实时获取批次的测试进度信息
/// - 包含每个通道的测试状态和进度
/// - 用于前端进度条和状态显示
/// 
/// 参数：
/// - state: 应用状态
/// - batch_id: 批次ID
/// 
/// 返回：
/// - Ok: 测试进度更新列表，每个元素包含通道状态和进度百分比
/// - Err: 错误信息
/// 
/// 调用链：
/// 前端轮询/WebSocket -> get_batch_progress -> TestCoordinationService -> 内存缓存
/// 
/// Rust知识点：
/// - Vec<T> 动态数组，存储多个进度更新
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
/// 
/// 业务说明：
/// - 获取指定批次的所有测试结果
/// - 包含每个通道的测试值、状态和错误信息
/// - 用于生成报告和结果展示
/// 
/// 参数：
/// - state: 应用状态
/// - batch_id: 批次ID
/// 
/// 返回：
/// - Ok: 原始测试结果列表，包含所有通道的测试数据
/// - Err: 错误信息
/// 
/// 调用链：
/// 前端结果页面 -> get_batch_results -> TestCoordinationService -> PersistenceService
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

/// 获取当前会话的所有批次信息
/// 
/// 业务说明：
/// - 获取用户在当前会话中创建的所有批次
/// - 支持会话隔离，不同用户看到不同的批次
/// - 如果会话为空，返回所有最近的批次
/// 
/// 执行流程：
/// 1. 从会话状态获取批次ID集合
/// 2. 如果为空，返回所有批次
/// 3. 否则筛选出会话相关的批次
/// 
/// 参数：
/// - state: 应用状态
/// 
/// 返回：
/// - Ok: 批次信息列表
/// - Err: 错误信息
/// 
/// 调用链：
/// 前端批次列表页面 -> get_session_batches -> PersistenceService
/// 
/// Rust知识点：
/// - HashSet::clone() 克隆集合避免长时间持有锁
/// - filter + collect 组合筛选元素
#[tauri::command]
pub async fn get_session_batches(
    state: State<'_, AppState>,
) -> Result<Vec<TestBatchInfo>, String> {
    // 获取会话中跟踪的批次ID
    let session_batch_ids = {
        let batch_ids = state.session_batch_ids.lock().await;
        batch_ids.clone()
    };

    // 如果没有跟踪的批次，返回最近的所有批次
    if session_batch_ids.is_empty() {
        state.persistence_service
            .load_all_batch_info()
            .await
            .map_err(|e| e.to_string())
    } else {
        // 获取所有批次，然后筛选出会话中的批次
        let all_batches = state.persistence_service
            .load_all_batch_info()
            .await
            .map_err(|e| e.to_string())?;

        let session_batches = all_batches
            .into_iter()
            .filter(|batch| session_batch_ids.contains(&batch.batch_id))
            .collect();

        Ok(session_batches)
    }
}

/// 清理完成的批次
/// 
/// 业务说明：
/// - 清理已完成或已停止的批次
/// - 释放相关资源（内存、临时文件等）
/// - 保留数据库中的测试结果
/// 
/// 参数：
/// - state: 应用状态
/// - batch_id: 要清理的批次ID
/// 
/// 返回：
/// - Ok(()): 成功清理
/// - Err: 错误信息
/// 
/// 调用链：
/// 前端清理按钮 -> cleanup_completed_batch -> TestCoordinationService
/// 
/// 注意：
/// - 只能清理已完成或已停止的批次
/// - 运行中的批次不能清理
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

/// 开始单个通道的硬点测试
/// 
/// 业务说明：
/// - 对单个通道执行完整的硬点测试
/// - 用于调试或重测失败的通道
/// - 不影响批次中其他通道的测试
/// 
/// 参数：
/// - state: 应用状态
/// - instance_id: 测试实例ID（不是通道定义ID）
/// 
/// 返回：
/// - Ok(()): 成功启动单通道测试
/// - Err: 错误信息
/// 
/// 调用链：
/// 前端单通道测试按钮 -> start_single_channel_test -> TestCoordinationService -> TestExecutionEngine
/// 
/// 使用场景：
/// - 批次测试中某个通道失败，需要单独重测
/// - 调试特定通道的配置问题
#[tauri::command]
pub async fn start_single_channel_test(
    state: State<'_, AppState>,
    instance_id: String,
) -> Result<(), String> {
    state.test_coordination_service
        .start_single_channel_test(&instance_id)
        .await
        .map_err(|e| e.to_string())
}

/// 创建测试数据 - 用于调试批次分配功能
/// 
/// 业务说明：
/// - 生成模拟的通道定义数据用于测试
/// - 包含各种类型的通道（AI/AO/DI/DO，有源/无源）
/// - 仅用于开发和测试环境
/// 
/// 生成的测试数据：
/// - 4个AI有源通道（温度传感器）
/// - 4个AO无源通道（输出信号）
/// - 4个DI有源通道（数字输入）
/// - 4个DO无源通道（数字输出）
/// - 4个AI无源通道（压力传感器）
/// 
/// 参数：
/// - state: 应用状态
/// 
/// 返回：
/// - Ok: 生成的通道定义列表
/// - Err: 错误信息
/// 
/// 调用链：
/// 前端测试工具 -> create_test_data -> PersistenceService
/// 
/// 注意：
/// - 生产环境应禁用此命令
/// - 每次调用会创建新的测试数据
/// 
/// Rust知识点：
/// - Vec::new() 创建空向量
/// - for循环和范围表达式 1..=4
/// - format! 宏格式化字符串
#[tauri::command]
pub async fn create_test_data(
    state: State<'_, AppState>,
) -> Result<Vec<ChannelPointDefinition>, String> {
    use crate::models::{ModuleType, PointDataType};

    log::info!("[CreateTestData] 开始创建测试数据");

    let mut definitions = Vec::new();

    // 创建AI有源通道（4个）
    for i in 1..=4 {
        let mut def = ChannelPointDefinition::new(
            format!("AI{:03}_有源", i),
            format!("Temperature_{}", i),
            format!("温度传感器{} (有源)", i),
            "Station1".to_string(),
            "Module1".to_string(),
            ModuleType::AI,
            format!("CH{:02}", i),
            PointDataType::Float,
            format!("DB1.DBD{}", i * 4),
        );
        def.power_supply_type = "有源".to_string();
        def.range_low_limit = Some(0.0);
        def.range_high_limit = Some(100.0);
        // 不再生成虚拟地址
        def.test_rig_plc_address = None;
        definitions.push(def);
    }

    // 创建AO无源通道（4个）
    for i in 1..=4 {
        let mut def = ChannelPointDefinition::new(
            format!("AO{:03}_无源", i),
            format!("Output_Signal_{}", i),
            format!("输出信号{} (无源)", i),
            "Station1".to_string(),
            "Module2".to_string(),
            ModuleType::AO,
            format!("CH{:02}", i),
            PointDataType::Float,
            format!("DB1.DBD{}", 100 + i * 4),
        );
        def.power_supply_type = "无源".to_string();
        def.range_low_limit = Some(4.0);
        def.range_high_limit = Some(20.0);
        definitions.push(def);
    }

    // 创建DI有源通道（4个）
    for i in 1..=4 {
        let mut def = ChannelPointDefinition::new(
            format!("DI{:03}_有源", i),
            format!("Digital_Input_{}", i),
            format!("数字输入{} (有源)", i),
            "Station2".to_string(),
            "Module3".to_string(),
            ModuleType::DI,
            format!("CH{:02}", i),
            PointDataType::Bool,
            format!("DB3.DBX{}.{}", i / 8, i % 8),
        );
        def.power_supply_type = "有源".to_string();
        def.test_rig_plc_address = Some(format!("DB4.DBX{}.{}", i / 8, i % 8));
        definitions.push(def);
    }

    // 创建DO无源通道（4个）
    for i in 1..=4 {
        let mut def = ChannelPointDefinition::new(
            format!("DO{:03}_无源", i),
            format!("Digital_Output_{}", i),
            format!("数字输出{} (无源)", i),
            "Station2".to_string(),
            "Module4".to_string(),
            ModuleType::DO,
            format!("CH{:02}", i),
            PointDataType::Bool,
            format!("DB5.DBX{}.{}", i / 8, i % 8),
        );
        def.power_supply_type = "无源".to_string();
        definitions.push(def);
    }

    // 创建AI无源通道（4个）
    for i in 1..=4 {
        let mut def = ChannelPointDefinition::new(
            format!("AI{:03}_无源", i + 4),
            format!("Pressure_{}", i),
            format!("压力传感器{} (无源)", i),
            "Station3".to_string(),
            "Module5".to_string(),
            ModuleType::AINone,
            format!("CH{:02}", i),
            PointDataType::Float,
            format!("DB6.DBD{}", i * 4),
        );
        def.power_supply_type = "无源".to_string();
        def.range_low_limit = Some(0.0);
        def.range_high_limit = Some(10.0);
        definitions.push(def);
    }

    log::info!("[CreateTestData] 创建了 {} 个测试通道定义", definitions.len());

    // 保存到数据库
    for def in &definitions {
        if let Err(e) = state.persistence_service.save_channel_definition(def).await {
            log::error!("[CreateTestData] 保存通道定义失败: {} - {}", def.id, e);
        } else {
            log::debug!("[CreateTestData] 保存通道定义成功: {} - {}", def.id, def.tag);
        }
    }

    log::info!("[CreateTestData] 所有测试数据创建完成");
    Ok(definitions)
}

// ============================================================================
// 数据管理相关命令
// ============================================================================

/// 导入Excel文件并解析通道定义
/// 
/// 业务说明：
/// - 解析Excel文件中的通道配置信息
/// - 支持标准的通道定义格式
/// - 返回解析后的通道定义列表，但不保存到数据库
/// 
/// 参数：
/// - file_path: Excel文件的绝对路径
/// 
/// 返回：
/// - Ok: 解析出的通道定义列表
/// - Err: 错误信息（文件不存在、格式错误等）
/// 
/// 调用链：
/// 前端文件选择器 -> import_excel_file -> ExcelImporter -> 返回解析结果
/// 
/// Rust知识点：
/// - async fn 异步函数
/// - Result<T, E> 错误处理
#[tauri::command]
pub async fn import_excel_file(
    file_path: String,
) -> Result<Vec<ChannelPointDefinition>, String> {
    ExcelImporter::parse_excel_file(&file_path)
        .await
        .map_err(|e| e.to_string())
}

/// 创建测试批次并保存通道定义
/// 
/// 业务说明：
/// - 创建新的测试批次并关联通道定义
/// - 为每个通道定义创建对应的测试实例
/// - 这是旧版API，新版本应使用 submit_test_execution
/// 
/// 执行流程：
/// 1. 保存批次信息到数据库
/// 2. 为所有通道定义设置批次ID
/// 3. 保存所有通道定义
/// 4. 为每个定义创建测试实例
/// 
/// 参数：
/// - state: 应用状态
/// - batch_info: 批次信息
/// - definitions: 通道定义列表
/// 
/// 返回：
/// - Ok: 创建的批次ID
/// - Err: 错误信息
/// 
/// 调用链：
/// 前端批次创建页面 -> create_test_batch_with_definitions -> PersistenceService
/// 
/// 注意：
/// - 这是旧版API，仅为向后兼容保留
/// - 新项目应使用 submit_test_execution
/// 
/// Rust知识点：
/// - mut 可变引用，允许修改集合元素
/// - for &mut 循环获取可变引用
#[tauri::command]
pub async fn create_test_batch_with_definitions(
    state: State<'_, AppState>,
    batch_info: TestBatchInfo,
    definitions: Vec<ChannelPointDefinition>,
) -> Result<String, String> {
    // 保存批次信息
    state.persistence_service
        .save_batch_info(&batch_info)
        .await
        .map_err(|e| e.to_string())?;

    // 🔥 保存通道定义（设置批次ID）
    let mut updated_definitions = definitions;
    for definition in &mut updated_definitions {
        definition.batch_id = Some(batch_info.batch_id.clone());
    }

    for definition in &updated_definitions {
        state.persistence_service
            .save_channel_definition(definition)
            .await
            .map_err(|e| e.to_string())?;

        // 为每个定义创建测试实例
        let instance = ChannelTestInstance::new(
            definition.id.clone(),
            batch_info.batch_id.clone(),
        );

        state.persistence_service
            .save_test_instance(&instance)
            .await
            .map_err(|e| e.to_string())?;
    }

    Ok(batch_info.batch_id)
}

/// 获取所有通道定义
/// 
/// 业务说明：
/// - 获取系统中的所有通道定义
/// - 包含所有批次的通道配置
/// - 用于通道管理和批次创建
/// 
/// 参数：
/// - state: 应用状态
/// 
/// 返回：
/// - Ok: 通道定义列表
/// - Err: 错误信息
/// 
/// 调用链：
/// 前端通道管理页面 -> get_all_channel_definitions -> PersistenceService
#[tauri::command]
pub async fn get_all_channel_definitions(
    state: State<'_, AppState>,
) -> Result<Vec<ChannelPointDefinition>, String> {
    state.persistence_service
        .load_all_channel_definitions()
        .await
        .map_err(|e| e.to_string())
}

/// 保存通道定义
/// 
/// 业务说明：
/// - 保存或更新单个通道定义
/// - 支持新增和更新操作
/// - 根据ID判断是新增还是更新
/// 
/// 参数：
/// - state: 应用状态
/// - definition: 通道定义对象
/// 
/// 返回：
/// - Ok(()): 保存成功
/// - Err: 错误信息
/// 
/// 调用链：
/// 前端通道编辑对话框 -> save_channel_definition -> PersistenceService
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
/// 
/// 业务说明：
/// - 删除指定的通道定义
/// - 如果通道已被使用，可能会导致删除失败
/// - 删除操作不可恢复
/// 
/// 参数：
/// - state: 应用状态
/// - definition_id: 通道定义ID
/// 
/// 返回：
/// - Ok(()): 删除成功
/// - Err: 错误信息
/// 
/// 调用链：
/// 前端删除确认对话框 -> delete_channel_definition -> PersistenceService
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
/// 
/// 业务说明：
/// - 获取系统中的所有测试批次
/// - 包含批次状态、创建时间、进度等信息
/// - 用于批次列表显示和管理
/// 
/// 参数：
/// - state: 应用状态
/// 
/// 返回：
/// - Ok: 批次信息列表
/// - Err: 错误信息
/// 
/// 调用链：
/// 前端批次管理页面 -> get_all_batch_info -> PersistenceService
#[tauri::command]
pub async fn get_all_batch_info(
    state: State<'_, AppState>,
) -> Result<Vec<TestBatchInfo>, String> {
    state.persistence_service
        .load_all_batch_info()
        .await
        .map_err(|e| e.to_string())
}

/// 保存批次信息
/// 
/// 业务说明：
/// - 保存或更新批次信息
/// - 只更新批次元数据，不影响测试结果
/// - 用于修改批次名称、描述等
/// 
/// 参数：
/// - state: 应用状态
/// - batch_info: 批次信息对象
/// 
/// 返回：
/// - Ok(()): 保存成功
/// - Err: 错误信息
/// 
/// 调用链：
/// 前端批次编辑对话框 -> save_batch_info -> PersistenceService
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

/// 获取批次测试实例
/// 
/// 业务说明：
/// - 获取指定批次的所有测试实例
/// - 测试实例包含通道分配和测试状态
/// - TODO: 当前为占位实现
/// 
/// 参数：
/// - _state: 应用状态（未使用）
/// - _batch_id: 批次ID（未使用）
/// 
/// 返回：
/// - Ok: 空列表（待实现）
/// - Err: 错误信息
/// 
/// 调用链：
/// 前端批次详情页面 -> get_batch_test_instances -> TODO
/// 
/// Rust知识点：
/// - _ 前缀表示参数未使用，避免编译器警告
#[tauri::command]
pub async fn get_batch_test_instances(
    _state: State<'_, AppState>,
    _batch_id: String,
) -> Result<Vec<ChannelTestInstance>, String> {
    // TODO: 实现获取批次测试实例的逻辑
    // 目前返回空列表
    Ok(vec![])
}

// ============================================================================
// 通道状态管理相关命令
// ============================================================================

/// 创建测试实例
/// 
/// 业务说明：
/// - 为通道定义创建测试实例
/// - 测试实例跟踪单次测试的状态和结果
/// - 一个通道定义可以有多个测试实例（不同批次）
/// 
/// 参数：
/// - state: 应用状态
/// - definition_id: 通道定义ID
/// - batch_id: 批次ID
/// 
/// 返回：
/// - Ok: 创建的测试实例
/// - Err: 错误信息
/// 
/// 调用链：
/// 前端批次创建 -> create_test_instance -> ChannelStateManager
#[tauri::command]
pub async fn create_test_instance(
    state: State<'_, AppState>,
    definition_id: String,
    batch_id: String,
) -> Result<ChannelTestInstance, String> {
    state.channel_state_manager
        .create_test_instance(&definition_id, &batch_id)
        .await
        .map_err(|e| e.to_string())
}

/// 获取实例状态
/// 
/// 业务说明：
/// - 获取测试实例的当前状态
/// - 包含测试进度、结果、错误信息等
/// - 用于前端实时显示测试状态
/// 
/// 参数：
/// - state: 应用状态
/// - instance_id: 测试实例ID
/// 
/// 返回：
/// - Ok: 测试实例状态
/// - Err: 错误信息
/// 
/// 调用链：
/// 前端状态轮询 -> get_instance_state -> ChannelStateManager
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
/// 
/// 业务说明：
/// - 更新测试实例的测试结果
/// - 包含测试值、状态、时间戳等
/// - 测试引擎在完成测试后调用
/// 
/// 参数：
/// - state: 应用状态
/// - outcome: 原始测试结果
/// 
/// 返回：
/// - Ok(()): 更新成功
/// - Err: 错误信息
/// 
/// 调用链：
/// TestExecutionEngine -> update_test_result -> ChannelStateManager -> 数据库
#[tauri::command]
pub async fn update_test_result(
    state: State<'_, AppState>,
    outcome: RawTestOutcome,
) -> Result<(), String> {
    state.channel_state_manager
        .update_test_result(outcome)
        .await
        .map_err(|e| e.to_string())
}

// ============================================================================
// 系统信息相关命令
// ============================================================================

/// 获取系统状态
/// 
/// 业务说明：
/// - 获取系统当前的运行状态
/// - 包含活动任务数、系统健康度、版本信息等
/// - 用于系统监控和诊断
/// 
/// 参数：
/// - _state: 应用状态（未使用）
/// 
/// 返回：
/// - Ok: 系统状态信息
/// - Err: 错误信息
/// 
/// 调用链：
/// 前端系统监控页面 -> get_system_status -> 返回状态
/// 
/// TODO:
/// - 从测试执行引擎获取实际的活动任务数
/// - 添加更多系统健康度指标
/// 
/// Rust知识点：
/// - env! 宏在编译时获取环境变量
/// - to_string() 转换为String类型
#[tauri::command]
pub async fn get_system_status(
    _state: State<'_, AppState>,
) -> Result<SystemStatus, String> {
    Ok(SystemStatus {
        active_test_tasks: 0, // TODO: 从测试执行引擎获取
        system_health: "healthy".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

/// 初始化应用状态
/// 
/// 业务说明：
/// - 应用启动时初始化全局状态
/// - 创建所有必需的服务实例
/// - 加载配置和历史数据
/// 
/// 返回：
/// - Ok: 初始化完成的应用状态
/// - Err: 初始化失败的错误信息
/// 
/// 调用链：
/// main.rs -> lib.rs -> init_app_state -> AppState::new()
/// 
/// 注意：
/// - 这是应用程序的入口点之一
/// - 失败将导致应用无法启动
pub async fn init_app_state() -> AppResult<AppState> {
    AppState::new().await
}

// ============================================================================
// 报告生成相关命令
// ============================================================================

/// 生成PDF报告
/// 
/// 业务说明：
/// - 生成指定批次的PDF格式测试报告
/// - 支持自定义模板和样式
/// - 包含测试结果、统计信息、图表等
/// 
/// 参数：
/// - state: 应用状态
/// - request: 报告生成请求，包含批次ID、模板等
/// 
/// 返回：
/// - Ok: 生成的报告信息，包含文件路径
/// - Err: 错误信息
/// 
/// 调用链：
/// 前端报告页面 -> generate_pdf_report -> ReportGenerationService -> PDF生成器
/// 
/// TODO:
/// - 从认证系统获取真实用户ID
/// - 当前使用"system"作为默认用户
#[tauri::command]
pub async fn generate_pdf_report(
    state: State<'_, AppState>,
    request: ReportGenerationRequest,
) -> Result<TestReport, String> {
    state.report_generation_service
        .generate_pdf_report(request, "system") // TODO: 从认证系统获取用户ID
        .await
        .map_err(|e| e.to_string())
}

/// 生成Excel报告
/// 
/// 业务说明：
/// - 生成指定批次的Excel格式测试报告
/// - 支持多工作表、公式、图表等
/// - 方便数据分析和二次处理
/// 
/// 参数：
/// - state: 应用状态
/// - request: 报告生成请求
/// 
/// 返回：
/// - Ok: 生成的报告信息
/// - Err: 错误信息
/// 
/// 调用链：
/// 前端报告页面 -> generate_excel_report -> ReportGenerationService -> Excel生成器
#[tauri::command]
pub async fn generate_excel_report(
    state: State<'_, AppState>,
    request: ReportGenerationRequest,
) -> Result<TestReport, String> {
    state.report_generation_service
        .generate_excel_report(request, "system") // TODO: 从认证系统获取用户ID
        .await
        .map_err(|e| e.to_string())
}

/// 获取报告列表
/// 
/// 业务说明：
/// - 获取已生成的报告列表
/// - 支持按批次筛选
/// - 返回报告元数据，不包含实际文件内容
/// 
/// 参数：
/// - state: 应用状态
/// - batch_id: 可选的批次ID筛选
/// 
/// 返回：
/// - Ok: 报告列表
/// - Err: 错误信息
/// 
/// 调用链：
/// 前端报告管理页面 -> get_reports -> ReportGenerationService
/// 
/// Rust知识点：
/// - Option::as_deref() 将Option<String>转换为Option<&str>
#[tauri::command]
pub async fn get_reports(
    state: State<'_, AppState>,
    batch_id: Option<String>,
) -> Result<Vec<TestReport>, String> {
    state.report_generation_service
        .get_reports(batch_id.as_deref())
        .await
        .map_err(|e| e.to_string())
}

/// 获取报告模板列表
/// 
/// 业务说明：
/// - 获取所有可用的报告模板
/// - 模板定义报告的样式和内容结构
/// - 用户可以选择不同模板生成报告
/// 
/// 参数：
/// - state: 应用状态
/// 
/// 返回：
/// - Ok: 报告模板列表
/// - Err: 错误信息
/// 
/// 调用链：
/// 前端报告生成对话框 -> get_report_templates -> ReportGenerationService
#[tauri::command]
pub async fn get_report_templates(
    state: State<'_, AppState>,
) -> Result<Vec<ReportTemplate>, String> {
    state.report_generation_service
        .get_templates()
        .await
        .map_err(|e| e.to_string())
}

/// 创建报告模板
/// 
/// 业务说明：
/// - 创建新的报告模板
/// - 允许用户自定义报告格式
/// - 模板保存后可重复使用
/// 
/// 参数：
/// - state: 应用状态
/// - template: 报告模板对象
/// 
/// 返回：
/// - Ok(()): 创建成功
/// - Err: 错误信息
/// 
/// 调用链：
/// 前端模板编辑器 -> create_report_template -> ReportGenerationService
#[tauri::command]
pub async fn create_report_template(
    state: State<'_, AppState>,
    template: ReportTemplate,
) -> Result<(), String> {
    state.report_generation_service
        .create_template(template)
        .await
        .map_err(|e| e.to_string())
}

/// 更新报告模板
/// 
/// 业务说明：
/// - 更新现有的报告模板
/// - 修改模板的样式、内容或配置
/// - 不影响已生成的报告
/// 
/// 参数：
/// - state: 应用状态
/// - template: 更新后的模板对象
/// 
/// 返回：
/// - Ok(()): 更新成功
/// - Err: 错误信息
/// 
/// 调用链：
/// 前端模板编辑器 -> update_report_template -> ReportGenerationService
#[tauri::command]
pub async fn update_report_template(
    state: State<'_, AppState>,
    template: ReportTemplate,
) -> Result<(), String> {
    state.report_generation_service
        .update_template(template)
        .await
        .map_err(|e| e.to_string())
}

/// 删除报告模板
/// 
/// 业务说明：
/// - 删除指定的报告模板
/// - 如果模板正在使用，可能无法删除
/// - 删除操作不可恢复
/// 
/// 参数：
/// - state: 应用状态
/// - template_id: 模板ID
/// 
/// 返回：
/// - Ok(()): 删除成功
/// - Err: 错误信息
/// 
/// 调用链：
/// 前端模板管理页面 -> delete_report_template -> ReportGenerationService
#[tauri::command]
pub async fn delete_report_template(
    state: State<'_, AppState>,
    template_id: String,
) -> Result<(), String> {
    state.report_generation_service
        .delete_template(&template_id)
        .await
        .map_err(|e| e.to_string())
}

/// 删除报告
/// 
/// 业务说明：
/// - 删除已生成的报告文件
/// - 同时删除文件和数据库记录
/// - 删除操作不可恢复
/// 
/// 参数：
/// - state: 应用状态
/// - report_id: 报告ID
/// 
/// 返回：
/// - Ok(()): 删除成功
/// - Err: 错误信息
/// 
/// 调用链：
/// 前端报告管理页面 -> delete_report -> ReportGenerationService
#[tauri::command]
pub async fn delete_report(
    state: State<'_, AppState>,
    report_id: String,
) -> Result<(), String> {
    state.report_generation_service
        .delete_report(&report_id)
        .await
        .map_err(|e| e.to_string())
}

// ============================================================================
// 应用配置相关命令
// ============================================================================

/// 加载应用配置
/// 
/// 业务说明：
/// - 加载应用程序的全局配置
/// - 如果配置文件不存在，返回默认配置
/// - 同时保存默认配置到文件
/// 
/// 参数：
/// - state: 应用状态
/// 
/// 返回：
/// - Ok: 应用配置对象
/// - Err: 错误信息（实际上总是返回成功）
/// 
/// 调用链：
/// 前端初始化 -> load_app_settings_cmd -> AppSettingsService
/// 
/// 容错机制：
/// - 配置加载失败时返回默认配置
/// - 确保应用总是能正常启动
/// 
/// Rust知识点：
/// - match 模式匹配处理多种情况
/// - Ok(Some(T)) 嵌套的Result和Option
#[tauri::command]
pub async fn load_app_settings_cmd(
    state: State<'_, AppState>,
) -> Result<AppSettings, String> {
    match state.app_settings_service.load_settings().await {
        Ok(Some(settings)) => Ok(settings),
        Ok(None) => {
            // 如果没有配置文件，返回默认配置
            let default_settings = AppSettings::default();
            // 保存默认配置到文件
            if let Err(e) = state.app_settings_service.save_settings(&default_settings).await {
                log::warn!("保存默认应用配置失败: {}", e);
            }
            Ok(default_settings)
        },
        Err(e) => {
            log::error!("加载应用配置失败: {}", e);
            // 发生错误时也返回默认配置
            Ok(AppSettings::default())
        }
    }
}

/// 保存应用配置
/// 
/// 业务说明：
/// - 保存应用程序的全局配置
/// - 配置保存到JSON文件
/// - 实时生效，无需重启
/// 
/// 参数：
/// - state: 应用状态
/// - settings: 应用配置对象
/// 
/// 返回：
/// - Ok(()): 保存成功
/// - Err: 错误信息
/// 
/// 调用链：
/// 前端设置页面 -> save_app_settings_cmd -> AppSettingsService
/// 
/// Rust知识点：
/// - map_err 转换错误类型并添加日志
#[tauri::command]
pub async fn save_app_settings_cmd(
    state: State<'_, AppState>,
    settings: AppSettings,
) -> Result<(), String> {
    state.app_settings_service
        .save_settings(&settings)
        .await
        .map_err(|e| {
            log::error!("保存应用配置失败: {}", e);
            format!("保存应用配置失败: {}", e)
        })
}

/// 导出测试结果表
/// 
/// 业务说明：
/// 这个结构体定义了导出测试结果的参数
/// 
/// Rust知识点：
/// - #[derive(Deserialize)] 自动实现反序列化
#[derive(Deserialize)]
pub struct ExportTestResultsArgs {
    pub target_path: Option<String>,  // 目标文件路径（可选）
}

/// 导出测试结果到Excel
/// 
/// 业务说明：
/// - 导出所有测试结果到Excel文件
/// - 包含通道信息、测试值、状态、错误备注等
/// - 支持指定导出路径或默认路径
/// 
/// 参数：
/// - state: 应用状态
/// - target_path: 目标路径（向后兼容）
/// - args: 导出参数（新版本）
/// 
/// 返回：
/// - Ok: 导出文件的完整路径
/// - Err: 错误信息
/// 
/// 调用链：
/// 前端导出按钮 -> export_test_results_cmd -> ExcelExportService
/// 
/// 兼容性：
/// - 同时支持旧版本和新版本的参数传递方式
/// 
/// Rust知识点：
/// - Option::and_then 链式处理嵌套的Option
/// - PathBuf::from 从字符串创建路径
#[tauri::command]
pub async fn export_test_results_cmd(
    state: State<'_, AppState>,
    target_path: Option<String>,
    args: Option<ExportTestResultsArgs>,
) -> Result<String, String> {
    let real_path_opt = args.and_then(|a| a.target_path).or(target_path.clone());
    log::info!("📤 [CMD] 收到导出测试结果请求, target_path={:?}", real_path_opt);

    let service = crate::infrastructure::excel_export_service::ExcelExportService::new(
        state.persistence_service.clone(),
        state.channel_state_manager.clone(),
    );

    let path_buf = real_path_opt.map(PathBuf::from);
    match service.export_test_results(path_buf).await {
        Ok(result_path) => {
            log::info!("✅ [CMD] 测试结果导出成功: {}", result_path);
            Ok(result_path)
        },
        Err(e) => {
            log::error!("❌ [CMD] 测试结果导出失败: {}", e);
            Err(e.to_string())
        }
    }
}

/// 导出通道分配表
/// 
/// 业务说明：
/// 这个结构体定义了导出通道分配表的参数
#[derive(Deserialize)]
pub struct ExportChannelAllocationArgs {
    pub target_path: Option<String>,     // 目标文件路径（可选）
    pub batch_ids: Option<Vec<String>>,  // 指定导出哪些批次（可选）
}

/// 导出通道分配表到Excel
/// 
/// 业务说明：
/// - 导出通道分配情况到Excel文件
/// - 显示被测通道与测试通道的对应关系
/// - 支持按批次筛选导出
/// 
/// 参数：
/// - state: 应用状态
/// - target_path: 目标路径（向后兼容）
/// - args: 导出参数（新版本）
/// 
/// 返回：
/// - Ok: 导出文件的完整路径
/// - Err: 错误信息
/// 
/// 调用链：
/// 前端导出按钮 -> export_channel_allocation_cmd -> ExcelExportService
/// 
/// 筛选逻辑：
/// - 如果指定了batch_ids，只导出这些批次
/// - 否则导出当前会话的所有批次
/// - 如果会话为空，导出所有批次
/// 
/// Rust知识点：
/// - into_iter().collect() 将HashSet转换为Vec
/// - Arc<Mutex<T>> 访问共享可变状态
#[tauri::command]
pub async fn export_channel_allocation_cmd(
    state: State<'_, AppState>,
    target_path: Option<String>, // 向后兼容旧调用
    args: Option<ExportChannelAllocationArgs>,
) -> Result<String, String> {
    // 兼容逻辑: 如果前端新版本使用 args 结构体，则覆盖 target_path
    let (real_path_opt, batch_ids_opt) = if let Some(a) = args {
        (a.target_path.or(target_path.clone()), a.batch_ids)
    } else {
        (target_path.clone(), None)
    };

    log::info!("📤 [CMD] 收到导出通道分配表请求, target_path={:?}, batch_ids={:?}", real_path_opt, batch_ids_opt);

    // 计算需要导出的批次ID集合
    let allowed_batch_ids: Option<Vec<String>> = if let Some(list) = batch_ids_opt {
        Some(list)
    } else {
        // 使用当前会话的批次
        let set = state.session_batch_ids.lock().await.clone();
        if set.is_empty() { None } else { Some(set.into_iter().collect()) }
    };

    let service = crate::infrastructure::excel_export_service::ExcelExportService::new(
        state.persistence_service.clone(),
        state.channel_state_manager.clone(),
    );

    let path_buf = real_path_opt.map(PathBuf::from);
    match service.export_channel_allocation_with_filter(path_buf, allowed_batch_ids).await {
        Ok(result_path) => {
            log::info!("✅ [CMD] 通道分配表导出成功: {}", result_path);
            Ok(result_path)
        },
        Err(e) => {
            log::error!("❌ [CMD] 通道分配表导出失败: {}", e);
            Err(e.to_string())
        }
    }
}

// ============================================================================
// 错误备注管理命令
// ============================================================================

/// 保存通道测试实例的错误备注
/// 
/// 业务说明：
/// - 保存测试失败时的错误分析和备注
/// - 支持三种错误类型：集成错误、PLC编程错误、HMI配置错误
/// - 用于后续分析和问题追踪
/// 
/// 参数：
/// - state: 应用状态
/// - instance_id: 测试实例ID
/// - integration_error_notes: 集成错误备注（可选）
/// - plc_programming_error_notes: PLC编程错误备注（可选）
/// - hmi_configuration_error_notes: HMI配置错误备注（可选）
/// 
/// 返回：
/// - Ok(()): 保存成功
/// - Err: 错误信息
/// 
/// 调用链：
/// 前端错误备注对话框 -> save_error_notes_cmd -> PersistenceService
/// 
/// 使用场景：
/// - 测试失败后，工程师分析原因并记录
/// - 生成报告时包含这些备注信息
/// 
/// Rust知识点：
/// - Option::as_deref() 将Option<String>转换为Option<&str>
/// - 多个可选参数的处理
#[tauri::command]
pub async fn save_error_notes_cmd(
    state: State<'_, AppState>,
    instance_id: String,
    integration_error_notes: Option<String>,
    plc_programming_error_notes: Option<String>,
    hmi_configuration_error_notes: Option<String>,
) -> Result<(), String> {
    log::info!("💾 [CMD] 保存错误备注: instance_id={}, integration={:?}, plc={:?}, hmi={:?}", 
        instance_id, integration_error_notes, plc_programming_error_notes, hmi_configuration_error_notes);

    // 调用持久化服务更新错误备注
    match state.persistence_service.update_instance_error_notes(
        &instance_id,
        integration_error_notes.as_deref(),
        plc_programming_error_notes.as_deref(),
        hmi_configuration_error_notes.as_deref(),
    ).await {
        Ok(_) => {
            log::info!("✅ [CMD] 错误备注保存成功: {}", instance_id);
            Ok(())
        },
        Err(e) => {
            log::error!("❌ [CMD] 错误备注保存失败: {}: {}", instance_id, e);
            Err(e.to_string())
        }
    }
}
