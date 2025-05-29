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