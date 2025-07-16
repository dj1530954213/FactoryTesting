//! # PLC通信服务实现模块
//!
//! ## 业务作用
//! 本模块是工厂测试系统中PLC通信的核心基础设施层实现，负责：
//! - 提供统一的PLC通信接口，支持Modbus TCP协议
//! - 管理PLC连接池，实现连接复用和资源优化
//! - 处理PLC数据的读写操作，支持多种数据类型
//! - 提供连接状态监控和故障恢复机制
//! - 实现异步非阻塞的通信模式，提高系统性能
//!
//! ## 架构设计
//! 采用分层架构设计：
//! - **连接池层**: `ModbusTcpConnectionPool` 管理底层TCP连接
//! - **服务层**: `ModbusTcpPlcService` 提供高级业务接口
//! - **全局管理**: 通过单例模式提供全局访问点
//!
//! ## 主要组件
//! - `ModbusTcpConnectionPool`: TCP连接池管理器
//! - `ModbusTcpPlcService`: PLC通信服务主体
//! - `ModbusTcpConnection`: 单个连接的封装
//! - 全局单例实例管理
//!
//! ## Rust知识点
//! - **OnceCell**: 线程安全的延迟初始化单例模式
//! - **Arc**: 原子引用计数，实现多线程共享所有权
//! - **RwLock/Mutex**: 读写锁和互斥锁，保证并发安全
//! - **async/await**: 异步编程模型，避免阻塞
//! - **trait对象**: 动态分发，实现接口抽象

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{Mutex, RwLock};
use tokio::time::{timeout, sleep};
use tokio_modbus::prelude::*;
use std::str::FromStr;
use uuid::Uuid;
use chrono::{DateTime, Utc};

use crate::domain::services::{
    BaseService,
    PlcConnectionConfig, PlcProtocol, ConnectionHandle,
    ReadRequest, WriteRequest, ReadResult, WriteResult,
    PlcDataType, PlcValue, ConnectionStats, ConnectionTestResult
};
use crate::utils::error::{AppError, AppResult};

// 复用领域层定义的通信服务接口，避免重复定义造成类型不一致
pub use crate::domain::services::plc_communication_service::IPlcCommunicationService;

use once_cell::sync::OnceCell;
use crate::domain::impls::plc_connection_manager::PlcConnectionManager;

/// 全局唯一的 ModbusTcpPlcService 实例
///
/// **Rust知识点 - OnceCell单例模式**:
/// - OnceCell提供线程安全的延迟初始化
/// - 只能被设置一次，后续访问返回相同实例
/// - 避免了传统单例模式的复杂性和竞态条件
/// - Arc<T>提供多线程共享所有权，引用计数自动管理内存
static GLOBAL_PLC_SERVICE: OnceCell<Arc<ModbusTcpPlcService>> = OnceCell::new();

/// 全局 PLC 连接管理器（供外层服务/命令查询端点信息等）
///
/// **业务作用**:
/// - 为上层应用提供PLC连接信息查询接口
/// - 统一管理所有PLC连接的生命周期
/// - 支持连接状态监控和故障诊断
static GLOBAL_PLC_MANAGER: OnceCell<Arc<PlcConnectionManager>> = OnceCell::new();

/// 获取全局 PLC 服务单例
///
/// **业务作用**: 为整个应用提供统一的PLC通信服务访问点
/// **Rust知识点**:
/// - `get_or_init()`: 线程安全的延迟初始化，首次调用时创建实例
/// - `clone()`: 克隆Arc智能指针，增加引用计数但不复制数据
/// - 单例模式确保全局只有一个PLC服务实例，避免资源冲突
pub fn global_plc_service() -> Arc<ModbusTcpPlcService> {
    GLOBAL_PLC_SERVICE
        .get_or_init(|| Arc::new(ModbusTcpPlcService::default()))
        .clone()
}

/// 设置全局 PLC 连接管理器（仅允许设置一次）
///
/// **业务作用**: 在应用启动时注册PLC连接管理器
/// **参数**: `mgr` - PLC连接管理器的Arc智能指针
/// **Rust知识点**:
/// - `let _ = `: 忽略返回值，OnceCell::set()返回Result但我们不关心失败情况
/// - 只能设置一次，后续调用会被忽略，保证管理器的唯一性
pub fn set_global_plc_manager(mgr: Arc<PlcConnectionManager>) {
    let _ = GLOBAL_PLC_MANAGER.set(mgr);
}

/// 获取全局 PLC 连接管理器
///
/// **业务作用**: 为上层服务提供PLC连接管理器的访问接口
/// **返回值**: `Option<Arc<PlcConnectionManager>>` - 可能为空的管理器引用
/// **Rust知识点**:
/// - `Option<T>`: 表示可能存在或不存在的值，避免空指针异常
/// - `cloned()`: 对Option内的Arc进行克隆，增加引用计数
pub fn get_global_plc_manager() -> Option<Arc<PlcConnectionManager>> {
    GLOBAL_PLC_MANAGER.get().cloned()
}

/// Modbus TCP连接池管理器
///
/// **业务作用**:
/// - 管理多个PLC设备的TCP连接，实现连接复用
/// - 维护连接配置和统计信息
/// - 提供连接的创建、获取、销毁等生命周期管理
/// - 支持连接健康检查和自动重连
///
/// **设计模式**: 对象池模式 - 预创建和复用昂贵的连接资源
///
/// **Rust知识点**:
/// - `#[derive(Debug)]`: 自动实现Debug trait，支持调试输出
/// - `Arc<RwLock<T>>`: 多线程共享的读写锁，支持多读单写
/// - `Arc<Mutex<T>>`: 多线程共享的互斥锁，保证独占访问
#[derive(Debug)]
pub struct ModbusTcpConnectionPool {
    /// 活动连接映射表
    /// **业务含义**: 以连接ID为键，存储所有活跃的PLC连接
    /// **并发安全**: RwLock允许多个线程同时读取，但写入时独占
    connections: Arc<RwLock<HashMap<String, ModbusTcpConnection>>>,

    /// 连接配置映射表
    /// **业务含义**: 存储每个连接的配置信息，用于重连和验证
    /// **数据一致性**: 与connections保持同步，确保配置和连接的对应关系
    configs: Arc<RwLock<HashMap<String, PlcConnectionConfig>>>,

    /// 全局统计信息
    /// **业务含义**: 记录连接池的运行状态和性能指标
    /// **线程安全**: Mutex确保统计数据的原子性更新
    global_stats: Arc<Mutex<GlobalConnectionStats>>,
}

/// 单个Modbus TCP连接的封装
///
/// **业务作用**:
/// - 封装单个PLC设备的TCP连接和相关配置
/// - 维护连接状态和统计信息
/// - 提供连接级别的配置参数（字节序、地址模式等）
/// - 支持连接健康监控和心跳检测
///
/// **设计模式**:
/// - 装饰器模式：在原始TCP连接基础上添加业务功能
/// - 状态模式：通过多个状态字段管理连接生命周期
///
/// **Rust知识点**:
/// - `#[derive(Debug, Clone)]`: 自动实现调试和克隆功能
/// - `Clone`: 允许连接对象的浅拷贝，Arc确保底层数据共享
#[derive(Debug, Clone)]
struct ModbusTcpConnection {
    /// 字节顺序配置
    /// **业务含义**: 定义多字节数据的存储顺序（大端/小端）
    /// **重要性**: 不同PLC厂商可能使用不同的字节序，影响数据解析正确性
    byte_order: crate::models::ByteOrder,

    /// 地址是否从0开始
    /// **业务含义**: 某些PLC地址从1开始，需要进行地址偏移转换
    /// **兼容性**: 支持不同PLC厂商的地址编码方式
    zero_based_address: bool,

    /// 连接句柄
    /// **业务含义**: 连接的唯一标识符和元数据
    /// **生命周期**: 贯穿整个连接的生命周期，用于追踪和管理
    handle: ConnectionHandle,

    /// Modbus客户端上下文
    /// **业务含义**: 底层Modbus协议的通信上下文
    /// **并发安全**: Arc<Mutex<Option<T>>>模式，支持多线程安全访问
    /// **可选性**: Option表示连接可能断开，需要重新建立
    /// **Rust知识点**: 三层包装 - Arc(共享) + Mutex(互斥) + Option(可空)
    context: Arc<Mutex<Option<tokio_modbus::client::Context>>>,

    /// 连接状态标志
    /// **业务含义**: 标识当前连接是否可用
    /// **原子性**: 通过Mutex保证状态更新的原子性
    /// **性能考虑**: 避免频繁的连接测试，通过缓存状态提高效率
    is_connected: Arc<Mutex<bool>>,

    /// 连接统计信息
    /// **业务含义**: 记录连接的使用情况和性能指标
    /// **监控价值**: 用于系统监控、故障诊断和性能优化
    /// **数据一致性**: Mutex确保统计数据的准确性
    stats: Arc<Mutex<ConnectionStats>>,

    /// 最后心跳时间
    /// **业务含义**: 记录最后一次成功通信的时间
    /// **故障检测**: 用于判断连接是否超时，触发重连机制
    /// **时间精度**: 使用UTC时间避免时区问题
    last_heartbeat: Arc<Mutex<DateTime<Utc>>>,
}

/// 全局连接统计信息
///
/// **业务作用**:
/// - 提供连接池级别的统计数据
/// - 支持系统监控和性能分析
/// - 帮助诊断连接问题和优化配置
///
/// **Rust知识点**:
/// - `#[derive(Debug, Default)]`: 自动实现调试输出和默认值初始化
/// - `Default`: 提供结构体的默认值，所有字段初始化为0
#[derive(Debug, Default)]
struct GlobalConnectionStats {
    /// 总连接数（累计创建的连接数量）
    total_connections: u64,
    /// 当前活跃连接数
    active_connections: u64,
    /// 失败连接数（累计连接失败次数）
    failed_connections: u64,
    /// 总操作数（累计执行的读写操作数量）
    total_operations: u64,
    /// 成功操作数（累计成功的读写操作数量）
    successful_operations: u64,
}

impl ModbusTcpConnectionPool {
    /// 创建新的连接池实例
    ///
    /// **业务作用**: 初始化连接池的所有组件
    /// **返回值**: 新的连接池实例，所有集合都为空
    /// **Rust知识点**:
    /// - `Self`: 指代当前类型，等价于ModbusTcpConnectionPool
    /// - `Arc::new()`: 创建原子引用计数智能指针
    /// - `HashMap::new()`: 创建空的哈希映射表
    pub fn new() -> Self {
        Self {
            connections: Arc::new(RwLock::new(HashMap::new())),
            configs: Arc::new(RwLock::new(HashMap::new())),
            global_stats: Arc::new(Mutex::new(GlobalConnectionStats::default())),
        }
    }

    /// 获取或创建PLC连接
    ///
    /// **业务逻辑**:
    /// 1. 首先检查是否存在有效的现有连接
    /// 2. 如果连接存在且状态正常，直接返回
    /// 3. 否则创建新的连接并缓存
    ///
    /// **性能优化**: 连接复用避免频繁建立TCP连接的开销
    /// **错误处理**: 连接失败时返回详细的错误信息
    ///
    /// **参数**: `config` - PLC连接配置
    /// **返回值**: `AppResult<Arc<ModbusTcpConnection>>` - 连接的共享引用
    async fn get_or_create_connection(&self, config: &PlcConnectionConfig) -> AppResult<Arc<ModbusTcpConnection>> {
        let connections = self.connections.read().await; // 获取读锁，允许并发读取

        // 检查是否已有连接
        // **业务逻辑**: 优先使用现有连接，避免重复创建
        if let Some(conn) = connections.get(&config.id) {
            // 检查连接是否仍然有效
            // **状态验证**: 确保返回的连接确实可用
            if *conn.is_connected.lock().await {
                return Ok(Arc::new(conn.clone())); // 返回现有连接的克隆
            }
        }

        drop(connections); // 显式释放读锁，避免死锁

        // 创建新连接
        // **故障恢复**: 当现有连接失效时，自动创建新连接
        self.create_new_connection(config).await
    }

    /// 创建新的Modbus TCP连接
    ///
    /// **业务流程**:
    /// 1. 验证协议类型是否支持
    /// 2. 解析和验证网络地址
    /// 3. 获取Modbus从站ID
    /// 4. 建立TCP连接
    /// 5. 执行初始心跳测试
    /// 6. 创建连接对象并缓存
    ///
    /// **错误处理**: 每个步骤都有详细的错误处理和日志记录
    /// **超时控制**: 使用tokio::time::timeout防止连接挂起
    async fn create_new_connection(&self, config: &PlcConnectionConfig) -> AppResult<Arc<ModbusTcpConnection>> {
        // 协议类型验证
        // **业务规则**: 当前实现只支持Modbus TCP协议
        if config.protocol != PlcProtocol::ModbusTcp {
            return Err(AppError::configuration_error(
                format!("不支持的协议类型: {:?}", config.protocol)
            ));
        }

        // 解析网络地址
        // **数据验证**: 确保地址格式正确，避免运行时错误
        let socket_addr = format!("{}:{}", config.host, config.port)
            .parse::<std::net::SocketAddr>()  // 解析为SocketAddr类型
            .map_err(|e| AppError::configuration_error(
                format!("无效的地址格式: {}:{}, 错误: {}", config.host, config.port, e)
            ))?;

        // 获取Modbus从站ID
        // **协议参数**: 从配置的协议参数中提取从站ID
        // **默认值**: 如果未配置，使用默认值1
        // **Rust知识点**: 链式调用 - get() -> and_then() -> unwrap_or()
        let slave_id = config.protocol_params
            .get("slave_id")                    // 获取参数值
            .and_then(|v| v.as_u64())          // 转换为u64类型
            .unwrap_or(1) as u8;               // 默认值1，转换为u8

        let slave = Slave(slave_id); // 创建Modbus从站对象

        // 建立TCP连接
        // **超时控制**: 防止连接操作无限期阻塞
        // **异步操作**: 使用tokio的异步TCP连接
        let mut context = timeout(
            Duration::from_millis(config.timeout_ms), // 连接超时时间
            tcp::connect_slave(socket_addr, slave)     // 异步连接操作
        ).await
        .map_err(|_| AppError::timeout_error("PLC连接", "连接超时"))? // 超时错误处理
        .map_err(|e| AppError::plc_communication_error(
            format!("Modbus连接失败: {}", e)
        ))?; // 连接失败错误处理

        // 初次连接后立即验证心跳（读取线圈 03001 / 地址3000）
        // **连接验证**: 确保连接不仅建立成功，而且PLC响应正常
        // **业务规则**: 使用标准的心跳地址进行连接测试
        // **地址说明**: 3000对应Modbus地址03001（线圈地址）
        if let Err(e) = context.read_coils(3000, 1).await {
            log::warn!("初次心跳失败，连接视为无效: {}:{} - {}", config.host, config.port, e);
            return Err(AppError::plc_communication_error(format!("初次心跳失败: {}", e)));
        }

        // 创建连接句柄
        // **唯一标识**: 每个连接都有唯一的句柄ID
        // **时间戳**: 记录连接创建和最后活动时间
        let handle = ConnectionHandle {
            connection_id: config.id.clone(),           // 连接配置ID
            handle_id: Uuid::new_v4().to_string(),     // 唯一句柄ID
            protocol: config.protocol,                  // 协议类型
            created_at: Utc::now(),                     // 创建时间
            last_activity: Utc::now(),                  // 最后活动时间
        };

        // 创建连接统计信息
        // **性能监控**: 记录连接的使用情况和性能指标
        // **故障诊断**: 错误计数帮助识别连接问题
        let stats = ConnectionStats {
            connection_id: config.id.clone(),          // 连接ID
            total_reads: 0,                            // 总读取次数
            total_writes: 0,                           // 总写入次数
            successful_reads: 0,                       // 成功读取次数
            successful_writes: 0,                      // 成功写入次数
            average_read_time_ms: 0.0,                 // 平均读取时间（毫秒）
            average_write_time_ms: 0.0,                // 平均写入时间（毫秒）
            connection_established_at: Utc::now(),     // 连接建立时间
            last_communication: Utc::now(),            // 最后通信时间
            connection_errors: 0,                      // 连接错误计数
        };

        // 创建连接对象
        // **配置解析**: 将字符串配置转换为枚举类型
        // **默认值处理**: 解析失败时使用默认字节序
        let byte_order_enum = crate::models::ByteOrder::from_str(&config.byte_order).unwrap_or_default();
        let connection = ModbusTcpConnection {
            handle: handle.clone(),                     // 连接句柄
            context: Arc::new(Mutex::new(Some(context))), // Modbus上下文（已连接）
            is_connected: Arc::new(Mutex::new(true)),   // 连接状态（初始为已连接）
            stats: Arc::new(Mutex::new(stats)),         // 统计信息
            last_heartbeat: Arc::new(Mutex::new(Utc::now())), // 最后心跳时间
            byte_order: byte_order_enum,                // 字节序配置
            zero_based_address: config.zero_based_address, // 地址模式配置
        };

        // 存储连接和配置到连接池
        // **写锁**: 获取写锁进行数据更新
        // **数据一致性**: 同时更新连接和配置映射表
        let mut connections = self.connections.write().await;
        let mut configs = self.configs.write().await;

        connections.insert(config.id.clone(), connection.clone()); // 存储连接对象
        configs.insert(config.id.clone(), config.clone());         // 存储配置对象

        // 启动心跳探活与自动重连任务
        // **业务目的**:
        // - 定期检测PLC连接状态，确保连接可用性
        // - 在连接断开时自动重连，提高系统可靠性
        // - 同步连接状态到领域层管理器
        //
        // **技术实现**:
        // - 使用tokio::spawn创建独立的异步任务
        // - 1秒间隔执行心跳检测（读取地址03001）
        // - 失败时自动尝试重新建立连接
        {
            let connections_map = self.connections.clone(); // 克隆连接映射表的Arc引用
            let configs_map = self.configs.clone();         // 克隆配置映射表的Arc引用
            let conn_id = config.id.clone();               // 克隆连接ID

            // **Rust知识点**: tokio::spawn创建独立的异步任务
            // 任务在后台运行，不会阻塞当前函数的返回
            tokio::spawn(async move {
                let interval = Duration::from_millis(1000); // 心跳间隔：1秒

                // **无限循环**: 持续监控连接状态直到连接被移除
                loop {
                    sleep(interval).await; // 等待心跳间隔

                    // 检查连接是否仍然存在于连接池中
                    // **生命周期管理**: 连接被移除时，心跳任务自动结束
                    let conn_opt = {
                        let conns = connections_map.read().await; // 获取读锁
                        conns.get(&conn_id).cloned()              // 克隆连接对象
                    }; // 读锁在此处自动释放

                    if conn_opt.is_none() {
                        break; // 连接已被移除，结束心跳任务
                    }
                    let conn = conn_opt.unwrap(); // 安全解包，因为已检查非空

                    // 执行心跳读取操作
                    // **心跳机制**: 读取线圈地址03001（内部地址3000）
                    // **故障检测**: 通过读取操作的成功与否判断连接状态
                    let heartbeat_ok = {
                        let mut ctx_guard = conn.context.lock().await; // 获取上下文互斥锁
                        if let Some(ctx) = ctx_guard.as_mut() {
                            // 尝试读取1个线圈，地址3000对应Modbus地址03001
                            ctx.read_coils(3000, 1).await.is_ok()
                        } else {
                            false // 上下文不存在，视为心跳失败
                        }
                    }; // 上下文锁在此处自动释放

                    // 心跳成功的处理逻辑
                    if heartbeat_ok {
                        // 更新连接状态为已连接
                        *conn.is_connected.lock().await = true;
                        // 更新最后心跳时间
                        *conn.last_heartbeat.lock().await = Utc::now();

                        // 同步状态到领域层连接管理器
                        // **状态同步**: 确保基础设施层和领域层状态一致
                        if let Some(mgr) = crate::infrastructure::plc_communication::get_global_plc_manager() {
                            let mut mgr_conns = mgr.connections.write().await;
                            if let Some(mgr_conn) = mgr_conns.get_mut(&conn_id) {
                                mgr_conn.state = crate::domain::impls::plc_connection_manager::PlcConnectionState::Connected;
                                mgr_conn.last_heartbeat = Some(Utc::now());
                                mgr_conn.error_message = None;           // 清除错误信息
                                mgr_conn.heart_failure_count = 0;       // 重置失败计数
                            }
                        }
                        continue; // 心跳成功，继续下一轮检测
                    }

                    // 心跳失败，尝试重连
                    *conn.is_connected.lock().await = false;

                    let cfg_opt = {
                        let cfgs = configs_map.read().await;
                        cfgs.get(&conn_id).cloned()
                    };
                    // 同步失败状态到 Domain 层
                    if let Some(mgr) = crate::infrastructure::plc_communication::get_global_plc_manager() {
                        let mut mgr_conns = mgr.connections.write().await;
                        if let Some(mgr_conn) = mgr_conns.get_mut(&conn_id) {
                            mgr_conn.state = crate::domain::impls::plc_connection_manager::PlcConnectionState::Reconnecting;
                            mgr_conn.error_message = Some("Heartbeat failed, reconnecting".to_string());
                        }
                    }

                    if let Some(cfg) = cfg_opt {
                        if let Ok(socket_addr) = format!("{}:{}", cfg.host, cfg.port).parse::<std::net::SocketAddr>() {
                            let slave_id = cfg.protocol_params
                                .get("slave_id")
                                .and_then(|v| v.as_u64())
                                .unwrap_or(1) as u8;
                            let slave = Slave(slave_id);
                            if let Ok(new_ctx) = tokio_modbus::client::tcp::connect_slave(socket_addr, slave).await {
                                let mut ctx_guard = conn.context.lock().await;
                                *ctx_guard = Some(new_ctx);
                                *conn.is_connected.lock().await = true;
                                *conn.last_heartbeat.lock().await = Utc::now();
                            }
                        }
                    }
                }
            });
        }

        // 更新全局统计
        let mut global_stats = self.global_stats.lock().await;
        global_stats.total_connections += 1;
        global_stats.active_connections += 1;

        log::info!("成功创建Modbus TCP连接: {} -> {}:{}", config.id, config.host, config.port);

        Ok(Arc::new(connection))
    }

    /// 移除连接
    async fn remove_connection(&self, connection_id: &str) -> AppResult<()> {
        let mut connections = self.connections.write().await;
        let mut configs = self.configs.write().await;

        if let Some(conn) = connections.remove(connection_id) {
            // 标记为断开
            *conn.is_connected.lock().await = false;

            // 关闭上下文
            let mut context = conn.context.lock().await;
            if let Some(ctx) = context.take() {
                drop(ctx); // 关闭连接
            }

            // 更新全局统计
            let mut global_stats = self.global_stats.lock().await;
            global_stats.active_connections = global_stats.active_connections.saturating_sub(1);

            log::info!("已移除Modbus TCP连接: {}", connection_id);
        }

        configs.remove(connection_id);

        Ok(())
    }

    /// 获取连接
    async fn get_connection(&self, handle: &ConnectionHandle) -> AppResult<Arc<ModbusTcpConnection>> {
        let connections = self.connections.read().await;

        connections.get(&handle.connection_id)
            .cloned()
            .map(Arc::new)
            .ok_or_else(|| AppError::not_found_error(
                "PLC连接",
                format!("连接不存在: {}", handle.connection_id)
            ))
    }
}

/// Modbus TCP PLC通信服务
#[derive(Debug)]
pub struct ModbusTcpPlcService {
    /// 连接池
    pool: ModbusTcpConnectionPool,

    /// 服务状态
    is_initialized: Arc<Mutex<bool>>,
    /// 多默认连接句柄映射，key 为连接ID
    default_handles: Arc<Mutex<HashMap<String, ConnectionHandle>>>,
    /// 向后兼容的最后一次默认连接句柄
    default_handle: Arc<Mutex<Option<ConnectionHandle>>>,
    /// 最后一次成功建立的默认连接配置（用于日志）
    last_default_config: Arc<Mutex<Option<PlcConnectionConfig>>>,
}

impl ModbusTcpPlcService {
    /// 返回最后一次成功建立的默认连接地址，如 127.0.0.1:502
    pub async fn last_default_address(&self) -> Option<String> {
        let guard = self.last_default_config.lock().await;
        guard.as_ref().map(|c| format!("{}:{}", c.host, c.port))
    }
    /// 创建新的服务实例
    pub fn new() -> Self {
        Self {
            pool: ModbusTcpConnectionPool::new(),
            is_initialized: Arc::new(Mutex::new(false)),
            default_handles: Arc::new(Mutex::new(HashMap::new())),
            default_handle: Arc::new(Mutex::new(None)),
            last_default_config: Arc::new(Mutex::new(None)),
        }
    }
}

impl Default for ModbusTcpPlcService {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl BaseService for ModbusTcpPlcService {
    fn service_name(&self) -> &'static str {
        "ModbusTcpPlcService"
    }

    async fn initialize(&mut self) -> AppResult<()> {
        let mut is_initialized = self.is_initialized.lock().await;
        if *is_initialized {
            return Ok(());
        }

        log::info!("初始化Modbus TCP PLC通信服务");

        *is_initialized = true;
        Ok(())
    }

    async fn shutdown(&mut self) -> AppResult<()> {
        let mut is_initialized = self.is_initialized.lock().await;
        if !*is_initialized {
            return Ok(());
        }

        log::info!("关闭Modbus TCP PLC通信服务");

        // 关闭所有连接
        let connections = self.pool.connections.read().await;
        let connection_ids: Vec<String> = connections.keys().cloned().collect();
        drop(connections);

        for connection_id in connection_ids {
            if let Err(e) = self.pool.remove_connection(&connection_id).await {
                log::warn!("关闭连接时出错 {}: {}", connection_id, e);
            }
        }

        *is_initialized = false;
        Ok(())
    }

    async fn health_check(&self) -> AppResult<()> {
        let is_initialized = self.is_initialized.lock().await;
        if !*is_initialized {
            return Err(AppError::service_initialization_error("ModbusTcpPlcService", "服务未初始化"));
        }

        // 检查活动连接数
        let global_stats = self.pool.global_stats.lock().await;
        log::debug!("PLC服务健康检查: 活动连接数 = {}", global_stats.active_connections);

        Ok(())
    }
}

#[async_trait::async_trait]
impl IPlcCommunicationService for ModbusTcpPlcService {
    async fn connect(&self, config: &PlcConnectionConfig) -> AppResult<ConnectionHandle> {
        log::info!("连接到PLC: {} ({}:{})", config.name, config.host, config.port);

        let connection = self.pool.get_or_create_connection(config).await?;

        // 保存/更新连接句柄映射
        {
            // 1. 更新多连接句柄映射（始终保持最新）
            let mut map = self.default_handles.lock().await;
            map.insert(config.id.clone(), connection.handle.clone());

            // 2. 仅当当前还没有默认句柄时，才设置向后兼容的 default_handle，
            //    避免后续新的连接（如手动测试用连接）覆盖业务逻辑正在使用的默认连接。
            let mut guard = self.default_handle.lock().await;
            if guard.is_none() {
                *guard = Some(connection.handle.clone());

                // 同步记录最后一次默认连接配置，便于日志输出
                let mut cfg_guard = self.last_default_config.lock().await;
                *cfg_guard = Some(config.clone());
            }
        }

        Ok(connection.handle.clone())
    }

    async fn disconnect(&self, handle: &ConnectionHandle) -> AppResult<()> {
        log::info!("断开PLC连接: {}", handle.connection_id);

        self.pool.remove_connection(&handle.connection_id).await
    }

    async fn is_connected(&self, handle: &ConnectionHandle) -> AppResult<bool> {
        let connection = self.pool.get_connection(handle).await?;
        let is_connected = *connection.is_connected.lock().await;
        Ok(is_connected)
    }

    async fn read_bool(&self, handle: &ConnectionHandle, address: &str) -> AppResult<bool> {
        let connection = self.pool.get_connection(handle).await?;
        let start_time = Utc::now();

        // 获取配置信息
        let (plc_name, plc_host, plc_port) = {
            let configs = self.pool.configs.read().await;
            let config = configs.get(&handle.connection_id)
                .ok_or_else(|| AppError::not_found_error("PLC配置", &handle.connection_id))?;
            (config.name.clone(), config.host.clone(), config.port)
        };

        // 解析Modbus地址
        let (register_type, offset) = parse_modbus_address_ex(address, connection.zero_based_address)?;

        //log::info!("🔍 [PLC_READ_BOOL] 开始读取布尔值: PLC={}({}:{}), 地址={}, 类型={:?}, 偏移={}",
                   //plc_name, plc_host, plc_port, address, register_type, offset);

        let mut context_guard = connection.context.lock().await;
        let context = context_guard.as_mut()
            .ok_or_else(|| {
                log::error!("❌ [PLC_READ_BOOL] PLC连接已断开: PLC={}, 地址={}",
                           plc_name, address);
                AppError::plc_communication_error("连接已断开".to_string())
            })?;

        let result = match register_type {
            ModbusRegisterType::Coil => {
                match context.read_coils(offset, 1).await {
                    Ok(Ok(values)) => {
                        let value = values.first().copied().unwrap_or(false);
                        //log::info!("✅ [PLC_READ_BOOL] 读取线圈成功: PLC={}, 地址={}, 值={}",
                                  //plc_name, address, value);
                        value
                    },
                    Ok(Err(exception)) => {
                        log::error!("❌ [PLC_READ_BOOL] Modbus异常: PLC={}, 地址={}, 异常={:?}",
                                   plc_name, address, exception);
                        return Err(AppError::plc_communication_error(format!("Modbus异常: {:?}", exception)));
                    },
                    Err(e) => {
                        log::error!("❌ [PLC_READ_BOOL] 读取线圈失败: PLC={}, 地址={}, 错误={:?}",
                                   plc_name, address, e);
                        return Err(AppError::plc_communication_error(format!("读取线圈失败: {:?}", e)));
                    },
                }
            },
            ModbusRegisterType::DiscreteInput => {
                match context.read_discrete_inputs(offset, 1).await {
                    Ok(Ok(values)) => {
                        let value = values.first().copied().unwrap_or(false);
                        //log::info!("✅ [PLC_READ_BOOL] 读取离散输入成功: PLC={}, 地址={}, 值={}",
                                  //plc_name, address, value);
                        value
                    },
                    Ok(Err(exception)) => {
                        log::error!("❌ [PLC_READ_BOOL] Modbus异常: PLC={}, 地址={}, 异常={:?}",
                                   plc_name, address, exception);
                        return Err(AppError::plc_communication_error(format!("Modbus异常: {:?}", exception)));
                    },
                    Err(e) => {
                        log::error!("❌ [PLC_READ_BOOL] 读取离散输入失败: PLC={}, 地址={}, 错误={:?}",
                                   plc_name, address, e);
                        return Err(AppError::plc_communication_error(format!("读取离散输入失败: {:?}", e)));
                    },
                }
            },
            _ => {
                log::error!("❌ [PLC_READ_BOOL] 无效的布尔型地址: PLC={}, 地址={}, 类型={:?}",
                           plc_name, address, register_type);
                return Err(AppError::plc_communication_error(
                    format!("地址 {} 不是有效的布尔型地址", address)
                ));
            },
        };

        // 更新统计信息
        update_read_stats(&connection.stats, start_time).await;

        Ok(result)
    }

    async fn write_bool(&self, handle: &ConnectionHandle, address: &str, value: bool) -> AppResult<()> {
        let connection = self.pool.get_connection(handle).await?;
        let start_time = Utc::now();

        // 获取配置信息
        let plc_name = {
            let configs = self.pool.configs.read().await;
            let config = configs.get(&handle.connection_id)
                .ok_or_else(|| AppError::not_found_error("PLC配置", &handle.connection_id))?;
            config.name.clone()
        };

        // 解析Modbus地址
        let (register_type, offset) = parse_modbus_address_ex(address, connection.zero_based_address)?;

        log::info!("🔍 [PLC_WRITE_BOOL] 开始写入布尔值: PLC={}, 地址={}, 类型={:?}, 偏移={}, 值={}",
                   plc_name, address, register_type, offset, value);

        let mut context_guard = connection.context.lock().await;
        let context = context_guard.as_mut()
            .ok_or_else(|| {
                log::error!("❌ [PLC_WRITE_BOOL] PLC连接已断开: PLC={}, 地址={}",
                           plc_name, address);
                AppError::plc_communication_error("连接已断开".to_string())
            })?;

        match register_type {
            ModbusRegisterType::Coil => {
                match context.write_single_coil(offset, value).await {
                    Ok(_) => {
                        log::info!("✅ [PLC_WRITE_BOOL] 写入线圈成功: PLC={}, 地址={}, 值={}",
                                  plc_name, address, value);
                    },
                    Err(e) => {
                        log::error!("❌ [PLC_WRITE_BOOL] 写入线圈失败: PLC={}, 地址={}, 值={}, 错误={}",
                                   plc_name, address, value, e);
                        return Err(AppError::plc_communication_error(format!("写入线圈失败: {}", e)));
                    }
                }
            },
            _ => {
                log::error!("❌ [PLC_WRITE_BOOL] 无效的可写布尔型地址: PLC={}, 地址={}, 类型={:?}",
                           plc_name, address, register_type);
                return Err(AppError::plc_communication_error(
                    format!("地址 {} 不是有效的可写布尔型地址", address)
                ));
            },
        }

        // 更新统计信息
        update_write_stats(&connection.stats, start_time).await;

        Ok(())
    }

    async fn read_f32(&self, handle: &ConnectionHandle, address: &str) -> AppResult<f32> {
        let connection = self.pool.get_connection(handle).await?;
        let start_time = Utc::now();

        // 解析Modbus地址
        let (register_type, offset) = parse_modbus_address_ex(address, connection.zero_based_address)?;

        let mut context_guard = connection.context.lock().await;
        let context = context_guard.as_mut()
            .ok_or_else(|| AppError::plc_communication_error("连接已断开".to_string()))?;

        let registers = match register_type {
            ModbusRegisterType::HoldingRegister => {
                match context.read_holding_registers(offset, 2).await {
                    Ok(Ok(regs)) => regs,
                    Ok(Err(exception)) => return Err(AppError::plc_communication_error(format!("Modbus异常: {:?}", exception))),
                    Err(e) => return Err(AppError::plc_communication_error(format!("读取保持寄存器失败: {:?}", e))),
                }
            },
            ModbusRegisterType::InputRegister => {
                match context.read_input_registers(offset, 2).await {
                    Ok(Ok(regs)) => regs,
                    Ok(Err(exception)) => return Err(AppError::plc_communication_error(format!("Modbus异常: {:?}", exception))),
                    Err(e) => return Err(AppError::plc_communication_error(format!("读取输入寄存器失败: {:?}", e))),
                }
            },
            _ => return Err(AppError::plc_communication_error(
                format!("地址 {} 不是有效的32位寄存器地址", address)
            )),
        };

        if registers.len() < 2 {
            return Err(AppError::plc_communication_error(
                "读取的寄存器数量不足".to_string()
            ));
        }

        // 转换为f32 (使用大端字节序)
        let result = ByteOrderConverter::registers_to_float(registers[0], registers[1], connection.byte_order);

        // 更新统计信息
        update_read_stats(&connection.stats, start_time).await;

        Ok(result)
    }

    async fn write_f32(&self, handle: &ConnectionHandle, address: &str, value: f32) -> AppResult<()> {
        let connection = self.pool.get_connection(handle).await?;
        let start_time = Utc::now();

        // 解析Modbus地址
        let (register_type, offset) = parse_modbus_address_ex(address, connection.zero_based_address)?;

        let mut context_guard = connection.context.lock().await;
        let context = context_guard.as_mut()
            .ok_or_else(|| AppError::plc_communication_error("连接已断开".to_string()))?;

        match register_type {
            ModbusRegisterType::HoldingRegister => {
                let (reg1, reg2) = ByteOrderConverter::float_to_registers(value, connection.byte_order);
                let registers = [reg1, reg2];
                context.write_multiple_registers(offset, &registers).await
                    .map_err(|e| AppError::plc_communication_error(format!("写入保持寄存器失败: {}", e)))?;
            },
            _ => return Err(AppError::plc_communication_error(
                format!("地址 {} 不是有效的可写32位寄存器地址", address)
            )),
        }

        // 更新统计信息
        update_write_stats(&connection.stats, start_time).await;

        Ok(())
    }

    async fn read_i32(&self, handle: &ConnectionHandle, address: &str) -> AppResult<i32> {
        let connection = self.pool.get_connection(handle).await?;
        let start_time = Utc::now();

        // 解析Modbus地址
        let (register_type, offset) = parse_modbus_address_ex(address, connection.zero_based_address)?;

        let mut context_guard = connection.context.lock().await;
        let context = context_guard.as_mut()
            .ok_or_else(|| AppError::plc_communication_error("连接已断开".to_string()))?;

        let registers = match register_type {
            ModbusRegisterType::HoldingRegister => {
                match context.read_holding_registers(offset, 2).await {
                    Ok(Ok(regs)) => regs,
                    Ok(Err(exception)) => return Err(AppError::plc_communication_error(format!("Modbus异常: {:?}", exception))),
                    Err(e) => return Err(AppError::plc_communication_error(format!("读取保持寄存器失败: {:?}", e))),
                }
            },
            ModbusRegisterType::InputRegister => {
                match context.read_input_registers(offset, 2).await {
                    Ok(Ok(regs)) => regs,
                    Ok(Err(exception)) => return Err(AppError::plc_communication_error(format!("Modbus异常: {:?}", exception))),
                    Err(e) => return Err(AppError::plc_communication_error(format!("读取输入寄存器失败: {:?}", e))),
                }
            },
            _ => return Err(AppError::plc_communication_error(
                format!("地址 {} 不是有效的32位寄存器地址", address)
            )),
        };

        if registers.len() < 2 {
            return Err(AppError::plc_communication_error(
                "读取的寄存器数量不足".to_string()
            ));
        }

        // 根据连接字节序转换为 i32
        let result = ByteOrderConverter::registers_to_i32(registers[0], registers[1], connection.byte_order);

        // 更新统计信息
        update_read_stats(&connection.stats, start_time).await;

        Ok(result)
    }

    async fn write_i32(&self, handle: &ConnectionHandle, address: &str, value: i32) -> AppResult<()> {
        let connection = self.pool.get_connection(handle).await?;
        let start_time = Utc::now();

        // 解析Modbus地址
        let (register_type, offset) = parse_modbus_address_ex(address, connection.zero_based_address)?;

        let mut context_guard = connection.context.lock().await;
        let context = context_guard.as_mut()
            .ok_or_else(|| AppError::plc_communication_error("连接已断开".to_string()))?;

        match register_type {
            ModbusRegisterType::HoldingRegister => {
                let (reg1, reg2) = ByteOrderConverter::i32_to_registers(value, connection.byte_order);
                let registers = [reg1, reg2];
                context.write_multiple_registers(offset, &registers).await
                    .map_err(|e| AppError::plc_communication_error(format!("写入保持寄存器失败: {}", e)))?;
            },
            _ => return Err(AppError::plc_communication_error(
                format!("地址 {} 不是有效的可写32位寄存器地址", address)
            )),
        }

        // 更新统计信息
        update_write_stats(&connection.stats, start_time).await;

        Ok(())
    }

    async fn batch_read(&self, handle: &ConnectionHandle, requests: &[ReadRequest]) -> AppResult<Vec<ReadResult>> {
        let connection = self.pool.get_connection(handle).await?;
        let mut results = Vec::with_capacity(requests.len());

        for request in requests {
            let start_time = Utc::now();
            let result = match request.data_type {
                PlcDataType::Bool => {
                    match self.read_bool(handle, &request.address).await {
                        Ok(value) => ReadResult {
                            request_id: request.id.clone(),
                            success: true,
                            value: Some(PlcValue::Bool(value)),
                            error_message: None,
                            execution_time_ms: Utc::now().signed_duration_since(start_time).num_milliseconds() as u64,
                        },
                        Err(e) => ReadResult {
                            request_id: request.id.clone(),
                            success: false,
                            value: None,
                            error_message: Some(e.to_string()),
                            execution_time_ms: Utc::now().signed_duration_since(start_time).num_milliseconds() as u64,
                        },
                    }
                },
                PlcDataType::Float32 => {
                    match self.read_f32(handle, &request.address).await {
                        Ok(value) => ReadResult {
                            request_id: request.id.clone(),
                            success: true,
                            value: Some(PlcValue::Float32(value)),
                            error_message: None,
                            execution_time_ms: Utc::now().signed_duration_since(start_time).num_milliseconds() as u64,
                        },
                        Err(e) => ReadResult {
                            request_id: request.id.clone(),
                            success: false,
                            value: None,
                            error_message: Some(e.to_string()),
                            execution_time_ms: Utc::now().signed_duration_since(start_time).num_milliseconds() as u64,
                        },
                    }
                },
                PlcDataType::Int32 => {
                    match self.read_i32(handle, &request.address).await {
                        Ok(value) => ReadResult {
                            request_id: request.id.clone(),
                            success: true,
                            value: Some(PlcValue::Int32(value)),
                            error_message: None,
                            execution_time_ms: Utc::now().signed_duration_since(start_time).num_milliseconds() as u64,
                        },
                        Err(e) => ReadResult {
                            request_id: request.id.clone(),
                            success: false,
                            value: None,
                            error_message: Some(e.to_string()),
                            execution_time_ms: Utc::now().signed_duration_since(start_time).num_milliseconds() as u64,
                        },
                    }
                },
                _ => ReadResult {
                    request_id: request.id.clone(),
                    success: false,
                    value: None,
                    error_message: Some(format!("不支持的数据类型: {:?}", request.data_type)),
                    execution_time_ms: 0,
                },
            };

            results.push(result);
        }

        Ok(results)
    }

    async fn batch_write(&self, handle: &ConnectionHandle, requests: &[WriteRequest]) -> AppResult<Vec<WriteResult>> {
        let connection = self.pool.get_connection(handle).await?;
        let mut results = Vec::with_capacity(requests.len());

        for request in requests {
            let start_time = Utc::now();
            let result = match &request.value {
                PlcValue::Bool(value) => {
                    match self.write_bool(handle, &request.address, *value).await {
                        Ok(_) => WriteResult {
                            request_id: request.id.clone(),
                            success: true,
                            error_message: None,
                            execution_time_ms: Utc::now().signed_duration_since(start_time).num_milliseconds() as u64,
                        },
                        Err(e) => WriteResult {
                            request_id: request.id.clone(),
                            success: false,
                            error_message: Some(e.to_string()),
                            execution_time_ms: Utc::now().signed_duration_since(start_time).num_milliseconds() as u64,
                        },
                    }
                },
                PlcValue::Float32(value) => {
                    match self.write_f32(handle, &request.address, *value).await {
                        Ok(_) => WriteResult {
                            request_id: request.id.clone(),
                            success: true,
                            error_message: None,
                            execution_time_ms: Utc::now().signed_duration_since(start_time).num_milliseconds() as u64,
                        },
                        Err(e) => WriteResult {
                            request_id: request.id.clone(),
                            success: false,
                            error_message: Some(e.to_string()),
                            execution_time_ms: Utc::now().signed_duration_since(start_time).num_milliseconds() as u64,
                        },
                    }
                },
                PlcValue::Int32(value) => {
                    match self.write_i32(handle, &request.address, *value).await {
                        Ok(_) => WriteResult {
                            request_id: request.id.clone(),
                            success: true,
                            error_message: None,
                            execution_time_ms: Utc::now().signed_duration_since(start_time).num_milliseconds() as u64,
                        },
                        Err(e) => WriteResult {
                            request_id: request.id.clone(),
                            success: false,
                            error_message: Some(e.to_string()),
                            execution_time_ms: Utc::now().signed_duration_since(start_time).num_milliseconds() as u64,
                        },
                    }
                },
                _ => WriteResult {
                    request_id: request.id.clone(),
                    success: false,
                    error_message: Some(format!("不支持的数据类型: {:?}", request.value)),
                    execution_time_ms: 0,
                },
            };

            results.push(result);
        }

        Ok(results)
    }

    async fn get_connection_stats(&self, handle: &ConnectionHandle) -> AppResult<ConnectionStats> {
        let connection = self.pool.get_connection(handle).await?;
        let stats = connection.stats.lock().await;
        Ok(stats.clone())
    }

    async fn default_handle_by_id(&self, connection_id: &str) -> Option<ConnectionHandle> {
        let guard = self.default_handles.lock().await;
        guard.get(connection_id).cloned()
    }

    async fn default_handle(&self) -> Option<ConnectionHandle> {
        let guard = self.default_handle.lock().await;
        guard.clone()
    }

    async fn test_connection(&self, config: &PlcConnectionConfig) -> AppResult<ConnectionTestResult> {
        let start_time = Utc::now();

        // 尝试建立临时连接进行测试
        match self.pool.get_or_create_connection(config).await {
            Ok(connection) => {
                let connection_time = Utc::now().signed_duration_since(start_time).num_milliseconds() as u64;

                // 尝试读取一个测试地址
                let test_result = {
                    let mut context_guard = connection.context.lock().await;
                    if let Some(context) = context_guard.as_mut() {
                        // 尝试读取第一个保持寄存器
                        context.read_holding_registers(0, 1).await.is_ok()
                    } else {
                        false
                    }
                };

                Ok(ConnectionTestResult {
                    success: test_result,
                    connection_time_ms: connection_time,
                    error_message: if test_result { None } else { Some("测试读取失败".to_string()) },
                    protocol_info: Some("Modbus TCP".to_string()),
                    device_info: Some(format!("{}:{}", config.host, config.port)),
                })
            },
            Err(e) => {
                let connection_time = Utc::now().signed_duration_since(start_time).num_milliseconds() as u64;
                Ok(ConnectionTestResult {
                    success: false,
                    connection_time_ms: connection_time,
                    error_message: Some(e.to_string()),
                    protocol_info: Some("Modbus TCP".to_string()),
                    device_info: Some(format!("{}:{}", config.host, config.port)),
                })
            }
        }
    }
}

/// Modbus寄存器类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModbusRegisterType {
    Coil,           // 0x 线圈
    DiscreteInput,  // 1x 离散输入
    InputRegister,  // 3x 输入寄存器
    HoldingRegister,// 4x 保持寄存器
}

/// 解析Modbus地址
///
/// 支持的格式：
/// - 0xxxx: 线圈 (Coil)
/// - 1xxxx: 离散输入 (Discrete Input)
/// - 3xxxx: 输入寄存器 (Input Register)
/// - 4xxxx: 保持寄存器 (Holding Register)
pub fn parse_modbus_address_ex(address: &str, zero_based: bool) -> AppResult<(ModbusRegisterType, u16)> {
    if address.is_empty() {
        return Err(AppError::validation_error("地址不能为空".to_string()));
    }

    if address.len() < 2 {
        return Err(AppError::validation_error(
            format!("地址格式无效: {}", address)
        ));
    }

    // 如果地址长度不足5位，默认认为省略了首位'0'，按线圈(Coils)处理
    if address.len() < 5 {
        let offset = address.parse::<u16>()
            .map_err(|_| AppError::validation_error(
                format!("无效的线圈地址: {}", address)
            ))?;
        let protocol_offset = if zero_based { offset } else { if offset > 0 { offset - 1 } else { 0 } };
        return Ok((ModbusRegisterType::Coil, protocol_offset));
    }

    let first_char = address.chars().next().unwrap();
    let offset_str = &address[1..];

    let offset = offset_str.parse::<u16>()
        .map_err(|_| AppError::validation_error(
            format!("无效的地址偏移量: {}", offset_str)
        ))?;

    let protocol_offset = if zero_based { offset } else { if offset > 0 { offset - 1 } else { 0 } };

    let register_type = match first_char {
        '0' => ModbusRegisterType::Coil,
        '1' => ModbusRegisterType::DiscreteInput,
        '3' => ModbusRegisterType::InputRegister,
        '4' => ModbusRegisterType::HoldingRegister,
        _ => return Err(AppError::validation_error(
            format!("不支持的地址类型前缀: '{}' in '{}'", first_char, address)
        )),
    };

    Ok((register_type, protocol_offset))
}

/// 兼容旧代码的单参数版本，默认按1基地址（zero_based = false）
pub fn parse_modbus_address(address: &str) -> AppResult<(ModbusRegisterType, u16)> {
    parse_modbus_address_ex(address, false)
}

/// 字节序转换工具
struct ByteOrderConverter;
impl ByteOrderConverter {
    fn registers_to_float(reg1: u16, reg2: u16, order: crate::models::ByteOrder) -> f32 {
        let bytes = match order {
            crate::models::ByteOrder::ABCD => [
                (reg1 >> 8) as u8,
                (reg1 & 0xFF) as u8,
                (reg2 >> 8) as u8,
                (reg2 & 0xFF) as u8,
            ],
            crate::models::ByteOrder::CDAB => [
                (reg2 >> 8) as u8,
                (reg2 & 0xFF) as u8,
                (reg1 >> 8) as u8,
                (reg1 & 0xFF) as u8,
            ],
            crate::models::ByteOrder::BADC => [
                (reg1 & 0xFF) as u8,
                (reg1 >> 8) as u8,
                (reg2 & 0xFF) as u8,
                (reg2 >> 8) as u8,
            ],
            crate::models::ByteOrder::DCBA => [
                (reg2 & 0xFF) as u8,
                (reg2 >> 8) as u8,
                (reg1 & 0xFF) as u8,
                (reg1 >> 8) as u8,
            ],
        };
        f32::from_be_bytes(bytes)
    }

    fn float_to_registers(value: f32, order: crate::models::ByteOrder) -> (u16, u16) {
        let bytes = value.to_be_bytes();
        match order {
            crate::models::ByteOrder::ABCD => {
                let reg1 = ((bytes[0] as u16) << 8) | (bytes[1] as u16);
                let reg2 = ((bytes[2] as u16) << 8) | (bytes[3] as u16);
                (reg1, reg2)
            }
            crate::models::ByteOrder::CDAB => {
                let reg1 = ((bytes[2] as u16) << 8) | (bytes[3] as u16);
                let reg2 = ((bytes[0] as u16) << 8) | (bytes[1] as u16);
                (reg1, reg2)
            }
            crate::models::ByteOrder::BADC => {
                let reg1 = ((bytes[1] as u16) << 8) | (bytes[0] as u16);
                let reg2 = ((bytes[3] as u16) << 8) | (bytes[2] as u16);
                (reg1, reg2)
            }
            crate::models::ByteOrder::DCBA => {
                let reg1 = ((bytes[3] as u16) << 8) | (bytes[2] as u16);
                let reg2 = ((bytes[1] as u16) << 8) | (bytes[0] as u16);
                (reg1, reg2)
            }
        }
    }

    // i32 <-> registers
    fn registers_to_i32(reg1: u16, reg2: u16, order: crate::models::ByteOrder) -> i32 {
        let bytes = match order {
            crate::models::ByteOrder::ABCD => [
                (reg1 >> 8) as u8,
                (reg1 & 0xFF) as u8,
                (reg2 >> 8) as u8,
                (reg2 & 0xFF) as u8,
            ],
            crate::models::ByteOrder::CDAB => [
                (reg2 >> 8) as u8,
                (reg2 & 0xFF) as u8,
                (reg1 >> 8) as u8,
                (reg1 & 0xFF) as u8,
            ],
            crate::models::ByteOrder::BADC => [
                (reg1 & 0xFF) as u8,
                (reg1 >> 8) as u8,
                (reg2 & 0xFF) as u8,
                (reg2 >> 8) as u8,
            ],
            crate::models::ByteOrder::DCBA => [
                (reg2 & 0xFF) as u8,
                (reg2 >> 8) as u8,
                (reg1 & 0xFF) as u8,
                (reg1 >> 8) as u8,
            ],
        };
        i32::from_be_bytes(bytes)
    }

    fn i32_to_registers(value: i32, order: crate::models::ByteOrder) -> (u16, u16) {
        let bytes = value.to_be_bytes();
        match order {
            crate::models::ByteOrder::ABCD => {
                let reg1 = ((bytes[0] as u16) << 8) | (bytes[1] as u16);
                let reg2 = ((bytes[2] as u16) << 8) | (bytes[3] as u16);
                (reg1, reg2)
            }
            crate::models::ByteOrder::CDAB => {
                let reg1 = ((bytes[2] as u16) << 8) | (bytes[3] as u16);
                let reg2 = ((bytes[0] as u16) << 8) | (bytes[1] as u16);
                (reg1, reg2)
            }
            crate::models::ByteOrder::BADC => {
                let reg1 = ((bytes[1] as u16) << 8) | (bytes[0] as u16);
                let reg2 = ((bytes[3] as u16) << 8) | (bytes[2] as u16);
                (reg1, reg2)
            }
            crate::models::ByteOrder::DCBA => {
                let reg1 = ((bytes[3] as u16) << 8) | (bytes[2] as u16);
                let reg2 = ((bytes[1] as u16) << 8) | (bytes[0] as u16);
                (reg1, reg2)
            }
        }
    }

}

/// 将两个16位寄存器转换为32位浮点数 (大端字节序)
pub fn registers_to_f32(registers: &[u16]) -> f32 {
    if registers.len() < 2 {
        return 0.0;
    }

    // 大端字节序: 高位在前
    let bytes = [
        (registers[0] >> 8) as u8,
        (registers[0] & 0xFF) as u8,
        (registers[1] >> 8) as u8,
        (registers[1] & 0xFF) as u8,
    ];

    f32::from_be_bytes(bytes)
}


/// 更新读取统计信息
async fn update_read_stats(stats: &Arc<Mutex<ConnectionStats>>, start_time: DateTime<Utc>) {
    let mut stats = stats.lock().await;
    let duration = Utc::now().signed_duration_since(start_time).num_milliseconds() as u64;

    stats.total_reads += 1;
    stats.successful_reads += 1;
    stats.last_communication = Utc::now();

    // 更新平均读取时间
    let total_time = stats.average_read_time_ms * (stats.successful_reads - 1) as f64 + duration as f64;
    stats.average_read_time_ms = total_time / stats.successful_reads as f64;
}

/// 更新写入统计信息
async fn update_write_stats(stats: &Arc<Mutex<ConnectionStats>>, start_time: DateTime<Utc>) {
    let mut stats = stats.lock().await;
    let duration = Utc::now().signed_duration_since(start_time).num_milliseconds() as u64;

    stats.total_writes += 1;
    stats.successful_writes += 1;
    stats.last_communication = Utc::now();

    // 更新平均写入时间
    let total_time = stats.average_write_time_ms * (stats.successful_writes - 1) as f64 + duration as f64;
    stats.average_write_time_ms = total_time / stats.successful_writes as f64;
}


