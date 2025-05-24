// 文件: src-tauri/src/services/domain/channel_state_manager.rs
// 详细注释：负责管理单个 ChannelTestInstance 的状态转换逻辑

use crate::models::{
    structs::{ChannelPointDefinition, ChannelTestInstance, RawTestOutcome, SubTestExecutionResult},
    enums::{OverallTestStatus, SubTestStatus, SubTestItem, ModuleType},
    AppError, AppResult,
};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use log::{debug, warn, error};
use uuid::Uuid;

/// `ChannelStateManagerService` trait 定义了通道状态管理的核心接口。
/// 此服务不执行I/O操作（如数据库读写或PLC通信），也不直接发送事件。
/// 它的主要职责是根据输入的原始测试结果或指令，纯粹地更新 `ChannelTestInstance` 的状态。
#[async_trait]
pub trait ChannelStateManagerService: Send + Sync {
    /// 初始化一个新的 `ChannelTestInstance`。
    ///
    /// # Arguments
    /// * `definition` - 相关的 `ChannelPointDefinition`。
    /// * `batch_id` - 该测试实例所属的批次ID。
    /// * `creation_time` - 实例的创建时间。
    /// * `operator` - （可选）执行操作的操作员名称。
    ///
    /// # Returns
    /// 返回一个初始化完成的 `ChannelTestInstance`。
    fn initialize_channel_test_instance(
        &self,
        definition: &ChannelPointDefinition,
        batch_id: &str,
        creation_time: DateTime<Utc>,
        operator: Option<String>,
    ) -> AppResult<ChannelTestInstance>;

    /// 应用一个原始测试结果到 `ChannelTestInstance`。
    /// 此方法会更新相应的子测试项状态，并重新评估整体状态。
    ///
    /// # Arguments
    /// * `instance` - 需要更新的 `ChannelTestInstance` 的可变引用。
    /// * `outcome` - 应用的 `RawTestOutcome`。
    async fn apply_raw_outcome(
        &self,
        instance: &mut ChannelTestInstance,
        outcome: RawTestOutcome,
    ) -> AppResult<()>;

    /// 将测试实例标记为已跳过。
    ///
    /// # Arguments
    /// * `instance` - 需要更新的 `ChannelTestInstance` 的可变引用。
    async fn mark_as_skipped(&self, instance: &mut ChannelTestInstance) -> AppResult<()>;

    /// 准备通道实例进行接线确认。
    /// (注意：此方法可能更多地由 TestOrchestrationService 调用以更新状态，
    /// ChannelStateManager 主要负责状态的计算和转换)
    ///
    /// # Arguments
    /// * `instance` - 需要更新的 `ChannelTestInstance` 的可变引用。
    async fn prepare_for_wiring_confirmation(
        &self,
        instance: &mut ChannelTestInstance,
    ) -> AppResult<()>;
    
    /// 标记通道实例的接线已确认。
    ///
    /// # Arguments
    /// * `instance` - 需要更新的 `ChannelTestInstance` 的可变引用。
    /// * `success` - 接线是否确认成功。
    /// * `details` - （可选）接线确认的详细信息或备注。
    async fn wiring_confirmed(
        &self,
        instance: &mut ChannelTestInstance,
        success: bool,
        details: Option<String>,
    ) -> AppResult<()>;


    /// 开始硬点测试。
    ///
    /// # Arguments
    /// * `instance` - 需要更新的 `ChannelTestInstance` 的可变引用。
    async fn begin_hard_point_test(&self, instance: &mut ChannelTestInstance) -> AppResult<()>;

    /// 开始手动子测试。
    ///
    /// # Arguments
    /// * `instance` - 需要更新的 `ChannelTestInstance` 的可变引用。
    /// * `sub_test_item` - 要开始的手动子测试项。
    async fn begin_manual_sub_test(
        &self,
        instance: &mut ChannelTestInstance,
        sub_test_item: &SubTestItem,
    ) -> AppResult<()>;

    /// 重置测试实例以进行重新测试。
    /// 这会将相关的子测试项状态重置为 `NotTested` 或 `NotApplicable`，
    /// 并将整体状态重置为适合重新测试的初始状态。
    ///
    /// # Arguments
    /// * `instance` - 需要更新的 `ChannelTestInstance` 的可变引用。
    async fn reset_for_retest(&self, instance: &mut ChannelTestInstance) -> AppResult<()>;

    /// (私有辅助) 评估并更新 ChannelTestInstance 的整体状态。
    /// 此方法不直接暴露，由其他公共方法在必要时调用。
    fn evaluate_overall_status(&self, instance: &mut ChannelTestInstance);
}

/// `ChannelStateManager` 是 `ChannelStateManagerService` trait 的具体实现。
#[derive(Debug, Default)]
pub struct ChannelStateManager {}

impl ChannelStateManager {
    pub fn new() -> Self {
        Self {}
    }

    /// 内部辅助函数：根据模块类型和点位定义确定哪些子测试是适用的。
    fn get_applicable_sub_tests(definition: &ChannelPointDefinition) -> HashMap<SubTestItem, SubTestExecutionResult> {
        let mut sub_tests = HashMap::new();
        let now = Utc::now();

        // 通用测试项 (几乎所有类型都需要的)
        sub_tests.insert(SubTestItem::CommunicationTest, SubTestExecutionResult::new(now, SubTestStatus::NotTested));
        sub_tests.insert(SubTestItem::HardPoint, SubTestExecutionResult::new(now, SubTestStatus::NotTested));

        match definition.module_type {
            ModuleType::AI => {
                sub_tests.insert(SubTestItem::TrendCheck, SubTestExecutionResult::new(now, SubTestStatus::NotTested));
                sub_tests.insert(SubTestItem::ReportCheck, SubTestExecutionResult::new(now, SubTestStatus::NotTested));
                // 仅当定义了相关报警点时，这些测试项才适用
                if definition.sll_set_point_address.is_some() && definition.sll_feedback_address.is_some() {
                    sub_tests.insert(SubTestItem::LowLowAlarm, SubTestExecutionResult::new(now, SubTestStatus::NotTested));
                } else {
                    sub_tests.insert(SubTestItem::LowLowAlarm, SubTestExecutionResult::new(now, SubTestStatus::NotApplicable));
                }
                if definition.sl_set_point_address.is_some() && definition.sl_feedback_address.is_some() {
                    sub_tests.insert(SubTestItem::LowAlarm, SubTestExecutionResult::new(now, SubTestStatus::NotTested));
                } else {
                    sub_tests.insert(SubTestItem::LowAlarm, SubTestExecutionResult::new(now, SubTestStatus::NotApplicable));
                }
                if definition.sh_set_point_address.is_some() && definition.sh_feedback_address.is_some() {
                    sub_tests.insert(SubTestItem::HighAlarm, SubTestExecutionResult::new(now, SubTestStatus::NotTested));
                } else {
                    sub_tests.insert(SubTestItem::HighAlarm, SubTestExecutionResult::new(now, SubTestStatus::NotApplicable));
                }
                if definition.shh_set_point_address.is_some() && definition.shh_feedback_address.is_some() {
                    sub_tests.insert(SubTestItem::HighHighAlarm, SubTestExecutionResult::new(now, SubTestStatus::NotTested));
                } else {
                    sub_tests.insert(SubTestItem::HighHighAlarm, SubTestExecutionResult::new(now, SubTestStatus::NotApplicable));
                }
                // 聚合报警设定状态
                sub_tests.insert(SubTestItem::AlarmValueSetting, SubTestExecutionResult::new(now, SubTestStatus::NotTested));
                
                if definition.maintenance_value_set_point_address.is_some() || definition.maintenance_enable_switch_point_address.is_some() {
                    sub_tests.insert(SubTestItem::MaintenanceFunction, SubTestExecutionResult::new(now, SubTestStatus::NotTested));
                } else {
                    sub_tests.insert(SubTestItem::MaintenanceFunction, SubTestExecutionResult::new(now, SubTestStatus::NotApplicable));
                }
            }
            ModuleType::AO => {
                sub_tests.insert(SubTestItem::TrendCheck, SubTestExecutionResult::new(now, SubTestStatus::NotTested));
                sub_tests.insert(SubTestItem::ReportCheck, SubTestExecutionResult::new(now, SubTestStatus::NotTested));
                if definition.maintenance_value_set_point_address.is_some() || definition.maintenance_enable_switch_point_address.is_some() {
                     sub_tests.insert(SubTestItem::MaintenanceFunction, SubTestExecutionResult::new(now, SubTestStatus::NotTested));
                } else {
                    sub_tests.insert(SubTestItem::MaintenanceFunction, SubTestExecutionResult::new(now, SubTestStatus::NotApplicable));
                }
                // AO 的特定百分比输出测试可能包含在 HardPoint 内部逻辑中，或者作为单独的 SubTestItem
                // 这里暂时不单独列出 OutputXPercent，假设它们由 HardPoint 测试覆盖
            }
            ModuleType::DI | ModuleType::DO => {
                sub_tests.insert(SubTestItem::StateDisplay, SubTestExecutionResult::new(now, SubTestStatus::NotTested));
            }
            ModuleType::AINone | ModuleType::AONone | ModuleType::DINone | ModuleType::DONone | ModuleType::Communication | ModuleType::Other(_) => {
                // 对于这些类型，可能只有最基础的测试，或者需要更具体的定义
                // 例如，Communication模块可能只有CommunicationTest
                if matches!(definition.module_type, ModuleType::Communication) {
                    // CommunicationTest 已经有了，其他都 NotApplicable
                    for item_key in sub_tests.keys().cloned().collect::<Vec<_>>() {
                        if item_key != SubTestItem::CommunicationTest {
                            sub_tests.get_mut(&item_key).unwrap().status = SubTestStatus::NotApplicable;
                        }
                    }
                } else {
                     // 其他类型，先默认大部分不适用，除了 CommunicationTest 和 HardPoint (如果适用)
                    // 这里需要根据实际业务规则细化
                    sub_tests.values_mut().for_each(|v| if v.status == SubTestStatus::NotTested {v.status = SubTestStatus::NotApplicable});
                    sub_tests.get_mut(&SubTestItem::CommunicationTest).unwrap().status = SubTestStatus::NotTested;
                    // HardPoint 对某些 None 类型可能也不适用，这里需要业务判断
                    if !matches!(definition.module_type, ModuleType::Communication | ModuleType::Other(_)) { // 例如 Other类型可能就没有HardPoint
                        sub_tests.get_mut(&SubTestItem::HardPoint).unwrap().status = SubTestStatus::NotTested;
                    } else {
                         sub_tests.get_mut(&SubTestItem::HardPoint).unwrap().status = SubTestStatus::NotApplicable;
                    }
                }
            }
        }
        sub_tests
    }
}

#[async_trait]
impl ChannelStateManagerService for ChannelStateManager {
    fn initialize_channel_test_instance(
        &self,
        definition: &ChannelPointDefinition,
        batch_id: &str,
        creation_time: DateTime<Utc>,
        operator: Option<String>,
    ) -> AppResult<ChannelTestInstance> {
        let instance_id = Uuid::new_v4().to_string();
        debug!(
            "Initializing ChannelTestInstance: id={}, definition_id={}, batch_id={}",
            instance_id, definition.id, batch_id
        );

        let sub_test_results = Self::get_applicable_sub_tests(definition);

        let mut instance = ChannelTestInstance {
            instance_id,
            definition_id: definition.id.clone(),
            test_batch_id: batch_id.to_string(),
            overall_status: OverallTestStatus::NotTested,
            current_step_details: Some("Initialized".to_string()),
            error_message: None,
            creation_time,
            start_time: None,
            last_updated_time: creation_time,
            final_test_time: None,
            total_test_duration_ms: None,
            sub_test_results,
            hardpoint_readings: None, // 初始化为空
            manual_test_current_value_input: None,
            manual_test_current_value_output: None,
            current_operator: operator,
            retries_count: 0,
            transient_data: HashMap::new(),
        };
        
        // 初始化后立即评估一次状态
        self.evaluate_overall_status(&mut instance);
        Ok(instance)
    }

    async fn apply_raw_outcome(
        &self,
        instance: &mut ChannelTestInstance,
        outcome: RawTestOutcome,
    ) -> AppResult<()> {
        debug!(
            "Applying RawTestOutcome for instance_id: {}, sub_test_item: {:?}, success: {}",
            instance.instance_id, outcome.sub_test_item, outcome.success
        );

        if instance.instance_id != outcome.channel_instance_id {
            let err_msg = format!(
                "Mismatched instance_id in RawTestOutcome. Expected: {}, Got: {}",
                instance.instance_id, outcome.channel_instance_id
            );
            error!("{}", err_msg);
            return Err(AppError::InvalidInputError(err_msg));
        }

        let current_sub_test_status = if outcome.success {
            SubTestStatus::Passed
        } else {
            SubTestStatus::Failed
        };

        // 更新子测试项结果
        if let Some(sub_test_result) = instance.sub_test_results.get_mut(&outcome.sub_test_item) {
            sub_test_result.status = current_sub_test_status;
            sub_test_result.details = outcome.message.clone();
            sub_test_result.expected_value = outcome.raw_value_read.clone(); // 假设 raw_value_read 是期望值或上下文相关值
            sub_test_result.actual_value = outcome.eng_value_calculated.clone(); // 假设 eng_value_calculated 是实际值
            sub_test_result.start_time = Some(outcome.start_time); // 使用 outcome 的时间
            sub_test_result.end_time = Some(outcome.end_time);
            sub_test_result.duration_ms = Some((outcome.end_time - outcome.start_time).num_milliseconds());

            if outcome.sub_test_item == SubTestItem::HardPoint {
                 if let Some(points) = outcome.analog_reading_points {
                    instance.hardpoint_readings = Some(points);
                }
            }

        } else {
            warn!(
                "SubTestItem {:?} not found in instance {}. Outcome not applied.",
                outcome.sub_test_item, instance.instance_id
            );
            // 根据业务需求，如果 outcome 对应的 sub_test_item 不在预定义的适用测试中，
            // 可能需要将其动态添加或记录为错误。目前先忽略。
        }
        
        instance.last_updated_time = Utc::now();
        self.evaluate_overall_status(instance);
        Ok(())
    }

    async fn mark_as_skipped(&self, instance: &mut ChannelTestInstance) -> AppResult<()> {
        debug!("Marking instance_id: {} as Skipped", instance.instance_id);
        instance.overall_status = OverallTestStatus::Skipped;
        // 将所有 NotTested 的子测试项标记为 Skipped
        for sub_result in instance.sub_test_results.values_mut() {
            if sub_result.status == SubTestStatus::NotTested {
                sub_result.status = SubTestStatus::Skipped;
            }
        }
        instance.last_updated_time = Utc::now();
        instance.final_test_time = Some(instance.last_updated_time);
        if let Some(start) = instance.start_time {
            instance.total_test_duration_ms = Some((instance.last_updated_time - start).num_milliseconds());
        }
        Ok(())
    }

    async fn prepare_for_wiring_confirmation(
        &self,
        instance: &mut ChannelTestInstance,
    ) -> AppResult<()> {
        debug!("Preparing instance_id: {} for Wiring Confirmation", instance.instance_id);
        // 通常，这个操作会将整体状态设置为 WiringConfirmationPending 或类似
        // 但具体的状态流可能由 TestOrchestrationService 控制
        // ChannelStateManager 主要确保子测试项状态正确
        if instance.overall_status == OverallTestStatus::NotTested {
            // 可以在这里将通讯测试子项设置为 Testing 或 NotTested，等待执行
            if let Some(comm_test) = instance.sub_test_results.get_mut(&SubTestItem::CommunicationTest) {
                 if comm_test.status == SubTestStatus::NotApplicable {
                    warn!("CommunicationTest is NotApplicable for instance {}, cannot prepare for wiring.", instance.instance_id);
                 } else {
                    // comm_test.status = SubTestStatus::NotTested; // 确保它是可测的
                 }
            }
            // 更新整体状态，或者这个状态的转换由 Orchestrator 完成
            // instance.overall_status = OverallTestStatus::WiringConfirmationPending;
            instance.current_step_details = Some("Awaiting Communication Test / Wiring Confirmation".to_string());
        } else {
            warn!("Instance {} is not in NotTested state, cannot prepare for wiring confirmation. Current state: {:?}", instance.instance_id, instance.overall_status);
        }
        instance.last_updated_time = Utc::now();
        Ok(())
    }
    
    async fn wiring_confirmed(
        &self,
        instance: &mut ChannelTestInstance,
        success: bool,
        details: Option<String>,
    ) -> AppResult<()> {
        debug!("Wiring confirmed for instance_id: {}: success={}", instance.instance_id, success);
        // 假设接线确认本身可以被视为一个特殊的子测试项，或者直接影响整体状态
        // 这里我们先更新 CommunicationTest 的状态，因为它通常是接线确认的前提
        if let Some(comm_test_result) = instance.sub_test_results.get_mut(&SubTestItem::CommunicationTest) {
            if comm_test_result.status != SubTestStatus::NotApplicable {
                 comm_test_result.status = if success { SubTestStatus::Passed } else { SubTestStatus::Failed };
                 comm_test_result.details = details.clone();
                 comm_test_result.end_time = Some(Utc::now());
            }
        }

        if success {
            instance.overall_status = OverallTestStatus::WiringConfirmed;
            instance.current_step_details = Some("Wiring Confirmed. Ready for Hardpoint Test.".to_string());
        } else {
            // 如果接线失败，通常整个测试实例会失败
            instance.overall_status = OverallTestStatus::TestCompletedFailed;
            instance.error_message = Some(details.unwrap_or_else(|| "Wiring confirmation failed".to_string()));
            instance.current_step_details = Some("Wiring Confirmation Failed.".to_string());
            instance.final_test_time = Some(Utc::now());
        }
        instance.last_updated_time = Utc::now();
        self.evaluate_overall_status(instance); // 重新评估确保状态一致
        Ok(())
    }


    async fn begin_hard_point_test(&self, instance: &mut ChannelTestInstance) -> AppResult<()> {
        debug!("Beginning HardPoint Test for instance_id: {}", instance.instance_id);
        if instance.overall_status == OverallTestStatus::WiringConfirmed {
            instance.overall_status = OverallTestStatus::HardPointTesting;
            instance.current_step_details = Some("Hardpoint testing in progress...".to_string());
            if instance.start_time.is_none() {
                instance.start_time = Some(Utc::now());
            }
            // 将 HardPoint 子测试项标记为 Testing
            if let Some(hp_test) = instance.sub_test_results.get_mut(&SubTestItem::HardPoint) {
                if hp_test.status == SubTestStatus::NotTested || hp_test.status == SubTestStatus::Skipped { //允许从Skipped开始重测
                    hp_test.status = SubTestStatus::Testing;
                    hp_test.start_time = Some(Utc::now());
                }
            }
        } else {
            warn!("Instance {} is not in WiringConfirmed state, cannot begin HardPoint Test. Current state: {:?}", instance.instance_id, instance.overall_status);
            return Err(AppError::StateTransitionError(format!("Cannot begin HardPoint Test. Instance {} current state: {:?}", instance.instance_id, instance.overall_status)));
        }
        instance.last_updated_time = Utc::now();
        Ok(())
    }

    async fn begin_manual_sub_test(
        &self,
        instance: &mut ChannelTestInstance,
        sub_test_item: &SubTestItem,
    ) -> AppResult<()> {
        debug!("Beginning Manual SubTest {:?} for instance_id: {}", sub_test_item, instance.instance_id);
        
        // 检查整体状态是否允许手动测试
        if !matches!(instance.overall_status, OverallTestStatus::HardPointTestCompleted | OverallTestStatus::ManualTesting | OverallTestStatus::Retesting) {
             warn!("Instance {} is not in a state suitable for manual testing. Current state: {:?}", instance.instance_id, instance.overall_status);
             return Err(AppError::StateTransitionError(format!("Cannot begin Manual Test {:?}. Instance {} current state: {:?}",sub_test_item, instance.instance_id, instance.overall_status)));
        }
        
        if let Some(manual_item) = instance.sub_test_results.get_mut(sub_test_item) {
            if manual_item.status == SubTestStatus::NotTested || manual_item.status == SubTestStatus::Passed || manual_item.status == SubTestStatus::Failed || manual_item.status == SubTestStatus::Skipped { // 允许重测已完成或跳过的手动项
                manual_item.status = SubTestStatus::Testing;
                manual_item.start_time = Some(Utc::now());
                manual_item.details = None; // 清除旧的详情
                manual_item.expected_value = None;
                manual_item.actual_value = None;


                instance.overall_status = OverallTestStatus::ManualTesting; // 更新整体状态
                instance.current_step_details = Some(format!("Manual testing {:?} in progress...", sub_test_item));
            } else if manual_item.status == SubTestStatus::NotApplicable {
                 warn!("Manual SubTest {:?} is NotApplicable for instance {}", sub_test_item, instance.instance_id);
                 return Err(AppError::InvalidInputError(format!("Manual test item {:?} is not applicable for instance {}", sub_test_item, instance.instance_id)));
            } else {
                warn!("Manual SubTest {:?} for instance {} is already in {:?} state.", sub_test_item, instance.instance_id, manual_item.status);
            }
        } else {
            warn!("Manual SubTestItem {:?} not found in instance {}.", sub_test_item, instance.instance_id);
            return Err(AppError::NotFoundError(format!("Manual test item {:?} not found for instance {}", sub_test_item, instance.instance_id)));
        }
        
        instance.last_updated_time = Utc::now();
        Ok(())
    }

    async fn reset_for_retest(&self, instance: &mut ChannelTestInstance) -> AppResult<()> {
        debug!("Resetting instance_id: {} for Retest", instance.instance_id);
        
        // 只有在测试完成（通过或失败）或跳过时才能重测
        if !matches!(instance.overall_status, OverallTestStatus::TestCompletedPassed | OverallTestStatus::TestCompletedFailed | OverallTestStatus::Skipped | OverallTestStatus::HardPointTestCompleted) {
            warn!("Instance {} cannot be reset for retest from state {:?}", instance.instance_id, instance.overall_status);
            return Err(AppError::StateTransitionError(format!("Cannot reset for retest. Instance {} current state: {:?}", instance.instance_id, instance.overall_status)));
        }
        
        let original_def_id = instance.definition_id.clone();
        let original_batch_id = instance.test_batch_id.clone();
        let original_creation_time = instance.creation_time;
        let original_operator = instance.current_operator.clone();
        let original_definition = ChannelPointDefinition::get_by_id_placeholder(&original_def_id); // 假设有方法可以获取

        // 重新初始化，但保留ID和一些元数据
        let reinitialized_instance = self.initialize_channel_test_instance(
            &original_definition, // 需要获取原始的ChannelPointDefinition
            &original_batch_id,
            original_creation_time, // 创建时间不变
            original_operator
        )?;

        instance.sub_test_results = reinitialized_instance.sub_test_results;
        instance.hardpoint_readings = None;
        instance.overall_status = OverallTestStatus::Retesting; // 标记为重测中
        instance.current_step_details = Some("Instance reset for retesting.".to_string());
        instance.error_message = None;
        instance.start_time = None; // 重置开始时间，将在下次测试开始时设置
        instance.final_test_time = None;
        instance.total_test_duration_ms = None;
        instance.retries_count += 1;
        instance.last_updated_time = Utc::now();
        
        self.evaluate_overall_status(instance); // 确保初始状态正确
        Ok(())
    }
    
    /// 评估整体状态的核心逻辑。
    /// 此方法应该在任何可能影响整体状态的子测试项更新后被调用。
    fn evaluate_overall_status(&self, instance: &mut ChannelTestInstance) {
        // 如果已经是最终失败或跳过状态，则不再改变
        if matches!(instance.overall_status, OverallTestStatus::TestCompletedFailed | OverallTestStatus::Skipped) && instance.final_test_time.is_some() {
            // 如果是 TestCompletedFailed，但不是因为接线失败，则允许通过重测改变
            if instance.overall_status == OverallTestStatus::TestCompletedFailed && 
               instance.error_message.as_deref() == Some("Wiring Confirmation Failed.") {
                return; // 接线失败是硬失败
            } else if instance.overall_status == OverallTestStatus::Skipped {
                return;
            }
            // 对于其他 TestCompletedFailed，允许通过 retest 流程改变
        }

        let mut all_required_passed = true;
        let mut any_failed = false;
        let mut any_testing = false;
        let mut all_hard_points_done = true; // 假设硬点测试只有一项 SubTestItem::HardPoint
        let mut all_manual_points_done_or_na = true;

        let mut hard_point_status = SubTestStatus::NotApplicable;
        let mut communication_status = SubTestStatus::NotApplicable;

        for (item, result) in &instance.sub_test_results {
            if result.status == SubTestStatus::NotApplicable || result.status == SubTestStatus::Skipped {
                continue; // 不适用或跳过的项不参与决定性评估
            }
            if result.status == SubTestStatus::Failed {
                any_failed = true;
            }
            if result.status == SubTestStatus::Testing {
                any_testing = true;
            }
            // 检查所有"必须通过"的项是否都通过
            // "必须通过"的定义需要根据业务逻辑确定，这里简化处理：所有非NA/Skipped的都必须通过
            if result.status != SubTestStatus::Passed {
                all_required_passed = false;
            }

            // 特定检查
            if *item == SubTestItem::HardPoint {
                hard_point_status = result.status;
                if result.status != SubTestStatus::Passed && result.status != SubTestStatus::Failed {
                    all_hard_points_done = false; // 如果硬点没完成(通过/失败)，则硬点未完成
                }
            }
            if *item == SubTestItem::CommunicationTest {
                communication_status = result.status;
            }

            // 检查手动测试项 (AI的报警, 维护功能; DI/DO的状态显示)
            // 手动项可以是 NotTested, Passed, Failed, Skipped, NotApplicable
            match item {
                SubTestItem::LowLowAlarm | SubTestItem::LowAlarm | SubTestItem::HighAlarm | SubTestItem::HighHighAlarm |
                SubTestItem::MaintenanceFunction | SubTestItem::StateDisplay | SubTestItem::TrendCheck | SubTestItem::ReportCheck => {
                    if result.status == SubTestStatus::NotTested || result.status == SubTestStatus::Testing {
                        all_manual_points_done_or_na = false;
                    }
                }
                _ => {} // 其他项不影响 all_manual_points_done_or_na
            }
        }
        
        let old_status = instance.overall_status;

        if any_failed {
            instance.overall_status = OverallTestStatus::TestCompletedFailed;
            instance.error_message = instance.error_message.clone().or_else(|| Some("One or more sub-tests failed.".to_string()));
        } else if any_testing {
            // 根据当前主要阶段确定 Testing 状态
            if old_status == OverallTestStatus::HardPointTesting || hard_point_status == SubTestStatus::Testing {
                instance.overall_status = OverallTestStatus::HardPointTesting;
            } else if old_status == OverallTestStatus::ManualTesting || !all_manual_points_done_or_na { // 如果有手动项在测试或未测试
                 instance.overall_status = OverallTestStatus::ManualTesting;
            } else if old_status == OverallTestStatus::Retesting {
                instance.overall_status = OverallTestStatus::Retesting; // 保持 Retesting 直到所有项完成
            } else {
                // 一般性的Testing，可能是通信测试等早期阶段
                // 保持之前的状态，或者根据更细的逻辑判断
            }
        } else if all_required_passed {
            instance.overall_status = OverallTestStatus::TestCompletedPassed;
            instance.error_message = None; // 清除之前的错误
        } else if all_hard_points_done && hard_point_status == SubTestStatus::Passed { // 硬点测试完成且通过
            if all_manual_points_done_or_na { // 并且所有手动项也完成了 (或不适用)
                 // 这种情况理论上应该被 any_failed 或 all_required_passed 覆盖
                 // 如果到这里，说明手动项并非都 Passed，但也没有 Failed，可能有 Skipped 或 NotTested (逻辑需要细化)
                 // 暂时先认为，如果硬点通过，且手动项没有失败，也没有在测试，则认为是HardPointCompleted
                 // 如果手动项有NotTested，则应该是ManualTesting状态
                 if instance.sub_test_results.values().any(|r| r.status == SubTestStatus::NotTested && r.status != SubTestStatus::NotApplicable) {
                    instance.overall_status = OverallTestStatus::ManualTesting;
                 } else { // 所有手动项都已处理 (Passed, Skipped, NA)，且硬点通过，那就是整体通过
                    instance.overall_status = OverallTestStatus::TestCompletedPassed;
                 }

            } else { // 硬点完成，但有手动项未完成
                instance.overall_status = OverallTestStatus::HardPointTestCompleted;
            }
        } else if communication_status == SubTestStatus::Passed && instance.overall_status == OverallTestStatus::NotTested { // 通信测试通过，且之前是NotTested
            // 仅当通信测试是第一个强制步骤时，这个状态才有意义
            // instance.overall_status = OverallTestStatus::WiringConfirmed; // 或者由 wiring_confirmed 方法设置
        }
        // 如果是 Retesting 状态，并且没有任何 failed 或 testing，且 all_required_passed，则变为 TestCompletedPassed
        else if old_status == OverallTestStatus::Retesting && !any_failed && !any_testing && all_required_passed {
             instance.overall_status = OverallTestStatus::TestCompletedPassed;
        }


        // 更新时间戳和耗时
        if matches!(instance.overall_status, OverallTestStatus::TestCompletedPassed | OverallTestStatus::TestCompletedFailed | OverallTestStatus::Skipped) {
            if instance.final_test_time.is_none() { // 只有在首次达到终态时设置
                instance.final_test_time = Some(Utc::now());
            }
        } else {
            instance.final_test_time = None; // 如果从终态变回非终态（例如重测），清除 final_test_time
        }

        if let (Some(start), Some(end)) = (instance.start_time, instance.final_test_time) {
            instance.total_test_duration_ms = Some((end - start).num_milliseconds());
        } else if instance.start_time.is_some() && instance.final_test_time.is_none() {
             instance.total_test_duration_ms = Some((Utc::now() - instance.start_time.unwrap()).num_milliseconds());
        } else {
            instance.total_test_duration_ms = None;
        }
        
        if old_status != instance.overall_status {
            debug!("Instance {} OverallStatus changed from {:?} to {:?}", instance.instance_id, old_status, instance.overall_status);
            instance.current_step_details = Some(format!("Status updated to: {:?}", instance.overall_status));
        }
        instance.last_updated_time = Utc::now();
    }
}


// 临时的辅助结构体和方法，用于 reset_for_retest
// 在实际应用中，ChannelPointDefinition 应该从持久化服务中获取
impl ChannelPointDefinition {
    fn get_by_id_placeholder(id: &str) -> Self {
        // 这只是一个占位符！实际中你需要从数据库或其他地方加载定义
        warn!("Using placeholder ChannelPointDefinition for id: {}. This should be replaced with actual data loading.", id);
        ChannelPointDefinition {
            id: id.to_string(),
            tag: "PlaceholderTag".to_string(),
            variable_name: "PlaceholderVar".to_string(),
            variable_description: "PlaceholderDesc".to_string(),
            station_name: "PlaceholderStation".to_string(),
            module_name: "PlaceholderModule".to_string(),
            module_type: ModuleType::AI, // 默认一个类型
            channel_tag_in_module: "1".to_string(),
            data_type: crate::models::enums::PointDataType::Float,
            power_supply_type: "24VDC".to_string(),
            wire_system: "2-wire".to_string(),
            plc_absolute_address: None,
            plc_communication_address: "DB1.DBD0".to_string(),
            range_lower_limit: Some(0.0),
            range_upper_limit: Some(100.0),
            engineering_unit: Some("%".to_string()),
            sll_set_value: None,
            sll_set_point_address: None,
            sll_feedback_address: None,
            sl_set_value: None,
            sl_set_point_address: None,
            sl_feedback_address: None,
            sh_set_value: None,
            sh_set_point_address: None,
            sh_feedback_address: None,
            shh_set_value: None,
            shh_set_point_address: None,
            shh_feedback_address: None,
            maintenance_value_set_point_address: None,
            maintenance_enable_switch_point_address: None,
            access_property: None,
            save_history: None,
            power_failure_protection: None,
            test_rig_plc_address: None,
            // created_at: Utc::now(), // 如果结构体有这些字段
            // updated_at: Utc::now(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::enums::{ModuleType, PointDataType};
    use crate::models::structs::ChannelPointDefinition; // 确保完整路径
    use chrono::Utc;

    // 辅助函数：创建一个基础的 ChannelPointDefinition 用于测试
    fn create_test_definition(id: &str, module_type: ModuleType) -> ChannelPointDefinition {
        ChannelPointDefinition {
            id: id.to_string(),
            tag: format!("Tag_{}", id),
            variable_name: format!("Var_{}", id),
            variable_description: format!("Desc_{}", id),
            station_name: "TestStation".to_string(),
            module_name: "TestModule".to_string(),
            module_type,
            channel_tag_in_module: "1".to_string(),
            data_type: PointDataType::Float, // 默认 Float，可按需修改
            power_supply_type: "24VDC".to_string(),
            wire_system: "2-wire".to_string(),
            plc_absolute_address: None,
            plc_communication_address: "PLC.Address.Test".to_string(),
            range_lower_limit: Some(0.0),
            range_upper_limit: Some(100.0),
            engineering_unit: Some("%".to_string()),
            sll_set_value: None,
            sll_set_point_address: None,
            sll_feedback_address: None,
            sl_set_value: None,
            sl_set_point_address: None,
            sl_feedback_address: None,
            sh_set_value: None,
            sh_set_point_address: None,
            sh_feedback_address: None,
            shh_set_value: None,
            shh_set_point_address: None,
            shh_feedback_address: None,
            maintenance_value_set_point_address: None,
            maintenance_enable_switch_point_address: None,
            access_property: None,
            save_history: None,
            power_failure_protection: None,
            test_rig_plc_address: None,
        }
    }

    #[test]
    fn test_initialize_ai_channel_no_alarms_no_maintenance() {
        let manager = ChannelStateManager::new();
        let definition = create_test_definition("AI001", ModuleType::AI);
        let batch_id = "Batch01";
        let now = Utc::now();

        let instance = manager.initialize_channel_test_instance(&definition, batch_id, now, None).unwrap();

        assert_eq!(instance.definition_id, "AI001");
        assert_eq!(instance.test_batch_id, batch_id);
        assert_eq!(instance.overall_status, OverallTestStatus::NotTested); // 初始状态

        // 检查 AI 点位的基础适用测试项
        assert_eq!(instance.sub_test_results.get(&SubTestItem::CommunicationTest).unwrap().status, SubTestStatus::NotTested);
        assert_eq!(instance.sub_test_results.get(&SubTestItem::HardPoint).unwrap().status, SubTestStatus::NotTested);
        assert_eq!(instance.sub_test_results.get(&SubTestItem::TrendCheck).unwrap().status, SubTestStatus::NotTested);
        assert_eq!(instance.sub_test_results.get(&SubTestItem::ReportCheck).unwrap().status, SubTestStatus::NotTested);
        
        // 由于 definition 中没有配置报警和维护点，这些应为 NotApplicable
        assert_eq!(instance.sub_test_results.get(&SubTestItem::LowLowAlarm).unwrap().status, SubTestStatus::NotApplicable);
        assert_eq!(instance.sub_test_results.get(&SubTestItem::LowAlarm).unwrap().status, SubTestStatus::NotApplicable);
        assert_eq!(instance.sub_test_results.get(&SubTestItem::HighAlarm).unwrap().status, SubTestStatus::NotApplicable);
        assert_eq!(instance.sub_test_results.get(&SubTestItem::HighHighAlarm).unwrap().status, SubTestStatus::NotApplicable);
        // AlarmValueSetting 聚合状态，即使单个报警不适用，它也应该是 NotTested (或根据逻辑调整为NA)
        // 当前 get_applicable_sub_tests 中 AlarmValueSetting 默认 NotTested，如果所有报警都是NA，它是否应该是NA？
        // 根据当前 get_applicable_sub_tests 实现，它会是 NotTested。
        assert_eq!(instance.sub_test_results.get(&SubTestItem::AlarmValueSetting).unwrap().status, SubTestStatus::NotTested); 
        assert_eq!(instance.sub_test_results.get(&SubTestItem::MaintenanceFunction).unwrap().status, SubTestStatus::NotApplicable);
    }

    #[test]
    fn test_initialize_ai_channel_with_all_alarms_and_maintenance() {
        let manager = ChannelStateManager::new();
        let mut definition = create_test_definition("AI002", ModuleType::AI);
        definition.sll_set_point_address = Some("SLL.Set".to_string());
        definition.sll_feedback_address = Some("SLL.Fb".to_string());
        definition.sl_set_point_address = Some("SL.Set".to_string());
        definition.sl_feedback_address = Some("SL.Fb".to_string());
        definition.sh_set_point_address = Some("SH.Set".to_string());
        definition.sh_feedback_address = Some("SH.Fb".to_string());
        definition.shh_set_point_address = Some("SHH.Set".to_string());
        definition.shh_feedback_address = Some("SHH.Fb".to_string());
        definition.maintenance_value_set_point_address = Some("Maint.Set".to_string());
        definition.maintenance_enable_switch_point_address = Some("Maint.En".to_string());

        let batch_id = "Batch02";
        let now = Utc::now();
        let instance = manager.initialize_channel_test_instance(&definition, batch_id, now, None).unwrap();

        assert_eq!(instance.overall_status, OverallTestStatus::NotTested);
        assert_eq!(instance.sub_test_results.get(&SubTestItem::LowLowAlarm).unwrap().status, SubTestStatus::NotTested);
        assert_eq!(instance.sub_test_results.get(&SubTestItem::LowAlarm).unwrap().status, SubTestStatus::NotTested);
        assert_eq!(instance.sub_test_results.get(&SubTestItem::HighAlarm).unwrap().status, SubTestStatus::NotTested);
        assert_eq!(instance.sub_test_results.get(&SubTestItem::HighHighAlarm).unwrap().status, SubTestStatus::NotTested);
        assert_eq!(instance.sub_test_results.get(&SubTestItem::AlarmValueSetting).unwrap().status, SubTestStatus::NotTested);
        assert_eq!(instance.sub_test_results.get(&SubTestItem::MaintenanceFunction).unwrap().status, SubTestStatus::NotTested);
    }

    #[test]
    fn test_initialize_do_channel() {
        let manager = ChannelStateManager::new();
        let definition = create_test_definition("DO001", ModuleType::DO);
        let batch_id = "Batch03";
        let now = Utc::now();
        let instance = manager.initialize_channel_test_instance(&definition, batch_id, now, None).unwrap();

        assert_eq!(instance.overall_status, OverallTestStatus::NotTested);
        assert_eq!(instance.sub_test_results.get(&SubTestItem::CommunicationTest).unwrap().status, SubTestStatus::NotTested);
        assert_eq!(instance.sub_test_results.get(&SubTestItem::HardPoint).unwrap().status, SubTestStatus::NotTested);
        assert_eq!(instance.sub_test_results.get(&SubTestItem::StateDisplay).unwrap().status, SubTestStatus::NotTested);

        // 对于 DO 点，AI 特有的测试项应为 NotApplicable
        assert_eq!(instance.sub_test_results.get(&SubTestItem::TrendCheck).map(|r| r.status), None); // 或者检查是否不存在
        assert_eq!(instance.sub_test_results.get(&SubTestItem::LowLowAlarm).map(|r| r.status), None);
    }
    
    #[test]
    fn test_initialize_communication_channel() {
        let manager = ChannelStateManager::new();
        let definition = create_test_definition("COMM001", ModuleType::Communication);
        let batch_id = "Batch04";
        let now = Utc::now();
        let instance = manager.initialize_channel_test_instance(&definition, batch_id, now, None).unwrap();

        assert_eq!(instance.overall_status, OverallTestStatus::NotTested);
        assert_eq!(instance.sub_test_results.get(&SubTestItem::CommunicationTest).unwrap().status, SubTestStatus::NotTested);
        // 对于纯通信模块，HardPoint 可能不适用
        assert_eq!(instance.sub_test_results.get(&SubTestItem::HardPoint).unwrap().status, SubTestStatus::NotApplicable);
        assert_eq!(instance.sub_test_results.get(&SubTestItem::StateDisplay).map(|r| r.status), None);
    }
} 