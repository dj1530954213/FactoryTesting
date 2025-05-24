# FAT_TEST 项目 Cursor 开发规则

## 项目概述

FAT_TEST 是一个工厂验收测试桌面应用程序，正在从 C# WPF 技术栈迁移到 **Rust + Angular + Tauri** 技术栈。

**重构前的代码实现参考**：https://github.com/dj1530954213/FAT_TEST/tree/StructureOptimization/FAT_TEST/FatFullVersion/FatFullVersion

**⚠️ 重要说明**：GitHub链接中的代码仅作为功能和实现参考，最终实现必须严格遵循本项目的技术栈迁移系统架构设计和详细实施步骤文档要求。

## 技术栈规范

### 后端 (Rust + Tauri)
- **框架**: Tauri 2.x
- **语言**: Rust 1.85+
- **异步运行时**: tokio
- **序列化**: serde (JSON)
- **错误处理**: thiserror + anyhow
- **PLC通信**: modbus-rs, opcua-client
- **数据库**: SQLite (rusqlite), JSON文件存储
- **日志**: log + env_logger
- **测试**: tokio-test, mockall

### 前端 (Angular + TypeScript)
- **框架**: Angular 18+
- **语言**: TypeScript 5.0+
- **UI组件库**: NG-ZORRO (基于 Ant Design 的 Angular 组件库)
- **图表库**: ECharts (用于数据可视化和图标展示)
- **状态管理**: 可选使用 NgRx 或 Akita
- **样式**: SCSS
- **测试**: Jasmine + Karma
- **Tauri集成**: @tauri-apps/api

## 架构设计原则

### 1. 分层架构 (DDD)
```
前端 (Angular + Tauri)
├── View Layer (Components)
├── Service Layer (Angular Services)
└── Tauri IPC Bridge

后端 (Rust)
├── Application Layer (应用服务层)
├── Domain Layer (领域服务层)
└── Infrastructure Layer (基础设施层)
```

### 2. 核心设计规则

#### FAT-CSM-001: 状态管理唯一入口
- `ChannelStateManager` 是唯一允许修改 `ChannelTestInstance` 状态的组件
- 所有状态变更必须通过 `apply_raw_outcome()` 方法

#### FAT-CSM-002: 状态转换规则
- 状态转换必须符合业务逻辑规则
- 不允许非法状态跳转
- 每次状态变更都要记录时间戳

#### FAT-TTM-001: 测试任务管理
- `TestExecutionEngine` 负责任务并发控制
- 使用信号量控制最大并发测试数
- 支持任务暂停、继续、取消操作

#### FAT-CTK-001: 通信任务规则
- 每个 `ISpecificTestStepExecutor` 只处理单一测试步骤
- 所有PLC通信必须异步执行
- 通信失败要有重试机制

#### FAT-EVT-001: 事件发布规则
- 重要状态变更必须发布事件给前端
- 事件Payload必须包含完整的状态信息
- 使用 Tauri 的 emit 系统进行事件通知

## 代码风格和约定

### Rust 代码规范

#### 命名约定
```rust
// 结构体使用 PascalCase
pub struct ChannelTestInstance {
    pub instance_id: String,
    pub definition_id: String,
}

// 枚举使用 PascalCase
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum OverallTestStatus {
    NotTested,
    WiringConfirmed,
    HardPointTesting,
}

// 函数和变量使用 snake_case
async fn apply_raw_outcome(&self, instance: &mut ChannelTestInstance) -> Result<(), AppError>

// 常量使用 SCREAMING_SNAKE_CASE
const MAX_RETRY_COUNT: u32 = 3;
```

#### 错误处理
```rust
// 使用统一的 AppError 类型
#[derive(Error, Debug, Clone, Serialize)]
pub enum AppError {
    #[error("PLC Communication Error: {0}")]
    PlcCommunicationError(String),
    
    #[error("State Transition Error: {0}")]
    StateTransitionError(String),
}

// 所有服务方法都返回 Result<T, AppError>
async fn start_test(&self) -> Result<(), AppError> {
    // 实现...
}
```

#### 异步编程
```rust
// 所有服务接口都使用 async trait
#[async_trait::async_trait]
pub trait ITestOrchestrationService: Send + Sync {
    async fn create_test_batch(&self) -> Result<TestBatchInfo, AppError>;
}

// 并发控制使用 tokio 原语
use tokio::sync::{Mutex, Semaphore};
```

### TypeScript 代码规范

#### 命名约定
```typescript
// 接口使用 PascalCase，以 I 开头
interface ITestOrchestrationService {
  createTestBatch(): Promise<TestBatchInfo>;
}

// 类使用 PascalCase
export class TestOrchestrationService implements ITestOrchestrationService {
  // 方法使用 camelCase
  async createTestBatch(): Promise<TestBatchInfo> {
    // 实现...
  }
}

// 枚举使用 PascalCase
export enum OverallTestStatus {
  NotTested = 'NotTested',
  WiringConfirmed = 'WiringConfirmed',
}
```

#### Tauri 集成
```typescript
// 使用统一的 invoke 包装
import { invoke } from '@tauri-apps/api/tauri';

export class BackendCommsService {
  async createTestBatch(productModel?: string): Promise<TestBatchInfo> {
    try {
      return await invoke('create_test_batch_cmd', { productModel });
    } catch (error) {
      console.error('Backend call failed:', error);
      throw new Error(`创建测试批次失败: ${error}`);
    }
  }
}
```

#### UI 组件库使用
```typescript
// NG-ZORRO 组件使用
import { NzTableModule } from 'ng-zorro-antd/table';
import { NzMessageService } from 'ng-zorro-antd/message';

// ECharts 图表使用
import * as echarts from 'echarts/core';
import { LineChart } from 'echarts/charts';
```

## 目录结构要求

**📁 项目目录结构**：请严格参考 [`docs/PROJECT_STRUCTURE.md`](./PROJECT_STRUCTURE.md) 文档中的完整目录结构要求。该文档详细描述了 Rust 后端和 Angular 前端的所有目录组织规范。

## 开发最佳实践

### 服务依赖注入
```rust
// Rust: 使用 Arc<dyn Trait> 进行依赖注入
pub struct TestOrchestrationService {
    channel_state_manager: Arc<dyn IChannelStateManager>,
    test_execution_engine: Arc<dyn ITestExecutionEngine>,
}
```

```typescript
// Angular: 使用依赖注入
@Injectable({
  providedIn: 'root'
})
export class TestOrchestrationService {
  constructor(
    private backendComms: BackendCommsService,
    private eventListener: EventListenerService
  ) {}
}
```

### 错误处理模式
```rust
// Rust: 统一错误转换
impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        AppError::IoError(err.to_string())
    }
}
```

### 测试规范
```rust
// Rust: 单元测试
#[cfg(test)]
mod tests {
    use super::*;
    use mockall::predicate::*;

    #[tokio::test]
    async fn test_apply_raw_outcome() {
        // 测试实现...
    }
}
```

## 核心数据模型要求

### 必须实现的核心结构

#### Rust 结构体
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelPointDefinition {
    pub id: String,
    pub tag: String,
    pub module_type: ModuleType,
    pub plc_communication_address: String,
    // ... 其他字段
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelTestInstance {
    pub instance_id: String,
    pub definition_id: String,
    pub overall_status: OverallTestStatus,
    pub sub_test_results: HashMap<SubTestItem, SubTestExecutionResult>,
    // ... 其他字段
}
```

#### TypeScript 接口
```typescript
export interface ChannelPointDefinition {
  id: string;
  tag: string;
  moduleType: ModuleType;
  plcCommunicationAddress: string;
  // ... 其他字段
}

export interface ChannelTestInstance {
  instanceId: string;
  definitionId: string;
  overallStatus: OverallTestStatus;
  subTestResults: Record<SubTestItem, SubTestExecutionResult>;
  // ... 其他字段
}
```

## PLC 通信规范

### 通信接口定义
```rust
#[async_trait::async_trait]
pub trait IPlcCommunicationService: Send + Sync {
    async fn connect(&self) -> Result<(), AppError>;
    async fn read_bool(&self, address: &str) -> Result<bool, AppError>;
    async fn write_f32(&self, address: &str, value: f32) -> Result<(), AppError>;
    // 支持 Modbus TCP, Siemens S7, OPC UA
}
```

### 测试步骤执行器
```rust
#[async_trait::async_trait]
pub trait ISpecificTestStepExecutor: Send + Sync {
    async fn execute(
        &self,
        instance: &ChannelTestInstance,
        definition: &ChannelPointDefinition,
        plc_service: Arc<dyn IPlcCommunicationService>
    ) -> Result<RawTestOutcome, AppError>;
    
    fn item_type(&self) -> SubTestItem;
}
```

## 质量保证要求

### 代码注释规范
```rust
/// 通道状态管理器 - 负责管理测试实例的状态转换
/// 
/// 这是系统中唯一允许修改 ChannelTestInstance 状态的组件
/// 符合 FAT-CSM-001 规则
pub struct ChannelStateManager {
    // 字段定义...
}
```

### 性能要求
- 使用信号量控制并发测试数量
- 使用 `Arc<T>` 共享所有权
- 避免长时间持有锁

### 安全要求
- 验证所有用户输入
- 不向前端泄露敏感信息
- 记录详细错误到日志

### 测试覆盖
- 所有核心业务逻辑必须有单元测试
- 测试覆盖率目标 > 80%
- 使用 mock 对象测试外部依赖

## 配置和部署

```rust
// 应用配置结构
pub struct AppConfig {
    pub max_concurrent_tests: usize,
    pub plc_timeout_ms: u64,
    pub database_path: PathBuf,
}

// 结构化日志
log::info!(
    "测试实例状态更新: instance_id={}, old_status={:?}, new_status={:?}",
    instance.instance_id,
    old_status,
    new_status
);
```

---

**重要提醒**：在开发过程中，始终参考 `Notes/技术栈迁移系统架构.md` 和 `Notes/技术栈迁移详细实施步骤.md` 文档，确保实现符合系统架构设计要求。遇到架构设计问题时，优先遵循这两个文档的指导原则。 