/// 特定测试步骤执行器
///
/// 包含各种具体的测试执行器实现，每个执行器负责一个原子的测试操作

use crate::models::{
    ChannelTestInstance, ChannelPointDefinition, RawTestOutcome, SubTestItem,
    AnalogReadingPoint, DigitalTestStep, ModuleType, SubTestStatus, PointDataType
};
use crate::services::infrastructure::IPlcCommunicationService;
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
/// 负责AI点的硬接线测试，包括0%, 25%, 50%, 75%, 100%的多点测试
pub struct AIHardPointPercentExecutor {
    /// 测试步骤执行器ID
    pub id: String,
}

impl AIHardPointPercentExecutor {
    pub fn new() -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
        }
    }

    /// 获取通道对应的真实测试台架地址
    /// 从通道实例中获取已分配的测试PLC通道地址
    fn get_test_rig_address_for_channel(&self, instance: &ChannelTestInstance) -> AppResult<String> {
        instance.test_plc_communication_address.as_ref()
            .ok_or_else(|| AppError::validation_error("测试实例未分配测试PLC通道地址"))
            .map(|addr| addr.clone())
    }

    /// 执行AI点的完整硬点测试流程
    /// 包括多点测试、线性度检查、报警功能验证等
    async fn execute_complete_ai_hardpoint_test(
        &self,
        instance: &ChannelTestInstance,
        definition: &ChannelPointDefinition,
        test_rig_plc: Arc<dyn IPlcCommunicationService>,
        target_plc: Arc<dyn IPlcCommunicationService>,
    ) -> Result<RawTestOutcome, AppError> {
        let mut readings = Vec::new();
        let test_percentages = vec![0.0, 0.25, 0.5, 0.75, 1.0];

        let is_ai_test = matches!(definition.module_type, ModuleType::AI | ModuleType::AINone);
        let test_type = if is_ai_test { "AI硬点测试" } else { "AO硬点测试" };
        info!("🔧 开始{}: {}", test_type, instance.instance_id);

        // 计算量程信息
        let range_lower = definition.range_low_limit.unwrap_or(0.0);
        let range_upper = definition.range_high_limit.unwrap_or(100.0);
        let range_span = range_upper - range_lower;

        if range_span <= 0.0 {
            return Ok(RawTestOutcome::failure(
                instance.instance_id.clone(),
                SubTestItem::HardPoint,
                "AI点量程配置无效".to_string(),
            ));
        }

        // 执行多点测试
        for percentage in test_percentages {
            let eng_value = range_lower + (range_span * percentage);

            // 获取真实的测试台架地址
            let test_rig_address = self.get_test_rig_address_for_channel(instance)?;

            // 设置测试台架输出值(直接输出0-100因为在测试PLC中直接设定了工程量为0-100)
            let test_rig_output_value = percentage * 100.0;
            info!("变量:{}, 写[{}]={:.2}", definition.tag, test_rig_address, test_rig_output_value);
            test_rig_plc.write_float32(&test_rig_address, test_rig_output_value).await
                .map_err(|e| AppError::plc_communication_error(format!("设置测试台架输出失败: {}", e)))?;

                // 等待信号稳定时间 - 调整为2秒
                tokio::time::sleep(tokio::time::Duration::from_millis(2000)).await;

                // 读取被测PLC的实际值
                let actual_raw = target_plc.read_float32(&definition.plc_communication_address).await
                    .map_err(|e| AppError::plc_communication_error(format!("读取被测PLC值失败: {}", e)))?;
                info!("变量:{}, 读[{}]={:.2}", definition.tag, definition.plc_communication_address, actual_raw);

                // 计算误差
                let error_percentage = if eng_value != 0.0 {
                    Some(((actual_raw - eng_value) / eng_value * 100.0).abs())
                } else {
                    Some(actual_raw.abs())
                };

                // 判断测试状态（误差容忍度2%）
                let test_status = if error_percentage.unwrap_or(100.0) <= 2.0 {
                    SubTestStatus::Passed
                } else {
                    SubTestStatus::Failed
                };

                let reading = AnalogReadingPoint {
                    set_percentage: percentage,
                    set_value_eng: eng_value,
                    expected_reading_raw: Some(eng_value),
                    actual_reading_raw: Some(actual_raw),
                    actual_reading_eng: Some(actual_raw),
                    status: test_status.clone(),
                    error_percentage,
                };

                readings.push(reading);

                // 🔧 精简日志：只显示结果，不显示详细过程
                let status_icon = if test_status == SubTestStatus::Passed { "✅" } else { "❌" };
                debug!("{} {}%: {:.2}", status_icon, percentage * 100.0, actual_raw);

                // 如果任意点测试失败，继续完成所有测试点，但标记为失败
                // 不要立即返回，而是继续测试以收集完整的过程数据
                if test_status == SubTestStatus::Failed {
                    // 标记为失败，但继续测试
                }
        }

        // 检查是否有任何测试点失败
        let has_failed_tests = readings.iter().any(|r| r.status == SubTestStatus::Failed);
        let overall_success = !has_failed_tests;

        // 检查线性度（可选的高级检查）
        let linearity_check = self.check_linearity(&readings);

        let status_msg = if overall_success {
            "AI硬点5点测试全部通过"
        } else {
            "AI硬点测试部分失败"
        };

        // 🔧 精简日志：只保留最终结果
        debug!("✅ 结果: {} - {}",
            definition.tag,
            if overall_success { "通过" } else { "失败" });

        // 🔄 测试完成后复位测试PLC输出为0%
        let test_rig_address = self.get_test_rig_address_for_channel(instance)?;
        debug!("🔄 测试完成，复位测试PLC [{}]: 0.00", test_rig_address);
        if let Err(e) = test_rig_plc.write_float32(&test_rig_address, 0.0).await {
            // 复位失败不影响测试结果，只记录警告
            log::warn!("⚠️ 测试PLC复位失败: {}", e);
        } else {
            info!("✅ 测试PLC已复位为0%");
        }

        // 提取百分比测试结果 - 存储实际工程量 (转换f32到f64)
        // 🔧 关键修复：无论测试成功还是失败，都要保存过程数据
        let test_result_0_percent = readings.get(0).and_then(|r| r.actual_reading_eng.map(|v| v as f64));
        let test_result_25_percent = readings.get(1).and_then(|r| r.actual_reading_eng.map(|v| v as f64));
        let test_result_50_percent = readings.get(2).and_then(|r| r.actual_reading_eng.map(|v| v as f64));
        let test_result_75_percent = readings.get(3).and_then(|r| r.actual_reading_eng.map(|v| v as f64));
        let test_result_100_percent = readings.get(4).and_then(|r| r.actual_reading_eng.map(|v| v as f64));

        // 返回测试结果（成功或失败都包含完整的过程数据）
        Ok(RawTestOutcome {
            channel_instance_id: instance.instance_id.clone(),
            sub_test_item: SubTestItem::HardPoint,
            success: overall_success,
            raw_value_read: Some("多点测试".to_string()),
            eng_value_calculated: Some(format!("{:.2}-{:.2}", range_lower, range_upper)),
            message: Some(status_msg.to_string()),
            start_time: Utc::now(),
            end_time: Utc::now(),
            readings: Some(readings),
            digital_steps: None,
            test_result_0_percent,
            test_result_25_percent,
            test_result_50_percent,
            test_result_75_percent,
            test_result_100_percent,
            details: HashMap::new(),
        })
    }

    /// 检查线性度
    fn check_linearity(&self, readings: &[AnalogReadingPoint]) -> bool {
        if readings.len() < 3 {
            return true; // 数据点太少，无法检查线性度
        }

        // 简单的线性度检查：计算R²
        let n = readings.len() as f32;
        let sum_x: f32 = readings.iter().map(|r| r.set_percentage).sum();
        let sum_y: f32 = readings.iter().map(|r| r.actual_reading_raw.unwrap_or(0.0)).sum();
        let sum_xy: f32 = readings.iter().map(|r| r.set_percentage * r.actual_reading_raw.unwrap_or(0.0)).sum();
        let sum_x2: f32 = readings.iter().map(|r| r.set_percentage * r.set_percentage).sum();

        let r_squared = if n * sum_x2 - sum_x * sum_x != 0.0 {
            let correlation = (n * sum_xy - sum_x * sum_y) /
                ((n * sum_x2 - sum_x * sum_x).sqrt() * (n * sum_y.powi(2) - sum_y * sum_y).sqrt());
            correlation * correlation
        } else {
            0.0
        };

        r_squared >= 0.95 // 线性度要求R²≥0.95
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
        // 🔧 精简日志：移除详细调试信息
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
        // AI/AO点位都支持硬点测试，不再依赖虚拟的test_rig_plc_address字段
        // 真实的测试台架地址将在执行时通过通道分配服务获取
        matches!(definition.module_type, ModuleType::AI | ModuleType::AINone | ModuleType::AO | ModuleType::AONone)
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
    /// 创建新的AI报警测试执行器
    pub fn new(alarm_type: SubTestItem, trigger_delay_ms: u64, reset_delay_ms: u64) -> Self {
        Self {
            alarm_type,
            trigger_delay_ms,
            reset_delay_ms,
        }
    }

    /// 获取报警设定值和反馈地址
    fn get_alarm_config(&self, definition: &ChannelPointDefinition) -> AppResult<(f32, String, String)> {
        match self.alarm_type {
            SubTestItem::LowLowAlarm => {
                let set_value = definition.sll_set_value.ok_or_else(||
                    AppError::validation_error("未配置低低报设定值"))?;
                let set_address = definition.sll_set_point_address.as_ref().ok_or_else(||
                    AppError::validation_error("未配置低低报设定地址"))?;
                let feedback_address = definition.sll_feedback_address.as_ref().ok_or_else(||
                    AppError::validation_error("未配置低低报反馈地址"))?;
                Ok((set_value, set_address.clone(), feedback_address.clone()))
            },
            SubTestItem::LowAlarm => {
                let set_value = definition.sl_set_value.ok_or_else(||
                    AppError::validation_error("未配置低报设定值"))?;
                let set_address = definition.sl_set_point_address.as_ref().ok_or_else(||
                    AppError::validation_error("未配置低报设定地址"))?;
                let feedback_address = definition.sl_feedback_address.as_ref().ok_or_else(||
                    AppError::validation_error("未配置低报反馈地址"))?;
                Ok((set_value, set_address.clone(), feedback_address.clone()))
            },
            SubTestItem::HighAlarm => {
                let set_value = definition.sh_set_value.ok_or_else(||
                    AppError::validation_error("未配置高报设定值"))?;
                let set_address = definition.sh_set_point_address.as_ref().ok_or_else(||
                    AppError::validation_error("未配置高报设定地址"))?;
                let feedback_address = definition.sh_feedback_address.as_ref().ok_or_else(||
                    AppError::validation_error("未配置高报反馈地址"))?;
                Ok((set_value, set_address.clone(), feedback_address.clone()))
            },
            SubTestItem::HighHighAlarm => {
                let set_value = definition.shh_set_value.ok_or_else(||
                    AppError::validation_error("未配置高高报设定值"))?;
                let set_address = definition.shh_set_point_address.as_ref().ok_or_else(||
                    AppError::validation_error("未配置高高报设定地址"))?;
                let feedback_address = definition.shh_feedback_address.as_ref().ok_or_else(||
                    AppError::validation_error("未配置高高报反馈地址"))?;
                Ok((set_value, set_address.clone(), feedback_address.clone()))
            },
            _ => Err(AppError::validation_error(
                format!("不支持的报警类型: {:?}", self.alarm_type)
            ))
        }
    }
}

#[async_trait]
impl ISpecificTestStepExecutor for AIAlarmTestExecutor {
    async fn execute(
        &self,
        instance: &ChannelTestInstance,
        definition: &ChannelPointDefinition,
        _plc_service_test_rig: Arc<dyn IPlcCommunicationService>,
        plc_service_target: Arc<dyn IPlcCommunicationService>,
    ) -> AppResult<RawTestOutcome> {
        // 🔧 精简日志：移除详细调试信息
        let (alarm_set_value, set_address, feedback_address) = self.get_alarm_config(definition)?;
        let start_time = Utc::now();

        // 步骤1: 设置报警触发值
        info!("📝 写入 [{}]: {:.3}", set_address, alarm_set_value);

        plc_service_target.write_float32(&set_address, alarm_set_value).await?;

        // 步骤2: 等待报警触发 - 统一设置为3秒
        tokio::time::sleep(tokio::time::Duration::from_millis(3000)).await;

        // 步骤3: 读取报警反馈状态
        info!("📖 读取报警反馈 [{}]", feedback_address);
        let alarm_active = plc_service_target.read_bool(&feedback_address).await?;

        // 步骤4: 复位报警（设置安全值）
        let safe_value = match self.alarm_type {
            SubTestItem::LowLowAlarm | SubTestItem::LowAlarm => {
                // 对于低报，设置一个高于报警值的安全值
                alarm_set_value + 10.0
            },
            SubTestItem::HighAlarm | SubTestItem::HighHighAlarm => {
                // 对于高报，设置一个低于报警值的安全值
                alarm_set_value - 10.0
            },
            _ => alarm_set_value
        };

        info!("📝 写入安全值复位报警 [{}]: {:.3}",
              set_address, safe_value);
        plc_service_target.write_float32(&set_address, safe_value).await?;

        // 步骤5: 等待报警复位 - 统一设置为3秒
        tokio::time::sleep(tokio::time::Duration::from_millis(3000)).await;

        // 步骤6: 确认报警已复位
        info!("📖 读取报警复位状态 [{}]", feedback_address);
        let alarm_reset = !plc_service_target.read_bool(&feedback_address).await?;

        let end_time = Utc::now();

        // 判断测试结果
        let is_success = alarm_active && alarm_reset;
        let status_icon = if is_success { "✅" } else { "❌" };
        let message = format!("{} 报警测试 {:?}: 触发值={:.3}, 激活={}, 复位={} - {}",
                   status_icon, self.alarm_type, alarm_set_value, alarm_active, alarm_reset,
                   if is_success { "通过" } else { "失败" });

        info!("{}", message);

        // 构造测试结果
        let mut outcome = RawTestOutcome::new(
            instance.instance_id.clone(),
            self.alarm_type.clone(),
            is_success,
        );

        outcome.message = Some(message);
        outcome.start_time = start_time;
        outcome.end_time = end_time;
        outcome.raw_value_read = Some(if alarm_active { "1" } else { "0" }.to_string());
        outcome.eng_value_calculated = Some(alarm_set_value.to_string());

        Ok(outcome)
    }

    fn item_type(&self) -> SubTestItem {
        self.alarm_type.clone()
    }

    fn executor_name(&self) -> &'static str {
        "AIAlarmTestExecutor"
    }

    fn supports_definition(&self, definition: &ChannelPointDefinition) -> bool {
        matches!(definition.module_type, ModuleType::AI | ModuleType::AINone) &&
        match self.alarm_type {
            SubTestItem::LowLowAlarm => definition.sll_set_point_address.is_some(),
            SubTestItem::LowAlarm => definition.sl_set_point_address.is_some(),
            SubTestItem::HighAlarm => definition.sh_set_point_address.is_some(),
            SubTestItem::HighHighAlarm => definition.shh_set_point_address.is_some(),
            _ => false,
        }
    }
}

/// DI硬点测试执行器
///
/// 执行DI点的完整硬点测试：测试PLC的DO通道输出 → 被测PLC的DI通道检测
/// 测试步骤：
/// 1. 测试PLC DO通道输出低电平 → 检查被测PLC DI通道显示"断开"
/// 2. 等待信号稳定
/// 3. 测试PLC DO通道输出高电平 → 检查被测PLC DI通道显示"接通"
/// 4. 等待信号稳定
/// 5. 测试PLC DO通道输出低电平 → 检查被测PLC DI通道显示"断开"
pub struct DIHardPointTestExecutor {
    /// 测试步骤间隔时间 (毫秒)
    pub step_interval_ms: u64,
}

impl DIHardPointTestExecutor {
    /// 创建新的DI硬点测试执行器
    pub fn new(step_interval_ms: u64) -> Self {
        Self {
            step_interval_ms,
        }
    }

    /// 获取测试PLC对应的DO通道地址
    /// 从通道实例中获取已分配的测试PLC通道地址
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
        plc_service_test_rig: Arc<dyn IPlcCommunicationService>,
        plc_service_target: Arc<dyn IPlcCommunicationService>,
    ) -> AppResult<RawTestOutcome> {
        debug!("🔧 DI硬点测试开始 - 测试PLC DO: {}, 被测PLC DI: {}",
               self.get_test_rig_do_address(instance)?, &definition.plc_communication_address);

        let start_time = Utc::now();
        let test_rig_do_address = self.get_test_rig_do_address(instance)?;
        let target_di_address = &definition.plc_communication_address;

        // 创建数字量测试步骤记录
        let mut digital_steps = Vec::new();

        // 步骤1: 测试PLC DO输出低电平
        info!("变量:{}, 写[{}]=false", definition.tag, test_rig_do_address);
        plc_service_test_rig.write_bool(&test_rig_do_address, false).await
            .map_err(|e| AppError::plc_communication_error(format!("设置测试PLC DO低电平失败: {}", e)))?;

        // 等待信号稳定
        tokio::time::sleep(tokio::time::Duration::from_millis(self.step_interval_ms)).await;

        // 步骤2: 检查被测PLC DI是否显示"断开"
        let di_state_1 = plc_service_target.read_bool(target_di_address).await
            .map_err(|e| AppError::plc_communication_error(format!("读取被测PLC DI状态失败: {}", e)))?;
        info!("变量:{}, 读[{}]={}", definition.tag, target_di_address, di_state_1);

        // 记录步骤1结果
        let step1_status = if di_state_1 {
            SubTestStatus::Failed
        } else {
            SubTestStatus::Passed
        };
        digital_steps.push(DigitalTestStep {
            step_number: 1,
            step_description: "测试PLC DO输出低电平，检查被测PLC DI显示断开".to_string(),
            set_value: false,
            expected_reading: false,
            actual_reading: di_state_1,
            status: step1_status.clone(),
            timestamp: Utc::now(),
        });

        if di_state_1 {
            let error_msg = format!("❌ DI硬点测试失败: DO低电平时DI应为false，实际为true");
            info!("❌ 结果: {} - 失败", definition.tag);
            let mut outcome = RawTestOutcome::failure(
                instance.instance_id.clone(),
                SubTestItem::HardPoint,
                error_msg,
            );
            outcome.digital_steps = Some(digital_steps);
            return Ok(outcome);
        }
        debug!("✅ 低电平: {}", di_state_1);

        // 步骤3: 测试PLC DO输出高电平
        info!("变量:{}, 写[{}]=true", definition.tag, test_rig_do_address);
        plc_service_test_rig.write_bool(&test_rig_do_address, true).await
            .map_err(|e| AppError::plc_communication_error(format!("设置测试PLC DO高电平失败: {}", e)))?;

        // 等待信号稳定
        tokio::time::sleep(tokio::time::Duration::from_millis(self.step_interval_ms)).await;

        // 步骤4: 检查被测PLC DI是否显示"接通"
        let di_state_2 = plc_service_target.read_bool(target_di_address).await
            .map_err(|e| AppError::plc_communication_error(format!("读取被测PLC DI状态失败: {}", e)))?;

        // 记录步骤2结果
        let step2_status = if !di_state_2 {
            SubTestStatus::Failed
        } else {
            SubTestStatus::Passed
        };
        digital_steps.push(DigitalTestStep {
            step_number: 2,
            step_description: "测试PLC DO输出高电平，检查被测PLC DI显示接通".to_string(),
            set_value: true,
            expected_reading: true,
            actual_reading: di_state_2,
            status: step2_status.clone(),
            timestamp: Utc::now(),
        });

        if !di_state_2 {
            let error_msg = format!("❌ DI硬点测试失败: DO高电平时DI应为true，实际为false");
            info!("❌ 结果: {} - 失败", definition.tag);
            let mut outcome = RawTestOutcome::failure(
                instance.instance_id.clone(),
                SubTestItem::HardPoint,
                error_msg,
            );
            outcome.digital_steps = Some(digital_steps);
            return Ok(outcome);
        }
        debug!("✅ 高电平: {}", di_state_2);

        // 步骤5: 测试PLC DO输出低电平(复位)
        info!("变量:{}, 写[{}]=false", definition.tag, test_rig_do_address);
        plc_service_test_rig.write_bool(&test_rig_do_address, false).await
            .map_err(|e| AppError::plc_communication_error(format!("复位测试PLC DO低电平失败: {}", e)))?;

        // 等待信号稳定
        tokio::time::sleep(tokio::time::Duration::from_millis(self.step_interval_ms)).await;

        // 步骤6: 最终检查被测PLC DI是否显示"断开"
        let di_state_3;
        di_state_3 = plc_service_target.read_bool(target_di_address).await
            .map_err(|e| AppError::plc_communication_error(format!("读取被测PLC DI状态失败: {}", e)))?;
        info!("变量:{}, 读[{}]={}", definition.tag, target_di_address, di_state_3);

        // 记录步骤3结果
        let step3_status = if di_state_3 {
            SubTestStatus::Failed
        } else {
            SubTestStatus::Passed
        };
        digital_steps.push(DigitalTestStep {
            step_number: 3,
            step_description: "测试PLC DO复位低电平，检查被测PLC DI显示断开".to_string(),
            set_value: false,
            expected_reading: false,
            actual_reading: di_state_3,
            status: step3_status.clone(),
            timestamp: Utc::now(),
        });

        if di_state_3 {
            let error_msg = format!("❌ DI硬点测试失败: 复位后DI应为false，实际为true");
            info!("{}", error_msg);
            let mut outcome = RawTestOutcome::failure(
                instance.instance_id.clone(),
                SubTestItem::HardPoint,
                error_msg,
            );
            outcome.digital_steps = Some(digital_steps);
            return Ok(outcome);
        }
        debug!("✅ 复位: {}", di_state_3);

        let end_time = Utc::now();
        debug!("🎯 DI硬点测试完成: 低({}) → 高({}) → 低({}) - 通过",
                                 di_state_1, di_state_2, di_state_3);

        // 构造成功的测试结果
        let success_msg = format!("DI硬点测试成功: 低→高→低电平切换，DI状态正确响应");
        let mut outcome = RawTestOutcome::success(
            instance.instance_id.clone(),
            SubTestItem::HardPoint,
        );
        outcome.message = Some(success_msg);
        outcome.start_time = start_time;
        outcome.end_time = end_time;
        outcome.digital_steps = Some(digital_steps);
        outcome.raw_value_read = Some(format!("状态序列: {} → {} → {}", di_state_1, di_state_2, di_state_3));

        Ok(outcome)
    }

    fn item_type(&self) -> SubTestItem {
        SubTestItem::HardPoint
    }

    fn executor_name(&self) -> &'static str {
        "DIHardPointTestExecutor"
    }

    fn supports_definition(&self, definition: &ChannelPointDefinition) -> bool {
        matches!(definition.module_type, ModuleType::DI | ModuleType::DINone)
    }
}

/// DO硬点测试执行器
///
/// 执行DO点的完整硬点测试：被测PLC的DO通道输出 → 测试PLC的DI通道检测
/// 测试步骤：
/// 1. 被测PLC DO通道输出低电平 → 检查测试PLC DI通道显示"断开"
/// 2. 等待信号稳定
/// 3. 被测PLC DO通道输出高电平 → 检查测试PLC DI通道显示"接通"
/// 4. 等待信号稳定
/// 5. 被测PLC DO通道输出低电平 → 检查测试PLC DI通道显示"断开"
pub struct DOHardPointTestExecutor {
    /// 测试步骤间隔时间 (毫秒)
    pub step_interval_ms: u64,
}

impl DOHardPointTestExecutor {
    /// 创建新的DO硬点测试执行器
    pub fn new(step_interval_ms: u64) -> Self {
        Self {
            step_interval_ms,
        }
    }

    /// 获取测试PLC对应的DI通道地址
    /// 从通道实例中获取已分配的测试PLC通道地址
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
        plc_service_test_rig: Arc<dyn IPlcCommunicationService>,
        plc_service_target: Arc<dyn IPlcCommunicationService>,
    ) -> AppResult<RawTestOutcome> {
        // 🔧 精简日志：移除详细调试信息
        info!("🚀 开始测试: {} [{}]", definition.tag, instance.instance_id);

        let start_time = Utc::now();
        let test_rig_di_address = self.get_test_rig_di_address(instance)?;
        let target_do_address = &definition.plc_communication_address;

        // 创建数字量测试步骤记录
        let mut digital_steps = Vec::new();

        // 步骤1: 被测PLC DO输出低电平
        plc_service_target.write_bool(target_do_address, false).await
            .map_err(|e| AppError::plc_communication_error(format!("设置被测PLC DO低电平失败: {}", e)))?;

        // 等待信号稳定
        tokio::time::sleep(tokio::time::Duration::from_millis(self.step_interval_ms)).await;

        // 步骤2: 检查测试PLC DI是否显示"断开"
        let di_state_1 = plc_service_test_rig.read_bool(&test_rig_di_address).await
            .map_err(|e| AppError::plc_communication_error(format!("读取测试PLC DI状态失败: {}", e)))?;

        // 记录步骤1结果
        let step1_status = if di_state_1 {
            SubTestStatus::Failed
        } else {
            SubTestStatus::Passed
        };
        digital_steps.push(DigitalTestStep {
            step_number: 1,
            step_description: "被测PLC DO输出低电平，检查测试PLC DI显示断开".to_string(),
            set_value: false,
            expected_reading: false,
            actual_reading: di_state_1,
            status: step1_status.clone(),
            timestamp: Utc::now(),
        });

        if di_state_1 {
            let error_msg = format!("❌ DO硬点测试失败: DO低电平时测试PLC DI应为false，实际为true");
            info!("{}", error_msg);
            let mut outcome = RawTestOutcome::failure(
                instance.instance_id.clone(),
                SubTestItem::HardPoint,
                error_msg,
            );
            outcome.digital_steps = Some(digital_steps);
            return Ok(outcome);
        }
        debug!("✅ 低电平测试通过: DO=false, DI={}", di_state_1);

        // 步骤3: 被测PLC DO输出高电平
        info!("变量:{}, 写[{}]=true", definition.tag, target_do_address);
        plc_service_target.write_bool(target_do_address, true).await
            .map_err(|e| AppError::plc_communication_error(format!("设置被测PLC DO高电平失败: {}", e)))?;

        // 等待信号稳定
        tokio::time::sleep(tokio::time::Duration::from_millis(self.step_interval_ms)).await;

        // 步骤4: 检查测试PLC DI是否显示"接通"
        let di_state_2;
        // 读取后再记录
        // 读取测试PLC DI
        di_state_2 = plc_service_test_rig.read_bool(&test_rig_di_address).await
            .map_err(|e| AppError::plc_communication_error(format!("读取测试PLC DI状态失败: {}", e)))?;
        info!("变量:{}, 读[{}]={}", definition.tag, test_rig_di_address, di_state_2);

        // 记录步骤2结果
        let step2_status = if !di_state_2 {
            SubTestStatus::Failed
        } else {
            SubTestStatus::Passed
        };
        digital_steps.push(DigitalTestStep {
            step_number: 2,
            step_description: "被测PLC DO输出高电平，检查测试PLC DI显示接通".to_string(),
            set_value: true,
            expected_reading: true,
            actual_reading: di_state_2,
            status: step2_status.clone(),
            timestamp: Utc::now(),
        });

        if !di_state_2 {
            let error_msg = format!("❌ DO硬点测试失败: DO高电平时测试PLC DI应为true，实际为false");
            info!("{}", error_msg);
            let mut outcome = RawTestOutcome::failure(
                instance.instance_id.clone(),
                SubTestItem::HardPoint,
                error_msg,
            );
            outcome.digital_steps = Some(digital_steps);
            return Ok(outcome);
        }
        debug!("✅ 高电平测试通过: DO=true, DI={}", di_state_2);

        // 步骤5: 被测PLC DO输出低电平(复位)
        info!("变量:{}, 写[{}]=false", definition.tag, target_do_address);
        plc_service_target.write_bool(target_do_address, false).await
            .map_err(|e| AppError::plc_communication_error(format!("复位被测PLC DO低电平失败: {}", e)))?;

        // 等待信号稳定
        tokio::time::sleep(tokio::time::Duration::from_millis(self.step_interval_ms)).await;

        // 步骤6: 最终检查测试PLC DI是否显示"断开"
        let di_state_3;
        di_state_3 = plc_service_test_rig.read_bool(&test_rig_di_address).await
            .map_err(|e| AppError::plc_communication_error(format!("读取测试PLC DI状态失败: {}", e)))?;
        info!("变量:{}, 读[{}]={}", definition.tag, test_rig_di_address, di_state_3);

        // 记录步骤3结果
        let step3_status = if di_state_3 {
            SubTestStatus::Failed
        } else {
            SubTestStatus::Passed
        };
        digital_steps.push(DigitalTestStep {
            step_number: 3,
            step_description: "被测PLC DO复位低电平，检查测试PLC DI显示断开".to_string(),
            set_value: false,
            expected_reading: false,
            actual_reading: di_state_3,
            status: step3_status.clone(),
            timestamp: Utc::now(),
        });

        if di_state_3 {
            let error_msg = format!("❌ DO硬点测试失败: 复位后测试PLC DI应为false，实际为true");
            info!("{}", error_msg);
            let mut outcome = RawTestOutcome::failure(
                instance.instance_id.clone(),
                SubTestItem::HardPoint,
                error_msg,
            );
            outcome.digital_steps = Some(digital_steps);
            return Ok(outcome);
        }
        debug!("✅ 复位测试通过: DO=false, DI={}", di_state_3);

        let end_time = Utc::now();
        debug!("🎯 DO硬点测试完成: 低({}) → 高({}) → 低({}) - 通过",
                                 di_state_1, di_state_2, di_state_3);

        // 构造成功的测试结果
        let success_msg = format!("DO硬点测试成功: 低→高→低电平切换，测试PLC DI状态正确响应");
        let mut outcome = RawTestOutcome::success(
            instance.instance_id.clone(),
            SubTestItem::HardPoint,
        );
        outcome.message = Some(success_msg);
        outcome.start_time = start_time;
        outcome.end_time = end_time;
        outcome.digital_steps = Some(digital_steps);
        outcome.raw_value_read = Some(format!("状态序列: {} → {} → {}", di_state_1, di_state_2, di_state_3));

        Ok(outcome)
    }

    fn item_type(&self) -> SubTestItem {
        SubTestItem::HardPoint
    }

    fn executor_name(&self) -> &'static str {
        "DOHardPointTestExecutor"
    }

    fn supports_definition(&self, definition: &ChannelPointDefinition) -> bool {
        matches!(definition.module_type, ModuleType::DO | ModuleType::DONone)
    }
}

/// AO硬点测试执行器
///
/// 执行AO点的完整硬点测试：被测PLC的AO通道输出 → 测试PLC的AI通道采集
/// 测试步骤：
/// 1. 被测PLC AO按序输出: 0%, 25%, 50%, 75%, 100%
/// 2. 测试PLC AI采集对应数值
/// 3. 验证采集值与期望值的偏差在允许范围内
pub struct AOHardPointTestExecutor {
    /// 测试步骤间隔时间 (毫秒)
    pub step_interval_ms: u64,
}

impl AOHardPointTestExecutor {
    /// 创建新的AO硬点测试执行器
    pub fn new(step_interval_ms: u64) -> Self {
        Self {
            step_interval_ms,
        }
    }

    /// 获取测试PLC对应的AI通道地址
    /// 从通道实例中获取已分配的测试PLC通道地址
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
        plc_service_test_rig: Arc<dyn IPlcCommunicationService>,
        plc_service_target: Arc<dyn IPlcCommunicationService>,
    ) -> AppResult<RawTestOutcome> {
        debug!("[{}] 开始执行AO硬点测试 - 实例: {}",
               self.executor_name(), instance.instance_id);

        let start_time = Utc::now();
        let test_rig_ai_address = self.get_test_rig_ai_address(instance)?;
        let target_ao_address = &definition.plc_communication_address;

        // 获取量程信息
        let range_lower = definition.range_low_limit.unwrap_or(0.0);
        let range_upper = definition.range_high_limit.unwrap_or(100.0);

        if range_upper <= range_lower {
            return Err(AppError::validation_error(
                format!("无效的量程范围: {} - {}", range_lower, range_upper)
            ));
        }

        info!("🔧 AO硬点测试开始 - 被测PLC AO: {}, 测试PLC AI: {}, 量程: {}-{}",
              target_ao_address, test_rig_ai_address, range_lower, range_upper);

        let test_percentages = vec![0.0, 0.25, 0.5, 0.75, 1.0];
        let mut readings = Vec::new();

        for (step, percentage) in test_percentages.iter().enumerate() {
            let output_value = range_lower + (range_upper - range_lower) * percentage;

            info!("📝 写入 [{}]: {:.2}", target_ao_address, output_value);

            // 设置被测PLC AO输出
            plc_service_target.write_float32(target_ao_address, output_value).await
                .map_err(|e| AppError::plc_communication_error(format!("设置被测PLC AO输出失败: {}", e)))?;

            // 等待信号稳定
            tokio::time::sleep(tokio::time::Duration::from_millis(self.step_interval_ms)).await;

            // 读取测试PLC AI采集值
            let read_value = plc_service_test_rig.read_float32(&test_rig_ai_address).await
                .map_err(|e| AppError::plc_communication_error(format!("读取测试PLC AI值失败: {}", e)))?;
            info!("📖 读取 [{}]: {:.2}", test_rig_ai_address, read_value);

            // 计算偏差
            let deviation = ((read_value - output_value) / (range_upper - range_lower) * 100.0).abs();
            let is_within_tolerance = deviation <= 5.0; // 5%偏差容限

            let status_icon = if is_within_tolerance { "✅" } else { "❌" };
            info!("{} {}%: {:.2}", status_icon, percentage * 100.0, read_value);

            readings.push((output_value, read_value, deviation, is_within_tolerance));

            // 🔧 不要立即返回失败，而是继续完成所有测试点以收集完整的过程数据
            if !is_within_tolerance {
                // 移除冗余的失败日志
            }
        }

        let end_time = Utc::now();

        // 🔧 检查是否有任何测试点失败
        let failed_points: Vec<_> = readings.iter().enumerate()
            .filter(|(_, (_, _, _, is_within_tolerance))| !*is_within_tolerance)
            .collect();
        let overall_success = failed_points.is_empty();

        let (success_msg, outcome) = if overall_success {
            info!("✅ 结果: {} - 通过", definition.tag);
            let msg = format!("AO硬点测试成功: 所有{}个测试点偏差均在5%以内", readings.len());
            (msg.clone(), RawTestOutcome::success(instance.instance_id.clone(), SubTestItem::HardPoint))
        } else {
            info!("❌ 结果: {} - 失败", definition.tag);
            let msg = format!("AO硬点测试失败: {}个测试点偏差过大", failed_points.len());
            (msg.clone(), RawTestOutcome::failure(instance.instance_id.clone(), SubTestItem::HardPoint, msg))
        };

        let mut outcome = outcome;
        outcome.message = Some(success_msg);
        outcome.start_time = start_time;
        outcome.end_time = end_time;

        // 转换读数为 AnalogReadingPoint 格式
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

        // 🔧 关键修复：无论测试成功还是失败，都要保存百分比测试结果
        // 提取百分比测试结果 - 存储实际工程量 (转换f32到f64)
        outcome.test_result_0_percent = analog_readings.get(0).and_then(|r| r.actual_reading_eng.map(|v| v as f64));
        outcome.test_result_25_percent = analog_readings.get(1).and_then(|r| r.actual_reading_eng.map(|v| v as f64));
        outcome.test_result_50_percent = analog_readings.get(2).and_then(|r| r.actual_reading_eng.map(|v| v as f64));
        outcome.test_result_75_percent = analog_readings.get(3).and_then(|r| r.actual_reading_eng.map(|v| v as f64));
        outcome.test_result_100_percent = analog_readings.get(4).and_then(|r| r.actual_reading_eng.map(|v| v as f64));

        // 🔄 测试完成后复位被测PLC的AO输出为0%
        let reset_value = range_lower; // 复位为量程下限
        info!("🔄 测试完成，复位被测PLC AO [{}]: {:.2}", target_ao_address, reset_value);
        if let Err(e) = plc_service_target.write_float32(target_ao_address, reset_value).await {
            // 复位失败不影响测试结果，只记录警告
            log::warn!("⚠️ 被测PLC AO复位失败: {}", e);
        } else {
            info!("✅ 被测PLC AO已复位为量程下限");
        }

        // 🔧 精简日志：移除详细百分比结果日志

        Ok(outcome)
    }

    fn item_type(&self) -> SubTestItem {
        SubTestItem::HardPoint
    }

    fn executor_name(&self) -> &'static str {
        "AOHardPointTestExecutor"
    }

    fn supports_definition(&self, definition: &ChannelPointDefinition) -> bool {
        matches!(definition.module_type, ModuleType::AO | ModuleType::AONone)
    }
}

/// DI状态读取执行器
/// 仅读取一次 DI 状态并与期望值比较（如有提供）。
pub struct DIStateReadExecutor {
    expected_state: Option<bool>,
    read_interval_ms: u64,
}

impl DIStateReadExecutor {
    pub fn new(expected_state: Option<bool>, read_interval_ms: u64) -> Self {
        Self { expected_state, read_interval_ms }
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
        // 可配置读取间隔，简单 sleep
        sleep(Duration::from_millis(self.read_interval_ms)).await;

        let actual_state = plc_service_target.read_bool(&definition.plc_communication_address).await?;

        let success = match self.expected_state {
            Some(expect) => actual_state == expect,
            None => true,
        };

        let message = if let Some(expect) = self.expected_state {
            if success {
                format!("DI状态读取成功: 期望={}, 实际={}, 一致", expect, actual_state)
            } else {
                format!("DI状态读取失败: 期望={}, 实际={}", expect, actual_state)
            }
        } else {
            format!("DI状态读取完成: 实际={} (无期望值)", actual_state)
        };

        let mut outcome = if success {
            RawTestOutcome::success(instance.instance_id.clone(), SubTestItem::StateDisplay)
        } else {
            RawTestOutcome::failure(instance.instance_id.clone(), SubTestItem::StateDisplay, message.clone())
        };

        outcome.message = Some(message);
        outcome.raw_value_read = Some(actual_state.to_string());
        Ok(outcome)
    }

    fn item_type(&self) -> SubTestItem { SubTestItem::StateDisplay }

    fn executor_name(&self) -> &'static str { "DIStateReadExecutor" }

    fn supports_definition(&self, definition: &ChannelPointDefinition) -> bool {
        matches!(definition.module_type, ModuleType::DI | ModuleType::DINone)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::infrastructure::MockPlcService;

    /// 创建测试用的AI点位定义
    fn create_test_ai_definition() -> ChannelPointDefinition {
        let mut definition = ChannelPointDefinition::new(
            "AI001".to_string(),
            "Temperature_1".to_string(),
            "温度传感器1".to_string(),
            "Station1".to_string(),
            "Module1".to_string(),
            ModuleType::AI,
            "CH01".to_string(),
            PointDataType::Float,
            "DB1.DBD0".to_string(),
        );

        definition.range_low_limit = Some(0.0);
        definition.range_high_limit = Some(100.0);
        // 不再使用虚拟地址
        definition.test_rig_plc_address = None;
        definition.sh_set_value = Some(80.0);
        definition.sh_set_point_address = Some("DB1.DBD4".to_string());
        definition.sh_feedback_address = Some("M0.0".to_string());

        definition
    }

    /// 创建测试用的DI点位定义
    fn create_test_di_definition() -> ChannelPointDefinition {
        ChannelPointDefinition::new(
            "DI001".to_string(),
            "Switch_1".to_string(),
            "开关状态1".to_string(),
            "Station1".to_string(),
            "Module1".to_string(),
            ModuleType::DI,
            "CH01".to_string(),
            PointDataType::Bool,
            "I0.0".to_string(),
        )
    }

    /// 创建测试用的通道测试实例
    fn create_test_instance() -> ChannelTestInstance {
        ChannelTestInstance::new(
            "def_001".to_string(),
            "batch_001".to_string(),
        )
    }

    #[tokio::test]
    async fn test_ai_hardpoint_executor_success() {
        // 创建Mock PLC服务
        let mut mock_test_rig = MockPlcService::new_for_testing("TestRig");
        let mut mock_target = MockPlcService::new_for_testing("Target");

        // 连接PLC服务
        mock_test_rig.connect().await.unwrap();
        mock_target.connect().await.unwrap();

        let mock_test_rig = Arc::new(mock_test_rig);
        let mock_target = Arc::new(mock_target);

        // 设置Mock返回值 - 模拟50%点测试成功
        mock_target.preset_read_value("DB1.DBD0", serde_json::json!(50.0));

        // 创建执行器
        let executor = AIHardPointPercentExecutor::new();

        // 创建测试数据
        let definition = create_test_ai_definition();
        let instance = create_test_instance();

        // 执行测试
        let result = executor.execute_complete_ai_hardpoint_test(
            &instance,
            &definition,
            mock_test_rig.clone(),
            mock_target.clone(),
        ).await;

        // 验证结果
        match &result {
            Ok(outcome) => {
                assert!(outcome.success);
                assert_eq!(outcome.sub_test_item, SubTestItem::HardPoint);
                assert!(outcome.message.is_some());
                assert!(outcome.readings.is_some());
            },
            Err(e) => {
                panic!("测试执行失败: {}", e);
            }
        }

        // 验证写入操作被调用
        let write_history = mock_test_rig.get_write_log();
        assert!(!write_history.is_empty());
    }

    #[tokio::test]
    async fn test_ai_hardpoint_executor_failure() {
        // 创建Mock PLC服务
        let mut mock_test_rig = MockPlcService::new_for_testing("TestRig");
        let mut mock_target = MockPlcService::new_for_testing("Target");

        // 连接PLC服务
        mock_test_rig.connect().await.unwrap();
        mock_target.connect().await.unwrap();

        let mock_test_rig = Arc::new(mock_test_rig);
        let mock_target = Arc::new(mock_target);

        // 设置Mock返回值 - 模拟读取值偏差过大
        mock_target.preset_read_value("DB1.DBD0", serde_json::json!(30.0)); // 期望50.0，实际30.0，偏差20%

        // 创建执行器
        let executor = AIHardPointPercentExecutor::new();

        // 创建测试数据
        let definition = create_test_ai_definition();
        let instance = create_test_instance();

        // 执行测试
        let result = executor.execute_complete_ai_hardpoint_test(
            &instance,
            &definition,
            mock_test_rig.clone(),
            mock_target.clone(),
        ).await;

        // 验证结果
        assert!(result.is_ok());
        let outcome = result.unwrap();
        assert!(!outcome.success); // 应该失败
        assert!(outcome.message.as_ref().unwrap().contains("失败"));
    }

    #[tokio::test]
    async fn test_ai_alarm_executor_success() {
        // 创建Mock PLC服务
        let mut mock_test_rig = MockPlcService::new_for_testing("TestRig");
        let mut mock_target = MockPlcService::new_for_testing("Target");

        // 连接PLC服务
        mock_test_rig.connect().await.unwrap();
        mock_target.connect().await.unwrap();

        let mock_test_rig = Arc::new(mock_test_rig);
        let mock_target = Arc::new(mock_target);

        // 设置Mock返回值 - 模拟报警激活
        mock_target.preset_read_value("M0.0", serde_json::json!(true));

        // 创建执行器
        let executor = AIAlarmTestExecutor::new(SubTestItem::HighAlarm, 50, 50); // 减少延时以加快测试

        // 创建测试数据
        let definition = create_test_ai_definition();
        let instance = create_test_instance();

        // 在测试执行过程中动态改变Mock返回值（模拟报警复位）
        let mock_target_clone = mock_target.clone();
        tokio::spawn(async move {
            tokio::time::sleep(tokio::time::Duration::from_millis(80)).await;
            mock_target_clone.preset_read_value("M0.0", serde_json::json!(false));
        });

        // 执行测试
        let result = executor.execute(
            &instance,
            &definition,
            mock_test_rig.clone(),
            mock_target.clone(),
        ).await;

        // 验证结果
        assert!(result.is_ok());
        let outcome = result.unwrap();
        assert!(outcome.success);
        assert_eq!(outcome.sub_test_item, SubTestItem::HighAlarm);
        assert!(outcome.message.as_ref().unwrap().contains("通过"));
    }

    #[tokio::test]
    async fn test_di_state_executor_success() {
        // 创建Mock PLC服务
        let mut mock_test_rig = MockPlcService::new_for_testing("TestRig");
        let mut mock_target = MockPlcService::new_for_testing("Target");

        // 连接PLC服务
        mock_test_rig.connect().await.unwrap();
        mock_target.connect().await.unwrap();

        let mock_test_rig = Arc::new(mock_test_rig);
        let mock_target = Arc::new(mock_target);

        // 设置Mock返回值
        mock_target.preset_read_value("I0.0", serde_json::json!(true));

        // 创建执行器
        let executor = DIStateReadExecutor::new(Some(true), 50);

        // 创建测试数据
        let definition = create_test_di_definition();
        let instance = create_test_instance();

        // 执行测试
        let result = executor.execute(
            &instance,
            &definition,
            mock_test_rig.clone(),
            mock_target.clone(),
        ).await;

        // 验证结果
        assert!(result.is_ok());
        let outcome = result.unwrap();
        assert!(outcome.success);
        assert_eq!(outcome.sub_test_item, SubTestItem::StateDisplay);
        assert!(outcome.message.as_ref().unwrap().contains("通过"));
    }

    #[tokio::test]
    async fn test_di_state_executor_no_expectation() {
        // 创建Mock PLC服务
        let mut mock_test_rig = MockPlcService::new_for_testing("TestRig");
        let mut mock_target = MockPlcService::new_for_testing("Target");

        // 连接PLC服务
        mock_test_rig.connect().await.unwrap();
        mock_target.connect().await.unwrap();

        let mock_test_rig = Arc::new(mock_test_rig);
        let mock_target = Arc::new(mock_target);

        // 设置Mock返回值
        mock_target.preset_read_value("I0.0", serde_json::json!(false));

        // 创建执行器 - 不指定期望值
        let executor = DIStateReadExecutor::new(None, 50);

        // 创建测试数据
        let definition = create_test_di_definition();
        let instance = create_test_instance();

        // 执行测试
        let result = executor.execute(
            &instance,
            &definition,
            mock_test_rig.clone(),
            mock_target.clone(),
        ).await;

        // 验证结果
        assert!(result.is_ok());
        let outcome = result.unwrap();
        assert!(outcome.success); // 没有期望值时总是成功
        assert!(outcome.message.as_ref().unwrap().contains("完成"));
    }

    #[tokio::test]
    async fn test_executor_supports_definition() {
        let ai_definition = create_test_ai_definition();
        let di_definition = create_test_di_definition();

        // 测试AI硬点执行器
        let ai_executor = AIHardPointPercentExecutor::new();
        assert!(ai_executor.supports_definition(&ai_definition));
        assert!(!ai_executor.supports_definition(&di_definition));

        // 测试AI报警执行器
        let alarm_executor = AIAlarmTestExecutor::new(SubTestItem::HighAlarm, 100, 100);
        assert!(alarm_executor.supports_definition(&ai_definition));
        assert!(!alarm_executor.supports_definition(&di_definition));

        // 测试DI状态执行器
        let di_executor = DIStateReadExecutor::new(None, 50);
        assert!(!di_executor.supports_definition(&ai_definition));
        assert!(di_executor.supports_definition(&di_definition));
    }

    #[tokio::test]
    async fn test_ai_hardpoint_executor_invalid_range() {
        // 创建Mock PLC服务
        let mut mock_test_rig = MockPlcService::new_for_testing("TestRig");
        let mut mock_target = MockPlcService::new_for_testing("Target");

        // 连接PLC服务
        mock_test_rig.connect().await.unwrap();
        mock_target.connect().await.unwrap();

        let mock_test_rig = Arc::new(mock_test_rig);
        let mock_target = Arc::new(mock_target);

        // 创建无效量程的定义
        let mut definition = create_test_ai_definition();
        definition.range_low_limit = Some(100.0);
        definition.range_high_limit = Some(50.0); // 上限小于下限

        let executor = AIHardPointPercentExecutor::new();
        let instance = create_test_instance();

        // 执行测试
        let result = executor.execute_complete_ai_hardpoint_test(
            &instance,
            &definition,
            mock_test_rig.clone(),
            mock_target.clone(),
        ).await;

        // 验证结果
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.to_string().contains("无效的量程范围"));
    }

    #[tokio::test]
    async fn test_ai_alarm_executor_missing_config() {
        // 创建Mock PLC服务
        let mut mock_test_rig = MockPlcService::new_for_testing("TestRig");
        let mut mock_target = MockPlcService::new_for_testing("Target");

        // 连接PLC服务
        mock_test_rig.connect().await.unwrap();
        mock_target.connect().await.unwrap();

        let mock_test_rig = Arc::new(mock_test_rig);
        let mock_target = Arc::new(mock_target);

        // 创建缺少报警配置的定义
        let mut definition = create_test_ai_definition();
        definition.sh_set_point_address = None; // 移除高报设定地址

        let executor = AIAlarmTestExecutor::new(SubTestItem::HighAlarm, 100, 100);
        let instance = create_test_instance();

        // 执行测试
        let result = executor.execute(
            &instance,
            &definition,
            mock_test_rig.clone(),
            mock_target.clone(),
        ).await;

        // 验证结果
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.to_string().contains("未配置高报设定地址"));
    }
}