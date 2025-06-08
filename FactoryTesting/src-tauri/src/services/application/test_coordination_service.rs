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
use tokio::sync::{mpsc, Mutex, Semaphore};
use serde::{Serialize, Deserialize};
use log::{debug, warn, error, info, trace};
use chrono::Utc;
use crate::services::{IChannelAllocationService};
use crate::services::traits::EventPublisher;
use tokio_util::sync::CancellationToken;

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
    /// 主批次ID（第一个批次，为了向后兼容）
    pub batch_id: String,
    /// 所有生成的批次信息
    pub all_batches: Vec<TestBatchInfo>,
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

    /// 加载现有批次到活动列表
    async fn load_existing_batch(&self, batch_id: &str) -> AppResult<()>;

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
///
/// 负责协调整个测试流程，包括批次管理、任务调度、状态监控等
/// 参考原始C#代码的TestTaskManager复杂度
pub struct TestCoordinationService {
    /// 通道状态管理器
    channel_state_manager: Arc<dyn IChannelStateManager>,
    /// 测试执行引擎
    test_execution_engine: Arc<dyn ITestExecutionEngine>,
    /// 持久化服务
    persistence_service: Arc<dyn IPersistenceService>,
    /// 事件发布器
    event_publisher: Arc<dyn EventPublisher>,
    /// 通道分配服务
    channel_allocation_service: Arc<dyn crate::services::channel_allocation_service::IChannelAllocationService>,
    /// 测试PLC配置服务
    test_plc_config_service: Arc<dyn crate::services::domain::test_plc_config_service::ITestPlcConfigService>,
    /// 当前活跃的批次
    active_batches: Arc<Mutex<HashMap<String, BatchExecutionInfo>>>,
    /// 测试进度缓存
    progress_cache: Arc<Mutex<HashMap<String, TestProgressUpdate>>>,
    /// 并发控制信号量
    concurrency_semaphore: Arc<Semaphore>,
    /// 全局取消令牌
    cancellation_token: Arc<Mutex<Option<CancellationToken>>>,
}

impl TestCoordinationService {
    /// 创建新的测试协调服务
    pub fn new(
        channel_state_manager: Arc<dyn IChannelStateManager>,
        test_execution_engine: Arc<dyn ITestExecutionEngine>,
        persistence_service: Arc<dyn IPersistenceService>,
        event_publisher: Arc<dyn EventPublisher>,
        channel_allocation_service: Arc<dyn crate::services::channel_allocation_service::IChannelAllocationService>,
        test_plc_config_service: Arc<dyn crate::services::domain::test_plc_config_service::ITestPlcConfigService>,
    ) -> Self {
        Self {
            channel_state_manager,
            test_execution_engine,
            persistence_service,
            event_publisher,
            channel_allocation_service,
            test_plc_config_service,
            active_batches: Arc::new(Mutex::new(HashMap::new())),
            progress_cache: Arc::new(Mutex::new(HashMap::new())),
            concurrency_semaphore: Arc::new(Semaphore::new(5)),
            cancellation_token: Arc::new(Mutex::new(None)),
        }
    }

    /// 启动结果收集任务
    async fn start_result_collection(&self, batch_id: String) -> AppResult<()> {
        let active_batches = self.active_batches.clone();
        let persistence_service = self.persistence_service.clone();
        let channel_state_manager = self.channel_state_manager.clone();
        let event_publisher = self.event_publisher.clone();

        tokio::spawn(async move {
            let receiver = {
                let mut batches = active_batches.lock().await;
                if let Some(batch_info) = batches.get_mut(&batch_id) {
                    batch_info.result_receiver.take()
                } else {
                    return;
                }
            };

            if let Some(mut receiver) = receiver {
                while let Some(result) = receiver.recv().await {
                    // 移除冗余的测试结果接收日志

                    // 保存结果到持久化存储
                    if let Err(e) = persistence_service.save_test_outcome(&result).await {
                        // 🔧 移除 [TestCoordination] 日志
                    }

                    // ===== 关键修复：更新 ChannelStateManager 中的测试实例状态 =====
                    if let Err(e) = channel_state_manager.update_test_result(result.clone()).await {
                        // 🔧 移除 [TestCoordination] 日志
                    } else {
                        // 🔧 移除 [TestCoordination] 日志

                        // ===== 新增：发布测试完成事件到前端 =====
                        if let Err(e) = event_publisher.publish_test_completed(&result).await {
                            // 🔧 移除 [TestCoordination] 日志
                        } else {
                            // 🔧 移除 [TestCoordination] 日志
                        }
                    }

                    // 更新批次信息中的结果集合
                    {
                        let mut batches = active_batches.lock().await;
                        if let Some(batch_info) = batches.get_mut(&batch_id) {
                            batch_info.collected_results.push(result);

                            // 计算批次统计信息
                            let total_instances = batch_info.test_instances.len();
                            let mut tested_instances = 0;
                            let mut passed_instances = 0;
                            let mut failed_instances = 0;
                            let mut skipped_instances = 0;
                            let mut in_progress_instances = 0;

                            // 统计每个实例的测试结果
                            let instance_results = batch_info.collected_results.iter()
                                .fold(std::collections::HashMap::new(), |mut map, result| {
                                    map.entry(result.channel_instance_id.clone())
                                        .or_insert_with(Vec::new)
                                        .push(result);
                                    map
                                });

                            // 计算已测试的实例数
                            for instance in &batch_info.test_instances {
                                if let Some(results) = instance_results.get(&instance.instance_id) {
                                    // 如果有硬点测试结果，则认为已测试
                                    let has_hardpoint_test = results.iter()
                                        .any(|r| r.sub_test_item == crate::models::enums::SubTestItem::HardPoint);

                                    if has_hardpoint_test {
                                        tested_instances += 1;

                                        // 判断通过/失败
                                        let all_success = results.iter()
                                            .filter(|r| r.sub_test_item == crate::models::enums::SubTestItem::HardPoint)
                                            .all(|r| r.success);

                                        if all_success {
                                            passed_instances += 1;
                                        } else {
                                            failed_instances += 1;
                                        }
                                    } else {
                                        in_progress_instances += 1;
                                    }
                                } else {
                                    // 没有任何测试结果
                                    skipped_instances += 1;
                                }
                            }

                            // 创建批次统计信息
                            let batch_statistics = crate::services::traits::BatchStatistics {
                                total_channels: total_instances as u32,
                                tested_channels: tested_instances as u32,
                                passed_channels: passed_instances as u32,
                                failed_channels: failed_instances as u32,
                                skipped_channels: skipped_instances as u32,
                                in_progress_channels: in_progress_instances as u32,
                                start_time: batch_info.started_at,
                                end_time: None,
                                estimated_completion_time: None,
                            };

                            // 发布批次状态变化事件
                            let batch_id_clone = batch_id.clone();
                            let event_publisher_clone = event_publisher.clone();
                            let statistics_clone = batch_statistics.clone();

                            tokio::spawn(async move {
                                if let Err(e) = event_publisher_clone.publish_batch_status_changed(&batch_id_clone, &statistics_clone).await {
                                    // 🔧 移除 [TestCoordination] 日志
                                } else {
                                    // 🔧 移除 [TestCoordination] 日志
                                }
                            });

                            // 检查批次是否完成
                            if tested_instances + skipped_instances >= total_instances {
                                batch_info.status = BatchExecutionStatus::Completed;
                                batch_info.completed_at = Some(Utc::now());
                                // 🔧 移除 [TestCoordination] 日志
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
        // 🔧 移除 [TestCoordination] 日志

        // 验证请求
        if request.channel_definitions.is_empty() {
            // 🔧 移除 [TestCoordination] 日志
            return Err(AppError::validation_error("通道定义列表不能为空"));
        }

        log::info!("[TestCoordination] 开始保存批次信息到数据库...");
        // 保存批次信息
        self.persistence_service
            .save_batch_info(&request.batch_info)
            .await?;
        log::info!("[TestCoordination] 批次信息保存成功");

        // ===== 使用通道分配服务来正确分配批次 =====
        log::info!("[TestCoordination] ===== 开始使用通道分配服务分配通道 =====");

        // 详细记录输入的通道定义
        let mut type_counts = std::collections::HashMap::new();
        for def in &request.channel_definitions {
            let key = format!("{:?}_{}", def.module_type, def.power_supply_type);
            *type_counts.entry(key).or_insert(0) += 1;
        }

        log::info!("[TestCoordination] 输入通道详情:");
        for (type_name, count) in &type_counts {
            log::info!("[TestCoordination]   {}: {} 个", type_name, count);
        }

        // 获取真实的测试PLC配置
        use crate::services::channel_allocation_service::TestPlcConfig;
        let test_plc_config = match self.test_plc_config_service.get_test_plc_config().await {
            Ok(config) => config,
            Err(e) => {
                warn!("[TestCoordination] 获取测试PLC配置失败，使用默认配置: {}", e);
                TestPlcConfig {
                    brand_type: "ModbusTcp".to_string(),
                    ip_address: "127.0.0.1".to_string(),
                    comparison_tables: Vec::new(),
                }
            }
        };

        log::info!("[TestCoordination] 测试PLC配置: 品牌={}, IP={}, 映射表数量={}",
            test_plc_config.brand_type, test_plc_config.ip_address, test_plc_config.comparison_tables.len());

        // 调用通道分配服务
        log::info!("[TestCoordination] 正在调用通道分配服务...");
        let allocation_result = self.channel_allocation_service
            .allocate_channels(
                request.channel_definitions.clone(),
                test_plc_config,
                request.batch_info.product_model.clone(),
                request.batch_info.serial_number.clone(),
            )
            .await?;



        // 🔧 通道分配服务已经将数据保存到数据库，无需额外保存到状态管理器
        log::info!("[TestCoordination] 通道分配完成，数据已保存到数据库");

        // 为每个分配的批次创建批次执行信息
        let mut total_instance_count = 0;
        for batch in &allocation_result.batches {
            // 🔧 从状态管理器获取属于此批次的实例（而不是使用分配服务的临时实例）
            let batch_instances = self.persistence_service
                .load_test_instances_by_batch(&batch.batch_id)
                .await?;

            info!("[TestCoordination] 批次 {} 从状态管理器加载了 {} 个实例",
                  batch.batch_name, batch_instances.len());

            // 创建批次执行信息
            let mut batch_execution = BatchExecutionInfo::new(
                batch.clone(),
                request.channel_definitions.clone(),
            );

            // 设置测试实例（使用从状态管理器加载的实例）
            batch_execution.test_instances = batch_instances;
            total_instance_count += batch_execution.test_instances.len();

            // 启动结果收集任务
            self.start_result_collection(batch.batch_id.clone()).await?;

            // 添加到活动批次
            {
                let mut batches = self.active_batches.lock().await;
                batches.insert(batch.batch_id.clone(), batch_execution);
            }
        }

        // 如果设置了自动开始，立即启动所有批次的测试
        if request.auto_start {
            for batch in &allocation_result.batches {
                if let Err(e) = self.start_batch_testing(&batch.batch_id).await {
                    warn!("[TestCoordination] 启动批次 {} 失败: {}", batch.batch_id, e);
                }
            }
        }

        // 返回响应，包含所有批次信息
        let primary_batch = allocation_result.batches.first()
            .ok_or_else(|| AppError::generic("没有生成任何批次"))?;

        let batches_count = allocation_result.batches.len();
        let batches_list = allocation_result.batches.iter()
            .map(|b| format!("{}({}个点位)", b.batch_name, b.total_points))
            .collect::<Vec<_>>()
            .join(", ");

        Ok(TestExecutionResponse {
            batch_id: primary_batch.batch_id.clone(),
            all_batches: allocation_result.batches,
            instance_count: total_instance_count,
            status: if request.auto_start { "running" } else { "submitted" }.to_string(),
            message: format!("成功分配 {} 个批次，共 {} 个测试实例{}。批次列表: {}",
                           batches_count,
                           total_instance_count,
                           if request.auto_start { "并开始执行" } else { "" },
                           batches_list),
        })
    }

    /// 加载现有批次到活动列表
    async fn load_existing_batch(&self, batch_id: &str) -> AppResult<()> {
        info!("[TestCoordination] 加载现有批次到活动列表: {}", batch_id);

        // 检查批次是否已经在活动列表中
        {
            let batches = self.active_batches.lock().await;
            if batches.contains_key(batch_id) {
                info!("[TestCoordination] 批次 {} 已在活动列表中", batch_id);
                return Ok(());
            }
        }

        // 从数据库加载批次信息
        let batch_info = self.persistence_service
            .load_batch_info(batch_id)
            .await?
            .ok_or_else(|| AppError::not_found_error("批次", batch_id))?;

        // 从数据库加载测试实例
        let test_instances = self.persistence_service
            .load_test_instances_by_batch(batch_id)
            .await?;

        if test_instances.is_empty() {
            return Err(AppError::validation_error(
                format!("批次 {} 中没有测试实例", batch_id)
            ));
        }

        // 从数据库加载通道定义
        let mut channel_definitions = Vec::new();
        for instance in &test_instances {
            if let Some(definition) = self.channel_state_manager
                .get_channel_definition(&instance.definition_id)
                .await
            {
                channel_definitions.push(definition);
            } else {
                warn!("[TestCoordination] 未找到通道定义: {}", instance.definition_id);
            }
        }

        if channel_definitions.is_empty() {
            return Err(AppError::validation_error(
                format!("批次 {} 中没有找到通道定义", batch_id)
            ));
        }

        // 创建批次执行信息
        let mut batch_execution = BatchExecutionInfo::new(
            batch_info.clone(),
            channel_definitions,
        );

        // 设置测试实例
        batch_execution.test_instances = test_instances.clone();

        // 启动结果收集任务
        self.start_result_collection(batch_id.to_string()).await?;

        // 添加到活动批次
        {
            let mut batches = self.active_batches.lock().await;
            batches.insert(batch_id.to_string(), batch_execution);
        }

        info!("[TestCoordination] 批次 {} 已加载到活动列表，包含 {} 个测试实例",
              batch_id, test_instances.len());

        Ok(())
    }

    /// 开始指定批次的测试
    async fn start_batch_testing(&self, batch_id: &str) -> AppResult<()> {
        info!("[TestCoordination] 开始批次测试: {}", batch_id);

        // 首先检查批次是否在活动列表中，如果不在则返回错误
        {
            let batches = self.active_batches.lock().await;
            if !batches.contains_key(batch_id) {
                return Err(AppError::validation_error(
                    format!("批次 {} 不在活动列表中，请先创建或加载批次", batch_id)
                ));
            }
        }

        let (instances, definitions, result_sender) = {
            let mut batches = self.active_batches.lock().await;
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

                // ===== 新增：发布测试开始事件到前端 =====
                if let Err(e) = self.event_publisher.publish_test_status_changed(
                    &instance.instance_id,
                    crate::models::enums::OverallTestStatus::NotTested,
                    crate::models::enums::OverallTestStatus::HardPointTesting,
                ).await {
                    error!("[TestCoordination] 发布测试开始事件失败: {}", e);
                } else {
                    trace!("[TestCoordination] 成功发布测试开始事件: {}", instance.instance_id);
                }

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
                    let mut batches = self.active_batches.lock().await;
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

        let mut batches = self.active_batches.lock().await;
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

        let batches = self.active_batches.lock().await;
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

        let mut batches = self.active_batches.lock().await;
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
        let batches = self.active_batches.lock().await;
        let batch_info = batches.get(batch_id)
            .ok_or_else(|| AppError::not_found_error("批次", batch_id))?;

        Ok(batch_info.get_progress())
    }

    /// 获取批次测试结果
    async fn get_batch_results(&self, batch_id: &str) -> AppResult<Vec<RawTestOutcome>> {
        let batches = self.active_batches.lock().await;
        let batch_info = batches.get(batch_id)
            .ok_or_else(|| AppError::not_found_error("批次", batch_id))?;

        Ok(batch_info.collected_results.clone())
    }

    /// 清理完成的批次
    async fn cleanup_completed_batch(&self, batch_id: &str) -> AppResult<()> {
        info!("[TestCoordination] 清理完成的批次: {}", batch_id);

        let mut batches = self.active_batches.lock().await;
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
    use crate::services::channel_allocation_service::{
        IChannelAllocationService, BatchAllocationResult, AllocationSummary, ModuleTypeStats,
        ValidationResult, TestPlcConfig
    };
    use crate::models::{ModuleType, PointDataType};
    use std::sync::Arc;
    use std::path::Path;
    use async_trait::async_trait;
    use crate::services::traits::{EventPublisher, BaseService, BatchStatistics};
    use crate::models::RawTestOutcome;
    use crate::utils::error::AppResult;

    /// Mock 事件发布器
    struct MockEventPublisher;

    #[async_trait]
    impl BaseService for MockEventPublisher {
        fn service_name(&self) -> &'static str {
            "MockEventPublisher"
        }

        async fn initialize(&mut self) -> AppResult<()> {
            Ok(())
        }

        async fn shutdown(&mut self) -> AppResult<()> {
            Ok(())
        }

        async fn health_check(&self) -> AppResult<()> {
            Ok(())
        }
    }

    #[async_trait]
    impl EventPublisher for MockEventPublisher {
        async fn publish_test_status_changed(
            &self,
            _instance_id: &str,
            _old_status: crate::models::enums::OverallTestStatus,
            _new_status: crate::models::enums::OverallTestStatus,
        ) -> AppResult<()> {
            Ok(())
        }

        async fn publish_test_completed(&self, _outcome: &RawTestOutcome) -> AppResult<()> {
            Ok(())
        }

        async fn publish_batch_status_changed(&self, _batch_id: &str, _statistics: &BatchStatistics) -> AppResult<()> {
            Ok(())
        }

        async fn publish_plc_connection_changed(&self, _connected: bool) -> AppResult<()> {
            Ok(())
        }

        async fn publish_error(&self, _error: &crate::utils::error::AppError) -> AppResult<()> {
            Ok(())
        }
    }

    /// Mock 通道分配服务
    struct MockChannelAllocationService;

    #[async_trait]
    impl IChannelAllocationService for MockChannelAllocationService {
        async fn allocate_channels(
            &self,
            definitions: Vec<ChannelPointDefinition>,
            _test_plc_config: TestPlcConfig,
            product_model: Option<String>,
            serial_number: Option<String>,
        ) -> Result<BatchAllocationResult, AppError> {
            // 模拟分配：使用新的正确分配逻辑
            let channels_per_batch = 8;
            let total_batches = (definitions.len() + channels_per_batch - 1) / channels_per_batch;

            let mut batches = Vec::new();
            let mut allocated_instances = Vec::new();

            for batch_idx in 0..total_batches {
                let batch_number = batch_idx + 1;
                let batch_id = format!("mock_batch_{}", batch_number);
                let batch_name = format!("模拟批次{}", batch_number);

                // 创建批次信息
                let mut batch_info = TestBatchInfo {
                    batch_id: batch_id.clone(),
                    product_model: product_model.clone(),
                    serial_number: serial_number.clone(),
                    customer_name: None,
                    creation_time: Utc::now(),
                    last_updated_time: Utc::now(),
                    operator_name: None,
                    status_summary: Some("模拟批次".to_string()),
                    total_points: 0, // 将在下面更新
                    tested_points: 0,
                    passed_points: 0,
                    failed_points: 0,
                    skipped_points: 0,
                    overall_status: OverallTestStatus::NotTested,
                    batch_name: batch_name.clone(),
                    custom_data: HashMap::new(),
                };

                // 计算此批次的通道范围
                let start_idx = batch_idx * channels_per_batch;
                let end_idx = std::cmp::min(start_idx + channels_per_batch, definitions.len());
                let batch_definitions = &definitions[start_idx..end_idx];

                // 为此批次的通道创建实例
                for definition in batch_definitions {
                    let instance = ChannelTestInstance {
                        instance_id: uuid::Uuid::new_v4().to_string(),
                        definition_id: definition.id.clone(),
                        test_batch_id: batch_id.clone(),
                        test_batch_name: batch_name.clone(),
                        overall_status: OverallTestStatus::NotTested,
                        current_step_details: None,
                        error_message: None,
                        creation_time: Utc::now(),
                        start_time: None,
                        last_updated_time: Utc::now(),
                        final_test_time: None,
                        total_test_duration_ms: None,
                        sub_test_results: HashMap::new(),
                        hardpoint_readings: None,
                        manual_test_current_value_input: None,
                        manual_test_current_value_output: None,
                        test_plc_channel_tag: Some(format!("MockChannel{}", allocated_instances.len() + 1)),
                        test_plc_communication_address: Some(format!("Mock.{}", allocated_instances.len() + 1)),
                        current_operator: None,
                        retries_count: 0,
                        transient_data: HashMap::new(),
                    };
                    allocated_instances.push(instance);
                }

                // 更新批次的通道数量
                batch_info.total_points = batch_definitions.len() as u32;
                batches.push(batch_info);
            }

            // 创建分配统计
            let allocation_summary = AllocationSummary {
                total_definitions: definitions.len() as u32,
                allocated_instances: allocated_instances.len() as u32,
                skipped_definitions: 0,
                by_module_type: HashMap::new(),
                allocation_errors: Vec::new(),
            };

            Ok(BatchAllocationResult {
                batches,
                allocated_instances,
                errors: Vec::new(), // 添加缺少的errors字段
                allocation_summary,
            })
        }

        async fn get_batch_instances(
            &self,
            _batch_id: &str,
        ) -> Result<Vec<ChannelTestInstance>, AppError> {
            Ok(Vec::new())
        }

        async fn clear_all_allocations(
            &self,
            instances: Vec<ChannelTestInstance>,
        ) -> Result<Vec<ChannelTestInstance>, AppError> {
            Ok(instances)
        }

        async fn validate_allocations(
            &self,
            _instances: &[ChannelTestInstance],
        ) -> Result<ValidationResult, AppError> {
            Ok(ValidationResult {
                is_valid: true,
                errors: Vec::new(),
                warnings: Vec::new(),
            })
        }
    }

    /// 创建测试用的服务依赖
    async fn create_test_services() -> (
        Arc<dyn IChannelStateManager>,
        Arc<dyn ITestExecutionEngine>,
        Arc<dyn IPersistenceService>,
        Arc<dyn EventPublisher>,
        Arc<dyn IChannelAllocationService>,
        Arc<dyn crate::services::domain::test_plc_config_service::ITestPlcConfigService>,
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

        // 创建Mock事件发布器
        let event_publisher = Arc::new(MockEventPublisher);

        // 创建Mock通道分配服务
        let channel_allocation_service = Arc::new(MockChannelAllocationService);

        // 创建Mock测试PLC配置服务
        use crate::services::domain::test_plc_config_service::TestPlcConfigService;
        let test_plc_config_service = Arc::new(
            TestPlcConfigService::new(persistence_service.clone())
        );

        (channel_state_manager, test_execution_engine, persistence_service, event_publisher, channel_allocation_service, test_plc_config_service)
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
        // 不再使用虚拟地址
        definition.test_rig_plc_address = None;

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
        let (channel_state_manager, test_execution_engine, persistence_service, event_publisher, channel_allocation_service, test_plc_config_service) =
            create_test_services().await;

        let coordination_service = TestCoordinationService::new(
            channel_state_manager,
            test_execution_engine,
            persistence_service,
            event_publisher,
            channel_allocation_service,
            test_plc_config_service,
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
        assert!(response.message.contains("成功分配"));
    }

    #[tokio::test]
    async fn test_submit_test_execution_with_auto_start() {
        let (channel_state_manager, test_execution_engine, persistence_service, event_publisher, channel_allocation_service, test_plc_config_service) =
            create_test_services().await;

        let coordination_service = TestCoordinationService::new(
            channel_state_manager,
            test_execution_engine,
            persistence_service,
            event_publisher,
            channel_allocation_service,
            test_plc_config_service,
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
        let (channel_state_manager, test_execution_engine, persistence_service, event_publisher, channel_allocation_service) =
            create_test_services().await;

        let coordination_service = TestCoordinationService::new(
            channel_state_manager,
            test_execution_engine,
            persistence_service,
            event_publisher,
            channel_allocation_service,
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
        let (channel_state_manager, test_execution_engine, persistence_service, event_publisher, channel_allocation_service, test_plc_config_service) =
            create_test_services().await;

        let coordination_service = TestCoordinationService::new(
            channel_state_manager,
            test_execution_engine,
            persistence_service,
            event_publisher,
            channel_allocation_service,
            test_plc_config_service,
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
        let (channel_state_manager, test_execution_engine, persistence_service, event_publisher, channel_allocation_service) =
            create_test_services().await;

        let coordination_service = TestCoordinationService::new(
            channel_state_manager,
            test_execution_engine,
            persistence_service,
            event_publisher,
            channel_allocation_service,
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