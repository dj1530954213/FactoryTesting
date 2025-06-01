use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::models::{
    ChannelPointDefinition, ChannelTestInstance, TestBatchInfo, ModuleType, OverallTestStatus
};
use crate::models::test_plc_config::TestPlcChannelConfig;
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
    /// 分配错误列表
    pub errors: Vec<String>,
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

    /// 执行统一的通道分配
    ///
    /// 正确的分配逻辑：
    /// 1. 根据测试PLC的实际通道容量来分配
    /// 2. 每批次要填满测试PLC的所有可用通道
    /// 3. 只有当测试PLC通道都满了，或者没有更多对应类型的被测通道时，才开始下一批次
    fn allocate_channels_unified(
        &self,
        definitions: Vec<ChannelPointDefinition>,
        test_plc_config: TestPlcConfig,
        product_model: Option<String>,
        serial_number: Option<String>,
    ) -> Result<BatchAllocationResult, AppError> {
        log::info!("===== 开始统一分配通道 =====");
        log::info!("总通道定义数: {}", definitions.len());

        // 输出详细的输入信息
        log::info!("=== 输入通道定义详情 ===");
        for (i, def) in definitions.iter().enumerate().take(10) {
            log::info!("通道[{}]: ID={}, Tag={}, Type={:?}, 供电={}",
                i + 1, def.id, def.tag, def.module_type, def.power_supply_type);
        }
        if definitions.len() > 10 {
            log::info!("... 还有 {} 个通道", definitions.len() - 10);
        }

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

        // 步骤1: 按照有源/无源分组被测通道
        let channel_groups = self.group_channels_by_power_type(&definitions);

        // 步骤2: 计算测试PLC的实际通道容量
        let test_channel_counts = self.calculate_test_channel_counts(&test_plc_config);
        log::info!("测试PLC通道容量统计:");
        log::info!("  AO无源(测试AI有源): {}", test_channel_counts.ao_powered_false_count);
        log::info!("  AO有源(测试AI无源): {}", test_channel_counts.ao_powered_true_count);
        log::info!("  AI有源(测试AO无源): {}", test_channel_counts.ai_powered_true_count);
        log::info!("  DO无源(测试DI有源): {}", test_channel_counts.do_powered_false_count);
        log::info!("  DO有源(测试DI无源): {}", test_channel_counts.do_powered_true_count);
        log::info!("  DI无源(测试DO有源): {}", test_channel_counts.di_powered_false_count);
        log::info!("  DI有源(测试DO无源): {}", test_channel_counts.di_powered_true_count);

        // 步骤3: 创建分配序列（按优先级）
        let mut allocation_sequence = Vec::new();

        // AI有源 → AO无源
        allocation_sequence.extend(channel_groups.ai_powered_true.clone());
        // AI无源 → AO有源
        allocation_sequence.extend(channel_groups.ai_powered_false.clone());
        // AO有源 → AI无源
        allocation_sequence.extend(channel_groups.ao_powered_true.clone());
        // AO无源 → AI有源
        allocation_sequence.extend(channel_groups.ao_powered_false.clone());
        // DI有源 → DO无源
        allocation_sequence.extend(channel_groups.di_powered_true.clone());
        // DI无源 → DO有源
        allocation_sequence.extend(channel_groups.di_powered_false.clone());
        // DO有源 → DI无源
        allocation_sequence.extend(channel_groups.do_powered_true.clone());
        // DO无源 → DI有源
        allocation_sequence.extend(channel_groups.do_powered_false.clone());

        log::info!("=== 分配序列统计 ===");
        log::info!("AI有源→AO无源: {} 个通道", channel_groups.ai_powered_true.len());
        log::info!("AI无源→AO有源: {} 个通道", channel_groups.ai_powered_false.len());
        log::info!("AO有源→AI无源: {} 个通道", channel_groups.ao_powered_true.len());
        log::info!("AO无源→AI有源: {} 个通道", channel_groups.ao_powered_false.len());
        log::info!("DI有源→DO无源: {} 个通道", channel_groups.di_powered_true.len());
        log::info!("DI无源→DO有源: {} 个通道", channel_groups.di_powered_false.len());
        log::info!("DO有源→DI无源: {} 个通道", channel_groups.do_powered_true.len());
        log::info!("DO无源→DI有源: {} 个通道", channel_groups.do_powered_false.len());
        log::info!("总计: {} 个通道需要分配", allocation_sequence.len());

        // 步骤4: 执行正确的批次分配（每个批次重新使用完整的测试PLC通道池）
        let mut batches = Vec::new();
        let mut all_instances = Vec::new();
        let mut remaining_channels = allocation_sequence;
        let mut batch_counter = 1;

        while !remaining_channels.is_empty() {
            log::info!("=== 开始分配批次{} ===", batch_counter);
            log::info!("剩余待分配通道数: {}", remaining_channels.len());

            // 每个批次重新创建完整的测试PLC通道池（支持通道复用）
            let mut fresh_test_channel_pools = self.create_test_channel_pools(&test_plc_config);

            // 为当前批次分配通道
            let (batch_instances, used_channels) = self.allocate_single_batch_with_capacity_limit(
                &remaining_channels,
                &mut fresh_test_channel_pools,
                batch_counter,
                &test_plc_config,
                product_model.clone(),
                serial_number.clone(),
            )?;

            if batch_instances.is_empty() {
                log::error!("批次{}分配失败：无法分配任何通道", batch_counter);
                break;
            }

            log::info!("批次{}分配完成，分配了{}个通道", batch_counter, batch_instances.len());

            // 创建批次信息
            let batch_info = self.create_batch_info(
                batch_counter,
                &batch_instances,
                product_model.clone(),
                serial_number.clone(),
            );

            batches.push(batch_info);
            all_instances.extend(batch_instances);

            // 移除已分配的通道
            remaining_channels = remaining_channels.into_iter()
                .filter(|def| !used_channels.contains(&def.id))
                .collect();

            batch_counter += 1;
        }

        log::info!("===== 统一分配完成 =====");
        log::info!("总批次数: {}", batches.len());
        log::info!("总实例数: {}", all_instances.len());
        log::info!("=============================");

        // 克隆all_instances用于计算分配摘要
        let instances_for_summary = all_instances.clone();

        Ok(BatchAllocationResult {
            batches,
            allocated_instances: all_instances,
            errors: Vec::new(),
            allocation_summary: self.calculate_allocation_summary(&definitions, &instances_for_summary, Vec::new()),
        })
    }

    /// 为单个批次分配通道
    ///
    /// 这是核心的批次分配逻辑：限制每批次最多88个通道
    fn allocate_single_batch(
        &self,
        remaining_channels: &[ChannelPointDefinition],
        test_channel_pools: &mut TestChannelPools,
        batch_number: u32,
        test_plc_config: &TestPlcConfig,
        product_model: Option<String>,
        serial_number: Option<String>,
    ) -> Result<(Vec<ChannelTestInstance>, Vec<String>), AppError> {
        let mut batch_instances = Vec::new();
        let mut used_channel_ids = Vec::new();

        // 设置每批次最大通道数限制 - 根据测试PLC的实际容量和分批策略
        // 测试PLC总容量：AI:8 + AO:16 + DI:32 + DO:32 = 88个通道
        // 修复：提高每批次限制以支持88个通道的完整分配
        const MAX_CHANNELS_PER_BATCH: usize = 88; // 支持完整的88个通道分配

        // 为当前批次生成统一的批次ID
        let batch_id = format!("{}_batch_{}", uuid::Uuid::new_v4().to_string(), batch_number);

        log::info!("--- 批次{}分配详情 ---", batch_number);
        log::info!("每批次最大通道数限制: {}", MAX_CHANNELS_PER_BATCH);
        log::info!("批次ID: {}", batch_id);

        // 按类型分配通道，尽量填满测试PLC通道

        // 1. AI有源 → AO无源
        let ai_powered_true_channels: Vec<_> = remaining_channels.iter()
            .filter(|def| matches!(def.module_type, ModuleType::AI) && self.is_powered_channel(def))
            .collect();

        let available_slots = MAX_CHANNELS_PER_BATCH.saturating_sub(batch_instances.len());
        let allocated_count = std::cmp::min(
            std::cmp::min(ai_powered_true_channels.len(), test_channel_pools.ao_powered_false.len()),
            available_slots
        );
        for i in 0..allocated_count {
            let def = ai_powered_true_channels[i];
            let test_channel = &test_channel_pools.ao_powered_false[i];

            let instance = self.create_test_instance(
                def,
                &batch_id,
                batch_number,
                test_channel,
                product_model.clone(),
                serial_number.clone(),
            )?;

            log::info!("  AI有源[{}]: {} → {}", i + 1, def.tag, test_channel.channel_address);
            batch_instances.push(instance);
            used_channel_ids.push(def.id.clone());

            if batch_instances.len() >= MAX_CHANNELS_PER_BATCH {
                log::info!("批次{}已达到最大通道数限制，停止分配", batch_number);
                break;
            }
        }

        // 2. AI无源 → AO有源
        let ai_powered_false_channels: Vec<_> = remaining_channels.iter()
            .filter(|def| matches!(def.module_type, ModuleType::AI) && !self.is_powered_channel(def))
            .filter(|def| !used_channel_ids.contains(&def.id))
            .collect();

        let allocated_count = std::cmp::min(ai_powered_false_channels.len(), test_channel_pools.ao_powered_true.len());
        for i in 0..allocated_count {
            let def = ai_powered_false_channels[i];
            let test_channel = &test_channel_pools.ao_powered_true[i];

            let instance = self.create_test_instance(
                def,
                &batch_id,
                batch_number,
                test_channel,
                product_model.clone(),
                serial_number.clone(),
            )?;

            log::info!("  AI无源[{}]: {} → {}", i + 1, def.tag, test_channel.channel_address);
            batch_instances.push(instance);
            used_channel_ids.push(def.id.clone());
        }

        // 3. AO有源 → AI无源
        let ao_powered_true_channels: Vec<_> = remaining_channels.iter()
            .filter(|def| matches!(def.module_type, ModuleType::AO) && self.is_powered_channel(def))
            .filter(|def| !used_channel_ids.contains(&def.id))
            .collect();

        let allocated_count = std::cmp::min(ao_powered_true_channels.len(), test_channel_pools.ai_powered_false.len());
        for i in 0..allocated_count {
            let def = ao_powered_true_channels[i];
            let test_channel = &test_channel_pools.ai_powered_false[i];

            let instance = self.create_test_instance(
                def,
                &batch_id,
                batch_number,
                test_channel,
                product_model.clone(),
                serial_number.clone(),
            )?;

            log::info!("  AO有源[{}]: {} → {}", i + 1, def.tag, test_channel.channel_address);
            batch_instances.push(instance);
            used_channel_ids.push(def.id.clone());
        }

        // 4. AO无源 → AI有源
        let ao_powered_false_channels: Vec<_> = remaining_channels.iter()
            .filter(|def| matches!(def.module_type, ModuleType::AO) && !self.is_powered_channel(def))
            .filter(|def| !used_channel_ids.contains(&def.id))
            .collect();

        let allocated_count = std::cmp::min(ao_powered_false_channels.len(), test_channel_pools.ai_powered_true.len());
        for i in 0..allocated_count {
            let def = ao_powered_false_channels[i];
            let test_channel = &test_channel_pools.ai_powered_true[i];

            let instance = self.create_test_instance(
                def,
                &batch_id,
                batch_number,
                test_channel,
                product_model.clone(),
                serial_number.clone(),
            )?;

            log::info!("  AO无源[{}]: {} → {}", i + 1, def.tag, test_channel.channel_address);
            batch_instances.push(instance);
            used_channel_ids.push(def.id.clone());
        }

        // 5. DI有源 → DO无源
        let di_powered_true_channels: Vec<_> = remaining_channels.iter()
            .filter(|def| matches!(def.module_type, ModuleType::DI) && self.is_powered_channel(def))
            .filter(|def| !used_channel_ids.contains(&def.id))
            .collect();

        let allocated_count = std::cmp::min(di_powered_true_channels.len(), test_channel_pools.do_powered_false.len());
        for i in 0..allocated_count {
            let def = di_powered_true_channels[i];
            let test_channel = &test_channel_pools.do_powered_false[i];

            let instance = self.create_test_instance(
                def,
                &batch_id,
                batch_number,
                test_channel,
                product_model.clone(),
                serial_number.clone(),
            )?;

            log::info!("  DI有源[{}]: {} → {}", i + 1, def.tag, test_channel.channel_address);
            batch_instances.push(instance);
            used_channel_ids.push(def.id.clone());
        }

        // 6. DI无源 → DO有源
        let di_powered_false_channels: Vec<_> = remaining_channels.iter()
            .filter(|def| matches!(def.module_type, ModuleType::DI) && !self.is_powered_channel(def))
            .filter(|def| !used_channel_ids.contains(&def.id))
            .collect();

        let allocated_count = std::cmp::min(di_powered_false_channels.len(), test_channel_pools.do_powered_true.len());
        for i in 0..allocated_count {
            let def = di_powered_false_channels[i];
            let test_channel = &test_channel_pools.do_powered_true[i];

            let instance = self.create_test_instance(
                def,
                &batch_id,
                batch_number,
                test_channel,
                product_model.clone(),
                serial_number.clone(),
            )?;

            log::info!("  DI无源[{}]: {} → {}", i + 1, def.tag, test_channel.channel_address);
            batch_instances.push(instance);
            used_channel_ids.push(def.id.clone());
        }

        // 7. DO有源 → DI无源
        let do_powered_true_channels: Vec<_> = remaining_channels.iter()
            .filter(|def| matches!(def.module_type, ModuleType::DO) && self.is_powered_channel(def))
            .filter(|def| !used_channel_ids.contains(&def.id))
            .collect();

        let allocated_count = std::cmp::min(do_powered_true_channels.len(), test_channel_pools.di_powered_false.len());
        for i in 0..allocated_count {
            let def = do_powered_true_channels[i];
            let test_channel = &test_channel_pools.di_powered_false[i];

            let instance = self.create_test_instance(
                def,
                &batch_id,
                batch_number,
                test_channel,
                product_model.clone(),
                serial_number.clone(),
            )?;

            log::info!("  DO有源[{}]: {} → {}", i + 1, def.tag, test_channel.channel_address);
            batch_instances.push(instance);
            used_channel_ids.push(def.id.clone());
        }

        // 8. DO无源 → DI有源
        let do_powered_false_channels: Vec<_> = remaining_channels.iter()
            .filter(|def| matches!(def.module_type, ModuleType::DO) && !self.is_powered_channel(def))
            .filter(|def| !used_channel_ids.contains(&def.id))
            .collect();

        let allocated_count = std::cmp::min(do_powered_false_channels.len(), test_channel_pools.di_powered_true.len());
        for i in 0..allocated_count {
            let def = do_powered_false_channels[i];
            let test_channel = &test_channel_pools.di_powered_true[i];

            let instance = self.create_test_instance(
                def,
                &batch_id,
                batch_number,
                test_channel,
                product_model.clone(),
                serial_number.clone(),
            )?;

            log::info!("  DO无源[{}]: {} → {}", i + 1, def.tag, test_channel.channel_address);
            batch_instances.push(instance);
            used_channel_ids.push(def.id.clone());
        }

        log::info!("批次{}分配完成：总共分配{}个通道", batch_number, batch_instances.len());

        Ok((batch_instances, used_channel_ids))
    }

    /// 为单个批次分配通道（带容量限制版本）
    ///
    /// 这是修复后的批次分配逻辑：
    /// 1. 每个批次重新使用完整的测试PLC通道池（支持通道复用）
    /// 2. 根据测试PLC的实际容量来确定每批次的最大通道数
    /// 3. 优先填满一个批次再开始下一个批次
    fn allocate_single_batch_with_capacity_limit(
        &self,
        remaining_channels: &[ChannelPointDefinition],
        test_channel_pools: &mut TestChannelPools,
        batch_number: u32,
        test_plc_config: &TestPlcConfig,
        product_model: Option<String>,
        serial_number: Option<String>,
    ) -> Result<(Vec<ChannelTestInstance>, Vec<String>), AppError> {
        let mut batch_instances = Vec::new();
        let mut used_channel_ids = Vec::new();

        // 为当前批次生成统一的批次ID
        let batch_id = format!("{}_batch_{}", uuid::Uuid::new_v4().to_string(), batch_number);

        log::info!("--- 批次{}分配详情 ---", batch_number);
        log::info!("批次ID: {}", batch_id);

        // 计算测试PLC的实际容量限制
        let max_channels_per_batch = self.calculate_max_channels_per_batch(test_plc_config);
        log::info!("每批次最大通道数限制: {}", max_channels_per_batch);

        // 显示当前测试PLC通道池状态
        log::info!("=== 当前测试PLC通道池状态 ===");
        log::info!("AO无源池: {} 个通道", test_channel_pools.ao_powered_false.len());
        log::info!("AO有源池: {} 个通道", test_channel_pools.ao_powered_true.len());
        log::info!("AI有源池: {} 个通道", test_channel_pools.ai_powered_true.len());
        log::info!("AI无源池: {} 个通道", test_channel_pools.ai_powered_false.len());
        log::info!("DO无源池: {} 个通道", test_channel_pools.do_powered_false.len());
        log::info!("DO有源池: {} 个通道", test_channel_pools.do_powered_true.len());
        log::info!("DI无源池: {} 个通道", test_channel_pools.di_powered_false.len());
        log::info!("DI有源池: {} 个通道", test_channel_pools.di_powered_true.len());

        // 按类型分配通道，限制每批次最大通道数

        // 按照正确的分配规则进行分配：
        // 测试PLC -> 被测PLC
        // AI有源 -> AO无源
        // AI无源 -> AO有源
        // AO无源 -> AI有源
        // AO有源 -> AI无源
        // DI有源 -> DO无源
        // DI无源 -> DO有源
        // DO有源 -> DI无源
        // DO无源 -> DI有源

        // 1. AI有源(被测) → AO无源(测试PLC)
        let ai_powered_true_channels: Vec<_> = remaining_channels.iter()
            .filter(|def| matches!(def.module_type, ModuleType::AI) && self.is_powered_channel(def))
            .collect();

        let available_slots = max_channels_per_batch.saturating_sub(batch_instances.len());
        let allocated_count = std::cmp::min(
            std::cmp::min(ai_powered_true_channels.len(), test_channel_pools.ao_powered_false.len()),
            available_slots
        );
        for i in 0..allocated_count {
            let def = ai_powered_true_channels[i];
            let test_channel = &test_channel_pools.ao_powered_false[i];

            let instance = self.create_test_instance(
                def,
                &batch_id,
                batch_number,
                test_channel,
                product_model.clone(),
                serial_number.clone(),
            )?;

            log::info!("  AI有源(被测)[{}]: {} → {}(测试PLC)", i + 1, def.tag, test_channel.channel_address);
            batch_instances.push(instance);
            used_channel_ids.push(def.id.clone());

            if batch_instances.len() >= max_channels_per_batch {
                log::info!("批次{}已达到最大通道数限制，停止分配", batch_number);
                break;
            }
        }

        // 2. AI无源(被测) → AO有源(测试PLC)
        if batch_instances.len() < max_channels_per_batch {
            let ai_powered_false_channels: Vec<_> = remaining_channels.iter()
                .filter(|def| matches!(def.module_type, ModuleType::AI) && !self.is_powered_channel(def))
                .filter(|def| !used_channel_ids.contains(&def.id))
                .collect();

            let available_slots = max_channels_per_batch.saturating_sub(batch_instances.len());
            let allocated_count = std::cmp::min(
                std::cmp::min(ai_powered_false_channels.len(), test_channel_pools.ao_powered_true.len()),
                available_slots
            );
            for i in 0..allocated_count {
                let def = ai_powered_false_channels[i];
                let test_channel = &test_channel_pools.ao_powered_true[i];

                let instance = self.create_test_instance(
                    def,
                    &batch_id,
                    batch_number,
                    test_channel,
                    product_model.clone(),
                    serial_number.clone(),
                )?;

                log::info!("  AI无源(被测)[{}]: {} → {}(测试PLC)", i + 1, def.tag, test_channel.channel_address);
                batch_instances.push(instance);
                used_channel_ids.push(def.id.clone());

                if batch_instances.len() >= max_channels_per_batch {
                    break;
                }
            }
        }

        // 3. AO有源(被测) → AI无源(测试PLC) - 注意：当前测试PLC配置中AI无源=0，此规则暂时无法分配
        if batch_instances.len() < max_channels_per_batch {
            let ao_powered_true_channels: Vec<_> = remaining_channels.iter()
                .filter(|def| matches!(def.module_type, ModuleType::AO) && self.is_powered_channel(def))
                .filter(|def| !used_channel_ids.contains(&def.id))
                .collect();

            let available_slots = max_channels_per_batch.saturating_sub(batch_instances.len());
            let allocated_count = std::cmp::min(
                std::cmp::min(ao_powered_true_channels.len(), test_channel_pools.ai_powered_false.len()),
                available_slots
            );
            for i in 0..allocated_count {
                let def = ao_powered_true_channels[i];
                let test_channel = &test_channel_pools.ai_powered_false[i];

                let instance = self.create_test_instance(
                    def,
                    &batch_id,
                    batch_number,
                    test_channel,
                    product_model.clone(),
                    serial_number.clone(),
                )?;

                log::info!("  AO有源(被测)[{}]: {} → {}(测试PLC)", i + 1, def.tag, test_channel.channel_address);
                batch_instances.push(instance);
                used_channel_ids.push(def.id.clone());

                if batch_instances.len() >= max_channels_per_batch {
                    break;
                }
            }
        }

        // 4. AO无源(被测) → AI有源(测试PLC)
        if batch_instances.len() < max_channels_per_batch {
            let ao_powered_false_channels: Vec<_> = remaining_channels.iter()
                .filter(|def| matches!(def.module_type, ModuleType::AO) && !self.is_powered_channel(def))
                .filter(|def| !used_channel_ids.contains(&def.id))
                .collect();

            let available_slots = max_channels_per_batch.saturating_sub(batch_instances.len());
            let allocated_count = std::cmp::min(
                std::cmp::min(ao_powered_false_channels.len(), test_channel_pools.ai_powered_true.len()),
                available_slots
            );
            for i in 0..allocated_count {
                let def = ao_powered_false_channels[i];
                let test_channel = &test_channel_pools.ai_powered_true[i];

                let instance = self.create_test_instance(
                    def,
                    &batch_id,
                    batch_number,
                    test_channel,
                    product_model.clone(),
                    serial_number.clone(),
                )?;

                log::info!("  AO无源(被测)[{}]: {} → {}(测试PLC)", i + 1, def.tag, test_channel.channel_address);
                batch_instances.push(instance);
                used_channel_ids.push(def.id.clone());

                if batch_instances.len() >= max_channels_per_batch {
                    break;
                }
            }
        }

        // 5. DI有源 → DO无源
        if batch_instances.len() < max_channels_per_batch {
            let di_powered_true_channels: Vec<_> = remaining_channels.iter()
                .filter(|def| matches!(def.module_type, ModuleType::DI) && self.is_powered_channel(def))
                .filter(|def| !used_channel_ids.contains(&def.id))
                .collect();

            let available_slots = max_channels_per_batch.saturating_sub(batch_instances.len());
            let allocated_count = std::cmp::min(
                std::cmp::min(di_powered_true_channels.len(), test_channel_pools.do_powered_false.len()),
                available_slots
            );
            for i in 0..allocated_count {
                let def = di_powered_true_channels[i];
                let test_channel = &test_channel_pools.do_powered_false[i];

                let instance = self.create_test_instance(
                    def,
                    &batch_id,
                    batch_number,
                    test_channel,
                    product_model.clone(),
                    serial_number.clone(),
                )?;

                log::info!("  DI有源[{}]: {} → {}", i + 1, def.tag, test_channel.channel_address);
                batch_instances.push(instance);
                used_channel_ids.push(def.id.clone());

                if batch_instances.len() >= max_channels_per_batch {
                    break;
                }
            }
        }

        // 6. DI无源 → DO有源
        if batch_instances.len() < max_channels_per_batch {
            let di_powered_false_channels: Vec<_> = remaining_channels.iter()
                .filter(|def| matches!(def.module_type, ModuleType::DI) && !self.is_powered_channel(def))
                .filter(|def| !used_channel_ids.contains(&def.id))
                .collect();

            let available_slots = max_channels_per_batch.saturating_sub(batch_instances.len());
            let allocated_count = std::cmp::min(
                std::cmp::min(di_powered_false_channels.len(), test_channel_pools.do_powered_true.len()),
                available_slots
            );
            for i in 0..allocated_count {
                let def = di_powered_false_channels[i];
                let test_channel = &test_channel_pools.do_powered_true[i];

                let instance = self.create_test_instance(
                    def,
                    &batch_id,
                    batch_number,
                    test_channel,
                    product_model.clone(),
                    serial_number.clone(),
                )?;

                log::info!("  DI无源[{}]: {} → {}", i + 1, def.tag, test_channel.channel_address);
                batch_instances.push(instance);
                used_channel_ids.push(def.id.clone());

                if batch_instances.len() >= max_channels_per_batch {
                    break;
                }
            }
        }

        // 7. DO有源 → DI无源
        if batch_instances.len() < max_channels_per_batch {
            let do_powered_true_channels: Vec<_> = remaining_channels.iter()
                .filter(|def| matches!(def.module_type, ModuleType::DO) && self.is_powered_channel(def))
                .filter(|def| !used_channel_ids.contains(&def.id))
                .collect();

            let available_slots = max_channels_per_batch.saturating_sub(batch_instances.len());
            let allocated_count = std::cmp::min(
                std::cmp::min(do_powered_true_channels.len(), test_channel_pools.di_powered_false.len()),
                available_slots
            );
            for i in 0..allocated_count {
                let def = do_powered_true_channels[i];
                let test_channel = &test_channel_pools.di_powered_false[i];

                let instance = self.create_test_instance(
                    def,
                    &batch_id,
                    batch_number,
                    test_channel,
                    product_model.clone(),
                    serial_number.clone(),
                )?;

                log::info!("  DO有源[{}]: {} → {}", i + 1, def.tag, test_channel.channel_address);
                batch_instances.push(instance);
                used_channel_ids.push(def.id.clone());

                if batch_instances.len() >= max_channels_per_batch {
                    break;
                }
            }
        }

        // 8. DO无源 → DI有源
        if batch_instances.len() < max_channels_per_batch {
            let do_powered_false_channels: Vec<_> = remaining_channels.iter()
                .filter(|def| matches!(def.module_type, ModuleType::DO) && !self.is_powered_channel(def))
                .filter(|def| !used_channel_ids.contains(&def.id))
                .collect();

            let available_slots = max_channels_per_batch.saturating_sub(batch_instances.len());
            let allocated_count = std::cmp::min(
                std::cmp::min(do_powered_false_channels.len(), test_channel_pools.di_powered_true.len()),
                available_slots
            );
            for i in 0..allocated_count {
                let def = do_powered_false_channels[i];
                let test_channel = &test_channel_pools.di_powered_true[i];

                let instance = self.create_test_instance(
                    def,
                    &batch_id,
                    batch_number,
                    test_channel,
                    product_model.clone(),
                    serial_number.clone(),
                )?;

                log::info!("  DO无源[{}]: {} → {}", i + 1, def.tag, test_channel.channel_address);
                batch_instances.push(instance);
                used_channel_ids.push(def.id.clone());

                if batch_instances.len() >= max_channels_per_batch {
                    break;
                }
            }
        }

        log::info!("批次{}分配完成：总共分配{}个通道", batch_number, batch_instances.len());

        Ok((batch_instances, used_channel_ids))
    }

    /// 计算每批次最大通道数
    ///
    /// 根据测试PLC的实际通道容量来确定每批次能分配的最大通道数
    fn calculate_max_channels_per_batch(&self, test_plc_config: &TestPlcConfig) -> usize {
        // 计算测试PLC各类型通道的最小容量
        let test_channel_counts = self.calculate_test_channel_counts(test_plc_config);

        // 每批次的容量受限于测试PLC通道池的最小容量
        // 例如：如果AO无源只有8个，那么AI有源最多只能分配8个
        let ai_capacity = test_channel_counts.ao_powered_false_count + test_channel_counts.ao_powered_true_count;
        let ao_capacity = test_channel_counts.ai_powered_true_count + test_channel_counts.ai_powered_false_count;
        let di_capacity = test_channel_counts.do_powered_false_count + test_channel_counts.do_powered_true_count;
        let do_capacity = test_channel_counts.di_powered_true_count + test_channel_counts.di_powered_false_count;

        let total_capacity = ai_capacity + ao_capacity + di_capacity + do_capacity;

        log::info!("=== 测试PLC容量计算 ===");
        log::info!("AI通道容量: {}", ai_capacity);
        log::info!("AO通道容量: {}", ao_capacity);
        log::info!("DI通道容量: {}", di_capacity);
        log::info!("DO通道容量: {}", do_capacity);
        log::info!("总容量: {}", total_capacity);

        // 返回测试PLC的实际总容量，不设置人为限制
        // 让分配算法根据实际的测试PLC通道可用性来决定每批次的大小
        total_capacity
    }

    /// 分配特定类型的通道
    ///
    /// 这是一个辅助方法，用于分配特定类型的通道到当前批次
    #[allow(clippy::too_many_arguments)]
    fn allocate_channel_type<F>(
        &self,
        remaining_channels: &[ChannelPointDefinition],
        _test_channel_pools: &mut TestChannelPools,
        batch_instances: &mut Vec<ChannelTestInstance>,
        used_channel_ids: &mut Vec<String>,
        batch_id: &str,
        batch_number: u32,
        max_channels_per_batch: usize,
        filter_fn: F,
        test_channel_pool: &mut Vec<ComparisonTable>,
        channel_type_name: &str,
        product_model: Option<String>,
        serial_number: Option<String>,
    ) -> Result<(), AppError>
    where
        F: Fn(&ChannelPointDefinition) -> bool,
    {
        let filtered_channels: Vec<_> = remaining_channels.iter()
            .filter(|def| filter_fn(def))
            .filter(|def| !used_channel_ids.contains(&def.id))
            .collect();

        let available_slots = max_channels_per_batch.saturating_sub(batch_instances.len());
        let allocated_count = std::cmp::min(
            std::cmp::min(filtered_channels.len(), test_channel_pool.len()),
            available_slots
        );

        for i in 0..allocated_count {
            let def = filtered_channels[i];
            let test_channel = &test_channel_pool[i];

            let instance = self.create_test_instance(
                def,
                batch_id,
                batch_number,
                test_channel,
                product_model.clone(),
                serial_number.clone(),
            )?;

            log::info!("  {}[{}]: {} → {}", channel_type_name, i + 1, def.tag, test_channel.channel_address);
            batch_instances.push(instance);
            used_channel_ids.push(def.id.clone());

            if batch_instances.len() >= max_channels_per_batch {
                log::info!("批次{}已达到最大通道数限制，停止分配", batch_number);
                break;
            }
        }

        Ok(())
    }

    /// 为单个批次分配通道（正确版本）
    ///
    /// 这是修复后的批次分配逻辑：
    /// 1. 使用全局通道池，逐步消耗
    /// 2. 根据测试PLC的实际容量来确定每批次的最大通道数
    /// 3. 优先填满一个批次再开始下一个批次
    fn allocate_single_batch_correct(
        &self,
        remaining_channels: &[ChannelPointDefinition],
        global_test_channel_pools: &mut TestChannelPools,
        batch_number: u32,
        test_plc_config: &TestPlcConfig,
        product_model: Option<String>,
        serial_number: Option<String>,
    ) -> Result<(Vec<ChannelTestInstance>, Vec<String>), AppError> {
        let mut batch_instances = Vec::new();
        let mut used_channel_ids = Vec::new();

        // 为当前批次生成统一的批次ID
        let batch_id = format!("{}_batch_{}", uuid::Uuid::new_v4().to_string(), batch_number);

        log::info!("--- 批次{}分配详情 ---", batch_number);
        log::info!("批次ID: {}", batch_id);

        // 显示当前全局通道池状态
        log::info!("=== 当前全局通道池状态 ===");
        log::info!("AO无源池: {} 个通道", global_test_channel_pools.ao_powered_false.len());
        log::info!("AO有源池: {} 个通道", global_test_channel_pools.ao_powered_true.len());
        log::info!("AI有源池: {} 个通道", global_test_channel_pools.ai_powered_true.len());
        log::info!("AI无源池: {} 个通道", global_test_channel_pools.ai_powered_false.len());
        log::info!("DO无源池: {} 个通道", global_test_channel_pools.do_powered_false.len());
        log::info!("DO有源池: {} 个通道", global_test_channel_pools.do_powered_true.len());
        log::info!("DI无源池: {} 个通道", global_test_channel_pools.di_powered_false.len());
        log::info!("DI有源池: {} 个通道", global_test_channel_pools.di_powered_true.len());

        // 按类型分配通道，逐步消耗全局通道池

        // 1. AI有源 → AO无源
        let ai_powered_true_channels: Vec<_> = remaining_channels.iter()
            .filter(|def| matches!(def.module_type, ModuleType::AI) && self.is_powered_channel(def))
            .collect();

        let allocated_count = std::cmp::min(ai_powered_true_channels.len(), global_test_channel_pools.ao_powered_false.len());
        for i in 0..allocated_count {
            let def = ai_powered_true_channels[i];
            // 从全局通道池中取出一个通道（消耗）
            let test_channel = global_test_channel_pools.ao_powered_false.remove(0);

            let instance = self.create_test_instance(
                def,
                &batch_id,
                batch_number,
                &test_channel,
                product_model.clone(),
                serial_number.clone(),
            )?;

            log::info!("  AI有源[{}]: {} → {}", i + 1, def.tag, test_channel.channel_address);
            batch_instances.push(instance);
            used_channel_ids.push(def.id.clone());
        }

        // 2. 继续分配剩余的AI有源通道（如果第一轮没有分配完）
        let remaining_ai_powered_true_channels: Vec<_> = remaining_channels.iter()
            .filter(|def| matches!(def.module_type, ModuleType::AI) && self.is_powered_channel(def))
            .filter(|def| !used_channel_ids.contains(&def.id))
            .collect();

        let allocated_count = std::cmp::min(remaining_ai_powered_true_channels.len(), global_test_channel_pools.ao_powered_false.len());
        for i in 0..allocated_count {
            let def = remaining_ai_powered_true_channels[i];
            // 从全局通道池中取出一个通道（消耗）
            let test_channel = global_test_channel_pools.ao_powered_false.remove(0);

            let instance = self.create_test_instance(
                def,
                &batch_id,
                batch_number,
                &test_channel,
                product_model.clone(),
                serial_number.clone(),
            )?;

            log::info!("  AI有源[{}]: {} → {}", i + allocated_count + 1, def.tag, test_channel.channel_address);
            batch_instances.push(instance);
            used_channel_ids.push(def.id.clone());
        }

        // 3. AI无源 → AO有源
        let ai_powered_false_channels: Vec<_> = remaining_channels.iter()
            .filter(|def| matches!(def.module_type, ModuleType::AI) && !self.is_powered_channel(def))
            .filter(|def| !used_channel_ids.contains(&def.id))
            .collect();

        let allocated_count = std::cmp::min(ai_powered_false_channels.len(), global_test_channel_pools.ao_powered_true.len());
        for i in 0..allocated_count {
            let def = ai_powered_false_channels[i];
            // 从全局通道池中取出一个通道（消耗）
            let test_channel = global_test_channel_pools.ao_powered_true.remove(0);

            let instance = self.create_test_instance(
                def,
                &batch_id,
                batch_number,
                &test_channel,
                product_model.clone(),
                serial_number.clone(),
            )?;

            log::info!("  AI无源[{}]: {} → {}", i + 1, def.tag, test_channel.channel_address);
            batch_instances.push(instance);
            used_channel_ids.push(def.id.clone());
        }

        // 4. AO无源 → AI有源
        let ao_powered_false_channels: Vec<_> = remaining_channels.iter()
            .filter(|def| matches!(def.module_type, ModuleType::AO) && !self.is_powered_channel(def))
            .filter(|def| !used_channel_ids.contains(&def.id))
            .collect();

        let allocated_count = std::cmp::min(ao_powered_false_channels.len(), global_test_channel_pools.ai_powered_true.len());
        for i in 0..allocated_count {
            let def = ao_powered_false_channels[i];
            // 从全局通道池中取出一个通道（消耗）
            let test_channel = global_test_channel_pools.ai_powered_true.remove(0);

            let instance = self.create_test_instance(
                def,
                &batch_id,
                batch_number,
                &test_channel,
                product_model.clone(),
                serial_number.clone(),
            )?;

            log::info!("  AO无源[{}]: {} → {}", i + 1, def.tag, test_channel.channel_address);
            batch_instances.push(instance);
            used_channel_ids.push(def.id.clone());
        }

        // 5. AO有源 → AI无源
        let ao_powered_true_channels: Vec<_> = remaining_channels.iter()
            .filter(|def| matches!(def.module_type, ModuleType::AO) && self.is_powered_channel(def))
            .filter(|def| !used_channel_ids.contains(&def.id))
            .collect();

        let allocated_count = std::cmp::min(ao_powered_true_channels.len(), global_test_channel_pools.ai_powered_false.len());
        for i in 0..allocated_count {
            let def = ao_powered_true_channels[i];
            // 从全局通道池中取出一个通道（消耗）
            let test_channel = global_test_channel_pools.ai_powered_false.remove(0);

            let instance = self.create_test_instance(
                def,
                &batch_id,
                batch_number,
                &test_channel,
                product_model.clone(),
                serial_number.clone(),
            )?;

            log::info!("  AO有源[{}]: {} → {}", i + 1, def.tag, test_channel.channel_address);
            batch_instances.push(instance);
            used_channel_ids.push(def.id.clone());
        }

        // 6. 继续分配剩余的DI有源通道
        let remaining_di_powered_true_channels: Vec<_> = remaining_channels.iter()
            .filter(|def| matches!(def.module_type, ModuleType::DI) && self.is_powered_channel(def))
            .filter(|def| !used_channel_ids.contains(&def.id))
            .collect();

        let allocated_count = std::cmp::min(remaining_di_powered_true_channels.len(), global_test_channel_pools.do_powered_false.len());
        for i in 0..allocated_count {
            let def = remaining_di_powered_true_channels[i];
            // 从全局通道池中取出一个通道（消耗）
            let test_channel = global_test_channel_pools.do_powered_false.remove(0);

            let instance = self.create_test_instance(
                def,
                &batch_id,
                batch_number,
                &test_channel,
                product_model.clone(),
                serial_number.clone(),
            )?;

            log::info!("  DI有源[{}]: {} → {}", i + 1, def.tag, test_channel.channel_address);
            batch_instances.push(instance);
            used_channel_ids.push(def.id.clone());
        }

        // 7. DI无源 → DO有源
        let di_powered_false_channels: Vec<_> = remaining_channels.iter()
            .filter(|def| matches!(def.module_type, ModuleType::DI) && !self.is_powered_channel(def))
            .filter(|def| !used_channel_ids.contains(&def.id))
            .collect();

        let allocated_count = std::cmp::min(di_powered_false_channels.len(), global_test_channel_pools.do_powered_true.len());
        for i in 0..allocated_count {
            let def = di_powered_false_channels[i];
            // 从全局通道池中取出一个通道（消耗）
            let test_channel = global_test_channel_pools.do_powered_true.remove(0);

            let instance = self.create_test_instance(
                def,
                &batch_id,
                batch_number,
                &test_channel,
                product_model.clone(),
                serial_number.clone(),
            )?;

            log::info!("  DI无源[{}]: {} → {}", i + 1, def.tag, test_channel.channel_address);
            batch_instances.push(instance);
            used_channel_ids.push(def.id.clone());
        }

        // 8. 继续分配剩余的DO有源通道
        let remaining_do_powered_true_channels: Vec<_> = remaining_channels.iter()
            .filter(|def| matches!(def.module_type, ModuleType::DO) && self.is_powered_channel(def))
            .filter(|def| !used_channel_ids.contains(&def.id))
            .collect();

        let allocated_count = std::cmp::min(remaining_do_powered_true_channels.len(), global_test_channel_pools.di_powered_false.len());
        for i in 0..allocated_count {
            let def = remaining_do_powered_true_channels[i];
            // 从全局通道池中取出一个通道（消耗）
            let test_channel = global_test_channel_pools.di_powered_false.remove(0);

            let instance = self.create_test_instance(
                def,
                &batch_id,
                batch_number,
                &test_channel,
                product_model.clone(),
                serial_number.clone(),
            )?;

            log::info!("  DO有源[{}]: {} → {}", i + 1, def.tag, test_channel.channel_address);
            batch_instances.push(instance);
            used_channel_ids.push(def.id.clone());
        }

        // 9. DO无源 → DI有源
        let do_powered_false_channels: Vec<_> = remaining_channels.iter()
            .filter(|def| matches!(def.module_type, ModuleType::DO) && !self.is_powered_channel(def))
            .filter(|def| !used_channel_ids.contains(&def.id))
            .collect();

        let allocated_count = std::cmp::min(do_powered_false_channels.len(), global_test_channel_pools.di_powered_true.len());
        for i in 0..allocated_count {
            let def = do_powered_false_channels[i];
            // 从全局通道池中取出一个通道（消耗）
            let test_channel = global_test_channel_pools.di_powered_true.remove(0);

            let instance = self.create_test_instance(
                def,
                &batch_id,
                batch_number,
                &test_channel,
                product_model.clone(),
                serial_number.clone(),
            )?;

            log::info!("  DO无源[{}]: {} → {}", i + 1, def.tag, test_channel.channel_address);
            batch_instances.push(instance);
            used_channel_ids.push(def.id.clone());
        }

        log::info!("批次{}分配完成：总共分配{}个通道", batch_number, batch_instances.len());

        Ok((batch_instances, used_channel_ids))
    }

    /// 创建测试PLC通道池
    ///
    /// 将测试PLC的通道按类型分组，方便分配
    fn create_test_channel_pools(&self, test_plc_config: &TestPlcConfig) -> TestChannelPools {
        let mut pools = TestChannelPools::default();

        for table in &test_plc_config.comparison_tables {
            match (&table.channel_type, table.is_powered) {
                (ModuleType::AO, false) => pools.ao_powered_false.push(table.clone()),
                (ModuleType::AO, true) => pools.ao_powered_true.push(table.clone()),
                (ModuleType::AI, true) => pools.ai_powered_true.push(table.clone()),
                (ModuleType::AI, false) => pools.ai_powered_false.push(table.clone()),
                (ModuleType::DO, false) => pools.do_powered_false.push(table.clone()),
                (ModuleType::DO, true) => pools.do_powered_true.push(table.clone()),
                (ModuleType::DI, false) => pools.di_powered_false.push(table.clone()),
                (ModuleType::DI, true) => pools.di_powered_true.push(table.clone()),
                _ => {}
            }
        }

        log::info!("=== 测试PLC通道池统计 ===");
        log::info!("AO无源池: {} 个通道", pools.ao_powered_false.len());
        log::info!("AO有源池: {} 个通道", pools.ao_powered_true.len());
        log::info!("AI有源池: {} 个通道", pools.ai_powered_true.len());
        log::info!("AI无源池: {} 个通道", pools.ai_powered_false.len());
        log::info!("DO无源池: {} 个通道", pools.do_powered_false.len());
        log::info!("DO有源池: {} 个通道", pools.do_powered_true.len());
        log::info!("DI无源池: {} 个通道", pools.di_powered_false.len());
        log::info!("DI有源池: {} 个通道", pools.di_powered_true.len());

        pools
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
            match (&table.channel_type, table.is_powered) {
                (ModuleType::AO, false) => {
                    counts.ao_powered_false_count += 1;  // 用于测试AI有源
                },
                (ModuleType::AO, true)  => {
                    counts.ao_powered_true_count += 1;    // 用于测试AI无源
                },
                (ModuleType::AI, false) => {
                    counts.ai_powered_false_count += 1;  // 用于测试AO有源
                },
                (ModuleType::AI, true)  => {
                    counts.ai_powered_true_count += 1;    // 用于测试AO无源
                },
                (ModuleType::DO, false) => {
                    counts.do_powered_false_count += 1;  // 用于测试DI有源
                },
                (ModuleType::DO, true)  => {
                    counts.do_powered_true_count += 1;    // 用于测试DI无源
                },
                (ModuleType::DI, false) => {
                    counts.di_powered_false_count += 1;  // 用于测试DO有源
                },
                (ModuleType::DI, true)  => {
                    counts.di_powered_true_count += 1;    // 用于测试DO无源
                },
                _ => {
                    log::warn!("映射[{}]: {} 未知通道类型 {:?}",
                        i + 1, table.channel_address, table.channel_type);
                }
            }
        }

        log::info!("=== 测试PLC通道统计结果 ===");
        log::info!("AO无源(用于测试AI有源): {}", counts.ao_powered_false_count);
        log::info!("AO有源(用于测试AI无源): {}", counts.ao_powered_true_count);
        log::info!("AI无源(用于测试AO有源): {}", counts.ai_powered_false_count);
        log::info!("AI有源(用于测试AO无源): {}", counts.ai_powered_true_count);
        log::info!("DO无源(用于测试DI有源): {}", counts.do_powered_false_count);
        log::info!("DO有源(用于测试DI无源): {}", counts.do_powered_true_count);
        log::info!("DI无源(用于测试DO有源): {}", counts.di_powered_false_count);
        log::info!("DI有源(用于测试DO无源): {}", counts.di_powered_true_count);

        counts
    }

    /// 创建测试实例
    fn create_test_instance(
        &self,
        definition: &ChannelPointDefinition,
        batch_id: &str,
        batch_number: u32,
        test_channel: &ComparisonTable,
        product_model: Option<String>,
        serial_number: Option<String>,
    ) -> Result<ChannelTestInstance, AppError> {
        let batch_name = format!("批次{}", batch_number);

        Ok(ChannelTestInstance {
            instance_id: uuid::Uuid::new_v4().to_string(),
            definition_id: definition.id.clone(),
            test_batch_id: batch_id.to_string(),
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
            test_plc_channel_tag: Some(test_channel.channel_address.clone()),
            test_plc_communication_address: Some(test_channel.communication_address.clone()),
            current_operator: None,
            retries_count: 0,
            transient_data: HashMap::new(),
        })
    }

    /// 创建批次信息
    fn create_batch_info(
        &self,
        batch_number: u32,
        instances: &[ChannelTestInstance],
        product_model: Option<String>,
        serial_number: Option<String>,
    ) -> TestBatchInfo {
        let batch_id = if let Some(first_instance) = instances.first() {
            first_instance.test_batch_id.clone()
        } else {
            format!("{}_batch_{}", uuid::Uuid::new_v4().to_string(), batch_number)
        };

        let mut batch_info = TestBatchInfo::new(product_model, serial_number);
        batch_info.batch_id = batch_id;
        batch_info.batch_name = format!("批次{}", batch_number);
        batch_info.total_points = instances.len() as u32;
        batch_info.last_updated_time = Utc::now();

        batch_info
    }

    /// 按有源/无源类型分组通道
    fn group_channels_by_power_type(&self, definitions: &[ChannelPointDefinition]) -> ChannelGroups {
        let mut groups = ChannelGroups::default();

        for def in definitions {
            let is_powered = self.is_powered_channel(def);

            match def.module_type {
                ModuleType::AI => {
                    if is_powered {
                        groups.ai_powered_true.push(def.clone());
                    } else {
                        groups.ai_powered_false.push(def.clone());
                    }
                }
                ModuleType::AO => {
                    if is_powered {
                        groups.ao_powered_true.push(def.clone());
                    } else {
                        groups.ao_powered_false.push(def.clone());
                    }
                }
                ModuleType::DI => {
                    if is_powered {
                        groups.di_powered_true.push(def.clone());
                    } else {
                        groups.di_powered_false.push(def.clone());
                    }
                }
                ModuleType::DO => {
                    if is_powered {
                        groups.do_powered_true.push(def.clone());
                    } else {
                        groups.do_powered_false.push(def.clone());
                    }
                }
                _ => {
                    // 其他类型暂时忽略
                }
            }
        }

        groups
    }

    /// 判断通道是否为有源
    /// 根据真实数据规则：power_supply_type字段中包含"无源"就是无源通道，否则为有源
    fn is_powered_channel(&self, definition: &ChannelPointDefinition) -> bool {
        // 首先检查power_supply_type字段
        if !definition.power_supply_type.is_empty() {
            return !definition.power_supply_type.contains("无源");
        }

        // 如果power_supply_type为空，则检查variable_description字段作为备用
        !definition.variable_description.contains("无源")
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

/// 测试PLC通道池，按类型和有源/无源分组
#[derive(Debug, Clone, Default)]
struct TestChannelPools {
    ao_powered_false: Vec<ComparisonTable>,  // AO无源通道（用于测试AI有源）
    ao_powered_true: Vec<ComparisonTable>,   // AO有源通道（用于测试AI无源）
    ai_powered_true: Vec<ComparisonTable>,   // AI有源通道（用于测试AO无源）
    ai_powered_false: Vec<ComparisonTable>,  // AI无源通道（用于测试AO有源）
    do_powered_false: Vec<ComparisonTable>,  // DO无源通道（用于测试DI有源）
    do_powered_true: Vec<ComparisonTable>,   // DO有源通道（用于测试DI无源）
    di_powered_false: Vec<ComparisonTable>,  // DI无源通道（用于测试DO有源）
    di_powered_true: Vec<ComparisonTable>,   // DI有源通道（用于测试DO无源）
}

#[derive(Debug, Clone, Default)]
struct TestChannelCounts {
    ao_powered_false_count: usize,    // AO无源通道数（用于测试AI有源）
    ao_powered_true_count: usize,      // AO有源通道数（用于测试AI无源）
    ai_powered_false_count: usize,    // AI无源通道数（用于测试AO有源）
    ai_powered_true_count: usize,      // AI有源通道数（用于测试AO无源）
    do_powered_false_count: usize,    // DO无源通道数（用于测试DI有源）
    do_powered_true_count: usize,      // DO有源通道数（用于测试DI无源）
    di_powered_false_count: usize,    // DI无源通道数（用于测试DO有源）
    di_powered_true_count: usize,      // DI有源通道数（用于测试DO无源）
    min_channels_per_batch: usize, // 每批次最小通道数
}

/// 通道分组结构体，按模块类型和有源/无源分组
#[derive(Debug, Clone, Default)]
struct ChannelGroups {
    ai_powered_true: Vec<ChannelPointDefinition>,   // AI有源通道
    ai_powered_false: Vec<ChannelPointDefinition>,  // AI无源通道
    ao_powered_true: Vec<ChannelPointDefinition>,   // AO有源通道
    ao_powered_false: Vec<ChannelPointDefinition>,  // AO无源通道
    di_powered_true: Vec<ChannelPointDefinition>,   // DI有源通道
    di_powered_false: Vec<ChannelPointDefinition>,  // DI无源通道
    do_powered_true: Vec<ChannelPointDefinition>,   // DO有源通道
    do_powered_false: Vec<ChannelPointDefinition>,  // DO无源通道
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

        // 清除分配信息，但不直接修改状态（符合FAT-CSM-001规则）
        for instance in &mut instances {
            instance.test_batch_id = String::new();
            instance.test_batch_name = String::new();
            instance.test_plc_channel_tag = None;
            instance.test_plc_communication_address = None;
            // 移除直接修改状态的代码 - 这应该通过ChannelStateManager处理
            // instance.overall_status = OverallTestStatus::NotTested;
            instance.last_updated_time = Utc::now();
        }

        // TODO: 如果需要重置状态，应该调用ChannelStateManager的方法
        // 例如: channel_state_manager.reset_for_reallocation(instance).await?;

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
        let mut comparison_tables = Vec::new();

        // 创建足够的测试PLC通道来支持测试
        // 每种类型创建8个通道，支持更大的批次

        // AO通道 (用于测试AI)
        for i in 0..8 {
            // AO无源 (用于测试AI有源)
            comparison_tables.push(ComparisonTable {
                channel_address: format!("TestRig.AO.CH{:02}_NoP", i + 1),
                communication_address: format!("DB1.DBD{}", i * 4),
                channel_type: ModuleType::AO,
                is_powered: false,
            });

            // AO有源 (用于测试AI无源)
            comparison_tables.push(ComparisonTable {
                channel_address: format!("TestRig.AO.CH{:02}_Pow", i + 1),
                communication_address: format!("DB1.DBD{}", (i + 8) * 4),
                channel_type: ModuleType::AO,
                is_powered: true,
            });
        }

        // AI通道 (用于测试AO)
        for i in 0..8 {
            // AI有源 (用于测试AO无源)
            comparison_tables.push(ComparisonTable {
                channel_address: format!("TestRig.AI.CH{:02}_Pow", i + 1),
                communication_address: format!("DB2.DBD{}", i * 4),
                channel_type: ModuleType::AI,
                is_powered: true,
            });

            // AI无源 (用于测试AO有源)
            comparison_tables.push(ComparisonTable {
                channel_address: format!("TestRig.AI.CH{:02}_NoP", i + 1),
                communication_address: format!("DB2.DBD{}", (i + 8) * 4),
                channel_type: ModuleType::AI,
                is_powered: false,
            });
        }

        // DO通道 (用于测试DI)
        for i in 0..8 {
            // DO无源 (用于测试DI有源)
            comparison_tables.push(ComparisonTable {
                channel_address: format!("TestRig.DO.CH{:02}_NoP", i + 1),
                communication_address: format!("DB3.DBX{}.{}", i / 8, i % 8),
                channel_type: ModuleType::DO,
                is_powered: false,
            });

            // DO有源 (用于测试DI无源)
            comparison_tables.push(ComparisonTable {
                channel_address: format!("TestRig.DO.CH{:02}_Pow", i + 1),
                communication_address: format!("DB3.DBX{}.{}", (i + 8) / 8, (i + 8) % 8),
                channel_type: ModuleType::DO,
                is_powered: true,
            });
        }

        // DI通道 (用于测试DO)
        for i in 0..8 {
            // DI无源 (用于测试DO有源)
            comparison_tables.push(ComparisonTable {
                channel_address: format!("TestRig.DI.CH{:02}_NoP", i + 1),
                communication_address: format!("DB4.DBX{}.{}", i / 8, i % 8),
                channel_type: ModuleType::DI,
                is_powered: false,
            });

            // DI有源 (用于测试DO无源)
            comparison_tables.push(ComparisonTable {
                channel_address: format!("TestRig.DI.CH{:02}_Pow", i + 1),
                communication_address: format!("DB4.DBX{}.{}", (i + 8) / 8, (i + 8) % 8),
                channel_type: ModuleType::DI,
                is_powered: true,
            });
        }

        TestPlcConfig {
            brand_type: "Siemens".to_string(),
            ip_address: "192.168.1.100".to_string(),
            comparison_tables,
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