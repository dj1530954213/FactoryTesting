/// 通道状态管理器
/// 
/// 负责管理通道测试实例的状态，是唯一可以修改 ChannelTestInstance 核心状态的组件

use crate::models::{
    ChannelTestInstance, ChannelPointDefinition, RawTestOutcome, 
    OverallTestStatus, SubTestStatus, SubTestItem, ModuleType, SubTestExecutionResult
};
use crate::services::infrastructure::IPersistenceService;
use crate::utils::error::{AppError, AppResult};
use async_trait::async_trait;
use std::sync::Arc;
use std::collections::HashMap;
use chrono::Utc;
use log::info;

/// 通道状态管理器接口
#[async_trait]
pub trait IChannelStateManager: Send + Sync {
    /// 初始化通道测试实例
    async fn initialize_channel_test_instance(
        &self,
        definition: ChannelPointDefinition,
        batch_id: String,
    ) -> AppResult<ChannelTestInstance>;

    /// 应用原始测试结果
    async fn apply_raw_outcome(
        &self,
        instance: &mut ChannelTestInstance,
        outcome: RawTestOutcome,
    ) -> AppResult<()>;

    /// 标记为跳过
    async fn mark_as_skipped(&self, instance: &mut ChannelTestInstance) -> AppResult<()>;

    /// 准备接线确认
    async fn prepare_for_wiring_confirmation(&self, instance: &mut ChannelTestInstance) -> AppResult<()>;

    /// 开始硬点测试
    async fn begin_hard_point_test(&self, instance: &mut ChannelTestInstance) -> AppResult<()>;

    /// 开始手动子测试
    async fn begin_manual_sub_test(
        &self,
        instance: &mut ChannelTestInstance,
        sub_test_item: SubTestItem,
    ) -> AppResult<()>;

    /// 重置为重测状态
    async fn reset_for_retest(&self, instance: &mut ChannelTestInstance) -> AppResult<()>;

    /// 创建新的测试实例（兼容现有接口）
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

    /// 评估整体状态（私有方法）
    fn evaluate_overall_status(&self, instance: &mut ChannelTestInstance) {
        let mut all_required_passed = true;
        let mut any_failed = false;
        let mut hard_point_completed = false;
        let mut has_manual_tests = false;
        let mut manual_tests_completed = true;

        // 遍历所有子测试结果
        for (sub_test_item, result) in &instance.sub_test_results {
            match result.status {
                SubTestStatus::Failed => {
                    any_failed = true;
                    all_required_passed = false;
                }
                SubTestStatus::NotTested => {
                    if self.is_required_test(sub_test_item) {
                        all_required_passed = false;
                    }
                    if self.is_manual_test(sub_test_item) {
                        manual_tests_completed = false;
                    }
                }
                SubTestStatus::Passed => {
                    if *sub_test_item == SubTestItem::HardPoint {
                        hard_point_completed = true;
                    }
                }
                SubTestStatus::NotApplicable => {
                    // 不影响整体状态
                }
                _ => {
                    // 其他状态
                }
            }

            if self.is_manual_test(sub_test_item) {
                has_manual_tests = true;
            }
        }

        // 根据状态机规则更新整体状态
        instance.overall_status = if any_failed {
            OverallTestStatus::TestCompletedFailed
        } else if all_required_passed {
            OverallTestStatus::TestCompletedPassed
        } else if hard_point_completed && has_manual_tests && !manual_tests_completed {
            OverallTestStatus::HardPointTestCompleted
        } else if hard_point_completed {
            OverallTestStatus::HardPointTestCompleted
        } else {
            OverallTestStatus::NotTested
        };

        // 如果测试完成，更新时间戳
        if matches!(instance.overall_status, 
            OverallTestStatus::TestCompletedPassed | OverallTestStatus::TestCompletedFailed) {
            instance.final_test_time = Some(Utc::now());
            if let Some(start_time) = instance.start_time {
                instance.total_test_duration_ms = Some(
                    (Utc::now() - start_time).num_milliseconds()
                );
            }
        }

        // 如果失败，构建错误消息
        if any_failed {
            let failed_tests: Vec<String> = instance.sub_test_results
                .iter()
                .filter(|(_, result)| result.status == SubTestStatus::Failed)
                .map(|(item, _)| format!("{:?}", item))
                .collect();
            instance.error_message = Some(format!("测试失败: {}", failed_tests.join(", ")));
        }
    }

    /// 判断是否为必需测试
    fn is_required_test(&self, sub_test_item: &SubTestItem) -> bool {
        matches!(sub_test_item, SubTestItem::HardPoint)
    }

    /// 判断是否为手动测试
    fn is_manual_test(&self, sub_test_item: &SubTestItem) -> bool {
        matches!(sub_test_item, 
            SubTestItem::Maintenance | 
            SubTestItem::Trend | 
            SubTestItem::Report
        )
    }

    /// 初始化子测试结果
    fn initialize_sub_test_results(&self, module_type: &ModuleType) -> HashMap<SubTestItem, SubTestExecutionResult> {
        let mut results = HashMap::new();
        
        match module_type {
            ModuleType::AI => {
                // AI点的测试项
                results.insert(SubTestItem::HardPoint, SubTestExecutionResult::new(SubTestStatus::NotTested, None, None, None));
                results.insert(SubTestItem::LowLowAlarm, SubTestExecutionResult::new(SubTestStatus::NotTested, None, None, None));
                results.insert(SubTestItem::LowAlarm, SubTestExecutionResult::new(SubTestStatus::NotTested, None, None, None));
                results.insert(SubTestItem::HighAlarm, SubTestExecutionResult::new(SubTestStatus::NotTested, None, None, None));
                results.insert(SubTestItem::HighHighAlarm, SubTestExecutionResult::new(SubTestStatus::NotTested, None, None, None));
                results.insert(SubTestItem::Maintenance, SubTestExecutionResult::new(SubTestStatus::NotTested, None, None, None));
                results.insert(SubTestItem::Trend, SubTestExecutionResult::new(SubTestStatus::NotTested, None, None, None));
                results.insert(SubTestItem::Report, SubTestExecutionResult::new(SubTestStatus::NotTested, None, None, None));
            }
            ModuleType::AO => {
                // AO点的测试项
                results.insert(SubTestItem::HardPoint, SubTestExecutionResult::new(SubTestStatus::NotTested, None, None, None));
                results.insert(SubTestItem::Maintenance, SubTestExecutionResult::new(SubTestStatus::NotTested, None, None, None));
                results.insert(SubTestItem::Trend, SubTestExecutionResult::new(SubTestStatus::NotTested, None, None, None));
                results.insert(SubTestItem::Report, SubTestExecutionResult::new(SubTestStatus::NotTested, None, None, None));
            }
            ModuleType::DI => {
                // DI点的测试项
                results.insert(SubTestItem::HardPoint, SubTestExecutionResult::new(SubTestStatus::NotTested, None, None, None));
                results.insert(SubTestItem::Maintenance, SubTestExecutionResult::new(SubTestStatus::NotTested, None, None, None));
                results.insert(SubTestItem::Report, SubTestExecutionResult::new(SubTestStatus::NotTested, None, None, None));
            }
            ModuleType::DO => {
                // DO点的测试项
                results.insert(SubTestItem::HardPoint, SubTestExecutionResult::new(SubTestStatus::NotTested, None, None, None));
                results.insert(SubTestItem::Maintenance, SubTestExecutionResult::new(SubTestStatus::NotTested, None, None, None));
                results.insert(SubTestItem::Report, SubTestExecutionResult::new(SubTestStatus::NotTested, None, None, None));
            }
            _ => {
                // 其他模块类型，默认只有硬点测试
                results.insert(SubTestItem::HardPoint, SubTestExecutionResult::new(SubTestStatus::NotTested, None, None, None));
            }
        }
        
        results
    }
}

#[async_trait]
impl IChannelStateManager for ChannelStateManager {
    /// 初始化通道测试实例
    async fn initialize_channel_test_instance(
        &self,
        definition: ChannelPointDefinition,
        batch_id: String,
    ) -> AppResult<ChannelTestInstance> {
        let mut instance = ChannelTestInstance::new(
            definition.id.clone(),
            batch_id,
        );

        // 根据模块类型初始化子测试结果
        instance.sub_test_results = self.initialize_sub_test_results(&definition.module_type);
        instance.overall_status = OverallTestStatus::NotTested;

        info!("初始化通道测试实例: {} ({})", instance.instance_id, definition.tag);
        Ok(instance)
    }

    /// 应用原始测试结果
    async fn apply_raw_outcome(
        &self,
        instance: &mut ChannelTestInstance,
        outcome: RawTestOutcome,
    ) -> AppResult<()> {
        // 更新对应的子测试结果
        if let Some(sub_result) = instance.sub_test_results.get_mut(&outcome.sub_test_item) {
            sub_result.status = if outcome.success {
                SubTestStatus::Passed
            } else {
                SubTestStatus::Failed
            };
            sub_result.timestamp = outcome.end_time;
            sub_result.actual_value = outcome.raw_value_read.clone();
            sub_result.expected_value = outcome.eng_value_calculated.clone();
            sub_result.details = outcome.message.clone();
        }

        // 重新评估整体状态
        self.evaluate_overall_status(instance);

        info!("应用测试结果: {} -> {:?} ({})", 
              instance.instance_id, outcome.sub_test_item, outcome.success);
        Ok(())
    }

    /// 标记为跳过
    async fn mark_as_skipped(&self, instance: &mut ChannelTestInstance) -> AppResult<()> {
        instance.overall_status = OverallTestStatus::Skipped;
        info!("标记为跳过: {}", instance.instance_id);
        Ok(())
    }

    /// 准备接线确认
    async fn prepare_for_wiring_confirmation(&self, instance: &mut ChannelTestInstance) -> AppResult<()> {
        instance.overall_status = OverallTestStatus::WiringConfirmationRequired;
        info!("准备接线确认: {}", instance.instance_id);
        Ok(())
    }

    /// 开始硬点测试
    async fn begin_hard_point_test(&self, instance: &mut ChannelTestInstance) -> AppResult<()> {
        instance.overall_status = OverallTestStatus::HardPointTestInProgress;
        instance.start_time = Some(Utc::now());
        info!("开始硬点测试: {}", instance.instance_id);
        Ok(())
    }

    /// 开始手动子测试
    async fn begin_manual_sub_test(
        &self,
        instance: &mut ChannelTestInstance,
        sub_test_item: SubTestItem,
    ) -> AppResult<()> {
        instance.overall_status = OverallTestStatus::ManualTestInProgress;
        
        // 标记特定的手动测试为进行中
        if let Some(sub_result) = instance.sub_test_results.get_mut(&sub_test_item) {
            sub_result.status = SubTestStatus::NotTested; // 重置状态，准备测试
        }

        info!("开始手动子测试: {} -> {:?}", instance.instance_id, sub_test_item);
        Ok(())
    }

    /// 重置为重测状态
    async fn reset_for_retest(&self, instance: &mut ChannelTestInstance) -> AppResult<()> {
        // 重置所有子测试状态
        for (_, sub_result) in instance.sub_test_results.iter_mut() {
            if sub_result.status != SubTestStatus::NotApplicable {
                sub_result.status = SubTestStatus::NotTested;
                sub_result.timestamp = Utc::now();
                sub_result.actual_value = None;
                sub_result.expected_value = None;
                sub_result.details = None;
            }
        }

        // 重置整体状态
        instance.overall_status = OverallTestStatus::NotTested;
        instance.start_time = None;
        instance.final_test_time = None;
        instance.total_test_duration_ms = None;
        instance.error_message = None;

        info!("重置为重测状态: {}", instance.instance_id);
        Ok(())
    }

    /// 创建新的测试实例（兼容现有接口）
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