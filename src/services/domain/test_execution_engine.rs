/// 测试执行引擎
/// 
/// 负责管理和并发执行由多个 SpecificTestStepExecutor 构成的完整测试序列
/// 提供并发控制、任务管理和生命周期控制功能

use crate::models::{
    ChannelTestInstance, ChannelPointDefinition, RawTestOutcome, SubTestItem, ModuleType
};
use crate::services::infrastructure::plc::IPlcCommunicationService;
use crate::services::domain::specific_test_executors::{
    ISpecificTestStepExecutor, AIHardPointPercentExecutor, AIAlarmTestExecutor, DIStateReadExecutor
};
use crate::utils::error::{AppError, AppResult};
use async_trait::async_trait;
use std::sync::Arc;
use std::collections::HashMap;
use tokio::sync::{mpsc, Semaphore, Mutex};
use tokio_util::sync::CancellationToken;
use log::{debug, warn, error, info};

/// 测试执行引擎接口
/// 
/// 定义了测试执行引擎的核心功能
#[async_trait]
pub trait ITestExecutionEngine: Send + Sync {
    /// 提交一个测试实例到执行队列
    /// 
    /// # 参数
    /// * `instance` - 要测试的通道实例
    /// * `definition` - 对应的点位定义
    /// * `outcome_sender` - 用于发送测试结果的通道
    async fn submit_test_instance(
        &self,
        instance: ChannelTestInstance,
        definition: ChannelPointDefinition,
        outcome_sender: mpsc::Sender<AppResult<RawTestOutcome>>,
    ) -> AppResult<String>; // 返回任务ID

    /// 暂停与特定实例ID关联的所有活动测试步骤
    async fn pause_instance_execution(&self, instance_id: &str) -> AppResult<()>;

    /// 继续与特定实例ID关联的所有暂停的测试步骤
    async fn resume_instance_execution(&self, instance_id: &str) -> AppResult<()>;

    /// 停止/取消与特定实例ID关联的所有测试步骤
    async fn stop_instance_execution(&self, instance_id: &str) -> AppResult<()>;

    /// 设置最大并发测试数
    async fn set_max_concurrent_tests(&self, max_concurrent: usize) -> AppResult<()>;

    /// 获取当前活动的测试任务数量
    async fn get_active_task_count(&self) -> usize;

    /// 停止所有测试任务
    async fn stop_all_tasks(&self) -> AppResult<()>;
}

/// 任务状态
#[derive(Debug, Clone, PartialEq)]
pub enum TaskStatus {
    /// 等待执行
    Pending,
    /// 正在执行
    Running,
    /// 已暂停
    Paused,
    /// 已完成
    Completed,
    /// 已取消
    Cancelled,
    /// 执行失败
    Failed,
}

/// 测试任务信息
#[derive(Debug, Clone)]
pub struct TestTask {
    /// 任务ID
    pub task_id: String,
    /// 实例ID
    pub instance_id: String,
    /// 任务状态
    pub status: TaskStatus,
    /// 取消令牌
    pub cancellation_token: CancellationToken,
    /// 创建时间
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// 开始时间
    pub started_at: Option<chrono::DateTime<chrono::Utc>>,
    /// 完成时间
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// 测试执行引擎实现
pub struct TestExecutionEngine {
    /// 最大并发测试数控制
    max_concurrent_tests: Arc<Semaphore>,
    /// 测试台架PLC服务
    plc_service_test_rig: Arc<dyn IPlcCommunicationService>,
    /// 被测系统PLC服务
    plc_service_target: Arc<dyn IPlcCommunicationService>,
    /// 活动任务管理
    active_tasks: Arc<Mutex<HashMap<String, TestTask>>>,
    /// 全局取消令牌
    global_cancellation_token: CancellationToken,
    /// 服务名称
    service_name: String,
}

impl TestExecutionEngine {
    /// 创建新的测试执行引擎
    pub fn new(
        max_concurrent_tests: usize,
        plc_service_test_rig: Arc<dyn IPlcCommunicationService>,
        plc_service_target: Arc<dyn IPlcCommunicationService>,
    ) -> Self {
        Self {
            max_concurrent_tests: Arc::new(Semaphore::new(max_concurrent_tests)),
            plc_service_test_rig,
            plc_service_target,
            active_tasks: Arc::new(Mutex::new(HashMap::new())),
            global_cancellation_token: CancellationToken::new(),
            service_name: "TestExecutionEngine".to_string(),
        }
    }

    /// 根据点位定义确定需要执行的测试步骤
    fn determine_test_steps(
        &self,
        definition: &ChannelPointDefinition,
    ) -> Vec<Arc<dyn ISpecificTestStepExecutor>> {
        let mut steps: Vec<Arc<dyn ISpecificTestStepExecutor>> = Vec::new();

        match definition.module_type {
            ModuleType::AI | ModuleType::AINone => {
                // AI点测试序列
                
                // 1. 硬点测试 - 多个百分比点
                let hardpoint_percentages = vec![0.0, 0.25, 0.5, 0.75, 1.0];
                for percentage in hardpoint_percentages {
                    let executor = AIHardPointPercentExecutor::new(
                        percentage,
                        0.05, // 5% 容差
                        500,  // 500ms 稳定时间
                    );
                    if executor.supports_definition(definition) {
                        steps.push(Arc::new(executor));
                    }
                }

                // 2. 报警测试
                let alarm_types = vec![
                    SubTestItem::LowLowAlarm,
                    SubTestItem::LowAlarm,
                    SubTestItem::HighAlarm,
                    SubTestItem::HighHighAlarm,
                ];
                
                for alarm_type in alarm_types {
                    let executor = AIAlarmTestExecutor::new(
                        alarm_type,
                        1000, // 1秒触发等待
                        500,  // 500ms恢复等待
                    );
                    if executor.supports_definition(definition) {
                        steps.push(Arc::new(executor));
                    }
                }
            },
            
            ModuleType::DI | ModuleType::DINone => {
                // DI点测试序列
                
                // 1. 状态读取测试
                let executor = DIStateReadExecutor::new(
                    None, // 不指定期望状态，只要能读取就算成功
                    100,  // 100ms读取延时
                );
                if executor.supports_definition(definition) {
                    steps.push(Arc::new(executor));
                }
            },
            
            ModuleType::DO | ModuleType::DONone => {
                // DO点测试序列 - 暂时使用DI读取器作为占位符
                // TODO: 实现专门的DO测试执行器
                let executor = DIStateReadExecutor::new(
                    None,
                    100,
                );
                steps.push(Arc::new(executor));
            },
            
            ModuleType::AO | ModuleType::AONone => {
                // AO点测试序列 - 暂时使用AI执行器作为占位符
                // TODO: 实现专门的AO测试执行器
                let executor = AIHardPointPercentExecutor::new(0.5, 0.05, 500);
                if executor.supports_definition(definition) {
                    steps.push(Arc::new(executor));
                }
            },
        }

        if steps.is_empty() {
            warn!("[{}] 未找到适合的测试执行器 - 模块类型: {:?}", 
                  self.service_name, definition.module_type);
        } else {
            info!("[{}] 为模块类型 {:?} 确定了 {} 个测试步骤", 
                  self.service_name, definition.module_type, steps.len());
        }

        steps
    }

    /// 执行单个测试实例的完整测试序列
    async fn execute_test_sequence(
        &self,
        task_id: String,
        instance: ChannelTestInstance,
        definition: ChannelPointDefinition,
        outcome_sender: mpsc::Sender<AppResult<RawTestOutcome>>,
        task_cancellation_token: CancellationToken,
        _permit: tokio::sync::OwnedSemaphorePermit, // 保持许可直到任务完成
    ) {
        info!("[{}] 开始执行测试序列 - 任务ID: {}, 实例: {}", 
              self.service_name, task_id, instance.instance_id);

        // 更新任务状态为运行中
        {
            let mut tasks = self.active_tasks.lock().await;
            if let Some(task) = tasks.get_mut(&task_id) {
                task.status = TaskStatus::Running;
                task.started_at = Some(chrono::Utc::now());
            }
        }

        // 确定测试步骤
        let test_steps = self.determine_test_steps(&definition);
        
        if test_steps.is_empty() {
            warn!("[{}] 没有可执行的测试步骤 - 任务ID: {}", self.service_name, task_id);
            self.complete_task(&task_id, TaskStatus::Completed).await;
            return;
        }

        let mut step_count = 0;
        let total_steps = test_steps.len();

        // 按顺序执行每个测试步骤
        for (index, executor) in test_steps.iter().enumerate() {
            // 检查取消信号
            if task_cancellation_token.is_cancelled() || self.global_cancellation_token.is_cancelled() {
                info!("[{}] 测试序列被取消 - 任务ID: {}, 步骤: {}/{}", 
                      self.service_name, task_id, index + 1, total_steps);
                self.complete_task(&task_id, TaskStatus::Cancelled).await;
                return;
            }

            // 检查暂停状态
            while self.is_task_paused(&task_id).await {
                debug!("[{}] 任务已暂停，等待恢复 - 任务ID: {}", self.service_name, task_id);
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                
                // 在暂停期间也要检查取消信号
                if task_cancellation_token.is_cancelled() || self.global_cancellation_token.is_cancelled() {
                    info!("[{}] 暂停期间收到取消信号 - 任务ID: {}", self.service_name, task_id);
                    self.complete_task(&task_id, TaskStatus::Cancelled).await;
                    return;
                }
            }

            debug!("[{}] 执行测试步骤 {}/{} - 任务ID: {}, 执行器: {}", 
                   self.service_name, index + 1, total_steps, task_id, executor.executor_name());

            // 执行测试步骤
            let result = executor.execute(
                &instance,
                &definition,
                self.plc_service_test_rig.clone(),
                self.plc_service_target.clone(),
            ).await;

            // 发送测试结果
            match result {
                Ok(outcome) => {
                    debug!("[{}] 测试步骤完成 - 任务ID: {}, 步骤: {}/{}, 结果: {}", 
                           self.service_name, task_id, index + 1, total_steps, outcome.success);
                    
                    if let Err(e) = outcome_sender.send(Ok(outcome)).await {
                        error!("[{}] 发送测试结果失败 - 任务ID: {}, 错误: {}", 
                               self.service_name, task_id, e);
                        self.complete_task(&task_id, TaskStatus::Failed).await;
                        return;
                    }
                    step_count += 1;
                },
                Err(e) => {
                    error!("[{}] 测试步骤执行失败 - 任务ID: {}, 步骤: {}/{}, 错误: {}", 
                           self.service_name, task_id, index + 1, total_steps, e);
                    
                    if let Err(send_err) = outcome_sender.send(Err(e)).await {
                        error!("[{}] 发送错误结果失败 - 任务ID: {}, 错误: {}", 
                               self.service_name, task_id, send_err);
                    }
                    
                    // 根据错误类型决定是否继续
                    // 对于关键错误（如硬点测试失败），可能需要中止后续步骤
                    if executor.item_type() == SubTestItem::HardPoint {
                        warn!("[{}] 硬点测试失败，中止后续测试 - 任务ID: {}", self.service_name, task_id);
                        self.complete_task(&task_id, TaskStatus::Failed).await;
                        return;
                    }
                    // 对于非关键错误，继续执行后续步骤
                }
            }

            // 步骤间短暂延时
            tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        }

        info!("[{}] 测试序列完成 - 任务ID: {}, 完成步骤: {}/{}", 
              self.service_name, task_id, step_count, total_steps);
        
        self.complete_task(&task_id, TaskStatus::Completed).await;
    }

    /// 检查任务是否处于暂停状态
    async fn is_task_paused(&self, task_id: &str) -> bool {
        let tasks = self.active_tasks.lock().await;
        if let Some(task) = tasks.get(task_id) {
            task.status == TaskStatus::Paused
        } else {
            false
        }
    }

    /// 完成任务并更新状态
    async fn complete_task(&self, task_id: &str, final_status: TaskStatus) {
        let mut tasks = self.active_tasks.lock().await;
        if let Some(task) = tasks.get_mut(task_id) {
            task.status = final_status;
            task.completed_at = Some(chrono::Utc::now());
            debug!("[{}] 任务状态更新 - 任务ID: {}, 状态: {:?}", 
                   self.service_name, task_id, task.status);
        }
    }

    /// 生成唯一的任务ID
    fn generate_task_id(&self, instance_id: &str) -> String {
        format!("task_{}_{}", instance_id, chrono::Utc::now().timestamp_millis())
    }
}

#[async_trait]
impl ITestExecutionEngine for TestExecutionEngine {
    async fn submit_test_instance(
        &self,
        instance: ChannelTestInstance,
        definition: ChannelPointDefinition,
        outcome_sender: mpsc::Sender<AppResult<RawTestOutcome>>,
    ) -> AppResult<String> {
        let task_id = self.generate_task_id(&instance.instance_id);
        let task_cancellation_token = self.global_cancellation_token.child_token();

        info!("[{}] 提交测试实例 - 任务ID: {}, 实例: {}, 模块类型: {:?}", 
              self.service_name, task_id, instance.instance_id, definition.module_type);

        // 创建任务记录
        let task = TestTask {
            task_id: task_id.clone(),
            instance_id: instance.instance_id.clone(),
            status: TaskStatus::Pending,
            cancellation_token: task_cancellation_token.clone(),
            created_at: chrono::Utc::now(),
            started_at: None,
            completed_at: None,
        };

        // 添加到活动任务列表
        {
            let mut tasks = self.active_tasks.lock().await;
            tasks.insert(task_id.clone(), task);
        }

        // 获取信号量许可（这会限制并发数）
        let permit = match self.max_concurrent_tests.clone().acquire_owned().await {
            Ok(permit) => permit,
            Err(e) => {
                error!("[{}] 获取并发许可失败 - 任务ID: {}, 错误: {}", 
                       self.service_name, task_id, e);
                self.complete_task(&task_id, TaskStatus::Failed).await;
                return Err(AppError::SystemError {
                    operation: "acquire_semaphore_permit".to_string(),
                    message: format!("无法获取并发执行许可: {}", e),
                });
            }
        };

        // 启动异步任务执行测试序列
        let plc_test_rig_clone = self.plc_service_test_rig.clone();
        let plc_target_clone = self.plc_service_target.clone();
        let active_tasks_clone = self.active_tasks.clone();
        let global_cancellation_token_clone = self.global_cancellation_token.clone();
        let service_name_clone = self.service_name.clone();
        let task_id_clone = task_id.clone();
        let instance_clone = instance;
        let definition_clone = definition;
        let outcome_sender_clone = outcome_sender;
        let task_cancellation_token_clone = task_cancellation_token;

        tokio::spawn(async move {
            // 创建一个临时的引擎实例来执行测试序列
            let temp_engine = TestExecutionEngine {
                max_concurrent_tests: Arc::new(Semaphore::new(1)), // 临时值，不会被使用
                plc_service_test_rig: plc_test_rig_clone,
                plc_service_target: plc_target_clone,
                active_tasks: active_tasks_clone,
                global_cancellation_token: global_cancellation_token_clone,
                service_name: service_name_clone,
            };
            
            temp_engine.execute_test_sequence(
                task_id_clone,
                instance_clone,
                definition_clone,
                outcome_sender_clone,
                task_cancellation_token_clone,
                permit,
            ).await;
        });

        debug!("[{}] 测试任务已提交 - 任务ID: {}", self.service_name, task_id);
        Ok(task_id)
    }

    async fn pause_instance_execution(&self, instance_id: &str) -> AppResult<()> {
        let mut tasks = self.active_tasks.lock().await;
        let mut paused_count = 0;

        for (task_id, task) in tasks.iter_mut() {
            if task.instance_id == instance_id && task.status == TaskStatus::Running {
                task.status = TaskStatus::Paused;
                paused_count += 1;
                debug!("[{}] 暂停任务 - 任务ID: {}, 实例ID: {}", 
                       self.service_name, task_id, instance_id);
            }
        }

        if paused_count > 0 {
            info!("[{}] 暂停了 {} 个任务 - 实例ID: {}", 
                  self.service_name, paused_count, instance_id);
            Ok(())
        } else {
            Err(AppError::ValidationError {
                field: "instance_id".to_string(),
                message: format!("未找到实例 {} 的运行中任务", instance_id),
            })
        }
    }

    async fn resume_instance_execution(&self, instance_id: &str) -> AppResult<()> {
        let mut tasks = self.active_tasks.lock().await;
        let mut resumed_count = 0;

        for (task_id, task) in tasks.iter_mut() {
            if task.instance_id == instance_id && task.status == TaskStatus::Paused {
                task.status = TaskStatus::Running;
                resumed_count += 1;
                debug!("[{}] 恢复任务 - 任务ID: {}, 实例ID: {}", 
                       self.service_name, task_id, instance_id);
            }
        }

        if resumed_count > 0 {
            info!("[{}] 恢复了 {} 个任务 - 实例ID: {}", 
                  self.service_name, resumed_count, instance_id);
            Ok(())
        } else {
            Err(AppError::ValidationError {
                field: "instance_id".to_string(),
                message: format!("未找到实例 {} 的暂停任务", instance_id),
            })
        }
    }

    async fn stop_instance_execution(&self, instance_id: &str) -> AppResult<()> {
        let mut tasks = self.active_tasks.lock().await;
        let mut stopped_count = 0;

        for (task_id, task) in tasks.iter_mut() {
            if task.instance_id == instance_id && 
               (task.status == TaskStatus::Running || task.status == TaskStatus::Paused) {
                task.cancellation_token.cancel();
                task.status = TaskStatus::Cancelled;
                task.completed_at = Some(chrono::Utc::now());
                stopped_count += 1;
                debug!("[{}] 停止任务 - 任务ID: {}, 实例ID: {}", 
                       self.service_name, task_id, instance_id);
            }
        }

        if stopped_count > 0 {
            info!("[{}] 停止了 {} 个任务 - 实例ID: {}", 
                  self.service_name, stopped_count, instance_id);
            Ok(())
        } else {
            Err(AppError::ValidationError {
                field: "instance_id".to_string(),
                message: format!("未找到实例 {} 的活动任务", instance_id),
            })
        }
    }

    async fn set_max_concurrent_tests(&self, max_concurrent: usize) -> AppResult<()> {
        // 注意：Semaphore 不支持动态调整许可数量
        // 这里我们只能记录新的设置，实际的并发控制仍使用原来的信号量
        // 在实际应用中，可能需要重新创建引擎实例来应用新的并发设置
        
        warn!("[{}] 动态调整并发数暂不支持，当前请求: {} (需要重新创建引擎实例)", 
              self.service_name, max_concurrent);
        
        Ok(())
    }

    async fn get_active_task_count(&self) -> usize {
        let tasks = self.active_tasks.lock().await;
        tasks.iter()
            .filter(|(_, task)| matches!(task.status, TaskStatus::Running | TaskStatus::Paused))
            .count()
    }

    async fn stop_all_tasks(&self) -> AppResult<()> {
        info!("[{}] 停止所有任务", self.service_name);
        
        // 取消全局令牌，这会影响所有子任务
        self.global_cancellation_token.cancel();
        
        // 更新所有活动任务的状态
        let mut tasks = self.active_tasks.lock().await;
        let mut stopped_count = 0;
        
        for (task_id, task) in tasks.iter_mut() {
            if matches!(task.status, TaskStatus::Running | TaskStatus::Paused | TaskStatus::Pending) {
                task.status = TaskStatus::Cancelled;
                task.completed_at = Some(chrono::Utc::now());
                stopped_count += 1;
                debug!("[{}] 全局停止任务 - 任务ID: {}", self.service_name, task_id);
            }
        }
        
        info!("[{}] 全局停止了 {} 个任务", self.service_name, stopped_count);
        Ok(())
    }
}

// 重新导出常用类型
pub use {
    ITestExecutionEngine,
    TestExecutionEngine,
    TaskStatus,
    TestTask,
};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::infrastructure::plc::MockPlcService;
    use crate::models::{ChannelTestInstance, ChannelPointDefinition, ModuleType, PointDataType};
    use std::sync::Arc;

    /// 创建测试用的 PLC 服务
    fn create_mock_plc_service() -> Arc<MockPlcService> {
        Arc::new(MockPlcService::new())
    }

    /// 创建测试用的通道点位定义
    fn create_test_definition() -> ChannelPointDefinition {
        ChannelPointDefinition::new(
            "AI001".to_string(),
            "Temperature_1".to_string(),
            "测试温度点".to_string(),
            "Station1".to_string(),
            "Module1".to_string(),
            ModuleType::AI,
            "CH01".to_string(),
            PointDataType::Float,
            "DB1.DBD0".to_string(),
        )
    }

    #[tokio::test]
    async fn test_engine_creation() {
        // 测试引擎创建
        let plc_test_rig = create_mock_plc_service();
        let plc_target = create_mock_plc_service();
        
        let engine = TestExecutionEngine::new(5, plc_test_rig, plc_target);
        
        // 验证初始状态
        assert_eq!(engine.get_active_task_count().await, 0);
        assert_eq!(engine.service_name, "TestExecutionEngine");
    }

    #[tokio::test]
    async fn test_determine_test_steps_ai() {
        // 测试 AI 点的测试步骤确定
        let plc_test_rig = create_mock_plc_service();
        let plc_target = create_mock_plc_service();
        let engine = TestExecutionEngine::new(5, plc_test_rig, plc_target);
        
        let definition = create_test_definition();
        let steps = engine.determine_test_steps(&definition);
        
        // AI 点应该有硬点测试（5个百分比）+ 报警测试（4种类型）= 9个步骤
        assert_eq!(steps.len(), 9);
        
        // 验证步骤类型
        let hardpoint_count = steps.iter()
            .filter(|step| step.item_type() == SubTestItem::HardPoint)
            .count();
        assert_eq!(hardpoint_count, 5); // 5个硬点百分比测试
        
        let alarm_count = steps.iter()
            .filter(|step| matches!(step.item_type(), 
                SubTestItem::LowLowAlarm | SubTestItem::LowAlarm | 
                SubTestItem::HighAlarm | SubTestItem::HighHighAlarm))
            .count();
        assert_eq!(alarm_count, 4); // 4种报警测试
    }

    #[test]
    fn test_task_status_enum() {
        // 测试任务状态枚举
        let status = TaskStatus::Pending;
        assert_eq!(status, TaskStatus::Pending);
        
        let status = TaskStatus::Running;
        assert_eq!(status, TaskStatus::Running);
    }
} 