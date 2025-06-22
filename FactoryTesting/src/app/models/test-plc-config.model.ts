// 测试PLC通道配置相关数据模型

/**
 * 通道类型枚举 - 对应数据库中的ChannelType字段
 */
export enum TestPlcChannelType {
  AI = 0,      // 模拟量输入
  AO = 1,      // 模拟量输出
  DI = 2,      // 数字量输入
  DO = 3,      // 数字量输出
  AINone = 4,  // 模拟量输入(无源)
  AONone = 5,  // 模拟量输出(无源)
  DINone = 6,  // 数字量输入(无源)
  DONone = 7   // 数字量输出(无源)
}

/**
 * 测试PLC通道配置表 - 对应数据库ComparisonTables表
 */
export interface TestPlcChannelConfig {
  id?: string;                           // 主键ID
  channelAddress: string;                // 通道位号 (如: AI1_1, AO1_2)
  channelType: TestPlcChannelType;       // 通道类型 (0-7)
  communicationAddress: string;          // 通讯地址 (如: 40101, 00101)
  powerSupplyType: string;               // 供电类型 (必填项)
  description?: string;                  // 描述信息
  isEnabled: boolean;                    // 是否启用
  createdAt?: Date;                      // 创建时间
  updatedAt?: Date;                      // 更新时间
}

/**
 * PLC连接配置
 */
export interface PlcConnectionConfig {
  id: string;                            // 配置ID
  name: string;                          // 配置名称
  plcType: PlcType;                      // PLC类型
  ipAddress: string;                     // IP地址
  port: number;                          // 端口号
  timeout: number;                       // 超时时间(ms)
  retryCount: number;                    // 重试次数
  /** 字节顺序 (ABCD / CDAB / BADC / DCBA) */
  byteOrder?: ByteOrder;                  // 字节顺序
  /** 地址是否从0开始(true表示用户输入为0基址) */
  zeroBasedAddress?: boolean;             // 0基地址开关
  isTestPlc: boolean;                    // 是否为测试PLC
  description?: string;                  // 描述
  isEnabled: boolean;                    // 是否启用
  lastConnected?: Date;                  // 最后连接时间
  connectionStatus: ConnectionStatus;    // 连接状态
}

/**
 * PLC类型枚举
 */
export enum PlcType {
  ModbusTcp = 'ModbusTcp',
  SiemensS7 = 'SiemensS7',
  OpcUa = 'OpcUa'
}

/**
 * 连接状态枚举
 */
export enum ConnectionStatus {
  Disconnected = 'Disconnected',        // 未连接
  Connecting = 'Connecting',             // 连接中
  Connected = 'Connected',               // 已连接
  Error = 'Error',                       // 连接错误
  Timeout = 'Timeout'                    // 连接超时
}

/**
 * 通道映射配置 - 用于将被测PLC通道映射到测试PLC通道
 */
export interface ChannelMappingConfig {
  id: string;                            // 映射ID
  targetChannelId: string;               // 被测通道ID (ChannelPointDefinition.id)
  testPlcChannelId: string;              // 测试PLC通道ID (TestPlcChannelConfig.id)
  mappingType: MappingType;              // 映射类型
  isActive: boolean;                     // 是否激活
  notes?: string;                        // 备注
  createdAt: Date;                       // 创建时间
}

/**
 * 映射类型枚举
 */
export enum MappingType {
  Direct = 'Direct',                     // 直接映射
  Inverse = 'Inverse',                   // 反向映射
  Scaled = 'Scaled',                     // 比例映射
  Custom = 'Custom'                      // 自定义映射
}

/**
 * 映射策略枚举
 */
export enum MappingStrategy {
  ByChannelType = 'ByChannelType',      // 按通道类型匹配
  Sequential = 'Sequential',             // 顺序分配
  LoadBalanced = 'LoadBalanced'          // 负载均衡
}

/**
 * 自动生成通道映射请求
 */
export interface GenerateChannelMappingsRequest {
  targetChannelIds: string[];            // 目标通道ID列表
  strategy: MappingStrategy;             // 映射策略
}

/**
 * 自动生成通道映射响应
 */
export interface GenerateChannelMappingsResponse {
  success: boolean;                      // 是否成功
  message: string;                       // 响应消息
  mappings: ChannelMappingConfig[];      // 生成的映射配置
  conflicts: string[];                   // 冲突列表
}

/**
 * 测试PLC连接响应
 */
export interface TestPlcConnectionResponse {
  success: boolean;                      // 连接是否成功
  message: string;                       // 响应消息
  connectionTimeMs?: number;             // 连接时间(毫秒)
}

/**
 * 地址读取测试响应
 */
export interface AddressReadTestResponse {
  success: boolean;                      // 读取是否成功
  value?: any;                          // 读取到的值
  error?: string;                       // 错误信息
  readTimeMs?: number;                  // 读取时间(毫秒)
}

/**
 * 通道类型标签映射
 */
export const TestPlcChannelTypeLabels: Record<TestPlcChannelType, string> = {
  [TestPlcChannelType.AI]: 'AI - 模拟量输入',
  [TestPlcChannelType.AO]: 'AO - 模拟量输出',
  [TestPlcChannelType.DI]: 'DI - 数字量输入',
  [TestPlcChannelType.DO]: 'DO - 数字量输出',
  [TestPlcChannelType.AINone]: 'AI无源 - 模拟量输入(无源)',
  [TestPlcChannelType.AONone]: 'AO无源 - 模拟量输出(无源)',
  [TestPlcChannelType.DINone]: 'DI无源 - 数字量输入(无源)',
  [TestPlcChannelType.DONone]: 'DO无源 - 数字量输出(无源)'
};

/**
 * PLC类型标签映射
 */
export const PlcTypeLabels: Record<PlcType, string> = {
  [PlcType.ModbusTcp]: 'Modbus TCP',
  [PlcType.SiemensS7]: 'Siemens S7',
  [PlcType.OpcUa]: 'OPC UA'
};

/**
 * 连接状态标签映射
 */
export const ConnectionStatusLabels: Record<ConnectionStatus, string> = {
  [ConnectionStatus.Disconnected]: '未连接',
  [ConnectionStatus.Connecting]: '连接中',
  [ConnectionStatus.Connected]: '已连接',
  [ConnectionStatus.Error]: '连接错误',
  [ConnectionStatus.Timeout]: '连接超时'
};

/**
 * 获取通道类型的颜色标识
 */
export function getChannelTypeColor(channelType: TestPlcChannelType): string {
  switch (channelType) {
    case TestPlcChannelType.AI:
    case TestPlcChannelType.AINone:
      return 'blue';
    case TestPlcChannelType.AO:
    case TestPlcChannelType.AONone:
      return 'green';
    case TestPlcChannelType.DI:
    case TestPlcChannelType.DINone:
      return 'orange';
    case TestPlcChannelType.DO:
    case TestPlcChannelType.DONone:
      return 'red';
    default:
      return 'default';
  }
}

/**
 * 获取连接状态的颜色标识
 */
export function getConnectionStatusColor(status: ConnectionStatus): string {
  switch (status) {
    case ConnectionStatus.Connected:
      return 'green';
    case ConnectionStatus.Connecting:
      return 'blue';
    case ConnectionStatus.Error:
    case ConnectionStatus.Timeout:
      return 'red';
    case ConnectionStatus.Disconnected:
    default:
      return 'default';
  }
}

/** 字节顺序枚举 */
export enum ByteOrder {
  ABCD = 'ABCD',
  CDAB = 'CDAB',
  BADC = 'BADC',
  DCBA = 'DCBA'
} 