// 兼容旧版 PLC 通信调用的扩展方法
// 此文件中的 trait 为过渡用途，待所有调用方迁移到统一的 IPlcCommunicationService 接口后即可删除。

use crate::infrastructure::plc_communication::IPlcCommunicationService;
use crate::utils::error::{AppError, AppResult};
use crate::domain::services::ConnectionHandle;
use chrono::Utc;
use crate::domain::services::plc_communication_service::PlcProtocol;

#[async_trait::async_trait]
pub trait PlcServiceLegacyExt {
    async fn write_float32(&self, address: &str, value: f32) -> AppResult<()>;
    async fn read_float32(&self, address: &str) -> AppResult<f32>;
    async fn write_bool(&self, address: &str, value: bool) -> AppResult<()>;
    async fn read_bool(&self, address: &str) -> AppResult<bool>;
}

#[async_trait::async_trait]
// Implementation for Arc<dyn IPlcCommunicationService + Send + Sync>
impl PlcServiceLegacyExt for std::sync::Arc<dyn IPlcCommunicationService + Send + Sync> {
    async fn write_float32(&self, _address: &str, _value: f32) -> AppResult<()> {
        let handle = ConnectionHandle {
            connection_id: "compat".to_string(),
            handle_id: "compat".to_string(),
            protocol: PlcProtocol::ModbusTcp,
            created_at: Utc::now(),
            last_activity: Utc::now(),
        };
        let service: &dyn IPlcCommunicationService = self.as_ref();
        service.write_f32(&handle, _address, _value).await
    }

    async fn read_float32(&self, _address: &str) -> AppResult<f32> {
        let handle = ConnectionHandle {
            connection_id: "compat".to_string(),
            handle_id: "compat".to_string(),
            protocol: PlcProtocol::ModbusTcp,
            created_at: Utc::now(),
            last_activity: Utc::now(),
        };
        let service: &dyn IPlcCommunicationService = self.as_ref();
        service.read_f32(&handle, _address).await
    }

    async fn write_bool(&self, _address: &str, _value: bool) -> AppResult<()> {
        let handle = ConnectionHandle {
            connection_id: "compat".to_string(),
            handle_id: "compat".to_string(),
            protocol: PlcProtocol::ModbusTcp,
            created_at: Utc::now(),
            last_activity: Utc::now(),
        };
        let service: &dyn IPlcCommunicationService = self.as_ref();
        service.write_bool(&handle, _address, _value).await
    }

    async fn read_bool(&self, _address: &str) -> AppResult<bool> {
        let handle = ConnectionHandle {
            connection_id: "compat".to_string(),
            handle_id: "compat".to_string(),
            protocol: PlcProtocol::ModbusTcp,
            created_at: Utc::now(),
            last_activity: Utc::now(),
        };
        let service: &dyn IPlcCommunicationService = self.as_ref();
        service.read_bool(&handle, _address).await
    }
}

// 为不含 Send+Sync 约束的 Arc 对象提供兼容实现
#[async_trait::async_trait]
impl PlcServiceLegacyExt for std::sync::Arc<dyn IPlcCommunicationService> {
    async fn write_float32(&self, address: &str, value: f32) -> AppResult<()> {
        let arc = self.clone() as std::sync::Arc<dyn IPlcCommunicationService + Send + Sync>;
        PlcServiceLegacyExt::write_float32(&arc, address, value).await
    }
    async fn read_float32(&self, address: &str) -> AppResult<f32> {
        let arc = self.clone() as std::sync::Arc<dyn IPlcCommunicationService + Send + Sync>;
        PlcServiceLegacyExt::read_float32(&arc, address).await
    }
    async fn write_bool(&self, address: &str, value: bool) -> AppResult<()> {
        let arc = self.clone() as std::sync::Arc<dyn IPlcCommunicationService + Send + Sync>;
        PlcServiceLegacyExt::write_bool(&arc, address, value).await
    }
    async fn read_bool(&self, address: &str) -> AppResult<bool> {
        let arc = self.clone() as std::sync::Arc<dyn IPlcCommunicationService + Send + Sync>;
        PlcServiceLegacyExt::read_bool(&arc, address).await
    }
}

// ===== 去掉泛型 / &Arc 实现，保留对 Arc<dyn ...> 的专用实现即可避免方法解析冲突 =====
// 调用方若持有 &Arc 可先 clone() 或解引用到 Arc

// 此兼容层不对 Arc<T> 提供额外实现，直接通过上面的泛型实现即可满足需求

// 为 &Arc 提供轻量代理实现，避免调用方必须 clone()
#[async_trait::async_trait]
impl PlcServiceLegacyExt for &std::sync::Arc<dyn IPlcCommunicationService + Send + Sync> {
    async fn write_float32(&self, address: &str, value: f32) -> AppResult<()> {
        self.clone().write_float32(address, value).await
    }
    async fn read_float32(&self, address: &str) -> AppResult<f32> {
        self.clone().read_float32(address).await
    }
    async fn write_bool(&self, address: &str, value: bool) -> AppResult<()> {
        self.clone().write_bool(address, value).await
    }
    async fn read_bool(&self, address: &str) -> AppResult<bool> {
        self.clone().read_bool(address).await
    }
} 
