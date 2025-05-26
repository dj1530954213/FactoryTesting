// 枚举类型定义
export enum ModuleType {
  AI = 'AI',
  AO = 'AO', 
  DI = 'DI',
  DO = 'DO',
  AINone = 'AINone',
  DINone = 'DINone'
}

export enum PointDataType {
  Bool = 'Bool',
  Int = 'Int',
  Float = 'Float',
  String = 'String'
}

export enum OverallTestStatus {
  NotTested = 'NotTested',
  HardPointTesting = 'HardPointTesting',
  AlarmTesting = 'AlarmTesting', 
  TestCompletedPassed = 'TestCompletedPassed',
  TestCompletedFailed = 'TestCompletedFailed'
}

export enum SubTestItem {
  HardPoint = 'HardPoint',
  LowLowAlarm = 'LowLowAlarm',
  LowAlarm = 'LowAlarm',
  HighAlarm = 'HighAlarm',
  HighHighAlarm = 'HighHighAlarm',
  StateDisplay = 'StateDisplay'
}

export enum SubTestStatus {
  NotStarted = 'NotStarted',
  InProgress = 'InProgress',
  Passed = 'Passed', 
  Failed = 'Failed',
  Skipped = 'Skipped'
}

// 核心数据模型
export interface ChannelPointDefinition {
  id: string;
  tag: string;
  variable_name: string;
  description: string;
  station_name: string;
  module_name: string;
  module_type: ModuleType;
  channel_number: string;
  point_data_type: PointDataType;
  plc_communication_address: string;
  analog_range_min?: number;
  analog_range_max?: number;
  created_at: string;
  updated_at: string;
}

export interface TestBatchInfo {
  batch_id: string;
  product_model?: string;
  serial_number?: string;
  operator_name?: string;
  total_points: number;
  passed_points?: number;
  failed_points?: number;
  test_start_time?: string;
  test_end_time?: string;
  overall_status: OverallTestStatus;
  created_at: string;
  updated_at: string;
}

export interface ChannelTestInstance {
  instance_id: string;
  definition_id: string;
  test_batch_id: string;
  overall_status: OverallTestStatus;
  sub_test_results: { [key: string]: SubTestStatus };
  created_at: string;
  updated_at: string;
}

export interface RawTestOutcome {
  channel_instance_id: string;
  sub_test_item: SubTestItem;
  success: boolean;
  raw_value_read?: string;
  eng_value_calculated?: string;
  message?: string;
  start_time: string;
  end_time: string;
  readings?: AnalogReadingPoint[];
  details?: { [key: string]: any };
}

export interface AnalogReadingPoint {
  tag: string;
  value: number;
  timestamp: string;
  quality: string;
}

// 应用层数据模型
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

// 应用配置相关类型
export interface AppSettings {
  id: string;
  theme: string;
  plc_ip_address?: string;
  plc_port?: number;
  default_operator_name?: string;
  auto_save_interval_minutes?: number;
  recent_projects: string[];
  last_backup_time?: string;
}

// Excel文件处理相关类型
export interface ParseExcelResponse {
  success: boolean;
  message: string;
  data?: ChannelPointDefinition[];
  total_count: number;
}

export interface CreateBatchRequest {
  file_name: string;
  file_path: string;
  preview_data: ChannelPointDefinition[];
  batch_info: BatchInfo;
}

export interface BatchInfo {
  product_model: string;
  serial_number: string;
  customer_name?: string;
  operator_name?: string;
}

export interface CreateBatchResponse {
  success: boolean;
  message: string;
  batch_id?: string;
}

// 中文标签常量映射
export const MODULE_TYPE_LABELS: { [key in ModuleType]: string } = {
  [ModuleType.AI]: '模拟量输入',
  [ModuleType.AO]: '模拟量输出',
  [ModuleType.DI]: '数字量输入', 
  [ModuleType.DO]: '数字量输出',
  [ModuleType.AINone]: '模拟量输入(无)',
  [ModuleType.DINone]: '数字量输入(无)'
};

export const POINT_DATA_TYPE_LABELS: { [key in PointDataType]: string } = {
  [PointDataType.Bool]: '布尔型',
  [PointDataType.Int]: '整数型',
  [PointDataType.Float]: '浮点型',
  [PointDataType.String]: '字符串型'
};

export const OVERALL_TEST_STATUS_LABELS: { [key in OverallTestStatus]: string } = {
  [OverallTestStatus.NotTested]: '未测试',
  [OverallTestStatus.HardPointTesting]: '硬点测试中',
  [OverallTestStatus.AlarmTesting]: '报警测试中',
  [OverallTestStatus.TestCompletedPassed]: '测试完成并通过',
  [OverallTestStatus.TestCompletedFailed]: '测试完成并失败'
};

export const SUB_TEST_ITEM_LABELS: { [key in SubTestItem]: string } = {
  [SubTestItem.HardPoint]: '硬点测试',
  [SubTestItem.LowLowAlarm]: '低低报警测试',
  [SubTestItem.LowAlarm]: '低报警测试',
  [SubTestItem.HighAlarm]: '高报警测试',
  [SubTestItem.HighHighAlarm]: '高高报警测试',
  [SubTestItem.StateDisplay]: '状态显示测试'
};

export const SUB_TEST_STATUS_LABELS: { [key in SubTestStatus]: string } = {
  [SubTestStatus.NotStarted]: '未开始',
  [SubTestStatus.InProgress]: '进行中',
  [SubTestStatus.Passed]: '通过',
  [SubTestStatus.Failed]: '失败',
  [SubTestStatus.Skipped]: '跳过'
};

// 报告相关接口
export interface TestReport {
  id: string;
  title: string;
  batch_id: string;
  template_id: string;
  format: 'PDF' | 'Excel';
  file_path: string;
  file_size?: number;
  generated_by: string;
  created_at: string;
  updated_at: string;
}

export interface ReportTemplate {
  id: string;
  name: string;
  description: string;
  content: string;
  created_at: string;
  updated_at: string;
}

export interface ReportGenerationRequest {
  batch_id: string;
  template_id: string;
  format: 'PDF' | 'Excel';
  options: {
    include_charts: boolean;
    include_details: boolean;
    custom_title?: string;
    custom_description?: string;
  };
} 