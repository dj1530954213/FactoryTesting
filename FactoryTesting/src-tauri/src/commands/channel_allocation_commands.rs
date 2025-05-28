use tauri::State;
use crate::services::{
    IChannelAllocationService, ChannelAllocationService,
    TestPlcConfig, BatchAllocationResult, ValidationResult
};
use crate::models::{ChannelPointDefinition, ChannelTestInstance};
use crate::error::AppError;
use std::sync::Arc;
use tokio::sync::Mutex;

/// 应用状态，包含通道分配服务
pub struct AppState {
    pub channel_allocation_service: Arc<Mutex<dyn IChannelAllocationService>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            channel_allocation_service: Arc::new(Mutex::new(ChannelAllocationService::new())),
        }
    }
}

/// 分配通道命令
/// 为通道定义分配测试批次和测试PLC通道
#[tauri::command]
pub async fn allocate_channels_cmd(
    definitions: Vec<ChannelPointDefinition>,
    test_plc_config: TestPlcConfig,
    product_model: Option<String>,
    serial_number: Option<String>,
    state: State<'_, AppState>,
) -> Result<BatchAllocationResult, String> {
    log::info!("收到通道分配请求，定义数量: {}", definitions.len());
    
    let service = state.channel_allocation_service.lock().await;
    
    service
        .allocate_channels(definitions, test_plc_config, product_model, serial_number)
        .await
        .map_err(|e| {
            log::error!("通道分配失败: {}", e);
            e.to_string()
        })
}

/// 获取批次实例命令
/// 获取指定批次的所有通道实例
#[tauri::command]
pub async fn get_batch_instances_cmd(
    batch_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<ChannelTestInstance>, String> {
    log::info!("获取批次实例: {}", batch_id);
    
    let service = state.channel_allocation_service.lock().await;
    
    service
        .get_batch_instances(&batch_id)
        .await
        .map_err(|e| {
            log::error!("获取批次实例失败: {}", e);
            e.to_string()
        })
}

/// 清除所有分配命令
/// 清除所有通道的分配信息
#[tauri::command]
pub async fn clear_all_allocations_cmd(
    instances: Vec<ChannelTestInstance>,
    state: State<'_, AppState>,
) -> Result<Vec<ChannelTestInstance>, String> {
    log::info!("清除所有通道分配，实例数: {}", instances.len());
    
    let service = state.channel_allocation_service.lock().await;
    
    service
        .clear_all_allocations(instances)
        .await
        .map_err(|e| {
            log::error!("清除分配失败: {}", e);
            e.to_string()
        })
}

/// 验证分配命令
/// 验证通道分配的有效性
#[tauri::command]
pub async fn validate_allocations_cmd(
    instances: Vec<ChannelTestInstance>,
    state: State<'_, AppState>,
) -> Result<ValidationResult, String> {
    log::info!("验证通道分配，实例数: {}", instances.len());
    
    let service = state.channel_allocation_service.lock().await;
    
    service
        .validate_allocations(&instances)
        .await
        .map_err(|e| {
            log::error!("验证分配失败: {}", e);
            e.to_string()
        })
}

/// 创建默认测试PLC配置命令
/// 创建一个包含默认通道映射的测试PLC配置
#[tauri::command]
pub async fn create_default_test_plc_config_cmd() -> Result<TestPlcConfig, String> {
    log::info!("创建默认测试PLC配置");
    
    // 创建默认的通道映射表
    let mut comparison_tables = Vec::new();
    
    // 添加AO通道 (用于测试AI)
    for module in 1..=2 {
        for channel in 1..=8 {
            comparison_tables.push(crate::services::ComparisonTable {
                channel_address: format!("AO{}_{}", module, channel),
                communication_address: format!("AO{}.{}", module, channel),
                channel_type: crate::models::ModuleType::AO,
                is_powered: channel % 2 == 1, // 奇数通道为有源，偶数通道为无源
            });
        }
    }
    
    // 添加AI通道 (用于测试AO)
    for module in 1..=2 {
        for channel in 1..=8 {
            comparison_tables.push(crate::services::ComparisonTable {
                channel_address: format!("AI{}_{}", module, channel),
                communication_address: format!("AI{}.{}", module, channel),
                channel_type: crate::models::ModuleType::AI,
                is_powered: channel % 2 == 1, // 奇数通道为有源，偶数通道为无源
            });
        }
    }
    
    // 添加DO通道 (用于测试DI)
    for module in 1..=4 {
        for channel in 1..=8 {
            comparison_tables.push(crate::services::ComparisonTable {
                channel_address: format!("DO{}_{}", module, channel),
                communication_address: format!("DO{}.{}", module, channel),
                channel_type: crate::models::ModuleType::DO,
                is_powered: channel % 2 == 1, // 奇数通道为有源，偶数通道为无源
            });
        }
    }
    
    // 添加DI通道 (用于测试DO)
    for module in 1..=4 {
        for channel in 1..=8 {
            comparison_tables.push(crate::services::ComparisonTable {
                channel_address: format!("DI{}_{}", module, channel),
                communication_address: format!("DI{}.{}", module, channel),
                channel_type: crate::models::ModuleType::DI,
                is_powered: channel % 2 == 1, // 奇数通道为有源，偶数通道为无源
            });
        }
    }
    
    let config = TestPlcConfig {
        brand_type: "Micro850".to_string(),
        ip_address: "127.0.0.1".to_string(),
        comparison_tables,
    };
    
    log::info!("默认测试PLC配置创建完成，通道数: {}", config.comparison_tables.len());
    Ok(config)
} 