/// 数据管理相关的Tauri命令
/// 
/// 包括Excel文件解析、批次创建等功能

use tauri::State;
use serde::{Deserialize, Serialize};
use crate::models::{ChannelPointDefinition, TestBatchInfo};
use crate::services::infrastructure::ExcelImporter;
use crate::services::{IChannelAllocationService, ChannelAllocationService};
use crate::tauri_commands::AppState;
use log::{info, error, warn};
use uuid;

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
    
    match persistence_service.load_all_batch_info().await {
        Ok(batches) => {
            info!("成功获取{}个批次", batches.len());
            Ok(batches)
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
        Ok(_) => info!("批次信息保存成功: {}", test_batch_info.batch_id),
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
    args: GetBatchStatusCmdArgs,
    state: State<'_, AppState>
) -> Result<BatchDetailsPayload, String> {
    info!("获取批次状态: {}", args.batch_id);
    
    // 获取批次信息
    let batch_info = match state.persistence_service.load_batch_info(&args.batch_id).await {
        Ok(Some(info)) => info,
        Ok(None) => return Err(format!("批次不存在: {}", args.batch_id)),
        Err(e) => {
            error!("获取批次信息失败: {}", e);
            return Err(format!("获取批次信息失败: {}", e));
        }
    };
    
    // 获取测试实例
    let instances = match state.persistence_service.load_test_instances_by_batch(&args.batch_id).await {
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
    // 根据正确的分配数据创建测试PLC通道映射
    let mut comparison_tables = Vec::new();
    
    // 添加AO通道 (用于测试AI) - 根据分配数据：AO1_1-AO1_6, AO2_1-AO2_8
    // AO1模块：6个通道
    for channel in 1..=6 {
        comparison_tables.push(crate::services::ComparisonTable {
            channel_address: format!("AO1_{}", channel),
            communication_address: format!("AO1.{}", channel),
            channel_type: crate::models::ModuleType::AO,
            is_powered: true, // AO有源
        });
    }
    
    // AO2模块：8个通道
    for channel in 1..=8 {
        comparison_tables.push(crate::services::ComparisonTable {
            channel_address: format!("AO2_{}", channel),
            communication_address: format!("AO2.{}", channel),
            channel_type: crate::models::ModuleType::AO,
            is_powered: true, // AO有源
        });
    }
    
    // 添加AI通道 (用于测试AO) - 根据分配数据：AI1_1-AI1_5
    for channel in 1..=5 {
        comparison_tables.push(crate::services::ComparisonTable {
            channel_address: format!("AI1_{}", channel),
            communication_address: format!("AI1.{}", channel),
            channel_type: crate::models::ModuleType::AI,
            is_powered: true, // AI有源
        });
    }
    
    // 添加DO通道 (用于测试DI) - 根据分配数据：DO2_1-DO2_16
    for channel in 1..=16 {
        comparison_tables.push(crate::services::ComparisonTable {
            channel_address: format!("DO2_{}", channel),
            communication_address: format!("DO2.{}", channel),
            channel_type: crate::models::ModuleType::DO,
            is_powered: true, // DO有源
        });
    }
    
    // 添加DI通道 (用于测试DO) - 根据分配数据：DI2_1-DI2_12
    for channel in 1..=12 {
        comparison_tables.push(crate::services::ComparisonTable {
            channel_address: format!("DI2_{}", channel),
            communication_address: format!("DI2.{}", channel),
            channel_type: crate::models::ModuleType::DI,
            is_powered: true, // DI有源
        });
    }
    
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