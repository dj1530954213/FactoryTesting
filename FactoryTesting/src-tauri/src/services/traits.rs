/// 服务层基础trait定义
/// 提供各层服务的接口规范，支持依赖注入和测试

use async_trait::async_trait;
use std::collections::HashMap;
use crate::utils::error::AppResult;
use crate::models::structs::*;

/// 基础服务trait，所有服务都应实现
#[async_trait]
pub trait BaseService: Send + Sync {
    /// 服务名称
    fn service_name(&self) -> &'static str;
    
    /// 初始化服务
    async fn initialize(&mut self) -> AppResult<()>;
    
    /// 关闭服务
    async fn shutdown(&mut self) -> AppResult<()>;
    
    /// 健康检查
    async fn health_check(&self) -> AppResult<()>;
}

/// 数据持久化服务trait
#[async_trait]
pub trait PersistenceService: BaseService {
    /// 保存通道点位定义
    async fn save_channel_definition(&self, definition: &ChannelPointDefinition) -> AppResult<()>;
    
    /// 加载通道点位定义
    async fn load_channel_definition(&self, id: &str) -> AppResult<Option<ChannelPointDefinition>>;
    
    /// 加载所有通道点位定义
    async fn load_all_channel_definitions(&self) -> AppResult<Vec<ChannelPointDefinition>>;
    
    /// 删除通道点位定义
    async fn delete_channel_definition(&self, id: &str) -> AppResult<()>;
    
    /// 保存测试实例
    async fn save_test_instance(&self, instance: &ChannelTestInstance) -> AppResult<()>;
    
    /// 加载测试实例
    async fn load_test_instance(&self, instance_id: &str) -> AppResult<Option<ChannelTestInstance>>;

    /// 加载所有测试实例
    async fn load_all_test_instances(&self) -> AppResult<Vec<ChannelTestInstance>>;

    /// 加载批次的所有测试实例
    async fn load_test_instances_by_batch(&self, batch_id: &str) -> AppResult<Vec<ChannelTestInstance>>;
    
    /// 删除测试实例
    async fn delete_test_instance(&self, instance_id: &str) -> AppResult<()>;
    
    /// 保存测试批次信息
    async fn save_batch_info(&self, batch: &TestBatchInfo) -> AppResult<()>;
    
    /// 加载测试批次信息
    async fn load_batch_info(&self, batch_id: &str) -> AppResult<Option<TestBatchInfo>>;
    
    /// 加载所有测试批次信息
    async fn load_all_batch_info(&self) -> AppResult<Vec<TestBatchInfo>>;
    
    /// 删除测试批次信息
    async fn delete_batch_info(&self, batch_id: &str) -> AppResult<()>;
    
    /// 保存测试结果
    async fn save_test_outcome(&self, outcome: &RawTestOutcome) -> AppResult<()>;
    
    /// 按测试实例ID查询测试结果
    async fn load_test_outcomes_by_instance(&self, instance_id: &str) -> AppResult<Vec<RawTestOutcome>>;
    
    /// 按批次ID查询测试结果
    async fn load_test_outcomes_by_batch(&self, batch_id: &str) -> AppResult<Vec<RawTestOutcome>>;
    
    /// 测试PLC配置相关方法
    
    /// 保存测试PLC通道配置
    async fn save_test_plc_channel(&self, channel: &crate::models::test_plc_config::TestPlcChannelConfig) -> AppResult<()>;
    
    /// 加载测试PLC通道配置
    async fn load_test_plc_channel(&self, id: &str) -> AppResult<Option<crate::models::test_plc_config::TestPlcChannelConfig>>;
    
    /// 加载所有测试PLC通道配置
    async fn load_all_test_plc_channels(&self) -> AppResult<Vec<crate::models::test_plc_config::TestPlcChannelConfig>>;
    
    /// 删除测试PLC通道配置
    async fn delete_test_plc_channel(&self, id: &str) -> AppResult<()>;
    
    /// 保存PLC连接配置
    async fn save_plc_connection(&self, connection: &crate::models::test_plc_config::PlcConnectionConfig) -> AppResult<()>;
    
    /// 加载PLC连接配置
    async fn load_plc_connection(&self, id: &str) -> AppResult<Option<crate::models::test_plc_config::PlcConnectionConfig>>;
    
    /// 加载所有PLC连接配置
    async fn load_all_plc_connections(&self) -> AppResult<Vec<crate::models::test_plc_config::PlcConnectionConfig>>;
    
    /// 删除PLC连接配置
    async fn delete_plc_connection(&self, id: &str) -> AppResult<()>;
    
    /// 保存通道映射配置
    async fn save_channel_mapping(&self, mapping: &crate::models::test_plc_config::ChannelMappingConfig) -> AppResult<()>;
    
    /// 加载通道映射配置
    async fn load_channel_mapping(&self, id: &str) -> AppResult<Option<crate::models::test_plc_config::ChannelMappingConfig>>;
    
    /// 加载所有通道映射配置
    async fn load_all_channel_mappings(&self) -> AppResult<Vec<crate::models::test_plc_config::ChannelMappingConfig>>;
    
    /// 删除通道映射配置
    async fn delete_channel_mapping(&self, id: &str) -> AppResult<()>;
}

/// PLC通信服务trait
#[async_trait]
pub trait PlcCommunicationService: BaseService {
    /// 连接到PLC
    async fn connect(&mut self) -> AppResult<()>;
    
    /// 断开PLC连接
    async fn disconnect(&mut self) -> AppResult<()>;
    
    /// 检查连接状态
    fn is_connected(&self) -> bool;
    
    /// 读取布尔值
    async fn read_bool(&self, address: &str) -> AppResult<bool>;
    
    /// 写入布尔值
    async fn write_bool(&self, address: &str, value: bool) -> AppResult<()>;
    
    /// 读取浮点数
    async fn read_float(&self, address: &str) -> AppResult<f32>;
    
    /// 写入浮点数
    async fn write_float(&self, address: &str, value: f32) -> AppResult<()>;
    
    /// 读取整数
    async fn read_int(&self, address: &str) -> AppResult<i32>;
    
    /// 写入整数
    async fn write_int(&self, address: &str, value: i32) -> AppResult<()>;
    
    /// 批量读取（地址 -> 值的映射）
    async fn batch_read(&self, addresses: &[String]) -> AppResult<HashMap<String, serde_json::Value>>;
    
    /// 批量写入
    async fn batch_write(&self, values: &HashMap<String, serde_json::Value>) -> AppResult<()>;
}

/// 测试执行器trait基础接口
#[async_trait]
pub trait TestExecutor: BaseService {
    /// 执行测试
    async fn execute_test(
        &self,
        definition: &ChannelPointDefinition,
        instance: &mut ChannelTestInstance,
    ) -> AppResult<RawTestOutcome>;
    
    /// 检查是否支持特定的测试项
    fn supports_test_item(&self, definition: &ChannelPointDefinition, test_item: &crate::models::enums::SubTestItem) -> bool;
    
    /// 获取支持的模块类型
    fn supported_module_types(&self) -> Vec<crate::models::enums::ModuleType>;
}

/// 状态管理服务trait
#[async_trait]
pub trait StateManagementService: BaseService {
    /// 获取通道当前状态
    async fn get_channel_state(&self, instance_id: &str) -> AppResult<crate::models::enums::OverallTestStatus>;
    
    /// 更新通道状态
    async fn update_channel_state(
        &self,
        instance_id: &str,
        new_status: crate::models::enums::OverallTestStatus,
    ) -> AppResult<()>;
    
    /// 检查状态转换是否有效
    fn is_valid_state_transition(
        &self,
        from: &crate::models::enums::OverallTestStatus,
        to: &crate::models::enums::OverallTestStatus,
    ) -> bool;
    
    /// 获取批次统计信息
    async fn get_batch_statistics(&self, batch_id: &str) -> AppResult<BatchStatistics>;
}

/// 批次统计信息
#[derive(Debug, Clone)]
pub struct BatchStatistics {
    pub total_channels: u32,
    pub tested_channels: u32,
    pub passed_channels: u32,
    pub failed_channels: u32,
    pub skipped_channels: u32,
    pub in_progress_channels: u32,
    pub start_time: Option<chrono::DateTime<chrono::Utc>>,
    pub end_time: Option<chrono::DateTime<chrono::Utc>>,
    pub estimated_completion_time: Option<chrono::DateTime<chrono::Utc>>,
}

/// 事件发布服务trait
#[async_trait]
pub trait EventPublisher: BaseService {
    /// 发布测试状态变化事件
    async fn publish_test_status_changed(
        &self,
        instance_id: &str,
        old_status: crate::models::enums::OverallTestStatus,
        new_status: crate::models::enums::OverallTestStatus,
    ) -> AppResult<()>;
    
    /// 发布测试完成事件
    async fn publish_test_completed(&self, outcome: &RawTestOutcome) -> AppResult<()>;
    
    /// 发布批次状态变化事件
    async fn publish_batch_status_changed(&self, batch_id: &str, statistics: &BatchStatistics) -> AppResult<()>;
    
    /// 发布PLC连接状态变化事件
    async fn publish_plc_connection_changed(&self, connected: bool) -> AppResult<()>;
    
    /// 发布错误事件
    async fn publish_error(&self, error: &crate::utils::error::AppError) -> AppResult<()>;
}

/// 服务容器trait - 用于依赖注入
pub trait ServiceContainer: Send + Sync {
    /// 获取持久化服务
    fn persistence_service(&self) -> &dyn PersistenceService;
    
    /// 获取PLC通信服务
    fn plc_service(&self) -> &dyn PlcCommunicationService;
    
    /// 获取状态管理服务
    fn state_service(&self) -> &dyn StateManagementService;
    
    /// 获取事件发布服务
    fn event_publisher(&self) -> &dyn EventPublisher;
    
    /// 获取测试执行器列表
    fn test_executors(&self) -> Vec<&dyn TestExecutor>;
} 