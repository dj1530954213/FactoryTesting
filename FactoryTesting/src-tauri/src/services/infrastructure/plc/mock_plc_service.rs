/// Mock PLC 服务实现
/// 用于开发和测试阶段，模拟真实的PLC通信行为

use async_trait::async_trait;
use chrono::Utc;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::time::sleep;
use base64;

use crate::utils::error::{AppError, AppResult};
use crate::services::traits::BaseService;
use super::plc_communication_service::{
    PlcCommunicationService, PlcConnectionStatus, PlcCommunicationStats, PlcTag, PlcDataType,
};

/// Mock PLC 服务实现
/// 提供完整的PLC通信接口模拟，支持数据存储和读写操作记录
pub struct MockPlcService {
    /// 服务名称
    service_name: String,
    /// 连接状态
    connection_status: PlcConnectionStatus,
    /// 内部数据存储（地址 -> 值）
    data_storage: Arc<Mutex<HashMap<String, Value>>>,
    /// 写入操作记录（用于测试验证）
    write_log: Arc<Mutex<Vec<WriteOperation>>>,
    /// 通信统计信息
    stats: Arc<Mutex<PlcCommunicationStats>>,
    /// 最后一次发生的错误
    last_error: Arc<Mutex<Option<AppError>>>,
    /// 读取超时时间（毫秒）
    read_timeout_ms: u32,
    /// 写入超时时间（毫秒）
    write_timeout_ms: u32,
    /// 是否模拟网络延迟
    simulate_network_delay: bool,
    /// 网络延迟时间（毫秒）
    network_delay_ms: u64,
    /// 是否模拟错误
    simulate_errors: bool,
    /// 错误率（0.0-1.0）
    error_rate: f64,
}

/// 写入操作记录
/// 用于测试验证写入操作是否按预期执行
#[derive(Debug, Clone)]
pub struct WriteOperation {
    /// 写入时间戳
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// 写入地址
    pub address: String,
    /// 写入的值
    pub value: Value,
    /// 操作类型描述
    pub operation_type: String,
}

impl MockPlcService {
    /// 创建新的 Mock PLC 服务实例
    pub fn new(service_name: impl Into<String>) -> Self {
        Self {
            service_name: service_name.into(),
            connection_status: PlcConnectionStatus::Disconnected,
            data_storage: Arc::new(Mutex::new(HashMap::new())),
            write_log: Arc::new(Mutex::new(Vec::new())),
            stats: Arc::new(Mutex::new(PlcCommunicationStats::default())),
            last_error: Arc::new(Mutex::new(None)),
            read_timeout_ms: 3000,
            write_timeout_ms: 3000,
            simulate_network_delay: true,
            network_delay_ms: 50,
            simulate_errors: false,
            error_rate: 0.01, // 1% 错误率
        }
    }

    /// 创建用于测试的 Mock PLC 服务实例
    /// 禁用网络延迟和错误模拟，以便快速测试
    pub fn new_for_testing(service_name: impl Into<String>) -> Self {
        let mut service = Self::new(service_name);
        service.simulate_network_delay = false;
        service.simulate_errors = false;
        service
    }

    /// 预设读取值
    /// 为指定地址设置预期的读取返回值
    pub fn preset_read_value(&self, address: impl Into<String>, value: Value) {
        let mut storage = self.data_storage.lock().unwrap();
        storage.insert(address.into(), value);
    }

    /// 预设多个读取值
    /// 批量设置多个地址的预期读取返回值
    pub fn preset_read_values(&self, values: HashMap<String, Value>) {
        let mut storage = self.data_storage.lock().unwrap();
        for (address, value) in values {
            storage.insert(address, value);
        }
    }

    /// 获取写入日志
    /// 返回所有记录的写入操作，用于测试验证
    pub fn get_write_log(&self) -> Vec<WriteOperation> {
        self.write_log.lock().unwrap().clone()
    }

    /// 获取最后一次错误
    pub fn get_last_error(&self) -> Option<AppError> {
        self.last_error.lock().unwrap().clone()
    }

    /// 清空写入日志
    /// 清除所有记录的写入操作
    pub fn clear_write_log(&self) {
        self.write_log.lock().unwrap().clear();
    }

    /// 获取最后一次写入操作
    /// 返回最近的一次写入操作，用于测试验证
    pub fn get_last_write(&self) -> Option<WriteOperation> {
        self.write_log.lock().unwrap().last().cloned()
    }

    /// 检查是否写入了指定地址
    /// 验证某个地址是否被写入过
    pub fn was_address_written(&self, address: &str) -> bool {
        self.write_log
            .lock()
            .unwrap()
            .iter()
            .any(|op| op.address == address)
    }

    /// 设置网络延迟模拟
    pub fn set_network_delay(&mut self, enable: bool, delay_ms: u64) {
        self.simulate_network_delay = enable;
        self.network_delay_ms = delay_ms;
    }

    /// 设置错误模拟
    pub fn set_error_simulation(&mut self, enable: bool, error_rate: f64) {
        self.simulate_errors = enable;
        self.error_rate = error_rate.clamp(0.0, 1.0);
    }

    /// 模拟网络延迟
    async fn simulate_delay(&self) {
        if self.simulate_network_delay {
            sleep(Duration::from_millis(self.network_delay_ms)).await;
        }
    }

    /// 检查是否应该模拟错误
    fn should_simulate_error(&self) -> bool {
        self.simulate_errors && fastrand::f64() < self.error_rate
    }

    /// 记录写入操作
    fn log_write_operation(&self, address: String, value: Value, operation_type: String) {
        let operation = WriteOperation {
            timestamp: Utc::now(),
            address,
            value,
            operation_type,
        };
        self.write_log.lock().unwrap().push(operation);
    }

    /// 更新统计信息
    fn update_stats<F>(&self, updater: F)
    where
        F: FnOnce(&mut PlcCommunicationStats),
    {
        let mut stats = self.stats.lock().unwrap();
        updater(&mut stats);
        stats.last_communication_time = Some(Utc::now());
    }

    /// 设置并记录错误
    fn set_and_record_error(&self, error: AppError) -> AppError {
        *self.last_error.lock().unwrap() = Some(error.clone());
        error
    }

    /// 获取存储的值
    fn get_stored_value(&self, address: &str) -> Option<Value> {
        self.data_storage.lock().unwrap().get(address).cloned()
    }

    /// 存储值
    fn store_value(&self, address: String, value: Value) {
        self.data_storage.lock().unwrap().insert(address, value);
    }
}

#[async_trait]
impl BaseService for MockPlcService {
    fn service_name(&self) -> &'static str {
        "MockPlcService"
    }

    async fn initialize(&mut self) -> AppResult<()> {
        // Mock 服务初始化
        self.connection_status = PlcConnectionStatus::Disconnected;
        Ok(())
    }

    async fn shutdown(&mut self) -> AppResult<()> {
        // Mock 服务关闭
        self.connection_status = PlcConnectionStatus::Disconnected;
        Ok(())
    }

    async fn health_check(&self) -> AppResult<()> {
        if self.is_connected() {
            Ok(())
        } else {
            Err(AppError::plc_communication_error("Mock PLC 服务未连接"))
        }
    }
}

#[async_trait]
impl PlcCommunicationService for MockPlcService {
    async fn connect(&mut self) -> AppResult<()> {
        self.simulate_delay().await;
        *self.last_error.lock().unwrap() = None;

        if self.should_simulate_error() {
            self.connection_status = PlcConnectionStatus::Error("模拟连接失败".to_string());
            let err = AppError::plc_communication_error("模拟的PLC连接失败");
            return Err(self.set_and_record_error(err));
        }

        self.connection_status = PlcConnectionStatus::Connected;
        self.update_stats(|stats| stats.connection_count += 1);
        
        log::info!("[{}] Mock PLC 连接成功", self.service_name);
        Ok(())
    }

    async fn disconnect(&mut self) -> AppResult<()> {
        self.simulate_delay().await;
        self.connection_status = PlcConnectionStatus::Disconnected;
        log::info!("[{}] Mock PLC 断开连接", self.service_name);
        Ok(())
    }

    fn get_connection_status(&self) -> PlcConnectionStatus {
        self.connection_status.clone()
    }

    async fn test_connection(&self) -> AppResult<bool> {
        self.simulate_delay().await;
        Ok(self.is_connected())
    }

    async fn read_bool(&self, address: &str) -> AppResult<bool> {
        self.simulate_delay().await;
        if !self.is_connected() {
            self.update_stats(|stats| stats.failed_reads += 1);
            return Err(self.set_and_record_error(AppError::plc_communication_error(
                "PLC未连接",
            )));
        }
        if self.should_simulate_error() {
            self.update_stats(|stats| stats.failed_reads += 1);
            return Err(self.set_and_record_error(AppError::plc_communication_error(
                format!("模拟读取错误 at {}", address),
            )));
        }

        match self.get_stored_value(address) {
            Some(Value::Bool(b_val)) => {
                self.update_stats(|stats| stats.successful_reads += 1);
                Ok(b_val)
            }
            Some(other_val) => {
                self.update_stats(|stats| stats.failed_reads += 1);
                Err(self.set_and_record_error(AppError::plc_communication_error(
                    format!("PLC type error at address {}: expected bool, got {:?}", address, other_val)
                )))
            }
            None => {
                self.update_stats(|stats| stats.failed_reads += 1);
                Err(self.set_and_record_error(AppError::plc_communication_error(
                    format!("PLC address not found: {}", address)
                )))
            }
        }
    }

    async fn read_int8(&self, address: &str) -> AppResult<i8> {
        self.simulate_delay().await;
        if !self.is_connected() {
            self.update_stats(|stats| stats.failed_reads += 1);
            return Err(self.set_and_record_error(AppError::plc_communication_error("PLC未连接")));
        }
        if self.should_simulate_error() {
            self.update_stats(|stats| stats.failed_reads += 1);
            return Err(self.set_and_record_error(AppError::plc_communication_error(format!("模拟读取错误 at {}", address))));
        }

        match self.get_stored_value(address) {
            Some(Value::Number(num)) => {
                if let Some(i_val) = num.as_i64() {
                    if i_val >= i8::MIN as i64 && i_val <= i8::MAX as i64 {
                        self.update_stats(|stats| stats.successful_reads += 1);
                        Ok(i_val as i8)
                    } else {
                        self.update_stats(|stats| stats.failed_reads += 1);
                        Err(self.set_and_record_error(AppError::plc_communication_error(
                            format!("PLC type error at address {}: value {} out of range for i8", address, i_val)
                        )))
                    }
                } else {
                    self.update_stats(|stats| stats.failed_reads += 1);
                    Err(self.set_and_record_error(AppError::plc_communication_error(
                        format!("PLC type error at address {}: expected i8, got non-integer Number", address)
                    )))
                }
            }
            Some(other_val) => {
                self.update_stats(|stats| stats.failed_reads += 1);
                Err(self.set_and_record_error(AppError::plc_communication_error(
                    format!("PLC type error at address {}: expected i8, got {:?}", address, other_val)
                )))
            }
            None => {
                self.update_stats(|stats| stats.failed_reads += 1);
                Err(self.set_and_record_error(AppError::plc_communication_error(
                    format!("PLC address not found: {}", address)
                )))
            }
        }
    }

    async fn read_uint8(&self, address: &str) -> AppResult<u8> {
        self.simulate_delay().await;
        if !self.is_connected() {
            self.update_stats(|stats| stats.failed_reads += 1);
            return Err(self.set_and_record_error(AppError::plc_communication_error("PLC未连接")));
        }
        if self.should_simulate_error() {
            self.update_stats(|stats| stats.failed_reads += 1);
            return Err(self.set_and_record_error(AppError::plc_communication_error(format!("模拟读取错误 at {}", address))));
        }
        match self.get_stored_value(address) {
            Some(Value::Number(num)) => {
                if let Some(u_val) = num.as_u64() {
                    if u_val <= u8::MAX as u64 {
                        self.update_stats(|stats| stats.successful_reads += 1);
                        Ok(u_val as u8)
                    } else {
                        self.update_stats(|stats| stats.failed_reads += 1);
                        Err(self.set_and_record_error(AppError::plc_communication_error(format!("PLC type error at address {}: value {} out of range for u8", address, u_val))))
                    }
                } else {
                    self.update_stats(|stats| stats.failed_reads += 1);
                    Err(self.set_and_record_error(AppError::plc_communication_error(format!("PLC type error at address {}: expected u8, got non-integer Number", address))))
                }
            }
            Some(other_val) => {
                self.update_stats(|stats| stats.failed_reads += 1);
                Err(self.set_and_record_error(AppError::plc_communication_error(format!("PLC type error at address {}: expected u8, got {:?}", address, other_val))))
            }
            None => {
                self.update_stats(|stats| stats.failed_reads += 1);
                Err(self.set_and_record_error(AppError::plc_communication_error(format!("PLC address not found: {}", address))))
            }
        }
    }

    async fn read_int16(&self, address: &str) -> AppResult<i16> {
        self.simulate_delay().await;
        if !self.is_connected() {
            self.update_stats(|stats| stats.failed_reads += 1);
            return Err(self.set_and_record_error(AppError::plc_communication_error("PLC未连接")));
        }
        if self.should_simulate_error() {
            self.update_stats(|stats| stats.failed_reads += 1);
            return Err(self.set_and_record_error(AppError::plc_communication_error(format!("模拟读取错误 at {}", address))));
        }
        match self.get_stored_value(address) {
            Some(Value::Number(num)) => {
                if let Some(i_val) = num.as_i64() {
                    if i_val >= i16::MIN as i64 && i_val <= i16::MAX as i64 {
                        self.update_stats(|stats| stats.successful_reads += 1);
                        Ok(i_val as i16)
                    } else {
                        self.update_stats(|stats| stats.failed_reads += 1);
                        Err(self.set_and_record_error(AppError::plc_communication_error(format!("PLC type error at address {}: value {} out of range for i16", address, i_val))))
                    }
                } else {
                    self.update_stats(|stats| stats.failed_reads += 1);
                    Err(self.set_and_record_error(AppError::plc_communication_error(format!("PLC type error at address {}: expected i16, got non-integer Number", address))))
                }
            }
            Some(other_val) => {
                self.update_stats(|stats| stats.failed_reads += 1);
                Err(self.set_and_record_error(AppError::plc_communication_error(format!("PLC type error at address {}: expected i16, got {:?}", address, other_val))))
            }
            None => {
                self.update_stats(|stats| stats.failed_reads += 1);
                Err(self.set_and_record_error(AppError::plc_communication_error(format!("PLC address not found: {}", address))))
            }
        }
    }

    async fn read_uint16(&self, address: &str) -> AppResult<u16> {
        self.simulate_delay().await;
        if !self.is_connected() {
            self.update_stats(|stats| stats.failed_reads += 1);
            return Err(self.set_and_record_error(AppError::plc_communication_error("PLC未连接")));
        }
        if self.should_simulate_error() {
            self.update_stats(|stats| stats.failed_reads += 1);
            return Err(self.set_and_record_error(AppError::plc_communication_error(format!("模拟读取错误 at {}", address))));
        }
        match self.get_stored_value(address) {
            Some(Value::Number(num)) => {
                if let Some(u_val) = num.as_u64() {
                    if u_val <= u16::MAX as u64 {
                        self.update_stats(|stats| stats.successful_reads += 1);
                        Ok(u_val as u16)
                    } else {
                        self.update_stats(|stats| stats.failed_reads += 1);
                        Err(self.set_and_record_error(AppError::plc_communication_error(format!("PLC type error at address {}: value {} out of range for u16", address, u_val))))
                    }
                } else {
                    self.update_stats(|stats| stats.failed_reads += 1);
                    Err(self.set_and_record_error(AppError::plc_communication_error(format!("PLC type error at address {}: expected u16, got non-integer Number", address))))
                }
            }
            Some(other_val) => {
                self.update_stats(|stats| stats.failed_reads += 1);
                Err(self.set_and_record_error(AppError::plc_communication_error(format!("PLC type error at address {}: expected u16, got {:?}", address, other_val))))
            }
            None => {
                self.update_stats(|stats| stats.failed_reads += 1);
                Err(self.set_and_record_error(AppError::plc_communication_error(format!("PLC address not found: {}", address))))
            }
        }
    }

    async fn read_int32(&self, address: &str) -> AppResult<i32> {
        self.simulate_delay().await;
        if !self.is_connected() {
            self.update_stats(|stats| stats.failed_reads += 1);
            return Err(self.set_and_record_error(AppError::plc_communication_error("PLC未连接")));
        }
        if self.should_simulate_error() {
            self.update_stats(|stats| stats.failed_reads += 1);
            return Err(self.set_and_record_error(AppError::plc_communication_error(format!("模拟读取错误 at {}", address))));
        }
        match self.get_stored_value(address) {
            Some(Value::Number(num)) => {
                if let Some(i_val) = num.as_i64() {
                    if i_val >= i32::MIN as i64 && i_val <= i32::MAX as i64 {
                        self.update_stats(|stats| stats.successful_reads += 1);
                        Ok(i_val as i32)
                    } else {
                        self.update_stats(|stats| stats.failed_reads += 1);
                        Err(self.set_and_record_error(AppError::plc_communication_error(format!("PLC type error at address {}: value {} out of range for i32", address, i_val))))
                    }
                } else {
                    self.update_stats(|stats| stats.failed_reads += 1);
                    Err(self.set_and_record_error(AppError::plc_communication_error(format!("PLC type error at address {}: expected i32, got non-integer Number", address))))
                }
            }
            Some(other_val) => {
                self.update_stats(|stats| stats.failed_reads += 1);
                Err(self.set_and_record_error(AppError::plc_communication_error(format!("PLC type error at address {}: expected i32, got {:?}", address, other_val))))
            }
            None => {
                self.update_stats(|stats| stats.failed_reads += 1);
                Err(self.set_and_record_error(AppError::plc_communication_error(format!("PLC address not found: {}", address))))
            }
        }
    }

    async fn read_uint32(&self, address: &str) -> AppResult<u32> {
        self.simulate_delay().await;
        if !self.is_connected() {
            self.update_stats(|stats| stats.failed_reads += 1);
            return Err(self.set_and_record_error(AppError::plc_communication_error("PLC未连接")));
        }
        if self.should_simulate_error() {
            self.update_stats(|stats| stats.failed_reads += 1);
            return Err(self.set_and_record_error(AppError::plc_communication_error(format!("模拟读取错误 at {}", address))));
        }
        match self.get_stored_value(address) {
            Some(Value::Number(num)) => {
                if let Some(u_val) = num.as_u64() {
                    if u_val <= u32::MAX as u64 {
                        self.update_stats(|stats| stats.successful_reads += 1);
                        Ok(u_val as u32)
                    } else {
                        self.update_stats(|stats| stats.failed_reads += 1);
                        Err(self.set_and_record_error(AppError::plc_communication_error(format!("PLC type error at address {}: value {} out of range for u32", address, u_val))))
                    }
                } else {
                    self.update_stats(|stats| stats.failed_reads += 1);
                    Err(self.set_and_record_error(AppError::plc_communication_error(format!("PLC type error at address {}: expected u32, got non-integer Number", address))))
                }
            }
            Some(other_val) => {
                self.update_stats(|stats| stats.failed_reads += 1);
                Err(self.set_and_record_error(AppError::plc_communication_error(format!("PLC type error at address {}: expected u32, got {:?}", address, other_val))))
            }
            None => {
                self.update_stats(|stats| stats.failed_reads += 1);
                Err(self.set_and_record_error(AppError::plc_communication_error(format!("PLC address not found: {}", address))))
            }
        }
    }

    async fn read_int64(&self, address: &str) -> AppResult<i64> {
        self.simulate_delay().await;
        if !self.is_connected() {
            self.update_stats(|stats| stats.failed_reads += 1);
            return Err(self.set_and_record_error(AppError::plc_communication_error("PLC未连接")));
        }
        if self.should_simulate_error() {
            self.update_stats(|stats| stats.failed_reads += 1);
            return Err(self.set_and_record_error(AppError::plc_communication_error(format!("模拟读取错误 at {}", address))));
        }
        match self.get_stored_value(address) {
            Some(Value::Number(num)) => {
                if let Some(i_val) = num.as_i64() {
                    self.update_stats(|stats| stats.successful_reads += 1);
                    Ok(i_val)
                } else {
                    self.update_stats(|stats| stats.failed_reads += 1);
                    Err(self.set_and_record_error(AppError::plc_communication_error(format!("PLC type error at address {}: expected i64, got non-integer Number", address))))
                }
            }
            Some(other_val) => {
                self.update_stats(|stats| stats.failed_reads += 1);
                Err(self.set_and_record_error(AppError::plc_communication_error(format!("PLC type error at address {}: expected i64, got {:?}", address, other_val))))
            }
            None => {
                self.update_stats(|stats| stats.failed_reads += 1);
                Err(self.set_and_record_error(AppError::plc_communication_error(format!("PLC address not found: {}", address))))
            }
        }
    }

    async fn read_uint64(&self, address: &str) -> AppResult<u64> {
        self.simulate_delay().await;
        if !self.is_connected() {
            self.update_stats(|stats| stats.failed_reads += 1);
            return Err(self.set_and_record_error(AppError::plc_communication_error("PLC未连接")));
        }
        if self.should_simulate_error() {
            self.update_stats(|stats| stats.failed_reads += 1);
            return Err(self.set_and_record_error(AppError::plc_communication_error(format!("模拟读取错误 at {}", address))));
        }
        match self.get_stored_value(address) {
            Some(Value::Number(num)) => {
                if let Some(u_val) = num.as_u64() {
                    self.update_stats(|stats| stats.successful_reads += 1);
                    Ok(u_val)
                } else {
                    self.update_stats(|stats| stats.failed_reads += 1);
                    Err(self.set_and_record_error(AppError::plc_communication_error(format!("PLC type error at address {}: expected u64, got non-integer Number", address))))
                }
            }
            Some(other_val) => {
                self.update_stats(|stats| stats.failed_reads += 1);
                Err(self.set_and_record_error(AppError::plc_communication_error(format!("PLC type error at address {}: expected u64, got {:?}", address, other_val))))
            }
            None => {
                self.update_stats(|stats| stats.failed_reads += 1);
                Err(self.set_and_record_error(AppError::plc_communication_error(format!("PLC address not found: {}", address))))
            }
        }
    }

    async fn read_float32(&self, address: &str) -> AppResult<f32> {
        self.simulate_delay().await;
        if !self.is_connected() {
            self.update_stats(|stats| stats.failed_reads += 1);
            return Err(self.set_and_record_error(AppError::plc_communication_error(
                "PLC未连接",
            )));
        }
        if self.should_simulate_error() {
            self.update_stats(|stats| stats.failed_reads += 1);
            return Err(self.set_and_record_error(AppError::plc_communication_error(
                format!("模拟读取错误 at {}", address),
            )));
        }

        match self.get_stored_value(address) {
            Some(Value::Number(num)) => {
                if let Some(f_val) = num.as_f64() {
                    self.update_stats(|stats| stats.successful_reads += 1);
                    Ok(f_val as f32)
                } else {
                    self.update_stats(|stats| stats.failed_reads += 1);
                    Err(self.set_and_record_error(AppError::plc_communication_error(
                        format!("PLC type error at address {}: expected f32, got non-float Number", address)
                    )))
                }
            }
            Some(other_val) => {
                self.update_stats(|stats| stats.failed_reads += 1);
                Err(self.set_and_record_error(AppError::plc_communication_error(
                    format!("PLC type error at address {}: expected f32, got {:?}", address, other_val)
                )))
            }
            None => {
                self.update_stats(|stats| stats.failed_reads += 1);
                Err(self.set_and_record_error(AppError::plc_communication_error(
                    format!("PLC address not found: {}", address)
                )))
            }
        }
    }

    async fn read_float64(&self, address: &str) -> AppResult<f64> {
        self.simulate_delay().await;
        if !self.is_connected() {
            self.update_stats(|stats| stats.failed_reads += 1);
            return Err(self.set_and_record_error(AppError::plc_communication_error(
                "PLC未连接",
            )));
        }
        if self.should_simulate_error() {
            self.update_stats(|stats| stats.failed_reads += 1);
            return Err(self.set_and_record_error(AppError::plc_communication_error(
                format!("模拟读取错误 at {}", address),
            )));
        }

        match self.get_stored_value(address) {
            Some(Value::Number(num)) => {
                if let Some(f_val) = num.as_f64() {
                    self.update_stats(|stats| stats.successful_reads += 1);
                    Ok(f_val)
                } else {
                    self.update_stats(|stats| stats.failed_reads += 1);
                    Err(self.set_and_record_error(AppError::plc_communication_error(
                        format!("PLC type error at address {}: expected f64, got non-float Number", address)
                    )))
                }
            }
            Some(other_val) => {
                self.update_stats(|stats| stats.failed_reads += 1);
                Err(self.set_and_record_error(AppError::plc_communication_error(
                    format!("PLC type error at address {}: expected f64, got {:?}", address, other_val)
                )))
            }
            None => {
                self.update_stats(|stats| stats.failed_reads += 1);
                Err(self.set_and_record_error(AppError::plc_communication_error(
                    format!("PLC address not found: {}", address)
                )))
            }
        }
    }

    async fn read_string(&self, address: &str, _max_length: usize) -> AppResult<String> {
        self.simulate_delay().await;
        if !self.is_connected() {
            self.update_stats(|stats| stats.failed_reads += 1);
            return Err(self.set_and_record_error(AppError::plc_communication_error(
                "PLC未连接",
            )));
        }
        if self.should_simulate_error() {
            self.update_stats(|stats| stats.failed_reads += 1);
            return Err(self.set_and_record_error(AppError::plc_communication_error(
                format!("模拟读取错误 at {}", address),
            )));
        }

        match self.get_stored_value(address) {
            Some(Value::String(s_val)) => {
                self.update_stats(|stats| stats.successful_reads += 1);
                Ok(s_val)
            }
            Some(other_val) => {
                self.update_stats(|stats| stats.failed_reads += 1);
                Err(self.set_and_record_error(AppError::plc_communication_error(
                    format!("PLC type error at address {}: expected String, got {:?}", address, other_val)
                )))
            }
            None => {
                self.update_stats(|stats| stats.failed_reads += 1);
                Err(self.set_and_record_error(AppError::plc_communication_error(
                    format!("PLC address not found: {}", address)
                )))
            }
        }
    }

    async fn read_bytes(&self, address: &str, _length: usize) -> AppResult<Vec<u8>> {
        self.simulate_delay().await;
        if !self.is_connected() {
            self.update_stats(|stats| stats.failed_reads += 1);
            return Err(self.set_and_record_error(AppError::plc_communication_error("PLC未连接")));
        }
        if self.should_simulate_error() {
            self.update_stats(|stats| stats.failed_reads += 1);
            return Err(self.set_and_record_error(AppError::plc_communication_error(format!("模拟读取错误 at {}", address))));
        }

        match self.get_stored_value(address) {
            Some(Value::Array(arr)) => {
                let mut bytes = Vec::new();
                for val_num in arr {
                    if let Some(num) = val_num.as_u64() {
                        if num <= u8::MAX as u64 {
                            bytes.push(num as u8);
                        } else {
                            self.update_stats(|stats| stats.failed_reads += 1);
                            return Err(self.set_and_record_error(AppError::plc_communication_error(format!("PLC type error at address {}: value in byte array out of range for u8", address))));
                        }
                    } else {
                        self.update_stats(|stats| stats.failed_reads += 1);
                        return Err(self.set_and_record_error(AppError::plc_communication_error(format!("PLC type error at address {}: expected Vec<u8>, got non-Number in array", address))));
                    }
                }
                self.update_stats(|stats| stats.successful_reads += 1);
                Ok(bytes)
            }
            Some(Value::String(s_val)) => { 
                match base64::decode(&s_val) {
                    Ok(bytes) => {
                        self.update_stats(|stats| stats.successful_reads += 1);
                        Ok(bytes)
                    }
                    Err(_) => {
                        self.update_stats(|stats| stats.failed_reads += 1);
                        Err(self.set_and_record_error(AppError::plc_communication_error(format!("PLC type error at address {}: expected Vec<u8>, failed to decode base64 string", address))))
                    }
                }
            }
            Some(other_val) => {
                self.update_stats(|stats| stats.failed_reads += 1);
                Err(self.set_and_record_error(AppError::plc_communication_error(format!("PLC type error at address {}: expected Vec<u8>, got {:?}", address, other_val))))
            }
            None => {
                self.update_stats(|stats| stats.failed_reads += 1);
                Err(self.set_and_record_error(AppError::plc_communication_error(format!("PLC address not found: {}", address))))
            }
        }
    }

    // --- Write Operations ---

    async fn write_bool(&self, address: &str, value: bool) -> AppResult<()> {
        self.simulate_delay().await;
        if !self.is_connected() {
            self.update_stats(|stats| stats.failed_writes += 1);
            return Err(self.set_and_record_error(AppError::plc_communication_error(
                "PLC未连接",
            )));
        }
        if self.should_simulate_error() {
            self.update_stats(|stats| stats.failed_writes += 1);
            return Err(self.set_and_record_error(AppError::plc_communication_error(
                format!("模拟写入错误 at {}", address),
            )));
        }

        let value_to_store = Value::Bool(value);
        self.store_value(address.to_string(), value_to_store.clone());
        self.log_write_operation(address.to_string(), value_to_store, "write_bool".to_string());
        self.update_stats(|stats| stats.successful_writes += 1);
        Ok(())
    }

    async fn write_int8(&self, address: &str, value: i8) -> AppResult<()> {
        self.simulate_delay().await;
        if !self.is_connected() {
            self.update_stats(|stats| stats.failed_writes += 1);
            return Err(self.set_and_record_error(AppError::plc_communication_error("PLC未连接")));
        }
        if self.should_simulate_error() {
            self.update_stats(|stats| stats.failed_writes += 1);
            return Err(self.set_and_record_error(AppError::plc_communication_error(format!("模拟写入错误 at {}", address))));
        }
        let value_to_store = Value::Number(serde_json::Number::from(value));
        self.store_value(address.to_string(), value_to_store.clone());
        self.log_write_operation(address.to_string(), value_to_store, "write_int8".to_string());
        self.update_stats(|stats| stats.successful_writes += 1);
        Ok(())
    }

    async fn write_uint8(&self, address: &str, value: u8) -> AppResult<()> {
        self.simulate_delay().await;
        if !self.is_connected() {
            self.update_stats(|stats| stats.failed_writes += 1);
            return Err(self.set_and_record_error(AppError::plc_communication_error("PLC未连接")));
        }
        if self.should_simulate_error() {
            self.update_stats(|stats| stats.failed_writes += 1);
            return Err(self.set_and_record_error(AppError::plc_communication_error(format!("模拟写入错误 at {}", address))));
        }
        let value_to_store = Value::Number(serde_json::Number::from(value));
        self.store_value(address.to_string(), value_to_store.clone());
        self.log_write_operation(address.to_string(), value_to_store, "write_uint8".to_string());
        self.update_stats(|stats| stats.successful_writes += 1);
        Ok(())
    }

    async fn write_int16(&self, address: &str, value: i16) -> AppResult<()> {
        self.simulate_delay().await;
        if !self.is_connected() {
            self.update_stats(|stats| stats.failed_writes += 1);
            return Err(self.set_and_record_error(AppError::plc_communication_error("PLC未连接")));
        }
        if self.should_simulate_error() {
            self.update_stats(|stats| stats.failed_writes += 1);
            return Err(self.set_and_record_error(AppError::plc_communication_error(format!("模拟写入错误 at {}", address))));
        }
        let value_to_store = Value::Number(serde_json::Number::from(value));
        self.store_value(address.to_string(), value_to_store.clone());
        self.log_write_operation(address.to_string(), value_to_store, "write_int16".to_string());
        self.update_stats(|stats| stats.successful_writes += 1);
        Ok(())
    }

    async fn write_uint16(&self, address: &str, value: u16) -> AppResult<()> {
        self.simulate_delay().await;
        if !self.is_connected() {
            self.update_stats(|stats| stats.failed_writes += 1);
            return Err(self.set_and_record_error(AppError::plc_communication_error("PLC未连接")));
        }
        if self.should_simulate_error() {
            self.update_stats(|stats| stats.failed_writes += 1);
            return Err(self.set_and_record_error(AppError::plc_communication_error(format!("模拟写入错误 at {}", address))));
        }
        let value_to_store = Value::Number(serde_json::Number::from(value));
        self.store_value(address.to_string(), value_to_store.clone());
        self.log_write_operation(address.to_string(), value_to_store, "write_uint16".to_string());
        self.update_stats(|stats| stats.successful_writes += 1);
        Ok(())
    }

    async fn write_int32(&self, address: &str, value: i32) -> AppResult<()> {
        self.simulate_delay().await;
        if !self.is_connected() {
            self.update_stats(|stats| stats.failed_writes += 1);
            return Err(self.set_and_record_error(AppError::plc_communication_error("PLC未连接")));
        }
        if self.should_simulate_error() {
            self.update_stats(|stats| stats.failed_writes += 1);
            return Err(self.set_and_record_error(AppError::plc_communication_error(format!("模拟写入错误 at {}", address))));
        }
        let value_to_store = Value::Number(serde_json::Number::from(value));
        self.store_value(address.to_string(), value_to_store.clone());
        self.log_write_operation(address.to_string(), value_to_store, "write_int32".to_string());
        self.update_stats(|stats| stats.successful_writes += 1);
        Ok(())
    }

    async fn write_uint32(&self, address: &str, value: u32) -> AppResult<()> {
        self.simulate_delay().await;
        if !self.is_connected() {
            self.update_stats(|stats| stats.failed_writes += 1);
            return Err(self.set_and_record_error(AppError::plc_communication_error("PLC未连接")));
        }
        if self.should_simulate_error() {
            self.update_stats(|stats| stats.failed_writes += 1);
            return Err(self.set_and_record_error(AppError::plc_communication_error(format!("模拟写入错误 at {}", address))));
        }
        let value_to_store = Value::Number(serde_json::Number::from(value));
        self.store_value(address.to_string(), value_to_store.clone());
        self.log_write_operation(address.to_string(), value_to_store, "write_uint32".to_string());
        self.update_stats(|stats| stats.successful_writes += 1);
        Ok(())
    }

    async fn write_int64(&self, address: &str, value: i64) -> AppResult<()> {
        self.simulate_delay().await;
        if !self.is_connected() {
            self.update_stats(|stats| stats.failed_writes += 1);
            return Err(self.set_and_record_error(AppError::plc_communication_error("PLC未连接")));
        }
        if self.should_simulate_error() {
            self.update_stats(|stats| stats.failed_writes += 1);
            return Err(self.set_and_record_error(AppError::plc_communication_error(format!("模拟写入错误 at {}", address))));
        }
        let value_to_store = Value::Number(serde_json::Number::from(value));
        self.store_value(address.to_string(), value_to_store.clone());
        self.log_write_operation(address.to_string(), value_to_store, "write_int64".to_string());
        self.update_stats(|stats| stats.successful_writes += 1);
        Ok(())
    }

    async fn write_uint64(&self, address: &str, value: u64) -> AppResult<()> {
        self.simulate_delay().await;
        if !self.is_connected() {
            self.update_stats(|stats| stats.failed_writes += 1);
            return Err(self.set_and_record_error(AppError::plc_communication_error("PLC未连接")));
        }
        if self.should_simulate_error() {
            self.update_stats(|stats| stats.failed_writes += 1);
            return Err(self.set_and_record_error(AppError::plc_communication_error(format!("模拟写入错误 at {}", address))));
        }
        let value_to_store = Value::Number(serde_json::Number::from(value));
        self.store_value(address.to_string(), value_to_store.clone());
        self.log_write_operation(address.to_string(), value_to_store, "write_uint64".to_string());
        self.update_stats(|stats| stats.successful_writes += 1);
        Ok(())
    }

    async fn write_float32(&self, address: &str, value: f32) -> AppResult<()> {
        self.simulate_delay().await;
        if !self.is_connected() {
            self.update_stats(|stats| stats.failed_writes += 1);
            return Err(self.set_and_record_error(AppError::plc_communication_error(
                "PLC未连接",
            )));
        }
        if self.should_simulate_error() {
            self.update_stats(|stats| stats.failed_writes += 1);
            return Err(self.set_and_record_error(AppError::plc_communication_error(
                format!("模拟写入错误 at {}", address),
            )));
        }

        // 将 f32 转换为 f64 存储，因为 serde_json::Number 主要支持 f64
        match serde_json::Number::from_f64(value as f64) {
            Some(num_val) => {
                let value_to_store = Value::Number(num_val);
                self.store_value(address.to_string(), value_to_store.clone());
                self.log_write_operation(address.to_string(), value_to_store, "write_float32".to_string());
                self.update_stats(|stats| stats.successful_writes += 1);
                Ok(())
            }
            None => {
                self.update_stats(|stats| stats.failed_writes += 1);
                Err(self.set_and_record_error(AppError::serialization_error(format!(
                    "无法将f32 {} 转换为JSON Number",
                    value
                ))))
            }
        }
    }

    async fn write_float64(&self, address: &str, value: f64) -> AppResult<()> {
        self.simulate_delay().await;
        if !self.is_connected() {
            self.update_stats(|stats| stats.failed_writes += 1);
            return Err(self.set_and_record_error(AppError::plc_communication_error(
                "PLC未连接",
            )));
        }
        if self.should_simulate_error() {
            self.update_stats(|stats| stats.failed_writes += 1);
            return Err(self.set_and_record_error(AppError::plc_communication_error(
                format!("模拟写入错误 at {}", address),
            )));
        }
        match serde_json::Number::from_f64(value) {
            Some(num_val) => {
                let value_to_store = Value::Number(num_val);
                self.store_value(address.to_string(), value_to_store.clone());
                self.log_write_operation(address.to_string(), value_to_store, "write_float64".to_string());
                self.update_stats(|stats| stats.successful_writes += 1);
                Ok(())
            }
            None => {
                self.update_stats(|stats| stats.failed_writes += 1);
                Err(self.set_and_record_error(AppError::serialization_error(format!(
                    "无法将f64 {} 转换为JSON Number",
                    value
                ))))
            }
        }
    }

    async fn write_string(&self, address: &str, value: &str) -> AppResult<()> {
        self.simulate_delay().await;
        if !self.is_connected() {
            self.update_stats(|stats| stats.failed_writes += 1);
            return Err(self.set_and_record_error(AppError::plc_communication_error("PLC未连接")));
        }
        if self.should_simulate_error() {
            self.update_stats(|stats| stats.failed_writes += 1);
            return Err(self.set_and_record_error(AppError::plc_communication_error(format!("模拟写入错误 at {}", address))));
        }
        let value_to_store = Value::String(value.to_string());
        self.store_value(address.to_string(), value_to_store.clone());
        self.log_write_operation(address.to_string(), value_to_store, "write_string".to_string());
        self.update_stats(|stats| stats.successful_writes += 1);
        Ok(())
    }

    async fn write_bytes(&self, address: &str, value: &[u8]) -> AppResult<()> {
        self.simulate_delay().await;
        if !self.is_connected() {
            self.update_stats(|stats| stats.failed_writes += 1);
            return Err(self.set_and_record_error(AppError::plc_communication_error("PLC未连接")));
        }
        if self.should_simulate_error() {
            self.update_stats(|stats| stats.failed_writes += 1);
            return Err(self.set_and_record_error(AppError::plc_communication_error(format!("模拟写入错误 at {}", address))));
        }
        
        // 存储为 base64 字符串，或者数字数组
        // 为简单起见，这里存储为base64字符串
        let base64_encoded = base64::encode(value);
        let value_to_store = Value::String(base64_encoded);

        self.store_value(address.to_string(), value_to_store.clone());
        self.log_write_operation(address.to_string(), value_to_store, "write_bytes".to_string());
        self.update_stats(|stats| stats.successful_writes += 1);
        Ok(())
    }

    // --- Batch Operations ---

    async fn batch_read(&self, addresses: &[String]) -> AppResult<HashMap<String, Value>> {
        self.simulate_delay().await;
        if !self.is_connected() {
            // 对于批量操作，如果连接失败，是否算作多次失败读取？
            // 这里将其视为一次批量读取操作的失败。具体的子读取不单独计数。
            self.update_stats(|stats| stats.failed_batch_reads += 1); 
            return Err(self.set_and_record_error(AppError::plc_communication_error("PLC未连接")));
        }
        if self.should_simulate_error() {
            self.update_stats(|stats| stats.failed_batch_reads += 1);
            return Err(self.set_and_record_error(AppError::plc_communication_error("模拟批量读取错误")));
        }

        let mut results = HashMap::new();
        let mut batch_had_errors = false;
        for address in addresses {
            // 这里不直接调用单个read方法，避免重复的连接检查和延迟
            match self.get_stored_value(address) {
                Some(value) => {
                    results.insert(address.clone(), value);
                    // 成功读取单个值，不在这里更新 successful_reads，在批量成功后统一更新
                }
                None => {
                    // 单个地址未找到，整个批量读取可以认为是部分成功或整体失败
                    // 这里选择将缺失的值也放入结果中，用 Value::Null 表示
                    results.insert(address.clone(), Value::Null);
                    // 或者，可以将此视为错误，并让整个批量读取失败：
                    // batch_had_errors = true;
                    // self.update_stats(|stats| stats.failed_reads += 1); // 计入单个失败
                    // break; 
                }
            }
        }

        if batch_had_errors { // 如果选择上面注释掉的逻辑
             self.update_stats(|stats| stats.failed_batch_reads += 1);
             Err(self.set_and_record_error(AppError::plc_communication_error(format!("Partial failure in batch read: Some addresses not found"))))
        } else {
            self.update_stats(|stats| stats.successful_batch_reads += 1);
            // 也可以在这里根据 results.len() 更新 successful_reads
            // self.update_stats(|stats| stats.successful_reads += addresses.len() as u64);
            Ok(results)
        }
    }

    async fn batch_write(&self, values: &HashMap<String, Value>) -> AppResult<()> {
        self.simulate_delay().await;
        if !self.is_connected() {
            self.update_stats(|stats| stats.failed_batch_writes += 1);
            return Err(self.set_and_record_error(AppError::plc_communication_error("PLC未连接")));
        }
        if self.should_simulate_error() {
            self.update_stats(|stats| stats.failed_batch_writes += 1);
            return Err(self.set_and_record_error(AppError::plc_communication_error("模拟批量写入错误")));
        }

        // 对于 batch_write，如果其中一个失败，是否应该回滚或部分应用？
        // Mock 实现：简单地逐个写入。
        for (address, value) in values {
            // 检查值是否可转换为 serde_json::Number (用于浮点数等)
            // 这是一个简化，真实PLC可能对类型有更严格的要求
            let current_value_to_store = match value {
                Value::Number(n) => Value::Number(n.clone()),
                Value::Bool(b) => Value::Bool(*b),
                Value::String(s) => Value::String(s.clone()),
                _ => {
                    // 对于不支持的类型，可以选择跳过，或返回错误
                    // 这里选择跳过并记录一个警告或内部错误
                    log::warn!("批量写入中遇到不支持的值类型 {:?} for address {}", value, address);
                    // 或者返回错误:
                    // self.update_stats(|stats| stats.failed_writes += 1); // 计入单个失败
                    // return Err(self.set_and_record_error(AppError::plc_type_error(address, "SupportedType", &format!("{:?}", value))));
                    continue; // 跳过此值
                }
            };

            self.store_value(address.clone(), current_value_to_store.clone());
            self.log_write_operation(address.clone(), current_value_to_store, "batch_write_item".to_string());
            // 成功写入单个值，不在这里更新 successful_writes，在批量成功后统一更新
        }
        
        self.update_stats(|stats| {
            stats.successful_batch_writes += 1;
            // 也可以在这里根据 values.len() 更新 successful_writes
            // stats.successful_writes += values.len() as u64;
        });
        Ok(())
    }

    async fn read_tag_info(&self, address: &str) -> AppResult<PlcTag> {
        self.simulate_delay().await;

        if !self.is_connected() {
            return Err(AppError::plc_communication_error("PLC未连接"));
        }

        // 返回模拟的标签信息
        Ok(PlcTag {
            address: address.to_string(),
            data_type: PlcDataType::Float32,
            description: Some(format!("模拟标签: {}", address)),
            readable: true,
            writable: true,
            unit: Some("mA".to_string()),
            min_value: Some(4.0),
            max_value: Some(20.0),
        })
    }

    async fn list_available_tags(&self) -> AppResult<Vec<PlcTag>> {
        self.simulate_delay().await;

        if !self.is_connected() {
            return Err(AppError::plc_communication_error("PLC未连接"));
        }

        // 返回存储中所有可用的标签
        let storage = self.data_storage.lock().unwrap();
        let tags = storage
            .keys()
            .map(|address| PlcTag {
                address: address.clone(),
                data_type: PlcDataType::Float32,
                description: Some(format!("模拟标签: {}", address)),
                readable: true,
                writable: true,
                unit: Some("unit".to_string()),
                min_value: Some(0.0),
                max_value: Some(100.0),
            })
            .collect();

        Ok(tags)
    }

    fn get_communication_stats(&self) -> PlcCommunicationStats {
        self.stats.lock().unwrap().clone()
    }

    fn reset_communication_stats(&mut self) {
        let mut stats = self.stats.lock().unwrap();
        *stats = PlcCommunicationStats::default();
        self.last_error.lock().unwrap().take();
        log::info!("通信统计信息已重置 for {}", self.service_name);
    }

    fn set_read_timeout(&mut self, timeout_ms: u32) -> AppResult<()> {
        self.read_timeout_ms = timeout_ms;
        Ok(())
    }

    fn set_write_timeout(&mut self, timeout_ms: u32) -> AppResult<()> {
        self.write_timeout_ms = timeout_ms;
        Ok(())
    }

    async fn get_device_info(&self) -> AppResult<HashMap<String, String>> {
        self.simulate_delay().await;

        if !self.is_connected() {
            return Err(AppError::plc_communication_error("PLC未连接"));
        }

        let mut info = HashMap::new();
        info.insert("device_type".to_string(), "Mock PLC".to_string());
        info.insert("service_name".to_string(), self.service_name.clone());
        info.insert("version".to_string(), "1.0.0".to_string());
        info.insert("vendor".to_string(), "FAT_TEST Mock".to_string());
        info.insert(
            "connection_status".to_string(),
            format!("{:?}", self.connection_status),
        );

        Ok(info)
    }
}

// 为测试需要添加fastrand依赖
// 由于我们在测试环境中，暂时用简单的随机数生成
mod fastrand {
    use std::cell::RefCell;
    use std::rc::Rc;

    thread_local! {
        static RNG: Rc<RefCell<u64>> = Rc::new(RefCell::new(1));
    }

    pub fn f64() -> f64 {
        RNG.with(|rng| {
            let mut state = rng.borrow_mut();
            *state = state.wrapping_mul(1103515245).wrapping_add(12345);
            (*state as f64) / (u64::MAX as f64)
        })
    }
} 