/// 数据管理相关的Tauri命令
/// 
/// 包括Excel文件解析、批次创建等功能

use tauri::State;
use serde::{Deserialize, Serialize};
use crate::models::{ChannelPointDefinition, TestBatchInfo};
use crate::services::IChannelAllocationService;
use crate::services::infrastructure::ExcelImporter;
use crate::AppState;
use log::{info, error, warn};

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

/// 获取批次列表
#[tauri::command]
pub async fn get_batch_list(
    state: State<'_, AppState>
) -> Result<Vec<TestBatchInfo>, String> {
    info!("获取批次列表");
    
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

/// 导入Excel文件并准备批次
#[tauri::command]
pub async fn import_excel_and_prepare_batch_cmd(
    args: ImportExcelAndPrepareBatchCmdArgs,
    state: State<'_, AppState>
) -> Result<ImportAndPrepareBatchResponse, String> {
    info!("收到导入Excel并准备批次请求: {}", args.file_path_str);
    
    // 1. 解析Excel文件
    let definitions = match ExcelImporter::parse_excel_file(&args.file_path_str).await {
        Ok(defs) => defs,
        Err(e) => {
            error!("Excel文件解析失败: {}", e);
            return Err(format!("Excel文件解析失败: {}", e));
        }
    };
    
    // 2. 创建测试批次
    let mut test_batch_info = TestBatchInfo::new(
        args.product_model.clone(),
        args.serial_number.clone(),
    );
    test_batch_info.total_points = definitions.len() as u32;
    
    // 3. 保存批次信息
    match state.persistence_service.save_batch_info(&test_batch_info).await {
        Ok(_) => {
            info!("批次信息保存成功: {}", test_batch_info.batch_id);
            
            // 将批次ID添加到当前会话跟踪中
            {
                let mut session_batch_ids = state.session_batch_ids.lock().await;
                session_batch_ids.insert(test_batch_info.batch_id.clone());
                info!("批次 {} 已添加到当前会话跟踪", test_batch_info.batch_id);
            }
        }
        Err(e) => {
            error!("保存批次信息失败: {}", e);
            return Err(format!("保存批次信息失败: {}", e));
        }
    }
    
    // 4. 为每个定义创建测试实例
    let mut instances = Vec::new();
    for definition in &definitions {
        // 保存通道定义
        if let Err(e) = state.persistence_service.save_channel_definition(definition).await {
            error!("保存通道定义失败: {} - {}", definition.tag, e);
        }
        
        // 创建测试实例
        match state.channel_state_manager.create_test_instance(
            &definition.id,
            &test_batch_info.batch_id
        ).await {
            Ok(instance) => {
                // 保存测试实例
                if let Err(e) = state.persistence_service.save_test_instance(&instance).await {
                    error!("保存测试实例失败: {} - {}", instance.instance_id, e);
                }
                instances.push(instance);
            }
            Err(e) => {
                error!("创建测试实例失败: {} - {}", definition.tag, e);
            }
        }
    }
    
    info!("成功创建批次，包含{}个测试实例", instances.len());
    
    Ok(ImportAndPrepareBatchResponse {
        batch_info: test_batch_info,
        instances,
    })
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
    batch_id: String,
    state: State<'_, AppState>
) -> Result<BatchDetailsPayload, String> {
    info!("获取批次状态: {}", batch_id);
    
    // 获取批次信息
    let batch_info = match state.persistence_service.load_batch_info(&batch_id).await {
        Ok(Some(info)) => info,
        Ok(None) => return Err(format!("批次不存在: {}", batch_id)),
        Err(e) => {
            error!("获取批次信息失败: {}", e);
            return Err(format!("获取批次信息失败: {}", e));
        }
    };
    
    // 获取测试实例
    let instances = match state.persistence_service.load_test_instances_by_batch(&batch_id).await {
        Ok(instances) => instances,
        Err(e) => {
            error!("获取测试实例失败: {}", e);
            return Err(format!("获取测试实例失败: {}", e));
        }
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
    
    Ok(BatchDetailsPayload {
        batch_info,
        instances,
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
) -> Result<crate::services::BatchAllocationResult, String> {
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
    let allocation_service = crate::services::ChannelAllocationService::new();
    let result = allocation_service
        .allocate_channels(definitions, test_plc_config, product_model, serial_number)
        .await
        .map_err(|e| e.to_string())?;
    
    log::info!(
        "通道分配完成，生成 {} 个批次，{} 个实例",
        result.batches.len(),
        result.allocated_instances.len()
    );
    
    Ok(result)
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
    
    // ===== 使用通道分配服务进行正确的批次分配 =====
    log::info!("[CreateBatchData] ===== 开始使用通道分配服务 =====");
    log::info!("[CreateBatchData] 输入: {} 个通道定义", request.definitions.len());
    
    // ===== 修复：从数据库获取真实的测试PLC配置 =====
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
    let allocation_result = match state.channel_allocation_service
        .allocate_channels(
            request.definitions.clone(),
            test_plc_config,
            request.batch_info.product_model.clone(),
            request.batch_info.serial_number.clone(),
        )
        .await
    {
        Ok(result) => {
            log::info!("[CreateBatchData] 通道分配成功: {} 个批次, {} 个实例", 
                result.batches.len(), result.allocated_instances.len());
            result
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
    
    let persistence_service = &state.persistence_service;
    
    // 第一步：保存所有生成的批次信息
    let mut saved_batches_count = 0;
    for batch in &allocation_result.batches {
        match persistence_service.save_batch_info(batch).await {
            Ok(_) => {
                log::info!("[CreateBatchData] 成功保存批次: {} ({})", batch.batch_name, batch.batch_id);
                saved_batches_count += 1;
                
                // 将批次ID添加到当前会话跟踪中
                {
                    let mut session_batch_ids = state.session_batch_ids.lock().await;
                    session_batch_ids.insert(batch.batch_id.clone());
                }
            }
            Err(e) => {
                error!("保存批次失败: {} - {}", batch.batch_id, e);
            }
        }
    }
    
    // 第二步：保存通道定义
    let mut saved_definitions_count = 0;
    let mut definition_errors = Vec::new();
    
    for definition in &request.definitions {
        match persistence_service.save_channel_definition(definition).await {
            Ok(_) => saved_definitions_count += 1,
            Err(e) => {
                let error_msg = format!("保存通道定义失败: {} - {}", definition.tag, e);
                error!("{}", error_msg);
                definition_errors.push(error_msg);
            }
        }
    }
    
    // 第三步：保存分配的测试实例
    let mut created_instances_count = 0;
    let mut instance_errors = Vec::new();
    
    for instance in &allocation_result.allocated_instances {
        match persistence_service.save_test_instance(instance).await {
            Ok(_) => created_instances_count += 1,
            Err(e) => {
                let error_msg = format!("保存测试实例失败: {} - {}", instance.instance_id, e);
                error!("{}", error_msg);
                instance_errors.push(error_msg);
            }
        }
    }
    
    // 第四步：生成结果消息
    let all_errors = [definition_errors, instance_errors].concat();
    let success = saved_batches_count > 0 && saved_definitions_count > 0 && created_instances_count > 0;
    
    let message = if success {
        if all_errors.is_empty() {
            format!("成功创建{}个批次，持久化{}个通道定义，创建{}个测试实例", 
                   saved_batches_count, saved_definitions_count, created_instances_count)
        } else {
            format!("批次创建成功，生成{}个批次，保存{}个通道定义，创建{}个测试实例，{}个操作失败", 
                   saved_batches_count, saved_definitions_count, created_instances_count, all_errors.len())
        }
    } else {
        format!("批次创建失败。错误: {}", all_errors.join("; "))
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
        saved_definitions_count,
        created_instances_count,
    })
} 