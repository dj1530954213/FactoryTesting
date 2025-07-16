use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio_modbus;
use rust_xlsxwriter;

/// 应用程序统一错误类型
/// 用于封装系统中可能出现的各种错误，提供统一的错误处理机制
#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum AppError {
    /// 通用错误，包含错误消息
    #[error("通用错误: {message}")]
    Generic { message: String },

    /// 输入/输出错误
    #[error("IO错误: {message} (Kind: {kind})")]
    IoError { message: String, kind: String },

    /// 数据持久化相关错误
    #[error("持久化错误: {message}")]
    PersistenceError { message: String },

    /// PLC通信相关错误
    ///
    /// **业务含义**: 表示与PLC设备通信过程中发生的各种错误
    /// **错误类型**:
    /// - 连接建立失败（网络不通、设备离线等）
    /// - 通信超时（设备响应慢、网络延迟等）
    /// - 协议错误（数据格式错误、功能码不支持等）
    /// - 地址错误（无效地址、权限不足等）
    /// - 数据类型错误（类型不匹配、范围超限等）
    ///
    /// **错误恢复**:
    /// - 自动重试机制：短暂的网络问题
    /// - 连接重建：连接断开的情况
    /// - 降级处理：使用缓存数据或默认值
    /// - 用户通知：严重错误需要人工干预
    ///
    /// **日志记录**:
    /// - 记录详细的错误上下文信息
    /// - 包含PLC地址、操作类型、错误时间等
    /// - 便于故障诊断和性能分析
    #[error("PLC通信错误: {message}")]
    PlcCommunicationError { message: String },

    /// 数据序列化/反序列化错误
    #[error("序列化错误: {message}")]
    SerializationError { message: String },

    /// 配置相关错误
    #[error("配置错误: {message}")]
    ConfigurationError { message: String },

    /// 验证错误（数据验证失败）
    #[error("验证错误: {message}")]
    ValidationError { message: String },

    /// 并发/异步操作错误
    #[error("并发错误: {message}")]
    ConcurrencyError { message: String },

    /// 资源未找到错误
    #[error("资源未找到: {resource_type} - {message}")]
    NotFoundError {
        resource_type: String,
        message: String,
    },

    /// 权限不足错误
    #[error("权限不足: {message}")]
    PermissionError { message: String },

    /// 超时错误
    #[error("操作超时: {operation} - {message}")]
    TimeoutError {
        operation: String,
        message: String,
    },

    /// 网络相关错误
    #[error("网络错误: {message}")]
    NetworkError { message: String },

    /// 业务逻辑错误
    #[error("业务逻辑错误: {message}")]
    BusinessLogicError { message: String },

    /// 测试执行相关错误
    #[error("测试执行错误: {test_type} - {message}")]
    TestExecutionError {
        test_type: String,
        message: String,
    },

    /// 状态转换错误
    #[error("状态转换错误: 从 {from_state} 到 {to_state} - {message}")]
    StateTransitionError {
        from_state: String,
        to_state: String,
        message: String,
    },

    /// 未知错误
    #[error("未知错误: {message}")]
    UnknownError { message: String },

    /// 服务未找到错误
    #[error("服务未找到: {service_name}")]
    ServiceNotFound { service_name: String },

    /// 服务初始化失败错误
    #[error("服务初始化失败: {service_name}, 原因: {reason}")]
    ServiceInitializationError { service_name: String, reason: String },

    /// 服务关闭失败错误
    #[error("服务关闭失败: {service_name}, 原因: {reason}")]
    ServiceShutdownError { service_name: String, reason: String },

    /// 服务健康检查失败错误
    #[error("服务健康检查失败: {service_name}, 原因: {reason}")]
    ServiceHealthCheckError { service_name: String, reason: String },

    /// JSON序列化/反序列化错误
    #[error("JSON序列化/反序列化错误: {message}")]
    JsonError { message: String },

    /// 未实现的功能错误
    #[error("未实现的功能: {feature_name}")]
    NotImplemented { feature_name: String },

    /// PDF生成错误
    #[error("PDF生成错误: {message}")]
    PdfError { message: String },

    /// Excel生成错误
    #[error("Excel生成错误: {message}")]
    ExcelError { message: String },

    /// 模板引擎错误
    #[error("模板引擎错误: {message}")]
    TemplateError { message: String },

    /// 报告生成错误
    #[error("报告生成错误: {message}")]
    ReportGenerationError { message: String },

    /// 数据分析错误
    #[error("数据分析错误: {message}")]
    AnalysisError { message: String },

    /// 配置管理错误
    #[error("配置管理错误: {message}")]
    ConfigManagementError { message: String },

    /// 用户认证错误
    #[error("用户认证错误: {message}")]
    AuthenticationError { message: String },

    /// 权限验证错误
    #[error("权限验证错误: {message}")]
    AuthorizationError { message: String },

    /// Mock错误（仅用于测试）
    #[error("Mock错误: {0}")]
    MockError(String),

    /// 依赖注入错误
    #[error("依赖注入错误: {0}")]
    DependencyInjectionError(String),
}

impl AppError {
    /// 创建通用错误
    pub fn generic(message: impl Into<String>) -> Self {
        Self::Generic {
            message: message.into(),
        }
    }

    /// 创建IO错误
    pub fn io_error(message: impl Into<String>, kind_str: impl Into<String>) -> Self {
        Self::IoError {
            message: message.into(),
            kind: kind_str.into(),
        }
    }

    /// 创建持久化错误
    pub fn persistence_error(message: impl Into<String>) -> Self {
        Self::PersistenceError {
            message: message.into(),
        }
    }

    /// 创建PLC通信错误
    ///
    /// **业务作用**: 为PLC通信相关的错误创建统一的错误实例
    /// **使用场景**:
    /// - PLC连接建立失败时
    /// - 数据读写操作失败时
    /// - 协议解析错误时
    /// - 设备响应超时时
    ///
    /// **参数**: `message` - 错误描述信息，支持任何可转换为String的类型
    /// **返回值**: AppError::PlcCommunicationError实例
    ///
    /// **使用示例**:
    /// ```rust
    /// return Err(AppError::plc_communication_error("连接PLC失败: 设备不在线"));
    /// ```
    ///
    /// **错误信息规范**:
    /// - 包含具体的错误原因
    /// - 提供足够的上下文信息
    /// - 使用用户友好的语言描述
    /// - 便于故障排查和问题定位
    ///
    /// **Rust知识点**:
    /// - `impl Into<String>`: 泛型约束，接受任何可转换为String的类型
    /// - `Into<String>`: trait，提供类型转换能力
    /// - `message.into()`: 调用Into trait进行类型转换
    pub fn plc_communication_error(message: impl Into<String>) -> Self {
        Self::PlcCommunicationError {
            message: message.into(), // 将输入转换为String类型
        }
    }

    /// 创建序列化错误
    pub fn serialization_error(message: impl Into<String>) -> Self {
        Self::SerializationError {
            message: message.into(),
        }
    }

    /// 创建配置错误
    pub fn configuration_error(message: impl Into<String>) -> Self {
        Self::ConfigurationError {
            message: message.into(),
        }
    }

    /// 创建验证错误
    pub fn validation_error(message: impl Into<String>) -> Self {
        Self::ValidationError {
            message: message.into(),
        }
    }

    /// 创建并发错误
    pub fn concurrency_error(message: impl Into<String>) -> Self {
        Self::ConcurrencyError {
            message: message.into(),
        }
    }

    /// 创建资源未找到错误
    pub fn not_found_error(resource_type: impl Into<String>, message: impl Into<String>) -> Self {
        Self::NotFoundError {
            resource_type: resource_type.into(),
            message: message.into(),
        }
    }

    /// 创建权限错误
    pub fn permission_error(message: impl Into<String>) -> Self {
        Self::PermissionError {
            message: message.into(),
        }
    }

    /// 创建超时错误
    pub fn timeout_error(operation: impl Into<String>, message: impl Into<String>) -> Self {
        Self::TimeoutError {
            operation: operation.into(),
            message: message.into(),
        }
    }

    /// 创建网络错误
    pub fn network_error(message: impl Into<String>) -> Self {
        Self::NetworkError {
            message: message.into(),
        }
    }

    /// 创建业务逻辑错误
    pub fn business_logic_error(message: impl Into<String>) -> Self {
        Self::BusinessLogicError {
            message: message.into(),
        }
    }

    /// 创建测试执行错误
    pub fn test_execution_error(test_type: impl Into<String>, message: impl Into<String>) -> Self {
        Self::TestExecutionError {
            test_type: test_type.into(),
            message: message.into(),
        }
    }

    /// 创建状态转换错误
    pub fn state_transition_error(
        from_state: impl Into<String>,
        to_state: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self::StateTransitionError {
            from_state: from_state.into(),
            to_state: to_state.into(),
            message: message.into(),
        }
    }

    /// 创建服务未找到错误
    pub fn service_not_found_error(service_name: impl Into<String>) -> Self {
        Self::ServiceNotFound {
            service_name: service_name.into(),
        }
    }

    /// 创建服务初始化失败错误
    pub fn service_initialization_error(service_name: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::ServiceInitializationError {
            service_name: service_name.into(),
            reason: reason.into(),
        }
    }

    /// 创建服务关闭失败错误
    pub fn service_shutdown_error(service_name: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::ServiceShutdownError {
            service_name: service_name.into(),
            reason: reason.into(),
        }
    }

    /// 创建服务健康检查失败错误
    pub fn service_health_check_error(service_name: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::ServiceHealthCheckError {
            service_name: service_name.into(),
            reason: reason.into(),
        }
    }

    /// 创建JSON序列化错误
    pub fn json_error(message: impl Into<String>) -> Self {
        Self::JsonError {
            message: message.into(),
        }
    }

    /// 创建未实现的功能错误
    pub fn not_implemented_error(feature_name: impl Into<String>) -> Self {
        Self::NotImplemented {
            feature_name: feature_name.into(),
        }
    }

    /// 创建PDF生成错误
    pub fn pdf_error(message: impl Into<String>) -> Self {
        Self::PdfError {
            message: message.into(),
        }
    }

    /// 创建Excel生成错误
    pub fn excel_error(message: impl Into<String>) -> Self {
        Self::ExcelError {
            message: message.into(),
        }
    }

    /// 创建模板引擎错误
    pub fn template_error(message: impl Into<String>) -> Self {
        Self::TemplateError {
            message: message.into(),
        }
    }

    /// 创建报告生成错误
    pub fn report_generation_error(message: impl Into<String>) -> Self {
        Self::ReportGenerationError {
            message: message.into(),
        }
    }

    /// 创建数据分析错误
    pub fn analysis_error(message: impl Into<String>) -> Self {
        Self::AnalysisError {
            message: message.into(),
        }
    }

    /// 创建配置管理错误
    pub fn config_management_error(message: impl Into<String>) -> Self {
        Self::ConfigManagementError {
            message: message.into(),
        }
    }

    /// 创建用户认证错误
    pub fn authentication_error(message: impl Into<String>) -> Self {
        Self::AuthenticationError {
            message: message.into(),
        }
    }

    /// 创建权限验证错误
    pub fn authorization_error(message: impl Into<String>) -> Self {
        Self::AuthorizationError {
            message: message.into(),
        }
    }

    /// 获取错误的简短描述
    pub fn error_code(&self) -> &'static str {
        match self {
            AppError::Generic { .. } => "GENERIC",
            AppError::IoError { .. } => "IO_ERROR",
            AppError::PersistenceError { .. } => "PERSISTENCE_ERROR",
            AppError::PlcCommunicationError { .. } => "PLC_COMMUNICATION_ERROR",
            AppError::SerializationError { .. } => "SERIALIZATION_ERROR",
            AppError::ConfigurationError { .. } => "CONFIGURATION_ERROR",
            AppError::ValidationError { .. } => "VALIDATION_ERROR",
            AppError::ConcurrencyError { .. } => "CONCURRENCY_ERROR",
            AppError::NotFoundError { .. } => "NOT_FOUND_ERROR",
            AppError::PermissionError { .. } => "PERMISSION_ERROR",
            AppError::TimeoutError { .. } => "TIMEOUT_ERROR",
            AppError::NetworkError { .. } => "NETWORK_ERROR",
            AppError::BusinessLogicError { .. } => "BUSINESS_LOGIC_ERROR",
            AppError::TestExecutionError { .. } => "TEST_EXECUTION_ERROR",
            AppError::StateTransitionError { .. } => "STATE_TRANSITION_ERROR",
            AppError::ServiceNotFound { .. } => "SERVICE_NOT_FOUND",
            AppError::ServiceInitializationError { .. } => "SERVICE_INIT_ERROR",
            AppError::ServiceShutdownError { .. } => "SERVICE_SHUTDOWN_ERROR",
            AppError::ServiceHealthCheckError { .. } => "SERVICE_HEALTH_CHECK_ERROR",
            AppError::JsonError { .. } => "JSON_ERROR",
            AppError::NotImplemented { .. } => "NOT_IMPLEMENTED_ERROR",
            AppError::PdfError { .. } => "PDF_ERROR",
            AppError::ExcelError { .. } => "EXCEL_ERROR",
            AppError::TemplateError { .. } => "TEMPLATE_ERROR",
            AppError::ReportGenerationError { .. } => "REPORT_GENERATION_ERROR",
            AppError::AnalysisError { .. } => "ANALYSIS_ERROR",
            AppError::ConfigManagementError { .. } => "CONFIG_MANAGEMENT_ERROR",
            AppError::AuthenticationError { .. } => "AUTHENTICATION_ERROR",
            AppError::AuthorizationError { .. } => "AUTHORIZATION_ERROR",
            AppError::UnknownError { .. } => "UNKNOWN_ERROR",
            AppError::MockError(..) => "MOCK_ERROR",
            AppError::DependencyInjectionError(..) => "DEPENDENCY_INJECTION_ERROR",
        }
    }
}

/// 标准 I/O 错误到 AppError 的转换
impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        AppError::IoError { message: err.to_string(), kind: format!("{:?}", err.kind()) }
    }
}

/// serde_json 错误到 AppError 的转换
impl From<serde_json::Error> for AppError {
    fn from(err: serde_json::Error) -> Self {
        AppError::JsonError { message: err.to_string() }
    }
}

/// 字符串错误到 AppError 的转换（通用错误）
impl From<String> for AppError {
    fn from(err_msg: String) -> Self {
        Self::Generic { message: err_msg }
    }
}

/// &str 错误到 AppError 的转换（通用错误）
impl From<&str> for AppError {
    fn from(err_msg: &str) -> Self {
        Self::Generic { message: err_msg.to_string() }
    }
}

/// 应用程序结果类型别名
/// 简化错误处理的类型定义
pub type AppResult<T> = Result<T, AppError>;

/// tokio_modbus 错误到 AppError 的转换
///
/// **业务作用**: 将底层Modbus库的错误转换为应用层统一的错误类型
/// **错误传播**: 实现错误的自动转换，简化错误处理代码
///
/// **转换逻辑**:
/// - 将tokio_modbus::Error包装为PlcCommunicationError
/// - 保留原始错误信息，便于调试和故障排查
/// - 提供统一的错误处理接口
///
/// **使用场景**:
/// - Modbus TCP连接失败
/// - 数据读写操作异常
/// - 协议解析错误
/// - 网络通信超时
///
/// **错误恢复策略**:
/// - 连接错误：尝试重新连接
/// - 超时错误：增加超时时间或重试
/// - 协议错误：检查地址和数据类型
/// - 设备错误：检查设备状态和配置
///
/// **Rust知识点**:
/// - `From<T>` trait: 提供类型转换功能
/// - `format!`: 字符串格式化宏
/// - 自动错误转换：使用`?`操作符时自动调用
impl From<tokio_modbus::Error> for AppError {
    fn from(err: tokio_modbus::Error) -> Self {
        // 将Modbus错误包装为PLC通信错误
        // **错误上下文**: 添加"Modbus error:"前缀，明确错误来源
        // **信息保留**: 保留原始错误的详细信息
        AppError::PlcCommunicationError {
            message: format!("Modbus error: {}", err)
        }
    }
}

/// rust_xlsxwriter 错误到 AppError 的转换
impl From<rust_xlsxwriter::XlsxError> for AppError {
    fn from(err: rust_xlsxwriter::XlsxError) -> Self {
        AppError::ExcelError { message: format!("Excel error: {}", err) }
    }
}
