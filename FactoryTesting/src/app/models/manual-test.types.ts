// 手动测试相关类型定义
import { OverallTestStatus, SubTestItem, SubTestStatus, ModuleType } from './index';

/**
 * 手动测试子项状态枚举
 */
export enum ManualTestSubItemStatus {
  NotTested = 'NotTested',
  Testing = 'Testing',
  Passed = 'Passed',
  Failed = 'Failed',
  Skipped = 'Skipped'
}

/**
 * 手动测试子项类型
 */
export enum ManualTestSubItem {
  // 通用测试项
  ShowValueCheck = 'ShowValueCheck',        // 显示值核对
  
  // AI点位专用测试项
  LowLowAlarmTest = 'LowLowAlarmTest',      // 低低报警测试
  LowAlarmTest = 'LowAlarmTest',            // 低报警测试
  HighAlarmTest = 'HighAlarmTest',          // 高报警测试
  HighHighAlarmTest = 'HighHighAlarmTest',  // 高高报警测试
  
  // AI/AO点位通用测试项
  MaintenanceFunction = 'MaintenanceFunction' // 维护功能测试
}

/**
 * 手动测试子项结果
 */
export interface ManualTestSubItemResult {
  subItem: ManualTestSubItem;
  status: ManualTestSubItemStatus;
  testTime?: Date;
  operatorNotes?: string;
  skipReason?: string;
}

/**
 * 手动测试状态
 */
export interface ManualTestStatus {
  instanceId: string;
  overallStatus: OverallTestStatus;
  subItemResults: Record<ManualTestSubItem, ManualTestSubItemResult>;
  startTime?: Date;
  completionTime?: Date;
  currentOperator?: string;
}

/**
 * PLC监控数据
 */
export interface PlcMonitoringData {
  instanceId: string;
  timestamp: Date;
  values: Record<string, any>;
}

/**
 * AI点位PLC监控数据
 */
export interface AiPlcMonitoringData extends PlcMonitoringData {
  values: {
    sllSetPoint?: number;    // SLL设定值
    slSetPoint?: number;     // SL设定值
    shSetPoint?: number;     // SH设定值
    shhSetPoint?: number;    // SHH设定值
    currentValue?: number;   // 当前值
  };
}

/**
 * AO点位PLC监控数据
 */
export interface AoPlcMonitoringData extends PlcMonitoringData {
  values: {
    currentOutput?: number;  // 当前输出值
    targetValue?: number;    // 目标值
  };
}

/**
 * DI/DO点位PLC监控数据
 */
export interface DigitalPlcMonitoringData extends PlcMonitoringData {
  values: {
    currentState?: boolean;  // 当前状态
    stateText?: string;      // 状态文本
  };
}

/**
 * 手动测试请求
 */
export interface StartManualTestRequest {
  instanceId: string;
  moduleType: ModuleType;
  operatorName?: string;
}

/**
 * 手动测试响应
 */
export interface StartManualTestResponse {
  success: boolean;
  message?: string;
  testStatus?: ManualTestStatus;
}

/**
 * 更新手动测试子项请求
 */
export interface UpdateManualTestSubItemRequest {
  instanceId: string;
  subItem: ManualTestSubItem;
  status: ManualTestSubItemStatus;
  operatorNotes?: string;
  skipReason?: string;
}

/**
 * 更新手动测试子项响应
 */
export interface UpdateManualTestSubItemResponse {
  success: boolean;
  message?: string;
  testStatus?: ManualTestStatus;
  isCompleted?: boolean;  // 是否所有测试项都已完成
}

/**
 * PLC监控请求
 */
export interface StartPlcMonitoringRequest {
  instanceId: string;
  moduleType: ModuleType;
  monitoringAddresses: string[];
  addressKeyMap?: Record<string, string>;
}

/**
 * PLC监控响应
 */
export interface StartPlcMonitoringResponse {
  success: boolean;
  message?: string;
  monitoringId?: string;
}

/**
 * 停止PLC监控请求
 */
export interface StopPlcMonitoringRequest {
  instanceId: string;
  monitoringId?: string;
}

/**
 * 手动测试配置
 */
export interface ManualTestConfig {
  moduleType: ModuleType;
  applicableSubItems: ManualTestSubItem[];
  plcMonitoringRequired: boolean;
  monitoringInterval: number; // 毫秒
}

/**
 * 获取模块类型对应的手动测试配置
 */
export function getManualTestConfig(moduleType: ModuleType): ManualTestConfig {
  switch (moduleType) {
    case ModuleType.AI:
      return {
        moduleType: ModuleType.AI,
        applicableSubItems: [
          ManualTestSubItem.ShowValueCheck,
          ManualTestSubItem.LowLowAlarmTest,
          ManualTestSubItem.LowAlarmTest,
          ManualTestSubItem.HighAlarmTest,
          ManualTestSubItem.HighHighAlarmTest,
          ManualTestSubItem.MaintenanceFunction
        ],
        plcMonitoringRequired: true,
        monitoringInterval: 500
      };
    
    case ModuleType.AO:
      return {
        moduleType: ModuleType.AO,
        applicableSubItems: [
          ManualTestSubItem.ShowValueCheck,
          ManualTestSubItem.MaintenanceFunction
        ],
        plcMonitoringRequired: true,
        monitoringInterval: 500
      };
    
    case ModuleType.DI:
    case ModuleType.DO:
      return {
        moduleType,
        applicableSubItems: [
          ManualTestSubItem.ShowValueCheck
        ],
        plcMonitoringRequired: true,
        monitoringInterval: 500
      };
    
    default:
      throw new Error(`不支持的模块类型: ${moduleType}`);
  }
}

/**
 * 手动测试子项显示名称映射
 */
export const MANUAL_TEST_SUB_ITEM_LABELS: Record<ManualTestSubItem, string> = {
  [ManualTestSubItem.ShowValueCheck]: '显示值核对',
  [ManualTestSubItem.LowLowAlarmTest]: '低低报警测试',
  [ManualTestSubItem.LowAlarmTest]: '低报警测试',
  [ManualTestSubItem.HighAlarmTest]: '高报警测试',
  [ManualTestSubItem.HighHighAlarmTest]: '高高报警测试',
  [ManualTestSubItem.MaintenanceFunction]: '维护功能测试'
};

/**
 * 手动测试子项状态显示名称映射
 */
export const MANUAL_TEST_SUB_ITEM_STATUS_LABELS: Record<ManualTestSubItemStatus, string> = {
  [ManualTestSubItemStatus.NotTested]: '未测试',
  [ManualTestSubItemStatus.Testing]: '测试中',
  [ManualTestSubItemStatus.Passed]: '通过',
  [ManualTestSubItemStatus.Failed]: '失败',
  [ManualTestSubItemStatus.Skipped]: '跳过'
};

/**
 * 手动测试子项状态颜色映射
 */
export const MANUAL_TEST_SUB_ITEM_STATUS_COLORS: Record<ManualTestSubItemStatus, string> = {
  [ManualTestSubItemStatus.NotTested]: 'default',
  [ManualTestSubItemStatus.Testing]: 'processing',
  [ManualTestSubItemStatus.Passed]: 'success',
  [ManualTestSubItemStatus.Failed]: 'error',
  [ManualTestSubItemStatus.Skipped]: 'warning'
};
