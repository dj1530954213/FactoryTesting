use super::*;
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
use crate::services::traits::{BaseService, AppResult};
use crate::services::infrastructure::{IPlcCommunicationService, event_publisher::IEventPublisher};

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
    event_publisher: Arc<dyn IEventPublisher>,
    
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
        event_publisher: Arc<dyn IEventPublisher>,
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
        
        // 启动监控任务
        let task_handle = tokio::spawn(async move {
            let mut interval = interval(Duration::from_millis(500)); // 0.5秒间隔
            
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
                            &plc_service,
                            &event_publisher,
                            &instance_id,
                            &addresses,
                            &module_type,
                        ).await {
                            log::error!("❌ [PLC_MONITORING] 监控读取失败: {} - {}", instance_id, e);
                            
                            // 发布错误事件
                            let error_event = serde_json::json!({
                                "instanceId": instance_id,
                                "error": e.to_string()
                            });
                            
                            if let Err(publish_err) = event_publisher.publish_event("plc-monitoring-error", error_event).await {
                                log::error!("❌ [PLC_MONITORING] 发布错误事件失败: {}", publish_err);
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
            
            if let Err(e) = event_publisher.publish_event("plc-monitoring-stopped", stop_event).await {
                log::error!("❌ [PLC_MONITORING] 发布停止事件失败: {}", e);
            }
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
        plc_service: &Arc<dyn IPlcCommunicationService>,
        event_publisher: &Arc<dyn IEventPublisher>,
        instance_id: &str,
        addresses: &[String],
        module_type: &crate::models::enums::ModuleType,
    ) -> AppResult<()> {
        let mut values = HashMap::new();
        
        // 读取所有地址的值
        for address in addresses {
            match module_type {
                crate::models::enums::ModuleType::AI | crate::models::enums::ModuleType::AO => {
                    // 读取浮点数值
                    match plc_service.read_f32(address).await {
                        Ok(value) => {
                            values.insert(Self::get_value_key(address, module_type), serde_json::Value::Number(serde_json::Number::from_f64(value as f64).unwrap_or_default()));
                        }
                        Err(e) => {
                            log::warn!("⚠️ [PLC_MONITORING] 读取地址失败: {} - {}", address, e);
                            values.insert(Self::get_value_key(address, module_type), serde_json::Value::Null);
                        }
                    }
                }
                crate::models::enums::ModuleType::DI | crate::models::enums::ModuleType::DO => {
                    // 读取布尔值
                    match plc_service.read_bool(address).await {
                        Ok(value) => {
                            values.insert(Self::get_value_key(address, module_type), serde_json::Value::Bool(value));
                        }
                        Err(e) => {
                            log::warn!("⚠️ [PLC_MONITORING] 读取地址失败: {} - {}", address, e);
                            values.insert(Self::get_value_key(address, module_type), serde_json::Value::Null);
                        }
                    }
                }
            }
        }
        
        // 创建监控数据
        let monitoring_data = PlcMonitoringData {
            instance_id: instance_id.to_string(),
            timestamp: chrono::Utc::now(),
            values,
        };
        
        // 发布监控数据事件
        let event_data = serde_json::to_value(&monitoring_data)?;
        event_publisher.publish_event("plc-monitoring-data", event_data).await?;
        
        Ok(())
    }
    
    /// 根据地址和模块类型获取值的键名
    fn get_value_key(address: &str, module_type: &crate::models::enums::ModuleType) -> String {
        // 这里可以根据地址类型映射到具体的键名
        // 例如：SLL设定值地址 -> "sllSetPoint"
        // 这个映射逻辑可以根据实际需求调整
        match module_type {
            crate::models::enums::ModuleType::AI => {
                if address.contains("SLL") || address.contains("sll") {
                    "sllSetPoint".to_string()
                } else if address.contains("SL") || address.contains("sl") {
                    "slSetPoint".to_string()
                } else if address.contains("SH") || address.contains("sh") {
                    "shSetPoint".to_string()
                } else if address.contains("SHH") || address.contains("shh") {
                    "shhSetPoint".to_string()
                } else {
                    "currentValue".to_string()
                }
            }
            crate::models::enums::ModuleType::AO => "currentOutput".to_string(),
            crate::models::enums::ModuleType::DI | crate::models::enums::ModuleType::DO => "currentState".to_string(),
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
