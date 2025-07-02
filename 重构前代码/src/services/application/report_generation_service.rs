/// 报告生成服务
/// 
/// 负责生成PDF、Excel等格式的测试报告

use crate::models::{
    TestReport, ReportTemplate, ReportGenerationRequest, ReportType, ReportStatus,
    ChannelTestInstance, RawTestOutcome
};
use crate::services::infrastructure::IPersistenceService;
use crate::utils::error::{AppError, AppResult};
use async_trait::async_trait;
use chrono::Utc;
use log::{info, warn};
use printpdf::*;
use rust_xlsxwriter::{Workbook, Format};
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tera::{Tera, Context};
use uuid::Uuid;

/// 报告生成服务接口
#[async_trait]
pub trait IReportGenerationService: Send + Sync {
    /// 生成PDF报告
    async fn generate_pdf_report(
        &self,
        request: ReportGenerationRequest,
        user_id: &str,
    ) -> AppResult<TestReport>;

    /// 生成Excel报告
    async fn generate_excel_report(
        &self,
        request: ReportGenerationRequest,
        user_id: &str,
    ) -> AppResult<TestReport>;

    /// 获取报告列表
    async fn get_reports(&self, _batch_id: Option<&str>) -> AppResult<Vec<TestReport>>;

    /// 获取报告模板列表
    async fn get_templates(&self) -> AppResult<Vec<ReportTemplate>>;

    /// 创建报告模板
    async fn create_template(&self, _template: ReportTemplate) -> AppResult<()>;

    /// 更新报告模板
    async fn update_template(&self, _template: ReportTemplate) -> AppResult<()>;

    /// 删除报告模板
    async fn delete_template(&self, _template_id: &str) -> AppResult<()>;

    /// 删除报告文件
    async fn delete_report(&self, _report_id: &str) -> AppResult<()>;
}

/// 报告生成服务实现
pub struct ReportGenerationService {
    persistence_service: Arc<dyn IPersistenceService>,
    template_engine: Tera,
    reports_dir: PathBuf,
}

impl ReportGenerationService {
    /// 创建新的报告生成服务
    pub fn new(
        persistence_service: Arc<dyn IPersistenceService>,
        reports_dir: PathBuf,
    ) -> AppResult<Self> {
        // 确保报告目录存在
        if !reports_dir.exists() {
            fs::create_dir_all(&reports_dir)
                .map_err(|e| AppError::io_error("创建报告目录失败".to_string(), e.to_string()))?;
        }

        // 初始化模板引擎
        let mut tera = Tera::new("templates/**/*")
            .unwrap_or_else(|_| Tera::new("").unwrap());

        // 添加默认模板
        Self::add_default_templates(&mut tera)?;

        Ok(Self {
            persistence_service,
            template_engine: tera,
            reports_dir,
        })
    }

    /// 添加默认模板
    fn add_default_templates(tera: &mut Tera) -> AppResult<()> {
        // 默认PDF报告模板
        let pdf_template = r#"
<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <title>测试报告 - {{ batch.product_model }}</title>
    <style>
        body { font-family: 'SimSun', serif; margin: 20px; }
        .header { text-align: center; margin-bottom: 30px; }
        .info-table { width: 100%; border-collapse: collapse; margin-bottom: 20px; }
        .info-table th, .info-table td { border: 1px solid #ccc; padding: 8px; text-align: left; }
        .info-table th { background-color: #f5f5f5; }
        .summary { margin: 20px 0; }
        .test-results { margin-top: 20px; }
        .result-table { width: 100%; border-collapse: collapse; }
        .result-table th, .result-table td { border: 1px solid #ccc; padding: 6px; text-align: center; }
        .result-table th { background-color: #f0f0f0; }
        .passed { color: green; font-weight: bold; }
        .failed { color: red; font-weight: bold; }
    </style>
</head>
<body>
    <div class="header">
        <h1>工厂验收测试报告</h1>
        <h2>{{ batch.product_model }} - {{ batch.serial_number }}</h2>
    </div>

    <table class="info-table">
        <tr><th>产品型号</th><td>{{ batch.product_model }}</td></tr>
        <tr><th>序列号</th><td>{{ batch.serial_number }}</td></tr>
        <tr><th>操作员</th><td>{{ batch.operator_name }}</td></tr>
        <tr><th>测试开始时间</th><td>{{ batch.creation_time }}</td></tr>
        <tr><th>报告生成时间</th><td>{{ report.generated_at }}</td></tr>
    </table>

    <div class="summary">
        <h3>测试摘要</h3>
        <p>总测试点数: {{ statistics.total_points }}</p>
        <p>通过点数: <span class="passed">{{ statistics.passed_points }}</span></p>
        <p>失败点数: <span class="failed">{{ statistics.failed_points }}</span></p>
        <p>成功率: {{ statistics.success_rate }}%</p>
    </div>

    <div class="test-results">
        <h3>详细测试结果</h3>
        <table class="result-table">
            <thead>
                <tr>
                    <th>位号</th>
                    <th>描述</th>
                    <th>模块类型</th>
                    <th>测试状态</th>
                    <th>测试时间</th>
                </tr>
            </thead>
            <tbody>
                {% for instance in instances %}
                <tr>
                    <td>{{ instance.definition.tag }}</td>
                    <td>{{ instance.definition.variable_description }}</td>
                    <td>{{ instance.definition.module_type }}</td>
                    <td class="{% if instance.overall_status == 'TestCompletedPassed' %}passed{% else %}failed{% endif %}">
                        {{ instance.overall_status }}
                    </td>
                    <td>{{ instance.creation_time }}</td>
                </tr>
                {% endfor %}
            </tbody>
        </table>
    </div>
</body>
</html>
        "#;

        tera.add_raw_template("default_pdf", pdf_template)
            .map_err(|e| AppError::template_error(format!("添加默认PDF模板失败: {}", e)))?;

        info!("默认报告模板已添加");
        Ok(())
    }

    /// 收集报告数据
    async fn collect_report_data(&self, batch_ids: &[String]) -> AppResult<HashMap<String, Value>> {
        let mut data = HashMap::new();

        for batch_id in batch_ids {
            // 获取批次信息
            let batches = self.persistence_service.load_all_batch_info().await?;
            let batch = batches.into_iter()
                .find(|b| &b.batch_id == batch_id)
                .ok_or_else(|| AppError::not_found_error("批次未找到".to_string(), batch_id.clone()))?;

            // 获取通道定义
            let definitions = self.persistence_service.load_all_channel_definitions().await?;

            // 暂时使用空的测试实例和结果，因为相关方法还未实现
            let instances: Vec<ChannelTestInstance> = Vec::new();
            let outcomes: Vec<RawTestOutcome> = Vec::new();

            // 计算统计数据
            let statistics = self.calculate_statistics(&instances, &outcomes);

            // 组装数据
            data.insert("batch".to_string(), serde_json::to_value(&batch)?);
            data.insert("instances".to_string(), serde_json::to_value(&instances)?);
            data.insert("definitions".to_string(), serde_json::to_value(&definitions)?);
            data.insert("outcomes".to_string(), serde_json::to_value(&outcomes)?);
            data.insert("statistics".to_string(), serde_json::to_value(&statistics)?);
        }

        Ok(data)
    }

    /// 计算统计数据
    fn calculate_statistics(
        &self,
        instances: &[ChannelTestInstance],
        _outcomes: &[RawTestOutcome],
    ) -> HashMap<String, Value> {
        let mut stats = HashMap::new();

        let total_points = instances.len() as u32;
        let passed_points = instances.iter()
            .filter(|i| matches!(i.overall_status, crate::models::OverallTestStatus::TestCompletedPassed))
            .count() as u32;
        let failed_points = total_points - passed_points;
        let success_rate = if total_points > 0 {
            (passed_points as f64 / total_points as f64) * 100.0
        } else {
            0.0
        };

        stats.insert("total_points".to_string(), Value::from(total_points));
        stats.insert("passed_points".to_string(), Value::from(passed_points));
        stats.insert("failed_points".to_string(), Value::from(failed_points));
        stats.insert("success_rate".to_string(), Value::from(format!("{:.1}", success_rate)));

        stats
    }

    /// 生成PDF文件
    async fn generate_pdf_file(
        &self,
        template_content: &str,
        data: &HashMap<String, Value>,
        output_path: &Path,
    ) -> AppResult<()> {
        // 使用模板引擎渲染HTML
        let mut context = Context::new();
        for (key, value) in data {
            context.insert(key, value);
        }

        // 使用 Tera::one_off 进行一次性模板渲染，避免借用检查器错误
        let _html_content = Tera::one_off(template_content, &context, true)
            .map_err(|e| AppError::template_error(format!("模板渲染失败: {}", e)))?;

        // 创建PDF文档
        let (doc, page1, layer1) = PdfDocument::new("测试报告", Mm(210.0), Mm(297.0), "Layer 1");
        let font = doc.add_builtin_font(BuiltinFont::HelveticaBold)
            .map_err(|e| AppError::pdf_error(format!("添加字体失败: {}", e)))?;

        let current_layer = doc.get_page(page1).get_layer(layer1);

        // 简单的PDF内容生成（实际项目中可能需要更复杂的HTML到PDF转换）
        current_layer.use_text("测试报告", 24.0, Mm(20.0), Mm(270.0), &font);
        
        // 添加基本信息
        if let Some(batch) = data.get("batch") {
            if let Some(product_model) = batch.get("product_model") {
                current_layer.use_text(
                    &format!("产品型号: {}", product_model.as_str().unwrap_or("N/A")),
                    12.0, Mm(20.0), Mm(250.0), &font
                );
            }
        }

        // 保存PDF文件
        doc.save(&mut std::io::BufWriter::new(std::fs::File::create(output_path)?))
            .map_err(|e| AppError::pdf_error(format!("保存PDF文件失败: {}", e)))?;

        info!("PDF报告已生成: {:?}", output_path);
        Ok(())
    }

    /// 生成Excel文件
    async fn generate_excel_file(
        &self,
        data: &HashMap<String, Value>,
        output_path: &Path,
    ) -> AppResult<()> {
        let mut workbook = Workbook::new();

        // 创建摘要工作表
        let mut summary_sheet = workbook.add_worksheet().set_name("测试摘要")?;
        
        // 添加标题格式
        let header_format = Format::new().set_bold().set_background_color("#D3D3D3");
        
        // 写入摘要信息
        summary_sheet.write_string_with_format(0, 0, "项目", &header_format)?;
        summary_sheet.write_string_with_format(0, 1, "值", &header_format)?;

        let mut row = 1;
        if let Some(batch) = data.get("batch") {
            if let Some(product_model) = batch.get("product_model") {
                summary_sheet.write_string(row, 0, "产品型号")?;
                summary_sheet.write_string(row, 1, product_model.as_str().unwrap_or("N/A"))?;
                row += 1;
            }
            if let Some(serial_number) = batch.get("serial_number") {
                summary_sheet.write_string(row, 0, "序列号")?;
                summary_sheet.write_string(row, 1, serial_number.as_str().unwrap_or("N/A"))?;
                row += 1;
            }
        }

        if let Some(statistics) = data.get("statistics") {
            if let Some(total_points) = statistics.get("total_points") {
                summary_sheet.write_string(row, 0, "总测试点数")?;
                summary_sheet.write_number(row, 1, total_points.as_u64().unwrap_or(0) as f64)?;
                row += 1;
            }
            if let Some(passed_points) = statistics.get("passed_points") {
                summary_sheet.write_string(row, 0, "通过点数")?;
                summary_sheet.write_number(row, 1, passed_points.as_u64().unwrap_or(0) as f64)?;
                row += 1;
            }
            if let Some(success_rate) = statistics.get("success_rate") {
                summary_sheet.write_string(row, 0, "成功率")?;
                summary_sheet.write_string(row, 1, success_rate.as_str().unwrap_or("0%"))?;
                row += 1;
            }
        }

        // 创建详细结果工作表
        let mut details_sheet = workbook.add_worksheet().set_name("详细结果")?;
        
        // 写入表头
        let headers = ["位号", "描述", "模块类型", "测试状态", "创建时间"];
        for (col, header) in headers.iter().enumerate() {
            details_sheet.write_string_with_format(0, col as u16, *header, &header_format)?;
        }

        // 写入测试实例数据
        if let Some(instances) = data.get("instances") {
            if let Some(instances_array) = instances.as_array() {
                for (row_idx, instance) in instances_array.iter().enumerate() {
                    let row = (row_idx + 1) as u32;
                    
                    if let Some(definition) = instance.get("definition") {
                        if let Some(tag) = definition.get("tag") {
                            details_sheet.write_string(row, 0, tag.as_str().unwrap_or("N/A"))?;
                        }
                        if let Some(description) = definition.get("variable_description") {
                            details_sheet.write_string(row, 1, description.as_str().unwrap_or("N/A"))?;
                        }
                        if let Some(module_type) = definition.get("module_type") {
                            details_sheet.write_string(row, 2, module_type.as_str().unwrap_or("N/A"))?;
                        }
                    }
                    
                    if let Some(status) = instance.get("overall_status") {
                        details_sheet.write_string(row, 3, status.as_str().unwrap_or("N/A"))?;
                    }
                    
                    if let Some(created_at) = instance.get("creation_time") {
                        details_sheet.write_string(row, 4, created_at.as_str().unwrap_or("N/A"))?;
                    }
                }
            }
        }

        // 保存Excel文件
        workbook.save(output_path)
            .map_err(|e| AppError::excel_error(format!("保存Excel文件失败: {}", e)))?;

        info!("Excel报告已生成: {:?}", output_path);
        Ok(())
    }

    /// 获取模板内容
    async fn get_template_content(&self, template_id: &str) -> AppResult<String> {
        // 从数据库获取模板
        let templates = self.get_templates().await?;
        let template = templates.into_iter()
            .find(|t| t.template_id == template_id)
            .ok_or_else(|| AppError::not_found_error("模板未找到".to_string(), template_id.to_string()))?;

        Ok(template.content)
    }
}

#[async_trait]
impl IReportGenerationService for ReportGenerationService {
    async fn generate_pdf_report(
        &self,
        request: ReportGenerationRequest,
        user_id: &str,
    ) -> AppResult<TestReport> {
        info!("开始生成PDF报告，批次: {:?}", request.batch_ids);

        // 收集报告数据
        let data = self.collect_report_data(&request.batch_ids).await?;

        // 获取模板内容
        let template_content = self.get_template_content(&request.template_id).await?;

        // 生成文件名
        let filename = request.output_filename.unwrap_or_else(|| {
            format!("report_{}_{}.pdf", request.batch_ids.join("_"), Utc::now().format("%Y%m%d_%H%M%S"))
        });

        let output_path = self.reports_dir.join(&filename);

        // 生成PDF文件
        self.generate_pdf_file(&template_content, &data, &output_path).await?;

        // 获取文件大小
        let file_size = fs::metadata(&output_path)
            .map_err(|e| AppError::io_error("获取文件大小失败".to_string(), e.to_string()))?
            .len();

        // 创建报告记录
        let report = TestReport {
            report_id: Uuid::new_v4().to_string(),
            batch_id: request.batch_ids.join(","),
            report_type: ReportType::PDF,
            template_id: request.template_id,
            generated_at: Utc::now(),
            generated_by: user_id.to_string(),
            file_path: output_path.to_string_lossy().to_string(),
            file_size,
            metadata: request.parameters,
            status: ReportStatus::Completed,
        };

        info!("PDF报告生成完成: {}", report.report_id);
        Ok(report)
    }

    async fn generate_excel_report(
        &self,
        request: ReportGenerationRequest,
        user_id: &str,
    ) -> AppResult<TestReport> {
        info!("开始生成Excel报告，批次: {:?}", request.batch_ids);

        // 收集报告数据
        let data = self.collect_report_data(&request.batch_ids).await?;

        // 生成文件名
        let filename = request.output_filename.unwrap_or_else(|| {
            format!("report_{}_{}.xlsx", request.batch_ids.join("_"), Utc::now().format("%Y%m%d_%H%M%S"))
        });

        let output_path = self.reports_dir.join(&filename);

        // 生成Excel文件
        self.generate_excel_file(&data, &output_path).await?;

        // 获取文件大小
        let file_size = fs::metadata(&output_path)
            .map_err(|e| AppError::io_error("获取文件大小失败".to_string(), e.to_string()))?
            .len();

        // 创建报告记录
        let report = TestReport {
            report_id: Uuid::new_v4().to_string(),
            batch_id: request.batch_ids.join(","),
            report_type: ReportType::Excel,
            template_id: request.template_id,
            generated_at: Utc::now(),
            generated_by: user_id.to_string(),
            file_path: output_path.to_string_lossy().to_string(),
            file_size,
            metadata: request.parameters,
            status: ReportStatus::Completed,
        };

        info!("Excel报告生成完成: {}", report.report_id);
        Ok(report)
    }

    async fn get_reports(&self, _batch_id: Option<&str>) -> AppResult<Vec<TestReport>> {
        // TODO: 从数据库获取报告列表
        // 目前返回空列表，实际实现需要数据库支持
        warn!("get_reports 方法尚未完全实现");
        Ok(Vec::new())
    }

    async fn get_templates(&self) -> AppResult<Vec<ReportTemplate>> {
        // TODO: 从数据库获取模板列表
        // 目前返回默认模板
        let default_template = ReportTemplate {
            template_id: "default_pdf".to_string(),
            name: "默认PDF模板".to_string(),
            description: "系统默认的PDF报告模板".to_string(),
            template_type: ReportType::PDF,
            content: "default_pdf".to_string(), // 模板引擎中的模板名
            styles: HashMap::new(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            created_by: "system".to_string(),
            is_default: true,
        };

        Ok(vec![default_template])
    }

    async fn create_template(&self, _template: ReportTemplate) -> AppResult<()> {
        // TODO: 保存模板到数据库
        warn!("create_template 方法尚未完全实现");
        Ok(())
    }

    async fn update_template(&self, _template: ReportTemplate) -> AppResult<()> {
        // TODO: 更新数据库中的模板
        warn!("update_template 方法尚未完全实现");
        Ok(())
    }

    async fn delete_template(&self, _template_id: &str) -> AppResult<()> {
        // TODO: 从数据库删除模板
        warn!("delete_template 方法尚未完全实现");
        Ok(())
    }

    async fn delete_report(&self, _report_id: &str) -> AppResult<()> {
        // TODO: 删除报告文件和数据库记录
        warn!("delete_report 方法尚未完全实现");
        Ok(())
    }
} 