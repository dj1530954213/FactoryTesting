/// 领域层模块
/// 
/// 业务说明：
/// 本模块是系统的领域层(Domain Layer)，基于领域驱动设计(DDD)架构
/// 负责封装核心业务逻辑，定义业务规则和领域服务
/// 领域层不依赖具体的技术实现，只关注业务本身
/// 
/// 架构定位：
/// - 上层：应用层(application) - 编排领域服务完成用例
/// - 当前：领域层(domain) - 核心业务逻辑和规则
/// - 下层：基础设施层(infrastructure) - 技术实现细节
/// 
/// 模块组织：
/// - services: 领域服务接口定义（trait）
/// - impls: 领域服务的具体实现
/// 
/// 设计原则：
/// - 依赖倒置：通过trait定义接口，实现依赖于抽象
/// - 关注分离：业务逻辑与技术实现分离
/// - 领域驱动：以业务概念为中心组织代码
/// 
/// Rust知识点：
/// - pub mod 声明公开的子模块
/// - pub use 重新导出，简化外部访问路径

/// 领域服务接口定义模块
/// 
/// 业务说明：
/// 定义所有领域服务的trait接口，包括：
/// - 批次分配服务 (BatchAllocationService)
/// - 通道状态管理 (ChannelStateManager)
/// - 事件发布服务 (EventPublisher)
/// - 持久化服务 (PersistenceService)
/// - PLC通信服务 (PlcCommunicationService)
/// - 量程计算服务 (RangeValueCalculator)
/// - 测试执行引擎 (TestExecutionEngine)
/// - 测试编排服务 (TestOrchestrationService)
pub mod services;

/// 领域服务实现模块
/// 
/// 业务说明：
/// 提供领域服务接口的具体实现，包括：
/// - 真实的业务实现 (real_*)
/// - 空实现用于测试 (noop_*)
/// - 特定功能实现 (specific_test_executors等)
/// 
/// 实现策略：
/// - 每个trait可能有多个实现版本
/// - 通过依赖注入选择具体实现
/// - 支持Mock实现便于单元测试
pub mod impls;

/// 重新导出领域服务接口
/// 
/// 业务说明：
/// 将services模块中的所有公开项重新导出到domain模块
/// 使外部代码可以直接通过 domain::ServiceTrait 访问
/// 而不需要 domain::services::ServiceTrait
/// 
/// 设计意图：
/// - 简化导入路径
/// - 隐藏内部模块结构
/// - 提供稳定的公共API
/// 
/// Rust知识点：
/// - pub use module::* 导出模块中的所有公开项
/// - 这是Rust中常见的API设计模式
pub use services::*;

/// 为过渡期显式重新导出实现模块
/// 
/// 业务说明：
/// 这是一个临时的兼容性措施，用于平滑迁移
/// 旧代码可能直接依赖这些具体实现模块
/// 通过重新导出避免大量修改现有代码
/// 
/// 重新导出的模块：
/// - specific_test_executors: 特定测试执行器（AI/AO/DI/DO）
/// - test_plc_config_service: 测试PLC配置服务
/// - plc_connection_manager: PLC连接管理器
/// 
/// 迁移计划：
/// - 逐步将直接依赖改为通过trait接口依赖
/// - 使用依赖注入而非直接引用实现
/// - 最终移除这些重新导出
/// 
/// Rust知识点：
/// - 选择性重新导出特定项
/// - 模块路径的简化
pub use impls::{
    specific_test_executors,
    test_plc_config_service,
    plc_connection_manager,
};
