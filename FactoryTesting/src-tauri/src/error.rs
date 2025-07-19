/// 错误处理模块
/// 
/// 业务说明：
/// 本模块是应用程序错误处理的统一入口点
/// 通过重新导出utils::error中的所有错误类型，简化了错误类型的导入路径
/// 使得其他模块可以通过 use crate::error::* 来使用所有错误相关的类型
/// 
/// 架构设计：
/// - 采用集中式错误管理，所有错误类型定义在utils::error中
/// - 通过本模块提供更短的导入路径，提高代码可读性
/// - 支持统一的错误处理和转换机制
/// 
/// 使用示例：
/// ```rust
/// use crate::error::{AppError, AppResult};
/// 
/// fn some_function() -> AppResult<String> {
///     // 使用AppError和AppResult进行错误处理
///     Ok("success".to_string())
/// }
/// ```
/// 
/// 调用链：
/// 其他模块 -> error模块 -> utils::error实际定义
/// 
/// Rust知识点：
/// - pub use 语句用于重新导出其他模块的内容
/// - * 通配符导出模块中的所有公开项
/// - 这种模式常用于创建更简洁的API接口

pub use crate::utils::error::*; 
