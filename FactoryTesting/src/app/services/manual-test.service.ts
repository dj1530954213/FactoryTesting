import { Injectable } from '@angular/core';
import { Observable, BehaviorSubject, Subject } from 'rxjs';
import { invoke } from '@tauri-apps/api/core';
import {
  StartManualTestRequest,
  StartManualTestResponse,
  UpdateManualTestSubItemRequest,
  UpdateManualTestSubItemResponse,
  ManualTestStatus,
  ManualTestSubItem,
  ManualTestSubItemStatus
} from '../models/manual-test.types';

/**
 * 手动测试服务
 * 负责管理手动测试的状态和与后端的通信
 */
@Injectable({
  providedIn: 'root'
})
export class ManualTestService {
  // 当前手动测试状态
  private currentTestStatus = new BehaviorSubject<ManualTestStatus | null>(null);
  public currentTestStatus$ = this.currentTestStatus.asObservable();

  // 手动测试完成事件
  private testCompleted = new Subject<ManualTestStatus>();
  public testCompleted$ = this.testCompleted.asObservable();

  // 当前是否有活跃的手动测试
  private hasActiveTest = new BehaviorSubject<boolean>(false);
  public hasActiveTest$ = this.hasActiveTest.asObservable();

  constructor() {}

  /**
   * 开始手动测试
   */
  async startManualTest(request: StartManualTestRequest): Promise<StartManualTestResponse> {
    try {
      console.log('🔧 [MANUAL_TEST_SERVICE] 开始手动测试:', request);

      // 检查是否已有活跃的测试
      if (this.hasActiveTest.value) {
        throw new Error('已有活跃的手动测试，请先完成当前测试');
      }

      const response = await invoke<StartManualTestResponse>('start_manual_test_cmd', {
        request
      });

      if (response.success && response.testStatus) {
        this.currentTestStatus.next(response.testStatus);
        this.hasActiveTest.next(true);
        console.log('✅ [MANUAL_TEST_SERVICE] 手动测试已启动:', response.testStatus);
      }

      return response;
    } catch (error) {
      console.error('❌ [MANUAL_TEST_SERVICE] 启动手动测试失败:', error);
      throw new Error(`启动手动测试失败: ${error}`);
    }
  }

  /**
   * 更新手动测试子项状态
   */
  async updateSubItemStatus(request: UpdateManualTestSubItemRequest): Promise<UpdateManualTestSubItemResponse> {
    try {
      console.log('🔧 [MANUAL_TEST_SERVICE] 更新子项状态:', request);

      const response = await invoke<UpdateManualTestSubItemResponse>('update_manual_test_subitem_cmd', {
        request
      });

      if (response.success && response.testStatus) {
        this.currentTestStatus.next(response.testStatus);
        
        console.log('[MANUAL_TEST_SERVICE] 最新 overallStatus:', response.testStatus.overallStatus);
        
        // 如果测试完成，发布完成事件
        if (response.isCompleted) {
          this.testCompleted.next(response.testStatus);
          this.hasActiveTest.next(false);
          console.log('🎉 [MANUAL_TEST_SERVICE] 手动测试已完成:', response.testStatus);
        }
      }

      return response;
    } catch (error) {
      console.error('❌ [MANUAL_TEST_SERVICE] 更新子项状态失败:', error);
      throw new Error(`更新子项状态失败: ${error}`);
    }
  }

  /**
   * 获取手动测试状态
   */
  async getManualTestStatus(instanceId: string): Promise<ManualTestStatus | null> {
    try {
      console.log('🔧 [MANUAL_TEST_SERVICE] 获取手动测试状态:', instanceId);

      const response = await invoke<{ success: boolean; testStatus?: ManualTestStatus }>('get_manual_test_status_cmd', {
        instanceId
      });

      if (response.success && response.testStatus) {
        this.currentTestStatus.next(response.testStatus);
        return response.testStatus;
      }

      return null;
    } catch (error) {
      console.error('❌ [MANUAL_TEST_SERVICE] 获取手动测试状态失败:', error);
      return null;
    }
  }

  /**
   * 完成手动测试子项（标记为通过）
   */
  async completeSubItem(
    instanceId: string, 
    subItem: ManualTestSubItem, 
    operatorNotes?: string
  ): Promise<UpdateManualTestSubItemResponse> {
    return this.updateSubItemStatus({
      instanceId,
      subItem,
      status: ManualTestSubItemStatus.Passed,
      operatorNotes
    });
  }

  /**
   * 跳过手动测试子项
   */
  async skipSubItem(
    instanceId: string, 
    subItem: ManualTestSubItem, 
    skipReason?: string
  ): Promise<UpdateManualTestSubItemResponse> {
    return this.updateSubItemStatus({
      instanceId,
      subItem,
      status: ManualTestSubItemStatus.Skipped,
      skipReason
    });
  }

  /**
   * 标记手动测试子项为失败
   */
  async failSubItem(
    instanceId: string, 
    subItem: ManualTestSubItem, 
    operatorNotes?: string
  ): Promise<UpdateManualTestSubItemResponse> {
    return this.updateSubItemStatus({
      instanceId,
      subItem,
      status: ManualTestSubItemStatus.Failed,
      operatorNotes
    });
  }

  /**
   * 取消当前手动测试
   */
  cancelCurrentTest(): void {
    console.log('🔧 [MANUAL_TEST_SERVICE] 取消当前手动测试');
    this.currentTestStatus.next(null);
    this.hasActiveTest.next(false);
  }

  /**
   * 检查指定子项是否已完成
   */
  isSubItemCompleted(subItem: ManualTestSubItem): boolean {
    const currentStatus = this.currentTestStatus.value;
    if (!currentStatus) return false;

    const subItemResult = currentStatus.subItemResults[subItem];
    return subItemResult && (
      subItemResult.status === ManualTestSubItemStatus.Passed ||
      subItemResult.status === ManualTestSubItemStatus.Skipped
    );
  }

  /**
   * 获取指定子项的状态
   */
  getSubItemStatus(subItem: ManualTestSubItem): ManualTestSubItemStatus {
    const currentStatus = this.currentTestStatus.value;
    if (!currentStatus) return ManualTestSubItemStatus.NotTested;

    const subItemResult = currentStatus.subItemResults[subItem];
    return subItemResult?.status || ManualTestSubItemStatus.NotTested;
  }

  /**
   * 检查所有子项是否都已完成
   */
  areAllSubItemsCompleted(applicableSubItems: ManualTestSubItem[]): boolean {
    return applicableSubItems.every(subItem => this.isSubItemCompleted(subItem));
  }

  /**
   * 获取已完成的子项数量
   */
  getCompletedSubItemsCount(applicableSubItems: ManualTestSubItem[]): number {
    return applicableSubItems.filter(subItem => this.isSubItemCompleted(subItem)).length;
  }

  /**
   * 重置服务状态（用于组件销毁时清理）
   */
  reset(): void {
    console.log('🔧 [MANUAL_TEST_SERVICE] 重置服务状态');
    this.currentTestStatus.next(null);
    this.hasActiveTest.next(false);
  }

  // ==================== AI手动测试专用方法 ====================

  /**
   * 生成随机显示值
   */
  async generateRandomDisplayValue(instanceId: string): Promise<{ success: boolean; randomValue: number; message?: string }> {
    try {
      console.log('🔧 [MANUAL_TEST_SERVICE] 生成随机显示值:', instanceId);

      const response = await invoke<{ success: boolean; random_value: number; message?: string }>('generate_random_display_value_cmd', {
        request: {
          instance_id: instanceId
        }
      });

      console.log('✅ [MANUAL_TEST_SERVICE] 随机值生成结果:', response);
      return {
        success: response.success,
        randomValue: response.random_value,
        message: response.message
      };
    } catch (error) {
      console.error('❌ [MANUAL_TEST_SERVICE] 生成随机显示值失败:', error);
      throw new Error(`生成随机显示值失败: ${error}`);
    }
  }

  /**
   * 执行显示值核对测试
   */
  async executeShowValueTest(instanceId: string, testValue: number): Promise<{ success: boolean; message?: string; sentPercentage?: number; testPlcAddress?: string }> {
    try {
      console.log('🔧 [MANUAL_TEST_SERVICE] 执行显示值核对测试:', { instanceId, testValue });

      const response = await invoke<{ success: boolean; message?: string; sent_percentage?: number; test_plc_address?: string }>('ai_show_value_test_cmd', {
        request: {
          instance_id: instanceId,
          test_value: testValue
        }
      });

      console.log('✅ [MANUAL_TEST_SERVICE] 显示值测试结果:', response);
      return {
        success: response.success,
        message: response.message,
        sentPercentage: response.sent_percentage,
        testPlcAddress: response.test_plc_address
      };
    } catch (error) {
      console.error('❌ [MANUAL_TEST_SERVICE] 显示值测试失败:', error);
      throw new Error(`显示值测试失败: ${error}`);
    }
  }

  /**
   * 执行报警测试
   */
  async executeAlarmTest(instanceId: string, alarmType: string): Promise<{ success: boolean; message?: string; sentValue?: number; sentPercentage?: number; testPlcAddress?: string }> {
    try {
      console.log('🔧 [MANUAL_TEST_SERVICE] 执行报警测试:', { instanceId, alarmType });

      const response = await invoke<{ success: boolean; message?: string; sent_value?: number; sent_percentage?: number; test_plc_address?: string }>('ai_alarm_test_cmd', {
        request: {
          instance_id: instanceId,
          alarm_type: alarmType
        }
      });

      console.log('✅ [MANUAL_TEST_SERVICE] 报警测试结果:', response);
      return {
        success: response.success,
        message: response.message,
        sentValue: response.sent_value,
        sentPercentage: response.sent_percentage,
        testPlcAddress: response.test_plc_address
      };
    } catch (error) {
      console.error('❌ [MANUAL_TEST_SERVICE] 报警测试失败:', error);
      throw new Error(`报警测试失败: ${error}`);
    }
  }

  /**
   * 执行维护功能测试
   */
  async executeMaintenanceTest(instanceId: string, enable: boolean): Promise<{ success: boolean; message?: string; maintenanceAddress?: string }> {
    try {
      console.log('🔧 [MANUAL_TEST_SERVICE] 执行维护功能测试:', { instanceId, enable });

      const response = await invoke<{ success: boolean; message?: string; maintenance_address?: string }>('ai_maintenance_test_cmd', {
        request: {
          instance_id: instanceId,
          enable: enable
        }
      });

      console.log('✅ [MANUAL_TEST_SERVICE] 维护功能测试结果:', response);
      return {
        success: response.success,
        message: response.message,
        maintenanceAddress: response.maintenance_address
      };
    } catch (error) {
      console.error('❌ [MANUAL_TEST_SERVICE] 维护功能测试失败:', error);
      throw new Error(`维护功能测试失败: ${error}`);
    }
  }

  // ==================== DI 手动测试 ====================

  /**
   * 执行 DI 信号下发 / 复位 测试
   */
  async executeDiSignalTest(instanceId: string, enable: boolean): Promise<{ success: boolean; message?: string; testPlcAddress?: string }> {
    try {
      console.log('🔧 [MANUAL_TEST_SERVICE] 执行DI信号测试:', { instanceId, enable });

      const response = await invoke<{ success: boolean; message?: string; test_plc_address?: string }>('di_signal_test_cmd', {
        request: {
          instance_id: instanceId,
          enable: enable
        }
      });

      console.log('✅ [MANUAL_TEST_SERVICE] DI信号测试结果:', response);
      return {
        success: response.success,
        message: response.message,
        testPlcAddress: response.test_plc_address
      };
    } catch (error) {
      console.error('❌ [MANUAL_TEST_SERVICE] DI信号测试失败:', error);
      throw new Error(`DI信号测试失败: ${error}`);
    }
  }
}
