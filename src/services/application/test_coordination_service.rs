/// 测试协调服务
/// 
/// 负责协调整个测试流程，包括：
/// 1. 接收测试请求并验证
/// 2. 协调通道状态管理器和测试执行引擎
/// 3. 管理测试进度和结果收集
/// 4. 提供统一的测试API

use crate::models::{
    ChannelTestInstance, ChannelPointDefinition, RawTestOutcome, TestBatchInfo, 
    OverallTestStatus, SubTestExecutionResult
};
use crate::services::domain::{
    IChannelStateManager, ITestExecutionEngine, TaskStatus
};
use crate::services::infrastructure::{
    IPersistenceService, IPlcCommunicationService
};
use crate::utils::error::{AppError, AppResult};
use async_trait::async_trait;
use std::sync::Arc;
use std::collections::HashMap;
use tokio::sync::{mpsc, Mutex, RwLock};
use serde::{Serialize, Deserialize};
use log::{debug, warn, error, info};

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

    /// 订阅测试进度更新
    async fn subscribe_progress_updates(&self) -> AppResult<mpsc::Receiver<TestProgressUpdate>>;

    /// 获取批次测试结果
    async fn get_batch_results(&self, batch_id: &str) -> AppResult<Vec<RawTestOutcome>>;

    /// 清理完成的批次
    async fn cleanup_completed_batch(&self, batch_id: &str) -> AppResult<()>;
}

/// 批次执行状态
#[derive(Debug, Clone)]
struct BatchExecutionState {
    /// 批次信息
    batch_info: TestBatchInfo,
    /// 测试实例映射 (instance_id -> (definition, task_id))
    instances: HashMap<String, (ChannelPointDefinition, Option<String>)>,
    /// 进度更新发送器
    progress_senders: Vec<mpsc::Sender<TestProgressUpdate>>,
    /// 结果收集器
    results: Vec<RawTestOutcome>,
    /// 批次状态
    status: String,
}

/// 测试协调服务实现
pub struct TestCoordinationService {
    /// 通道状态管理器
    channel_state_manager: Arc<dyn IChannelStateManager>,
    /// 测试执行引擎
    test_execution_engine: Arc<dyn ITestExecutionEngine>,
    /// 持久化服务
    persistence_service: Arc<dyn IPersistenceService>,
    /// 活动批次状态
    active_batches: Arc<RwLock<HashMap<String, BatchExecutionState>>>,
    /// 全局进度更新发送器
    global_progress_sender: Arc<Mutex<Option<mpsc::Sender<TestProgressUpdate>>>>,
    /// 服务名称
    service_name: String,
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
            global_progress_sender: Arc::new(Mutex::new(None)),
            service_name: "TestCoordinationService".to_string(),
        }
    }

    /// 创建测试实例
    async fn create_test_instances(
        &self,
        batch_info: &TestBatchInfo,
        definitions: &[ChannelPointDefinition],
    ) -> AppResult<Vec<ChannelTestInstance>> {
        let mut instances = Vec::new();

        for definition in definitions {
            // 使用通道状态管理器创建测试实例
            let instance = self.channel_state_manager
                .create_test_instance(definition.id.clone(), batch_info.batch_id.clone())
                .await?;
            
            instances.push(instance);
        }

        info!("[{}] 为批次 {} 创建了 {} 个测试实例", 
              self.service_name, batch_info.batch_id, instances.len());

        Ok(instances)
    }

    /// 启动测试实例执行
    async fn start_instance_execution(
        &self,
        instance: ChannelTestInstance,
        definition: ChannelPointDefinition,
        batch_id: String,
    ) -> AppResult<String> {
        // 创建结果接收通道
        let (tx, mut rx) = mpsc::channel(1000);
        
        // 提交到测试执行引擎
        let task_id = self.test_execution_engine
            .submit_test_instance(instance.clone(), definition.clone(), tx)
            .await?;

        // 启动结果处理任务
        let coordination_service = Arc::new(self);
        let instance_id = instance.instance_id.clone();
        let definition_tag = definition.tag.clone();
        
        tokio::spawn(async move {
            while let Some(result) = rx.recv().await {
                match result {
                    Ok(outcome) => {
                        debug!("[TestCoordinationService] 收到测试结果 - 实例: {}, 成功: {}", 
                               instance_id, outcome.success);
                        
                        // 更新通道状态
                        if let Err(e) = coordination_service.channel_state_manager
                            .update_test_result(&instance_id, outcome.clone()).await {
                            error!("[TestCoordinationService] 更新测试结果失败: {}", e);
                        }

                        // 发送进度更新
                        coordination_service.send_progress_update(
                            batch_id.clone(),
                            instance_id.clone(),
                            definition_tag.clone(),
                            Some(outcome),
                        ).await;
                    },
                    Err(e) => {
                        error!("[TestCoordinationService] 测试执行错误 - 实例: {}, 错误: {}", 
                               instance_id, e);
                        
                        // 发送错误进度更新
                        coordination_service.send_progress_update(
                            batch_id.clone(),
                            instance_id.clone(),
                            definition_tag.clone(),
                            None,
                        ).await;
                    }
                }
            }
        });

        Ok(task_id)
    }

    /// 发送进度更新
    async fn send_progress_update(
        &self,
        batch_id: String,
        instance_id: String,
        point_tag: String,
        latest_result: Option<RawTestOutcome>,
    ) {
        // 获取实例当前状态
        let instance_state = match self.channel_state_manager
            .get_instance_state(&instance_id).await {
            Ok(state) => state,
            Err(e) => {
                error!("[{}] 获取实例状态失败: {}", self.service_name, e);
                return;
            }
        };

        let progress_update = TestProgressUpdate {
            batch_id: batch_id.clone(),
            instance_id: instance_id.clone(),
            point_tag,
            overall_status: instance_state.overall_status,
            completed_sub_tests: instance_state.sub_test_results.len(),
            total_sub_tests: instance_state.sub_test_results.len() + 
                            instance_state.pending_sub_tests.len(),
            latest_result,
            timestamp: chrono::Utc::now(),
        };

        // 发送到批次特定的发送器
        {
            let batches = self.active_batches.read().await;
            if let Some(batch_state) = batches.get(&batch_id) {
                for sender in &batch_state.progress_senders {
                    if let Err(e) = sender.send(progress_update.clone()).await {
                        debug!("[{}] 发送进度更新失败: {}", self.service_name, e);
                    }
                }
            }
        }

        // 发送到全局发送器
        {
            let global_sender = self.global_progress_sender.lock().await;
            if let Some(sender) = global_sender.as_ref() {
                if let Err(e) = sender.send(progress_update).await {
                    debug!("[{}] 发送全局进度更新失败: {}", self.service_name, e);
                }
            }
        }
    }

    /// 收集批次结果
    async fn collect_batch_results(&self, batch_id: &str) -> AppResult<Vec<RawTestOutcome>> {
        let batches = self.active_batches.read().await;
        if let Some(batch_state) = batches.get(batch_id) {
            Ok(batch_state.results.clone())
        } else {
            Err(AppError::ValidationError {
                field: "batch_id".to_string(),
                message: format!("批次 {} 不存在", batch_id),
            })
        }
    }
}

#[async_trait]
impl ITestCoordinationService for TestCoordinationService {
    async fn submit_test_execution(
        &self,
        request: TestExecutionRequest,
    ) -> AppResult<TestExecutionResponse> {
        let batch_id = request.batch_info.batch_id.clone();
        
        info!("[{}] 提交测试执行请求 - 批次: {}, 通道数: {}", 
              self.service_name, batch_id, request.channel_definitions.len());

        // 验证请求
        if request.channel_definitions.is_empty() {
            return Err(AppError::ValidationError {
                field: "channel_definitions".to_string(),
                message: "通道定义列表不能为空".to_string(),
            });
        }

        // 保存批次信息
        self.persistence_service.save_batch_info(&request.batch_info).await?;

        // 保存通道定义
        for definition in &request.channel_definitions {
            self.persistence_service.save_channel_definition(definition).await?;
        }

        // 创建测试实例
        let instances = self.create_test_instances(
            &request.batch_info, 
            &request.channel_definitions
        ).await?;

        // 保存测试实例
        for instance in &instances {
            self.persistence_service.save_test_instance(instance).await?;
        }

        // 创建批次执行状态
        let mut instance_map = HashMap::new();
        for (instance, definition) in instances.iter().zip(request.channel_definitions.iter()) {
            instance_map.insert(
                instance.instance_id.clone(), 
                (definition.clone(), None)
            );
        }

        let batch_state = BatchExecutionState {
            batch_info: request.batch_info.clone(),
            instances: instance_map,
            progress_senders: Vec::new(),
            results: Vec::new(),
            status: "created".to_string(),
        };

        // 添加到活动批次
        {
            let mut batches = self.active_batches.write().await;
            batches.insert(batch_id.clone(), batch_state);
        }

        // 如果设置了自动开始，立即开始测试
        if request.auto_start {
            self.start_batch_testing(&batch_id).await?;
        }

        Ok(TestExecutionResponse {
            batch_id,
            instance_count: instances.len(),
            status: if request.auto_start { "started" } else { "created" }.to_string(),
            message: format!("成功创建 {} 个测试实例", instances.len()),
        })
    }

    async fn start_batch_testing(&self, batch_id: &str) -> AppResult<()> {
        info!("[{}] 开始批次测试 - 批次: {}", self.service_name, batch_id);

        let instances_to_start = {
            let mut batches = self.active_batches.write().await;
            let batch_state = batches.get_mut(batch_id)
                .ok_or_else(|| AppError::ValidationError {
                    field: "batch_id".to_string(),
                    message: format!("批次 {} 不存在", batch_id),
                })?;

            batch_state.status = "running".to_string();
            batch_state.instances.clone()
        };

        // 启动所有实例的测试
        for (instance_id, (definition, _)) in instances_to_start {
            // 获取实例状态
            let instance = self.channel_state_manager
                .get_instance_state(&instance_id).await?;

            // 启动测试执行
            let task_id = self.start_instance_execution(
                instance, 
                definition, 
                batch_id.to_string()
            ).await?;

            // 更新任务ID
            {
                let mut batches = self.active_batches.write().await;
                if let Some(batch_state) = batches.get_mut(batch_id) {
                    if let Some((_, task_id_ref)) = batch_state.instances.get_mut(&instance_id) {
                        *task_id_ref = Some(task_id);
                    }
                }
            }
        }

        info!("[{}] 批次测试已启动 - 批次: {}", self.service_name, batch_id);
        Ok(())
    }

    async fn pause_batch_testing(&self, batch_id: &str) -> AppResult<()> {
        info!("[{}] 暂停批次测试 - 批次: {}", self.service_name, batch_id);

        let instances_to_pause = {
            let mut batches = self.active_batches.write().await;
            let batch_state = batches.get_mut(batch_id)
                .ok_or_else(|| AppError::ValidationError {
                    field: "batch_id".to_string(),
                    message: format!("批次 {} 不存在", batch_id),
                })?;

            batch_state.status = "paused".to_string();
            batch_state.instances.keys().cloned().collect::<Vec<_>>()
        };

        // 暂停所有实例的执行
        for instance_id in instances_to_pause {
            if let Err(e) = self.test_execution_engine
                .pause_instance_execution(&instance_id).await {
                warn!("[{}] 暂停实例 {} 失败: {}", self.service_name, instance_id, e);
            }
        }

        Ok(())
    }

    async fn resume_batch_testing(&self, batch_id: &str) -> AppResult<()> {
        info!("[{}] 恢复批次测试 - 批次: {}", self.service_name, batch_id);

        let instances_to_resume = {
            let mut batches = self.active_batches.write().await;
            let batch_state = batches.get_mut(batch_id)
                .ok_or_else(|| AppError::ValidationError {
                    field: "batch_id".to_string(),
                    message: format!("批次 {} 不存在", batch_id),
                })?;

            batch_state.status = "running".to_string();
            batch_state.instances.keys().cloned().collect::<Vec<_>>()
        };

        // 恢复所有实例的执行
        for instance_id in instances_to_resume {
            if let Err(e) = self.test_execution_engine
                .resume_instance_execution(&instance_id).await {
                warn!("[{}] 恢复实例 {} 失败: {}", self.service_name, instance_id, e);
            }
        }

        Ok(())
    }

    async fn stop_batch_testing(&self, batch_id: &str) -> AppResult<()> {
        info!("[{}] 停止批次测试 - 批次: {}", self.service_name, batch_id);

        let instances_to_stop = {
            let mut batches = self.active_batches.write().await;
            let batch_state = batches.get_mut(batch_id)
                .ok_or_else(|| AppError::ValidationError {
                    field: "batch_id".to_string(),
                    message: format!("批次 {} 不存在", batch_id),
                })?;

            batch_state.status = "stopped".to_string();
            batch_state.instances.keys().cloned().collect::<Vec<_>>()
        };

        // 停止所有实例的执行
        for instance_id in instances_to_stop {
            if let Err(e) = self.test_execution_engine
                .stop_instance_execution(&instance_id).await {
                warn!("[{}] 停止实例 {} 失败: {}", self.service_name, instance_id, e);
            }
        }

        Ok(())
    }

    async fn get_batch_progress(&self, batch_id: &str) -> AppResult<Vec<TestProgressUpdate>> {
        let batches = self.active_batches.read().await;
        let batch_state = batches.get(batch_id)
            .ok_or_else(|| AppError::ValidationError {
                field: "batch_id".to_string(),
                message: format!("批次 {} 不存在", batch_id),
            })?;

        let mut progress_updates = Vec::new();

        for (instance_id, (definition, _)) in &batch_state.instances {
            // 获取实例当前状态
            let instance_state = self.channel_state_manager
                .get_instance_state(instance_id).await?;

            let progress_update = TestProgressUpdate {
                batch_id: batch_id.to_string(),
                instance_id: instance_id.clone(),
                point_tag: definition.tag.clone(),
                overall_status: instance_state.overall_status,
                completed_sub_tests: instance_state.sub_test_results.len(),
                total_sub_tests: instance_state.sub_test_results.len() + 
                                instance_state.pending_sub_tests.len(),
                latest_result: instance_state.sub_test_results.last().map(|r| r.outcome.clone()),
                timestamp: chrono::Utc::now(),
            };

            progress_updates.push(progress_update);
        }

        Ok(progress_updates)
    }

    async fn subscribe_progress_updates(&self) -> AppResult<mpsc::Receiver<TestProgressUpdate>> {
        let (tx, rx) = mpsc::channel(1000);
        
        // 设置全局进度发送器
        {
            let mut global_sender = self.global_progress_sender.lock().await;
            *global_sender = Some(tx);
        }

        Ok(rx)
    }

    async fn get_batch_results(&self, batch_id: &str) -> AppResult<Vec<RawTestOutcome>> {
        self.collect_batch_results(batch_id).await
    }

    async fn cleanup_completed_batch(&self, batch_id: &str) -> AppResult<()> {
        info!("[{}] 清理完成的批次 - 批次: {}", self.service_name, batch_id);

        // 从活动批次中移除
        {
            let mut batches = self.active_batches.write().await;
            batches.remove(batch_id);
        }

        info!("[{}] 批次清理完成 - 批次: {}", self.service_name, batch_id);
        Ok(())
    }
}

// 重新导出常用类型
pub use {
    ITestCoordinationService,
    TestCoordinationService,
    TestExecutionRequest,
    TestExecutionResponse,
    TestProgressUpdate,
}; 