/// 特定测试步骤执行器
/// 
/// 这个模块实现了具体的测试步骤执行逻辑，每个执行器负责一个原子的测试操作
/// 如硬点测试的某个百分比点、报警测试、状态读取等

use crate::models::{
    ChannelTestInstance, ChannelPointDefinition, RawTestOutcome, SubTestItem, 
    AnalogReadingPoint, SubTestStatus, ModuleType, PointDataType
};
use crate::services::infrastructure::plc::IPlcCommunicationService;
use crate::utils::error::{AppError, AppResult};
use async_trait::async_trait;
use chrono::Utc;
use std::sync::Arc;
use log::{debug, warn, error, info};

/// 特定测试步骤执行器接口
/// 
/// 每个执行器负责执行一个原子的测试步骤，与PLC交互并返回原始测试结果
#[async_trait]
pub trait ISpecificTestStepExecutor: Send + Sync {
    /// 执行特定的测试步骤
    async fn execute(
        &self,
        instance: &ChannelTestInstance,
        definition: &ChannelPointDefinition,
        plc_service_test_rig: Arc<dyn IPlcCommunicationService>,
        plc_service_target: Arc<dyn IPlcCommunicationService>,
    ) -> AppResult<RawTestOutcome>;

    /// 返回此执行器处理的 SubTestItem 类型
    fn item_type(&self) -> SubTestItem;

    /// 返回执行器名称，用于日志记录
    fn executor_name(&self) -> &'static str;

    /// 检查是否支持指定的点位定义
    fn supports_definition(&self, definition: &ChannelPointDefinition) -> bool;
}

/// AI点硬点百分比测试执行器
/// 
/// 执行AI点硬接线测试中某个百分比的设定与读取
pub struct AIHardPointPercentExecutor {
    /// 目标百分比 (0.0 到 1.0)
    pub target_percentage: f32,
    /// 读取容差 (百分比)
    pub tolerance_percentage: f32,
    /// 稳定等待时间 (毫秒)
    pub stabilization_delay_ms: u64,
}

impl AIHardPointPercentExecutor {
    /// 创建新的AI硬点百分比测试执行器
    pub fn new(target_percentage: f32, tolerance_percentage: f32, stabilization_delay_ms: u64) -> Self {
        Self {
            target_percentage: target_percentage.clamp(0.0, 1.0),
            tolerance_percentage: tolerance_percentage.abs(),
            stabilization_delay_ms,
        }
    }

    /// 计算工程单位设定值
    fn calculate_engineering_value(&self, definition: &ChannelPointDefinition) -> AppResult<f32> {
        let lower = definition.range_lower_limit.unwrap_or(0.0);
        let upper = definition.range_upper_limit.unwrap_or(100.0);
        
        if upper <= lower {
            return Err(AppError::ValidationError {
                field: "range".to_string(),
                message: format!("无效的量程范围: {} 到 {}", lower, upper),
            });
        }

        let eng_value = lower + (upper - lower) * self.target_percentage;
        Ok(eng_value)
    }

    /// 检查读取值是否在容差范围内
    fn is_within_tolerance(&self, expected: f32, actual: f32, range_span: f32) -> bool {
        let tolerance_value = range_span * self.tolerance_percentage;
        (actual - expected).abs() <= tolerance_value
    }
}

#[async_trait]
impl ISpecificTestStepExecutor for AIHardPointPercentExecutor {
    async fn execute(
        &self,
        instance: &ChannelTestInstance,
        definition: &ChannelPointDefinition,
        plc_service_test_rig: Arc<dyn IPlcCommunicationService>,
        plc_service_target: Arc<dyn IPlcCommunicationService>,
    ) -> AppResult<RawTestOutcome> {
        debug!("[{}] 开始执行AI硬点测试 {}% - 实例: {}", 
               self.executor_name(), self.target_percentage * 100.0, instance.instance_id);

        // 计算设定值
        let eng_set_value = self.calculate_engineering_value(definition)?;
        let range_span = definition.range_upper_limit.unwrap_or(100.0) - 
                        definition.range_lower_limit.unwrap_or(0.0);

        // 获取测试台架输出地址
        let test_rig_address = definition.test_rig_plc_address.as_ref()
            .ok_or_else(|| AppError::ValidationError {
                field: "test_rig_plc_address".to_string(),
                message: "未配置测试台架PLC地址".to_string(),
            })?;

        // 获取被测点读取地址
        let target_address = &definition.plc_communication_address;

        let timestamp = Utc::now();

        // 步骤1: 设定测试台架输出值
        info!("[{}] 设定测试台架输出: {} = {:.3} ({}%)", 
              self.executor_name(), test_rig_address, eng_set_value, self.target_percentage * 100.0);

        match definition.data_type {
            PointDataType::Float => {
                plc_service_test_rig.write_f32(test_rig_address, eng_set_value).await?;
            },
            PointDataType::Int => {
                plc_service_test_rig.write_i32(test_rig_address, eng_set_value as i32).await?;
            },
            _ => {
                return Err(AppError::ValidationError {
                    field: "data_type".to_string(),
                    message: format!("AI点不支持的数据类型: {:?}", definition.data_type),
                });
            }
        }

        // 步骤2: 等待信号稳定
        if self.stabilization_delay_ms > 0 {
            debug!("[{}] 等待信号稳定 {} ms", self.executor_name(), self.stabilization_delay_ms);
            tokio::time::sleep(tokio::time::Duration::from_millis(self.stabilization_delay_ms)).await;
        }

        // 步骤3: 读取被测点实际值
        debug!("[{}] 读取被测点实际值: {}", self.executor_name(), target_address);
        
        let (actual_raw_value, actual_eng_value) = match definition.data_type {
            PointDataType::Float => {
                let raw_val = plc_service_target.read_f32(target_address).await?;
                (raw_val, raw_val)
            },
            PointDataType::Int => {
                let raw_val = plc_service_target.read_i32(target_address).await?;
                (raw_val as f32, raw_val as f32)
            },
            _ => {
                return Err(AppError::ValidationError {
                    field: "data_type".to_string(),
                    message: format!("AI点不支持的数据类型: {:?}", definition.data_type),
                });
            }
        };

        // 步骤4: 判断测试结果
        let is_success = self.is_within_tolerance(eng_set_value, actual_eng_value, range_span);
        
        let message = if is_success {
            format!("硬点测试 {}% 通过 - 设定: {:.3}, 实际: {:.3}, 容差: {:.1}%", 
                   self.target_percentage * 100.0, eng_set_value, actual_eng_value, 
                   self.tolerance_percentage * 100.0)
        } else {
            format!("硬点测试 {}% 失败 - 设定: {:.3}, 实际: {:.3}, 偏差超出容差 {:.1}%", 
                   self.target_percentage * 100.0, eng_set_value, actual_eng_value,
                   self.tolerance_percentage * 100.0)
        };

        info!("[{}] {}", self.executor_name(), message);

        // 构造模拟量读数点
        let analog_reading_point = AnalogReadingPoint {
            set_percentage: self.target_percentage,
            set_value_eng: eng_set_value,
            expected_reading_raw: Some(eng_set_value),
            actual_reading_raw: Some(actual_raw_value),
            actual_reading_eng: Some(actual_eng_value),
            status: if is_success { SubTestStatus::Passed } else { SubTestStatus::Failed },
        };

        // 构造原始测试结果
        let outcome = RawTestOutcome {
            channel_instance_id: instance.instance_id.clone(),
            sub_test_item: SubTestItem::HardPoint,
            success: is_success,
            raw_value_read: Some(actual_raw_value.to_string()),
            eng_value_calculated: Some(format!("{:.3}{}", actual_eng_value, 
                definition.engineering_unit.as_deref().unwrap_or(""))),
            message: Some(message),
            timestamp,
            analog_reading_point: Some(analog_reading_point),
        };

        debug!("[{}] 硬点测试 {}% 完成 - 结果: {}", 
               self.executor_name(), self.target_percentage * 100.0, is_success);

        Ok(outcome)
    }

    fn item_type(&self) -> SubTestItem {
        SubTestItem::HardPoint
    }

    fn executor_name(&self) -> &'static str {
        "AIHardPointPercentExecutor"
    }

    fn supports_definition(&self, definition: &ChannelPointDefinition) -> bool {
        matches!(definition.module_type, ModuleType::AI | ModuleType::AINone) &&
        definition.test_rig_plc_address.is_some() &&
        definition.range_lower_limit.is_some() &&
        definition.range_upper_limit.is_some()
    }
}

/// AI点报警测试执行器
/// 
/// 执行AI点某个报警项的测试（设置报警触发条件，验证报警是否产生）
pub struct AIAlarmTestExecutor {
    /// 报警类型
    pub alarm_type: SubTestItem,
    /// 触发等待时间 (毫秒)
    pub trigger_delay_ms: u64,
    /// 恢复等待时间 (毫秒)
    pub recovery_delay_ms: u64,
}

impl AIAlarmTestExecutor {
    /// 创建新的AI报警测试执行器
    pub fn new(alarm_type: SubTestItem, trigger_delay_ms: u64, recovery_delay_ms: u64) -> Self {
        Self {
            alarm_type,
            trigger_delay_ms,
            recovery_delay_ms,
        }
    }

    /// 获取报警相关的地址和设定值
    fn get_alarm_config(&self, definition: &ChannelPointDefinition) -> AppResult<(String, String, f32)> {
        match self.alarm_type {
            SubTestItem::LowLowAlarm => {
                let set_addr = definition.sll_set_point_address.as_ref()
                    .ok_or_else(|| AppError::ValidationError {
                        field: "sll_set_point_address".to_string(),
                        message: "未配置低低报设定地址".to_string(),
                    })?;
                let feedback_addr = definition.sll_feedback_address.as_ref()
                    .ok_or_else(|| AppError::ValidationError {
                        field: "sll_feedback_address".to_string(),
                        message: "未配置低低报反馈地址".to_string(),
                    })?;
                let set_value = definition.sll_set_value
                    .ok_or_else(|| AppError::ValidationError {
                        field: "sll_set_value".to_string(),
                        message: "未配置低低报设定值".to_string(),
                    })?;
                Ok((set_addr.clone(), feedback_addr.clone(), set_value))
            },
            SubTestItem::LowAlarm => {
                let set_addr = definition.sl_set_point_address.as_ref()
                    .ok_or_else(|| AppError::ValidationError {
                        field: "sl_set_point_address".to_string(),
                        message: "未配置低报设定地址".to_string(),
                    })?;
                let feedback_addr = definition.sl_feedback_address.as_ref()
                    .ok_or_else(|| AppError::ValidationError {
                        field: "sl_feedback_address".to_string(),
                        message: "未配置低报反馈地址".to_string(),
                    })?;
                let set_value = definition.sl_set_value
                    .ok_or_else(|| AppError::ValidationError {
                        field: "sl_set_value".to_string(),
                        message: "未配置低报设定值".to_string(),
                    })?;
                Ok((set_addr.clone(), feedback_addr.clone(), set_value))
            },
            SubTestItem::HighAlarm => {
                let set_addr = definition.sh_set_point_address.as_ref()
                    .ok_or_else(|| AppError::ValidationError {
                        field: "sh_set_point_address".to_string(),
                        message: "未配置高报设定地址".to_string(),
                    })?;
                let feedback_addr = definition.sh_feedback_address.as_ref()
                    .ok_or_else(|| AppError::ValidationError {
                        field: "sh_feedback_address".to_string(),
                        message: "未配置高报反馈地址".to_string(),
                    })?;
                let set_value = definition.sh_set_value
                    .ok_or_else(|| AppError::ValidationError {
                        field: "sh_set_value".to_string(),
                        message: "未配置高报设定值".to_string(),
                    })?;
                Ok((set_addr.clone(), feedback_addr.clone(), set_value))
            },
            SubTestItem::HighHighAlarm => {
                let set_addr = definition.shh_set_point_address.as_ref()
                    .ok_or_else(|| AppError::ValidationError {
                        field: "shh_set_point_address".to_string(),
                        message: "未配置高高报设定地址".to_string(),
                    })?;
                let feedback_addr = definition.shh_feedback_address.as_ref()
                    .ok_or_else(|| AppError::ValidationError {
                        field: "shh_feedback_address".to_string(),
                        message: "未配置高高报反馈地址".to_string(),
                    })?;
                let set_value = definition.shh_set_value
                    .ok_or_else(|| AppError::ValidationError {
                        field: "shh_set_value".to_string(),
                        message: "未配置高高报设定值".to_string(),
                    })?;
                Ok((set_addr.clone(), feedback_addr.clone(), set_value))
            },
            _ => Err(AppError::ValidationError {
                field: "alarm_type".to_string(),
                message: format!("不支持的报警类型: {:?}", self.alarm_type),
            }),
        }
    }

    /// 计算触发报警的测试值
    fn calculate_trigger_value(&self, alarm_set_value: f32, definition: &ChannelPointDefinition) -> f32 {
        let range_span = definition.range_upper_limit.unwrap_or(100.0) - 
                        definition.range_lower_limit.unwrap_or(0.0);
        let offset = range_span * 0.05; // 5% 的偏移量

        match self.alarm_type {
            SubTestItem::LowLowAlarm | SubTestItem::LowAlarm => {
                // 低报警：设定值减去偏移量来触发
                alarm_set_value - offset
            },
            SubTestItem::HighAlarm | SubTestItem::HighHighAlarm => {
                // 高报警：设定值加上偏移量来触发
                alarm_set_value + offset
            },
            _ => alarm_set_value,
        }
    }
}

#[async_trait]
impl ISpecificTestStepExecutor for AIAlarmTestExecutor {
    async fn execute(
        &self,
        instance: &ChannelTestInstance,
        definition: &ChannelPointDefinition,
        plc_service_test_rig: Arc<dyn IPlcCommunicationService>,
        plc_service_target: Arc<dyn IPlcCommunicationService>,
    ) -> AppResult<RawTestOutcome> {
        debug!("[{}] 开始执行AI报警测试 {:?} - 实例: {}", 
               self.executor_name(), self.alarm_type, instance.instance_id);

        let timestamp = Utc::now();

        // 获取报警配置
        let (set_point_address, feedback_address, alarm_set_value) = self.get_alarm_config(definition)?;

        // 获取测试台架地址
        let test_rig_address = definition.test_rig_plc_address.as_ref()
            .ok_or_else(|| AppError::ValidationError {
                field: "test_rig_plc_address".to_string(),
                message: "未配置测试台架PLC地址".to_string(),
            })?;

        // 计算触发值
        let trigger_value = self.calculate_trigger_value(alarm_set_value, definition);

        info!("[{}] 报警测试 {:?} - 设定值: {:.3}, 触发值: {:.3}", 
              self.executor_name(), self.alarm_type, alarm_set_value, trigger_value);

        // 步骤1: 设定报警阈值
        debug!("[{}] 设定报警阈值: {} = {:.3}", self.executor_name(), set_point_address, alarm_set_value);
        plc_service_target.write_f32(&set_point_address, alarm_set_value).await?;

        // 步骤2: 设定测试台架输出到触发值
        debug!("[{}] 设定测试台架触发值: {} = {:.3}", self.executor_name(), test_rig_address, trigger_value);
        plc_service_test_rig.write_f32(test_rig_address, trigger_value).await?;

        // 步骤3: 等待报警触发
        if self.trigger_delay_ms > 0 {
            debug!("[{}] 等待报警触发 {} ms", self.executor_name(), self.trigger_delay_ms);
            tokio::time::sleep(tokio::time::Duration::from_millis(self.trigger_delay_ms)).await;
        }

        // 步骤4: 读取报警反馈状态
        debug!("[{}] 读取报警反馈状态: {}", self.executor_name(), feedback_address);
        let alarm_triggered = plc_service_target.read_bool(&feedback_address).await?;

        // 步骤5: 恢复正常值（可选）
        if self.recovery_delay_ms > 0 {
            let normal_value = (definition.range_lower_limit.unwrap_or(0.0) + 
                               definition.range_upper_limit.unwrap_or(100.0)) / 2.0;
            debug!("[{}] 恢复正常值: {} = {:.3}", self.executor_name(), test_rig_address, normal_value);
            plc_service_test_rig.write_f32(test_rig_address, normal_value).await?;
            
            tokio::time::sleep(tokio::time::Duration::from_millis(self.recovery_delay_ms)).await;
        }

        // 判断测试结果
        let is_success = alarm_triggered;
        let message = if is_success {
            format!("报警测试 {:?} 通过 - 设定: {:.3}, 触发值: {:.3}, 报警状态: {}", 
                   self.alarm_type, alarm_set_value, trigger_value, alarm_triggered)
        } else {
            format!("报警测试 {:?} 失败 - 设定: {:.3}, 触发值: {:.3}, 报警未触发", 
                   self.alarm_type, alarm_set_value, trigger_value)
        };

        info!("[{}] {}", self.executor_name(), message);

        // 构造原始测试结果
        let outcome = RawTestOutcome {
            channel_instance_id: instance.instance_id.clone(),
            sub_test_item: self.alarm_type,
            success: is_success,
            raw_value_read: Some(alarm_triggered.to_string()),
            eng_value_calculated: Some(format!("报警状态: {}", if alarm_triggered { "触发" } else { "未触发" })),
            message: Some(message),
            timestamp,
            analog_reading_point: None,
        };

        debug!("[{}] 报警测试 {:?} 完成 - 结果: {}", 
               self.executor_name(), self.alarm_type, is_success);

        Ok(outcome)
    }

    fn item_type(&self) -> SubTestItem {
        self.alarm_type
    }

    fn executor_name(&self) -> &'static str {
        "AIAlarmTestExecutor"
    }

    fn supports_definition(&self, definition: &ChannelPointDefinition) -> bool {
        if !matches!(definition.module_type, ModuleType::AI | ModuleType::AINone) {
            return false;
        }

        match self.alarm_type {
            SubTestItem::LowLowAlarm => {
                definition.sll_set_point_address.is_some() && 
                definition.sll_feedback_address.is_some() &&
                definition.sll_set_value.is_some()
            },
            SubTestItem::LowAlarm => {
                definition.sl_set_point_address.is_some() && 
                definition.sl_feedback_address.is_some() &&
                definition.sl_set_value.is_some()
            },
            SubTestItem::HighAlarm => {
                definition.sh_set_point_address.is_some() && 
                definition.sh_feedback_address.is_some() &&
                definition.sh_set_value.is_some()
            },
            SubTestItem::HighHighAlarm => {
                definition.shh_set_point_address.is_some() && 
                definition.shh_feedback_address.is_some() &&
                definition.shh_set_value.is_some()
            },
            _ => false,
        }
    }
}

/// DI点状态读取执行器
/// 
/// 读取DI点的当前状态
pub struct DIStateReadExecutor {
    /// 期望的状态值（可选）
    pub expected_state: Option<bool>,
    /// 读取等待时间 (毫秒)
    pub read_delay_ms: u64,
}

impl DIStateReadExecutor {
    /// 创建新的DI状态读取执行器
    pub fn new(expected_state: Option<bool>, read_delay_ms: u64) -> Self {
        Self {
            expected_state,
            read_delay_ms,
        }
    }
}

#[async_trait]
impl ISpecificTestStepExecutor for DIStateReadExecutor {
    async fn execute(
        &self,
        instance: &ChannelTestInstance,
        definition: &ChannelPointDefinition,
        _plc_service_test_rig: Arc<dyn IPlcCommunicationService>,
        plc_service_target: Arc<dyn IPlcCommunicationService>,
    ) -> AppResult<RawTestOutcome> {
        debug!("[{}] 开始执行DI状态读取 - 实例: {}", 
               self.executor_name(), instance.instance_id);

        let timestamp = Utc::now();
        let target_address = &definition.plc_communication_address;

        // 等待读取延时
        if self.read_delay_ms > 0 {
            debug!("[{}] 等待读取延时 {} ms", self.executor_name(), self.read_delay_ms);
            tokio::time::sleep(tokio::time::Duration::from_millis(self.read_delay_ms)).await;
        }

        // 读取DI状态
        debug!("[{}] 读取DI状态: {}", self.executor_name(), target_address);
        let actual_state = plc_service_target.read_bool(target_address).await?;

        // 判断测试结果
        let is_success = if let Some(expected) = self.expected_state {
            actual_state == expected
        } else {
            true // 如果没有期望值，只要能读取就算成功
        };

        let message = if let Some(expected) = self.expected_state {
            if is_success {
                format!("DI状态读取通过 - 期望: {}, 实际: {}", expected, actual_state)
            } else {
                format!("DI状态读取失败 - 期望: {}, 实际: {}", expected, actual_state)
            }
        } else {
            format!("DI状态读取完成 - 当前状态: {}", actual_state)
        };

        info!("[{}] {}", self.executor_name(), message);

        // 构造原始测试结果
        let outcome = RawTestOutcome {
            channel_instance_id: instance.instance_id.clone(),
            sub_test_item: SubTestItem::HardPoint,
            success: is_success,
            raw_value_read: Some(actual_state.to_string()),
            eng_value_calculated: Some(format!("状态: {}", if actual_state { "ON" } else { "OFF" })),
            message: Some(message),
            timestamp,
            analog_reading_point: None,
        };

        debug!("[{}] DI状态读取完成 - 结果: {}", self.executor_name(), is_success);

        Ok(outcome)
    }

    fn item_type(&self) -> SubTestItem {
        SubTestItem::HardPoint
    }

    fn executor_name(&self) -> &'static str {
        "DIStateReadExecutor"
    }

    fn supports_definition(&self, definition: &ChannelPointDefinition) -> bool {
        matches!(definition.module_type, ModuleType::DI | ModuleType::DINone) &&
        matches!(definition.data_type, PointDataType::Bool)
    }
}

// 重新导出常用类型
pub use {
    ISpecificTestStepExecutor,
    AIHardPointPercentExecutor,
    AIAlarmTestExecutor,
    DIStateReadExecutor,
};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::infrastructure::plc::MockPlcService;
    use crate::models::{PointDataType, ModuleType};
    use std::collections::HashMap;

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

    /// 创建测试用的通道测试实例
    fn create_test_instance() -> ChannelTestInstance {
        ChannelTestInstance {
            instance_id: "test_instance_001".to_string(),
            definition_id: "test_ai_001".to_string(),
            test_batch_id: "test_batch_001".to_string(),
            overall_status: crate::models::OverallTestStatus::NotTested,
            current_step_details: None,
            error_message: None,
            start_time: None,
            last_updated_time: Utc::now(),
            final_test_time: None,
            total_test_duration_ms: None,
            sub_test_results: HashMap::new(),
            hardpoint_readings: None,
            manual_test_current_value_input: None,
            manual_test_current_value_output: None,
        }
    }

    #[tokio::test]
    async fn test_ai_hardpoint_percent_executor_success() {
        // 创建执行器
        let executor = AIHardPointPercentExecutor::new(0.5, 0.05, 100);
        
        // 创建测试数据
        let definition = create_test_ai_definition();
        let instance = create_test_instance();
        
        // 创建Mock PLC服务
        let mut mock_test_rig = MockPlcService::new();
        let mut mock_target = MockPlcService::new();
        
        // 设置Mock期望 - 测试台架写入50.0
        mock_test_rig.expect_write_f32()
            .with(mockall::predicate::eq("DB2.DBD0"), mockall::predicate::eq(50.0))
            .times(1)
            .returning(|_, _| Ok(()));
        
        // 设置Mock期望 - 被测点读取返回50.2（在容差范围内）
        mock_target.expect_read_f32()
            .with(mockall::predicate::eq("DB1.DBD0"))
            .times(1)
            .returning(|_| Ok(50.2));
        
        // 执行测试
        let result = executor.execute(
            &instance,
            &definition,
            Arc::new(mock_test_rig),
            Arc::new(mock_target),
        ).await;
        
        // 验证结果
        assert!(result.is_ok());
        let outcome = result.unwrap();
        assert!(outcome.success);
        assert_eq!(outcome.sub_test_item, SubTestItem::HardPoint);
        assert_eq!(outcome.channel_instance_id, instance.instance_id);
        assert!(outcome.analog_reading_point.is_some());
        
        let reading_point = outcome.analog_reading_point.unwrap();
        assert_eq!(reading_point.set_percentage, 0.5);
        assert_eq!(reading_point.set_value_eng, 50.0);
        assert_eq!(reading_point.actual_reading_raw, Some(50.2));
        assert_eq!(reading_point.status, SubTestStatus::Passed);
    }

    #[tokio::test]
    async fn test_ai_hardpoint_percent_executor_failure() {
        // 创建执行器
        let executor = AIHardPointPercentExecutor::new(0.5, 0.05, 100);
        
        // 创建测试数据
        let definition = create_test_ai_definition();
        let instance = create_test_instance();
        
        // 创建Mock PLC服务
        let mut mock_test_rig = MockPlcService::new();
        let mut mock_target = MockPlcService::new();
        
        // 设置Mock期望 - 测试台架写入50.0
        mock_test_rig.expect_write_f32()
            .with(mockall::predicate::eq("DB2.DBD0"), mockall::predicate::eq(50.0))
            .times(1)
            .returning(|_, _| Ok(()));
        
        // 设置Mock期望 - 被测点读取返回60.0（超出容差范围）
        mock_target.expect_read_f32()
            .with(mockall::predicate::eq("DB1.DBD0"))
            .times(1)
            .returning(|_| Ok(60.0));
        
        // 执行测试
        let result = executor.execute(
            &instance,
            &definition,
            Arc::new(mock_test_rig),
            Arc::new(mock_target),
        ).await;
        
        // 验证结果
        assert!(result.is_ok());
        let outcome = result.unwrap();
        assert!(!outcome.success); // 应该失败
        assert_eq!(outcome.sub_test_item, SubTestItem::HardPoint);
        
        let reading_point = outcome.analog_reading_point.unwrap();
        assert_eq!(reading_point.status, SubTestStatus::Failed);
    }

    #[tokio::test]
    async fn test_ai_alarm_test_executor_success() {
        // 创建高报警执行器
        let executor = AIAlarmTestExecutor::new(SubTestItem::HighAlarm, 100, 100);
        
        // 创建测试数据
        let definition = create_test_ai_definition();
        let instance = create_test_instance();
        
        // 创建Mock PLC服务
        let mut mock_test_rig = MockPlcService::new();
        let mut mock_target = MockPlcService::new();
        
        // 设置Mock期望 - 设定报警阈值
        mock_target.expect_write_f32()
            .with(mockall::predicate::eq("DB1.DBD16"), mockall::predicate::eq(80.0))
            .times(1)
            .returning(|_, _| Ok(()));
        
        // 设置Mock期望 - 设定测试台架触发值（80 + 5% = 85.0）
        mock_test_rig.expect_write_f32()
            .with(mockall::predicate::eq("DB2.DBD0"), mockall::predicate::eq(85.0))
            .times(1)
            .returning(|_, _| Ok(()));
        
        // 设置Mock期望 - 读取报警反馈状态（触发）
        mock_target.expect_read_bool()
            .with(mockall::predicate::eq("DB1.DBX8.2"))
            .times(1)
            .returning(|_| Ok(true));
        
        // 设置Mock期望 - 恢复正常值
        mock_test_rig.expect_write_f32()
            .with(mockall::predicate::eq("DB2.DBD0"), mockall::predicate::eq(50.0))
            .times(1)
            .returning(|_, _| Ok(()));
        
        // 执行测试
        let result = executor.execute(
            &instance,
            &definition,
            Arc::new(mock_test_rig),
            Arc::new(mock_target),
        ).await;
        
        // 验证结果
        assert!(result.is_ok());
        let outcome = result.unwrap();
        assert!(outcome.success);
        assert_eq!(outcome.sub_test_item, SubTestItem::HighAlarm);
        assert_eq!(outcome.raw_value_read, Some("true".to_string()));
    }

    #[tokio::test]
    async fn test_di_state_read_executor_success() {
        // 创建DI状态读取执行器
        let executor = DIStateReadExecutor::new(Some(true), 50);
        
        // 创建测试数据
        let definition = create_test_di_definition();
        let instance = create_test_instance();
        
        // 创建Mock PLC服务
        let mock_test_rig = MockPlcService::new();
        let mut mock_target = MockPlcService::new();
        
        // 设置Mock期望 - 读取DI状态
        mock_target.expect_read_bool()
            .with(mockall::predicate::eq("DB1.DBX0.0"))
            .times(1)
            .returning(|_| Ok(true));
        
        // 执行测试
        let result = executor.execute(
            &instance,
            &definition,
            Arc::new(mock_test_rig),
            Arc::new(mock_target),
        ).await;
        
        // 验证结果
        assert!(result.is_ok());
        let outcome = result.unwrap();
        assert!(outcome.success);
        assert_eq!(outcome.sub_test_item, SubTestItem::HardPoint);
        assert_eq!(outcome.raw_value_read, Some("true".to_string()));
        assert!(outcome.eng_value_calculated.as_ref().unwrap().contains("ON"));
    }

    #[test]
    fn test_ai_hardpoint_executor_supports_definition() {
        let executor = AIHardPointPercentExecutor::new(0.5, 0.05, 100);
        let ai_definition = create_test_ai_definition();
        let di_definition = create_test_di_definition();
        
        // AI定义应该被支持
        assert!(executor.supports_definition(&ai_definition));
        
        // DI定义不应该被支持
        assert!(!executor.supports_definition(&di_definition));
    }

    #[test]
    fn test_di_state_executor_supports_definition() {
        let executor = DIStateReadExecutor::new(None, 50);
        let ai_definition = create_test_ai_definition();
        let di_definition = create_test_di_definition();
        
        // DI定义应该被支持
        assert!(executor.supports_definition(&di_definition));
        
        // AI定义不应该被支持
        assert!(!executor.supports_definition(&ai_definition));
    }

    #[test]
    fn test_ai_alarm_executor_supports_definition() {
        let executor = AIAlarmTestExecutor::new(SubTestItem::HighAlarm, 100, 100);
        let ai_definition = create_test_ai_definition();
        let di_definition = create_test_di_definition();
        
        // AI定义应该被支持（有高报警配置）
        assert!(executor.supports_definition(&ai_definition));
        
        // DI定义不应该被支持
        assert!(!executor.supports_definition(&di_definition));
    }

    #[test]
    fn test_ai_hardpoint_executor_calculate_engineering_value() {
        let executor = AIHardPointPercentExecutor::new(0.5, 0.05, 100);
        let definition = create_test_ai_definition();
        
        // 测试50%点的计算
        let result = executor.calculate_engineering_value(&definition);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 50.0); // 0 + (100-0) * 0.5 = 50.0
        
        // 测试0%点
        let executor_0 = AIHardPointPercentExecutor::new(0.0, 0.05, 100);
        let result_0 = executor_0.calculate_engineering_value(&definition);
        assert!(result_0.is_ok());
        assert_eq!(result_0.unwrap(), 0.0);
        
        // 测试100%点
        let executor_100 = AIHardPointPercentExecutor::new(1.0, 0.05, 100);
        let result_100 = executor_100.calculate_engineering_value(&definition);
        assert!(result_100.is_ok());
        assert_eq!(result_100.unwrap(), 100.0);
    }

    #[test]
    fn test_ai_hardpoint_executor_tolerance_check() {
        let executor = AIHardPointPercentExecutor::new(0.5, 0.05, 100); // 5%容差
        
        // 测试在容差范围内
        assert!(executor.is_within_tolerance(50.0, 52.0, 100.0)); // 2%偏差，小于5%容差
        assert!(executor.is_within_tolerance(50.0, 48.0, 100.0)); // 2%偏差，小于5%容差
        
        // 测试超出容差范围
        assert!(!executor.is_within_tolerance(50.0, 57.0, 100.0)); // 7%偏差，大于5%容差
        assert!(!executor.is_within_tolerance(50.0, 43.0, 100.0)); // 7%偏差，大于5%容差
    }
} 