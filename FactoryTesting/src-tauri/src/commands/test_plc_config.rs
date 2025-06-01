// 文件: FactoryTesting/src-tauri/src/commands/test_plc_config.rs
// 详细注释：测试PLC配置相关的Tauri命令

use tauri::State;
use crate::tauri_commands::AppState;
use crate::models::test_plc_config::*;
use crate::utils::error::AppResult;
use crate::models::entities::test_plc_channel_config;
use chrono::Utc;
use uuid;
use log::{info, debug};

/// 获取测试PLC通道配置
#[tauri::command]
pub async fn get_test_plc_channels_cmd(
    channel_type_filter: Option<TestPlcChannelType>,
    enabled_only: Option<bool>,
    state: State<'_, AppState>
) -> Result<Vec<TestPlcChannelConfig>, String> {
    debug!("获取测试PLC通道配置命令");
    
    let request = GetTestPlcChannelsRequest {
        channel_type_filter,
        enabled_only,
    };
    
    match state.test_plc_config_service.get_test_plc_channels(request).await {
        Ok(channels) => {
            info!("成功获取 {} 个测试PLC通道配置", channels.len());
            Ok(channels)
        }
        Err(e) => {
            log::error!("获取测试PLC通道配置失败: {}", e);
            Err(format!("获取测试PLC通道配置失败: {}", e))
        }
    }
}

/// 保存测试PLC通道配置
#[tauri::command]
pub async fn save_test_plc_channel_cmd(
    channel: TestPlcChannelConfig,
    state: State<'_, AppState>
) -> Result<TestPlcChannelConfig, String> {
    debug!("保存测试PLC通道配置命令: {:?}", channel.channel_address);
    
    // 添加详细的输入验证日志
    info!("接收到通道配置数据: ID={:?}, 地址={}, 类型={:?}, 通讯地址={}, 供电类型={}", 
          channel.id, channel.channel_address, channel.channel_type, 
          channel.communication_address, channel.power_supply_type);
    
    match state.test_plc_config_service.save_test_plc_channel(channel).await {
        Ok(saved_channel) => {
            info!("成功保存测试PLC通道配置: {}", saved_channel.channel_address);
            Ok(saved_channel)
        }
        Err(e) => {
            log::error!("保存测试PLC通道配置失败: {}", e);
            log::error!("错误详情: {:?}", e);
            
            // 确保错误信息不会导致panic
            let error_message = format!("保存测试PLC通道配置失败: {}", e);
            Err(error_message)
        }
    }
}

/// 删除测试PLC通道配置
#[tauri::command]
pub async fn delete_test_plc_channel_cmd(
    channel_id: String,
    state: State<'_, AppState>
) -> Result<(), String> {
    debug!("删除测试PLC通道配置命令: {}", channel_id);
    
    match state.test_plc_config_service.delete_test_plc_channel(&channel_id).await {
        Ok(_) => {
            info!("成功删除测试PLC通道配置: {}", channel_id);
            Ok(())
        }
        Err(e) => {
            log::error!("删除测试PLC通道配置失败: {}", e);
            Err(format!("删除测试PLC通道配置失败: {}", e))
        }
    }
}

/// 获取PLC连接配置
#[tauri::command]
pub async fn get_plc_connections_cmd(
    state: State<'_, AppState>
) -> Result<Vec<PlcConnectionConfig>, String> {
    debug!("获取PLC连接配置命令");
    
    match state.test_plc_config_service.get_plc_connections().await {
        Ok(connections) => {
            info!("成功获取 {} 个PLC连接配置", connections.len());
            Ok(connections)
        }
        Err(e) => {
            log::error!("获取PLC连接配置失败: {}", e);
            Err(format!("获取PLC连接配置失败: {}", e))
        }
    }
}

/// 保存PLC连接配置
#[tauri::command]
pub async fn save_plc_connection_cmd(
    connection: PlcConnectionConfig,
    state: State<'_, AppState>
) -> Result<PlcConnectionConfig, String> {
    debug!("保存PLC连接配置命令: {:?}", connection.name);
    
    match state.test_plc_config_service.save_plc_connection(connection).await {
        Ok(saved_connection) => {
            info!("成功保存PLC连接配置: {}", saved_connection.name);
            Ok(saved_connection)
        }
        Err(e) => {
            log::error!("保存PLC连接配置失败: {}", e);
            Err(format!("保存PLC连接配置失败: {}", e))
        }
    }
}

/// 测试PLC连接
#[tauri::command]
pub async fn test_plc_connection_cmd(
    connection_id: String,
    state: State<'_, AppState>
) -> Result<TestPlcConnectionResponse, String> {
    debug!("测试PLC连接命令: {}", connection_id);
    
    match state.test_plc_config_service.test_plc_connection(&connection_id).await {
        Ok(response) => {
            info!("PLC连接测试完成: {} - {}", connection_id, if response.success { "成功" } else { "失败" });
            Ok(response)
        }
        Err(e) => {
            log::error!("测试PLC连接失败: {}", e);
            Err(format!("测试PLC连接失败: {}", e))
        }
    }
}

/// 获取通道映射配置
#[tauri::command]
pub async fn get_channel_mappings_cmd(
    state: State<'_, AppState>
) -> Result<Vec<ChannelMappingConfig>, String> {
    debug!("获取通道映射配置命令");
    
    match state.test_plc_config_service.get_channel_mappings().await {
        Ok(mappings) => {
            info!("成功获取 {} 个通道映射配置", mappings.len());
            Ok(mappings)
        }
        Err(e) => {
            log::error!("获取通道映射配置失败: {}", e);
            Err(format!("获取通道映射配置失败: {}", e))
        }
    }
}

/// 自动生成通道映射
#[tauri::command]
pub async fn generate_channel_mappings_cmd(
    request: GenerateChannelMappingsRequest,
    state: State<'_, AppState>
) -> Result<GenerateChannelMappingsResponse, String> {
    debug!("自动生成通道映射命令，策略: {:?}", request.strategy);
    
    match state.test_plc_config_service.generate_channel_mappings(request).await {
        Ok(response) => {
            info!("成功生成 {} 个通道映射", response.mappings.len());
            Ok(response)
        }
        Err(e) => {
            log::error!("自动生成通道映射失败: {}", e);
            Err(format!("自动生成通道映射失败: {}", e))
        }
    }
}

/// 初始化默认测试PLC通道配置
#[tauri::command]
pub async fn initialize_default_test_plc_channels_cmd(
    state: State<'_, AppState>
) -> Result<(), String> {
    debug!("初始化默认测试PLC通道配置命令");

    match state.test_plc_config_service.initialize_default_test_plc_channels().await {
        Ok(_) => {
            info!("成功初始化默认测试PLC通道配置");
            Ok(())
        }
        Err(e) => {
            log::error!("初始化默认测试PLC通道配置失败: {}", e);
            Err(format!("初始化默认测试PLC通道配置失败: {}", e))
        }
    }
}

/// 恢复88个默认测试PLC通道配置
#[tauri::command]
pub async fn restore_default_test_plc_channels_cmd(
    state: State<'_, AppState>
) -> Result<String, String> {
    info!("开始恢复88个默认测试PLC通道配置");

    // 创建88个测试PLC通道配置
    let configs = create_88_test_plc_channels();

    info!("创建了 {} 个测试PLC通道配置", configs.len());

    // 批量保存新配置
    let mut saved_count = 0;
    for config in configs {
        match state.test_plc_config_service.save_test_plc_channel(config.clone()).await {
            Ok(_) => {
                saved_count += 1;
                debug!("已保存通道配置: {}", config.channel_address);
            }
            Err(e) => {
                log::error!("保存通道配置失败 {}: {}", config.channel_address, e);
                return Err(format!("保存通道配置失败 {}: {}", config.channel_address, e));
            }
        }
    }

    let result_msg = format!("成功恢复 {} 个测试PLC通道配置", saved_count);
    info!("{}", result_msg);

    Ok(result_msg)
}

/// 创建88个测试PLC通道配置（基于重构前的真实数据）
fn create_88_test_plc_channels() -> Vec<TestPlcChannelConfig> {
    let mut configs = Vec::new();
    let now = Utc::now();

    // AI通道 (8个有源)
    for i in 1..=8 {
        configs.push(TestPlcChannelConfig {
            id: Some(uuid::Uuid::new_v4().to_string()),
            channel_address: format!("AI1_{}", i),
            channel_type: TestPlcChannelType::AI,
            communication_address: format!("{}", 40100 + i * 2 - 1),
            power_supply_type: "24V DC".to_string(),
            description: Some(format!("模拟量输入通道 {}", i)),
            is_enabled: true,
            created_at: Some(now),
            updated_at: Some(now),
        });
    }

    // AO通道 (8个有源)
    for i in 1..=8 {
        configs.push(TestPlcChannelConfig {
            id: Some(uuid::Uuid::new_v4().to_string()),
            channel_address: format!("AO1_{}", i),
            channel_type: TestPlcChannelType::AO,
            communication_address: format!("{}", 40200 + i * 2 - 1),
            power_supply_type: "24V DC".to_string(),
            description: Some(format!("模拟量输出通道 {}", i)),
            is_enabled: true,
            created_at: Some(now),
            updated_at: Some(now),
        });
    }

    // AO2通道 (8个无源)
    for i in 1..=8 {
        configs.push(TestPlcChannelConfig {
            id: Some(uuid::Uuid::new_v4().to_string()),
            channel_address: format!("AO2_{}", i),
            channel_type: TestPlcChannelType::AONone,
            communication_address: format!("{}", 40300 + i * 2 - 1),
            power_supply_type: "无源".to_string(),
            description: Some(format!("模拟量输出通道(无源) {}", i)),
            is_enabled: true,
            created_at: Some(now),
            updated_at: Some(now),
        });
    }

    // DI1通道 (16个有源)
    for i in 1..=16 {
        configs.push(TestPlcChannelConfig {
            id: Some(uuid::Uuid::new_v4().to_string()),
            channel_address: format!("DI1_{}", i),
            channel_type: TestPlcChannelType::DI,
            communication_address: format!("{:05}", 100 + i),
            power_supply_type: "24V DC".to_string(),
            description: Some(format!("数字量输入通道 {}", i)),
            is_enabled: true,
            created_at: Some(now),
            updated_at: Some(now),
        });
    }

    // DI2通道 (16个无源)
    for i in 1..=16 {
        configs.push(TestPlcChannelConfig {
            id: Some(uuid::Uuid::new_v4().to_string()),
            channel_address: format!("DI2_{}", i),
            channel_type: TestPlcChannelType::DINone,
            communication_address: format!("{:05}", 200 + i),
            power_supply_type: "无源".to_string(),
            description: Some(format!("数字量输入通道(无源) {}", i)),
            is_enabled: true,
            created_at: Some(now),
            updated_at: Some(now),
        });
    }

    // DO1通道 (16个有源)
    for i in 1..=16 {
        configs.push(TestPlcChannelConfig {
            id: Some(uuid::Uuid::new_v4().to_string()),
            channel_address: format!("DO1_{}", i),
            channel_type: TestPlcChannelType::DO,
            communication_address: format!("{:05}", 300 + i),
            power_supply_type: "24V DC".to_string(),
            description: Some(format!("数字量输出通道 {}", i)),
            is_enabled: true,
            created_at: Some(now),
            updated_at: Some(now),
        });
    }

    // DO2通道 (16个无源)
    for i in 1..=16 {
        configs.push(TestPlcChannelConfig {
            id: Some(uuid::Uuid::new_v4().to_string()),
            channel_address: format!("DO2_{}", i),
            channel_type: TestPlcChannelType::DONone,
            communication_address: format!("{:05}", 400 + i),
            power_supply_type: "无源".to_string(),
            description: Some(format!("数字量输出通道(无源) {}", i)),
            is_enabled: true,
            created_at: Some(now),
            updated_at: Some(now),
        });
    }

    info!("创建了 {} 个测试PLC通道配置", configs.len());
    configs
}