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
  Skipped = 'Skipped',
  WiringConfirmationRequired = 'WiringConfirmationRequired',
  WiringConfirmed = 'WiringConfirmed',
  HardPointTestInProgress = 'HardPointTestInProgress',
  HardPointTesting = 'HardPointTesting',
  HardPointTestCompleted = 'HardPointTestCompleted',
  ManualTestInProgress = 'ManualTestInProgress',
  ManualTesting = 'ManualTesting',
  AlarmTesting = 'AlarmTesting',
  TestCompletedPassed = 'TestCompletedPassed',
  TestCompletedFailed = 'TestCompletedFailed',
  Retesting = 'Retesting'
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
  range_low_limit?: number;
  range_high_limit?: number;
  created_at: string;
  updated_at: string;
}

export interface TestBatchInfo {
  batch_id: string;
  batch_name: string;                    // 批次名称
  product_model?: string;                // 产品型号
  serial_number?: string;                // 序列号
  customer_name?: string;                // 客户名称
  station_name?: string;                 // 站场信息
  operator_name?: string;                // 操作员名称

  // 时间信息
  creation_time: string;                 // 创建时间
  last_updated_time: string;             // 最后更新时间
  test_start_time?: string;              // 测试开始时间
  test_end_time?: string;                // 测试结束时间

  // 统计信息
  total_points: number;                  // 总点位数
  tested_points: number;                 // 已测试点位数
  passed_points: number;                 // 通过点位数
  failed_points: number;                 // 失败点位数
  skipped_points: number;                // 跳过点位数

  // 状态信息
  overall_status: OverallTestStatus;     // 整体状态
  status_summary?: string;               // 状态摘要

  // 兼容性字段（保持向后兼容）
  created_at: string;
  updated_at: string;
}

export interface SubTestExecutionResult {
  status: SubTestStatus;
  details?: string;
  expected_value?: string;
  actual_value?: string;
  timestamp: string;
}

export interface ChannelTestInstance {
  instance_id: string;
  definition_id: string;
  test_batch_id: string;
  test_batch_name?: string;
  overall_status: OverallTestStatus;
  sub_test_results: { [key: string]: SubTestExecutionResult };
  test_plc_channel_tag?: string;
  test_plc_communication_address?: string;
  error_message?: string;
  current_step_details?: string;
  creation_time?: string;
  start_time?: string;
  last_updated_time?: string;
  final_test_time?: string;
  total_test_duration_ms?: number;
  current_operator?: string;
  retries_count?: number;

  // 百分比测试结果字段 - 存储实际工程量
  test_result_0_percent?: number;
  test_result_25_percent?: number;
  test_result_50_percent?: number;
  test_result_75_percent?: number;
  test_result_100_percent?: number;

  // 测试数据字段
  hardpoint_readings?: AnalogReadingPoint[];
  digital_test_steps?: DigitalTestStep[];

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

export interface DigitalTestStep {
  step_number: number;
  step_description: string;
  set_value: boolean;
  expected_reading: boolean;
  actual_reading: boolean;
  status: SubTestStatus;
  timestamp: string;
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
  all_batches: TestBatchInfo[];
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
  [OverallTestStatus.Skipped]: '跳过测试',
  [OverallTestStatus.WiringConfirmationRequired]: '需要接线确认',
  [OverallTestStatus.WiringConfirmed]: '接线已确认',
  [OverallTestStatus.HardPointTestInProgress]: '硬点测试进行中',
  [OverallTestStatus.HardPointTesting]: '硬点测试中',
  [OverallTestStatus.HardPointTestCompleted]: '硬点测试已完成',
  [OverallTestStatus.ManualTestInProgress]: '手动测试进行中',
  [OverallTestStatus.ManualTesting]: '手动测试中',
  [OverallTestStatus.AlarmTesting]: '报警测试中',
  [OverallTestStatus.TestCompletedPassed]: '测试完成并通过',
  [OverallTestStatus.TestCompletedFailed]: '测试完成并失败',
  [OverallTestStatus.Retesting]: '重新测试中'
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

// 新增测试PLC配置模型导出
export * from './test-plc-config.model';

// 准备测试实例相关接口
export interface PrepareTestInstancesRequest {
  batch_id: string;
  definition_ids?: string[]; // 可选的定义ID列表
}

export interface AllocationSummary {
  total_definitions: number;
  allocated_instances: number;
  skipped_definitions: number;
  allocation_errors: string[];
}

export interface PrepareTestInstancesResponse {
  batch_info: TestBatchInfo;
  instances: ChannelTestInstance[];
  definitions: ChannelPointDefinition[];
  allocation_summary: AllocationSummary;
}

// 批次详情载荷
export interface BatchDetailsPayload {
  batch_info: TestBatchInfo;
  instances: ChannelTestInstance[];
  definitions?: ChannelPointDefinition[];
  allocation_summary?: AllocationSummary;
  progress: BatchProgressInfo;
}

// 批次进度信息
export interface BatchProgressInfo {
  total_points: number;
  tested_points: number;
  passed_points: number;
  failed_points: number;
  skipped_points: number;
}

// 通道分配相关接口
export interface ComparisonTable {
  channel_address: string;
  communication_address: string;
  channel_type: ModuleType;
  is_powered: boolean;
}

export interface TestPlcConfig {
  brand_type: string;
  ip_address: string;
  comparison_tables: ComparisonTable[];
}

export interface ModuleTypeStats {
  definition_count: number;
  allocated_count: number;
  batch_count: number;
}

export interface AllocationSummaryDetailed {
  total_definitions: number;
  allocated_instances: number;
  skipped_definitions: number;
  total_channels: number;
  by_module_type: { [key in ModuleType]?: ModuleTypeStats };
}

export interface BatchAllocationResult {
  batches: TestBatchInfo[];
  allocated_instances: ChannelTestInstance[];
  allocation_summary: AllocationSummaryDetailed;
}

export interface ValidationResult {
  is_valid: boolean;
  errors: string[];
  warnings: string[];
}

export interface ImportResult {
  successful_imports: number;
  failed_imports: number;
  total_processed: number;
  errors: string[];
  warnings: string[];
}

export interface AllocationResult {
  success: boolean;
  message: string;
  batch_id: string;
  allocated_count: number;
  conflict_count: number;
  total_count: number;
  total_batches: number;
  batches: TestBatchInfo[];
  allocated_instances: ChannelTestInstance[];
  allocation_summary: AllocationSummaryDetailed;
}

export interface ImportExcelAndCreateBatchResponse {
  batch_info: TestBatchInfo;
  instances: ChannelTestInstance[];
}

// 导入Excel并分配通道的请求
export interface ImportExcelAndAllocateRequest {
  file_path: string;
  product_model?: string;
  serial_number?: string;
}

// 仪表盘批次信息 - 包含是否为当前会话的标识
// 注意：后端使用了 #[serde(flatten)]，所以 TestBatchInfo 的字段都在顶层
export interface DashboardBatchInfo extends TestBatchInfo {
  is_current_session: boolean;  // 是否为当前会话的批次
  has_station_name: boolean;    // 是否有站场名称（用于调试）
}

// 删除批次响应
export interface DeleteBatchResponse {
  success: boolean;
  message: string;
  deleted_definitions_count: number;
  deleted_instances_count: number;
}