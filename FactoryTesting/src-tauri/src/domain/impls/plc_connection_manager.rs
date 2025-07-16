//! # PLC连接管理器实现模块
//!
//! ## 业务作用
//! 本模块实现了PLC连接的统一管理，提供：
//! - **连接池管理**: 统一管理多个PLC设备的连接
//! - **状态监控**: 实时监控所有PLC连接的状态
//! - **自动重连**: 连接断开时自动尝试重新连接
//! - **心跳检测**: 定期检测连接健康状态
//! - **故障恢复**: 智能的故障检测和恢复机制
//!
//! ## 设计模式
//! - **管理器模式**: 统一管理多个连接资源
//! - **状态机模式**: 连接状态的规范化管理
//! - **观察者模式**: 连接状态变化的通知机制
//! - **策略模式**: 不同的重连和恢复策略
//!
//! ## 技术特点
//! - **异步操作**: 基于tokio的异步I/O操作
//! - **并发安全**: 使用RwLock和Mutex保证线程安全
//! - **资源管理**: 智能的连接资源生命周期管理
//! - **可配置性**: 支持灵活的参数配置
//!
//! ## Rust知识点
//! - **Arc<RwLock<T>>**: 多线程共享的读写锁
//! - **async/await**: 异步编程模式
//! - **枚举状态机**: 使用枚举表示状态转换
//! - **trait对象**: 动态分发和接口抽象

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{Mutex, RwLock};
use tokio::time::{interval, sleep};
use crate::domain::services::plc_communication_service::IPlcCommunicationService;
use tokio_modbus::prelude::*;
use log::{info, warn, error, debug};
use serde::{Serialize, Deserialize};

use crate::models::test_plc_config::PlcConnectionConfig;
use crate::domain::test_plc_config_service::ITestPlcConfigService;
use crate::error::AppError;

/// PLC连接状态枚举
///
/// **业务含义**: 表示PLC连接在其生命周期中的各种状态
/// **状态转换**: 连接状态按照特定的状态机规则进行转换
///
/// **状态说明**:
/// - `Disconnected`: 未连接状态，初始状态或主动断开后的状态
/// - `Connecting`: 正在连接状态，连接建立过程中的临时状态
/// - `Connected`: 已连接状态，连接正常可用的状态
/// - `Reconnecting`: 重连状态，连接断开后尝试重新连接的状态
/// - `Error`: 错误状态，连接出现不可恢复错误的状态
///
/// **Rust知识点**:
/// - `#[derive(...)]`: 自动实现常用trait
/// - `Debug`: 支持调试输出
/// - `Clone`: 支持值的克隆
/// - `PartialEq`: 支持相等性比较
/// - `Serialize/Deserialize`: 支持序列化和反序列化
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PlcConnectionState {
    /// 未连接状态
    /// **触发条件**: 初始状态、主动断开、连接失败
    Disconnected,

    /// 正在连接状态
    /// **触发条件**: 开始建立连接时
    /// **持续时间**: 通常几秒钟，取决于网络延迟
    Connecting,

    /// 已连接状态
    /// **触发条件**: 连接成功建立且心跳正常
    /// **维持条件**: 心跳检测持续成功
    Connected,

    /// 重连状态
    /// **触发条件**: 连接断开后自动重连
    /// **重连策略**: 按照配置的间隔和次数进行重连
    Reconnecting,

    /// 错误状态
    /// **触发条件**: 出现不可恢复的错误
    /// **处理方式**: 需要人工干预或重新配置
    Error,
}

/// PLC连接信息结构体
///
/// **业务作用**: 封装单个PLC连接的完整信息和状态
/// **生命周期**: 从连接创建到销毁的整个过程
///
/// **设计理念**:
/// - **状态封装**: 将连接的所有相关信息集中管理
/// - **监控支持**: 提供丰富的状态信息用于监控和诊断
/// - **故障恢复**: 包含重连和错误恢复所需的信息
///
/// **Rust知识点**:
/// - `#[derive(Debug, Clone)]`: 自动实现调试和克隆功能
/// - `pub`: 公开字段，允许外部访问
/// - `Option<T>`: 表示可能为空的值
#[derive(Debug, Clone)]
pub struct PlcConnection {
    /// PLC连接配置
    /// **业务含义**: 包含IP地址、端口、协议参数等连接信息
    /// **不变性**: 连接建立后配置通常不会改变
    pub config: PlcConnectionConfig,

    /// 当前连接状态
    /// **业务含义**: 表示连接的实时状态
    /// **状态转换**: 根据连接事件进行状态转换
    pub state: PlcConnectionState,

    /// Modbus客户端上下文
    /// **业务含义**: 底层的Modbus通信上下文
    /// **并发安全**: Arc<Mutex<T>>确保多线程安全访问
    /// **可选性**: Option表示连接可能不存在（未连接状态）
    pub context: Option<Arc<Mutex<tokio_modbus::client::Context>>>,

    /// 最后心跳时间
    /// **业务含义**: 记录最后一次成功心跳的时间
    /// **故障检测**: 用于判断连接是否超时
    /// **时区处理**: 使用UTC时间避免时区问题
    pub last_heartbeat: Option<chrono::DateTime<chrono::Utc>>,

    /// 错误信息
    /// **业务含义**: 记录最后一次错误的详细信息
    /// **故障诊断**: 帮助定位和解决连接问题
    /// **可选性**: 正常状态下为None
    pub error_message: Option<String>,

    /// 重连尝试次数
    /// **业务含义**: 记录当前重连周期中的尝试次数
    /// **重连策略**: 用于实现重连次数限制
    /// **计数器**: 每次重连尝试后递增
    pub reconnect_attempts: u32,

    /// 连续心跳失败次数
    /// **业务含义**: 记录连续心跳失败的次数
    /// **故障判定**: 达到阈值时触发重连机制
    /// **重置条件**: 心跳成功时重置为0
    pub heart_failure_count: u32,
}

/// PLC连接管理器
///
/// **业务职责**:
/// - **连接池管理**: 统一管理多个PLC设备的连接
/// - **状态监控**: 实时监控所有连接的健康状态
/// - **自动恢复**: 实现连接断开后的自动重连机制
/// - **资源优化**: 合理分配和回收连接资源
///
/// **设计模式**:
/// - **单例模式**: 全局唯一的连接管理器实例
/// - **工厂模式**: 统一创建和配置连接
/// - **观察者模式**: 监控连接状态变化
///
/// **并发安全**:
/// - 使用Arc<RwLock<T>>实现多线程安全的连接池
/// - 使用Arc<Mutex<T>>保护共享状态
/// - 支持高并发的连接操作
pub struct PlcConnectionManager {
    /// 连接池
    /// **数据结构**: HashMap<连接ID, 连接信息>
    /// **并发控制**: RwLock支持多读单写
    /// **共享访问**: Arc允许多线程共享
    pub connections: Arc<RwLock<HashMap<String, PlcConnection>>>,

    /// PLC配置服务
    /// **业务依赖**: 用于获取PLC连接配置信息
    /// **接口抽象**: 通过trait对象实现依赖注入
    test_plc_config_service: Arc<dyn ITestPlcConfigService>,

    /// 心跳检测间隔
    /// **业务含义**: 定期检测连接健康状态的时间间隔
    /// **性能平衡**: 间隔太短影响性能，太长影响故障检测及时性
    heartbeat_interval: Duration,

    /// 重连间隔
    /// **业务含义**: 连接断开后重新尝试连接的时间间隔
    /// **避免频繁重连**: 防止对PLC设备造成过大压力
    reconnect_interval: Duration,

    /// 最大重连尝试次数
    /// **业务含义**: 单次重连周期中的最大尝试次数
    /// **故障处理**: 0表示无限重连，非0表示有限次数
    max_reconnect_attempts: u32,

    /// 运行状态标志
    /// **业务含义**: 标识管理器是否正在运行
    /// **并发控制**: Mutex确保状态变更的原子性
    /// **生命周期**: 控制心跳和重连任务的启停
    is_running: Arc<Mutex<bool>>,
}

impl PlcConnectionManager {
    /// 根据连接ID获取端点地址字符串
    ///
    /// **业务作用**: 为外部模块提供PLC设备的网络地址信息
    /// **使用场景**: 日志记录、错误报告、监控显示等
    ///
    /// **实现逻辑**:
    /// 1. 获取连接池的读锁
    /// 2. 查找指定ID的连接信息
    /// 3. 格式化IP地址和端口为字符串
    ///
    /// **参数**: `connection_id` - 连接的唯一标识符
    /// **返回值**: `Option<String>` - 格式为"IP:端口"的地址字符串，不存在时返回None
    ///
    /// **并发安全**: 使用读锁，支持多线程并发访问
    pub async fn endpoint_by_id(&self, connection_id: &str) -> Option<String> {
        let conns = self.connections.read().await; // 获取读锁
        conns.get(connection_id).map(|c| format!("{}:{}", c.config.ip_address, c.config.port))
    }

    /// 创建新的PLC连接管理器实例
    ///
    /// **业务作用**: 初始化连接管理器的所有组件和配置
    /// **设计模式**: 构造器模式，通过参数注入依赖服务
    ///
    /// **默认配置**:
    /// - 心跳间隔: 1秒 - 平衡及时性和性能
    /// - 重连间隔: 10秒 - 避免频繁重连对设备的冲击
    /// - 重连次数: 无限 - 确保连接的持久性
    /// - 初始状态: 未运行 - 需要显式启动
    ///
    /// **参数**: `test_plc_config_service` - PLC配置服务的trait对象
    /// **返回值**: 新的连接管理器实例
    ///
    /// **Rust知识点**:
    /// - `Arc::new()`: 创建原子引用计数智能指针
    /// - `RwLock::new()`: 创建读写锁
    /// - `HashMap::new()`: 创建空的哈希映射
    /// - `Duration::from_secs()`: 从秒数创建时间间隔
    pub fn new(test_plc_config_service: Arc<dyn ITestPlcConfigService>) -> Self {
        Self {
            connections: Arc::new(RwLock::new(HashMap::new())), // 初始化空连接池
            test_plc_config_service,                            // 注入配置服务
            heartbeat_interval: Duration::from_secs(1),         // 每1秒心跳检测
            reconnect_interval: Duration::from_secs(10),        // 每10秒重连尝试
            max_reconnect_attempts: 0,                          // 无限重连
            is_running: Arc::new(Mutex::new(false)),            // 初始状态为未运行
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

            // 调用全局 PLC 服务建立连接，确保句柄注册到 default_handles
            use crate::domain::services::plc_communication_service::{PlcConnectionConfig as ServicePlcConfig, PlcProtocol};
            use std::collections::HashMap;
            let svc_cfg = ServicePlcConfig {
                id: config.id.clone(),
                name: config.name.clone(),
                protocol: PlcProtocol::ModbusTcp,
                host: config.ip_address.clone(),
                port: config.port as u16,
                timeout_ms: config.timeout as u64,
                read_timeout_ms: config.timeout as u64,
                write_timeout_ms: config.timeout as u64,
                retry_count: config.retry_count as u32,
                retry_interval_ms: 500,
                byte_order: config.byte_order.clone(),
                zero_based_address: config.zero_based_address,
                protocol_params: HashMap::new(),
            };
            let plc_service = crate::infrastructure::plc_communication::global_plc_service();
            let mut connection_state = PlcConnectionState::Disconnected;
            match plc_service.connect(&svc_cfg).await {
                Ok(handle) => {
                    info!("✅ PLC连接成功: {} → connection_id={}", config.name, handle.connection_id);
                    connection_state = PlcConnectionState::Connected;
                },
                Err(err) => {
                    error!("❌ PLC连接失败: {} - {}", config.name, err);
                }
            }
            let connection = PlcConnection {
                config: config.clone(),
                state: connection_state,
                context: None,
                last_heartbeat: None,
                error_message: None,
                reconnect_attempts: 0,
                heart_failure_count: 0,
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

    /// 等待首次所有可用PLC建立连接（至少一台 Connected 且有 context）
    async fn wait_for_initial_connections(&self, max_wait: Duration) {
        let start = Instant::now();
        loop {
            {
                let connections = self.connections.read().await;
                let ready = connections.values().any(|c| c.context.is_some());
                if ready {
                    info!("✅ 首次PLC连接已就绪，开始心跳检测");
                    return;
                }
            }
            if start.elapsed() >= max_wait {
                warn!("⌛ 等待首次PLC连接超时，继续启动心跳检测");
                return;
            }
            sleep(Duration::from_millis(100)).await;
        }
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
                sleep(Duration::from_secs(2)).await;
            }
        });

        // 等待首次连接完成（最多3秒）
        self.wait_for_initial_connections(Duration::from_secs(3)).await;


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
        //debug!("🔍 执行心跳检测任务，对 {} 个连接", connection_ids.len());
        
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
        let (context, config_name, current_state) = {
            let connections_read = connections.read().await;
            if let Some(connection) = connections_read.get(&connection_id) {
                (
                    connection.context.clone(),
                    connection.config.name.clone(),
                    connection.state.clone(),
                )
            } else {
                return;
            }
        };
        
        if let Some(context_arc) = context {
            // 尝试读取线圈03001 (地址3000，因为Modbus地址从0开始)
            //debug!("↪️ 发送心跳读线圈请求: {}", config_name);
            let heartbeat_result = {
                let mut context_guard = context_arc.lock().await;
                context_guard.read_coils(3000, 1).await
            };

            match heartbeat_result {
                Ok(_) => {
                    //debug!("✅ PLC心跳成功: {}", config_name);
                    let mut connections_write = connections.write().await;
                    if let Some(connection) = connections_write.get_mut(&connection_id) {
                        connection.last_heartbeat = Some(chrono::Utc::now());
                        connection.error_message = None;
                        connection.heart_failure_count = 0;
                        if connection.state != PlcConnectionState::Connected {
                            debug!("🔄 状态修正: {} -> Connected", config_name);
                            connection.state = PlcConnectionState::Connected;
                            connection.reconnect_attempts = 0;
                        }
                    }
                }
                Err(e) => {
                    warn!("💔 PLC心跳失败: {} - {}", config_name, e);

                    let mut connections_write = connections.write().await;
                    if let Some(connection) = connections_write.get_mut(&connection_id) {
                        connection.error_message = Some(format!("心跳失败: {}", e));
                        connection.heart_failure_count += 1;
                        if connection.heart_failure_count >= 3 {
                            warn!("🔄 连续心跳失败达到阈值，切换为 Reconnecting: {}", config_name);
                            connection.state = PlcConnectionState::Reconnecting;
                            connection.context = None;
                            connection.heart_failure_count = 0;
                        }
                    }
                }
            }
        } else {
            // 无有效 context，无法执行心跳
            warn!("⚠️ PLC心跳跳过: {} 无 Modbus context", config_name);
            let mut connections_write = connections.write().await;
            if let Some(connection) = connections_write.get_mut(&connection_id) {
                connection.heart_failure_count += 1;
                if connection.heart_failure_count >= 3 {
                    warn!("🔄 连续缺失 context 达到阈值，切换为 Reconnecting: {}", config_name);
                    connection.state = PlcConnectionState::Reconnecting;
                    connection.error_message = Some("Modbus context lost".to_string());
                    connection.heart_failure_count = 0;
                }
            }
        }
    }
}
