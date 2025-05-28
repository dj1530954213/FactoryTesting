use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::models::{
    ChannelPointDefinition, ChannelTestInstance, TestBatchInfo, ModuleType, OverallTestStatus
};
use crate::error::AppError;
use chrono::{DateTime, Utc};

/// 测试PLC通道映射表
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComparisonTable {
    /// 通道地址标识 (如 "AO1_1")
    pub channel_address: String,
    /// 通信地址 (如 "AO1.1")
    pub communication_address: String,
    /// 通道类型
    pub channel_type: ModuleType,
    /// 是否有源 (true=有源, false=无源)
    pub is_powered: bool,
}

/// 测试PLC配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestPlcConfig {
    /// PLC品牌类型
    pub brand_type: String,
    /// IP地址
    pub ip_address: String,
    /// 通道映射表
    pub comparison_tables: Vec<ComparisonTable>,
}

/// 批次分配结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchAllocationResult {
    /// 批次信息列表
    pub batches: Vec<TestBatchInfo>,
    /// 分配后的通道实例
    pub allocated_instances: Vec<ChannelTestInstance>,
    /// 分配统计
    pub allocation_summary: AllocationSummary,
}

/// 分配统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllocationSummary {
    /// 总定义数
    pub total_definitions: u32,
    /// 已分配实例数
    pub allocated_instances: u32,
    /// 跳过的定义数
    pub skipped_definitions: u32,
    /// 按模块类型分组的统计
    pub by_module_type: HashMap<ModuleType, ModuleTypeStats>,
    /// 分配错误列表
    pub allocation_errors: Vec<String>,
}

/// 模块类型统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleTypeStats {
    /// 定义数量
    pub definition_count: u32,
    /// 分配的实例数量
    pub allocated_count: u32,
    /// 批次数量
    pub batch_count: u32,
}

/// 通道分配服务接口
#[async_trait::async_trait]
pub trait IChannelAllocationService: Send + Sync {
    /// 为通道定义分配测试批次和测试PLC通道
    async fn allocate_channels(
        &self,
        definitions: Vec<ChannelPointDefinition>,
        test_plc_config: TestPlcConfig,
        product_model: Option<String>,
        serial_number: Option<String>,
    ) -> Result<BatchAllocationResult, AppError>;

    /// 获取指定批次的通道实例
    async fn get_batch_instances(
        &self,
        batch_id: &str,
    ) -> Result<Vec<ChannelTestInstance>, AppError>;

    /// 清除所有通道分配
    async fn clear_all_allocations(
        &self,
        instances: Vec<ChannelTestInstance>,
    ) -> Result<Vec<ChannelTestInstance>, AppError>;

    /// 验证通道分配的有效性
    async fn validate_allocations(
        &self,
        instances: &[ChannelTestInstance],
    ) -> Result<ValidationResult, AppError>;
}

/// 验证结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

/// 通道分配服务实现
pub struct ChannelAllocationService;

impl ChannelAllocationService {
    pub fn new() -> Self {
        Self
    }

    /// 按模块类型和有源/无源分组通道定义
    fn group_definitions_by_type_and_power(
        &self,
        definitions: &[ChannelPointDefinition],
    ) -> HashMap<(ModuleType, bool), Vec<ChannelPointDefinition>> {
        let mut grouped = HashMap::new();
        
        for definition in definitions {
            // 从通道定义中提取有源/无源信息
            // 根据power_supply_type字段判断有源/无源
            let is_powered = definition.power_supply_type.contains("有源");
            
            grouped
                .entry((definition.module_type.clone(), is_powered))
                .or_insert_with(Vec::new)
                .push(definition.clone());
        }
        
        grouped
    }

    /// 获取测试PLC通道映射（按类型和有源/无源筛选）
    fn get_test_channel_mappings_by_power(
        &self,
        config: &TestPlcConfig,
        channel_type: &ModuleType,
        is_powered: bool,
    ) -> Vec<ComparisonTable> {
        config
            .comparison_tables
            .iter()
            .filter(|table| &table.channel_type == channel_type && table.is_powered == is_powered)
            .cloned()
            .collect()
    }

    /// 计算所需批次数量
    fn calculate_required_batches(
        &self,
        channel_count: usize,
        test_channel_count: usize,
    ) -> usize {
        if test_channel_count == 0 {
            return 0;
        }
        // 向上取整：Math.Ceiling((double)channelsToAllocate.Count / totalTestChannelsForType)
        (channel_count + test_channel_count - 1) / test_channel_count
    }

    /// 为特定类型和有源/无源的通道分配批次和测试PLC通道
    fn allocate_channels_with_power_matching(
        &self,
        channels: &[ChannelPointDefinition],
        test_channel_mappings: &[ComparisonTable],
        base_batch_id: &str,
        module_type: &ModuleType,
        is_powered: bool,
        batch_offset: &mut usize,
    ) -> Result<Vec<ChannelTestInstance>, AppError> {
        if channels.is_empty() || test_channel_mappings.is_empty() {
            return Ok(Vec::new());
        }

        let test_channel_count = test_channel_mappings.len();
        let required_batches = self.calculate_required_batches(channels.len(), test_channel_count);
        let mut allocated_instances = Vec::new();

        log::info!(
            "分配 {:?} {} 通道: {} 个通道，{} 个测试PLC通道，需要 {} 个批次",
            module_type,
            if is_powered { "有源" } else { "无源" },
            channels.len(),
            test_channel_count,
            required_batches
        );

        // 按批次分配通道
        for batch_index in 0..required_batches {
            let current_batch_number = *batch_offset + batch_index + 1;
            let batch_id = format!("{}_batch_{}", base_batch_id, current_batch_number);
            let batch_name = format!("批次{}", current_batch_number);

            // 计算当前批次的通道范围
            let start_index = batch_index * test_channel_count;
            let end_index = std::cmp::min(start_index + test_channel_count, channels.len());
            let batch_channels = &channels[start_index..end_index];

            // 为当前批次的每个通道分配测试PLC通道
            for (channel_index, channel) in batch_channels.iter().enumerate() {
                let test_channel_mapping = &test_channel_mappings[channel_index];
                
                let instance = ChannelTestInstance {
                    instance_id: uuid::Uuid::new_v4().to_string(),
                    definition_id: channel.id.clone(),
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
                    test_plc_channel_tag: Some(test_channel_mapping.channel_address.clone()),
                    test_plc_communication_address: Some(test_channel_mapping.communication_address.clone()),
                    current_operator: None,
                    retries_count: 0,
                    transient_data: HashMap::new(),
                };
                
                allocated_instances.push(instance);
            }
        }

        // 更新批次偏移量
        *batch_offset += required_batches;

        Ok(allocated_instances)
    }

    /// 提取批次信息
    fn extract_batch_info(
        &self,
        instances: &[ChannelTestInstance],
        product_model: Option<String>,
        serial_number: Option<String>,
    ) -> Vec<TestBatchInfo> {
        let mut batch_map: HashMap<String, Vec<&ChannelTestInstance>> = HashMap::new();
        
        // 按批次ID分组
        for instance in instances {
            batch_map
                .entry(instance.test_batch_id.clone())
                .or_insert_with(Vec::new)
                .push(instance);
        }
        
        let mut batches = Vec::new();
        
        for (batch_id, batch_instances) in batch_map {
            let total_points = batch_instances.len() as u32;
            let tested_points = batch_instances
                .iter()
                .filter(|i| i.overall_status != OverallTestStatus::NotTested)
                .count() as u32;
            let passed_points = batch_instances
                .iter()
                .filter(|i| i.overall_status == OverallTestStatus::TestCompletedPassed)
                .count() as u32;
            let failed_points = batch_instances
                .iter()
                .filter(|i| i.overall_status == OverallTestStatus::TestCompletedFailed)
                .count() as u32;

            // 确定批次状态
            let overall_status = if tested_points == 0 {
                OverallTestStatus::NotTested
            } else if tested_points < total_points {
                OverallTestStatus::HardPointTesting
            } else if failed_points > 0 {
                OverallTestStatus::TestCompletedFailed
            } else {
                OverallTestStatus::TestCompletedPassed
            };

            // 获取批次名称（从第一个实例中提取）
            let batch_name = batch_instances
                .first()
                .map(|i| i.test_batch_name.clone())
                .unwrap_or_else(|| format!("批次{}", batch_id));

            let batch_info = TestBatchInfo {
                batch_id: batch_id.clone(),
                product_model: product_model.clone(),
                serial_number: serial_number.clone(),
                customer_name: None,
                creation_time: Utc::now(),
                last_updated_time: Utc::now(),
                operator_name: None,
                status_summary: Some(format!(
                    "{} - {}/{} 通过",
                    match overall_status {
                        OverallTestStatus::NotTested => "未开始",
                        OverallTestStatus::HardPointTesting => "进行中",
                        OverallTestStatus::TestCompletedPassed => "已完成",
                        OverallTestStatus::TestCompletedFailed => "已完成(有失败)",
                        _ => "未知状态",
                    },
                    passed_points,
                    total_points
                )),
                total_points,
                tested_points,
                passed_points,
                failed_points,
                skipped_points: 0,
                overall_status,
                batch_name,
                custom_data: HashMap::new(),
            };
            
            batches.push(batch_info);
        }
        
        // 按批次ID排序
        batches.sort_by(|a, b| a.batch_id.cmp(&b.batch_id));
        batches
    }

    /// 计算分配统计信息
    fn calculate_allocation_summary(
        &self,
        definitions: &[ChannelPointDefinition],
        instances: &[ChannelTestInstance],
        allocation_errors: Vec<String>,
    ) -> AllocationSummary {
        let total_definitions = definitions.len() as u32;
        let allocated_instances = instances.len() as u32;
        let skipped_definitions = total_definitions - allocated_instances;

        // 按模块类型统计
        let mut by_module_type = HashMap::new();
        
        // 统计定义数量
        let mut definition_counts: HashMap<ModuleType, u32> = HashMap::new();
        for definition in definitions {
            *definition_counts.entry(definition.module_type.clone()).or_insert(0) += 1;
        }

        // 统计分配的实例数量和批次数量
        let mut allocated_counts: HashMap<ModuleType, u32> = HashMap::new();
        let mut batch_counts: HashMap<ModuleType, std::collections::HashSet<String>> = HashMap::new();
        
        for instance in instances {
            // 通过definition_id找到对应的定义
            if let Some(definition) = definitions.iter().find(|d| d.id == instance.definition_id) {
                *allocated_counts.entry(definition.module_type.clone()).or_insert(0) += 1;
                batch_counts
                    .entry(definition.module_type.clone())
                    .or_insert_with(std::collections::HashSet::new)
                    .insert(instance.test_batch_id.clone());
            }
        }

        // 构建模块类型统计
        for module_type in [ModuleType::AI, ModuleType::AO, ModuleType::DI, ModuleType::DO] {
            let definition_count = definition_counts.get(&module_type).copied().unwrap_or(0);
            let allocated_count = allocated_counts.get(&module_type).copied().unwrap_or(0);
            let batch_count = batch_counts
                .get(&module_type)
                .map(|set| set.len() as u32)
                .unwrap_or(0);

            if definition_count > 0 || allocated_count > 0 {
                by_module_type.insert(
                    module_type,
                    ModuleTypeStats {
                        definition_count,
                        allocated_count,
                        batch_count,
                    },
                );
            }
        }

        AllocationSummary {
            total_definitions,
            allocated_instances,
            skipped_definitions,
            by_module_type,
            allocation_errors,
        }
    }
}

#[async_trait::async_trait]
impl IChannelAllocationService for ChannelAllocationService {
    async fn allocate_channels(
        &self,
        definitions: Vec<ChannelPointDefinition>,
        test_plc_config: TestPlcConfig,
        product_model: Option<String>,
        serial_number: Option<String>,
    ) -> Result<BatchAllocationResult, AppError> {
        log::info!("开始分配通道，总定义数: {}", definitions.len());

        // 按模块类型和有源/无源分组通道定义
        let grouped_definitions = self.group_definitions_by_type_and_power(&definitions);
        
        let mut all_instances = Vec::new();
        let mut allocation_errors = Vec::new();
        let mut batch_offset = 0usize;
        let base_batch_id = uuid::Uuid::new_v4().to_string();

        // 实现正确的有源/无源匹配逻辑
        
        // AI有源 → AO无源
        if let Some(ai_powered) = grouped_definitions.get(&(ModuleType::AI, true)) {
            let ao_unpowered_mappings = self.get_test_channel_mappings_by_power(&test_plc_config, &ModuleType::AO, false);
            if ao_unpowered_mappings.is_empty() {
                allocation_errors.push("缺少AO无源测试PLC通道配置".to_string());
            } else {
                let instances = self.allocate_channels_with_power_matching(
                    ai_powered,
                    &ao_unpowered_mappings,
                    &base_batch_id,
                    &ModuleType::AI,
                    true,
                    &mut batch_offset,
                )?;
                all_instances.extend(instances);
                log::info!("AI有源→AO无源分配完成，分配数量: {}", ai_powered.len());
            }
        }

        // AI无源 → AO有源
        if let Some(ai_unpowered) = grouped_definitions.get(&(ModuleType::AI, false)) {
            let ao_powered_mappings = self.get_test_channel_mappings_by_power(&test_plc_config, &ModuleType::AO, true);
            if ao_powered_mappings.is_empty() {
                allocation_errors.push("缺少AO有源测试PLC通道配置".to_string());
            } else {
                let instances = self.allocate_channels_with_power_matching(
                    ai_unpowered,
                    &ao_powered_mappings,
                    &base_batch_id,
                    &ModuleType::AI,
                    false,
                    &mut batch_offset,
                )?;
                all_instances.extend(instances);
                log::info!("AI无源→AO有源分配完成，分配数量: {}", ai_unpowered.len());
            }
        }

        // AO有源 → AI无源
        if let Some(ao_powered) = grouped_definitions.get(&(ModuleType::AO, true)) {
            let ai_unpowered_mappings = self.get_test_channel_mappings_by_power(&test_plc_config, &ModuleType::AI, false);
            if ai_unpowered_mappings.is_empty() {
                allocation_errors.push("缺少AI无源测试PLC通道配置".to_string());
            } else {
                let instances = self.allocate_channels_with_power_matching(
                    ao_powered,
                    &ai_unpowered_mappings,
                    &base_batch_id,
                    &ModuleType::AO,
                    true,
                    &mut batch_offset,
                )?;
                all_instances.extend(instances);
                log::info!("AO有源→AI无源分配完成，分配数量: {}", ao_powered.len());
            }
        }

        // AO无源 → AI有源
        if let Some(ao_unpowered) = grouped_definitions.get(&(ModuleType::AO, false)) {
            let ai_powered_mappings = self.get_test_channel_mappings_by_power(&test_plc_config, &ModuleType::AI, true);
            if ai_powered_mappings.is_empty() {
                allocation_errors.push("缺少AI有源测试PLC通道配置".to_string());
            } else {
                let instances = self.allocate_channels_with_power_matching(
                    ao_unpowered,
                    &ai_powered_mappings,
                    &base_batch_id,
                    &ModuleType::AO,
                    false,
                    &mut batch_offset,
                )?;
                all_instances.extend(instances);
                log::info!("AO无源→AI有源分配完成，分配数量: {}", ao_unpowered.len());
            }
        }

        // DI有源 → DO无源
        if let Some(di_powered) = grouped_definitions.get(&(ModuleType::DI, true)) {
            let do_unpowered_mappings = self.get_test_channel_mappings_by_power(&test_plc_config, &ModuleType::DO, false);
            if do_unpowered_mappings.is_empty() {
                allocation_errors.push("缺少DO无源测试PLC通道配置".to_string());
            } else {
                let instances = self.allocate_channels_with_power_matching(
                    di_powered,
                    &do_unpowered_mappings,
                    &base_batch_id,
                    &ModuleType::DI,
                    true,
                    &mut batch_offset,
                )?;
                all_instances.extend(instances);
                log::info!("DI有源→DO无源分配完成，分配数量: {}", di_powered.len());
            }
        }

        // DI无源 → DO有源
        if let Some(di_unpowered) = grouped_definitions.get(&(ModuleType::DI, false)) {
            let do_powered_mappings = self.get_test_channel_mappings_by_power(&test_plc_config, &ModuleType::DO, true);
            if do_powered_mappings.is_empty() {
                allocation_errors.push("缺少DO有源测试PLC通道配置".to_string());
            } else {
                let instances = self.allocate_channels_with_power_matching(
                    di_unpowered,
                    &do_powered_mappings,
                    &base_batch_id,
                    &ModuleType::DI,
                    false,
                    &mut batch_offset,
                )?;
                all_instances.extend(instances);
                log::info!("DI无源→DO有源分配完成，分配数量: {}", di_unpowered.len());
            }
        }

        // DO有源 → DI无源
        if let Some(do_powered) = grouped_definitions.get(&(ModuleType::DO, true)) {
            let di_unpowered_mappings = self.get_test_channel_mappings_by_power(&test_plc_config, &ModuleType::DI, false);
            if di_unpowered_mappings.is_empty() {
                allocation_errors.push("缺少DI无源测试PLC通道配置".to_string());
            } else {
                let instances = self.allocate_channels_with_power_matching(
                    do_powered,
                    &di_unpowered_mappings,
                    &base_batch_id,
                    &ModuleType::DO,
                    true,
                    &mut batch_offset,
                )?;
                all_instances.extend(instances);
                log::info!("DO有源→DI无源分配完成，分配数量: {}", do_powered.len());
            }
        }

        // DO无源 → DI有源
        if let Some(do_unpowered) = grouped_definitions.get(&(ModuleType::DO, false)) {
            let di_powered_mappings = self.get_test_channel_mappings_by_power(&test_plc_config, &ModuleType::DI, true);
            if di_powered_mappings.is_empty() {
                allocation_errors.push("缺少DI有源测试PLC通道配置".to_string());
            } else {
                let instances = self.allocate_channels_with_power_matching(
                    do_unpowered,
                    &di_powered_mappings,
                    &base_batch_id,
                    &ModuleType::DO,
                    false,
                    &mut batch_offset,
                )?;
                all_instances.extend(instances);
                log::info!("DO无源→DI有源分配完成，分配数量: {}", do_unpowered.len());
            }
        }

        // 提取批次信息
        let batches = self.extract_batch_info(&all_instances, product_model, serial_number);
        
        // 计算分配统计
        let allocation_summary = self.calculate_allocation_summary(&definitions, &all_instances, allocation_errors);

        log::info!(
            "通道分配完成，总批次数: {}, 总实例数: {}, 错误数: {}",
            batches.len(),
            all_instances.len(),
            allocation_summary.allocation_errors.len()
        );

        Ok(BatchAllocationResult {
            batches,
            allocated_instances: all_instances,
            allocation_summary,
        })
    }

    async fn get_batch_instances(
        &self,
        batch_id: &str,
    ) -> Result<Vec<ChannelTestInstance>, AppError> {
        // 这里应该从持久化存储中获取，暂时返回空
        // 在实际实现中，需要调用 persistence service
        log::info!("获取批次实例: {}", batch_id);
        Ok(Vec::new())
    }

    async fn clear_all_allocations(
        &self,
        mut instances: Vec<ChannelTestInstance>,
    ) -> Result<Vec<ChannelTestInstance>, AppError> {
        log::info!("清除所有通道分配，实例数: {}", instances.len());

        for instance in &mut instances {
            instance.test_batch_id = String::new();
            instance.test_batch_name = String::new();
            instance.test_plc_channel_tag = None;
            instance.test_plc_communication_address = None;
            instance.overall_status = OverallTestStatus::NotTested;
            instance.error_message = None;
            instance.last_updated_time = Utc::now();
        }

        Ok(instances)
    }

    async fn validate_allocations(
        &self,
        instances: &[ChannelTestInstance],
    ) -> Result<ValidationResult, AppError> {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        // 检查重复的测试PLC通道分配（同一批次内）
        let mut batch_channel_usage: HashMap<String, HashMap<String, Vec<String>>> = HashMap::new();
        
        for instance in instances {
            if let Some(ref channel_tag) = instance.test_plc_channel_tag {
                batch_channel_usage
                    .entry(instance.test_batch_id.clone())
                    .or_insert_with(HashMap::new)
                    .entry(channel_tag.clone())
                    .or_insert_with(Vec::new)
                    .push(instance.instance_id.clone());
            }
        }

        // 检查同一批次内的重复分配
        for (batch_id, channel_usage) in batch_channel_usage {
            for (channel_tag, instance_ids) in channel_usage {
                if instance_ids.len() > 1 {
                    errors.push(format!(
                        "批次 {} 中测试PLC通道 {} 被重复分配给多个实例: {:?}",
                        batch_id, channel_tag, instance_ids
                    ));
                }
            }
        }

        // 检查缺失的分配信息
        for instance in instances {
            if instance.test_plc_channel_tag.is_none() {
                warnings.push(format!(
                    "实例 {} 缺少测试PLC通道分配",
                    instance.instance_id
                ));
            }
        }

        Ok(ValidationResult {
            is_valid: errors.is_empty(),
            errors,
            warnings,
        })
    }
} 