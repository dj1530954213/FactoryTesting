//! # 数据库实体模块 (Database Entities Module)
//!
//! ## 业务说明
//! 本模块包含所有SeaORM数据库实体的定义，对应数据库中的表结构
//! 每个实体都完全匹配数据库表的字段结构，支持ORM操作和数据持久化
//!
//! ## 实体分类
//! ### 核心业务实体
//! - **channel_point_definition**: 通道点位定义表，存储测试点位的完整配置信息
//! - **test_batch_info**: 测试批次信息表，管理测试批次的元数据
//! - **channel_test_instance**: 通道测试实例表，记录具体的测试执行状态
//! - **raw_test_outcome**: 原始测试结果表，存储测试的详细结果数据
//!
//! ### PLC配置实体
//! - **test_plc_channel_config**: 测试PLC通道配置表
//! - **plc_connection_config**: PLC连接配置表
//! - **channel_mapping_config**: 通道映射配置表
//!
//! ### 系统功能实体
//! - **global_function_test_status**: 全局功能测试状态表
//! - **range_register**: PLC量程寄存器配置表
//!
//! ## 设计特点
//! - **完整映射**: 每个实体都完整映射数据库表结构
//! - **类型安全**: 使用Rust类型系统确保数据一致性
//! - **序列化支持**: 所有实体都支持JSON序列化
//! - **关联关系**: 通过外键维护实体间的关联关系
//!
//! ## Rust知识点
//! - **SeaORM**: 使用SeaORM宏自动生成数据库操作代码
//! - **derive宏**: 自动实现常用trait如Serialize、Deserialize
//! - **模块组织**: 每个实体一个独立文件，便于维护

pub mod channel_point_definition;
pub mod test_batch_info;
pub mod channel_test_instance;
pub mod raw_test_outcome;

// 测试PLC配置相关实体
pub mod test_plc_channel_config;
pub mod plc_connection_config;
pub mod channel_mapping_config;

pub mod global_function_test_status;

// 存放 PLC 量程寄存器
pub mod range_register;

// 后续会在这里添加其他实体模块的声明，例如：
// pub mod raw_test_outcome; 
