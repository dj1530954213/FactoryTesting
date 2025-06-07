/// 测试执行引擎
///
/// 负责管理和并发执行测试任务，协调多个测试执行器完成完整的测试序列

use crate::models::{ChannelTestInstance, ChannelPointDefinition, RawTestOutcome, ModuleType, SubTestItem};
use crate::services::infrastructure::IPlcCommunicationService;
use crate::services::domain::specific_test_executors::{
    ISpecificTestStepExecutor, AIHardPointPercentExecutor,
    DIHardPointTestExecutor, DOHardPointTestExecutor, AOHardPointTestExecutor
};
use crate::utils::error::{AppError, AppResult};
use async_trait::async_trait;
use std::sync::Arc;
use std::collections::HashMap;
use tokio::sync::{mpsc, Semaphore, RwLock};
use tokio_util::sync::CancellationToken;
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use log::{debug, info, warn, error};

/// 任务状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TaskStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

/// 测试任务
#[derive(Debug, Clone)]
pub struct TestTask {
    pub task_id: String,
    pub instance: ChannelTestInstance,
    pub definition: ChannelPointDefinition,
    pub status: TaskStatus,
    pub cancellation_token: CancellationToken,
}

/// 测试执行引擎接口
#[async_trait]
pub trait ITestExecutionEngine: Send + Sync {
    /// 提交测试实例执行
    async fn submit_test_instance(
        &self,
        instance: ChannelTestInstance,
        definition: ChannelPointDefinition,
        result_sender: mpsc::Sender<RawTestOutcome>,
    ) -> AppResult<String>;

    /// 获取任务状态
    async fn get_task_status(&self, task_id: &str) -> AppResult<TaskStatus>;

    /// 取消任务
    async fn cancel_task(&self, task_id: &str) -> AppResult<()>;

    /// 获取活动任务数量
    async fn get_active_task_count(&self) -> usize;

    /// 停止所有任务
    async fn stop_all_tasks(&self) -> AppResult<()>;
}

/// 测试执行引擎实现
pub struct TestExecutionEngine {
    /// 并发控制信号量
    semaphore: Arc<Semaphore>,
    /// 测试台PLC服务
    plc_service_test_rig: Arc<dyn IPlcCommunicationService>,
    /// 目标PLC服务
    plc_service_target: Arc<dyn IPlcCommunicationService>,
    /// 活动任务管理
    active_tasks: Arc<RwLock<HashMap<String, TestTask>>>,
    /// 全局取消令牌
    global_cancellation_token: CancellationToken,
}

impl TestExecutionEngine {
    /// 创建新的测试执行引擎
    pub fn new(
        max_concurrent_tasks: usize,
        plc_service_test_rig: Arc<dyn IPlcCommunicationService>,
        plc_service_target: Arc<dyn IPlcCommunicationService>,
    ) -> Self {
        Self {
            semaphore: Arc::new(Semaphore::new(max_concurrent_tasks)),
            plc_service_test_rig,
            plc_service_target,
            active_tasks: Arc::new(RwLock::new(HashMap::new())),
            global_cancellation_token: CancellationToken::new(),
        }
    }

    /// 根据点位定义确定测试步骤
    fn determine_test_steps(&self, definition: &ChannelPointDefinition) -> Vec<Box<dyn ISpecificTestStepExecutor>> {
        let mut executors: Vec<Box<dyn ISpecificTestStepExecutor>> = Vec::new();

        match definition.module_type {
            ModuleType::AI | ModuleType::AINone => {
                // AI点硬点测试：测试PLC的AO通道输出 → 被测PLC的AI通道采集
                // AI点硬点测试不需要时间间隔参数
                executors.push(Box::new(AIHardPointPercentExecutor::new()));
                //TODO:(DJ)这里没有项下面的3个测试项添加测试的时间间隔。我需要编写方式保持一致
            },
            ModuleType::DI | ModuleType::DINone => {
                // DI点硬点测试：测试PLC的DO通道输出 → 被测PLC的DI通道检测
                executors.push(Box::new(DIHardPointTestExecutor::new(3000))); // 3秒间隔
            },
            ModuleType::DO | ModuleType::DONone => {
                // DO点硬点测试：被测PLC的DO通道输出 → 测试PLC的DI通道检测
                executors.push(Box::new(DOHardPointTestExecutor::new(3000))); // 3秒间隔
            },
            ModuleType::AO | ModuleType::AONone => {
                // AO点硬点测试：被测PLC的AO通道输出 → 测试PLC的AI通道采集
                executors.push(Box::new(AOHardPointTestExecutor::new(3000))); // 3秒间隔
            },
            ModuleType::Communication => {
                // TODO: 实现通信模块测试执行器
                debug!("通信模块测试执行器尚未实现: {}", definition.tag);
            },
            ModuleType::Other(_) => {
                // TODO: 实现其他类型模块测试执行器
                debug!("其他类型模块测试执行器尚未实现: {}", definition.tag);
            },
        }

        executors
    }

    /// 执行单个测试实例的完整测试序列
    async fn execute_test_sequence(
        &self,
        task_id: String,
        instance: ChannelTestInstance,
        definition: ChannelPointDefinition,
        result_sender: mpsc::Sender<RawTestOutcome>,
        task_cancellation_token: CancellationToken,
    ) {
        info!("🚀 开始测试: {} [{}]", definition.tag, instance.instance_id);

        // 更新任务状态为运行中
        {
            let mut tasks = self.active_tasks.write().await;
            if let Some(task) = tasks.get_mut(&task_id) {
                task.status = TaskStatus::Running;
            }
        }

        // 确定测试步骤
        let executors = self.determine_test_steps(&definition);

        if executors.is_empty() {
            warn!("[TestEngine] 没有找到适用的测试执行器 - 点位: {}, 类型: {:?}",
                  definition.tag, definition.module_type);

            // 更新任务状态为失败
            {
                let mut tasks = self.active_tasks.write().await;
                if let Some(task) = tasks.get_mut(&task_id) {
                    task.status = TaskStatus::Failed;
                }
            }
            return;
        }

        // 减少冗余日志 - 只在debug模式下显示步骤数量
        debug!("[TestEngine] 确定了 {} 个测试步骤 - 任务: {}", executors.len(), task_id);

        let mut step_count = 0;
        let total_steps = executors.len();
        let mut has_failure = false;

        // 按顺序执行每个测试步骤
        for executor in executors {
            // 检查取消令牌
            if task_cancellation_token.is_cancelled() || self.global_cancellation_token.is_cancelled() {
                info!("[TestEngine] 任务被取消 - 任务: {}", task_id);

                // 更新任务状态为已取消
                {
                    let mut tasks = self.active_tasks.write().await;
                    if let Some(task) = tasks.get_mut(&task_id) {
                        task.status = TaskStatus::Cancelled;
                    }
                }
                return;
            }

            step_count += 1;

            // 检查执行器是否支持当前点位定义
            if !executor.supports_definition(&definition) {
                debug!("[TestEngine] 跳过不支持的测试步骤 - 任务: {}, 步骤: {}/{}, 执行器: {}",
                       task_id, step_count, total_steps, executor.executor_name());
                continue;
            }

            // 减少冗余日志 - 只在debug模式下显示步骤执行信息
            debug!("[TestEngine] 执行测试步骤 - 任务: {}, 步骤: {}/{}, 执行器: {}",
                   task_id, step_count, total_steps, executor.executor_name());

            // 执行测试步骤
            match executor.execute(
                &instance,
                &definition,
                self.plc_service_test_rig.clone(),
                self.plc_service_target.clone(),
            ).await {
                Ok(outcome) => {
                    // 减少冗余日志 - 只在debug模式下显示步骤完成信息
                    debug!("[TestEngine] 测试步骤完成 - 任务: {}, 步骤: {}/{}, 结果: {}",
                           task_id, step_count, total_steps, outcome.success);

                    // 发送测试结果
                    if let Err(e) = result_sender.send(outcome.clone()).await {
                        error!("[TestEngine] 发送测试结果失败 - 任务: {}, 错误: {}", task_id, e);
                    }

                    // 记录失败状态，但继续执行后续步骤以获得完整测试数据
                    if !outcome.success {
                        warn!("[TestEngine] 测试步骤失败，但继续执行以获得完整数据 - 任务: {}, 步骤: {:?}",
                              task_id, outcome.sub_test_item);
                        has_failure = true;
                        // 不再break，继续执行后续测试步骤
                    }
                },
                Err(e) => {
                    error!("[TestEngine] 测试步骤执行失败 - 任务: {}, 步骤: {}/{}, 错误: {}",
                           task_id, step_count, total_steps, e);

                    // 创建失败的测试结果
                    let mut failed_outcome = RawTestOutcome::new(
                        instance.instance_id.clone(),
                        executor.item_type(),
                        false,
                    );
                    failed_outcome.message = Some(format!("执行失败: {}", e));

                    // 发送失败结果
                    if let Err(send_err) = result_sender.send(failed_outcome).await {
                        error!("[TestEngine] 发送失败结果失败 - 任务: {}, 错误: {}", task_id, send_err);
                    }

                    has_failure = true;
                    // 继续执行后续步骤以获得完整测试数据
                }
            }
        }

        // 更新最终任务状态
        {
            let mut tasks = self.active_tasks.write().await;
            if let Some(task) = tasks.get_mut(&task_id) {
                task.status = if has_failure { TaskStatus::Failed } else { TaskStatus::Completed };
            }
        }

        let status_icon = if has_failure { "❌" } else { "✅" };
        info!("{} 测试完成: {} - {}",
              status_icon, definition.tag, if has_failure { "失败" } else { "成功" });
    }
}

#[async_trait]
impl ITestExecutionEngine for TestExecutionEngine {
    /// 提交测试实例执行
    async fn submit_test_instance(
        &self,
        instance: ChannelTestInstance,
        definition: ChannelPointDefinition,
        result_sender: mpsc::Sender<RawTestOutcome>,
    ) -> AppResult<String> {
        let task_id = Uuid::new_v4().to_string();
        let task_cancellation_token = self.global_cancellation_token.child_token();

        // 减少冗余日志 - 只在debug模式下显示任务提交信息
        debug!("[TestEngine] 提交测试任务: {} for instance: {}, 点位: {}",
              task_id, instance.instance_id, definition.tag);

        // 创建任务记录
        let task = TestTask {
            task_id: task_id.clone(),
            instance: instance.clone(),
            definition: definition.clone(),
            status: TaskStatus::Pending,
            cancellation_token: task_cancellation_token.clone(),
        };

        // 添加到活动任务列表
        {
            let mut tasks = self.active_tasks.write().await;
            tasks.insert(task_id.clone(), task);
        }

        // 获取信号量许可并启动异步任务
        let semaphore = self.semaphore.clone();
        let active_tasks = self.active_tasks.clone();
        let plc_test_rig = self.plc_service_test_rig.clone();
        let plc_target = self.plc_service_target.clone();
        let global_token = self.global_cancellation_token.clone();

        let engine_clone = TestExecutionEngine {
            semaphore: semaphore.clone(),
            plc_service_test_rig: plc_test_rig,
            plc_service_target: plc_target,
            active_tasks: active_tasks.clone(),
            global_cancellation_token: global_token,
        };

        // 克隆task_id用于返回
        let return_task_id = task_id.clone();

        tokio::spawn(async move {
            // 获取信号量许可
            let _permit = match semaphore.acquire().await {
                Ok(permit) => permit,
                Err(_) => {
                    error!("[TestEngine] 获取信号量许可失败 - 任务: {}", task_id);

                    // 更新任务状态为失败
                    {
                        let mut tasks = active_tasks.write().await;
                        if let Some(task) = tasks.get_mut(&task_id) {
                            task.status = TaskStatus::Failed;
                        }
                    }
                    return;
                }
            };

            // 执行测试序列
            engine_clone.execute_test_sequence(
                task_id.clone(),
                instance,
                definition,
                result_sender,
                task_cancellation_token,
            ).await;

            // 任务完成后从活动任务列表中移除
            {
                let mut tasks = active_tasks.write().await;
                tasks.remove(&task_id);
            }

            debug!("[TestEngine] 任务清理完成 - 任务: {}", task_id);
        });

        Ok(return_task_id)
    }

    /// 获取任务状态
    async fn get_task_status(&self, task_id: &str) -> AppResult<TaskStatus> {
        let tasks = self.active_tasks.read().await;
        match tasks.get(task_id) {
            Some(task) => Ok(task.status.clone()),
            None => Err(AppError::not_found_error("任务", &format!("任务不存在: {}", task_id))),
        }
    }

    /// 取消任务
    async fn cancel_task(&self, task_id: &str) -> AppResult<()> {
        info!("[TestEngine] 取消任务: {}", task_id);

        let tasks = self.active_tasks.read().await;
        match tasks.get(task_id) {
            Some(task) => {
                task.cancellation_token.cancel();
                info!("[TestEngine] 任务取消信号已发送: {}", task_id);
                Ok(())
            },
            None => Err(AppError::not_found_error("任务", &format!("任务不存在: {}", task_id))),
        }
    }

    /// 获取活动任务数量
    async fn get_active_task_count(&self) -> usize {
        let tasks = self.active_tasks.read().await;
        tasks.len()
    }

    /// 停止所有任务
    async fn stop_all_tasks(&self) -> AppResult<()> {
        info!("[TestEngine] 停止所有任务");

        self.global_cancellation_token.cancel();

        // 等待所有任务完成
        let mut retry_count = 0;
        const MAX_RETRIES: usize = 50; // 最多等待5秒

        while retry_count < MAX_RETRIES {
            let active_count = self.get_active_task_count().await;
            if active_count == 0 {
                break;
            }

            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            retry_count += 1;
        }

        let final_count = self.get_active_task_count().await;
        if final_count > 0 {
            warn!("[TestEngine] 仍有 {} 个任务未完成", final_count);
        } else {
            info!("[TestEngine] 所有任务已停止");
        }

        Ok(())
    }
}