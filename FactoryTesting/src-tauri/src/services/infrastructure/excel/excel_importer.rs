/// Excel文件导入服务
/// 
/// 负责解析Excel文件中的通道点位定义数据
use std::path::Path;
use calamine::{Reader, Xlsx, open_workbook};
use crate::models::{ChannelPointDefinition, ModuleType, PointDataType};
use crate::utils::error::{AppError, AppResult};
use log::info;

/// Excel导入器
pub struct ExcelImporter;

impl ExcelImporter {
    /// 解析Excel文件并返回通道点位定义列表
    /// 
    /// # 参数
    /// * `file_path` - Excel文件路径
    /// 
    /// # 返回
    /// * `AppResult<Vec<ChannelPointDefinition>>` - 解析的通道定义列表
    pub async fn parse_excel_file(file_path: &str) -> AppResult<Vec<ChannelPointDefinition>> {
        info!("开始解析Excel文件: {}", file_path);
        
        // 检查文件是否存在
        if !Path::new(file_path).exists() {
            return Err(AppError::validation_error(&format!("文件不存在: {}", file_path)));
        }
        
        // 打开Excel文件
        let mut workbook: Xlsx<_> = open_workbook(file_path)
            .map_err(|e| AppError::validation_error(&format!("无法打开Excel文件: {}", e)))?;
        
        // 获取第一个工作表
        let worksheet_names = workbook.sheet_names();
        if worksheet_names.is_empty() {
            return Err(AppError::validation_error("Excel文件中没有工作表"));
        }
        
        let sheet_name = &worksheet_names[0];
        info!("读取工作表: {}", sheet_name);
        
        let range = match workbook.worksheet_range(sheet_name) {
            Some(Ok(range)) => range,
            Some(Err(e)) => return Err(AppError::validation_error(&format!("无法读取工作表: {}", e))),
            None => return Err(AppError::validation_error(&format!("工作表不存在: {}", sheet_name))),
        };
        
        let mut definitions = Vec::new();
        let mut row_count = 0;
        
        // 跳过标题行，从第二行开始解析
        for (row_idx, row) in range.rows().enumerate() {
            if row_idx == 0 {
                // 验证标题行格式
                Self::validate_header_row(row)?;
                continue;
            }
            
            row_count += 1;
            
            // 解析数据行
            match Self::parse_data_row(row, row_idx + 1) {
                Ok(definition) => definitions.push(definition),
                Err(e) => {
                    // 记录错误但继续处理其他行
                    log::warn!("第{}行解析失败: {}", row_idx + 1, e);
                }
            }
        }
        
        info!("Excel解析完成，共处理{}行数据，成功解析{}个通道定义", row_count, definitions.len());
        
        if definitions.is_empty() {
            return Err(AppError::validation_error("Excel文件中没有有效的通道定义数据"));
        }
        
        Ok(definitions)
    }
    
    /// 验证Excel文件的标题行格式
    fn validate_header_row(row: &[calamine::DataType]) -> AppResult<()> {
        // 根据真实Excel文件的列名更新期望的标题
        let expected_headers = vec![
            "序号", "模块名称", "模块类型", "供电类型（有源/无源）", "线制", 
            "通道位号", "位号", "场站名", "变量名称（HMI）", "变量描述", 
            "数据类型", "读写属性"
        ];
        
        if row.len() < 12 {  // 至少需要前12列
            return Err(AppError::validation_error(&format!(
                "Excel标题行列数不足，期望至少12列，实际{}列", 
                row.len()
            )));
        }
        
        // 验证关键列的存在（不要求完全匹配所有列名）
        let key_columns = vec![
            (2, "模块类型"),
            (3, "供电类型"),
            (6, "位号"), 
            (8, "变量名称（HMI）"),
            (9, "变量描述"),
            (10, "数据类型")
        ];
        
        for (index, expected) in key_columns {
            if index < row.len() {
                let actual_string = row[index].to_string();
                let actual = actual_string.trim();
                if !actual.contains(expected) {
                    log::warn!("Excel标题行第{}列可能不匹配，期望包含'{}'，实际'{}'", 
                              index + 1, expected, actual);
                }
            }
        }
        
        Ok(())
    }
    
    /// 解析Excel数据行为ChannelPointDefinition
    fn parse_data_row(row: &[calamine::DataType], row_number: usize) -> AppResult<ChannelPointDefinition> {
        if row.len() < 53 {  // 根据真实Excel文件，至少需要53列
            return Err(AppError::validation_error(&format!(
                "第{}行数据列数不足，期望53列，实际{}列", 
                row_number, 
                row.len()
            )));
        }
        
        // 根据真实Excel文件的列索引提取数据
        // 实际列映射：
        // 第0列：序号
        // 第1列：模块名称  
        // 第2列：模块类型
        // 第3列：供电类型（有源/无源）
        // 第5列：通道位号
        // 第6列：位号
        // 第7列：场站名
        // 第8列：变量名称（HMI）
        // 第9列：变量描述
        // 第10列：数据类型
        // 第51列：PLC绝对地址
        
        let tag = Self::get_string_value(&row[6], row_number, "位号")?;  // 第6列：位号
        let variable_name = Self::get_string_value(&row[8], row_number, "变量名称（HMI）")?;  // 第8列：变量名称（HMI）
        let description = Self::get_optional_string_value(&row[9], "变量描述");  // 第9列：变量描述（可能为空）
        let station = Self::get_string_value(&row[7], row_number, "场站名")?;  // 第7列：场站名
        let module = Self::get_string_value(&row[1], row_number, "模块名称")?;  // 第1列：模块名称
        let module_type_str = Self::get_string_value(&row[2], row_number, "模块类型")?;  // 第2列：模块类型
        let power_supply_type = Self::get_optional_string_value(&row[3], "供电类型");  // 第3列：供电类型（有源/无源）
        let channel_number = Self::get_string_value(&row[5], row_number, "通道位号")?;  // 第5列：通道位号
        let data_type_str = Self::get_string_value(&row[10], row_number, "数据类型")?;  // 第10列：数据类型
        let plc_address = Self::get_string_value(&row[51], row_number, "PLC绝对地址")?;  // 第51列：PLC绝对地址
        
        // 解析模块类型
        let module_type = Self::parse_module_type(&module_type_str, row_number)?;
        
        // 解析数据类型
        let data_type = Self::parse_data_type(&data_type_str, row_number)?;
        
        // 创建通道定义
        let mut definition = ChannelPointDefinition::new(
            tag,
            variable_name,
            description,
            station,
            module,
            module_type,
            channel_number,
            data_type,
            plc_address,
        );
        
        // 设置供电类型（如果Excel中有值则使用，否则使用默认值）
        if !power_supply_type.is_empty() {
            definition.power_supply_type = power_supply_type;
        }
        
        Ok(definition)
    }
    
    /// 从Excel单元格获取字符串值
    fn get_string_value(cell: &calamine::DataType, row_number: usize, column_name: &str) -> AppResult<String> {
        let value = cell.to_string().trim().to_string();
        if value.is_empty() {
            return Err(AppError::validation_error(&format!(
                "第{}行'{}'列不能为空", 
                row_number, 
                column_name
            )));
        }
        Ok(value)
    }
    
    /// 从Excel单元格获取可选字符串值（允许为空）
    fn get_optional_string_value(cell: &calamine::DataType, _column_name: &str) -> String {
        cell.to_string().trim().to_string()
    }
    
    /// 解析模块类型字符串
    fn parse_module_type(type_str: &str, row_number: usize) -> AppResult<ModuleType> {
        match type_str.to_uppercase().as_str() {
            "AI" => Ok(ModuleType::AI),
            "AO" => Ok(ModuleType::AO),
            "DI" => Ok(ModuleType::DI),
            "DO" => Ok(ModuleType::DO),
            _ => Err(AppError::validation_error(&format!(
                "第{}行模块类型'{}'无效，支持的类型: AI, AO, DI, DO", 
                row_number, 
                type_str
            )))
        }
    }
    
    /// 解析数据类型字符串
    fn parse_data_type(type_str: &str, row_number: usize) -> AppResult<PointDataType> {
        match type_str.to_uppercase().as_str() {
            "BOOL" | "BOOLEAN" => Ok(PointDataType::Bool),
            "INT" | "INTEGER" => Ok(PointDataType::Int),
            "FLOAT" | "REAL" => Ok(PointDataType::Float),  // 支持REAL类型
            "STRING" => Ok(PointDataType::String),
            _ => Err(AppError::validation_error(&format!(
                "第{}行数据类型'{}'无效，支持的类型: Bool, Int, Float/Real, String", 
                row_number, 
                type_str
            )))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[tokio::test]
    async fn test_parse_real_excel_file() {
        // 测试真实Excel文件
        let real_file_path = r"C:\Program Files\Git\code\FactoryTesting\测试文件\测试IO.xlsx";
        
        if std::path::Path::new(real_file_path).exists() {
            println!("测试真实Excel文件: {}", real_file_path);
            
            let result = ExcelImporter::parse_excel_file(real_file_path).await;
            
            match result {
                Ok(definitions) => {
                    println!("成功解析Excel文件！");
                    println!("总共解析了 {} 个通道定义", definitions.len());
                    
                    // 显示前几个定义的详细信息
                    for (i, def) in definitions.iter().take(5).enumerate() {
                        println!("\n第{}个定义:", i + 1);
                        println!("  位号: {}", def.tag);
                        println!("  变量名: {}", def.variable_name);
                        println!("  描述: {}", def.variable_description);
                        println!("  模块类型: {:?}", def.module_type);
                        println!("  数据类型: {:?}", def.data_type);
                        println!("  PLC地址: {}", def.plc_communication_address);
                    }
                    
                    // 验证数据的基本正确性
                    assert!(!definitions.is_empty(), "应该解析出至少一个通道定义");
                    
                    // 验证每个定义都有必要的字段
                    for def in &definitions {
                        assert!(!def.tag.is_empty(), "位号不能为空");
                        assert!(!def.variable_name.is_empty(), "变量名不能为空");
                        // 描述可能为空，所以不验证
                        assert!(!def.plc_communication_address.is_empty(), "PLC地址不能为空");
                    }
                    
                    // 统计不同模块类型的数量
                    let ai_count = definitions.iter().filter(|d| matches!(d.module_type, ModuleType::AI)).count();
                    let ao_count = definitions.iter().filter(|d| matches!(d.module_type, ModuleType::AO)).count();
                    let di_count = definitions.iter().filter(|d| matches!(d.module_type, ModuleType::DI)).count();
                    let do_count = definitions.iter().filter(|d| matches!(d.module_type, ModuleType::DO)).count();
                    
                    println!("\n模块类型统计:");
                    println!("  AI: {} 个", ai_count);
                    println!("  AO: {} 个", ao_count);
                    println!("  DI: {} 个", di_count);
                    println!("  DO: {} 个", do_count);
                    
                    // 验证至少有一些数据
                    assert!(ai_count + ao_count + di_count + do_count > 0, "应该有至少一个有效的模块类型");
                }
                Err(e) => {
                    panic!("解析Excel文件失败: {:?}", e);
                }
            }
        } else {
            println!("真实Excel文件不存在，跳过测试: {}", real_file_path);
        }
    }

    #[test]
    fn test_parse_module_type() {
        assert_eq!(ExcelImporter::parse_module_type("AI", 1).unwrap(), ModuleType::AI);
        assert_eq!(ExcelImporter::parse_module_type("AO", 1).unwrap(), ModuleType::AO);
        assert_eq!(ExcelImporter::parse_module_type("DI", 1).unwrap(), ModuleType::DI);
        assert_eq!(ExcelImporter::parse_module_type("DO", 1).unwrap(), ModuleType::DO);
        assert_eq!(ExcelImporter::parse_module_type("ai", 1).unwrap(), ModuleType::AI);
        
        assert!(ExcelImporter::parse_module_type("INVALID", 1).is_err());
    }

    #[test]
    fn test_parse_data_type() {
        assert_eq!(ExcelImporter::parse_data_type("BOOL", 1).unwrap(), PointDataType::Bool);
        assert_eq!(ExcelImporter::parse_data_type("REAL", 1).unwrap(), PointDataType::Float);
        assert_eq!(ExcelImporter::parse_data_type("INT", 1).unwrap(), PointDataType::Int);
        assert_eq!(ExcelImporter::parse_data_type("STRING", 1).unwrap(), PointDataType::String);
        assert_eq!(ExcelImporter::parse_data_type("float", 1).unwrap(), PointDataType::Float);
        
        assert!(ExcelImporter::parse_data_type("INVALID", 1).is_err());
    }
} 