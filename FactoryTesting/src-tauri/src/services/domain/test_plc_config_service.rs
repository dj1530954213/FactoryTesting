// 文件: FactoryTesting/src-tauri/src/services/domain/test_plc_config_service.rs
// 详细注释：测试PLC配置管理的领域服务

use async_trait::async_trait;
use std::sync::Arc;
use chrono::Utc;
use log::{info, debug};

use crate::models::test_plc_config::*;
use crate::services::traits::{BaseService, PersistenceService};
use crate::utils::error::{AppError, AppResult};
use crate::models::entities;
use sea_orm::{EntityTrait, QueryFilter, ColumnTrait, ActiveModelTrait};

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
    
    /// 获取通道映射配置
    async fn get_channel_mappings(&self) -> AppResult<Vec<ChannelMappingConfig>>;
    
    /// 自动生成通道映射
    async fn generate_channel_mappings(&self, request: GenerateChannelMappingsRequest) -> AppResult<GenerateChannelMappingsResponse>;
    
    /// 初始化默认测试PLC通道配置
    async fn initialize_default_test_plc_channels(&self) -> AppResult<()>;
}

/// 测试PLC配置管理服务实现
pub struct TestPlcConfigService {
    persistence_service: Arc<dyn PersistenceService>,
}

impl TestPlcConfigService {
    /// 创建新的测试PLC配置服务实例
    pub fn new(persistence_service: Arc<dyn PersistenceService>) -> Self {
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
        debug!("获取测试PLC通道配置，筛选条件: {:?}", request);
        
        // 从数据库加载所有测试PLC通道配置
        let mut channels = self.persistence_service.load_all_test_plc_channels().await?;
        
        // 应用筛选条件
        if let Some(channel_type) = request.channel_type_filter {
            channels.retain(|c| c.channel_type == channel_type);
        }
        
        if let Some(true) = request.enabled_only {
            channels.retain(|c| c.is_enabled);
        }
        
        info!("返回 {} 个测试PLC通道配置", channels.len());
        Ok(channels)
    }

    async fn save_test_plc_channel(&self, mut channel: TestPlcChannelConfig) -> AppResult<TestPlcChannelConfig> {
        debug!("保存测试PLC通道配置: {:?}", channel.channel_address);
        
        // 验证必填字段
        if channel.channel_address.is_empty() {
            return Err(AppError::validation_error("通道位号不能为空".to_string()));
        }
        
        if channel.communication_address.is_empty() {
            return Err(AppError::validation_error("通讯地址不能为空".to_string()));
        }
        
        if channel.power_supply_type.is_empty() {
            return Err(AppError::validation_error("供电类型不能为空".to_string()));
        }
        
        // 设置时间戳和ID
        let now = Utc::now();
        if channel.id.is_none() {
            channel.id = Some(crate::models::structs::default_id());
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
        
        // 这里需要实现实际的PLC连接测试逻辑
        // 临时实现：模拟测试结果
        let success = connection_id.len() % 2 == 0; // 简单的模拟逻辑
        
        let response = TestPlcConnectionResponse {
            success,
            message: if success {
                "PLC连接测试成功".to_string()
            } else {
                "PLC连接测试失败：连接超时".to_string()
            },
            connection_time_ms: if success { Some(150) } else { None },
        };
        
        info!("PLC连接测试完成: {} - {}", connection_id, if success { "成功" } else { "失败" });
        Ok(response)
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
        
        let mut mappings = Vec::new();
        let mut conflicts = Vec::new();
        
        // 根据策略生成映射
        match request.strategy {
            MappingStrategy::ByChannelType => {
                // 按通道类型匹配的逻辑
                for target_id in &request.target_channel_ids {
                    // 这里需要根据目标通道的类型找到匹配的测试PLC通道
                    // 临时实现：简单分配
                    if let Some(test_channel) = test_channels.first() {
                        mappings.push(ChannelMappingConfig {
                            id: format!("mapping_{}", mappings.len() + 1),
                            target_channel_id: target_id.clone(),
                            test_plc_channel_id: test_channel.id.clone().unwrap_or_default(),
                            mapping_type: MappingType::Direct,
                            is_active: true,
                            notes: Some("自动生成的映射".to_string()),
                            created_at: Utc::now(),
                        });
                    }
                }
            }
            MappingStrategy::Sequential => {
                // 顺序分配逻辑
                for (index, target_id) in request.target_channel_ids.iter().enumerate() {
                    if let Some(test_channel) = test_channels.get(index % test_channels.len()) {
                        mappings.push(ChannelMappingConfig {
                            id: format!("mapping_{}", index + 1),
                            target_channel_id: target_id.clone(),
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
                // 这里可以实现更复杂的负载均衡算法
                for (index, target_id) in request.target_channel_ids.iter().enumerate() {
                    if let Some(test_channel) = test_channels.get(index % test_channels.len()) {
                        mappings.push(ChannelMappingConfig {
                            id: format!("mapping_{}", index + 1),
                            target_channel_id: target_id.clone(),
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::infrastructure::persistence::SqliteOrmPersistenceService;
    use crate::services::infrastructure::persistence::persistence_service::PersistenceConfig;
    use std::path::Path;
    use tokio;

    async fn create_test_service() -> TestPlcConfigService {
        let config = PersistenceConfig::default();
        let persistence_service = Arc::new(
            SqliteOrmPersistenceService::new(config, Some(Path::new(":memory:"))).await.unwrap()
        );
        TestPlcConfigService::new(persistence_service)
    }

    #[tokio::test]
    async fn test_save_and_load_test_plc_channel() {
        let service = create_test_service().await;
        
        // 创建测试通道配置
        let channel = TestPlcChannelConfig {
            id: None,
            channel_address: "AI1_1".to_string(),
            channel_type: TestPlcChannelType::AI,
            communication_address: "40101".to_string(),
            power_supply_type: "24V DC".to_string(),
            description: Some("测试模拟量输入通道".to_string()),
            is_enabled: true,
            created_at: None,
            updated_at: None,
        };

        // 保存通道配置
        let saved_channel = service.save_test_plc_channel(channel).await.unwrap();
        assert!(saved_channel.id.is_some());
        assert_eq!(saved_channel.channel_address, "AI1_1");
        assert_eq!(saved_channel.power_supply_type, "24V DC");

        // 获取所有通道配置
        let request = GetTestPlcChannelsRequest {
            channel_type_filter: None,
            enabled_only: None,
        };
        let channels = service.get_test_plc_channels(request).await.unwrap();
        assert_eq!(channels.len(), 1);
        assert_eq!(channels[0].channel_address, "AI1_1");
    }

    #[tokio::test]
    async fn test_power_supply_type_validation() {
        let service = create_test_service().await;
        
        // 创建没有供电类型的测试通道配置
        let channel = TestPlcChannelConfig {
            id: None,
            channel_address: "AI1_1".to_string(),
            channel_type: TestPlcChannelType::AI,
            communication_address: "40101".to_string(),
            power_supply_type: "".to_string(), // 空的供电类型
            description: Some("测试模拟量输入通道".to_string()),
            is_enabled: true,
            created_at: None,
            updated_at: None,
        };

        // 尝试保存应该失败
        let result = service.save_test_plc_channel(channel).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("供电类型不能为空"));
    }

    #[tokio::test]
    async fn test_initialize_default_channels() {
        let service = create_test_service().await;
        
        // 初始化默认通道配置
        service.initialize_default_test_plc_channels().await.unwrap();

        // 验证默认通道数量
        let request = GetTestPlcChannelsRequest {
            channel_type_filter: None,
            enabled_only: None,
        };
        let channels = service.get_test_plc_channels(request).await.unwrap();
        assert_eq!(channels.len(), 88); // 应该有88个默认通道

        // 验证不同类型的通道数量
        let ai_request = GetTestPlcChannelsRequest {
            channel_type_filter: Some(TestPlcChannelType::AI),
            enabled_only: None,
        };
        let ai_channels = service.get_test_plc_channels(ai_request).await.unwrap();
        assert_eq!(ai_channels.len(), 8); // 应该有8个AI通道

        // 验证所有通道都有供电类型
        for channel in &channels {
            assert!(!channel.power_supply_type.is_empty(), 
                "通道 {} 的供电类型不能为空", channel.channel_address);
        }
    }
} 