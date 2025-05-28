use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::models::{
    ChannelPointDefinition, ChannelTestInstance, TestBatchInfo, ModuleType, OverallTestStatus
};
use crate::error::AppError;
use chrono::Utc;

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
/// 
/// 根据FAT-CSM-001规则，此服务负责创建ChannelTestInstance的初始状态，
/// 但不直接修改状态，状态管理由ChannelStateManager负责
pub struct ChannelAllocationService;

impl ChannelAllocationService {
    pub fn new() -> Self {
        Self
    }

    /// 按模块类型和有源/无源分组通道定义
    /// 
    /// 根据原始C#代码的逻辑，正确识别有源/无源类型
    fn group_definitions_by_type_and_power(
        &self,
        definitions: &[ChannelPointDefinition],
    ) -> HashMap<(ModuleType, bool), Vec<ChannelPointDefinition>> {
        let mut grouped = HashMap::new();
        
        for definition in definitions {
            // 根据power_supply_type字段判断有源/无源
            // 参考原始C#代码中的IsPowered逻辑
            let is_powered = definition.power_supply_type.contains("有源");
            
            grouped
                .entry((definition.module_type.clone(), is_powered))
                .or_insert_with(Vec::new)
                .push(definition.clone());
        }
        
        log::info!("通道分组完成: {:?}", 
            grouped.iter().map(|((module_type, is_powered), channels)| 
                format!("{:?}_{}: {}", module_type, if *is_powered { "有源" } else { "无源" }, channels.len())
            ).collect::<Vec<_>>()
        );
        
        grouped
    }

    /// 获取测试PLC通道映射（按类型和有源/无源筛选）
    fn get_test_channel_mappings_by_type_and_power(
        &self,
        config: &TestPlcConfig,
        channel_type: &ModuleType,
        is_powered: bool,
    ) -> Vec<ComparisonTable> {
        let mappings = config
            .comparison_tables
            .iter()
            .filter(|table| &table.channel_type == channel_type && table.is_powered == is_powered)
            .cloned()
            .collect::<Vec<_>>();
            
        log::debug!("获取测试PLC通道映射: {:?}_{} = {} 个通道", 
            channel_type, if is_powered { "有源" } else { "无源" }, mappings.len());
            
        mappings
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
    /// 
    /// 参考原始C#代码中的AllocateChannelsWithConfigAndApplyState方法
    /// 使用统一的全局批次计数器，确保批次编号连续
    fn allocate_channels_with_unified_batching(
        &self,
        channels: &[ChannelPointDefinition],
        test_channel_mappings: &[ComparisonTable],
        base_batch_id: &str,
        module_type: &ModuleType,
        is_powered: bool,
        global_batch_counter: &mut usize,
    ) -> Result<Vec<ChannelTestInstance>, AppError> {
        if channels.is_empty() {
            return Ok(Vec::new());
        }

        // 如果没有测试通道映射，使用通道数量作为默认值，确保至少有一个批次
        let test_channel_count = if test_channel_mappings.is_empty() {
            channels.len()
        } else {
            test_channel_mappings.len()
        };
        
        let required_batches = self.calculate_required_batches(channels.len(), test_channel_count);
        let mut allocated_instances = Vec::new();

        log::info!(
            "开始分配 {:?} {} 通道: {} 个通道，{} 个测试PLC通道，需要 {} 个批次，起始批次号: {}",
            module_type,
            if is_powered { "有源" } else { "无源" },
            channels.len(),
            test_channel_count,
            required_batches,
            *global_batch_counter + 1
        );

        // 按批次分配通道
        for batch_index in 0..required_batches {
            let current_batch_number = *global_batch_counter + batch_index + 1;
            let batch_id = format!("{}_batch_{}", base_batch_id, current_batch_number);
            let batch_name = format!("批次{}", current_batch_number);

            // 计算当前批次的通道范围
            let start_index = batch_index * test_channel_count;
            let end_index = std::cmp::min(start_index + test_channel_count, channels.len());

            log::debug!("处理批次 {}: 通道索引 {} 到 {}", current_batch_number, start_index, end_index);

            // 为当前批次的通道创建测试实例
            for (channel_index, channel) in channels[start_index..end_index].iter().enumerate() {
                let instance_id = uuid::Uuid::new_v4().to_string();
                let index_in_batch = channel_index;
                
                // 获取对应的测试通道映射（如果存在）
                let (test_plc_channel_tag, test_plc_communication_address) = 
                    if !test_channel_mappings.is_empty() && index_in_batch < test_channel_mappings.len() {
                        let mapping = &test_channel_mappings[index_in_batch];
                        (mapping.channel_address.clone(), mapping.communication_address.clone())
                    } else {
                        // 如果没有映射，使用空值，但仍然分配批次
                        (String::new(), String::new())
                    };

                // 创建测试实例，符合FAT-CSM-001规则，初始状态为NotTested
                let instance = ChannelTestInstance {
                    instance_id,
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
                    // 添加批次和测试PLC信息
                    test_plc_channel_tag: if test_plc_channel_tag.is_empty() { None } else { Some(test_plc_channel_tag) },
                    test_plc_communication_address: if test_plc_communication_address.is_empty() { None } else { Some(test_plc_communication_address) },
                    current_operator: None,
                    retries_count: 0,
                    transient_data: HashMap::new(),
                };

                allocated_instances.push(instance);
            }
        }

        // 更新全局批次计数器
        *global_batch_counter += required_batches;

        log::info!(
            "完成分配 {:?} {} 通道: 创建了 {} 个实例，使用了 {} 个批次",
            module_type,
            if is_powered { "有源" } else { "无源" },
            allocated_instances.len(),
            required_batches
        );

        Ok(allocated_instances)
    }

    /// 提取批次信息
    /// 
    /// 根据分配的实例生成批次信息列表
    fn extract_batch_info(
        &self,
        instances: &[ChannelTestInstance],
        product_model: Option<String>,
        serial_number: Option<String>,
    ) -> Vec<TestBatchInfo> {
        let mut batch_map: HashMap<String, Vec<&ChannelTestInstance>> = HashMap::new();
        
        // 按批次ID分组实例
        for instance in instances {
            batch_map
                .entry(instance.test_batch_id.clone())
                .or_insert_with(Vec::new)
                .push(instance);
        }

        let mut batches = Vec::new();
        for (batch_id, batch_instances) in batch_map {
            // 从批次ID中提取批次名称
            let batch_name = batch_instances
                .first()
                .map(|instance| instance.test_batch_name.clone())
                .unwrap_or_else(|| format!("批次{}", batches.len() + 1));

            let batch_info = TestBatchInfo {
                batch_id,
                product_model: product_model.clone(),
                serial_number: serial_number.clone(),
                customer_name: None,
                creation_time: Utc::now(),
                last_updated_time: Utc::now(),
                operator_name: None,
                status_summary: Some("已创建，等待测试".to_string()),
                total_points: batch_instances.len() as u32,
                tested_points: 0,
                passed_points: 0,
                failed_points: 0,
                skipped_points: 0,
                overall_status: OverallTestStatus::NotTested,
                batch_name,
                custom_data: HashMap::new(),
            };
            
            batches.push(batch_info);
        }

        // 按批次名称排序
        batches.sort_by(|a, b| {
            a.batch_name.cmp(&b.batch_name)
        });

        log::info!("提取批次信息完成: 生成了 {} 个批次", batches.len());
        
        batches
    }

    /// 计算分配统计
    fn calculate_allocation_summary(
        &self,
        definitions: &[ChannelPointDefinition],
        instances: &[ChannelTestInstance],
        allocation_errors: Vec<String>,
    ) -> AllocationSummary {
        let mut by_module_type = HashMap::new();
        
        // 统计定义数量
        let mut definition_counts: HashMap<ModuleType, u32> = HashMap::new();
        for definition in definitions {
            *definition_counts.entry(definition.module_type.clone()).or_insert(0) += 1;
        }
        
        // 统计分配的实例数量和批次数量
        let mut instance_counts: HashMap<ModuleType, u32> = HashMap::new();
        let mut batch_counts: HashMap<ModuleType, std::collections::HashSet<String>> = HashMap::new();
        
        for instance in instances {
            // 需要通过definition_id找到对应的模块类型
            if let Some(definition) = definitions.iter().find(|d| d.id == instance.definition_id) {
                *instance_counts.entry(definition.module_type.clone()).or_insert(0) += 1;
                batch_counts
                    .entry(definition.module_type.clone())
                    .or_insert_with(std::collections::HashSet::new)
                    .insert(instance.test_batch_id.clone());
            }
        }
        
        // 构建模块类型统计
        for module_type in [ModuleType::AI, ModuleType::AO, ModuleType::DI, ModuleType::DO] {
            let definition_count = definition_counts.get(&module_type).copied().unwrap_or(0);
            let allocated_count = instance_counts.get(&module_type).copied().unwrap_or(0);
            let batch_count = batch_counts.get(&module_type).map(|set| set.len()).unwrap_or(0) as u32;
            
            if definition_count > 0 || allocated_count > 0 {
                by_module_type.insert(module_type, ModuleTypeStats {
                    definition_count,
                    allocated_count,
                    batch_count,
                });
            }
        }

        AllocationSummary {
            total_definitions: definitions.len() as u32,
            allocated_instances: instances.len() as u32,
            skipped_definitions: definitions.len() as u32 - instances.len() as u32,
            by_module_type,
            allocation_errors,
        }
    }
}

#[async_trait::async_trait]
impl IChannelAllocationService for ChannelAllocationService {
    /// 为通道定义分配测试批次和测试PLC通道
    /// 
    /// 实现正确的有源/无源匹配逻辑，参考原始C#代码：
    /// - AI有源 → AO无源
    /// - AI无源 → AO有源  
    /// - AO有源 → AI无源
    /// - AO无源 → AI有源
    /// - DI有源 → DO无源
    /// - DI无源 → DO有源
    /// - DO有源 → DI无源
    /// - DO无源 → DI有源
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
        let allocation_errors = Vec::new();
        let mut global_batch_counter = 0usize; // 统一的批次计数器
        let base_batch_id = uuid::Uuid::new_v4().to_string();

        // 实现正确的有源/无源匹配逻辑，参考原始C#代码
        
        // 1. AI有源 → AO无源
        if let Some(ai_powered) = grouped_definitions.get(&(ModuleType::AI, true)) {
            let ao_unpowered_mappings = self.get_test_channel_mappings_by_type_and_power(&test_plc_config, &ModuleType::AO, false);
            if ao_unpowered_mappings.is_empty() {
                log::warn!("缺少AO无源测试PLC通道配置，将使用默认分配");
            }
            let instances = self.allocate_channels_with_unified_batching(
                ai_powered,
                &ao_unpowered_mappings,
                &base_batch_id,
                &ModuleType::AI,
                true,
                &mut global_batch_counter,
            )?;
            all_instances.extend(instances);
            log::info!("AI有源→AO无源分配完成，分配数量: {}", ai_powered.len());
        }

        // 2. AI无源 → AO有源
        if let Some(ai_unpowered) = grouped_definitions.get(&(ModuleType::AI, false)) {
            let ao_powered_mappings = self.get_test_channel_mappings_by_type_and_power(&test_plc_config, &ModuleType::AO, true);
            if ao_powered_mappings.is_empty() {
                log::warn!("缺少AO有源测试PLC通道配置，将使用默认分配");
            }
            let instances = self.allocate_channels_with_unified_batching(
                ai_unpowered,
                &ao_powered_mappings,
                &base_batch_id,
                &ModuleType::AI,
                false,
                &mut global_batch_counter,
            )?;
            all_instances.extend(instances);
            log::info!("AI无源→AO有源分配完成，分配数量: {}", ai_unpowered.len());
        }

        // 3. AO有源 → AI无源
        if let Some(ao_powered) = grouped_definitions.get(&(ModuleType::AO, true)) {
            let ai_unpowered_mappings = self.get_test_channel_mappings_by_type_and_power(&test_plc_config, &ModuleType::AI, false);
            if ai_unpowered_mappings.is_empty() {
                log::warn!("缺少AI无源测试PLC通道配置，将使用默认分配");
            }
            let instances = self.allocate_channels_with_unified_batching(
                ao_powered,
                &ai_unpowered_mappings,
                &base_batch_id,
                &ModuleType::AO,
                true,
                &mut global_batch_counter,
            )?;
            all_instances.extend(instances);
            log::info!("AO有源→AI无源分配完成，分配数量: {}", ao_powered.len());
        }

        // 4. AO无源 → AI有源
        if let Some(ao_unpowered) = grouped_definitions.get(&(ModuleType::AO, false)) {
            let ai_powered_mappings = self.get_test_channel_mappings_by_type_and_power(&test_plc_config, &ModuleType::AI, true);
            if ai_powered_mappings.is_empty() {
                log::warn!("缺少AI有源测试PLC通道配置，将使用默认分配");
            }
            let instances = self.allocate_channels_with_unified_batching(
                ao_unpowered,
                &ai_powered_mappings,
                &base_batch_id,
                &ModuleType::AO,
                false,
                &mut global_batch_counter,
            )?;
            all_instances.extend(instances);
            log::info!("AO无源→AI有源分配完成，分配数量: {}", ao_unpowered.len());
        }

        // 5. DI有源 → DO无源
        if let Some(di_powered) = grouped_definitions.get(&(ModuleType::DI, true)) {
            let do_unpowered_mappings = self.get_test_channel_mappings_by_type_and_power(&test_plc_config, &ModuleType::DO, false);
            if do_unpowered_mappings.is_empty() {
                log::warn!("缺少DO无源测试PLC通道配置，将使用默认分配");
            }
            let instances = self.allocate_channels_with_unified_batching(
                di_powered,
                &do_unpowered_mappings,
                &base_batch_id,
                &ModuleType::DI,
                true,
                &mut global_batch_counter,
            )?;
            all_instances.extend(instances);
            log::info!("DI有源→DO无源分配完成，分配数量: {}", di_powered.len());
        }

        // 6. DI无源 → DO有源
        if let Some(di_unpowered) = grouped_definitions.get(&(ModuleType::DI, false)) {
            let do_powered_mappings = self.get_test_channel_mappings_by_type_and_power(&test_plc_config, &ModuleType::DO, true);
            if do_powered_mappings.is_empty() {
                log::warn!("缺少DO有源测试PLC通道配置，将使用默认分配");
            }
            let instances = self.allocate_channels_with_unified_batching(
                di_unpowered,
                &do_powered_mappings,
                &base_batch_id,
                &ModuleType::DI,
                false,
                &mut global_batch_counter,
            )?;
            all_instances.extend(instances);
            log::info!("DI无源→DO有源分配完成，分配数量: {}", di_unpowered.len());
        }

        // 7. DO有源 → DI无源
        if let Some(do_powered) = grouped_definitions.get(&(ModuleType::DO, true)) {
            let di_unpowered_mappings = self.get_test_channel_mappings_by_type_and_power(&test_plc_config, &ModuleType::DI, false);
            if di_unpowered_mappings.is_empty() {
                log::warn!("缺少DI无源测试PLC通道配置，将使用默认分配");
            }
            let instances = self.allocate_channels_with_unified_batching(
                do_powered,
                &di_unpowered_mappings,
                &base_batch_id,
                &ModuleType::DO,
                true,
                &mut global_batch_counter,
            )?;
            all_instances.extend(instances);
            log::info!("DO有源→DI无源分配完成，分配数量: {}", do_powered.len());
        }

        // 8. DO无源 → DI有源
        if let Some(do_unpowered) = grouped_definitions.get(&(ModuleType::DO, false)) {
            let di_powered_mappings = self.get_test_channel_mappings_by_type_and_power(&test_plc_config, &ModuleType::DI, true);
            if di_powered_mappings.is_empty() {
                log::warn!("缺少DI有源测试PLC通道配置，将使用默认分配");
            }
            let instances = self.allocate_channels_with_unified_batching(
                do_unpowered,
                &di_powered_mappings,
                &base_batch_id,
                &ModuleType::DO,
                false,
                &mut global_batch_counter,
            )?;
            all_instances.extend(instances);
            log::info!("DO无源→DI有源分配完成，分配数量: {}", do_unpowered.len());
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
        
        // 清除分配信息，符合FAT-CSM-001规则，重置状态为NotTested
        for instance in &mut instances {
            instance.test_batch_id = String::new();
            instance.test_batch_name = String::new();
            instance.test_plc_channel_tag = None;
            instance.test_plc_communication_address = None;
            instance.overall_status = OverallTestStatus::NotTested;
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
        
        // 验证批次分配的一致性
        let mut batch_instance_counts: HashMap<String, usize> = HashMap::new();
        
        for instance in instances {
            if instance.test_batch_id.is_empty() {
                errors.push(format!("实例 {} 缺少批次分配", instance.instance_id));
            } else {
                *batch_instance_counts.entry(instance.test_batch_id.clone()).or_insert(0) += 1;
            }
            
            if instance.test_plc_channel_tag.is_none() {
                warnings.push(format!("实例 {} 缺少测试PLC通道标签", instance.instance_id));
            }
        }
        
        // 检查批次大小的合理性
        for (batch_id, count) in batch_instance_counts {
            if count == 0 {
                errors.push(format!("批次 {} 没有分配任何实例", batch_id));
            } else if count > 100 {
                warnings.push(format!("批次 {} 的实例数量过多: {}", batch_id, count));
            }
        }
        
        let is_valid = errors.is_empty();
        
        log::info!("分配验证完成: 有效={}, 错误数={}, 警告数={}", is_valid, errors.len(), warnings.len());
        
        Ok(ValidationResult {
            is_valid,
            errors,
            warnings,
        })
    }
} 