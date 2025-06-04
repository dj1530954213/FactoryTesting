/**
 * PLC连接状态模型
 */

/**
 * PLC连接状态信息
 */
export interface PlcConnectionStatus {
  testPlcConnected: boolean;      // 测试PLC连接状态
  targetPlcConnected: boolean;    // 被测PLC连接状态
  testPlcName?: string;           // 测试PLC名称
  targetPlcName?: string;         // 被测PLC名称
  lastCheckTime: string;          // 最后检查时间
}

/**
 * PLC连接状态枚举
 */
export enum PlcConnectionState {
  Connected = 'connected',
  Disconnected = 'disconnected',
  Connecting = 'connecting',
  Error = 'error'
}

/**
 * PLC类型枚举
 */
export enum PlcType {
  TestPlc = 'test',
  TargetPlc = 'target'
}
