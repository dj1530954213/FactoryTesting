/// Excel文件导入服务
///
/// 负责解析Excel文件中的通道点位定义数据
/// 基于重构后的数据模型和原C#项目的点表结构
use std::path::Path;
use calamine::{Reader, Xlsx, open_workbook, DataType};
use crate::models::structs::ChannelPointDefinition;
use crate::models::enums::{ModuleType, PointDataType};
use crate::error::AppError;
use log::{info, error};
use std::collections::HashMap;

type AppResult<T> = Result<T, AppError>;

/// Excel导入器
pub struct ExcelImporter;

/// 标题行关键列关键词常量
const COL_KEY_MODULE_TYPE: &str = "模块类型";
const COL_KEY_POWER_TYPE: &str = "供电";            // "供电类型" 或 "供电类型（有源/无源）" 均可
const COL_KEY_CHANNEL_POS: &str = "通道位号";
const COL_KEY_HMI_NAME: &str = "变量名称";           // 全称 "变量名称（HMI）"
const COL_KEY_DESCRIPTION: &str = "变量描述";
const COL_KEY_DATA_TYPE: &str = "数据类型";
const COL_KEY_UNIT: &str = "单位";
const COL_KEY_STATION: &str = "场站名";
const COL_KEY_STATION_CODE: &str = "场站编号";
const COL_KEY_PLC_ADDR: &str = "PLC绝对地址";
const COL_KEY_COMM_ADDR: &str = "通讯地址";          // "上位机通讯地址" / "通讯地址"
const COL_KEY_SEQUENCE: &str = "序号";

/// 根据标题行生成列索引映射
fn build_header_index(row: &[DataType]) -> HashMap<String, usize> {
    let mut map = HashMap::new();
    log::info!("构建头部索引映射，总列数: {}", row.len());

    for (idx, cell) in row.iter().enumerate() {
        let title = cell.to_string();
        log::info!("列 {}: '{}'", idx, title);
        if title.contains(COL_KEY_MODULE_TYPE) {
            map.insert(COL_KEY_MODULE_TYPE.to_string(), idx);
        } else if title.contains(COL_KEY_POWER_TYPE) {
            map.insert(COL_KEY_POWER_TYPE.to_string(), idx);
        } else if title.contains(COL_KEY_CHANNEL_POS) {
            map.insert(COL_KEY_CHANNEL_POS.to_string(), idx);
        } else if title.contains(COL_KEY_HMI_NAME) {
            map.insert(COL_KEY_HMI_NAME.to_string(), idx);
        } else if title.contains(COL_KEY_DESCRIPTION) {
            map.insert(COL_KEY_DESCRIPTION.to_string(), idx);
        } else if title.contains(COL_KEY_DATA_TYPE) {
            map.insert(COL_KEY_DATA_TYPE.to_string(), idx);
        } else if title.contains(COL_KEY_UNIT) {
            map.insert(COL_KEY_UNIT.to_string(), idx);
        } else if title.contains(COL_KEY_STATION) {
            map.insert(COL_KEY_STATION.to_string(), idx);
        } else if title.contains(COL_KEY_STATION_CODE) {
            map.insert(COL_KEY_STATION_CODE.to_string(), idx);
        } else if title.contains(COL_KEY_PLC_ADDR) {
            map.insert(COL_KEY_PLC_ADDR.to_string(), idx);
        } else if title.contains(COL_KEY_COMM_ADDR) {
            map.insert(COL_KEY_COMM_ADDR.to_string(), idx);
        } else if title.contains(COL_KEY_SEQUENCE) {
            map.insert(COL_KEY_SEQUENCE.to_string(), idx);
            log::info!("找到序号列，索引: {}", idx);
        }
    }

    log::info!("头部映射完成: {:?}", map);
    map
}

impl ExcelImporter {
    /// 解析Excel文件并返回通道点位定义列表
    ///
    /// # 参数
    /// * `file_path` - Excel文件路径
    ///
    /// # 返回
    /// * `AppResult<Vec<ChannelPointDefinition>>` - 解析的通道定义列表
    pub async fn parse_excel_file(file_path: &str) -> AppResult<Vec<ChannelPointDefinition>> {


        // 检查文件是否存在
        if !Path::new(file_path).exists() {
            return Err(AppError::validation_error(format!("文件不存在: {}", file_path)));
        }

        // 打开Excel文件
        let mut workbook: Xlsx<_> = open_workbook(file_path)
            .map_err(|e| AppError::validation_error(format!("无法打开Excel文件: {}", e)))?;

        // 获取第一个工作表
        let worksheet_names = workbook.sheet_names();
        if worksheet_names.is_empty() {
            return Err(AppError::validation_error("Excel文件中没有工作表"));
        }

        let sheet_name = &worksheet_names[0];
        info!("读取工作表: {}", sheet_name);

        let range = match workbook.worksheet_range(sheet_name) {
            Some(Ok(range)) => range,
            Some(Err(e)) => return Err(AppError::validation_error(format!("无法读取工作表: {}", e))),
            None => return Err(AppError::validation_error(format!("工作表不存在: {}", sheet_name))),
        };

        let mut definitions = Vec::new();
        let mut row_count = 0;

        // 生成列索引映射
        let mut header_map: Option<HashMap<String, usize>> = None;

        for (row_idx, row) in range.rows().enumerate() {
            if row_idx == 0 {
                // 构建标题映射
                let map = build_header_index(row);

                // 基本列检查
                for key in [COL_KEY_MODULE_TYPE, COL_KEY_POWER_TYPE, COL_KEY_CHANNEL_POS,
                            COL_KEY_HMI_NAME, COL_KEY_DATA_TYPE, COL_KEY_COMM_ADDR, COL_KEY_SEQUENCE] {
                    if !map.contains_key(key) {
                        return Err(AppError::validation_error(format!("Excel标题缺少关键列: {}", key)));
                    }
                }
                header_map = Some(map);
                continue;
            }

            row_count += 1;
            let actual_row_number = row_idx + 1; // Excel中的实际行号

            // 解析数据行
            match Self::parse_data_row(row, actual_row_number, header_map.as_ref().unwrap()) {
                Ok(definition) => {
                    definitions.push(definition);
                },
                Err(e) => {
                    // 只记录错误，不显示详细调试信息
                    log::error!("第{}行解析失败: {}", actual_row_number, e);
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
            return Err(AppError::validation_error(format!(
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
    fn parse_data_row(row: &[calamine::DataType], row_number: usize, header_map: &HashMap<String, usize>) -> AppResult<ChannelPointDefinition> {


        // 根据标题映射动态检查列数
        let max_required_index = header_map.values().max().copied().unwrap_or(0);
        if row.len() <= max_required_index {
            return Err(AppError::validation_error(format!("第{}行数据列数不足，实际{}列", row_number, row.len())));
        }

        // 通过 header_map 获取列索引
        let idx = |key: &str| header_map.get(key).copied().unwrap();

        let variable_name = Self::get_string_value(&row[idx(COL_KEY_HMI_NAME)], row_number, COL_KEY_HMI_NAME)?;
        let tag = variable_name.clone(); // ➜ 用 HMI 名替代原位号

        let description = Self::get_optional_string_value(&row[idx(COL_KEY_DESCRIPTION)], COL_KEY_DESCRIPTION);
        let station = Self::get_string_value(&row[idx(COL_KEY_STATION)], row_number, COL_KEY_STATION)?;

        // station_code 可选
        let station_code = header_map.get(COL_KEY_STATION_CODE).map(|i| Self::get_optional_string_value(&row[*i], COL_KEY_STATION_CODE));

        // 模块名称列固定索引1（表结构未变）
        let module = Self::get_string_value(&row[1], row_number, "模块名称")?;

        let module_type_str = Self::get_string_value(&row[idx(COL_KEY_MODULE_TYPE)], row_number, COL_KEY_MODULE_TYPE)?;
        let power_supply_type = Self::get_optional_string_value(&row[idx(COL_KEY_POWER_TYPE)], COL_KEY_POWER_TYPE);
        let wire_system = Self::get_optional_string_value(&row[4], "线制");
        let channel_number = Self::get_string_value(&row[idx(COL_KEY_CHANNEL_POS)], row_number, COL_KEY_CHANNEL_POS)?;
        let data_type_str = Self::get_string_value(&row[idx(COL_KEY_DATA_TYPE)], row_number, COL_KEY_DATA_TYPE)?;

        let access_property_idx = header_map.get("读写属性").copied();
        let access_property = access_property_idx.map(|i| Self::get_optional_string_value(&row[i], "读写属性"));

        let plc_absolute_address = header_map.get(COL_KEY_PLC_ADDR).map(|i| Self::get_optional_string_value(&row[*i], COL_KEY_PLC_ADDR)).unwrap_or_default();
        let modbus_communication_address = Self::get_string_value(&row[idx(COL_KEY_COMM_ADDR)], row_number, COL_KEY_COMM_ADDR)?;

        // 单位可选
        let engineering_unit = header_map.get(COL_KEY_UNIT).map(|i| Self::get_optional_string_value(&row[*i], COL_KEY_UNIT));

        // 解析模块类型
        let module_type = Self::parse_module_type(&module_type_str, row_number)?;

        // 解析数据类型
        let data_type = Self::parse_data_type(&data_type_str, row_number)?;

        // 序号（可能为浮点或整数字符串）
        let sequence_number = Self::parse_sequence_number(&row[idx(COL_KEY_SEQUENCE)]);

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
            modbus_communication_address,
        );

        // 单位
        if let Some(unit_val) = engineering_unit { if !unit_val.is_empty() { definition.engineering_unit = Some(unit_val); } }

        // 场站编号
        if let Some(code_opt) = station_code { if !code_opt.is_empty() { definition.station_name = format!("{}-{}", definition.station_name, code_opt); } }

        // 设置PLC绝对地址（如%MD100）
        if !plc_absolute_address.is_empty() && plc_absolute_address != "/" {
            definition.plc_absolute_address = Some(plc_absolute_address);
        }

        // 设置额外字段
        definition.power_supply_type = power_supply_type;
        definition.wire_system = wire_system;

        // 从Excel中提取更多字段（如果存在）
        Self::extract_additional_fields(&mut definition, row, row_number)?;

        definition.sequence_number = sequence_number;

        Ok(definition)
    }

    /// 从Excel行中提取额外的字段信息
    fn extract_additional_fields(
        definition: &mut ChannelPointDefinition,
        row: &[calamine::DataType],
        row_number: usize
    ) -> AppResult<()> {


        // 辅助函数：安全获取字符串值
        let get_string = |index: usize| -> String {
            if index < row.len() {
                match &row[index] {
                    calamine::DataType::String(s) => s.trim().to_string(),
                    calamine::DataType::Float(f) => f.to_string(),
                    calamine::DataType::Int(i) => i.to_string(),
                    calamine::DataType::Bool(b) => b.to_string(),
                    _ => String::new(),
                }
            } else {
                String::new()
            }
        };

        // 辅助函数：安全获取浮点数值（返回f32）
        let get_float = |index: usize| -> Option<f32> {
            if index < row.len() {
                match &row[index] {
                    calamine::DataType::Float(f) => Some(*f as f32),
                    calamine::DataType::Int(i) => Some(*i as f32),
                    calamine::DataType::String(s) => {
                        if s.trim().is_empty() || s.trim() == "/" {
                            None
                        } else {
                            s.trim().parse::<f32>().ok()
                        }
                    }
                    _ => None,
                }
            } else {
                None
            }
        };

        // 根据Excel文件的实际列结构提取字段
        // 列索引映射（基于测试IO.txt文件的标题行）：
        // 第12列：保存历史
        // 第13列：掉电保护
        // 第14列：量程低限
        // 第15列：量程高限
        // 第16列：SLL设定值
        // 第17列：SLL设定点位
        // 第18列：SLL设定点位_PLC地址
        // 第19列：SLL设定点位_通讯地址
        // 第20列：SL设定值
        // 第21列：SL设定点位
        // 第22列：SL设定点位_PLC地址
        // 第23列：SL设定点位_通讯地址
        // 第24列：SH设定值
        // 第25列：SH设定点位
        // 第26列：SH设定点位_PLC地址
        // 第27列：SH设定点位_通讯地址
        // 第28列：SHH设定值
        // 第29列：SHH设定点位
        // 第30列：SHH设定点位_PLC地址
        // 第31列：SHH设定点位_通讯地址

        // 提取保存历史和掉电保护
        let save_history = get_string(12);
        let power_failure_protection = get_string(13);

        if !save_history.is_empty() && save_history != "/" {
            definition.save_history = Some(save_history == "是");
        }

        if !power_failure_protection.is_empty() && power_failure_protection != "/" {
            definition.power_failure_protection = Some(power_failure_protection == "是");
        }

        // 提取量程信息（仅对模拟量有效）
        if matches!(definition.module_type, ModuleType::AI | ModuleType::AO) {
            definition.range_low_limit = get_float(14);
            definition.range_high_limit = get_float(15);

            // 不再生成虚拟地址，测试台架地址将通过通道分配时从测试PLC配置表获取
            definition.test_rig_plc_address = None;
        }

        // 提取SLL（超低低）报警设定
        definition.sll_set_value = get_float(16);
        let sll_set_point = get_string(17);
        let sll_set_point_plc = get_string(18);
        let _sll_set_point_comm = get_string(19);

        if !sll_set_point.is_empty() && sll_set_point != "/" {
            definition.sll_set_point_address = Some(sll_set_point);
        }
        if !sll_set_point_plc.is_empty() && sll_set_point_plc != "/" {
            definition.sll_set_point_plc_address = Some(sll_set_point_plc);
        }
        if !_sll_set_point_comm.is_empty() && _sll_set_point_comm != "/" {
            definition.sll_set_point_communication_address = Some(_sll_set_point_comm);
        }

        // 提取SL（低）报警设定
        definition.sl_set_value = get_float(20);
        let sl_set_point = get_string(21);
        let sl_set_point_plc = get_string(22);
        let _sl_set_point_comm = get_string(23);

        if !sl_set_point.is_empty() && sl_set_point != "/" {
            definition.sl_set_point_address = Some(sl_set_point);
        }
        if !sl_set_point_plc.is_empty() && sl_set_point_plc != "/" {
            definition.sl_set_point_plc_address = Some(sl_set_point_plc);
        }
        if !_sl_set_point_comm.is_empty() && _sl_set_point_comm != "/" {
            definition.sl_set_point_communication_address = Some(_sl_set_point_comm);
        }

        // 提取SH（高）报警设定
        definition.sh_set_value = get_float(24);
        let sh_set_point = get_string(25);
        let sh_set_point_plc = get_string(26);
        let _sh_set_point_comm = get_string(27);

        if !sh_set_point.is_empty() && sh_set_point != "/" {
            definition.sh_set_point_address = Some(sh_set_point);
        }
        if !sh_set_point_plc.is_empty() && sh_set_point_plc != "/" {
            definition.sh_set_point_plc_address = Some(sh_set_point_plc);
        }
        if !_sh_set_point_comm.is_empty() && _sh_set_point_comm != "/" {
            definition.sh_set_point_communication_address = Some(_sh_set_point_comm);
        }

        // 提取SHH（超高高）报警设定
        definition.shh_set_value = get_float(28);
        let shh_set_point = get_string(29);
        let shh_set_point_plc = get_string(30);
        let _shh_set_point_comm = get_string(31);

        if !shh_set_point.is_empty() && shh_set_point != "/" {
            definition.shh_set_point_address = Some(shh_set_point);
        }
        if !shh_set_point_plc.is_empty() && shh_set_point_plc != "/" {
            definition.shh_set_point_plc_address = Some(shh_set_point_plc);
        }
        if !_shh_set_point_comm.is_empty() && _shh_set_point_comm != "/" {
            definition.shh_set_point_communication_address = Some(_shh_set_point_comm);
        }

        // 继续提取更多字段
        // 第32-43列：LL/L/H/HH报警反馈地址
        // 第44列：维护值设定
        // 第45列：维护值设定点位
        // 第46列：维护值设定点位_PLC地址
        // 第47列：维护值设定点位_通讯地址
        // 第48列：维护使能开关点位
        // 第49列：维护使能开关点位_PLC地址
        // 第50列：维护使能开关点位_通讯地址

        // 提取LL报警反馈地址
        let ll_feedback = get_string(32);
        let ll_feedback_plc = get_string(33);
        let _ll_feedback_comm = get_string(34);

        if !ll_feedback.is_empty() && ll_feedback != "/" {
            definition.sll_feedback_address = Some(ll_feedback);
        }
        if !ll_feedback_plc.is_empty() && ll_feedback_plc != "/" {
            definition.sll_feedback_plc_address = Some(ll_feedback_plc);
        }
        if !_ll_feedback_comm.is_empty() && _ll_feedback_comm != "/" {
            definition.sll_feedback_communication_address = Some(_ll_feedback_comm);
        }

        // 提取L报警反馈地址
        let l_feedback = get_string(35);
        let l_feedback_plc = get_string(36);
        let _l_feedback_comm = get_string(37);

        if !l_feedback.is_empty() && l_feedback != "/" {
            definition.sl_feedback_address = Some(l_feedback);
        }
        if !l_feedback_plc.is_empty() && l_feedback_plc != "/" {
            definition.sl_feedback_plc_address = Some(l_feedback_plc);
        }
        if !_l_feedback_comm.is_empty() && _l_feedback_comm != "/" {
            definition.sl_feedback_communication_address = Some(_l_feedback_comm);
        }

        // 提取H报警反馈地址
        let h_feedback = get_string(38);
        let h_feedback_plc = get_string(39);
        let _h_feedback_comm = get_string(40);

        if !h_feedback.is_empty() && h_feedback != "/" {
            definition.sh_feedback_address = Some(h_feedback);
        }
        if !h_feedback_plc.is_empty() && h_feedback_plc != "/" {
            definition.sh_feedback_plc_address = Some(h_feedback_plc);
        }
        if !_h_feedback_comm.is_empty() && _h_feedback_comm != "/" {
            definition.sh_feedback_communication_address = Some(_h_feedback_comm);
        }

        // 提取HH报警反馈地址
        let hh_feedback = get_string(41);
        let hh_feedback_plc = get_string(42);
        let _hh_feedback_comm = get_string(43);

        if !hh_feedback.is_empty() && hh_feedback != "/" {
            definition.shh_feedback_address = Some(hh_feedback);
        }
        if !hh_feedback_plc.is_empty() && hh_feedback_plc != "/" {
            definition.shh_feedback_plc_address = Some(hh_feedback_plc);
        }
        if !_hh_feedback_comm.is_empty() && _hh_feedback_comm != "/" {
            definition.shh_feedback_communication_address = Some(_hh_feedback_comm);
        }

        // 提取维护相关字段
        let _maintenance_value = get_string(44);
        let maintenance_point = get_string(45);
        let maintenance_point_plc = get_string(46);
        let _maintenance_point_comm = get_string(47);
        let maintenance_enable = get_string(48);
        let maintenance_enable_plc = get_string(49);
        let _maintenance_enable_comm = get_string(50);

        if !maintenance_point.is_empty() && maintenance_point != "/" {
            definition.maintenance_value_set_point_address = Some(maintenance_point);
        }
        if !maintenance_point_plc.is_empty() && maintenance_point_plc != "/" {
            definition.maintenance_value_set_point_plc_address = Some(maintenance_point_plc);
        }
        if !_maintenance_point_comm.is_empty() && _maintenance_point_comm != "/" {
            definition.maintenance_value_set_point_communication_address = Some(_maintenance_point_comm);
        }
        if !maintenance_enable.is_empty() && maintenance_enable != "/" {
            definition.maintenance_enable_switch_point_address = Some(maintenance_enable);
        }
        if !maintenance_enable_plc.is_empty() && maintenance_enable_plc != "/" {
            definition.maintenance_enable_switch_point_plc_address = Some(maintenance_enable_plc);
        }
        if !_maintenance_enable_comm.is_empty() && _maintenance_enable_comm != "/" {
            definition.maintenance_enable_switch_point_communication_address = Some(_maintenance_enable_comm);
        }

        // 注意：PLC绝对地址和上位机通讯地址已经在基础解析中正确设置了
        // 这里不需要重复处理，避免混淆

        // 提取读写属性（第12列）
        let access_property = get_string(11);
        if !access_property.is_empty() && access_property != "/" {
            definition.access_property = Some(access_property);
        }

        // 修复线制字段的默认值设置
        if definition.wire_system.is_empty() {
            definition.wire_system = match definition.module_type {
                ModuleType::AI => "四线制".to_string(),
                ModuleType::AO => "四线制".to_string(),
                ModuleType::DI => "二线制".to_string(),
                ModuleType::DO => "二线制".to_string(),
                _ => "未知".to_string(),
            };
        }



        Ok(())
    }

    /// 从Excel单元格获取字符串值
    fn get_string_value(cell: &calamine::DataType, row_number: usize, column_name: &str) -> AppResult<String> {
        let value = cell.to_string().trim().to_string();
        if value.is_empty() {
            return Err(AppError::validation_error(format!(
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
            _ => Err(AppError::validation_error(format!(
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
            _ => Err(AppError::validation_error(format!(
                "第{}行数据类型'{}'无效，支持的类型: Bool, Int, Float/Real, String",
                row_number,
                type_str
            )))
        }
    }

    /// 解析序列号
    fn parse_sequence_number(cell: &calamine::DataType) -> Option<u32> {
        log::info!("解析序号: {:?}", cell);
        match cell {
            calamine::DataType::String(s) => s.trim().parse::<u32>().ok(),
            calamine::DataType::Float(f) => Some(*f as u32),
            calamine::DataType::Int(i) => Some(*i as u32),
            _ => None,
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
        let real_file_path = r"D:\GIT\Git\code\FactoryTesting\测试文件\测试IO.xlsx";

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