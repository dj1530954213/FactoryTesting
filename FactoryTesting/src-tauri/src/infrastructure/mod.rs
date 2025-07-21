//! # 基础设施层模块 (Infrastructure Layer)
//!
//! ## 业务说明
//! 基础设施层负责提供技术实现细节，包括数据存储、外部通信、系统集成等
//! 支撑领域层和应用层的业务逻辑实现，但不包含业务规则
//!
//! ## 架构职责
//! - **依赖注入容器**: 管理系统中各种服务的生命周期和依赖关系
//! - **PLC通信**: 处理与工业设备的通信协议（Modbus、S7等）
//! - **数据存储**: 提供数据库访问、文件操作等持久化功能
//! - **外部系统集成**: Excel导入导出、报告生成等
//!
//! ## 设计原则
//! - **关注分离**: 基础设施代码与业务逻辑分离
//! - **可替换性**: 通过接口定义，支持不同的技术实现
//! - **配置驱动**: 通过配置文件控制技术选型和参数
//!
//! ## Rust知识点
//! - **模块组织**: 使用pub mod声明公开子模块
//! - **重新导出**: 通过pub use简化外部访问路径
//! - **条件编译**: 支持不同环境下的实现切换

pub mod di_container;
pub mod plc_communication;
pub mod range_register_repository;
// pub mod plc_compat; // 已迁移到 domain::services::plc_comm_extension
pub mod extra; // 临时迁移的基础设施代码，后续合并重构

// 重新导出基础设施组件
pub use di_container::*;
pub use plc_communication::*;
pub use range_register_repository::*;
// 兼容层仅供过渡使用，保持显式路径引用，避免重复导出造成歧义
// pub use plc_compat::*;
pub use extra::*;
