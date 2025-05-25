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
        let expected_headers = vec![
            "标签", "变量名", "描述", "工位", "模块", "模块类型", "通道号", "数据类型", "PLC地址"
        ];
        
        if row.len() < expected_headers.len() {
            return Err(AppError::validation_error(&format!(
                "Excel标题行列数不足，期望{}列，实际{}列", 
                expected_headers.len(), 
                row.len()
            )));
        }
        
        for (i, expected) in expected_headers.iter().enumerate() {
            let actual_string = row[i].to_string();
            let actual = actual_string.trim();
            if actual != *expected {
                return Err(AppError::validation_error(&format!(
                    "Excel标题行第{}列不匹配，期望'{}'，实际'{}'", 
                    i + 1, 
                    expected, 
                    actual
                )));
            }
        }
        
        Ok(())
    }
    
    /// 解析Excel数据行为ChannelPointDefinition
    fn parse_data_row(row: &[calamine::DataType], row_number: usize) -> AppResult<ChannelPointDefinition> {
        if row.len() < 9 {
            return Err(AppError::validation_error(&format!(
                "第{}行数据列数不足，期望9列，实际{}列", 
                row_number, 
                row.len()
            )));
        }
        
        // 提取各列数据
        let tag = Self::get_string_value(&row[0], row_number, "标签")?;
        let variable_name = Self::get_string_value(&row[1], row_number, "变量名")?;
        let description = Self::get_string_value(&row[2], row_number, "描述")?;
        let station = Self::get_string_value(&row[3], row_number, "工位")?;
        let module = Self::get_string_value(&row[4], row_number, "模块")?;
        let module_type_str = Self::get_string_value(&row[5], row_number, "模块类型")?;
        let channel_number = Self::get_string_value(&row[6], row_number, "通道号")?;
        let data_type_str = Self::get_string_value(&row[7], row_number, "数据类型")?;
        let plc_address = Self::get_string_value(&row[8], row_number, "PLC地址")?;
        
        // 解析模块类型
        let module_type = Self::parse_module_type(&module_type_str, row_number)?;
        
        // 解析数据类型
        let data_type = Self::parse_data_type(&data_type_str, row_number)?;
        
        // 创建通道定义
        let definition = ChannelPointDefinition::new(
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
            "FLOAT" | "REAL" => Ok(PointDataType::Float),
            "STRING" => Ok(PointDataType::String),
            _ => Err(AppError::validation_error(&format!(
                "第{}行数据类型'{}'无效，支持的类型: Bool, Int, Float, String", 
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
    async fn test_parse_module_type() {
        assert_eq!(ExcelImporter::parse_module_type("AI", 1).unwrap(), ModuleType::AI);
        assert_eq!(ExcelImporter::parse_module_type("ao", 1).unwrap(), ModuleType::AO);
        assert_eq!(ExcelImporter::parse_module_type("Di", 1).unwrap(), ModuleType::DI);
        assert_eq!(ExcelImporter::parse_module_type("DO", 1).unwrap(), ModuleType::DO);
        
        assert!(ExcelImporter::parse_module_type("INVALID", 1).is_err());
    }
    
    #[tokio::test]
    async fn test_parse_data_type() {
        assert_eq!(ExcelImporter::parse_data_type("BOOL", 1).unwrap(), PointDataType::Bool);
        assert_eq!(ExcelImporter::parse_data_type("int", 1).unwrap(), PointDataType::Int);
        assert_eq!(ExcelImporter::parse_data_type("Float", 1).unwrap(), PointDataType::Float);
        assert_eq!(ExcelImporter::parse_data_type("STRING", 1).unwrap(), PointDataType::String);
        
        assert!(ExcelImporter::parse_data_type("INVALID", 1).is_err());
    }
} 