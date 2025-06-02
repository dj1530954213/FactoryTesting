/// 数据导入服务
///
/// 负责从Excel文件导入通道点位定义数据到数据库
/// 支持批量导入和数据验证
use std::sync::Arc;
use sea_orm::{DatabaseConnection, EntityTrait, Set, ActiveModelTrait, QueryFilter, ColumnTrait, PaginatorTrait};
use crate::models::entities::channel_point_definition::{Entity as ChannelPointDefinitionEntity, ActiveModel as ChannelPointDefinitionActiveModel};
use crate::models::structs::ChannelPointDefinition;
use crate::services::infrastructure::excel::ExcelImporter;
use crate::error::AppError;
use log::{info, warn, error};

/// 数据导入结果
#[derive(Debug, Clone, serde::Serialize)]
pub struct ImportResult {
    pub total_rows: usize,
    pub successful_imports: usize,
    pub failed_imports: usize,
    pub errors: Vec<String>,
    pub imported_definitions: Vec<ChannelPointDefinition>,
}

impl ImportResult {
    pub fn new() -> Self {
        Self {
            total_rows: 0,
            successful_imports: 0,
            failed_imports: 0,
            errors: Vec::new(),
            imported_definitions: Vec::new(),
        }
    }

    pub fn add_success(&mut self, definition: ChannelPointDefinition) {
        self.successful_imports += 1;
        self.imported_definitions.push(definition);
    }

    pub fn add_error(&mut self, error: String) {
        self.failed_imports += 1;
        self.errors.push(error);
    }

    pub fn is_successful(&self) -> bool {
        self.failed_imports == 0 && self.successful_imports > 0
    }

    pub fn success_rate(&self) -> f32 {
        if self.total_rows == 0 {
            0.0
        } else {
            (self.successful_imports as f32 / self.total_rows as f32) * 100.0
        }
    }
}

/// 数据导入服务
pub struct DataImportService {
    db: Arc<DatabaseConnection>,
}

impl DataImportService {
    /// 创建新的数据导入服务实例
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }

    /// 从Excel文件导入通道点位定义
    ///
    /// # 参数
    /// * `file_path` - Excel文件路径
    /// * `replace_existing` - 是否替换已存在的数据（如果为true，会先清空所有数据）
    ///
    /// # 返回
    /// * `Result<ImportResult, AppError>` - 导入结果
    pub async fn import_from_excel(
        &self,
        file_path: &str,
        replace_existing: bool,
    ) -> Result<ImportResult, AppError> {
        info!("开始从Excel文件导入数据: {}", file_path);

        let mut result = ImportResult::new();

        // 1. 如果需要替换现有数据，先清空数据库
        if replace_existing {
            info!("清空现有通道定义数据...");
            match self.clear_all_data().await {
                Ok(deleted_count) => {
                    info!("成功清空{}条现有数据", deleted_count);
                }
                Err(e) => {
                    error!("清空现有数据失败: {:?}", e);
                    return Err(e);
                }
            }
        }

        // 2. 解析Excel文件
        let definitions = match ExcelImporter::parse_excel_file(file_path).await {
            Ok(defs) => defs,
            Err(e) => {
                error!("解析Excel文件失败: {:?}", e);
                return Err(e);
            }
        };

        result.total_rows = definitions.len();
        info!("从Excel文件解析出{}个通道定义", definitions.len());

        // 3. 验证数据
        let validated_definitions = self.validate_definitions(definitions, &mut result).await?;

        // 4. 导入到数据库
        self.import_to_database(validated_definitions, false, &mut result).await?; // 由于已经清空，这里不需要再检查重复

        info!(
            "数据导入完成: 总计{}行，成功{}行，失败{}行，成功率{:.1}%",
            result.total_rows,
            result.successful_imports,
            result.failed_imports,
            result.success_rate()
        );

        Ok(result)
    }

    /// 验证通道定义数据
    async fn validate_definitions(
        &self,
        definitions: Vec<ChannelPointDefinition>,
        result: &mut ImportResult,
    ) -> Result<Vec<ChannelPointDefinition>, AppError> {
        info!("开始验证通道定义数据...");

        let mut validated = Vec::new();
        let mut tag_set = std::collections::HashSet::new();

        for definition in definitions {
            // 检查必填字段
            if definition.tag.is_empty() {
                result.add_error(format!("通道标识不能为空"));
                continue;
            }

            if definition.variable_name.is_empty() {
                result.add_error(format!("变量名称不能为空: {}", definition.tag));
                continue;
            }

            if definition.plc_communication_address.is_empty() {
                result.add_error(format!("PLC通信地址不能为空: {}", definition.tag));
                continue;
            }

            // 检查重复的标识
            if tag_set.contains(&definition.tag) {
                result.add_error(format!("重复的通道标识: {}", definition.tag));
                continue;
            }
            tag_set.insert(definition.tag.clone());

            // 检查数据库中是否已存在
            let existing = ChannelPointDefinitionEntity::find()
                .filter(crate::models::entities::channel_point_definition::Column::Tag.eq(&definition.tag))
                .one(&*self.db)
                .await
                .map_err(|e| AppError::persistence_error(format!("查询数据库失败: {}", e)))?;

            if existing.is_some() {
                warn!("通道标识已存在: {}", definition.tag);
                // 这里不算错误，后续根据replace_existing参数决定是否替换
            }

            validated.push(definition);
        }

        info!("数据验证完成，有效数据{}条", validated.len());
        Ok(validated)
    }

    /// 导入数据到数据库
    async fn import_to_database(
        &self,
        definitions: Vec<ChannelPointDefinition>,
        replace_existing: bool,
        result: &mut ImportResult,
    ) -> Result<(), AppError> {
        info!("开始导入数据到数据库...");

        for definition in definitions {
            match self.import_single_definition(&definition, replace_existing).await {
                Ok(_) => {
                    result.add_success(definition);
                }
                Err(e) => {
                    error!("导入通道定义失败: {} - {:?}", definition.tag, e);
                    result.add_error(format!("导入失败 {}: {}", definition.tag, e));
                }
            }
        }

        Ok(())
    }

    /// 导入单个通道定义
    async fn import_single_definition(
        &self,
        definition: &ChannelPointDefinition,
        replace_existing: bool,
    ) -> Result<(), AppError> {
        // 检查是否已存在
        let existing = ChannelPointDefinitionEntity::find()
            .filter(crate::models::entities::channel_point_definition::Column::Tag.eq(&definition.tag))
            .one(&*self.db)
            .await
            .map_err(|e| AppError::persistence_error(format!("查询数据库失败: {}", e)))?;

        if let Some(existing_model) = existing {
            if replace_existing {
                // 更新现有记录
                let mut active_model: ChannelPointDefinitionActiveModel = existing_model.into();

                // 更新字段
                active_model.variable_name = Set(definition.variable_name.clone());
                active_model.variable_description = Set(Some(definition.variable_description.clone()));
                active_model.station_name = Set(Some(definition.station_name.clone()));
                active_model.module_name = Set(Some(definition.module_name.clone()));
                active_model.module_type = Set(definition.module_type.to_string());
                active_model.channel_tag_in_module = Set(definition.channel_tag_in_module.clone());
                active_model.data_type = Set(Some(definition.data_type.to_string()));
                active_model.power_supply_type = Set(match definition.power_supply_type.as_str() {
                    "有源" => 1,
                    "无源" => 0,
                    _ => 1,
                });
                active_model.wire_system = Set(Some(definition.wire_system.clone()));
                active_model.plc_communication_address = Set(definition.plc_communication_address.clone());

                // 更新可选字段
                if let Some(ref addr) = definition.plc_absolute_address {
                    active_model.plc_absolute_address = Set(Some(addr.clone()));
                }
                if let Some(ref prop) = definition.access_property {
                    active_model.read_write_property = Set(Some(prop.clone()));
                }

                active_model.update(&*self.db).await
                    .map_err(|e| AppError::persistence_error(format!("更新数据库失败: {}", e)))?;

                info!("更新通道定义: {}", definition.tag);
            } else {
                return Err(AppError::validation_error(format!("通道标识已存在: {}", definition.tag)));
            }
        } else {
            // 插入新记录
            let active_model: ChannelPointDefinitionActiveModel = definition.into();

            active_model.insert(&*self.db).await
                .map_err(|e| AppError::persistence_error(format!("插入数据库失败: {}", e)))?;

            info!("插入新通道定义: {}", definition.tag);
        }

        Ok(())
    }

    /// 获取数据库中的通道定义总数
    pub async fn get_total_count(&self) -> Result<u64, AppError> {
        let count = ChannelPointDefinitionEntity::find()
            .count(&*self.db)
            .await
            .map_err(|e| AppError::persistence_error(format!("查询数据库失败: {}", e)))?;

        Ok(count)
    }

    /// 清空所有通道定义数据
    pub async fn clear_all_data(&self) -> Result<u64, AppError> {
        let result = ChannelPointDefinitionEntity::delete_many()
            .exec(&*self.db)
            .await
            .map_err(|e| AppError::persistence_error(format!("删除数据失败: {}", e)))?;

        info!("清空通道定义数据，删除{}条记录", result.rows_affected);
        Ok(result.rows_affected)
    }
}
