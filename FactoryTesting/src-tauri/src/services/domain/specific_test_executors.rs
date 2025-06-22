/// 特定测试步骤执行器
///
/// 包含各种具体的测试执行器实现，每个执行器负责一个原子的测试操作

use crate::models::{
    ChannelTestInstance, ChannelPointDefinition, RawTestOutcome, SubTestItem,
    AnalogReadingPoint, DigitalTestStep, ModuleType, SubTestStatus
};
use crate::services::infrastructure::plc::{ ModbusPlcService, plc_communication_service::PlcCommunicationService };
use crate::utils::error::{AppError, AppResult};
use async_trait::async_trait;
use chrono::Utc;
use log::{debug, info, warn, error};
use std::sync::Arc;
use std::collections::HashMap;
use std::time::Duration;
use tokio::time::sleep;
use uuid::Uuid;

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
        plc_service_test_rig: Arc<ModbusPlcService>,
        plc_service_target: Arc<ModbusPlcService>,
    ) -> AppResult<RawTestOutcome>;

    /// 返回此执行器处理的 SubTestItem 类型
    fn item_type(&self) -> SubTestItem;

    /// 返回执行器名称，用于日志记录
    fn executor_name(&self) -> &'static str;

    /// 检查是否支持指定的点位定义
    fn supports_definition(&self, definition: &ChannelPointDefinition) -> bool;
}

/// AI点硬点百分比测试执行器
/// 负责AI点的硬接线测试，包括0%, 25%, 50%, 75%, 100%的多点测试
pub struct AIHardPointPercentExecutor {
    /// 测试步骤执行器ID
    pub id: String,
}

impl AIHardPointPercentExecutor {
    /// 创建新的AI硬点测试执行器
    pub fn new() -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
        }
    }

    /// 获取测试PLC对应的AO通道地址
    fn get_test_rig_ao_address(&self, instance: &ChannelTestInstance) -> AppResult<String> {
        instance.test_plc_communication_address.as_ref()
            .ok_or_else(|| AppError::validation_error("测试实例未分配测试PLC通道地址"))
            .map(|addr| addr.clone())
    }

    /// 执行完整的AI硬点测试，包括5个百分比点
    async fn execute_complete_ai_hardpoint_test(
        &self,
        instance: &ChannelTestInstance,
        definition: &ChannelPointDefinition,
        test_rig_plc: Arc<ModbusPlcService>,
        target_plc: Arc<ModbusPlcService>,
    ) -> Result<RawTestOutcome, AppError> {
        let mut readings = Vec::new();

        let start_time = Utc::now();
        let test_rig_ao_address = self.get_test_rig_ao_address(instance)?;
        let target_ai_address = &definition.plc_communication_address;

        let range_lower = definition.range_low_limit.unwrap_or(0.0);
        let range_upper = definition.range_high_limit.unwrap_or(100.0);

        if range_upper <= range_lower {
            return Err(AppError::validation_error(format!("无效的量程范围: {} - {}", range_lower, range_upper)));
        }

        info!("🔧 AI硬点测试开始 - 测试PLC AO: {}, 被测PLC AI: {}, 量程: {}-{}",
              test_rig_ao_address, target_ai_address, range_lower, range_upper);

        let test_percentages = vec![0.0, 0.25, 0.5, 0.75, 1.0];
        let mut overall_success = true;

        for percentage in &test_percentages {
            let output_value = range_lower + (range_upper - range_lower) * percentage;

            info!("📝 写入 [{}]: {:.2}", test_rig_ao_address, output_value);
            test_rig_plc.write_float32(&test_rig_ao_address, output_value).await?;

            tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;

            let read_value = target_plc.read_float32(target_ai_address).await?;
            info!("📖 读取 [{}]: {:.2}", target_ai_address, read_value);

            let deviation = ((read_value - output_value) / (range_upper - range_lower) * 100.0).abs();
            let is_within_tolerance = deviation <= 5.0; // 5%偏差容限
            
            if !is_within_tolerance {
                overall_success = false;
            }

            let status_icon = if is_within_tolerance { "✅" } else { "❌" };
            info!("{} {}%: {:.2}", status_icon, percentage * 100.0, read_value);

            readings.push((output_value, read_value, deviation, is_within_tolerance));
        }

        let end_time = Utc::now();

        let (success_msg, mut outcome) = if overall_success {
            info!("✅ 结果: {} - 通过", definition.tag);
            let msg = format!("AI硬点测试成功: 所有{}个测试点偏差均在5%以内", readings.len());
            (msg.clone(), RawTestOutcome::success(instance.instance_id.clone(), SubTestItem::HardPoint))
        } else {
            info!("❌ 结果: {} - 失败", definition.tag);
            let failed_count = readings.iter().filter(|(_, _, _, success)| !*success).count();
            let msg = format!("AI硬点测试失败: {}个测试点偏差过大", failed_count);
            (msg.clone(), RawTestOutcome::failure(instance.instance_id.clone(), SubTestItem::HardPoint, msg))
        };
        
        outcome.message = Some(success_msg);
        outcome.start_time = start_time;
        outcome.end_time = end_time;

        let analog_readings: Vec<AnalogReadingPoint> = readings.iter().enumerate().map(|(i, (expected, actual, deviation, is_within_tolerance))| {
            AnalogReadingPoint {
                set_percentage: test_percentages[i],
                set_value_eng: *expected,
                expected_reading_raw: Some(*expected),
                actual_reading_raw: Some(*actual),
                actual_reading_eng: Some(*actual),
                status: if *is_within_tolerance { SubTestStatus::Passed } else { SubTestStatus::Failed },
                error_percentage: Some(*deviation),
            }
        }).collect();

        outcome.readings = Some(analog_readings.clone());

        outcome.test_result_0_percent = analog_readings.get(0).and_then(|r| r.actual_reading_eng.map(|v| v as f64));
        outcome.test_result_25_percent = analog_readings.get(1).and_then(|r| r.actual_reading_eng.map(|v| v as f64));
        outcome.test_result_50_percent = analog_readings.get(2).and_then(|r| r.actual_reading_eng.map(|v| v as f64));
        outcome.test_result_75_percent = analog_readings.get(3).and_then(|r| r.actual_reading_eng.map(|v| v as f64));
        outcome.test_result_100_percent = analog_readings.get(4).and_then(|r| r.actual_reading_eng.map(|v| v as f64));

        info!("🔄 测试完成，复位测试PLC AO [{}]: 0.0", test_rig_ao_address);
        if let Err(e) = test_rig_plc.write_float32(&test_rig_ao_address, 0.0).await {
            warn!("⚠️ 测试PLC AO复位失败: {}", e);
        }

        Ok(outcome)
    }
}

#[async_trait]
impl ISpecificTestStepExecutor for AIHardPointPercentExecutor {
    async fn execute(
        &self,
        instance: &ChannelTestInstance,
        definition: &ChannelPointDefinition,
        plc_service_test_rig: Arc<ModbusPlcService>,
        plc_service_target: Arc<ModbusPlcService>,
    ) -> AppResult<RawTestOutcome> {
        info!("🚀 开始测试: {} [{}]", definition.tag, instance.instance_id);
        let result = self.execute_complete_ai_hardpoint_test(instance, definition, plc_service_test_rig, plc_service_target).await?;
        Ok(result)
    }

    fn item_type(&self) -> SubTestItem {
        SubTestItem::HardPoint
    }

    fn executor_name(&self) -> &'static str {
        "AIHardPointPercentExecutor"
    }

    fn supports_definition(&self, definition: &ChannelPointDefinition) -> bool {
        matches!(definition.module_type, ModuleType::AI | ModuleType::AINone)
    }
}

/// AI报警测试执行器
///
/// 执行AI点某个报警项的测试（如设置高报触发条件，验证报警是否产生）
pub struct AIAlarmTestExecutor {
    /// 报警类型
    pub alarm_type: SubTestItem,
    /// 触发延时 (毫秒)
    pub trigger_delay_ms: u64,
    /// 复位延时 (毫秒)
    pub reset_delay_ms: u64,
}

impl AIAlarmTestExecutor {
    pub fn new(alarm_type: SubTestItem) -> Self {
        Self {
            alarm_type,
            trigger_delay_ms: 3000,
            reset_delay_ms: 3000,
        }
    }

    fn get_alarm_config(&self, definition: &ChannelPointDefinition) -> AppResult<(f32, String, String)> {
        let (set_address, feedback_address) = match self.alarm_type {
            SubTestItem::LowLowAlarm => (definition.low_low_alarm_set_address.as_ref(), definition.low_low_alarm_feedback_address.as_ref()),
            SubTestItem::LowAlarm => (definition.low_alarm_set_address.as_ref(), definition.low_alarm_feedback_address.as_ref()),
            SubTestItem::HighAlarm => (definition.high_alarm_set_address.as_ref(), definition.high_alarm_feedback_address.as_ref()),
            SubTestItem::HighHighAlarm => (definition.high_high_alarm_set_address.as_ref(), definition.high_high_alarm_feedback_address.as_ref()),
            _ => return Err(AppError::validation_error("不支持的报警测试类型")),
        };

        let set_value = match self.alarm_type {
            SubTestItem::LowLowAlarm => definition.low_low_alarm_value.unwrap_or(0.0) as f32,
            SubTestItem::LowAlarm => definition.low_alarm_value.unwrap_or(0.0) as f32,
            SubTestItem::HighAlarm => definition.high_alarm_value.unwrap_or(0.0) as f32,
            SubTestItem::HighHighAlarm => definition.high_high_alarm_value.unwrap_or(0.0) as f32,
            _ => 0.0,
        };

        if let (Some(set_addr), Some(fb_addr)) = (set_address, feedback_address) {
            Ok((set_value, set_addr.clone(), fb_addr.clone()))
        } else {
            Err(AppError::validation_error("报警测试地址未配置"))
        }
    }
}

#[async_trait]
impl ISpecificTestStepExecutor for AIAlarmTestExecutor {
    async fn execute(
        &self,
        instance: &ChannelTestInstance,
        definition: &ChannelPointDefinition,
        _plc_service_test_rig: Arc<ModbusPlcService>,
        plc_service_target: Arc<ModbusPlcService>,
    ) -> AppResult<RawTestOutcome> {
        let (alarm_set_value, set_address, feedback_address) = self.get_alarm_config(definition)?;
        let start_time = Utc::now();

        info!("📝 写入 [{}]: {:.3}", set_address, alarm_set_value);
        plc_service_target.write_float32(&set_address, alarm_set_value).await?;
        tokio::time::sleep(tokio::time::Duration::from_millis(self.trigger_delay_ms)).await;

        info!("📖 读取报警反馈 [{}]", feedback_address);
        let alarm_active = plc_service_target.read_bool(&feedback_address).await?;

        let safe_value = match self.alarm_type {
            SubTestItem::LowLowAlarm | SubTestItem::LowAlarm => alarm_set_value + 10.0,
            SubTestItem::HighAlarm | SubTestItem::HighHighAlarm => alarm_set_value - 10.0,
            _ => alarm_set_value,
        };

        info!("📝 写入安全值复位报警 [{}]: {:.3}", set_address, safe_value);
        plc_service_target.write_float32(&set_address, safe_value).await?;
        tokio::time::sleep(tokio::time::Duration::from_millis(self.reset_delay_ms)).await;
        
        info!("📖 确认报警已复位 [{}]", feedback_address);
        let alarm_reset = !plc_service_target.read_bool(&feedback_address).await?;

        let success = alarm_active && alarm_reset;
        let end_time = Utc::now();
        let status_icon = if success { "✅" } else { "❌" };
        let result_msg = format!("{} 结果: {}", status_icon, if success { "通过" } else { "失败" });
        info!("{}", result_msg);
        
        let mut outcome = if success {
            RawTestOutcome::success(instance.instance_id.clone(), self.item_type())
        } else {
            RawTestOutcome::failure(instance.instance_id.clone(), self.item_type(), "报警测试失败".to_string())
        };
        outcome.message = Some(result_msg);
        outcome.start_time = start_time;
        outcome.end_time = end_time;
        Ok(outcome)
    }

    fn item_type(&self) -> SubTestItem {
        self.alarm_type.clone()
    }

    fn executor_name(&self) -> &'static str {
        "AIAlarmTestExecutor"
    }

    fn supports_definition(&self, definition: &ChannelPointDefinition) -> bool {
        matches!(definition.module_type, ModuleType::AI | ModuleType::AINone)
    }
}

/// DI硬点测试执行器
pub struct DIHardPointTestExecutor {
    pub step_interval_ms: u64,
}

impl DIHardPointTestExecutor {
    pub fn new(step_interval_ms: u64) -> Self { Self { step_interval_ms } }

    fn get_test_rig_do_address(&self, instance: &ChannelTestInstance) -> AppResult<String> {
        instance.test_plc_communication_address.as_ref()
            .ok_or_else(|| AppError::validation_error("测试实例未分配测试PLC通道地址"))
            .map(|addr| addr.clone())
    }
}

#[async_trait]
impl ISpecificTestStepExecutor for DIHardPointTestExecutor {
    async fn execute(
        &self,
        instance: &ChannelTestInstance,
        definition: &ChannelPointDefinition,
        plc_service_test_rig: Arc<ModbusPlcService>,
        plc_service_target: Arc<ModbusPlcService>,
    ) -> AppResult<RawTestOutcome> {
        let start_time = Utc::now();
        let test_rig_do_address = self.get_test_rig_do_address(instance)?;
        let target_di_address = &definition.plc_communication_address;
        info!("🔧 DI硬点测试开始 - 测试PLC DO: {}, 被测PLC DI: {}", test_rig_do_address, target_di_address);

        let mut digital_steps = Vec::new();
        
        plc_service_test_rig.write_bool(&test_rig_do_address, false).await?;
        tokio::time::sleep(tokio::time::Duration::from_millis(self.step_interval_ms)).await;
        let di_state_1 = plc_service_target.read_bool(target_di_address).await?;
        let step1_status = if !di_state_1 { SubTestStatus::Passed } else { SubTestStatus::Failed };
        digital_steps.push(DigitalTestStep { step_number: 1, step_description: "测试PLC DO输出低电平，检查被测PLC DI显示断开".to_string(), set_value: false, expected_reading: false, actual_reading: di_state_1, status: step1_status.clone(), timestamp: Utc::now() });
        if di_state_1 { return Ok(RawTestOutcome::failure(instance.instance_id.clone(), SubTestItem::HardPoint, "DI硬点测试失败: DO低电平时DI应为false，实际为true".to_string())); }

        plc_service_test_rig.write_bool(&test_rig_do_address, true).await?;
        tokio::time::sleep(tokio::time::Duration::from_millis(self.step_interval_ms)).await;
        let di_state_2 = plc_service_target.read_bool(target_di_address).await?;
        let step2_status = if di_state_2 { SubTestStatus::Passed } else { SubTestStatus::Failed };
        digital_steps.push(DigitalTestStep { step_number: 2, step_description: "测试PLC DO输出高电平，检查被测PLC DI显示接通".to_string(), set_value: true, expected_reading: true, actual_reading: di_state_2, status: step2_status.clone(), timestamp: Utc::now() });
        if !di_state_2 { return Ok(RawTestOutcome::failure(instance.instance_id.clone(), SubTestItem::HardPoint, "DI硬点测试失败: DO高电平时DI应为true，实际为false".to_string())); }

        plc_service_test_rig.write_bool(&test_rig_do_address, false).await?;
        tokio::time::sleep(tokio::time::Duration::from_millis(self.step_interval_ms)).await;
        let di_state_3 = plc_service_target.read_bool(target_di_address).await?;
        let step3_status = if !di_state_3 { SubTestStatus::Passed } else { SubTestStatus::Failed };
        digital_steps.push(DigitalTestStep { step_number: 3, step_description: "测试PLC DO复位低电平，检查被测PLC DI显示断开".to_string(), set_value: false, expected_reading: false, actual_reading: di_state_3, status: step3_status.clone(), timestamp: Utc::now() });
        if di_state_3 { return Ok(RawTestOutcome::failure(instance.instance_id.clone(), SubTestItem::HardPoint, "DI硬点测试失败: 复位后DI应为false，实际为true".to_string())); }

        let end_time = Utc::now();
        let mut outcome = RawTestOutcome::success(instance.instance_id.clone(), SubTestItem::HardPoint);
        outcome.message = Some("DI硬点测试成功: 低→高→低电平切换，DI状态正确响应".to_string());
        outcome.start_time = start_time;
        outcome.end_time = end_time;
        outcome.digital_steps = Some(digital_steps);
        Ok(outcome)
    }

    fn item_type(&self) -> SubTestItem { SubTestItem::HardPoint }
    fn executor_name(&self) -> &'static str { "DIHardPointTestExecutor" }
    fn supports_definition(&self, definition: &ChannelPointDefinition) -> bool { matches!(definition.module_type, ModuleType::DI | ModuleType::DINone) }
}

/// DO硬点测试执行器
pub struct DOHardPointTestExecutor {
    pub step_interval_ms: u64,
}

impl DOHardPointTestExecutor {
    pub fn new(step_interval_ms: u64) -> Self { Self { step_interval_ms } }

    fn get_test_rig_di_address(&self, instance: &ChannelTestInstance) -> AppResult<String> {
        instance.test_plc_communication_address.as_ref()
            .ok_or_else(|| AppError::validation_error("测试实例未分配测试PLC通道地址"))
            .map(|addr| addr.clone())
    }
}

#[async_trait]
impl ISpecificTestStepExecutor for DOHardPointTestExecutor {
    async fn execute(
        &self,
        instance: &ChannelTestInstance,
        definition: &ChannelPointDefinition,
        plc_service_test_rig: Arc<ModbusPlcService>,
        plc_service_target: Arc<ModbusPlcService>,
    ) -> AppResult<RawTestOutcome> {
        let test_rig_di_address = self.get_test_rig_di_address(instance)?;
        let target_do_address = &definition.plc_communication_address;
        
        plc_service_target.write_bool(target_do_address, false).await?;
        tokio::time::sleep(tokio::time::Duration::from_millis(self.step_interval_ms)).await;
        let di_state_1 = plc_service_test_rig.read_bool(&test_rig_di_address).await?;
        if di_state_1 { return Ok(RawTestOutcome::failure(instance.instance_id.clone(), SubTestItem::HardPoint, "DO硬点测试失败: DO低电平时测试DI应为false，实际为true".to_string())); }

        plc_service_target.write_bool(target_do_address, true).await?;
        tokio::time::sleep(tokio::time::Duration::from_millis(self.step_interval_ms)).await;
        let di_state_2 = plc_service_test_rig.read_bool(&test_rig_di_address).await?;
        if !di_state_2 { return Ok(RawTestOutcome::failure(instance.instance_id.clone(), SubTestItem::HardPoint, "DO硬点测试失败: DO高电平时测试DI应为true，实际为false".to_string())); }
        
        plc_service_target.write_bool(target_do_address, false).await?;
        tokio::time::sleep(tokio::time::Duration::from_millis(self.step_interval_ms)).await;
        let di_state_3 = plc_service_test_rig.read_bool(&test_rig_di_address).await?;
        if di_state_3 { return Ok(RawTestOutcome::failure(instance.instance_id.clone(), SubTestItem::HardPoint, "DO硬点测试失败: 复位后测试DI应为false，实际为true".to_string())); }

        let mut outcome = RawTestOutcome::success(instance.instance_id.clone(), SubTestItem::HardPoint);
        outcome.message = Some("DO硬点测试成功".to_string());
        Ok(outcome)
    }

    fn item_type(&self) -> SubTestItem { SubTestItem::HardPoint }
    fn executor_name(&self) -> &'static str { "DOHardPointTestExecutor" }
    fn supports_definition(&self, definition: &ChannelPointDefinition) -> bool { matches!(definition.module_type, ModuleType::DO | ModuleType::DONone) }
}

/// AO硬点测试执行器
pub struct AOHardPointTestExecutor {
    pub step_interval_ms: u64,
}

impl AOHardPointTestExecutor {
    pub fn new(step_interval_ms: u64) -> Self { Self { step_interval_ms } }

    fn get_test_rig_ai_address(&self, instance: &ChannelTestInstance) -> AppResult<String> {
        instance.test_plc_communication_address.as_ref()
            .ok_or_else(|| AppError::validation_error("测试实例未分配测试PLC通道地址"))
            .map(|addr| addr.clone())
    }
}

#[async_trait]
impl ISpecificTestStepExecutor for AOHardPointTestExecutor {
    async fn execute(
        &self,
        instance: &ChannelTestInstance,
        definition: &ChannelPointDefinition,
        plc_service_test_rig: Arc<ModbusPlcService>,
        plc_service_target: Arc<ModbusPlcService>,
    ) -> AppResult<RawTestOutcome> {
        let start_time = Utc::now();
        let test_rig_ai_address = self.get_test_rig_ai_address(instance)?;
        let target_ao_address = &definition.plc_communication_address;

        let range_lower = definition.range_low_limit.unwrap_or(0.0);
        let range_upper = definition.range_high_limit.unwrap_or(100.0);

        if range_upper <= range_lower {
            return Err(AppError::validation_error(format!("无效的量程范围: {} - {}", range_lower, range_upper)));
        }

        info!("🔧 AO硬点测试开始 - 被测PLC AO: {}, 测试PLC AI: {}, 量程: {}-{}",
              target_ao_address, test_rig_ai_address, range_lower, range_upper);

        let test_percentages = vec![0.0, 0.25, 0.5, 0.75, 1.0];
        let mut readings = Vec::new();
        let mut overall_success = true;

        for percentage in &test_percentages {
            let output_value = range_lower + (range_upper - range_lower) * *percentage;

            plc_service_target.write_float32(target_ao_address, output_value).await?;
            tokio::time::sleep(tokio::time::Duration::from_millis(self.step_interval_ms)).await;
            let read_value = plc_service_test_rig.read_float32(&test_rig_ai_address).await?;

            let deviation = ((read_value - output_value) / (range_upper - range_lower) * 100.0).abs();
            let is_within_tolerance = deviation <= 5.0;

            if !is_within_tolerance {
                overall_success = false;
            }

            readings.push((output_value, read_value, deviation, is_within_tolerance));
        }

        let end_time = Utc::now();
        
        let (success_msg, mut outcome) = if overall_success {
            let msg = format!("AO硬点测试成功: 所有测试点偏差均在5%以内");
            (msg.clone(), RawTestOutcome::success(instance.instance_id.clone(), SubTestItem::HardPoint))
        } else {
            let failed_count = readings.iter().filter(|(_, _, _, success)| !*success).count();
            let msg = format!("AO硬点测试失败: {}个测试点偏差过大", failed_count);
            (msg.clone(), RawTestOutcome::failure(instance.instance_id.clone(), SubTestItem::HardPoint, msg))
        };
        
        outcome.message = Some(success_msg);
        outcome.start_time = start_time;
        outcome.end_time = end_time;

        let analog_readings: Vec<AnalogReadingPoint> = readings.iter().enumerate().map(|(i, (expected, actual, deviation, is_within_tolerance))| {
            AnalogReadingPoint {
                set_percentage: test_percentages[i],
                set_value_eng: *expected,
                expected_reading_raw: Some(*expected),
                actual_reading_raw: Some(*actual),
                actual_reading_eng: Some(*actual),
                status: if *is_within_tolerance { SubTestStatus::Passed } else { SubTestStatus::Failed },
                error_percentage: Some(*deviation),
            }
        }).collect();

        outcome.readings = Some(analog_readings.clone());
        outcome.test_result_0_percent = analog_readings.get(0).and_then(|r| r.actual_reading_eng.map(|v| v as f64));
        outcome.test_result_25_percent = analog_readings.get(1).and_then(|r| r.actual_reading_eng.map(|v| v as f64));
        outcome.test_result_50_percent = analog_readings.get(2).and_then(|r| r.actual_reading_eng.map(|v| v as f64));
        outcome.test_result_75_percent = analog_readings.get(3).and_then(|r| r.actual_reading_eng.map(|v| v as f64));
        outcome.test_result_100_percent = analog_readings.get(4).and_then(|r| r.actual_reading_eng.map(|v| v as f64));

        let reset_value = range_lower;
        if let Err(e) = plc_service_target.write_float32(target_ao_address, reset_value).await {
            warn!("⚠️ 被测PLC AO复位失败: {}", e);
        }

        Ok(outcome)
    }

    fn item_type(&self) -> SubTestItem { SubTestItem::HardPoint }
    fn executor_name(&self) -> &'static str { "AOHardPointTestExecutor" }
    fn supports_definition(&self, definition: &ChannelPointDefinition) -> bool { matches!(definition.module_type, ModuleType::AO | ModuleType::AONone) }
}