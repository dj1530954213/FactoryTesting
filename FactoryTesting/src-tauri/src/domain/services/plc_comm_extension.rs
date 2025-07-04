//! PLC 通信便捷扩展（过渡期）
//! 将旧版 Modbus 兼容辅助 trait 迁移到 domain 层，避免与基础设施层耦合。

use crate::domain::services::plc_communication_service::IPlcCommunicationService;
use crate::utils::error::{AppError, AppResult};

#[async_trait::async_trait]
pub trait PlcServiceLegacyExt {
    async fn write_float32(&self, address: &str, value: f32) -> AppResult<()>;
    async fn read_float32(&self, address: &str) -> AppResult<f32>;
    async fn write_bool(&self, address: &str, value: bool) -> AppResult<()>;
    async fn read_bool(&self, address: &str) -> AppResult<bool>;

    async fn write_float32_by_id(&self, connection_id: &str, address: &str, value: f32) -> AppResult<()>;
    async fn read_float32_by_id(&self, connection_id: &str, address: &str) -> AppResult<f32>;
    async fn write_bool_by_id(&self, connection_id: &str, address: &str, value: bool) -> AppResult<()>;
    async fn read_bool_by_id(&self, connection_id: &str, address: &str) -> AppResult<bool>;
}

// ---------------- Arc<dyn … + Send + Sync> ----------------
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
            return Err(AppError::plc_communication_error(format!("默认PLC连接未建立 (PLC地址: {})", plc_addr)));
        };
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
                log::info!("PLC {}({}) 读[{}] 请求开始", ep, connection_id, address);
            }
        }
        let value = svc.read_f32(&handle, address).await?;
        if let Some(mgr) = crate::domain::services::plc_communication_service::get_global_plc_manager() {
            if let Some(ep) = mgr.endpoint_by_id(connection_id).await {
                log::info!("PLC {}({}) 读[{}]={}", ep, connection_id, address, value);
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
                log::info!("PLC {} 读Bool[{}] 请求开始", ep, address);
            }
        }
        let value = svc.read_bool(&handle, address).await?;
        if let Some(mgr) = crate::domain::services::plc_communication_service::get_global_plc_manager() {
            if let Some(ep) = mgr.endpoint_by_id(connection_id).await {
                log::info!("PLC {} 读Bool[{}]={}", ep, address, value);
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
