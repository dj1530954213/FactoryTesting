//! # 数据管理Tauri命令 (Data Management Tauri Commands)
//!
//! ## 业务说明
//! 本模块处理所有与测试数据管理相关的前端请求，是数据流的核心入口点
//! 负责将前端的数据操作需求转换为后端的业务处理流程
//!
//! ## 核心功能
//! ### 1. Excel文件处理
//! - **文件解析**: 解析Excel中的测试点位定义数据
//! - **数据验证**: 验证导入数据的格式和完整性
//! - **错误处理**: 提供详细的导入错误信息
//!
//! ### 2. 批次生命周期管理
//! - **批次创建**: 根据通道定义创建测试批次
//! - **批次查询**: 获取批次列表和详细信息
//! - **批次删除**: 清理不需要的测试批次
//! - **状态跟踪**: 实时跟踪批次执行状态
//!
//! ### 3. 通道分配算法
//! - **智能分配**: 将测试点位合理分配到物理测试通道
//! - **策略选择**: 支持多种分配策略(按类型、按站点等)
//! - **资源优化**: 最大化测试效率，最小化资源冲突
//!
//! ### 4. 会话数据管理
//! - **临时存储**: 管理测试过程中的临时数据
//! - **状态恢复**: 支持测试中断后的状态恢复
//! - **数据清理**: 及时清理无用的临时数据
//!
//! ## 调用链路
//! ```
//! 前端UI → Tauri IPC → 命令处理器 → 数据验证 → 
//! 应用层服务 → 领域层业务逻辑 → 基础设施层 → 数据库/文件系统
//! ```
//!
//! ## Rust知识点
//! - **异步命令**: 使用#[tauri::command]宏定义异步命令
//! - **序列化**: 使用serde进行JSON数据的序列化/反序列化
//! - **错误处理**: 统一的Result<T, String>错误处理模式

use tauri::State;
use serde::{Deserialize, Serialize};
use crate::models::structs::{ChannelPointDefinition, TestBatchInfo};
use crate::application::services::data_import_service::{DataImportService, ImportResult};
use crate::application::services::batch_allocation_service::{BatchAllocationService, AllocationStrategy, AllocationResult as BatchAllocationResult};
use crate::infrastructure::excel::ExcelImporter;
use crate::application::services::channel_allocation_service::{ChannelAllocationService, IChannelAllocationService};
use crate::tauri_commands::AppState;
use log::{info, error, warn, debug};
use sea_orm::ActiveModelTrait;
use std::collections::HashMap;
use std::sync::Arc;

/// 通道分配结果（用于命令层）
/// 
/// 业务说明：
/// 封装通道分配的完整结果，包括批次信息、分配的测试实例和分配统计
/// 这是返回给前端的数据结构
/// 
/// Rust知识点：
/// - #[derive(Debug, Clone, Serialize)] 自动实现调试、克隆和序列化trait
/// - Serialize trait 允许结构体被序列化为JSON返回给前端
#[derive(Debug, Clone, Serialize)]
pub struct AllocationResult {
    pub batches: Vec<TestBatchInfo>,                    // 创建的批次列表
    pub allocated_instances: Vec<crate::models::structs::ChannelTestInstance>, // 分配的测试实例
    pub allocation_summary: crate::application::services::batch_allocation_service::AllocationSummary, // 分配统计摘要
    /// 🔧 修复：添加通道定义字段，用于保存到数据库
    pub channel_definitions: Option<Vec<ChannelPointDefinition>>, // 原始通道定义，可选字段
}

/// Excel文件解析请求
/// 
/// 业务说明：前端请求解析Excel文件时的参数
/// Rust知识点：Deserialize trait 允许从JSON反序列化为Rust结构体
#[derive(Debug, Deserialize)]
pub struct ParseExcelRequest {
    pub file_path: String,  // Excel文件的完整路径
}

/// Excel文件解析响应
/// 
/// 业务说明：返回Excel解析结果，包括解析状态和通道定义数据
#[derive(Debug, Serialize)]
pub struct ParseExcelResponse {
    pub success: bool,                                    // 解析是否成功
    pub message: String,                                  // 结果消息
    pub data: Option<Vec<ChannelPointDefinition>>,      // 解析出的通道定义列表
    pub total_count: usize,                             // 总通道数
}

/// Excel解析响应（用于allocate_channels_cmd）
/// 
/// 业务说明：增强版的Excel解析响应，包含了批次建议和分配预览信息
/// 用于一步完成Excel解析和通道分配预览
#[derive(Debug, Serialize)]
pub struct ExcelParseResponse {
    pub success: bool,                                    // 解析是否成功
    pub message: Option<String>,                          // 错误消息（如果有）
    pub definitions: Vec<ChannelPointDefinition>,         // 通道定义列表
    pub suggested_batch_info: Option<TestBatchInfo>,      // 建议的批次信息
    pub allocation_summary: Option<crate::application::services::batch_allocation_service::AllocationSummary>, // 分配预览统计
}

/// 创建批次请求
/// 
/// 业务说明：创建测试批次时的完整请求参数
/// 包含了文件信息、预览数据和批次元数据
#[derive(Debug, Deserialize)]
pub struct CreateBatchRequest {
    pub file_name: String,                                // 原始文件名
    pub file_path: String,                                // 文件路径
    pub preview_data: Vec<ChannelPointDefinition>,        // 预览的通道数据
    pub batch_info: BatchInfo,                            // 批次信息
}

/// 批次信息
/// 
/// 业务说明：测试批次的元数据，记录产品型号、序列号等信息
/// Option<T> 表示可选字段，前端可以不传
#[derive(Debug, Deserialize)]
pub struct BatchInfo {
    pub product_model: String,                            // 产品型号（必填）
    pub serial_number: String,                            // 序列号（必填）
    pub customer_name: Option<String>,                    // 客户名称（可选）
    pub operator_name: Option<String>,                    // 操作员名称（可选）
}

/// 创建批次响应
/// 
/// 业务说明：返回批次创建结果
#[derive(Debug, Serialize)]
pub struct CreateBatchResponse {
    pub success: bool,                                    // 创建是否成功
    pub message: String,                                  // 结果消息
    pub batch_id: Option<String>,                         // 创建成功后的批次ID
}

/// 解析Excel文件
///
/// 业务说明：
/// 解析Excel文件中的测试点位定义数据
/// 这是数据导入流程的第一步，只解析不保存
/// 
/// 调用链：
/// 前端调用 -> parse_excel_file -> ExcelImporter::parse_excel_file -> 返回解析结果
/// 
/// # 参数
/// * `file_path` - Excel文件路径
/// * `state` - 应用状态（这里未使用，但Tauri要求保留）
///
/// # 返回
/// * `Result<ParseExcelResponse, String>` - 解析结果或错误信息
/// 
/// Rust知识点：
/// - #[tauri::command] 宏将函数暴露为Tauri命令
/// - State<'_, T> 是Tauri的状态管理机制，'_ 表示生命周期由编译器推断
/// - Result<T, E> 是Rust的错误处理机制
#[tauri::command]
pub async fn parse_excel_file(
    file_path: String,
    state: State<'_, AppState>  // Tauri状态，包含全局服务实例
) -> Result<ParseExcelResponse, String> {
    info!("收到Excel文件解析请求: {}", file_path);

    // 调用Excel导入器解析文件
    // Rust知识点：match 表达式用于模式匹配Result
    match ExcelImporter::parse_excel_file(&file_path).await {
        Ok(definitions) => {
            let total_count = definitions.len();
            info!("Excel文件解析成功，共解析{}个通道定义", total_count);

            Ok(ParseExcelResponse {
                success: true,
                message: format!("成功解析{}个通道定义", total_count),
                data: Some(definitions),
                total_count,
            })
        }
        Err(e) => {
            // 错误处理：返回失败响应
            error!("Excel文件解析失败: {}", e);
            Ok(ParseExcelResponse {
                success: false,
                message: format!("解析失败: {}", e),
                data: None,
                total_count: 0,
            })
        }
    }
}

/// 创建测试批次
///
/// 业务说明：
/// 创建新的测试批次，包括保存批次信息和关联的通道定义
/// 这是数据导入流程的第二步，将解析的数据持久化到数据库
/// 
/// 调用链：
/// 前端 -> create_test_batch -> PersistenceService -> 数据库
/// 
/// # 参数
/// * `batch_data` - 批次创建请求数据
/// * `state` - 应用状态
///
/// # 返回
/// * `Result<CreateBatchResponse, String>` - 创建结果
/// 
/// Rust知识点：
/// - Clone trait 用于复制数据，避免所有权转移
/// - as u32 显式类型转换
#[tauri::command]
pub async fn create_test_batch(
    batch_data: CreateBatchRequest,
    state: State<'_, AppState>
) -> Result<CreateBatchResponse, String> {
    info!("收到创建测试批次请求: 产品型号={}, 序列号={}",
          batch_data.batch_info.product_model,
          batch_data.batch_info.serial_number);

    // 创建测试批次信息
    // 业务说明：TestBatchInfo::new 会自动生成唯一的批次ID
    let mut test_batch = TestBatchInfo::new(
        Some(batch_data.batch_info.product_model.clone()),
        Some(batch_data.batch_info.serial_number.clone()),
    );

    // 设置可选信息
    test_batch.customer_name = batch_data.batch_info.customer_name.clone();
    test_batch.operator_name = batch_data.batch_info.operator_name.clone();
    test_batch.total_points = batch_data.preview_data.len() as u32;
    // 注释掉不存在的字段
    // test_batch.source_file_name = Some(batch_data.file_name.clone());
    // test_batch.source_file_path = Some(batch_data.file_path.clone());

    // 获取持久化服务
    // Rust知识点：&state 获取State的引用，避免所有权转移
    let persistence_service = &state.persistence_service;

    // 保存批次信息
    match persistence_service.save_batch_info(&test_batch).await {
        Ok(_) => {
            info!("测试批次创建成功: {}", test_batch.batch_id);

            // 将批次ID添加到当前会话跟踪中
            // 业务说明：会话跟踪用于区分不同用户的批次，避免数据混淆
            // Rust知识点：作用域{}确保锁在使用后立即释放
            {
                let mut session_batch_ids = state.session_batch_ids.lock().await;
                session_batch_ids.insert(test_batch.batch_id.clone());
                info!("批次 {} 已添加到当前会话跟踪", test_batch.batch_id);
            }

            // 🔥 保存通道定义（设置批次ID）
            // 业务说明：通道定义必须关联到批次，建立一对多关系
            let mut saved_count = 0;
            let mut updated_definitions = batch_data.preview_data.clone();

            // 为每个通道定义设置批次ID
            // Rust知识点：&mut 可变引用，允许修改数据
            for definition in &mut updated_definitions {
                definition.batch_id = Some(test_batch.batch_id.clone());
                info!("🔗 为通道定义 {} 设置批次ID: {}", definition.tag, test_batch.batch_id);
            }

            // 批量保存通道定义
            for definition in &updated_definitions {
                match persistence_service.save_channel_definition(definition).await {
                    Ok(_) => saved_count += 1,
                    Err(e) => {
                        // 单个保存失败不影响整体流程，记录错误继续
                        error!("保存通道定义失败: {} - {}", definition.tag, e);
                    }
                }
            }

            info!("成功保存{}个通道定义", saved_count);

            Ok(CreateBatchResponse {
                success: true,
                message: format!("成功创建测试批次，保存{}个通道定义", saved_count),
                batch_id: Some(test_batch.batch_id.clone()),
            })
        }
        Err(e) => {
            // 批次创建失败，返回错误信息
            error!("创建测试批次失败: {}", e);
            Ok(CreateBatchResponse {
                success: false,
                message: format!("创建失败: {}", e),
                batch_id: None,
            })
        }
    }
}

/// 获取批次列表 - 用于测试区域，只返回当前会话的批次
/// 
/// 业务说明：
/// 为了多用户隔离，只返回当前会话创建的批次
/// 避免不同用户看到其他人的测试数据
/// 
/// 调用链：
/// 前端测试区域 -> get_batch_list -> PersistenceService -> 过滤当前会话批次
#[tauri::command]
pub async fn get_batch_list(
    state: State<'_, AppState>
) -> Result<Vec<TestBatchInfo>, String> {
    let persistence_service = &state.persistence_service;

    // 获取当前会话中的批次ID列表
    // Rust知识点：使用作用域{}来控制锁的生命周期，避免死锁
    let session_batch_ids = {
        let session_batch_ids_guard = state.session_batch_ids.lock().await;
        session_batch_ids_guard.clone()  // 克隆数据后立即释放锁
    };

    // 如果当前会话中没有批次，直接返回空列表
    // 业务说明：优化性能，避免不必要的数据库查询
    if session_batch_ids.is_empty() {
        return Ok(vec![]);
    }

    match persistence_service.load_all_batch_info().await {
        Ok(batches) => {
            // 只返回当前会话中创建的批次
            // Rust知识点：into_iter() 消费原集合，filter() 过滤，collect() 收集结果
            let current_session_batches: Vec<TestBatchInfo> = batches.into_iter()
                .filter(|batch| session_batch_ids.contains(&batch.batch_id))
                .collect();

            Ok(current_session_batches)
        }
        Err(e) => {
            error!("获取批次列表失败: {}", e);
            Err(format!("获取失败: {}", e))
        }
    }
}

/// 仪表盘批次信息 - 包含是否为当前会话的标识
/// 
/// 业务说明：
/// 仪表盘需要显示所有历史批次，但要区分当前会话和历史批次
/// 
/// Rust知识点：
/// - #[serde(flatten)] 将嵌套结构体的字段平铺到当前层级
#[derive(Debug, Serialize)]
pub struct DashboardBatchInfo {
    #[serde(flatten)]
    pub batch_info: TestBatchInfo,             // 批次基本信息
    pub is_current_session: bool,              // 是否为当前会话的批次
    pub has_station_name: bool,                // 是否有站场名称（用于调试）
}

/// 获取仪表盘批次列表 - 从数据库获取所有批次，并标识当前会话批次
/// 
/// 业务说明：
/// 1. 加载所有历史批次数据
/// 2. 尝试修复缺失的站场信息（历史数据修复）
/// 3. 标识哪些是当前会话的批次
/// 
/// 调用链：
/// 前端仪表盘 -> get_dashboard_batch_list -> PersistenceService -> 返回所有批次
#[tauri::command]
pub async fn get_dashboard_batch_list(
    state: State<'_, AppState>
) -> Result<Vec<DashboardBatchInfo>, String> {
    let persistence_service = &state.persistence_service;

    // 获取当前会话中的批次ID列表
    let session_batch_ids = {
        let session_batch_ids_guard = state.session_batch_ids.lock().await;
        session_batch_ids_guard.clone()
    };

    // 从数据库加载所有批次信息
    match persistence_service.load_all_batch_info().await {
        Ok(mut batches) => {
            // 🔧 修复：检查并修复缺失的站场信息
            // 业务说明：早期版本可能没有保存站场信息，这里尝试从测试实例中恢复
            for batch in &mut batches {
                if batch.station_name.is_none() {
                    // 尝试从关联的测试实例中恢复站场信息
                    match persistence_service.load_test_instances_by_batch(&batch.batch_id).await {
                        Ok(instances) => {
                            if let Some(first_instance) = instances.first() {
                                // 从实例的变量描述或其他字段中尝试提取站场信息
                                if let Some(station_from_instance) = extract_station_from_instance(first_instance) {
                                    batch.station_name = Some(station_from_instance.clone());

                                    // 将恢复的站场信息保存回数据库
                                    // Rust知识点：if let Err(e) 模式匹配错误情况
                                    if let Err(e) = persistence_service.save_batch_info(batch).await {
                                        warn!("保存恢复的站场信息失败: {}", e);
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            warn!("加载批次 {} 的测试实例失败: {}", batch.batch_id, e);
                        }
                    }
                }
            }

            // 转换为仪表盘批次信息，并标识当前会话批次
            // Rust知识点：map() 转换每个元素，闭包 |batch| 捕获参数
            let dashboard_batches: Vec<DashboardBatchInfo> = batches.into_iter()
                .map(|batch| {
                    let is_current_session = session_batch_ids.contains(&batch.batch_id);
                    let has_station_name = batch.station_name.is_some();

                    DashboardBatchInfo {
                        batch_info: batch,
                        is_current_session,
                        has_station_name,
                    }
                })
                .collect();

            // 统计当前会话和历史批次数量（用于日志）
            let current_session_count = dashboard_batches.iter()
                .filter(|b| b.is_current_session)
                .count();
            let historical_count = dashboard_batches.len() - current_session_count;

            Ok(dashboard_batches)
        }
        Err(e) => {
            error!("获取仪表盘批次列表失败: {}", e);
            Err(format!("获取失败: {}", e))
        }
    }
}

/// 从测试实例中提取站场信息的辅助函数
/// 
/// 业务说明：
/// 尝试从多个来源提取站场信息，用于修复历史数据
/// 
/// 优先级：
/// 1. 从批次名称中提取
/// 2. 从实例ID中提取
/// 3. 返回默认值
fn extract_station_from_instance(instance: &crate::models::structs::ChannelTestInstance) -> Option<String> {
    // 尝试从测试批次名称中提取站场信息
    if let Some(station) = extract_station_from_description(&instance.test_batch_name) {
        return Some(station);
    }

    // 尝试从实例ID中提取站场信息（如果包含站场前缀）
    if let Some(station) = extract_station_from_tag(&instance.instance_id) {
        return Some(station);
    }

    // 如果都无法提取，返回默认值
    Some("未知站场".to_string())
}

/// 从描述文本中提取站场信息
/// 
/// 业务说明：
/// 使用预定义的站场名称模式进行匹配
/// 这些是常见的电厂和能源集团名称
fn extract_station_from_description(description: &str) -> Option<String> {
    // 常见的站场名称模式
    let station_patterns = [
        "樟洋电厂", "华能电厂", "大唐电厂", "国电电厂", "中电投",
        "华电集团", "神华集团", "中煤集团", "国家电投"
    ];

    // Rust知识点：&station_patterns 引用数组，避免所有权转移
    for pattern in &station_patterns {
        if description.contains(pattern) {
            return Some(pattern.to_string());
        }
    }

    None
}

/// 从标签中提取站场信息
/// 
/// 业务说明：
/// 根据标签前缀判断站场，这是一种简化的站场识别规则
/// ZY = 樟洋电厂, HN = 华能电厂, DT = 大唐电厂
fn extract_station_from_tag(tag: &str) -> Option<String> {
    // 如果标签包含站场信息的前缀，尝试提取
    if tag.len() > 2 {
        // Rust知识点：&tag[..2] 字符串切片，获取前两个字符
        let prefix = &tag[..2];
        match prefix {
            "ZY" => Some("樟洋电厂".to_string()),
            "HN" => Some("华能电厂".to_string()),
            "DT" => Some("大唐电厂".to_string()),
            _ => None,
        }
    } else {
        None
    }
}

/// 获取批次的通道定义列表
/// 
/// 业务说明：
/// 获取指定批次关联的所有通道定义
/// 用于显示批次详情或进行测试准备
/// 
/// 调用链：
/// 前端 -> get_batch_channel_definitions -> PersistenceService -> 返回通道定义
#[tauri::command]
pub async fn get_batch_channel_definitions(
    batch_id: String,
    state: State<'_, AppState>
) -> Result<Vec<ChannelPointDefinition>, String> {
    info!("获取批次{}的通道定义", batch_id);

    let persistence_service = &state.persistence_service;

    match persistence_service.load_all_channel_definitions().await {
        Ok(definitions) => {
            // TODO: 这里应该根据batch_id过滤，但目前的持久化服务接口还不支持
            // 暂时返回所有定义
            info!("成功获取{}个通道定义", definitions.len());
            Ok(definitions)
        }
        Err(e) => {
            error!("获取通道定义失败: {}", e);
            Err(format!("获取失败: {}", e))
        }
    }
}

// ============================================================================
// 步骤3.4要求的核心业务流程命令
// ============================================================================

/// 导入Excel并准备批次的参数
/// 
/// 业务说明：
/// 这是核心导入流程的请求参数
#[derive(Debug, Deserialize)]
pub struct ImportExcelAndPrepareBatchCmdArgs {
    pub file_path_str: String,              // Excel文件路径
    pub product_model: Option<String>,      // 产品型号（目前未使用）
    pub serial_number: Option<String>,      // 序列号（目前未使用）
}

/// 导入Excel并准备批次的响应
/// 
/// 业务说明：
/// 返回创建的批次信息、分配的测试实例和统计摘要
#[derive(Debug, Serialize)]
pub struct ImportAndPrepareBatchResponse {
    pub batch_info: TestBatchInfo,                      // 批次信息
    pub instances: Vec<crate::models::ChannelTestInstance>, // 分配的测试实例
    /// 分配摘要（包含各模块类型点位数量等统计信息）
    pub allocation_summary: crate::application::services::batch_allocation_service::AllocationSummary,
}

/// 开始批次测试的参数
#[derive(Debug, Deserialize)]
pub struct StartTestsForBatchCmdArgs {
    pub batch_id: String,                   // 要开始测试的批次ID
}

/// 获取批次状态的参数
#[derive(Debug, Deserialize)]
pub struct GetBatchStatusCmdArgs {
    pub batch_id: String,                   // 要查询状态的批次ID
}

/// 批次详情载荷
/// 
/// 业务说明：
/// 包含批次的完整信息，用于前端显示批次详情
#[derive(Debug, Serialize)]
pub struct BatchDetailsPayload {
    pub batch_info: TestBatchInfo,                      // 批次基本信息
    pub instances: Vec<crate::models::ChannelTestInstance>, // 测试实例列表
    pub definitions: Vec<ChannelPointDefinition>,       // 通道定义列表
    pub allocation_summary: AllocationSummary,          // 分配统计
    pub progress: BatchProgressInfo,                    // 进度信息
}

/// 批次进度信息
/// 
/// 业务说明：
/// 实时统计批次的测试进度
#[derive(Debug, Serialize)]
pub struct BatchProgressInfo {
    pub total_points: u32,                  // 总点位数
    pub tested_points: u32,                 // 已测试点位数
    pub passed_points: u32,                 // 通过的点位数
    pub failed_points: u32,                 // 失败的点位数
    pub skipped_points: u32,                // 跳过的点位数
}

/// 导入Excel文件并自动分配批次(核心逻辑，前端调用的入口点)
/// 
/// 业务说明：
/// 这是整个数据导入流程的核心入口，执行以下步骤：
/// 1. 清理旧数据和状态
/// 2. 解析Excel文件
/// 3. 初始化全局功能测试
/// 4. 创建批次和分配通道
/// 5. 保存所有数据到数据库
/// 
/// 调用链：
/// 前端 -> import_excel_and_prepare_batch_cmd -> ExcelImporter -> BatchAllocationService -> PersistenceService
/// 
/// Rust知识点：
/// - async/await 异步编程模式
/// - Result<T, E> 错误处理
/// - 作用域 {} 控制锁的生命周期
#[tauri::command]
pub async fn import_excel_and_prepare_batch_cmd(
    args: ImportExcelAndPrepareBatchCmdArgs,
    state: State<'_, AppState>
) -> Result<ImportAndPrepareBatchResponse, String> {
    // State<'_, AppState>AppState 是全局状态结构体，它包含了整个应用共享的数据和服务(依赖注入)
    
    // ===== 先行清空旧的内存状态 & 会话批次 =====
    // 业务说明：确保每次导入都是干净的状态，避免数据混乱
    state.channel_state_manager.clear_caches().await;
    {
        // Rust知识点：使用作用域{}来限制锁的生命周期
        let mut ids = state.session_batch_ids.lock().await;
        ids.clear();
    }

    // 1. 解析Excel文件
    let definitions = match ExcelImporter::parse_excel_file(&args.file_path_str).await {
        Ok(defs) => defs,
        Err(e) => {
            error!("❌ [IMPORT_EXCEL] Excel文件解析失败: {}", e);
            return Err(format!("Excel文件解析失败: {}", e));
        }
    };

    // === 为新站场初始化全局功能测试状态 ===
    // 业务说明：每个站场都需要进行全局功能测试（如报警测试、通信测试等）
    {
        use std::collections::HashSet;
        use crate::models::structs::{GlobalFunctionTestStatus, GlobalFunctionKey, default_id};
        use crate::models::enums::OverallTestStatus;
        
        // 使用HashSet来存储站场名称，避免重复
        let mut stations: HashSet<String> = HashSet::new();
        for def in &definitions {
            stations.insert(def.station_name.clone());
        }
        
        let import_time = chrono::Utc::now().to_rfc3339();
        for station in stations {
            // 根据站场名和时间查询数据库，但是导入的时候肯定是新的记录，所以这部分应该永远都是空的
            let existing = match state.persistence_service.load_global_function_test_statuses_by_station_time(&station, &import_time).await {
                Ok(v) => v,
                Err(e) => {
                    error!("查询站场 {} 全局功能测试状态失败: {}", station, e);
                    Vec::new()
                }
            };
            
            // 如果查询结果为空，说明是新的站场，需要初始化
            if existing.is_empty() {
                // 先调用确保批次默认记录存在（幂等）
                    // 确保全局功能测试记录存在（幂等操作）
                    if let Err(e) = state.persistence_service.ensure_global_function_tests(&station, &import_time).await {
                        error!("初始化全局功能测试状态失败: {}", e);
                    }
                    
                    // TODO: 这里加载的是上位机功能检查的5个项
                    // `ensure_global_function_tests` 已确保数据库存在 5 条默认记录，这里仅同步到内存缓存
                    if let Ok(list) = state
                        .persistence_service
                        .load_global_function_test_statuses_by_station_time(&station, &import_time)
                        .await {
                        // 将数据库中的上位机功能检查状态填充至内存中
                        let mut guard = state.global_function_tests.lock().await;
                        guard.extend(list);
                    }
                }
            }
        }
    }

    if definitions.is_empty() {
        error!("❌ [IMPORT_EXCEL] Excel文件中没有找到有效的通道定义");
        return Err("Excel文件中没有找到有效的通道定义".to_string());
    }

    // 2. 立即执行批次分配 - 这是关键步骤
    // 业务说明：BatchAllocationService负责将点位分配到物理测试通道
    let mut allocation_result = match execute_batch_allocation(&definitions, &args, &state).await {
        Ok(result) => result,
        Err(e) => {
            error!("❌ [IMPORT_EXCEL] 批次分配失败: {}", e);
            return Err(format!("批次分配失败: {}", e));
        }
    };

    // === 回填站场名称（功能检查用） ===
    // 业务说明：确保批次信息中包含站场名称，用于后续的功能检查
    if let Some(first_def) = definitions.first() {
        let primary_station = first_def.station_name.clone();
        // Rust知识点：iter_mut() 获取可变迭代器，允许修改元素
        for b in allocation_result.batches.iter_mut() {
            if b.station_name.is_none() || b.station_name.as_ref().unwrap().is_empty() {
                b.station_name = Some(primary_station.clone());
            }
        }
    }

    // 3. 将分配结果存储到状态管理器
    // 业务说明：状态管理器维护内存中的测试状态，提供快速访问
    match store_allocation_to_state_manager(&allocation_result, &state).await {
        Ok(_) => {},
        Err(e) => {
            error!("存储到状态管理器失败: {}", e);
            return Err(format!("存储批次数据失败: {}", e));
        }
    }

    // 4. 构建响应数据
    // 从分配结果中获取第一个批次作为主要批次信息
    // Rust知识点：ok_or_else() 将 Option 转换为 Result，None时执行闭包
    let primary_batch = allocation_result.batches.first()
        .ok_or_else(|| "批次分配失败：没有生成任何批次".to_string())?;

    let response = ImportAndPrepareBatchResponse {
        batch_info: primary_batch.clone(),
        instances: allocation_result.allocated_instances.clone(),
        allocation_summary: allocation_result.allocation_summary.clone(),
    };

    Ok(response)
}

/// 开始批次测试
/// 
/// 业务说明：
/// 触发批次的自动测试流程
/// 测试协调服务会依次执行每个测试实例
/// 
/// 调用链：
/// 前端 -> start_tests_for_batch_cmd -> TestCoordinationService -> 测试引擎
#[tauri::command]
pub async fn start_tests_for_batch_cmd(
    args: StartTestsForBatchCmdArgs,
    state: State<'_, AppState>
) -> Result<(), String> {
    info!("开始批次测试: {}", args.batch_id);

    // 委托给测试协调服务执行
    // Rust知识点：map_err() 转换错误类型
    state.test_coordination_service
        .start_batch_testing(&args.batch_id)
        .await
        .map_err(|e| {
            error!("开始批次测试失败: {}", e);
            e.to_string()
        })
}

/// 获取批次状态
/// 
/// 业务说明：
/// 获取批次的详细状态信息，包括：
/// 1. 批次基本信息
/// 2. 测试实例列表（优先从内存获取最新状态）
/// 3. 通道定义列表
/// 4. 测试进度统计
/// 
/// 调用链：
/// 前端轮询 -> get_batch_status_cmd -> ChannelStateManager/PersistenceService -> 返回状态
/// 
/// 性能优化：
/// - 优先从内存缓存获取数据
/// - 减少日志输出
/// - 按标签排序保证顺序一致性
#[tauri::command]
pub async fn get_batch_status_cmd(
    args: GetBatchStatusCmdArgs,
    state: State<'_, AppState>
) -> Result<BatchDetailsPayload, String> {
    let batch_id = args.batch_id;
    // 🔧 性能优化：移除详细状态获取日志

    // 获取批次信息
    let batch_info = match state.persistence_service.load_batch_info(&batch_id).await {
        Ok(Some(info)) => {
            // 🔧 性能优化：移除批次信息获取日志
            info
        },
        Ok(None) => {
            error!("❌ [GET_BATCH_STATUS] 批次不存在: {}", batch_id);
            return Err(format!("批次不存在: {}", batch_id));
        },
        Err(e) => {
            error!("❌ [GET_BATCH_STATUS] 获取批次信息失败: {}", e);
            return Err(format!("获取批次信息失败: {}", e));
        }
    };

    // 🔧 修复：优先从状态管理器内存获取测试实例，确保获取最新数据
    let instances = {
        // 首先尝试从状态管理器内存缓存获取
        let cached_instances = state.channel_state_manager.get_all_cached_test_instances().await;

        // 过滤出属于当前批次的实例
        let batch_instances: Vec<_> = cached_instances.into_iter()
            .filter(|instance| instance.test_batch_id == batch_id)
            .collect();

        if !batch_instances.is_empty() {
            // 🔧 性能优化：移除内存数据获取日志

            // 🔧 修复：按照定义的标签排序测试实例
            let mut sorted_instances = batch_instances;
            sorted_instances.sort_by(|a, b| {
                // 获取对应的定义来比较标签
                let def_a = state.channel_state_manager.get_channel_definition(&a.definition_id);
                let def_b = state.channel_state_manager.get_channel_definition(&b.definition_id);

                // 使用 futures::executor::block_on 来等待异步操作
                // Rust知识点：block_on 在同步上下文中执行异步代码
                let tag_a = match futures::executor::block_on(def_a) {
                    Some(def) => def.tag.clone(),
                    None => String::new(),
                };
                let tag_b = match futures::executor::block_on(def_b) {
                    Some(def) => def.tag.clone(),
                    None => String::new(),
                };

                tag_a.cmp(&tag_b)
            });

            sorted_instances
        } else {
            // 如果内存中没有数据，则从数据库获取（兜底方案）
            // 🔧 性能优化：移除数据库获取警告日志
            match state.persistence_service.load_test_instances_by_batch(&batch_id).await {
                Ok(mut instances) => {
                    // 🔧 修复：对数据库获取的实例也进行排序
                    instances.sort_by(|a, b| {
                        // 获取对应的定义来比较标签
                        let def_a = state.channel_state_manager.get_channel_definition(&a.definition_id);
                        let def_b = state.channel_state_manager.get_channel_definition(&b.definition_id);

                        let tag_a = match futures::executor::block_on(def_a) {
                            Some(def) => def.tag.clone(),
                            None => String::new(),
                        };
                        let tag_b = match futures::executor::block_on(def_b) {
                            Some(def) => def.tag.clone(),
                            None => String::new(),
                        };

                        tag_a.cmp(&tag_b)
                    });

                    instances
                },
                Err(e) => {
                    error!("❌ [GET_BATCH_STATUS] 获取测试实例失败: {}", e);
                    return Err(format!("获取测试实例失败: {}", e));
                }
            }
        }
    };

    // 从状态管理器获取通道定义，并按照导入时的顺序排序
    let definitions = {
        let state_manager = &state.channel_state_manager;
        // Rust知识点：HashSet 用于去重
        let instance_definition_ids: std::collections::HashSet<String> = instances
            .iter()
            .map(|instance| instance.definition_id.clone())
            .collect();

        let mut definitions = Vec::new();
        for definition_id in &instance_definition_ids {
            if let Some(definition) = state_manager.get_channel_definition(definition_id).await {
                definitions.push(definition);
            } else {
                warn!("状态管理器中未找到定义: {}", definition_id);
            }
        }

        // 🔧 修复：按照点位标签排序（保持一致的顺序）
        definitions.sort_by(|a, b| {
            a.tag.cmp(&b.tag)
        });

        definitions
    };

    // 计算进度信息
    let total_points = instances.len() as u32;
    let mut tested_points = 0;
    let mut passed_points = 0;
    let mut failed_points = 0;
    let mut skipped_points = 0;

    for instance in &instances {
        match instance.overall_status {
            crate::models::OverallTestStatus::TestCompletedPassed => {
                tested_points += 1;
                passed_points += 1;
            }
            crate::models::OverallTestStatus::TestCompletedFailed => {
                tested_points += 1;
                failed_points += 1;
            }
            crate::models::OverallTestStatus::NotTested => {
                skipped_points += 1;
            }
            _ => {
                tested_points += 1;
            }
        }
    }

    let progress = BatchProgressInfo {
        total_points,
        tested_points,
        passed_points,
        failed_points,
        skipped_points,
    };

    // 创建分配摘要
    let allocation_summary = AllocationSummary {
        total_definitions: definitions.len() as u32,
        allocated_instances: instances.len() as u32,
        skipped_definitions: 0, // 这里可以根据实际情况计算
        allocation_errors: Vec::new(), // 这里可以根据实际情况填充
    };

    // 🔧 性能优化：移除批次状态统计日志

    let payload = BatchDetailsPayload {
        batch_info,
        instances,
        definitions,
        allocation_summary,
        progress,
    };

    // 🔧 性能优化：移除序列化检查，直接返回数据
    // 序列化检查已在开发阶段验证，生产环境无需重复检查

    Ok(payload)
}

/// 准备批次测试实例的参数
#[derive(Debug, Deserialize)]
pub struct PrepareTestInstancesForBatchCmdArgs {
    pub batch_id: String,
    pub definition_ids: Option<Vec<String>>, // 可选的定义ID列表，如果为空则使用所有可用定义
}

/// 准备批次测试实例的响应
#[derive(Debug, Serialize)]
pub struct PrepareTestInstancesResponse {
    pub batch_info: TestBatchInfo,
    pub instances: Vec<crate::models::ChannelTestInstance>,
    pub definitions: Vec<ChannelPointDefinition>,
    pub allocation_summary: AllocationSummary,
}

/// 分配摘要信息
#[derive(Debug, Serialize)]
pub struct AllocationSummary {
    pub total_definitions: u32,
    pub allocated_instances: u32,
    pub skipped_definitions: u32,
    pub allocation_errors: Vec<String>,
}

/// 准备批次测试实例 - 实现自动分配逻辑
#[tauri::command]
pub async fn prepare_test_instances_for_batch_cmd(
    args: PrepareTestInstancesForBatchCmdArgs,
    state: State<'_, AppState>
) -> Result<PrepareTestInstancesResponse, String> {
    info!("准备批次测试实例: 批次ID = {}", args.batch_id);

    // 1. 验证批次是否存在
    let mut batch_info = match state.persistence_service.load_batch_info(&args.batch_id).await {
        Ok(Some(info)) => info,
        Ok(None) => return Err(format!("批次不存在: {}", args.batch_id)),
        Err(e) => {
            error!("获取批次信息失败: {}", e);
            return Err(format!("获取批次信息失败: {}", e));
        }
    };

    // 2. 获取要分配的通道定义
    let all_definitions = match state.persistence_service.load_all_channel_definitions().await {
        Ok(definitions) => definitions,
        Err(e) => {
            error!("获取通道定义失败: {}", e);
            return Err(format!("获取通道定义失败: {}", e));
        }
    };

    // 3. 根据definition_ids过滤定义（如果提供了）
    let target_definitions = if let Some(ref definition_ids) = args.definition_ids {
        all_definitions.into_iter()
            .filter(|def| definition_ids.contains(&def.id))
            .collect::<Vec<_>>()
    } else {
        all_definitions
    };

    if target_definitions.is_empty() {
        return Err("没有找到可用的通道定义进行分配".to_string());
    }

    info!("找到 {} 个通道定义需要分配测试实例", target_definitions.len());

    // 4. 检查是否已存在测试实例
    let existing_instances = match state.persistence_service.load_test_instances_by_batch(&args.batch_id).await {
        Ok(instances) => instances,
        Err(e) => {
            warn!("获取现有测试实例失败，将创建新实例: {}", e);
            Vec::new()
        }
    };

    let existing_definition_ids: std::collections::HashSet<String> = existing_instances
        .iter()
        .map(|instance| instance.definition_id.clone())
        .collect();

    // 5. 为每个定义创建测试实例（跳过已存在的）
    let mut instances = existing_instances;
    let mut allocation_errors = Vec::new();
    let mut allocated_count = 0;
    let mut skipped_count = 0;

    for definition in &target_definitions {
        if existing_definition_ids.contains(&definition.id) {
            info!("跳过已存在的测试实例: 定义ID = {}", definition.id);
            skipped_count += 1;
            continue;
        }

        // 使用通道状态管理器创建测试实例
        match state.channel_state_manager.create_test_instance(
            &definition.id,
            &args.batch_id
        ).await {
            Ok(instance) => {
                // 保存测试实例到数据库
                if let Err(e) = state.persistence_service.save_test_instance(&instance).await {
                    let error_msg = format!("保存测试实例失败: {} - {}", instance.instance_id, e);
                    error!("{}", error_msg);
                    allocation_errors.push(error_msg);
                } else {
                    info!("成功创建并保存测试实例: {} (定义: {})", instance.instance_id, definition.tag);
                    instances.push(instance);
                    allocated_count += 1;
                }
            }
            Err(e) => {
                let error_msg = format!("创建测试实例失败: {} - {}", definition.tag, e);
                error!("{}", error_msg);
                allocation_errors.push(error_msg);
            }
        }
    }

    // 6. 更新批次信息
    batch_info.total_points = instances.len() as u32;
    batch_info.last_updated_time = chrono::Utc::now();

    // 保存更新后的批次信息
    if let Err(e) = state.persistence_service.save_batch_info(&batch_info).await {
        warn!("更新批次信息失败: {}", e);
    }

    // 7. 构建分配摘要
    let allocation_summary = AllocationSummary {
        total_definitions: target_definitions.len() as u32,
        allocated_instances: allocated_count,
        skipped_definitions: skipped_count,
        allocation_errors,
    };

    info!("批次测试实例准备完成: 总定义数={}, 新分配={}, 跳过={}, 错误数={}",
          allocation_summary.total_definitions,
          allocation_summary.allocated_instances,
          allocation_summary.skipped_definitions,
          allocation_summary.allocation_errors.len());

    Ok(PrepareTestInstancesResponse {
        batch_info,
        instances,
        definitions: target_definitions,
        allocation_summary,
    })
}

/// 导入Excel并自动分配通道命令
/// 这个命令会导入Excel数据，创建通道定义，然后自动分配测试批次
#[tauri::command]
pub async fn import_excel_and_allocate_channels_cmd(
    file_path: String,
    product_model: Option<String>,
    serial_number: Option<String>,
    state: State<'_, AppState>
) -> Result<AllocationResult, String> {
    log::info!("开始导入Excel文件并分配通道: {}", file_path);

    // 1. 解析Excel文件
    let excel_response = match ExcelImporter::parse_excel_file(&file_path).await {
        Ok(definitions) => definitions,
        Err(e) => {
            log::error!("Excel文件解析失败: {}", e);
            return Err(format!("Excel文件解析失败: {}", e));
        }
    };

    // 2. 转换为通道定义
    let definitions = excel_response;

    log::info!("成功转换 {} 个通道定义", definitions.len());

    // 🔧 修复：只返回解析结果，不创建批次
    Ok(AllocationResult {
        batches: Vec::new(), // 不创建批次
        allocated_instances: Vec::new(), // 不创建实例
        allocation_summary: crate::application::services::batch_allocation_service::AllocationSummary {
            total_channels: definitions.len(),
            ai_channels: definitions.iter().filter(|d| d.module_type == crate::models::ModuleType::AI).count(),
            ao_channels: definitions.iter().filter(|d| d.module_type == crate::models::ModuleType::AO).count(),
            di_channels: definitions.iter().filter(|d| d.module_type == crate::models::ModuleType::DI).count(),
            do_channels: definitions.iter().filter(|d| d.module_type == crate::models::ModuleType::DO).count(),
            stations: definitions.iter().map(|d| d.station_name.clone()).collect::<std::collections::HashSet<_>>().into_iter().collect(),
            estimated_test_duration_minutes: 0,
        },
        channel_definitions: Some(definitions), // 只返回解析的定义
    })
}

// 🔧 修复：删除默认配置创建函数，强制用户配置真实的测试PLC

/// 解析Excel文件并创建批次的参数
#[derive(Debug, Deserialize)]
pub struct ParseExcelAndCreateBatchCmdArgs {
    pub file_path: String,
    pub file_name: String,
}

/// 解析Excel文件但不持久化数据的响应
#[derive(Debug, Serialize)]
pub struct ParseExcelWithoutPersistenceResponse {
    pub success: bool,
    pub message: String,
    pub definitions: Vec<ChannelPointDefinition>,
    pub definitions_count: usize,
    pub suggested_batch_info: TestBatchInfo,
}

/// 解析Excel文件但不持久化数据
///
/// 这个命令只解析Excel文件，将结果返回给前端，
/// 不会将数据保存到数据库中。数据只有在用户明确开始测试时才会持久化。
///
/// # 参数
/// * `args` - 包含文件路径和文件名的参数
///
/// # 返回
/// * `Result<ParseExcelWithoutPersistenceResponse, String>` - 解析结果（不持久化）
#[tauri::command]
pub async fn parse_excel_without_persistence_cmd(
    args: ParseExcelAndCreateBatchCmdArgs,
) -> Result<ParseExcelWithoutPersistenceResponse, String> {
    info!("收到解析Excel文件请求（不持久化）: 文件={}, 路径={}", args.file_name, args.file_path);

    // 解析Excel文件
    let definitions = match ExcelImporter::parse_excel_file(&args.file_path).await {
        Ok(defs) => {
            info!("Excel文件解析成功，共解析{}个通道定义", defs.len());
            defs
        }
        Err(e) => {
            error!("Excel文件解析失败: {}", e);
            return Ok(ParseExcelWithoutPersistenceResponse {
                success: false,
                message: format!("Excel解析失败: {}", e),
                definitions: vec![],
                definitions_count: 0,
                suggested_batch_info: TestBatchInfo::new(None, None),
            });
        }
    };

    if definitions.is_empty() {
        warn!("Excel文件中没有找到有效的通道定义");
        return Ok(ParseExcelWithoutPersistenceResponse {
            success: false,
            message: "Excel文件中没有找到有效的通道定义".to_string(),
            definitions: vec![],
            definitions_count: 0,
            suggested_batch_info: TestBatchInfo::new(None, None),
        });
    }

    // 创建建议的批次信息（不保存）
    let mut suggested_batch = TestBatchInfo::new(
        Some("自动导入".to_string()), // 默认产品型号
        None, // 序列号留空，用户可以后续修改
    );

    // 设置批次信息
    suggested_batch.total_points = definitions.len() as u32;
    suggested_batch.batch_name = format!("从{}导入", args.file_name);

    let definitions_count = definitions.len();
    info!("Excel解析完成，返回{}个通道定义（未持久化）", definitions_count);

    Ok(ParseExcelWithoutPersistenceResponse {
        success: true,
        message: format!("成功解析{}个通道定义，数据未持久化", definitions_count),
        definitions,
        definitions_count,
        suggested_batch_info: suggested_batch,
    })
}

/// 解析Excel文件并创建批次的响应
#[derive(Debug, Serialize)]
pub struct ParseExcelAndCreateBatchResponse {
    pub success: bool,
    pub message: String,
    pub batch_id: Option<String>,
    pub definitions_count: usize,
    pub batch_info: Option<TestBatchInfo>,
}

/// 解析Excel文件并自动创建批次
///
/// 这个命令将Excel解析和批次创建合并为一个操作，
/// 简化前端的调用流程
///
/// # 参数
/// * `args` - 包含文件路径和文件名的参数
/// * `state` - 应用状态
///
/// # 返回
/// * `Result<ParseExcelAndCreateBatchResponse, String>` - 操作结果
#[tauri::command]
pub async fn parse_excel_and_create_batch_cmd(
    args: ParseExcelAndCreateBatchCmdArgs,
    state: State<'_, AppState>
) -> Result<ParseExcelAndCreateBatchResponse, String> {
    // ===== 先清空旧的内存缓存和会话批次集合 =====
    state.channel_state_manager.clear_caches().await;
    {
        let mut ids = state.session_batch_ids.lock().await;
        ids.clear();
    }

    info!("收到解析Excel并创建批次请求: 文件={}, 路径={}", args.file_name, args.file_path);

    // 第一步：解析Excel文件
    let definitions = match ExcelImporter::parse_excel_file(&args.file_path).await {
        Ok(defs) => {
            info!("Excel文件解析成功，共解析{}个通道定义", defs.len());
            defs
        }
        Err(e) => {
            error!("Excel文件解析失败: {}", e);
            return Ok(ParseExcelAndCreateBatchResponse {
                success: false,
                message: format!("Excel解析失败: {}", e),
                batch_id: None,
                definitions_count: 0,
                batch_info: None,
            });
        }
    };

    if definitions.is_empty() {
        warn!("Excel文件中没有找到有效的通道定义");
        return Ok(ParseExcelAndCreateBatchResponse {
            success: false,
            message: "Excel文件中没有找到有效的通道定义".to_string(),
            batch_id: None,
            definitions_count: 0,
            batch_info: None,
        });
    }

    // 第二步：创建测试批次
    let mut test_batch = TestBatchInfo::new(
        Some("自动导入".to_string()), // 默认产品型号
        None, // 序列号留空，用户可以后续修改
    );

    // 设置批次信息
    test_batch.total_points = definitions.len() as u32;
    test_batch.batch_name = format!("从{}导入", args.file_name);

    // 获取持久化服务
    let persistence_service = &state.persistence_service;

    // 第三步：保存批次信息
    match persistence_service.save_batch_info(&test_batch).await {
        Ok(_) => {
            info!("测试批次创建成功: {}", test_batch.batch_id);

            // 将批次ID添加到当前会话跟踪中
            {
                let mut session_batch_ids = state.session_batch_ids.lock().await;
                session_batch_ids.insert(test_batch.batch_id.clone());
                info!("批次 {} 已添加到当前会话跟踪", test_batch.batch_id);
            }
        }
        Err(e) => {
            error!("创建测试批次失败: {}", e);
            return Ok(ParseExcelAndCreateBatchResponse {
                success: false,
                message: format!("创建批次失败: {}", e),
                batch_id: None,
                definitions_count: definitions.len(),
                batch_info: None,
            });
        }
    }

    // 🔥 第四步：为通道定义设置批次ID并保存
    let mut saved_count = 0;
    let mut errors = Vec::new();

    // 为每个通道定义设置批次ID
    let mut updated_definitions = definitions.clone();
    for definition in &mut updated_definitions {
        definition.batch_id = Some(test_batch.batch_id.clone());
        info!("🔗 为通道定义 {} 设置批次ID: {}", definition.tag, test_batch.batch_id);
    }

    for definition in &updated_definitions {
        match persistence_service.save_channel_definition(definition).await {
            Ok(_) => saved_count += 1,
            Err(e) => {
                let error_msg = format!("保存通道定义失败: {} - {}", definition.tag, e);
                error!("{}", error_msg);
                errors.push(error_msg);
            }
        }
    }

    // 第五步：返回结果
    let success = saved_count > 0;
    let message = if success {
        if errors.is_empty() {
            format!("成功创建批次并保存{}个通道定义", saved_count)
        } else {
            format!("批次创建成功，保存{}个通道定义，{}个失败", saved_count, errors.len())
        }
    } else {
        format!("批次创建失败，无法保存任何通道定义。错误: {}", errors.join("; "))
    };

    info!("{}", message);

    Ok(ParseExcelAndCreateBatchResponse {
        success,
        message,
        batch_id: if success { Some(test_batch.batch_id.clone()) } else { None },
        definitions_count: definitions.len(),
        batch_info: if success { Some(test_batch) } else { None },
    })
}

/// 清理当前会话数据
///
/// 这个命令会清除当前会话中创建的所有批次数据，
/// 确保测试区域回到初始状态
#[tauri::command]
pub async fn clear_session_data(
    state: State<'_, AppState>
) -> Result<String, String> {
    info!("收到清理会话数据请求");

    // 获取当前会话中的批次ID列表
    let session_batch_ids = {
        let mut session_batch_ids_guard = state.session_batch_ids.lock().await;
        let ids = session_batch_ids_guard.clone();
        session_batch_ids_guard.clear(); // 清空会话跟踪
        ids
    };

    if session_batch_ids.is_empty() {
        info!("当前会话中没有需要清理的数据");
        return Ok("当前会话中没有需要清理的数据".to_string());
    }

    info!("开始清理{}个批次的数据", session_batch_ids.len());

    // 额外：重置全局功能测试状态
    if let Err(e) = state.persistence_service.reset_global_function_test_statuses().await {
        error!("重置全局功能测试状态失败: {}", e);
    } else {
        // 清空缓存
        let mut guard = state.global_function_tests.lock().await;
        guard.clear();
    }

    let persistence_service = &state.persistence_service;
    let mut cleaned_count = 0;
    let mut errors = Vec::new();

    // 删除每个批次及其相关数据
    for batch_id in &session_batch_ids {
        // 删除批次的测试实例
        match persistence_service.load_test_instances_by_batch(batch_id).await {
            Ok(instances) => {
                for instance in instances {
                    if let Err(e) = persistence_service.delete_test_instance(&instance.instance_id).await {
                        errors.push(format!("删除测试实例失败: {} - {}", instance.instance_id, e));
                    }
                }
            }
            Err(e) => {
                errors.push(format!("加载批次{}的测试实例失败: {}", batch_id, e));
            }
        }

        // 删除批次信息
        match persistence_service.delete_batch_info(batch_id).await {
            Ok(_) => {
                cleaned_count += 1;
            }
            Err(e) => {
                errors.push(format!("删除批次{}失败: {}", batch_id, e));
            }
        }
    }

    let message = if errors.is_empty() {
        format!("成功清理{}个批次的会话数据", cleaned_count)
    } else {
        format!("清理完成，成功删除{}个批次，{}个操作失败", cleaned_count, errors.len())
    };

    info!("{}", message);
    Ok(message)
}

/// 创建批次并持久化数据的请求
#[derive(Debug, Deserialize)]
pub struct CreateBatchAndPersistDataRequest {
    pub batch_info: TestBatchInfo,
    pub definitions: Vec<ChannelPointDefinition>,
}

/// 创建批次并持久化数据的响应
#[derive(Debug, Serialize)]
pub struct CreateBatchAndPersistDataResponse {
    pub success: bool,
    pub message: String,
    pub batch_id: Option<String>,
    /// 所有生成的批次信息
    pub all_batches: Vec<TestBatchInfo>,
    pub saved_definitions_count: usize,
    pub created_instances_count: usize,
}

/// 一键导入Excel并创建批次的响应结构
#[derive(Debug, Serialize)]
pub struct ImportExcelAndCreateBatchResponse {
    pub success: bool,
    pub message: String,
    pub import_result: ImportResult,
    pub allocation_result: AllocationResult,
}

/// 创建批次并持久化数据
///
/// 这个命令在用户明确开始测试时被调用，
/// 将之前解析的Excel数据持久化到数据库中
///
/// ⚠️ 修复：现在使用通道分配服务来正确生成多个批次
///
/// # 参数
/// * `request` - 包含批次信息和通道定义的请求
/// * `state` - 应用状态
///
/// # 返回
/// * `Result<CreateBatchAndPersistDataResponse, String>` - 持久化结果
#[tauri::command]
pub async fn create_batch_and_persist_data_cmd(
    request: CreateBatchAndPersistDataRequest,
    state: State<'_, AppState>
) -> Result<CreateBatchAndPersistDataResponse, String> {
    info!("收到创建批次并持久化数据请求: 批次ID={}, 定义数量={}",
          request.batch_info.batch_id, request.definitions.len());

    // ===== 重要：根据架构设计，批次创建时不应该保存通道定义到数据库 =====
    // 通道定义应该在导入点表时已经保存到数据库
    // 批次创建时只需要在内存状态管理器中管理测试实例
    log::info!("[CreateBatchData] ===== 开始批次分配（仅内存操作） =====");
    log::info!("[CreateBatchData] 输入: {} 个通道定义", request.definitions.len());

    // 验证输入的通道定义
    if request.definitions.is_empty() {
        error!("没有提供任何通道定义，无法进行批次分配");
        return Ok(CreateBatchAndPersistDataResponse {
            success: false,
            message: "没有提供任何通道定义".to_string(),
            batch_id: None,
            all_batches: Vec::new(),
            saved_definitions_count: 0,
            created_instances_count: 0,
        });
    }

    log::info!("[CreateBatchData] 验证通过，开始批次分配");

    // 第二步：使用通道分配服务进行批次分配
    log::info!("[CreateBatchData] ===== 开始使用通道分配服务 =====");

    // ===== 从数据库获取真实的测试PLC配置 =====
    let test_plc_config = match state.test_plc_config_service.get_test_plc_config().await {
        Ok(config) => {
            log::info!("[CreateBatchData] 成功获取数据库中的测试PLC配置: {} 个通道映射",
                config.comparison_tables.len());
            config
        }
        Err(e) => {
            error!("[CreateBatchData] 获取数据库测试PLC配置失败: {}", e);
            return Ok(CreateBatchAndPersistDataResponse {
                success: false,
                message: format!("获取测试PLC配置失败: {}，请先配置测试PLC", e),
                batch_id: None,
                all_batches: Vec::new(),
                saved_definitions_count: 0,
                created_instances_count: 0,
            });
        }
    };

    // 调用通道分配服务
    let db_conn = state.persistence_service.get_database_connection();
    let allocation_service = crate::application::services::batch_allocation_service::BatchAllocationService::new(
        Arc::new(db_conn), 
        state.channel_state_manager.clone()
    );

    let allocation_result = match allocation_service
        .create_test_batch(
            request.batch_info.batch_name.clone(),
            request.batch_info.product_model.clone(),
            request.batch_info.operator_name.clone(),
            crate::application::services::batch_allocation_service::AllocationStrategy::Smart,
            None, // filter_criteria
        )
        .await
    {
        Ok(result) => {
            log::info!("[CreateBatchData] 通道分配成功: 批次 {}, {} 个实例",
                result.batch_info.batch_name, result.test_instances.len());
            // 转换为期望的格式
            AllocationResult {
                batches: vec![result.batch_info],
                allocated_instances: result.test_instances,
                allocation_summary: result.allocation_summary,
                channel_definitions: None, // 这里没有通道定义数据
            }
        }
        Err(e) => {
            error!("通道分配失败: {}", e);
            return Ok(CreateBatchAndPersistDataResponse {
                success: false,
                message: format!("通道分配失败: {}", e),
                batch_id: None,
                all_batches: Vec::new(),
                saved_definitions_count: 0,
                created_instances_count: 0,
            });
        }
    };

    // 第三步：将批次添加到会话跟踪中（仅内存操作）
    let mut saved_batches_count = 0;
    for batch in &allocation_result.batches {

        saved_batches_count += 1;

        // 将批次ID添加到当前会话跟踪中
        {
            let mut session_batch_ids = state.session_batch_ids.lock().await;
            session_batch_ids.insert(batch.batch_id.clone());
        }
    }

    // 第四步：将测试实例添加到状态管理器中（仅内存操作）
    // 注意：根据架构设计，测试实例应该由状态管理器管理，不应该立即持久化
    let created_instances_count = allocation_result.allocated_instances.len();
    log::info!("[CreateBatchData] 创建了 {} 个测试实例（仅在内存中管理）", created_instances_count);

    // TODO: 这里应该将测试实例添加到状态管理器中
    // 当前暂时跳过，等状态管理器完善后再实现

    // 第五步：生成结果消息
    let success = saved_batches_count > 0 && created_instances_count > 0;

    let message = if success {
        format!("成功创建{}个批次，生成{}个测试实例（仅在内存中管理）",
               saved_batches_count, created_instances_count)
    } else {
        "批次创建失败".to_string()
    };

    info!("{}", message);



    Ok(CreateBatchAndPersistDataResponse {
        success,
        message,
        batch_id: if success {
            allocation_result.batches.first().map(|b| b.batch_id.clone())
        } else {
            None
        },
        all_batches: allocation_result.batches,
        saved_definitions_count: 0, // 不再保存通道定义到数据库
        created_instances_count,
    })
}

// ============================================================================
// 新的重构后的命令 - 使用重构后的服务
// ============================================================================

/// 导入Excel文件到数据库
#[tauri::command]
pub async fn import_excel_to_database_cmd(
    file_path: String,
    replace_existing: bool,
    state: State<'_, AppState>
) -> Result<ImportResult, String> {
    // 从持久化服务获取数据库连接
    let db = state.persistence_service.get_database_connection();
    let import_service = DataImportService::new(Arc::new(db.clone()));

    match import_service.import_from_excel(&file_path, replace_existing).await {
        Ok(result) => {
            Ok(result)
        }
        Err(e) => {
            error!("Excel导入失败: {:?}", e);
            Err(e.to_string())
        }
    }
}

/// 创建测试批次并分配通道
#[tauri::command]
pub async fn create_test_batch_with_allocation_cmd(
    batch_name: String,
    product_model: Option<String>,
    operator_name: Option<String>,
    strategy: String, // "ByModuleType", "ByStation", "Smart"
    filter_criteria: Option<HashMap<String, String>>,
    state: State<'_, AppState>
) -> Result<AllocationResult, String> {
    info!("收到创建测试批次请求: {}", batch_name);

    // 解析分配策略
    let allocation_strategy = match strategy.as_str() {
        "ByModuleType" => AllocationStrategy::ByModuleType,
        "ByStation" => AllocationStrategy::ByStation,
        "ByProductModel" => AllocationStrategy::ByProductModel,
        "Smart" => AllocationStrategy::Smart,
        _ => AllocationStrategy::Smart, // 默认使用智能分配
    };

    let db = state.persistence_service.get_database_connection();
    let allocation_service = BatchAllocationService::new(
        Arc::new(db.clone()), 
        state.channel_state_manager.clone()
    );

    match allocation_service.create_test_batch(
        batch_name,
        product_model,
        operator_name,
        allocation_strategy,
        filter_criteria,
    ).await {
        Ok(result) => {
            info!("测试批次创建完成: {} - {}个通道",
                  result.batch_info.batch_name,
                  result.allocation_summary.total_channels);
            // 转换为命令层的 AllocationResult
            Ok(AllocationResult {
                batches: vec![result.batch_info],
                allocated_instances: result.test_instances,
                allocation_summary: result.allocation_summary,
                channel_definitions: None, // 这里没有通道定义数据
            })
        }
        Err(e) => {
            error!("创建测试批次失败: {:?}", e);
            Err(e.to_string())
        }
    }
}

/// 获取数据库中的通道定义总数
#[tauri::command]
pub async fn get_channel_definitions_count_cmd(
    state: State<'_, AppState>
) -> Result<u64, String> {
    let db = state.persistence_service.get_database_connection();
    let import_service = DataImportService::new(Arc::new(db.clone()));

    match import_service.get_total_count().await {
        Ok(count) => Ok(count),
        Err(e) => {
            error!("获取通道定义总数失败: {:?}", e);
            Err(e.to_string())
        }
    }
}

/// 清空所有通道定义数据
#[tauri::command]
pub async fn clear_all_channel_definitions_cmd(
    state: State<'_, AppState>
) -> Result<u64, String> {
    warn!("收到清空所有通道定义数据请求");

    let db = state.persistence_service.get_database_connection();
    let import_service = DataImportService::new(Arc::new(db.clone()));

    match import_service.clear_all_data().await {
        Ok(deleted_count) => {
            Ok(deleted_count)
        }
        Err(e) => {
            error!("清空通道定义数据失败: {:?}", e);
            Err(e.to_string())
        }
    }
}

/// 删除批次请求参数
#[derive(Debug, Deserialize)]
pub struct DeleteBatchRequest {
    pub batch_id: String,
}

/// 删除批次响应
#[derive(Debug, Serialize)]
pub struct DeleteBatchResponse {
    pub success: bool,
    pub message: String,
    pub deleted_definitions_count: usize,
    pub deleted_instances_count: usize,
}

/// 删除单个批次及其相关数据
///
/// 这个命令会删除指定批次在三张表中的所有相关数据：
/// 1. test_batch_info 表中的批次信息
/// 2. channel_test_instances 表中的测试实例
/// 3. channel_point_definitions 表中的通道定义（如果只属于该批次）
///
/// # 参数
/// * `request` - 删除批次请求，包含批次ID
/// * `state` - 应用状态
///
/// # 返回
/// * `Result<DeleteBatchResponse, String>` - 删除结果
#[tauri::command]
pub async fn delete_batch_cmd(
    request: DeleteBatchRequest,
    state: State<'_, AppState>
) -> Result<DeleteBatchResponse, String> {
    let batch_id = &request.batch_id;


    let persistence_service = &state.persistence_service;

    // 检查批次是否存在
    let batch_info = match persistence_service.load_batch_info(batch_id).await {
        Ok(Some(info)) => {

            info
        },
        Ok(None) => {
            error!("❌ [DELETE_BATCH] 批次不存在: {}", batch_id);
            return Ok(DeleteBatchResponse {
                success: false,
                message: format!("批次不存在: {}", batch_id),
                deleted_definitions_count: 0,
                deleted_instances_count: 0,
            });
        },
        Err(e) => {
            error!("❌ [DELETE_BATCH] 查询批次信息失败: {}", e);
            return Ok(DeleteBatchResponse {
                success: false,
                message: format!("查询批次信息失败: {}", e),
                deleted_definitions_count: 0,
                deleted_instances_count: 0,
            });
        }
    };

    // 检查批次状态，不允许删除正在进行的测试
    if batch_info.overall_status == crate::models::OverallTestStatus::HardPointTesting ||
       batch_info.overall_status == crate::models::OverallTestStatus::ManualTesting {
        error!("❌ [DELETE_BATCH] 无法删除正在进行测试的批次: {}", batch_id);
        return Ok(DeleteBatchResponse {
            success: false,
            message: "无法删除正在进行测试的批次，请先停止测试".to_string(),
            deleted_definitions_count: 0,
            deleted_instances_count: 0,
        });
    }

    let mut deleted_definitions_count = 0;
    let mut deleted_instances_count = 0;
    let mut errors = Vec::new();

    // 1. 首先收集需要删除的通道定义ID（在删除测试实例之前）
    let mut definition_ids_to_delete = std::collections::HashSet::new();
    match persistence_service.load_test_instances_by_batch(batch_id).await {
        Ok(instances) => {
            for instance in &instances {
                definition_ids_to_delete.insert(instance.definition_id.clone());
            }
        }
        Err(e) => {
            errors.push(format!("加载批次测试实例失败（用于收集定义ID）: {}", e));
            error!("加载批次测试实例失败（用于收集定义ID）: {}", e);
        }
    }

    // 2. 删除该批次的所有测试实例
    match persistence_service.load_test_instances_by_batch(batch_id).await {
        Ok(instances) => {
            for instance in instances {
                match persistence_service.delete_test_instance(&instance.instance_id).await {
                    Ok(_) => {
                        deleted_instances_count += 1;
                    }
                    Err(e) => {
                        errors.push(format!("删除测试实例失败: {} - {}", instance.instance_id, e));
                        error!("删除测试实例失败: {} - {}", instance.instance_id, e);
                    }
                }
            }
        }
        Err(e) => {
            errors.push(format!("加载批次测试实例失败: {}", e));
            error!("加载批次测试实例失败: {}", e);
        }
    }

    // 3. 删除收集到的通道定义
    for definition_id in definition_ids_to_delete {
        // 注意：这里简化处理，假设每个批次的定义都是独立的
        // 在实际项目中可能需要更复杂的逻辑来检查引用关系
        match persistence_service.delete_channel_definition(&definition_id).await {
            Ok(_) => {
                deleted_definitions_count += 1;
            }
            Err(e) => {
                errors.push(format!("删除通道定义失败: {} - {}", definition_id, e));
                error!("删除通道定义失败: {} - {}", definition_id, e);
            }
        }
    }

    // 4. 最后删除批次信息
    match persistence_service.delete_batch_info(batch_id).await {
        Ok(_) => {
            // 删除成功
        }
        Err(e) => {
            errors.push(format!("删除批次信息失败: {}", e));
            error!("删除批次信息失败: {}", e);
        }
    }

    // === 额外：删除关联的全局功能测试状态 ===
    if let Some(station) = &batch_info.station_name {
        if let Err(e) = persistence_service.reset_global_function_test_statuses_by_station(station).await {
            error!("删除全局功能测试状态失败: {}", e);
        } else {
            // 同步清理缓存
            let mut guard = state.global_function_tests.lock().await;
            guard.retain(|s| &s.station_name != station);
        }
    }

    // 5. 从会话跟踪中移除该批次
    {
        let mut session_batch_ids_guard = state.session_batch_ids.lock().await;
        session_batch_ids_guard.retain(|id| id != batch_id);
    }

    let success = errors.is_empty();
    let message = if success {
        format!(
            "成功删除批次 '{}': 删除了{}个通道定义和{}个测试实例",
            batch_info.batch_name,
            deleted_definitions_count,
            deleted_instances_count
        )
    } else {
        format!(
            "批次删除部分成功: 删除了{}个通道定义和{}个测试实例，但有{}个操作失败",
            deleted_definitions_count,
            deleted_instances_count,
            errors.len()
        )
    };

    if !errors.is_empty() {
        error!("删除过程中的错误: {:?}", errors);
    }

    Ok(DeleteBatchResponse {
        success,
        message,
        deleted_definitions_count,
        deleted_instances_count,
    })
}

/// 创建测试批次并保存通道定义（用于前端测试数据生成）
#[tauri::command]
pub async fn create_test_batch_with_definitions_cmd(
    batch_info: TestBatchInfo,
    definitions: Vec<ChannelPointDefinition>,
    state: State<'_, AppState>
) -> Result<String, String> {
    info!("收到创建测试批次并保存通道定义请求: 批次={}, 定义数量={}",
          batch_info.batch_name, definitions.len());

    if definitions.is_empty() {
        return Err("没有提供任何通道定义".to_string());
    }

    // 第一步：保存通道定义到数据库
    let persistence_service = &state.persistence_service;

    let mut saved_count = 0;
    for definition in &definitions {
        match persistence_service.save_channel_definition(definition).await {
            Ok(_) => {
                saved_count += 1;
                debug!("成功保存通道定义: {}", definition.id);
            }
            Err(e) => {
                error!("保存通道定义失败: {} - {}", definition.id, e);
                // 继续保存其他定义，不中断整个过程
            }
        }
    }

    if saved_count == 0 {
        return Err("没有成功保存任何通道定义".to_string());
    }

    // 第二步：创建测试批次
    let db = persistence_service.get_database_connection();
    let allocation_service = BatchAllocationService::new(
        Arc::new(db.clone()), 
        state.channel_state_manager.clone()
    );

    // 第二步：创建测试批次，确保station_name被正确设置
    let mut updated_batch_info = batch_info.clone();

    // 🔧 修复：如果station_name为空，从第一个定义中获取
    if updated_batch_info.station_name.is_none() && !definitions.is_empty() {
        updated_batch_info.station_name = Some(definitions[0].station_name.clone());
        info!("🔧 从通道定义中获取站场名称: {:?}", updated_batch_info.station_name);
    }

    // 第三步：保存通道定义到数据库
    let mut saved_count = 0;
    let mut failed_count = 0;

    for definition in definitions.iter() {
        match state.persistence_service.save_channel_definition(definition).await {
            Ok(_) => {
                saved_count += 1;
            }
            Err(e) => {
                failed_count += 1;
                warn!("保存通道定义失败: {} - {}", definition.tag, e);
            }
        }
    }

    // 第四步：创建测试批次
    match allocation_service.create_test_batch(
        updated_batch_info.batch_name.clone(),
        updated_batch_info.product_model.clone(),
        updated_batch_info.operator_name.clone(),
        AllocationStrategy::Smart,
        None, // filter_criteria
    ).await {
        Ok(result) => {
            info!("测试批次创建完成: {} - {}个通道",
                  result.batch_info.batch_name, result.allocation_summary.total_channels);

            // 将批次ID添加到当前会话跟踪中
            {
                let mut session_batch_ids = state.session_batch_ids.lock().await;
                session_batch_ids.insert(result.batch_info.batch_id.clone());
            }

            Ok(result.batch_info.batch_id)
        }
        Err(e) => {
            error!("创建测试批次失败: {:?}", e);
            Err(e.to_string())
        }
    }
}

/// 一键导入Excel并创建批次
#[tauri::command]
pub async fn import_excel_and_create_batch_cmd(
    file_path: String,
    batch_name: String,
    product_model: Option<String>,
    operator_name: Option<String>,
    replace_existing: bool,
    allocation_strategy: String,
    state: State<'_, AppState>
) -> Result<ImportExcelAndCreateBatchResponse, String> {


    // === 清理旧会话缓存（导入新点表） ===
    state.channel_state_manager.clear_caches().await;
    {
        let mut ids = state.session_batch_ids.lock().await;
        ids.clear();
    }

    // 第一步：导入Excel数据
    let db = state.persistence_service.get_database_connection();
    let import_service = DataImportService::new(Arc::new(db.clone()));
    let import_result = match import_service.import_from_excel(&file_path, replace_existing).await {
        Ok(result) => {
            result
        }
        Err(e) => {
            error!("Excel导入失败: {:?}", e);
            return Err(e.to_string());
        }
    };

    // 如果导入失败，直接返回
    if !import_result.is_successful() {
        return Err("Excel导入失败，无法创建批次".to_string());
    }

    // 第二步：创建测试批次
    let strategy = match allocation_strategy.as_str() {
        "ByModuleType" => AllocationStrategy::ByModuleType,
        "ByStation" => AllocationStrategy::ByStation,
        "ByProductModel" => AllocationStrategy::ByProductModel,
        "Smart" => AllocationStrategy::Smart,
        _ => AllocationStrategy::Smart,
    };

    let allocation_service = BatchAllocationService::new(
        Arc::new(db.clone()), 
        state.channel_state_manager.clone()
    );
    let allocation_result = match allocation_service.create_test_batch(
        batch_name,
        product_model,
        operator_name,
        strategy,
        None, // 不使用过滤条件，使用所有导入的数据
    ).await {
        Ok(result) => {
            // 转换为命令层的 AllocationResult
            let allocation_result = AllocationResult {
                batches: vec![result.batch_info.clone()],
                allocated_instances: result.test_instances.clone(),
                allocation_summary: result.allocation_summary.clone(),
                channel_definitions: None, // 这里没有通道定义数据
            };

            // 将分配结果存储到状态管理器中
            match state.channel_state_manager.store_batch_allocation_result(allocation_result.clone()).await {
                Ok(_) => {
                    // 存储成功
                }
                Err(e) => {
                    error!("存储分配结果到状态管理器失败: {:?}", e);
                    // 不返回错误，因为数据已经保存到数据库了
                }
            }

            // ==============================
            // 将新创建的批次加入会话批次集合
            // ==============================
            {
                let mut ids = state.session_batch_ids.lock().await;
                for batch in &allocation_result.batches {
                    ids.insert(batch.batch_id.clone());
                }
            }

            allocation_result
        }
        Err(e) => {
            error!("创建测试批次失败: {:?}", e);
            return Err(e.to_string());
        }
    };

    Ok(ImportExcelAndCreateBatchResponse {
        success: true,
        message: format!("成功导入{}个通道定义并创建{}个测试批次",
                        import_result.successful_imports,
                        allocation_result.batches.len()),
        import_result,
        allocation_result,
    })
}

// ============================================================================
// 辅助函数 - 执行批次分配和状态管理
// ============================================================================

/// 执行批次分配的核心逻辑
///
/// 业务说明：
/// 这个函数负责协调整个批次分配流程：
/// 1. 保存通道定义到数据库
/// 2. 获取测试PLC配置
/// 3. 执行通道分配算法
/// 4. 转换结果格式
/// 
/// 调用链：
/// import_excel_and_prepare_batch_cmd -> execute_batch_allocation -> ChannelAllocationService
/// 
/// Rust知识点：
/// - async fn 异步函数
/// - &[T] 切片引用，避免所有权转移
/// - Result<T, E> 错误处理
async fn execute_batch_allocation(
    definitions: &[ChannelPointDefinition],
    args: &ImportExcelAndPrepareBatchCmdArgs,
    state: &AppState,
) -> Result<AllocationResult, String> {
    // 1. 首先保存通道定义到数据库
    // 业务说明：确保所有通道定义都持久化，即使后续分配失败也能保留数据
    let mut saved_definitions_count = 0;
    let mut failed_definitions_count = 0;

    for definition in definitions.iter() {
        match state.persistence_service.save_channel_definition(definition).await {
            Ok(_) => {
                saved_definitions_count += 1;
            }
            Err(e) => {
                failed_definitions_count += 1;
                warn!("保存通道定义失败: {} - {}", definition.tag, e);
            }
        }
    }

    // 2. 获取测试PLC配置
    // 业务说明：测试PLC配置定义了物理测试通道的能力和约束
    let test_plc_config = match state.test_plc_config_service.get_test_plc_config().await {
        Ok(config) => config,
        Err(e) => {
            error!("获取数据库测试PLC配置失败: {}", e);
            return Err(format!("获取测试PLC配置失败: {}，请先配置测试PLC", e));
        }
    };

    // 3. 执行通道分配
    // 业务说明：ChannelAllocationService实现了智能分配算法
    let allocation_service = ChannelAllocationService::new();
    let batch_allocation_result = allocation_service
        .allocate_channels(
            definitions.to_vec(),  // Rust知识点：to_vec() 从切片创建Vec
            test_plc_config,
            args.product_model.clone(),
            args.serial_number.clone(),
        )
        .await
        .map_err(|e| {
            error!("通道分配失败: {:?}", e);
            format!("通道分配失败: {}", e)
        })?;

    // 4. 转换为期望的AllocationResult格式
    // 业务说明：将服务层的结果转换为命令层的格式
    // Rust知识点：HashMap的get()返回Option<&V>，需要处理None情况
    let allocation_result = AllocationResult {
        batches: batch_allocation_result.batches,
        allocated_instances: batch_allocation_result.allocated_instances,
        allocation_summary: crate::application::services::batch_allocation_service::AllocationSummary {
            total_channels: batch_allocation_result.allocation_summary.total_definitions as usize,
            // 统计各模块类型的通道数
            ai_channels: batch_allocation_result.allocation_summary.by_module_type
                .get(&crate::models::ModuleType::AI)
                .map(|stats| stats.allocated_count as usize)
                .unwrap_or(0),
            ao_channels: batch_allocation_result.allocation_summary.by_module_type
                .get(&crate::models::ModuleType::AO)
                .map(|stats| stats.allocated_count as usize)
                .unwrap_or(0),
            di_channels: batch_allocation_result.allocation_summary.by_module_type
                .get(&crate::models::ModuleType::DI)
                .map(|stats| stats.allocated_count as usize)
                .unwrap_or(0),
            do_channels: batch_allocation_result.allocation_summary.by_module_type
                .get(&crate::models::ModuleType::DO)
                .map(|stats| stats.allocated_count as usize)
                .unwrap_or(0),
            stations: Vec::new(), // 可以根据需要填充
            estimated_test_duration_minutes: 30, // 默认估计时间
        },
        channel_definitions: Some(definitions.to_vec()),
    };

    Ok(allocation_result)
}

/// 将分配结果存储到状态管理器
///
/// 业务说明：
/// 负责将批次分配的结果存储到内存状态管理器中
/// 包括：
/// 1. 保存通道定义（如果还未保存）
/// 2. 存储分配结果到状态管理器
/// 3. 更新会话批次跟踪
/// 
/// Rust知识点：
/// - if let Some(ref x) 模式匹配，ref避免移动所有权
async fn store_allocation_to_state_manager(
    allocation_result: &AllocationResult,
    state: &AppState,
) -> Result<(), String> {
    // 保存通道定义到数据库
    if let Some(ref channel_definitions) = allocation_result.channel_definitions {
        let mut saved_count = 0;
        let mut failed_count = 0;

        for definition in channel_definitions.iter() {
            match state.persistence_service.save_channel_definition(definition).await {
                Ok(_) => {
                    saved_count += 1;
                }
                Err(e) => {
                    failed_count += 1;
                    error!("保存通道定义失败: ID={}, Tag={} - {}",
                        definition.id, definition.tag, e);
                }
            }
        }

        if failed_count > 0 {
            error!("通道定义保存完成: 成功={}, 失败={}", saved_count, failed_count);
        }
    } else {
        warn!("分配结果中没有通道定义数据！");
    }

    // 1. 存储批次分配结果到状态管理器
    // 业务说明：状态管理器维护测试状态的内存缓存
    match state.channel_state_manager.store_batch_allocation_result(allocation_result.clone()).await {
        Ok(_) => {
            // 存储成功
        }
        Err(e) => {
            error!("存储分配结果到状态管理器失败: {:?}", e);
            return Err(format!("存储分配结果到状态管理器失败: {}", e));
        }
    }

    // 2. 将批次ID添加到会话跟踪
    // 业务说明：会话跟踪用于区分不同用户的批次
    for batch in &allocation_result.batches {
        let mut session_batch_ids = state.session_batch_ids.lock().await;
        session_batch_ids.insert(batch.batch_id.clone());
    }
    Ok(())
}

// ============================================================================
// 会话恢复命令
// ============================================================================

/// 恢复会话命令
/// 
/// 业务说明：
/// 支持三种恢复方式：
/// 1. 传 `batch_id` → 自动推导其所属会话（同秒级 creation_time）
/// 2. 传 `session_key` → 直接使用指定会话
/// 3. 均为空 → 恢复最新会话
/// 
/// 会话概念：
/// - 同一秒内创建的批次属于同一个会话
/// - 会话键格式：YYYY-MM-DDTHH:MM:SS
/// 
/// 调用链：
/// 前端恢复会话 -> restore_session_cmd -> ChannelStateManager -> 恢复批次数据

/// 恢复会话命令（从数据库恢复批次数据到内存）
/// 
/// 业务说明：
/// 这是系统重启后恢复上次工作状态的核心功能
/// 支持三种恢复模式：
/// 1. 指定批次ID恢复：精确恢复特定批次
/// 2. 指定会话键恢复：恢复某个时间点的所有批次
/// 3. 自动恢复最新会话：恢复最近创建的批次
/// 
/// 参数说明：
/// - batch_id: 可选的批次ID，指定恢复特定批次
/// - session_key: 可选的会话键（时间戳），指定恢复特定时间点的批次
/// - state: 应用状态，包含持久化服务和状态管理器
/// 
/// Rust知识点：
/// - HashMap 用于组织会话数据
/// - Option::as_ref() 避免移动所有权
/// - remove() 从HashMap中取出值并获得所有权
/// 
/// 调用链：前端启动/刷新 -> restore_session_cmd -> ChannelStateManager -> PersistenceService
#[tauri::command]
pub async fn restore_session_cmd(
    batch_id: Option<String>,
    session_key: Option<String>,
    state: State<'_, AppState>
) -> Result<Vec<TestBatchInfo>, String> {
    // 1. 同步加载全局功能测试状态
    // 业务说明：全局功能测试（如报警测试）的状态需要首先恢复
    match state.persistence_service.load_all_global_function_test_statuses().await {
        Ok(list) => {
            // Rust知识点：使用Mutex guard确保线程安全地更新共享状态
            let mut guard = state.global_function_tests.lock().await;
            *guard = list;
        }
        Err(e) => {
            // 非致命错误，记录日志但继续执行
            error!("加载全局功能测试状态失败: {}", e);
        }
    }

    // 2. 清空 ChannelStateManager 缓存
    // 业务说明：确保从数据库加载最新数据，避免缓存不一致
    state.channel_state_manager.clear_caches().await;

    // 3. 恢复所有批次（先全部加载到缓存，便于后续使用）
    // 业务说明：从数据库加载所有批次及其关联的测试实例
    let all_batches = match state.channel_state_manager.restore_all_batches().await {
        Ok(list) => list,
        Err(e) => {
            error!("恢复会话失败: {}", e);
            return Err(format!("恢复会话失败: {}", e));
        }
    };

    // === 4. 根据 session_key 选择需要恢复的批次 ===
    // 组织到秒级 creation_time 作为会话分组
    // 业务说明：同一秒创建的批次属于同一个会话
    // Rust知识点：HashMap的entry API提供了便捷的插入或更新操作
    let mut session_map: std::collections::HashMap<String, Vec<TestBatchInfo>> = std::collections::HashMap::new();
    for b in &all_batches {
        // 生成两种格式的键，兼容不同的输入格式
        let ts_iso = crate::utils::time_utils::format_bj(b.creation_time, "%Y-%m-%dT%H:%M:%S");
        let ts_space = ts_iso.replace('T', " ");
        // 截取前19位确保格式统一（YYYY-MM-DDTHH:MM:SS）
        let key_iso = ts_iso.chars().take(19).collect::<String>();
        let key_space = ts_space.chars().take(19).collect::<String>();
        // 同一批次用两种键存储，提高命中率
        session_map.entry(key_iso).or_default().push(b.clone());
        session_map.entry(key_space).or_default().push(b.clone());
    }

    // ==== 选择目标会话键 ====
    // === 对 session_key 进行规范化，统一成 "YYYY-MM-DDTHH:MM:SS" 格式（无空格、19 位） ===
    let canonical_session_key = session_key.as_ref().map(|k| {
        // 替换空格为 T，截取前 19 位
        // Rust知识点：chars().take() 是Unicode安全的字符串截取方式
        let mut s = k.replace(' ', "T");
        if s.len() > 19 { s = s.chars().take(19).collect(); }
        s
    });

    log::info!("[RESTORE] 入参 batch_id={:?}, session_key={:?}, canonical={:?}", batch_id, session_key, canonical_session_key);

    // 确定要恢复的目标会话键
    // 业务逻辑优先级：batch_id > session_key > 最新会话
    let mut target_key = if let Some(id) = batch_id {
        // 根据 batch_id 找对应 creation_time 秒级键
        if let Some(batch) = all_batches.iter().find(|b| b.batch_id == id) {
            crate::utils::time_utils::format_bj(batch.creation_time, "%Y-%m-%dT%H:%M:%S")
        } else {
            warn!("未找到 batch_id={}, 回退到 session_key/最新会话", id);
            // 如果 batch_id 无效，则继续使用 session_key 或最新
            // Rust知识点：unwrap_or_else 提供延迟计算的默认值
            canonical_session_key.clone().unwrap_or_else(|| session_map.keys().max().cloned().unwrap_or_default())
        }
    } else if let Some(k) = canonical_session_key.clone() {
        // 若直接命中则使用
        if session_map.contains_key(&k) {
            k
        } else {
            // 尝试分钟级前缀匹配（前16位：YYYY-MM-DDTHH:MM）
            // 业务说明：支持模糊匹配，提高用户体验
            let minute_prefix: String = k.chars().take(16).collect();
            let mut candidate: Option<String> = None;
            for key in session_map.keys() {
                if key.starts_with(&minute_prefix) {
                    candidate = Some(key.clone());
                    break;
                }
            }
            if let Some(c) = candidate {
                log::warn!("[RESTORE] session_key 未精确命中，使用分钟级前缀匹配到 {}", c);
                c
            } else {
                k // 使用原始值，后面可能匹配不到而返回空数组
            }
        }
    } else {
        // 均为空 → 最新会话
        // Rust知识点：keys().max() 利用字符串的字典序找到最新时间
        session_map.keys().max().cloned().unwrap_or_default()
    };

    log::info!("[RESTORE] 最终 target_key = {}", target_key);

    // 从映射中移除并获取目标批次
    // Rust知识点：remove() 转移所有权，避免后续克隆
    let target_batches = session_map.remove(&target_key).unwrap_or_default();

    // 4. 更新 session_batch_ids（先清空再插入目标批次）
    // 业务说明：session_batch_ids 用于标识当前会话中的活跃批次
    {
        let mut ids = state.session_batch_ids.lock().await;
        ids.clear();
        for b in &target_batches {
            ids.insert(b.batch_id.clone());
        }
    }

    // 为前端增加北京时间字段，避免时区误差
    // 业务说明：确保前端显示正确的本地时间
    let target_batches: Vec<TestBatchInfo> = target_batches
        .into_iter()
        .map(|mut b| {
            let bj_str = crate::utils::time_utils::format_bj(b.creation_time, "%Y-%m-%d %H:%M:%S");
            // 在custom_data中存储北京时间
            b.custom_data.insert(
                "creation_time_bj".to_string(),
                bj_str.clone(),
            );
            // 同时更新import_time字段
            b.import_time = Some(bj_str);
            b
        })
        .collect();

    info!("恢复完成，会话键={}，加载 {} 个批次", target_key, target_batches.len());
    Ok(target_batches)
}
