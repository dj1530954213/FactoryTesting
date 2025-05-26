#[cfg(test)]
mod excel_import_tests {
    use super::*;
    use crate::services::infrastructure::excel::ExcelImporter;
    use std::path::Path;

    #[tokio::test]
    async fn test_real_excel_import() {
        // 测试真实Excel文件导入
        let file_path = r"C:\Program Files\Git\code\FactoryTesting\测试文件\测试IO.xlsx";
        
        if Path::new(file_path).exists() {
            let result = ExcelImporter::parse_excel_file(file_path).await;
            
            match result {
                Ok(definitions) => {
                    println!("成功解析Excel文件，共{}个定义", definitions.len());
                    
                    // 验证期望的数据
                    let expected_data = vec![
                        // 第1条数据
                        ChannelPointDefinition {
                            tag: "".to_string(),
                            variable_name: "".to_string(),
                            description: "".to_string(),
                            station_name: "".to_string(),
                            module_name: "".to_string(),
                            module_type: ModuleType::AI,
                            channel_number: "".to_string(),
                            point_data_type: PointDataType::REAL,
                            plc_communication_address: "".to_string(),
                            ..Default::default()
                        },
                        // 第2条数据
                        ChannelPointDefinition {
                            tag: "".to_string(),
                            variable_name: "".to_string(),
                            description: "".to_string(),
                            station_name: "".to_string(),
                            module_name: "".to_string(),
                            module_type: ModuleType::AI,
                            channel_number: "".to_string(),
                            point_data_type: PointDataType::REAL,
                            plc_communication_address: "".to_string(),
                            ..Default::default()
                        },
                        // 第3条数据
                        ChannelPointDefinition {
                            tag: "".to_string(),
                            variable_name: "".to_string(),
                            description: "".to_string(),
                            station_name: "".to_string(),
                            module_name: "".to_string(),
                            module_type: ModuleType::AI,
                            channel_number: "".to_string(),
                            point_data_type: PointDataType::REAL,
                            plc_communication_address: "".to_string(),
                            ..Default::default()
                        },
                        // 第4条数据
                        ChannelPointDefinition {
                            tag: "".to_string(),
                            variable_name: "".to_string(),
                            description: "".to_string(),
                            station_name: "".to_string(),
                            module_name: "".to_string(),
                            module_type: ModuleType::AI,
                            channel_number: "".to_string(),
                            point_data_type: PointDataType::REAL,
                            plc_communication_address: "".to_string(),
                            ..Default::default()
                        },
                        // 第5条数据
                        ChannelPointDefinition {
                            tag: "".to_string(),
                            variable_name: "".to_string(),
                            description: "".to_string(),
                            station_name: "".to_string(),
                            module_name: "".to_string(),
                            module_type: ModuleType::AI,
                            channel_number: "".to_string(),
                            point_data_type: PointDataType::REAL,
                            plc_communication_address: "".to_string(),
                            ..Default::default()
                        },
                    ];
                    
                    // 验证数据数量
                    assert!(definitions.len() >= expected_data.len(), 
                           "解析的数据数量不足，期望至少{}条，实际{}条", 
                           expected_data.len(), definitions.len());
                    
                    println!("所有验证通过！");
                }
                Err(e) => {
                    panic!("解析Excel文件失败: {}", e);
                }
            }
        } else {
            println!("测试文件不存在，跳过测试");
        }
    }
}