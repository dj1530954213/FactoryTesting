# C# 到 Rust 字段映射表

## ChannelPointDefinitions 表

| C# 字段名 | C# 类型 | Rust 字段名 | Rust 类型 | 说明 |
|-----------|---------|-------------|-----------|------|
| Id | string | id | String | 主键 |
| ChannelTag | string | tag | String | 通道标识 |
| VariableName | string | variable_name | String | 变量名 |
| VariableDescription | string | variable_description | Option<String> | 变量描述 |
| StationName | string | station_name | String | 站点名称 |
| ModuleName | string | module_name | String | 模块名称 |
| ModuleType | string | module_type | ModuleType | 模块类型枚举 |
| ChannelTagInModule | string | channel_tag_in_module | String | 模块内通道标识 |
| DataType | string | data_type | PointDataType | 数据类型枚举 |
| PowerSupplyType | string | power_supply_type | String | 供电类型 |
| WireSystem | string | wire_system | String | 线制 |
| PlcAbsoluteAddress | string | plc_absolute_address | Option<String> | PLC绝对地址 |
| PlcCommunicationAddress | string | plc_communication_address | String | PLC通信地址 |
| RangeLowerLimit | decimal? | range_lower_limit | Option<f64> | 量程下限 |
| RangeUpperLimit | decimal? | range_upper_limit | Option<f64> | 量程上限 |
| EngineeringUnit | string | engineering_unit | Option<String> | 工程单位 |
| SllSetValue | decimal? | sll_set_value | Option<f64> | 低低报警设定值 |
| SllSetPointAddress | string | sll_set_point_address | Option<String> | 低低报警设定点地址 |
| SllSetPointPlcAddress | string | sll_set_point_plc_address | Option<String> | 低低报警设定点PLC地址 |
| SllFeedbackAddress | string | sll_feedback_address | Option<String> | 低低报警反馈地址 |
| SllFeedbackPlcAddress | string | sll_feedback_plc_address | Option<String> | 低低报警反馈PLC地址 |
| SlSetValue | decimal? | sl_set_value | Option<f64> | 低报警设定值 |
| SlSetPointAddress | string | sl_set_point_address | Option<String> | 低报警设定点地址 |
| SlSetPointPlcAddress | string | sl_set_point_plc_address | Option<String> | 低报警设定点PLC地址 |
| SlFeedbackAddress | string | sl_feedback_address | Option<String> | 低报警反馈地址 |
| SlFeedbackPlcAddress | string | sl_feedback_plc_address | Option<String> | 低报警反馈PLC地址 |
| ShSetValue | decimal? | sh_set_value | Option<f64> | 高报警设定值 |
| ShSetPointAddress | string | sh_set_point_address | Option<String> | 高报警设定点地址 |
| ShSetPointPlcAddress | string | sh_set_point_plc_address | Option<String> | 高报警设定点PLC地址 |
| ShFeedbackAddress | string | sh_feedback_address | Option<String> | 高报警反馈地址 |
| ShFeedbackPlcAddress | string | sh_feedback_plc_address | Option<String> | 高报警反馈PLC地址 |
| ShhSetValue | decimal? | shh_set_value | Option<f64> | 高高报警设定值 |
| ShhSetPointAddress | string | shh_set_point_address | Option<String> | 高高报警设定点地址 |
| ShhSetPointPlcAddress | string | shh_set_point_plc_address | Option<String> | 高高报警设定点PLC地址 |
| ShhFeedbackAddress | string | shh_feedback_address | Option<String> | 高高报警反馈地址 |
| ShhFeedbackPlcAddress | string | shh_feedback_plc_address | Option<String> | 高高报警反馈PLC地址 |
| MaintenanceValueSetPointAddress | string | maintenance_value_set_point_address | Option<String> | 维护值设定点地址 |
| MaintenanceValueSetPointPlcAddress | string | maintenance_value_set_point_plc_address | Option<String> | 维护值设定点PLC地址 |
| MaintenanceEnableSwitchPointAddress | string | maintenance_enable_switch_point_address | Option<String> | 维护使能开关点地址 |
| MaintenanceEnableSwitchPointPlcAddress | string | maintenance_enable_switch_point_plc_address | Option<String> | 维护使能开关点PLC地址 |
| AccessProperty | string | access_property | Option<String> | 访问属性 |
| SaveHistory | bool | save_history | Option<bool> | 是否保存历史 |
| PowerFailureProtection | bool | power_failure_protection | Option<bool> | 断电保护 |
| TestRigPlcAddress | string | test_rig_plc_address | Option<String> | 测试台PLC地址 |
| CreatedTime | DateTime | created_time | String | 创建时间 |
| UpdatedTime | DateTime | updated_time | String | 更新时间 |

## TestBatchInfo 表

| C# 字段名 | C# 类型 | Rust 字段名 | Rust 类型 | 说明 |
|-----------|---------|-------------|-----------|------|
| BatchId | string | batch_id | String | 批次ID主键 |
| BatchName | string | batch_name | String | 批次名称 |
| ProductModel | string | product_model | Option<String> | 产品型号 |
| SerialNumber | string | serial_number | Option<String> | 序列号 |
| CustomerName | string | customer_name | Option<String> | 客户名称 |
| StationName | string | station_name | Option<String> | 站点名称 |
| CreatedTime | DateTime | created_time | String | 创建时间 |
| UpdatedTime | DateTime | updated_time | String | 更新时间 |
| StartTime | DateTime? | start_time | Option<String> | 开始时间 |
| EndTime | DateTime? | end_time | Option<String> | 结束时间 |
| TotalDurationMs | long? | total_duration_ms | Option<i64> | 总耗时(毫秒) |
| OperatorName | string | operator_name | Option<String> | 操作员 |
| CreatedBy | string | created_by | Option<String> | 创建者 |
| OverallStatus | string | overall_status | String | 整体状态 |
| StatusSummary | string | status_summary | Option<String> | 状态摘要 |
| ErrorMessage | string | error_message | Option<String> | 错误信息 |
| TotalPoints | int | total_points | u32 | 总点数 |
| TestedPoints | int | tested_points | u32 | 已测试点数 |
| PassedPoints | int | passed_points | u32 | 通过点数 |
| FailedPoints | int | failed_points | u32 | 失败点数 |
| SkippedPoints | int | skipped_points | u32 | 跳过点数 |
| NotTestedPoints | int | not_tested_points | u32 | 未测试点数 |
| ProgressPercentage | float | progress_percentage | f32 | 进度百分比 |
| CurrentTestingChannel | string | current_testing_channel | Option<String> | 当前测试通道 |
| TestConfiguration | string | test_configuration | Option<String> | 测试配置 |
| ImportSource | string | import_source | Option<String> | 导入源 |
| CustomData | Dictionary<string,string> | custom_data_json | Option<String> | 自定义数据(JSON) |

## ChannelTestInstances 表

| C# 字段名 | C# 类型 | Rust 字段名 | Rust 类型 | 说明 |
|-----------|---------|-------------|-----------|------|
| InstanceId | string | instance_id | String | 实例ID主键 |
| DefinitionId | string | definition_id | String | 关联定义ID |
| TestBatchId | string | test_batch_id | String | 关联批次ID |
| TestBatchName | string | test_batch_name | String | 批次名称 |
| ChannelTag | string | channel_tag | String | 通道标识 |
| VariableName | string | variable_name | String | 变量名称 |
| VariableDescription | string | variable_description | String | 变量描述 |
| ModuleType | string | module_type | String | 模块类型 |
| DataType | string | data_type | String | 数据类型 |
| PlcCommunicationAddress | string | plc_communication_address | String | PLC通信地址 |
| OverallStatus | string | overall_status | OverallTestStatus | 整体状态枚举 |
| CurrentStepDetails | string | current_step_details | Option<String> | 当前步骤详情 |
| ErrorMessage | string | error_message | Option<String> | 错误信息 |
| CreatedTime | DateTime | created_time | String | 创建时间 |
| StartTime | DateTime? | start_time | Option<String> | 开始时间 |
| UpdatedTime | DateTime | updated_time | String | 更新时间 |
| FinalTestTime | DateTime? | final_test_time | Option<String> | 最终测试时间 |
| TotalTestDurationMs | long? | total_test_duration_ms | Option<i64> | 总测试耗时 |
| HardPointStatus | int? | hard_point_status | Option<i32> | 硬点状态 |
| HardPointTestResult | string | hard_point_test_result | Option<String> | 硬点测试结果 |
| HardPointErrorDetail | string | hard_point_error_detail | Option<String> | 硬点错误详情 |
| ActualValue | string | actual_value | Option<String> | 实际值 |
| ExpectedValue | string | expected_value | Option<String> | 期望值 |
| CurrentValue | string | current_value | Option<String> | 当前值 |
| LowLowAlarmStatus | int? | low_low_alarm_status | Option<i32> | 低低报警状态 |
| LowAlarmStatus | int? | low_alarm_status | Option<i32> | 低报警状态 |
| HighAlarmStatus | int? | high_alarm_status | Option<i32> | 高报警状态 |
| HighHighAlarmStatus | int? | high_high_alarm_status | Option<i32> | 高高报警状态 |
| MaintenanceFunction | int? | maintenance_function | Option<i32> | 维护功能状态 |
| TrendCheck | int? | trend_check | Option<i32> | 趋势检查状态 |
| ReportCheck | int? | report_check | Option<i32> | 报表检查状态 |
| ShowValueStatus | int? | show_value_status | Option<i32> | 显示值状态 |
| TestResultStatus | int? | test_result_status | Option<i32> | 测试结果状态 |
| TestPlcChannelTag | string | test_plc_channel_tag | Option<String> | 测试PLC通道标识 |
| TestPlcCommunicationAddress | string | test_plc_communication_address | Option<String> | 测试PLC通信地址 |
| CurrentOperator | string | current_operator | Option<String> | 当前操作员 |
| RetriesCount | int | retries_count | i32 | 重试次数 |
| SubTestResults | Dictionary | sub_test_results_json | Option<String> | 子测试结果(JSON) |
| HardpointReadings | List | hardpoint_readings_json | Option<String> | 硬点读数(JSON) |
| TransientData | Dictionary | transient_data_json | Option<String> | 瞬态数据(JSON) |

## RawTestOutcomes 表

| C# 字段名 | C# 类型 | Rust 字段名 | Rust 类型 | 说明 |
|-----------|---------|-------------|-----------|------|
| Id | string | id | String | 主键 |
| ChannelInstanceId | string | channel_instance_id | String | 关联测试实例ID |
| SubTestItem | string | sub_test_item | SubTestItem | 子测试项枚举 |
| Success | bool | success | bool | 是否成功 |
| Message | string | message | Option<String> | 消息 |
| ExecutionTime | DateTime | execution_time | String | 执行时间 |
| Readings | List | readings_json | Option<String> | 读数数据(JSON) |

## 数据类型映射规则

### 基本类型映射
- `string` → `String` (必填) 或 `Option<String>` (可选)
- `int` → `i32`
- `long` → `i64`
- `float` → `f32`
- `double` → `f64`
- `decimal` → `f64`
- `bool` → `bool`
- `DateTime` → `String` (ISO 8601格式)
- `Dictionary<string,string>` → `Option<String>` (JSON序列化)
- `List<T>` → `Option<String>` (JSON序列化)

### 枚举类型映射
- C#枚举 → Rust枚举 (使用serde序列化)
- 字符串常量 → Rust枚举变体

### 可空类型处理
- C# `T?` → Rust `Option<T>`
- C# `string` (可为null) → Rust `Option<String>`

### JSON字段处理
- 复杂对象在数据库中存储为JSON字符串
- 使用serde进行序列化/反序列化
- 在应用层转换为强类型结构
