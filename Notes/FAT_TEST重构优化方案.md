# FAT_TEST 系统架构重构优化方案

## 📋 重构背景与目标

### 🔍 当前架构问题分析

通过对现有代码的深入分析，发现以下关键问题：

#### 1. 状态管理违规 (FAT-CSM-001)
- **问题**: 在 `ChannelAllocationService` 中发现直接修改 `overall_status` 的违规代码
- **影响**: 破坏了状态管理的单一入口原则，可能导致状态不一致
- **位置**: `channel_allocation_service.rs:810` 行

#### 2. 数据模型职责不清
- **问题**: 没有明确区分配置数据、运行时数据和持久化数据的职责边界
- **影响**: 数据访问混乱，难以维护数据一致性
- **表现**: 配置数据和运行时数据混合存储，修改路径不统一

#### 3. 服务职责重叠
- **问题**: 某些服务承担了过多职责，违反单一职责原则
- **影响**: 代码复杂度高，难以测试和维护
- **表现**: 业务逻辑分散在多个服务中，缺乏清晰的组合模式

#### 4. 测试任务管理不够稳定
- **问题**: 并发控制和任务生命周期管理存在潜在问题
- **影响**: 系统稳定性不足，可能出现任务死锁或状态错乱
- **表现**: 缺乏完善的任务恢复机制和错误处理

### 🎯 重构目标

#### 核心目标
1. **统一数据源管理**: 建立Repository模式，所有数据访问通过统一接口
2. **严格状态控制**: 确保所有状态修改只能通过ChannelStateManager
3. **数据模型分层**: 按照可变性和职责重新划分数据模型
4. **服务单一职责**: 每个服务只负责一个明确的业务领域
5. **稳定任务管理**: 设计更稳定的任务调度和并发控制机制

#### 质量目标
- **可维护性**: 代码结构清晰，易于理解和修改
- **可测试性**: 每个组件都可以独立测试
- **可扩展性**: 新功能添加不影响现有代码
- **稳定性**: 系统能够处理各种异常情况

## 🏗️ 新架构设计

### 🔄 分层架构重新设计

```
┌─────────────────────────────────────────────────────────────┐
│                    前端层 (Angular + Tauri)                   │
├─────────────────────────────────────────────────────────────┤
│                     应用服务层 (Application)                    │
│  ┌─────────────────┐ ┌─────────────────┐ ┌─────────────────┐ │
│  │  工作流服务     │ │   数据管理服务   │ │   系统管理服务   │ │
│  │ WorkflowService │ │DataMgmtService  │ │ SystemService   │ │
│  └─────────────────┘ └─────────────────┘ └─────────────────┘ │
├─────────────────────────────────────────────────────────────┤
│                     领域服务层 (Domain)                       │
│  ┌─────────────────┐ ┌─────────────────┐ ┌─────────────────┐ │
│  │   状态管理器    │ │   任务调度器    │ │   测试执行器    │ │
│  │  StateManager   │ │TaskScheduler    │ │TestExecutor     │ │
│  └─────────────────┘ └─────────────────┘ └─────────────────┘ │
├─────────────────────────────────────────────────────────────┤
│                    数据访问层 (Repository)                     │
│  ┌─────────────────┐ ┌─────────────────┐ ┌─────────────────┐ │
│  │   配置仓储      │ │   运行时仓储    │ │   持久化仓储    │ │
│  │ConfigRepository │ │RuntimeRepository│ │PersistRepository│ │
│  └─────────────────┘ └─────────────────┘ └─────────────────┘ │
├─────────────────────────────────────────────────────────────┤
│                   基础设施层 (Infrastructure)                  │
│  ┌─────────────────┐ ┌─────────────────┐ ┌─────────────────┐ │
│  │    PLC通信      │ │   数据持久化    │ │   事件发布      │ │
│  │   PLCService    │ │PersistenceService│ │  EventService   │ │
│  └─────────────────┘ └─────────────────┘ └─────────────────┘ │
└─────────────────────────────────────────────────────────────┘
```

### 📊 数据模型重新分层

#### 1. Configuration Layer (配置层) - 只读数据
```rust
// 点位定义 - 系统配置，不可变
pub struct ChannelPointDefinition {
    pub id: String,
    pub tag: String,
    pub module_type: ModuleType,
    pub plc_addresses: PlcAddressConfig,
    pub test_parameters: TestParameterConfig,
    // ... 其他配置字段
}

// PLC地址配置
pub struct PlcAddressConfig {
    pub read_address: String,
    pub write_address: Option<String>,
    pub data_type: PointDataType,
    // ... 地址相关配置
}

// 测试参数配置
pub struct TestParameterConfig {
    pub range_config: Option<RangeConfig>,
    pub alarm_config: Option<AlarmConfig>,
    pub test_sequence: Vec<SubTestItem>,
    // ... 测试相关配置
}
```

#### 2. Runtime Layer (运行时层) - 可变数据
```rust
// 通道测试实例 - 运行时状态，可变
pub struct ChannelTestInstance {
    pub instance_id: String,
    pub definition_id: String,
    pub batch_id: String,
    pub runtime_state: ChannelRuntimeState,
    pub test_results: TestResultCollection,
    pub execution_context: ExecutionContext,
}

// 运行时状态
pub struct ChannelRuntimeState {
    pub overall_status: OverallTestStatus,
    pub current_phase: TestPhase,
    pub error_info: Option<ErrorInfo>,
    pub timestamps: TimestampCollection,
    pub progress_info: ProgressInfo,
}

// 测试结果集合
pub struct TestResultCollection {
    pub sub_test_results: HashMap<SubTestItem, SubTestResult>,
    pub measurement_data: Vec<MeasurementPoint>,
    pub validation_results: Vec<ValidationResult>,
}
```

#### 3. Persistent Layer (持久化层) - 需要保存的数据
```rust
// 批次信息 - 需要持久化
pub struct TestBatchPersistent {
    pub batch_id: String,
    pub metadata: BatchMetadata,
    pub statistics: BatchStatistics,
    pub audit_trail: Vec<AuditRecord>,
}

// 测试记录 - 需要持久化
pub struct TestRecord {
    pub record_id: String,
    pub instance_id: String,
    pub test_results: TestResultSnapshot,
    pub execution_summary: ExecutionSummary,
    pub created_at: DateTime<Utc>,
}
```

## 🔧 核心组件重新设计

### 1. 统一数据访问层 (Repository Pattern)

#### IRepository 基础接口
```rust
#[async_trait::async_trait]
pub trait IRepository<T, K>: Send + Sync {
    async fn get(&self, key: K) -> Result<Option<T>, RepositoryError>;
    async fn save(&self, entity: T) -> Result<(), RepositoryError>;
    async fn delete(&self, key: K) -> Result<(), RepositoryError>;
    async fn exists(&self, key: K) -> Result<bool, RepositoryError>;
}
```

#### 配置数据仓储
```rust
#[async_trait::async_trait]
pub trait IConfigurationRepository: Send + Sync {
    // 点位定义管理
    async fn get_channel_definition(&self, id: &str) -> Result<Option<ChannelPointDefinition>, RepositoryError>;
    async fn list_channel_definitions(&self) -> Result<Vec<ChannelPointDefinition>, RepositoryError>;
    async fn save_channel_definitions(&self, definitions: Vec<ChannelPointDefinition>) -> Result<(), RepositoryError>;
    
    // 测试参数管理
    async fn get_test_parameters(&self, module_type: ModuleType) -> Result<TestParameterSet, RepositoryError>;
    async fn update_test_parameters(&self, module_type: ModuleType, params: TestParameterSet) -> Result<(), RepositoryError>;
}
```

#### 运行时数据仓储
```rust
#[async_trait::async_trait]
pub trait IRuntimeRepository: Send + Sync {
    // 通道实例管理
    async fn get_channel_instance(&self, instance_id: &str) -> Result<Option<ChannelTestInstance>, RepositoryError>;
    async fn save_channel_instance(&self, instance: ChannelTestInstance) -> Result<(), RepositoryError>;
    async fn list_batch_instances(&self, batch_id: &str) -> Result<Vec<ChannelTestInstance>, RepositoryError>;
    
    // 批次管理
    async fn get_test_batch(&self, batch_id: &str) -> Result<Option<TestBatchRuntime>, RepositoryError>;
    async fn save_test_batch(&self, batch: TestBatchRuntime) -> Result<(), RepositoryError>;
    async fn list_active_batches(&self) -> Result<Vec<TestBatchRuntime>, RepositoryError>;
}
```

### 2. 严格的状态管理器

#### IChannelStateManager 增强接口
```rust
#[async_trait::async_trait]
pub trait IChannelStateManager: Send + Sync {
    // 状态查询 (只读操作)
    async fn get_current_state(&self, instance_id: &str) -> Result<ChannelRuntimeState, StateError>;
    async fn can_transition_to(&self, instance_id: &str, target_status: OverallTestStatus) -> Result<bool, StateError>;
    
    // 状态修改 (唯一修改入口)
    async fn apply_test_outcome(&self, instance_id: &str, outcome: TestOutcome) -> Result<StateTransition, StateError>;
    async fn force_state_transition(&self, instance_id: &str, target_status: OverallTestStatus, reason: String) -> Result<StateTransition, StateError>;
    async fn reset_for_retest(&self, instance_id: &str) -> Result<StateTransition, StateError>;
    
    // 状态批量操作
    async fn batch_state_update(&self, updates: Vec<StateUpdateRequest>) -> Result<Vec<StateTransition>, StateError>;
    
    // 状态事件
    async fn subscribe_state_changes(&self) -> Result<Receiver<StateChangeEvent>, StateError>;
}
```

### 3. 优化的任务调度器

#### ITaskScheduler 接口
```rust
#[async_trait::async_trait]
pub trait ITaskScheduler: Send + Sync {
    // 任务调度
    async fn schedule_test_task(&self, task: TestTask) -> Result<TaskHandle, SchedulerError>;
    async fn schedule_batch_tasks(&self, batch_id: &str, tasks: Vec<TestTask>) -> Result<Vec<TaskHandle>, SchedulerError>;
    
    // 任务控制
    async fn pause_task(&self, task_handle: TaskHandle) -> Result<(), SchedulerError>;
    async fn resume_task(&self, task_handle: TaskHandle) -> Result<(), SchedulerError>;
    async fn cancel_task(&self, task_handle: TaskHandle) -> Result<(), SchedulerError>;
    
    // 批次控制
    async fn pause_batch(&self, batch_id: &str) -> Result<(), SchedulerError>;
    async fn resume_batch(&self, batch_id: &str) -> Result<(), SchedulerError>;
    async fn cancel_batch(&self, batch_id: &str) -> Result<(), SchedulerError>;
    
    // 状态查询
    async fn get_task_status(&self, task_handle: TaskHandle) -> Result<TaskStatus, SchedulerError>;
    async fn get_batch_progress(&self, batch_id: &str) -> Result<BatchProgress, SchedulerError>;
    async fn list_active_tasks(&self) -> Result<Vec<TaskInfo>, SchedulerError>;
    
    // 资源管理
    async fn set_concurrency_limit(&self, limit: usize) -> Result<(), SchedulerError>;
    async fn get_system_load(&self) -> Result<SystemLoad, SchedulerError>;
}
```

### 4. 测试执行器重构

#### ITestExecutor 接口
```rust
#[async_trait::async_trait]
pub trait ITestExecutor: Send + Sync {
    // 执行器信息
    fn executor_type(&self) -> ExecutorType;
    fn supported_items(&self) -> Vec<SubTestItem>;
    
    // 测试执行
    async fn execute_test(&self, request: TestExecutionRequest) -> Result<TestOutcome, ExecutorError>;
    async fn validate_prerequisites(&self, request: &TestExecutionRequest) -> Result<Vec<ValidationIssue>, ExecutorError>;
    
    // 资源管理
    async fn acquire_resources(&self, request: &TestExecutionRequest) -> Result<ResourceHandle, ExecutorError>;
    async fn release_resources(&self, handle: ResourceHandle) -> Result<(), ExecutorError>;
    
    // 健康检查
    async fn health_check(&self) -> Result<ExecutorHealth, ExecutorError>;
}
```

## 📝 详细重构步骤

接下来的部分将在另一个文档中详细说明每个步骤的具体实现... 

## 💾 数据持久化架构优化方案

### 🎯 优化目标

基于用户需求，优化数据持久化策略：
1. 点表数据在分配完批次后直接由状态管理器管理
2. 测试过程中多个关键节点进行状态备份
3. 支持从任意备份点恢复测试继续执行

### 🏗️ 优化后的数据流向

```
┌─────────────────────────────────────────────────────────────┐
│                     优化后数据流                             │
├─────────────────────────────────────────────────────────────┤
│ Excel导入 → ConfigRepository (只读配置)                      │
│     ↓                                                       │
│ 批次分配 → StateManager (接管状态管理)                       │
│     ↓           ↓                                           │
│ 运行时缓存 → 备份节点 → PersistentRepository                │
│ (快速访问)   (状态快照)    (永久存储)                        │
│     ↑                           ↓                          │
│ 复测恢复 ←─────────────────── 恢复管理器                    │
└─────────────────────────────────────────────────────────────┘
```

### 🔧 核心组件增强

#### 1. 简化的状态管理器

```rust
/// 简化的状态管理器，支持最新状态备份和恢复
#[async_trait]
pub trait IEnhancedStateManager: Send + Sync {
    // 状态管理 (原有功能)
    async fn apply_test_outcome(&self, instance_id: &str, outcome: TestOutcome) -> Result<StateTransition, StateError>;
    
    // 简化的备份功能
    async fn create_latest_backup(&self, batch_id: &str) -> Result<BackupId, StateError>;
    async fn auto_backup_on_milestone(&self, instance_id: &str, milestone: TestMilestone) -> Result<(), StateError>;
    
    // 简化的恢复功能
    async fn get_latest_backup(&self, batch_id: &str) -> Result<Option<LatestBackup>, StateError>;
    async fn restore_from_latest(&self, batch_id: &str) -> Result<RestoreResult, StateError>;
    async fn prepare_retest(&self, batch_id: &str, scope: RestoreScope) -> Result<(), StateError>;
    
    // 状态同步
    async fn sync_state_to_persistent(&self, instance_ids: Vec<String>) -> Result<(), StateError>;
    async fn validate_state_consistency(&self) -> Result<ConsistencyReport, StateError>;
}

/// 测试里程碑 - 自动备份触发点 (简化)
#[derive(Debug, Clone, PartialEq)]
pub enum TestMilestone {
    BatchCreated,           // 批次创建完成
    WiringConfirmed,        // 接线确认完成
    HardPointTestComplete,  // 硬点测试完成
    AlarmTestComplete,      // 报警测试完成
    BatchCompleted,         // 批次测试完成
}

/// 最新备份信息
#[derive(Debug, Clone)]
pub struct LatestBackup {
    pub backup_id: BackupId,
    pub batch_id: String,
    pub created_at: DateTime<Utc>,
    pub last_milestone: TestMilestone,
    pub instance_count: usize,
    pub completion_percentage: f64,
    pub backup_size_mb: f64,
}

/// 恢复结果
#[derive(Debug, Clone)]
pub struct RestoreResult {
    pub restored_instances: Vec<String>,
    pub failed_instances: Vec<String>,
    pub warnings: Vec<String>,
    pub restored_to_milestone: TestMilestone,
    pub restore_time: DateTime<Utc>,
}
```

#### 2. 简化的备份策略配置

```rust
/// 简化的备份策略配置
#[derive(Debug, Clone)]
pub struct BackupStrategy {
    // 自动备份触发条件
    pub auto_backup_milestones: Vec<TestMilestone>,
    pub backup_interval_minutes: Option<u32>,
    pub backup_on_error: bool,
    
    // 简化的保留策略
    pub keep_only_latest: bool,           // 只保留最新备份
    pub retention_hours: u32,             // 备份保留时间(小时)
    pub auto_cleanup: bool,               // 自动清理旧备份
    
    // 性能优化
    pub async_backup: bool,
    pub compress_backup: bool,
    pub backup_priority: BackupPriority,
}

#[derive(Debug, Clone)]
pub enum BackupPriority {
    Low,     // 后台异步备份
    Normal,  // 平衡性能和及时性
    High,    // 立即同步备份
}
```

#### 3. 简化恢复管理器

```rust
/// 简化的恢复管理器接口
#[async_trait]
pub trait IRecoveryManager: Send + Sync {
    // 获取最新备份状态
    async fn get_latest_backup(&self, batch_id: &str) -> Result<Option<StateSnapshot>, RecoveryError>;
    async fn get_backup_info(&self, batch_id: &str) -> Result<BackupInfo, RecoveryError>;
    
    // 简单的恢复最新状态
    async fn restore_latest_state(&self, batch_id: &str) -> Result<RestoreResult, RecoveryError>;
    async fn validate_restore_feasibility(&self, batch_id: &str) -> Result<bool, RecoveryError>;
    
    // 准备重测 (从最新备份状态开始)
    async fn prepare_retest(&self, batch_id: &str, scope: RestoreScope) -> Result<RetestInfo, RecoveryError>;
}

/// 备份信息
#[derive(Debug, Clone)]
pub struct BackupInfo {
    pub latest_backup_time: DateTime<Utc>,
    pub backup_size_mb: f64,
    pub instance_count: usize,
    pub completion_percentage: f64,
    pub is_valid: bool,
}

/// 重测信息
#[derive(Debug, Clone)]
pub struct RetestInfo {
    pub resettable_instances: Vec<String>,
    pub completed_instances: Vec<String>,
    pub failed_instances: Vec<String>,
    pub estimated_retest_time: Duration,
}

/// 简化的恢复计划
#[derive(Debug, Clone)]
pub struct SimpleRecoveryPlan {
    pub batch_id: String,
    pub restore_scope: RestoreScope,
    pub reset_failed_only: bool,
    pub notify_users: bool,
}

#[derive(Debug, Clone)]
pub enum RestoreScope {
    AllInstances,        // 恢复所有实例到最后备份状态
    FailedOnly,          // 只恢复失败的实例
    IncompleteOnly,      // 只恢复未完成的实例
}
```

### 🚀 实施优化的关键步骤

#### 步骤 1: 状态管理器接管数据流

```rust
impl EnhancedStateManager {
    /// 批次分配完成后，接管所有实例的状态管理
    pub async fn take_control_of_batch(&self, batch_id: &str) -> Result<ControlTransferResult, StateError> {
        // 1. 从RuntimeRepository获取所有实例
        let instances = self.runtime_repo.list_batch_instances(batch_id).await?;
        
        // 2. 创建初始备份
        let backup_id = self.create_latest_backup(batch_id).await?;
        
        // 3. 注册自动备份规则
        self.register_auto_backup_for_batch(batch_id, &self.backup_strategy).await?;
        
        // 4. 设置状态变更监听
        self.setup_state_change_listeners(batch_id).await?;
        
        Ok(ControlTransferResult {
            controlled_instances: instances.len(),
            initial_backup: backup_id,
            auto_backup_enabled: true,
        })
    }
    
    /// 在关键测试节点自动创建备份 (简化版)
    async fn auto_backup_on_milestone(&self, instance_id: &str, milestone: TestMilestone) -> Result<(), StateError> {
        let instance = self.runtime_repo.get_channel_instance(instance_id).await?
            .ok_or_else(|| StateError::InstanceNotFound(instance_id.to_string()))?;
        
        // 检查是否需要在此里程碑备份
        if self.backup_strategy.auto_backup_milestones.contains(&milestone) {
            if self.backup_strategy.async_backup {
                // 异步备份，不阻塞测试进程
                let state_manager = self.clone();
                let batch_id = instance.test_batch_id.clone();
                tokio::spawn(async move {
                    if let Err(e) = state_manager.create_latest_backup(&batch_id).await {
                        log::error!("自动备份失败: {}", e);
                    } else {
                        log::info!("自动备份完成: 批次={}, 里程碑={:?}", batch_id, milestone);
                    }
                });
            } else {
                // 同步备份
                self.create_latest_backup(&instance.test_batch_id).await?;
            }
            
            // 清理旧备份
            if self.backup_strategy.auto_cleanup {
                self.cleanup_old_backups(&instance.test_batch_id).await?;
            }
        }
        
        Ok(())
    }
}
```

#### 步骤 2: 简化恢复机制

```rust
impl RecoveryManager {
    /// 简单的恢复最新状态
    pub async fn restore_latest_state(&self, batch_id: &str) -> Result<RestoreResult, RecoveryError> {
        // 1. 获取最新备份
        let latest_backup = self.state_manager.get_latest_backup(batch_id).await?
            .ok_or_else(|| RecoveryError::NoBackupFound(batch_id.to_string()))?;
        
        // 2. 验证备份有效性
        if !self.validate_restore_feasibility(batch_id).await? {
            return Err(RecoveryError::RestoreNotFeasible("备份数据不完整或损坏".to_string()));
        }
        
        // 3. 执行恢复
        log::info!("开始恢复批次: {}, 备份时间: {}", batch_id, latest_backup.created_at);
        
        let restore_result = self.state_manager.restore_from_latest(batch_id).await?;
        
        log::info!("恢复完成: 成功={}, 失败={}", 
                  restore_result.restored_instances.len(),
                  restore_result.failed_instances.len());
        
        Ok(restore_result)
    }
    
    /// 准备重测 (从最新备份状态开始)
    pub async fn prepare_retest(&self, batch_id: &str, scope: RestoreScope) -> Result<RetestInfo, RecoveryError> {
        // 1. 先恢复到最新状态
        let restore_result = self.restore_latest_state(batch_id).await?;
        
        // 2. 根据范围准备重测实例
        let instances = match scope {
            RestoreScope::AllInstances => {
                self.runtime_repo.list_batch_instances(batch_id).await?
            },
            RestoreScope::FailedOnly => {
                self.runtime_repo.list_instances_by_status(batch_id, OverallTestStatus::Failed).await?
            },
            RestoreScope::IncompleteOnly => {
                self.get_incomplete_instances(batch_id).await?
            },
        };
        
        // 3. 重置实例状态为可重测
        for instance_id in &instances {
            self.state_manager.reset_for_retest(instance_id).await?;
        }
        
        Ok(RetestInfo {
            resettable_instances: instances.iter().map(|i| i.instance_id.clone()).collect(),
            completed_instances: restore_result.restored_instances,
            failed_instances: restore_result.failed_instances,
            estimated_retest_time: self.estimate_retest_duration(&instances),
        })
    }
}
```

### 🎉 简化方案总结

这个**简化的**数据持久化优化方案完全符合你的需求：

#### ✅ 核心功能保障
1. **✅ 点表数据分配后直接由状态管理器统一管理**
2. **✅ 测试过程中关键节点自动备份最新状态**  
3. **✅ 简单高效的最新状态恢复功能**
4. **✅ 灵活的重测范围选择**
5. **✅ 用户友好的一键操作界面**

#### 🚀 简化优势
- **降低复杂度**: 去掉多快照管理，只保留最新状态备份
- **提高性能**: 减少存储空间占用，备份恢复更快速
- **简化操作**: 用户界面更直观，操作更简单
- **维护成本低**: 代码逻辑清晰，易于维护和扩展

#### 📱 实际应用场景

```bash
# 场景1: 系统意外重启，一键恢复最新状态
用户点击 "恢复到最新状态" → 自动恢复到最近备份点 → 继续测试

# 场景2: 部分测试失败，选择性重测
用户点击 "重测失败项目" → 只重测失败的实例 → 节省时间

# 场景3: 全部重新开始测试
用户点击 "全部重新测试" → 重置所有实例状态 → 从头开始
```

这个简化方案**既满足了业务需求，又保证了系统的简洁性和高效性**！完美契合你"只恢复最后状态"的需求。 