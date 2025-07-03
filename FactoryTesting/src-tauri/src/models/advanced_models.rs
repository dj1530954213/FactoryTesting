/// Phase 4 高级功能数据模型
/// 
/// 包含报告生成、数据分析、配置管理和用户权限管理的数据结构

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

// ============================================================================
// 报告生成相关模型
// ============================================================================

/// 测试报告结构体
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TestReport {
    /// 报告唯一ID
    pub report_id: String,
    /// 关联的批次ID
    pub batch_id: String,
    /// 报告类型
    pub report_type: ReportType,
    /// 使用的模板ID
    pub template_id: String,
    /// 生成时间
    pub generated_at: DateTime<Utc>,
    /// 生成者用户ID
    pub generated_by: String,
    /// 文件路径
    pub file_path: String,
    /// 文件大小（字节）
    pub file_size: u64,
    /// 报告元数据
    pub metadata: HashMap<String, serde_json::Value>,
    /// 报告状态
    pub status: ReportStatus,
}

/// 报告类型枚举
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ReportType {
    /// PDF格式报告
    PDF,
    /// Excel格式报告
    Excel,
    /// HTML格式报告
    HTML,
    /// CSV格式数据导出
    CSV,
}

/// 报告状态枚举
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ReportStatus {
    /// 生成中
    Generating,
    /// 生成完成
    Completed,
    /// 生成失败
    Failed,
    /// 已删除
    Deleted,
}

/// 报告模板结构体
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ReportTemplate {
    /// 模板唯一ID
    pub template_id: String,
    /// 模板名称
    pub name: String,
    /// 模板描述
    pub description: String,
    /// 模板类型
    pub template_type: ReportType,
    /// 模板内容（Tera模板语法）
    pub content: String,
    /// 样式配置
    pub styles: HashMap<String, serde_json::Value>,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 更新时间
    pub updated_at: DateTime<Utc>,
    /// 创建者
    pub created_by: String,
    /// 是否为默认模板
    pub is_default: bool,
}

/// 报告生成请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportGenerationRequest {
    /// 批次ID列表
    pub batch_ids: Vec<String>,
    /// 模板ID
    pub template_id: String,
    /// 报告类型
    pub report_type: ReportType,
    /// 自定义参数
    pub parameters: HashMap<String, serde_json::Value>,
    /// 输出文件名（可选）
    pub output_filename: Option<String>,
}

// ============================================================================
// 数据分析相关模型
// ============================================================================

/// 分析结果结构体
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AnalysisResult {
    /// 分析唯一ID
    pub analysis_id: String,
    /// 分析的批次ID列表
    pub batch_ids: Vec<String>,
    /// 分析类型
    pub analysis_type: AnalysisType,
    /// 分析指标
    pub metrics: HashMap<String, f64>,
    /// 图表数据
    pub charts: Vec<ChartData>,
    /// 分析摘要
    pub summary: String,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 分析者
    pub analyzed_by: String,
}

/// 分析类型枚举
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AnalysisType {
    /// 成功率分析
    SuccessRate,
    /// 性能趋势分析
    PerformanceTrend,
    /// 质量指标分析
    QualityMetrics,
    /// 批次对比分析
    BatchComparison,
    /// 时间序列分析
    TimeSeries,
}

/// 图表数据结构体
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ChartData {
    /// 图表ID
    pub chart_id: String,
    /// 图表类型
    pub chart_type: ChartType,
    /// 图表标题
    pub title: String,
    /// X轴标签
    pub x_label: String,
    /// Y轴标签
    pub y_label: String,
    /// 数据系列
    pub series: Vec<DataSeries>,
    /// 图表配置
    pub config: HashMap<String, serde_json::Value>,
}

/// 图表类型枚举
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ChartType {
    /// 折线图
    Line,
    /// 柱状图
    Bar,
    /// 饼图
    Pie,
    /// 散点图
    Scatter,
    /// 面积图
    Area,
    /// 雷达图
    Radar,
}

/// 数据系列结构体
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DataSeries {
    /// 系列名称
    pub name: String,
    /// 数据点
    pub data: Vec<DataPoint>,
    /// 系列颜色
    pub color: Option<String>,
}

/// 数据点结构体
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DataPoint {
    /// X轴值
    pub x: serde_json::Value,
    /// Y轴值
    pub y: f64,
    /// 标签（可选）
    pub label: Option<String>,
}

/// 统计数据结构体
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Statistics {
    /// 总测试点数
    pub total_points: u32,
    /// 通过点数
    pub passed_points: u32,
    /// 失败点数
    pub failed_points: u32,
    /// 成功率
    pub success_rate: f64,
    /// 平均测试时间（毫秒）
    pub avg_test_time_ms: f64,
    /// 最快测试时间（毫秒）
    pub min_test_time_ms: f64,
    /// 最慢测试时间（毫秒）
    pub max_test_time_ms: f64,
    /// 按模块类型统计
    pub by_module_type: HashMap<String, ModuleStatistics>,
    /// 按时间段统计
    pub by_time_period: Vec<TimeSeriesPoint>,
}

/// 模块统计数据
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ModuleStatistics {
    /// 总点数
    pub total: u32,
    /// 通过数
    pub passed: u32,
    /// 失败数
    pub failed: u32,
    /// 成功率
    pub success_rate: f64,
}

/// 时间序列数据点
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TimeSeriesPoint {
    /// 时间戳
    pub timestamp: DateTime<Utc>,
    /// 值
    pub value: f64,
    /// 标签
    pub label: String,
}

// ============================================================================
// 配置管理相关模型
// ============================================================================

/// 系统配置结构体
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SystemConfig {
    /// 配置唯一ID
    pub config_id: String,
    /// 配置分类
    pub category: ConfigCategory,
    /// 配置键
    pub key: String,
    /// 配置值
    pub value: serde_json::Value,
    /// 配置描述
    pub description: String,
    /// 数据类型
    pub data_type: ConfigDataType,
    /// 是否必需
    pub is_required: bool,
    /// 默认值
    pub default_value: Option<serde_json::Value>,
    /// 验证规则
    pub validation_rules: Option<String>,
    /// 更新时间
    pub updated_at: DateTime<Utc>,
    /// 更新者
    pub updated_by: String,
}

/// 配置分类枚举
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ConfigCategory {
    /// PLC连接配置
    PlcConnection,
    /// 测试参数配置
    TestParameters,
    /// 用户界面配置
    UserInterface,
    /// 系统行为配置
    System,
    /// 报告配置
    Report,
    /// 安全配置
    Security,
}

/// 配置数据类型枚举
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ConfigDataType {
    /// 字符串
    String,
    /// 整数
    Integer,
    /// 浮点数
    Float,
    /// 布尔值
    Boolean,
    /// JSON对象
    Object,
    /// 数组
    Array,
}

/// 用户偏好设置
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UserPreferences {
    /// 用户ID
    pub user_id: String,
    /// 界面主题
    pub theme: String,
    /// 语言设置
    pub language: String,
    /// 时区
    pub timezone: String,
    /// 默认页面大小
    pub default_page_size: u32,
    /// 自动刷新间隔（秒）
    pub auto_refresh_interval: u32,
    /// 通知设置
    pub notifications: NotificationSettings,
    /// 自定义设置
    pub custom_settings: HashMap<String, serde_json::Value>,
    /// 更新时间
    pub updated_at: DateTime<Utc>,
}

/// 通知设置
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NotificationSettings {
    /// 是否启用邮件通知
    pub email_enabled: bool,
    /// 是否启用桌面通知
    pub desktop_enabled: bool,
    /// 测试完成通知
    pub test_completion: bool,
    /// 测试失败通知
    pub test_failure: bool,
    /// 系统错误通知
    pub system_error: bool,
}

// ============================================================================
// 用户权限管理相关模型
// ============================================================================

/// 用户结构体
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct User {
    /// 用户唯一ID
    pub user_id: String,
    /// 用户名
    pub username: String,
    /// 邮箱
    pub email: String,
    /// 密码哈希（不序列化）
    #[serde(skip_serializing)]
    pub password_hash: String,
    /// 用户角色
    pub role: UserRole,
    /// 真实姓名
    pub full_name: Option<String>,
    /// 部门
    pub department: Option<String>,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 最后登录时间
    pub last_login: Option<DateTime<Utc>>,
    /// 是否激活
    pub is_active: bool,
    /// 是否锁定
    pub is_locked: bool,
    /// 登录失败次数
    pub failed_login_attempts: u32,
}

/// 用户角色枚举
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum UserRole {
    /// 系统管理员
    Administrator,
    /// 测试工程师
    TestEngineer,
    /// 操作员
    Operator,
    /// 查看者
    Viewer,
}

/// 认证令牌
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthToken {
    /// 访问令牌
    pub access_token: String,
    /// 刷新令牌
    pub refresh_token: String,
    /// 令牌类型
    pub token_type: String,
    /// 过期时间（秒）
    pub expires_in: u64,
    /// 用户信息
    pub user: User,
}

/// 权限结构体
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Permission {
    /// 权限ID
    pub permission_id: String,
    /// 权限名称
    pub name: String,
    /// 权限描述
    pub description: String,
    /// 资源类型
    pub resource: String,
    /// 操作类型
    pub action: String,
}

/// 角色权限映射
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RolePermission {
    /// 角色
    pub role: UserRole,
    /// 权限列表
    pub permissions: Vec<Permission>,
}

/// 审计日志结构体
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuditLog {
    /// 日志唯一ID
    pub log_id: String,
    /// 用户ID
    pub user_id: String,
    /// 用户名
    pub username: String,
    /// 操作类型
    pub action: String,
    /// 资源类型
    pub resource: String,
    /// 资源ID
    pub resource_id: Option<String>,
    /// 操作结果
    pub result: AuditResult,
    /// 时间戳
    pub timestamp: DateTime<Utc>,
    /// IP地址
    pub ip_address: String,
    /// 用户代理
    pub user_agent: Option<String>,
    /// 操作详情
    pub details: HashMap<String, serde_json::Value>,
}

/// 审计结果枚举
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AuditResult {
    /// 成功
    Success,
    /// 失败
    Failure,
    /// 拒绝访问
    AccessDenied,
}

// ============================================================================
// 实用函数和默认实现
// ============================================================================

impl Default for TestReport {
    fn default() -> Self {
        Self {
            report_id: Uuid::new_v4().to_string(),
            batch_id: String::new(),
            report_type: ReportType::PDF,
            template_id: String::new(),
            generated_at: Utc::now(),
            generated_by: String::new(),
            file_path: String::new(),
            file_size: 0,
            metadata: HashMap::new(),
            status: ReportStatus::Generating,
        }
    }
}

impl Default for ReportTemplate {
    fn default() -> Self {
        Self {
            template_id: Uuid::new_v4().to_string(),
            name: String::new(),
            description: String::new(),
            template_type: ReportType::PDF,
            content: String::new(),
            styles: HashMap::new(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            created_by: String::new(),
            is_default: false,
        }
    }
}

impl Default for AnalysisResult {
    fn default() -> Self {
        Self {
            analysis_id: Uuid::new_v4().to_string(),
            batch_ids: Vec::new(),
            analysis_type: AnalysisType::SuccessRate,
            metrics: HashMap::new(),
            charts: Vec::new(),
            summary: String::new(),
            created_at: Utc::now(),
            analyzed_by: String::new(),
        }
    }
}

impl Default for SystemConfig {
    fn default() -> Self {
        Self {
            config_id: Uuid::new_v4().to_string(),
            category: ConfigCategory::System,
            key: String::new(),
            value: serde_json::Value::Null,
            description: String::new(),
            data_type: ConfigDataType::String,
            is_required: false,
            default_value: None,
            validation_rules: None,
            updated_at: Utc::now(),
            updated_by: String::new(),
        }
    }
}

impl Default for User {
    fn default() -> Self {
        Self {
            user_id: Uuid::new_v4().to_string(),
            username: String::new(),
            email: String::new(),
            password_hash: String::new(),
            role: UserRole::Viewer,
            full_name: None,
            department: None,
            created_at: Utc::now(),
            last_login: None,
            is_active: true,
            is_locked: false,
            failed_login_attempts: 0,
        }
    }
}

impl Default for AuditLog {
    fn default() -> Self {
        Self {
            log_id: Uuid::new_v4().to_string(),
            user_id: String::new(),
            username: String::new(),
            action: String::new(),
            resource: String::new(),
            resource_id: None,
            result: AuditResult::Success,
            timestamp: Utc::now(),
            ip_address: String::new(),
            user_agent: None,
            details: HashMap::new(),
        }
    }
} 
