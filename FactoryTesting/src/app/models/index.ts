/**
 * # 工厂测试系统模型定义 - Models Index
 * 
 * ## 业务功能说明
 * - 定义工厂测试系统的核心数据模型和枚举
 * - 提供前后端数据交换的接口规范
 * - 支持工厂自动化测试的完整业务流程
 * - 包含中文标签映射，支持国际化显示
 * 
 * ## 前后端调用链
 * - **数据模型**: 与Rust后端的struct保持一致
 * - **枚举定义**: 对应后端的enum类型
 * - **接口规范**: API调用的请求和响应格式
 * 
 * ## Angular知识点
 * - **TypeScript接口**: 类型安全的数据定义
 * - **枚举类型**: 常量值的类型安全管理
 * - **模块导出**: barrel pattern统一导出
 * 
 * ## 业务领域模型
 * - **测试批次**: 工厂测试的基本单位
 * - **通道定义**: PLC点位的配置信息
 * - **测试实例**: 具体的测试执行记录
 * - **测试结果**: 测试过程和结果数据
 */

// ============================================================================
// 枚举类型定义 - 业务常量
// ============================================================================

/**
 * 模块类型枚举
 * 
 * **业务含义**: 工厂自动化中的I/O模块类型
 * **应用场景**: 区分不同类型的PLC模块和信号处理方式
 */
export enum ModuleType {
  AI = 'AI',        // 模拟量输入模块
  AO = 'AO',        // 模拟量输出模块
  DI = 'DI',        // 数字量输入模块
  DO = 'DO',        // 数字量输出模块
  AINone = 'AINone', // 无模拟量输入
  DINone = 'DINone'  // 无数字量输入
}

/**
 * 点位数据类型枚举
 * 
 * **业务含义**: PLC点位的数据类型定义
 * **技术用途**: 确定数据的读写方式和值域范围
 */
export enum PointDataType {
  Bool = 'Bool',     // 布尔型（开关量）
  Int = 'Int',       // 整数型
  Float = 'Float',   // 浮点型（模拟量）
  String = 'String'  // 字符串型
}

/**
 * 整体测试状态枚举
 * 
 * **业务含义**: 测试实例在整个测试流程中的状态
 * **状态流转**: 从未测试 → 接线确认 → 硬点测试 → 手动测试 → 报警测试 → 完成
 */
export enum OverallTestStatus {
  NotTested = 'NotTested',                           // 未测试
  Skipped = 'Skipped',                              // 跳过测试
  WiringConfirmationRequired = 'WiringConfirmationRequired', // 需要接线确认
  WiringConfirmed = 'WiringConfirmed',              // 接线已确认
  HardPointTestInProgress = 'HardPointTestInProgress',       // 硬点测试进行中
  HardPointTesting = 'HardPointTesting',            // 硬点测试中
  HardPointTestCompleted = 'HardPointTestCompleted', // 硬点测试已完成
  ManualTestInProgress = 'ManualTestInProgress',     // 手动测试进行中
  ManualTesting = 'ManualTesting',                  // 手动测试中
  AlarmTesting = 'AlarmTesting',                    // 报警测试中
  TestCompletedPassed = 'TestCompletedPassed',     // 测试完成并通过
  TestCompletedFailed = 'TestCompletedFailed',     // 测试完成并失败
  Retesting = 'Retesting'                          // 重新测试中
}

/**
 * 子测试项目枚举
 * 
 * **业务含义**: 每个通道测试包含的具体测试项目
 * **测试流程**: 硬点测试 → 报警测试 → 状态显示测试
 */
export enum SubTestItem {
  HardPoint = 'HardPoint',         // 硬点测试（基础功能测试）
  LowLowAlarm = 'LowLowAlarm',     // 低低报警测试
  LowAlarm = 'LowAlarm',           // 低报警测试
  HighAlarm = 'HighAlarm',         // 高报警测试
  HighHighAlarm = 'HighHighAlarm', // 高高报警测试
  StateDisplay = 'StateDisplay'    // 状态显示测试
}

/**
 * 子测试状态枚举
 * 
 * **业务含义**: 单个测试项目的执行状态
 * **状态流转**: 未开始 → 进行中 → 通过/失败/跳过
 */
export enum SubTestStatus {
  NotStarted = 'NotStarted', // 未开始
  InProgress = 'InProgress', // 进行中
  Passed = 'Passed',         // 通过
  Failed = 'Failed',         // 失败
  Skipped = 'Skipped'        // 跳过
}

// ============================================================================
// 核心数据模型 - 业务实体
// ============================================================================

/**
 * 通道点位定义接口
 * 
 * **业务含义**: Excel点表中每一行的完整定义信息
 * **数据来源**: Excel导入解析后的结构化数据
 * **用途**: 
 * - 定义PLC点位的基本属性和通信地址
 * - 配置测试参数和报警设定值
 * - 建立测试实例的基础数据
 */
export interface ChannelPointDefinition {
  id: string;
  sequenceNumber?: number;
  tag: string;
  variable_name: string;
  description: string;
  /** 与后端结构保持一致的可选别名 */
  variable_description?: string;
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
  // 报警设定值通信地址（用于手动测试）
  sll_set_point_communication_address?: string;
  sl_set_point_communication_address?: string;
  sh_set_point_communication_address?: string;
  shh_set_point_communication_address?: string;
  sll_set_point_plc_address?: string;
  sl_set_point_plc_address?: string;
  sh_set_point_plc_address?: string;
  shh_set_point_plc_address?: string;
  // 报警设定固定值（导入表格中的设定值）
  sll_set_value?: number;
  sl_set_value?: number;
  sh_set_value?: number;
  shh_set_value?: number;
  created_at: string;
  updated_at: string;
}

/**
 * 测试批次信息接口
 * 
 * **业务含义**: 工厂测试的基本单位，包含一组相关的测试点位
 * **生命周期**: 创建 → 分配测试实例 → 执行测试 → 完成/失败
 * **数据来源**: Excel导入时自动创建，或手动创建
 * **用途**:
 * - 组织和管理测试点位
 * - 跟踪测试进度和统计信息
 * - 提供测试报告的基础数据
 */
export interface TestBatchInfo {
  // === 基本信息 ===
  batch_id: string;                      // 批次唯一标识
  batch_name: string;                    // 批次显示名称
  product_model?: string;                // 被测产品型号
  serial_number?: string;                // 产品序列号
  customer_name?: string;                // 客户名称
  station_name?: string;                 // 工厂站场信息
  operator_name?: string;                // 测试操作员

  // === 时间信息 ===
  creation_time: string;                 // 批次创建时间
  last_updated_time: string;             // 最后更新时间
  test_start_time?: string;              // 测试开始时间
  test_end_time?: string;                // 测试结束时间

  // === 统计信息 ===
  total_points: number;                  // 总点位数量
  tested_points: number;                 // 已测试点位数
  passed_points: number;                 // 测试通过点位数
  failed_points: number;                 // 测试失败点位数
  skipped_points: number;                // 跳过测试点位数
  started_points?: number;               // 已开始测试点位数（包括中间状态）

  // === 状态信息 ===
  overall_status: OverallTestStatus;     // 批次整体状态
  status_summary?: string;               // 状态文字摘要

  // === 兼容性字段 ===
  created_at: string;                    // 创建时间（备用）
  updated_at: string;                    // 更新时间（备用）
}

export interface SubTestExecutionResult {
  status: SubTestStatus;
  details?: string;
  expected_value?: string;
  actual_value?: string;
  timestamp: string;
}

/**
 * 通道测试实例接口
 * 
 * **业务含义**: 基于通道定义创建的具体测试执行单元
 * **生命周期**: 创建 → 分配PLC通道 → 执行测试 → 记录结果
 * **数据关系**: 一个ChannelPointDefinition对应一个ChannelTestInstance
 * **用途**:
 * - 跟踪单个点位的测试执行状态
 * - 存储测试过程数据和结果
 * - 支持测试重试和错误记录
 */
export interface ChannelTestInstance {
  // === 基本信息 ===
  instance_id: string;                     // 测试实例唯一标识
  definition_id: string;                   // 关联的通道定义ID
  test_batch_id: string;                   // 所属测试批次ID
  test_batch_name?: string;                // 批次名称（冗余字段）
  overall_status: OverallTestStatus;       // 实例整体状态

  // === 测试结果 ===
  sub_test_results: { [key: string]: SubTestExecutionResult }; // 子测试结果集合

  // === PLC配置信息 ===
  test_plc_channel_tag?: string;           // 分配的测试PLC通道标签
  test_plc_communication_address?: string; // 测试PLC通信地址

  // === 状态和错误信息 ===
  error_message?: string;                  // 测试错误消息
  current_step_details?: string;          // 当前步骤详情

  // === 时间信息 ===
  creation_time?: string;                  // 实例创建时间
  start_time?: string;                     // 测试开始时间
  last_updated_time?: string;              // 最后更新时间
  final_test_time?: string;                // 测试完成时间
  total_test_duration_ms?: number;         // 总测试耗时（毫秒）

  // === 操作信息 ===
  current_operator?: string;               // 当前操作员
  retries_count?: number;                  // 重试次数

  // === 百分比测试结果 ===
  // **业务含义**: 模拟量测试时的工程量读数
  test_result_0_percent?: number;          // 0%量程时的读数
  test_result_25_percent?: number;         // 25%量程时的读数
  test_result_50_percent?: number;         // 50%量程时的读数
  test_result_75_percent?: number;         // 75%量程时的读数
  test_result_100_percent?: number;        // 100%量程时的读数

  // === 错误备注字段 ===
  // **业务用途**: 人工记录测试失败的具体原因，便于问题追溯
  integration_error_notes?: string;        // 集成测试错误备注
  plc_programming_error_notes?: string;    // PLC编程相关错误备注
  hmi_configuration_error_notes?: string;  // 上位机组态错误备注

  // === 测试数据 ===
  hardpoint_readings?: AnalogReadingPoint[]; // 模拟量硬点测试读数
  digital_test_steps?: DigitalTestStep[];    // 数字量测试步骤

  // === 时间戳 ===
  created_at: string;                      // 创建时间戳
  updated_at: string;                      // 更新时间戳
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

// ============================================================================
// 中文标签常量映射 - 国际化支持
// ============================================================================

/**
 * 模块类型中文标签映射
 * 
 * **业务用途**: 在UI中显示用户友好的中文标签
 * **技术实现**: 使用TypeScript的映射类型确保完整性
 */
export const MODULE_TYPE_LABELS: { [key in ModuleType]: string } = {
  [ModuleType.AI]: '模拟量输入',
  [ModuleType.AO]: '模拟量输出',
  [ModuleType.DI]: '数字量输入', 
  [ModuleType.DO]: '数字量输出',
  [ModuleType.AINone]: '模拟量输入(无)',
  [ModuleType.DINone]: '数字量输入(无)'
};

/**
 * 点位数据类型中文标签映射
 * 
 * **业务用途**: 数据类型的中文显示名称
 */
export const POINT_DATA_TYPE_LABELS: { [key in PointDataType]: string } = {
  [PointDataType.Bool]: '布尔型',
  [PointDataType.Int]: '整数型',
  [PointDataType.Float]: '浮点型',
  [PointDataType.String]: '字符串型'
};

/**
 * 整体测试状态中文标签映射
 * 
 * **业务用途**: 测试状态的中文显示，便于操作员理解
 * **设计考虑**: 标签简洁明了，体现测试流程的进展
 */
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

// 整体站场测试进度接口（精简版）
export interface OverallStationProgress {
  // 核心点位统计
  totalPoints: number;            // 总点位数
  testedPoints: number;           // 已测试点位数  
  pendingPoints: number;          // 待测试点位数
  successPoints: number;          // 成功点位数
  failedPoints: number;           // 失败点位数
  
  // 核心进度计算
  progressPercentage: number;     // 总体进度百分比（最醒目显示）
}