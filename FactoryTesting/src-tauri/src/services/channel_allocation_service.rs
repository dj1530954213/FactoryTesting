use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::models::{
    ChannelPointDefinition, ChannelTestInstance, TestBatchInfo, ModuleType, OverallTestStatus, PointDataType
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
            // ===== 改进有源/无源判断逻辑 =====
            // 参考原始C#代码：!string.IsNullOrWhiteSpace(ch.PowerSupplyType) && ch.PowerSupplyType.Contains("有源")
            let is_powered = !definition.power_supply_type.trim().is_empty() 
                && definition.power_supply_type.contains("有源");
            
            log::debug!("通道 {} ({}): 供电类型='{}', 判断为{}", 
                definition.tag, 
                format!("{:?}", definition.module_type),
                definition.power_supply_type,
                if is_powered { "有源" } else { "无源" }
            );
            
            grouped
                .entry((definition.module_type.clone(), is_powered))
                .or_insert_with(Vec::new)
                .push(definition.clone());
        }
        
        // 详细的分组统计日志
        log::info!("=== 通道分组统计 ===");
        for ((module_type, is_powered), channels) in &grouped {
            let power_str = if *is_powered { "有源" } else { "无源" };
            log::info!("{:?}_{}: {} 个通道", module_type, power_str, channels.len());
            
            // 显示每个分组中前3个通道的详情
            for (i, channel) in channels.iter().take(3).enumerate() {
                log::debug!("  [{}/{}] {} - 供电: '{}', 通信地址: {}", 
                    i + 1, channels.len(), channel.tag, channel.power_supply_type, channel.plc_communication_address);
            }
            if channels.len() > 3 {
                log::debug!("  ... 还有 {} 个通道", channels.len() - 3);
            }
        }
        log::info!("==================");
        
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

        // ===== 修复关键问题：正确计算测试通道数量 =====
        // 参考原始C#代码：int batchCount = (int)Math.Ceiling((double)channels.Count / totalTestChannels);
        // 如果没有测试通道映射，使用默认的通道数量（比如8个），而不是通道总数
        let test_channel_count = if test_channel_mappings.is_empty() {
            // 当没有测试PLC配置时，使用合理的默认值，而不是通道总数
            // 这样可以确保生成多个批次
            match module_type {
                ModuleType::AI | ModuleType::AO => 8,  // AI/AO通道通常每批次8个
                ModuleType::DI | ModuleType::DO => 16, // DI/DO通道通常每批次16个
                _ => 8, // 默认8个
            }
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

            log::debug!("处理批次 {}: 通道索引 {} 到 {} (共{}个通道)", 
                current_batch_number, start_index, end_index, end_index - start_index);

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
                        // 如果没有映射，使用默认的通道地址
                        let default_tag = format!("{}{}", 
                            match module_type {
                                ModuleType::AI => if is_powered { "AO2" } else { "AO1" },
                                ModuleType::AO => if is_powered { "AI1" } else { "AI2" },
                                ModuleType::DI => if is_powered { "DO2" } else { "DO1" },
                                ModuleType::DO => if is_powered { "DI1" } else { "DI2" },
                                _ => "DEFAULT",
                            },
                            index_in_batch + 1
                        );
                        let default_addr = format!("{}.{}", 
                            &default_tag[..default_tag.len()-1], 
                            index_in_batch + 1
                        );
                        (default_tag, default_addr)
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

    /// 统一分配所有通道类型（参考原始C#代码的整体分配逻辑）
    /// 
    /// 这个方法会模拟原始C#代码中的整体分配流程：
    /// 1. 将所有通道按模块类型和有源/无源分组
    /// 2. 根据测试PLC的通道配置计算总的批次数
    /// 3. 为每个通道分配批次编号和测试PLC通道地址
    /// 
    /// 参考原始 AllocateChannelsTestAsync 方法的分配逻辑
    fn allocate_channels_unified(
        &self,
        definitions: Vec<ChannelPointDefinition>,
        test_plc_config: TestPlcConfig,
        product_model: Option<String>,
        serial_number: Option<String>,
    ) -> Result<BatchAllocationResult, AppError> {
        log::info!("===== 开始统一分配通道 =====");
        log::info!("总通道定义数: {}", definitions.len());
        
        // ===== 增加输入数据的详细日志 =====
        log::info!("=== 输入通道定义详情 ===");
        for (i, def) in definitions.iter().enumerate().take(10) {
            log::info!("通道[{}]: ID={}, Tag={}, Type={:?}, 供电={}", 
                i + 1, def.id, def.tag, def.module_type, def.power_supply_type);
        }
        if definitions.len() > 10 {
            log::info!("... 还有 {} 个通道", definitions.len() - 10);
        }
        
        // ===== 增加测试PLC配置的详细日志 =====
        log::info!("=== 测试PLC配置详情 ===");
        log::info!("PLC品牌: {}, IP: {}", test_plc_config.brand_type, test_plc_config.ip_address);
        log::info!("测试PLC通道映射表数量: {}", test_plc_config.comparison_tables.len());
        
        for (i, table) in test_plc_config.comparison_tables.iter().enumerate().take(10) {
            log::info!("映射[{}]: {} -> {} ({:?}, {})", 
                i + 1, table.channel_address, table.communication_address, 
                table.channel_type, if table.is_powered { "有源" } else { "无源" });
        }
        if test_plc_config.comparison_tables.len() > 10 {
            log::info!("... 还有 {} 个映射", test_plc_config.comparison_tables.len() - 10);
        }

        // 按模块类型和有源/无源分组通道定义
        let grouped_definitions = self.group_definitions_by_type_and_power(&definitions);
        
        // 模拟原始C#代码的分配逻辑：将各类型通道按特定顺序处理
        let mut all_channels_for_allocation = Vec::new();
        let mut allocation_sequence = Vec::new();
        
        // ===== 按照原始C#代码的顺序添加通道 =====
        
        // 1. AI有源通道 → 需要AO无源测试通道
        if let Some(ai_powered) = grouped_definitions.get(&(ModuleType::AI, true)) {
            allocation_sequence.push(("AI有源→AO无源", ai_powered.len()));
            all_channels_for_allocation.extend(ai_powered.clone());
            log::debug!("添加AI有源通道: {} 个", ai_powered.len());
        }
        
        // 2. AI无源通道 → 需要AO有源测试通道  
        if let Some(ai_unpowered) = grouped_definitions.get(&(ModuleType::AI, false)) {
            allocation_sequence.push(("AI无源→AO有源", ai_unpowered.len()));
            all_channels_for_allocation.extend(ai_unpowered.clone());
            log::debug!("添加AI无源通道: {} 个", ai_unpowered.len());
        }
        
        // 3. AO有源通道 → 需要AI无源测试通道
        if let Some(ao_powered) = grouped_definitions.get(&(ModuleType::AO, true)) {
            allocation_sequence.push(("AO有源→AI无源", ao_powered.len()));
            all_channels_for_allocation.extend(ao_powered.clone());
            log::debug!("添加AO有源通道: {} 个", ao_powered.len());
        }
        
        // 4. AO无源通道 → 需要AI有源测试通道
        if let Some(ao_unpowered) = grouped_definitions.get(&(ModuleType::AO, false)) {
            allocation_sequence.push(("AO无源→AI有源", ao_unpowered.len()));
            all_channels_for_allocation.extend(ao_unpowered.clone());
            log::debug!("添加AO无源通道: {} 个", ao_unpowered.len());
        }
        
        // 5. DI有源通道 → 需要DO无源测试通道
        if let Some(di_powered) = grouped_definitions.get(&(ModuleType::DI, true)) {
            allocation_sequence.push(("DI有源→DO无源", di_powered.len()));
            all_channels_for_allocation.extend(di_powered.clone());
            log::debug!("添加DI有源通道: {} 个", di_powered.len());
        }
        
        // 6. DI无源通道 → 需要DO有源测试通道
        if let Some(di_unpowered) = grouped_definitions.get(&(ModuleType::DI, false)) {
            allocation_sequence.push(("DI无源→DO有源", di_unpowered.len()));
            all_channels_for_allocation.extend(di_unpowered.clone());
            log::debug!("添加DI无源通道: {} 个", di_unpowered.len());
        }
        
        // 7. DO有源通道 → 需要DI无源测试通道
        if let Some(do_powered) = grouped_definitions.get(&(ModuleType::DO, true)) {
            allocation_sequence.push(("DO有源→DI无源", do_powered.len()));
            all_channels_for_allocation.extend(do_powered.clone());
            log::debug!("添加DO有源通道: {} 个", do_powered.len());
        }
        
        // 8. DO无源通道 → 需要DI有源测试通道
        if let Some(do_unpowered) = grouped_definitions.get(&(ModuleType::DO, false)) {
            allocation_sequence.push(("DO无源→DI有源", do_unpowered.len()));
            all_channels_for_allocation.extend(do_unpowered.clone());
            log::debug!("添加DO无源通道: {} 个", do_unpowered.len());
        }
        
        log::info!("=== 分配序列 ===");
        for (desc, count) in &allocation_sequence {
            log::info!("{}: {} 个通道", desc, count);
        }
        log::info!("总计: {} 个通道需要分配", all_channels_for_allocation.len());
        
        // ===== 统一分配逻辑：参考原始C# AllocateChannelsWithConfig方法 =====
        
        // 计算每个批次的最大通道数（使用所有测试PLC通道类型的最小值作为基准）
        let test_channel_counts = self.calculate_test_channel_counts(&test_plc_config);
        let channels_per_batch = test_channel_counts.min_channels_per_batch;
        
        log::info!("=== 测试PLC通道统计详情 ===");
        log::info!("AO无源(测试AI有源): {}", test_channel_counts.ao_unpowered);
        log::info!("AO有源(测试AI无源): {}", test_channel_counts.ao_powered);
        log::info!("AI无源(测试AO有源): {}", test_channel_counts.ai_unpowered);
        log::info!("AI有源(测试AO无源): {}", test_channel_counts.ai_powered);
        log::info!("DO无源(测试DI有源): {}", test_channel_counts.do_unpowered);
        log::info!("DO有源(测试DI无源): {}", test_channel_counts.do_powered);
        log::info!("DI无源(测试DO有源): {}", test_channel_counts.di_unpowered);
        log::info!("DI有源(测试DO无源): {}", test_channel_counts.di_powered);
        log::info!("每批次最大通道数: {}", channels_per_batch);
        
        // ===== 增加批次计算的详细日志 =====
        log::info!("=== 批次计算过程 ===");
        log::info!("需要分配的总通道数: {}", all_channels_for_allocation.len());
        log::info!("每批次最大通道数: {}", channels_per_batch);
        
        // 计算总批次数
        let total_batches = if channels_per_batch > 0 {
            let calculated = (all_channels_for_allocation.len() + channels_per_batch - 1) / channels_per_batch;
            log::info!("批次计算公式: ({} + {} - 1) / {} = {}", 
                all_channels_for_allocation.len(), channels_per_batch, channels_per_batch, calculated);
            calculated
        } else {
            log::warn!("每批次通道数为0，强制设置为1个批次");
            1 // 至少一个批次
        };
        
        log::info!("计算得出总批次数: {}", total_batches);
        
        // ===== 增加分配过程的详细日志 =====
        log::info!("=== 开始逐个通道分配 ===");
        
        // 为每个通道分配批次和测试PLC通道
        let mut allocated_instances = Vec::new();
        let base_batch_id = uuid::Uuid::new_v4().to_string();
        let mut batch_statistics: HashMap<usize, usize> = HashMap::new();
        
        for (channel_index, channel) in all_channels_for_allocation.iter().enumerate() {
            // 计算批次号（从1开始）
            let batch_number = if channels_per_batch > 0 {
                channel_index / channels_per_batch + 1
            } else {
                1
            };
            let index_in_batch = if channels_per_batch > 0 {
                channel_index % channels_per_batch
            } else {
                channel_index
            };
            
            // 统计每个批次的通道数
            *batch_statistics.entry(batch_number).or_insert(0) += 1;
            
            let batch_id = format!("{}_batch_{}", base_batch_id, batch_number);
            let batch_name = format!("批次{}", batch_number);
            
            // 获取对应的测试PLC通道信息
            let (test_plc_channel_tag, test_plc_communication_address) = 
                self.get_test_plc_channel_for_channel(channel, index_in_batch, &test_plc_config);
            
            // 创建测试实例
            let instance = ChannelTestInstance {
                instance_id: uuid::Uuid::new_v4().to_string(),
                definition_id: channel.id.clone(),
                test_batch_id: batch_id,
                test_batch_name: batch_name,
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
                test_plc_channel_tag: if test_plc_channel_tag.is_empty() { None } else { Some(test_plc_channel_tag.clone()) },
                test_plc_communication_address: if test_plc_communication_address.is_empty() { None } else { Some(test_plc_communication_address.clone()) },
                current_operator: None,
                retries_count: 0,
                transient_data: HashMap::new(),
            };
            
            allocated_instances.push(instance);
            
            // 详细记录前几个和关键的分配操作
            if channel_index < 20 || channel_index % 50 == 0 || channel_index == all_channels_for_allocation.len() - 1 {
                log::info!("分配通道[{}]: {} ({}:{}) → 批次{} (索引{}), 测试通道: {}", 
                    channel_index + 1, 
                    channel.tag, 
                    format!("{:?}", channel.module_type),
                    if channel.power_supply_type.contains("有源") { "有源" } else { "无源" },
                    batch_number, 
                    index_in_batch,
                    test_plc_channel_tag
                );
            }
        }
        
        // ===== 输出批次统计 =====
        log::info!("=== 批次分配统计 ===");
        for batch_num in 1..=total_batches {
            let count = batch_statistics.get(&batch_num).copied().unwrap_or(0);
            log::info!("批次{}: {} 个通道", batch_num, count);
        }
        
        // 提取批次信息
        let batches = self.extract_batch_info(&allocated_instances, product_model, serial_number);
        
        // ===== 输出最终批次信息 =====
        log::info!("=== 最终生成的批次列表 ===");
        for (i, batch) in batches.iter().enumerate() {
            log::info!("批次[{}]: ID={}, 名称={}, 通道数={}", 
                i + 1, batch.batch_id, batch.batch_name, batch.total_points);
        }
        
        // 计算分配统计
        let allocation_summary = self.calculate_allocation_summary(&definitions, &allocated_instances, Vec::new());

        log::info!("===== 统一分配完成 =====");
        log::info!("预期批次数: {}", total_batches);
        log::info!("实际生成批次数: {}", batches.len());
        log::info!("生成实例数: {}", allocated_instances.len());
        log::info!("=============================");

        Ok(BatchAllocationResult {
            batches,
            allocated_instances,
            allocation_summary,
        })
    }
    
    /// 计算测试PLC通道配置统计
    fn calculate_test_channel_counts(&self, config: &TestPlcConfig) -> TestChannelCounts {
        let mut counts = TestChannelCounts::default();
        
        log::info!("=== 开始计算测试PLC通道统计 ===");
        log::info!("测试PLC通道映射表总数: {}", config.comparison_tables.len());
        
        // 如果没有测试PLC配置，使用默认配置
        if config.comparison_tables.is_empty() {
            log::warn!("没有测试PLC通道映射配置，使用默认每批次通道数");
            counts.min_channels_per_batch = 8; // 默认每批次8个通道
            log::info!("使用默认每批次通道数: {}", counts.min_channels_per_batch);
            return counts;
        }
        
        for (i, table) in config.comparison_tables.iter().enumerate() {
            let before_counts = counts.clone();
            
            match (&table.channel_type, table.is_powered) {
                (ModuleType::AO, false) => {
                    counts.ao_unpowered += 1;  // 用于测试AI有源
                    log::debug!("映射[{}]: {} -> AO无源计数 {} -> {}", 
                        i + 1, table.channel_address, before_counts.ao_unpowered, counts.ao_unpowered);
                },
                (ModuleType::AO, true)  => {
                    counts.ao_powered += 1;    // 用于测试AI无源
                    log::debug!("映射[{}]: {} -> AO有源计数 {} -> {}", 
                        i + 1, table.channel_address, before_counts.ao_powered, counts.ao_powered);
                },
                (ModuleType::AI, false) => {
                    counts.ai_unpowered += 1;  // 用于测试AO有源
                    log::debug!("映射[{}]: {} -> AI无源计数 {} -> {}", 
                        i + 1, table.channel_address, before_counts.ai_unpowered, counts.ai_unpowered);
                },
                (ModuleType::AI, true)  => {
                    counts.ai_powered += 1;    // 用于测试AO无源
                    log::debug!("映射[{}]: {} -> AI有源计数 {} -> {}", 
                        i + 1, table.channel_address, before_counts.ai_powered, counts.ai_powered);
                },
                (ModuleType::DO, false) => {
                    counts.do_unpowered += 1;  // 用于测试DI有源
                    log::debug!("映射[{}]: {} -> DO无源计数 {} -> {}", 
                        i + 1, table.channel_address, before_counts.do_unpowered, counts.do_unpowered);
                },
                (ModuleType::DO, true)  => {
                    counts.do_powered += 1;    // 用于测试DI无源
                    log::debug!("映射[{}]: {} -> DO有源计数 {} -> {}", 
                        i + 1, table.channel_address, before_counts.do_powered, counts.do_powered);
                },
                (ModuleType::DI, false) => {
                    counts.di_unpowered += 1;  // 用于测试DO有源
                    log::debug!("映射[{}]: {} -> DI无源计数 {} -> {}", 
                        i + 1, table.channel_address, before_counts.di_unpowered, counts.di_unpowered);
                },
                (ModuleType::DI, true)  => {
                    counts.di_powered += 1;    // 用于测试DO无源
                    log::debug!("映射[{}]: {} -> DI有源计数 {} -> {}", 
                        i + 1, table.channel_address, before_counts.di_powered, counts.di_powered);
                },
                _ => {
                    log::warn!("映射[{}]: {} 未知通道类型 {:?}", 
                        i + 1, table.channel_address, table.channel_type);
                }
            }
        }
        
        log::info!("=== 测试PLC通道统计结果 ===");
        log::info!("AO无源(用于测试AI有源): {}", counts.ao_unpowered);
        log::info!("AO有源(用于测试AI无源): {}", counts.ao_powered);
        log::info!("AI无源(用于测试AO有源): {}", counts.ai_unpowered);
        log::info!("AI有源(用于测试AO无源): {}", counts.ai_powered);
        log::info!("DO无源(用于测试DI有源): {}", counts.do_unpowered);
        log::info!("DO有源(用于测试DI无源): {}", counts.do_powered);
        log::info!("DI无源(用于测试DO有源): {}", counts.di_unpowered);
        log::info!("DI有源(用于测试DO无源): {}", counts.di_powered);
        
        // 计算每批次最小通道数（取各类型通道数的最小值，但不小于8）
        log::info!("=== 计算每批次最小通道数 ===");
        
        // 检查是否有任何测试PLC通道配置
        let total_test_channels = counts.ao_unpowered + counts.ao_powered + 
                                 counts.ai_unpowered + counts.ai_powered +
                                 counts.do_unpowered + counts.do_powered +
                                 counts.di_unpowered + counts.di_powered;
        
        if total_test_channels == 0 {
            log::warn!("没有有效的测试PLC通道配置，使用默认每批次通道数");
            counts.min_channels_per_batch = 8; // 默认每批次8个通道
            log::info!("使用默认每批次通道数: {}", counts.min_channels_per_batch);
        } else {
            let min_analog = std::cmp::min(
                if counts.ao_unpowered > 0 { counts.ao_unpowered } else { 8 },
                if counts.ai_unpowered > 0 { counts.ai_unpowered } else { 8 }
            );
            log::info!("模拟通道最小值: min({}, {}) = {}", 
                if counts.ao_unpowered > 0 { counts.ao_unpowered } else { 8 },
                if counts.ai_unpowered > 0 { counts.ai_unpowered } else { 8 },
                min_analog);
                
            let min_digital = std::cmp::min(
                if counts.do_unpowered > 0 { counts.do_unpowered } else { 16 },
                if counts.di_unpowered > 0 { counts.di_unpowered } else { 16 }
            );
            log::info!("数字通道最小值: min({}, {}) = {}", 
                if counts.do_unpowered > 0 { counts.do_unpowered } else { 16 },
                if counts.di_unpowered > 0 { counts.di_unpowered } else { 16 },
                min_digital);
            
            counts.min_channels_per_batch = std::cmp::min(min_analog, min_digital).max(8);
            log::info!("最终每批次通道数: min({}, {}).max(8) = {}", 
                min_analog, min_digital, counts.min_channels_per_batch);
        }
        
        log::info!("================================");
        
        counts
    }
    
    /// 为指定通道获取对应的测试PLC通道信息
    fn get_test_plc_channel_for_channel(
        &self,
        channel: &ChannelPointDefinition,
        index_in_batch: usize,
        config: &TestPlcConfig,
    ) -> (String, String) {
        // 根据被测通道类型和有源/无源状态，找到对应的测试PLC通道
        let is_powered = !channel.power_supply_type.trim().is_empty() 
            && channel.power_supply_type.contains("有源");
        
        let target_mappings = match (&channel.module_type, is_powered) {
            // AI有源 → AO无源
            (ModuleType::AI, true) => self.get_test_channel_mappings_by_type_and_power(config, &ModuleType::AO, false),
            // AI无源 → AO有源
            (ModuleType::AI, false) => self.get_test_channel_mappings_by_type_and_power(config, &ModuleType::AO, true),
            // AO有源 → AI无源
            (ModuleType::AO, true) => self.get_test_channel_mappings_by_type_and_power(config, &ModuleType::AI, false),
            // AO无源 → AI有源
            (ModuleType::AO, false) => self.get_test_channel_mappings_by_type_and_power(config, &ModuleType::AI, true),
            // DI有源 → DO无源
            (ModuleType::DI, true) => self.get_test_channel_mappings_by_type_and_power(config, &ModuleType::DO, false),
            // DI无源 → DO有源
            (ModuleType::DI, false) => self.get_test_channel_mappings_by_type_and_power(config, &ModuleType::DO, true),
            // DO有源 → DI无源
            (ModuleType::DO, true) => self.get_test_channel_mappings_by_type_and_power(config, &ModuleType::DI, false),
            // DO无源 → DI有源
            (ModuleType::DO, false) => self.get_test_channel_mappings_by_type_and_power(config, &ModuleType::DI, true),
            _ => Vec::new(),
        };
        
        if !target_mappings.is_empty() && index_in_batch < target_mappings.len() {
            let mapping = &target_mappings[index_in_batch];
            (mapping.channel_address.clone(), mapping.communication_address.clone())
        } else {
            // 如果没有找到映射，生成默认地址
            let default_tag = format!("{}{}", 
                match channel.module_type {
                    ModuleType::AI => if is_powered { "AO2" } else { "AO1" },
                    ModuleType::AO => if is_powered { "AI1" } else { "AI2" },
                    ModuleType::DI => if is_powered { "DO2" } else { "DO1" },
                    ModuleType::DO => if is_powered { "DI1" } else { "DI2" },
                    _ => "DEFAULT",
                },
                index_in_batch + 1
            );
            let default_addr = format!("{}.{}", 
                &default_tag[..default_tag.len()-1], 
                index_in_batch + 1
            );
            (default_tag, default_addr)
        }
    }
}

/// 测试PLC通道统计结构
#[derive(Debug, Clone, Default)]
struct TestChannelCounts {
    ao_unpowered: usize,    // AO无源通道数（用于测试AI有源）
    ao_powered: usize,      // AO有源通道数（用于测试AI无源）
    ai_unpowered: usize,    // AI无源通道数（用于测试AO有源）
    ai_powered: usize,      // AI有源通道数（用于测试AO无源）
    do_unpowered: usize,    // DO无源通道数（用于测试DI有源）
    do_powered: usize,      // DO有源通道数（用于测试DI无源）
    di_unpowered: usize,    // DI无源通道数（用于测试DO有源）
    di_powered: usize,      // DI有源通道数（用于测试DO无源）
    min_channels_per_batch: usize, // 每批次最小通道数
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
        log::info!("[ChannelAllocation] ===== 开始通道分配 =====");
        log::info!("[ChannelAllocation] 输入: {} 个定义, 产品型号: {:?}, 序列号: {:?}", 
                  definitions.len(), product_model, serial_number);

        // 调用统一分配方法
        let result = self.allocate_channels_unified(definitions, test_plc_config, product_model, serial_number)?;
        
        log::info!("[ChannelAllocation] ===== 分配完成 =====");
        log::info!("[ChannelAllocation] 结果: {} 个批次, {} 个实例", 
                  result.batches.len(), result.allocated_instances.len());
        
        Ok(result)
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{ModuleType, PointDataType};

    /// 创建测试用的通道定义
    fn create_test_channel_definition(
        id: &str,
        tag: &str,
        module_type: ModuleType,
        power_supply_type: &str,
    ) -> ChannelPointDefinition {
        let mut definition = ChannelPointDefinition::new(
            id.to_string(),
            tag.to_string(),
            format!("测试通道 {}", tag),
            "Station1".to_string(),
            "Module1".to_string(),
            module_type.clone(),
            "CH01".to_string(),
            if matches!(module_type, ModuleType::AI | ModuleType::AO) {
                PointDataType::Float
            } else {
                PointDataType::Bool
            },
            format!("DB1.DBD{}", id.len() * 4),
        );
        
        definition.power_supply_type = power_supply_type.to_string();
        if matches!(module_type, ModuleType::AI | ModuleType::AO) {
            definition.range_lower_limit = Some(0.0);
            definition.range_upper_limit = Some(100.0);
            definition.test_rig_plc_address = Some(format!("DB2.DBD{}", id.len() * 4));
        }
        
        definition
    }

    /// 创建默认的测试PLC配置
    fn create_default_test_plc_config() -> TestPlcConfig {
        TestPlcConfig {
            brand_type: "Siemens".to_string(),
            ip_address: "192.168.1.100".to_string(),
            comparison_tables: vec![
                // 模拟一些测试PLC通道映射
                ComparisonTable {
                    channel_address: "TestRig.AI.CH01".to_string(),
                    communication_address: "DB1.DBD0".to_string(),
                    channel_type: ModuleType::AO,
                    is_powered: false,
                },
                ComparisonTable {
                    channel_address: "TestRig.AO.CH01".to_string(),
                    communication_address: "DB1.DBD4".to_string(),
                    channel_type: ModuleType::AO,
                    is_powered: true,
                },
                ComparisonTable {
                    channel_address: "TestRig.DI.CH01".to_string(),
                    communication_address: "DB1.DBD8".to_string(),
                    channel_type: ModuleType::DO,
                    is_powered: false,
                },
                ComparisonTable {
                    channel_address: "TestRig.DO.CH01".to_string(),
                    communication_address: "DB1.DBD12".to_string(),
                    channel_type: ModuleType::DO,
                    is_powered: true,
                },
            ],
        }
    }

    #[test]
    fn test_multiple_batch_allocation() {
        // 初始化日志
        let _ = env_logger::builder().is_test(true).try_init();
        
        println!("=== 开始测试多批次分配 ===");
        
        let service = ChannelAllocationService::new();
        
        // 创建20个通道定义（应该生成3个批次，每批次8个通道）
        let mut definitions = Vec::new();
        
        for i in 0..20 {
            let module_type = match i % 4 {
                0 => ModuleType::AI,
                1 => ModuleType::AO,
                2 => ModuleType::DI,
                _ => ModuleType::DO,
            };
            
            let power_type = if i % 8 < 4 { "有源" } else { "无源" };
            
            let definition = create_test_channel_definition(
                &format!("CH_{:03}", i + 1),
                &format!("Channel_{:03}", i + 1),
                module_type,
                power_type,
            );
            
            definitions.push(definition);
        }
        
        println!("创建了 {} 个通道定义", definitions.len());
        
        let test_plc_config = create_default_test_plc_config();
        
        // 执行分配
        let result = service.allocate_channels_unified(
            definitions,
            test_plc_config,
            Some("TestProduct".to_string()),
            Some("SN001".to_string()),
        );
        
        assert!(result.is_ok(), "批次分配应该成功");
        
        let allocation_result = result.unwrap();
        
        // 验证批次数量
        println!("生成的批次数量: {}", allocation_result.batches.len());
        assert!(
            allocation_result.batches.len() >= 2,
            "应该生成至少2个批次，实际生成: {}",
            allocation_result.batches.len()
        );
        
        // 验证实例总数
        println!("生成的实例数量: {}", allocation_result.allocated_instances.len());
        assert_eq!(
            allocation_result.allocated_instances.len(),
            20,
            "应该生成20个测试实例"
        );
        
        // 验证每个批次的实例数量
        for (i, batch) in allocation_result.batches.iter().enumerate() {
            let batch_instances: Vec<_> = allocation_result
                .allocated_instances
                .iter()
                .filter(|instance| instance.test_batch_id == batch.batch_id)
                .collect();
            
            println!("批次 {} ({}) 包含 {} 个实例", 
                i + 1, batch.batch_id, batch_instances.len());
            
            // 显示批次中的前3个实例详情
            for (j, instance) in batch_instances.iter().take(3).enumerate() {
                println!("  实例[{}]: {} - 测试PLC通道: {:?}", 
                    j + 1, instance.instance_id, instance.test_plc_channel_tag);
            }
            if batch_instances.len() > 3 {
                println!("  ... 还有 {} 个实例", batch_instances.len() - 3);
            }
            
            // 最后一个批次可能少于8个实例
            if i < allocation_result.batches.len() - 1 {
                assert!(
                    batch_instances.len() <= 8,
                    "批次 {} 实例数量不应超过8个，实际: {}",
                    i + 1,
                    batch_instances.len()
                );
            }
        }
        
        // 验证分配统计
        let summary = &allocation_result.allocation_summary;
        assert_eq!(summary.total_definitions, 20);
        assert_eq!(summary.allocated_instances, 20);
        assert_eq!(summary.skipped_definitions, 0);
        
        println!("分配统计:");
        println!("  总定义数: {}", summary.total_definitions);
        println!("  已分配实例数: {}", summary.allocated_instances);
        println!("  跳过的定义数: {}", summary.skipped_definitions);
        
        println!("=== 多批次分配测试通过 ===");
    }

    #[test]
    fn test_single_batch_allocation() {
        log::info!("=== 开始测试单批次分配 ===");
        
        let service = ChannelAllocationService::new();
        
        // 创建5个通道定义（应该生成1个批次）
        let mut definitions = Vec::new();
        
        for i in 0..5 {
            let definition = create_test_channel_definition(
                &format!("CH_{:03}", i + 1),
                &format!("Channel_{:03}", i + 1),
                ModuleType::AI,
                "有源",
            );
            definitions.push(definition);
        }
        
        let test_plc_config = create_default_test_plc_config();
        
        // 执行分配
        let result = service.allocate_channels_unified(
            definitions,
            test_plc_config,
            Some("TestProduct".to_string()),
            Some("SN001".to_string()),
        );
        
        assert!(result.is_ok(), "批次分配应该成功");
        
        let allocation_result = result.unwrap();
        
        // 验证只生成1个批次
        log::info!("生成的批次数量: {}", allocation_result.batches.len());
        assert_eq!(
            allocation_result.batches.len(),
            1,
            "应该生成1个批次，实际生成: {}",
            allocation_result.batches.len()
        );
        
        // 验证实例总数
        assert_eq!(allocation_result.allocated_instances.len(), 5);
        
        log::info!("=== 单批次分配测试通过 ===");
    }

    #[test]
    fn test_empty_definitions() {
        log::info!("=== 开始测试空定义列表 ===");
        
        let service = ChannelAllocationService::new();
        let test_plc_config = create_default_test_plc_config();
        
        // 执行分配
        let result = service.allocate_channels_unified(
            vec![], // 空的定义列表
            test_plc_config,
            Some("TestProduct".to_string()),
            Some("SN001".to_string()),
        );
        
        assert!(result.is_ok(), "空定义列表的分配应该成功");
        
        let allocation_result = result.unwrap();
        
        // 验证结果
        assert_eq!(allocation_result.batches.len(), 0, "空定义列表应该生成0个批次");
        assert_eq!(allocation_result.allocated_instances.len(), 0, "空定义列表应该生成0个实例");
        
        log::info!("=== 空定义列表测试通过 ===");
    }
} 