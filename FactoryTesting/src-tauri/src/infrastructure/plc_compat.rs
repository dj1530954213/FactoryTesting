//! # PLC通信兼容性适配器模块
//!
//! ## 业务作用
//! 本模块提供向后兼容的PLC通信接口，用于支持旧版代码的平滑迁移：
//! - 为旧版代码提供简化的PLC操作接口
//! - 自动处理连接句柄的获取和管理
//! - 支持默认连接和指定连接ID两种操作模式
//! - 在系统重构期间保持API的稳定性
//!
//! ## 设计模式
//! - **适配器模式**: 将新的IPlcCommunicationService接口适配为旧版简化接口
//! - **扩展trait模式**: 通过trait为现有类型添加新方法
//! - **过渡期设计**: 临时性解决方案，待迁移完成后可删除
//!
//! ## 技术特点
//! - 支持多种Arc类型的trait实现
//! - 异步trait实现，保持性能优势
//! - 自动错误处理和状态管理
//! - 线程安全的智能指针操作
//!
//! ## Rust知识点
//! - **trait扩展**: 为外部类型实现自定义trait
//! - **async trait**: 异步trait的实现和使用
//! - **Arc智能指针**: 多线程共享所有权管理
//! - **trait对象**: 动态分发和类型擦除

/*use crate::domain::services::plc_communication_service::IPlcCommunicationService;
use crate::utils::error::{AppError, AppResult};

/// PLC服务遗留扩展trait
///
/// **业务目的**:
/// - 为旧版代码提供简化的PLC操作接口
/// - 隐藏连接句柄管理的复杂性
/// - 支持默认连接和指定连接两种模式
///
/// **设计理念**:
/// - 向后兼容：保持旧版API不变
/// - 简化使用：自动处理连接管理
/// - 渐进迁移：支持逐步迁移到新接口
///
/// **Rust知识点**:
/// - `#[async_trait::async_trait]`: 支持trait中的异步方法
/// - trait扩展：为现有类型添加新功能
#[async_trait::async_trait]
pub trait PlcServiceLegacyExt {
    // === 默认连接操作方法 ===
    // **业务含义**: 使用系统默认的PLC连接进行操作
    // **适用场景**: 单PLC系统或主要PLC操作

    /// 写入32位浮点数到默认连接
    /// **参数**: address - PLC地址, value - 要写入的浮点值
    async fn write_float32(&self, address: &str, value: f32) -> AppResult<()>;

    /// 从默认连接读取32位浮点数
    /// **参数**: address - PLC地址
    /// **返回**: 读取的浮点值
    async fn read_float32(&self, address: &str) -> AppResult<f32>;

    /// 写入布尔值到默认连接
    /// **参数**: address - PLC地址, value - 要写入的布尔值
    async fn write_bool(&self, address: &str, value: bool) -> AppResult<()>;

    /// 从默认连接读取布尔值
    /// **参数**: address - PLC地址
    /// **返回**: 读取的布尔值
    async fn read_bool(&self, address: &str) -> AppResult<bool>;

    // === 指定连接ID操作方法 ===
    // **业务含义**: 使用指定的连接ID进行操作
    // **适用场景**: 多PLC系统或需要明确指定连接的场景
    // **优势**: 避免连接句柄混用，提高操作的准确性

    /// 写入32位浮点数到指定连接
    /// **参数**: connection_id - 连接ID, address - PLC地址, value - 浮点值
    async fn write_float32_by_id(&self, connection_id: &str, address: &str, value: f32) -> AppResult<()>;

    /// 从指定连接读取32位浮点数
    /// **参数**: connection_id - 连接ID, address - PLC地址
    async fn read_float32_by_id(&self, connection_id: &str, address: &str) -> AppResult<f32>;

    /// 写入布尔值到指定连接
    /// **参数**: connection_id - 连接ID, address - PLC地址, value - 布尔值
    async fn write_bool_by_id(&self, connection_id: &str, address: &str, value: bool) -> AppResult<()>;

    /// 从指定连接读取布尔值
    /// **参数**: connection_id - 连接ID, address - PLC地址
    async fn read_bool_by_id(&self, connection_id: &str, address: &str) -> AppResult<bool>;
}

/// Arc<dyn IPlcCommunicationService + Send + Sync> 的trait实现
///
/// **业务作用**: 为带有Send+Sync约束的PLC服务Arc智能指针实现兼容接口
/// **适用场景**: 多线程环境下的PLC服务使用
///
/// **Rust知识点**:
/// - `Arc<dyn Trait + Send + Sync>`: 线程安全的trait对象智能指针
/// - `Send + Sync`: 标记trait，表示类型可以在线程间安全传递和共享
/// - `as_ref()`: 将Arc<T>转换为&T引用
/// - 条件编译：通过不同的impl块支持不同的类型约束
#[async_trait::async_trait]
impl PlcServiceLegacyExt for std::sync::Arc<dyn IPlcCommunicationService + Send + Sync> {
    /// 写入32位浮点数到默认连接
    ///
    /// **实现逻辑**:
    /// 1. 获取PLC服务的引用
    /// 2. 尝试获取默认连接句柄
    /// 3. 如果句柄存在，执行写入操作
    /// 4. 如果句柄不存在，返回连接未建立错误
    ///
    /// **错误处理**: 提供详细的错误信息，包括PLC地址
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
        let handle = svc
            .default_handle()
            .await;
        let handle = if let Some(h) = handle {
            h
        } else {
            let plc_addr = crate::infrastructure::plc_communication::global_plc_service()
                .last_default_address()
                .await
                .unwrap_or_else(|| "未知地址".to_string());
            return Err(AppError::plc_communication_error(
                format!("默认PLC连接未建立 (PLC地址: {})", plc_addr)
            ));
        };
        svc.read_f32(&handle, address).await
    }
    async fn write_bool(&self, address: &str, value: bool) -> AppResult<()> {
        let svc = self.as_ref();
        let handle = svc
            .default_handle()
            .await;
        let handle = if let Some(h) = handle {
            h
        } else {
            let plc_addr = crate::infrastructure::plc_communication::global_plc_service()
                .last_default_address()
                .await
                .unwrap_or_else(|| "未知地址".to_string());
            return Err(AppError::plc_communication_error(
                format!("默认PLC连接未建立 (PLC地址: {})", plc_addr)
            ));
        };
        svc.write_bool(&handle, address, value).await
    }
    async fn read_bool(&self, address: &str) -> AppResult<bool> {
        let svc = self.as_ref();
        let handle = svc
            .default_handle()
            .await;
        let handle = if let Some(h) = handle {
            h
        } else {
            let plc_addr = crate::infrastructure::plc_communication::global_plc_service()
                .last_default_address()
                .await
                .unwrap_or_else(|| "未知地址".to_string());
            return Err(AppError::plc_communication_error(
                format!("默认PLC连接未建立 (PLC地址: {})", plc_addr)
            ));
        };
        svc.read_bool(&handle, address).await
    }

    // ---------------- 新增：按连接 ID ----------------
    async fn write_float32_by_id(&self, connection_id: &str, address: &str, value: f32) -> AppResult<()> {
        let svc = self.as_ref();
        let maybe_handle = svc.default_handle_by_id(connection_id).await;
        let handle = if let Some(h) = maybe_handle {
            h
        } else {
            return Err(AppError::plc_communication_error(
                format!("PLC连接未建立: {}", connection_id)
            ));
        };
        if let Some(mgr) = crate::infrastructure::plc_communication::get_global_plc_manager() {
            if let Some(ep) = mgr.endpoint_by_id(connection_id).await {
                log::info!("PLC {}({}) 写[{}]={}", ep, connection_id, address, value);
            }
        }
        svc.write_f32(&handle, address, value).await
    }
    async fn read_float32_by_id(&self, connection_id: &str, address: &str) -> AppResult<f32> {
        let svc = self.as_ref();
        let maybe_handle = svc.default_handle_by_id(connection_id).await;
        let handle = if let Some(h) = maybe_handle {
            h
        } else {
            return Err(AppError::plc_communication_error(
                format!("PLC连接未建立: {}", connection_id)
            ));
        };
        if let Some(mgr) = crate::infrastructure::plc_communication::get_global_plc_manager() {
            if let Some(ep) = mgr.endpoint_by_id(connection_id).await {
                log::info!("PLC {}({}) 读[{}] 请求开始", ep, connection_id, address);
            }
        }
        let value = svc.read_f32(&handle, address).await?;
        if let Some(mgr) = crate::infrastructure::plc_communication::get_global_plc_manager() {
            if let Some(ep) = mgr.endpoint_by_id(connection_id).await {
                log::info!("PLC {}({}) 读[{}]={}", ep, connection_id, address, value);
            }
        }
        Ok(value)
    }
    async fn write_bool_by_id(&self, connection_id: &str, address: &str, value: bool) -> AppResult<()> {
        let svc = self.as_ref();
        let maybe_handle = svc.default_handle_by_id(connection_id).await;
        let handle = if let Some(h) = maybe_handle {
            h
        } else {
            return Err(AppError::plc_communication_error(
                format!("PLC连接未建立: {}", connection_id)
            ));
        };
        if let Some(mgr) = crate::infrastructure::plc_communication::get_global_plc_manager() {
            if let Some(ep) = mgr.endpoint_by_id(connection_id).await {
                log::info!("PLC {} 写Bool[{}]={}", ep, address, value);
            }
        }
        svc.write_bool(&handle, address, value).await
    }
    async fn read_bool_by_id(&self, connection_id: &str, address: &str) -> AppResult<bool> {
        let svc = self.as_ref();
        let maybe_handle = svc.default_handle_by_id(connection_id).await;
        let handle = if let Some(h) = maybe_handle {
            h
        } else {
            return Err(AppError::plc_communication_error(
                format!("PLC连接未建立: {}", connection_id)
            ));
        };
        if let Some(mgr) = crate::infrastructure::plc_communication::get_global_plc_manager() {
            if let Some(ep) = mgr.endpoint_by_id(connection_id).await {
                log::info!("PLC {} 读Bool[{}] 请求开始", ep, address);
            }
        }
        let value = svc.read_bool(&handle, address).await?;
        if let Some(mgr) = crate::infrastructure::plc_communication::get_global_plc_manager() {
            if let Some(ep) = mgr.endpoint_by_id(connection_id).await {
                log::info!("PLC {} 读Bool[{}]={}", ep, address, value);
            }
        }
        Ok(value)
    }
}

/// `Arc<dyn …>` —— 无 `Send+Sync` 约束时的轻量转型
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

    // 直接转调到带 Send+Sync 的实现
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

/// `&Arc<…>` —— 避免调用方显式 `clone()` （带 `Send+Sync`）
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

/// `&Arc<…>` —— 无 `Send+Sync` 约束
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

    */