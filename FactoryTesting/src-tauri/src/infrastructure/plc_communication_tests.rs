//! PLC通信服务测试
//!
//! 测试Modbus TCP PLC通信服务的功能

#[cfg(test)]
mod tests {
    use super::super::plc_communication::{
        ModbusTcpPlcService,
        IPlcCommunicationService,
        parse_modbus_address,
        ModbusRegisterType,
        f32_to_registers,
        registers_to_f32,
        i32_to_registers,
        registers_to_i32,
    };
    use crate::domain::services::{BaseService, PlcConnectionConfig, PlcProtocol};
    use std::collections::HashMap;
    use tokio;

    /// 创建测试用的PLC连接配置
    fn create_test_config() -> PlcConnectionConfig {
        let mut protocol_params = HashMap::new();
        protocol_params.insert("slave_id".to_string(), serde_json::Value::Number(serde_json::Number::from(1)));

        PlcConnectionConfig {
            id: "test_plc".to_string(),
            name: "测试PLC".to_string(),
            protocol: PlcProtocol::ModbusTcp,
            host: "127.0.0.1".to_string(),
            port: 502,
            timeout_ms: 5000,
            read_timeout_ms: 1000,
            write_timeout_ms: 1000,
            retry_count: 3,
            retry_interval_ms: 100,
            protocol_params,
        }
    }

    #[tokio::test]
    async fn test_service_initialization() {
        let mut service = ModbusTcpPlcService::new();

        // 测试初始化
        let result = service.initialize().await;
        assert!(result.is_ok(), "服务初始化应该成功");

        // 测试健康检查
        let health_result = service.health_check().await;
        assert!(health_result.is_ok(), "健康检查应该成功");

        // 测试关闭
        let shutdown_result = service.shutdown().await;
        assert!(shutdown_result.is_ok(), "服务关闭应该成功");
    }

    #[tokio::test]
    async fn test_connection_config_validation() {
        let service = ModbusTcpPlcService::new();
        let config = create_test_config();

        // 测试连接配置验证（这会失败，因为没有真实的PLC服务器）
        let test_result = service.test_connection(&config).await;
        assert!(test_result.is_ok(), "测试连接应该返回结果");

        let connection_test = test_result.unwrap();
        assert!(!connection_test.success, "连接到不存在的PLC应该失败");
        assert!(connection_test.error_message.is_some(), "应该有错误信息");
    }

    #[tokio::test]
    async fn test_address_parsing() {
        // 测试地址解析函数

        // 测试线圈地址
        let result = parse_modbus_address("00001");
        assert!(result.is_ok());
        let (reg_type, offset) = result.unwrap();
        assert_eq!(reg_type, ModbusRegisterType::Coil);
        assert_eq!(offset, 0);

        // 测试保持寄存器地址
        let result = parse_modbus_address("40001");
        assert!(result.is_ok());
        let (reg_type, offset) = result.unwrap();
        assert_eq!(reg_type, ModbusRegisterType::HoldingRegister);
        assert_eq!(offset, 0);

        // 测试输入寄存器地址
        let result = parse_modbus_address("30100");
        assert!(result.is_ok());
        let (reg_type, offset) = result.unwrap();
        assert_eq!(reg_type, ModbusRegisterType::InputRegister);
        assert_eq!(offset, 99);

        // 测试离散输入地址
        let result = parse_modbus_address("10001");
        assert!(result.is_ok());
        let (reg_type, offset) = result.unwrap();
        assert_eq!(reg_type, ModbusRegisterType::DiscreteInput);
        assert_eq!(offset, 0);

        // 测试无效地址
        let result = parse_modbus_address("");
        assert!(result.is_err());

        let result = parse_modbus_address("5001");
        assert!(result.is_err());

        let result = parse_modbus_address("4abc");
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_data_conversion() {
        // 测试f32转换
        let test_value = 123.456f32;
        let registers = f32_to_registers(test_value);
        let converted_back = registers_to_f32(&registers);

        // 由于浮点数精度问题，使用近似比较
        assert!((test_value - converted_back).abs() < 0.001,
                "f32转换应该保持精度: {} != {}", test_value, converted_back);

        // 测试i32转换
        let test_value = 123456i32;
        let registers = i32_to_registers(test_value);
        let converted_back = registers_to_i32(&registers);

        assert_eq!(test_value, converted_back, "i32转换应该精确");

        // 测试边界值
        let test_value = f32::MAX;
        let registers = f32_to_registers(test_value);
        let converted_back = registers_to_f32(&registers);
        assert_eq!(test_value, converted_back, "f32最大值转换应该精确");

        let test_value = i32::MIN;
        let registers = i32_to_registers(test_value);
        let converted_back = registers_to_i32(&registers);
        assert_eq!(test_value, converted_back, "i32最小值转换应该精确");
    }

    #[tokio::test]
    async fn test_connection_pool_behavior() {
        let service = ModbusTcpPlcService::new();
        let config = create_test_config();

        // 尝试连接（会失败，但测试连接池逻辑）
        let result1 = service.connect(&config).await;
        assert!(result1.is_err(), "连接到不存在的PLC应该失败");

        // 测试重复连接配置
        let result2 = service.connect(&config).await;
        assert!(result2.is_err(), "重复连接到不存在的PLC应该失败");
    }

    #[tokio::test]
    async fn test_batch_operations() {
        use crate::domain::services::{ReadRequest, WriteRequest, PlcDataType, PlcValue};

        let service = ModbusTcpPlcService::new();
        let config = create_test_config();

        // 创建一个假的连接句柄（实际测试中这会失败）
        let handle = crate::domain::services::ConnectionHandle {
            connection_id: "test".to_string(),
            handle_id: "test_handle".to_string(),
            protocol: PlcProtocol::ModbusTcp,
            created_at: chrono::Utc::now(),
            last_activity: chrono::Utc::now(),
        };

        // 测试批量读取
        let read_requests = vec![
            ReadRequest {
                id: "req1".to_string(),
                address: "40001".to_string(),
                data_type: PlcDataType::Float32,
                array_length: None,
            },
            ReadRequest {
                id: "req2".to_string(),
                address: "00001".to_string(),
                data_type: PlcDataType::Bool,
                array_length: None,
            },
        ];

        let result = service.batch_read(&handle, &read_requests).await;
        assert!(result.is_err(), "批量读取应该失败（没有真实连接）");

        // 测试批量写入
        let write_requests = vec![
            WriteRequest {
                id: "write1".to_string(),
                address: "40001".to_string(),
                value: PlcValue::Float32(123.45),
            },
            WriteRequest {
                id: "write2".to_string(),
                address: "00001".to_string(),
                value: PlcValue::Bool(true),
            },
        ];

        let result = service.batch_write(&handle, &write_requests).await;
        assert!(result.is_err(), "批量写入应该失败（没有真实连接）");
    }

    #[tokio::test]
    async fn test_error_handling() {
        let service = ModbusTcpPlcService::new();

        // 测试无效配置
        let mut invalid_config = create_test_config();
        invalid_config.host = "invalid_host_name_that_does_not_exist".to_string();

        let result = service.test_connection(&invalid_config).await;
        assert!(result.is_ok(), "测试连接应该返回结果");

        let connection_test = result.unwrap();
        assert!(!connection_test.success, "连接到无效主机应该失败");
        assert!(connection_test.error_message.is_some(), "应该有错误信息");

        // 测试无效端口
        invalid_config.host = "127.0.0.1".to_string();
        invalid_config.port = 65535; // 无效端口

        let result = service.test_connection(&invalid_config).await;
        assert!(result.is_ok(), "测试连接应该返回结果");

        let connection_test = result.unwrap();
        assert!(!connection_test.success, "连接到无效端口应该失败");
    }
}
