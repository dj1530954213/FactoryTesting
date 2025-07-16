//! # PLC通信服务接口模块
//!
//! ## 业务作用
//! 本模块定义了PLC通信服务的核心接口，是整个PLC通信系统的抽象层：
//! - 定义统一的PLC通信接口规范
//! - 支持多种PLC协议（Modbus TCP、Siemens S7、OPC UA等）
//! - 提供连接管理、数据读写、状态监控等核心功能
//! - 实现领域驱动设计(DDD)的服务接口层
//!
//! ## 设计理念
//! - **接口隔离**: 将PLC通信的抽象接口与具体实现分离
//! - **协议无关**: 支持多种PLC通信协议的统一接口
//! - **异步优先**: 所有操作都是异步的，提高系统性能
//! - **类型安全**: 使用强类型确保数据操作的安全性
//! - **错误处理**: 统一的错误处理机制和详细的错误信息
//!
//! ## 架构位置
//! 在DDD架构中，本模块属于领域服务层，位于：
//! - 应用层之下：为应用层提供PLC通信能力
//! - 基础设施层之上：定义基础设施层需要实现的接口
//!
//! ## Rust知识点
//! - **async trait**: 异步trait的定义和使用
//! - **trait对象**: 动态分发和接口抽象
//! - **泛型约束**: BaseService trait约束
//! - **生命周期**: 引用参数的生命周期管理

use super::*;
use std::collections::HashMap;
use once_cell::sync::OnceCell;
use std::sync::Arc;
use crate::domain::impls::plc_connection_manager::PlcConnectionManager;

/// 全局PLC连接管理器
///
/// **业务作用**:
/// - 提供全局唯一的PLC连接管理器访问点
/// - 支持跨模块的连接状态查询和管理
/// - 实现连接资源的统一管理和监控
///
/// **技术实现**:
/// - 使用OnceCell确保线程安全的单例初始化
/// - Arc智能指针支持多线程共享访问
/// - 延迟初始化，在首次使用时创建
///
/// **Rust知识点**:
/// - `OnceCell<T>`: 线程安全的延迟初始化容器
/// - `static`: 全局静态变量，程序生命周期内存在
/// - `Arc<T>`: 原子引用计数智能指针
static GLOBAL_PLC_MANAGER: OnceCell<Arc<PlcConnectionManager>> = OnceCell::new();

/// 设置全局PLC连接管理器
///
/// **业务作用**: 在应用启动时注册PLC连接管理器实例
/// **调用时机**: 通常在依赖注入容器初始化时调用
/// **线程安全**: 只能成功设置一次，后续调用会被忽略
///
/// **参数**: `mgr` - PLC连接管理器的Arc智能指针
///
/// **使用示例**:
/// ```rust
/// let manager = Arc::new(PlcConnectionManager::new());
/// set_global_plc_manager(manager);
/// ```
pub fn set_global_plc_manager(mgr: Arc<PlcConnectionManager>) {
    let _ = GLOBAL_PLC_MANAGER.set(mgr); // 忽略返回值，设置失败表示已设置过
}

/// 获取全局PLC连接管理器
///
/// **业务作用**: 为其他模块提供PLC连接管理器的访问接口
/// **返回值**: `Option<Arc<PlcConnectionManager>>` - 可能为空的管理器引用
/// **使用场景**: 在需要查询连接状态或执行连接管理操作时调用
///
/// **错误处理**: 返回None表示管理器尚未初始化
///
/// **Rust知识点**:
/// - `Option<T>`: 表示可能存在或不存在的值
/// - `cloned()`: 对Option内的Arc进行克隆，增加引用计数
pub fn get_global_plc_manager() -> Option<Arc<PlcConnectionManager>> {
    GLOBAL_PLC_MANAGER.get().cloned()
}

/// PLC通信服务接口
///
/// ## 业务职责
/// 本trait定义了PLC通信服务的核心接口，负责：
/// - **连接管理**: 建立、维护和断开PLC连接
/// - **数据通信**: 读写各种数据类型（布尔、整数、浮点数等）
/// - **状态监控**: 监控连接状态和通信质量
/// - **错误处理**: 统一的错误处理和恢复机制
/// - **协议抽象**: 为不同PLC协议提供统一接口
///
/// ## 设计原则
/// - **协议无关**: 支持Modbus TCP、Siemens S7、OPC UA等多种协议
/// - **异步优先**: 所有操作都是异步的，避免阻塞
/// - **类型安全**: 使用强类型确保数据操作的正确性
/// - **资源管理**: 通过连接句柄管理连接生命周期
/// - **可扩展性**: 易于添加新的数据类型和操作
///
/// ## 符合规范
/// 遵循 FAT-CTK-001 规则：通信任务规则
///
/// ## Rust知识点
/// - `#[async_trait]`: 支持trait中的异步方法
/// - `BaseService`: trait继承，获得基础服务能力
/// - `&self`: 不可变引用，支持并发访问
/// - `AppResult<T>`: 统一的错误处理类型
#[async_trait]
pub trait IPlcCommunicationService: BaseService {
    /// 连接到PLC设备
    ///
    /// **业务逻辑**:
    /// 1. 验证连接配置的有效性
    /// 2. 建立与PLC设备的网络连接
    /// 3. 执行协议握手和认证
    /// 4. 创建连接句柄用于后续操作
    ///
    /// **错误处理**: 连接失败时返回详细的错误信息
    /// **性能考虑**: 支持连接池和连接复用
    ///
    /// **参数**:
    /// * `config` - PLC连接配置，包含IP地址、端口、协议参数等
    ///
    /// **返回值**:
    /// * `ConnectionHandle` - 连接句柄，用于标识和管理连接
    async fn connect(&self, config: &PlcConnectionConfig) -> AppResult<ConnectionHandle>;

    /// 断开PLC连接
    ///
    /// **业务逻辑**:
    /// 1. 验证连接句柄的有效性
    /// 2. 优雅地关闭网络连接
    /// 3. 清理相关资源和缓存
    /// 4. 更新连接状态
    ///
    /// **资源管理**: 确保连接资源被正确释放
    /// **并发安全**: 支持多线程环境下的安全断开
    ///
    /// **参数**:
    /// * `handle` - 要断开的连接句柄
    async fn disconnect(&self, handle: &ConnectionHandle) -> AppResult<()>;

    /// 检查连接状态
    ///
    /// **业务逻辑**:
    /// 1. 验证连接句柄的有效性
    /// 2. 检查网络连接的活跃状态
    /// 3. 可选择性地执行心跳测试
    /// 4. 返回当前连接状态
    ///
    /// **性能优化**: 使用缓存避免频繁的网络检测
    /// **实时性**: 提供准确的连接状态信息
    ///
    /// **参数**:
    /// * `handle` - 要检查的连接句柄
    ///
    /// **返回值**:
    /// * `bool` - true表示已连接，false表示未连接
    async fn is_connected(&self, handle: &ConnectionHandle) -> AppResult<bool>;
    
    /// 读取布尔值
    ///
    /// **业务场景**: 读取PLC中的开关状态、报警信号、运行状态等布尔型数据
    /// **数据类型**: 对应PLC中的线圈(Coil)或离散输入(Discrete Input)
    ///
    /// **实现要点**:
    /// - 根据地址类型选择合适的Modbus功能码
    /// - 处理不同PLC厂商的地址编码差异
    /// - 提供详细的错误信息用于故障诊断
    ///
    /// **参数**:
    /// * `handle` - 连接句柄，标识目标PLC连接
    /// * `address` - PLC地址，如"00001"、"10001"等
    ///
    /// **返回值**:
    /// * `bool` - 读取的布尔值，true/false
    ///
    /// **错误处理**: 地址无效、连接断开、读取超时等情况
    async fn read_bool(&self, handle: &ConnectionHandle, address: &str) -> AppResult<bool>;

    /// 写入布尔值
    ///
    /// **业务场景**: 控制PLC中的输出线圈、设置控制标志、触发操作等
    /// **数据类型**: 对应PLC中的线圈(Coil)输出
    ///
    /// **实现要点**:
    /// - 验证地址是否可写（某些地址可能是只读的）
    /// - 确保写入操作的原子性
    /// - 记录写入操作的日志用于审计
    ///
    /// **参数**:
    /// * `handle` - 连接句柄，标识目标PLC连接
    /// * `address` - PLC地址，如"00001"等可写线圈地址
    /// * `value` - 要写入的布尔值
    ///
    /// **安全考虑**: 写入操作可能影响设备状态，需要权限验证
    async fn write_bool(&self, handle: &ConnectionHandle, address: &str, value: bool) -> AppResult<()>;

    /// 读取32位浮点数
    ///
    /// **业务场景**: 读取PLC中的模拟量数据，如温度、压力、流量等传感器值
    /// **数据类型**: 对应PLC中的保持寄存器(Holding Register)或输入寄存器(Input Register)
    ///
    /// **技术细节**:
    /// - 32位浮点数占用2个连续的16位寄存器
    /// - 需要处理字节序(Endianness)问题
    /// - 支持IEEE 754浮点数标准
    ///
    /// **参数**:
    /// * `handle` - 连接句柄，标识目标PLC连接
    /// * `address` - PLC地址，如"40001"、"30001"等寄存器地址
    ///
    /// **返回值**:
    /// * `f32` - 读取的32位浮点数
    ///
    /// **精度考虑**: 浮点数的精度受PLC和网络传输影响
    async fn read_f32(&self, handle: &ConnectionHandle, address: &str) -> AppResult<f32>;

    /// 写入32位浮点数
    ///
    /// **业务场景**: 设置PLC中的设定值，如温度设定点、速度设定值等
    /// **数据类型**: 对应PLC中的保持寄存器(Holding Register)
    ///
    /// **技术细节**:
    /// - 将32位浮点数转换为2个16位寄存器值
    /// - 处理字节序转换确保数据正确性
    /// - 支持原子性写入操作
    ///
    /// **参数**:
    /// * `handle` - 连接句柄，标识目标PLC连接
    /// * `address` - PLC地址，如"40001"等可写寄存器地址
    /// * `value` - 要写入的32位浮点数
    ///
    /// **数据验证**: 检查浮点数的有效性（非NaN、非无穷大等）
    async fn write_f32(&self, handle: &ConnectionHandle, address: &str, value: f32) -> AppResult<()>;
    
    /// 读取32位整数
    /// 
    /// # 参数
    /// * `handle` - 连接句柄
    /// * `address` - PLC地址
    /// 
    /// # 返回
    /// * `i32` - 读取的整数
    async fn read_i32(&self, handle: &ConnectionHandle, address: &str) -> AppResult<i32>;
    
    /// 写入32位整数
    /// 
    /// # 参数
    /// * `handle` - 连接句柄
    /// * `address` - PLC地址
    /// * `value` - 要写入的值
    async fn write_i32(&self, handle: &ConnectionHandle, address: &str, value: i32) -> AppResult<()>;
    
    /// 批量读取操作
    /// 
    /// # 参数
    /// * `handle` - 连接句柄
    /// * `requests` - 读取请求列表
    /// 
    /// # 返回
    /// * `Vec<ReadResult>` - 读取结果列表
    async fn batch_read(&self, handle: &ConnectionHandle, requests: &[ReadRequest]) -> AppResult<Vec<ReadResult>>;
    
    /// 批量写入操作
    /// 
    /// # 参数
    /// * `handle` - 连接句柄
    /// * `requests` - 写入请求列表
    /// 
    /// # 返回
    /// * `Vec<WriteResult>` - 写入结果列表
    async fn batch_write(&self, handle: &ConnectionHandle, requests: &[WriteRequest]) -> AppResult<Vec<WriteResult>>;
    
    /// 获取连接统计信息
    /// 
    /// # 参数
    /// * `handle` - 连接句柄
    /// 
    /// # 返回
    /// * `ConnectionStats` - 连接统计信息
    async fn get_connection_stats(&self, handle: &ConnectionHandle) -> AppResult<ConnectionStats>;
    
    /// 测试连接
    /// 
    /// # 参数
    /// * `config` - 连接配置
    /// 
    /// # 返回
    /// * `ConnectionTestResult` - 连接测试结果
    async fn test_connection(&self, config: &PlcConnectionConfig) -> AppResult<ConnectionTestResult>;

    /// 获取指定连接ID的默认连接句柄（若尚未连接则返回 None）
    async fn default_handle_by_id(&self, connection_id: &str) -> Option<ConnectionHandle>;

    /// 获取最后一次连接的默认连接句柄（向后兼容）
    async fn default_handle(&self) -> Option<ConnectionHandle>;
}

/// PLC连接配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlcConnectionConfig {
    /// 连接ID
    pub id: String,
    
    /// 连接名称
    pub name: String,
    
    /// 协议类型
    pub protocol: PlcProtocol,
    
    /// 主机地址
    pub host: String,
    
    /// 端口号
    pub port: u16,
    
    /// 连接超时（毫秒）
    pub timeout_ms: u64,
    
    /// 读取超时（毫秒）
    pub read_timeout_ms: u64,
    
    /// 写入超时（毫秒）
    pub write_timeout_ms: u64,

    /// 字节顺序，如 "ABCD" "CDAB" "BADC" "DCBA"
    pub byte_order: String,

    /// 地址是否从0开始（Modbus中有的PLC地址0基）
    pub zero_based_address: bool,
    
    /// 重试次数
    pub retry_count: u32,
    
    /// 重试间隔（毫秒）
    pub retry_interval_ms: u64,
    
    /// 协议特定参数
    pub protocol_params: HashMap<String, serde_json::Value>,
}

/// PLC协议类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlcProtocol {
    ModbusTcp,
    SiemensS7,
    OpcUa,
    EthernetIp,
}

/// PLC连接句柄
///
/// **业务作用**:
/// - 作为PLC连接的唯一标识符和访问凭证
/// - 封装连接的元数据信息，便于连接管理
/// - 提供连接的生命周期跟踪能力
/// - 支持连接的安全访问和权限控制
///
/// **设计理念**:
/// - **句柄模式**: 通过句柄间接访问连接，提供抽象层
/// - **不可变性**: 句柄创建后其核心信息不会改变
/// - **可追踪性**: 记录连接的创建和活动时间
/// - **协议无关**: 支持多种PLC通信协议
///
/// **使用场景**:
/// - 作为PLC操作方法的参数
/// - 连接池中的连接标识
/// - 连接状态监控和管理
/// - 连接权限验证和审计
///
/// **生命周期**:
/// 1. 连接建立时创建句柄
/// 2. 每次操作时更新最后活动时间
/// 3. 连接断开时句柄失效
///
/// **Rust知识点**:
/// - `#[derive(...)]`: 自动实现常用trait
/// - `Clone`: 支持句柄的复制，便于传递
/// - `Serialize/Deserialize`: 支持序列化，便于存储和传输
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionHandle {
    /// 连接配置ID
    /// **业务含义**: 对应连接配置的唯一标识符
    /// **关联关系**: 与PlcConnectionConfig的id字段对应
    /// **用途**: 用于查找连接配置和管理连接池
    pub connection_id: String,

    /// 句柄唯一ID
    /// **业务含义**: 每个句柄实例的唯一标识符
    /// **生成方式**: 通常使用UUID确保全局唯一性
    /// **用途**: 区分同一连接的不同句柄实例
    pub handle_id: String,

    /// PLC通信协议类型
    /// **业务含义**: 标识使用的通信协议
    /// **协议支持**: ModbusTcp, SiemensS7, OpcUa, EthernetIp
    /// **用途**: 协议特定的操作和优化
    pub protocol: PlcProtocol,

    /// 句柄创建时间
    /// **业务含义**: 记录连接建立的时间戳
    /// **时区处理**: 使用UTC时间避免时区问题
    /// **用途**: 连接时长统计、超时检测
    pub created_at: DateTime<Utc>,

    /// 最后活动时间
    /// **业务含义**: 记录最后一次使用连接的时间
    /// **更新时机**: 每次读写操作后更新
    /// **用途**: 空闲连接检测、连接回收
    pub last_activity: DateTime<Utc>,
}

/// PLC数据读取请求
///
/// **业务作用**:
/// - 封装PLC数据读取操作的所有必要参数
/// - 提供统一的读取请求格式，支持不同协议
/// - 支持单值和数组数据的读取
/// - 便于请求的序列化、传输和日志记录
///
/// **设计特点**:
/// - **类型安全**: 明确指定要读取的数据类型
/// - **地址抽象**: 使用字符串地址，支持不同的地址格式
/// - **数组支持**: 可选的数组长度参数支持批量读取
/// - **可追踪**: 每个请求都有唯一ID便于跟踪
///
/// **使用流程**:
/// 1. 创建读取请求，指定地址和数据类型
/// 2. 通过PLC服务执行读取操作
/// 3. 获得ReadResult结果
/// 4. 处理读取的数据或错误
///
/// **地址格式示例**:
/// - Modbus: "40001", "00001", "30001"
/// - Siemens: "DB1.DBD0", "M0.0", "I0.0"
/// - OPC UA: "ns=2;s=Temperature"
///
/// **Rust知识点**:
/// - 结构体字段都是公开的，便于外部构造和访问
/// - Option<T>表示可选字段，数组长度可能不需要
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadRequest {
    /// 请求唯一标识符
    /// **业务含义**: 用于跟踪和关联请求与响应
    /// **生成方式**: 通常使用UUID或递增序号
    /// **用途**: 异步操作的请求-响应匹配、日志关联
    pub id: String,

    /// PLC数据地址
    /// **业务含义**: 指定要读取的PLC内存地址
    /// **格式依赖**: 格式取决于具体的PLC协议
    /// **示例**:
    /// - Modbus: "40001"(保持寄存器), "00001"(线圈)
    /// - Siemens: "DB1.DBD0"(数据块), "M0.0"(标志位)
    pub address: String,

    /// 期望的数据类型
    /// **业务含义**: 指定如何解释读取的原始数据
    /// **类型安全**: 确保数据按正确类型解析
    /// **支持类型**: Bool, Int16, Int32, Float32, String等
    /// **重要性**: 错误的数据类型会导致数据解析错误
    pub data_type: PlcDataType,

    /// 数组长度（用于批量读取）
    /// **业务含义**: 指定要读取的数组元素个数
    /// **可选性**: 单值读取时为None，数组读取时指定长度
    /// **性能优化**: 批量读取比多次单值读取更高效
    /// **限制**: 长度不能超过PLC和协议的限制
    pub array_length: Option<u32>,
}

/// PLC数据写入请求
///
/// **业务作用**:
/// - 封装PLC数据写入操作的所有必要参数
/// - 提供统一的写入请求格式，支持不同协议
/// - 支持多种数据类型的写入操作
/// - 便于请求的序列化、传输和审计记录
///
/// **设计特点**:
/// - **类型安全**: 通过PlcValue枚举确保类型正确性
/// - **地址抽象**: 使用字符串地址，支持不同的地址格式
/// - **值封装**: PlcValue统一封装不同类型的数据
/// - **可追踪**: 每个请求都有唯一ID便于审计
///
/// **使用流程**:
/// 1. 创建写入请求，指定地址和要写入的值
/// 2. 通过PLC服务执行写入操作
/// 3. 获得WriteResult结果
/// 4. 处理写入结果或错误
///
/// **安全考虑**:
/// - 写入操作可能影响设备状态，需要权限验证
/// - 应该记录所有写入操作用于审计
/// - 需要验证地址的可写性
/// - 考虑并发写入的冲突处理
///
/// **性能考虑**:
/// - 批量写入比多次单值写入更高效
/// - 写入操作通常比读取操作耗时更长
/// - 需要考虑写入确认和重试机制
///
/// **Rust知识点**:
/// - PlcValue枚举提供类型安全的值表示
/// - 结构体设计简洁，便于使用和维护
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WriteRequest {
    /// 请求唯一标识符
    /// **业务含义**: 用于跟踪和关联请求与响应
    /// **审计价值**: 写入操作的审计跟踪
    /// **生成方式**: 通常使用UUID确保唯一性
    /// **用途**: 异步操作匹配、日志关联、错误追踪
    pub id: String,

    /// PLC数据地址
    /// **业务含义**: 指定要写入的PLC内存地址
    /// **格式依赖**: 格式取决于具体的PLC协议
    /// **可写性**: 需要确保地址是可写的
    /// **示例**:
    /// - Modbus: "40001"(保持寄存器), "00001"(线圈)
    /// - Siemens: "DB1.DBD0"(数据块), "Q0.0"(输出)
    pub address: String,

    /// 要写入的数据值
    /// **业务含义**: 封装要写入PLC的具体数值
    /// **类型安全**: PlcValue枚举确保类型匹配
    /// **支持类型**: Bool, Int16, Int32, Float32, String, ByteArray等
    /// **验证**: 写入前应验证值的有效性和范围
    pub value: PlcValue,
}

/// 读取结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadResult {
    /// 请求ID
    pub request_id: String,
    
    /// 是否成功
    pub success: bool,
    
    /// 读取的值
    pub value: Option<PlcValue>,
    
    /// 错误信息
    pub error_message: Option<String>,
    
    /// 执行时间（毫秒）
    pub execution_time_ms: u64,
}

/// 写入结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WriteResult {
    /// 请求ID
    pub request_id: String,
    
    /// 是否成功
    pub success: bool,
    
    /// 错误信息
    pub error_message: Option<String>,
    
    /// 执行时间（毫秒）
    pub execution_time_ms: u64,
}

/// PLC数据类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlcDataType {
    Bool,
    Int16,
    Int32,
    Float32,
    String,
    ByteArray,
}

/// PLC数据值枚举
///
/// **业务作用**:
/// - 统一表示PLC中的各种数据类型
/// - 提供类型安全的数据封装和传输
/// - 支持不同PLC协议的数据类型映射
/// - 便于数据的序列化和反序列化
///
/// **设计理念**:
/// - **类型安全**: 编译时确保数据类型正确性
/// - **统一接口**: 为不同数据类型提供统一的处理方式
/// - **可扩展性**: 易于添加新的数据类型支持
/// - **序列化友好**: 支持JSON等格式的序列化
///
/// **数据类型映射**:
/// - **Bool**: 对应PLC中的位/线圈/离散量
/// - **Int16**: 对应PLC中的16位整数/字
/// - **Int32**: 对应PLC中的32位整数/双字
/// - **Float32**: 对应PLC中的32位浮点数/实数
/// - **String**: 对应PLC中的字符串数据
/// - **ByteArray**: 对应PLC中的原始字节数据
/// - **Array**: 对应PLC中的数组数据
///
/// **使用场景**:
/// - PLC数据的读取结果封装
/// - PLC数据的写入参数传递
/// - 不同协议间的数据转换
/// - 数据的网络传输和存储
///
/// **Rust知识点**:
/// - `enum`: Rust的代数数据类型，支持不同变体
/// - 每个变体可以携带不同类型的数据
/// - `Clone`: 支持值的深拷贝
/// - `Serialize/Deserialize`: 支持serde序列化
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PlcValue {
    /// 布尔值类型
    /// **PLC对应**: 线圈(Coil)、离散输入(Discrete Input)、位标志
    /// **用途**: 开关状态、报警信号、控制标志等
    /// **示例**: 电机运行状态、阀门开关状态
    Bool(bool),

    /// 16位有符号整数
    /// **PLC对应**: 字(Word)、16位寄存器
    /// **范围**: -32,768 到 32,767
    /// **用途**: 计数值、状态码、小范围数值
    Int16(i16),

    /// 32位有符号整数
    /// **PLC对应**: 双字(DWord)、32位寄存器
    /// **范围**: -2,147,483,648 到 2,147,483,647
    /// **用途**: 大数值、时间戳、累计值
    Int32(i32),

    /// 32位浮点数
    /// **PLC对应**: 实数(Real)、浮点寄存器
    /// **精度**: IEEE 754单精度浮点数
    /// **用途**: 模拟量数值、传感器读数、设定值
    /// **示例**: 温度、压力、流量等物理量
    Float32(f32),

    /// 字符串类型
    /// **PLC对应**: 字符串变量、文本数据
    /// **编码**: 通常使用ASCII或UTF-8编码
    /// **用途**: 设备名称、状态描述、配置信息
    String(String),

    /// 字节数组类型
    /// **PLC对应**: 原始字节数据、二进制数据
    /// **用途**: 复杂数据结构、文件数据、协议数据
    /// **灵活性**: 可以表示任意二进制数据
    ByteArray(Vec<u8>),

    /// 数组类型
    /// **PLC对应**: 数组变量、批量数据
    /// **嵌套性**: 可以包含任意类型的PlcValue
    /// **用途**: 批量读写、表格数据、配置数组
    /// **递归性**: 支持多维数组和复杂数据结构
    Array(Vec<PlcValue>),
}

/// 连接统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionStats {
    /// 连接ID
    pub connection_id: String,
    
    /// 总读取次数
    pub total_reads: u64,
    
    /// 总写入次数
    pub total_writes: u64,
    
    /// 成功读取次数
    pub successful_reads: u64,
    
    /// 成功写入次数
    pub successful_writes: u64,
    
    /// 平均读取时间（毫秒）
    pub average_read_time_ms: f64,
    
    /// 平均写入时间（毫秒）
    pub average_write_time_ms: f64,
    
    /// 连接建立时间
    pub connection_established_at: DateTime<Utc>,
    
    /// 最后通信时间
    pub last_communication: DateTime<Utc>,
    
    /// 连接错误次数
    pub connection_errors: u64,
}

/// 连接测试结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionTestResult {
    /// 是否成功
    pub success: bool,
    
    /// 连接时间（毫秒）
    pub connection_time_ms: u64,
    
    /// 错误信息
    pub error_message: Option<String>,
    
    /// 协议版本信息
    pub protocol_info: Option<String>,
    
    /// 设备信息
    pub device_info: Option<String>,
}