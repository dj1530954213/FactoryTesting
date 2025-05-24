/// Mock PLC 服务实现
/// 用于开发和测试阶段，模拟真实的PLC通信行为

use async_trait::async_trait;
use chrono::Utc;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::time::sleep;

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

        if self.should_simulate_error() {
            self.connection_status = PlcConnectionStatus::Error("模拟连接失败".to_string());
            return Err(AppError::plc_communication_error("模拟的PLC连接失败"));
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
        let start_time = Instant::now();
        self.simulate_delay().await;

        if !self.is_connected() {
            return Err(AppError::plc_communication_error("PLC未连接"));
        }

        if self.should_simulate_error() {
            self.update_stats(|stats| stats.failed_reads += 1);
            return Err(AppError::plc_communication_error(format!(
                "模拟读取错误: {}",
                address
            )));
        }

        let result = self
            .get_stored_value(address)
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        self.update_stats(|stats| {
            stats.successful_reads += 1;
            stats.total_read_time_ms += start_time.elapsed().as_millis() as u64;
        });

        Ok(result)
    }

    async fn read_int8(&self, address: &str) -> AppResult<i8> {
        let start_time = Instant::now();
        self.simulate_delay().await;

        if !self.is_connected() {
            return Err(AppError::plc_communication_error("PLC未连接"));
        }

        if self.should_simulate_error() {
            self.update_stats(|stats| stats.failed_reads += 1);
            return Err(AppError::plc_communication_error(format!(
                "模拟读取错误: {}",
                address
            )));
        }

        let result = self
            .get_stored_value(address)
            .and_then(|v| v.as_i64())
            .map(|v| v as i8)
            .unwrap_or(0);

        self.update_stats(|stats| {
            stats.successful_reads += 1;
            stats.total_read_time_ms += start_time.elapsed().as_millis() as u64;
        });

        Ok(result)
    }

    async fn read_uint8(&self, address: &str) -> AppResult<u8> {
        let start_time = Instant::now();
        self.simulate_delay().await;

        if !self.is_connected() {
            return Err(AppError::plc_communication_error("PLC未连接"));
        }

        if self.should_simulate_error() {
            self.update_stats(|stats| stats.failed_reads += 1);
            return Err(AppError::plc_communication_error(format!(
                "模拟读取错误: {}",
                address
            )));
        }

        let result = self
            .get_stored_value(address)
            .and_then(|v| v.as_u64())
            .map(|v| v as u8)
            .unwrap_or(0);

        self.update_stats(|stats| {
            stats.successful_reads += 1;
            stats.total_read_time_ms += start_time.elapsed().as_millis() as u64;
        });

        Ok(result)
    }

    async fn read_int16(&self, address: &str) -> AppResult<i16> {
        let start_time = Instant::now();
        self.simulate_delay().await;

        if !self.is_connected() {
            return Err(AppError::plc_communication_error("PLC未连接"));
        }

        if self.should_simulate_error() {
            self.update_stats(|stats| stats.failed_reads += 1);
            return Err(AppError::plc_communication_error(format!(
                "模拟读取错误: {}",
                address
            )));
        }

        let result = self
            .get_stored_value(address)
            .and_then(|v| v.as_i64())
            .map(|v| v as i16)
            .unwrap_or(0);

        self.update_stats(|stats| {
            stats.successful_reads += 1;
            stats.total_read_time_ms += start_time.elapsed().as_millis() as u64;
        });

        Ok(result)
    }

    async fn read_uint16(&self, address: &str) -> AppResult<u16> {
        let start_time = Instant::now();
        self.simulate_delay().await;

        if !self.is_connected() {
            return Err(AppError::plc_communication_error("PLC未连接"));
        }

        if self.should_simulate_error() {
            self.update_stats(|stats| stats.failed_reads += 1);
            return Err(AppError::plc_communication_error(format!(
                "模拟读取错误: {}",
                address
            )));
        }

        let result = self
            .get_stored_value(address)
            .and_then(|v| v.as_u64())
            .map(|v| v as u16)
            .unwrap_or(0);

        self.update_stats(|stats| {
            stats.successful_reads += 1;
            stats.total_read_time_ms += start_time.elapsed().as_millis() as u64;
        });

        Ok(result)
    }

    async fn read_int32(&self, address: &str) -> AppResult<i32> {
        let start_time = Instant::now();
        self.simulate_delay().await;

        if !self.is_connected() {
            return Err(AppError::plc_communication_error("PLC未连接"));
        }

        if self.should_simulate_error() {
            self.update_stats(|stats| stats.failed_reads += 1);
            return Err(AppError::plc_communication_error(format!(
                "模拟读取错误: {}",
                address
            )));
        }

        let result = self
            .get_stored_value(address)
            .and_then(|v| v.as_i64())
            .map(|v| v as i32)
            .unwrap_or(0);

        self.update_stats(|stats| {
            stats.successful_reads += 1;
            stats.total_read_time_ms += start_time.elapsed().as_millis() as u64;
        });

        Ok(result)
    }

    async fn read_uint32(&self, address: &str) -> AppResult<u32> {
        let start_time = Instant::now();
        self.simulate_delay().await;

        if !self.is_connected() {
            return Err(AppError::plc_communication_error("PLC未连接"));
        }

        if self.should_simulate_error() {
            self.update_stats(|stats| stats.failed_reads += 1);
            return Err(AppError::plc_communication_error(format!(
                "模拟读取错误: {}",
                address
            )));
        }

        let result = self
            .get_stored_value(address)
            .and_then(|v| v.as_u64())
            .map(|v| v as u32)
            .unwrap_or(0);

        self.update_stats(|stats| {
            stats.successful_reads += 1;
            stats.total_read_time_ms += start_time.elapsed().as_millis() as u64;
        });

        Ok(result)
    }

    async fn read_int64(&self, address: &str) -> AppResult<i64> {
        let start_time = Instant::now();
        self.simulate_delay().await;

        if !self.is_connected() {
            return Err(AppError::plc_communication_error("PLC未连接"));
        }

        if self.should_simulate_error() {
            self.update_stats(|stats| stats.failed_reads += 1);
            return Err(AppError::plc_communication_error(format!(
                "模拟读取错误: {}",
                address
            )));
        }

        let result = self
            .get_stored_value(address)
            .and_then(|v| v.as_i64())
            .unwrap_or(0);

        self.update_stats(|stats| {
            stats.successful_reads += 1;
            stats.total_read_time_ms += start_time.elapsed().as_millis() as u64;
        });

        Ok(result)
    }

    async fn read_uint64(&self, address: &str) -> AppResult<u64> {
        let start_time = Instant::now();
        self.simulate_delay().await;

        if !self.is_connected() {
            return Err(AppError::plc_communication_error("PLC未连接"));
        }

        if self.should_simulate_error() {
            self.update_stats(|stats| stats.failed_reads += 1);
            return Err(AppError::plc_communication_error(format!(
                "模拟读取错误: {}",
                address
            )));
        }

        let result = self
            .get_stored_value(address)
            .and_then(|v| v.as_u64())
            .unwrap_or(0);

        self.update_stats(|stats| {
            stats.successful_reads += 1;
            stats.total_read_time_ms += start_time.elapsed().as_millis() as u64;
        });

        Ok(result)
    }

    async fn read_float32(&self, address: &str) -> AppResult<f32> {
        let start_time = Instant::now();
        self.simulate_delay().await;

        if !self.is_connected() {
            return Err(AppError::plc_communication_error("PLC未连接"));
        }

        if self.should_simulate_error() {
            self.update_stats(|stats| stats.failed_reads += 1);
            return Err(AppError::plc_communication_error(format!(
                "模拟读取错误: {}",
                address
            )));
        }

        let result = self
            .get_stored_value(address)
            .and_then(|v| v.as_f64())
            .map(|v| v as f32)
            .unwrap_or(0.0);

        self.update_stats(|stats| {
            stats.successful_reads += 1;
            stats.total_read_time_ms += start_time.elapsed().as_millis() as u64;
        });

        Ok(result)
    }

    async fn read_float64(&self, address: &str) -> AppResult<f64> {
        let start_time = Instant::now();
        self.simulate_delay().await;

        if !self.is_connected() {
            return Err(AppError::plc_communication_error("PLC未连接"));
        }

        if self.should_simulate_error() {
            self.update_stats(|stats| stats.failed_reads += 1);
            return Err(AppError::plc_communication_error(format!(
                "模拟读取错误: {}",
                address
            )));
        }

        let result = self
            .get_stored_value(address)
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);

        self.update_stats(|stats| {
            stats.successful_reads += 1;
            stats.total_read_time_ms += start_time.elapsed().as_millis() as u64;
        });

        Ok(result)
    }

    async fn read_string(&self, address: &str, _max_length: usize) -> AppResult<String> {
        let start_time = Instant::now();
        self.simulate_delay().await;

        if !self.is_connected() {
            return Err(AppError::plc_communication_error("PLC未连接"));
        }

        if self.should_simulate_error() {
            self.update_stats(|stats| stats.failed_reads += 1);
            return Err(AppError::plc_communication_error(format!(
                "模拟读取错误: {}",
                address
            )));
        }

        let result = self
            .get_stored_value(address)
            .and_then(|v| v.as_str().map(|s| s.to_string()))
            .unwrap_or_default();

        self.update_stats(|stats| {
            stats.successful_reads += 1;
            stats.total_read_time_ms += start_time.elapsed().as_millis() as u64;
        });

        Ok(result)
    }

    async fn read_bytes(&self, address: &str, _length: usize) -> AppResult<Vec<u8>> {
        let start_time = Instant::now();
        self.simulate_delay().await;

        if !self.is_connected() {
            return Err(AppError::plc_communication_error("PLC未连接"));
        }

        if self.should_simulate_error() {
            self.update_stats(|stats| stats.failed_reads += 1);
            return Err(AppError::plc_communication_error(format!(
                "模拟读取错误: {}",
                address
            )));
        }

        let result = self
            .get_stored_value(address)
            .and_then(|v| {
                v.as_array().map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_u64())
                        .map(|v| v as u8)
                        .collect()
                })
            })
            .unwrap_or_default();

        self.update_stats(|stats| {
            stats.successful_reads += 1;
            stats.total_read_time_ms += start_time.elapsed().as_millis() as u64;
        });

        Ok(result)
    }

    async fn write_bool(&self, address: &str, value: bool) -> AppResult<()> {
        let start_time = Instant::now();
        self.simulate_delay().await;

        if !self.is_connected() {
            return Err(AppError::plc_communication_error("PLC未连接"));
        }

        if self.should_simulate_error() {
            self.update_stats(|stats| stats.failed_writes += 1);
            return Err(AppError::plc_communication_error(format!(
                "模拟写入错误: {}",
                address
            )));
        }

        let json_value = Value::Bool(value);
        self.store_value(address.to_string(), json_value.clone());
        self.log_write_operation(address.to_string(), json_value, "write_bool".to_string());

        self.update_stats(|stats| {
            stats.successful_writes += 1;
            stats.total_write_time_ms += start_time.elapsed().as_millis() as u64;
        });

        Ok(())
    }

    async fn write_int8(&self, address: &str, value: i8) -> AppResult<()> {
        let start_time = Instant::now();
        self.simulate_delay().await;

        if !self.is_connected() {
            return Err(AppError::plc_communication_error("PLC未连接"));
        }

        if self.should_simulate_error() {
            self.update_stats(|stats| stats.failed_writes += 1);
            return Err(AppError::plc_communication_error(format!(
                "模拟写入错误: {}",
                address
            )));
        }

        let json_value = Value::Number(serde_json::Number::from(value as i64));
        self.store_value(address.to_string(), json_value.clone());
        self.log_write_operation(address.to_string(), json_value, "write_int8".to_string());

        self.update_stats(|stats| {
            stats.successful_writes += 1;
            stats.total_write_time_ms += start_time.elapsed().as_millis() as u64;
        });

        Ok(())
    }

    async fn write_uint8(&self, address: &str, value: u8) -> AppResult<()> {
        let start_time = Instant::now();
        self.simulate_delay().await;

        if !self.is_connected() {
            return Err(AppError::plc_communication_error("PLC未连接"));
        }

        if self.should_simulate_error() {
            self.update_stats(|stats| stats.failed_writes += 1);
            return Err(AppError::plc_communication_error(format!(
                "模拟写入错误: {}",
                address
            )));
        }

        let json_value = Value::Number(serde_json::Number::from(value as u64));
        self.store_value(address.to_string(), json_value.clone());
        self.log_write_operation(address.to_string(), json_value, "write_uint8".to_string());

        self.update_stats(|stats| {
            stats.successful_writes += 1;
            stats.total_write_time_ms += start_time.elapsed().as_millis() as u64;
        });

        Ok(())
    }

    async fn write_int16(&self, address: &str, value: i16) -> AppResult<()> {
        let start_time = Instant::now();
        self.simulate_delay().await;

        if !self.is_connected() {
            return Err(AppError::plc_communication_error("PLC未连接"));
        }

        if self.should_simulate_error() {
            self.update_stats(|stats| stats.failed_writes += 1);
            return Err(AppError::plc_communication_error(format!(
                "模拟写入错误: {}",
                address
            )));
        }

        let json_value = Value::Number(serde_json::Number::from(value as i64));
        self.store_value(address.to_string(), json_value.clone());
        self.log_write_operation(address.to_string(), json_value, "write_int16".to_string());

        self.update_stats(|stats| {
            stats.successful_writes += 1;
            stats.total_write_time_ms += start_time.elapsed().as_millis() as u64;
        });

        Ok(())
    }

    async fn write_uint16(&self, address: &str, value: u16) -> AppResult<()> {
        let start_time = Instant::now();
        self.simulate_delay().await;

        if !self.is_connected() {
            return Err(AppError::plc_communication_error("PLC未连接"));
        }

        if self.should_simulate_error() {
            self.update_stats(|stats| stats.failed_writes += 1);
            return Err(AppError::plc_communication_error(format!(
                "模拟写入错误: {}",
                address
            )));
        }

        let json_value = Value::Number(serde_json::Number::from(value as u64));
        self.store_value(address.to_string(), json_value.clone());
        self.log_write_operation(address.to_string(), json_value, "write_uint16".to_string());

        self.update_stats(|stats| {
            stats.successful_writes += 1;
            stats.total_write_time_ms += start_time.elapsed().as_millis() as u64;
        });

        Ok(())
    }

    async fn write_int32(&self, address: &str, value: i32) -> AppResult<()> {
        let start_time = Instant::now();
        self.simulate_delay().await;

        if !self.is_connected() {
            return Err(AppError::plc_communication_error("PLC未连接"));
        }

        if self.should_simulate_error() {
            self.update_stats(|stats| stats.failed_writes += 1);
            return Err(AppError::plc_communication_error(format!(
                "模拟写入错误: {}",
                address
            )));
        }

        let json_value = Value::Number(serde_json::Number::from(value as i64));
        self.store_value(address.to_string(), json_value.clone());
        self.log_write_operation(address.to_string(), json_value, "write_int32".to_string());

        self.update_stats(|stats| {
            stats.successful_writes += 1;
            stats.total_write_time_ms += start_time.elapsed().as_millis() as u64;
        });

        Ok(())
    }

    async fn write_uint32(&self, address: &str, value: u32) -> AppResult<()> {
        let start_time = Instant::now();
        self.simulate_delay().await;

        if !self.is_connected() {
            return Err(AppError::plc_communication_error("PLC未连接"));
        }

        if self.should_simulate_error() {
            self.update_stats(|stats| stats.failed_writes += 1);
            return Err(AppError::plc_communication_error(format!(
                "模拟写入错误: {}",
                address
            )));
        }

        let json_value = Value::Number(serde_json::Number::from(value as u64));
        self.store_value(address.to_string(), json_value.clone());
        self.log_write_operation(address.to_string(), json_value, "write_uint32".to_string());

        self.update_stats(|stats| {
            stats.successful_writes += 1;
            stats.total_write_time_ms += start_time.elapsed().as_millis() as u64;
        });

        Ok(())
    }

    async fn write_int64(&self, address: &str, value: i64) -> AppResult<()> {
        let start_time = Instant::now();
        self.simulate_delay().await;

        if !self.is_connected() {
            return Err(AppError::plc_communication_error("PLC未连接"));
        }

        if self.should_simulate_error() {
            self.update_stats(|stats| stats.failed_writes += 1);
            return Err(AppError::plc_communication_error(format!(
                "模拟写入错误: {}",
                address
            )));
        }

        let json_value = Value::Number(serde_json::Number::from(value));
        self.store_value(address.to_string(), json_value.clone());
        self.log_write_operation(address.to_string(), json_value, "write_int64".to_string());

        self.update_stats(|stats| {
            stats.successful_writes += 1;
            stats.total_write_time_ms += start_time.elapsed().as_millis() as u64;
        });

        Ok(())
    }

    async fn write_uint64(&self, address: &str, value: u64) -> AppResult<()> {
        let start_time = Instant::now();
        self.simulate_delay().await;

        if !self.is_connected() {
            return Err(AppError::plc_communication_error("PLC未连接"));
        }

        if self.should_simulate_error() {
            self.update_stats(|stats| stats.failed_writes += 1);
            return Err(AppError::plc_communication_error(format!(
                "模拟写入错误: {}",
                address
            )));
        }

        let json_value = Value::Number(serde_json::Number::from(value));
        self.store_value(address.to_string(), json_value.clone());
        self.log_write_operation(address.to_string(), json_value, "write_uint64".to_string());

        self.update_stats(|stats| {
            stats.successful_writes += 1;
            stats.total_write_time_ms += start_time.elapsed().as_millis() as u64;
        });

        Ok(())
    }

    async fn write_float32(&self, address: &str, value: f32) -> AppResult<()> {
        let start_time = Instant::now();
        self.simulate_delay().await;

        if !self.is_connected() {
            return Err(AppError::plc_communication_error("PLC未连接"));
        }

        if self.should_simulate_error() {
            self.update_stats(|stats| stats.failed_writes += 1);
            return Err(AppError::plc_communication_error(format!(
                "模拟写入错误: {}",
                address
            )));
        }

        let json_value = Value::Number(
            serde_json::Number::from_f64(value as f64)
                .unwrap_or_else(|| serde_json::Number::from(0)),
        );
        self.store_value(address.to_string(), json_value.clone());
        self.log_write_operation(address.to_string(), json_value, "write_float32".to_string());

        self.update_stats(|stats| {
            stats.successful_writes += 1;
            stats.total_write_time_ms += start_time.elapsed().as_millis() as u64;
        });

        Ok(())
    }

    async fn write_float64(&self, address: &str, value: f64) -> AppResult<()> {
        let start_time = Instant::now();
        self.simulate_delay().await;

        if !self.is_connected() {
            return Err(AppError::plc_communication_error("PLC未连接"));
        }

        if self.should_simulate_error() {
            self.update_stats(|stats| stats.failed_writes += 1);
            return Err(AppError::plc_communication_error(format!(
                "模拟写入错误: {}",
                address
            )));
        }

        let json_value = Value::Number(
            serde_json::Number::from_f64(value).unwrap_or_else(|| serde_json::Number::from(0)),
        );
        self.store_value(address.to_string(), json_value.clone());
        self.log_write_operation(address.to_string(), json_value, "write_float64".to_string());

        self.update_stats(|stats| {
            stats.successful_writes += 1;
            stats.total_write_time_ms += start_time.elapsed().as_millis() as u64;
        });

        Ok(())
    }

    async fn write_string(&self, address: &str, value: &str) -> AppResult<()> {
        let start_time = Instant::now();
        self.simulate_delay().await;

        if !self.is_connected() {
            return Err(AppError::plc_communication_error("PLC未连接"));
        }

        if self.should_simulate_error() {
            self.update_stats(|stats| stats.failed_writes += 1);
            return Err(AppError::plc_communication_error(format!(
                "模拟写入错误: {}",
                address
            )));
        }

        let json_value = Value::String(value.to_string());
        self.store_value(address.to_string(), json_value.clone());
        self.log_write_operation(address.to_string(), json_value, "write_string".to_string());

        self.update_stats(|stats| {
            stats.successful_writes += 1;
            stats.total_write_time_ms += start_time.elapsed().as_millis() as u64;
        });

        Ok(())
    }

    async fn write_bytes(&self, address: &str, value: &[u8]) -> AppResult<()> {
        let start_time = Instant::now();
        self.simulate_delay().await;

        if !self.is_connected() {
            return Err(AppError::plc_communication_error("PLC未连接"));
        }

        if self.should_simulate_error() {
            self.update_stats(|stats| stats.failed_writes += 1);
            return Err(AppError::plc_communication_error(format!(
                "模拟写入错误: {}",
                address
            )));
        }

        let json_value = Value::Array(
            value
                .iter()
                .map(|&b| Value::Number(serde_json::Number::from(b as u64)))
                .collect(),
        );
        self.store_value(address.to_string(), json_value.clone());
        self.log_write_operation(address.to_string(), json_value, "write_bytes".to_string());

        self.update_stats(|stats| {
            stats.successful_writes += 1;
            stats.total_write_time_ms += start_time.elapsed().as_millis() as u64;
        });

        Ok(())
    }

    async fn batch_read(&self, addresses: &[String]) -> AppResult<HashMap<String, Value>> {
        let start_time = Instant::now();
        self.simulate_delay().await;

        if !self.is_connected() {
            return Err(AppError::plc_communication_error("PLC未连接"));
        }

        if self.should_simulate_error() {
            self.update_stats(|stats| stats.failed_reads += addresses.len() as u64);
            return Err(AppError::plc_communication_error("模拟批量读取错误"));
        }

        let mut results = HashMap::new();
        let storage = self.data_storage.lock().unwrap();

        for address in addresses {
            if let Some(value) = storage.get(address) {
                results.insert(address.clone(), value.clone());
            } else {
                results.insert(address.clone(), Value::Null);
            }
        }

        self.update_stats(|stats| {
            stats.successful_reads += addresses.len() as u64;
            stats.total_read_time_ms += start_time.elapsed().as_millis() as u64;
        });

        Ok(results)
    }

    async fn batch_write(&self, values: &HashMap<String, Value>) -> AppResult<()> {
        let start_time = Instant::now();
        self.simulate_delay().await;

        if !self.is_connected() {
            return Err(AppError::plc_communication_error("PLC未连接"));
        }

        if self.should_simulate_error() {
            self.update_stats(|stats| stats.failed_writes += values.len() as u64);
            return Err(AppError::plc_communication_error("模拟批量写入错误"));
        }

        {
            let mut storage = self.data_storage.lock().unwrap();
            for (address, value) in values {
                storage.insert(address.clone(), value.clone());
            }
        }

        // 记录批量写入操作
        for (address, value) in values {
            self.log_write_operation(
                address.clone(),
                value.clone(),
                "batch_write".to_string(),
            );
        }

        self.update_stats(|stats| {
            stats.successful_writes += values.len() as u64;
            stats.total_write_time_ms += start_time.elapsed().as_millis() as u64;
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
        *self.stats.lock().unwrap() = PlcCommunicationStats::default();
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