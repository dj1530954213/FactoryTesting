/// 批次分配服务
///
/// 负责实现智能的通道分配算法
/// 基于原C#项目的分配逻辑重构
use std::sync::Arc;
use std::collections::HashMap;
use sea_orm::{DatabaseConnection, EntityTrait, ColumnTrait, QueryFilter, ActiveModelTrait, Statement, ConnectionTrait};
use crate::models::entities::{channel_point_definition, channel_test_instance, test_batch_info};
use crate::models::structs::{ChannelPointDefinition, ChannelTestInstance, TestBatchInfo};
use crate::models::enums::ModuleType;
use crate::error::AppError;
use chrono::Utc;
use uuid::Uuid;
use serde_json;
use log::{info, warn, error};
use crate::domain::services::channel_state_manager::IChannelStateManager;

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
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
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
    channel_state_manager: Arc<dyn IChannelStateManager>,
}

impl BatchAllocationService {
    /// 创建新的批次分配服务实例
    pub fn new(db: Arc<DatabaseConnection>, channel_state_manager: Arc<dyn IChannelStateManager>) -> Self {
        Self { db, channel_state_manager }
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
        info!("🔥 [BATCH_ALLOCATION] 开始创建测试批次: {}", batch_name);
        info!("🔥 [BATCH_ALLOCATION] 产品型号: {:?}", product_model);
        info!("🔥 [BATCH_ALLOCATION] 操作员: {:?}", operator_name);
        info!("🔥 [BATCH_ALLOCATION] 分配策略: {:?}", strategy);
        info!("🔥 [BATCH_ALLOCATION] 过滤条件: {:?}", filter_criteria);

        // 1. 获取可用的通道定义
        info!("🔥 [BATCH_ALLOCATION] 步骤1: 获取可用的通道定义");
        let available_definitions = self.get_available_definitions(filter_criteria).await?;
        info!("🔥 [BATCH_ALLOCATION] 从数据库查询到{}个通道定义", available_definitions.len());

        if available_definitions.is_empty() {
            error!("🔥 [BATCH_ALLOCATION] 错误: 没有可用的通道定义");
            return Err(AppError::validation_error("没有可用的通道定义"));
        }

        info!("找到{}个可用的通道定义", available_definitions.len());

        // 2. 根据策略分组通道
        let grouped_definitions = self.group_definitions_by_strategy(&available_definitions, &strategy);

        // 3. 创建测试批次信息
        let batch_info = self.create_batch_info(
            batch_name,
            product_model,
            operator_name.clone(),
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

        // 保存分配记录到数据库
        self.save_allocation_record(&batch_info.batch_id, &strategy, &allocation_summary, operator_name.as_deref()).await?;

        Ok(AllocationResult {
            batch_info,
            test_instances,
            allocation_summary,
        })
    }

    /// 创建测试批次并分配通道（使用完整的TestBatchInfo对象）
    ///
    /// # 参数
    /// * `batch_info` - 完整的批次信息对象
    /// * `strategy` - 分配策略
    /// * `filter_criteria` - 过滤条件（可选）
    ///
    /// # 返回
    /// * `Result<AllocationResult, AppError>` - 分配结果
    pub async fn create_test_batch_with_full_info(
        &self,
        mut batch_info: TestBatchInfo,
        strategy: AllocationStrategy,
        filter_criteria: Option<HashMap<String, String>>,
    ) -> Result<AllocationResult, AppError> {
        info!("🔥 [BATCH_ALLOCATION_FULL] 开始创建测试批次: {}", batch_info.batch_name);
        info!("🔥 [BATCH_ALLOCATION_FULL] 产品型号: {:?}", batch_info.product_model);
        info!("🔥 [BATCH_ALLOCATION_FULL] 站场名称: {:?}", batch_info.station_name);
        info!("🔥 [BATCH_ALLOCATION_FULL] 操作员: {:?}", batch_info.operator_name);
        info!("🔥 [BATCH_ALLOCATION_FULL] 分配策略: {:?}", strategy);

        // 1. 获取可用的通道定义
        info!("🔥 [BATCH_ALLOCATION_FULL] 步骤1: 获取可用的通道定义");
        let available_definitions = self.get_available_definitions(filter_criteria).await?;
        info!("🔥 [BATCH_ALLOCATION_FULL] 从数据库查询到{}个通道定义", available_definitions.len());

        if available_definitions.is_empty() {
            error!("🔥 [BATCH_ALLOCATION_FULL] 错误: 没有可用的通道定义");
            return Err(AppError::validation_error("没有可用的通道定义"));
        }

        // 2. 根据策略分组通道
        let grouped_definitions = self.group_definitions_by_strategy(&available_definitions, &strategy);

        // 3. 更新批次信息的统计数据
        batch_info.total_points = available_definitions.len() as u32;
        batch_info.last_updated_time = chrono::Utc::now();

        // 4. 如果站场名称为空，从第一个定义中获取
        if batch_info.station_name.is_none() {
            if let Some(first_def) = available_definitions.first() {
                batch_info.station_name = Some(first_def.station_name.clone());
                info!("🔥 [BATCH_ALLOCATION_FULL] 从通道定义中获取站场名称: {:?}", batch_info.station_name);
            }
        }

        // 5. 保存批次信息到数据库
        info!("🔥 [BATCH_ALLOCATION_FULL] 步骤2: 保存批次信息到数据库");
        let batch_entity: crate::models::entities::test_batch_info::ActiveModel = (&batch_info).into();
        let saved_batch = batch_entity.insert(&*self.db).await
            .map_err(|e| AppError::persistence_error(format!("保存批次信息失败: {}", e)))?;

        let final_batch_info: TestBatchInfo = (&saved_batch).into();
        info!("🔥 [BATCH_ALLOCATION_FULL] 批次信息已保存: ID={}, 站场={:?}",
              final_batch_info.batch_id, final_batch_info.station_name);

        // 6. 创建测试实例
        let test_instances = self.create_test_instances(
            &final_batch_info,
            &grouped_definitions,
        ).await?;

        // 7. 生成分配摘要
        let allocation_summary = self.generate_allocation_summary(&available_definitions);

        info!(
            "测试批次创建完成: {} - 总计{}个通道，预计测试时长{}分钟",
            final_batch_info.batch_name,
            allocation_summary.total_channels,
            allocation_summary.estimated_test_duration_minutes
        );

        // 保存分配记录到数据库
        self.save_allocation_record(&final_batch_info.batch_id, &strategy, &allocation_summary, final_batch_info.operator_name.as_deref()).await?;

        Ok(AllocationResult {
            batch_info: final_batch_info,
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
        if let Some(first_def) = definitions.first() {
            batch_info.station_name = Some(first_def.station_name.clone());
        }

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
                info!("🔧 [BATCH_ALLOCATION] 使用ChannelStateManager创建测试实例: {}", definition.tag);
                
                // 使用ChannelStateManager的initialize_channel_test_instance方法
                // 这确保了所有的跳过逻辑（YLDW和设定值策略）都会被正确应用
                let mut test_instance = self.channel_state_manager
                    .initialize_channel_test_instance(definition.clone(), batch_info.batch_id.clone())
                    .await
                    .map_err(|e| AppError::persistence_error(format!("初始化测试实例失败: {}", e)))?;
                
                test_instance.test_batch_name = batch_info.batch_name.clone();

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

    /// 保存批次分配记录
    async fn save_allocation_record(&self,
        batch_id: &str,
        strategy: &AllocationStrategy,
        summary: &AllocationSummary,
        operator_name: Option<&str>,
    ) -> Result<(), AppError> {
        let record_id = Uuid::new_v4().to_string();
        let summary_json = serde_json::to_string(summary)
            .map_err(|e| AppError::generic(format!("序列化分配摘要失败: {}", e)))?;
        let now = Utc::now().to_rfc3339();

        let sql = r#"INSERT INTO allocation_records (id, batch_id, strategy, summary_json, operator_name, created_time)
                     VALUES (?, ?, ?, ?, ?, ?)"#;

        self.db.execute(Statement::from_sql_and_values(
            sea_orm::DatabaseBackend::Sqlite,
            sql,
            vec![
                record_id.into(),
                batch_id.into(),
                format!("{:?}", strategy).into(),
                summary_json.into(),
                operator_name.unwrap_or("").into(),
                now.into(),
            ],
        ))
        .await
        .map_err(|e| AppError::persistence_error(format!("保存分配记录失败: {}", e)))?;

        Ok(())
    }
}
