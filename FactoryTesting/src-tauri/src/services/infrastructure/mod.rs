/// 基础设施层服务模块
/// 负责与外部系统的交互，如PLC通信、数据持久化等

/// PLC通信相关模块
pub mod plc;

/// 数据持久化相关模块
pub mod persistence;

// 为后续步骤准备的模块（暂时注释）
// pub mod excel;

// 重新导出常用接口和实现
pub use plc::*; 
pub use persistence::*; 