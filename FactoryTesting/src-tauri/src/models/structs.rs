use std::collections::HashMap;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::enums::{
    ModuleType, OverallTestStatus, PointDataType, SubTestItem, SubTestStatus
};

/// 生成默认UUID字符串的辅助函数
pub fn default_id() -> String {
    Uuid::new_v4().to_string()
}

/// 通道点位定义结构体
/// 描述一个测试点的静态配置信息，通常从Excel或配置文件导入
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ChannelPointDefinition {
    /// 唯一标识符
    #[serde(default = "default_id")]
    pub id: String,
    /// 批次ID - 关联到测试批次
    pub batch_id: Option<String>,
    /// 位号
    pub tag: String,
    /// 变量名（HMI）
    pub variable_name: String,
    /// 变量描述
    pub variable_description: String,

    /// 站名
    pub station_name: String,
    /// 模块名
    pub module_name: String,
    /// 模块类型（AI, DI, etc.）
    pub module_type: ModuleType,
    /// 在模块内的通道号/标签
    #[serde(rename = "channel_number")]
    pub channel_tag_in_module: String,

    /// 数据类型（Bool, Float）
    #[serde(rename = "point_data_type")]
    pub data_type: PointDataType,
    /// 供电类型（例如："有源", "无源"）
    pub power_supply_type: String,
    /// 线制（例如："2线制", "4线制"）
    pub wire_system: String,

    // PLC 相关地址信息
    /// PLC绝对地址（如有）
    pub plc_absolute_address: Option<String>,
    /// PLC通信地址（核心）
    pub plc_communication_address: String,

    // 量程信息（主要用于AI/AO）
    /// 量程下限
    pub range_low_limit: Option<f32>,
    /// 量程上限
    pub range_high_limit: Option<f32>,
    /// 工程单位（例如："mA", "V", "°C"）
    pub engineering_unit: Option<String>,

    // 报警设定点信息（主要用于AI）
    // 低低报
    /// 低低报设定值
    pub sll_set_value: Option<f32>,
    /// 低低报设定值写入地址
    pub sll_set_point_address: Option<String>,
    /// 低低报设定值写入PLC地址
    pub sll_set_point_plc_address: Option<String>,
    /// 低低报设定值写入通讯地址
    pub sll_set_point_communication_address: Option<String>,
    /// 低低报状态读取地址
    pub sll_feedback_address: Option<String>,
    /// 低低报状态读取PLC地址
    pub sll_feedback_plc_address: Option<String>,
    /// 低低报状态读取通讯地址
    pub sll_feedback_communication_address: Option<String>,

    // 低报
    /// 低报设定值
    pub sl_set_value: Option<f32>,
    /// 低报设定值写入地址
    pub sl_set_point_address: Option<String>,
    /// 低报设定值写入PLC地址
    pub sl_set_point_plc_address: Option<String>,
    /// 低报设定值写入通讯地址
    pub sl_set_point_communication_address: Option<String>,
    /// 低报状态读取地址
    pub sl_feedback_address: Option<String>,
    /// 低报状态读取PLC地址
    pub sl_feedback_plc_address: Option<String>,
    /// 低报状态读取通讯地址
    pub sl_feedback_communication_address: Option<String>,

    // 高报
    /// 高报设定值
    pub sh_set_value: Option<f32>,
    /// 高报设定值写入地址
    pub sh_set_point_address: Option<String>,
    /// 高报设定值写入PLC地址
    pub sh_set_point_plc_address: Option<String>,
    /// 高报设定值写入通讯地址
    pub sh_set_point_communication_address: Option<String>,
    /// 高报状态读取地址
    pub sh_feedback_address: Option<String>,
    /// 高报状态读取PLC地址
    pub sh_feedback_plc_address: Option<String>,
    /// 高报状态读取通讯地址
    pub sh_feedback_communication_address: Option<String>,

    // 高高报
    /// 高高报设定值
    pub shh_set_value: Option<f32>,
    /// 高高报设定值写入地址
    pub shh_set_point_address: Option<String>,
    /// 高高报设定值写入PLC地址
    pub shh_set_point_plc_address: Option<String>,
    /// 高高报设定值写入通讯地址
    pub shh_set_point_communication_address: Option<String>,
    /// 高高报状态读取地址
    pub shh_feedback_address: Option<String>,
    /// 高高报状态读取PLC地址
    pub shh_feedback_plc_address: Option<String>,
    /// 高高报状态读取通讯地址
    pub shh_feedback_communication_address: Option<String>,

    // 维护模式相关（主要用于AI）
    /// 维护值设定点地址
    pub maintenance_value_set_point_address: Option<String>,
    /// 维护值设定点PLC地址
    pub maintenance_value_set_point_plc_address: Option<String>,
    /// 维护值设定点通讯地址
    pub maintenance_value_set_point_communication_address: Option<String>,
    /// 维护使能开关点地址
    pub maintenance_enable_switch_point_address: Option<String>,
    /// 维护使能开关点PLC地址
    pub maintenance_enable_switch_point_plc_address: Option<String>,
    /// 维护使能开关点通讯地址
    pub maintenance_enable_switch_point_communication_address: Option<String>,

    // 其他配置信息
    /// 读写属性
    pub access_property: Option<String>,
    /// 是否保存历史
    pub save_history: Option<bool>,
    /// 是否掉电保护
    pub power_failure_protection: Option<bool>,

    // 测试台架（硬接线）相关配置
    /// 测试台架上对应的PLC地址（如果与被测PLC地址不同）
    pub test_rig_plc_address: Option<String>,
}

impl ChannelPointDefinition {
    /// 创建新的通道点位定义
    pub fn new(
        tag: String,
        variable_name: String,
        variable_description: String,
        station_name: String,
        module_name: String,
        module_type: ModuleType,
        channel_tag_in_module: String,
        data_type: PointDataType,
        plc_communication_address: String,
    ) -> Self {
        Self {
            id: default_id(),
            batch_id: None,
            tag,
            variable_name,
            variable_description,
            station_name,
            module_name,
            module_type,
            channel_tag_in_module,
            data_type,
            power_supply_type: String::from("有源"),
            wire_system: String::from("4线制"),
            plc_absolute_address: None,
            plc_communication_address,
            range_low_limit: None,
            range_high_limit: None,
            engineering_unit: None,
            sll_set_value: None,
            sll_set_point_address: None,
            sll_set_point_plc_address: None,
            sll_set_point_communication_address: None,
            sll_feedback_address: None,
            sll_feedback_plc_address: None,
            sll_feedback_communication_address: None,
            sl_set_value: None,
            sl_set_point_address: None,
            sl_set_point_plc_address: None,
            sl_set_point_communication_address: None,
            sl_feedback_address: None,
            sl_feedback_plc_address: None,
            sl_feedback_communication_address: None,
            sh_set_value: None,
            sh_set_point_address: None,
            sh_set_point_plc_address: None,
            sh_set_point_communication_address: None,
            sh_feedback_address: None,
            sh_feedback_plc_address: None,
            sh_feedback_communication_address: None,
            shh_set_value: None,
            shh_set_point_address: None,
            shh_set_point_plc_address: None,
            shh_set_point_communication_address: None,
            shh_feedback_address: None,
            shh_feedback_plc_address: None,
            shh_feedback_communication_address: None,
            maintenance_value_set_point_address: None,
            maintenance_value_set_point_plc_address: None,
            maintenance_value_set_point_communication_address: None,
            maintenance_enable_switch_point_address: None,
            maintenance_enable_switch_point_plc_address: None,
            maintenance_enable_switch_point_communication_address: None,
            access_property: None,
            save_history: None,
            power_failure_protection: None,
            test_rig_plc_address: None,
        }
    }

    /// 创建新的通道点位定义（可指定供电类型）
    pub fn new_with_power_type(
        tag: String,
        variable_name: String,
        variable_description: String,
        station_name: String,
        module_name: String,
        module_type: ModuleType,
        channel_tag_in_module: String,
        data_type: PointDataType,
        plc_communication_address: String,
        power_supply_type: String,
    ) -> Self {
        Self {
            id: default_id(),
            batch_id: None,
            tag,
            variable_name,
            variable_description,
            station_name,
            module_name,
            module_type,
            channel_tag_in_module,
            data_type,
            power_supply_type,
            wire_system: String::from("4线制"),
            plc_absolute_address: None,
            plc_communication_address,
            range_low_limit: None,
            range_high_limit: None,
            engineering_unit: None,
            sll_set_value: None,
            sll_set_point_address: None,
            sll_set_point_plc_address: None,
            sll_set_point_communication_address: None,
            sll_feedback_address: None,
            sll_feedback_plc_address: None,
            sll_feedback_communication_address: None,
            sl_set_value: None,
            sl_set_point_address: None,
            sl_set_point_plc_address: None,
            sl_set_point_communication_address: None,
            sl_feedback_address: None,
            sl_feedback_plc_address: None,
            sl_feedback_communication_address: None,
            sh_set_value: None,
            sh_set_point_address: None,
            sh_set_point_plc_address: None,
            sh_set_point_communication_address: None,
            sh_feedback_address: None,
            sh_feedback_plc_address: None,
            sh_feedback_communication_address: None,
            shh_set_value: None,
            shh_set_point_address: None,
            shh_set_point_plc_address: None,
            shh_set_point_communication_address: None,
            shh_feedback_address: None,
            shh_feedback_plc_address: None,
            shh_feedback_communication_address: None,
            maintenance_value_set_point_address: None,
            maintenance_value_set_point_plc_address: None,
            maintenance_value_set_point_communication_address: None,
            maintenance_enable_switch_point_address: None,
            maintenance_enable_switch_point_plc_address: None,
            maintenance_enable_switch_point_communication_address: None,
            access_property: None,
            save_history: None,
            power_failure_protection: None,
            test_rig_plc_address: None,
        }
    }
}

impl Default for ChannelPointDefinition {
    fn default() -> Self {
        Self {
            id: default_id(),
            batch_id: None,
            tag: String::new(),
            variable_name: String::new(),
            variable_description: String::new(),
            station_name: String::new(),
            module_name: String::new(),
            module_type: ModuleType::default(),
            channel_tag_in_module: String::new(),
            data_type: PointDataType::default(),
            power_supply_type: String::from("有源"),
            wire_system: String::from("4线制"),
            plc_absolute_address: None,
            plc_communication_address: String::new(),
            range_low_limit: None,
            range_high_limit: None,
            engineering_unit: None,
            sll_set_value: None,
            sll_set_point_address: None,
            sll_set_point_plc_address: None,
            sll_set_point_communication_address: None,
            sll_feedback_address: None,
            sll_feedback_plc_address: None,
            sll_feedback_communication_address: None,
            sl_set_value: None,
            sl_set_point_address: None,
            sl_set_point_plc_address: None,
            sl_set_point_communication_address: None,
            sl_feedback_address: None,
            sl_feedback_plc_address: None,
            sl_feedback_communication_address: None,
            sh_set_value: None,
            sh_set_point_address: None,
            sh_set_point_plc_address: None,
            sh_set_point_communication_address: None,
            sh_feedback_address: None,
            sh_feedback_plc_address: None,
            sh_feedback_communication_address: None,
            shh_set_value: None,
            shh_set_point_address: None,
            shh_set_point_plc_address: None,
            shh_set_point_communication_address: None,
            shh_feedback_address: None,
            shh_feedback_plc_address: None,
            shh_feedback_communication_address: None,
            maintenance_value_set_point_address: None,
            maintenance_value_set_point_plc_address: None,
            maintenance_value_set_point_communication_address: None,
            maintenance_enable_switch_point_address: None,
            maintenance_enable_switch_point_plc_address: None,
            maintenance_enable_switch_point_communication_address: None,
            access_property: None,
            save_history: None,
            power_failure_protection: None,
            test_rig_plc_address: None,
        }
    }
}

/// 子测试执行结果结构体
/// 表示单个子测试项的执行结果
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SubTestExecutionResult {
    /// 测试状态
    pub status: SubTestStatus,
    /// 详细信息或错误消息
    pub details: Option<String>,
    /// 期望值
    pub expected_value: Option<String>,
    /// 实际值
    pub actual_value: Option<String>,
    /// 时间戳
    pub timestamp: DateTime<Utc>,
}

impl SubTestExecutionResult {
    /// 创建新的子测试执行结果
    pub fn new(status: SubTestStatus, details: Option<String>, expected: Option<String>, actual: Option<String>) -> Self {
        Self {
            status,
            details,
            expected_value: expected,
            actual_value: actual,
            timestamp: Utc::now(),
        }
    }
}

impl Default for SubTestExecutionResult {
    fn default() -> Self {
        Self {
            status: SubTestStatus::NotTested,
            details: None,
            expected_value: None,
            actual_value: None,
            timestamp: Utc::now(),
        }
    }
}

/// 模拟量读数点结构体
/// 用于AI/AO测试中记录多点测试数据
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct AnalogReadingPoint {
    /// 设定百分比（例如：0.0, 0.25, 0.5, 0.75, 1.0）
    pub set_percentage: f32,
    /// 对应的工程单位设定值
    pub set_value_eng: f32,
    /// 期望的PLC原始读值（如适用）
    pub expected_reading_raw: Option<f32>,
    /// 实际的PLC原始读值
    pub actual_reading_raw: Option<f32>,
    /// 转换后的工程单位读值
    pub actual_reading_eng: Option<f32>,
    /// 该点的测试状态
    pub status: SubTestStatus,
    /// 误差百分比
    pub error_percentage: Option<f32>,
}

/// 数字量测试步骤结构体
/// 用于DI/DO测试中记录测试步骤的详细数据
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct DigitalTestStep {
    /// 步骤编号（1, 2, 3等）
    pub step_number: u32,
    /// 步骤描述
    pub step_description: String,
    /// 设定值（true/false）
    pub set_value: bool,
    /// 期望读取值（true/false）
    pub expected_reading: bool,
    /// 实际读取值（true/false）
    pub actual_reading: bool,
    /// 该步骤的测试状态
    pub status: SubTestStatus,
    /// 时间戳
    pub timestamp: DateTime<Utc>,
}

/// 通道测试实例结构体
/// 代表一个ChannelPointDefinition在某次特定测试执行中的实例
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ChannelTestInstance {
    /// 唯一测试实例ID
    #[serde(default = "default_id")]
    pub instance_id: String,
    /// 关联的ChannelPointDefinition ID
    pub definition_id: String,
    /// 所属测试批次ID
    pub test_batch_id: String,
    /// 测试批次名称（如"批次1"）
    pub test_batch_name: String,

    /// 运行时状态（由ChannelStateManager管理）
    pub overall_status: OverallTestStatus,
    /// 当前正在执行的测试步骤描述
    pub current_step_details: Option<String>,
    /// 最近的错误信息
    pub error_message: Option<String>,

    /// 时间信息
    pub creation_time: DateTime<Utc>,
    pub start_time: Option<DateTime<Utc>>,
    pub last_updated_time: DateTime<Utc>,
    pub final_test_time: Option<DateTime<Utc>>,
    pub total_test_duration_ms: Option<i64>,

    /// 各子测试项的状态和结果
    #[serde(default)]
    pub sub_test_results: HashMap<SubTestItem, SubTestExecutionResult>,

    /// 硬接线测试中的特定数据（模拟量测试）
    pub hardpoint_readings: Option<Vec<AnalogReadingPoint>>,

    /// 数字量测试步骤数据（数字量测试）
    pub digital_test_steps: Option<Vec<DigitalTestStep>>,

    /// 手动测试时的临时输入值
    pub manual_test_current_value_input: Option<String>,
    /// 手动测试时的临时输出值
    pub manual_test_current_value_output: Option<String>,

    /// 分配的测试PLC通道标签（如"AO1_1"）
    pub test_plc_channel_tag: Option<String>,
    /// 分配的测试PLC通信地址（如"AO1.1"）
    pub test_plc_communication_address: Option<String>,

    /// 当前操作员
    pub current_operator: Option<String>,
    /// 重测次数
    pub retries_count: u32,

    /// 运行时临时数据，不一定持久化
    #[serde(default)]
    pub transient_data: HashMap<String, serde_json::Value>,
}

impl ChannelTestInstance {
    /// 创建新的通道测试实例
    pub fn new(definition_id: String, test_batch_id: String) -> Self {
        let now = Utc::now();
        Self {
            instance_id: default_id(),
            definition_id,
            test_batch_id,
            test_batch_name: String::new(),
            overall_status: OverallTestStatus::NotTested,
            current_step_details: None,
            error_message: None,
            creation_time: now,
            start_time: None,
            last_updated_time: now,
            final_test_time: None,
            total_test_duration_ms: None,
            sub_test_results: HashMap::new(),
            hardpoint_readings: None,
            digital_test_steps: None,
            manual_test_current_value_input: None,
            manual_test_current_value_output: None,
            test_plc_channel_tag: None,
            test_plc_communication_address: None,
            current_operator: None,
            retries_count: 0,
            transient_data: HashMap::new(),
        }
    }
}

/// 测试批次信息结构体
/// 包含一个测试批次的基本信息和统计数据
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TestBatchInfo {
    /// 唯一批次ID
    #[serde(default = "default_id")]
    pub batch_id: String,
    /// 产品型号
    pub product_model: Option<String>,
    /// 序列号
    pub serial_number: Option<String>,
    /// 客户名称
    pub customer_name: Option<String>,
    /// 站场名称
    pub station_name: Option<String>,
    /// 创建时间
    pub creation_time: DateTime<Utc>,
    pub last_updated_time: DateTime<Utc>,
    /// 操作员姓名
    pub operator_name: Option<String>,
    /// 状态摘要（例如："进行中", "已完成 - 50/52 通过"）
    pub status_summary: Option<String>,
    /// 统计信息
    pub total_points: u32,
    pub tested_points: u32,
    pub passed_points: u32,
    pub failed_points: u32,
    pub skipped_points: u32,
    /// 批次整体状态
    pub overall_status: OverallTestStatus,
    /// 批次名称（如"批次1"）
    pub batch_name: String,
    /// 其他批次相关信息
    #[serde(default)]
    pub custom_data: HashMap<String, String>,
}

impl TestBatchInfo {
    /// 创建新的测试批次信息
    pub fn new(product_model: Option<String>, serial_number: Option<String>) -> Self {
        let now = Utc::now();
        Self {
            batch_id: default_id(),
            product_model,
            serial_number,
            customer_name: None,
            station_name: None,
            creation_time: now,
            last_updated_time: now,
            operator_name: None,
            status_summary: None,
            total_points: 0,
            tested_points: 0,
            passed_points: 0,
            failed_points: 0,
            skipped_points: 0,
            overall_status: OverallTestStatus::NotTested,
            batch_name: String::new(),
            custom_data: HashMap::new(),
        }
    }
}

/// 原始测试结果结构体
/// 由SpecificTestStepExecutor执行后返回的原始数据，供ChannelStateManager处理
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RawTestOutcome {
    /// 通道实例ID
    pub channel_instance_id: String,
    /// 子测试项
    pub sub_test_item: SubTestItem,
    /// 操作是否成功
    pub success: bool,
    /// 从PLC读取的原始值
    pub raw_value_read: Option<String>,
    /// 计算得到的工程单位值
    pub eng_value_calculated: Option<String>,
    /// 附加消息或错误细节
    pub message: Option<String>,
    /// 开始时间
    pub start_time: DateTime<Utc>,
    /// 结束时间
    pub end_time: DateTime<Utc>,
    /// 一系列读数，如AI多点测试
    pub readings: Option<Vec<AnalogReadingPoint>>,
    /// 数字量测试步骤，如DI/DO测试
    pub digital_steps: Option<Vec<DigitalTestStep>>,
    /// 百分比测试结果 - 存储实际工程量
    pub test_result_0_percent: Option<f64>,
    pub test_result_25_percent: Option<f64>,
    pub test_result_50_percent: Option<f64>,
    pub test_result_75_percent: Option<f64>,
    pub test_result_100_percent: Option<f64>,
    /// 更多细节
    #[serde(default)]
    pub details: HashMap<String, serde_json::Value>,
}

impl RawTestOutcome {
    /// 创建新的原始测试结果
    pub fn new(
        channel_instance_id: String,
        sub_test_item: SubTestItem,
        success: bool,
    ) -> Self {
        let now = Utc::now();
        Self {
            channel_instance_id,
            sub_test_item,
            success,
            raw_value_read: None,
            eng_value_calculated: None,
            message: None,
            start_time: now,
            end_time: now,
            readings: None,
            digital_steps: None,
            test_result_0_percent: None,
            test_result_25_percent: None,
            test_result_50_percent: None,
            test_result_75_percent: None,
            test_result_100_percent: None,
            details: HashMap::new(),
        }
    }

    /// 创建成功的测试结果
    pub fn success(
        channel_instance_id: String,
        sub_test_item: SubTestItem,
    ) -> Self {
        Self::new(channel_instance_id, sub_test_item, true)
    }

    /// 创建失败的测试结果
    pub fn failure(
        channel_instance_id: String,
        sub_test_item: SubTestItem,
        message: String,
    ) -> Self {
        let mut outcome = Self::new(channel_instance_id, sub_test_item, false);
        outcome.message = Some(message);
        outcome
    }
}

impl Default for RawTestOutcome {
    fn default() -> Self {
        Self {
            channel_instance_id: default_id(),
            sub_test_item: SubTestItem::HardPoint,
            success: false,
            raw_value_read: None,
            eng_value_calculated: None,
            message: None,
            start_time: Utc::now(),
            end_time: Utc::now(),
            readings: None,
            digital_steps: None,
            test_result_0_percent: None,
            test_result_25_percent: None,
            test_result_50_percent: None,
            test_result_75_percent: None,
            test_result_100_percent: None,
            details: HashMap::new(),
        }
    }
}

/// 应用配置结构体
/// 用于存储用户界面设置、默认参数、PLC连接配置等
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AppSettings {
    #[serde(default = "default_id")]
    pub id: String, // 通常只有一个实例，ID可以是固定的如 "default_settings"
    pub theme: String,
    pub plc_ip_address: Option<String>,
    pub plc_port: Option<u16>,
    pub default_operator_name: Option<String>,
    pub auto_save_interval_minutes: Option<u32>,
    #[serde(default)]
    pub recent_projects: Vec<String>,
    #[serde(default = "default_last_backup_time")]
    pub last_backup_time: Option<DateTime<Utc>>,
    // 其他配置项...
}

fn default_last_backup_time() -> Option<DateTime<Utc>> {
    None
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            id: "default_settings".to_string(),
            theme: "light".to_string(),
            plc_ip_address: Some("127.0.0.1".to_string()),
            plc_port: Some(502),
            default_operator_name: None,
            auto_save_interval_minutes: Some(5),
            recent_projects: Vec::new(),
            last_backup_time: None,
        }
    }
}

// 重新导出手动测试相关结构体
pub mod manual_test;
pub use manual_test::*;

