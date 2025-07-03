//! 领域层模块
//!
//! 包含业务逻辑和领域服务接口定义

pub mod services;
pub mod impls;

// 重新导出领域服务
// 仅重新导出服务接口及兼容别名；
pub use services::*;

// 为过渡期显式重新导出实现模块，避免修改大量调用代码
pub use impls::{
    specific_test_executors,
    test_plc_config_service,
    plc_connection_manager,
};
