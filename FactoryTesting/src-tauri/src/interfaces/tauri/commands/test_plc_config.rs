//! # 测试PLC配置管理命令 (Test PLC Configuration Commands)
//!
//! ## 业务说明
//! 本模块负责管理测试PLC的全面配置，是测试系统正常运行的基础配置中心
//! 提供了完整的PLC配置生命周期管理和实时配置验证功能
//!
//! ## 核心配置管理
//! ### 1. 测试PLC通道配置
//! - **通道定义**: AI/AO/DI/DO通道的基本属性配置
//! - **地址映射**: PLC通信地址和物理通道的映射关系
//! - **供电类型**: 通道的供电方式配置（有源/无源）
//! - **参数设置**: 量程、精度、报警等技术参数
//!
//! ### 2. PLC连接配置
//! - **网络配置**: IP地址、端口、子网等网络参数
//! - **协议配置**: Modbus TCP、Siemens S7等通信协议
//! - **连接参数**: 超时、重试、心跳等连接参数
//! - **安全配置**: 认证、加密等安全参数
//!
//! ### 3. 通道映射配置
//! - **映射关系**: 被测通道与测试通道的对应关系
//! - **分组管理**: 按功能或类型进行通道分组
//! - **资源分配**: 测试资源的合理分配和调度
//!
//! ## 配置验证功能
//! - **连接测试**: 实时验证PLC连接状态
//! - **地址测试**: 验证通信地址的可读写性
//! - **配置校验**: 检查配置的完整性和一致性
//! - **性能测试**: 测试通信性能和响应时间
//!
//! ## 架构特点
//! - **热配置**: 支持运行时动态更新配置，无需重启
//! - **持久化**: 所有配置自动保存到数据库
//! - **版本管理**: 支持配置的版本控制和回滚
//! - **导入导出**: 支持配置的批量导入和导出
//!
//! ## 调用链路
//! ```
//! 前端配置界面 → Tauri命令 → 配置验证 → TestPlcConfigService → 
//! 数据库持久化 → 配置应用 → PLC连接更新
//! ```
//!
//! ## Rust知识点
//! - **配置管理**: 使用结构化数据管理复杂配置
//! - **异步验证**: 异步执行配置验证和测试
//! - **错误传播**: 详细的配置错误信息传播

use tauri::State;
use serde::{Deserialize, Serialize};
use crate::tauri_commands::AppState;
use crate::models::test_plc_config::*;
use crate::utils::error::AppResult;
use crate::models::entities::test_plc_channel_config;
use chrono::Utc;
use uuid;
use log::{info, debug};

/// 获取测试PLC通道配置
/// 
/// 业务说明：
/// - 获取系统中配置的测试PLC通道信息
/// - 支持按通道类型过滤（AI/AO/DI/DO）
/// - 支持只获取启用的通道
/// 
/// 参数：
/// - channel_type_filter: 可选的通道类型过滤器
/// - enabled_only: 是否只返回启用的通道
/// - state: 应用状态，包含配置服务
/// 
/// 返回：
/// - Ok: 测试PLC通道配置列表
/// - Err: 错误信息
/// 
/// 调用链：
/// 前端通道配置页面 -> get_test_plc_channels_cmd -> TestPlcConfigService -> 数据库
/// 
/// Rust知识点：
/// - Option<T> 表示可选值，可以是Some(T)或None
/// - State<'_, T> 是Tauri的状态管理类型
#[tauri::command]
pub async fn get_test_plc_channels_cmd(
    channel_type_filter: Option<TestPlcChannelType>,
    enabled_only: Option<bool>,
    state: State<'_, AppState>
) -> Result<Vec<TestPlcChannelConfig>, String> {
    debug!("获取测试PLC通道配置命令");
    
    // 构建请求参数
    let request = GetTestPlcChannelsRequest {
        channel_type_filter,
        enabled_only,
    };
    
    // 调用服务层获取通道配置
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
/// 
/// 业务说明：
/// - 保存或更新测试PLC通道配置
/// - 包含通道地址、类型、通讯地址、供电类型等信息
/// - 支持新增和更新操作（根据ID判断）
/// 
/// 参数：
/// - channel: 要保存的通道配置
/// - state: 应用状态
/// 
/// 返回：
/// - Ok: 保存后的通道配置（包含生成的ID等）
/// - Err: 错误信息
/// 
/// 调用链：
/// 前端配置编辑界面 -> save_test_plc_channel_cmd -> TestPlcConfigService -> 数据库
/// 
/// Rust知识点：
/// - move 语义：channel 参数的所有权转移到函数内
/// - {:?} 是Debug trait的格式化输出
#[tauri::command]
pub async fn save_test_plc_channel_cmd(
    channel: TestPlcChannelConfig,
    state: State<'_, AppState>
) -> Result<TestPlcChannelConfig, String> {
    debug!("保存测试PLC通道配置命令: {:?}", channel.channel_address);
    
    // 添加详细的输入验证日志
    // 业务说明：记录完整的配置信息便于调试和问题追踪
    info!("接收到通道配置数据: ID={:?}, 地址={}, 类型={:?}, 通讯地址={}, 供电类型={}", 
          channel.id, channel.channel_address, channel.channel_type, 
          channel.communication_address, channel.power_supply_type);
    
    // 调用服务层保存配置
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
/// 
/// 业务说明：
/// - 删除指定的测试PLC通道配置
/// - 删除前应确保该通道未被使用
/// 
/// 参数：
/// - channel_id: 要删除的通道ID
/// - state: 应用状态
/// 
/// 返回：
/// - Ok(()): 删除成功
/// - Err: 错误信息
/// 
/// 调用链：
/// 前端删除按钮 -> delete_test_plc_channel_cmd -> TestPlcConfigService -> 数据库
/// 
/// Rust知识点：
/// - () 是unit type，表示没有返回值
/// - &str 是字符串切片，避免了String的所有权转移
#[tauri::command]
pub async fn delete_test_plc_channel_cmd(
    channel_id: String,
    state: State<'_, AppState>
) -> Result<(), String> {
    debug!("删除测试PLC通道配置命令: {}", channel_id);
    
    // 调用服务层删除配置
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
/// 
/// 业务说明：
/// - 获取系统中所有的PLC连接配置
/// - 包括被测PLC和测试PLC的连接信息
/// - 每个连接包含IP地址、端口、协议类型等
/// 
/// 参数：
/// - state: 应用状态
/// 
/// 返回：
/// - Ok: PLC连接配置列表
/// - Err: 错误信息
/// 
/// 调用链：
/// 前端PLC连接管理页面 -> get_plc_connections_cmd -> TestPlcConfigService -> 数据库
#[tauri::command]
pub async fn get_plc_connections_cmd(
    state: State<'_, AppState>
) -> Result<Vec<PlcConnectionConfig>, String> {
    debug!("获取PLC连接配置命令");
    
    // 调用服务层获取所有PLC连接配置
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
/// 
/// 业务说明：
/// - 保存或更新PLC连接配置
/// - 包含连接名称、IP地址、端口、协议类型等
/// - 用于配置被测PLC和测试PLC的通信参数
/// 
/// 参数：
/// - connection: PLC连接配置对象
/// - state: 应用状态
/// 
/// 返回：
/// - Ok: 保存后的连接配置
/// - Err: 错误信息
/// 
/// 调用链：
/// 前端连接配置编辑界面 -> save_plc_connection_cmd -> TestPlcConfigService -> 数据库
#[tauri::command]
pub async fn save_plc_connection_cmd(
    connection: PlcConnectionConfig,
    state: State<'_, AppState>
) -> Result<PlcConnectionConfig, String> {
    debug!("保存PLC连接配置命令: {:?}", connection.name);
    
    // 调用服务层保存连接配置
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
/// 
/// 业务说明：
/// - 测试已保存的PLC连接是否可用
/// - 尝试建立连接并执行基本通信测试
/// - 返回连接状态和延迟信息
/// 
/// 参数：
/// - connection_id: 要测试的连接ID
/// - state: 应用状态
/// 
/// 返回：
/// - Ok: 测试结果，包含成功状态、延迟时间、错误信息等
/// - Err: 错误信息
/// 
/// 调用链：
/// 前端测试按钮 -> test_plc_connection_cmd -> TestPlcConfigService -> PLC通信层
/// 
/// Rust知识点：
/// - if表达式可以作为值使用
#[tauri::command]
pub async fn test_plc_connection_cmd(
    connection_id: String,
    state: State<'_, AppState>
) -> Result<TestPlcConnectionResponse, String> {
    debug!("测试PLC连接命令: {}", connection_id);
    
    // 调用服务层测试连接
    match state.test_plc_config_service.test_plc_connection(&connection_id).await {
        Ok(response) => {
            // Rust知识点：if表达式作为参数使用
            info!("PLC连接测试完成: {} - {}", connection_id, if response.success { "成功" } else { "失败" });
            Ok(response)
        }
        Err(e) => {
            log::error!("测试PLC连接失败: {}", e);
            Err(format!("测试PLC连接失败: {}", e))
        }
    }
}

/// 测试临时PLC连接配置
/// 
/// 业务说明：
/// - 测试未保存的临时PLC连接配置
/// - 用于在保存前验证连接参数是否正确
/// - 不会将配置保存到数据库
/// 
/// 参数：
/// - connection: 临时的PLC连接配置
/// - state: 应用状态
/// 
/// 返回：
/// - Ok: 测试结果
/// - Err: 错误信息
/// 
/// 调用链：
/// 前端连接配置界面（测试按钮） -> test_temp_plc_connection_cmd -> TestPlcConfigService -> PLC通信层
#[tauri::command]
pub async fn test_temp_plc_connection_cmd(
    connection: PlcConnectionConfig,
    state: State<'_, AppState>
) -> Result<TestPlcConnectionResponse, String> {
    debug!("测试临时PLC连接配置: {} ({}:{})", connection.name, connection.ip_address, connection.port);
    
    // 测试临时连接配置，不保存到数据库
    match state.test_plc_config_service.test_temp_plc_connection(&connection).await {
        Ok(response) => {
            info!("临时PLC连接测试完成: {} - {}", connection.name, if response.success { "成功" } else { "失败" });
            Ok(response)
        }
        Err(e) => {
            log::error!("测试临时PLC连接失败: {}", e);
            Err(format!("测试临时PLC连接失败: {}", e))
        }
    }
}



/// 测试地址读取
/// 
/// 业务说明：
/// - 测试特定PLC地址的读取功能
/// - 验证地址配置是否正确，数据类型是否匹配
/// - 用于调试和验证通道配置
/// 
/// 参数：
/// - connection: PLC连接配置
/// - address: 要测试的Modbus地址
/// - data_type: 数据类型（Float、Int、Bool等）
/// - state: 应用状态
/// 
/// 返回：
/// - Ok: 读取结果，包含成功状态、读取值、错误信息等
/// - Err: 错误信息
/// 
/// 调用链：
/// 前端地址测试工具 -> test_address_read_cmd -> TestPlcConfigService -> PLC通信层
/// 
/// Rust知识点：
/// - 多个参数的格式化输出
#[tauri::command]
pub async fn test_address_read_cmd(
    connection: PlcConnectionConfig,
    address: String,
    data_type: String,
    state: State<'_, AppState>
) -> Result<AddressReadTestResponse, String> {
    debug!("测试地址读取: {} ({}:{}) - 地址: {}, 类型: {}", 
           connection.name, connection.ip_address, connection.port, address, data_type);
    
    // 调用服务层测试地址读取
    match state.test_plc_config_service.test_address_read(&connection, &address, &data_type).await {
        Ok(response) => {
            info!("地址读取测试完成: {} [{}] - {}", 
                  address, data_type, if response.success { "成功" } else { "失败" });
            Ok(response)
        }
        Err(e) => {
            log::error!("测试地址读取失败: {}", e);
            Err(format!("测试地址读取失败: {}", e))
        }
    }
}

/// 获取通道映射配置
/// 
/// 业务说明：
/// - 获取被测通道与测试通道的映射关系
/// - 每个映射定义了被测PLC的某个通道对应测试PLC的哪个通道
/// - 用于自动测试时确定信号路由
/// 
/// 参数：
/// - state: 应用状态
/// 
/// 返回：
/// - Ok: 通道映射配置列表
/// - Err: 错误信息
/// 
/// 调用链：
/// 前端通道映射管理页面 -> get_channel_mappings_cmd -> TestPlcConfigService -> 数据库
#[tauri::command]
pub async fn get_channel_mappings_cmd(
    state: State<'_, AppState>
) -> Result<Vec<ChannelMappingConfig>, String> {
    debug!("获取通道映射配置命令");
    
    // 获取所有通道映射配置
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
/// 
/// 业务说明：
/// - 根据策略自动生成被测通道与测试通道的映射关系
/// - 支持多种映射策略（按类型匹配、按名称匹配等）
/// - 减少手动配置的工作量
/// 
/// 参数：
/// - request: 生成请求，包含映射策略等参数
/// - state: 应用状态
/// 
/// 返回：
/// - Ok: 生成结果，包含映射列表和统计信息
/// - Err: 错误信息
/// 
/// 调用链：
/// 前端映射配置向导 -> generate_channel_mappings_cmd -> TestPlcConfigService -> 映射算法
#[tauri::command]
pub async fn generate_channel_mappings_cmd(
    request: GenerateChannelMappingsRequest,
    state: State<'_, AppState>
) -> Result<GenerateChannelMappingsResponse, String> {
    debug!("自动生成通道映射命令，策略: {:?}", request.strategy);
    
    // 根据策略生成通道映射
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
/// 
/// 业务说明：
/// - 初始化系统默认的测试PLC通道配置
/// - 通常在系统首次启动或重置时调用
/// - 创建基础的通道配置模板
/// 
/// 参数：
/// - state: 应用状态
/// 
/// 返回：
/// - Ok(()): 初始化成功
/// - Err: 错误信息
/// 
/// 调用链：
/// 前端初始化向导 -> initialize_default_test_plc_channels_cmd -> TestPlcConfigService -> 数据库
#[tauri::command]
pub async fn initialize_default_test_plc_channels_cmd(
    state: State<'_, AppState>
) -> Result<(), String> {
    debug!("初始化默认测试PLC通道配置命令");

    // 调用服务层初始化默认配置
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
/// 
/// 业务说明：
/// - 恢复系统标准的88个测试PLC通道配置
/// - 包含8个AI、8个AO、8个AO无源、16个DI、16个DI无源、16个DO、16个DO无源
/// - 用于快速配置测试环境或恢复出厂设置
/// 
/// 参数：
/// - state: 应用状态
/// 
/// 返回：
/// - Ok: 成功消息，包含恢复的通道数量
/// - Err: 错误信息
/// 
/// 调用链：
/// 前端恢复默认按钮 -> restore_default_test_plc_channels_cmd -> create_88_test_plc_channels -> 批量保存
/// 
/// Rust知识点：
/// - for循环遍历集合
/// - mut变量用于计数
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
    // Rust知识点：for循环遍历Vec，config的所有权在每次迭代中转移
    for config in configs {
        match state.test_plc_config_service.save_test_plc_channel(config.clone()).await {
            Ok(_) => {
                saved_count += 1;
                debug!("已保存通道配置: {}", config.channel_address);
            }
            Err(e) => {
                log::error!("保存通道配置失败 {}: {}", config.channel_address, e);
                // 如果有任何一个配置保存失败，立即返回错误
                return Err(format!("保存通道配置失败 {}: {}", config.channel_address, e));
            }
        }
    }

    let result_msg = format!("成功恢复 {} 个测试PLC通道配置", saved_count);
    info!("{}", result_msg);

    Ok(result_msg)
}

/// 从SQL文件恢复默认测试PLC通道配置
/// 
/// 业务说明：
/// - 从嵌入的SQL文件读取默认配置
/// - 先清空现有的test_plc_channel_configs表
/// - 执行SQL文件中的INSERT语句恢复默认配置
/// - 确保操作的事务性，要么全部成功要么全部失败
/// 
/// 参数：
/// - state: 应用状态
/// 
/// 返回：
/// - Ok: 成功消息，包含恢复的通道数量
/// - Err: 错误信息
/// 
/// 调用链：
/// 前端恢复默认按钮 -> restore_default_channels_from_sql_cmd -> TestPlcConfigService -> 数据库
/// 
/// Rust知识点：
/// - include_str! 宏在编译时嵌入文件内容
/// - 事务处理确保数据一致性
#[tauri::command]
pub async fn restore_default_channels_from_sql_cmd(
    state: State<'_, AppState>
) -> Result<String, String> {
    info!("开始从SQL文件恢复默认测试PLC通道配置");

    // 调用服务层恢复默认配置
    match state.test_plc_config_service.restore_default_channels_from_sql().await {
        Ok(count) => {
            let result_msg = format!("成功恢复 {} 个测试PLC通道配置", count);
            info!("{}", result_msg);
            Ok(result_msg)
        }
        Err(e) => {
            log::error!("恢复默认测试PLC通道配置失败: {}", e);
            Err(format!("恢复默认测试PLC通道配置失败: {}", e))
        }
    }
}

/// 创建88个测试PLC通道配置（基于重构前的真实数据）
/// 
/// 业务说明：
/// - 创建标准的88个测试PLC通道配置
/// - 包含所有类型的通道：AI、AO、DI、DO，每种都有有源和无源版本
/// - 地址分配遵循标准Modbus地址规范
/// 
/// 返回：
/// - 88个测试PLC通道配置的Vec
/// 
/// 通道分配：
/// - AI通道: 8个有源 (地址40101-40115)
/// - AO通道: 8个有源 (地址40201-40215) + 8个无源 (地址40301-40315)
/// - DI通道: 16个有源 (地址00101-00116) + 16个无源 (地址00201-00216)
/// - DO通道: 16个有源 (地址00301-00316) + 16个无源 (地址00401-00416)
/// 
/// Rust知识点：
/// - Vec::new() 创建空向量
/// - for循环和范围表达式 1..=8
/// - format! 宏进行字符串格式化
fn create_88_test_plc_channels() -> Vec<TestPlcChannelConfig> {
    let mut configs = Vec::new();
    let now = Utc::now();

    // AI通道 (8个有源)
    // 业务说明：模拟量输入通道，用于读取模拟信号（温度、压力等）
    for i in 1..=8 {
        configs.push(TestPlcChannelConfig {
            id: Some(uuid::Uuid::new_v4().to_string()),  // 生成唯一ID
            channel_address: format!("AI1_{}", i),       // 通道地址
            channel_type: TestPlcChannelType::AI,        // 通道类型
            communication_address: format!("{}", 40100 + i * 2 - 1), // Modbus地址
            power_supply_type: "24V DC".to_string(),     // 供电类型
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
