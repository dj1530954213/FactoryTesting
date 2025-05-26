/// 测试协调服务
/// 
/// 负责协调整个测试流程，包括：
/// 1. 接收测试请求并验证
/// 2. 协调通道状态管理器和测试执行引擎
/// 3. 管理测试进度和结果收集
/// 4. 提供统一的测试API

use crate::models::{
    ChannelTestInstance, ChannelPointDefinition, RawTestOutcome, TestBatchInfo, 
    OverallTestStatus
};
use crate::services::domain::{
    IChannelStateManager, ITestExecutionEngine
};
use crate::services::infrastructure::{
    IPersistenceService
};
use crate::utils::error::{AppError, AppResult};
use async_trait::async_trait;
use std::sync::Arc;
use std::collections::HashMap;
use tokio::sync::{mpsc, RwLock};
use serde::{Serialize, Deserialize};
use log::{debug, warn, error, info};
use chrono::Utc;

/// 测试执行请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestExecutionRequest {
    /// 批次信息
    pub batch_info: TestBatchInfo,
    /// 要测试的通道定义列表
    pub channel_definitions: Vec<ChannelPointDefinition>,
    /// 最大并发测试数
    pub max_concurrent_tests: Option<usize>,
    /// 是否自动开始测试
    pub auto_start: bool,
}

/// 测试执行响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestExecutionResponse {
    /// 批次ID
    pub batch_id: String,
    /// 创建的测试实例数量
    pub instance_count: usize,
    /// 执行状态
    pub status: String,
    /// 消息
    pub message: String,
}

/// 测试进度更新
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestProgressUpdate {
    /// 批次ID
    pub batch_id: String,
    /// 实例ID
    pub instance_id: String,
    /// 点位标签
    pub point_tag: String,
    /// 整体状态
    pub overall_status: OverallTestStatus,
    /// 完成的子测试数量
    pub completed_sub_tests: usize,
    /// 总子测试数量
    pub total_sub_tests: usize,
    /// 最新的测试结果
    pub latest_result: Option<RawTestOutcome>,
    /// 更新时间
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// 批次执行状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum BatchExecutionStatus {
    /// 已提交，等待开始
    Submitted,
    /// 运行中
    Running,
    /// 已暂停
    Paused,
    /// 已完成
    Completed,
    /// 已停止
    Stopped,
    /// 失败
    Failed,
}

/// 批次执行信息
#[derive(Debug)]
pub struct BatchExecutionInfo {
    /// 批次信息
    pub batch_info: TestBatchInfo,
    /// 通道定义列表
    pub channel_definitions: Vec<ChannelPointDefinition>,
    /// 测试实例列表
    pub test_instances: Vec<ChannelTestInstance>,
    /// 任务ID映射 (instance_id -> task_id)
    pub task_mappings: HashMap<String, String>,
    /// 执行状态
    pub status: BatchExecutionStatus,
    /// 结果收集器
    pub result_receiver: Option<mpsc::Receiver<RawTestOutcome>>,
    /// 结果发送器
    pub result_sender: mpsc::Sender<RawTestOutcome>,
    /// 收集到的结果
    pub collected_results: Vec<RawTestOutcome>,
    /// 创建时间
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// 开始时间
    pub started_at: Option<chrono::DateTime<chrono::Utc>>,
    /// 完成时间
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl BatchExecutionInfo {
    /// 创建新的批次执行信息
    pub fn new(
        batch_info: TestBatchInfo,
        channel_definitions: Vec<ChannelPointDefinition>,
    ) -> Self {
        let (result_sender, result_receiver) = mpsc::channel(1000);
        
        Self {
            batch_info,
            channel_definitions,
            test_instances: Vec::new(),
            task_mappings: HashMap::new(),
            status: BatchExecutionStatus::Submitted,
            result_receiver: Some(result_receiver),
            result_sender,
            collected_results: Vec::new(),
            created_at: Utc::now(),
            started_at: None,
            completed_at: None,
        }
    }

    /// 获取进度信息
    pub fn get_progress(&self) -> Vec<TestProgressUpdate> {
        let mut progress = Vec::new();
        
        for instance in &self.test_instances {
            // 计算该实例的完成状态
            let instance_results: Vec<_> = self.collected_results
                .iter()
                .filter(|r| r.channel_instance_id == instance.instance_id)
                .collect();

            let completed_sub_tests = instance_results.len();
            let total_sub_tests = self.estimate_total_sub_tests(&instance.definition_id);
            
            // 确定整体状态 - 使用现有的OverallTestStatus变体
            let overall_status = if completed_sub_tests == 0 {
                OverallTestStatus::NotTested
            } else if completed_sub_tests < total_sub_tests {
                OverallTestStatus::HardPointTesting
            } else {
                // 检查是否有失败的测试
                let has_failures = instance_results.iter().any(|r| !r.success);
                if has_failures {
                    OverallTestStatus::TestCompletedFailed
                } else {
                    OverallTestStatus::TestCompletedPassed
                }
            };

            let latest_result = instance_results.last().cloned().cloned();

            progress.push(TestProgressUpdate {
                batch_id: self.batch_info.batch_id.clone(),
                instance_id: instance.instance_id.clone(),
                point_tag: format!("Point_{}", instance.definition_id), // 简化的标签
                overall_status,
                completed_sub_tests,
                total_sub_tests,
                latest_result,
                timestamp: Utc::now(),
            });
        }

        progress
    }

    /// 估算总的子测试数量（基于点位类型）
    fn estimate_total_sub_tests(&self, definition_id: &str) -> usize {
        // 查找对应的定义
        if let Some(definition) = self.channel_definitions.iter().find(|d| d.id == *definition_id) {
            match definition.module_type {
                crate::models::ModuleType::AI | crate::models::ModuleType::AINone => {
                    let mut count = 0;
                    // 硬点测试：5个百分比点
                    if definition.test_rig_plc_address.is_some() {
                        count += 5;
                    }
                    // 报警测试
                    if definition.sll_set_point_address.is_some() { count += 1; }
                    if definition.sl_set_point_address.is_some() { count += 1; }
                    if definition.sh_set_point_address.is_some() { count += 1; }
                    if definition.shh_set_point_address.is_some() { count += 1; }
                    count
                },
                crate::models::ModuleType::DI | crate::models::ModuleType::DINone => 1,
                _ => 1, // 其他类型暂时估算为1
            }
        } else {
            1 // 默认值
        }
    }
}

/// 测试协调服务接口
#[async_trait]
pub trait ITestCoordinationService: Send + Sync {
    /// 提交测试执行请求
    async fn submit_test_execution(
        &self,
        request: TestExecutionRequest,
    ) -> AppResult<TestExecutionResponse>;

    /// 开始指定批次的测试
    async fn start_batch_testing(&self, batch_id: &str) -> AppResult<()>;

    /// 暂停指定批次的测试
    async fn pause_batch_testing(&self, batch_id: &str) -> AppResult<()>;

    /// 恢复指定批次的测试
    async fn resume_batch_testing(&self, batch_id: &str) -> AppResult<()>;

    /// 停止指定批次的测试
    async fn stop_batch_testing(&self, batch_id: &str) -> AppResult<()>;

    /// 获取批次测试进度
    async fn get_batch_progress(&self, batch_id: &str) -> AppResult<Vec<TestProgressUpdate>>;

    /// 获取批次测试结果
    async fn get_batch_results(&self, batch_id: &str) -> AppResult<Vec<RawTestOutcome>>;

    /// 清理完成的批次
    async fn cleanup_completed_batch(&self, batch_id: &str) -> AppResult<()>;
}

/// 测试协调服务实现
pub struct TestCoordinationService {
    /// 通道状态管理器
    channel_state_manager: Arc<dyn IChannelStateManager>,
    /// 测试执行引擎
    test_execution_engine: Arc<dyn ITestExecutionEngine>,
    /// 持久化服务
    persistence_service: Arc<dyn IPersistenceService>,
    /// 活动批次执行信息
    active_batches: Arc<RwLock<HashMap<String, BatchExecutionInfo>>>,
}

impl TestCoordinationService {
    /// 创建新的测试协调服务
    pub fn new(
        channel_state_manager: Arc<dyn IChannelStateManager>,
        test_execution_engine: Arc<dyn ITestExecutionEngine>,
        persistence_service: Arc<dyn IPersistenceService>,
    ) -> Self {
        Self {
            channel_state_manager,
            test_execution_engine,
            persistence_service,
            active_batches: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 启动结果收集任务
    async fn start_result_collection(&self, batch_id: String) -> AppResult<()> {
        let active_batches = self.active_batches.clone();
        let persistence_service = self.persistence_service.clone();

        tokio::spawn(async move {
            let mut receiver = {
                let mut batches = active_batches.write().await;
                if let Some(batch_info) = batches.get_mut(&batch_id) {
                    batch_info.result_receiver.take()
                } else {
                    return;
                }
            };

            if let Some(mut receiver) = receiver {
                while let Some(result) = receiver.recv().await {
                    debug!("[TestCoordination] 收到测试结果: {} - {}", 
                           result.channel_instance_id, result.success);

                    // 保存结果到持久化存储
                    if let Err(e) = persistence_service.save_test_outcome(&result).await {
                        error!("[TestCoordination] 保存测试结果失败: {}", e);
                    }

                    // 更新批次信息中的结果集合
                    {
                        let mut batches = active_batches.write().await;
                        if let Some(batch_info) = batches.get_mut(&batch_id) {
                            batch_info.collected_results.push(result);

                            // 检查是否所有测试都完成了
                            let total_expected = batch_info.test_instances.len() * 
                                batch_info.test_instances.iter()
                                    .map(|inst| batch_info.estimate_total_sub_tests(&inst.definition_id))
                                    .sum::<usize>();

                            if batch_info.collected_results.len() >= total_expected {
                                batch_info.status = BatchExecutionStatus::Completed;
                                batch_info.completed_at = Some(Utc::now());
                                info!("[TestCoordination] 批次 {} 测试完成", batch_id);
                            }
                        }
                    }
                }
            }
        });

        Ok(())
    }
}

#[async_trait]
impl ITestCoordinationService for TestCoordinationService {
    /// 提交测试执行请求
    async fn submit_test_execution(
        &self,
        request: TestExecutionRequest,
    ) -> AppResult<TestExecutionResponse> {
        info!("[TestCoordination] 提交测试执行请求: 批次 {}, {} 个通道", 
              request.batch_info.batch_id, request.channel_definitions.len());

        // 验证请求
        if request.channel_definitions.is_empty() {
            return Err(AppError::validation_error("通道定义列表不能为空"));
        }

        // 保存批次信息
        self.persistence_service
            .save_batch_info(&request.batch_info)
            .await?;

        // 创建批次执行信息
        let mut batch_execution = BatchExecutionInfo::new(
            request.batch_info.clone(),
            request.channel_definitions.clone(),
        );

        // 创建测试实例
        let mut instance_count = 0;
        for definition in &request.channel_definitions {
            let instance = self.channel_state_manager
                .create_test_instance(&definition.id, &request.batch_info.batch_id)
                .await?;
            
            batch_execution.test_instances.push(instance);
            instance_count += 1;
        }

        // 启动结果收集任务
        self.start_result_collection(request.batch_info.batch_id.clone()).await?;

        // 添加到活动批次
        {
            let mut batches = self.active_batches.write().await;
            batches.insert(request.batch_info.batch_id.clone(), batch_execution);
        }

        // 如果设置了自动开始，立即启动测试
        if request.auto_start {
            self.start_batch_testing(&request.batch_info.batch_id).await?;
        }

        Ok(TestExecutionResponse {
            batch_id: request.batch_info.batch_id.clone(),
            instance_count,
            status: if request.auto_start { "running" } else { "submitted" }.to_string(),
            message: format!("成功提交 {} 个测试实例{}", 
                           instance_count, 
                           if request.auto_start { "并开始执行" } else { "" }),
        })
    }

    /// 开始指定批次的测试
    async fn start_batch_testing(&self, batch_id: &str) -> AppResult<()> {
        info!("[TestCoordination] 开始批次测试: {}", batch_id);

        let (instances, definitions, result_sender) = {
            let mut batches = self.active_batches.write().await;
            let batch_info = batches.get_mut(batch_id)
                .ok_or_else(|| AppError::not_found_error("批次", batch_id))?;

            if batch_info.status != BatchExecutionStatus::Submitted && 
               batch_info.status != BatchExecutionStatus::Paused {
                return Err(AppError::validation_error(
                    format!("批次状态不允许启动: {:?}", batch_info.status)
                ));
            }

            batch_info.status = BatchExecutionStatus::Running;
            batch_info.started_at = Some(Utc::now());

            (
                batch_info.test_instances.clone(),
                batch_info.channel_definitions.clone(),
                batch_info.result_sender.clone(),
            )
        };

        // 为每个测试实例提交执行任务
        for instance in instances {
            // 查找对应的定义
            if let Some(definition) = definitions.iter().find(|d| d.id == instance.definition_id) {
                debug!("[TestCoordination] 提交测试任务: 实例 {}, 定义 {}", 
                       instance.instance_id, definition.id);

                let task_id = self.test_execution_engine
                    .submit_test_instance(
                        instance.clone(),
                        definition.clone(),
                        result_sender.clone(),
                    )
                    .await?;

                // 记录任务映射
                let instance_id_clone = instance.instance_id.clone();
                let task_id_clone = task_id.clone();
                {
                    let mut batches = self.active_batches.write().await;
                    if let Some(batch_info) = batches.get_mut(batch_id) {
                        batch_info.task_mappings.insert(instance_id_clone, task_id_clone);
                    }
                }

                debug!("[TestCoordination] 测试任务已提交: {} -> {}", 
                       instance.instance_id, task_id);
            } else {
                warn!("[TestCoordination] 未找到实例 {} 对应的定义 {}", 
                      instance.instance_id, instance.definition_id);
            }
        }

        info!("[TestCoordination] 批次 {} 的所有测试任务已提交", batch_id);
        Ok(())
    }

    /// 暂停指定批次的测试
    async fn pause_batch_testing(&self, batch_id: &str) -> AppResult<()> {
        info!("[TestCoordination] 暂停批次测试: {}", batch_id);

        let mut batches = self.active_batches.write().await;
        let batch_info = batches.get_mut(batch_id)
            .ok_or_else(|| AppError::not_found_error("批次", batch_id))?;

        if batch_info.status != BatchExecutionStatus::Running {
            return Err(AppError::validation_error(
                format!("批次状态不允许暂停: {:?}", batch_info.status)
            ));
        }

        // 取消所有相关任务
        for task_id in batch_info.task_mappings.values() {
            if let Err(e) = self.test_execution_engine.cancel_task(task_id).await {
                warn!("[TestCoordination] 取消任务失败: {} - {}", task_id, e);
            }
        }

        batch_info.status = BatchExecutionStatus::Paused;
        info!("[TestCoordination] 批次 {} 已暂停", batch_id);
        Ok(())
    }

    /// 恢复指定批次的测试
    async fn resume_batch_testing(&self, batch_id: &str) -> AppResult<()> {
        info!("[TestCoordination] 恢复批次测试: {}", batch_id);

        let batches = self.active_batches.read().await;
        let batch_info = batches.get(batch_id)
            .ok_or_else(|| AppError::not_found_error("批次", batch_id))?;

        if batch_info.status != BatchExecutionStatus::Paused {
            return Err(AppError::validation_error(
                format!("批次状态不允许恢复: {:?}", batch_info.status)
            ));
        }

        drop(batches);

        // 重新启动测试（类似于开始测试）
        self.start_batch_testing(batch_id).await
    }

    /// 停止指定批次的测试
    async fn stop_batch_testing(&self, batch_id: &str) -> AppResult<()> {
        info!("[TestCoordination] 停止批次测试: {}", batch_id);

        let mut batches = self.active_batches.write().await;
        let batch_info = batches.get_mut(batch_id)
            .ok_or_else(|| AppError::not_found_error("批次", batch_id))?;

        if batch_info.status == BatchExecutionStatus::Completed || 
           batch_info.status == BatchExecutionStatus::Stopped {
            return Ok(()); // 已经完成或停止
        }

        // 取消所有相关任务
        for task_id in batch_info.task_mappings.values() {
            if let Err(e) = self.test_execution_engine.cancel_task(task_id).await {
                warn!("[TestCoordination] 取消任务失败: {} - {}", task_id, e);
            }
        }

        batch_info.status = BatchExecutionStatus::Stopped;
        batch_info.completed_at = Some(Utc::now());
        info!("[TestCoordination] 批次 {} 已停止", batch_id);
        Ok(())
    }

    /// 获取批次测试进度
    async fn get_batch_progress(&self, batch_id: &str) -> AppResult<Vec<TestProgressUpdate>> {
        let batches = self.active_batches.read().await;
        let batch_info = batches.get(batch_id)
            .ok_or_else(|| AppError::not_found_error("批次", batch_id))?;

        Ok(batch_info.get_progress())
    }

    /// 获取批次测试结果
    async fn get_batch_results(&self, batch_id: &str) -> AppResult<Vec<RawTestOutcome>> {
        let batches = self.active_batches.read().await;
        let batch_info = batches.get(batch_id)
            .ok_or_else(|| AppError::not_found_error("批次", batch_id))?;

        Ok(batch_info.collected_results.clone())
    }

    /// 清理完成的批次
    async fn cleanup_completed_batch(&self, batch_id: &str) -> AppResult<()> {
        info!("[TestCoordination] 清理完成的批次: {}", batch_id);

        let mut batches = self.active_batches.write().await;
        if let Some(batch_info) = batches.get(batch_id) {
            if batch_info.status != BatchExecutionStatus::Completed && 
               batch_info.status != BatchExecutionStatus::Stopped {
                return Err(AppError::validation_error(
                    format!("批次状态不允许清理: {:?}", batch_info.status)
                ));
            }
        }

        batches.remove(batch_id);
        info!("[TestCoordination] 批次 {} 已清理", batch_id);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::domain::{ChannelStateManager, TestExecutionEngine};
    use crate::services::infrastructure::{MockPlcService, SqliteOrmPersistenceService};
    use crate::services::infrastructure::plc::plc_communication_service::PlcCommunicationService;
    use crate::services::infrastructure::persistence::persistence_service::PersistenceConfig;
    use crate::models::{ModuleType, PointDataType};
    use std::sync::Arc;
    use std::path::Path;

    /// 创建测试用的服务依赖
    async fn create_test_services() -> (
        Arc<dyn IChannelStateManager>,
        Arc<dyn ITestExecutionEngine>,
        Arc<dyn IPersistenceService>,
    ) {
        // 创建Mock PLC服务
        let mut mock_test_rig = MockPlcService::new_for_testing("TestRig");
        let mut mock_target = MockPlcService::new_for_testing("Target");
        mock_test_rig.connect().await.unwrap();
        mock_target.connect().await.unwrap();

        // 创建持久化服务配置
        let config = PersistenceConfig::default();
        
        // 创建内存数据库持久化服务
        let persistence_service = Arc::new(
            SqliteOrmPersistenceService::new(config, Some(Path::new(":memory:"))).await.unwrap()
        );

        // 创建通道状态管理器
        let channel_state_manager = Arc::new(
            ChannelStateManager::new(persistence_service.clone())
        );

        // 创建测试执行引擎
        let test_execution_engine = Arc::new(
            TestExecutionEngine::new(
                5, // 最大并发数
                Arc::new(mock_test_rig),
                Arc::new(mock_target),
            )
        );

        (channel_state_manager, test_execution_engine, persistence_service)
    }

    /// 创建测试用的通道定义
    fn create_test_channel_definition() -> ChannelPointDefinition {
        let mut definition = ChannelPointDefinition::new(
            "AI001".to_string(),
            "Temperature_1".to_string(),
            "温度传感器1".to_string(),
            "Station1".to_string(),
            "Module1".to_string(),
            ModuleType::AI,
            "CH01".to_string(),
            PointDataType::Float,
            "DB1.DBD0".to_string(),
        );
        
        definition.range_lower_limit = Some(0.0);
        definition.range_upper_limit = Some(100.0);
        definition.test_rig_plc_address = Some("DB2.DBD0".to_string());
        
        definition
    }

    /// 创建测试用的批次信息
    fn create_test_batch_info() -> TestBatchInfo {
        TestBatchInfo::new(
            Some("TestProduct".to_string()),
            Some("SN001".to_string()),
        )
    }

    #[tokio::test]
    async fn test_submit_test_execution_success() {
        let (channel_state_manager, test_execution_engine, persistence_service) = 
            create_test_services().await;

        let coordination_service = TestCoordinationService::new(
            channel_state_manager,
            test_execution_engine,
            persistence_service,
        );

        let request = TestExecutionRequest {
            batch_info: create_test_batch_info(),
            channel_definitions: vec![create_test_channel_definition()],
            max_concurrent_tests: Some(5),
            auto_start: false,
        };

        let response = coordination_service.submit_test_execution(request).await;
        
        assert!(response.is_ok());
        let response = response.unwrap();
        assert_eq!(response.instance_count, 1);
        assert_eq!(response.status, "submitted");
        assert!(response.message.contains("成功提交 1 个测试实例"));
    }

    #[tokio::test]
    async fn test_submit_test_execution_with_auto_start() {
        let (channel_state_manager, test_execution_engine, persistence_service) = 
            create_test_services().await;

        let coordination_service = TestCoordinationService::new(
            channel_state_manager,
            test_execution_engine,
            persistence_service,
        );

        let request = TestExecutionRequest {
            batch_info: create_test_batch_info(),
            channel_definitions: vec![create_test_channel_definition()],
            max_concurrent_tests: Some(5),
            auto_start: true,
        };

        let response = coordination_service.submit_test_execution(request).await;
        
        assert!(response.is_ok());
        let response = response.unwrap();
        assert_eq!(response.instance_count, 1);
        assert_eq!(response.status, "running");
        assert!(response.message.contains("并开始执行"));
    }

    #[tokio::test]
    async fn test_submit_empty_channel_definitions() {
        let (channel_state_manager, test_execution_engine, persistence_service) = 
            create_test_services().await;

        let coordination_service = TestCoordinationService::new(
            channel_state_manager,
            test_execution_engine,
            persistence_service,
        );

        let request = TestExecutionRequest {
            batch_info: create_test_batch_info(),
            channel_definitions: vec![], // 空的定义列表
            max_concurrent_tests: Some(5),
            auto_start: false,
        };

        let response = coordination_service.submit_test_execution(request).await;
        
        assert!(response.is_err());
        let error = response.unwrap_err();
        assert!(error.to_string().contains("通道定义列表不能为空"));
    }

    #[tokio::test]
    async fn test_batch_lifecycle() {
        let (channel_state_manager, test_execution_engine, persistence_service) = 
            create_test_services().await;

        let coordination_service = TestCoordinationService::new(
            channel_state_manager,
            test_execution_engine,
            persistence_service,
        );

        let batch_info = create_test_batch_info();
        let batch_id = batch_info.batch_id.clone();

        // 1. 提交测试
        let request = TestExecutionRequest {
            batch_info,
            channel_definitions: vec![create_test_channel_definition()],
            max_concurrent_tests: Some(5),
            auto_start: false,
        };

        let response = coordination_service.submit_test_execution(request).await;
        assert!(response.is_ok());

        // 2. 开始测试
        let result = coordination_service.start_batch_testing(&batch_id).await;
        assert!(result.is_ok());

        // 3. 获取进度
        let progress = coordination_service.get_batch_progress(&batch_id).await;
        assert!(progress.is_ok());
        let progress = progress.unwrap();
        assert_eq!(progress.len(), 1);

        // 4. 暂停测试
        let result = coordination_service.pause_batch_testing(&batch_id).await;
        assert!(result.is_ok());

        // 5. 恢复测试
        let result = coordination_service.resume_batch_testing(&batch_id).await;
        assert!(result.is_ok());

        // 6. 停止测试
        let result = coordination_service.stop_batch_testing(&batch_id).await;
        assert!(result.is_ok());

        // 7. 清理批次
        let result = coordination_service.cleanup_completed_batch(&batch_id).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_batch_results() {
        let (channel_state_manager, test_execution_engine, persistence_service) = 
            create_test_services().await;

        let coordination_service = TestCoordinationService::new(
            channel_state_manager,
            test_execution_engine,
            persistence_service,
        );

        let batch_info = create_test_batch_info();
        let batch_id = batch_info.batch_id.clone();

        // 提交测试
        let request = TestExecutionRequest {
            batch_info,
            channel_definitions: vec![create_test_channel_definition()],
            max_concurrent_tests: Some(5),
            auto_start: false,
        };

        coordination_service.submit_test_execution(request).await.unwrap();

        // 获取结果（应该为空，因为还没有执行）
        let results = coordination_service.get_batch_results(&batch_id).await;
        assert!(results.is_ok());
        let results = results.unwrap();
        assert_eq!(results.len(), 0);
    }

    #[tokio::test]
    async fn test_batch_execution_status_transitions() {
        let batch_info = create_test_batch_info();
        let channel_definitions = vec![create_test_channel_definition()];
        
        let mut batch_execution = BatchExecutionInfo::new(batch_info, channel_definitions);
        
        // 初始状态应该是Submitted
        assert_eq!(batch_execution.status, BatchExecutionStatus::Submitted);
        
        // 模拟状态转换
        batch_execution.status = BatchExecutionStatus::Running;
        assert_eq!(batch_execution.status, BatchExecutionStatus::Running);
        
        batch_execution.status = BatchExecutionStatus::Completed;
        assert_eq!(batch_execution.status, BatchExecutionStatus::Completed);
    }

    #[tokio::test]
    async fn test_estimate_total_sub_tests() {
        let mut ai_definition = create_test_channel_definition();
        ai_definition.module_type = ModuleType::AI;
        ai_definition.test_rig_plc_address = Some("DB2.DBD0".to_string());
        ai_definition.sh_set_point_address = Some("DB1.DBD4".to_string());
        ai_definition.sl_set_point_address = Some("DB1.DBD8".to_string());

        let batch_info = create_test_batch_info();
        let batch_execution = BatchExecutionInfo::new(
            batch_info,
            vec![ai_definition.clone()],
        );

        // AI点应该有7个子测试：5个硬点 + 2个报警
        let count = batch_execution.estimate_total_sub_tests(&ai_definition.id);
        assert_eq!(count, 7);

        // DI点应该有1个子测试
        let mut di_definition = create_test_channel_definition();
        di_definition.module_type = ModuleType::DI;
        di_definition.data_type = PointDataType::Bool;
        
        let batch_execution = BatchExecutionInfo::new(
            create_test_batch_info(),
            vec![di_definition.clone()],
        );
        
        let count = batch_execution.estimate_total_sub_tests(&di_definition.id);
        assert_eq!(count, 1);
    }
} 