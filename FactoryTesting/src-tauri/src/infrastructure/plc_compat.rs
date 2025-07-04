// 兼容旧版 PLC 通信调用的扩展方法
// 该 trait 仅用于过渡期，待调用方全部迁移到 IPlcCommunicationService 后可删除。

use crate::domain::services::plc_communication_service::IPlcCommunicationService;
use crate::utils::error::{AppError, AppResult};

#[async_trait::async_trait]
pub trait PlcServiceLegacyExt {
    // 使用最后一次默认连接（向后兼容）
    async fn write_float32(&self, address: &str, value: f32) -> AppResult<()>;
    async fn read_float32(&self, address: &str) -> AppResult<f32>;
    async fn write_bool(&self, address: &str, value: bool) -> AppResult<()>;
    async fn read_bool(&self, address: &str) -> AppResult<bool>;

    // —— 新增 —— 按连接 ID 进行操作，避免句柄混用 ——
    async fn write_float32_by_id(&self, connection_id: &str, address: &str, value: f32) -> AppResult<()>;
    async fn read_float32_by_id(&self, connection_id: &str, address: &str) -> AppResult<f32>;
    async fn write_bool_by_id(&self, connection_id: &str, address: &str, value: bool) -> AppResult<()>;
    async fn read_bool_by_id(&self, connection_id: &str, address: &str) -> AppResult<bool>;
}

/// `Arc<dyn … + Send + Sync>` —— 正常路径
#[async_trait::async_trait]
impl PlcServiceLegacyExt for std::sync::Arc<dyn IPlcCommunicationService + Send + Sync> {
    async fn write_float32(&self, address: &str, value: f32) -> AppResult<()> {
        let svc = self.as_ref();
        let maybe_handle = svc.default_handle().await;
        let handle = if let Some(h) = maybe_handle {
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