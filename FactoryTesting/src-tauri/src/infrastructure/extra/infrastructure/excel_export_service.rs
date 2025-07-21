use std::path::PathBuf;
use std::sync::Arc;
use chrono::{Local};
use crate::utils::time_utils;
use rust_xlsxwriter::{Workbook, Format, FormatAlign, FormatBorder, Color};

use crate::models::{ChannelPointDefinition, ModuleType, ChannelTestInstance};
use crate::models::enums::{SubTestItem, OverallTestStatus, SubTestStatus};
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
        ModuleType::Communication => Color::RGB(0xFFE6CC), // Light Orange
        ModuleType::Other(_) => Color::White,
    }
}

/// Excel 导出服务
pub struct ExcelExportService {
    persistence_service: Arc<dyn IPersistenceService>,
    channel_state_manager: Arc<dyn IChannelStateManager>,
}

impl ExcelExportService {
    /// 导出测试结果表（全部批次，无过滤）
    /// `target_path` 可以是目录或完整文件路径；为空时写入临时目录。
    /// 返回生成的文件完整路径
    pub async fn export_test_results(&self, target_path: Option<PathBuf>) -> AppResult<String> {
        // 1. 加载所需数据
        let definitions = self.persistence_service.load_all_channel_definitions().await?;
        if definitions.is_empty() {
            return Err(AppError::ValidationError { message: "暂无通道定义，无法导出测试结果".into() });
        }
        let def_map: std::collections::HashMap<String, &ChannelPointDefinition> =
            definitions.iter().map(|d| (d.id.clone(), d)).collect();

        let instances = self.persistence_service.load_all_test_instances().await?;
        if instances.is_empty() {
            return Err(AppError::ValidationError { message: "暂无测试实例数据，无法导出测试结果".into() });
        }

        // 为了避免一条条去数据库查询 outcome，先尝试批量 by batch；若无对应接口，则逐个 fetch
        let mut outcome_cache: std::collections::HashMap<String, Vec<crate::models::structs::RawTestOutcome>> = std::collections::HashMap::new();
        for inst in &instances {
            if !outcome_cache.contains_key(&inst.instance_id) {
                let list = self
                    .persistence_service
                    .load_test_outcomes_by_instance(&inst.instance_id)
                    .await
                    .unwrap_or_default();
                outcome_cache.insert(inst.instance_id.clone(), list);
            }
        }

        // 2. 准备输出文件路径
        let station_name = definitions[0].station_name.clone();
        let timestamp = Local::now().format("%Y%m%d_%H%M").to_string();
        let filename = format!("{}_{}_测试结果.xlsx", station_name, timestamp);
        let file_path: PathBuf = if let Some(p) = target_path.clone() {
            let is_dir_path = p.is_dir() || p.extension().is_none();
            if is_dir_path { p.join(&filename) } else { p }
        } else { std::env::temp_dir().join(&filename) };
        if let Some(parent) = file_path.parent() { std::fs::create_dir_all(parent).ok(); }

        // 3. 创建工作簿和工作表
        let mut workbook = Workbook::new();
        let mut sheet = workbook.add_worksheet();

        // 4. 样式
        let header_fmt = Format::new().set_bold().set_align(FormatAlign::Center).set_border(FormatBorder::Thin);
        let default_fmt = Format::new().set_align(FormatAlign::Center).set_border(FormatBorder::Thin);

        // 5. 表头
        let headers = vec![
            "测试ID", "测试批次", "变量名称", "点表类型", "数据类型", "测试PLC通道位号", "被测PLC通道位号", 
            "行程最小值", "行程最大值", "0%对比值", "25%对比值", "50%对比值", "75%对比值", "100%对比值", 
            "低低报反馈状态", "低报反馈状态", "高报反馈状态", "高高报反馈状态", "维护功能检测", 
            "开始测试时间", "最终测试时间", "测试时长", "通道硬点测试结果", "测试结果"
        ];
        for (col, title) in headers.iter().enumerate() {
            sheet.write_with_format(0, col as u16, *title, &header_fmt)?;
        }

        // 6. 写数据行
        let mut row = 1u32;
        // 按测试ID(定义中的 sequence_number) 升序排序
        let mut instance_refs: Vec<&ChannelTestInstance> = instances.iter().collect();
        instance_refs.sort_by_key(|inst| {
            def_map
                .get(&inst.definition_id)
                .and_then(|d| d.sequence_number)
                .unwrap_or(u32::MAX)
        });

        for (idx, inst) in instance_refs.iter().enumerate() {
            let def = match def_map.get(&inst.definition_id) {
                Some(d) => *d,
                None => continue, // 没找到定义，跳过
            };
            let outcomes = outcome_cache.get(&inst.instance_id).cloned().unwrap_or_default();

            // 提取通用列
            let test_id = def.sequence_number.unwrap_or((idx + 1) as u32);
            let point_type = format!("{:?}", def.module_type); // 点表类型暂用模块类型
            let data_type = format!("{:?}", def.data_type);
            let test_plc_tag = inst.test_plc_channel_tag.clone().unwrap_or_else(|| "-".into());
            let measured_tag = def.tag.clone();
            let range_min = def.range_low_limit.map(|v| v.to_string()).unwrap_or_else(|| "-".into());
            let range_max = def.range_high_limit.map(|v| v.to_string()).unwrap_or_else(|| "-".into());

            // 百分比对比值
            let mut pct_vals = vec!["-".to_string(); 5];
            // 硬点测试结果 & 报警反馈等
            let mut hardpoint_result = "-".to_string();
            let mut maint_result = "-".to_string();
            let mut alarm_vals = vec!["-".to_string(); 4];

            for oc in &outcomes {
                use crate::models::enums::SubTestItem;
                match oc.sub_test_item {
                    SubTestItem::HardPoint => {
                        hardpoint_result = if oc.success { "PASS".into() } else { "FAIL".into() };
                    }
                    // 维护功能检测可能映射为 Maintenance 或 MaintenanceFunction
                    SubTestItem::Maintenance | SubTestItem::MaintenanceFunction => {
                        maint_result = if oc.success { "PASS".into() } else { "FAIL".into() };
                    }
                    // 百分比结果可能记录在单独的 OutputX% 或 TrendCheck 等子项中，统一提取
                    _ => {}
                }

                // 若当前 outcome 含有百分比字段，统一覆盖（只要非 None 即写入）
                //百分比需要增加小数点过滤。
                if let Some(v) = oc.test_result_0_percent { pct_vals[0] = format!("{:.3}", v); }
                if let Some(v) = oc.test_result_25_percent { pct_vals[1] = format!("{:.3}", v); }
                if let Some(v) = oc.test_result_50_percent { pct_vals[2] = format!("{:.3}", v); }
                if let Some(v) = oc.test_result_75_percent { pct_vals[3] = format!("{:.3}", v); }
                if let Some(v) = oc.test_result_100_percent { pct_vals[4] = format!("{:.3}", v); }

                match oc.sub_test_item {
                    SubTestItem::LowLowAlarm => {
                        alarm_vals[0] = if oc.success { "PASS".into() } else { "FAIL".into() };
                    }
                    SubTestItem::LowAlarm => {
                        alarm_vals[1] = if oc.success { "PASS".into() } else { "FAIL".into() };
                    }
                    SubTestItem::HighAlarm => {
                        alarm_vals[2] = if oc.success { "PASS".into() } else { "FAIL".into() };
                    }
                    SubTestItem::HighHighAlarm => {
                        alarm_vals[3] = if oc.success { "PASS".into() } else { "FAIL".into() };
                    }
                    _ => {}
                }
            }

            // 如果维护功能检测仍为 "-"，尝试从实例的 sub_test_results 中获取
            if maint_result == "-" {
                if let Some(res) = inst.sub_test_results.get(&SubTestItem::Maintenance).or_else(|| inst.sub_test_results.get(&SubTestItem::MaintenanceFunction)) {
                    use crate::models::enums::SubTestStatus;
                    maint_result = match res.status {
                        SubTestStatus::Passed => "PASS".into(),
                        SubTestStatus::Failed => "FAIL".into(),
                        _ => "-".into(),
                    };
                }
            }

            // 开始/结束/时长
            let start_time_utc = inst.start_time.unwrap_or_else(|| outcomes.first().map(|o| o.start_time).unwrap_or_else(chrono::Utc::now));
            let end_time_utc = inst.final_test_time.unwrap_or_else(|| outcomes.last().map(|o| o.end_time).unwrap_or(start_time_utc));
            // 转换为本地(北京时间)显示
            let start_time = start_time_utc.with_timezone(&Local);
            let end_time = end_time_utc.with_timezone(&Local);
            let duration = end_time.signed_duration_since(start_time);
            let total_minutes = duration.num_minutes();
            let hours = total_minutes / 60;
            let minutes = total_minutes % 60;
            let duration_ms = duration.num_milliseconds(); // 保留原毫秒以防后续使用
            let duration_fmt = format!("{}小时{}分钟", hours, minutes);

            // 整体测试结果
            let overall = match inst.overall_status {
                crate::models::enums::OverallTestStatus::TestCompletedPassed => "PASS",
                crate::models::enums::OverallTestStatus::TestCompletedFailed => "FAIL",
                crate::models::enums::OverallTestStatus::Skipped => "-",
                _ => "-",
            };

            // 写入单元格
            let values: Vec<String> = vec![
                test_id.to_string(),
                inst.test_batch_name.clone(),
                def.variable_name.clone(),
                point_type,
                data_type,
                test_plc_tag,
                measured_tag,
                range_min,
                range_max,
            ]
            .into_iter()
            .chain(pct_vals.into_iter())
            .chain(alarm_vals.into_iter())
            .chain(vec![maint_result])
            .chain(vec![
                time_utils::format_bj(start_time, "%Y-%m-%d %H:%M:%S"),
                time_utils::format_bj(end_time, "%Y-%m-%d %H:%M:%S"),
                duration_fmt,
                hardpoint_result,
                overall.into(),
            ])
            .collect();

            for (col, val) in values.iter().enumerate() {
                sheet.write_string_with_format(row, col as u16, val, &default_fmt)?;
            }
            row += 1;
        }

        // 自动列宽
        for col_index in 0..headers.len() {
            sheet.set_column_width(col_index as u16, 18)?;
        }

        // 创建错误信息汇总工作表
        self.create_error_summary_sheet(&mut workbook, &instances, &def_map).await?;

        // 保存
        workbook.save(&file_path)?;
        log::info!("📤 [EXPORT] 测试结果 Excel 已保存到 {}", file_path.to_string_lossy());
        Ok(file_path.to_string_lossy().to_string())
    }

    /// 创建错误信息汇总工作表 - 以点位为基线的错误信息汇总
    async fn create_error_summary_sheet(
        &self,
        workbook: &mut Workbook,
        instances: &[ChannelTestInstance],
        def_map: &std::collections::HashMap<String, &ChannelPointDefinition>,
    ) -> AppResult<()> {
        let mut error_sheet = workbook.add_worksheet().set_name("错误信息汇总")?;

        // 样式定义
        let title_fmt = Format::new()
            .set_bold()
            .set_font_size(16)
            .set_align(FormatAlign::Center)
            .set_background_color(Color::RGB(0x4F81BD))
            .set_font_color(Color::White)
            .set_border(FormatBorder::Thin);

        let header_fmt = Format::new()
            .set_bold()
            .set_font_size(12)
            .set_align(FormatAlign::Center)
            .set_background_color(Color::RGB(0xDCE6F1))
            .set_border(FormatBorder::Thin);

        let channel_fmt = Format::new()
            .set_bold()
            .set_font_size(11)
            .set_align(FormatAlign::Center)
            .set_background_color(Color::RGB(0xE8F4FD))
            .set_border(FormatBorder::Thin);

        let hardpoint_fmt = Format::new()
            .set_font_size(12)
            .set_align(FormatAlign::Center)
            .set_border(FormatBorder::Thin)
            .set_background_color(Color::RGB(0xFFF2F0))
            .set_text_wrap();

        let manual_fmt = Format::new()
            .set_font_size(12)
            .set_align(FormatAlign::Center)
            .set_border(FormatBorder::Thin)
            .set_background_color(Color::RGB(0xFFF7E6))
            .set_text_wrap();

        let notes_fmt = Format::new()
            .set_font_size(12)
            .set_align(FormatAlign::Center)
            .set_border(FormatBorder::Thin)
            .set_background_color(Color::RGB(0xF6FFED))
            .set_text_wrap();

        let stats_fmt = Format::new()
            .set_font_size(12)
            .set_align(FormatAlign::Center)
            .set_border(FormatBorder::Thin)
            .set_background_color(Color::RGB(0xF0F2F5));

        // 主标题
        error_sheet.merge_range(0, 0, 0, 7, "错误信息汇总报告 - 按点位分类", &title_fmt)?;

        // 表头
        let headers = vec![
            "点位名称", "通道位号", "点位描述", "通道类型", 
            "硬点测试错误汇总", "手动测试错误汇总", "用户错误备注汇总", "测试时间"
        ];
        
        let mut current_row = 2u32;
        for (col, header) in headers.iter().enumerate() {
            error_sheet.write_with_format(current_row, col as u16, *header, &header_fmt)?;
        }
        current_row += 1;

        // 设置列宽
        error_sheet.set_column_width(0, 15.0)?; // 点位名称
        error_sheet.set_column_width(1, 12.0)?; // 通道位号  
        error_sheet.set_column_width(2, 20.0)?; // 点位描述
        error_sheet.set_column_width(3, 10.0)?; // 通道类型
        error_sheet.set_column_width(4, 30.0)?; // 硬点测试错误汇总
        error_sheet.set_column_width(5, 25.0)?; // 手动测试错误汇总
        error_sheet.set_column_width(6, 25.0)?; // 用户错误备注汇总
        error_sheet.set_column_width(7, 18.0)?; // 测试时间

        // 筛选出有错误的点位
        let mut error_instances = Vec::new();
        for instance in instances {
            if instance.overall_status == OverallTestStatus::TestCompletedFailed {
                if let Some(_def) = def_map.get(&instance.definition_id) {
                    // 检查是否有任何类型的错误
                    let has_hardpoint_error = self.has_hardpoint_error(instance).await;
                    let has_manual_error = self.has_manual_test_error(instance).await;
                    let has_user_notes = instance.integration_error_notes.as_ref().map(|s| !s.trim().is_empty()).unwrap_or(false)
                        || instance.plc_programming_error_notes.as_ref().map(|s| !s.trim().is_empty()).unwrap_or(false)
                        || instance.hmi_configuration_error_notes.as_ref().map(|s| !s.trim().is_empty()).unwrap_or(false);

                    if has_hardpoint_error || has_manual_error || has_user_notes {
                        error_instances.push(instance);
                    }
                }
            }
        }

        // 如果没有错误实例，显示提示信息
        if error_instances.is_empty() {
            error_sheet.merge_range(current_row, 0, current_row, 7, "所有点位测试均通过，无错误信息", &stats_fmt)?;
            current_row += 2;
        } else {
            // 按点位名称排序
            error_instances.sort_by(|a, b| {
                let def_a = def_map.get(&a.definition_id);
                let def_b = def_map.get(&b.definition_id);
                match (def_a, def_b) {
                    (Some(da), Some(db)) => da.tag.cmp(&db.tag),
                    _ => std::cmp::Ordering::Equal,
                }
            });

            // 为每个有错误的点位创建一行
            for instance in error_instances {
                if let Some(def) = def_map.get(&instance.definition_id) {
                    let start_row = current_row;

                    // 点位基本信息
                    error_sheet.write_with_format(current_row, 0, &def.tag, &channel_fmt)?;
                    error_sheet.write_with_format(current_row, 1, &def.channel_tag_in_module, &channel_fmt)?;
                    error_sheet.write_with_format(current_row, 2, &def.variable_description, &channel_fmt)?;
                    error_sheet.write_with_format(current_row, 3, format!("{:?}", def.module_type), &channel_fmt)?;

                    // 硬点测试错误汇总
                    let hardpoint_errors = self.get_hardpoint_error_summary(instance).await;
                    let hardpoint_lines = hardpoint_errors.matches('\n').count() + 1;
                    error_sheet.write_with_format(current_row, 4, &hardpoint_errors, &hardpoint_fmt)?;

                    // 手动测试错误汇总
                    let manual_errors = self.get_manual_test_error_summary(instance).await;
                    let manual_lines = manual_errors.matches('\n').count() + 1;
                    error_sheet.write_with_format(current_row, 5, &manual_errors, &manual_fmt)?;

                    // 用户错误备注汇总
                    let user_notes = self.get_user_notes_summary(instance);
                    let notes_lines = user_notes.matches('\n').count() + 1;
                    error_sheet.write_with_format(current_row, 6, &user_notes, &notes_fmt)?;

                    // 测试时间
                    let test_time = instance.final_test_time
                        .map(|t| t.with_timezone(&Local).format("%Y-%m-%d %H:%M:%S").to_string())
                        .unwrap_or_else(|| "-".into());
                    error_sheet.write_with_format(current_row, 7, test_time, &stats_fmt)?;

                    // 设置行高，根据内容调整
                    let max_lines = hardpoint_lines.max(manual_lines).max(notes_lines);
                    
                    if max_lines > 1 {
                        error_sheet.set_row_height(current_row, (max_lines as f64 * 18.0).max(30.0))?;
                    }

                    current_row += 1;
                }
            }
        }

        // === 错误统计信息汇总 ===
        current_row += 1;
        error_sheet.merge_range(current_row, 0, current_row, 6, "错误统计信息汇总", &title_fmt)?;
        current_row += 1;

        let hardpoint_failed = instances.iter().filter(|i| {
            if i.overall_status == OverallTestStatus::TestCompletedFailed {
                // 使用同步方式检查硬点错误
                for (test_item, result) in &i.sub_test_results {
                    if test_item == &SubTestItem::HardPoint && matches!(result.status, SubTestStatus::Failed) {
                        return true;
                    }
                }
            }
            false
        }).count();
        
        let manual_failed = instances.iter().filter(|i| {
            if i.overall_status == OverallTestStatus::TestCompletedFailed {
                for (test_item, result) in &i.sub_test_results {
                    if matches!(test_item, SubTestItem::LowLowAlarm | SubTestItem::LowAlarm | SubTestItem::HighAlarm | SubTestItem::HighHighAlarm | SubTestItem::StateDisplay) {
                        if matches!(result.status, SubTestStatus::Failed) {
                            return true;
                        }
                    }
                }
            }
            false
        }).count();

        let stats = vec![
            ("硬点测试失败", hardpoint_failed),
            ("手动测试失败", manual_failed),
        ];

        for (i, (label, count)) in stats.iter().enumerate() {
            error_sheet.write_with_format(current_row + i as u32, 0, *label, &header_fmt)?;
            error_sheet.write_with_format(current_row + i as u32, 1, *count as f64, &stats_fmt)?;
        }

        // 设置列宽 - 优化显示效果
        let column_widths = vec![20, 25, 15, 35, 35, 35, 20];
        for (col, width) in column_widths.iter().enumerate() {
            error_sheet.set_column_width(col as u16, *width)?;
        }

        // 设置行高以适应文本换行
        for row in 3..current_row {
            error_sheet.set_row_height(row, 30)?;
        }

        log::info!("📊 [EXPORT] 以点位为基线的错误信息汇总工作表创建完成");
        Ok(())
    }

    /// 判断是否为硬点测试错误
    async fn is_hardpoint_test_error(&self, instance: &ChannelTestInstance, _error_msg: &str) -> bool {
        self.has_hardpoint_error(instance).await
    }

    /// 检查是否有硬点测试错误
    async fn has_hardpoint_error(&self, instance: &ChannelTestInstance) -> bool {
        for (test_item, result) in &instance.sub_test_results {
            if test_item == &SubTestItem::HardPoint {
                return matches!(result.status, SubTestStatus::Failed);
            }
        }
        false
    }

    /// 检查是否有手动测试错误
    async fn has_manual_test_error(&self, instance: &ChannelTestInstance) -> bool {
        for (test_item, result) in &instance.sub_test_results {
            if matches!(test_item, SubTestItem::LowLowAlarm | SubTestItem::LowAlarm | SubTestItem::HighAlarm | SubTestItem::HighHighAlarm | SubTestItem::StateDisplay) {
                if matches!(result.status, SubTestStatus::Failed) {
                    return true;
                }
            }
        }
        false
    }

    /// 获取硬点测试错误汇总
    async fn get_hardpoint_error_summary(&self, instance: &ChannelTestInstance) -> String {
        for (test_item, result) in &instance.sub_test_results {
            if test_item == &SubTestItem::HardPoint && matches!(result.status, SubTestStatus::Failed) {
                let mut summary = String::new();
                
                // 错误详情
                if let Some(details) = &result.details {
                    summary.push_str(&format!("错误信息: {}\n", details));
                    summary.push_str("\n");
                }
                
                // 详细的5个检查点数据
                summary.push_str("检查点详情:\n");
                summary.push_str("点位    | 期望值   | 实际值   | 偏差\n");
                summary.push_str("-------|---------|---------|--------\n");
                
                // 0%检查点
                if let Some(actual_0) = instance.test_result_0_percent {
                    let expected_0 = self.calculate_expected_value(instance, 0.0).await;
                    let deviation_0 = self.calculate_deviation(expected_0, actual_0);
                    summary.push_str(&format!("0%     | {:.2}    | {:.2}    | {:.2}%\n", expected_0, actual_0, deviation_0));
                }
                
                // 25%检查点
                if let Some(actual_25) = instance.test_result_25_percent {
                    let expected_25 = self.calculate_expected_value(instance, 0.25).await;
                    let deviation_25 = self.calculate_deviation(expected_25, actual_25);
                    summary.push_str(&format!("25%    | {:.2}    | {:.2}    | {:.2}%\n", expected_25, actual_25, deviation_25));
                }
                
                // 50%检查点
                if let Some(actual_50) = instance.test_result_50_percent {
                    let expected_50 = self.calculate_expected_value(instance, 0.5).await;
                    let deviation_50 = self.calculate_deviation(expected_50, actual_50);
                    summary.push_str(&format!("50%    | {:.2}    | {:.2}    | {:.2}%\n", expected_50, actual_50, deviation_50));
                }
                
                // 75%检查点
                if let Some(actual_75) = instance.test_result_75_percent {
                    let expected_75 = self.calculate_expected_value(instance, 0.75).await;
                    let deviation_75 = self.calculate_deviation(expected_75, actual_75);
                    summary.push_str(&format!("75%    | {:.2}    | {:.2}    | {:.2}%\n", expected_75, actual_75, deviation_75));
                }
                
                // 100%检查点
                if let Some(actual_100) = instance.test_result_100_percent {
                    let expected_100 = self.calculate_expected_value(instance, 1.0).await;
                    let deviation_100 = self.calculate_deviation(expected_100, actual_100);
                    summary.push_str(&format!("100%   | {:.2}    | {:.2}    | {:.2}%\n", expected_100, actual_100, deviation_100));
                }
                
                // 如果没有百分比数据，显示原有的期望值和实际值
                if instance.test_result_0_percent.is_none() && 
                   instance.test_result_25_percent.is_none() && 
                   instance.test_result_50_percent.is_none() && 
                   instance.test_result_75_percent.is_none() && 
                   instance.test_result_100_percent.is_none() {
                    if let Some(expected) = &result.expected_value {
                        summary.push_str(&format!("期望值: {}\n", expected));
                    }
                    if let Some(actual) = &result.actual_value {
                        summary.push_str(&format!("实际值: {}\n", actual));
                    }
                }
                
                // 如果没有详细信息，至少显示失败状态
                if summary.trim().is_empty() {
                    summary = "硬点测试失败".to_string();
                }
                
                return summary.trim().to_string();
            }
        }
        "-".to_string()
    }

    /// 计算期望值（基于通道范围和百分比）
    async fn calculate_expected_value(&self, instance: &ChannelTestInstance, percentage: f64) -> f64 {
        // 获取通道定义以获取工程量范围
        if let Ok(definitions) = self.persistence_service.load_all_channel_definitions().await {
            if let Some(def) = definitions.iter().find(|d| d.id == instance.definition_id) {
                // 计算工程量范围内的期望值
                let min_value = def.range_low_limit.unwrap_or(0.0) as f64;
                let max_value = def.range_high_limit.unwrap_or(100.0) as f64;
                return min_value + (max_value - min_value) * percentage;
            }
        }
        
        // 如果无法获取工程量范围，使用默认范围 0-100
        percentage * 100.0
    }
    
    /// 计算偏差百分比
    fn calculate_deviation(&self, expected: f64, actual: f64) -> f64 {
        if expected == 0.0 {
            if actual == 0.0 { 0.0 } else { 100.0 }
        } else {
            ((actual - expected) / expected.abs()) * 100.0
        }
    }

    /// 获取手动测试错误汇总
    async fn get_manual_test_error_summary(&self, instance: &ChannelTestInstance) -> String {
        let mut failed_items = Vec::new();
        
        for (test_item, result) in &instance.sub_test_results {
            if matches!(test_item, SubTestItem::LowLowAlarm | SubTestItem::LowAlarm | SubTestItem::HighAlarm | SubTestItem::HighHighAlarm | SubTestItem::StateDisplay) {
                if matches!(result.status, SubTestStatus::Failed) {
                    let item_name = match test_item {
                        SubTestItem::LowLowAlarm => "低低报警测试",
                        SubTestItem::LowAlarm => "低报警测试",
                        SubTestItem::HighAlarm => "高报警测试",
                        SubTestItem::HighHighAlarm => "高高报警测试",
                        SubTestItem::StateDisplay => "状态显示测试",
                        _ => "未知测试项",
                    };
                    
                    let details = result.details.as_ref().map(|d| format!(" ({})", d)).unwrap_or_default();
                    failed_items.push(format!("{}{}", item_name, details));
                }
            }
        }
        
        if failed_items.is_empty() {
            "-".to_string()
        } else {
            failed_items.join("\n")
        }
    }

    /// 获取用户错误备注汇总
    fn get_user_notes_summary(&self, instance: &ChannelTestInstance) -> String {
        let mut notes = Vec::new();
        
        if let Some(integration) = &instance.integration_error_notes {
            if !integration.trim().is_empty() {
                notes.push(format!("集成错误: {}", integration.trim()));
            }
        }
        
        if let Some(plc) = &instance.plc_programming_error_notes {
            if !plc.trim().is_empty() {
                notes.push(format!("PLC编程: {}", plc.trim()));
            }
        }
        
        if let Some(hmi) = &instance.hmi_configuration_error_notes {
            if !hmi.trim().is_empty() {
                notes.push(format!("上位机组态: {}", hmi.trim()));
            }
        }
        
        if notes.is_empty() {
            "-".to_string()
        } else {
            notes.join("\n")
        }
    }

    /// 获取失败的手动测试项
    async fn get_failed_manual_tests(&self, instance: &ChannelTestInstance) -> Vec<(String, String)> {
        let mut failed_tests = Vec::new();
        
        for (test_item, result) in &instance.sub_test_results {
            // 手动测试相关的项目
            if matches!(test_item, SubTestItem::LowLowAlarm | SubTestItem::LowAlarm | SubTestItem::HighAlarm | SubTestItem::HighHighAlarm | SubTestItem::StateDisplay) {
                if matches!(result.status, SubTestStatus::Failed) {
                    let details = result.details.clone().unwrap_or_else(|| "测试失败".into());
                    failed_tests.push((format!("{:?}", test_item), details));
                }
            }
        }
        
        failed_tests
    }

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
