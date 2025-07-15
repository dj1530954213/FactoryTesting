use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
use tokio::time::{interval, Duration};
use async_trait::async_trait;
use uuid;

use crate::models::structs::{
    StartPlcMonitoringRequest,
    StartPlcMonitoringResponse,
    StopPlcMonitoringRequest,
    PlcMonitoringData,
};
use crate::domain::services::BaseService;
use crate::utils::error::AppResult;
use crate::infrastructure::plc_communication::{IPlcCommunicationService, get_global_plc_manager};
use crate::domain::services::plc_comm_extension::PlcServiceLegacyExt;
use crate::domain::services::EventPublisher;

/// PLC监控服务接口
/// 
/// 负责实时监控PLC变量，支持0.5秒间隔的定时读取
/// 用于手动测试期间的实时数据显示
#[async_trait]
pub trait IPlcMonitoringService: BaseService {
    /// 开始PLC监控
    /// 
    /// # 参数
    /// * `request` - 监控请求
    /// 
    /// # 返回
    /// * `StartPlcMonitoringResponse` - 监控启动响应
    async fn start_monitoring(&self, request: StartPlcMonitoringRequest) -> AppResult<StartPlcMonitoringResponse>;
    
    /// 停止PLC监控
    /// 
    /// # 参数
    /// * `request` - 停止请求
    async fn stop_monitoring(&self, request: StopPlcMonitoringRequest) -> AppResult<()>;
    
    /// 检查是否正在监控指定实例
    /// 
    /// # 参数
    /// * `instance_id` - 实例ID
    /// 
    /// # 返回
    /// * `bool` - 是否正在监控
    fn is_monitoring(&self, instance_id: &str) -> bool;
    
    /// 获取当前监控的实例列表
    /// 
    /// # 返回
    /// * `Vec<String>` - 正在监控的实例ID列表
    fn get_monitoring_instances(&self) -> Vec<String>;
}

/// PLC监控服务实现
pub struct PlcMonitoringService {
    /// PLC通信服务
    plc_service: Arc<dyn IPlcCommunicationService>,
    
    /// 事件发布器
    event_publisher: Arc<dyn EventPublisher>,
    
    /// 活跃的监控任务
    active_monitors: Arc<Mutex<HashMap<String, MonitoringTask>>>,
    
    /// 服务名称
    service_name: String,
}

/// 监控任务
struct MonitoringTask {
    /// 实例ID
    instance_id: String,
    
    /// 监控ID
    monitoring_id: String,
    
    /// 取消令牌发送器
    cancel_sender: mpsc::UnboundedSender<()>,
    
    /// 任务句柄
    task_handle: tokio::task::JoinHandle<()>,
}

impl PlcMonitoringService {
    /// 创建新的PLC监控服务
    pub fn new(
        plc_service: Arc<dyn IPlcCommunicationService>,
        event_publisher: Arc<dyn EventPublisher>,
    ) -> Self {
        Self {
            plc_service,
            event_publisher,
            active_monitors: Arc::new(Mutex::new(HashMap::new())),
            service_name: "PlcMonitoringService".to_string(),
        }
    }
    
    /// 启动监控任务
    async fn start_monitoring_task(
        &self,
        request: StartPlcMonitoringRequest,
        monitoring_id: String,
    ) -> AppResult<()> {
        let (cancel_sender, mut cancel_receiver) = mpsc::unbounded_channel();
        
        let plc_service = self.plc_service.clone();
        let event_publisher = self.event_publisher.clone();
        let instance_id = request.instance_id.clone();
        let addresses = request.monitoring_addresses.clone();
        let module_type = request.module_type.clone();
        let address_key_map = request.address_key_map.clone();
        let opt_connection_id = request.connection_id.clone();
        
        // 启动监控任务
        let task_handle = tokio::spawn(async move {
            let mut interval = interval(Duration::from_millis(500)); // 0.5秒间隔
            let mut error_count = 0;
            let mut last_error_log = std::time::Instant::now();

            log::info!("🔧 [PLC_MONITORING] 开始监控任务: {} ({}个地址)", instance_id, addresses.len());

            loop {
                tokio::select! {
                    _ = cancel_receiver.recv() => {
                        log::info!("🛑 [PLC_MONITORING] 监控任务已取消: {}", instance_id);
                        break;
                    }
                    _ = interval.tick() => {
                        // 执行监控读取
                        if let Err(e) = Self::perform_monitoring_read(
                            plc_service.clone(),
                            event_publisher.clone(),
                            &instance_id,
                            &addresses,
                            &module_type,
                            address_key_map.as_ref(),
                            opt_connection_id.as_deref(),
                        ).await {
                            error_count += 1;

                            // 只在第一次错误或每10秒记录一次错误日志
                            let now = std::time::Instant::now();
                            if error_count == 1 || now.duration_since(last_error_log).as_secs() >= 10 {
                                log::error!("❌ [PLC_MONITORING] 监控读取失败: {} - {} (错误次数: {})", instance_id, e, error_count);
                                last_error_log = now;
                            }

                            // 发布错误事件
                            let _error_event = serde_json::json!({
                                "instanceId": instance_id,
                                "error": e.to_string()
                            });
                        } else {
                            // 成功读取时重置错误计数
                            if error_count > 0 {
                                log::info!("✅ [PLC_MONITORING] 监控读取恢复正常: {}", instance_id);
                                error_count = 0;
                            }
                        }
                    }
                }
            }
            
            // 发布监控停止事件
            let stop_event = serde_json::json!({
                "instanceId": instance_id,
                "reason": "任务完成"
            });
            
            // 简化停止事件处理，实际项目中需要使用正确的事件发布方法
            log::info!("🛑 [PLC_MONITORING] 监控任务已停止: {}", instance_id);
        });
        
        // 保存监控任务
        let task = MonitoringTask {
            instance_id: request.instance_id.clone(),
            monitoring_id: monitoring_id.clone(),
            cancel_sender,
            task_handle,
        };
        
        let mut monitors = self.active_monitors.lock().await;
        monitors.insert(request.instance_id, task);
        
        Ok(())
    }
    
    /// 执行监控读取
    async fn perform_monitoring_read(
        plc_service: Arc<dyn IPlcCommunicationService>,
        event_publisher: Arc<dyn EventPublisher>,
        instance_id: &str,
        addresses: &[String],
        module_type: &crate::models::enums::ModuleType,
        address_key_map: Option<&std::collections::HashMap<String, String>>,
        connection_id: Option<&str>,
    ) -> AppResult<()> {
        let mut values = HashMap::new();
        

        for address in addresses {
            let value_key = if let Some(map) = address_key_map {
                if let Some(k) = map.get(address) {
                    k.clone()
                } else {
                    Self::get_value_key(address, module_type)
                }
            } else {
                Self::get_value_key(address, module_type)
            };

            // 根据模块类型选择 PLC 连接 ID
            let connection_id = if let Some(cid) = connection_id { cid } else { match module_type {
                crate::models::enums::ModuleType::AI | crate::models::enums::ModuleType::DI |
                crate::models::enums::ModuleType::DINone | crate::models::enums::ModuleType::AINone => "target_plc",
                crate::models::enums::ModuleType::DO | crate::models::enums::ModuleType::AO |
                crate::models::enums::ModuleType::DONone | crate::models::enums::ModuleType::AONone => "manual_test_plc",
                crate::models::enums::ModuleType::Communication | crate::models::enums::ModuleType::Other(_) => "manual_test_plc",
            } };

            // 🐞 调试日志：记录当前读取所使用的连接ID及其对应的 PLC 端点（IP:Port）
            let endpoint_info = if let Some(mgr) = get_global_plc_manager() {
                match mgr.endpoint_by_id(connection_id).await {
                    Some(ep) => ep,
                    None => "unknown".to_string(),
                }
            } else {
                "n/a".to_string()
            };


            match module_type {
                crate::models::enums::ModuleType::AI | crate::models::enums::ModuleType::AO |
                crate::models::enums::ModuleType::AINone | crate::models::enums::ModuleType::AONone => {
                    // 读取浮点数值
                    match crate::domain::services::plc_comm_extension::PlcServiceLegacyExt::read_float32_by_id(&plc_service, connection_id, address).await {
                        Ok(value) => {
                            // log::debug!("✅ [PLC_MONITORING] 读取成功: {} = {}", address, value);
                            if let Some(number) = serde_json::Number::from_f64(value as f64) {
                                values.insert(value_key, serde_json::Value::Number(number));
                            }
                        }
                        Err(e) => {
                            log::warn!("⚠️ [PLC_MONITORING] 读取地址失败: {} - {}", address, e);
                            values.insert(value_key, serde_json::Value::Null);
                        }
                    }
                }
                crate::models::enums::ModuleType::DI | crate::models::enums::ModuleType::DO |
                crate::models::enums::ModuleType::DINone | crate::models::enums::ModuleType::DONone => {
                    // 读取布尔值
                    match crate::domain::services::plc_comm_extension::PlcServiceLegacyExt::read_bool_by_id(&plc_service, connection_id, address).await {
                        Ok(value) => {
                            // log::debug!("✅ [PLC_MONITORING] 读取成功: {} = {}", address, value);
                            values.insert(value_key, serde_json::Value::Bool(value));
                        }
                        Err(e) => {
                            log::warn!("⚠️ [PLC_MONITORING] 读取地址失败: {} - {}", address, e);
                            values.insert(value_key, serde_json::Value::Null);
                        }
                    }
                }
                crate::models::enums::ModuleType::Communication => {
                    // 通信模块，读取状态信息
                    values.insert("status".to_string(), serde_json::Value::String("connected".to_string()));
                }
                crate::models::enums::ModuleType::Other(_) => {
                    // 其他类型模块，读取通用状态
                    values.insert("status".to_string(), serde_json::Value::String("unknown".to_string()));
                }
            }
        }

        // log::debug!("📊 [PLC_MONITORING] 读取完成，共获得 {} 个值: {:?}", values.len(), values);

        // 创建监控数据
        let monitoring_data = PlcMonitoringData {
            instance_id: instance_id.to_string(),
            timestamp: chrono::Utc::now(),
            values,
        };
        
        // 发布监控数据事件到前端
        let event_payload = serde_json::json!({
            "instanceId": instance_id,
            "timestamp": monitoring_data.timestamp,
            "values": monitoring_data.values
        });

        // 发布PLC监控数据事件
        if let Err(e) = event_publisher.publish_custom("plc-monitoring-data", event_payload).await {
            log::warn!("⚠️ [PLC_MONITORING] 发布监控数据事件失败: {} - {}", instance_id, e);
        }

        // log::debug!("📊 [PLC_MONITORING] 监控数据已发布: {} 个值", monitoring_data.values.len());

        log::trace!("📊 [PLC_MONITORING] 监控数据已更新并发布: {}", instance_id);

        Ok(())
    }
    
    /// 根据地址和模块类型获取值的键名
    fn get_value_key(address: &str, module_type: &crate::models::enums::ModuleType) -> String {
        // 特殊处理：若地址以"%MD"开头，则解析数值部分
        let (addr_num_opt, is_md) = if address.starts_with("%MD") {
            (address[3..].parse::<u32>().ok(), true)
        } else {
            (address.parse::<u32>().ok(), false)
        };

        match module_type {
            crate::models::enums::ModuleType::AI | crate::models::enums::ModuleType::AINone => {
                if let Some(addr_num) = addr_num_opt {
                    if is_md {
                        // %MD 地址：按照16字节(4寄存器)一组，每+0,+4,+8,+12 分别对应 SLL/SL/SH/SHH
                        let offset = addr_num % 16;
                        return match offset {
                            0 => "sllSetPoint".to_string(),
                            4 => "slSetPoint".to_string(),
                            8 => "shSetPoint".to_string(),
                            12 => "shhSetPoint".to_string(),
                            _ => "currentValue".to_string(),
                        };
                    }
                    match addr_num {
                        40000..=41999 => {
                            "currentValue".to_string()
                        },
                        // 43000 通道报警设定值区：每4个寄存器对应一个通道的 SLL/SL/SH/SHH
                        43000..=44999 => {
                            let offset = addr_num % 4;
                            match offset {
                                0 => "sllSetPoint".to_string(),
                                1 => "slSetPoint".to_string(),
                                2 => "shSetPoint".to_string(),
                                3 => "shhSetPoint".to_string(),
                                _ => "currentValue".to_string(), // 理论不会走到这里
                            }
                        },
                        _ => "currentValue".to_string(),
                    }
                } else {
                    "currentValue".to_string()
                }
            }
            crate::models::enums::ModuleType::AO | crate::models::enums::ModuleType::AONone => {
                // AO点位监控当前输出值
                "currentOutput".to_string()
            }
            crate::models::enums::ModuleType::DI | crate::models::enums::ModuleType::DO |
            crate::models::enums::ModuleType::DINone | crate::models::enums::ModuleType::DONone => {
                // 数字量点位监控当前状态
                "currentState".to_string()
            }
            crate::models::enums::ModuleType::Communication => "status".to_string(),
            crate::models::enums::ModuleType::Other(_) => "status".to_string(),
        }
    }
}

#[async_trait]
impl BaseService for PlcMonitoringService {
    fn service_name(&self) -> &'static str {
        "PlcMonitoringService"
    }
    
    async fn initialize(&mut self) -> AppResult<()> {
        log::info!("🔧 [PLC_MONITORING] 初始化PLC监控服务");
        Ok(())
    }
    
    async fn shutdown(&mut self) -> AppResult<()> {
        log::info!("🔧 [PLC_MONITORING] 关闭PLC监控服务");
        
        // 停止所有监控任务
        let mut monitors = self.active_monitors.lock().await;
        for (instance_id, task) in monitors.drain() {
            log::info!("🛑 [PLC_MONITORING] 停止监控任务: {}", instance_id);
            let _ = task.cancel_sender.send(());
            task.task_handle.abort();
        }
        
        Ok(())
    }
    
    async fn health_check(&self) -> AppResult<()> {
        // 检查PLC服务健康状态
        self.plc_service.health_check().await
    }
}

#[async_trait]
impl IPlcMonitoringService for PlcMonitoringService {
    async fn start_monitoring(&self, request: StartPlcMonitoringRequest) -> AppResult<StartPlcMonitoringResponse> {
        log::info!("🔧 [PLC_MONITORING] 开始监控: {:?}", request);
        
        // 检查是否已在监控
        if self.is_monitoring(&request.instance_id) {
            return Ok(StartPlcMonitoringResponse {
                success: false,
                message: Some("该实例已在监控中".to_string()),
                monitoring_id: None,
            });
        }
        
        // 生成监控ID
        let monitoring_id = uuid::Uuid::new_v4().to_string();
        
        // 启动监控任务
        self.start_monitoring_task(request.clone(), monitoring_id.clone()).await?;
        
        Ok(StartPlcMonitoringResponse {
            success: true,
            message: Some("监控已启动".to_string()),
            monitoring_id: Some(monitoring_id),
        })
    }
    
    async fn stop_monitoring(&self, request: StopPlcMonitoringRequest) -> AppResult<()> {
        log::info!("🛑 [PLC_MONITORING] 停止监控: {:?}", request);
        
        let mut monitors = self.active_monitors.lock().await;
        if let Some(task) = monitors.remove(&request.instance_id) {
            let _ = task.cancel_sender.send(());
            task.task_handle.abort();
            log::info!("✅ [PLC_MONITORING] 监控已停止: {}", request.instance_id);
        } else {
            log::warn!("⚠️ [PLC_MONITORING] 未找到监控任务: {}", request.instance_id);
        }
        
        Ok(())
    }
    
    fn is_monitoring(&self, instance_id: &str) -> bool {
        // 这里需要使用try_lock或者异步版本，但trait方法是同步的
        // 简化实现，实际使用中可能需要调整
        if let Ok(monitors) = self.active_monitors.try_lock() {
            monitors.contains_key(instance_id)
        } else {
            false
        }
    }
    
    fn get_monitoring_instances(&self) -> Vec<String> {
        if let Ok(monitors) = self.active_monitors.try_lock() {
            monitors.keys().cloned().collect()
        } else {
            Vec::new()
        }
    }
}
