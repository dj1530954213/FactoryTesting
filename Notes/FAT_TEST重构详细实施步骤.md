# FAT_TEST 系统重构详细实施步骤

## 📋 重构实施概览

### 🎯 实施原则
1. **零停机重构**: 通过分阶段实施，确保系统持续可用
2. **向后兼容**: 新接口兼容现有功能，逐步替换旧实现
3. **测试驱动**: 每个步骤都有完整的测试覆盖
4. **持续验证**: 每个阶段完成后进行集成测试

### 🗓️ 实施计划 (4个阶段)

#### Phase 1: 数据访问层重构 (Repository Layer)
- **目标**: 建立统一的数据访问接口
- **工期**: 1-2周
- **关键产出**: Repository接口和实现类

#### Phase 2: 状态管理器重构 (State Management)
- **目标**: 建立严格的状态管理机制
- **工期**: 1-2周
- **关键产出**: 增强的ChannelStateManager

#### Phase 3: 任务调度器重构 (Task Scheduling)
- **目标**: 优化测试任务管理和调度
- **工期**: 2-3周
- **关键产出**: 新的TaskScheduler和TestExecutor

#### Phase 4: 应用服务层重构 (Application Services)
- **目标**: 重构应用服务，实现服务组合模式
- **工期**: 1-2周
- **关键产出**: 重构的应用服务

---

## 🚀 Phase 1: 数据访问层重构

### 步骤 1.1: 创建Repository基础接口

#### 🎯 目标
建立统一的数据访问抽象，为所有数据操作提供一致的接口。

#### 📝 具体实施

##### 1.1.1 创建基础Repository接口
```rust
// src/repositories/mod.rs
use async_trait::async_trait;
use serde::{Serialize, de::DeserializeOwned};
use std::fmt::Debug;
use crate::models::errors::RepositoryError;

/// 基础Repository接口
#[async_trait]
pub trait IRepository<T, K>: Send + Sync 
where 
    T: Send + Sync + Clone + Debug,
    K: Send + Sync + Clone + Debug,
{
    /// 根据键获取实体
    async fn get(&self, key: &K) -> Result<Option<T>, RepositoryError>;
    
    /// 保存实体
    async fn save(&self, entity: &T) -> Result<(), RepositoryError>;
    
    /// 删除实体
    async fn delete(&self, key: &K) -> Result<bool, RepositoryError>;
    
    /// 检查实体是否存在
    async fn exists(&self, key: &K) -> Result<bool, RepositoryError>;
    
    /// 获取所有实体
    async fn list_all(&self) -> Result<Vec<T>, RepositoryError>;
    
    /// 根据条件查询
    async fn query(&self, criteria: &QueryCriteria) -> Result<Vec<T>, RepositoryError>;
}

/// 查询条件
#[derive(Debug, Clone)]
pub struct QueryCriteria {
    pub filters: Vec<Filter>,
    pub sort_by: Option<String>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

/// 过滤条件
#[derive(Debug, Clone)]
pub struct Filter {
    pub field: String,
    pub operator: FilterOperator,
    pub value: FilterValue,
}

#[derive(Debug, Clone)]
pub enum FilterOperator {
    Equal,
    NotEqual,
    GreaterThan,
    LessThan,
    Contains,
    In,
}

#[derive(Debug, Clone)]
pub enum FilterValue {
    String(String),
    Number(f64),
    Boolean(bool),
    List(Vec<String>),
}
```

##### 1.1.2 创建Repository错误类型
```rust
// src/models/errors.rs (扩展现有错误类型)
use thiserror::Error;
use serde::Serialize;

#[derive(Error, Debug, Clone, Serialize)]
pub enum RepositoryError {
    #[error("Entity not found: {0}")]
    NotFound(String),
    
    #[error("Serialization error: {0}")]
    SerializationError(String),
    
    #[error("Deserialization error: {0}")]
    DeserializationError(String),
    
    #[error("Database error: {0}")]
    DatabaseError(String),
    
    #[error("Concurrency conflict: {0}")]
    ConcurrencyError(String),
    
    #[error("Validation error: {0}")]
    ValidationError(String),
    
    #[error("Access denied: {0}")]
    AccessDenied(String),
    
    #[error("Internal error: {0}")]
    InternalError(String),
}

impl From<std::io::Error> for RepositoryError {
    fn from(err: std::io::Error) -> Self {
        RepositoryError::InternalError(err.to_string())
    }
}

impl From<serde_json::Error> for RepositoryError {
    fn from(err: serde_json::Error) -> Self {
        RepositoryError::SerializationError(err.to_string())
    }
}
```

#### 🧪 测试方案

##### 1.1.3 创建Repository测试框架
```rust
// src/repositories/tests/mod.rs
use super::*;
use tokio;
use uuid::Uuid;

/// Repository测试用例基础trait
#[async_trait]
pub trait RepositoryTestSuite<T, K, R>
where
    T: Send + Sync + Clone + Debug + PartialEq,
    K: Send + Sync + Clone + Debug,
    R: IRepository<T, K>,
{
    /// 创建测试用Repository实例
    async fn create_repository() -> R;
    
    /// 创建测试实体
    fn create_test_entity() -> T;
    
    /// 获取实体的键
    fn get_entity_key(entity: &T) -> K;
    
    /// 基础CRUD测试
    async fn test_crud_operations() {
        let repo = Self::create_repository().await;
        let entity = Self::create_test_entity();
        let key = Self::get_entity_key(&entity);
        
        // 测试保存
        repo.save(&entity).await.expect("保存失败");
        
        // 测试获取
        let retrieved = repo.get(&key).await.expect("获取失败");
        assert!(retrieved.is_some(), "实体应该存在");
        assert_eq!(retrieved.unwrap(), entity, "实体数据应该一致");
        
        // 测试存在性检查
        let exists = repo.exists(&key).await.expect("存在性检查失败");
        assert!(exists, "实体应该存在");
        
        // 测试删除
        let deleted = repo.delete(&key).await.expect("删除失败");
        assert!(deleted, "应该删除成功");
        
        // 验证删除
        let exists_after_delete = repo.exists(&key).await.expect("删除后存在性检查失败");
        assert!(!exists_after_delete, "实体应该不存在");
    }
    
    /// 查询功能测试
    async fn test_query_operations() {
        let repo = Self::create_repository().await;
        
        // 创建多个测试实体
        let entities = (0..5).map(|_| Self::create_test_entity()).collect::<Vec<_>>();
        
        // 保存所有实体
        for entity in &entities {
            repo.save(entity).await.expect("批量保存失败");
        }
        
        // 测试列表查询
        let all_entities = repo.list_all().await.expect("列表查询失败");
        assert!(all_entities.len() >= entities.len(), "应该包含所有保存的实体");
        
        // 测试条件查询
        let criteria = QueryCriteria {
            filters: vec![],
            sort_by: None,
            limit: Some(3),
            offset: None,
        };
        let limited_results = repo.query(&criteria).await.expect("条件查询失败");
        assert!(limited_results.len() <= 3, "限制查询结果数量");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_repository_interface() {
        // 测试将在具体Repository实现中进行
    }
}
```

### 步骤 1.2: 创建配置数据Repository

#### 🎯 目标
实现专门管理配置数据的Repository，确保配置数据的一致性和完整性。

#### 📝 具体实施

##### 1.2.1 定义配置Repository接口
```rust
// src/repositories/configuration_repository.rs
use async_trait::async_trait;
use crate::models::structs::{ChannelPointDefinition, TestParameterSet};
use crate::models::enums::ModuleType;
use crate::models::errors::RepositoryError;
use super::{IRepository, QueryCriteria};

/// 配置数据Repository接口
#[async_trait]
pub trait IConfigurationRepository: Send + Sync {
    // 点位定义管理
    async fn get_channel_definition(&self, id: &str) -> Result<Option<ChannelPointDefinition>, RepositoryError>;
    async fn save_channel_definition(&self, definition: &ChannelPointDefinition) -> Result<(), RepositoryError>;
    async fn delete_channel_definition(&self, id: &str) -> Result<bool, RepositoryError>;
    async fn list_channel_definitions(&self) -> Result<Vec<ChannelPointDefinition>, RepositoryError>;
    async fn query_channel_definitions(&self, criteria: &QueryCriteria) -> Result<Vec<ChannelPointDefinition>, RepositoryError>;
    
    // 批量操作
    async fn save_channel_definitions_batch(&self, definitions: &[ChannelPointDefinition]) -> Result<(), RepositoryError>;
    async fn import_from_excel(&self, file_path: &str, config_name: &str) -> Result<Vec<ChannelPointDefinition>, RepositoryError>;
    async fn export_to_excel(&self, definitions: &[ChannelPointDefinition], file_path: &str) -> Result<(), RepositoryError>;
    
    // 配置集管理
    async fn save_configuration_set(&self, name: &str, definitions: &[ChannelPointDefinition]) -> Result<(), RepositoryError>;
    async fn load_configuration_set(&self, name: &str) -> Result<Vec<ChannelPointDefinition>, RepositoryError>;
    async fn list_configuration_sets(&self) -> Result<Vec<String>, RepositoryError>;
    async fn delete_configuration_set(&self, name: &str) -> Result<bool, RepositoryError>;
    
    // 测试参数管理
    async fn get_test_parameters(&self, module_type: ModuleType) -> Result<Option<TestParameterSet>, RepositoryError>;
    async fn save_test_parameters(&self, module_type: ModuleType, params: &TestParameterSet) -> Result<(), RepositoryError>;
    async fn list_test_parameters(&self) -> Result<Vec<(ModuleType, TestParameterSet)>, RepositoryError>;
    
    // 模板管理
    async fn save_definition_template(&self, template_name: &str, template: &ChannelPointDefinition) -> Result<(), RepositoryError>;
    async fn get_definition_template(&self, template_name: &str) -> Result<Option<ChannelPointDefinition>, RepositoryError>;
    async fn list_definition_templates(&self) -> Result<Vec<String>, RepositoryError>;
    
    // 验证和一致性检查
    async fn validate_definitions(&self, definitions: &[ChannelPointDefinition]) -> Result<Vec<ValidationIssue>, RepositoryError>;
    async fn check_consistency(&self) -> Result<ConsistencyReport, RepositoryError>;
}

/// 验证问题
#[derive(Debug, Clone)]
pub struct ValidationIssue {
    pub severity: ValidationSeverity,
    pub message: String,
    pub field: Option<String>,
    pub suggestion: Option<String>,
}

#[derive(Debug, Clone)]
pub enum ValidationSeverity {
    Error,
    Warning,
    Info,
}

/// 一致性报告
#[derive(Debug, Clone)]
pub struct ConsistencyReport {
    pub total_definitions: usize,
    pub duplicates: Vec<String>,
    pub orphaned_references: Vec<String>,
    pub missing_parameters: Vec<String>,
    pub issues: Vec<ValidationIssue>,
}
```

##### 1.2.2 实现内存配置Repository
```rust
// src/repositories/configuration_repository.rs (continued)
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// 内存配置Repository实现
pub struct MemoryConfigurationRepository {
    definitions: Arc<RwLock<HashMap<String, ChannelPointDefinition>>>,
    configuration_sets: Arc<RwLock<HashMap<String, Vec<String>>>>, // name -> definition_ids
    test_parameters: Arc<RwLock<HashMap<ModuleType, TestParameterSet>>>,
    templates: Arc<RwLock<HashMap<String, ChannelPointDefinition>>>,
}

impl MemoryConfigurationRepository {
    pub fn new() -> Self {
        Self {
            definitions: Arc::new(RwLock::new(HashMap::new())),
            configuration_sets: Arc::new(RwLock::new(HashMap::new())),
            test_parameters: Arc::new(RwLock::new(HashMap::new())),
            templates: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// 创建预填充测试数据的实例
    pub async fn new_with_test_data() -> Self {
        let repo = Self::new();
        repo.populate_test_data().await;
        repo
    }
    
    async fn populate_test_data(&self) {
        // 添加默认测试参数
        let ai_params = TestParameterSet {
            default_range: Some((0.0, 100.0)),
            test_points: vec![0.0, 25.0, 50.0, 75.0, 100.0],
            tolerance: 1.0,
            test_sequence: vec![
                SubTestItem::HardPoint,
                SubTestItem::LowAlarm,
                SubTestItem::HighAlarm,
            ],
        };
        
        let mut params = self.test_parameters.write().await;
        params.insert(ModuleType::AI, ai_params);
    }
}

#[async_trait]
impl IConfigurationRepository for MemoryConfigurationRepository {
    async fn get_channel_definition(&self, id: &str) -> Result<Option<ChannelPointDefinition>, RepositoryError> {
        let definitions = self.definitions.read().await;
        Ok(definitions.get(id).cloned())
    }
    
    async fn save_channel_definition(&self, definition: &ChannelPointDefinition) -> Result<(), RepositoryError> {
        let mut definitions = self.definitions.write().await;
        definitions.insert(definition.id.clone(), definition.clone());
        Ok(())
    }
    
    async fn delete_channel_definition(&self, id: &str) -> Result<bool, RepositoryError> {
        let mut definitions = self.definitions.write().await;
        Ok(definitions.remove(id).is_some())
    }
    
    async fn list_channel_definitions(&self) -> Result<Vec<ChannelPointDefinition>, RepositoryError> {
        let definitions = self.definitions.read().await;
        Ok(definitions.values().cloned().collect())
    }
    
    async fn query_channel_definitions(&self, criteria: &QueryCriteria) -> Result<Vec<ChannelPointDefinition>, RepositoryError> {
        let definitions = self.definitions.read().await;
        let mut results: Vec<ChannelPointDefinition> = definitions.values().cloned().collect();
        
        // 应用过滤器
        for filter in &criteria.filters {
            results = self.apply_filter(results, filter)?;
        }
        
        // 应用排序
        if let Some(sort_field) = &criteria.sort_by {
            self.sort_definitions(&mut results, sort_field);
        }
        
        // 应用分页
        if let Some(offset) = criteria.offset {
            results = results.into_iter().skip(offset).collect();
        }
        
        if let Some(limit) = criteria.limit {
            results.truncate(limit);
        }
        
        Ok(results)
    }
    
    async fn save_channel_definitions_batch(&self, definitions: &[ChannelPointDefinition]) -> Result<(), RepositoryError> {
        let mut def_map = self.definitions.write().await;
        for definition in definitions {
            def_map.insert(definition.id.clone(), definition.clone());
        }
        Ok(())
    }
    
    async fn import_from_excel(&self, file_path: &str, config_name: &str) -> Result<Vec<ChannelPointDefinition>, RepositoryError> {
        use calamine::{Reader, Xlsx, open_workbook};
        
        // 打开Excel工作簿
        let mut workbook: Xlsx<_> = open_workbook(file_path)
            .map_err(|e| RepositoryError::DeserializationError(format!("无法打开Excel文件: {}", e)))?;
        
        // 获取第一个工作表
        let worksheet_name = workbook.sheet_names().get(0)
            .ok_or_else(|| RepositoryError::DeserializationError("Excel文件没有工作表".to_string()))?
            .clone();
        
        let range = workbook.worksheet_range(&worksheet_name)
            .map_err(|e| RepositoryError::DeserializationError(format!("无法读取工作表: {}", e)))?;
        
        let mut definitions = Vec::new();
        
        // 跳过表头，从第二行开始解析
        for (row_index, row) in range.rows().enumerate().skip(1) {
            if row.is_empty() || row.len() < 52 {
                continue;
            }
            
            // 解析每一行的数据，构建ChannelPointDefinition
            let definition = self.parse_excel_row(row, row_index + 2)?; // +2是因为Excel行号从1开始，我们跳过了表头
            definitions.push(definition);
        }
        
        // 验证导入的数据
        let validation_issues = self.validate_definitions(&definitions).await?;
        if validation_issues.iter().any(|issue| matches!(issue.severity, ValidationSeverity::Error)) {
            return Err(RepositoryError::ValidationError(
                format!("导入的数据存在错误，请检查Excel文件格式")
            ));
        }
        
        // 保存为配置集
        if !definitions.is_empty() {
            self.save_configuration_set(config_name, &definitions).await?;
        }
        
        log::info!("从Excel导入配置完成: 文件={}, 配置集={}, 导入数量={}", file_path, config_name, definitions.len());
        Ok(definitions)
    }
    
    /// 解析Excel行数据为ChannelPointDefinition
    fn parse_excel_row(&self, row: &[calamine::DataType], row_number: usize) -> Result<ChannelPointDefinition, RepositoryError> {
        use calamine::DataType;
        
        // 定义Excel列的顺序映射（基于实际表头）
        let get_cell_string = |index: usize| -> String {
            if index < row.len() {
                match &row[index] {
                    DataType::String(s) => s.trim().to_string(),
                    DataType::Float(f) => f.to_string(),
                    DataType::Int(i) => i.to_string(),
                    DataType::Bool(b) => b.to_string(),
                    _ => String::new(),
                }
            } else {
                String::new()
            }
        };
        
        let get_cell_float = |index: usize| -> Option<f64> {
            if index < row.len() {
                match &row[index] {
                    DataType::Float(f) => Some(*f),
                    DataType::Int(i) => Some(*i as f64),
                    DataType::String(s) => {
                        let s = s.trim();
                        if s.is_empty() || s == "/" {
                            None
                        } else {
                            s.parse().ok()
                        }
                    },
                    _ => None,
                }
            } else {
                None
            }
        };
        
        let get_cell_bool = |index: usize| -> Option<bool> {
            if index < row.len() {
                match &row[index] {
                    DataType::Bool(b) => Some(*b),
                    DataType::String(s) => {
                        let s = s.trim();
                        match s {
                            "是" | "true" | "True" | "TRUE" | "1" => Some(true),
                            "否" | "false" | "False" | "FALSE" | "0" => Some(false),
                            _ => None,
                        }
                    },
                    DataType::Int(i) => Some(*i != 0),
                    _ => None,
                }
            } else {
                None
            }
        };
        
        let get_optional_string = |index: usize| -> Option<String> {
            let s = get_cell_string(index);
            if s.is_empty() || s == "/" {
                None
            } else {
                Some(s)
            }
        };
        
        // 解析模块类型
        let module_type_str = get_cell_string(2); // 第2列：模块类型
        let module_type = module_type_str.parse::<ModuleType>()
            .map_err(|_| RepositoryError::ValidationError(
                format!("第{}行: 无效的模块类型 '{}'", row_number, module_type_str)
            ))?;
        
        // 解析数据类型
        let data_type_str = get_cell_string(10); // 第10列：数据类型
        let data_type = data_type_str.parse::<PointDataType>()
            .map_err(|_| RepositoryError::ValidationError(
                format!("第{}行: 无效的数据类型 '{}'", row_number, data_type_str)
            ))?;
        
        // 构建定义对象（根据实际Excel列索引）
        let definition = ChannelPointDefinition {
            id: Uuid::new_v4().to_string(),
            tag: get_cell_string(6),                    // 第6列：位号
            variable_name: get_cell_string(8),          // 第8列：变量名称（HMI）
            variable_description: get_cell_string(9),   // 第9列：变量描述
            station_name: get_cell_string(7),           // 第7列：场站名
            module_name: get_cell_string(1),            // 第1列：模块名称
            module_type,                                 // 第2列：模块类型
            channel_tag_in_module: get_cell_string(5),  // 第5列：通道位号
            data_type,                                   // 第10列：数据类型
            power_supply_type: get_cell_string(3),      // 第3列：供电类型（有源/无源）
            wire_system: get_cell_string(4),            // 第4列：线制
            plc_absolute_address: get_optional_string(51), // 第51列：PLC绝对地址
            plc_communication_address: get_cell_string(52), // 第52列：上位机通讯地址
            range_lower_limit: get_cell_float(14),      // 第14列：量程低限
            range_upper_limit: get_cell_float(15),      // 第15列：量程高限
            engineering_unit: None, // Excel中没有对应列
            sll_set_value: get_cell_float(16),          // 第16列：SLL设定值
            sll_set_point_address: get_optional_string(17), // 第17列：SLL设定点位
            sll_feedback_address: get_optional_string(18), // 第18列：SLL设定点位_PLC地址
            sl_set_value: get_cell_float(20),           // 第20列：SL设定值
            sl_set_point_address: get_optional_string(21), // 第21列：SL设定点位
            sl_feedback_address: get_optional_string(22),  // 第22列：SL设定点位_PLC地址
            sh_set_value: get_cell_float(24),           // 第24列：SH设定值
            sh_set_point_address: get_optional_string(25), // 第25列：SH设定点位
            sh_feedback_address: get_optional_string(26),  // 第26列：SH设定点位_PLC地址
            shh_set_value: get_cell_float(28),          // 第28列：SHH设定值
            shh_set_point_address: get_optional_string(29), // 第29列：SHH设定点位
            shh_feedback_address: get_optional_string(30),  // 第30列：SHH设定点位_PLC地址
            maintenance_value_set_point_address: get_optional_string(46), // 第46列：维护值设定点位_PLC地址
            maintenance_enable_switch_point_address: get_optional_string(49), // 第49列：维护使能开关点位_PLC地址
            access_property: get_optional_string(11),   // 第11列：读写属性
            save_history: get_cell_bool(12),            // 第12列：保存历史
            power_failure_protection: get_cell_bool(13), // 第13列：掉电保护
            test_rig_plc_address: None, // Excel中没有对应列，可以后续添加
        };
        
        Ok(definition)
    }
    
    async fn export_to_excel(&self, definitions: &[ChannelPointDefinition], file_path: &str) -> Result<(), RepositoryError> {
        use rust_xlsxwriter::{Workbook, Worksheet, Format};
        
        // 创建工作簿
        let mut workbook = Workbook::new();
        let mut worksheet = workbook.add_worksheet();
        
        // 设置表头格式
        let header_format = Format::new()
            .set_bold()
            .set_background_color("#E0E0E0")
            .set_border(rust_xlsxwriter::FormatBorder::Thin);
        
        // 定义表头（基于实际Excel格式）
        let headers = vec![
            "序号", "模块名称", "模块类型", "供电类型（有源/无源）", "线制", 
            "通道位号", "位号", "场站名", "变量名称（HMI）", "变量描述", 
            "数据类型", "读写属性", "保存历史", "掉电保护", "量程低限", "量程高限",
            "SLL设定值", "SLL设定点位", "SLL设定点位_PLC地址", "SLL设定点位_通讯地址",
            "SL设定值", "SL设定点位", "SL设定点位_PLC地址", "SL设定点位_通讯地址",
            "SH设定值", "SH设定点位", "SH设定点位_PLC地址", "SH设定点位_通讯地址",
            "SHH设定值", "SHH设定点位", "SHH设定点位_PLC地址", "SHH设定点位_通讯地址",
            "LL报警", "LL报警_PLC地址", "LL报警_通讯地址", "L报警", "L报警_PLC地址", "L报警_通讯地址",
            "H报警", "H报警_PLC地址", "H报警_通讯地址", "HH报警", "HH报警_PLC地址", "HH报警_通讯地址",
            "维护值设定", "维护值设定点位", "维护值设定点位_PLC地址", "维护值设定点位_通讯地址",
            "维护使能开关点位", "维护使能开关点位_PLC地址", "维护使能开关点位_通讯地址",
            "PLC绝对地址", "上位机通讯地址"
        ];
        
        // 写入表头
        for (col_index, header) in headers.iter().enumerate() {
            worksheet.write_string_with_format(0, col_index as u16, header, &header_format)
                .map_err(|e| RepositoryError::SerializationError(format!("写入表头失败: {}", e)))?;
        }
        
        // 写入数据行
        for (row_index, definition) in definitions.iter().enumerate() {
            let row = (row_index + 1) as u32; // +1 跳过表头
            
            // 写入每列数据（按照实际Excel格式）
            let row_data = vec![
                (row_index + 1).to_string(),                                    // 0: 序号
                definition.module_name.clone(),                                 // 1: 模块名称
                format!("{:?}", definition.module_type),                        // 2: 模块类型
                definition.power_supply_type.clone(),                           // 3: 供电类型（有源/无源）
                definition.wire_system.clone(),                                 // 4: 线制
                definition.channel_tag_in_module.clone(),                       // 5: 通道位号
                definition.tag.clone(),                                         // 6: 位号
                definition.station_name.clone(),                                // 7: 场站名
                definition.variable_name.clone(),                               // 8: 变量名称（HMI）
                definition.variable_description.clone(),                        // 9: 变量描述
                format!("{:?}", definition.data_type),                          // 10: 数据类型
                definition.access_property.clone().unwrap_or_default(),         // 11: 读写属性
                definition.save_history.map_or(String::new(), |v| if v { "是".to_string() } else { "否".to_string() }), // 12: 保存历史
                definition.power_failure_protection.map_or(String::new(), |v| if v { "是".to_string() } else { "否".to_string() }), // 13: 掉电保护
                definition.range_lower_limit.map_or(String::new(), |v| v.to_string()), // 14: 量程低限
                definition.range_upper_limit.map_or(String::new(), |v| v.to_string()), // 15: 量程高限
                definition.sll_set_value.map_or(String::new(), |v| v.to_string()),     // 16: SLL设定值
                definition.sll_set_point_address.clone().unwrap_or_default(),   // 17: SLL设定点位
                definition.sll_feedback_address.clone().unwrap_or_default(),    // 18: SLL设定点位_PLC地址
                "".to_string(), // 19: SLL设定点位_通讯地址（预留）
                definition.sl_set_value.map_or(String::new(), |v| v.to_string()),      // 20: SL设定值
                definition.sl_set_point_address.clone().unwrap_or_default(),    // 21: SL设定点位
                definition.sl_feedback_address.clone().unwrap_or_default(),     // 22: SL设定点位_PLC地址
                "".to_string(), // 23: SL设定点位_通讯地址（预留）
                definition.sh_set_value.map_or(String::new(), |v| v.to_string()),      // 24: SH设定值
                definition.sh_set_point_address.clone().unwrap_or_default(),    // 25: SH设定点位
                definition.sh_feedback_address.clone().unwrap_or_default(),     // 26: SH设定点位_PLC地址
                "".to_string(), // 27: SH设定点位_通讯地址（预留）
                definition.shh_set_value.map_or(String::new(), |v| v.to_string()),     // 28: SHH设定值
                definition.shh_set_point_address.clone().unwrap_or_default(),   // 29: SHH设定点位
                definition.shh_feedback_address.clone().unwrap_or_default(),    // 30: SHH设定点位_PLC地址
                "".to_string(), // 31: SHH设定点位_通讯地址（预留）
                "".to_string(), // 32: LL报警（预留）
                "".to_string(), // 33: LL报警_PLC地址（预留）
                "".to_string(), // 34: LL报警_通讯地址（预留）
                "".to_string(), // 35: L报警（预留）
                "".to_string(), // 36: L报警_PLC地址（预留）
                "".to_string(), // 37: L报警_通讯地址（预留）
                "".to_string(), // 38: H报警（预留）
                "".to_string(), // 39: H报警_PLC地址（预留）
                "".to_string(), // 40: H报警_通讯地址（预留）
                "".to_string(), // 41: HH报警（预留）
                "".to_string(), // 42: HH报警_PLC地址（预留）
                "".to_string(), // 43: HH报警_通讯地址（预留）
                "".to_string(), // 44: 维护值设定（预留）
                "".to_string(), // 45: 维护值设定点位（预留）
                definition.maintenance_value_set_point_address.clone().unwrap_or_default(), // 46: 维护值设定点位_PLC地址
                "".to_string(), // 47: 维护值设定点位_通讯地址（预留）
                "".to_string(), // 48: 维护使能开关点位（预留）
                definition.maintenance_enable_switch_point_address.clone().unwrap_or_default(), // 49: 维护使能开关点位_PLC地址
                "".to_string(), // 50: 维护使能开关点位_通讯地址（预留）
                definition.plc_absolute_address.clone().unwrap_or_default(),    // 51: PLC绝对地址
                definition.plc_communication_address.clone(),                   // 52: 上位机通讯地址
            ];
            
            for (col_index, cell_value) in row_data.iter().enumerate() {
                worksheet.write_string(row, col_index as u16, cell_value)
                    .map_err(|e| RepositoryError::SerializationError(format!("写入数据失败: {}", e)))?;
            }
        }
        
        // 自动调整列宽
        for col_index in 0..headers.len() {
            worksheet.autofit();
        }
        
        // 保存文件
        workbook.save(file_path)
            .map_err(|e| RepositoryError::SerializationError(format!("保存Excel文件失败: {}", e)))?;
        
        log::info!("Excel导出完成: 文件={}, 导出数量={}", file_path, definitions.len());
        Ok(())
     }
    
    async fn save_configuration_set(&self, name: &str, definitions: &[ChannelPointDefinition]) -> Result<(), RepositoryError> {
        // 先保存所有定义
        self.save_channel_definitions_batch(definitions).await?;
        
        // 然后保存配置集
        let definition_ids: Vec<String> = definitions.iter().map(|d| d.id.clone()).collect();
        let mut sets = self.configuration_sets.write().await;
        sets.insert(name.to_string(), definition_ids);
        Ok(())
    }
    
    async fn load_configuration_set(&self, name: &str) -> Result<Vec<ChannelPointDefinition>, RepositoryError> {
        let sets = self.configuration_sets.read().await;
        let definition_ids = sets.get(name)
            .ok_or_else(|| RepositoryError::NotFound(format!("配置集不存在: {}", name)))?;
        
        let definitions = self.definitions.read().await;
        let mut results = Vec::new();
        
        for id in definition_ids {
            if let Some(definition) = definitions.get(id) {
                results.push(definition.clone());
            }
        }
        
        Ok(results)
    }
    
    async fn list_configuration_sets(&self) -> Result<Vec<String>, RepositoryError> {
        let sets = self.configuration_sets.read().await;
        Ok(sets.keys().cloned().collect())
    }
    
    async fn delete_configuration_set(&self, name: &str) -> Result<bool, RepositoryError> {
        let mut sets = self.configuration_sets.write().await;
        Ok(sets.remove(name).is_some())
    }
    
    async fn get_test_parameters(&self, module_type: ModuleType) -> Result<Option<TestParameterSet>, RepositoryError> {
        let params = self.test_parameters.read().await;
        Ok(params.get(&module_type).cloned())
    }
    
    async fn save_test_parameters(&self, module_type: ModuleType, params: &TestParameterSet) -> Result<(), RepositoryError> {
        let mut param_map = self.test_parameters.write().await;
        param_map.insert(module_type, params.clone());
        Ok(())
    }
    
    async fn list_test_parameters(&self) -> Result<Vec<(ModuleType, TestParameterSet)>, RepositoryError> {
        let params = self.test_parameters.read().await;
        Ok(params.iter().map(|(k, v)| (*k, v.clone())).collect())
    }
    
    async fn save_definition_template(&self, template_name: &str, template: &ChannelPointDefinition) -> Result<(), RepositoryError> {
        let mut templates = self.templates.write().await;
        templates.insert(template_name.to_string(), template.clone());
        Ok(())
    }
    
    async fn get_definition_template(&self, template_name: &str) -> Result<Option<ChannelPointDefinition>, RepositoryError> {
        let templates = self.templates.read().await;
        Ok(templates.get(template_name).cloned())
    }
    
    async fn list_definition_templates(&self) -> Result<Vec<String>, RepositoryError> {
        let templates = self.templates.read().await;
        Ok(templates.keys().cloned().collect())
    }
    
    async fn validate_definitions(&self, definitions: &[ChannelPointDefinition]) -> Result<Vec<ValidationIssue>, RepositoryError> {
        let mut issues = Vec::new();
        
        for definition in definitions {
            // 检查必填字段
            if definition.tag.is_empty() {
                issues.push(ValidationIssue {
                    severity: ValidationSeverity::Error,
                    message: format!("点位 {} 的标签不能为空", definition.id),
                    field: Some("tag".to_string()),
                    suggestion: Some("请填写有效的点位标签".to_string()),
                });
            }
            
            if definition.plc_communication_address.is_empty() {
                issues.push(ValidationIssue {
                    severity: ValidationSeverity::Error,
                    message: format!("点位 {} 的PLC通信地址不能为空", definition.id),
                    field: Some("plc_communication_address".to_string()),
                    suggestion: Some("请填写有效的PLC地址".to_string()),
                });
            }
            
            // 检查量程配置
            if let (Some(low), Some(high)) = (definition.range_lower_limit, definition.range_upper_limit) {
                if low >= high {
                    issues.push(ValidationIssue {
                        severity: ValidationSeverity::Error,
                        message: format!("点位 {} 的量程配置错误: 下限 {} 应小于上限 {}", definition.id, low, high),
                        field: Some("range".to_string()),
                        suggestion: Some("请确保下限小于上限".to_string()),
                    });
                }
            }
        }
        
        Ok(issues)
    }
    
    async fn check_consistency(&self) -> Result<ConsistencyReport, RepositoryError> {
        let definitions = self.definitions.read().await;
        let sets = self.configuration_sets.read().await;
        
        let mut report = ConsistencyReport {
            total_definitions: definitions.len(),
            duplicates: Vec::new(),
            orphaned_references: Vec::new(),
            missing_parameters: Vec::new(),
            issues: Vec::new(),
        };
        
        // 检查重复的标签
        let mut tag_counts = HashMap::new();
        for definition in definitions.values() {
            *tag_counts.entry(&definition.tag).or_insert(0) += 1;
        }
        
        for (tag, count) in tag_counts {
            if count > 1 {
                report.duplicates.push(tag.clone());
            }
        }
        
        // 检查配置集中的孤立引用
        for (set_name, def_ids) in sets.iter() {
            for def_id in def_ids {
                if !definitions.contains_key(def_id) {
                    report.orphaned_references.push(format!("{}:{}", set_name, def_id));
                }
            }
        }
        
        Ok(report)
    }
    
    // 辅助方法
    fn apply_filter(&self, mut definitions: Vec<ChannelPointDefinition>, filter: &Filter) -> Result<Vec<ChannelPointDefinition>, RepositoryError> {
        match filter.field.as_str() {
            "module_type" => {
                if let FilterValue::String(ref value) = filter.value {
                    let module_type = value.parse::<ModuleType>()
                        .map_err(|_| RepositoryError::ValidationError(format!("无效的模块类型: {}", value)))?;
                    definitions.retain(|d| d.module_type == module_type);
                }
            }
            "tag" => {
                if let FilterValue::String(ref value) = filter.value {
                    match filter.operator {
                        FilterOperator::Equal => definitions.retain(|d| d.tag == *value),
                        FilterOperator::Contains => definitions.retain(|d| d.tag.contains(value)),
                        _ => return Err(RepositoryError::ValidationError("不支持的操作符".to_string())),
                    }
                }
            }
            _ => return Err(RepositoryError::ValidationError(format!("不支持的过滤字段: {}", filter.field))),
        }
        Ok(definitions)
    }
    
    fn sort_definitions(&self, definitions: &mut Vec<ChannelPointDefinition>, sort_field: &str) {
        match sort_field {
            "tag" => definitions.sort_by(|a, b| a.tag.cmp(&b.tag)),
            "module_type" => definitions.sort_by(|a, b| format!("{:?}", a.module_type).cmp(&format!("{:?}", b.module_type))),
            _ => {} // 忽略不支持的排序字段
        }
    }
}
```

#### 🧪 测试方案

##### 1.2.3 配置Repository测试
```rust
// src/repositories/tests/configuration_repository_tests.rs
use super::super::configuration_repository::*;
use tokio;

/// 配置Repository测试套件
struct ConfigurationRepositoryTestSuite;

impl ConfigurationRepositoryTestSuite {
    fn create_test_definition() -> ChannelPointDefinition {
        ChannelPointDefinition {
            id: uuid::Uuid::new_v4().to_string(),
            tag: "TEST_AI_001".to_string(),
            variable_name: "Test AI Point".to_string(),
            variable_description: "Test AI Point Description".to_string(),
            station_name: "Test Station".to_string(),
            module_name: "AI Module".to_string(),
            module_type: ModuleType::AI,
            channel_tag_in_module: "CH01".to_string(),
            data_type: PointDataType::Float,
            power_supply_type: "有源".to_string(),
            wire_system: "4线制".to_string(),
            plc_absolute_address: Some("DB1.DBD0".to_string()),
            plc_communication_address: "DB1.DBD0".to_string(),
            range_lower_limit: Some(0.0),
            range_upper_limit: Some(100.0),
            engineering_unit: Some("mA".to_string()),
            sll_set_value: Some(10.0),
            sll_set_point_address: Some("DB1.DBD4".to_string()),
            sll_feedback_address: Some("DB1.DBX8.0".to_string()),
            sl_set_value: Some(20.0),
            sl_set_point_address: Some("DB1.DBD8".to_string()),
            sl_feedback_address: Some("DB1.DBX8.1".to_string()),
            sh_set_value: Some(80.0),
            sh_set_point_address: Some("DB1.DBD12".to_string()),
            sh_feedback_address: Some("DB1.DBX8.2".to_string()),
            shh_set_value: Some(90.0),
            shh_set_point_address: Some("DB1.DBD16".to_string()),
            shh_feedback_address: Some("DB1.DBX8.3".to_string()),
            maintenance_value_set_point_address: Some("DB1.DBD20".to_string()),
            maintenance_enable_switch_point_address: Some("DB1.DBX8.4".to_string()),
            access_property: Some("RW".to_string()),
            save_history: Some(true),
            power_failure_protection: Some(true),
            test_rig_plc_address: Some("DB2.DBD0".to_string()),
        }
    }
}

#[tokio::test]
async fn test_configuration_repository_basic_operations() {
    let repo = MemoryConfigurationRepository::new();
    let definition = ConfigurationRepositoryTestSuite::create_test_definition();
    let id = definition.id.clone();
    
    // 测试保存
    repo.save_channel_definition(&definition).await.expect("保存失败");
    
    // 测试获取
    let retrieved = repo.get_channel_definition(&id).await.expect("获取失败");
    assert!(retrieved.is_some(), "应该能获取到定义");
    assert_eq!(retrieved.unwrap().tag, definition.tag, "标签应该一致");
    
    // 测试列表
    let all_definitions = repo.list_channel_definitions().await.expect("列表查询失败");
    assert!(all_definitions.len() >= 1, "应该包含至少一个定义");
    
    // 测试删除
    let deleted = repo.delete_channel_definition(&id).await.expect("删除失败");
    assert!(deleted, "应该删除成功");
    
    // 验证删除
    let after_delete = repo.get_channel_definition(&id).await.expect("删除后查询失败");
    assert!(after_delete.is_none(), "删除后应该不存在");
}

#[tokio::test]
async fn test_configuration_set_operations() {
    let repo = MemoryConfigurationRepository::new();
    let definition1 = ConfigurationRepositoryTestSuite::create_test_definition();
    let mut definition2 = ConfigurationRepositoryTestSuite::create_test_definition();
    definition2.tag = "TEST_AI_002".to_string();
    
    let definitions = vec![definition1.clone(), definition2.clone()];
    let set_name = "test_config_set";
    
    // 测试保存配置集
    repo.save_configuration_set(set_name, &definitions).await.expect("保存配置集失败");
    
    // 测试加载配置集
    let loaded_definitions = repo.load_configuration_set(set_name).await.expect("加载配置集失败");
    assert_eq!(loaded_definitions.len(), 2, "应该加载2个定义");
    
    // 测试列表配置集
    let sets = repo.list_configuration_sets().await.expect("列表配置集失败");
    assert!(sets.contains(&set_name.to_string()), "应该包含测试配置集");
    
    // 测试删除配置集
    let deleted = repo.delete_configuration_set(set_name).await.expect("删除配置集失败");
    assert!(deleted, "应该删除成功");
}

#[tokio::test]
async fn test_query_operations() {
    let repo = MemoryConfigurationRepository::new();
    
    // 创建不同类型的定义
    let mut ai_definition = ConfigurationRepositoryTestSuite::create_test_definition();
    ai_definition.module_type = ModuleType::AI;
    ai_definition.tag = "AI_001".to_string();
    
    let mut di_definition = ConfigurationRepositoryTestSuite::create_test_definition();
    di_definition.module_type = ModuleType::DI;
    di_definition.tag = "DI_001".to_string();
    
    repo.save_channel_definition(&ai_definition).await.expect("保存AI定义失败");
    repo.save_channel_definition(&di_definition).await.expect("保存DI定义失败");
    
    // 测试按模块类型过滤
    let criteria = QueryCriteria {
        filters: vec![Filter {
            field: "module_type".to_string(),
            operator: FilterOperator::Equal,
            value: FilterValue::String("AI".to_string()),
        }],
        sort_by: None,
        limit: None,
        offset: None,
    };
    
    let ai_results = repo.query_channel_definitions(&criteria).await.expect("查询失败");
    assert_eq!(ai_results.len(), 1, "应该只返回AI类型的定义");
    assert_eq!(ai_results[0].module_type, ModuleType::AI, "应该是AI类型");
    
    // 测试按标签过滤
    let tag_criteria = QueryCriteria {
        filters: vec![Filter {
            field: "tag".to_string(),
            operator: FilterOperator::Contains,
            value: FilterValue::String("AI".to_string()),
        }],
        sort_by: Some("tag".to_string()),
        limit: None,
        offset: None,
    };
    
    let tag_results = repo.query_channel_definitions(&tag_criteria).await.expect("标签查询失败");
    assert!(tag_results.len() >= 1, "应该包含标签中含有AI的定义");
}

#[tokio::test]
async fn test_validation_operations() {
    let repo = MemoryConfigurationRepository::new();
    
    // 创建无效定义
    let mut invalid_definition = ConfigurationRepositoryTestSuite::create_test_definition();
    invalid_definition.tag = "".to_string(); // 空标签
    invalid_definition.plc_communication_address = "".to_string(); // 空地址
    invalid_definition.range_lower_limit = Some(100.0);
    invalid_definition.range_upper_limit = Some(50.0); // 下限大于上限
    
    let definitions = vec![invalid_definition];
    
    // 测试验证
    let issues = repo.validate_definitions(&definitions).await.expect("验证失败");
    assert!(issues.len() >= 3, "应该发现至少3个问题");
    
    // 检查错误类型
    let error_count = issues.iter().filter(|i| matches!(i.severity, ValidationSeverity::Error)).count();
    assert!(error_count >= 3, "应该有至少3个错误");
}

#[tokio::test]
async fn test_consistency_check() {
    let repo = MemoryConfigurationRepository::new();
    
    // 创建重复标签的定义
    let mut def1 = ConfigurationRepositoryTestSuite::create_test_definition();
    def1.tag = "DUPLICATE_TAG".to_string();
    
    let mut def2 = ConfigurationRepositoryTestSuite::create_test_definition();
    def2.tag = "DUPLICATE_TAG".to_string();
    
    repo.save_channel_definition(&def1).await.expect("保存定义1失败");
    repo.save_channel_definition(&def2).await.expect("保存定义2失败");
    
    // 测试一致性检查
    let report = repo.check_consistency().await.expect("一致性检查失败");
    assert_eq!(report.total_definitions, 2, "应该有2个定义");
    assert!(report.duplicates.contains(&"DUPLICATE_TAG".to_string()), "应该发现重复标签");
}

#[tokio::test] 
async fn test_test_parameters_operations() {
    let repo = MemoryConfigurationRepository::new_with_test_data().await;
    
    // 测试获取测试参数
    let ai_params = repo.get_test_parameters(ModuleType::AI).await.expect("获取AI参数失败");
    assert!(ai_params.is_some(), "应该有AI参数");
    
    let params = ai_params.unwrap();
    assert_eq!(params.test_points.len(), 5, "应该有5个测试点");
    
    // 测试保存新参数
    let new_params = TestParameterSet {
        default_range: Some((0.0, 10.0)),
        test_points: vec![0.0, 50.0, 100.0],
        tolerance: 0.5,
        test_sequence: vec![SubTestItem::HardPoint],
    };
    
    repo.save_test_parameters(ModuleType::DI, &new_params).await.expect("保存DI参数失败");
    
    // 验证保存
    let saved_params = repo.get_test_parameters(ModuleType::DI).await.expect("获取DI参数失败");
    assert!(saved_params.is_some(), "应该有DI参数");
    assert_eq!(saved_params.unwrap().test_points.len(), 3, "应该有3个测试点");
    
    // 测试列表所有参数
    let all_params = repo.list_test_parameters().await.expect("列表参数失败");
    assert!(all_params.len() >= 2, "应该有至少2种类型的参数");
}

/// 压力测试
#[tokio::test]
async fn test_large_dataset_performance() {
    let repo = MemoryConfigurationRepository::new();
    
    // 创建大量定义
    let mut definitions = Vec::new();
    for i in 0..1000 {
        let mut def = ConfigurationRepositoryTestSuite::create_test_definition();
        def.tag = format!("PERF_TEST_{:04}", i);
        definitions.push(def);
    }
    
    let start = std::time::Instant::now();
    
    // 批量保存
    repo.save_channel_definitions_batch(&definitions).await.expect("批量保存失败");
    
    let save_duration = start.elapsed();
    println!("批量保存1000个定义耗时: {:?}", save_duration);
    
    // 查询测试
    let query_start = std::time::Instant::now();
    let all_defs = repo.list_channel_definitions().await.expect("查询失败");
    let query_duration = query_start.elapsed();
    
    println!("查询1000个定义耗时: {:?}", query_duration);
    assert_eq!(all_defs.len(), 1000, "应该返回1000个定义");
    
    // 确保性能在合理范围内
    assert!(save_duration.as_millis() < 1000, "批量保存应该在1秒内完成");
    assert!(query_duration.as_millis() < 100, "查询应该在100毫秒内完成");
}
```

### ✅ 步骤1.2完成标准

1. **接口完整性**: 所有IConfigurationRepository方法都有完整实现
2. **测试覆盖**: 单元测试覆盖率达到90%以上
3. **性能指标**: 1000个定义的CRUD操作在合理时间内完成
4. **验证功能**: 数据验证和一致性检查正常工作
5. **错误处理**: 各种异常情况都有适当的错误处理

---

## 🚀 Phase 1 后续步骤预览

接下来我们将实施：

### 步骤 1.3: 创建运行时数据Repository
- 实现IRuntimeRepository接口
- 管理ChannelTestInstance和TestBatch的运行时数据
- 提供高性能的内存缓存机制

### 步骤 1.4: 创建持久化数据Repository
- 实现IPersistentRepository接口
- 管理需要永久保存的测试记录和审计数据
- 集成SQLite数据库

### 步骤 1.5: Repository集成测试
- 多Repository协同工作测试
- 数据一致性验证
- 性能和并发测试

---

*本文档持续更新中...*

### 步骤 1.3: 创建运行时数据Repository

#### 🎯 目标
实现专门管理运行时数据的Repository，提供高性能的内存缓存和状态管理。

#### 📝 具体实施

##### 1.3.1 定义运行时Repository接口
```rust
// src/repositories/runtime_repository.rs
use async_trait::async_trait;
use crate::models::structs::{ChannelTestInstance, TestBatchInfo};
use crate::models::enums::OverallTestStatus;
use crate::models::errors::RepositoryError;
use super::{IRepository, QueryCriteria};
use std::collections::HashMap;

/// 运行时数据Repository接口
#[async_trait]
pub trait IRuntimeRepository: Send + Sync {
    // 通道实例管理
    async fn get_channel_instance(&self, instance_id: &str) -> Result<Option<ChannelTestInstance>, RepositoryError>;
    async fn save_channel_instance(&self, instance: &ChannelTestInstance) -> Result<(), RepositoryError>;
    async fn delete_channel_instance(&self, instance_id: &str) -> Result<bool, RepositoryError>;
    async fn list_channel_instances(&self) -> Result<Vec<ChannelTestInstance>, RepositoryError>;
    async fn query_channel_instances(&self, criteria: &QueryCriteria) -> Result<Vec<ChannelTestInstance>, RepositoryError>;
    
    // 批次级别操作
    async fn list_batch_instances(&self, batch_id: &str) -> Result<Vec<ChannelTestInstance>, RepositoryError>;
    async fn count_batch_instances(&self, batch_id: &str) -> Result<usize, RepositoryError>;
    async fn get_batch_statistics(&self, batch_id: &str) -> Result<BatchStatistics, RepositoryError>;
    
    // 状态查询
    async fn list_instances_by_status(&self, status: OverallTestStatus) -> Result<Vec<ChannelTestInstance>, RepositoryError>;
    async fn count_instances_by_status(&self, batch_id: &str, status: OverallTestStatus) -> Result<usize, RepositoryError>;
    
    // 批量操作
    async fn save_channel_instances_batch(&self, instances: &[ChannelTestInstance]) -> Result<(), RepositoryError>;
    async fn update_instances_status(&self, instance_ids: &[String], status: OverallTestStatus) -> Result<usize, RepositoryError>;
    
    // 批次管理
    async fn get_test_batch(&self, batch_id: &str) -> Result<Option<TestBatchInfo>, RepositoryError>;
    async fn save_test_batch(&self, batch: &TestBatchInfo) -> Result<(), RepositoryError>;
    async fn delete_test_batch(&self, batch_id: &str) -> Result<bool, RepositoryError>;
    async fn list_test_batches(&self) -> Result<Vec<TestBatchInfo>, RepositoryError>;
    async fn list_active_batches(&self) -> Result<Vec<TestBatchInfo>, RepositoryError>;
    
    // 缓存管理
    async fn clear_cache(&self) -> Result<(), RepositoryError>;
    async fn get_cache_stats(&self) -> Result<CacheStatistics, RepositoryError>;
    async fn invalidate_batch_cache(&self, batch_id: &str) -> Result<(), RepositoryError>;
    
    // 事务支持
    async fn begin_transaction(&self) -> Result<TransactionHandle, RepositoryError>;
    async fn commit_transaction(&self, handle: TransactionHandle) -> Result<(), RepositoryError>;
    async fn rollback_transaction(&self, handle: TransactionHandle) -> Result<(), RepositoryError>;
}

/// 批次统计信息
#[derive(Debug, Clone)]
pub struct BatchStatistics {
    pub total_instances: usize,
    pub not_tested: usize,
    pub testing: usize,
    pub passed: usize,
    pub failed: usize,
    pub skipped: usize,
    pub progress_percentage: f64,
}

/// 缓存统计信息
#[derive(Debug, Clone)]
pub struct CacheStatistics {
    pub total_entries: usize,
    pub hit_count: u64,
    pub miss_count: u64,
    pub hit_rate: f64,
    pub memory_usage_bytes: usize,
}

/// 事务句柄
#[derive(Debug, Clone)]
pub struct TransactionHandle {
    pub id: String,
    pub start_time: chrono::DateTime<chrono::Utc>,
}
```

##### 1.3.2 实现内存运行时Repository
```rust
// src/repositories/runtime_repository.rs (continued)
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use chrono::Utc;

/// 内存运行时Repository实现
pub struct MemoryRuntimeRepository {
    instances: Arc<RwLock<HashMap<String, ChannelTestInstance>>>,
    batches: Arc<RwLock<HashMap<String, TestBatchInfo>>>,
    batch_instance_mapping: Arc<RwLock<HashMap<String, Vec<String>>>>, // batch_id -> instance_ids
    cache_stats: Arc<RwLock<CacheStatistics>>,
    transactions: Arc<RwLock<HashMap<String, TransactionState>>>,
}

#[derive(Debug, Clone)]
struct TransactionState {
    handle: TransactionHandle,
    snapshot_instances: HashMap<String, ChannelTestInstance>,
    snapshot_batches: HashMap<String, TestBatchInfo>,
}

impl MemoryRuntimeRepository {
    pub fn new() -> Self {
        Self {
            instances: Arc::new(RwLock::new(HashMap::new())),
            batches: Arc::new(RwLock::new(HashMap::new())),
            batch_instance_mapping: Arc::new(RwLock::new(HashMap::new())),
            cache_stats: Arc::new(RwLock::new(CacheStatistics {
                total_entries: 0,
                hit_count: 0,
                miss_count: 0,
                hit_rate: 0.0,
                memory_usage_bytes: 0,
            })),
            transactions: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    async fn update_cache_stats(&self, hit: bool) {
        let mut stats = self.cache_stats.write().await;
        if hit {
            stats.hit_count += 1;
        } else {
            stats.miss_count += 1;
        }
        let total = stats.hit_count + stats.miss_count;
        if total > 0 {
            stats.hit_rate = stats.hit_count as f64 / total as f64;
        }
    }
    
    async fn add_instance_to_batch_mapping(&self, batch_id: &str, instance_id: &str) {
        let mut mapping = self.batch_instance_mapping.write().await;
        mapping.entry(batch_id.to_string())
            .or_insert_with(Vec::new)
            .push(instance_id.to_string());
    }
    
    async fn remove_instance_from_batch_mapping(&self, batch_id: &str, instance_id: &str) {
        let mut mapping = self.batch_instance_mapping.write().await;
        if let Some(instance_ids) = mapping.get_mut(batch_id) {
            instance_ids.retain(|id| id != instance_id);
            if instance_ids.is_empty() {
                mapping.remove(batch_id);
            }
        }
    }
}

#[async_trait]
impl IRuntimeRepository for MemoryRuntimeRepository {
    async fn get_channel_instance(&self, instance_id: &str) -> Result<Option<ChannelTestInstance>, RepositoryError> {
        let instances = self.instances.read().await;
        let found = instances.get(instance_id).is_some();
        self.update_cache_stats(found).await;
        Ok(instances.get(instance_id).cloned())
    }
    
    async fn save_channel_instance(&self, instance: &ChannelTestInstance) -> Result<(), RepositoryError> {
        let instance_id = instance.instance_id.clone();
        let batch_id = instance.test_batch_id.clone();
        
        let mut instances = self.instances.write().await;
        let is_new = !instances.contains_key(&instance_id);
        instances.insert(instance_id.clone(), instance.clone());
        
        // 更新批次映射
        if is_new {
            self.add_instance_to_batch_mapping(&batch_id, &instance_id).await;
        }
        
        // 更新缓存统计
        let mut stats = self.cache_stats.write().await;
        stats.total_entries = instances.len();
        
        Ok(())
    }
    
    async fn delete_channel_instance(&self, instance_id: &str) -> Result<bool, RepositoryError> {
        let mut instances = self.instances.write().await;
        if let Some(instance) = instances.get(instance_id) {
            let batch_id = instance.test_batch_id.clone();
            instances.remove(instance_id);
            
            // 更新批次映射
            self.remove_instance_from_batch_mapping(&batch_id, instance_id).await;
            
            // 更新缓存统计
            let mut stats = self.cache_stats.write().await;
            stats.total_entries = instances.len();
            
            Ok(true)
        } else {
            Ok(false)
        }
    }
    
    async fn list_channel_instances(&self) -> Result<Vec<ChannelTestInstance>, RepositoryError> {
        let instances = self.instances.read().await;
        Ok(instances.values().cloned().collect())
    }
    
    async fn query_channel_instances(&self, criteria: &QueryCriteria) -> Result<Vec<ChannelTestInstance>, RepositoryError> {
        let instances = self.instances.read().await;
        let mut results: Vec<ChannelTestInstance> = instances.values().cloned().collect();
        
        // 应用过滤器
        for filter in &criteria.filters {
            results = self.apply_instance_filter(results, filter)?;
        }
        
        // 应用排序
        if let Some(sort_field) = &criteria.sort_by {
            self.sort_instances(&mut results, sort_field);
        }
        
        // 应用分页
        if let Some(offset) = criteria.offset {
            results = results.into_iter().skip(offset).collect();
        }
        
        if let Some(limit) = criteria.limit {
            results.truncate(limit);
        }
        
        Ok(results)
    }
    
    async fn list_batch_instances(&self, batch_id: &str) -> Result<Vec<ChannelTestInstance>, RepositoryError> {
        let mapping = self.batch_instance_mapping.read().await;
        let instances = self.instances.read().await;
        
        if let Some(instance_ids) = mapping.get(batch_id) {
            let mut results = Vec::new();
            for instance_id in instance_ids {
                if let Some(instance) = instances.get(instance_id) {
                    results.push(instance.clone());
                }
            }
            Ok(results)
        } else {
            Ok(Vec::new())
        }
    }
    
    async fn count_batch_instances(&self, batch_id: &str) -> Result<usize, RepositoryError> {
        let mapping = self.batch_instance_mapping.read().await;
        Ok(mapping.get(batch_id).map_or(0, |ids| ids.len()))
    }
    
    async fn get_batch_statistics(&self, batch_id: &str) -> Result<BatchStatistics, RepositoryError> {
        let instances = self.list_batch_instances(batch_id).await?;
        
        let mut stats = BatchStatistics {
            total_instances: instances.len(),
            not_tested: 0,
            testing: 0,
            passed: 0,
            failed: 0,
            skipped: 0,
            progress_percentage: 0.0,
        };
        
        for instance in &instances {
            match instance.overall_status {
                OverallTestStatus::NotTested => stats.not_tested += 1,
                OverallTestStatus::HardPointTesting | OverallTestStatus::ManualTesting => stats.testing += 1,
                OverallTestStatus::TestCompletedPassed => stats.passed += 1,
                OverallTestStatus::TestCompletedFailed => stats.failed += 1,
                OverallTestStatus::Skipped => stats.skipped += 1,
                _ => {} // 其他状态
            }
        }
        
        if stats.total_instances > 0 {
            let completed = stats.passed + stats.failed + stats.skipped;
            stats.progress_percentage = (completed as f64 / stats.total_instances as f64) * 100.0;
        }
        
        Ok(stats)
    }
    
    async fn list_instances_by_status(&self, status: OverallTestStatus) -> Result<Vec<ChannelTestInstance>, RepositoryError> {
        let instances = self.instances.read().await;
        Ok(instances.values()
            .filter(|instance| instance.overall_status == status)
            .cloned()
            .collect())
    }
    
    async fn count_instances_by_status(&self, batch_id: &str, status: OverallTestStatus) -> Result<usize, RepositoryError> {
        let instances = self.list_batch_instances(batch_id).await?;
        Ok(instances.iter()
            .filter(|instance| instance.overall_status == status)
            .count())
    }
    
    async fn save_channel_instances_batch(&self, instances: &[ChannelTestInstance]) -> Result<(), RepositoryError> {
        for instance in instances {
            self.save_channel_instance(instance).await?;
        }
        Ok(())
    }
    
    async fn update_instances_status(&self, instance_ids: &[String], status: OverallTestStatus) -> Result<usize, RepositoryError> {
        let mut instances = self.instances.write().await;
        let mut updated_count = 0;
        
        for instance_id in instance_ids {
            if let Some(instance) = instances.get_mut(instance_id) {
                instance.overall_status = status;
                instance.last_updated_time = Utc::now();
                updated_count += 1;
            }
        }
        
        Ok(updated_count)
    }
    
    async fn get_test_batch(&self, batch_id: &str) -> Result<Option<TestBatchInfo>, RepositoryError> {
        let batches = self.batches.read().await;
        Ok(batches.get(batch_id).cloned())
    }
    
    async fn save_test_batch(&self, batch: &TestBatchInfo) -> Result<(), RepositoryError> {
        let mut batches = self.batches.write().await;
        batches.insert(batch.batch_id.clone(), batch.clone());
        Ok(())
    }
    
    async fn delete_test_batch(&self, batch_id: &str) -> Result<bool, RepositoryError> {
        let mut batches = self.batches.write().await;
        let mut mapping = self.batch_instance_mapping.write().await;
        
        let existed = batches.remove(batch_id).is_some();
        mapping.remove(batch_id);
        
        Ok(existed)
    }
    
    async fn list_test_batches(&self) -> Result<Vec<TestBatchInfo>, RepositoryError> {
        let batches = self.batches.read().await;
        Ok(batches.values().cloned().collect())
    }
    
    async fn list_active_batches(&self) -> Result<Vec<TestBatchInfo>, RepositoryError> {
        let batches = self.batches.read().await;
        // 简化实现：假设所有批次都是活跃的
        // 实际实现可能需要根据批次状态或最后活动时间过滤
        Ok(batches.values().cloned().collect())
    }
    
    async fn clear_cache(&self) -> Result<(), RepositoryError> {
        let mut instances = self.instances.write().await;
        let mut batches = self.batches.write().await;
        let mut mapping = self.batch_instance_mapping.write().await;
        let mut stats = self.cache_stats.write().await;
        
        instances.clear();
        batches.clear();
        mapping.clear();
        
        *stats = CacheStatistics {
            total_entries: 0,
            hit_count: 0,
            miss_count: 0,
            hit_rate: 0.0,
            memory_usage_bytes: 0,
        };
        
        Ok(())
    }
    
    async fn get_cache_stats(&self) -> Result<CacheStatistics, RepositoryError> {
        let stats = self.cache_stats.read().await;
        Ok(stats.clone())
    }
    
    async fn invalidate_batch_cache(&self, batch_id: &str) -> Result<(), RepositoryError> {
        // 从缓存中移除指定批次的所有实例
        let mapping = self.batch_instance_mapping.read().await;
        if let Some(instance_ids) = mapping.get(batch_id) {
            let mut instances = self.instances.write().await;
            for instance_id in instance_ids {
                instances.remove(instance_id);
            }
        }
        
        // 移除批次本身
        let mut batches = self.batches.write().await;
        batches.remove(batch_id);
        
        Ok(())
    }
    
    async fn begin_transaction(&self) -> Result<TransactionHandle, RepositoryError> {
        let handle = TransactionHandle {
            id: Uuid::new_v4().to_string(),
            start_time: Utc::now(),
        };
        
        // 创建当前状态的快照
        let instances = self.instances.read().await;
        let batches = self.batches.read().await;
        
        let transaction_state = TransactionState {
            handle: handle.clone(),
            snapshot_instances: instances.clone(),
            snapshot_batches: batches.clone(),
        };
        
        let mut transactions = self.transactions.write().await;
        transactions.insert(handle.id.clone(), transaction_state);
        
        Ok(handle)
    }
    
    async fn commit_transaction(&self, handle: TransactionHandle) -> Result<(), RepositoryError> {
        let mut transactions = self.transactions.write().await;
        if transactions.remove(&handle.id).is_some() {
            Ok(())
        } else {
            Err(RepositoryError::NotFound(format!("事务不存在: {}", handle.id)))
        }
    }
    
    async fn rollback_transaction(&self, handle: TransactionHandle) -> Result<(), RepositoryError> {
        let mut transactions = self.transactions.write().await;
        if let Some(transaction_state) = transactions.remove(&handle.id) {
            // 恢复快照状态
            let mut instances = self.instances.write().await;
            let mut batches = self.batches.write().await;
            
            *instances = transaction_state.snapshot_instances;
            *batches = transaction_state.snapshot_batches;
            
            Ok(())
        } else {
            Err(RepositoryError::NotFound(format!("事务不存在: {}", handle.id)))
        }
    }
    
    // 辅助方法
    fn apply_instance_filter(&self, mut instances: Vec<ChannelTestInstance>, filter: &Filter) -> Result<Vec<ChannelTestInstance>, RepositoryError> {
        match filter.field.as_str() {
            "batch_id" => {
                if let FilterValue::String(ref value) = filter.value {
                    instances.retain(|i| i.test_batch_id == *value);
                }
            }
            "overall_status" => {
                if let FilterValue::String(ref value) = filter.value {
                    let status = value.parse::<OverallTestStatus>()
                        .map_err(|_| RepositoryError::ValidationError(format!("无效的状态: {}", value)))?;
                    instances.retain(|i| i.overall_status == status);
                }
            }
            "definition_id" => {
                if let FilterValue::String(ref value) = filter.value {
                    instances.retain(|i| i.definition_id == *value);
                }
            }
            _ => return Err(RepositoryError::ValidationError(format!("不支持的过滤字段: {}", filter.field))),
        }
        Ok(instances)
    }
    
    fn sort_instances(&self, instances: &mut Vec<ChannelTestInstance>, sort_field: &str) {
        match sort_field {
            "instance_id" => instances.sort_by(|a, b| a.instance_id.cmp(&b.instance_id)),
            "last_updated_time" => instances.sort_by(|a, b| a.last_updated_time.cmp(&b.last_updated_time)),
            "overall_status" => instances.sort_by(|a, b| format!("{:?}", a.overall_status).cmp(&format!("{:?}", b.overall_status))),
            _ => {} // 忽略不支持的排序字段
        }
    }
}
```

#### 🧪 测试方案

##### 1.3.3 运行时Repository测试
```rust
// src/repositories/tests/runtime_repository_tests.rs
use super::super::runtime_repository::*;
use crate::models::structs::ChannelTestInstance;
use crate::models::enums::OverallTestStatus;
use tokio;

struct RuntimeRepositoryTestSuite;

impl RuntimeRepositoryTestSuite {
    fn create_test_instance(batch_id: &str, tag: &str) -> ChannelTestInstance {
        ChannelTestInstance {
            instance_id: uuid::Uuid::new_v4().to_string(),
            definition_id: uuid::Uuid::new_v4().to_string(),
            test_batch_id: batch_id.to_string(),
            overall_status: OverallTestStatus::NotTested,
            current_step_details: None,
            error_message: None,
            start_time: None,
            last_updated_time: chrono::Utc::now(),
            final_test_time: None,
            total_test_duration_ms: None,
            sub_test_results: std::collections::HashMap::new(),
            hardpoint_readings: None,
            manual_test_current_value_input: None,
            manual_test_current_value_output: None,
        }
    }
    
    fn create_test_batch() -> TestBatchInfo {
        TestBatchInfo {
            batch_id: uuid::Uuid::new_v4().to_string(),
            product_model: Some("测试产品".to_string()),
            serial_number: Some("SN001".to_string()),
            customer_name: Some("测试客户".to_string()),
            creation_time: chrono::Utc::now(),
            status_summary: Some("测试中".to_string()),
            total_points: 0,
            tested_points: 0,
            passed_points: 0,
            failed_points: 0,
        }
    }
}

#[tokio::test]
async fn test_runtime_repository_basic_operations() {
    let repo = MemoryRuntimeRepository::new();
    let batch_id = "test_batch_001";
    let instance = RuntimeRepositoryTestSuite::create_test_instance(batch_id, "TEST_001");
    let instance_id = instance.instance_id.clone();
    
    // 测试保存实例
    repo.save_channel_instance(&instance).await.expect("保存实例失败");
    
    // 测试获取实例
    let retrieved = repo.get_channel_instance(&instance_id).await.expect("获取实例失败");
    assert!(retrieved.is_some(), "应该能获取到实例");
    assert_eq!(retrieved.unwrap().test_batch_id, batch_id, "批次ID应该一致");
    
    // 测试列表实例
    let all_instances = repo.list_channel_instances().await.expect("列表查询失败");
    assert!(all_instances.len() >= 1, "应该包含至少一个实例");
    
    // 测试删除实例
    let deleted = repo.delete_channel_instance(&instance_id).await.expect("删除失败");
    assert!(deleted, "应该删除成功");
    
    // 验证删除
    let after_delete = repo.get_channel_instance(&instance_id).await.expect("删除后查询失败");
    assert!(after_delete.is_none(), "删除后应该不存在");
}

#[tokio::test]
async fn test_batch_operations() {
    let repo = MemoryRuntimeRepository::new();
    let batch = RuntimeRepositoryTestSuite::create_test_batch();
    let batch_id = batch.batch_id.clone();
    
    // 创建测试实例
    let instance1 = RuntimeRepositoryTestSuite::create_test_instance(&batch_id, "TEST_001");
    let instance2 = RuntimeRepositoryTestSuite::create_test_instance(&batch_id, "TEST_002");
    let mut instance3 = RuntimeRepositoryTestSuite::create_test_instance(&batch_id, "TEST_003");
    instance3.overall_status = OverallTestStatus::TestCompletedPassed;
    
    // 保存批次和实例
    repo.save_test_batch(&batch).await.expect("保存批次失败");
    repo.save_channel_instance(&instance1).await.expect("保存实例1失败");
    repo.save_channel_instance(&instance2).await.expect("保存实例2失败");
    repo.save_channel_instance(&instance3).await.expect("保存实例3失败");
    
    // 测试批次实例列表
    let batch_instances = repo.list_batch_instances(&batch_id).await.expect("获取批次实例失败");
    assert_eq!(batch_instances.len(), 3, "应该有3个实例");
    
    // 测试批次实例计数
    let count = repo.count_batch_instances(&batch_id).await.expect("计数失败");
    assert_eq!(count, 3, "计数应该为3");
    
    // 测试批次统计
    let stats = repo.get_batch_statistics(&batch_id).await.expect("获取统计失败");
    assert_eq!(stats.total_instances, 3, "总实例数应该为3");
    assert_eq!(stats.not_tested, 2, "未测试实例数应该为2");
    assert_eq!(stats.passed, 1, "通过实例数应该为1");
    assert!(stats.progress_percentage > 0.0, "进度应该大于0");
    
    // 测试按状态查询
    let not_tested = repo.list_instances_by_status(OverallTestStatus::NotTested).await.expect("按状态查询失败");
    assert_eq!(not_tested.len(), 2, "应该有2个未测试实例");
    
    let passed_count = repo.count_instances_by_status(&batch_id, OverallTestStatus::TestCompletedPassed).await.expect("按状态计数失败");
    assert_eq!(passed_count, 1, "应该有1个通过实例");
}

#[tokio::test]
async fn test_batch_update_operations() {
    let repo = MemoryRuntimeRepository::new();
    let batch_id = "test_batch_002";
    
    // 创建多个实例
    let instances: Vec<ChannelTestInstance> = (0..5)
        .map(|i| RuntimeRepositoryTestSuite::create_test_instance(batch_id, &format!("TEST_{:03}", i)))
        .collect();
    
    let instance_ids: Vec<String> = instances.iter().map(|i| i.instance_id.clone()).collect();
    
    // 批量保存
    repo.save_channel_instances_batch(&instances).await.expect("批量保存失败");
    
    // 验证批量保存
    let saved_instances = repo.list_batch_instances(batch_id).await.expect("获取保存的实例失败");
    assert_eq!(saved_instances.len(), 5, "应该有5个实例");
    
    // 测试批量状态更新
    let updated_count = repo.update_instances_status(&instance_ids[0..3], OverallTestStatus::HardPointTesting).await.expect("批量更新失败");
    assert_eq!(updated_count, 3, "应该更新3个实例");
    
    // 验证更新结果
    let testing_instances = repo.list_instances_by_status(OverallTestStatus::HardPointTesting).await.expect("查询测试中实例失败");
    assert_eq!(testing_instances.len(), 3, "应该有3个测试中实例");
}

#[tokio::test]
async fn test_query_operations() {
    let repo = MemoryRuntimeRepository::new();
    let batch_id_1 = "batch_001";
    let batch_id_2 = "batch_002";
    
    // 创建不同批次的实例
    let instance1 = RuntimeRepositoryTestSuite::create_test_instance(batch_id_1, "TEST_001");
    let instance2 = RuntimeRepositoryTestSuite::create_test_instance(batch_id_1, "TEST_002");
    let instance3 = RuntimeRepositoryTestSuite::create_test_instance(batch_id_2, "TEST_003");
    
    repo.save_channel_instance(&instance1).await.expect("保存实例1失败");
    repo.save_channel_instance(&instance2).await.expect("保存实例2失败");
    repo.save_channel_instance(&instance3).await.expect("保存实例3失败");
    
    // 测试按批次过滤
    let criteria = QueryCriteria {
        filters: vec![Filter {
            field: "batch_id".to_string(),
            operator: FilterOperator::Equal,
            value: FilterValue::String(batch_id_1.to_string()),
        }],
        sort_by: Some("instance_id".to_string()),
        limit: None,
        offset: None,
    };
    
    let batch1_results = repo.query_channel_instances(&criteria).await.expect("查询失败");
    assert_eq!(batch1_results.len(), 2, "批次1应该有2个实例");
    
    // 测试分页查询
    let paging_criteria = QueryCriteria {
        filters: vec![],
        sort_by: Some("instance_id".to_string()),
        limit: Some(2),
        offset: Some(1),
    };
    
    let paging_results = repo.query_channel_instances(&paging_criteria).await.expect("分页查询失败");
    assert!(paging_results.len() <= 2, "分页结果不应超过限制");
}

#[tokio::test]
async fn test_cache_operations() {
    let repo = MemoryRuntimeRepository::new();
    let instance = RuntimeRepositoryTestSuite::create_test_instance("test_batch", "TEST_001");
    let instance_id = instance.instance_id.clone();
    
    // 测试缓存未命中
    let _ = repo.get_channel_instance(&instance_id).await.expect("查询失败");
    let stats = repo.get_cache_stats().await.expect("获取缓存统计失败");
    assert_eq!(stats.miss_count, 1, "应该有1次未命中");
    
    // 保存实例
    repo.save_channel_instance(&instance).await.expect("保存失败");
    
    // 测试缓存命中
    let _ = repo.get_channel_instance(&instance_id).await.expect("查询失败");
    let stats = repo.get_cache_stats().await.expect("获取缓存统计失败");
    assert_eq!(stats.hit_count, 1, "应该有1次命中");
    assert!(stats.hit_rate > 0.0, "命中率应该大于0");
    
    // 测试清除缓存
    repo.clear_cache().await.expect("清除缓存失败");
    let stats = repo.get_cache_stats().await.expect("获取缓存统计失败");
    assert_eq!(stats.total_entries, 0, "缓存应该为空");
    assert_eq!(stats.hit_count, 0, "命中计数应该重置");
    assert_eq!(stats.miss_count, 0, "未命中计数应该重置");
}

#[tokio::test]
async fn test_transaction_operations() {
    let repo = MemoryRuntimeRepository::new();
    let instance = RuntimeRepositoryTestSuite::create_test_instance("test_batch", "TEST_001");
    
    // 保存初始数据
    repo.save_channel_instance(&instance).await.expect("保存初始数据失败");
    
    // 开始事务
    let transaction = repo.begin_transaction().await.expect("开始事务失败");
    
    // 修改数据
    let mut modified_instance = instance.clone();
    modified_instance.overall_status = OverallTestStatus::TestCompletedPassed;
    repo.save_channel_instance(&modified_instance).await.expect("修改数据失败");
    
    // 验证修改
    let current = repo.get_channel_instance(&instance.instance_id).await.expect("查询当前数据失败");
    assert_eq!(current.unwrap().overall_status, OverallTestStatus::TestCompletedPassed, "数据应该已修改");
    
    // 回滚事务
    repo.rollback_transaction(transaction).await.expect("回滚事务失败");
    
    // 验证回滚
    let after_rollback = repo.get_channel_instance(&instance.instance_id).await.expect("查询回滚后数据失败");
    assert_eq!(after_rollback.unwrap().overall_status, OverallTestStatus::NotTested, "数据应该已回滚");
}

/// 性能测试
#[tokio::test]
async fn test_performance_with_large_dataset() {
    let repo = MemoryRuntimeRepository::new();
    let batch_id = "performance_test_batch";
    
    // 创建大量实例
    let start = std::time::Instant::now();
    let instances: Vec<ChannelTestInstance> = (0..10000)
        .map(|i| RuntimeRepositoryTestSuite::create_test_instance(batch_id, &format!("PERF_TEST_{:05}", i)))
        .collect();
    
    let creation_time = start.elapsed();
    println!("创建10000个实例耗时: {:?}", creation_time);
    
    // 批量保存
    let save_start = std::time::Instant::now();
    repo.save_channel_instances_batch(&instances).await.expect("批量保存失败");
    let save_time = save_start.elapsed();
    println!("批量保存10000个实例耗时: {:?}", save_time);
    
    // 查询测试
    let query_start = std::time::Instant::now();
    let all_instances = repo.list_batch_instances(batch_id).await.expect("查询失败");
    let query_time = query_start.elapsed();
    println!("查询10000个实例耗时: {:?}", query_time);
    
    assert_eq!(all_instances.len(), 10000, "应该返回10000个实例");
    
    // 统计测试
    let stats_start = std::time::Instant::now();
    let stats = repo.get_batch_statistics(batch_id).await.expect("统计失败");
    let stats_time = stats_start.elapsed();
    println!("统计10000个实例耗时: {:?}", stats_time);
    
    assert_eq!(stats.total_instances, 10000, "统计总数应该为10000");
    
    // 确保性能在合理范围内
    assert!(save_time.as_millis() < 2000, "批量保存应该在2秒内完成");
    assert!(query_time.as_millis() < 500, "查询应该在500毫秒内完成");
    assert!(stats_time.as_millis() < 200, "统计应该在200毫秒内完成");
}
```

### ✅ 步骤1.3完成标准

1. **接口完整性**: 所有IRuntimeRepository方法都有完整实现
2. **性能优化**: 支持高并发读写和大数据集操作
3. **缓存机制**: 完整的缓存统计和管理功能
4. **事务支持**: 提供基本的事务回滚能力
5. **测试覆盖**: 包含性能测试和并发测试

---

## 🚀 Phase 1 后续步骤预览

接下来我们将实施：

### 步骤 1.4: 创建持久化数据Repository
- 实现IPersistentRepository接口
- 管理需要永久保存的测试记录和审计数据
- 集成SQLite数据库

### 步骤 1.5: Repository集成测试
- 多Repository协同工作测试
- 数据一致性验证
- 性能和并发测试

---

*本文档持续更新中...*

### 步骤 1.4: 创建持久化数据Repository

#### 🎯 目标
实现专门管理持久化数据的Repository，确保重要数据的长期保存和查询。

#### 📝 具体实施

##### 1.4.1 定义持久化Repository接口
```rust
// src/repositories/persistent_repository.rs
use async_trait::async_trait;
use crate::models::structs::{TestRecord, TestBatchPersistent, AuditRecord};
use crate::models::errors::RepositoryError;
use super::{IRepository, QueryCriteria};
use chrono::{DateTime, Utc};

/// 持久化数据Repository接口
#[async_trait]
pub trait IPersistentRepository: Send + Sync {
    // 测试记录管理
    async fn save_test_record(&self, record: &TestRecord) -> Result<(), RepositoryError>;
    async fn get_test_record(&self, record_id: &str) -> Result<Option<TestRecord>, RepositoryError>;
    async fn list_test_records(&self, batch_id: &str) -> Result<Vec<TestRecord>, RepositoryError>;
    async fn query_test_records(&self, criteria: &QueryCriteria) -> Result<Vec<TestRecord>, RepositoryError>;
    
    // 批次持久化管理
    async fn save_batch_persistent(&self, batch: &TestBatchPersistent) -> Result<(), RepositoryError>;
    async fn get_batch_persistent(&self, batch_id: &str) -> Result<Option<TestBatchPersistent>, RepositoryError>;
    async fn list_historical_batches(&self, from_date: DateTime<Utc>, to_date: DateTime<Utc>) -> Result<Vec<TestBatchPersistent>, RepositoryError>;
    
    // 审计记录管理
    async fn save_audit_record(&self, record: &AuditRecord) -> Result<(), RepositoryError>;
    async fn list_audit_records(&self, entity_id: &str) -> Result<Vec<AuditRecord>, RepositoryError>;
    
    // 报表和统计
    async fn generate_batch_report(&self, batch_id: &str) -> Result<BatchReport, RepositoryError>;
    async fn get_test_statistics(&self, from_date: DateTime<Utc>, to_date: DateTime<Utc>) -> Result<TestStatistics, RepositoryError>;
}
```

### ✅ 步骤1.4完成标准

1. **接口完整性**: 所有IPersistentRepository方法都有完整实现
2. **数据持久化**: 重要数据能够可靠保存到SQLite数据库
3. **查询性能**: 复杂查询在合理时间内完成
4. **报表功能**: 能够生成完整的测试报表
5. **审计跟踪**: 完整的操作审计记录

---

## 🚀 Phase 2: 状态管理器重构

### 步骤 2.1: 创建增强的状态管理器接口

#### 🎯 目标
重新设计状态管理器，确保严格的状态控制和一致性保证。

#### 📝 具体实施

##### 2.1.1 定义状态管理器核心接口
```rust
// src/services/state_management/channel_state_manager.rs
use async_trait::async_trait;
use crate::models::structs::{ChannelTestInstance, StateTransition, TestOutcome};
use crate::models::enums::{OverallTestStatus, SubTestItem};
use crate::models::errors::StateError;
use crate::repositories::{IRuntimeRepository, IPersistentRepository};
use std::sync::Arc;
use tokio::sync::{RwLock, broadcast};

/// 状态管理器接口 - 系统中唯一允许修改实例状态的组件
#[async_trait]
pub trait IChannelStateManager: Send + Sync {
    // 状态查询 (只读操作)
    async fn get_current_state(&self, instance_id: &str) -> Result<ChannelRuntimeState, StateError>;
    async fn can_transition_to(&self, instance_id: &str, target_status: OverallTestStatus) -> Result<bool, StateError>;
    async fn get_transition_history(&self, instance_id: &str) -> Result<Vec<StateTransition>, StateError>;
    
    // 状态修改 (唯一修改入口)
    async fn apply_test_outcome(&self, instance_id: &str, outcome: TestOutcome) -> Result<StateTransition, StateError>;
    async fn force_state_transition(&self, instance_id: &str, target_status: OverallTestStatus, reason: String) -> Result<StateTransition, StateError>;
    async fn reset_for_retest(&self, instance_id: &str) -> Result<StateTransition, StateError>;
    
    // 批量状态操作
    async fn batch_state_update(&self, updates: Vec<StateUpdateRequest>) -> Result<Vec<StateTransition>, StateError>;
    async fn batch_reset_for_retest(&self, instance_ids: Vec<String>) -> Result<Vec<StateTransition>, StateError>;
    
    // 状态事件订阅
    async fn subscribe_state_changes(&self) -> Result<broadcast::Receiver<StateChangeEvent>, StateError>;
    async fn subscribe_instance_changes(&self, instance_id: &str) -> Result<broadcast::Receiver<StateChangeEvent>, StateError>;
    
    // 状态验证和审计
    async fn validate_state_consistency(&self, instance_id: &str) -> Result<StateValidationResult, StateError>;
    async fn audit_state_changes(&self, from_time: DateTime<Utc>, to_time: DateTime<Utc>) -> Result<Vec<StateAuditRecord>, StateError>;
}

/// 状态更新请求
#[derive(Debug, Clone)]
pub struct StateUpdateRequest {
    pub instance_id: String,
    pub outcome: TestOutcome,
    pub force_transition: bool,
    pub reason: Option<String>,
}

/// 状态变更事件
#[derive(Debug, Clone)]
pub struct StateChangeEvent {
    pub instance_id: String,
    pub old_status: OverallTestStatus,
    pub new_status: OverallTestStatus,
    pub transition_reason: String,
    pub timestamp: DateTime<Utc>,
    pub operator: Option<String>,
}

/// 状态验证结果
#[derive(Debug, Clone)]
pub struct StateValidationResult {
    pub is_valid: bool,
    pub issues: Vec<StateValidationIssue>,
    pub recommendations: Vec<String>,
}

/// 状态验证问题
#[derive(Debug, Clone)]
pub struct StateValidationIssue {
    pub severity: ValidationSeverity,
    pub message: String,
    pub field: Option<String>,
}

/// 状态审计记录
#[derive(Debug, Clone)]
pub struct StateAuditRecord {
    pub instance_id: String,
    pub transition: StateTransition,
    pub context: StateAuditContext,
}

#[derive(Debug, Clone)]
pub struct StateAuditContext {
    pub user_id: Option<String>,
    pub operation_id: String,
    pub client_info: Option<String>,
}
```

##### 2.1.2 实现增强的状态管理器
```rust
// src/services/state_management/channel_state_manager.rs (continued)
use std::collections::HashMap;
use uuid::Uuid;
use chrono::Utc;

/// 增强的通道状态管理器实现
pub struct EnhancedChannelStateManager {
    runtime_repo: Arc<dyn IRuntimeRepository>,
    persistent_repo: Arc<dyn IPersistentRepository>,
    state_transitions: Arc<RwLock<HashMap<String, Vec<StateTransition>>>>,
    event_broadcaster: broadcast::Sender<StateChangeEvent>,
    transition_rules: Arc<StateTransitionRules>,
}

impl EnhancedChannelStateManager {
    pub fn new(
        runtime_repo: Arc<dyn IRuntimeRepository>,
        persistent_repo: Arc<dyn IPersistentRepository>,
    ) -> Self {
        let (event_broadcaster, _) = broadcast::channel(1000);
        
        Self {
            runtime_repo,
            persistent_repo,
            state_transitions: Arc::new(RwLock::new(HashMap::new())),
            event_broadcaster,
            transition_rules: Arc::new(StateTransitionRules::new()),
        }
    }
    
    /// 验证状态转换是否合法
    async fn validate_transition(
        &self,
        current_status: OverallTestStatus,
        target_status: OverallTestStatus,
        force: bool,
    ) -> Result<bool, StateError> {
        if force {
            return Ok(true);
        }
        
        self.transition_rules.is_valid_transition(current_status, target_status)
    }
    
    /// 创建状态转换记录
    fn create_transition(
        &self,
        instance_id: &str,
        old_status: OverallTestStatus,
        new_status: OverallTestStatus,
        reason: &str,
    ) -> StateTransition {
        StateTransition {
            transition_id: Uuid::new_v4().to_string(),
            instance_id: instance_id.to_string(),
            old_status,
            new_status,
            reason: reason.to_string(),
            timestamp: Utc::now(),
            duration_ms: None,
        }
    }
    
    /// 发布状态变更事件
    async fn publish_state_change(&self, event: StateChangeEvent) -> Result<(), StateError> {
        if let Err(_) = self.event_broadcaster.send(event) {
            // 如果没有订阅者，这是正常的，不需要报错
        }
        Ok(())
    }
    
    /// 保存状态转换历史
    async fn save_transition_history(&self, transition: &StateTransition) -> Result<(), StateError> {
        let mut transitions = self.state_transitions.write().await;
        transitions
            .entry(transition.instance_id.clone())
            .or_insert_with(Vec::new)
            .push(transition.clone());
        Ok(())
    }
}

#[async_trait]
impl IChannelStateManager for EnhancedChannelStateManager {
    async fn get_current_state(&self, instance_id: &str) -> Result<ChannelRuntimeState, StateError> {
        let instance = self.runtime_repo
            .get_channel_instance(instance_id)
            .await
            .map_err(|e| StateError::RepositoryError(e.to_string()))?
            .ok_or_else(|| StateError::InstanceNotFound(instance_id.to_string()))?;
        
        Ok(ChannelRuntimeState {
            overall_status: instance.overall_status,
            current_phase: TestPhase::from_status(instance.overall_status),
            error_info: instance.error_message.map(|msg| ErrorInfo::new(msg)),
            timestamps: TimestampCollection::from_instance(&instance),
            progress_info: ProgressInfo::calculate_from_results(&instance.sub_test_results),
        })
    }
    
    async fn can_transition_to(&self, instance_id: &str, target_status: OverallTestStatus) -> Result<bool, StateError> {
        let current_state = self.get_current_state(instance_id).await?;
        self.validate_transition(current_state.overall_status, target_status, false).await
    }
    
    async fn get_transition_history(&self, instance_id: &str) -> Result<Vec<StateTransition>, StateError> {
        let transitions = self.state_transitions.read().await;
        Ok(transitions
            .get(instance_id)
            .cloned()
            .unwrap_or_default())
    }
    
    async fn apply_test_outcome(&self, instance_id: &str, outcome: TestOutcome) -> Result<StateTransition, StateError> {
        // 获取当前实例
        let mut instance = self.runtime_repo
            .get_channel_instance(instance_id)
            .await
            .map_err(|e| StateError::RepositoryError(e.to_string()))?
            .ok_or_else(|| StateError::InstanceNotFound(instance_id.to_string()))?;
        
        let old_status = instance.overall_status;
        
        // 根据测试结果确定新状态
        let new_status = match outcome.result {
            TestResult::Passed => {
                if outcome.test_item == SubTestItem::HardPoint {
                    OverallTestStatus::HardPointTesting
                } else {
                    OverallTestStatus::TestCompletedPassed
                }
            }
            TestResult::Failed => OverallTestStatus::TestCompletedFailed,
            TestResult::Skipped => OverallTestStatus::Skipped,
            TestResult::InProgress => {
                match outcome.test_item {
                    SubTestItem::HardPoint => OverallTestStatus::HardPointTesting,
                    _ => OverallTestStatus::ManualTesting,
                }
            }
        };
        
        // 验证状态转换
        if !self.validate_transition(old_status, new_status, false).await? {
            return Err(StateError::InvalidTransition {
                from: old_status,
                to: new_status,
                reason: "不合法的状态转换".to_string(),
            });
        }
        
        // 更新实例状态
        instance.overall_status = new_status;
        instance.last_updated_time = Utc::now();
        
        // 更新测试结果
        instance.sub_test_results.insert(outcome.test_item, SubTestExecutionResult {
            result: outcome.result,
            measured_value: outcome.measured_value,
            expected_value: outcome.expected_value,
            tolerance: outcome.tolerance,
            timestamp: Utc::now(),
            error_message: outcome.error_message,
            duration_ms: outcome.duration_ms,
        });
        
        // 保存更新后的实例
        self.runtime_repo
            .save_channel_instance(&instance)
            .await
            .map_err(|e| StateError::RepositoryError(e.to_string()))?;
        
        // 创建状态转换记录
        let transition = self.create_transition(
            instance_id,
            old_status,
            new_status,
            &format!("测试结果应用: {:?}", outcome.test_item),
        );
        
        // 保存转换历史
        self.save_transition_history(&transition).await?;
        
        // 发布状态变更事件
        let event = StateChangeEvent {
            instance_id: instance_id.to_string(),
            old_status,
            new_status,
            transition_reason: transition.reason.clone(),
            timestamp: transition.timestamp,
            operator: None,
        };
        self.publish_state_change(event).await?;
        
        Ok(transition)
    }
    
    async fn force_state_transition(&self, instance_id: &str, target_status: OverallTestStatus, reason: String) -> Result<StateTransition, StateError> {
        // 获取当前实例
        let mut instance = self.runtime_repo
            .get_channel_instance(instance_id)
            .await
            .map_err(|e| StateError::RepositoryError(e.to_string()))?
            .ok_or_else(|| StateError::InstanceNotFound(instance_id.to_string()))?;
        
        let old_status = instance.overall_status;
        
        // 强制状态转换不需要验证合法性
        instance.overall_status = target_status;
        instance.last_updated_time = Utc::now();
        
        // 保存更新后的实例
        self.runtime_repo
            .save_channel_instance(&instance)
            .await
            .map_err(|e| StateError::RepositoryError(e.to_string()))?;
        
        // 创建状态转换记录
        let transition = self.create_transition(
            instance_id,
            old_status,
            target_status,
            &format!("强制转换: {}", reason),
        );
        
        // 保存转换历史
        self.save_transition_history(&transition).await?;
        
        // 发布状态变更事件
        let event = StateChangeEvent {
            instance_id: instance_id.to_string(),
            old_status,
            new_status: target_status,
            transition_reason: transition.reason.clone(),
            timestamp: transition.timestamp,
            operator: None,
        };
        self.publish_state_change(event).await?;
        
        Ok(transition)
    }
    
    async fn reset_for_retest(&self, instance_id: &str) -> Result<StateTransition, StateError> {
        // 获取当前实例
        let mut instance = self.runtime_repo
            .get_channel_instance(instance_id)
            .await
            .map_err(|e| StateError::RepositoryError(e.to_string()))?
            .ok_or_else(|| StateError::InstanceNotFound(instance_id.to_string()))?;
        
        let old_status = instance.overall_status;
        
        // 重置为未测试状态
        instance.overall_status = OverallTestStatus::NotTested;
        instance.last_updated_time = Utc::now();
        instance.start_time = None;
        instance.final_test_time = None;
        instance.total_test_duration_ms = None;
        instance.error_message = None;
        instance.sub_test_results.clear();
        instance.hardpoint_readings = None;
        instance.manual_test_current_value_input = None;
        instance.manual_test_current_value_output = None;
        
        // 保存更新后的实例
        self.runtime_repo
            .save_channel_instance(&instance)
            .await
            .map_err(|e| StateError::RepositoryError(e.to_string()))?;
        
        // 创建状态转换记录
        let transition = self.create_transition(
            instance_id,
            old_status,
            OverallTestStatus::NotTested,
            "重置进行重测",
        );
        
        // 保存转换历史
        self.save_transition_history(&transition).await?;
        
        // 发布状态变更事件
        let event = StateChangeEvent {
            instance_id: instance_id.to_string(),
            old_status,
            new_status: OverallTestStatus::NotTested,
            transition_reason: transition.reason.clone(),
            timestamp: transition.timestamp,
            operator: None,
        };
        self.publish_state_change(event).await?;
        
        Ok(transition)
    }
    
    async fn batch_state_update(&self, updates: Vec<StateUpdateRequest>) -> Result<Vec<StateTransition>, StateError> {
        let mut results = Vec::new();
        
        for update in updates {
            let result = if update.force_transition {
                if let Some(reason) = update.reason {
                    // 从TestOutcome中提取目标状态
                    let target_status = match update.outcome.result {
                        TestResult::Passed => OverallTestStatus::TestCompletedPassed,
                        TestResult::Failed => OverallTestStatus::TestCompletedFailed,
                        TestResult::Skipped => OverallTestStatus::Skipped,
                        TestResult::InProgress => OverallTestStatus::HardPointTesting,
                    };
                    self.force_state_transition(&update.instance_id, target_status, reason).await
                } else {
                    self.apply_test_outcome(&update.instance_id, update.outcome).await
                }
            } else {
                self.apply_test_outcome(&update.instance_id, update.outcome).await
            };
            
            match result {
                Ok(transition) => results.push(transition),
                Err(e) => {
                    // 记录错误但继续处理其他更新
                    log::error!("批量状态更新失败 - 实例ID: {}, 错误: {:?}", update.instance_id, e);
                    // 可以选择返回错误或继续处理
                    return Err(e);
                }
            }
        }
        
        Ok(results)
    }
    
    async fn batch_reset_for_retest(&self, instance_ids: Vec<String>) -> Result<Vec<StateTransition>, StateError> {
        let mut results = Vec::new();
        
        for instance_id in instance_ids {
            match self.reset_for_retest(&instance_id).await {
                Ok(transition) => results.push(transition),
                Err(e) => {
                    log::error!("批量重置失败 - 实例ID: {}, 错误: {:?}", instance_id, e);
                    return Err(e);
                }
            }
        }
        
        Ok(results)
    }
    
    async fn subscribe_state_changes(&self) -> Result<broadcast::Receiver<StateChangeEvent>, StateError> {
        Ok(self.event_broadcaster.subscribe())
    }
    
    async fn subscribe_instance_changes(&self, instance_id: &str) -> Result<broadcast::Receiver<StateChangeEvent>, StateError> {
        // 创建过滤特定实例的接收器
        let receiver = self.event_broadcaster.subscribe();
        Ok(receiver) // 实际实现可能需要过滤逻辑
    }
    
    async fn validate_state_consistency(&self, instance_id: &str) -> Result<StateValidationResult, StateError> {
        let instance = self.runtime_repo
            .get_channel_instance(instance_id)
            .await
            .map_err(|e| StateError::RepositoryError(e.to_string()))?
            .ok_or_else(|| StateError::InstanceNotFound(instance_id.to_string()))?;
        
        let mut issues = Vec::new();
        let mut is_valid = true;
        
        // 检查状态与测试结果的一致性
        if instance.overall_status == OverallTestStatus::TestCompletedPassed {
            let has_failed_results = instance.sub_test_results.values()
                .any(|result| result.result == TestResult::Failed);
            
            if has_failed_results {
                is_valid = false;
                issues.push(StateValidationIssue {
                    severity: ValidationSeverity::Error,
                    message: "状态为已通过，但存在失败的测试结果".to_string(),
                    field: Some("overall_status".to_string()),
                });
            }
        }
        
        // 检查时间戳的逻辑性
        if let (Some(start), Some(end)) = (instance.start_time, instance.final_test_time) {
            if start > end {
                is_valid = false;
                issues.push(StateValidationIssue {
                    severity: ValidationSeverity::Error,
                    message: "开始时间晚于结束时间".to_string(),
                    field: Some("timestamps".to_string()),
                });
            }
        }
        
        let recommendations = if !is_valid {
            vec!["建议重置实例状态并重新进行测试".to_string()]
        } else {
            vec![]
        };
        
        Ok(StateValidationResult {
            is_valid,
            issues,
            recommendations,
        })
    }
    
    async fn audit_state_changes(&self, from_time: DateTime<Utc>, to_time: DateTime<Utc>) -> Result<Vec<StateAuditRecord>, StateError> {
        let transitions = self.state_transitions.read().await;
        let mut audit_records = Vec::new();
        
        for (instance_id, instance_transitions) in transitions.iter() {
            for transition in instance_transitions {
                if transition.timestamp >= from_time && transition.timestamp <= to_time {
                    audit_records.push(StateAuditRecord {
                        instance_id: instance_id.clone(),
                        transition: transition.clone(),
                        context: StateAuditContext {
                            user_id: None, // 可以从上下文中获取
                            operation_id: transition.transition_id.clone(),
                            client_info: None,
                        },
                    });
                }
            }
        }
        
        // 按时间排序
        audit_records.sort_by(|a, b| a.transition.timestamp.cmp(&b.transition.timestamp));
        
        Ok(audit_records)
    }
}

/// 状态转换规则
pub struct StateTransitionRules {
    valid_transitions: HashMap<OverallTestStatus, Vec<OverallTestStatus>>,
}

impl StateTransitionRules {
    pub fn new() -> Self {
        let mut valid_transitions = HashMap::new();
        
        // 定义合法的状态转换
        valid_transitions.insert(
            OverallTestStatus::NotTested,
            vec![
                OverallTestStatus::WiringConfirmed,
                OverallTestStatus::HardPointTesting,
                OverallTestStatus::Skipped,
            ],
        );
        
        valid_transitions.insert(
            OverallTestStatus::WiringConfirmed,
            vec![
                OverallTestStatus::HardPointTesting,
                OverallTestStatus::Skipped,
            ],
        );
        
        valid_transitions.insert(
            OverallTestStatus::HardPointTesting,
            vec![
                OverallTestStatus::ManualTesting,
                OverallTestStatus::TestCompletedPassed,
                OverallTestStatus::TestCompletedFailed,
                OverallTestStatus::NotTested, // 重测
            ],
        );
        
        valid_transitions.insert(
            OverallTestStatus::ManualTesting,
            vec![
                OverallTestStatus::TestCompletedPassed,
                OverallTestStatus::TestCompletedFailed,
                OverallTestStatus::NotTested, // 重测
            ],
        );
        
        valid_transitions.insert(
            OverallTestStatus::TestCompletedPassed,
            vec![
                OverallTestStatus::NotTested, // 重测
            ],
        );
        
        valid_transitions.insert(
            OverallTestStatus::TestCompletedFailed,
            vec![
                OverallTestStatus::NotTested, // 重测
            ],
        );
        
        valid_transitions.insert(
            OverallTestStatus::Skipped,
            vec![
                OverallTestStatus::NotTested, // 重测
                OverallTestStatus::HardPointTesting,
            ],
        );
        
        Self { valid_transitions }
    }
    
    pub fn is_valid_transition(
        &self,
        from: OverallTestStatus,
        to: OverallTestStatus,
    ) -> Result<bool, StateError> {
        if let Some(allowed_transitions) = self.valid_transitions.get(&from) {
            Ok(allowed_transitions.contains(&to))
        } else {
            Ok(false)
        }
    }
}
```

#### 🧪 测试方案

##### 2.1.3 状态管理器测试
```rust
// src/services/state_management/tests/channel_state_manager_tests.rs
use super::super::*;
use crate::repositories::runtime_repository::MemoryRuntimeRepository;
use crate::repositories::persistent_repository::MemoryPersistentRepository;
use tokio;

struct StateManagerTestSuite;

impl StateManagerTestSuite {
    async fn create_test_state_manager() -> EnhancedChannelStateManager {
        let runtime_repo = Arc::new(MemoryRuntimeRepository::new());
        let persistent_repo = Arc::new(MemoryPersistentRepository::new());
        EnhancedChannelStateManager::new(runtime_repo, persistent_repo)
    }
    
    async fn create_test_instance(manager: &EnhancedChannelStateManager) -> String {
        let instance = ChannelTestInstance {
            instance_id: uuid::Uuid::new_v4().to_string(),
            definition_id: uuid::Uuid::new_v4().to_string(),
            test_batch_id: "test_batch".to_string(),
            overall_status: OverallTestStatus::NotTested,
            current_step_details: None,
            error_message: None,
            start_time: None,
            last_updated_time: chrono::Utc::now(),
            final_test_time: None,
            total_test_duration_ms: None,
            sub_test_results: std::collections::HashMap::new(),
            hardpoint_readings: None,
            manual_test_current_value_input: None,
            manual_test_current_value_output: None,
        };
        
        let instance_id = instance.instance_id.clone();
        manager.runtime_repo
            .save_channel_instance(&instance)
            .await
            .expect("保存测试实例失败");
        
        instance_id
    }
}

#[tokio::test]
async fn test_state_manager_basic_operations() {
    let manager = StateManagerTestSuite::create_test_state_manager().await;
    let instance_id = StateManagerTestSuite::create_test_instance(&manager).await;
    
    // 测试获取当前状态
    let current_state = manager.get_current_state(&instance_id).await.expect("获取状态失败");
    assert_eq!(current_state.overall_status, OverallTestStatus::NotTested, "初始状态应该是未测试");
    
    // 测试状态转换验证
    let can_transition = manager.can_transition_to(&instance_id, OverallTestStatus::HardPointTesting).await.expect("转换验证失败");
    assert!(can_transition, "应该允许从未测试转换到硬点测试");
    
    let cannot_transition = manager.can_transition_to(&instance_id, OverallTestStatus::TestCompletedPassed).await.expect("转换验证失败");
    assert!(!cannot_transition, "不应该允许直接从未测试转换到测试通过");
}

#[tokio::test]
async fn test_apply_test_outcome() {
    let manager = StateManagerTestSuite::create_test_state_manager().await;
    let instance_id = StateManagerTestSuite::create_test_instance(&manager).await;
    
    // 应用硬点测试结果
    let hardpoint_outcome = TestOutcome {
        test_item: SubTestItem::HardPoint,
        result: TestResult::Passed,
        measured_value: Some(50.0),
        expected_value: Some(50.0),
        tolerance: Some(1.0),
        duration_ms: Some(1000),
        error_message: None,
    };
    
    let transition = manager.apply_test_outcome(&instance_id, hardpoint_outcome).await.expect("应用测试结果失败");
    assert_eq!(transition.old_status, OverallTestStatus::NotTested, "原状态应该是未测试");
    assert_eq!(transition.new_status, OverallTestStatus::HardPointTesting, "新状态应该是硬点测试中");
    
    // 验证状态已更新
    let current_state = manager.get_current_state(&instance_id).await.expect("获取状态失败");
    assert_eq!(current_state.overall_status, OverallTestStatus::HardPointTesting, "状态应该已更新");
}

#[tokio::test]
async fn test_force_state_transition() {
    let manager = StateManagerTestSuite::create_test_state_manager().await;
    let instance_id = StateManagerTestSuite::create_test_instance(&manager).await;
    
    // 强制状态转换（跳过正常流程）
    let transition = manager.force_state_transition(
        &instance_id,
        OverallTestStatus::TestCompletedPassed,
        "管理员强制完成".to_string(),
    ).await.expect("强制状态转换失败");
    
    assert_eq!(transition.old_status, OverallTestStatus::NotTested, "原状态应该是未测试");
    assert_eq!(transition.new_status, OverallTestStatus::TestCompletedPassed, "新状态应该是测试通过");
    assert!(transition.reason.contains("强制转换"), "原因应该包含强制转换");
}

#[tokio::test]
async fn test_reset_for_retest() {
    let manager = StateManagerTestSuite::create_test_state_manager().await;
    let instance_id = StateManagerTestSuite::create_test_instance(&manager).await;
    
    // 先设置为已完成状态
    manager.force_state_transition(
        &instance_id,
        OverallTestStatus::TestCompletedPassed,
        "设置为已完成".to_string(),
    ).await.expect("设置状态失败");
    
    // 重置进行重测
    let transition = manager.reset_for_retest(&instance_id).await.expect("重置失败");
    assert_eq!(transition.new_status, OverallTestStatus::NotTested, "重置后状态应该是未测试");
    
    // 验证实例数据已清理
    let instance = manager.runtime_repo
        .get_channel_instance(&instance_id)
        .await
        .expect("获取实例失败")
        .expect("实例应该存在");
    
    assert!(instance.sub_test_results.is_empty(), "测试结果应该已清空");
    assert!(instance.start_time.is_none(), "开始时间应该已清空");
    assert!(instance.error_message.is_none(), "错误信息应该已清空");
}

#[tokio::test]
async fn test_batch_operations() {
    let manager = StateManagerTestSuite::create_test_state_manager().await;
    
    // 创建多个测试实例
    let instance_ids: Vec<String> = (0..3).map(|_| {
        futures::executor::block_on(StateManagerTestSuite::create_test_instance(&manager))
    }).collect();
    
    // 准备批量更新请求
    let updates: Vec<StateUpdateRequest> = instance_ids.iter().map(|id| {
        StateUpdateRequest {
            instance_id: id.clone(),
            outcome: TestOutcome {
                test_item: SubTestItem::HardPoint,
                result: TestResult::Passed,
                measured_value: Some(25.0),
                expected_value: Some(25.0),
                tolerance: Some(1.0),
                duration_ms: Some(500),
                error_message: None,
            },
            force_transition: false,
            reason: None,
        }
    }).collect();
    
    // 执行批量更新
    let transitions = manager.batch_state_update(updates).await.expect("批量更新失败");
    assert_eq!(transitions.len(), 3, "应该有3个状态转换");
    
    // 验证所有实例状态都已更新
    for instance_id in &instance_ids {
        let state = manager.get_current_state(instance_id).await.expect("获取状态失败");
        assert_eq!(state.overall_status, OverallTestStatus::HardPointTesting, "状态应该已更新");
    }
    
    // 测试批量重置
    let reset_transitions = manager.batch_reset_for_retest(instance_ids.clone()).await.expect("批量重置失败");
    assert_eq!(reset_transitions.len(), 3, "应该有3个重置转换");
    
    // 验证所有实例都已重置
    for instance_id in &instance_ids {
        let state = manager.get_current_state(instance_id).await.expect("获取状态失败");
        assert_eq!(state.overall_status, OverallTestStatus::NotTested, "状态应该已重置");
    }
}

#[tokio::test]
async fn test_state_events() {
    let manager = StateManagerTestSuite::create_test_state_manager().await;
    let instance_id = StateManagerTestSuite::create_test_instance(&manager).await;
    
    // 订阅状态变更事件
    let mut event_receiver = manager.subscribe_state_changes().await.expect("订阅事件失败");
    
    // 在另一个任务中监听事件
    let event_listener = tokio::spawn(async move {
        match tokio::time::timeout(std::time::Duration::from_secs(5), event_receiver.recv()).await {
            Ok(Ok(event)) => Some(event),
            _ => None,
        }
    });
    
    // 触发状态变更
    let outcome = TestOutcome {
        test_item: SubTestItem::HardPoint,
        result: TestResult::Passed,
        measured_value: Some(75.0),
        expected_value: Some(75.0),
        tolerance: Some(1.0),
        duration_ms: Some(800),
        error_message: None,
    };
    
    manager.apply_test_outcome(&instance_id, outcome).await.expect("应用测试结果失败");
    
    // 验证事件已发布
    let received_event = event_listener.await.expect("事件监听任务失败");
    assert!(received_event.is_some(), "应该接收到状态变更事件");
    
    let event = received_event.unwrap();
    assert_eq!(event.instance_id, instance_id, "事件实例ID应该匹配");
    assert_eq!(event.old_status, OverallTestStatus::NotTested, "事件原状态应该匹配");
    assert_eq!(event.new_status, OverallTestStatus::HardPointTesting, "事件新状态应该匹配");
}

#[tokio::test]
async fn test_state_validation() {
    let manager = StateManagerTestSuite::create_test_state_manager().await;
    let instance_id = StateManagerTestSuite::create_test_instance(&manager).await;
    
    // 创建不一致的状态（强制设置为通过，但添加失败的测试结果）
    manager.force_state_transition(
        &instance_id,
        OverallTestStatus::TestCompletedPassed,
        "测试设置".to_string(),
    ).await.expect("强制转换失败");
    
    // 手动添加失败的测试结果来制造不一致
    let mut instance = manager.runtime_repo
        .get_channel_instance(&instance_id)
        .await
        .expect("获取实例失败")
        .expect("实例应该存在");
    
    instance.sub_test_results.insert(SubTestItem::HardPoint, SubTestExecutionResult {
        result: TestResult::Failed,
        measured_value: Some(10.0),
        expected_value: Some(50.0),
        tolerance: Some(1.0),
        timestamp: chrono::Utc::now(),
        error_message: Some("测试失败".to_string()),
        duration_ms: Some(1000),
    });
    
    manager.runtime_repo
        .save_channel_instance(&instance)
        .await
        .expect("保存实例失败");
    
    // 验证状态一致性
    let validation_result = manager.validate_state_consistency(&instance_id).await.expect("状态验证失败");
    assert!(!validation_result.is_valid, "状态应该被检测为不一致");
    assert!(!validation_result.issues.is_empty(), "应该有验证问题");
    assert!(!validation_result.recommendations.is_empty(), "应该有修复建议");
}

#[tokio::test]
async fn test_transition_history() {
    let manager = StateManagerTestSuite::create_test_state_manager().await;
    let instance_id = StateManagerTestSuite::create_test_instance(&manager).await;
    
    // 执行多次状态转换
    let outcome1 = TestOutcome {
        test_item: SubTestItem::HardPoint,
        result: TestResult::InProgress,
        measured_value: None,
        expected_value: None,
        tolerance: None,
        duration_ms: None,
        error_message: None,
    };
    
    manager.apply_test_outcome(&instance_id, outcome1).await.expect("第一次转换失败");
    
    let outcome2 = TestOutcome {
        test_item: SubTestItem::HardPoint,
        result: TestResult::Passed,
        measured_value: Some(50.0),
        expected_value: Some(50.0),
        tolerance: Some(1.0),
        duration_ms: Some(1000),
        error_message: None,
    };
    
    manager.apply_test_outcome(&instance_id, outcome2).await.expect("第二次转换失败");
    
    // 获取转换历史
    let history = manager.get_transition_history(&instance_id).await.expect("获取历史失败");
    assert_eq!(history.len(), 2, "应该有2个状态转换记录");
    
    // 验证转换顺序
    assert_eq!(history[0].old_status, OverallTestStatus::NotTested, "第一次转换的原状态");
    assert_eq!(history[0].new_status, OverallTestStatus::HardPointTesting, "第一次转换的新状态");
    assert_eq!(history[1].old_status, OverallTestStatus::HardPointTesting, "第二次转换的原状态");
}

/// 压力测试
#[tokio::test]
async fn test_concurrent_state_operations() {
    let manager = Arc::new(StateManagerTestSuite::create_test_state_manager().await);
    
    // 创建多个实例
    let instance_ids: Vec<String> = (0..100).map(|_| {
        futures::executor::block_on(StateManagerTestSuite::create_test_instance(&manager))
    }).collect();
    
    // 并发执行状态操作
    let tasks: Vec<_> = instance_ids.into_iter().map(|instance_id| {
        let manager_clone = manager.clone();
        tokio::spawn(async move {
            let outcome = TestOutcome {
                test_item: SubTestItem::HardPoint,
                result: TestResult::Passed,
                measured_value: Some(50.0),
                expected_value: Some(50.0),
                tolerance: Some(1.0),
                duration_ms: Some(1000),
                error_message: None,
            };
            
            manager_clone.apply_test_outcome(&instance_id, outcome).await
        })
    }).collect();
    
    // 等待所有任务完成
    let results = futures::future::join_all(tasks).await;
    
    // 验证所有操作都成功
    for result in results {
        assert!(result.expect("任务应该成功").is_ok(), "所有状态操作都应该成功");
    }
}
```

### ✅ 步骤2.1完成标准

1. **接口完整性**: 所有IChannelStateManager方法都有完整实现
2. **状态一致性**: 严格的状态转换规则和验证机制
3. **事件机制**: 完整的状态变更事件发布和订阅
4. **并发安全**: 支持高并发的状态操作
5. **审计跟踪**: 完整的状态变更历史记录

---

*本文档持续更新中...* 

## 🚀 Phase 3: 任务调度器重构

### 步骤 3.1: 创建高级任务调度器

#### 🎯 目标
基于原有C#代码逻辑，设计更稳定可靠的任务调度和执行引擎。

#### 📝 具体实施

##### 3.1.1 定义任务调度器核心接口
```rust
// src/services/task_scheduling/task_scheduler.rs
use async_trait::async_trait;
use crate::models::structs::{TestTask, TaskHandle, TaskInfo, BatchProgress};
use crate::models::enums::{TaskStatus, TaskPriority};
use crate::models::errors::SchedulerError;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Semaphore, RwLock, broadcast};
use chrono::{DateTime, Utc};

/// 高级任务调度器接口
#[async_trait]
pub trait ITaskScheduler: Send + Sync {
    // 任务调度
    async fn schedule_test_task(&self, task: TestTask) -> Result<TaskHandle, SchedulerError>;
    async fn schedule_batch_tasks(&self, batch_id: &str, tasks: Vec<TestTask>) -> Result<Vec<TaskHandle>, SchedulerError>;
    async fn schedule_priority_task(&self, task: TestTask, priority: TaskPriority) -> Result<TaskHandle, SchedulerError>;
    
    // 任务控制
    async fn pause_task(&self, task_handle: TaskHandle) -> Result<(), SchedulerError>;
    async fn resume_task(&self, task_handle: TaskHandle) -> Result<(), SchedulerError>;
    async fn cancel_task(&self, task_handle: TaskHandle) -> Result<(), SchedulerError>;
    async fn retry_task(&self, task_handle: TaskHandle) -> Result<TaskHandle, SchedulerError>;
    
    // 批次控制 (参考原有C#逻辑)
    async fn pause_batch(&self, batch_id: &str) -> Result<(), SchedulerError>;
    async fn resume_batch(&self, batch_id: &str) -> Result<(), SchedulerError>;
    async fn cancel_batch(&self, batch_id: &str) -> Result<(), SchedulerError>;
    async fn restart_batch(&self, batch_id: &str) -> Result<(), SchedulerError>;
    
    // 状态查询
    async fn get_task_status(&self, task_handle: TaskHandle) -> Result<TaskStatus, SchedulerError>;
    async fn get_task_info(&self, task_handle: TaskHandle) -> Result<TaskInfo, SchedulerError>;
    async fn get_batch_progress(&self, batch_id: &str) -> Result<BatchProgress, SchedulerError>;
    async fn list_active_tasks(&self) -> Result<Vec<TaskInfo>, SchedulerError>;
    async fn list_batch_tasks(&self, batch_id: &str) -> Result<Vec<TaskInfo>, SchedulerError>;
    
    // 资源管理
    async fn set_concurrency_limit(&self, limit: usize) -> Result<(), SchedulerError>;
    async fn get_system_load(&self) -> Result<SystemLoad, SchedulerError>;
    async fn get_resource_usage(&self) -> Result<ResourceUsage, SchedulerError>;
    
    // 健康检查和监控
    async fn health_check(&self) -> Result<SchedulerHealth, SchedulerError>;
    async fn get_performance_metrics(&self) -> Result<PerformanceMetrics, SchedulerError>;
    
    // 事件订阅
    async fn subscribe_task_events(&self) -> Result<broadcast::Receiver<TaskEvent>, SchedulerError>;
    async fn subscribe_batch_events(&self) -> Result<broadcast::Receiver<BatchEvent>, SchedulerError>;
}
```

---

*本文档持续更新中...*

##### 3.1.2 定义任务调度器数据结构
```rust
/// 测试任务定义
#[derive(Debug, Clone)]
pub struct TestTask {
    pub task_id: String,
    pub instance_id: String,
    pub definition_id: String,
    pub batch_id: String,
    pub test_items: Vec<SubTestItem>,
    pub priority: TaskPriority,
    pub retry_count: u32,
    pub max_retries: u32,
    pub timeout_ms: Option<u64>,
    pub dependencies: Vec<String>, // 依赖的任务ID
    pub metadata: HashMap<String, String>,
}

/// 任务句柄
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TaskHandle {
    pub id: u64,
}

/// 任务信息
#[derive(Debug, Clone)]
pub struct TaskInfo {
    pub handle: TaskHandle,
    pub task: TestTask,
    pub status: TaskStatus,
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub error_message: Option<String>,
    pub progress_percentage: f64,
    pub resource_usage: TaskResourceUsage,
}

/// 批次进度
#[derive(Debug, Clone)]
pub struct BatchProgress {
    pub batch_id: String,
    pub total_tasks: usize,
    pub pending_tasks: usize,
    pub running_tasks: usize,
    pub completed_tasks: usize,
    pub failed_tasks: usize,
    pub cancelled_tasks: usize,
    pub progress_percentage: f64,
    pub estimated_completion_time: Option<DateTime<Utc>>,
}

/// 系统负载信息
#[derive(Debug, Clone)]
pub struct SystemLoad {
    pub active_tasks: usize,
    pub pending_tasks: usize,
    pub cpu_usage_percentage: f64,
    pub memory_usage_bytes: usize,
    pub available_slots: usize,
}

/// 资源使用情况
#[derive(Debug, Clone)]
pub struct ResourceUsage {
    pub total_memory_bytes: usize,
    pub used_memory_bytes: usize,
    pub active_connections: usize,
    pub thread_pool_usage: f64,
}

/// 调度器健康状态
#[derive(Debug, Clone)]
pub struct SchedulerHealth {
    pub is_healthy: bool,
    pub uptime_seconds: u64,
    pub total_tasks_processed: u64,
    pub error_rate_percentage: f64,
    pub average_task_duration_ms: f64,
    pub issues: Vec<HealthIssue>,
}

/// 性能指标
#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    pub tasks_per_second: f64,
    pub average_queue_time_ms: f64,
    pub average_execution_time_ms: f64,
    pub success_rate_percentage: f64,
    pub retry_rate_percentage: f64,
}

/// 任务事件
#[derive(Debug, Clone)]
pub struct TaskEvent {
    pub task_handle: TaskHandle,
    pub event_type: TaskEventType,
    pub timestamp: DateTime<Utc>,
    pub details: Option<String>,
}

/// 批次事件
#[derive(Debug, Clone)]
pub struct BatchEvent {
    pub batch_id: String,
    pub event_type: BatchEventType,
    pub timestamp: DateTime<Utc>,
    pub details: Option<String>,
}

#[derive(Debug, Clone)]
pub enum TaskEventType {
    Scheduled,
    Started,
    Completed,
    Failed,
    Cancelled,
    Paused,
    Resumed,
    Retrying,
}

#[derive(Debug, Clone)]
pub enum BatchEventType {
    Started,
    Completed,
    Paused,
    Resumed,
    Cancelled,
    ProgressUpdated,
}
```

---

*本文档持续更新中...*

##### 3.1.3 实现高级任务调度器 (Part 1)
```rust
// src/services/task_scheduling/advanced_task_scheduler.rs
use super::*;
use crate::services::test_execution::ITestExecutor;
use crate::services::state_management::IChannelStateManager;
use crate::repositories::IRuntimeRepository;
use std::collections::{HashMap, VecDeque, BinaryHeap};
use std::cmp::Reverse;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use tokio::sync::Mutex;
use tokio::time::{Duration, Instant};

/// 高级任务调度器实现
/// 参考原有C#代码中的TestTaskManager逻辑进行优化
pub struct AdvancedTaskScheduler {
    // 核心组件
    state_manager: Arc<dyn IChannelStateManager>,
    runtime_repo: Arc<dyn IRuntimeRepository>,
    test_executors: HashMap<SubTestItem, Arc<dyn ITestExecutor>>,
    
    // 任务管理
    task_queue: Arc<Mutex<PriorityQueue<ScheduledTask>>>,
    active_tasks: Arc<RwLock<HashMap<TaskHandle, RunningTask>>>,
    completed_tasks: Arc<RwLock<HashMap<TaskHandle, CompletedTask>>>,
    batch_tasks: Arc<RwLock<HashMap<String, Vec<TaskHandle>>>>,
    
    // 资源控制
    concurrency_semaphore: Arc<Semaphore>,
    max_concurrent_tasks: AtomicUsize,
    
    // 状态统计
    next_task_id: AtomicU64,
    total_tasks_processed: AtomicU64,
    startup_time: Instant,
    
    // 事件发布
    task_event_broadcaster: broadcast::Sender<TaskEvent>,
    batch_event_broadcaster: broadcast::Sender<BatchEvent>,
    
    // 重试策略
    retry_policy: Arc<dyn IRetryPolicy>,
    
    // 健康监控
    health_monitor: Arc<HealthMonitor>,
}

/// 调度任务包装器
#[derive(Debug, Clone)]
struct ScheduledTask {
    handle: TaskHandle,
    task: TestTask,
    scheduled_at: DateTime<Utc>,
    retry_attempt: u32,
}

/// 运行中任务
#[derive(Debug, Clone)]
struct RunningTask {
    info: TaskInfo,
    started_at: Instant,
    abort_handle: Option<tokio::task::AbortHandle>,
}

/// 已完成任务
#[derive(Debug, Clone)]
struct CompletedTask {
    info: TaskInfo,
    result: TaskResult,
}

/// 优先级队列实现
struct PriorityQueue<T> {
    heap: BinaryHeap<Reverse<PrioritizedItem<T>>>,
}

#[derive(Debug, Clone)]
struct PrioritizedItem<T> {
    priority: TaskPriority,
    created_at: DateTime<Utc>,
    item: T,
}

impl<T> Ord for PrioritizedItem<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.priority.cmp(&other.priority)
            .then_with(|| self.created_at.cmp(&other.created_at))
    }
}

impl<T> PartialOrd for PrioritizedItem<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<T> PartialEq for PrioritizedItem<T> {
    fn eq(&self, other: &Self) -> bool {
        self.priority == other.priority && self.created_at == other.created_at
    }
}

impl<T> Eq for PrioritizedItem<T> {}

impl AdvancedTaskScheduler {
    pub fn new(
        state_manager: Arc<dyn IChannelStateManager>,
        runtime_repo: Arc<dyn IRuntimeRepository>,
        test_executors: HashMap<SubTestItem, Arc<dyn ITestExecutor>>,
        max_concurrent_tasks: usize,
    ) -> Self {
        let (task_event_broadcaster, _) = broadcast::channel(1000);
        let (batch_event_broadcaster, _) = broadcast::channel(1000);
        
        Self {
            state_manager,
            runtime_repo,
            test_executors,
            task_queue: Arc::new(Mutex::new(PriorityQueue::new())),
            active_tasks: Arc::new(RwLock::new(HashMap::new())),
            completed_tasks: Arc::new(RwLock::new(HashMap::new())),
            batch_tasks: Arc::new(RwLock::new(HashMap::new())),
            concurrency_semaphore: Arc::new(Semaphore::new(max_concurrent_tasks)),
            max_concurrent_tasks: AtomicUsize::new(max_concurrent_tasks),
            next_task_id: AtomicU64::new(1),
            total_tasks_processed: AtomicU64::new(0),
            startup_time: Instant::now(),
            task_event_broadcaster,
            batch_event_broadcaster,
            retry_policy: Arc::new(DefaultRetryPolicy::new()),
            health_monitor: Arc::new(HealthMonitor::new()),
        }
    }
    
    /// 启动调度器工作循环
    pub async fn start(&self) -> Result<(), SchedulerError> {
        let scheduler = Arc::new(self);
        
        // 启动任务处理循环
        let task_processor = scheduler.clone();
        tokio::spawn(async move {
            task_processor.task_processing_loop().await;
        });
        
        // 启动健康监控
        let health_monitor = scheduler.clone();
        tokio::spawn(async move {
            health_monitor.health_monitoring_loop().await;
        });
        
        Ok(())
    }
}
```

---

*本文档持续更新中...*

##### 3.1.4 任务调度器接口实现 (Part 2)
```rust
#[async_trait]
impl ITaskScheduler for AdvancedTaskScheduler {
    async fn schedule_test_task(&self, task: TestTask) -> Result<TaskHandle, SchedulerError> {
        let task_handle = TaskHandle {
            id: self.next_task_id.fetch_add(1, Ordering::Relaxed),
        };
        
        let scheduled_task = ScheduledTask {
            handle: task_handle,
            task: task.clone(),
            scheduled_at: Utc::now(),
            retry_attempt: 0,
        };
        
        // 添加到队列
        {
            let mut queue = self.task_queue.lock().await;
            queue.push(scheduled_task);
        }
        
        // 添加到批次映射
        {
            let mut batch_tasks = self.batch_tasks.write().await;
            batch_tasks.entry(task.batch_id.clone())
                .or_insert_with(Vec::new)
                .push(task_handle);
        }
        
        // 发布调度事件
        let _ = self.task_event_broadcaster.send(TaskEvent {
            task_handle,
            event_type: TaskEventType::Scheduled,
            timestamp: Utc::now(),
            details: None,
        });
        
        Ok(task_handle)
    }
    
    async fn schedule_batch_tasks(&self, batch_id: &str, tasks: Vec<TestTask>) -> Result<Vec<TaskHandle>, SchedulerError> {
        let mut handles = Vec::new();
        
        for task in tasks {
            let handle = self.schedule_test_task(task).await?;
            handles.push(handle);
        }
        
        // 发布批次开始事件
        let _ = self.batch_event_broadcaster.send(BatchEvent {
            batch_id: batch_id.to_string(),
            event_type: BatchEventType::Started,
            timestamp: Utc::now(),
            details: Some(format!("批次开始，包含 {} 个任务", handles.len())),
        });
        
        Ok(handles)
    }
    
    async fn schedule_priority_task(&self, mut task: TestTask, priority: TaskPriority) -> Result<TaskHandle, SchedulerError> {
        task.priority = priority;
        self.schedule_test_task(task).await
    }
    
    async fn pause_task(&self, task_handle: TaskHandle) -> Result<(), SchedulerError> {
        let mut active_tasks = self.active_tasks.write().await;
        if let Some(running_task) = active_tasks.get_mut(&task_handle) {
            running_task.info.status = TaskStatus::Paused;
            
            // 发布暂停事件
            let _ = self.task_event_broadcaster.send(TaskEvent {
                task_handle,
                event_type: TaskEventType::Paused,
                timestamp: Utc::now(),
                details: None,
            });
            
            Ok(())
        } else {
            Err(SchedulerError::TaskNotFound(task_handle))
        }
    }
    
    async fn resume_task(&self, task_handle: TaskHandle) -> Result<(), SchedulerError> {
        let mut active_tasks = self.active_tasks.write().await;
        if let Some(running_task) = active_tasks.get_mut(&task_handle) {
            running_task.info.status = TaskStatus::Running;
            
            // 发布恢复事件
            let _ = self.task_event_broadcaster.send(TaskEvent {
                task_handle,
                event_type: TaskEventType::Resumed,
                timestamp: Utc::now(),
                details: None,
            });
            
            Ok(())
        } else {
            Err(SchedulerError::TaskNotFound(task_handle))
        }
    }
    
    async fn cancel_task(&self, task_handle: TaskHandle) -> Result<(), SchedulerError> {
        // 首先尝试从队列中移除
        {
            let mut queue = self.task_queue.lock().await;
            if queue.remove_by_handle(task_handle) {
                // 任务在队列中，直接移除
                let _ = self.task_event_broadcaster.send(TaskEvent {
                    task_handle,
                    event_type: TaskEventType::Cancelled,
                    timestamp: Utc::now(),
                    details: Some("任务在队列中被取消".to_string()),
                });
                return Ok(());
            }
        }
        
        // 尝试取消正在运行的任务
        let mut active_tasks = self.active_tasks.write().await;
        if let Some(running_task) = active_tasks.remove(&task_handle) {
            if let Some(abort_handle) = running_task.abort_handle {
                abort_handle.abort();
            }
            
            // 发布取消事件
            let _ = self.task_event_broadcaster.send(TaskEvent {
                task_handle,
                event_type: TaskEventType::Cancelled,
                timestamp: Utc::now(),
                details: Some("运行中任务被取消".to_string()),
            });
            
            Ok(())
        } else {
            Err(SchedulerError::TaskNotFound(task_handle))
        }
    }
    
    async fn retry_task(&self, task_handle: TaskHandle) -> Result<TaskHandle, SchedulerError> {
        // 获取原任务信息
        let completed_tasks = self.completed_tasks.read().await;
        if let Some(completed_task) = completed_tasks.get(&task_handle) {
            let mut new_task = completed_task.info.task.clone();
            new_task.retry_count = 0; // 重置重试计数
            
            drop(completed_tasks);
            self.schedule_test_task(new_task).await
        } else {
            Err(SchedulerError::TaskNotFound(task_handle))
        }
    }
}
```

---

*本文档持续更新中...*

#### 🧪 测试方案

##### 3.1.5 任务调度器测试
```rust
// src/services/task_scheduling/tests/advanced_task_scheduler_tests.rs
use super::super::*;
use tokio;

struct TaskSchedulerTestSuite;

impl TaskSchedulerTestSuite {
    async fn create_test_scheduler() -> AdvancedTaskScheduler {
        let state_manager = Arc::new(MockChannelStateManager::new());
        let runtime_repo = Arc::new(MockRuntimeRepository::new());
        let test_executors = HashMap::new();
        
        AdvancedTaskScheduler::new(
            state_manager,
            runtime_repo,
            test_executors,
            10, // max concurrent tasks
        )
    }
    
    fn create_test_task(batch_id: &str) -> TestTask {
        TestTask {
            task_id: uuid::Uuid::new_v4().to_string(),
            instance_id: uuid::Uuid::new_v4().to_string(),
            definition_id: uuid::Uuid::new_v4().to_string(),
            batch_id: batch_id.to_string(),
            test_items: vec![SubTestItem::HardPoint],
            priority: TaskPriority::Normal,
            retry_count: 0,
            max_retries: 3,
            timeout_ms: Some(30000),
            dependencies: vec![],
            metadata: HashMap::new(),
        }
    }
}

#[tokio::test]
async fn test_task_scheduling_basic() {
    let scheduler = TaskSchedulerTestSuite::create_test_scheduler().await;
    let task = TaskSchedulerTestSuite::create_test_task("test_batch");
    
    // 测试任务调度
    let task_handle = scheduler.schedule_test_task(task).await.expect("调度任务失败");
    assert!(task_handle.id > 0, "任务句柄ID应该大于0");
    
    // 测试任务状态查询
    let status = scheduler.get_task_status(task_handle).await.expect("获取任务状态失败");
    assert_eq!(status, TaskStatus::Pending, "新调度的任务应该是待执行状态");
}

#[tokio::test]
async fn test_batch_operations() {
    let scheduler = TaskSchedulerTestSuite::create_test_scheduler().await;
    let batch_id = "test_batch_001";
    
    // 创建批次任务
    let tasks = (0..5).map(|_| TaskSchedulerTestSuite::create_test_task(batch_id)).collect();
    let handles = scheduler.schedule_batch_tasks(batch_id, tasks).await.expect("批次调度失败");
    
    assert_eq!(handles.len(), 5, "应该调度5个任务");
    
    // 测试批次进度查询
    let progress = scheduler.get_batch_progress(batch_id).await.expect("获取批次进度失败");
    assert_eq!(progress.total_tasks, 5, "批次应该包含5个任务");
    assert_eq!(progress.pending_tasks, 5, "应该有5个待执行任务");
    
    // 测试批次暂停
    scheduler.pause_batch(batch_id).await.expect("暂停批次失败");
    
    // 测试批次恢复
    scheduler.resume_batch(batch_id).await.expect("恢复批次失败");
    
    // 测试批次取消
    scheduler.cancel_batch(batch_id).await.expect("取消批次失败");
}

#[tokio::test]
async fn test_concurrent_execution() {
    let scheduler = Arc::new(TaskSchedulerTestSuite::create_test_scheduler().await);
    
    // 并发调度多个任务
    let tasks: Vec<_> = (0..20).map(|i| {
        let scheduler_clone = scheduler.clone();
        let task = TaskSchedulerTestSuite::create_test_task(&format!("batch_{}", i % 3));
        tokio::spawn(async move {
            scheduler_clone.schedule_test_task(task).await
        })
    }).collect();
    
    // 等待所有任务调度完成
    let results = futures::future::join_all(tasks).await;
    
    // 验证所有任务都成功调度
    for result in results {
        assert!(result.expect("任务调度任务失败").is_ok(), "所有任务调度都应该成功");
    }
    
    // 检查系统负载
    let load = scheduler.get_system_load().await.expect("获取系统负载失败");
    assert!(load.pending_tasks <= 20, "待执行任务数应该不超过20");
}

#[tokio::test]
async fn test_retry_mechanism() {
    let scheduler = TaskSchedulerTestSuite::create_test_scheduler().await;
    let task = TaskSchedulerTestSuite::create_test_task("retry_test_batch");
    
    // 调度任务
    let original_handle = scheduler.schedule_test_task(task).await.expect("调度任务失败");
    
    // 模拟任务完成（这里需要实际的执行逻辑）
    // 在实际测试中，需要等待任务完成或手动设置为完成状态
    
    // 测试重试
    let retry_handle = scheduler.retry_task(original_handle).await.expect("重试任务失败");
    assert_ne!(original_handle.id, retry_handle.id, "重试任务应该有新的句柄");
}

#[tokio::test]
async fn test_error_recovery() {
    let scheduler = TaskSchedulerTestSuite::create_test_scheduler().await;
    
    // 测试健康检查
    let health = scheduler.health_check().await.expect("健康检查失败");
    assert!(health.is_healthy, "新创建的调度器应该是健康的");
    
    // 测试性能指标
    let metrics = scheduler.get_performance_metrics().await.expect("获取性能指标失败");
    assert!(metrics.tasks_per_second >= 0.0, "任务处理速率应该不为负");
}

#[tokio::test]
async fn test_event_subscription() {
    let scheduler = TaskSchedulerTestSuite::create_test_scheduler().await;
    
    // 订阅任务事件
    let mut task_receiver = scheduler.subscribe_task_events().await.expect("订阅任务事件失败");
    
    // 订阅批次事件
    let mut batch_receiver = scheduler.subscribe_batch_events().await.expect("订阅批次事件失败");
    
    // 在另一个任务中监听事件
    let task_listener = tokio::spawn(async move {
        match tokio::time::timeout(std::time::Duration::from_secs(5), task_receiver.recv()).await {
            Ok(Ok(event)) => Some(event),
            _ => None,
        }
    });
    
    let batch_listener = tokio::spawn(async move {
        match tokio::time::timeout(std::time::Duration::from_secs(5), batch_receiver.recv()).await {
            Ok(Ok(event)) => Some(event),
            _ => None,
        }
    });
    
    // 触发事件
    let task = TaskSchedulerTestSuite::create_test_task("event_test_batch");
    let batch_id = task.batch_id.clone();
    let tasks = vec![task];
    
    scheduler.schedule_batch_tasks(&batch_id, tasks).await.expect("调度批次失败");
    
    // 验证事件
    let task_event = task_listener.await.expect("任务监听器失败");
    let batch_event = batch_listener.await.expect("批次监听器失败");
    
    assert!(task_event.is_some(), "应该接收到任务事件");
    assert!(batch_event.is_some(), "应该接收到批次事件");
}

/// 性能压力测试
#[tokio::test]
async fn test_performance_stress() {
    let scheduler = Arc::new(TaskSchedulerTestSuite::create_test_scheduler().await);
    let start_time = std::time::Instant::now();
    
    // 调度大量任务
    let batch_size = 1000;
    let tasks: Vec<TestTask> = (0..batch_size)
        .map(|i| TaskSchedulerTestSuite::create_test_task(&format!("stress_batch_{}", i % 10)))
        .collect();
    
    let handles = scheduler.schedule_batch_tasks("stress_test", tasks).await.expect("压力测试调度失败");
    
    let schedule_duration = start_time.elapsed();
    println!("调度{}个任务耗时: {:?}", batch_size, schedule_duration);
    
    assert_eq!(handles.len(), batch_size, "应该调度所有任务");
    assert!(schedule_duration.as_millis() < 5000, "调度应该在5秒内完成");
    
    // 检查系统负载
    let load = scheduler.get_system_load().await.expect("获取系统负载失败");
    assert_eq!(load.pending_tasks, batch_size, "所有任务都应该在待执行状态");
}
```

### ✅ 步骤3.1完成标准

1. **调度功能**: 支持优先级调度和批次管理
2. **并发控制**: 可配置的并发限制和资源管理
3. **错误恢复**: 完善的重试机制和错误处理
4. **监控功能**: 健康检查和性能指标
5. **事件系统**: 完整的任务和批次事件通知
6. **性能表现**: 支持大规模任务调度和执行
7. **稳定性**: 参考原有C#逻辑并增强错误处理能力

---

*本文档持续更新中...*

---

*本文档持续更新中...*

## 🚀 Phase 4: 应用服务层重构

### 步骤 4.1: 创建应用服务层

#### 🎯 目标
通过服务组合模式实现复杂业务流程，确保单一职责和高可维护性。

#### 📝 具体实施

##### 4.1.1 定义核心应用服务接口
```rust
// src/services/application/mod.rs
use async_trait::async_trait;
use crate::models::structs::{TestBatchInfo, ChannelTestInstance, ChannelPointDefinition};
use crate::models::errors::ApplicationError;
use std::sync::Arc;

/// 测试编排服务接口
#[async_trait]
pub trait ITestOrchestrationService: Send + Sync {
    // 批次管理
    async fn create_test_batch(&self, product_model: Option<String>) -> Result<TestBatchInfo, ApplicationError>;
    async fn prepare_instances_for_batch(&self, batch_id: &str, definitions: &[ChannelPointDefinition]) -> Result<Vec<String>, ApplicationError>;
    async fn start_batch_testing(&self, batch_id: &str) -> Result<(), ApplicationError>;
    async fn pause_batch_testing(&self, batch_id: &str) -> Result<(), ApplicationError>;
    async fn resume_batch_testing(&self, batch_id: &str) -> Result<(), ApplicationError>;
    async fn cancel_batch_testing(&self, batch_id: &str) -> Result<(), ApplicationError>;
    
    // 实例管理
    async fn start_instance_testing(&self, instance_id: &str) -> Result<(), ApplicationError>;
    async fn pause_instance_testing(&self, instance_id: &str) -> Result<(), ApplicationError>;
    async fn reset_instance_for_retest(&self, instance_id: &str) -> Result<(), ApplicationError>;
    
    // 批量操作
    async fn batch_reset_instances(&self, instance_ids: Vec<String>) -> Result<(), ApplicationError>;
    async fn batch_start_testing(&self, instance_ids: Vec<String>) -> Result<(), ApplicationError>;
}

/// 数据管理服务接口
#[async_trait]
pub trait IDataManagementService: Send + Sync {
    // 配置导入导出
    async fn import_configuration_from_excel(&self, file_path: &str, config_name: &str) -> Result<Vec<ChannelPointDefinition>, ApplicationError>;
    async fn export_configuration_to_excel(&self, config_name: &str, file_path: &str) -> Result<(), ApplicationError>;
    
    // 配置管理
    async fn save_configuration_set(&self, name: &str, definitions: &[ChannelPointDefinition]) -> Result<(), ApplicationError>;
    async fn load_configuration_set(&self, name: &str) -> Result<Vec<ChannelPointDefinition>, ApplicationError>;
    async fn list_configuration_sets(&self) -> Result<Vec<String>, ApplicationError>;
    async fn delete_configuration_set(&self, name: &str) -> Result<(), ApplicationError>;
    
    // 数据验证
    async fn validate_configuration(&self, definitions: &[ChannelPointDefinition]) -> Result<ValidationReport, ApplicationError>;
    async fn check_data_consistency(&self) -> Result<ConsistencyReport, ApplicationError>;
}

/// 系统管理服务接口
#[async_trait]
pub trait ISystemManagementService: Send + Sync {
    // 系统配置
    async fn get_system_configuration(&self) -> Result<SystemConfiguration, ApplicationError>;
    async fn update_system_configuration(&self, config: &SystemConfiguration) -> Result<(), ApplicationError>;
    
    // 系统监控
    async fn get_system_status(&self) -> Result<SystemStatus, ApplicationError>;
    async fn get_performance_metrics(&self) -> Result<SystemPerformanceMetrics, ApplicationError>;
    
    // 维护操作
    async fn cleanup_old_data(&self, retention_days: u32) -> Result<CleanupReport, ApplicationError>;
    async fn backup_data(&self, backup_path: &str) -> Result<BackupReport, ApplicationError>;
    async fn restore_data(&self, backup_path: &str) -> Result<RestoreReport, ApplicationError>;
}

/// 验证报告
#[derive(Debug, Clone)]
pub struct ValidationReport {
    pub is_valid: bool,
    pub total_checked: usize,
    pub issues: Vec<ValidationIssue>,
    pub warnings: Vec<ValidationWarning>,
    pub summary: String,
}

/// 一致性报告
#[derive(Debug, Clone)]
pub struct ConsistencyReport {
    pub is_consistent: bool,
    pub checked_components: Vec<String>,
    pub inconsistencies: Vec<InconsistencyIssue>,
    pub recommendations: Vec<String>,
}

/// 系统配置
#[derive(Debug, Clone)]
pub struct SystemConfiguration {
    pub max_concurrent_tests: usize,
    pub default_timeout_ms: u64,
    pub auto_backup_enabled: bool,
    pub backup_retention_days: u32,
    pub log_level: String,
    pub plc_connection_timeout_ms: u64,
}

/// 系统状态
#[derive(Debug, Clone)]
pub struct SystemStatus {
    pub is_healthy: bool,
    pub uptime_seconds: u64,
    pub active_batches: usize,
    pub active_tests: usize,
    pub memory_usage_mb: f64,
    pub cpu_usage_percentage: f64,
    pub disk_usage_percentage: f64,
}

/// 系统性能指标
#[derive(Debug, Clone)]
pub struct SystemPerformanceMetrics {
    pub tests_completed_today: u64,
    pub average_test_duration_ms: f64,
    pub success_rate_percentage: f64,
    pub error_rate_percentage: f64,
    pub throughput_tests_per_hour: f64,
}

/// 清理报告
#[derive(Debug, Clone)]
pub struct CleanupReport {
    pub cleaned_files: usize,
    pub freed_space_mb: f64,
    pub deleted_records: usize,
    pub cleanup_duration_ms: u64,
}

/// 备份报告
#[derive(Debug, Clone)]
pub struct BackupReport {
    pub backup_path: String,
    pub backup_size_mb: f64,
    pub included_files: usize,
    pub backup_duration_ms: u64,
    pub is_successful: bool,
}

/// 恢复报告
#[derive(Debug, Clone)]
pub struct RestoreReport {
    pub restored_files: usize,
    pub restored_records: usize,
    pub restore_duration_ms: u64,
    pub is_successful: bool,
    pub verification_passed: bool,
}
```

---

*本文档持续更新中...*

##### 4.1.2 实现测试编排服务
```rust
// src/services/application/test_orchestration_service.rs
use super::*;
use crate::services::state_management::IChannelStateManager;
use crate::services::task_scheduling::ITaskScheduler;
use crate::repositories::{IConfigurationRepository, IRuntimeRepository};
use crate::models::structs::{TestTask, SubTestItem, TaskPriority};
use crate::models::enums::OverallTestStatus;

/// 测试编排服务实现
pub struct TestOrchestrationService {
    config_repo: Arc<dyn IConfigurationRepository>,
    runtime_repo: Arc<dyn IRuntimeRepository>,
    state_manager: Arc<dyn IChannelStateManager>,
    task_scheduler: Arc<dyn ITaskScheduler>,
}

impl TestOrchestrationService {
    pub fn new(
        config_repo: Arc<dyn IConfigurationRepository>,
        runtime_repo: Arc<dyn IRuntimeRepository>,
        state_manager: Arc<dyn IChannelStateManager>,
        task_scheduler: Arc<dyn ITaskScheduler>,
    ) -> Self {
        Self {
            config_repo,
            runtime_repo,
            state_manager,
            task_scheduler,
        }
    }
}

#[async_trait]
impl ITestOrchestrationService for TestOrchestrationService {
    async fn create_test_batch(&self, product_model: Option<String>) -> Result<TestBatchInfo, ApplicationError> {
        let batch_id = uuid::Uuid::new_v4().to_string();
        
        let batch_info = TestBatchInfo {
            batch_id: batch_id.clone(),
            product_model,
            serial_number: None,
            customer_name: None,
            creation_time: chrono::Utc::now(),
            status_summary: Some("已创建".to_string()),
            total_points: 0,
            tested_points: 0,
            passed_points: 0,
            failed_points: 0,
        };
        
        self.runtime_repo
            .save_test_batch(&batch_info)
            .await
            .map_err(|e| ApplicationError::RepositoryError(e.to_string()))?;
        
        log::info!("测试批次已创建: batch_id={}, product_model={:?}", batch_id, product_model);
        Ok(batch_info)
    }
    
    async fn prepare_instances_for_batch(&self, batch_id: &str, definitions: &[ChannelPointDefinition]) -> Result<Vec<String>, ApplicationError> {
        let mut instance_ids = Vec::new();
        
        for definition in definitions {
            let instance = ChannelTestInstance {
                instance_id: uuid::Uuid::new_v4().to_string(),
                definition_id: definition.id.clone(),
                test_batch_id: batch_id.to_string(),
                overall_status: OverallTestStatus::NotTested,
                current_step_details: None,
                error_message: None,
                start_time: None,
                last_updated_time: chrono::Utc::now(),
                final_test_time: None,
                total_test_duration_ms: None,
                sub_test_results: std::collections::HashMap::new(),
                hardpoint_readings: None,
                manual_test_current_value_input: None,
                manual_test_current_value_output: None,
            };
            
            instance_ids.push(instance.instance_id.clone());
            
            self.runtime_repo
                .save_channel_instance(&instance)
                .await
                .map_err(|e| ApplicationError::RepositoryError(e.to_string()))?;
        }
        
        // 更新批次信息
        if let Ok(Some(mut batch_info)) = self.runtime_repo.get_test_batch(batch_id).await {
            batch_info.total_points = instance_ids.len();
            batch_info.status_summary = Some("已准备就绪".to_string());
            
            self.runtime_repo
                .save_test_batch(&batch_info)
                .await
                .map_err(|e| ApplicationError::RepositoryError(e.to_string()))?;
        }
        
        log::info!("批次实例准备完成: batch_id={}, instance_count={}", batch_id, instance_ids.len());
        Ok(instance_ids)
    }
    
    async fn start_batch_testing(&self, batch_id: &str) -> Result<(), ApplicationError> {
        // 获取批次中的所有实例
        let instances = self.runtime_repo
            .list_batch_instances(batch_id)
            .await
            .map_err(|e| ApplicationError::RepositoryError(e.to_string()))?;
        
        if instances.is_empty() {
            return Err(ApplicationError::InvalidOperation("批次中没有测试实例".to_string()));
        }
        
        // 为每个实例创建测试任务
        let mut tasks = Vec::new();
        for instance in instances {
            let test_task = TestTask {
                task_id: uuid::Uuid::new_v4().to_string(),
                instance_id: instance.instance_id,
                definition_id: instance.definition_id,
                batch_id: batch_id.to_string(),
                test_items: vec![SubTestItem::HardPoint], // 根据实际需要配置
                priority: TaskPriority::Normal,
                retry_count: 0,
                max_retries: 3,
                timeout_ms: Some(30000),
                dependencies: vec![],
                metadata: std::collections::HashMap::new(),
            };
            tasks.push(test_task);
        }
        
        // 调度批次任务
        let task_handles = self.task_scheduler
            .schedule_batch_tasks(batch_id, tasks)
            .await
            .map_err(|e| ApplicationError::SchedulerError(e.to_string()))?;
        
        // 更新批次状态
        if let Ok(Some(mut batch_info)) = self.runtime_repo.get_test_batch(batch_id).await {
            batch_info.status_summary = Some("测试中".to_string());
            
            self.runtime_repo
                .save_test_batch(&batch_info)
                .await
                .map_err(|e| ApplicationError::RepositoryError(e.to_string()))?;
        }
        
        log::info!("批次测试已启动: batch_id={}, task_count={}", batch_id, task_handles.len());
        Ok(())
    }
    
    async fn pause_batch_testing(&self, batch_id: &str) -> Result<(), ApplicationError> {
        self.task_scheduler
            .pause_batch(batch_id)
            .await
            .map_err(|e| ApplicationError::SchedulerError(e.to_string()))?;
        
        // 更新批次状态
        if let Ok(Some(mut batch_info)) = self.runtime_repo.get_test_batch(batch_id).await {
            batch_info.status_summary = Some("已暂停".to_string());
            
            self.runtime_repo
                .save_test_batch(&batch_info)
                .await
                .map_err(|e| ApplicationError::RepositoryError(e.to_string()))?;
        }
        
        log::info!("批次测试已暂停: batch_id={}", batch_id);
        Ok(())
    }
    
    async fn resume_batch_testing(&self, batch_id: &str) -> Result<(), ApplicationError> {
        self.task_scheduler
            .resume_batch(batch_id)
            .await
            .map_err(|e| ApplicationError::SchedulerError(e.to_string()))?;
        
        // 更新批次状态
        if let Ok(Some(mut batch_info)) = self.runtime_repo.get_test_batch(batch_id).await {
            batch_info.status_summary = Some("测试中".to_string());
            
            self.runtime_repo
                .save_test_batch(&batch_info)
                .await
                .map_err(|e| ApplicationError::RepositoryError(e.to_string()))?;
        }
        
        log::info!("批次测试已恢复: batch_id={}", batch_id);
        Ok(())
    }
    
    async fn cancel_batch_testing(&self, batch_id: &str) -> Result<(), ApplicationError> {
        self.task_scheduler
            .cancel_batch(batch_id)
            .await
            .map_err(|e| ApplicationError::SchedulerError(e.to_string()))?;
        
        // 更新批次状态
        if let Ok(Some(mut batch_info)) = self.runtime_repo.get_test_batch(batch_id).await {
            batch_info.status_summary = Some("已取消".to_string());
            
            self.runtime_repo
                .save_test_batch(&batch_info)
                .await
                .map_err(|e| ApplicationError::RepositoryError(e.to_string()))?;
        }
        
        log::info!("批次测试已取消: batch_id={}", batch_id);
        Ok(())
    }
    
    async fn start_instance_testing(&self, instance_id: &str) -> Result<(), ApplicationError> {
        // 获取实例信息
        let instance = self.runtime_repo
            .get_channel_instance(instance_id)
            .await
            .map_err(|e| ApplicationError::RepositoryError(e.to_string()))?
            .ok_or_else(|| ApplicationError::InstanceNotFound(instance_id.to_string()))?;
        
        // 检查实例状态
        if instance.overall_status != OverallTestStatus::NotTested {
            return Err(ApplicationError::InvalidOperation(
                format!("实例状态不允许启动测试: {}", instance_id)
            ));
        }
        
        // 创建单个实例的测试任务
        let test_task = TestTask {
            task_id: uuid::Uuid::new_v4().to_string(),
            instance_id: instance.instance_id.clone(),
            definition_id: instance.definition_id.clone(),
            batch_id: instance.test_batch_id.clone(),
            test_items: vec![SubTestItem::HardPoint], // 根据实际需要配置
            priority: TaskPriority::Normal,
            retry_count: 0,
            max_retries: 3,
            timeout_ms: Some(30000),
            dependencies: vec![],
            metadata: std::collections::HashMap::new(),
        };
        
        // 调度任务
        let task_handle = self.task_scheduler
            .schedule_test_task(test_task)
            .await
            .map_err(|e| ApplicationError::SchedulerError(e.to_string()))?;
        
        log::info!("实例测试已启动: instance_id={}, task_handle={:?}", instance_id, task_handle);
        Ok(())
    }
    
    async fn pause_instance_testing(&self, instance_id: &str) -> Result<(), ApplicationError> {
        // 在实际实现中，需要找到对应的任务句柄
        // 这里简化处理，在实际项目中需要维护实例ID到任务句柄的映射
        log::info!("实例测试暂停请求: instance_id={}", instance_id);
        Ok(())
    }
    
    async fn reset_instance_for_retest(&self, instance_id: &str) -> Result<(), ApplicationError> {
        self.state_manager
            .reset_for_retest(instance_id)
            .await
            .map_err(|e| ApplicationError::StateError(e.to_string()))?;
        
        log::info!("实例已重置进行重测: instance_id={}", instance_id);
        Ok(())
    }
    
    async fn batch_reset_instances(&self, instance_ids: Vec<String>) -> Result<(), ApplicationError> {
        self.state_manager
            .batch_reset_for_retest(instance_ids.clone())
            .await
            .map_err(|e| ApplicationError::StateError(e.to_string()))?;
        
        log::info!("批量重置实例完成: count={}", instance_ids.len());
        Ok(())
    }
    
    async fn batch_start_testing(&self, instance_ids: Vec<String>) -> Result<(), ApplicationError> {
        let mut success_count = 0;
        let mut error_count = 0;
        
        for instance_id in instance_ids {
            match self.start_instance_testing(&instance_id).await {
                Ok(()) => success_count += 1,
                Err(e) => {
                    error_count += 1;
                    log::error!("启动实例测试失败: instance_id={}, error={:?}", instance_id, e);
                }
            }
        }
        
        if error_count > 0 {
            return Err(ApplicationError::PartialFailure(
                format!("批量启动测试部分失败: 成功={}, 失败={}", success_count, error_count)
            ));
        }
        
        log::info!("批量启动测试完成: count={}", success_count);
        Ok(())
    }
}
```

---

*本文档持续更新中...*

#### 🧪 测试方案

##### 4.1.3 应用服务测试
```rust
// src/services/application/tests/test_orchestration_service_tests.rs
use super::super::*;
use tokio;

struct ApplicationServiceTestSuite;

impl ApplicationServiceTestSuite {
    async fn create_test_orchestration_service() -> TestOrchestrationService {
        let config_repo = Arc::new(MockConfigurationRepository::new());
        let runtime_repo = Arc::new(MockRuntimeRepository::new());
        let state_manager = Arc::new(MockChannelStateManager::new());
        let task_scheduler = Arc::new(MockTaskScheduler::new());
        
        TestOrchestrationService::new(
            config_repo,
            runtime_repo,
            state_manager,
            task_scheduler,
        )
    }
    
    fn create_test_definitions() -> Vec<ChannelPointDefinition> {
        (0..5).map(|i| ChannelPointDefinition {
            id: format!("def_{}", i),
            tag: format!("TEST_TAG_{:03}", i),
            variable_name: format!("Test Variable {}", i),
            module_type: ModuleType::AI,
            // ... 其他字段
        }).collect()
    }
}

#[tokio::test]
async fn test_create_and_start_batch() {
    let service = ApplicationServiceTestSuite::create_test_orchestration_service().await;
    
    // 创建测试批次
    let batch_info = service.create_test_batch(Some("测试产品".to_string())).await.expect("创建批次失败");
    assert!(!batch_info.batch_id.is_empty(), "批次ID不应为空");
    assert_eq!(batch_info.product_model, Some("测试产品".to_string()), "产品型号应该匹配");
    
    // 准备实例
    let definitions = ApplicationServiceTestSuite::create_test_definitions();
    let instance_ids = service.prepare_instances_for_batch(&batch_info.batch_id, &definitions).await.expect("准备实例失败");
    assert_eq!(instance_ids.len(), 5, "应该创建5个实例");
    
    // 启动批次测试
    service.start_batch_testing(&batch_info.batch_id).await.expect("启动批次测试失败");
}

#[tokio::test]
async fn test_batch_lifecycle() {
    let service = ApplicationServiceTestSuite::create_test_orchestration_service().await;
    
    // 创建和准备批次
    let batch_info = service.create_test_batch(None).await.expect("创建批次失败");
    let definitions = ApplicationServiceTestSuite::create_test_definitions();
    let _instance_ids = service.prepare_instances_for_batch(&batch_info.batch_id, &definitions).await.expect("准备实例失败");
    
    // 启动测试
    service.start_batch_testing(&batch_info.batch_id).await.expect("启动测试失败");
    
    // 暂停测试
    service.pause_batch_testing(&batch_info.batch_id).await.expect("暂停测试失败");
    
    // 恢复测试
    service.resume_batch_testing(&batch_info.batch_id).await.expect("恢复测试失败");
    
    // 取消测试
    service.cancel_batch_testing(&batch_info.batch_id).await.expect("取消测试失败");
}

#[tokio::test]
async fn test_instance_operations() {
    let service = ApplicationServiceTestSuite::create_test_orchestration_service().await;
    
    // 创建和准备批次
    let batch_info = service.create_test_batch(None).await.expect("创建批次失败");
    let definitions = ApplicationServiceTestSuite::create_test_definitions();
    let instance_ids = service.prepare_instances_for_batch(&batch_info.batch_id, &definitions).await.expect("准备实例失败");
    
    let instance_id = &instance_ids[0];
    
    // 启动单个实例测试
    service.start_instance_testing(instance_id).await.expect("启动实例测试失败");
    
    // 重置实例
    service.reset_instance_for_retest(instance_id).await.expect("重置实例失败");
    
    // 批量重置
    service.batch_reset_instances(instance_ids.clone()).await.expect("批量重置失败");
    
    // 批量启动
    service.batch_start_testing(instance_ids).await.expect("批量启动失败");
}

#[tokio::test]
async fn test_error_handling() {
    let service = ApplicationServiceTestSuite::create_test_orchestration_service().await;
    
    // 测试启动不存在的批次
    let result = service.start_batch_testing("not_exist_batch").await;
    assert!(result.is_err(), "启动不存在的批次应该失败");
    
    // 测试重置不存在的实例
    let result = service.reset_instance_for_retest("not_exist_instance").await;
    assert!(result.is_err(), "重置不存在的实例应该失败");
}
```

### ✅ 步骤4.1完成标准

1. **服务组合**: 通过组合领域服务实现复杂业务流程
2. **单一职责**: 每个应用服务职责明确单一
3. **错误处理**: 统一的错误处理和转换机制
4. **事务一致性**: 确保跨服务操作的一致性
5. **高可维护性**: 清晰的代码结构和完整测试覆盖
6. **日志记录**: 完整的操作日志记录
7. **异常处理**: 优雅的错误处理和用户友好的错误信息

---

## 📊 重构完成总结

### 🎯 达成的目标

通过四个阶段的详细重构，我们成功解决了用户提出的所有关键问题：

1. **✅ 统一数据源管理**: Repository模式确保所有数据访问都通过统一接口
   - IConfigurationRepository 管理配置数据
   - IRuntimeRepository 管理运行时数据
   - IPersistentRepository 管理持久化数据

2. **✅ 数据模型职责划分**: 三层数据模型明确区分配置、运行时和持久化数据
   - Configuration Layer: 只读配置数据
   - Runtime Layer: 可变运行时数据
   - Persistent Layer: 需要保存的历史数据

3. **✅ 服务单一职责**: 每个服务都有明确单一的功能边界
   - ChannelStateManager: 只负责状态管理
   - TaskScheduler: 只负责任务调度
   - TestExecutor: 只负责测试执行

4. **✅ 服务组合模式**: 应用服务层通过组合领域服务实现复杂业务流程
   - TestOrchestrationService: 编排测试流程
   - DataManagementService: 管理数据操作
   - SystemManagementService: 管理系统维护

5. **✅ 任务管理优化**: 参考原有逻辑并增强稳定性和错误恢复能力
   - 优先级调度和批次管理
   - 完善的重试机制和错误恢复
   - 健康监控和性能指标

### 🏗️ 架构优势

- **数据一致性**: 通过Repository和状态管理器确保数据访问的一致性
- **高可维护性**: 清晰的分层架构和单一职责原则
- **高可扩展性**: 服务组合模式支持灵活的业务流程扩展
- **高稳定性**: 完善的错误处理、重试机制和健康监控
- **高性能**: 内存缓存、并发控制和批量操作优化

### 📈 质量保证

- **完整测试覆盖**: 每个组件都有详细的单元测试和集成测试
- **性能验证**: 支持大规模数据集的性能测试
- **并发安全**: 全异步设计和并发安全保证
- **错误恢复**: 完善的重试策略和故障恢复机制

### 🚀 实施路径

1. **Phase 1 (1-2周)**: 数据访问层重构 - 建立Repository基础
2. **Phase 2 (1-2周)**: 状态管理器重构 - 确保状态一致性
3. **Phase 3 (2-3周)**: 任务调度器重构 - 优化任务管理
4. **Phase 4 (1-2周)**: 应用服务层重构 - 实现服务组合

### 💡 后续建议

1. **渐进式重构**: 按阶段实施，确保每个阶段都能独立工作
2. **测试先行**: 为每个组件编写充分的测试用例
3. **监控指标**: 建立完善的系统监控和性能指标
4. **文档维护**: 保持架构文档和API文档的更新

这个重构方案为FAT_TEST系统奠定了坚实的技术基础，确保系统能够长期稳定运行并支持未来的功能扩展。

---

## 📖 附录

### A. 错误类型定义
```rust
#[derive(Error, Debug, Clone, Serialize)]
pub enum ApplicationError {
    #[error("Repository error: {0}")]
    RepositoryError(String),
    
    #[error("State error: {0}")]
    StateError(String),
    
    #[error("Scheduler error: {0}")]
    SchedulerError(String),
    
    #[error("Instance not found: {0}")]
    InstanceNotFound(String),
    
    #[error("Invalid operation: {0}")]
    InvalidOperation(String),
    
    #[error("Partial failure: {0}")]
    PartialFailure(String),
}
```

### B. 配置文件示例
```toml
[system]
max_concurrent_tests = 10
default_timeout_ms = 30000
auto_backup_enabled = true
backup_retention_days = 30

[logging]
level = "info"
file_path = "logs/fat_test.log"

[plc]
connection_timeout_ms = 5000
retry_count = 3
```

---

**重构详细实施步骤文档完成**

*此文档提供了FAT_TEST系统完整的重构实施方案，通过四个阶段的详细步骤，确保系统架构的现代化和稳定性。*