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
use log::{info, error, warn};

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

    /// 存储批次分配结果到状态管理器
    async fn store_batch_allocation_result(
        &self,
        allocation_result: crate::commands::data_management::AllocationResult,
    ) -> AppResult<()>;

    /// 获取通道定义
    async fn get_channel_definition(&self, definition_id: &str) -> Option<ChannelPointDefinition>;
}

/// 通道状态管理器实现
pub struct ChannelStateManager {
    /// 持久化服务
    persistence_service: Arc<dyn IPersistenceService>,
    /// 通道定义内存缓存
    channel_definitions_cache: Arc<std::sync::RwLock<HashMap<String, ChannelPointDefinition>>>,
}

impl ChannelStateManager {
    /// 创建新的通道状态管理器
    pub fn new(persistence_service: Arc<dyn IPersistenceService>) -> Self {
        Self {
            persistence_service,
            channel_definitions_cache: Arc::new(std::sync::RwLock::new(HashMap::new())),
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

    /// 存储批次分配结果到状态管理器
    async fn store_batch_allocation_result(
        &self,
        allocation_result: crate::commands::data_management::AllocationResult,
    ) -> AppResult<()> {
        info!("🔥 [STATE_MANAGER] 存储批次分配结果到状态管理器");
        info!("🔥 [STATE_MANAGER] 批次数量: {}", allocation_result.batches.len());
        info!("🔥 [STATE_MANAGER] 分配实例数量: {}", allocation_result.allocated_instances.len());

        // 🔧 修复：首先保存通道定义到数据库
        info!("🔥 [STATE_MANAGER] 步骤1: 保存通道定义到数据库");

        // 从分配结果中获取所有通道定义
        if let Some(ref channel_definitions) = allocation_result.channel_definitions {
            info!("🔥 [STATE_MANAGER] 开始保存{}个通道定义到数据库", channel_definitions.len());

            let mut saved_count = 0;
            let mut failed_count = 0;

            for (index, definition) in channel_definitions.iter().enumerate() {
                info!("💾 [STATE_MANAGER] 保存定义 {}/{}: ID={}, Tag={}",
                    index + 1, channel_definitions.len(), definition.id, definition.tag);

                match self.persistence_service.save_channel_definition(definition).await {
                    Ok(_) => {
                        saved_count += 1;
                        info!("✅ [STATE_MANAGER] 成功保存通道定义到数据库: ID={}, Tag={}",
                            definition.id, definition.tag);
                    }
                    Err(e) => {
                        failed_count += 1;
                        error!("❌ [STATE_MANAGER] 保存通道定义到数据库失败: ID={}, Tag={} - {}",
                            definition.id, definition.tag, e);
                        // 不要因为单个定义失败而中断整个流程
                    }
                }
            }

            info!("✅ [STATE_MANAGER] 通道定义保存完成: 成功={}, 失败={}", saved_count, failed_count);

            if failed_count > 0 {
                warn!("⚠️ [STATE_MANAGER] 有{}个通道定义保存失败，但继续处理", failed_count);
            }
        } else {
            warn!("⚠️ [STATE_MANAGER] 分配结果中没有通道定义数据");
        }

        // 步骤2: 将通道定义存储到内存缓存中
        info!("🔥 [STATE_MANAGER] 步骤2: 将通道定义存储到内存缓存");

        // 从测试实例中提取所有相关的通道定义ID
        let definition_ids: std::collections::HashSet<String> = allocation_result.allocated_instances
            .iter()
            .map(|instance| instance.definition_id.clone())
            .collect();

        info!("🔥 [STATE_MANAGER] 需要缓存{}个通道定义", definition_ids.len());

        // 从数据库加载这些通道定义并存储到缓存中
        let mut loaded_definitions = Vec::new();
        for definition_id in &definition_ids {
            match self.persistence_service.load_channel_definition(definition_id).await {
                Ok(Some(definition)) => {
                    info!("🔥 [STATE_MANAGER] 成功加载通道定义: ID={}, Tag={}", definition_id, definition.tag);
                    loaded_definitions.push((definition_id.clone(), definition));
                }
                Ok(None) => {
                    warn!("⚠️ [STATE_MANAGER] 数据库中未找到通道定义: {}", definition_id);
                }
                Err(e) => {
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
            info!("🔥 [STATE_MANAGER] 内存缓存完成，缓存中共有{}个通道定义", cache.len());
        }

        // 详细记录批次信息
        for (index, batch) in allocation_result.batches.iter().enumerate() {
            info!("🔥 [STATE_MANAGER] 批次 {}/{}: ID={}, 名称={}, 总点位={}",
                index + 1, allocation_result.batches.len(),
                batch.batch_id, batch.batch_name, batch.total_points);
        }

        // 详细记录测试实例信息
        for (index, instance) in allocation_result.allocated_instances.iter().enumerate() {
            info!("🔥 [STATE_MANAGER] 实例 {}/{}: ID={}, 定义ID={}, 批次ID={}, 分配PLC通道={:?}",
                index + 1, allocation_result.allocated_instances.len(),
                instance.instance_id, instance.definition_id, instance.test_batch_id,
                instance.test_plc_channel_tag);
        }

        // 将批次信息保存到持久化服务
        for batch in &allocation_result.batches {
            if let Err(e) = self.persistence_service.save_batch_info(batch).await {
                error!("🔥 [STATE_MANAGER] 保存批次信息失败: {} - {}", batch.batch_id, e);
            } else {
                info!("🔥 [STATE_MANAGER] 成功保存批次信息: {}", batch.batch_id);
            }
        }

        // 将测试实例保存到持久化服务
        for instance in &allocation_result.allocated_instances {
            if let Err(e) = self.persistence_service.save_test_instance(instance).await {
                error!("🔥 [STATE_MANAGER] 保存测试实例失败: {} - {}", instance.instance_id, e);
            } else {
                info!("🔥 [STATE_MANAGER] 成功保存测试实例: {}", instance.instance_id);
            }
        }

        info!("🔥 [STATE_MANAGER] 批次分配结果存储完成");
        Ok(())
    }

    /// 获取通道定义
    async fn get_channel_definition(&self, definition_id: &str) -> Option<ChannelPointDefinition> {
        // 首先尝试从内存缓存获取
        {
            let cache = self.channel_definitions_cache.read().unwrap();
            if let Some(definition) = cache.get(definition_id) {
                info!("✅ [STATE_MANAGER] 从内存缓存获取通道定义: ID={}, Tag={}", definition_id, definition.tag);
                return Some(definition.clone());
            }
        }

        // 如果缓存中没有，则从数据库获取并缓存
        match self.persistence_service.load_channel_definition(definition_id).await {
            Ok(Some(definition)) => {
                info!("✅ [STATE_MANAGER] 从数据库获取通道定义: ID={}, Tag={}", definition_id, definition.tag);

                // 将定义存储到缓存中
                {
                    let mut cache = self.channel_definitions_cache.write().unwrap();
                    cache.insert(definition_id.to_string(), definition.clone());
                }

                Some(definition)
            }
            Ok(None) => {
                warn!("⚠️ [STATE_MANAGER] 通道定义不存在: {}", definition_id);
                None
            }
            Err(e) => {
                warn!("⚠️ [STATE_MANAGER] 获取通道定义失败: {} - {}", definition_id, e);
                None
            }
        }
    }
}