// 文件: FactoryTesting/src-tauri/src/services/infrastructure/plc/tests.rs
// 详细注释：PLC服务相关的单元测试

#[cfg(test)]
mod tests {
    // 从父模块 (services::infrastructure::plc) 的子模块导入
    use crate::services::infrastructure::plc::mock_plc_service::MockPlcService;
    use crate::services::infrastructure::plc::plc_communication_service::{
        PlcCommunicationService, // Trait
        PlcTag,
        PlcDataType,
        PlcConnectionStatus,
        PlcCommunicationStats,
    };

    // 从项目其他地方导入
    use crate::services::traits::BaseService; // Trait
    use std::collections::HashMap;
    use serde_json::Value; // mock_plc_service.rs 的 preset_read_value 使用了 Value，所以测试中也可能需要
    use crate::utils::error::AppError;

    // 定义一个小的容差用于浮点数比较
    const FLOAT_COMPARISON_TOLERANCE_F32: f32 = 1e-5;
    const FLOAT_COMPARISON_TOLERANCE_F64: f64 = 1e-5;

    /// 测试Mock PLC服务的基本功能
    #[tokio::test]
    async fn test_mock_plc_service_basic_operations() {
        let mut service = MockPlcService::new_for_testing("TestPLC");
        
        // 测试初始化
        service.initialize().await.unwrap();
        assert_eq!(service.service_name(), "MockPlcService");
        assert!(!service.is_connected());
        
        // 测试连接
        service.connect().await.unwrap();
        assert!(service.is_connected());
        assert!(service.test_connection().await.unwrap());
        
        // 测试健康检查
        service.health_check().await.unwrap();
        
        // 测试断开连接
        service.disconnect().await.unwrap();
        assert!(!service.is_connected());
        
        // 测试关闭服务
        service.shutdown().await.unwrap();
    }

    /// 测试布尔值读写操作
    #[tokio::test]
    async fn test_bool_read_write() {
        let mut service = MockPlcService::new_for_testing("TestPLC");
        service.connect().await.unwrap();
        
        let address = "DB1.DBX0.0";
        
        // 写入布尔值
        service.write_bool(address, true).await.unwrap();
        
        // 读取布尔值
        let value = service.read_bool(address).await.unwrap();
        assert_eq!(value, true);
        
        // 验证写入日志
        assert!(service.was_address_written(address));
        let last_write = service.get_last_write().unwrap();
        assert_eq!(last_write.address, address);
        assert_eq!(last_write.value, Value::Bool(true));
        assert_eq!(last_write.operation_type, "write_bool");
    }

    /// 测试整数读写操作
    #[tokio::test]
    async fn test_integer_read_write() {
        let mut service = MockPlcService::new_for_testing("TestPLC");
        service.connect().await.unwrap();
        
        // 测试 int32
        let address_int32 = "DB1.DBD0";
        service.write_int32(address_int32, 12345).await.unwrap();
        let value_int32 = service.read_int32(address_int32).await.unwrap();
        assert_eq!(value_int32, 12345);
        
        // 测试 uint16
        let address_uint16 = "DB1.DBW4";
        service.write_uint16(address_uint16, 65535).await.unwrap();
        let value_uint16 = service.read_uint16(address_uint16).await.unwrap();
        assert_eq!(value_uint16, 65535);
        
        // 测试便捷方法
        let address_generic = "DB1.DBD8";
        service.write_int(address_generic, 999999).await.unwrap();
        let value_generic = service.read_int(address_generic).await.unwrap();
        assert_eq!(value_generic, 999999);
    }

    /// 测试浮点数读写操作
    #[tokio::test]
    async fn test_float_read_write() {
        let mut service = MockPlcService::new_for_testing("TestPLC");
        service.connect().await.unwrap();
        
        // 测试 float32
        let address_f32 = "DB1.DBD100";
        let test_value_f32 = 12.5f32;
        service.write_float32(address_f32, test_value_f32).await.unwrap();
        let value_f32 = service.read_float32(address_f32).await.unwrap();
        println!("Test f32: Expected: {}, Got: {}. Difference: {}", test_value_f32, value_f32, (value_f32 - test_value_f32).abs());
        assert!((value_f32 - test_value_f32).abs() < FLOAT_COMPARISON_TOLERANCE_F32);
        
        // 测试 float64
        let address_f64 = "DB1.DBD104";
        let test_value_f64 = 123.456789;
        service.write_float64(address_f64, test_value_f64).await.unwrap();
        let value_f64 = service.read_float64(address_f64).await.unwrap();
        println!("Test f64 (write/read): Expected: {}, Got: {}. Difference: {}", test_value_f64, value_f64, (value_f64 - test_value_f64).abs());
        assert!((value_f64 - test_value_f64).abs() < FLOAT_COMPARISON_TOLERANCE_F64);
        
        // 测试便捷方法 (read_float, write_float 对应 f64)
        let address_generic = "DB1.DBD108";
        let test_value_generic = 987.654;
        service.write_float(address_generic, test_value_generic).await.unwrap();
        let value_generic = service.read_float(address_generic).await.unwrap();
        println!("Test f64 generic (write/read): Expected: {}, Got: {}. Difference: {}", test_value_generic, value_generic, (value_generic - test_value_generic).abs());
        assert!((value_generic - test_value_generic).abs() < FLOAT_COMPARISON_TOLERANCE_F64, 
            "Generic float comparison failed. Expected: {}, Got: {}, Diff: {}", test_value_generic, value_generic, (value_generic - test_value_generic).abs());
    }

    /// 测试字符串读写操作
    #[tokio::test]
    async fn test_string_read_write() {
        let mut service = MockPlcService::new_for_testing("TestPLC");
        service.connect().await.unwrap();
        
        let address = "DB1.STRING200";
        let test_string = "Hello, PLC!";
        
        service.write_string(address, test_string).await.unwrap();
        let value = service.read_string(address, 100).await.unwrap();
        assert_eq!(value, test_string);
    }

    /// 测试字节数组读写操作
    #[tokio::test]
    async fn test_bytes_read_write() {
        let mut service = MockPlcService::new_for_testing("TestPLC");
        service.connect().await.unwrap();
        
        let address = "DB1.BYTES300";
        let test_bytes = vec![0x01, 0x02, 0x03, 0xFF, 0xAA, 0x55];
        
        service.write_bytes(address, &test_bytes).await.unwrap();
        let value = service.read_bytes(address, test_bytes.len()).await.unwrap();
        assert_eq!(value, test_bytes);
    }

    /// 测试批量读写操作
    #[tokio::test]
    async fn test_batch_operations() {
        let mut service = MockPlcService::new_for_testing("TestPLC");
        service.connect().await.unwrap();
        
        // 准备测试数据
        let mut write_values = HashMap::new();
        write_values.insert("DB1.DBD0".to_string(), Value::Number(serde_json::Number::from(100)));
        write_values.insert("DB1.DBD4".to_string(), Value::Number(serde_json::Number::from_f64(25.5).unwrap()));
        write_values.insert("DB1.DBX8.0".to_string(), Value::Bool(true));
        write_values.insert("DB1.STRING100".to_string(), Value::String("测试字符串".to_string()));
        
        // 执行批量写入
        service.batch_write(&write_values).await.unwrap();
        
        // 执行批量读取
        let addresses: Vec<String> = write_values.keys().cloned().collect();
        let read_values = service.batch_read(&addresses).await.unwrap();
        
        // 验证读取结果
        assert_eq!(read_values.len(), write_values.len());
        for (address, expected_value) in &write_values {
            let actual_value = &read_values[address];
            assert_eq!(actual_value, expected_value);
        }
        
        // 验证写入日志包含所有操作
        let write_log = service.get_write_log();
        assert!(write_log.len() >= write_values.len());
        
        // 检查每个地址都被写入了
        for address in addresses {
            assert!(service.was_address_written(&address));
        }
    }

    /// 测试预设读取值功能
    #[tokio::test]
    async fn test_preset_values() {
        let mut service = MockPlcService::new("TestPLC_Preset");
        service.connect().await.unwrap();

        // 设置预设值，使用 serde_json::Value
        service.preset_read_value("PRESET_BOOL", Value::Bool(true));
        service.preset_read_value("PRESET_FLOAT", Value::Number(serde_json::Number::from_f64(123.45).unwrap()));

        // 读取预设值
        assert_eq!(service.read_bool("PRESET_BOOL").await.unwrap(), true);
        let preset_float_val = service.read_float("PRESET_FLOAT").await.unwrap();
        println!("Test f64 preset: Expected: 123.45, Got: {}. Difference: {}", preset_float_val, (preset_float_val - 123.45).abs());
        assert!((preset_float_val - 123.45).abs() < FLOAT_COMPARISON_TOLERANCE_F64, 
            "Preset float comparison failed. Expected: 123.45, Got: {}, Diff: {}", preset_float_val, (preset_float_val - 123.45).abs());

        // 尝试读取不存在的预设值 (应该失败或返回默认)
        // 根据 MockPlcService 的实现，读取不存在的地址会返回默认值或错误
        // 这里假设它返回错误或特定逻辑处理
        assert!(service.read_int("NON_EXISTENT_PRESET").await.is_err());
    }

    /// 测试通信统计功能
    #[tokio::test]
    async fn test_communication_stats() {
        let mut service = MockPlcService::new("TestPLC_Stats");
        service.preset_read_value("ADDR_R1", serde_json::Value::Bool(true)); // 预设值
        service.preset_read_value("ADDR_R2", serde_json::Value::Number(serde_json::Number::from_f64(1.23).unwrap())); // 预设值
        
        // 初始统计信息
        let initial_stats = service.get_communication_stats();
        assert_eq!(initial_stats.connection_count, 0);
        assert_eq!(initial_stats.successful_reads, 0);
        assert_eq!(initial_stats.successful_writes, 0);
        assert!(initial_stats.last_communication_time.is_none());

        // 连接
        service.connect().await.unwrap();
        let after_connect_stats = service.get_communication_stats();
        assert_eq!(after_connect_stats.connection_count, 1);

        // 执行一些成功操作
        let _ = service.read_bool("ADDR_R1").await; // 传递 &str
        let _ = service.write_bool("ADDR_W1", true).await; // 传递 &str
        let _ = service.read_float("ADDR_R2").await;

        let success_stats = service.get_communication_stats();
        assert_eq!(success_stats.successful_reads, 2);
        assert_eq!(success_stats.successful_writes, 1);
        assert_eq!(success_stats.failed_reads, 0);
        assert_eq!(success_stats.failed_writes, 0);
        assert!(success_stats.last_communication_time.is_some());
        assert!(service.get_last_error().is_none());

        // 断开连接并尝试操作（应失败）
        service.disconnect().await.unwrap();
        let read_result = service.read_bool("ADDR_R3").await;
        assert!(read_result.is_err());
        let write_result = service.write_bool("ADDR_W2", false).await;
        assert!(write_result.is_err());

        let failure_stats = service.get_communication_stats();
        assert_eq!(failure_stats.successful_reads, 2); // 成功计数不变
        assert_eq!(failure_stats.successful_writes, 1); // 成功计数不变
        assert_eq!(failure_stats.failed_reads, 1);
        assert_eq!(failure_stats.failed_writes, 1);
        assert!(failure_stats.last_communication_time.is_some());
        assert!(service.get_last_error().is_some());
        
        // 重置统计
        service.reset_communication_stats();
        let reset_stats = service.get_communication_stats();
        assert_eq!(reset_stats.connection_count, 0);
        assert_eq!(reset_stats.successful_reads, 0);
        assert_eq!(reset_stats.successful_writes, 0);
        assert_eq!(reset_stats.failed_reads, 0);
        assert_eq!(reset_stats.failed_writes, 0);
        assert!(reset_stats.last_communication_time.is_none());
        assert!(service.get_last_error().is_none());
    }

    /// 测试标签信息功能
    #[tokio::test]
    async fn test_tag_operations() {
        let mut service = MockPlcService::new_for_testing("TestPLC");
        service.connect().await.unwrap();
        
        // 测试读取标签信息
        let tag_info = service.read_tag_info("DB1.DBD0").await.unwrap();
        assert_eq!(tag_info.address, "DB1.DBD0");
        assert!(tag_info.readable);
        assert!(tag_info.writable);
        assert!(tag_info.description.is_some());
        
        // 添加一些数据到存储中
        service.preset_read_value("TAG1", Value::Number(serde_json::Number::from(100)));
        service.preset_read_value("TAG2", Value::Bool(true));
        service.preset_read_value("TAG3", Value::String("test".to_string()));
        
        // 测试列出可用标签
        let available_tags = service.list_available_tags().await.unwrap();
        assert!(available_tags.len() >= 3);
        
        let tag_addresses: Vec<&str> = available_tags.iter().map(|t| t.address.as_str()).collect();
        assert!(tag_addresses.contains(&"TAG1"));
        assert!(tag_addresses.contains(&"TAG2"));
        assert!(tag_addresses.contains(&"TAG3"));
    }

    /// 测试设备信息功能
    #[tokio::test]
    async fn test_device_info() {
        let mut service = MockPlcService::new_for_testing("TestPLC");
        service.connect().await.unwrap();
        
        let device_info = service.get_device_info().await.unwrap();
        
        assert!(device_info.contains_key("device_type"));
        assert!(device_info.contains_key("service_name"));
        assert!(device_info.contains_key("version"));
        assert!(device_info.contains_key("vendor"));
        assert!(device_info.contains_key("connection_status"));
        
        assert_eq!(device_info["device_type"], "Mock PLC");
        assert_eq!(device_info["vendor"], "FAT_TEST Mock");
    }

    /// 测试超时设置功能
    #[tokio::test]
    async fn test_timeout_settings() {
        let mut service = MockPlcService::new_for_testing("TestPLC");
        
        // 测试设置读取超时
        service.set_read_timeout(5000).unwrap();
        // 注意：由于字段是私有的，我们无法直接访问，只能确保方法不出错
        
        // 测试设置写入超时
        service.set_write_timeout(6000).unwrap();
        // 注意：由于字段是私有的，我们无法直接访问，只能确保方法不出错
    }

    /// 测试未连接状态下的错误处理
    #[tokio::test]
    async fn test_disconnected_operations() {
        let service = MockPlcService::new_for_testing("TestPLC");
        // 注意：这里没有调用 connect()
        
        // 所有操作都应该返回连接错误
        assert!(service.read_bool("TEST").await.is_err());
        assert!(service.write_bool("TEST", true).await.is_err());
        assert!(service.read_float32("TEST").await.is_err());
        assert!(service.write_float32("TEST", 1.0).await.is_err());
        assert!(service.batch_read(&["TEST".to_string()]).await.is_err());
        assert!(service.batch_write(&HashMap::new()).await.is_err());
        assert!(service.read_tag_info("TEST").await.is_err());
        assert!(service.list_available_tags().await.is_err());
        assert!(service.get_device_info().await.is_err());
        
        // 健康检查应该失败
        assert!(service.health_check().await.is_err());
    }

    /// 测试写入日志功能
    #[tokio::test]
    async fn test_write_log_functionality() {
        let mut service = MockPlcService::new_for_testing("TestPLC");
        service.connect().await.unwrap();
        
        // 执行一些写入操作
        service.write_bool("ADDR1", true).await.unwrap();
        service.write_int32("ADDR2", 123).await.unwrap();
        service.write_float32("ADDR3", 45.6).await.unwrap();
        
        // 检查写入日志
        let write_log = service.get_write_log();
        assert_eq!(write_log.len(), 3);
        
        // 检查第一个写入操作
        let first_write = &write_log[0];
        assert_eq!(first_write.address, "ADDR1");
        assert_eq!(first_write.value, Value::Bool(true));
        assert_eq!(first_write.operation_type, "write_bool");
        assert!(first_write.timestamp <= chrono::Utc::now());
        
        // 检查地址写入状态
        assert!(service.was_address_written("ADDR1"));
        assert!(service.was_address_written("ADDR2"));
        assert!(service.was_address_written("ADDR3"));
        assert!(!service.was_address_written("NONEXISTENT"));
        
        // 清空日志
        service.clear_write_log();
        let empty_log = service.get_write_log();
        assert_eq!(empty_log.len(), 0);
        
        // 写入状态应该被重置
        assert!(!service.was_address_written("ADDR1"));
    }

    /// 测试错误模拟功能
    #[tokio::test]
    async fn test_error_simulation() {
        let mut service = MockPlcService::new("TestPLC");
        
        // 启用高错误率的错误模拟
        service.set_error_simulation(true, 1.0); // 100% 错误率
        
        // 连接应该失败
        assert!(service.connect().await.is_err());
        
        // 禁用错误模拟
        service.set_error_simulation(false, 0.0);
        
        // 现在连接应该成功
        service.connect().await.unwrap();
        assert!(service.is_connected());
    }

    /// 测试网络延迟模拟
    #[tokio::test]
    async fn test_network_delay_simulation() {
        let mut service = MockPlcService::new("TestPLC");
        
        // 设置较长的网络延迟
        service.set_network_delay(true, 100); // 100ms 延迟
        
        let start_time = std::time::Instant::now();
        service.connect().await.unwrap();
        let elapsed = start_time.elapsed();
        
        // 操作应该花费一定时间（至少接近设定的延迟）
        assert!(elapsed.as_millis() >= 90); // 允许一些误差
        
        // 禁用网络延迟
        service.set_network_delay(false, 0);
        
        let start_time = std::time::Instant::now();
        service.disconnect().await.unwrap();
        let elapsed = start_time.elapsed();
        
        // 现在操作应该很快
        assert!(elapsed.as_millis() < 50);
    }

    /// 测试PlcTag结构体
    #[test]
    fn test_plc_tag_structure() {
        let tag = PlcTag {
            address: "DB1.DBD0".to_string(),
            data_type: PlcDataType::Float32,
            description: Some("测试标签".to_string()),
            readable: true,
            writable: true,
            unit: Some("mA".to_string()),
            min_value: Some(4.0),
            max_value: Some(20.0),
        };
        
        assert_eq!(tag.address, "DB1.DBD0");
        assert_eq!(tag.data_type, PlcDataType::Float32);
        assert!(tag.readable);
        assert!(tag.writable);
        assert_eq!(tag.unit.as_ref().unwrap(), "mA");
        assert_eq!(tag.min_value.unwrap(), 4.0);
        assert_eq!(tag.max_value.unwrap(), 20.0);
    }

    /// 测试PlcDataType枚举
    #[test]
    fn test_plc_data_type() {
        // 测试默认值
        let default_type = PlcDataType::default();
        assert_eq!(default_type, PlcDataType::Float32);
        
        // 测试所有枚举值
        let types = vec![
            PlcDataType::Bool,
            PlcDataType::Int8,
            PlcDataType::UInt8,
            PlcDataType::Int16,
            PlcDataType::UInt16,
            PlcDataType::Int32,
            PlcDataType::UInt32,
            PlcDataType::Int64,
            PlcDataType::UInt64,
            PlcDataType::Float32,
            PlcDataType::Float64,
            PlcDataType::String,
            PlcDataType::ByteArray,
        ];
        
        // 确保每个类型都能正确比较
        for data_type in types {
            assert_eq!(data_type.clone(), data_type);
        }
    }

    /// 测试PlcConnectionStatus枚举
    #[test]
    fn test_plc_connection_status() {
        let statuses = vec![
            PlcConnectionStatus::Disconnected,
            PlcConnectionStatus::Connecting,
            PlcConnectionStatus::Connected,
            PlcConnectionStatus::Error("测试错误".to_string()),
        ];
        
        // 测试状态比较
        assert_ne!(statuses[0], statuses[2]);
        assert_eq!(statuses[0].clone(), statuses[0]);
        
        // 测试错误状态
        if let PlcConnectionStatus::Error(msg) = &statuses[3] {
            assert_eq!(msg, "测试错误");
        } else {
            panic!("错误状态类型不正确");
        }
    }
} 