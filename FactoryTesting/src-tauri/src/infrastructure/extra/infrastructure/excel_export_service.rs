use std::path::PathBuf;
use std::sync::Arc;
use chrono::Local;
use rust_xlsxwriter::{Workbook, Format, FormatAlign, FormatBorder, Color};

use crate::models::{ChannelPointDefinition, ModuleType, ChannelTestInstance};
use crate::utils::error::{AppResult, AppError};
use crate::infrastructure::IPersistenceService;
use crate::domain::services::IChannelStateManager;

/// 颜色常量（柔和不刺眼）
fn color_for_module(module_type: &ModuleType) -> Color {
    match module_type {
        ModuleType::AI | ModuleType::AINone => Color::RGB(0xB0E0E6), // PowderBlue
        ModuleType::AO | ModuleType::AONone => Color::RGB(0xC5E1A5), // LightGreen
        ModuleType::DI | ModuleType::DINone => Color::RGB(0xFFF59D), // LightYellow
        ModuleType::DO | ModuleType::DONone => Color::RGB(0xE1BEE7), // Lavender
        _ => Color::White,
    }
}

/// Excel 导出服务
pub struct ExcelExportService {
    persistence_service: Arc<dyn IPersistenceService>,
    channel_state_manager: Arc<dyn IChannelStateManager>,
}

impl ExcelExportService {
    pub fn new(
        persistence_service: Arc<dyn IPersistenceService>,
        channel_state_manager: Arc<dyn IChannelStateManager>,
    ) -> Self {
        Self { persistence_service, channel_state_manager }
    }

    /// 导出通道分配表（向后兼容，默认导出全部定义）
    pub async fn export_channel_allocation(&self, target_path: Option<PathBuf>) -> AppResult<String> {
        self.export_channel_allocation_with_filter(target_path, None).await
    }

    /// 导出带过滤的通道分配表
    /// 如果 provided_batch_ids 为 Some(vec)，则只导出这些批次的点位；
    /// 否则默认导出当前会话(由调用方保证)全部。
    pub async fn export_channel_allocation_with_filter(&self, target_path: Option<PathBuf>, provided_batch_ids: Option<Vec<String>>) -> AppResult<String> {
        // 加载全部定义
        let mut definitions = self.persistence_service.load_all_channel_definitions().await?;

        if let Some(batch_ids) = &provided_batch_ids {
            // 🔧 修复: 若 definition.batch_id 为空, 使用其关联实例的 test_batch_id 进行匹配
            let set: std::collections::HashSet<String> = batch_ids.iter().cloned().collect();

            // 先构建 definition_id → test_batch_id 映射
            let instance_list = self.persistence_service.load_all_test_instances().await.unwrap_or_default();
            let mut inst_batch_map: std::collections::HashMap<String, String> = std::collections::HashMap::new();
            for inst in instance_list {
                inst_batch_map.insert(inst.definition_id.clone(), inst.test_batch_id.clone());
            }

            definitions.retain(|def| {
                // 1) 优先用 definition 本身 batch_id 判断
                if let Some(bid) = &def.batch_id {
                    if set.contains(bid) { return true; }
                }
                // 2) 回退到实例映射
                if let Some(bid) = inst_batch_map.get(&def.id) {
                    return set.contains(bid);
                }
                false
            });
        }

        log::info!("📤 [EXPORT] 过滤后剩余 {} 条通道定义", definitions.len());

        // 调用现有逻辑
        self.export_channel_allocation_inner(target_path, definitions).await
    }

    // 重构：将原主要实现体提取成内部函数，方便复用
    async fn export_channel_allocation_inner(&self, target_path: Option<PathBuf>, definitions: Vec<ChannelPointDefinition>) -> AppResult<String> {
        if definitions.is_empty() {
            return Err(AppError::ValidationError { message: "暂无通道数据可导出".into() });
        }

        let station_name = &definitions[0].station_name;
        let timestamp = Local::now().format("%Y%m%d_%H%M").to_string();
        let filename = format!("{}_{}_通道分配表.xlsx", station_name, timestamp);

        let file_path: PathBuf = if let Some(p) = target_path.clone() {
            let is_dir_path = p.is_dir() || p.extension().is_none();
            if is_dir_path {
                p.join(&filename)
            } else { p }
        } else { std::env::temp_dir().join(&filename) };

        if let Some(parent) = file_path.parent() { std::fs::create_dir_all(parent).ok(); }

        // -----------------------------------------------------------------------------
        // 加载所有测试实例，建立 definition_id -> instance 映射，并基于实例的 batch_id
        // 生成 "批次显示名" 映射（batch_id → 批次N）。这样即使 definition 未写入 batch_id
        // 也能正确显示。例如导入流程未给 definition 赋值时仍可通过实例获取。
        // -----------------------------------------------------------------------------
        let instance_list = self.persistence_service.load_all_test_instances().await.unwrap_or_default();
        log::info!("📤 [EXPORT] 从数据库加载到 {} 条测试实例", instance_list.len());
        let instance_map: std::collections::HashMap<String, ChannelTestInstance> = instance_list
            .into_iter()
            .map(|inst| (inst.definition_id.clone(), inst))
            .collect();

        // -----------------------------------------------------------------
        // 对 definitions 进行去重：相同 站场名+Tag 只取第一条记录
        // -----------------------------------------------------------------
        let mut unique_definitions: Vec<&ChannelPointDefinition> = Vec::new();
        let mut seen_keys: std::collections::HashSet<String> = std::collections::HashSet::new();
        for def in &definitions {
            let key = format!("{}::{}", def.station_name, def.tag);
            if seen_keys.insert(key) {
                unique_definitions.push(def);
            }
        }

        // 对 unique_definitions 根据 "批次名称 → sequence_number" 排序，确保导出顺序正确
        use std::cmp::Ordering;
        fn extract_batch_num(name: &str) -> u32 {
            name.trim_start_matches('批').trim_start_matches('次').trim_start_matches(' ').trim_start_matches(|c: char| !c.is_ascii_digit())
                .chars()
                .take_while(|c| c.is_ascii_digit())
                .collect::<String>()
                .parse::<u32>()
                .unwrap_or(u32::MAX)
        }

        unique_definitions.sort_by(|a, b| {
            let name_a = instance_map.get(&a.id).map(|i| i.test_batch_name.clone()).unwrap_or_default();
            let name_b = instance_map.get(&b.id).map(|i| i.test_batch_name.clone()).unwrap_or_default();
            let num_cmp = extract_batch_num(&name_a).cmp(&extract_batch_num(&name_b));
            if num_cmp != Ordering::Equal {
                return num_cmp;
            }
            let seq_a = a.sequence_number.unwrap_or(0);
            let seq_b = b.sequence_number.unwrap_or(0);
            seq_a.cmp(&seq_b)
        });

        // 2. 创建 Workbook
        let mut workbook = Workbook::new();
        let mut sheet = workbook.add_worksheet();

        // 3. 表头格式
        let header_fmt = Format::new()
            .set_bold()
            .set_align(FormatAlign::Center)
            .set_border(FormatBorder::Thin);
        let default_fmt = Format::new()
            .set_align(FormatAlign::Center)
            .set_border(FormatBorder::Thin);

        // 4. 写表头
        let headers = vec![
            "站场名", "测试ID", "测试批次", "变量名称", "变量描述", "模块类型",
            "测试PLC通道位号", "被测PLC通道位号", "被测PLC模块型号", "供电类型", "线制",
        ];
        for (col, title) in headers.iter().enumerate() {
            sheet.write_with_format(0, col as u16, *title, &header_fmt)?;
        }

        // 5. 写数据行
        let mut current_row = 1u32;
        for (index, def) in unique_definitions.iter().enumerate() {
            // 测试ID (保持原点表序号)
            let test_id = def.sequence_number.unwrap_or((index + 1) as u32);

            sheet.write_string_with_format(current_row, 0u16, &def.station_name, &default_fmt)?;
            sheet.write_number_with_format(current_row, 1u16, test_id as f64, &default_fmt)?;
            // 获取批次名称
            let batch_display = instance_map
                .get(&def.id)
                .map(|inst| inst.test_batch_name.clone())
                .filter(|name| !name.is_empty())
                .unwrap_or_else(|| "未知批次".to_string());
            sheet.write_string_with_format(current_row, 2u16, &batch_display, &default_fmt)?;
            sheet.write_string_with_format(current_row, 3u16, &def.variable_name, &default_fmt)?;
            sheet.write_string_with_format(current_row, 4u16, &def.variable_description, &default_fmt)?;

            // 模块类型单元格带颜色
            let module_fmt = Format::new()
                .set_align(FormatAlign::Center)
                .set_border(FormatBorder::Thin)
                .set_background_color(color_for_module(&def.module_type));
            sheet.write_with_format(current_row, 5u16, format!("{:?}", def.module_type), &module_fmt)?;

            // 测试PLC通道位号：从测试实例映射获取
            if let Some(inst) = instance_map.get(&def.id) {
                if let Some(ref tag) = inst.test_plc_channel_tag {
                    sheet.write_string_with_format(current_row, 6u16, tag, &default_fmt)?;
                } else {
                    sheet.write_blank(current_row, 6u16, &default_fmt)?;
                }
            } else {
                sheet.write_blank(current_row, 6u16, &default_fmt)?;
            }

            sheet.write_string_with_format(current_row, 7u16, &def.channel_tag_in_module, &default_fmt)?; // 临时用 channel tag
            sheet.write_string_with_format(current_row, 8u16, &def.module_name, &default_fmt)?;
            sheet.write_string_with_format(current_row, 9u16, &def.power_supply_type, &default_fmt)?;
            sheet.write_string_with_format(current_row, 10u16, &def.wire_system, &default_fmt)?;

            current_row += 1;
        }

        // 6. 合并站场列（整列）
        let last_row = current_row - 1;
        if last_row > 1 {
            sheet.merge_range(1, 0, last_row, 0, &unique_definitions[0].station_name, &default_fmt)?;
        }

        // 7. 合并测试批次列（相同批次连续行）
        // 使用批次名称进行合并
        self.merge_same_values(&mut sheet, 2, 1, last_row, &default_fmt, |r| {
            let idx = (r - 1) as usize;
            instance_map
                .get(&unique_definitions[idx].id)
                .map(|inst| inst.test_batch_name.clone())
                .unwrap_or_default()
        })?;

        // 8. 合并被测PLC模块型号列（机架_槽 维度）
        self.merge_same_values(&mut sheet, 8, 1, last_row, &default_fmt, |row| {
            let idx = (row - 1) as usize;
            unique_definitions[idx].module_name.clone()
        })?;

        // 9. 自动列宽 & 居中显示
        for col in 0..headers.len() {
            sheet.set_column_width(col as u16, 20)?;
        }

        // 10. 保存
        if last_row < 1 {
            workbook.save(&file_path)?;
            log::warn!("📤 [EXPORT] 无数据行，仅创建空文件 {}", file_path.to_string_lossy());
            return Ok(file_path.to_string_lossy().to_string());
        }
        workbook.save(&file_path)?;
        log::info!("📤 [EXPORT] Excel 文件已保存到 {}", file_path.to_string_lossy());
        Ok(file_path.to_string_lossy().to_string())
    }

    /// 将同一列中相邻且值相同的单元格进行合并
    fn merge_same_values<F>(
        &self,
        sheet: &mut rust_xlsxwriter::Worksheet,
        col: u16,
        start_row: u32,
        end_row: u32,
        fmt: &Format,
        value_fn: F,
    ) -> Result<(), rust_xlsxwriter::XlsxError>
    where
        F: Fn(u32) -> String,
    {
        if end_row <= start_row {
            return Ok(());
        }

        let mut current_value: Option<String> = None;
        let mut range_start = start_row;

        for r in start_row..=end_row {
            let cell_value = value_fn(r);

            if current_value.as_ref().map(|v| v == &cell_value).unwrap_or(false) {
                // same, continue accumulating
            } else {
                // flush previous group
                if let Some(val) = current_value.take() {
                    if r - 1 > range_start {
                        sheet.merge_range(range_start, col, r - 1, col, &val, fmt)?;
                    }
                }
                current_value = Some(cell_value);
                range_start = r;
            }
        }

        // flush last group
        if let Some(val) = current_value {
            if end_row > range_start {
                sheet.merge_range(range_start, col, end_row, col, &val, fmt)?;
            }
        }

        Ok(())
    }
} 
