use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};

use crate::models::enums::{ModuleType, OverallTestStatus, SubTestItem, SubTestStatus};
use crate::models::structs::ChannelTestInstance;

/// 手动测试子项状态枚举
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ManualTestSubItemStatus {
    NotTested,
    Testing,
    Passed,
    Failed,
    Skipped,
}

/// 手动测试子项类型
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ManualTestSubItem {
    // 通用测试项
    ShowValueCheck,        // 显示值核对
    
    // AI点位专用测试项
    LowLowAlarmTest,      // 低低报警测试
    LowAlarmTest,         // 低报警测试
    HighAlarmTest,        // 高报警测试
    HighHighAlarmTest,    // 高高报警测试
    
    // AI/AO点位通用测试项
    MaintenanceFunction,  // 维护功能测试
}

/// 手动测试子项结果
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ManualTestSubItemResult {
    pub sub_item: ManualTestSubItem,
    pub status: ManualTestSubItemStatus,
    pub test_time: Option<DateTime<Utc>>,
    pub operator_notes: Option<String>,
    pub skip_reason: Option<String>,
}

/// 手动测试状态
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ManualTestStatus {
    pub instance_id: String,
    pub overall_status: OverallTestStatus,
    pub sub_item_results: HashMap<ManualTestSubItem, ManualTestSubItemResult>,
    pub start_time: Option<DateTime<Utc>>,
    pub completion_time: Option<DateTime<Utc>>,
    pub current_operator: Option<String>,
}

/// PLC监控数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlcMonitoringData {
    pub instance_id: String,
    pub timestamp: DateTime<Utc>,
    pub values: HashMap<String, serde_json::Value>,
}

/// 手动测试请求
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StartManualTestRequest {
    #[serde(rename = "instanceId")]
    pub instance_id: String,
    #[serde(rename = "moduleType")]
    pub module_type: ModuleType,
    #[serde(rename = "operatorName")]
    pub operator_name: Option<String>,
}

/// 手动测试响应
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StartManualTestResponse {
    pub success: bool,
    pub message: Option<String>,
    pub test_status: Option<ManualTestStatus>,
}

/// 更新手动测试子项请求
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateManualTestSubItemRequest {
    #[serde(rename = "instanceId")]
    pub instance_id: String,
    #[serde(rename = "subItem")]
    pub sub_item: ManualTestSubItem,
    pub status: ManualTestSubItemStatus,
    #[serde(rename = "operatorNotes")]
    pub operator_notes: Option<String>,
    #[serde(rename = "skipReason")]
    pub skip_reason: Option<String>,
}

/// 更新手动测试子项响应
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateManualTestSubItemResponse {
    pub success: bool,
    pub message: Option<String>,
    pub test_status: Option<ManualTestStatus>,
    pub is_completed: Option<bool>,
}

/// PLC监控请求
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StartPlcMonitoringRequest {
    #[serde(rename = "instanceId")]
    pub instance_id: String,
    #[serde(rename = "moduleType")]
    pub module_type: ModuleType,
    #[serde(rename = "monitoringAddresses")]
    pub monitoring_addresses: Vec<String>,
    /// 地址到值键名的映射，可选
    #[serde(rename = "addressKeyMap", skip_serializing_if = "Option::is_none")]
    pub address_key_map: Option<std::collections::HashMap<String, String>>,
    /// 本次监控使用的 PLC 连接 ID（区分 target_plc / manual_test_plc）
    #[serde(rename = "connectionId", skip_serializing_if = "Option::is_none", default)]
    pub connection_id: Option<String>,
}

/// PLC监控响应
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StartPlcMonitoringResponse {
    pub success: bool,
    pub message: Option<String>,
    pub monitoring_id: Option<String>,
}

/// 停止PLC监控请求
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StopPlcMonitoringRequest {
    #[serde(rename = "instanceId")]
    pub instance_id: String,
    #[serde(rename = "monitoringId")]
    pub monitoring_id: Option<String>,
}

/// 手动测试配置
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ManualTestConfig {
    pub module_type: ModuleType,
    pub applicable_sub_items: Vec<ManualTestSubItem>,
    pub plc_monitoring_required: bool,
    pub monitoring_interval: u64, // 毫秒
}

impl ManualTestConfig {
    /// 获取模块类型对应的手动测试配置
    pub fn for_module_type(module_type: ModuleType) -> Self {
        match module_type {
            ModuleType::AI => Self {
                module_type: ModuleType::AI,
                applicable_sub_items: vec![
                    ManualTestSubItem::ShowValueCheck,
                    ManualTestSubItem::LowLowAlarmTest,
                    ManualTestSubItem::LowAlarmTest,
                    ManualTestSubItem::HighAlarmTest,
                    ManualTestSubItem::HighHighAlarmTest,
                    ManualTestSubItem::MaintenanceFunction,
                ],
                plc_monitoring_required: true,
                monitoring_interval: 500,
            },
            ModuleType::AO => Self {
                module_type: ModuleType::AO,
                applicable_sub_items: vec![
                    ManualTestSubItem::ShowValueCheck,
                ],
                plc_monitoring_required: true,
                monitoring_interval: 500,
            },
            ModuleType::DI | ModuleType::DO => Self {
                module_type,
                applicable_sub_items: vec![
                    ManualTestSubItem::ShowValueCheck,
                ],
                plc_monitoring_required: true,
                monitoring_interval: 500,
            },
            // 无源模块类型
            ModuleType::AINone => Self {
                module_type: ModuleType::AINone,
                applicable_sub_items: vec![
                    ManualTestSubItem::ShowValueCheck,
                ],
                plc_monitoring_required: true,
                monitoring_interval: 500,
            },
            ModuleType::AONone => Self {
                module_type: ModuleType::AONone,
                applicable_sub_items: vec![
                    ManualTestSubItem::ShowValueCheck,
                ],
                plc_monitoring_required: true,
                monitoring_interval: 500,
            },
            ModuleType::DINone | ModuleType::DONone => Self {
                module_type,
                applicable_sub_items: vec![
                    ManualTestSubItem::ShowValueCheck,
                ],
                plc_monitoring_required: true,
                monitoring_interval: 500,
            },
            ModuleType::Communication => Self {
                module_type: ModuleType::Communication,
                applicable_sub_items: vec![], // 通信模块暂不支持手动测试
                plc_monitoring_required: false,
                monitoring_interval: 1000,
            },
            ModuleType::Other(_) => Self {
                module_type,
                applicable_sub_items: vec![], // 其他类型模块暂不支持手动测试
                plc_monitoring_required: false,
                monitoring_interval: 1000,
            },
        }
    }
}

impl ManualTestStatus {
    /// 创建新的手动测试状态
    pub fn new(instance_id: String, module_type: ModuleType, operator: Option<String>) -> Self {
        let config = ManualTestConfig::for_module_type(module_type);
        let mut sub_item_results = HashMap::new();
        
        // 初始化所有适用的子项为未测试状态
        for sub_item in config.applicable_sub_items {
            sub_item_results.insert(sub_item.clone(), ManualTestSubItemResult {
                sub_item,
                status: ManualTestSubItemStatus::NotTested,
                test_time: None,
                operator_notes: None,
                skip_reason: None,
            });
        }
        
        Self {
            instance_id,
            overall_status: OverallTestStatus::ManualTesting,
            sub_item_results,
            start_time: Some(Utc::now()),
            completion_time: None,
            current_operator: operator,
        }
    }
    
    /// 检查是否所有子项都已完成
    pub fn is_all_completed(&self) -> bool {
        self.sub_item_results.values().all(|result| {
            matches!(result.status, ManualTestSubItemStatus::Passed | ManualTestSubItemStatus::Skipped)
        })
    }
    
    /// 获取已完成的子项数量
    pub fn get_completed_count(&self) -> usize {
        self.sub_item_results.values()
            .filter(|result| matches!(result.status, ManualTestSubItemStatus::Passed | ManualTestSubItemStatus::Skipped))
            .count()
    }
    
    /// 获取总子项数量
    pub fn get_total_count(&self) -> usize {
        self.sub_item_results.len()
    }
    
    /// 更新子项状态
    pub fn update_sub_item(&mut self, sub_item: ManualTestSubItem, status: ManualTestSubItemStatus, operator_notes: Option<String>, skip_reason: Option<String>) -> bool {
        if let Some(result) = self.sub_item_results.get_mut(&sub_item) {
            result.status = status;
            result.test_time = Some(Utc::now());
            result.operator_notes = operator_notes;
            result.skip_reason = skip_reason;
            
            // 检查是否所有子项都已完成
            if self.is_all_completed() {
                self.overall_status = OverallTestStatus::TestCompletedPassed;
                self.completion_time = Some(Utc::now());
            }
            
            true
        } else {
            false
        }
    }

    /// 根据 ChannelTestInstance 构造手动测试状态（用于前端刷新）
    pub fn from_instance(instance: &ChannelTestInstance) -> Self {
        // 暂时无法直接确定模块类型，此处简单推断：若数字量子测试步骤存在则认为 DI，否则 AI
        let inferred_module_type = if let Some(digital_steps) = &instance.digital_test_steps {
            if !digital_steps.is_empty() { ModuleType::DI } else { ModuleType::AI }
        } else {
            ModuleType::AI
        };

        let config = ManualTestConfig::for_module_type(inferred_module_type.clone());

        // 初始化所有子项状态为 NotTested
        let mut sub_item_results: HashMap<ManualTestSubItem, ManualTestSubItemResult> = config
            .applicable_sub_items
            .iter()
            .map(|it| {
                (
                    it.clone(),
                    ManualTestSubItemResult {
                        sub_item: it.clone(),
                        status: ManualTestSubItemStatus::NotTested,
                        test_time: None,
                        operator_notes: None,
                        skip_reason: None,
                    },
                )
            })
            .collect();

        // 将 instance.sub_test_results 映射过来
        for (sub_item, exec_result) in &instance.sub_test_results {
            // 跳过硬点测试结果，避免其 PASS 状态导致显示值核对被误标记已完成
            if *sub_item == SubTestItem::HardPoint {
                continue;
            }

            let manual_item: ManualTestSubItem = sub_item.clone().into();
            if let Some(result_slot) = sub_item_results.get_mut(&manual_item) {
                result_slot.status = match exec_result.status {
                    SubTestStatus::NotTested => ManualTestSubItemStatus::NotTested,
                    SubTestStatus::Testing => ManualTestSubItemStatus::Testing,
                    SubTestStatus::Passed => ManualTestSubItemStatus::Passed,
                    SubTestStatus::Failed => ManualTestSubItemStatus::Failed,
                    SubTestStatus::Skipped => ManualTestSubItemStatus::Skipped,
                    SubTestStatus::NotApplicable => ManualTestSubItemStatus::NotTested,
                };
                result_slot.test_time = Some(exec_result.timestamp);
                result_slot.operator_notes = exec_result.details.clone();
            }
        }

        // 计算 overall_status：若全部完成则保持实例状态，否则 ManualTesting
        let overall_status = if sub_item_results.values().all(|r| matches!(r.status, ManualTestSubItemStatus::Passed | ManualTestSubItemStatus::Skipped)) {
            OverallTestStatus::TestCompletedPassed
        } else {
            OverallTestStatus::ManualTesting
        };

        Self {
            instance_id: instance.instance_id.clone(),
            overall_status,
            sub_item_results,
            start_time: instance.start_time,
            completion_time: instance.final_test_time,
            current_operator: instance.current_operator.clone(),
        }
    }
}

// ======================= ManualTestSubItem <-> SubTestItem =======================

impl From<ManualTestSubItem> for SubTestItem {
    fn from(item: ManualTestSubItem) -> Self {
        match item {
            ManualTestSubItem::ShowValueCheck => SubTestItem::StateDisplay,
            ManualTestSubItem::LowLowAlarmTest => SubTestItem::LowLowAlarm,
            ManualTestSubItem::LowAlarmTest => SubTestItem::LowAlarm,
            ManualTestSubItem::HighAlarmTest => SubTestItem::HighAlarm,
            ManualTestSubItem::HighHighAlarmTest => SubTestItem::HighHighAlarm,
            ManualTestSubItem::MaintenanceFunction => SubTestItem::Maintenance,
        }
    }
}

impl From<SubTestItem> for ManualTestSubItem {
    fn from(item: SubTestItem) -> Self {
        match item {
            SubTestItem::StateDisplay => ManualTestSubItem::ShowValueCheck,
            SubTestItem::LowLowAlarm => ManualTestSubItem::LowLowAlarmTest,
            SubTestItem::LowAlarm => ManualTestSubItem::LowAlarmTest,
            SubTestItem::HighAlarm => ManualTestSubItem::HighAlarmTest,
            SubTestItem::HighHighAlarm => ManualTestSubItem::HighHighAlarmTest,
            SubTestItem::Maintenance | SubTestItem::MaintenanceFunction => ManualTestSubItem::MaintenanceFunction,
            _ => ManualTestSubItem::ShowValueCheck, // 兜底
        }
    }
}
