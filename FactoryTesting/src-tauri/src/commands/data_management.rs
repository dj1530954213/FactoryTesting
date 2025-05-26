/// 数据管理相关的Tauri命令
/// 
/// 包括Excel文件解析、批次创建等功能

use tauri::State;
use serde::{Deserialize, Serialize};
use crate::models::{ChannelPointDefinition, TestBatchInfo};
use crate::services::infrastructure::ExcelImporter;
use crate::tauri_commands::AppState;
use log::{info, error};

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