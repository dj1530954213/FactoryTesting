/// 通道状态管理器
/// 
/// 这是整个系统中唯一负责修改 ChannelTestInstance 核心状态的地方。
/// 根据业务规则和 RawTestOutcome 更新状态，不包含I/O或UI逻辑。
/// 符合 FAT-CSM-001, FAT-CSM-002 规则。

use crate::models::{
    ChannelTestInstance, ChannelPointDefinition, RawTestOutcome, SubTestItem, 
    OverallTestStatus, SubTestStatus, SubTestExecutionResult, AnalogReadingPoint,
    ModuleType
};
use crate::utils::error::{AppError, AppResult};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use log::{debug, warn, error};

/// 通道状态管理器接口
/// 
/// 定义了所有状态管理相关的核心方法
#[async_trait]
pub trait IChannelStateManager: Send + Sync {
    /// 基于点位定义和批次信息，初始化一个新的 ChannelTestInstance
    async fn initialize_channel_test_instance(
        &self,
        definition: &ChannelPointDefinition,
        batch_id: &str,
        instance_id: String,
    ) -> AppResult<ChannelTestInstance>;

    /// 应用原始测试结果来更新 ChannelTestInstance 的状态
    /// 这是核心的状态转换入口
    async fn apply_raw_outcome(
        &self,
        instance: &mut ChannelTestInstance,
        outcome: RawTestOutcome,
    ) -> AppResult<()>;

    /// 将 ChannelTestInstance 标记为"已跳过"
    async fn mark_as_skipped(
        &self,
        instance: &mut ChannelTestInstance,
        reason: String,
        skip_time: DateTime<Utc>,
    ) -> AppResult<()>;

    /// 为接线确认准备状态
    async fn prepare_for_wiring_confirmation(
        &self,
        instance: &mut ChannelTestInstance,
        confirm_time: DateTime<Utc>,
    ) -> AppResult<()>;

    /// 标记硬点测试开始
    async fn begin_hard_point_test(
        &self,
        instance: &mut ChannelTestInstance,
        start_time: DateTime<Utc>,
    ) -> AppResult<()>;

    /// 标记手动测试某个子项开始
    async fn begin_manual_sub_test(
        &self,
        instance: &mut ChannelTestInstance,
        item: SubTestItem,
        start_time: DateTime<Utc>,
    ) -> AppResult<()>;

    /// 为重测重置 ChannelTestInstance 的相关状态
    async fn reset_for_retest(
        &self,
        instance: &mut ChannelTestInstance,
    ) -> AppResult<()>;
}

/// 通道状态管理器实现
/// 
/// 无状态的服务，只依赖配置信息，不持有可变运行时数据
pub struct ChannelStateManager {
    /// 服务名称，用于日志记录
    service_name: String,
}

impl ChannelStateManager {
    /// 创建新的通道状态管理器实例
    pub fn new() -> Self {
        Self {
            service_name: "ChannelStateManager".to_string(),
        }
    }

    /// 根据模块类型确定需要初始化的子测试项
    fn determine_applicable_sub_tests(&self, module_type: ModuleType, definition: &ChannelPointDefinition) -> HashMap<SubTestItem, SubTestExecutionResult> {
        let mut sub_tests = HashMap::new();
        let now = Utc::now();

        match module_type {
            ModuleType::AI | ModuleType::AINone => {
                // AI点的基础测试项
                sub_tests.insert(SubTestItem::HardPoint, SubTestExecutionResult {
                    status: SubTestStatus::NotTested,
                    details: None,
                    expected_value: None,
                    actual_value: None,
                    timestamp: now,
                });

                // 趋势和报表检查
                sub_tests.insert(SubTestItem::TrendCheck, SubTestExecutionResult {
                    status: SubTestStatus::NotTested,
                    details: None,
                    expected_value: None,
                    actual_value: None,
                    timestamp: now,
                });

                sub_tests.insert(SubTestItem::ReportCheck, SubTestExecutionResult {
                    status: SubTestStatus::NotTested,
                    details: None,
                    expected_value: None,
                    actual_value: None,
                    timestamp: now,
                });

                // 报警测试项（如果配置了相应的地址）
                if definition.sll_set_point_address.is_some() {
                    sub_tests.insert(SubTestItem::LowLowAlarm, SubTestExecutionResult {
                        status: SubTestStatus::NotTested,
                        details: None,
                        expected_value: definition.sll_set_value.map(|v| v.to_string()),
                        actual_value: None,
                        timestamp: now,
                    });
                } else {
                    sub_tests.insert(SubTestItem::LowLowAlarm, SubTestExecutionResult {
                        status: SubTestStatus::NotApplicable,
                        details: Some("未配置低低报地址".to_string()),
                        expected_value: None,
                        actual_value: None,
                        timestamp: now,
                    });
                }

                if definition.sl_set_point_address.is_some() {
                    sub_tests.insert(SubTestItem::LowAlarm, SubTestExecutionResult {
                        status: SubTestStatus::NotTested,
                        details: None,
                        expected_value: definition.sl_set_value.map(|v| v.to_string()),
                        actual_value: None,
                        timestamp: now,
                    });
                } else {
                    sub_tests.insert(SubTestItem::LowAlarm, SubTestExecutionResult {
                        status: SubTestStatus::NotApplicable,
                        details: Some("未配置低报地址".to_string()),
                        expected_value: None,
                        actual_value: None,
                        timestamp: now,
                    });
                }

                if definition.sh_set_point_address.is_some() {
                    sub_tests.insert(SubTestItem::HighAlarm, SubTestExecutionResult {
                        status: SubTestStatus::NotTested,
                        details: None,
                        expected_value: definition.sh_set_value.map(|v| v.to_string()),
                        actual_value: None,
                        timestamp: now,
                    });
                } else {
                    sub_tests.insert(SubTestItem::HighAlarm, SubTestExecutionResult {
                        status: SubTestStatus::NotApplicable,
                        details: Some("未配置高报地址".to_string()),
                        expected_value: None,
                        actual_value: None,
                        timestamp: now,
                    });
                }

                if definition.shh_set_point_address.is_some() {
                    sub_tests.insert(SubTestItem::HighHighAlarm, SubTestExecutionResult {
                        status: SubTestStatus::NotTested,
                        details: None,
                        expected_value: definition.shh_set_value.map(|v| v.to_string()),
                        actual_value: None,
                        timestamp: now,
                    });
                } else {
                    sub_tests.insert(SubTestItem::HighHighAlarm, SubTestExecutionResult {
                        status: SubTestStatus::NotApplicable,
                        details: Some("未配置高高报地址".to_string()),
                        expected_value: None,
                        actual_value: None,
                        timestamp: now,
                    });
                }

                // 报警值设定整体状态
                sub_tests.insert(SubTestItem::AlarmValueSetting, SubTestExecutionResult {
                    status: SubTestStatus::NotTested,
                    details: None,
                    expected_value: None,
                    actual_value: None,
                    timestamp: now,
                });

                // 维护功能（如果配置了相应的地址）
                if definition.maintenance_value_set_point_address.is_some() {
                    sub_tests.insert(SubTestItem::MaintenanceFunction, SubTestExecutionResult {
                        status: SubTestStatus::NotTested,
                        details: None,
                        expected_value: None,
                        actual_value: None,
                        timestamp: now,
                    });
                } else {
                    sub_tests.insert(SubTestItem::MaintenanceFunction, SubTestExecutionResult {
                        status: SubTestStatus::NotApplicable,
                        details: Some("未配置维护功能地址".to_string()),
                        expected_value: None,
                        actual_value: None,
                        timestamp: now,
                    });
                }
            },

            ModuleType::AO | ModuleType::AONone => {
                // AO点的基础测试项
                sub_tests.insert(SubTestItem::HardPoint, SubTestExecutionResult {
                    status: SubTestStatus::NotTested,
                    details: None,
                    expected_value: None,
                    actual_value: None,
                    timestamp: now,
                });

                sub_tests.insert(SubTestItem::TrendCheck, SubTestExecutionResult {
                    status: SubTestStatus::NotTested,
                    details: None,
                    expected_value: None,
                    actual_value: None,
                    timestamp: now,
                });

                sub_tests.insert(SubTestItem::ReportCheck, SubTestExecutionResult {
                    status: SubTestStatus::NotTested,
                    details: None,
                    expected_value: None,
                    actual_value: None,
                    timestamp: now,
                });

                // 维护功能（如果配置了相应的地址）
                if definition.maintenance_value_set_point_address.is_some() {
                    sub_tests.insert(SubTestItem::MaintenanceFunction, SubTestExecutionResult {
                        status: SubTestStatus::NotTested,
                        details: None,
                        expected_value: None,
                        actual_value: None,
                        timestamp: now,
                    });
                } else {
                    sub_tests.insert(SubTestItem::MaintenanceFunction, SubTestExecutionResult {
                        status: SubTestStatus::NotApplicable,
                        details: Some("未配置维护功能地址".to_string()),
                        expected_value: None,
                        actual_value: None,
                        timestamp: now,
                    });
                }
            },

            ModuleType::DI | ModuleType::DINone => {
                // DI点的基础测试项
                sub_tests.insert(SubTestItem::HardPoint, SubTestExecutionResult {
                    status: SubTestStatus::NotTested,
                    details: None,
                    expected_value: None,
                    actual_value: None,
                    timestamp: now,
                });

                sub_tests.insert(SubTestItem::StateDisplay, SubTestExecutionResult {
                    status: SubTestStatus::NotTested,
                    details: None,
                    expected_value: None,
                    actual_value: None,
                    timestamp: now,
                });
            },

            ModuleType::DO | ModuleType::DONone => {
                // DO点的基础测试项
                sub_tests.insert(SubTestItem::HardPoint, SubTestExecutionResult {
                    status: SubTestStatus::NotTested,
                    details: None,
                    expected_value: None,
                    actual_value: None,
                    timestamp: now,
                });

                sub_tests.insert(SubTestItem::StateDisplay, SubTestExecutionResult {
                    status: SubTestStatus::NotTested,
                    details: None,
                    expected_value: None,
                    actual_value: None,
                    timestamp: now,
                });
            },
        }

        sub_tests
    }

    /// 评估整体状态 - 这是状态机的核心逻辑
    fn evaluate_overall_status(&self, instance: &mut ChannelTestInstance) {
        debug!("[{}] 开始评估实例 {} 的整体状态", self.service_name, instance.instance_id);

        // 统计各种状态的子测试项数量
        let mut total_applicable = 0;
        let mut not_tested = 0;
        let mut testing = 0;
        let mut passed = 0;
        let mut failed = 0;
        let mut skipped = 0;

        for (item, result) in &instance.sub_test_results {
            match result.status {
                SubTestStatus::NotApplicable => {
                    // 不计入统计
                    debug!("[{}] 子测试项 {:?} 不适用", self.service_name, item);
                },
                SubTestStatus::NotTested => {
                    total_applicable += 1;
                    not_tested += 1;
                },
                SubTestStatus::Testing => {
                    total_applicable += 1;
                    testing += 1;
                },
                SubTestStatus::Passed => {
                    total_applicable += 1;
                    passed += 1;
                },
                SubTestStatus::Failed => {
                    total_applicable += 1;
                    failed += 1;
                },
                SubTestStatus::Skipped => {
                    total_applicable += 1;
                    skipped += 1;
                },
            }
        }

        debug!("[{}] 状态统计 - 总计: {}, 未测试: {}, 测试中: {}, 通过: {}, 失败: {}, 跳过: {}", 
               self.service_name, total_applicable, not_tested, testing, passed, failed, skipped);

        // 根据业务规则确定整体状态
        let new_status = if failed > 0 {
            // 任何关键项失败则整体失败
            instance.error_message = Some(format!("有 {} 个子测试项失败", failed));
            OverallTestStatus::TestCompletedFailed
        } else if testing > 0 {
            // 有测试项正在进行
            if instance.sub_test_results.get(&SubTestItem::HardPoint)
                .map(|r| r.status == SubTestStatus::Testing)
                .unwrap_or(false) {
                OverallTestStatus::HardPointTesting
            } else {
                OverallTestStatus::ManualTesting
            }
        } else if not_tested > 0 {
            // 还有未测试的项目
            if instance.overall_status == OverallTestStatus::WiringConfirmed {
                OverallTestStatus::WiringConfirmed // 保持接线确认状态
            } else {
                OverallTestStatus::NotTested
            }
        } else if passed == total_applicable {
            // 所有适用项都通过
            instance.final_test_time = Some(Utc::now());
            if let Some(start_time) = instance.start_time {
                instance.total_test_duration_ms = Some(
                    (Utc::now() - start_time).num_milliseconds()
                );
            }
            instance.error_message = None;
            OverallTestStatus::TestCompletedPassed
        } else if passed > 0 && skipped == (total_applicable - passed) {
            // 部分通过，其余跳过
            instance.final_test_time = Some(Utc::now());
            if let Some(start_time) = instance.start_time {
                instance.total_test_duration_ms = Some(
                    (Utc::now() - start_time).num_milliseconds()
                );
            }
            OverallTestStatus::TestCompletedPassed
        } else {
            // 其他情况，检查硬点测试是否完成
            if let Some(hardpoint_result) = instance.sub_test_results.get(&SubTestItem::HardPoint) {
                if hardpoint_result.status == SubTestStatus::Passed {
                    OverallTestStatus::HardPointTestCompleted
                } else {
                    instance.overall_status // 保持当前状态
                }
            } else {
                instance.overall_status // 保持当前状态
            }
        };

        if new_status != instance.overall_status {
            debug!("[{}] 实例 {} 状态从 {:?} 变更为 {:?}", 
                   self.service_name, instance.instance_id, instance.overall_status, new_status);
            instance.overall_status = new_status;
        }

        instance.last_updated_time = Utc::now();
    }
}

#[async_trait]
impl IChannelStateManager for ChannelStateManager {
    async fn initialize_channel_test_instance(
        &self,
        definition: &ChannelPointDefinition,
        batch_id: &str,
        instance_id: String,
    ) -> AppResult<ChannelTestInstance> {
        debug!("[{}] 初始化通道测试实例: {} (定义ID: {})", 
               self.service_name, instance_id, definition.id);

        let now = Utc::now();
        
        // 根据模块类型确定适用的子测试项
        let sub_test_results = self.determine_applicable_sub_tests(definition.module_type, definition);

        let mut instance = ChannelTestInstance {
            instance_id,
            definition_id: definition.id.clone(),
            test_batch_id: batch_id.to_string(),
            overall_status: OverallTestStatus::NotTested,
            current_step_details: None,
            error_message: None,
            start_time: None,
            last_updated_time: now,
            final_test_time: None,
            total_test_duration_ms: None,
            sub_test_results,
            hardpoint_readings: None,
            manual_test_current_value_input: None,
            manual_test_current_value_output: None,
        };

        debug!("[{}] 成功初始化实例 {}, 包含 {} 个子测试项", 
               self.service_name, instance.instance_id, instance.sub_test_results.len());

        Ok(instance)
    }

    async fn apply_raw_outcome(
        &self,
        instance: &mut ChannelTestInstance,
        outcome: RawTestOutcome,
    ) -> AppResult<()> {
        debug!("[{}] 应用原始测试结果到实例 {}: {:?}", 
               self.service_name, instance.instance_id, outcome.sub_test_item);

        // 验证outcome是否属于这个实例
        if outcome.channel_instance_id != instance.instance_id {
            return Err(AppError::StateTransitionError {
                from_state: "任意状态".to_string(),
                to_state: "应用测试结果".to_string(),
                message: format!("测试结果的实例ID ({}) 与当前实例ID ({}) 不匹配", 
                        outcome.channel_instance_id, instance.instance_id)
            });
        }

        // 更新对应的子测试结果
        if let Some(sub_result) = instance.sub_test_results.get_mut(&outcome.sub_test_item) {
            sub_result.status = if outcome.success {
                SubTestStatus::Passed
            } else {
                SubTestStatus::Failed
            };
            
            sub_result.details = outcome.message.clone();
            sub_result.actual_value = outcome.raw_value_read.clone();
            sub_result.timestamp = outcome.timestamp;

            // 如果是模拟量硬点测试，更新硬点读数
            if outcome.sub_test_item == SubTestItem::HardPoint {
                if let Some(analog_point) = outcome.analog_reading_point {
                    if instance.hardpoint_readings.is_none() {
                        instance.hardpoint_readings = Some(Vec::new());
                    }
                    if let Some(ref mut readings) = instance.hardpoint_readings {
                        readings.push(analog_point);
                    }
                }
            }

            debug!("[{}] 更新子测试项 {:?} 状态为 {:?}", 
                   self.service_name, outcome.sub_test_item, sub_result.status);
        } else {
            warn!("[{}] 实例 {} 中未找到子测试项 {:?}", 
                  self.service_name, instance.instance_id, outcome.sub_test_item);
        }

        // 重新评估整体状态
        self.evaluate_overall_status(instance);

        Ok(())
    }

    async fn mark_as_skipped(
        &self,
        instance: &mut ChannelTestInstance,
        reason: String,
        skip_time: DateTime<Utc>,
    ) -> AppResult<()> {
        debug!("[{}] 标记实例 {} 为跳过: {}", 
               self.service_name, instance.instance_id, reason);

        instance.overall_status = OverallTestStatus::Skipped;
        instance.error_message = Some(reason);
        instance.final_test_time = Some(skip_time);
        instance.last_updated_time = skip_time;

        // 将所有未完成的子测试项标记为跳过
        for (item, result) in instance.sub_test_results.iter_mut() {
            if matches!(result.status, SubTestStatus::NotTested | SubTestStatus::Testing) {
                result.status = SubTestStatus::Skipped;
                result.details = Some("整个测试实例被跳过".to_string());
                result.timestamp = skip_time;
                debug!("[{}] 子测试项 {:?} 被标记为跳过", self.service_name, item);
            }
        }

        Ok(())
    }

    async fn prepare_for_wiring_confirmation(
        &self,
        instance: &mut ChannelTestInstance,
        confirm_time: DateTime<Utc>,
    ) -> AppResult<()> {
        debug!("[{}] 准备实例 {} 的接线确认", 
               self.service_name, instance.instance_id);

        instance.overall_status = OverallTestStatus::WiringConfirmed;
        instance.current_step_details = Some("接线已确认，等待开始测试".to_string());
        instance.last_updated_time = confirm_time;

        Ok(())
    }

    async fn begin_hard_point_test(
        &self,
        instance: &mut ChannelTestInstance,
        start_time: DateTime<Utc>,
    ) -> AppResult<()> {
        debug!("[{}] 开始实例 {} 的硬点测试", 
               self.service_name, instance.instance_id);

        instance.overall_status = OverallTestStatus::HardPointTesting;
        instance.current_step_details = Some("正在进行硬点回路测试".to_string());
        instance.start_time = Some(start_time);
        instance.last_updated_time = start_time;

        // 将硬点测试项标记为测试中
        if let Some(hardpoint_result) = instance.sub_test_results.get_mut(&SubTestItem::HardPoint) {
            hardpoint_result.status = SubTestStatus::Testing;
            hardpoint_result.timestamp = start_time;
        }

        Ok(())
    }

    async fn begin_manual_sub_test(
        &self,
        instance: &mut ChannelTestInstance,
        item: SubTestItem,
        start_time: DateTime<Utc>,
    ) -> AppResult<()> {
        debug!("[{}] 开始实例 {} 的手动子测试: {:?}", 
               self.service_name, instance.instance_id, item);

        instance.overall_status = OverallTestStatus::ManualTesting;
        instance.current_step_details = Some(format!("正在进行手动测试: {:?}", item));
        instance.last_updated_time = start_time;

        // 将指定的子测试项标记为测试中
        if let Some(sub_result) = instance.sub_test_results.get_mut(&item) {
            sub_result.status = SubTestStatus::Testing;
            sub_result.timestamp = start_time;
        } else {
            return Err(AppError::StateTransitionError {
                from_state: "未知状态".to_string(),
                to_state: "手动测试".to_string(),
                message: format!("实例 {} 中未找到子测试项 {:?}", instance.instance_id, item)
            });
        }

        Ok(())
    }

    async fn reset_for_retest(
        &self,
        instance: &mut ChannelTestInstance,
    ) -> AppResult<()> {
        debug!("[{}] 重置实例 {} 以进行重测", 
               self.service_name, instance.instance_id);

        // 重置整体状态
        instance.overall_status = OverallTestStatus::Retesting;
        instance.current_step_details = Some("准备重新测试".to_string());
        instance.error_message = None;
        instance.start_time = None;
        instance.final_test_time = None;
        instance.total_test_duration_ms = None;
        instance.last_updated_time = Utc::now();

        // 重置硬点读数
        instance.hardpoint_readings = None;
        instance.manual_test_current_value_input = None;
        instance.manual_test_current_value_output = None;

        // 重置所有失败或已完成的子测试项为未测试状态
        let reset_time = Utc::now();
        for (item, result) in instance.sub_test_results.iter_mut() {
            match result.status {
                SubTestStatus::Failed | SubTestStatus::Passed => {
                    result.status = SubTestStatus::NotTested;
                    result.details = None;
                    result.actual_value = None;
                    result.timestamp = reset_time;
                    debug!("[{}] 重置子测试项 {:?} 为未测试状态", self.service_name, item);
                },
                SubTestStatus::NotApplicable => {
                    // 保持不适用状态
                },
                _ => {
                    // 其他状态也重置为未测试
                    result.status = SubTestStatus::NotTested;
                    result.details = None;
                    result.actual_value = None;
                    result.timestamp = reset_time;
                }
            }
        }

        // 重新评估整体状态
        self.evaluate_overall_status(instance);

        Ok(())
    }
}

impl Default for ChannelStateManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{PointDataType, ModuleType};

    #[test]
    fn test_simple_verification() {
        // 简单的验证测试
        assert_eq!(2 + 2, 4);
    }

    /// 创建测试用的AI点位定义
    fn create_test_ai_definition() -> ChannelPointDefinition {
        ChannelPointDefinition {
            id: "test_ai_001".to_string(),
            tag: "AI_001".to_string(),
            variable_name: "Temperature_01".to_string(),
            variable_description: "温度测量点1".to_string(),
            station_name: "Station1".to_string(),
            module_name: "AI_Module_1".to_string(),
            module_type: ModuleType::AI,
            channel_tag_in_module: "CH01".to_string(),
            data_type: PointDataType::Float,
            power_supply_type: "有源".to_string(),
            wire_system: "4线制".to_string(),
            plc_absolute_address: Some("DB1.DBD0".to_string()),
            plc_communication_address: "DB1.DBD0".to_string(),
            range_lower_limit: Some(0.0),
            range_upper_limit: Some(100.0),
            engineering_unit: Some("°C".to_string()),
            sll_set_value: Some(10.0),
            sll_set_point_address: Some("DB1.DBD4".to_string()),
            sll_feedback_address: Some("DB1.DBX8.0".to_string()),
            sl_set_value: Some(20.0),
            sl_set_point_address: Some("DB1.DBD12".to_string()),
            sl_feedback_address: Some("DB1.DBX8.1".to_string()),
            sh_set_value: Some(80.0),
            sh_set_point_address: Some("DB1.DBD16".to_string()),
            sh_feedback_address: Some("DB1.DBX8.2".to_string()),
            shh_set_value: Some(90.0),
            shh_set_point_address: Some("DB1.DBD20".to_string()),
            shh_feedback_address: Some("DB1.DBX8.3".to_string()),
            maintenance_value_set_point_address: Some("DB1.DBD24".to_string()),
            maintenance_enable_switch_point_address: Some("DB1.DBX8.4".to_string()),
            access_property: Some("读写".to_string()),
            save_history: Some(true),
            power_failure_protection: Some(false),
            test_rig_plc_address: Some("DB2.DBD0".to_string()),
        }
    }

    /// 创建测试用的DI点位定义
    fn create_test_di_definition() -> ChannelPointDefinition {
        ChannelPointDefinition {
            id: "test_di_001".to_string(),
            tag: "DI_001".to_string(),
            variable_name: "Switch_01".to_string(),
            variable_description: "开关状态1".to_string(),
            station_name: "Station1".to_string(),
            module_name: "DI_Module_1".to_string(),
            module_type: ModuleType::DI,
            channel_tag_in_module: "CH01".to_string(),
            data_type: PointDataType::Bool,
            power_supply_type: "无源".to_string(),
            wire_system: "2线制".to_string(),
            plc_absolute_address: Some("DB1.DBX0.0".to_string()),
            plc_communication_address: "DB1.DBX0.0".to_string(),
            range_lower_limit: None,
            range_upper_limit: None,
            engineering_unit: None,
            sll_set_value: None,
            sll_set_point_address: None,
            sll_feedback_address: None,
            sl_set_value: None,
            sl_set_point_address: None,
            sl_feedback_address: None,
            sh_set_value: None,
            sh_set_point_address: None,
            sh_feedback_address: None,
            shh_set_value: None,
            shh_set_point_address: None,
            shh_feedback_address: None,
            maintenance_value_set_point_address: None,
            maintenance_enable_switch_point_address: None,
            access_property: Some("只读".to_string()),
            save_history: Some(true),
            power_failure_protection: Some(false),
            test_rig_plc_address: Some("DB2.DBX0.0".to_string()),
        }
    }

    #[test]
    fn test_channel_state_manager_creation() {
        let manager = ChannelStateManager::new();
        assert_eq!(manager.service_name, "ChannelStateManager");
    }

    #[test]
    fn test_initialize_ai_channel_test_instance() {
        let manager = ChannelStateManager::new();
        let definition = create_test_ai_definition();
        let batch_id = "test_batch_001";
        let instance_id = "test_instance_001".to_string();

        // 由于我们移除了async，这里需要使用同步的方式来测试
        // 但是我们的方法是async的，所以我们需要使用tokio的block_on
        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(manager.initialize_channel_test_instance(
            &definition,
            batch_id,
            instance_id.clone(),
        ));

        assert!(result.is_ok());
        let instance = result.unwrap();

        assert_eq!(instance.instance_id, instance_id);
        assert_eq!(instance.definition_id, definition.id);
        assert_eq!(instance.test_batch_id, batch_id);
        assert_eq!(instance.overall_status, OverallTestStatus::NotTested);

        // 验证AI点的子测试项
        assert!(instance.sub_test_results.contains_key(&SubTestItem::HardPoint));
        assert!(instance.sub_test_results.contains_key(&SubTestItem::TrendCheck));
        assert!(instance.sub_test_results.contains_key(&SubTestItem::ReportCheck));
        assert!(instance.sub_test_results.contains_key(&SubTestItem::LowLowAlarm));
        assert!(instance.sub_test_results.contains_key(&SubTestItem::LowAlarm));
        assert!(instance.sub_test_results.contains_key(&SubTestItem::HighAlarm));
        assert!(instance.sub_test_results.contains_key(&SubTestItem::HighHighAlarm));
        assert!(instance.sub_test_results.contains_key(&SubTestItem::AlarmValueSetting));
        assert!(instance.sub_test_results.contains_key(&SubTestItem::MaintenanceFunction));

        // 验证初始状态
        for (item, result) in &instance.sub_test_results {
            if matches!(item, SubTestItem::LowLowAlarm | SubTestItem::LowAlarm | 
                              SubTestItem::HighAlarm | SubTestItem::HighHighAlarm | 
                              SubTestItem::MaintenanceFunction) {
                assert_eq!(result.status, SubTestStatus::NotTested);
            } else {
                assert_eq!(result.status, SubTestStatus::NotTested);
            }
        }
    }

    #[test]
    fn test_initialize_di_channel_test_instance() {
        let manager = ChannelStateManager::new();
        let definition = create_test_di_definition();
        let batch_id = "test_batch_001";
        let instance_id = "test_instance_002".to_string();

        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(manager.initialize_channel_test_instance(
            &definition,
            batch_id,
            instance_id.clone(),
        ));

        assert!(result.is_ok());
        let instance = result.unwrap();

        // 验证DI点的子测试项
        assert!(instance.sub_test_results.contains_key(&SubTestItem::HardPoint));
        assert!(instance.sub_test_results.contains_key(&SubTestItem::StateDisplay));
        
        // DI点不应该有报警相关的测试项
        assert!(!instance.sub_test_results.contains_key(&SubTestItem::LowLowAlarm));
        assert!(!instance.sub_test_results.contains_key(&SubTestItem::HighAlarm));
    }

    #[test]
    fn test_apply_successful_hardpoint_outcome() {
        let manager = ChannelStateManager::new();
        let definition = create_test_ai_definition();
        let batch_id = "test_batch_001";
        let instance_id = "test_instance_003".to_string();

        let rt = tokio::runtime::Runtime::new().unwrap();
        let mut instance = rt.block_on(manager.initialize_channel_test_instance(
            &definition,
            batch_id,
            instance_id.clone(),
        )).unwrap();

        // 创建成功的硬点测试结果
        let outcome = RawTestOutcome {
            channel_instance_id: instance_id.clone(),
            sub_test_item: SubTestItem::HardPoint,
            success: true,
            raw_value_read: Some("50.5".to_string()),
            eng_value_calculated: Some("50.5°C".to_string()),
            message: Some("硬点测试通过".to_string()),
            timestamp: Utc::now(),
            analog_reading_point: Some(AnalogReadingPoint {
                set_percentage: 0.5,
                set_value_eng: 50.0,
                expected_reading_raw: Some(50.0),
                actual_reading_raw: Some(50.5),
                actual_reading_eng: Some(50.5),
                status: SubTestStatus::Passed,
            }),
        };

        let result = rt.block_on(manager.apply_raw_outcome(&mut instance, outcome));
        assert!(result.is_ok());

        // 验证硬点测试项状态更新
        let hardpoint_result = instance.sub_test_results.get(&SubTestItem::HardPoint).unwrap();
        assert_eq!(hardpoint_result.status, SubTestStatus::Passed);
        assert_eq!(hardpoint_result.actual_value, Some("50.5".to_string()));

        // 验证硬点读数被记录
        assert!(instance.hardpoint_readings.is_some());
        let readings = instance.hardpoint_readings.as_ref().unwrap();
        assert_eq!(readings.len(), 1);
        assert_eq!(readings[0].set_percentage, 0.5);
    }

    #[test]
    fn test_apply_failed_outcome() {
        let manager = ChannelStateManager::new();
        let definition = create_test_ai_definition();
        let batch_id = "test_batch_001";
        let instance_id = "test_instance_004".to_string();

        let rt = tokio::runtime::Runtime::new().unwrap();
        let mut instance = rt.block_on(manager.initialize_channel_test_instance(
            &definition,
            batch_id,
            instance_id.clone(),
        )).unwrap();

        // 创建失败的测试结果
        let outcome = RawTestOutcome {
            channel_instance_id: instance_id.clone(),
            sub_test_item: SubTestItem::HighAlarm,
            success: false,
            raw_value_read: Some("false".to_string()),
            eng_value_calculated: None,
            message: Some("高报警未触发".to_string()),
            timestamp: Utc::now(),
            analog_reading_point: None,
        };

        let result = rt.block_on(manager.apply_raw_outcome(&mut instance, outcome));
        assert!(result.is_ok());

        // 验证测试项状态更新
        let alarm_result = instance.sub_test_results.get(&SubTestItem::HighAlarm).unwrap();
        assert_eq!(alarm_result.status, SubTestStatus::Failed);

        // 验证整体状态变为失败
        assert_eq!(instance.overall_status, OverallTestStatus::TestCompletedFailed);
        assert!(instance.error_message.is_some());
    }

    #[test]
    fn test_mark_as_skipped() {
        let manager = ChannelStateManager::new();
        let definition = create_test_ai_definition();
        let batch_id = "test_batch_001";
        let instance_id = "test_instance_005".to_string();

        let rt = tokio::runtime::Runtime::new().unwrap();
        let mut instance = rt.block_on(manager.initialize_channel_test_instance(
            &definition,
            batch_id,
            instance_id.clone(),
        )).unwrap();

        let skip_time = Utc::now();
        let reason = "用户手动跳过".to_string();

        let result = rt.block_on(manager.mark_as_skipped(&mut instance, reason.clone(), skip_time));
        assert!(result.is_ok());

        // 验证整体状态
        assert_eq!(instance.overall_status, OverallTestStatus::Skipped);
        assert_eq!(instance.error_message, Some(reason));
        assert_eq!(instance.final_test_time, Some(skip_time));

        // 验证所有子测试项都被标记为跳过
        for (_, result) in &instance.sub_test_results {
            assert_eq!(result.status, SubTestStatus::Skipped);
        }
    }

    #[test]
    fn test_prepare_for_wiring_confirmation() {
        let manager = ChannelStateManager::new();
        let definition = create_test_ai_definition();
        let batch_id = "test_batch_001";
        let instance_id = "test_instance_006".to_string();

        let rt = tokio::runtime::Runtime::new().unwrap();
        let mut instance = rt.block_on(manager.initialize_channel_test_instance(
            &definition,
            batch_id,
            instance_id.clone(),
        )).unwrap();

        let confirm_time = Utc::now();

        let result = rt.block_on(manager.prepare_for_wiring_confirmation(&mut instance, confirm_time));
        assert!(result.is_ok());

        assert_eq!(instance.overall_status, OverallTestStatus::WiringConfirmed);
        assert!(instance.current_step_details.is_some());
        assert_eq!(instance.last_updated_time, confirm_time);
    }

    #[test]
    fn test_begin_hard_point_test() {
        let manager = ChannelStateManager::new();
        let definition = create_test_ai_definition();
        let batch_id = "test_batch_001";
        let instance_id = "test_instance_007".to_string();

        let rt = tokio::runtime::Runtime::new().unwrap();
        let mut instance = rt.block_on(manager.initialize_channel_test_instance(
            &definition,
            batch_id,
            instance_id.clone(),
        )).unwrap();

        let start_time = Utc::now();

        let result = rt.block_on(manager.begin_hard_point_test(&mut instance, start_time));
        assert!(result.is_ok());

        assert_eq!(instance.overall_status, OverallTestStatus::HardPointTesting);
        assert_eq!(instance.start_time, Some(start_time));

        // 验证硬点测试项状态
        let hardpoint_result = instance.sub_test_results.get(&SubTestItem::HardPoint).unwrap();
        assert_eq!(hardpoint_result.status, SubTestStatus::Testing);
    }

    #[test]
    fn test_begin_manual_sub_test() {
        let manager = ChannelStateManager::new();
        let definition = create_test_ai_definition();
        let batch_id = "test_batch_001";
        let instance_id = "test_instance_008".to_string();

        let rt = tokio::runtime::Runtime::new().unwrap();
        let mut instance = rt.block_on(manager.initialize_channel_test_instance(
            &definition,
            batch_id,
            instance_id.clone(),
        )).unwrap();

        let start_time = Utc::now();
        let test_item = SubTestItem::HighAlarm;

        let result = rt.block_on(manager.begin_manual_sub_test(&mut instance, test_item, start_time));
        assert!(result.is_ok());

        assert_eq!(instance.overall_status, OverallTestStatus::ManualTesting);

        // 验证指定测试项状态
        let alarm_result = instance.sub_test_results.get(&test_item).unwrap();
        assert_eq!(alarm_result.status, SubTestStatus::Testing);
    }

    #[test]
    fn test_reset_for_retest() {
        let manager = ChannelStateManager::new();
        let definition = create_test_ai_definition();
        let batch_id = "test_batch_001";
        let instance_id = "test_instance_009".to_string();

        let rt = tokio::runtime::Runtime::new().unwrap();
        let mut instance = rt.block_on(manager.initialize_channel_test_instance(
            &definition,
            batch_id,
            instance_id.clone(),
        )).unwrap();

        // 先设置一些测试结果
        instance.overall_status = OverallTestStatus::TestCompletedFailed;
        instance.error_message = Some("测试失败".to_string());
        instance.start_time = Some(Utc::now());
        instance.final_test_time = Some(Utc::now());

        // 设置一些子测试项为失败状态
        if let Some(result) = instance.sub_test_results.get_mut(&SubTestItem::HardPoint) {
            result.status = SubTestStatus::Failed;
            result.details = Some("硬点测试失败".to_string());
        }

        let reset_result = rt.block_on(manager.reset_for_retest(&mut instance));
        assert!(reset_result.is_ok());

        // 验证重置后的状态
        assert_eq!(instance.overall_status, OverallTestStatus::Retesting);
        assert!(instance.error_message.is_none());
        assert!(instance.start_time.is_none());
        assert!(instance.final_test_time.is_none());
        assert!(instance.hardpoint_readings.is_none());

        // 验证失败的子测试项被重置
        let hardpoint_result = instance.sub_test_results.get(&SubTestItem::HardPoint).unwrap();
        assert_eq!(hardpoint_result.status, SubTestStatus::NotTested);
        assert!(hardpoint_result.details.is_none());
    }

    #[test]
    fn test_outcome_instance_id_mismatch() {
        let manager = ChannelStateManager::new();
        let definition = create_test_ai_definition();
        let batch_id = "test_batch_001";
        let instance_id = "test_instance_010".to_string();

        let rt = tokio::runtime::Runtime::new().unwrap();
        let mut instance = rt.block_on(manager.initialize_channel_test_instance(
            &definition,
            batch_id,
            instance_id.clone(),
        )).unwrap();

        // 创建错误实例ID的测试结果
        let outcome = RawTestOutcome {
            channel_instance_id: "wrong_instance_id".to_string(),
            sub_test_item: SubTestItem::HardPoint,
            success: true,
            raw_value_read: Some("50.5".to_string()),
            eng_value_calculated: Some("50.5°C".to_string()),
            message: Some("硬点测试通过".to_string()),
            timestamp: Utc::now(),
            analog_reading_point: None,
        };

        let result = rt.block_on(manager.apply_raw_outcome(&mut instance, outcome));
        assert!(result.is_err());
        
        if let Err(AppError::StateTransitionError { from_state, to_state, message }) = result {
            assert!(from_state.contains("任意状态"));
            assert!(to_state.contains("应用测试结果"));
            assert!(message.contains("实例ID") && message.contains("不匹配"));
        } else {
            panic!("期望StateTransitionError");
        }
    }
} 