use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio_modbus;

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
    pub fn plc_communication_error(message: impl Into<String>) -> Self {
        Self::PlcCommunicationError {
            message: message.into(),
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
            AppError::UnknownError { .. } => "UNKNOWN_ERROR",
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
        AppError::UnknownError { message: err_msg }
    }
}

/// &str 错误到 AppError 的转换（通用错误）
impl From<&str> for AppError {
    fn from(err_msg: &str) -> Self {
        AppError::UnknownError { message: err_msg.to_string() }
    }
}

/// 应用程序结果类型别名
/// 简化错误处理的类型定义
pub type AppResult<T> = Result<T, AppError>;

/// tokio_modbus 错误到 AppError 的转换
impl From<tokio_modbus::Error> for AppError {
    fn from(err: tokio_modbus::Error) -> Self {
        AppError::PlcCommunicationError { message: format!("Modbus error: {}", err) }
    }
} 