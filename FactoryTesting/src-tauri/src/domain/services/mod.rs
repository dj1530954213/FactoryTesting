/// 领域服务接口定义模块
/// 
/// 业务说明：
/// 本模块是领域层的核心，定义了所有业务服务的trait接口
/// 这些接口描述了业务能力，但不包含具体实现
/// 遵循依赖倒置原则(DIP)，高层模块不依赖低层模块
/// 
/// 架构设计：
/// - 接口定义在领域层，实现在基础设施层
/// - 通过依赖注入在运行时绑定具体实现
/// - 支持多种实现方式（真实实现、Mock实现等）
/// 
/// 模块组织原则：
/// - 每个服务一个独立模块文件
/// - 服务接口以trait形式定义
/// - 相关的类型定义放在对应模块中
/// 
/// Rust知识点：
/// - trait定义抽象接口
/// - async_trait支持异步trait方法

/// 测试编排服务
/// 
/// 业务说明：编排整个测试流程，协调各个服务完成测试任务
/// 主要职责：批次管理、测试流程控制、状态同步
pub mod test_orchestration_service;

/// 通道状态管理器
/// 
/// 业务说明：管理测试通道的状态转换和状态查询
/// 支持：状态更新、状态查询、状态监控、批量操作
pub mod channel_state_manager;

/// 测试执行引擎
/// 
/// 业务说明：执行具体的测试任务，包括各类型通道测试
/// 支持：AI/AO/DI/DO通道测试、硬接点测试、报警测试
pub mod test_execution_engine;

/// PLC通信服务
/// 
/// 业务说明：与PLC设备进行通信的核心服务
/// 支持：读写操作、连接管理、协议适配
pub mod plc_communication_service;

/// PLC通信扩展
/// 
/// 业务说明：扩展PLC通信能力，提供高级功能
/// 支持：批量读写、事务支持、性能优化
pub mod plc_comm_extension;

/// 量程寄存器仓储
/// 
/// 业务说明：管理AO通道的量程寄存器映射关系
/// 支持：寄存器查询、映射配置、缓存管理
pub mod range_register_repository;

/// 量程值计算器
/// 
/// 业务说明：计算模拟量通道的工程值和百分比值
/// 支持：线性转换、非线性校正、单位换算
pub mod range_value_calculator;

/// 批次分配服务
/// 
/// 业务说明：为测试任务分配批次和资源
/// 支持：智能分配、负载均衡、资源优化
pub mod batch_allocation_service;

/// 事件发布服务
/// 
/// 业务说明：发布领域事件，支持事件驱动架构
/// 支持：事件发布、订阅管理、异步通知
pub mod event_publisher;

/// 持久化服务
/// 
/// 业务说明：提供数据持久化的统一接口
/// 支持：实体CRUD、事务管理、查询优化
pub mod persistence_service;


/// 重新导出所有服务接口
/// 
/// 业务说明：
/// 将各个子模块中定义的服务接口统一导出
/// 外部代码可以直接从services模块访问所有接口
/// 无需了解具体的模块组织结构
/// 
/// 使用示例：
/// ```rust
/// use crate::domain::services::{ITestOrchestrationService, IChannelStateManager};
/// ```
/// 
/// Rust知识点：
/// - pub use module::* 导出模块中的所有公开项
/// - 简化了外部模块的导入语句
pub use test_orchestration_service::*;
pub use channel_state_manager::*;
pub use test_execution_engine::*;
pub use plc_communication_service::*;
pub use plc_comm_extension::*;
pub use range_register_repository::*;
pub use range_value_calculator::*;
pub use batch_allocation_service::*;
pub use event_publisher::*;
pub use persistence_service::*;

/// ---------------------------------------------------------------------------
/// 兼容旧路径 / 命名的重新导出（Backward-compatibility Aliases）
/// 
/// 业务说明：
/// 这部分是为了保持向后兼容性而存在的临时措施
/// 在系统重构过程中，避免破坏现有代码
/// 计划在 Phase 3 清理阶段完全移除
/// 
/// 兼容策略：
/// 1. 将具体实现类重新导出到services命名空间
/// 2. 为新的trait名称（带I前缀）提供旧名称别名
/// 3. 保持原有的导入路径可用
/// ---------------------------------------------------------------------------

/// 将具体实现结构体重新导出到 `services` 命名空间
/// 
/// 业务说明：
/// 旧代码可能直接从services模块导入这些实现类
/// 通过重新导出保持兼容，避免大规模修改
/// 
/// 迁移建议：
/// - 新代码应该通过trait接口而非具体实现
/// - 使用依赖注入获取服务实例
pub use crate::domain::impls::{
    ChannelStateManager,      // 通道状态管理器实现
    TestExecutionEngine,       // 测试执行引擎实现
    TestPlcConfigService,      // 测试PLC配置服务实现
    PlcConnectionManager,      // PLC连接管理器实现
};

/// 为旧的不带 `I` 前缀的 trait 名称提供别名
/// 
/// 业务说明：
/// 新的命名规范要求trait名称以I开头（如IEventPublisher）
/// 但旧代码使用不带I的名称（如EventPublisher）
/// 通过别名保持兼容
/// 
/// Rust知识点：
/// - as关键字创建类型别名
/// - 允许同一个类型有多个名称
pub use event_publisher::IEventPublisher as EventPublisher;
pub use persistence_service::IPersistenceService as PersistenceService;

/// 将 ITestPlcConfigService 重新导出到 services 命名空间
/// 
/// 业务说明：
/// 特殊处理TestPlcConfigService的trait接口
/// 确保新旧代码都能正确访问
pub use crate::domain::impls::test_plc_config_service::ITestPlcConfigService;

/// 重新导出枚举类型以便在领域服务中方便访问
/// 
/// 业务说明：
/// 领域服务经常需要使用各种枚举类型
/// 如：TestStatus、ChannelType、ModuleType等
/// 通过重新导出避免冗长的导入路径
/// 
/// Rust知识点：
/// - 从models模块导出所有枚举
/// - 提高代码可读性
pub use crate::models::enums::*;

/// 导入基础类型和依赖
/// 
/// 业务说明：
/// 导入领域服务共同需要的基础类型和第三方依赖
/// 这些类型在多个服务中被广泛使用
/// 
/// 导入说明：
/// - AppResult: 统一的错误处理类型
/// - models::structs: 业务模型结构体
/// - async_trait: 支持异步trait方法
/// - HashMap: 键值对集合，用于数据映射
/// - chrono: 时间处理库
/// - serde: 序列化/反序列化支持
use crate::utils::error::AppResult;
use crate::models::structs::*;
use async_trait::async_trait;
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// 基础服务trait，所有服务都应实现
/// 
/// 业务说明：
/// 定义了所有领域服务的基础行为规范
/// 确保服务具有统一的生命周期管理能力
/// 
/// 设计意图：
/// - 标准化服务接口
/// - 支持服务健康监控
/// - 便于服务容器管理
/// 
/// Rust知识点：
/// - Send + Sync: 确保服务可以在线程间安全共享
/// - &'static str: 静态字符串生命周期
/// - async_trait: 使trait支持async方法
#[async_trait]
pub trait BaseService: Send + Sync {
    /// 服务名称
    /// 
    /// 业务说明：返回服务的唯一标识名称
    /// 用于日志记录、监控和调试
    fn service_name(&self) -> &'static str;
    
    /// 初始化服务
    /// 
    /// 业务说明：
    /// 在服务启动时调用，执行必要的初始化操作
    /// 如：建立连接、加载配置、预热缓存等
    /// 
    /// 错误处理：
    /// 初始化失败应返回明确的错误信息
    async fn initialize(&mut self) -> AppResult<()>;
    
    /// 关闭服务
    /// 
    /// 业务说明：
    /// 在服务停止时调用，执行清理操作
    /// 如：关闭连接、保存状态、释放资源等
    /// 
    /// 注意事项：
    /// 应确保优雅关闭，避免数据丢失
    async fn shutdown(&mut self) -> AppResult<()>;
    
    /// 健康检查
    /// 
    /// 业务说明：
    /// 检查服务当前是否正常运行
    /// 用于监控系统和负载均衡决策
    /// 
    /// 检查内容：
    /// - 依赖服务可用性
    /// - 资源使用情况
    /// - 关键功能可用性
    async fn health_check(&self) -> AppResult<()>;
}

/// 测试值类型
/// 
/// 业务说明：
/// 统一表示测试过程中的各种数据类型
/// 支持布尔、整数、浮点数和字符串四种基本类型
/// 用于测试结果、期望值、实际值的统一表示
/// 
/// 使用场景：
/// - DI/DO通道：Bool类型
/// - AI/AO通道：Float类型
/// - 计数器通道：Int类型
/// - 状态描述：String类型
/// 
/// Rust知识点：
/// - enum枚举支持关联数据
/// - #[derive] 自动实现常用trait
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TestValue {
    /// 布尔值，用于开关量信号
    Bool(bool),
    /// 整数值，用于计数器、状态码等
    Int(i32),
    /// 浮点值，用于模拟量信号
    Float(f32),
    /// 字符串值，用于状态描述、错误信息等
    String(String),
}

/// 测试结果类型
/// 
/// 业务说明：
/// 封装单个测试项的完整结果信息
/// 包含成功状态、测试值、错误信息和执行时间
/// 是测试执行的基本返回类型
/// 
/// 字段说明：
/// - success: 测试是否通过
/// - actual_value: 实际读取或计算的值
/// - expected_value: 期望的目标值
/// - error_message: 失败时的错误描述
/// - execution_time_ms: 执行耗时（毫秒）
/// - timestamp: 执行时间戳
/// 
/// Rust知识点：
/// - Option<T> 表示可选值
/// - DateTime<Utc> 使用UTC时区避免时区问题
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestResult {
    /// 测试是否成功
    pub success: bool,
    /// 实际测试值
    pub actual_value: Option<TestValue>,
    /// 期望值
    pub expected_value: Option<TestValue>,
    /// 错误信息（失败时）
    pub error_message: Option<String>,
    /// 执行时间（毫秒）
    pub execution_time_ms: u64,
    /// 测试时间戳
    pub timestamp: DateTime<Utc>,
}

/// 批次统计信息
/// 
/// 业务说明：
/// 实时统计测试批次的执行情况
/// 提供全面的进度和状态信息
/// 用于前端展示和决策支持
/// 
/// 统计维度：
/// - 总数统计：总通道数
/// - 状态统计：已测、通过、失败、跳过、进行中
/// - 时间统计：开始时间、结束时间、预计完成时间
/// 
/// 使用场景：
/// - 进度条显示
/// - 测试报告生成
/// - 性能分析
/// 
/// Rust知识点：
/// - u32用于计数，避免负数
/// - Option<DateTime> 表示可能未开始或未结束
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchStatistics {
    /// 总通道数
    pub total_channels: u32,
    /// 已测试通道数
    pub tested_channels: u32,
    /// 测试通过的通道数
    pub passed_channels: u32,
    /// 测试失败的通道数
    pub failed_channels: u32,
    /// 跳过的通道数
    pub skipped_channels: u32,
    /// 正在测试的通道数
    pub in_progress_channels: u32,
    /// 批次开始时间
    pub start_time: Option<DateTime<Utc>>,
    /// 批次结束时间
    pub end_time: Option<DateTime<Utc>>,
    /// 预计完成时间
    pub estimated_completion_time: Option<DateTime<Utc>>,
}

/// 测试进度更新信息
/// 
/// 业务说明：
/// 封装测试进度的实时更新数据
/// 用于向前端推送进度通知
/// 支持细粒度的进度跟踪
/// 
/// 更新内容：
/// - 标识信息：批次ID、实例ID
/// - 进度信息：百分比、当前步骤
/// - 时间信息：剩余时间估算
/// - 统计信息：完整的批次统计
/// 
/// 推送时机：
/// - 通道测试开始/结束
/// - 重要步骤完成
/// - 定期进度更新
/// 
/// Rust知识点：
/// - f32表示百分比（0.0-100.0）
/// - 嵌套结构体组合数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestProgressUpdate {
    /// 批次ID
    pub batch_id: String,
    /// 测试实例ID
    pub instance_id: String,
    /// 进度百分比(0.0-100.0)
    pub progress_percentage: f32,
    /// 当前执行步骤描述
    pub current_step: String,
    /// 预计剩余时间（毫秒）
    pub estimated_remaining_time_ms: Option<u64>,
    /// 批次统计信息
    pub statistics: BatchStatistics,
    /// 更新时间戳
    pub timestamp: DateTime<Utc>,
}

/// 服务健康状态
/// 
/// 业务说明：
/// 记录单个服务的健康检查结果
/// 用于系统监控和故障诊断
/// 支持服务网格和负载均衡
/// 
/// 检查项目：
/// - 服务可用性
/// - 最近检查时间
/// - 错误信息（如果不健康）
/// - 运行时长
/// 
/// 应用场景：
/// - 健康检查端点
/// - 监控告警
/// - 自动故障转移
/// 
/// Rust知识点：
/// - bool表示健康/不健康二元状态
/// - uptime_seconds用于计算可用性指标
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceHealth {
    /// 服务名称
    pub service_name: String,
    /// 是否健康
    pub is_healthy: bool,
    /// 最后检查时间
    pub last_check: DateTime<Utc>,
    /// 错误信息（不健康时）
    pub error_message: Option<String>,
    /// 服务运行时长（秒）
    pub uptime_seconds: u64,
}
