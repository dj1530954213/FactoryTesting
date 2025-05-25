/// 通道状态管理器
/// 
/// 负责管理通道测试实例的状态，是唯一可以修改 ChannelTestInstance 核心状态的组件

use crate::models::{ChannelTestInstance, RawTestOutcome, OverallTestStatus};
use crate::services::infrastructure::IPersistenceService;
use crate::utils::error::{AppError, AppResult};
use async_trait::async_trait;
use std::sync::Arc;
use log::{debug, info, warn, error};

/// 通道状态管理器接口
#[async_trait]
pub trait IChannelStateManager: Send + Sync {
    /// 创建新的测试实例
    async fn create_test_instance(
        &self,
        definition_id: &str,
        batch_id: &str,
    ) -> AppResult<ChannelTestInstance>;

    /// 获取测试实例状态
    async fn get_instance_state(&self, instance_id: &str) -> AppResult<ChannelTestInstance>;

    /// 更新测试结果
    async fn update_test_result(
        &self,
        instance_id: &str,
        outcome: RawTestOutcome,
    ) -> AppResult<()>;

    /// 更新实例整体状态
    async fn update_overall_status(
        &self,
        instance_id: &str,
        status: OverallTestStatus,
    ) -> AppResult<()>;
}

/// 通道状态管理器实现
pub struct ChannelStateManager {
    /// 持久化服务
    persistence_service: Arc<dyn IPersistenceService>,
}

impl ChannelStateManager {
    /// 创建新的通道状态管理器
    pub fn new(persistence_service: Arc<dyn IPersistenceService>) -> Self {
        Self {
            persistence_service,
        }
    }
}

#[async_trait]
impl IChannelStateManager for ChannelStateManager {
    /// 创建新的测试实例
    async fn create_test_instance(
        &self,
        definition_id: &str,
        batch_id: &str,
    ) -> AppResult<ChannelTestInstance> {
        let instance = ChannelTestInstance::new(
            definition_id.to_string(),
            batch_id.to_string(),
        );

        info!("创建测试实例: {}", instance.instance_id);
        Ok(instance)
    }

    /// 获取测试实例状态
    async fn get_instance_state(&self, instance_id: &str) -> AppResult<ChannelTestInstance> {
        // TODO: 从持久化服务获取实例状态
        Err(AppError::not_found_error("测试实例", &format!("测试实例未找到: {}", instance_id)))
    }

    /// 更新测试结果
    async fn update_test_result(
        &self,
        instance_id: &str,
        outcome: RawTestOutcome,
    ) -> AppResult<()> {
        info!("更新测试结果: {} -> {:?}", instance_id, outcome.success);
        // TODO: 实现具体的结果更新逻辑
        Ok(())
    }

    /// 更新实例整体状态
    async fn update_overall_status(
        &self,
        instance_id: &str,
        status: OverallTestStatus,
    ) -> AppResult<()> {
        info!("更新整体状态: {} -> {:?}", instance_id, status);
        // TODO: 实现具体的状态更新逻辑
        Ok(())
    }
} 