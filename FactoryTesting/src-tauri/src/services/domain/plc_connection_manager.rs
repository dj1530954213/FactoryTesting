use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{Mutex, RwLock};
use tokio::time::{interval, sleep};
use tokio_modbus::prelude::*;
use log::{info, warn, error, debug};
use serde::{Serialize, Deserialize};

use crate::models::test_plc_config::PlcConnectionConfig;
use crate::services::domain::test_plc_config_service::ITestPlcConfigService;
use crate::error::AppError;

/// PLC连接状态
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PlcConnectionState {
    Disconnected,
    Connecting,
    Connected,
    Reconnecting,
    Error,
}

/// PLC连接信息
#[derive(Debug, Clone)]
pub struct PlcConnection {
    pub config: PlcConnectionConfig,
    pub state: PlcConnectionState,
    pub context: Option<Arc<Mutex<tokio_modbus::client::Context>>>,
    pub last_heartbeat: Option<chrono::DateTime<chrono::Utc>>,
    pub error_message: Option<String>,
    pub reconnect_attempts: u32,
}

/// PLC连接管理器
pub struct PlcConnectionManager {
    pub connections: Arc<RwLock<HashMap<String, PlcConnection>>>,
    test_plc_config_service: Arc<dyn ITestPlcConfigService>,
    heartbeat_interval: Duration,
    reconnect_interval: Duration,
    max_reconnect_attempts: u32,
    is_running: Arc<Mutex<bool>>,
}

impl PlcConnectionManager {
    pub fn new(test_plc_config_service: Arc<dyn ITestPlcConfigService>) -> Self {
        Self {
            connections: Arc::new(RwLock::new(HashMap::new())),
            test_plc_config_service,
            heartbeat_interval: Duration::from_secs(1), // 每1秒心跳检测
            reconnect_interval: Duration::from_secs(10), // 每10秒重连尝试
            max_reconnect_attempts: 0, // 无限重连
            is_running: Arc::new(Mutex::new(false)),
        }
    }

    /// 开始连接所有启用的PLC
    pub async fn start_connections(&self) -> Result<(), AppError> {
        info!("🔗 开始连接所有启用的PLC");
        
        let mut is_running = self.is_running.lock().await;
        if *is_running {
            warn!("⚠️ PLC连接管理器已经在运行中");
            return Ok(());
        }
        *is_running = true;
        drop(is_running);

        // 获取所有PLC连接配置
        let plc_configs = self.test_plc_config_service.get_plc_connections().await?;
        
        let mut connections = self.connections.write().await;
        
        // 初始化连接
        for config in plc_configs {
            if !config.is_enabled {
                debug!("⏭️ 跳过未启用的PLC: {}", config.name);
                continue;
            }
            
            info!("🔗 初始化PLC连接: {} ({}:{})", config.name, config.ip_address, config.port);
            
            let connection = PlcConnection {
                config: config.clone(),
                state: PlcConnectionState::Disconnected,
                context: None,
                last_heartbeat: None,
                error_message: None,
                reconnect_attempts: 0,
            };
            
            connections.insert(config.id.clone(), connection);
        }
        
        drop(connections);

        // 启动连接和心跳检测任务
        self.start_connection_tasks().await;
        
        info!("✅ PLC连接管理器启动完成");
        Ok(())
    }

    /// 停止所有PLC连接
    pub async fn stop_connections(&self) -> Result<(), AppError> {
        info!("🛑 停止所有PLC连接");
        
        let mut is_running = self.is_running.lock().await;
        *is_running = false;
        drop(is_running);

        let mut connections = self.connections.write().await;
        for (id, connection) in connections.iter_mut() {
            if connection.state == PlcConnectionState::Connected {
                info!("🔌 断开PLC连接: {}", connection.config.name);
                connection.context = None;
                connection.state = PlcConnectionState::Disconnected;
            }
        }
        
        info!("✅ 所有PLC连接已停止");
        Ok(())
    }

    /// 获取所有PLC连接状态
    pub async fn get_connection_status(&self) -> HashMap<String, (PlcConnectionState, Option<String>)> {
        let connections = self.connections.read().await;
        let mut status = HashMap::new();
        
        for (id, connection) in connections.iter() {
            status.insert(
                id.clone(),
                (connection.state.clone(), connection.config.name.clone().into())
            );
        }
        
        status
    }

    /// 获取测试PLC和被测PLC的连接状态
    pub async fn get_plc_status_summary(&self) -> (bool, bool, Option<String>, Option<String>) {
        let connections = self.connections.read().await;
        
        let mut test_plc_connected = false;
        let mut target_plc_connected = false;
        let mut test_plc_name = None;
        let mut target_plc_name = None;
        
        for connection in connections.values() {
            let is_connected = connection.state == PlcConnectionState::Connected;
            
            if connection.config.is_test_plc {
                test_plc_connected = is_connected;
                test_plc_name = Some(connection.config.name.clone());
            } else {
                target_plc_connected = is_connected;
                target_plc_name = Some(connection.config.name.clone());
            }
        }
        
        (test_plc_connected, target_plc_connected, test_plc_name, target_plc_name)
    }

    /// 启动连接和心跳检测任务
    async fn start_connection_tasks(&self) {
        let connections = self.connections.clone();
        let is_running = self.is_running.clone();
        let heartbeat_interval = self.heartbeat_interval;
        let reconnect_interval = self.reconnect_interval;

        // 启动连接任务
        let connections_for_connection_task = connections.clone();
        let is_running_for_connection_task = is_running.clone();
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(1));

            loop {
                interval.tick().await;

                let running = *is_running_for_connection_task.lock().await;
                if !running {
                    break;
                }

                let connection_ids: Vec<String> = {
                    let connections_read = connections_for_connection_task.read().await;
                    connections_read.keys().cloned().collect()
                };

                for connection_id in connection_ids {
                    let connections_clone = connections_for_connection_task.clone();
                    let reconnect_interval_clone = reconnect_interval;

                    tokio::spawn(async move {
                        Self::handle_connection_task(connections_clone, connection_id, reconnect_interval_clone).await;
                    });
                }

                // 等待一段时间再检查
                sleep(Duration::from_secs(5)).await;
            }
        });

        // 启动心跳检测任务
        let connections_for_heartbeat_task = connections.clone();
        let is_running_for_heartbeat_task = is_running.clone();
        tokio::spawn(async move {
            let mut interval = interval(heartbeat_interval);

            loop {
                interval.tick().await;

                let running = *is_running_for_heartbeat_task.lock().await;
                if !running {
                    break;
                }

                Self::perform_heartbeat_check(connections_for_heartbeat_task.clone()).await;
            }
        });
    }

    /// 处理单个连接任务
    async fn handle_connection_task(
        connections: Arc<RwLock<HashMap<String, PlcConnection>>>,
        connection_id: String,
        reconnect_interval: Duration,
    ) {
        let should_connect = {
            let connections_read = connections.read().await;
            if let Some(connection) = connections_read.get(&connection_id) {
                matches!(connection.state, PlcConnectionState::Disconnected | PlcConnectionState::Reconnecting)
            } else {
                false
            }
        };
        
        if should_connect {
            Self::attempt_connection(connections, connection_id, reconnect_interval).await;
        }
    }

    /// 尝试连接PLC
    async fn attempt_connection(
        connections: Arc<RwLock<HashMap<String, PlcConnection>>>,
        connection_id: String,
        reconnect_interval: Duration,
    ) {
        let config = {
            let mut connections_write = connections.write().await;
            if let Some(connection) = connections_write.get_mut(&connection_id) {
                connection.state = PlcConnectionState::Connecting;
                connection.config.clone()
            } else {
                return;
            }
        };
        
        info!("🔗 尝试连接PLC: {} ({}:{})", config.name, config.ip_address, config.port);
        
        // 尝试建立连接
        let socket_addr = format!("{}:{}", config.ip_address, config.port);
        match socket_addr.parse::<std::net::SocketAddr>() {
            Ok(addr) => {
                match tokio_modbus::client::tcp::connect_slave(addr, Slave(1)).await {
                    Ok(mut context) => {
                        info!("✅ PLC连接成功: {}", config.name);

                        let mut connections_write = connections.write().await;
                        if let Some(connection) = connections_write.get_mut(&connection_id) {
                            connection.context = Some(Arc::new(Mutex::new(context)));
                            connection.state = PlcConnectionState::Connected;
                            connection.last_heartbeat = Some(chrono::Utc::now());
                            connection.error_message = None;
                            connection.reconnect_attempts = 0;
                        }
                    }
                    Err(e) => {
                        error!("❌ PLC连接失败: {} - {}", config.name, e);
                        
                        let mut connections_write = connections.write().await;
                        if let Some(connection) = connections_write.get_mut(&connection_id) {
                            connection.state = PlcConnectionState::Reconnecting;
                            connection.error_message = Some(e.to_string());
                            connection.reconnect_attempts += 1;
                        }
                        
                        // 等待后重试
                        sleep(reconnect_interval).await;
                    }
                }
            }
            Err(e) => {
                error!("❌ 无效的PLC地址: {} - {}", config.name, e);
                
                let mut connections_write = connections.write().await;
                if let Some(connection) = connections_write.get_mut(&connection_id) {
                    connection.state = PlcConnectionState::Error;
                    connection.error_message = Some(format!("无效地址: {}", e));
                }
            }
        }
    }

    /// 执行心跳检测
    async fn perform_heartbeat_check(connections: Arc<RwLock<HashMap<String, PlcConnection>>>) {
        let connection_ids: Vec<String> = {
            let connections_read = connections.read().await;
            connections_read.keys().cloned().collect()
        };
        
        for connection_id in connection_ids {
            let connections_clone = connections.clone();
            
            tokio::spawn(async move {
                Self::check_single_connection_heartbeat(connections_clone, connection_id).await;
            });
        }
    }

    /// 检查单个连接的心跳
    async fn check_single_connection_heartbeat(
        connections: Arc<RwLock<HashMap<String, PlcConnection>>>,
        connection_id: String,
    ) {
        let (context, config_name) = {
            let connections_read = connections.read().await;
            if let Some(connection) = connections_read.get(&connection_id) {
                if connection.state == PlcConnectionState::Connected {
                    (connection.context.clone(), connection.config.name.clone())
                } else {
                    return;
                }
            } else {
                return;
            }
        };
        
        if let Some(context_arc) = context {
            // 尝试读取线圈03001 (地址3000，因为Modbus地址从0开始)
            let heartbeat_result = {
                let mut context_guard = context_arc.lock().await;
                context_guard.read_coils(3000, 1).await
            };

            match heartbeat_result {
                Ok(_) => {
                    let mut connections_write = connections.write().await;
                    if let Some(connection) = connections_write.get_mut(&connection_id) {
                        connection.last_heartbeat = Some(chrono::Utc::now());
                        connection.error_message = None;
                    }
                }
                Err(e) => {
                    warn!("💔 PLC心跳失败: {} - {}", config_name, e);

                    let mut connections_write = connections.write().await;
                    if let Some(connection) = connections_write.get_mut(&connection_id) {
                        connection.state = PlcConnectionState::Reconnecting;
                        connection.context = None;
                        connection.error_message = Some(format!("心跳失败: {}", e));
                    }
                }
            }
        }
    }
}
