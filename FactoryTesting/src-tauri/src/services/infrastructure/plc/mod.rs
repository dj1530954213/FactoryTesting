/// PLC通信相关模块

/// PLC通信服务接口定义
pub mod plc_communication_service;

/// Mock PLC服务实现（用于开发和测试）
pub mod mock_plc_service;

/// 单元测试模块
#[cfg(test)]
pub mod tests;

// 为后续步骤准备的真实PLC实现（暂时注释）
pub mod modbus_plc_service;
// pub mod s7_plc_service;
// pub mod opcua_plc_service;

// 重新导出主要接口和类型
pub use plc_communication_service::*;
pub use mock_plc_service::*;
pub use modbus_plc_service::{ModbusPlcService, ModbusConfig}; 