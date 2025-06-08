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
}
