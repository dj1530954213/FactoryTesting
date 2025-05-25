/**
 * 前端 TypeScript 类型定义
 * 对应后端 Rust 模型
 */

// ============================================================================
// 枚举类型
// ============================================================================

export enum ModuleType {
  AI = 'AI',
  AINone = 'AINone',
  DI = 'DI',
  DINone = 'DINone',
  DO = 'DO',
  DONone = 'DONone',
  AO = 'AO',
  AONone = 'AONone',
}

export enum PointDataType {
  Bool = 'Bool',
  Int16 = 'Int16',
  Int32 = 'Int32',
  Float = 'Float',
  String = 'String',
}

export enum OverallTestStatus {
  NotTested = 'NotTested',
  InProgress = 'InProgress',
  Passed = 'Passed',
  Failed = 'Failed',
  Skipped = 'Skipped',
  Error = 'Error',
}

export enum SubTestItem {
  HardPoint = 'HardPoint',
  LowLowAlarm = 'LowLowAlarm',
  LowAlarm = 'LowAlarm',
  HighAlarm = 'HighAlarm',
  HighHighAlarm = 'HighHighAlarm',
  Maintenance = 'Maintenance',
  Trend = 'Trend',
  Report = 'Report',
  Manual = 'Manual',
}

export enum SubTestStatus {
  NotTested = 'NotTested',
  InProgress = 'InProgress',
  Passed = 'Passed',
  Failed = 'Failed',
  Skipped = 'Skipped',
  Error = 'Error',
}

// ============================================================================
// 核心数据模型
// ============================================================================

export interface ChannelPointDefinition {
  id: string;
  tag: string;
  variable_name: string;
  variable_description: string;
  station_name: string;
  module_name: string;
  module_type: ModuleType;
  channel_tag_in_module: string;
  data_type: PointDataType;
  power_supply_type: string;
  wire_system: string;
  plc_absolute_address?: string;
  plc_communication_address: string;
  range_lower_limit?: number;
  range_upper_limit?: number;
  engineering_unit?: string;
  sll_set_value?: number;
  sll_set_point_address?: string;
  sll_feedback_address?: string;
  sl_set_value?: number;
  sl_set_point_address?: string;
  sl_feedback_address?: string;
  sh_set_value?: number;
  sh_set_point_address?: string;
  sh_feedback_address?: string;
  shh_set_value?: number;
  shh_set_point_address?: string;
  shh_feedback_address?: string;
  maintenance_value_set_point_address?: string;
  maintenance_enable_switch_point_address?: string;
  access_property?: string;
  save_history?: boolean;
  power_failure_protection?: boolean;
  test_rig_plc_address?: string;
}

export interface TestBatchInfo {
  batch_id: string;
  product_model?: string;
  serial_number?: string;
  total_points: number;
  tested_points: number;
  passed_points: number;
  failed_points: number;
  skipped_points: number;
  operator_name?: string;
  test_start_time?: string;
  test_end_time?: string;
  total_test_duration_ms?: number;
  notes?: string;
  created_at: string;
  updated_at: string;
}

export interface AnalogReadingPoint {
  set_percentage: number;
  set_value_eng: number;
  actual_reading_raw?: number;
  actual_reading_eng?: number;
  status: SubTestStatus;
  timestamp?: string;
  tolerance_percentage?: number;
  within_tolerance?: boolean;
  deviation_percentage?: number;
  notes?: string;
}

export interface SubTestExecutionResult {
  status: SubTestStatus;
  error_message?: string;
  start_time?: string;
  end_time?: string;
  duration_ms?: number;
  outcome?: RawTestOutcome;
}

export interface ChannelTestInstance {
  instance_id: string;
  definition_id: string;
  test_batch_id: string;
  overall_status: OverallTestStatus;
  current_step_details?: string;
  error_message?: string;
  start_time?: string;
  last_updated_time: string;
  final_test_time?: string;
  total_test_duration_ms?: number;
  sub_test_results: Record<SubTestItem, SubTestExecutionResult>;
  pending_sub_tests: SubTestItem[];
  hardpoint_readings?: AnalogReadingPoint[];
  manual_test_current_value_input?: number;
  manual_test_current_value_output?: number;
}

export interface RawTestOutcome {
  channel_instance_id: string;
  sub_test_item: SubTestItem;
  success: boolean;
  raw_value_read?: string;
  eng_value_calculated?: string;
  message?: string;
  timestamp: string;
  analog_reading_point?: AnalogReadingPoint;
  readings?: AnalogReadingPoint[];
}

// ============================================================================
// 应用层数据模型
// ============================================================================

export interface TestExecutionRequest {
  batch_info: TestBatchInfo;
  channel_definitions: ChannelPointDefinition[];
  max_concurrent_tests?: number;
  auto_start: boolean;
}

export interface TestExecutionResponse {
  batch_id: string;
  instance_count: number;
  status: string;
  message: string;
}

export interface TestProgressUpdate {
  batch_id: string;
  instance_id: string;
  point_tag: string;
  overall_status: OverallTestStatus;
  completed_sub_tests: number;
  total_sub_tests: number;
  latest_result?: RawTestOutcome;
  timestamp: string;
}

export interface SystemStatus {
  active_test_tasks: number;
  system_health: string;
  version: string;
}

// ============================================================================
// 工具类型
// ============================================================================

export type AppResult<T> = T | { error: string };

export interface ApiError {
  error: string;
  code?: string;
  details?: any;
}

// ============================================================================
// 常量定义
// ============================================================================

export const MODULE_TYPE_LABELS: Record<ModuleType, string> = {
  [ModuleType.AI]: 'AI模拟量输入',
  [ModuleType.AINone]: 'AI无模块',
  [ModuleType.DI]: 'DI数字量输入',
  [ModuleType.DINone]: 'DI无模块',
  [ModuleType.DO]: 'DO数字量输出',
  [ModuleType.DONone]: 'DO无模块',
  [ModuleType.AO]: 'AO模拟量输出',
  [ModuleType.AONone]: 'AO无模块',
};

export const OVERALL_TEST_STATUS_LABELS: Record<OverallTestStatus, string> = {
  [OverallTestStatus.NotTested]: '未测试',
  [OverallTestStatus.InProgress]: '测试中',
  [OverallTestStatus.Passed]: '通过',
  [OverallTestStatus.Failed]: '失败',
  [OverallTestStatus.Skipped]: '跳过',
  [OverallTestStatus.Error]: '错误',
};

export const SUB_TEST_ITEM_LABELS: Record<SubTestItem, string> = {
  [SubTestItem.HardPoint]: '硬点测试',
  [SubTestItem.LowLowAlarm]: '低低报警',
  [SubTestItem.LowAlarm]: '低报警',
  [SubTestItem.HighAlarm]: '高报警',
  [SubTestItem.HighHighAlarm]: '高高报警',
  [SubTestItem.Maintenance]: '维护测试',
  [SubTestItem.Trend]: '趋势测试',
  [SubTestItem.Report]: '报表测试',
  [SubTestItem.Manual]: '手动测试',
};

export const SUB_TEST_STATUS_LABELS: Record<SubTestStatus, string> = {
  [SubTestStatus.NotTested]: '未测试',
  [SubTestStatus.InProgress]: '测试中',
  [SubTestStatus.Passed]: '通过',
  [SubTestStatus.Failed]: '失败',
  [SubTestStatus.Skipped]: '跳过',
  [SubTestStatus.Error]: '错误',
}; 