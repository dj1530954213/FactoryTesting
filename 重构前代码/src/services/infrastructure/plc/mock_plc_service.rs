/// Mock PLC服务实现
/// 用于测试和开发环境的PLC通信模拟

use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use serde_json::Value;
use chrono::Utc;

use crate::utils::error::{AppError, AppResult};
use crate::services::traits::BaseService;
use super::plc_communication_service::{
    PlcCommunicationService, PlcConnectionStatus, PlcDataType, PlcTag, PlcCommunicationStats
};

/// Mock PLC服务
/// 模拟PLC通信行为，用于测试和开发
pub struct MockPlcService {
    /// 服务名称
    name: String,
    /// 连接状态
    connection_status: PlcConnectionStatus,
    /// 预设的读取值
    preset_values: Arc<Mutex<HashMap<String, Value>>>,
    /// 写入历史记录
    write_history: Arc<Mutex<Vec<(String, Value)>>>,
    /// 通信统计
    stats: PlcCommunicationStats,
}

impl MockPlcService {
    /// 创建新的Mock PLC服务实例
    pub fn new_for_testing(name: &str) -> Self {
        log::info!("🔧 [MOCK_PLC] 创建Mock PLC服务: {}", name);
        Self {
            name: name.to_string(),
            connection_status: PlcConnectionStatus::Disconnected,
            preset_values: Arc::new(Mutex::new(HashMap::new())),
            write_history: Arc::new(Mutex::new(Vec::new())),
            stats: PlcCommunicationStats::default(),
        }
    }

    /// 预设读取值
    pub fn preset_read_value(&self, address: &str, value: Value) {
        log::info!("🔧 [MOCK_PLC] 预设读取值: {} -> {:?}", address, value);
        let mut values = self.preset_values.lock().unwrap();
        values.insert(address.to_string(), value);
    }

    /// 获取写入历史记录
    pub fn get_write_log(&self) -> Vec<(String, Value)> {
        let history = self.write_history.lock().unwrap();
        history.clone()
    }

    /// 清除写入历史记录
    pub fn clear_write_log(&self) {
        let mut history = self.write_history.lock().unwrap();
        history.clear();
    }

    /// 记录写入操作
    fn record_write(&self, address: &str, value: Value) {
        log::info!("📝 [MOCK_PLC] 记录写入: {} -> {:?}", address, value);
        let mut history = self.write_history.lock().unwrap();
        history.push((address.to_string(), value));
    }

    /// 获取预设值
    fn get_preset_value(&self, address: &str) -> Option<Value> {
        let values = self.preset_values.lock().unwrap();
        values.get(address).cloned()
    }
}

#[async_trait]
impl BaseService for MockPlcService {
    fn service_name(&self) -> &'static str {
        "MockPlcService"
    }

    async fn initialize(&mut self) -> AppResult<()> {
        log::info!("🔧 [MOCK_PLC] 初始化Mock PLC服务: {}", self.name);
        Ok(())
    }

    async fn shutdown(&mut self) -> AppResult<()> {
        log::info!("🔧 [MOCK_PLC] 关闭Mock PLC服务: {}", self.name);
        self.connection_status = PlcConnectionStatus::Disconnected;
        Ok(())
    }

    async fn health_check(&self) -> AppResult<()> {
        Ok(())
    }
}

#[async_trait]
impl PlcCommunicationService for MockPlcService {
    async fn connect(&mut self) -> AppResult<()> {
        log::info!("🔗 [MOCK_PLC] 连接Mock PLC: {}", self.name);
        self.connection_status = PlcConnectionStatus::Connected;
        Ok(())
    }

    async fn disconnect(&mut self) -> AppResult<()> {
        log::info!("🔌 [MOCK_PLC] 断开Mock PLC: {}", self.name);
        self.connection_status = PlcConnectionStatus::Disconnected;
        Ok(())
    }

    fn get_connection_status(&self) -> PlcConnectionStatus {
        self.connection_status.clone()
    }

    async fn test_connection(&self) -> AppResult<bool> {
        Ok(matches!(self.connection_status, PlcConnectionStatus::Connected))
    }

    // 基础数据类型读取方法
    async fn read_bool_impl(&self, address: &str) -> AppResult<bool> {
        log::info!("🔍 [MOCK_PLC_READ_BOOL] 地址: {}", address);
        
        if let Some(value) = self.get_preset_value(address) {
            match value {
                Value::Bool(b) => {
                    log::info!("✅ [MOCK_PLC_READ_BOOL] 地址: {}, 值: {}", address, b);
                    Ok(b)
                },
                _ => {
                    log::error!("❌ [MOCK_PLC_READ_BOOL] 地址: {}, 类型错误: {:?}", address, value);
                    Err(AppError::PlcCommunicationError { 
                        message: format!("地址 {} 的预设值不是布尔类型", address) 
                    })
                }
            }
        } else {
            // 默认返回false
            log::info!("🔍 [MOCK_PLC_READ_BOOL] 地址: {}, 使用默认值: false", address);
            Ok(false)
        }
    }

    async fn read_int8(&self, address: &str) -> AppResult<i8> {
        if let Some(value) = self.get_preset_value(address) {
            match value {
                Value::Number(n) => Ok(n.as_i64().unwrap_or(0) as i8),
                _ => Err(AppError::PlcCommunicationError { 
                    message: format!("地址 {} 的预设值不是数字类型", address) 
                })
            }
        } else {
            Ok(0)
        }
    }

    async fn read_uint8(&self, address: &str) -> AppResult<u8> {
        if let Some(value) = self.get_preset_value(address) {
            match value {
                Value::Number(n) => Ok(n.as_u64().unwrap_or(0) as u8),
                _ => Err(AppError::PlcCommunicationError { 
                    message: format!("地址 {} 的预设值不是数字类型", address) 
                })
            }
        } else {
            Ok(0)
        }
    }

    async fn read_int16(&self, address: &str) -> AppResult<i16> {
        if let Some(value) = self.get_preset_value(address) {
            match value {
                Value::Number(n) => Ok(n.as_i64().unwrap_or(0) as i16),
                _ => Err(AppError::PlcCommunicationError { 
                    message: format!("地址 {} 的预设值不是数字类型", address) 
                })
            }
        } else {
            Ok(0)
        }
    }

    async fn read_uint16(&self, address: &str) -> AppResult<u16> {
        if let Some(value) = self.get_preset_value(address) {
            match value {
                Value::Number(n) => Ok(n.as_u64().unwrap_or(0) as u16),
                _ => Err(AppError::PlcCommunicationError { 
                    message: format!("地址 {} 的预设值不是数字类型", address) 
                })
            }
        } else {
            Ok(0)
        }
    }

    async fn read_int32(&self, address: &str) -> AppResult<i32> {
        if let Some(value) = self.get_preset_value(address) {
            match value {
                Value::Number(n) => Ok(n.as_i64().unwrap_or(0) as i32),
                _ => Err(AppError::PlcCommunicationError { 
                    message: format!("地址 {} 的预设值不是数字类型", address) 
                })
            }
        } else {
            Ok(0)
        }
    }

    async fn read_uint32(&self, address: &str) -> AppResult<u32> {
        if let Some(value) = self.get_preset_value(address) {
            match value {
                Value::Number(n) => Ok(n.as_u64().unwrap_or(0) as u32),
                _ => Err(AppError::PlcCommunicationError { 
                    message: format!("地址 {} 的预设值不是数字类型", address) 
                })
            }
        } else {
            Ok(0)
        }
    }

    async fn read_int64(&self, address: &str) -> AppResult<i64> {
        if let Some(value) = self.get_preset_value(address) {
            match value {
                Value::Number(n) => Ok(n.as_i64().unwrap_or(0)),
                _ => Err(AppError::PlcCommunicationError { 
                    message: format!("地址 {} 的预设值不是数字类型", address) 
                })
            }
        } else {
            Ok(0)
        }
    }

    async fn read_uint64(&self, address: &str) -> AppResult<u64> {
        if let Some(value) = self.get_preset_value(address) {
            match value {
                Value::Number(n) => Ok(n.as_u64().unwrap_or(0)),
                _ => Err(AppError::PlcCommunicationError { 
                    message: format!("地址 {} 的预设值不是数字类型", address) 
                })
            }
        } else {
            Ok(0)
        }
    }

    async fn read_float32_impl(&self, address: &str) -> AppResult<f32> {
        log::info!("🔍 [MOCK_PLC_READ_F32] 地址: {}", address);
        
        if let Some(value) = self.get_preset_value(address) {
            match value {
                Value::Number(n) => {
                    let f_val = n.as_f64().unwrap_or(0.0) as f32;
                    log::info!("✅ [MOCK_PLC_READ_F32] 地址: {}, 值: {}", address, f_val);
                    Ok(f_val)
                },
                _ => {
                    log::error!("❌ [MOCK_PLC_READ_F32] 地址: {}, 类型错误: {:?}", address, value);
                    Err(AppError::PlcCommunicationError { 
                        message: format!("地址 {} 的预设值不是数字类型", address) 
                    })
                }
            }
        } else {
            log::info!("🔍 [MOCK_PLC_READ_F32] 地址: {}, 使用默认值: 0.0", address);
            Ok(0.0)
        }
    }

    async fn read_float64(&self, address: &str) -> AppResult<f64> {
        if let Some(value) = self.get_preset_value(address) {
            match value {
                Value::Number(n) => Ok(n.as_f64().unwrap_or(0.0)),
                _ => Err(AppError::PlcCommunicationError { 
                    message: format!("地址 {} 的预设值不是数字类型", address) 
                })
            }
        } else {
            Ok(0.0)
        }
    }

    async fn read_string(&self, address: &str, _max_length: usize) -> AppResult<String> {
        if let Some(value) = self.get_preset_value(address) {
            match value {
                Value::String(s) => Ok(s),
                _ => Err(AppError::PlcCommunicationError { 
                    message: format!("地址 {} 的预设值不是字符串类型", address) 
                })
            }
        } else {
            Ok(String::new())
        }
    }

    async fn read_bytes(&self, address: &str, length: usize) -> AppResult<Vec<u8>> {
        if let Some(value) = self.get_preset_value(address) {
            match value {
                Value::Array(arr) => {
                    let mut bytes = Vec::new();
                    for item in arr.iter().take(length) {
                        if let Value::Number(n) = item {
                            bytes.push(n.as_u64().unwrap_or(0) as u8);
                        }
                    }
                    Ok(bytes)
                },
                _ => Err(AppError::PlcCommunicationError { 
                    message: format!("地址 {} 的预设值不是数组类型", address) 
                })
            }
        } else {
            Ok(vec![0; length])
        }
    }

    // 基础数据类型写入方法
    async fn write_bool_impl(&self, address: &str, value: bool) -> AppResult<()> {
        log::info!("📝 [MOCK_PLC_WRITE_BOOL] 地址: {}, 值: {}", address, value);
        self.record_write(address, Value::Bool(value));
        log::info!("✅ [MOCK_PLC_WRITE_BOOL] 地址: {}, 值: {} - 写入成功", address, value);
        Ok(())
    }

    async fn write_int8(&self, address: &str, value: i8) -> AppResult<()> {
        self.record_write(address, Value::Number(serde_json::Number::from(value)));
        Ok(())
    }

    async fn write_uint8(&self, address: &str, value: u8) -> AppResult<()> {
        self.record_write(address, Value::Number(serde_json::Number::from(value)));
        Ok(())
    }

    async fn write_int16(&self, address: &str, value: i16) -> AppResult<()> {
        self.record_write(address, Value::Number(serde_json::Number::from(value)));
        Ok(())
    }

    async fn write_uint16(&self, address: &str, value: u16) -> AppResult<()> {
        self.record_write(address, Value::Number(serde_json::Number::from(value)));
        Ok(())
    }

    async fn write_int32(&self, address: &str, value: i32) -> AppResult<()> {
        self.record_write(address, Value::Number(serde_json::Number::from(value)));
        Ok(())
    }

    async fn write_uint32(&self, address: &str, value: u32) -> AppResult<()> {
        self.record_write(address, Value::Number(serde_json::Number::from(value)));
        Ok(())
    }

    async fn write_int64(&self, address: &str, value: i64) -> AppResult<()> {
        self.record_write(address, Value::Number(serde_json::Number::from(value)));
        Ok(())
    }

    async fn write_uint64(&self, address: &str, value: u64) -> AppResult<()> {
        self.record_write(address, Value::Number(serde_json::Number::from(value)));
        Ok(())
    }

    async fn write_float32_impl(&self, address: &str, value: f32) -> AppResult<()> {
        log::info!("📝 [MOCK_PLC_WRITE_F32] 地址: {}, 值: {}", address, value);
        self.record_write(address, Value::Number(serde_json::Number::from_f64(value as f64).unwrap()));
        log::info!("✅ [MOCK_PLC_WRITE_F32] 地址: {}, 值: {} - 写入成功", address, value);
        Ok(())
    }

    async fn write_float64(&self, address: &str, value: f64) -> AppResult<()> {
        self.record_write(address, Value::Number(serde_json::Number::from_f64(value).unwrap()));
        Ok(())
    }

    async fn write_string(&self, address: &str, value: &str) -> AppResult<()> {
        self.record_write(address, Value::String(value.to_string()));
        Ok(())
    }

    async fn write_bytes(&self, address: &str, value: &[u8]) -> AppResult<()> {
        let array: Vec<Value> = value.iter().map(|&b| Value::Number(serde_json::Number::from(b))).collect();
        self.record_write(address, Value::Array(array));
        Ok(())
    }

    // 高级操作方法
    async fn batch_read(&self, addresses: &[String]) -> AppResult<HashMap<String, Value>> {
        let mut results = HashMap::new();
        for address in addresses {
            if let Some(value) = self.get_preset_value(address) {
                results.insert(address.clone(), value);
            } else {
                results.insert(address.clone(), Value::Null);
            }
        }
        Ok(results)
    }

    async fn batch_write(&self, values: &HashMap<String, Value>) -> AppResult<()> {
        for (address, value) in values {
            self.record_write(address, value.clone());
        }
        Ok(())
    }

    async fn read_tag_info(&self, address: &str) -> AppResult<PlcTag> {
        Ok(PlcTag {
            address: address.to_string(),
            data_type: PlcDataType::Float32,
            description: Some(format!("Mock标签: {}", address)),
            readable: true,
            writable: true,
            unit: None,
            min_value: None,
            max_value: None,
        })
    }

    async fn list_available_tags(&self) -> AppResult<Vec<PlcTag>> {
        let values = self.preset_values.lock().unwrap();
        let mut tags = Vec::new();
        for address in values.keys() {
            tags.push(PlcTag {
                address: address.clone(),
                data_type: PlcDataType::Float32,
                description: Some(format!("Mock标签: {}", address)),
                readable: true,
                writable: true,
                unit: None,
                min_value: None,
                max_value: None,
            });
        }
        Ok(tags)
    }

    fn get_communication_stats(&self) -> PlcCommunicationStats {
        self.stats.clone()
    }

    fn reset_communication_stats(&mut self) {
        self.stats = PlcCommunicationStats::default();
    }

    fn set_read_timeout(&mut self, _timeout_ms: u32) -> AppResult<()> {
        Ok(())
    }

    fn set_write_timeout(&mut self, _timeout_ms: u32) -> AppResult<()> {
        Ok(())
    }

    async fn get_device_info(&self) -> AppResult<HashMap<String, String>> {
        let mut info = HashMap::new();
        info.insert("device_type".to_string(), "Mock PLC".to_string());
        info.insert("name".to_string(), self.name.clone());
        info.insert("version".to_string(), "1.0.0".to_string());
        Ok(info)
    }
}
