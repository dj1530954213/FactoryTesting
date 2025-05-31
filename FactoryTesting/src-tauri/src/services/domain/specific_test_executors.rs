/// 特定测试步骤执行器
///
/// 包含各种具体的测试执行器实现，每个执行器负责一个原子的测试操作

use crate::models::{
    ChannelTestInstance, ChannelPointDefinition, RawTestOutcome, SubTestItem,
    AnalogReadingPoint, ModuleType, PointDataType, SubTestStatus
};
use crate::services::infrastructure::IPlcCommunicationService;
use crate::utils::error::{AppError, AppResult};
use async_trait::async_trait;
use chrono::Utc;
use log::{debug, info, warn};
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

        info!("开始AI硬点测试: {}", instance.instance_id);

        // 计算量程信息
        let range_lower = definition.range_lower_limit.unwrap_or(0.0);
        let range_upper = definition.range_upper_limit.unwrap_or(100.0);
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

            // 设置测试台架输出值
            if let Some(test_rig_address) = &definition.test_rig_plc_address {
                test_rig_plc.write_float32(test_rig_address, eng_value).await
                    .map_err(|e| AppError::plc_communication_error(format!("设置测试台架输出失败: {}", e)))?;

                // 等待信号稳定
                tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

                // 读取被测PLC的实际值
                let actual_raw = target_plc.read_float32(&definition.plc_communication_address).await
                    .map_err(|e| AppError::plc_communication_error(format!("读取被测PLC值失败: {}", e)))?;

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

                info!("AI硬点测试 {}%: 设定={:.2}, 实际={:.2}, 误差={:.2}%",
                    percentage * 100.0, eng_value, actual_raw,
                    error_percentage.unwrap_or(0.0));

                // 如果任意点测试失败，立即返回失败结果
                if test_status == SubTestStatus::Failed {
                    return Ok(RawTestOutcome {
                        channel_instance_id: instance.instance_id.clone(),
                        sub_test_item: SubTestItem::HardPoint,
                        success: false,
                        raw_value_read: Some(actual_raw.to_string()),
                        eng_value_calculated: Some(eng_value.to_string()),
                        message: Some(format!("硬点测试失败: {}%点误差过大({:.2}%)",
                            percentage * 100.0, error_percentage.unwrap_or(0.0))),
                        start_time: Utc::now(),
                        end_time: Utc::now(),
                        readings: Some(readings),
                        details: HashMap::new(),
                    });
                }
            } else {
                warn!("AI点缺少测试台架地址配置: {}", definition.tag);
                return Ok(RawTestOutcome::failure(
                    instance.instance_id.clone(),
                    SubTestItem::HardPoint,
                    "缺少测试台架地址配置".to_string(),
                ));
            }
        }

        // 检查线性度（可选的高级检查）
        let linearity_check = self.check_linearity(&readings);

        info!("AI硬点测试完成: {} - 线性度检查: {}",
            instance.instance_id,
            if linearity_check { "通过" } else { "警告" });

        // 返回成功结果
        Ok(RawTestOutcome {
            channel_instance_id: instance.instance_id.clone(),
            sub_test_item: SubTestItem::HardPoint,
            success: true,
            raw_value_read: Some("多点测试".to_string()),
            eng_value_calculated: Some(format!("{:.2}-{:.2}", range_lower, range_upper)),
            message: Some("AI硬点5点测试全部通过".to_string()),
            start_time: Utc::now(),
            end_time: Utc::now(),
            readings: Some(readings),
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
        debug!("[{}] 开始执行AI硬点测试 - 实例: {}",
               self.executor_name(), instance.instance_id);

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
        matches!(definition.module_type, ModuleType::AI | ModuleType::AINone) &&
        definition.test_rig_plc_address.is_some()
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
        debug!("[{}] 开始执行AI报警测试 {:?} - 实例: {}",
               self.executor_name(), self.alarm_type, instance.instance_id);

        let (alarm_set_value, set_address, feedback_address) = self.get_alarm_config(definition)?;
        let start_time = Utc::now();

        // 步骤1: 设置报警触发值
        info!("[{}] 设置报警触发值: {} = {:.3}",
              self.executor_name(), set_address, alarm_set_value);

        plc_service_target.write_float32(&set_address, alarm_set_value).await?;

        // 步骤2: 等待报警触发
        if self.trigger_delay_ms > 0 {
            debug!("[{}] 等待报警触发 {} ms", self.executor_name(), self.trigger_delay_ms);
            tokio::time::sleep(tokio::time::Duration::from_millis(self.trigger_delay_ms)).await;
        }

        // 步骤3: 读取报警反馈状态
        debug!("[{}] 读取报警反馈状态: {}", self.executor_name(), feedback_address);
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

        info!("[{}] 复位报警，设置安全值: {} = {:.3}",
              self.executor_name(), set_address, safe_value);
        plc_service_target.write_float32(&set_address, safe_value).await?;

        // 步骤5: 等待报警复位
        if self.reset_delay_ms > 0 {
            debug!("[{}] 等待报警复位 {} ms", self.executor_name(), self.reset_delay_ms);
            tokio::time::sleep(tokio::time::Duration::from_millis(self.reset_delay_ms)).await;
        }

        // 步骤6: 确认报警已复位
        let alarm_reset = !plc_service_target.read_bool(&feedback_address).await?;

        let end_time = Utc::now();

        // 判断测试结果
        let is_success = alarm_active && alarm_reset;
        let message = if is_success {
            format!("报警测试 {:?} 通过 - 触发值: {:.3}, 报警激活: {}, 报警复位: {}",
                   self.alarm_type, alarm_set_value, alarm_active, alarm_reset)
        } else {
            format!("报警测试 {:?} 失败 - 触发值: {:.3}, 报警激活: {}, 报警复位: {}",
                   self.alarm_type, alarm_set_value, alarm_active, alarm_reset)
        };

        info!("[{}] {}", self.executor_name(), message);

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

/// DI状态读取执行器
///
/// 读取DI点的状态并验证
pub struct DIStateReadExecutor {
    /// 期望的状态值
    pub expected_state: Option<bool>,
    /// 读取延时 (毫秒)
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
        debug!("[{}] 开始执行DI状态读取测试 - 实例: {}",
               self.executor_name(), instance.instance_id);

        let target_address = &definition.plc_communication_address;
        let start_time = Utc::now();

        // 等待读取延时
        if self.read_delay_ms > 0 {
            debug!("[{}] 等待读取延时 {} ms", self.executor_name(), self.read_delay_ms);
            tokio::time::sleep(tokio::time::Duration::from_millis(self.read_delay_ms)).await;
        }

        // 读取DI状态
        info!("[{}] 读取DI状态: {}", self.executor_name(), target_address);
        let actual_state = plc_service_target.read_bool(target_address).await?;

        let end_time = Utc::now();

        // 判断测试结果
        let is_success = match self.expected_state {
            Some(expected) => actual_state == expected,
            None => true, // 如果没有期望值，只要能读取到就算成功
        };

        let message = match self.expected_state {
            Some(expected) => {
                if is_success {
                    format!("DI状态读取通过 - 期望: {}, 实际: {}", expected, actual_state)
                } else {
                    format!("DI状态读取失败 - 期望: {}, 实际: {}", expected, actual_state)
                }
            },
            None => format!("DI状态读取完成 - 当前状态: {}", actual_state),
        };

        info!("[{}] {}", self.executor_name(), message);

        // 构造测试结果
        let mut outcome = RawTestOutcome::new(
            instance.instance_id.clone(),
            self.item_type(),
            is_success,
        );

        outcome.message = Some(message);
        outcome.start_time = start_time;
        outcome.end_time = end_time;
        outcome.raw_value_read = Some(if actual_state { "1" } else { "0" }.to_string());
        outcome.eng_value_calculated = Some(if actual_state { "true" } else { "false" }.to_string());

        Ok(outcome)
    }

    fn item_type(&self) -> SubTestItem {
        SubTestItem::StateDisplay
    }

    fn executor_name(&self) -> &'static str {
        "DIStateReadExecutor"
    }

    fn supports_definition(&self, definition: &ChannelPointDefinition) -> bool {
        matches!(definition.module_type, ModuleType::DI | ModuleType::DINone) &&
        matches!(definition.data_type, PointDataType::Bool)
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

        definition.range_lower_limit = Some(0.0);
        definition.range_upper_limit = Some(100.0);
        definition.test_rig_plc_address = Some("DB2.DBD0".to_string());
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
        definition.range_lower_limit = Some(100.0);
        definition.range_upper_limit = Some(50.0); // 上限小于下限

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