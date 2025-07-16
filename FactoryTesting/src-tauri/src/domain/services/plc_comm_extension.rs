//! # PLC通信便捷扩展模块（过渡期）
//!
//! ## 业务作用
//! 本模块提供PLC通信的便捷扩展接口，主要用于系统重构过渡期：
//! - 为旧版代码提供简化的PLC操作接口
//! - 自动处理连接句柄的获取和管理
//! - 支持默认连接和指定连接ID两种操作模式
//! - 减少代码迁移的工作量和风险
//!
//! ## 设计目标
//! - **平滑迁移**: 最小化旧代码的修改量
//! - **向后兼容**: 保持现有API的稳定性
//! - **渐进重构**: 支持逐步迁移到新架构
//! - **临时性质**: 迁移完成后可以安全删除
//!
//! ## 架构位置
//! - 位于领域层，避免与基础设施层直接耦合
//! - 作为适配器层，连接新旧接口
//! - 提供统一的错误处理和日志记录
//!
//! ## 使用场景
//! - 旧版测试代码的快速适配
//! - 简化单PLC系统的操作接口
//! - 提供更直观的API给业务开发人员
//!
//! ## Rust知识点
//! - **trait扩展**: 为现有类型添加新方法
//! - **async trait**: 异步trait的实现
//! - **泛型约束**: 不同类型的trait实现
//! - **错误传播**: 统一的错误处理机制

use crate::domain::services::plc_communication_service::IPlcCommunicationService;
use crate::utils::error::{AppError, AppResult};

/// PLC服务遗留扩展trait
///
/// **业务目的**:
/// - 为旧版代码提供简化的PLC操作接口
/// - 隐藏连接句柄管理的复杂性
/// - 支持两种操作模式：默认连接和指定连接
///
/// **设计理念**:
/// - **简化使用**: 自动处理连接管理细节
/// - **向后兼容**: 保持旧版API不变
/// - **类型安全**: 使用强类型确保操作正确性
/// - **错误友好**: 提供清晰的错误信息
///
/// **方法分类**:
/// 1. **默认连接方法**: 使用系统默认PLC连接
/// 2. **指定连接方法**: 使用特定连接ID的PLC连接
///
/// **Rust知识点**:
/// - `#[async_trait::async_trait]`: 支持trait中的异步方法
/// - trait扩展：为现有类型添加新功能
/// - 方法命名约定：`_by_id`后缀表示需要指定连接ID
#[async_trait::async_trait]
pub trait PlcServiceLegacyExt {
    // === 默认连接操作方法 ===
    // **适用场景**: 单PLC系统或主要PLC操作
    // **优势**: 使用简单，无需管理连接ID

    /// 写入32位浮点数到默认PLC连接
    /// **业务场景**: 设置温度、压力等模拟量设定值
    async fn write_float32(&self, address: &str, value: f32) -> AppResult<()>;

    /// 从默认PLC连接读取32位浮点数
    /// **业务场景**: 读取传感器数值、过程变量等
    async fn read_float32(&self, address: &str) -> AppResult<f32>;

    /// 写入布尔值到默认PLC连接
    /// **业务场景**: 控制开关、启停设备等
    async fn write_bool(&self, address: &str, value: bool) -> AppResult<()>;

    /// 从默认PLC连接读取布尔值
    /// **业务场景**: 读取开关状态、报警信号等
    async fn read_bool(&self, address: &str) -> AppResult<bool>;

    // === 指定连接ID操作方法 ===
    // **适用场景**: 多PLC系统或需要明确指定连接的场景
    // **优势**: 避免连接混用，提高操作准确性

    /// 写入32位浮点数到指定PLC连接
    /// **业务场景**: 多PLC系统中的精确控制
    async fn write_float32_by_id(&self, connection_id: &str, address: &str, value: f32) -> AppResult<()>;

    /// 从指定PLC连接读取32位浮点数
    /// **业务场景**: 多PLC系统中的数据采集
    async fn read_float32_by_id(&self, connection_id: &str, address: &str) -> AppResult<f32>;

    /// 写入布尔值到指定PLC连接
    /// **业务场景**: 多PLC系统中的开关控制
    async fn write_bool_by_id(&self, connection_id: &str, address: &str, value: bool) -> AppResult<()>;

    /// 从指定PLC连接读取布尔值
    /// **业务场景**: 多PLC系统中的状态监控
    async fn read_bool_by_id(&self, connection_id: &str, address: &str) -> AppResult<bool>;
}

// === Arc<dyn IPlcCommunicationService + Send + Sync> 的trait实现 ===
///
/// **适用类型**: 带有Send+Sync约束的PLC服务Arc智能指针
/// **线程安全**: 支持多线程环境下的并发访问
/// **使用场景**: 大多数多线程应用场景
///
/// **Rust知识点**:
/// - `Arc<dyn Trait + Send + Sync>`: 线程安全的trait对象智能指针
/// - `Send + Sync`: 标记trait，表示类型可以在线程间安全传递和共享
/// - `as_ref()`: 将Arc<T>转换为&T引用，避免所有权转移
#[async_trait::async_trait]
impl PlcServiceLegacyExt for std::sync::Arc<dyn IPlcCommunicationService + Send + Sync> {
    /// 写入32位浮点数到默认连接
    ///
    /// **实现逻辑**:
    /// 1. 获取PLC服务的引用
    /// 2. 尝试获取默认连接句柄
    /// 3. 如果句柄存在，执行写入操作
    /// 4. 如果句柄不存在，返回详细的错误信息
    ///
    /// **错误处理**:
    /// - 提供PLC地址信息用于故障诊断
    /// - 使用统一的错误类型和格式
    /// - 支持错误链追踪
    async fn write_float32(&self, address: &str, value: f32) -> AppResult<()> {
        let svc = self.as_ref(); // 获取trait对象的引用
        let maybe_handle = svc.default_handle().await; // 尝试获取默认连接句柄

        // 检查默认连接句柄是否存在
        let handle = if let Some(h) = maybe_handle {
            h // 句柄存在，使用该句柄
        } else {
            // 句柄不存在，获取PLC地址用于错误信息
            let plc_addr = crate::infrastructure::plc_communication::global_plc_service()
                .last_default_address()
                .await
                .unwrap_or_else(|| "未知地址".to_string());
            return Err(AppError::plc_communication_error(
                format!("默认PLC连接未建立 (PLC地址: {})", plc_addr)
            ));
        };

        // 使用获取的句柄执行写入操作
        svc.write_f32(&handle, address, value).await
    }

    async fn read_float32(&self, address: &str) -> AppResult<f32> {
        let svc = self.as_ref();
        let handle = if let Some(h) = svc.default_handle().await {
            h
        } else {
            let plc_addr = crate::infrastructure::plc_communication::global_plc_service()
                .last_default_address()
                .await
                .unwrap_or_else(|| "未知地址".to_string());
            return Err(AppError::plc_communication_error(format!("默认PLC连接未建立 (PLC地址: {})", plc_addr)));
        };
        svc.read_f32(&handle, address).await
    }

    async fn write_bool(&self, address: &str, value: bool) -> AppResult<()> {
        let svc = self.as_ref();
        let handle = if let Some(h) = svc.default_handle().await {
            h
        } else {
            let plc_addr = crate::infrastructure::plc_communication::global_plc_service()
                .last_default_address()
                .await
                .unwrap_or_else(|| "未知地址".to_string());
            return Err(AppError::plc_communication_error(format!("默认PLC连接未建立 (PLC地址: {})", plc_addr)));
        };
        svc.write_bool(&handle, address, value).await
    }

    async fn read_bool(&self, address: &str) -> AppResult<bool> {
        let svc = self.as_ref();
        let handle = if let Some(h) = svc.default_handle().await {
            h
        } else {
            let plc_addr = crate::infrastructure::plc_communication::global_plc_service()
                .last_default_address()
                .await
                .unwrap_or_else(|| "未知地址".to_string());
            return Err(AppError::plc_communication_error(format!("默认PLC连接未建立 (PLC地址: {})", plc_addr)));
        };
        svc.read_bool(&handle, address).await
    }

    // ---------------- 按连接 ID ----------------
    async fn write_float32_by_id(&self, connection_id: &str, address: &str, value: f32) -> AppResult<()> {
        let svc = self.as_ref();
        let maybe_handle = svc.default_handle_by_id(connection_id).await;
        let handle = maybe_handle.ok_or_else(|| AppError::plc_communication_error(format!("PLC连接未建立: {}", connection_id)))?;

        if let Some(mgr) = crate::domain::services::plc_communication_service::get_global_plc_manager() {
            if let Some(ep) = mgr.endpoint_by_id(connection_id).await {
                log::info!("PLC {}({}) 写[{}]={}", ep, connection_id, address, value);
            }
        }
        svc.write_f32(&handle, address, value).await
    }

    async fn read_float32_by_id(&self, connection_id: &str, address: &str) -> AppResult<f32> {
        let svc = self.as_ref();
        let handle = svc.default_handle_by_id(connection_id)
            .await
            .ok_or_else(|| AppError::plc_communication_error(format!("PLC连接未建立: {}", connection_id)))?;

        if let Some(mgr) = crate::domain::services::plc_communication_service::get_global_plc_manager() {
            if let Some(ep) = mgr.endpoint_by_id(connection_id).await {
                //log::info!("PLC {}({}) 读[{}] 请求开始", ep, connection_id, address);
            }
        }
        let value = svc.read_f32(&handle, address).await?;
        if let Some(mgr) = crate::domain::services::plc_communication_service::get_global_plc_manager() {
            if let Some(ep) = mgr.endpoint_by_id(connection_id).await {
                //log::info!("PLC {}({}) 读[{}]={}", ep, connection_id, address, value);
            }
        }
        Ok(value)
    }

    async fn write_bool_by_id(&self, connection_id: &str, address: &str, value: bool) -> AppResult<()> {
        let svc = self.as_ref();
        let handle = svc.default_handle_by_id(connection_id)
            .await
            .ok_or_else(|| AppError::plc_communication_error(format!("PLC连接未建立: {}", connection_id)))?;

        if let Some(mgr) = crate::domain::services::plc_communication_service::get_global_plc_manager() {
            if let Some(ep) = mgr.endpoint_by_id(connection_id).await {
                log::info!("PLC {} 写Bool[{}]={}", ep, address, value);
            }
        }
        svc.write_bool(&handle, address, value).await
    }

    async fn read_bool_by_id(&self, connection_id: &str, address: &str) -> AppResult<bool> {
        let svc = self.as_ref();
        let handle = svc.default_handle_by_id(connection_id)
            .await
            .ok_or_else(|| AppError::plc_communication_error(format!("PLC连接未建立: {}", connection_id)))?;

        if let Some(mgr) = crate::domain::services::plc_communication_service::get_global_plc_manager() {
            if let Some(ep) = mgr.endpoint_by_id(connection_id).await {
                //log::info!("PLC {} 读Bool[{}] 请求开始", ep, address);
            }
        }
        let value = svc.read_bool(&handle, address).await?;
        if let Some(mgr) = crate::domain::services::plc_communication_service::get_global_plc_manager() {
            if let Some(ep) = mgr.endpoint_by_id(connection_id).await {
                //log::info!("PLC {} 读Bool[{}]={}", ep, address, value);
            }
        }
        Ok(value)
    }
}

// ---------------- 无 Send+Sync 版本 Forward ----------------
#[async_trait::async_trait]
impl PlcServiceLegacyExt for std::sync::Arc<dyn IPlcCommunicationService> {
    async fn write_float32(&self, address: &str, value: f32) -> AppResult<()> {
        let tmp = self.clone() as std::sync::Arc<dyn IPlcCommunicationService + Send + Sync>;
        tmp.write_float32(address, value).await
    }
    async fn read_float32(&self, address: &str) -> AppResult<f32> {
        let tmp = self.clone() as std::sync::Arc<dyn IPlcCommunicationService + Send + Sync>;
        tmp.read_float32(address).await
    }
    async fn write_bool(&self, address: &str, value: bool) -> AppResult<()> {
        let tmp = self.clone() as std::sync::Arc<dyn IPlcCommunicationService + Send + Sync>;
        tmp.write_bool(address, value).await
    }
    async fn read_bool(&self, address: &str) -> AppResult<bool> {
        let tmp = self.clone() as std::sync::Arc<dyn IPlcCommunicationService + Send + Sync>;
        tmp.read_bool(address).await
    }

    async fn write_float32_by_id(&self, connection_id: &str, address: &str, value: f32) -> AppResult<()> {
        let tmp = self.clone() as std::sync::Arc<dyn IPlcCommunicationService + Send + Sync>;
        tmp.write_float32_by_id(connection_id, address, value).await
    }
    async fn read_float32_by_id(&self, connection_id: &str, address: &str) -> AppResult<f32> {
        let tmp = self.clone() as std::sync::Arc<dyn IPlcCommunicationService + Send + Sync>;
        tmp.read_float32_by_id(connection_id, address).await
    }
    async fn write_bool_by_id(&self, connection_id: &str, address: &str, value: bool) -> AppResult<()> {
        let tmp = self.clone() as std::sync::Arc<dyn IPlcCommunicationService + Send + Sync>;
        tmp.write_bool_by_id(connection_id, address, value).await
    }
    async fn read_bool_by_id(&self, connection_id: &str, address: &str) -> AppResult<bool> {
        let tmp = self.clone() as std::sync::Arc<dyn IPlcCommunicationService + Send + Sync>;
        tmp.read_bool_by_id(connection_id, address).await
    }
}

// ---------------- &Arc Forwarding (Send+Sync) ----------------
#[async_trait::async_trait]
impl PlcServiceLegacyExt for &std::sync::Arc<dyn IPlcCommunicationService + Send + Sync> {
    async fn write_float32(&self, address: &str, value: f32) -> AppResult<()> {
        (*self).write_float32(address, value).await
    }
    async fn read_float32(&self, address: &str) -> AppResult<f32> {
        (*self).read_float32(address).await
    }
    async fn write_bool(&self, address: &str, value: bool) -> AppResult<()> {
        (*self).write_bool(address, value).await
    }
    async fn read_bool(&self, address: &str) -> AppResult<bool> {
        (*self).read_bool(address).await
    }

    async fn write_float32_by_id(&self, connection_id: &str, address: &str, value: f32) -> AppResult<()> {
        (*self).write_float32_by_id(connection_id, address, value).await
    }
    async fn read_float32_by_id(&self, connection_id: &str, address: &str) -> AppResult<f32> {
        (*self).read_float32_by_id(connection_id, address).await
    }
    async fn write_bool_by_id(&self, connection_id: &str, address: &str, value: bool) -> AppResult<()> {
        (*self).write_bool_by_id(connection_id, address, value).await
    }
    async fn read_bool_by_id(&self, connection_id: &str, address: &str) -> AppResult<bool> {
        (*self).read_bool_by_id(connection_id, address).await
    }
}

// ---------------- &Arc Forwarding (no Send+Sync) ----------------
#[async_trait::async_trait]
impl PlcServiceLegacyExt for &std::sync::Arc<dyn IPlcCommunicationService> {
    async fn write_float32(&self, address: &str, value: f32) -> AppResult<()> {
        (*self).write_float32(address, value).await
    }
    async fn read_float32(&self, address: &str) -> AppResult<f32> {
        (*self).read_float32(address).await
    }
    async fn write_bool(&self, address: &str, value: bool) -> AppResult<()> {
        (*self).write_bool(address, value).await
    }
    async fn read_bool(&self, address: &str) -> AppResult<bool> {
        (*self).read_bool(address).await
    }

    async fn write_float32_by_id(&self, connection_id: &str, address: &str, value: f32) -> AppResult<()> {
        (*self).write_float32_by_id(connection_id, address, value).await
    }
    async fn read_float32_by_id(&self, connection_id: &str, address: &str) -> AppResult<f32> {
        (*self).read_float32_by_id(connection_id, address).await
    }
    async fn write_bool_by_id(&self, connection_id: &str, address: &str, value: bool) -> AppResult<()> {
        (*self).write_bool_by_id(connection_id, address, value).await
    }
    async fn read_bool_by_id(&self, connection_id: &str, address: &str) -> AppResult<bool> {
        (*self).read_bool_by_id(connection_id, address).await
    }
}
