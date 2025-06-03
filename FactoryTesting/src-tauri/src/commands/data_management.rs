/// 数据管理相关的Tauri命令
///
/// 包括Excel文件解析、批次创建等功能

use tauri::State;
use serde::{Deserialize, Serialize};
use crate::models::structs::{ChannelPointDefinition, TestBatchInfo};
use crate::services::application::data_import_service::{DataImportService, ImportResult};
use crate::services::application::batch_allocation_service::{BatchAllocationService, AllocationStrategy, AllocationResult as BatchAllocationResult};
use crate::services::infrastructure::excel::ExcelImporter;
use crate::services::channel_allocation_service::{ChannelAllocationService, IChannelAllocationService};
use crate::tauri_commands::AppState;
use log::{info, error, warn, debug};
use sea_orm::ActiveModelTrait;
use std::collections::HashMap;
use std::sync::Arc;

/// 通道分配结果（用于命令层）
#[derive(Debug, Clone, Serialize)]
pub struct AllocationResult {
    pub batches: Vec<TestBatchInfo>,
    pub allocated_instances: Vec<crate::models::structs::ChannelTestInstance>,
    pub allocation_summary: crate::services::application::batch_allocation_service::AllocationSummary,
    /// 🔧 修复：添加通道定义字段，用于保存到数据库
    pub channel_definitions: Option<Vec<ChannelPointDefinition>>,
}

/// Excel文件解析请求
#[derive(Debug, Deserialize)]
pub struct ParseExcelRequest {
    pub file_path: String,
}

/// Excel文件解析响应
#[derive(Debug, Serialize)]
pub struct ParseExcelResponse {
    pub success: bool,
    pub message: String,
    pub data: Option<Vec<ChannelPointDefinition>>,
    pub total_count: usize,
}

/// 创建批次请求
#[derive(Debug, Deserialize)]
pub struct CreateBatchRequest {
    pub file_name: String,
    pub file_path: String,
    pub preview_data: Vec<ChannelPointDefinition>,
    pub batch_info: BatchInfo,
}

/// 批次信息
#[derive(Debug, Deserialize)]
pub struct BatchInfo {
    pub product_model: String,
    pub serial_number: String,
    pub customer_name: Option<String>,
    pub operator_name: Option<String>,
}

/// 创建批次响应
#[derive(Debug, Serialize)]
pub struct CreateBatchResponse {
    pub success: bool,
    pub message: String,
    pub batch_id: Option<String>,
}

/// 解析Excel文件
///
/// # 参数
/// * `file_path` - Excel文件路径
/// * `state` - 应用状态
///
/// # 返回
/// * `Result<ParseExcelResponse, String>` - 解析结果
#[tauri::command]
pub async fn parse_excel_file(
    file_path: String,
    state: State<'_, AppState>
) -> Result<ParseExcelResponse, String> {
    info!("收到Excel文件解析请求: {}", file_path);

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
/// # 参数
/// * `batch_data` - 批次创建请求数据
/// * `state` - 应用状态
///
/// # 返回
/// * `Result<CreateBatchResponse, String>` - 创建结果
#[tauri::command]
pub async fn create_test_batch(
    batch_data: CreateBatchRequest,
    state: State<'_, AppState>
) -> Result<CreateBatchResponse, String> {
    info!("收到创建测试批次请求: 产品型号={}, 序列号={}",
          batch_data.batch_info.product_model,
          batch_data.batch_info.serial_number);

    // 创建测试批次信息
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
    let persistence_service = &state.persistence_service;

    // 保存批次信息
    match persistence_service.save_batch_info(&test_batch).await {
        Ok(_) => {
            info!("测试批次创建成功: {}", test_batch.batch_id);

            // 将批次ID添加到当前会话跟踪中
            {
                let mut session_batch_ids = state.session_batch_ids.lock().await;
                session_batch_ids.insert(test_batch.batch_id.clone());
                info!("批次 {} 已添加到当前会话跟踪", test_batch.batch_id);
            }

            // 保存通道定义
            let mut saved_count = 0;
            for definition in &batch_data.preview_data {
                match persistence_service.save_channel_definition(definition).await {
                    Ok(_) => saved_count += 1,
                    Err(e) => {
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
#[tauri::command]
pub async fn get_batch_list(
    state: State<'_, AppState>
) -> Result<Vec<TestBatchInfo>, String> {
    info!("获取批次列表 - 测试区域专用");

    let persistence_service = &state.persistence_service;

    // 获取当前会话中的批次ID列表
    let session_batch_ids = {
        let session_batch_ids_guard = state.session_batch_ids.lock().await;
        session_batch_ids_guard.clone()
    };

    info!("当前会话中有{}个批次", session_batch_ids.len());

    // 如果当前会话中没有批次，直接返回空列表
    if session_batch_ids.is_empty() {
        info!("当前会话中没有可用的测试批次，用户需要先导入Excel文件");
        return Ok(vec![]);
    }

    match persistence_service.load_all_batch_info().await {
        Ok(batches) => {
            // 只返回当前会话中创建的批次
            let current_session_batches: Vec<TestBatchInfo> = batches.into_iter()
                .filter(|batch| session_batch_ids.contains(&batch.batch_id))
                .collect();

            info!("成功获取{}个当前会话批次", current_session_batches.len());
            Ok(current_session_batches)
        }
        Err(e) => {
            error!("获取批次列表失败: {}", e);
            Err(format!("获取失败: {}", e))
        }
    }
}

/// 仪表盘批次信息 - 包含是否为当前会话的标识
#[derive(Debug, Serialize)]
pub struct DashboardBatchInfo {
    #[serde(flatten)]
    pub batch_info: TestBatchInfo,
    pub is_current_session: bool,  // 是否为当前会话的批次
    pub has_station_name: bool,    // 是否有站场名称（用于调试）
}

/// 获取仪表盘批次列表 - 从数据库获取所有批次，并标识当前会话批次
#[tauri::command]
pub async fn get_dashboard_batch_list(
    state: State<'_, AppState>
) -> Result<Vec<DashboardBatchInfo>, String> {
    info!("📊 获取仪表盘批次列表 - 包含所有历史批次");

    let persistence_service = &state.persistence_service;

    // 获取当前会话中的批次ID列表
    let session_batch_ids = {
        let session_batch_ids_guard = state.session_batch_ids.lock().await;
        session_batch_ids_guard.clone()
    };

    info!("📊 当前会话中有{}个批次", session_batch_ids.len());

    // 从数据库加载所有批次信息
    match persistence_service.load_all_batch_info().await {
        Ok(mut batches) => {
            info!("📊 从数据库成功获取{}个批次", batches.len());

            // 🔧 修复：检查并修复缺失的站场信息
            for batch in &mut batches {
                if batch.station_name.is_none() {
                    // 尝试从关联的测试实例中恢复站场信息
                    match persistence_service.load_test_instances_by_batch(&batch.batch_id).await {
                        Ok(instances) => {
                            if let Some(first_instance) = instances.first() {
                                // 从实例的变量描述或其他字段中尝试提取站场信息
                                if let Some(station_from_instance) = extract_station_from_instance(first_instance) {
                                    batch.station_name = Some(station_from_instance.clone());
                                    info!("📊 从测试实例恢复批次 {} 的站场信息: {}", batch.batch_name, station_from_instance);

                                    // 🔧 将恢复的站场信息保存回数据库
                                    if let Err(e) = persistence_service.save_batch_info(batch).await {
                                        warn!("📊 保存恢复的站场信息失败: {}", e);
                                    }
                                } else {
                                    warn!("📊 无法从测试实例中恢复批次 {} 的站场信息", batch.batch_name);
                                }
                            }
                        }
                        Err(e) => {
                            warn!("📊 加载批次 {} 的测试实例失败: {}", batch.batch_id, e);
                        }
                    }
                }
            }

            // 转换为仪表盘批次信息，并标识当前会话批次
            let dashboard_batches: Vec<DashboardBatchInfo> = batches.into_iter()
                .map(|batch| {
                    let is_current_session = session_batch_ids.contains(&batch.batch_id);
                    let has_station_name = batch.station_name.is_some();

                    // 🔍 调试：记录站场信息
                    if let Some(ref station_name) = batch.station_name {
                        info!("📊 批次 {} 的站场信息: {}", batch.batch_name, station_name);
                    } else {
                        warn!("📊 批次 {} 缺少站场信息", batch.batch_name);
                    }

                    DashboardBatchInfo {
                        batch_info: batch,
                        is_current_session,
                        has_station_name,
                    }
                })
                .collect();

            let current_session_count = dashboard_batches.iter()
                .filter(|b| b.is_current_session)
                .count();
            let historical_count = dashboard_batches.len() - current_session_count;

            info!("📊 仪表盘批次统计: 总计={}, 当前会话={}, 历史批次={}",
                  dashboard_batches.len(), current_session_count, historical_count);

            Ok(dashboard_batches)
        }
        Err(e) => {
            error!("📊 获取仪表盘批次列表失败: {}", e);
            Err(format!("获取失败: {}", e))
        }
    }
}

/// 从测试实例中提取站场信息的辅助函数
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
fn extract_station_from_description(description: &str) -> Option<String> {
    // 常见的站场名称模式
    let station_patterns = [
        "樟洋电厂", "华能电厂", "大唐电厂", "国电电厂", "中电投",
        "华电集团", "神华集团", "中煤集团", "国家电投"
    ];

    for pattern in &station_patterns {
        if description.contains(pattern) {
            return Some(pattern.to_string());
        }
    }

    None
}

/// 从标签中提取站场信息
fn extract_station_from_tag(tag: &str) -> Option<String> {
    // 如果标签包含站场信息的前缀，尝试提取
    if tag.len() > 2 {
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
#[tauri::command]
pub async fn get_batch_channel_definitions(
    batch_id: String,
    state: State<'_, AppState>
) -> Result<Vec<ChannelPointDefinition>, String> {
    info!("获取批次{}的通道定义", batch_id);

    let persistence_service = &state.persistence_service;

    match persistence_service.load_all_channel_definitions().await {
        Ok(definitions) => {
            // 这里应该根据batch_id过滤，但目前的持久化服务接口还不支持
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
#[derive(Debug, Deserialize)]
pub struct ImportExcelAndPrepareBatchCmdArgs {
    pub file_path_str: String,
    pub product_model: Option<String>,
    pub serial_number: Option<String>,
}

/// 导入Excel并准备批次的响应
#[derive(Debug, Serialize)]
pub struct ImportAndPrepareBatchResponse {
    pub batch_info: TestBatchInfo,
    pub instances: Vec<crate::models::ChannelTestInstance>,
}

/// 开始批次测试的参数
#[derive(Debug, Deserialize)]
pub struct StartTestsForBatchCmdArgs {
    pub batch_id: String,
}

/// 获取批次状态的参数
#[derive(Debug, Deserialize)]
pub struct GetBatchStatusCmdArgs {
    pub batch_id: String,
}

/// 批次详情载荷
#[derive(Debug, Serialize)]
pub struct BatchDetailsPayload {
    pub batch_info: TestBatchInfo,
    pub instances: Vec<crate::models::ChannelTestInstance>,
    pub definitions: Vec<ChannelPointDefinition>,
    pub allocation_summary: AllocationSummary,
    pub progress: BatchProgressInfo,
}

/// 批次进度信息
#[derive(Debug, Serialize)]
pub struct BatchProgressInfo {
    pub total_points: u32,
    pub tested_points: u32,
    pub passed_points: u32,
    pub failed_points: u32,
    pub skipped_points: u32,
}

/// 导入Excel文件并自动分配批次 - 这是主要的点表导入入口
#[tauri::command]
pub async fn import_excel_and_prepare_batch_cmd(
    args: ImportExcelAndPrepareBatchCmdArgs,
    state: State<'_, AppState>
) -> Result<ImportAndPrepareBatchResponse, String> {
    info!("🚀 [IMPORT_EXCEL] 收到导入Excel并准备批次请求: {}", args.file_path_str);
    info!("🚀 [IMPORT_EXCEL] 产品型号: {:?}, 序列号: {:?}", args.product_model, args.serial_number);

    // 1. 解析Excel文件
    info!("🔍 [IMPORT_EXCEL] 步骤1: 开始解析Excel文件");
    let definitions = match ExcelImporter::parse_excel_file(&args.file_path_str).await {
        Ok(defs) => {
            info!("✅ [IMPORT_EXCEL] Excel文件解析成功，获得{}个通道定义", defs.len());
            defs
        },
        Err(e) => {
            error!("❌ [IMPORT_EXCEL] Excel文件解析失败: {}", e);
            return Err(format!("Excel文件解析失败: {}", e));
        }
    };

    if definitions.is_empty() {
        error!("❌ [IMPORT_EXCEL] Excel文件中没有找到有效的通道定义");
        return Err("Excel文件中没有找到有效的通道定义".to_string());
    }

    // 2. 立即执行批次分配 - 这是关键步骤
    info!("🔄 [IMPORT_EXCEL] 步骤2: 开始执行自动批次分配");
    let allocation_result = match execute_batch_allocation(&definitions, &args, &state).await {
        Ok(result) => {
            info!("✅ [IMPORT_EXCEL] 批次分配成功，生成{}个批次", result.batches.len());
            // 🔍 调试：检查分配结果中的通道定义
            if let Some(ref channel_definitions) = result.channel_definitions {
                info!("🔍 [IMPORT_EXCEL] 分配结果包含{}个通道定义", channel_definitions.len());
            } else {
                warn!("⚠️ [IMPORT_EXCEL] 分配结果中没有通道定义数据！");
            }
            result
        },
        Err(e) => {
            error!("❌ [IMPORT_EXCEL] 批次分配失败: {}", e);
            return Err(format!("批次分配失败: {}", e));
        }
    };

    // 3. 将分配结果存储到状态管理器
    info!("💾 [IMPORT_EXCEL] 步骤3: 将批次数据存储到状态管理器");
    match store_allocation_to_state_manager(&allocation_result, &state).await {
        Ok(_) => {
            info!("✅ [IMPORT_EXCEL] 批次数据已成功存储到状态管理器");
        },
        Err(e) => {
            error!("❌ [IMPORT_EXCEL] 存储到状态管理器失败: {}", e);
            return Err(format!("存储批次数据失败: {}", e));
        }
    }

    // 4. 构建响应数据
    info!("🎉 [IMPORT_EXCEL] 步骤4: 构建响应数据");

    // 从分配结果中获取第一个批次作为主要批次信息
    let primary_batch = allocation_result.batches.first()
        .ok_or_else(|| "批次分配失败：没有生成任何批次".to_string())?;

    let response = ImportAndPrepareBatchResponse {
        batch_info: primary_batch.clone(),
        instances: allocation_result.allocated_instances.clone(),
    };

    info!("✅ [IMPORT_EXCEL] 导入Excel并准备批次完成");
    info!("✅ [IMPORT_EXCEL] 主要批次: {}", primary_batch.batch_id);
    info!("✅ [IMPORT_EXCEL] 总批次数: {}", allocation_result.batches.len());
    info!("✅ [IMPORT_EXCEL] 总实例数: {}", allocation_result.allocated_instances.len());

    Ok(response)
}

/// 开始批次测试
#[tauri::command]
pub async fn start_tests_for_batch_cmd(
    args: StartTestsForBatchCmdArgs,
    state: State<'_, AppState>
) -> Result<(), String> {
    info!("开始批次测试: {}", args.batch_id);

    state.test_coordination_service
        .start_batch_testing(&args.batch_id)
        .await
        .map_err(|e| {
            error!("开始批次测试失败: {}", e);
            e.to_string()
        })
}

/// 获取批次状态
#[tauri::command]
pub async fn get_batch_status_cmd(
    args: GetBatchStatusCmdArgs,
    state: State<'_, AppState>
) -> Result<BatchDetailsPayload, String> {
    let batch_id = args.batch_id;
    info!("📊 [GET_BATCH_STATUS] 获取批次状态: {}", batch_id);

    // 获取批次信息
    let batch_info = match state.persistence_service.load_batch_info(&batch_id).await {
        Ok(Some(info)) => {
            info!("✅ [GET_BATCH_STATUS] 成功获取批次信息: {}", info.batch_name);
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

    // 获取测试实例
    let instances = match state.persistence_service.load_test_instances_by_batch(&batch_id).await {
        Ok(instances) => {
            info!("✅ [GET_BATCH_STATUS] 成功获取测试实例: {} 个", instances.len());
            // 详细记录前几个实例
            for (index, instance) in instances.iter().take(5).enumerate() {
                info!("✅ [GET_BATCH_STATUS] 实例 {}: ID={}, 定义ID={}, 分配PLC通道={:?}, 状态={:?}",
                    index + 1, instance.instance_id, instance.definition_id,
                    instance.test_plc_channel_tag, instance.overall_status);
            }
            if instances.len() > 5 {
                info!("✅ [GET_BATCH_STATUS] ... 还有 {} 个实例", instances.len() - 5);
            }
            instances
        },
        Err(e) => {
            error!("❌ [GET_BATCH_STATUS] 获取测试实例失败: {}", e);
            return Err(format!("获取测试实例失败: {}", e));
        }
    };

    // 从状态管理器获取通道定义
    info!("🔍 [GET_BATCH_STATUS] 从状态管理器获取通道定义");
    let definitions = {
        let state_manager = &state.channel_state_manager;
        let instance_definition_ids: std::collections::HashSet<String> = instances
            .iter()
            .map(|instance| instance.definition_id.clone())
            .collect();

        let mut definitions = Vec::new();
        for definition_id in &instance_definition_ids {
            if let Some(definition) = state_manager.get_channel_definition(definition_id).await {
                info!("✅ [GET_BATCH_STATUS] 从状态管理器获取定义: ID={}, Tag={}", definition_id, definition.tag);
                definitions.push(definition);
            } else {
                warn!("⚠️ [GET_BATCH_STATUS] 状态管理器中未找到定义: {}", definition_id);
            }
        }

        info!("✅ [GET_BATCH_STATUS] 从状态管理器获取通道定义: {} 个", definitions.len());
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

    info!("✅ [GET_BATCH_STATUS] 批次状态获取完成");
    info!("✅ [GET_BATCH_STATUS] 总点位: {}, 已测试: {}, 通过: {}, 失败: {}",
          total_points, tested_points, passed_points, failed_points);

    Ok(BatchDetailsPayload {
        batch_info,
        instances,
        definitions,
        allocation_summary,
        progress,
    })
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

    // 3. 创建默认测试PLC配置
    let test_plc_config = create_default_test_plc_config().await?;

    // 4. 执行通道分配
    let persistence_service = &state.persistence_service;
    let db_conn = persistence_service.get_database_connection();
    let allocation_service = crate::services::application::batch_allocation_service::BatchAllocationService::new(Arc::new(db_conn));

    let result = allocation_service
        .create_test_batch(
            "自动分配批次".to_string(),
            product_model,
            None, // operator_name
            crate::services::application::batch_allocation_service::AllocationStrategy::Smart,
            None, // filter_criteria
        )
        .await
        .map_err(|e| e.to_string())?;

    log::info!(
        "通道分配完成，生成批次: {}，{} 个实例",
        result.batch_info.batch_name,
        result.test_instances.len()
    );

    // 转换为期望的返回格式
    Ok(AllocationResult {
        batches: vec![result.batch_info],
        allocated_instances: result.test_instances,
        allocation_summary: result.allocation_summary,
        channel_definitions: Some(definitions),
    })
}

/// 创建默认测试PLC配置的辅助函数
async fn create_default_test_plc_config() -> Result<crate::services::TestPlcConfig, String> {
    log::info!("创建默认测试PLC配置 - 基于正确分配数据的映射规则");
    let mut comparison_tables = Vec::new();

    // ===== 根据correct_allocation_data.json的正确分配规则创建测试PLC通道映射 =====

    // ===== AI测试需要的AO通道 =====
    // AI有源 → AO无源 (AO1_X)
    for channel in 1..=8 {
        comparison_tables.push(crate::services::ComparisonTable {
            channel_address: format!("AO1_{}", channel),
            communication_address: format!("AO1.{}", channel),
            channel_type: crate::models::ModuleType::AO,
            is_powered: false, // AO无源，用于测试AI有源
        });
    }

    // AI无源 → AO有源 (AO2_X)
    for channel in 1..=8 {
        comparison_tables.push(crate::services::ComparisonTable {
            channel_address: format!("AO2_{}", channel),
            communication_address: format!("AO2.{}", channel),
            channel_type: crate::models::ModuleType::AO,
            is_powered: true, // AO有源，用于测试AI无源
        });
    }

    // ===== AO测试需要的AI通道 =====
    // AO无源 → AI有源 (AI1_X)
    for channel in 1..=8 {
        comparison_tables.push(crate::services::ComparisonTable {
            channel_address: format!("AI1_{}", channel),
            communication_address: format!("AI1.{}", channel),
            channel_type: crate::models::ModuleType::AI,
            is_powered: true, // AI有源，用于测试AO无源
        });
    }

    // AO有源 → AI无源 (AI2_X) - 暂时不需要，因为在正确数据中AO有源数量很少

    // ===== DI测试需要的DO通道 =====
    // DI有源 → DO无源 (DO1_X)
    for channel in 1..=16 {
        comparison_tables.push(crate::services::ComparisonTable {
            channel_address: format!("DO1_{}", channel),
            communication_address: format!("DO1.{}", channel),
            channel_type: crate::models::ModuleType::DO,
            is_powered: false, // DO无源，用于测试DI有源
        });
    }

    // DI无源 → DO有源 (DO2_X)
    for channel in 1..=16 {
        comparison_tables.push(crate::services::ComparisonTable {
            channel_address: format!("DO2_{}", channel),
            communication_address: format!("DO2.{}", channel),
            channel_type: crate::models::ModuleType::DO,
            is_powered: true, // DO有源，用于测试DI无源
        });
    }

    // ===== DO测试需要的DI通道 =====
    // DO有源 → DI无源 (DI1_X)
    for channel in 1..=16 {
        comparison_tables.push(crate::services::ComparisonTable {
            channel_address: format!("DI1_{}", channel),
            communication_address: format!("DI1.{}", channel),
            channel_type: crate::models::ModuleType::DI,
            is_powered: false, // DI无源，用于测试DO有源
        });
    }

    // DO无源 → DI有源 (DI2_X)
    for channel in 1..=16 {
        comparison_tables.push(crate::services::ComparisonTable {
            channel_address: format!("DI2_{}", channel),
            communication_address: format!("DI2.{}", channel),
            channel_type: crate::models::ModuleType::DI,
            is_powered: true, // DI有源，用于测试DO无源
        });
    }

    log::info!("创建默认测试PLC配置完成，总通道数: {}", comparison_tables.len());
    log::info!("通道分布详情:");
    log::info!("  AO无源(测试AI有源): {} 个 -> {}", 8, "AO1_1..AO1_8");
    log::info!("  AO有源(测试AI无源): {} 个 -> {}", 8, "AO2_1..AO2_8");
    log::info!("  AI有源(测试AO无源): {} 个 -> {}", 8, "AI1_1..AI1_8");
    log::info!("  DO无源(测试DI有源): {} 个 -> {}", 16, "DO1_1..DO1_16");
    log::info!("  DO有源(测试DI无源): {} 个 -> {}", 16, "DO2_1..DO2_16");
    log::info!("  DI无源(测试DO有源): {} 个 -> {}", 16, "DI1_1..DI1_16");
    log::info!("  DI有源(测试DO无源): {} 个 -> {}", 16, "DI2_1..DI2_16");

    Ok(crate::services::TestPlcConfig {
        brand_type: "Micro850".to_string(),
        ip_address: "127.0.0.1".to_string(),
        comparison_tables,
    })
}

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

    // 第四步：保存通道定义
    let mut saved_count = 0;
    let mut errors = Vec::new();

    for definition in &definitions {
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
                info!("成功删除批次: {}", batch_id);
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
            log::warn!("[CreateBatchData] 获取数据库测试PLC配置失败: {}, 使用默认配置", e);
            // 如果无法获取数据库配置，则创建默认配置
            match create_default_test_plc_config().await {
                Ok(config) => config,
                Err(e) => {
                    error!("创建默认测试PLC配置失败: {}", e);
                    return Ok(CreateBatchAndPersistDataResponse {
                        success: false,
                        message: format!("创建默认测试PLC配置失败: {}", e),
                        batch_id: None,
                        all_batches: Vec::new(),
                        saved_definitions_count: 0,
                        created_instances_count: 0,
                    });
                }
            }
        }
    };

    // 调用通道分配服务
    let db_conn = state.persistence_service.get_database_connection();
    let allocation_service = crate::services::application::batch_allocation_service::BatchAllocationService::new(Arc::new(db_conn));

    let allocation_result = match allocation_service
        .create_test_batch(
            request.batch_info.batch_name.clone(),
            request.batch_info.product_model.clone(),
            request.batch_info.operator_name.clone(),
            crate::services::application::batch_allocation_service::AllocationStrategy::Smart,
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
        log::info!("[CreateBatchData] 添加批次到会话跟踪: {} ({})", batch.batch_name, batch.batch_id);
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

    // 详细记录所有生成的批次
    log::info!("[CreateBatchData] ===== 批次分配完成 =====");
    for (i, batch) in allocation_result.batches.iter().enumerate() {
        log::info!("[CreateBatchData] 批次{}: ID={}, 名称={}, 点位数={}",
            i + 1, batch.batch_id, batch.batch_name, batch.total_points);
    }

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
    info!("收到Excel导入数据库请求: {}", file_path);

    // 从持久化服务获取数据库连接
    let db = state.persistence_service.get_database_connection();
    let import_service = DataImportService::new(Arc::new(db.clone()));

    match import_service.import_from_excel(&file_path, replace_existing).await {
        Ok(result) => {
            info!("Excel导入完成: 成功{}个，失败{}个", result.successful_imports, result.failed_imports);
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
    let allocation_service = BatchAllocationService::new(Arc::new(db.clone()));

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
            info!("成功清空{}条通道定义数据", deleted_count);
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
    info!("🗑️ [DELETE_BATCH] 收到删除批次请求: {}", batch_id);

    let persistence_service = &state.persistence_service;

    // 检查批次是否存在
    let batch_info = match persistence_service.load_batch_info(batch_id).await {
        Ok(Some(info)) => {
            info!("✅ [DELETE_BATCH] 找到要删除的批次: {}", info.batch_name);
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
    info!("🗑️ [DELETE_BATCH] 开始收集需要删除的通道定义ID...");
    let mut definition_ids_to_delete = std::collections::HashSet::new();
    match persistence_service.load_test_instances_by_batch(batch_id).await {
        Ok(instances) => {
            for instance in &instances {
                definition_ids_to_delete.insert(instance.definition_id.clone());
            }
            info!("📊 [DELETE_BATCH] 从{}个测试实例中收集到{}个唯一的通道定义ID",
                instances.len(), definition_ids_to_delete.len());
        }
        Err(e) => {
            errors.push(format!("加载批次测试实例失败（用于收集定义ID）: {}", e));
            error!("❌ [DELETE_BATCH] 加载批次测试实例失败（用于收集定义ID）: {}", e);
        }
    }

    // 2. 删除该批次的所有测试实例
    info!("🗑️ [DELETE_BATCH] 开始删除批次的测试实例...");
    match persistence_service.load_test_instances_by_batch(batch_id).await {
        Ok(instances) => {
            info!("📊 [DELETE_BATCH] 找到{}个测试实例需要删除", instances.len());
            for instance in instances {
                match persistence_service.delete_test_instance(&instance.instance_id).await {
                    Ok(_) => {
                        deleted_instances_count += 1;
                        info!("✅ [DELETE_BATCH] 成功删除测试实例: {}", instance.instance_id);
                    }
                    Err(e) => {
                        errors.push(format!("删除测试实例失败: {} - {}", instance.instance_id, e));
                        error!("❌ [DELETE_BATCH] 删除测试实例失败: {} - {}", instance.instance_id, e);
                    }
                }
            }
        }
        Err(e) => {
            errors.push(format!("加载批次测试实例失败: {}", e));
            error!("❌ [DELETE_BATCH] 加载批次测试实例失败: {}", e);
        }
    }

    // 3. 删除收集到的通道定义
    info!("🗑️ [DELETE_BATCH] 开始删除批次的通道定义...");
    for definition_id in definition_ids_to_delete {
        // 注意：这里简化处理，假设每个批次的定义都是独立的
        // 在实际项目中可能需要更复杂的逻辑来检查引用关系
        match persistence_service.delete_channel_definition(&definition_id).await {
            Ok(_) => {
                deleted_definitions_count += 1;
                info!("✅ [DELETE_BATCH] 成功删除通道定义: {}", definition_id);
            }
            Err(e) => {
                errors.push(format!("删除通道定义失败: {} - {}", definition_id, e));
                error!("❌ [DELETE_BATCH] 删除通道定义失败: {} - {}", definition_id, e);
            }
        }
    }

    // 4. 最后删除批次信息
    info!("🗑️ [DELETE_BATCH] 开始删除批次信息...");
    match persistence_service.delete_batch_info(batch_id).await {
        Ok(_) => {
            info!("✅ [DELETE_BATCH] 成功删除批次信息: {}", batch_id);
        }
        Err(e) => {
            errors.push(format!("删除批次信息失败: {}", e));
            error!("❌ [DELETE_BATCH] 删除批次信息失败: {}", e);
        }
    }

    // 5. 从会话跟踪中移除该批次
    {
        let mut session_batch_ids_guard = state.session_batch_ids.lock().await;
        session_batch_ids_guard.retain(|id| id != batch_id);
        info!("✅ [DELETE_BATCH] 从会话跟踪中移除批次: {}", batch_id);
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
        error!("❌ [DELETE_BATCH] 删除过程中的错误: {:?}", errors);
    }

    info!("🎉 [DELETE_BATCH] 批次删除操作完成: {}", message);

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
    let allocation_service = BatchAllocationService::new(Arc::new(db.clone()));

    // 第二步：创建测试批次，确保station_name被正确设置
    let mut updated_batch_info = batch_info.clone();

    // 🔧 修复：如果station_name为空，从第一个定义中获取
    if updated_batch_info.station_name.is_none() && !definitions.is_empty() {
        updated_batch_info.station_name = Some(definitions[0].station_name.clone());
        info!("🔧 从通道定义中获取站场名称: {:?}", updated_batch_info.station_name);
    }

    // 第三步：保存通道定义到数据库
    info!("💾 开始保存{}个通道定义到数据库", definitions.len());
    let mut saved_count = 0;
    let mut failed_count = 0;

    for definition in definitions.iter() {
        match state.persistence_service.save_channel_definition(definition).await {
            Ok(_) => {
                saved_count += 1;
                debug!("✅ 成功保存通道定义: {}", definition.tag);
            }
            Err(e) => {
                failed_count += 1;
                warn!("⚠️ 保存通道定义失败: {} - {}", definition.tag, e);
            }
        }
    }

    info!("💾 通道定义保存完成: 成功={}, 失败={}", saved_count, failed_count);

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
    error!("🚀🚀🚀 [IMPORT_EXCEL_AND_CREATE_BATCH] ===== 新命令被调用了！！！ =====");
    error!("🚀🚀🚀 [IMPORT_EXCEL_AND_CREATE_BATCH] 收到一键导入Excel并创建批次请求");
    error!("🚀🚀🚀 [IMPORT_EXCEL_AND_CREATE_BATCH] 文件路径: {}", file_path);
    error!("🚀🚀🚀 [IMPORT_EXCEL_AND_CREATE_BATCH] 批次名称: {}", batch_name);
    error!("🚀🚀🚀 [IMPORT_EXCEL_AND_CREATE_BATCH] 产品型号: {:?}", product_model);
    error!("🚀🚀🚀 [IMPORT_EXCEL_AND_CREATE_BATCH] 操作员: {:?}", operator_name);
    error!("🚀🚀🚀 [IMPORT_EXCEL_AND_CREATE_BATCH] 替换现有数据: {}", replace_existing);
    error!("🚀🚀🚀 [IMPORT_EXCEL_AND_CREATE_BATCH] 分配策略: {}", allocation_strategy);

    // 第一步：导入Excel数据
    let db = state.persistence_service.get_database_connection();
    let import_service = DataImportService::new(Arc::new(db.clone()));
    let import_result = match import_service.import_from_excel(&file_path, replace_existing).await {
        Ok(result) => {
            info!("Excel导入完成: 成功{}个，失败{}个", result.successful_imports, result.failed_imports);
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

    let allocation_service = BatchAllocationService::new(Arc::new(db.clone()));
    let allocation_result = match allocation_service.create_test_batch(
        batch_name,
        product_model,
        operator_name,
        strategy,
        None, // 不使用过滤条件，使用所有导入的数据
    ).await {
        Ok(result) => {
            info!("🔥 [IMPORT_EXCEL_AND_CREATE_BATCH] 测试批次创建完成: {} - {}个通道",
                  result.batch_info.batch_name,
                  result.allocation_summary.total_channels);

            // 转换为命令层的 AllocationResult
            let allocation_result = AllocationResult {
                batches: vec![result.batch_info.clone()],
                allocated_instances: result.test_instances.clone(),
                allocation_summary: result.allocation_summary.clone(),
                channel_definitions: None, // 这里没有通道定义数据
            };

            // 🚀 重要：将分配结果存储到状态管理器中
            info!("🔥 [IMPORT_EXCEL_AND_CREATE_BATCH] 将分配结果存储到状态管理器");
            match state.channel_state_manager.store_batch_allocation_result(allocation_result.clone()).await {
                Ok(_) => {
                    info!("🔥 [IMPORT_EXCEL_AND_CREATE_BATCH] 成功存储分配结果到状态管理器");
                }
                Err(e) => {
                    error!("🔥 [IMPORT_EXCEL_AND_CREATE_BATCH] 存储分配结果到状态管理器失败: {:?}", e);
                    // 不返回错误，因为数据已经保存到数据库了
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
/// 这个函数使用已经验证过的通道分配服务来执行批次分配
async fn execute_batch_allocation(
    definitions: &[ChannelPointDefinition],
    args: &ImportExcelAndPrepareBatchCmdArgs,
    state: &AppState,
) -> Result<AllocationResult, String> {
    info!("🔄 [EXECUTE_BATCH_ALLOCATION] 开始执行批次分配");
    info!("🔄 [EXECUTE_BATCH_ALLOCATION] 输入通道定义数量: {}", definitions.len());

    // 1. 首先保存通道定义到数据库 - 这是关键步骤！
    info!("💾 [EXECUTE_BATCH_ALLOCATION] 步骤1: 保存通道定义到数据库");
    info!("💾 [EXECUTE_BATCH_ALLOCATION] 总定义数量: {}", definitions.len());

    let mut saved_definitions_count = 0;
    let mut failed_definitions_count = 0;

    for (index, definition) in definitions.iter().enumerate() {
        info!("💾 [EXECUTE_BATCH_ALLOCATION] 保存定义 {}/{}: ID={}, Tag={}, 通道标识={}",
            index + 1, definitions.len(), definition.id, definition.tag, definition.channel_tag_in_module);

        match state.persistence_service.save_channel_definition(definition).await {
            Ok(_) => {
                saved_definitions_count += 1;
                info!("✅ [EXECUTE_BATCH_ALLOCATION] 成功保存定义: {}", definition.tag);

                // 🔧 立即验证保存是否成功
                match state.persistence_service.load_channel_definition(&definition.id).await {
                    Ok(Some(loaded_def)) => {
                        info!("✅ [EXECUTE_BATCH_ALLOCATION] 立即验证成功: ID={}, Tag={}",
                            loaded_def.id, loaded_def.tag);
                    }
                    Ok(None) => {
                        error!("❌ [EXECUTE_BATCH_ALLOCATION] 立即验证失败: 保存后立即查询找不到定义 ID={}",
                            definition.id);
                    }
                    Err(e) => {
                        error!("❌ [EXECUTE_BATCH_ALLOCATION] 立即验证出错: ID={} - {}",
                            definition.id, e);
                    }
                }
            }
            Err(e) => {
                failed_definitions_count += 1;
                warn!("⚠️ [EXECUTE_BATCH_ALLOCATION] 保存通道定义失败: {} - {}", definition.tag, e);
                // 详细记录失败的定义信息
                warn!("⚠️ [EXECUTE_BATCH_ALLOCATION] 失败定义详情: ID={}, 通道标识={}, 模块类型={:?}, 变量名={}",
                    definition.id, definition.channel_tag_in_module, definition.module_type, definition.variable_name);
                // 继续处理其他定义，不因为单个定义失败而中断整个流程
            }
        }
    }

    info!("✅ [EXECUTE_BATCH_ALLOCATION] 数据库保存完成: 成功={}, 失败={}", saved_definitions_count, failed_definitions_count);

    // 2. 获取测试PLC配置
    info!("🔄 [EXECUTE_BATCH_ALLOCATION] 步骤2: 获取测试PLC配置");
    let test_plc_config = match state.test_plc_config_service.get_test_plc_config().await {
        Ok(config) => {
            info!("✅ [EXECUTE_BATCH_ALLOCATION] 成功获取测试PLC配置: {} 个通道映射",
                  config.comparison_tables.len());
            config
        }
        Err(e) => {
            warn!("⚠️ [EXECUTE_BATCH_ALLOCATION] 获取数据库测试PLC配置失败: {}, 使用默认配置", e);
            match create_default_test_plc_config().await {
                Ok(config) => config,
                Err(e) => {
                    error!("❌ [EXECUTE_BATCH_ALLOCATION] 创建默认测试PLC配置失败: {}", e);
                    return Err(format!("创建默认测试PLC配置失败: {}", e));
                }
            }
        }
    };

    // 3. 使用已验证的通道分配服务
    info!("🔄 [EXECUTE_BATCH_ALLOCATION] 步骤3: 调用通道分配服务");
    let allocation_service = ChannelAllocationService::new();

    // 4. 执行分配
    info!("🔄 [EXECUTE_BATCH_ALLOCATION] 步骤4: 执行通道分配");
    let batch_allocation_result = allocation_service
        .allocate_channels(
            definitions.to_vec(),
            test_plc_config,
            args.product_model.clone(),
            args.serial_number.clone(),
        )
        .await
        .map_err(|e| {
            error!("❌ [EXECUTE_BATCH_ALLOCATION] 通道分配失败: {:?}", e);
            format!("通道分配失败: {}", e)
        })?;

    info!("✅ [EXECUTE_BATCH_ALLOCATION] 批次分配成功");
    info!("✅ [EXECUTE_BATCH_ALLOCATION] 生成批次数量: {}", batch_allocation_result.batches.len());
    info!("✅ [EXECUTE_BATCH_ALLOCATION] 分配实例数量: {}", batch_allocation_result.allocated_instances.len());

    // 5. 记录详细的分配结果
    for (i, batch) in batch_allocation_result.batches.iter().enumerate() {
        info!("📊 [EXECUTE_BATCH_ALLOCATION] 批次{}: ID={}, 名称={}, 点位数={}",
              i + 1, batch.batch_id, batch.batch_name, batch.total_points);
    }

    // 6. 转换为期望的AllocationResult格式
    info!("🔧 [EXECUTE_BATCH_ALLOCATION] 创建AllocationResult，包含{}个通道定义", definitions.len());
    let allocation_result = AllocationResult {
        batches: batch_allocation_result.batches,
        allocated_instances: batch_allocation_result.allocated_instances,
        allocation_summary: crate::services::application::batch_allocation_service::AllocationSummary {
            total_channels: batch_allocation_result.allocation_summary.total_definitions as usize,
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
        /// 🔧 修复：包含通道定义，用于保存到数据库
        channel_definitions: Some(definitions.to_vec()),
    };

    // 🔍 验证AllocationResult中的通道定义
    if let Some(ref channel_definitions) = allocation_result.channel_definitions {
        info!("✅ [EXECUTE_BATCH_ALLOCATION] AllocationResult包含{}个通道定义", channel_definitions.len());
    } else {
        error!("❌ [EXECUTE_BATCH_ALLOCATION] AllocationResult中没有通道定义！");
    }

    Ok(allocation_result)
}

/// 将分配结果存储到状态管理器
///
/// 这个函数负责将批次分配的结果存储到内存状态管理器中
async fn store_allocation_to_state_manager(
    allocation_result: &AllocationResult,
    state: &AppState,
) -> Result<(), String> {
    info!("💾 [STORE_TO_STATE_MANAGER] 开始存储分配结果到状态管理器");
    info!("💾 [STORE_TO_STATE_MANAGER] 批次数量: {}", allocation_result.batches.len());
    info!("💾 [STORE_TO_STATE_MANAGER] 实例数量: {}", allocation_result.allocated_instances.len());

    // 🔍 调试：检查通道定义字段状态
    if let Some(ref channel_definitions) = allocation_result.channel_definitions {
        info!("🔍 [STORE_TO_STATE_MANAGER] 分配结果包含{}个通道定义", channel_definitions.len());

        // 🔧 直接在这里保存通道定义到数据库，避免clone问题
        info!("💾 [STORE_TO_STATE_MANAGER] 开始保存{}个通道定义到数据库", channel_definitions.len());
        let mut saved_count = 0;
        let mut failed_count = 0;

        for definition in channel_definitions.iter() {
            match state.persistence_service.save_channel_definition(definition).await {
                Ok(_) => {
                    saved_count += 1;
                }
                Err(e) => {
                    failed_count += 1;
                    error!("❌ [STORE_TO_STATE_MANAGER] 保存通道定义失败: ID={}, Tag={} - {}",
                        definition.id, definition.tag, e);
                }
            }
        }

        if failed_count == 0 {
            info!("✅ [STORE_TO_STATE_MANAGER] 通道定义保存完成: 成功保存{}个", saved_count);
        } else {
            error!("⚠️ [STORE_TO_STATE_MANAGER] 通道定义保存完成: 成功={}, 失败={}", saved_count, failed_count);
        }
    } else {
        warn!("⚠️ [STORE_TO_STATE_MANAGER] 分配结果中没有通道定义数据！");
    }

    // 1. 存储批次分配结果到状态管理器
    match state.channel_state_manager.store_batch_allocation_result(allocation_result.clone()).await {
        Ok(_) => {
            info!("✅ [STORE_TO_STATE_MANAGER] 成功存储分配结果到状态管理器");
        }
        Err(e) => {
            error!("❌ [STORE_TO_STATE_MANAGER] 存储分配结果到状态管理器失败: {:?}", e);
            return Err(format!("存储分配结果到状态管理器失败: {}", e));
        }
    }

    // 2. 将批次ID添加到会话跟踪
    for batch in &allocation_result.batches {
        let mut session_batch_ids = state.session_batch_ids.lock().await;
        session_batch_ids.insert(batch.batch_id.clone());
        info!("📝 [STORE_TO_STATE_MANAGER] 批次 {} 已添加到会话跟踪", batch.batch_id);
    }

    info!("✅ [STORE_TO_STATE_MANAGER] 所有数据已成功存储到状态管理器");
    Ok(())
}