/// PLC通信相关模块

/// PLC通信服务接口定义
pub mod plc_communication_service;

/// Modbus PLC服务实现
pub mod modbus_plc_service;


// pub mod s7_plc_service;
// pub mod opcua_plc_service;

// 重新导出主要接口和类型
pub use plc_communication_service::*;
pub use modbus_plc_service::{ModbusPlcService, ModbusConfig};