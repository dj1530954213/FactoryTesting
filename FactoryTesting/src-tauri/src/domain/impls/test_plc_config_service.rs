// 文件: FactoryTesting/src-tauri/src/services/domain/test_plc_config_service.rs
// 详细注释：测试PLC配置管理的领域服务

use async_trait::async_trait;
use std::sync::Arc;
use chrono::Utc;
use log::{info, debug, warn};

use crate::models::test_plc_config::*;
use crate::domain::services::BaseService;
use crate::infrastructure::IPersistenceService;
use crate::utils::error::{AppError, AppResult};
use crate::domain::services::plc_communication_service::{IPlcCommunicationService, PlcConnectionConfig as DomainPlcConnectionConfig, PlcProtocol, ConnectionTestResult};

/// Modbus寄存器类型枚举
#[derive(Debug, Clone, Copy, PartialEq)]
enum ModbusRegisterType {
    Coil,                  // 线圈 (0xxxxx)
    DiscreteInput,         // 离散输入 (1xxxxx)
    InputRegister,         // 输入寄存器 (3xxxxx)
    HoldingRegister,       // 保持寄存器 (4xxxxx)
}

/// 测试PLC配置管理服务接口
#[async_trait]
pub trait ITestPlcConfigService: BaseService + Send + Sync {
    /// 获取所有测试PLC通道配置
    async fn get_test_plc_channels(&self, request: GetTestPlcChannelsRequest) -> AppResult<Vec<TestPlcChannelConfig>>;
    
    /// 保存测试PLC通道配置
    async fn save_test_plc_channel(&self, channel: TestPlcChannelConfig) -> AppResult<TestPlcChannelConfig>;
    
    /// 删除测试PLC通道配置
    async fn delete_test_plc_channel(&self, channel_id: &str) -> AppResult<()>;
    
    /// 获取所有PLC连接配置
    async fn get_plc_connections(&self) -> AppResult<Vec<PlcConnectionConfig>>;
    
    /// 保存PLC连接配置
    async fn save_plc_connection(&self, connection: PlcConnectionConfig) -> AppResult<PlcConnectionConfig>;
    
    /// 测试PLC连接
    async fn test_plc_connection(&self, connection_id: &str) -> AppResult<TestPlcConnectionResponse>;
    
    /// 测试临时PLC连接配置
    async fn test_temp_plc_connection(&self, connection: &PlcConnectionConfig) -> AppResult<TestPlcConnectionResponse>;
    
    /// 测试地址读取
    async fn test_address_read(&self, connection: &PlcConnectionConfig, address: &str, data_type: &str) -> AppResult<crate::models::test_plc_config::AddressReadTestResponse>;
    
    /// 获取通道映射配置
    async fn get_channel_mappings(&self) -> AppResult<Vec<ChannelMappingConfig>>;
    
    /// 自动生成通道映射
    async fn generate_channel_mappings(&self, request: GenerateChannelMappingsRequest) -> AppResult<GenerateChannelMappingsResponse>;
    
    /// 初始化默认测试PLC通道配置
    async fn initialize_default_test_plc_channels(&self) -> AppResult<()>;
    
    /// 从SQL文件恢复默认测试PLC通道配置
    async fn restore_default_channels_from_sql(&self) -> AppResult<usize>;
    
    /// 获取测试PLC配置 (用于通道分配服务)
    async fn get_test_plc_config(&self) -> AppResult<crate::application::services::channel_allocation_service::TestPlcConfig>;
}

/// 测试PLC配置管理服务实现
pub struct TestPlcConfigService {
    persistence_service: Arc<dyn IPersistenceService>,
}

impl TestPlcConfigService {
    /// 创建新的测试PLC配置服务实例
    pub fn new(persistence_service: Arc<dyn IPersistenceService>) -> Self {
        Self {
            persistence_service,
        }
    }

    /// 获取默认的测试PLC通道配置数据（基于您提供的88个通道数据）
    fn get_default_test_plc_channels() -> Vec<TestPlcChannelConfig> {
        let mut channels = Vec::new();
        
        // AI通道 (8个)
        for i in 1..=8 {
            channels.push(TestPlcChannelConfig {
                id: None,
                channel_address: format!("AI1_{}", i),
                channel_type: TestPlcChannelType::AI,
                communication_address: format!("{}", 40101 + (i - 1) * 2),
                power_supply_type: "24V DC".to_string(),
                description: Some(format!("模拟量输入通道 {}", i)),
                is_enabled: true,
                created_at: Some(Utc::now()),
                updated_at: Some(Utc::now()),
            });
        }
        
        // AO通道 (8个)
        for i in 1..=8 {
            channels.push(TestPlcChannelConfig {
                id: None,
                channel_address: format!("AO1_{}", i),
                channel_type: TestPlcChannelType::AO,
                communication_address: format!("{}", 40201 + (i - 1) * 2),
                power_supply_type: "24V DC".to_string(),
                description: Some(format!("模拟量输出通道 {}", i)),
                is_enabled: true,
                created_at: Some(Utc::now()),
                updated_at: Some(Utc::now()),
            });
        }
        
        // AO无源通道 (8个)
        for i in 1..=8 {
            channels.push(TestPlcChannelConfig {
                id: None,
                channel_address: format!("AO2_{}", i),
                channel_type: TestPlcChannelType::AONone,
                communication_address: format!("{}", 40301 + (i - 1) * 2),
                power_supply_type: "无源".to_string(),
                description: Some(format!("模拟量输出通道(无源) {}", i)),
                is_enabled: true,
                created_at: Some(Utc::now()),
                updated_at: Some(Utc::now()),
            });
        }
        
        // DI通道 (16个)
        for i in 1..=16 {
            channels.push(TestPlcChannelConfig {
                id: None,
                channel_address: format!("DI1_{}", i),
                channel_type: TestPlcChannelType::DI,
                communication_address: format!("{:05}", 101 + i - 1),
                power_supply_type: "24V DC".to_string(),
                description: Some(format!("数字量输入通道 {}", i)),
                is_enabled: true,
                created_at: Some(Utc::now()),
                updated_at: Some(Utc::now()),
            });
        }
        
        // DI无源通道 (16个)
        for i in 1..=16 {
            channels.push(TestPlcChannelConfig {
                id: None,
                channel_address: format!("DI2_{}", i),
                channel_type: TestPlcChannelType::DINone,
                communication_address: format!("{:05}", 201 + i - 1),
                power_supply_type: "无源".to_string(),
                description: Some(format!("数字量输入通道(无源) {}", i)),
                is_enabled: true,
                created_at: Some(Utc::now()),
                updated_at: Some(Utc::now()),
            });
        }
        
        // DO通道 (16个)
        for i in 1..=16 {
            channels.push(TestPlcChannelConfig {
                id: None,
                channel_address: format!("DO1_{}", i),
                channel_type: TestPlcChannelType::DO,
                communication_address: format!("{:05}", 301 + i - 1),
                power_supply_type: "24V DC".to_string(),
                description: Some(format!("数字量输出通道 {}", i)),
                is_enabled: true,
                created_at: Some(Utc::now()),
                updated_at: Some(Utc::now()),
            });
        }
        
        // DO无源通道 (16个)
        for i in 1..=16 {
            channels.push(TestPlcChannelConfig {
                id: None,
                channel_address: format!("DO2_{}", i),
                channel_type: TestPlcChannelType::DONone,
                communication_address: format!("{:05}", 401 + i - 1),
                power_supply_type: "无源".to_string(),
                description: Some(format!("数字量输出通道(无源) {}", i)),
                is_enabled: true,
                created_at: Some(Utc::now()),
                updated_at: Some(Utc::now()),
            });
        }
        
        channels
    }

    /// 获取默认的PLC连接配置
    fn get_default_plc_connections() -> Vec<PlcConnectionConfig> {
        vec![
            PlcConnectionConfig {
                id: "test_plc_1".to_string(),
                name: "测试PLC".to_string(),
                plc_type: PlcType::ModbusTcp,
                ip_address: "192.168.1.100".to_string(),
                port: 502,
                timeout: 5000,
                retry_count: 3,
                byte_order: "CDAB".to_string(),
                zero_based_address: false,
                is_test_plc: true,
                description: Some("用于测试的PLC设备".to_string()),
                is_enabled: true,
                last_connected: None,
                connection_status: ConnectionStatus::Disconnected,
            },
            PlcConnectionConfig {
                id: "target_plc_1".to_string(),
                name: "被测PLC".to_string(),
                plc_type: PlcType::ModbusTcp,
                ip_address: "192.168.1.200".to_string(),
                port: 502,
                timeout: 5000,
                retry_count: 3,
                byte_order: "CDAB".to_string(),
                zero_based_address: false,
                is_test_plc: false,
                description: Some("被测试的PLC设备".to_string()),
                is_enabled: true,
                last_connected: None,
                connection_status: ConnectionStatus::Disconnected,
            },
        ]
    }
}

#[async_trait]
impl BaseService for TestPlcConfigService {
    fn service_name(&self) -> &'static str {
        "TestPlcConfigService"
    }

    async fn initialize(&mut self) -> AppResult<()> {
        info!("正在初始化 {}", self.service_name());
        
        // 初始化默认的测试PLC通道配置
        self.initialize_default_test_plc_channels().await?;
        
        info!("{} 初始化完成", self.service_name());
        Ok(())
    }

    async fn shutdown(&mut self) -> AppResult<()> {
        info!("{} 正在关闭", self.service_name());
        Ok(())
    }

    async fn health_check(&self) -> AppResult<()> {
        // 检查持久化服务是否可用
        self.persistence_service.health_check().await?;
        debug!("{} 健康检查通过", self.service_name());
        Ok(())
    }
}

#[async_trait]
impl ITestPlcConfigService for TestPlcConfigService {
    async fn get_test_plc_channels(&self, request: GetTestPlcChannelsRequest) -> AppResult<Vec<TestPlcChannelConfig>> {
        debug!("获取测试PLC通道配置，过滤条件: {:?}", request);
        
        // 从数据库加载所有测试PLC通道配置
        let mut channels = self.persistence_service.load_all_test_plc_channels().await?;
        
        // 应用过滤条件
        if let Some(channel_type) = request.channel_type_filter {
            channels.retain(|ch| ch.channel_type == channel_type);
        }
        
        if let Some(enabled_only) = request.enabled_only {
            if enabled_only {
                channels.retain(|ch| ch.is_enabled);
            }
        }
        
        info!("返回 {} 个测试PLC通道配置", channels.len());
        Ok(channels)
    }

    async fn save_test_plc_channel(&self, mut channel: TestPlcChannelConfig) -> AppResult<TestPlcChannelConfig> {
        debug!("保存测试PLC通道配置: {:?}", channel.channel_address);
        
        // 验证必填字段
        if channel.channel_address.is_empty() {
            return Err(AppError::validation_error("通道地址不能为空".to_string()));
        }
        
        if channel.communication_address.is_empty() {
            return Err(AppError::validation_error("通讯地址不能为空".to_string()));
        }
        
        if channel.power_supply_type.is_empty() {
            return Err(AppError::validation_error("供电类型不能为空".to_string()));
        }
        
        // 设置时间戳
        let now = Utc::now();
        if channel.id.is_none() {
            channel.created_at = Some(now);
        }
        channel.updated_at = Some(now);
        
        // 保存到数据库
        self.persistence_service.save_test_plc_channel(&channel).await?;
        
        info!("测试PLC通道配置保存成功: {}", channel.channel_address);
        Ok(channel)
    }

    async fn delete_test_plc_channel(&self, channel_id: &str) -> AppResult<()> {
        debug!("删除测试PLC通道配置: {}", channel_id);
        
        // 从数据库删除
        self.persistence_service.delete_test_plc_channel(channel_id).await?;
        
        info!("测试PLC通道配置删除成功: {}", channel_id);
        Ok(())
    }

    async fn get_plc_connections(&self) -> AppResult<Vec<PlcConnectionConfig>> {
        debug!("获取PLC连接配置");
        
        // 从数据库加载所有PLC连接配置
        let connections = self.persistence_service.load_all_plc_connections().await?;
        
        info!("返回 {} 个PLC连接配置", connections.len());
        Ok(connections)
    }

    async fn save_plc_connection(&self, connection: PlcConnectionConfig) -> AppResult<PlcConnectionConfig> {
        debug!("保存PLC连接配置: {:?}", connection.name);
        
        // 验证必填字段
        if connection.name.is_empty() {
            return Err(AppError::validation_error("连接名称不能为空".to_string()));
        }
        
        if connection.ip_address.is_empty() {
            return Err(AppError::validation_error("IP地址不能为空".to_string()));
        }
        
        // 保存到数据库
        self.persistence_service.save_plc_connection(&connection).await?;
        
        info!("PLC连接配置保存成功: {}", connection.name);
        Ok(connection)
    }

    async fn test_plc_connection(&self, connection_id: &str) -> AppResult<TestPlcConnectionResponse> {
        debug!("测试PLC连接: {}", connection_id);

        // 从数据库获取PLC连接配置
        let connection_config = self.persistence_service.load_plc_connection(connection_id).await?
            .ok_or_else(|| AppError::not_found_error("PLC连接配置", &format!("ID: {}", connection_id)))?;

        debug!("找到PLC连接配置: {} ({}:{})", connection_config.name, connection_config.ip_address, connection_config.port);

        // 根据PLC类型选择相应的通信服务进行测试
        let test_result = match connection_config.plc_type {
            crate::models::test_plc_config::PlcType::ModbusTcp => {
                self.test_modbus_tcp_connection(&connection_config).await
            }

            _ => {
                // 其他协议暂未实现
                Ok(TestPlcConnectionResponse {
                    success: false,
                    message: format!("暂不支持的PLC类型: {:?}", connection_config.plc_type),
                    connection_time_ms: None,
                })
            }
        };

        let response = test_result?;
        info!("PLC连接测试完成: {} - {}", connection_id, if response.success { "成功" } else { "失败" });
        Ok(response)
    }

    async fn test_temp_plc_connection(&self, connection: &PlcConnectionConfig) -> AppResult<TestPlcConnectionResponse> {
        debug!("测试临时PLC连接: {} ({}:{})", connection.name, connection.ip_address, connection.port);

        // 直接使用提供的连接配置进行测试，不需要从数据库查找
        let test_result = match connection.plc_type {
            crate::models::test_plc_config::PlcType::ModbusTcp => {
                self.test_modbus_tcp_connection(connection).await
            }

            _ => {
                // 其他协议暂未实现
                Ok(TestPlcConnectionResponse {
                    success: false,
                    message: format!("暂不支持的PLC类型: {:?}", connection.plc_type),
                    connection_time_ms: None,
                })
            }
        };

        let response = test_result?;
        info!("临时PLC连接测试完成: {} - {}", connection.name, if response.success { "成功" } else { "失败" });
        Ok(response)
    }

    async fn test_address_read(&self, connection: &PlcConnectionConfig, address: &str, data_type: &str) -> AppResult<crate::models::test_plc_config::AddressReadTestResponse> {
        debug!("测试地址读取: {} - 地址: {}, 类型: {}", connection.name, address, data_type);

        // 根据PLC类型进行地址读取测试
        match connection.plc_type {
            crate::models::test_plc_config::PlcType::ModbusTcp => {
                self.test_modbus_address_read(connection, address, data_type).await
            }

            _ => {
                // 其他协议暂未实现
                Ok(crate::models::test_plc_config::AddressReadTestResponse {
                    success: false,
                    value: None,
                    error: Some(format!("暂不支持的PLC类型: {:?}", connection.plc_type)),
                    read_time_ms: None,
                })
            }
        }
    }

    async fn get_channel_mappings(&self) -> AppResult<Vec<ChannelMappingConfig>> {
        debug!("获取通道映射配置");
        
        // 从数据库加载所有通道映射配置
        let mappings = self.persistence_service.load_all_channel_mappings().await?;
        
        info!("返回 {} 个通道映射配置", mappings.len());
        Ok(mappings)
    }

    async fn generate_channel_mappings(&self, request: GenerateChannelMappingsRequest) -> AppResult<GenerateChannelMappingsResponse> {
        debug!("自动生成通道映射，策略: {:?}", request.strategy);
        
        // 获取可用的测试PLC通道
        let test_channels = self.get_test_plc_channels(GetTestPlcChannelsRequest {
            channel_type_filter: None,
            enabled_only: Some(true),
        }).await?;
        
        // 获取目标通道定义（需要从持久化服务获取）
        let mut target_definitions = Vec::new();
        for target_id in &request.target_channel_ids {
            if let Ok(Some(definition)) = self.persistence_service.load_channel_definition(target_id).await {
                target_definitions.push(definition);
            }
        }
        
        let mut mappings = Vec::new();
        let mut conflicts = Vec::new();
        
        // 根据策略生成映射
        match request.strategy {
            MappingStrategy::ByChannelType => {
                // 智能匹配：有源对无源，无源对有源
                mappings = Self::generate_intelligent_mappings(&target_definitions, &test_channels, &mut conflicts)?;
            }
            MappingStrategy::Sequential => {
                // 顺序分配逻辑
                for (index, definition) in target_definitions.iter().enumerate() {
                    if let Some(test_channel) = test_channels.get(index % test_channels.len()) {
                        mappings.push(ChannelMappingConfig {
                            id: format!("mapping_{}", index + 1),
                            target_channel_id: definition.id.clone(),
                            test_plc_channel_id: test_channel.id.clone().unwrap_or_default(),
                            mapping_type: MappingType::Direct,
                            is_active: true,
                            notes: Some("顺序分配的映射".to_string()),
                            created_at: Utc::now(),
                        });
                    }
                }
            }
            MappingStrategy::LoadBalanced => {
                // 负载均衡分配逻辑
                for (index, definition) in target_definitions.iter().enumerate() {
                    if let Some(test_channel) = test_channels.get(index % test_channels.len()) {
                        mappings.push(ChannelMappingConfig {
                            id: format!("mapping_{}", index + 1),
                            target_channel_id: definition.id.clone(),
                            test_plc_channel_id: test_channel.id.clone().unwrap_or_default(),
                            mapping_type: MappingType::Direct,
                            is_active: true,
                            notes: Some("负载均衡分配的映射".to_string()),
                            created_at: Utc::now(),
                        });
                    }
                }
            }
        }
        
        let response = GenerateChannelMappingsResponse {
            success: true,
            message: format!("成功生成 {} 个通道映射", mappings.len()),
            mappings,
            conflicts,
        };
        
        info!("通道映射生成完成，生成 {} 个映射", response.mappings.len());
        Ok(response)
    }

    async fn initialize_default_test_plc_channels(&self) -> AppResult<()> {
        debug!("初始化默认测试PLC通道配置");
        
        // 检查是否已有配置
        let existing_channels = self.persistence_service.load_all_test_plc_channels().await?;
        
        if !existing_channels.is_empty() {
            debug!("已存在 {} 个测试PLC通道配置，跳过初始化", existing_channels.len());
            return Ok(());
        }
        
        // 创建默认配置
        let default_channels = Self::get_default_test_plc_channels();
        
        for channel in default_channels {
            self.persistence_service.save_test_plc_channel(&channel).await?;
        }
        
        // 创建默认PLC连接配置
        let default_connections = Self::get_default_plc_connections();
        
        for connection in default_connections {
            self.persistence_service.save_plc_connection(&connection).await?;
        }
        
        info!("默认测试PLC配置初始化完成");
        Ok(())
    }

    async fn restore_default_channels_from_sql(&self) -> AppResult<usize> {
        debug!("从SQL文件恢复默认测试PLC通道配置");
        
        // 嵌入SQL文件内容
        // Rust知识点：include_str! 宏在编译时将文件内容作为字符串常量嵌入
        const DEFAULT_CHANNELS_SQL: &str = include_str!("../../../data/test_plc_channel_configs_defult.sql");
        
        // 执行SQL恢复操作
        let result = self.persistence_service.restore_test_plc_channels_from_sql(DEFAULT_CHANNELS_SQL).await?;
        
        info!("成功从SQL文件恢复 {} 个测试PLC通道配置", result);
        Ok(result)
    }

    async fn get_test_plc_config(&self) -> AppResult<crate::application::services::channel_allocation_service::TestPlcConfig> {
        debug!("获取测试PLC配置");
        
        // 获取所有测试PLC通道配置
        let request = GetTestPlcChannelsRequest {
            channel_type_filter: None,
            enabled_only: Some(true), // 只获取启用的通道
        };
        let test_channels = self.get_test_plc_channels(request).await?;
        
        // 获取第一个PLC连接配置作为默认配置
        let plc_connections = self.get_plc_connections().await?;
        let test_plc_connection = plc_connections.iter()
            .find(|conn| conn.is_test_plc && conn.is_enabled)
            .ok_or_else(|| AppError::not_found_error("测试PLC连接", "没有找到启用的测试PLC连接配置"))?;
        
        // 转换TestPlcChannelConfig到ComparisonTable
        let mut comparison_tables = Vec::new();
        for channel in test_channels {
            let is_powered = !channel.power_supply_type.trim().is_empty() 
                && !channel.power_supply_type.contains("无源");
            
            let module_type = match channel.channel_type {
                crate::models::test_plc_config::TestPlcChannelType::AI => crate::models::ModuleType::AI,
                crate::models::test_plc_config::TestPlcChannelType::AINone => crate::models::ModuleType::AI,
                crate::models::test_plc_config::TestPlcChannelType::AO => crate::models::ModuleType::AO,
                crate::models::test_plc_config::TestPlcChannelType::AONone => crate::models::ModuleType::AO,
                crate::models::test_plc_config::TestPlcChannelType::DI => crate::models::ModuleType::DI,
                crate::models::test_plc_config::TestPlcChannelType::DINone => crate::models::ModuleType::DI,
                crate::models::test_plc_config::TestPlcChannelType::DO => crate::models::ModuleType::DO,
                crate::models::test_plc_config::TestPlcChannelType::DONone => crate::models::ModuleType::DO,
            };
            
            comparison_tables.push(crate::application::services::channel_allocation_service::ComparisonTable {
                channel_address: channel.channel_address,
                communication_address: channel.communication_address,
                channel_type: module_type,
                is_powered,
            });
        }
        
        debug!("转换完成：{} 个通道映射表", comparison_tables.len());
        
        Ok(crate::application::services::channel_allocation_service::TestPlcConfig {
            brand_type: format!("{:?}", test_plc_connection.plc_type),
            ip_address: test_plc_connection.ip_address.clone(),
            comparison_tables,
        })
    }
}

impl TestPlcConfigService {
    /// 测试Modbus TCP地址读取
    async fn test_modbus_address_read(&self, connection: &PlcConnectionConfig, address: &str, data_type: &str) -> AppResult<crate::models::test_plc_config::AddressReadTestResponse> {
        use std::time::Instant;
        use tokio::time::{timeout, Duration};
        use tokio_modbus::prelude::*;

        let start_time = Instant::now();
        debug!("开始测试Modbus地址读取: {}:{} - 地址: {}, 类型: {}", connection.ip_address, connection.port, address, data_type);

        // 设置超时时间
        let timeout_duration = Duration::from_millis(connection.timeout as u64);

        // 解析地址
        let (register_type, register_offset) = match Self::parse_modbus_address(address, connection.zero_based_address) {
            Ok(result) => result,
            Err(e) => {
                return Ok(crate::models::test_plc_config::AddressReadTestResponse {
                    success: false,
                    value: None,
                    error: Some(format!("地址解析失败: {}", e)),
                    read_time_ms: Some(start_time.elapsed().as_millis() as u64),
                });
            }
        };

        // 建立连接并读取数据
        let socket_addr = format!("{}:{}", connection.ip_address, connection.port);
        let socket_addr = match socket_addr.parse::<std::net::SocketAddr>() {
            Ok(addr) => addr,
            Err(e) => {
                return Ok(crate::models::test_plc_config::AddressReadTestResponse {
                    success: false,
                    value: None,
                    error: Some(format!("无效的地址格式: {}", e)),
                    read_time_ms: Some(start_time.elapsed().as_millis() as u64),
                });
            }
        };

        let read_result = timeout(timeout_duration, async {
            // 建立Modbus连接
            let mut ctx = tokio_modbus::client::tcp::connect_slave(socket_addr, Slave(1)).await
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::ConnectionRefused, format!("Modbus连接失败: {:?}", e)))?;

            // 根据寄存器类型和数据类型进行读取
            match (register_type, data_type.to_lowercase().as_str()) {
                (ModbusRegisterType::Coil, "bool") => {
                    // 先获取线圈向量，再取首值
                    let coils_vec: Vec<bool> = ctx.read_coils(register_offset, 1).await
                        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, format!("读取线圈IO失败: {:?}", e)))?
                        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, format!("读取线圈Modbus异常: {:?}", e)))?;
                    let value = coils_vec.get(0).copied().unwrap_or(false);
                    Ok(serde_json::Value::Bool(value))
                }
                (ModbusRegisterType::DiscreteInput, "bool") => {
                    let di_vec: Vec<bool> = ctx.read_discrete_inputs(register_offset, 1).await
                        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, format!("读取离散输入IO失败: {:?}", e)))?
                        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, format!("读取离散输入Modbus异常: {:?}", e)))?;
                    let value = di_vec.get(0).copied().unwrap_or(false);
                    Ok(serde_json::Value::Bool(value))
                }
                (ModbusRegisterType::HoldingRegister, "float") => {
                    let value = {
                        let regs_vec: Vec<u16> = ctx.read_holding_registers(register_offset, 2).await
                            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, format!("读取保持寄存器IO失败: {:?}", e)))?
                            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, format!("读取保持寄存器Modbus异常: {:?}", e)))?;
                        if regs_vec.len() < 2 {
                            return Err(std::io::Error::new(
                                std::io::ErrorKind::InvalidData,
                                "读取的寄存器数据不足",
                            ));
                        }
                        Self::convert_registers_to_f32(&regs_vec, &connection.byte_order)
                    };
                    let number = serde_json::Number::from_f64(value as f64)
                        .unwrap_or_else(|| serde_json::Number::from(0i64));
                    Ok(serde_json::Value::Number(number))
                }
                (ModbusRegisterType::InputRegister, "float") => {
                    let value = {
                        let regs_vec: Vec<u16> = ctx.read_input_registers(register_offset, 2).await
                            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, format!("读取输入寄存器IO失败: {:?}", e)))?
                            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, format!("读取输入寄存器Modbus异常: {:?}", e)))?;
                        if regs_vec.len() < 2 {
                            return Err(std::io::Error::new(
                                std::io::ErrorKind::InvalidData,
                                "读取的寄存器数据不足",
                            ));
                        }
                        Self::convert_registers_to_f32(&regs_vec, &connection.byte_order)
                    };
                    let number = serde_json::Number::from_f64(value as f64)
                        .unwrap_or_else(|| serde_json::Number::from(0i64));
                    Ok(serde_json::Value::Number(number))
                }
                (ModbusRegisterType::HoldingRegister, "int") => {
                    let regs_vec: Vec<u16> = ctx.read_holding_registers(register_offset, 1).await
                        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, format!("读取保持寄存器IO失败: {:?}", e)))?
                        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, format!("读取保持寄存器Modbus异常: {:?}", e)))?;
                    let raw_val = *regs_vec.get(0).ok_or_else(|| {
                        std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            "读取的寄存器数据为空",
                        )
                    })?;
                    Ok(serde_json::Value::Number(serde_json::Number::from(raw_val as i64)))
                }
                (ModbusRegisterType::InputRegister, "int") => {
                    let regs_vec: Vec<u16> = ctx.read_input_registers(register_offset, 1).await
                        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, format!("读取输入寄存器IO失败: {:?}", e)))?
                        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, format!("读取输入寄存器Modbus异常: {:?}", e)))?;
                    let raw_val = *regs_vec.get(0).ok_or_else(|| {
                        std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            "读取的寄存器数据为空",
                        )
                    })?;
                    Ok(serde_json::Value::Number(serde_json::Number::from(raw_val as i64)))
                }
                _ => {
                    Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, 
                        format!("不支持的寄存器类型和数据类型组合: {:?} + {}", register_type, data_type)))
                }
            }
        }).await;

        let elapsed_ms = start_time.elapsed().as_millis() as u64;

        match read_result {
            Ok(Ok(value)) => {
                info!("地址读取成功: {} = {:?}", address, value);
                Ok(crate::models::test_plc_config::AddressReadTestResponse {
                    success: true,
                    value: Some(value),
                    error: None,
                    read_time_ms: Some(elapsed_ms),
                })
            }
            Ok(Err(e)) => {
                warn!("地址读取失败: {} - {}", address, e);
                Ok(crate::models::test_plc_config::AddressReadTestResponse {
                    success: false,
                    value: None,
                    error: Some(format!("读取失败: {}", e)),
                    read_time_ms: Some(elapsed_ms),
                })
            }
            Err(_) => {
                warn!("地址读取超时: {}", address);
                Ok(crate::models::test_plc_config::AddressReadTestResponse {
                    success: false,
                    value: None,
                    error: Some("读取超时".to_string()),
                    read_time_ms: Some(elapsed_ms),
                })
            }
        }
    }

    /// 解析Modbus地址
    fn parse_modbus_address(address: &str, zero_based: bool) -> Result<(ModbusRegisterType, u16), String> {
        if address.len() < 2 {
            return Err("地址长度不足".to_string());
        }

        let prefix = &address[..1];
        let number_part = &address[1..];
        
        let number: u16 = number_part.parse()
            .map_err(|_| format!("无效的地址数字: {}", number_part))?;

        // 根据是否为0基地址计算偏移量
        // 1基地址(标准Modbus): 地址1对应寄存器0, 所以偏移量是地址-1
        // 0基地址: 地址0对应寄存器0, 所以偏移量就是地址本身
        let offset = if zero_based {
            number
        } else {
            number.saturating_sub(1)
        };

        match prefix {
            "0" => Ok((ModbusRegisterType::Coil, offset)),
            "1" => Ok((ModbusRegisterType::DiscreteInput, offset)),
            "3" => Ok((ModbusRegisterType::InputRegister, offset)),
            "4" => Ok((ModbusRegisterType::HoldingRegister, offset)),
            _ => Err(format!("不支持的地址前缀: {}", prefix)),
        }
    }

    /// 将寄存器数据转换为float32
    fn convert_registers_to_f32(registers: &[u16], byte_order: &str) -> f32 {
        if registers.len() < 2 {
            warn!("转换f32需要2个寄存器，但只收到: {}个", registers.len());
            return 0.0;
        }

        let high_word = registers[0];
        let low_word = registers[1];

        let b1h = (high_word >> 8) as u8;
        let b1l = (high_word & 0xFF) as u8;
        let b2h = (low_word >> 8) as u8;
        let b2l = (low_word & 0xFF) as u8;

        match byte_order.to_uppercase().as_str() {
            // 小端 (DCBA): 低字节在前，低字在前
            "DCBA" => f32::from_le_bytes([b2l, b2h, b1l, b1h]),
            
            // 小端，字交换 (CDAB): 高字节在前，低字在前
            "CDAB" => f32::from_be_bytes([b2h, b2l, b1h, b1l]),

            // 大端，字节交换 (BADC): 低字节在前，高字在前
            "BADC" => f32::from_be_bytes([b1l, b1h, b2l, b2h]),
            
            // 大端 (ABCD) 或默认: 高字节在前，高字在前
            _ => f32::from_be_bytes([b1h, b1l, b2h, b2l]),
        }
    }
    /// 测试Modbus TCP连接
    async fn test_modbus_tcp_connection(&self, config: &crate::models::test_plc_config::PlcConnectionConfig) -> AppResult<TestPlcConnectionResponse> {
        use std::time::Instant;
        use tokio::time::{timeout, Duration};
        use tokio_modbus::prelude::*;

        let start_time = Instant::now();

        debug!("开始测试Modbus TCP连接: {}:{}", config.ip_address, config.port);

        // 设置超时时间
        let timeout_duration = Duration::from_millis(config.timeout as u64);

        // 直接使用tokio_modbus进行连接测试
        let socket_addr = format!("{}:{}", config.ip_address, config.port);
        let socket_addr = match socket_addr.parse::<std::net::SocketAddr>() {
            Ok(addr) => addr,
            Err(e) => {
                let elapsed_ms = start_time.elapsed().as_millis() as u64;
                warn!("无效的地址格式: {}", e);
                return Ok(TestPlcConnectionResponse {
                    success: false,
                    message: format!("无效的地址格式: {}", e),
                    connection_time_ms: Some(elapsed_ms),
                });
            }
        };

        let test_result = timeout(timeout_duration, async {
            // 尝试建立Modbus TCP连接
            match tokio_modbus::client::tcp::connect_slave(socket_addr, Slave(1)).await {
                Ok(mut ctx) => {
                    // 尝试读取一个测试寄存器
                    match ctx.read_holding_registers(0, 1).await {
                        Ok(_) => Ok(true),
                        Err(_) => Ok(false), // 连接成功但读取失败，可能是PLC不支持该地址
                    }
                }
                Err(e) => Err(e),
            }
        }).await;

        let elapsed_ms = start_time.elapsed().as_millis() as u64;

        match test_result {
            Ok(Ok(true)) => {
                debug!("Modbus TCP连接测试成功，耗时: {}ms", elapsed_ms);
                Ok(TestPlcConnectionResponse {
                    success: true,
                    message: format!("Modbus TCP连接测试成功 ({}ms)", elapsed_ms),
                    connection_time_ms: Some(elapsed_ms),
                })
            }
            Ok(Ok(false)) => {
                debug!("Modbus TCP连接建立成功但读取测试失败，耗时: {}ms", elapsed_ms);
                Ok(TestPlcConnectionResponse {
                    success: true, // 连接成功，即使读取失败
                    message: format!("Modbus TCP连接成功，但测试读取失败 ({}ms)", elapsed_ms),
                    connection_time_ms: Some(elapsed_ms),
                })
            }
            Ok(Err(e)) => {
                warn!("Modbus TCP连接测试失败: {}", e);
                Ok(TestPlcConnectionResponse {
                    success: false,
                    message: format!("连接测试失败: {}", e),
                    connection_time_ms: Some(elapsed_ms),
                })
            }
            Err(_) => {
                warn!("Modbus TCP连接测试超时 ({}ms)", elapsed_ms);
                Ok(TestPlcConnectionResponse {
                    success: false,
                    message: format!("连接测试超时 ({}ms)", elapsed_ms),
                    connection_time_ms: Some(elapsed_ms),
                })
            }
        }
    }

    /// 智能映射生成：实现有源/无源匹配逻辑
    fn generate_intelligent_mappings(
        target_definitions: &[crate::models::ChannelPointDefinition],
        test_channels: &[TestPlcChannelConfig],
        conflicts: &mut Vec<String>
    ) -> AppResult<Vec<ChannelMappingConfig>> {
        let mut mappings = Vec::new();
        let mut used_test_channels = std::collections::HashSet::new();
        
        for definition in target_definitions {
            // 根据被测通道的模块类型和供电类型，找到匹配的测试PLC通道
            let target_module_type = &definition.module_type;
            let target_power_type = &definition.power_supply_type;
            
            // 确定需要的测试PLC通道类型
            let required_test_channel_type = Self::determine_test_channel_type(target_module_type, target_power_type);
            
            // 查找可用的匹配通道
            let available_channel = test_channels.iter()
                .find(|ch| {
                    ch.channel_type == required_test_channel_type && 
                    ch.is_enabled && 
                    !used_test_channels.contains(&ch.id.as_ref().unwrap_or(&String::new()).clone())
                });
            
            if let Some(test_channel) = available_channel {
                let test_channel_id = test_channel.id.clone().unwrap_or_default();
                used_test_channels.insert(test_channel_id.clone());
                
                mappings.push(ChannelMappingConfig {
                    id: format!("mapping_{}", mappings.len() + 1),
                    target_channel_id: definition.id.clone(),
                    test_plc_channel_id: test_channel_id,
                    mapping_type: if target_power_type == "有源" { MappingType::Inverse } else { MappingType::Direct },
                    is_active: true,
                    notes: Some(format!("智能匹配: {} {} -> {} {}", 
                        definition.tag, target_power_type,
                        test_channel.channel_address, 
                        if target_power_type == "有源" { "无源" } else { "有源" })),
                    created_at: Utc::now(),
                });
                
                info!("成功匹配: {} ({}) -> {} ({})", 
                    definition.tag, target_power_type,
                    test_channel.channel_address, 
                    if target_power_type == "有源" { "无源" } else { "有源" });
            } else {
                let conflict_msg = format!("无法为通道 {} ({}) 找到匹配的测试PLC通道 (需要: {:?})", 
                    definition.tag, target_power_type, required_test_channel_type);
                conflicts.push(conflict_msg);
                warn!("映射冲突: {}", conflicts.last().unwrap());
            }
        }
        
        Ok(mappings)
    }
    
    /// 根据被测通道类型和供电类型，确定需要的测试PLC通道类型
    fn determine_test_channel_type(
        target_module_type: &crate::models::ModuleType,
        target_power_type: &str
    ) -> TestPlcChannelType {
        use crate::models::ModuleType;
        
        match (target_module_type, target_power_type) {
            // AI通道：有源对无源，无源对有源
            (ModuleType::AI, "有源") => TestPlcChannelType::AINone,  // 有源AI需要无源测试通道
            (ModuleType::AI, "无源") => TestPlcChannelType::AI,      // 无源AI需要有源测试通道
            (ModuleType::AINone, _) => TestPlcChannelType::AI,       // 无源AI需要有源测试通道
            
            // AO通道：有源对无源，无源对有源  
            (ModuleType::AO, "有源") => TestPlcChannelType::AONone,  // 有源AO需要无源测试通道
            (ModuleType::AO, "无源") => TestPlcChannelType::AO,      // 无源AO需要有源测试通道
            (ModuleType::AONone, _) => TestPlcChannelType::AO,       // 无源AO需要有源测试通道
            
            // DI通道：有源对无源，无源对有源
            (ModuleType::DI, "有源") => TestPlcChannelType::DINone,  // 有源DI需要无源测试通道
            (ModuleType::DI, "无源") => TestPlcChannelType::DI,      // 无源DI需要有源测试通道
            (ModuleType::DINone, _) => TestPlcChannelType::DI,       // 无源DI需要有源测试通道
            
            // DO通道：有源对无源，无源对有源
            (ModuleType::DO, "有源") => TestPlcChannelType::DONone,  // 有源DO需要无源测试通道
            (ModuleType::DO, "无源") => TestPlcChannelType::DO,      // 无源DO需要有源测试通道
            (ModuleType::DONone, _) => TestPlcChannelType::DO,       // 无源DO需要有源测试通道
            
            // 其他类型的默认处理
            (ModuleType::Communication, _) => TestPlcChannelType::DI, // 通信类型默认为DI
            (ModuleType::Other(_), _) => TestPlcChannelType::DI,      // 其他类型默认为DI
            
            // 默认情况：直接匹配类型
            (ModuleType::AI, _) => TestPlcChannelType::AI,
            (ModuleType::AO, _) => TestPlcChannelType::AO,
            (ModuleType::DI, _) => TestPlcChannelType::DI,
            (ModuleType::DO, _) => TestPlcChannelType::DO,
        }
    }
}
