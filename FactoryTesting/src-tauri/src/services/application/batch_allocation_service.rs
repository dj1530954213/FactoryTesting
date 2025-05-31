/// 批次分配服务
///
/// 负责实现智能的通道分配算法
/// 基于原C#项目的分配逻辑重构
use std::sync::Arc;
use std::collections::HashMap;
use sea_orm::{DatabaseConnection, EntityTrait, ColumnTrait, QueryFilter, ActiveModelTrait};
use crate::models::entities::{channel_point_definition, channel_test_instance, test_batch_info};
use crate::models::structs::{ChannelPointDefinition, ChannelTestInstance, TestBatchInfo};
use crate::models::enums::ModuleType;
use crate::error::AppError;
use log::{info, warn};

/// 分配策略
#[derive(Debug, Clone)]
pub enum AllocationStrategy {
    /// 按模块类型分组
    ByModuleType,
    /// 按站点分组
    ByStation,
    /// 按产品型号分组
    ByProductModel,
    /// 智能分配（综合考虑多个因素）
    Smart,
}

/// 分配结果
#[derive(Debug, Clone)]
pub struct AllocationResult {
    pub batch_info: TestBatchInfo,
    pub test_instances: Vec<ChannelTestInstance>,
    pub allocation_summary: AllocationSummary,
}

/// 分配摘要
#[derive(Debug, Clone, serde::Serialize)]
pub struct AllocationSummary {
    pub total_channels: usize,
    pub ai_channels: usize,
    pub ao_channels: usize,
    pub di_channels: usize,
    pub do_channels: usize,
    pub stations: Vec<String>,
    pub estimated_test_duration_minutes: u32,
}

impl AllocationSummary {
    pub fn new() -> Self {
        Self {
            total_channels: 0,
            ai_channels: 0,
            ao_channels: 0,
            di_channels: 0,
            do_channels: 0,
            stations: Vec::new(),
            estimated_test_duration_minutes: 0,
        }
    }

    pub fn add_channel(&mut self, module_type: &ModuleType, station: &str) {
        self.total_channels += 1;

        match module_type {
            ModuleType::AI => self.ai_channels += 1,
            ModuleType::AO => self.ao_channels += 1,
            ModuleType::DI => self.di_channels += 1,
            ModuleType::DO => self.do_channels += 1,
            _ => {}
        }

        if !self.stations.contains(&station.to_string()) {
            self.stations.push(station.to_string());
        }
    }

    pub fn calculate_estimated_duration(&mut self) {
        // 基于通道类型估算测试时长
        // AI/AO: 每个通道约3分钟（包括硬点测试、报警测试等）
        // DI/DO: 每个通道约1分钟（主要是硬点测试）
        let ai_ao_duration = (self.ai_channels + self.ao_channels) * 3;
        let di_do_duration = (self.di_channels + self.do_channels) * 1;

        self.estimated_test_duration_minutes = (ai_ao_duration + di_do_duration) as u32;
    }
}

/// 批次分配服务
pub struct BatchAllocationService {
    db: Arc<DatabaseConnection>,
}

impl BatchAllocationService {
    /// 创建新的批次分配服务实例
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }

    /// 创建测试批次并分配通道
    ///
    /// # 参数
    /// * `batch_name` - 批次名称
    /// * `product_model` - 产品型号（可选）
    /// * `operator_name` - 操作员名称（可选）
    /// * `strategy` - 分配策略
    /// * `filter_criteria` - 过滤条件（可选）
    ///
    /// # 返回
    /// * `Result<AllocationResult, AppError>` - 分配结果
    pub async fn create_test_batch(
        &self,
        batch_name: String,
        product_model: Option<String>,
        operator_name: Option<String>,
        strategy: AllocationStrategy,
        filter_criteria: Option<HashMap<String, String>>,
    ) -> Result<AllocationResult, AppError> {
        info!("开始创建测试批次: {}", batch_name);

        // 1. 获取可用的通道定义
        let available_definitions = self.get_available_definitions(filter_criteria).await?;

        if available_definitions.is_empty() {
            return Err(AppError::validation_error("没有可用的通道定义"));
        }

        info!("找到{}个可用的通道定义", available_definitions.len());

        // 2. 根据策略分组通道
        let grouped_definitions = self.group_definitions_by_strategy(&available_definitions, &strategy);

        // 3. 创建测试批次信息
        let batch_info = self.create_batch_info(
            batch_name,
            product_model,
            operator_name,
            &available_definitions,
        ).await?;

        // 4. 创建测试实例
        let test_instances = self.create_test_instances(
            &batch_info,
            &grouped_definitions,
        ).await?;

        // 5. 生成分配摘要
        let allocation_summary = self.generate_allocation_summary(&available_definitions);

        info!(
            "测试批次创建完成: {} - 总计{}个通道，预计测试时长{}分钟",
            batch_info.batch_name,
            allocation_summary.total_channels,
            allocation_summary.estimated_test_duration_minutes
        );

        Ok(AllocationResult {
            batch_info,
            test_instances,
            allocation_summary,
        })
    }

    /// 获取可用的通道定义
    async fn get_available_definitions(
        &self,
        filter_criteria: Option<HashMap<String, String>>,
    ) -> Result<Vec<ChannelPointDefinition>, AppError> {
        let mut query = channel_point_definition::Entity::find();

        // 应用过滤条件
        if let Some(criteria) = filter_criteria {
            for (key, value) in criteria {
                match key.as_str() {
                    "station_name" => {
                        query = query.filter(channel_point_definition::Column::StationName.eq(value));
                    }
                    "module_type" => {
                        query = query.filter(channel_point_definition::Column::ModuleType.eq(value));
                    }
                    "module_name" => {
                        query = query.filter(channel_point_definition::Column::ModuleName.eq(value));
                    }
                    _ => {
                        warn!("未知的过滤条件: {}", key);
                    }
                }
            }
        }

        let models = query.all(&*self.db).await
            .map_err(|e| AppError::persistence_error(format!("查询通道定义失败: {}", e)))?;

        let definitions: Vec<ChannelPointDefinition> = models.iter().map(|m| m.into()).collect();

        Ok(definitions)
    }

    /// 根据策略分组通道定义
    fn group_definitions_by_strategy(
        &self,
        definitions: &[ChannelPointDefinition],
        strategy: &AllocationStrategy,
    ) -> Vec<Vec<ChannelPointDefinition>> {
        match strategy {
            AllocationStrategy::ByModuleType => {
                self.group_by_module_type(definitions)
            }
            AllocationStrategy::ByStation => {
                self.group_by_station(definitions)
            }
            AllocationStrategy::ByProductModel => {
                // 简单实现：按模块类型分组
                self.group_by_module_type(definitions)
            }
            AllocationStrategy::Smart => {
                self.smart_grouping(definitions)
            }
        }
    }

    /// 按模块类型分组
    fn group_by_module_type(&self, definitions: &[ChannelPointDefinition]) -> Vec<Vec<ChannelPointDefinition>> {
        let mut groups: HashMap<ModuleType, Vec<ChannelPointDefinition>> = HashMap::new();

        for def in definitions {
            groups.entry(def.module_type.clone()).or_insert_with(Vec::new).push(def.clone());
        }

        groups.into_values().collect()
    }

    /// 按站点分组
    fn group_by_station(&self, definitions: &[ChannelPointDefinition]) -> Vec<Vec<ChannelPointDefinition>> {
        let mut groups: HashMap<String, Vec<ChannelPointDefinition>> = HashMap::new();

        for def in definitions {
            groups.entry(def.station_name.clone()).or_insert_with(Vec::new).push(def.clone());
        }

        groups.into_values().collect()
    }

    /// 智能分组（综合考虑模块类型、站点等因素）
    fn smart_grouping(&self, definitions: &[ChannelPointDefinition]) -> Vec<Vec<ChannelPointDefinition>> {
        // 先按站点分组，再按模块类型细分
        let station_groups = self.group_by_station(definitions);
        let mut final_groups = Vec::new();

        for station_group in station_groups {
            let module_groups = self.group_by_module_type(&station_group);
            final_groups.extend(module_groups);
        }

        final_groups
    }

    /// 创建批次信息
    async fn create_batch_info(
        &self,
        batch_name: String,
        product_model: Option<String>,
        operator_name: Option<String>,
        definitions: &[ChannelPointDefinition],
    ) -> Result<TestBatchInfo, AppError> {
        let mut batch_info = TestBatchInfo::new(
            product_model,
            None, // serial_number
        );
        batch_info.batch_name = batch_name;
        batch_info.operator_name = operator_name;

        // 设置统计信息
        batch_info.total_points = definitions.len() as u32;
        // 初始状态下，所有点位都是未测试的
        // batch_info.not_tested_points = definitions.len() as u32; // 这个字段不存在

        // 设置站点信息（取第一个定义的站点）
        // if let Some(first_def) = definitions.first() {
        //     batch_info.station_name = Some(first_def.station_name.clone()); // 这个字段不存在
        // }

        // 保存到数据库
        let active_model: test_batch_info::ActiveModel = (&batch_info).into();
        let saved_model = active_model.insert(&*self.db).await
            .map_err(|e| AppError::persistence_error(format!("保存批次信息失败: {}", e)))?;

        Ok((&saved_model).into())
    }

    /// 创建测试实例
    async fn create_test_instances(
        &self,
        batch_info: &TestBatchInfo,
        grouped_definitions: &[Vec<ChannelPointDefinition>],
    ) -> Result<Vec<ChannelTestInstance>, AppError> {
        let mut test_instances = Vec::new();

        for group in grouped_definitions {
            for definition in group {
                let mut test_instance = ChannelTestInstance::new(
                    definition.id.clone(),
                    batch_info.batch_id.clone(),
                );
                test_instance.test_batch_name = batch_info.batch_name.clone();
                // 其他字段可以通过定义获取，但不在构造函数中设置

                // 保存到数据库
                let active_model: channel_test_instance::ActiveModel = (&test_instance).into();
                let saved_model = active_model.insert(&*self.db).await
                    .map_err(|e| AppError::persistence_error(format!("保存测试实例失败: {}", e)))?;

                test_instances.push((&saved_model).into());
            }
        }

        info!("创建了{}个测试实例", test_instances.len());
        Ok(test_instances)
    }

    /// 生成分配摘要
    fn generate_allocation_summary(&self, definitions: &[ChannelPointDefinition]) -> AllocationSummary {
        let mut summary = AllocationSummary::new();

        for definition in definitions {
            summary.add_channel(&definition.module_type, &definition.station_name);
        }

        summary.calculate_estimated_duration();
        summary
    }
}
