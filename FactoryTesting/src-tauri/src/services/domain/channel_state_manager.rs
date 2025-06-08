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
use log::{info, error, warn, debug, trace};

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

    /// 重置为重新分配状态（新增方法）
    async fn reset_for_reallocation(&self, instance: &mut ChannelTestInstance) -> AppResult<()>;

    /// 创建新的测试实例（兼容现有接口）
    async fn create_test_instance(
        &self,
        definition_id: &str,
        batch_id: &str,
    ) -> AppResult<ChannelTestInstance>;

    /// 获取测试实例状态
    async fn get_instance_state(&self, instance_id: &str) -> AppResult<ChannelTestInstance>;

    /// 更新测试结果
    async fn update_test_result(&self, outcome: RawTestOutcome) -> AppResult<()>;

    /// 更新实例整体状态
    async fn update_overall_status(
        &self,
        instance_id: &str,
        status: OverallTestStatus,
    ) -> AppResult<()>;

    /// 存储批次分配结果到状态管理器
    async fn store_batch_allocation_result(
        &self,
        allocation_result: crate::commands::data_management::AllocationResult,
    ) -> AppResult<()>;

    /// 获取通道定义
    async fn get_channel_definition(&self, definition_id: &str) -> Option<ChannelPointDefinition>;

    /// 🔧 获取内存缓存中的测试实例
    async fn get_cached_test_instance(&self, instance_id: &str) -> Option<ChannelTestInstance>;

    /// 🔧 获取所有缓存的测试实例
    async fn get_all_cached_test_instances(&self) -> Vec<ChannelTestInstance>;
}

/// 通道状态管理器实现
pub struct ChannelStateManager {
    /// 持久化服务
    persistence_service: Arc<dyn IPersistenceService>,
    /// 通道定义内存缓存
    channel_definitions_cache: Arc<std::sync::RwLock<HashMap<String, ChannelPointDefinition>>>,
    /// 🔧 测试实例内存缓存 - 关键修复
    test_instances_cache: Arc<std::sync::RwLock<HashMap<String, ChannelTestInstance>>>,
}

impl ChannelStateManager {
    /// 创建新的通道状态管理器
    pub fn new(persistence_service: Arc<dyn IPersistenceService>) -> Self {
        Self {
            persistence_service,
            channel_definitions_cache: Arc::new(std::sync::RwLock::new(HashMap::new())),
            test_instances_cache: Arc::new(std::sync::RwLock::new(HashMap::new())),
        }
    }

    /// 评估整体状态（私有方法）
    fn evaluate_overall_status(&self, instance: &mut ChannelTestInstance) {
        let mut all_required_passed = true;
        let mut any_failed = false;
        let mut hard_point_completed = false;
        let mut has_manual_tests = false;
        let mut manual_tests_completed = true;

        trace!("🔍 [EVALUATE_STATUS] 开始评估状态: {}", instance.instance_id);

        // 遍历所有子测试结果
        for (sub_test_item, result) in &instance.sub_test_results {
            trace!("🔍 [EVALUATE_STATUS] 检查子测试: {:?} -> {:?}", sub_test_item, result.status);

            match result.status {
                SubTestStatus::Failed => {
                    trace!("🔍 [EVALUATE_STATUS] 发现失败测试: {:?}", sub_test_item);
                    any_failed = true;
                    all_required_passed = false;
                }
                SubTestStatus::NotTested => {
                    if self.is_required_test(sub_test_item) {
                        trace!("🔍 [EVALUATE_STATUS] 必需测试未完成: {:?}", sub_test_item);
                        all_required_passed = false;
                    }
                    if self.is_manual_test(sub_test_item) {
                        manual_tests_completed = false;
                    }
                }
                SubTestStatus::Passed => {
                    trace!("🔍 [EVALUATE_STATUS] 测试通过: {:?}", sub_test_item);
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

        trace!("🔍 [EVALUATE_STATUS] 状态评估结果:");
        trace!("   - any_failed: {}", any_failed);
        trace!("   - all_required_passed: {}", all_required_passed);
        trace!("   - hard_point_completed: {}", hard_point_completed);
        trace!("   - has_manual_tests: {}", has_manual_tests);
        trace!("   - manual_tests_completed: {}", manual_tests_completed);

        // 根据状态机规则更新整体状态
        let new_status = if any_failed {
            trace!("🔍 [EVALUATE_STATUS] 选择状态: TestCompletedFailed (因为有失败测试)");
            OverallTestStatus::TestCompletedFailed
        } else if all_required_passed {
            trace!("🔍 [EVALUATE_STATUS] 选择状态: TestCompletedPassed (所有必需测试通过)");
            OverallTestStatus::TestCompletedPassed
        } else if hard_point_completed && has_manual_tests && !manual_tests_completed {
            trace!("🔍 [EVALUATE_STATUS] 选择状态: HardPointTestCompleted (硬点完成，等待手动测试)");
            OverallTestStatus::HardPointTestCompleted
        } else if hard_point_completed {
            trace!("🔍 [EVALUATE_STATUS] 选择状态: HardPointTestCompleted (硬点完成)");
            OverallTestStatus::HardPointTestCompleted
        } else {
            trace!("🔍 [EVALUATE_STATUS] 选择状态: NotTested (默认状态)");
            OverallTestStatus::NotTested
        };

        instance.overall_status = new_status;
        trace!("🔍 [EVALUATE_STATUS] 最终状态: {:?}", instance.overall_status);

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
        trace!("🔍 [APPLY_OUTCOME] 开始应用测试结果: {} -> {:?} ({})",
              instance.instance_id, outcome.sub_test_item, outcome.success);

        // 🔧 修复：如果 sub_test_results 是空的，先初始化它
        if instance.sub_test_results.is_empty() {
            warn!("🔧 [APPLY_OUTCOME] 检测到空的 sub_test_results，正在修复...");

            // 尝试获取通道定义来正确初始化
            if let Some(definition) = self.get_channel_definition(&instance.definition_id).await {
                // 🔧 使用现有的 initialize_sub_test_results 方法
                instance.sub_test_results = self.initialize_sub_test_results(&definition.module_type);
                info!("🔧 [APPLY_OUTCOME] 已根据通道定义初始化 {} 个子测试项", instance.sub_test_results.len());
            } else {
                // 如果找不到定义，至少添加当前测试项
                instance.sub_test_results.insert(
                    outcome.sub_test_item.clone(),
                    SubTestExecutionResult::new(SubTestStatus::NotTested, None, None, None)
                );
                info!("🔧 [APPLY_OUTCOME] 已添加当前测试项: {:?}", outcome.sub_test_item);
            }
        }

        // 🔧 调试：检查 sub_test_results 的状态 - 完全移除冗余日志
        trace!("🔍 [APPLY_OUTCOME] sub_test_results 包含 {} 个项目:", instance.sub_test_results.len());
        for (item, result) in &instance.sub_test_results {
            trace!("   - {:?}: {:?}", item, result.status);
        }

        // 检查是否存在对应的子测试项，如果不存在则动态添加
        if !instance.sub_test_results.contains_key(&outcome.sub_test_item) {
            warn!("🔧 [APPLY_OUTCOME] 动态添加缺失的子测试项: {:?}", outcome.sub_test_item);
            instance.sub_test_results.insert(
                outcome.sub_test_item.clone(),
                SubTestExecutionResult::new(SubTestStatus::NotTested, None, None, None)
            );
        }

        // 更新对应的子测试结果
        if let Some(sub_result) = instance.sub_test_results.get_mut(&outcome.sub_test_item) {
            trace!("🔍 [APPLY_OUTCOME] 找到对应的子测试项: {:?}", outcome.sub_test_item);
            sub_result.status = if outcome.success {
                SubTestStatus::Passed
            } else {
                SubTestStatus::Failed
            };
            sub_result.timestamp = outcome.end_time;
            sub_result.actual_value = outcome.raw_value_read.clone();
            sub_result.expected_value = outcome.eng_value_calculated.clone();
            sub_result.details = outcome.message.clone();
            trace!("🔍 [APPLY_OUTCOME] 子测试状态已更新为: {:?}", sub_result.status);
        } else {
            error!("❌ [APPLY_OUTCOME] 这不应该发生：仍然找不到子测试项: {:?}", outcome.sub_test_item);
        }

        // 🔧 处理硬点测试结果 - 存储百分比数据和硬点读数
        if outcome.sub_test_item == SubTestItem::HardPoint {
            // 存储硬点读数到实例中（AI/AO点位）
            if let Some(readings) = &outcome.readings {
                instance.hardpoint_readings = Some(readings.clone());
                trace!("🔍 [APPLY_OUTCOME] 已存储硬点读数数据");
            }

            // 存储数字量测试步骤到实例中（DI/DO点位）
            if let Some(digital_steps) = &outcome.digital_steps {
                instance.digital_test_steps = Some(digital_steps.clone());
                trace!("🔍 [APPLY_OUTCOME] 已存储数字量测试步骤数据");
            }

            // 🔧 处理百分比测试结果 - 优先使用outcome中的直接数据，但总是尝试从readings中提取
            let mut percentage_data_stored = false;

            if outcome.test_result_0_percent.is_some() {
                // 直接从outcome中提取百分比测试结果，存储到临时数据中
                instance.transient_data.insert("test_result_0_percent".to_string(),
                    serde_json::json!(outcome.test_result_0_percent));
                instance.transient_data.insert("test_result_25_percent".to_string(),
                    serde_json::json!(outcome.test_result_25_percent));
                instance.transient_data.insert("test_result_50_percent".to_string(),
                    serde_json::json!(outcome.test_result_50_percent));
                instance.transient_data.insert("test_result_75_percent".to_string(),
                    serde_json::json!(outcome.test_result_75_percent));
                instance.transient_data.insert("test_result_100_percent".to_string(),
                    serde_json::json!(outcome.test_result_100_percent));

                trace!("🔍 [APPLY_OUTCOME] 已从outcome直接存储百分比测试结果");
                percentage_data_stored = true;
            }

            // 🔧 总是尝试从readings中提取数据（作为备选或补充）
            if let Some(readings) = &outcome.readings {
                if readings.len() >= 5 && !percentage_data_stored {
                    instance.transient_data.insert("test_result_0_percent".to_string(),
                        serde_json::json!(readings[0].actual_reading_eng.map(|v| v as f64)));
                    instance.transient_data.insert("test_result_25_percent".to_string(),
                        serde_json::json!(readings[1].actual_reading_eng.map(|v| v as f64)));
                    instance.transient_data.insert("test_result_50_percent".to_string(),
                        serde_json::json!(readings[2].actual_reading_eng.map(|v| v as f64)));
                    instance.transient_data.insert("test_result_75_percent".to_string(),
                        serde_json::json!(readings[3].actual_reading_eng.map(|v| v as f64)));
                    instance.transient_data.insert("test_result_100_percent".to_string(),
                        serde_json::json!(readings[4].actual_reading_eng.map(|v| v as f64)));

                    trace!("🔍 [APPLY_OUTCOME] 已从readings提取百分比测试结果到临时数据");
                    percentage_data_stored = true;
                }
            }

            if !percentage_data_stored {
                warn!("⚠️ [APPLY_OUTCOME] 未能存储百分比测试结果：outcome中无直接数据且readings不足5个");
            }
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

    /// 重置为重新分配状态（新增方法）
    async fn reset_for_reallocation(&self, instance: &mut ChannelTestInstance) -> AppResult<()> {
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

        info!("重置为重新分配状态: {}", instance.instance_id);
        Ok(())
    }

    /// 创建新的测试实例（兼容现有接口）
    async fn create_test_instance(
        &self,
        definition_id: &str,
        batch_id: &str,
    ) -> AppResult<ChannelTestInstance> {
        // 🔧 修复：获取通道定义以便正确初始化 sub_test_results
        let definition = match self.get_channel_definition(definition_id).await {
            Some(def) => def,
            None => {
                // 如果找不到定义，创建一个默认的实例（向后兼容）
                warn!("⚠️ [STATE_MANAGER] 未找到通道定义: {}，创建默认实例", definition_id);
                let mut instance = ChannelTestInstance::new(
                    definition_id.to_string(),
                    batch_id.to_string(),
                );
                // 至少初始化硬点测试项
                instance.sub_test_results.insert(
                    SubTestItem::HardPoint,
                    SubTestExecutionResult::new(SubTestStatus::NotTested, None, None, None)
                );
                info!("创建默认测试实例: {}", instance.instance_id);
                return Ok(instance);
            }
        };

        // 🔧 使用正确的初始化方法
        let instance = self.initialize_channel_test_instance(definition, batch_id.to_string()).await?;

        info!("创建测试实例: {} (已正确初始化sub_test_results)", instance.instance_id);
        Ok(instance)
    }

    /// 获取测试实例状态
    async fn get_instance_state(&self, instance_id: &str) -> AppResult<ChannelTestInstance> {
        // TODO: 从持久化服务获取实例状态
        Err(AppError::not_found_error("测试实例", &format!("测试实例未找到: {}", instance_id)))
    }

    /// 更新测试结果
    async fn update_test_result(&self, outcome: RawTestOutcome) -> AppResult<()> {
        let instance_id = outcome.channel_instance_id.clone();
        // 完全移除状态管理器的冗余日志
        trace!("🔍 [STATE_MANAGER] 尝试更新测试结果: {} -> {:?}", instance_id, outcome.success);

        // 🔧 第一步：尝试从内存缓存获取测试实例
        let mut instance_from_cache = {
            let cache = self.test_instances_cache.read().unwrap();
            let cached_result = cache.get(&instance_id).cloned();
            trace!("🔍 [STATE_MANAGER] 内存缓存查询结果: {}", if cached_result.is_some() { "找到" } else { "未找到" });
            cached_result
        };

        // 🔧 第二步：如果缓存中没有，从数据库加载
        if instance_from_cache.is_none() {
            trace!("🔍 [STATE_MANAGER] 准备从数据库查询实例ID: {}", instance_id);
            match self.persistence_service.load_test_instance(&instance_id).await {
                Ok(Some(instance)) => {
                    trace!("✅ [STATE_MANAGER] 从数据库加载测试实例: {} (定义ID: {})", instance_id, instance.definition_id);

                    // 将实例添加到缓存
                    {
                        let mut cache = self.test_instances_cache.write().unwrap();
                        cache.insert(instance_id.to_string(), instance.clone());
                    }

                    instance_from_cache = Some(instance);
                }
                Ok(None) => {
                    warn!("⚠️ [STATE_MANAGER] 数据库中未找到测试实例: {}", instance_id);

                    // 🔧 添加调试信息：列出数据库中的所有实例ID
                    match self.persistence_service.load_all_test_instances().await {
                        Ok(all_instances) => {
                            warn!("🔍 [STATE_MANAGER] 数据库中共有 {} 个测试实例", all_instances.len());
                            if all_instances.len() <= 20 {
                                for (i, inst) in all_instances.iter().enumerate() {
                                    warn!("   {}. 实例ID: {} (长度: {})", i + 1, inst.instance_id, inst.instance_id.len());
                                    if inst.instance_id.contains(&instance_id[0..20]) {
                                        warn!("      ⚠️ 部分匹配的实例！");
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            error!("❌ [STATE_MANAGER] 查询所有测试实例失败: {}", e);
                        }
                    }

                    return Err(AppError::not_found_error("测试实例", &format!("实例ID: {}", instance_id)));
                }
                Err(e) => {
                    error!("❌ [STATE_MANAGER] 加载测试实例失败: {} - {}", instance_id, e);
                    return Err(e);
                }
            }
        } else {
            trace!("✅ [STATE_MANAGER] 从内存缓存获取测试实例: {}", instance_id);
        }

        // 🔧 第三步：更新测试实例状态
        if let Some(mut instance) = instance_from_cache {
            // 应用测试结果
            self.apply_raw_outcome(&mut instance, outcome).await?;

            // 🔧 第四步：同时更新内存缓存和数据库
            {
                let mut cache = self.test_instances_cache.write().unwrap();
                cache.insert(instance_id.to_string(), instance.clone());
            }

            // 保存到数据库
            self.persistence_service.save_test_instance(&instance).await?;

            // 🔧 修复：立即验证数据是否正确保存
            if let Some(ref digital_steps) = instance.digital_test_steps {
                log::info!("🔍 [STATE_MANAGER] 保存后验证 - digital_test_steps 数量: {}", digital_steps.len());

                // 立即从数据库重新加载验证
                if let Ok(Some(reloaded_instance)) = self.persistence_service.load_test_instance(&instance_id).await {
                    if let Some(ref reloaded_steps) = reloaded_instance.digital_test_steps {
                        log::info!("✅ [STATE_MANAGER] 数据库验证成功 - digital_test_steps 数量: {}", reloaded_steps.len());
                    } else {
                        log::error!("❌ [STATE_MANAGER] 数据库验证失败 - digital_test_steps 为空！");
                    }
                }
            }

            trace!("✅ [STATE_MANAGER] 成功更新测试结果: {} -> {:?}", instance_id, instance.overall_status);
        }

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

    /// 存储批次分配结果到状态管理器
    async fn store_batch_allocation_result(
        &self,
        allocation_result: crate::commands::data_management::AllocationResult,
    ) -> AppResult<()> {
        // 首先保存通道定义到数据库

        // 从分配结果中获取所有通道定义
        if let Some(ref channel_definitions) = allocation_result.channel_definitions {


            let mut saved_count = 0;
            let mut failed_count = 0;

            for definition in channel_definitions.iter() {
                match self.persistence_service.save_channel_definition(definition).await {
                    Ok(_) => {
                        saved_count += 1;
                    }
                    Err(e) => {
                        failed_count += 1;
                        error!("❌ [STATE_MANAGER] 保存通道定义到数据库失败: ID={}, Tag={} - {}",
                            definition.id, definition.tag, e);
                        // 不要因为单个定义失败而中断整个流程
                    }
                }
            }

    

            if failed_count > 0 {
                warn!("⚠️ [STATE_MANAGER] 有{}个通道定义保存失败，但继续处理", failed_count);
            }
        } else {
            warn!("⚠️ [STATE_MANAGER] 分配结果中没有通道定义数据");
        }

        // 步骤2: 将通道定义存储到内存缓存中

        // 从测试实例中提取所有相关的通道定义ID
        let definition_ids: std::collections::HashSet<String> = allocation_result.allocated_instances
            .iter()
            .map(|instance| instance.definition_id.clone())
            .collect();



        // 从数据库加载这些通道定义并存储到缓存中
        let mut loaded_definitions = Vec::new();
        let mut loaded_count = 0;
        let mut not_found_count = 0;
        let mut error_count = 0;

        for definition_id in &definition_ids {
            match self.persistence_service.load_channel_definition(definition_id).await {
                Ok(Some(definition)) => {
                    loaded_count += 1;
                    loaded_definitions.push((definition_id.clone(), definition));
                }
                Ok(None) => {
                    not_found_count += 1;
                    warn!("⚠️ [STATE_MANAGER] 数据库中未找到通道定义: {}", definition_id);
                }
                Err(e) => {
                    error_count += 1;
                    error!("❌ [STATE_MANAGER] 加载通道定义失败: {} - {}", definition_id, e);
                }
            }
        }



        // 将加载的定义存储到缓存中（避免跨await持有锁）
        {
            let mut cache = self.channel_definitions_cache.write().unwrap();
            for (definition_id, definition) in loaded_definitions {
                cache.insert(definition_id, definition);
            }

        }

        // 将批次信息保存到持久化服务
        let mut batch_saved_count = 0;
        let mut batch_failed_count = 0;

        for batch in &allocation_result.batches {
            if let Err(e) = self.persistence_service.save_batch_info(batch).await {
                batch_failed_count += 1;
                error!("🔥 [STATE_MANAGER] 保存批次信息失败: {} - {}", batch.batch_id, e);
            } else {
                batch_saved_count += 1;
            }
        }

        // 🔧 将测试实例保存到持久化服务和内存缓存
        let mut instance_saved_count = 0;
        let mut instance_failed_count = 0;

        for instance in &allocation_result.allocated_instances {
            // 保存到数据库
            if let Err(e) = self.persistence_service.save_test_instance(instance).await {
                instance_failed_count += 1;
                error!("🔥 [STATE_MANAGER] 保存测试实例到数据库失败: {} - {}", instance.instance_id, e);
            } else {
                instance_saved_count += 1;

                // 🔧 同时保存到内存缓存
                {
                    let mut cache = self.test_instances_cache.write().unwrap();
                    cache.insert(instance.instance_id.clone(), instance.clone());
                }

                info!("✅ [STATE_MANAGER] 测试实例已保存到数据库和缓存: {}", instance.instance_id);
            }
        }


        Ok(())
    }

    /// 获取通道定义
    async fn get_channel_definition(&self, definition_id: &str) -> Option<ChannelPointDefinition> {
        // 首先尝试从内存缓存获取
        {
            let cache = self.channel_definitions_cache.read().unwrap();
            if let Some(definition) = cache.get(definition_id) {
                return Some(definition.clone());
            }
        }

        // 如果缓存中没有，则从数据库获取并缓存
        match self.persistence_service.load_channel_definition(definition_id).await {
            Ok(Some(definition)) => {
                // 将定义存储到缓存中
                {
                    let mut cache = self.channel_definitions_cache.write().unwrap();
                    cache.insert(definition_id.to_string(), definition.clone());
                }

                Some(definition)
            }
            Ok(None) => {
                None
            }
            Err(e) => {
                warn!("⚠️ [STATE_MANAGER] 获取通道定义失败: {} - {}", definition_id, e);
                None
            }
        }
    }

    /// 🔧 获取内存缓存中的测试实例
    async fn get_cached_test_instance(&self, instance_id: &str) -> Option<ChannelTestInstance> {
        let cache = self.test_instances_cache.read().unwrap();
        cache.get(instance_id).cloned()
    }

    /// 🔧 获取所有缓存的测试实例
    async fn get_all_cached_test_instances(&self) -> Vec<ChannelTestInstance> {
        let cache = self.test_instances_cache.read().unwrap();
        cache.values().cloned().collect()
    }
}