import { Component, Input, Output, EventEmitter, OnInit, OnDestroy, OnChanges, SimpleChanges, ChangeDetectorRef } from '@angular/core';
import { CommonModule } from '@angular/common';
import { FormsModule } from '@angular/forms';
import { NzCardModule } from 'ng-zorro-antd/card';
import { NzButtonModule } from 'ng-zorro-antd/button';
import { NzTagModule } from 'ng-zorro-antd/tag';
import { NzIconModule } from 'ng-zorro-antd/icon';
import { NzMessageService } from 'ng-zorro-antd/message';
import { NzModalService } from 'ng-zorro-antd/modal';
import { NzDividerModule } from 'ng-zorro-antd/divider';
import { NzStatisticModule } from 'ng-zorro-antd/statistic';
import { invoke } from '@tauri-apps/api/core';
import { Subscription } from 'rxjs';

import { ChannelTestInstance, ChannelPointDefinition, OverallTestStatus } from '../../models';
import { ManualTestService } from '../../services/manual-test.service';
import { PlcMonitoringService } from '../../services/plc-monitoring.service';
import {
  ManualTestStatus,
  ManualTestSubItem,
  ManualTestSubItemStatus,
  MANUAL_TEST_SUB_ITEM_LABELS,
  MANUAL_TEST_SUB_ITEM_STATUS_COLORS,
  getManualTestConfig
} from '../../models/manual-test.types';

/**
 * DO点位手动测试组件
 * 包含1个测试项：显示值核对 + 数字状态采集功能（低-高-低电平）
 */
@Component({
  selector: 'app-do-manual-test',
  standalone: true,
  imports: [
    CommonModule,
    FormsModule,
    NzCardModule,
    NzButtonModule,
    NzTagModule,
    NzIconModule,
    NzDividerModule,
    NzStatisticModule
  ],
  template: `
    <div class="do-manual-test">
      
      <!-- 实时监控数据显示 -->
      <nz-card nzTitle="实时监控数据" nzSize="small" class="monitoring-card">
        <div class="monitoring-grid">
          <nz-statistic 
            nzTitle="当前状态" 
            [nzValue]="getCurrentState()" 
            [nzValueStyle]="{ color: getCurrentStateColor() }">
          </nz-statistic>
        </div>
      </nz-card>

      <nz-divider></nz-divider>

      <!-- DO 状态采集按钮 -->
      <nz-card nzTitle="采集数字状态测试" nzSize="small" class="capture-card">
        
        <!-- 测试完成状态提示 -->
        <div *ngIf="isStateCollectionCompleted()" 
             style="margin-bottom: 12px; padding: 8px 12px; background-color: #f6ffed; border: 1px solid #b7eb8f; border-radius: 4px; display: flex; align-items: center; gap: 8px; color: #52c41a; font-size: 12px;">
          <i nz-icon nzType="check-circle" nzTheme="twotone" [nzTwotoneColor]="'#52c41a'"></i>
          <span>状态采集已完成，采集按钮已禁用以保护数据一致性</span>
        </div>

        <div class="capture-buttons">
          <button *ngFor="let state of digitalStates; let i = index"
                  nz-button
                  [nzType]="getButtonType(state, i)"
                  [class.completed-state]="isStateCompleted(state, i)"
                  [disabled]="isStateButtonDisabled(state, i) || isStateCollectionCompleted()"
                  [nzLoading]="isCapturing && currentCapturingState === state"
                  (click)="captureDigitalState(state, i)"
                  [title]="getStateButtonTooltip(state, i)">
            <span class="button-text">{{ getStateLabel(state, i) }}</span>
            <span *ngIf="getStateResult(state, i)" class="state-text">
              采集值: {{ getStateResult(state, i).actualValue }}
            </span>
          </button>
        </div>

        <!-- 采集结果汇总 -->
        <div *ngIf="hasAnyStateResults()" class="capture-summary">
          <nz-divider nzText="采集结果汇总" nzOrientation="left"></nz-divider>
          <div class="summary-grid">
            <div *ngFor="let state of digitalStates; let i = index" class="summary-item" [class.completed]="isStateCompleted(state, i)">
              <span class="summary-label">{{ getStateLabel(state, i) }}:</span>
              <span *ngIf="getStateResult(state, i)" class="summary-value">
                {{ getStateResult(state, i).actualValue }} ({{ getStateResult(state, i).timestamp | date:'HH:mm:ss' }})
              </span>
              <span *ngIf="!getStateResult(state, i)" class="summary-pending">待采集</span>
            </div>
          </div>
        </div>
      </nz-card>

      <nz-divider></nz-divider>

      <!-- 手动测试项列表 -->
      <div class="test-items-section">
        <h4>手动测试项目</h4>
        <div class="test-items-grid">
          
          <!-- 显示值核对 -->
          <nz-card nzSize="small" class="test-item-card">
            <div class="test-item-header">
              <span class="test-item-title">显示值核对</span>
              <nz-tag [nzColor]="getSubItemStatusColor(ManualTestSubItem.ShowValueCheck)">
                {{ getSubItemStatusText(ManualTestSubItem.ShowValueCheck) }}
              </nz-tag>
            </div>
            <div class="test-item-content">
              <p>请确认HMI界面显示的状态与实际输出状态一致</p>
              <p>当前状态: <strong>{{ getCurrentState() }}</strong></p>

              <!-- 状态采集完成检查提示 -->
              <div *ngIf="!areAllStatesCollected() && !isSubItemPassedOrSkipped(ManualTestSubItem.ShowValueCheck)" 
                   style="margin: 8px 0; padding: 8px 12px; background-color: #fffbe6; border: 1px solid #fadb14; border-radius: 4px; display: flex; align-items: center; gap: 8px; color: #faad14; font-size: 12px;">
                <i nz-icon nzType="exclamation-circle" nzTheme="twotone" [nzTwotoneColor]="'#faad14'"></i>
                <span>确认通过需要先完成所有状态（低-高-低电平）的数据采集</span>
              </div>

              <div class="test-item-actions">
                <button 
                  nz-button 
                  nzType="primary" 
                  nzSize="small"
                  [disabled]="isConfirmButtonDisabled(ManualTestSubItem.ShowValueCheck)"
                  [title]="getConfirmButtonTooltip()"
                  (click)="completeSubItem(ManualTestSubItem.ShowValueCheck)">
                  <i nz-icon nzType="check"></i>
                  确认通过
                </button>
                <button 
                  nz-button 
                  nzSize="small"
                  [disabled]="isSubItemPassedOrSkipped(ManualTestSubItem.ShowValueCheck)"
                  (click)="skipSubItem(ManualTestSubItem.ShowValueCheck)">
                  <i nz-icon nzType="close"></i>
                  测试失败
                </button>
              </div>
            </div>
          </nz-card>

        </div>
      </div>

      <nz-divider></nz-divider>

      <!-- 测试进度 -->
      <div class="test-progress-section">
        <span>测试进度: {{ getCompletedCount() }} / {{ getTotalCount() }}</span>
      </div>

    </div>
  `,
  styleUrls: ['./ai-manual-test.component.css'],
  styles: [`
    /* DO组件特定样式 - 绿色的已完成按钮 */
    button[nz-button].completed-state {
      background-color: #52c41a !important;
      border-color: #52c41a !important;
      color: white !important;
    }
    
    button[nz-button].completed-state:hover {
      background-color: #73d13d !important;
      border-color: #73d13d !important;
    }
    
    button[nz-button].completed-state:focus {
      background-color: #52c41a !important;
      border-color: #73d13d !important;
      box-shadow: 0 0 0 2px rgba(82, 196, 26, 0.2) !important;
    }
    
    /* 等待状态按钮样式调整 */
    button[nz-button][nzType="text"]:disabled {
      color: #bfbfbf !important;
      background-color: #f5f5f5 !important;
      border-color: #d9d9d9 !important;
    }
  `]
})
export class DoManualTestComponent implements OnInit, OnDestroy, OnChanges {
  @Input() instance: ChannelTestInstance | null = null;
  @Input() definition: ChannelPointDefinition | null = null;
  @Input() testStatus: ManualTestStatus | null = null;

  @Output() testCompleted = new EventEmitter<void>();
  @Output() testCancelled = new EventEmitter<void>();

  // 测试配置
  private testConfig = getManualTestConfig('DO' as any);
  
  // 订阅管理
  private subscriptions = new Subscription();

  // DO 数字状态采集相关
  digitalStates: string[] = ['low', 'high', 'low']; // 低-高-低电平序列
  stateResults: Record<string, { actualValue: boolean | string; timestamp: Date; stepNumber: number }> = {};
  isCapturing: boolean = false;
  currentCapturingState: string | null = null;

  // 枚举引用（用于模板）
  ManualTestSubItem = ManualTestSubItem;

  constructor(
    private manualTestService: ManualTestService,
    private plcMonitoringService: PlcMonitoringService,
    private message: NzMessageService,
    private modal: NzModalService,
    private cdr: ChangeDetectorRef
  ) {}

  // 防止重复触发完成事件
  private completedEmitted = false;
  // 状态初始化标志
  private statusInitialized = false;
  private previousCompleted = false;
  /** 用户是否已点击"确认通过" */
  private confirmClicked = false;

  // ============== 状态恢复 ==============
  /**
   * 根据 instance 中的 digital_test_steps_json 字段恢复数字状态采集进度
   */
  private restoreDigitalState(): void {
    // 每次进入窗口重置确认标记
    this.confirmClicked = false;
    
    // 清空现有状态结果，避免残留数据影响
    this.stateResults = {};
    
    if (!this.instance?.digital_test_steps_json) return;

    try {
      const digitalSteps = JSON.parse(this.instance.digital_test_steps_json);
      
      digitalSteps.forEach((step: any) => {
        // 更严格的验证：确保有步骤号且有实际采集的数据
        if (step.step_number && step.actual_reading !== undefined && step.actual_reading !== null) {
          const resultKey = `step_${step.step_number}`;
          this.stateResults[resultKey] = {
            actualValue: step.actual_reading,
            timestamp: new Date(step.timestamp || Date.now()),
            stepNumber: step.step_number
          };
        }
      });

      // 触发变更检测，确保UI立即刷新
      this.cdr.markForCheck();
    } catch (error) {
      console.warn('恢复DO数字状态失败:', error);
      // 出错时清空状态结果
      this.stateResults = {};
    }
  }

  /**
   * 根据状态和索引获取步骤号
   */
  private getStepNumberByStateAndIndex(state: string, index: number): number {
    // low-high-low 序列：索引0是第1步，索引1是第2步，索引2是第3步
    return index + 1;
  }

  ngOnChanges(changes: SimpleChanges): void {
    if (changes['instance'] && changes['instance'].currentValue) {
      this.restoreDigitalState();
    }
  }

  ngOnInit(): void {
    // 根据已存在的数据恢复采集状态
    this.restoreDigitalState();

    // 订阅PLC监控数据，保持界面刷新
    this.subscriptions.add(
      this.plcMonitoringService.currentMonitoringData$.subscribe(() => {
        // 监控数据更新后手动触发变更检测，确保界面刷新
        this.cdr.markForCheck();
      })
    );

    // 订阅手动测试状态变化，自动检测是否完成
    this.subscriptions.add(
      this.manualTestService.currentTestStatus$.subscribe(status => {
        if (!status) {
          return; // 等待有效状态
        }
        const allCompleted = this.isAllCompleted();
        if (!this.statusInitialized) {
          // 第一次获得有效状态，记录基线
          this.statusInitialized = true;
          this.previousCompleted = allCompleted;
          return;
        }
        // 仅在状态从未完成 -> 已完成 的瞬间触发 finishTest
        if (!this.completedEmitted && !this.previousCompleted && allCompleted) {
          this.finishTest();
        }
        this.previousCompleted = allCompleted;
        this.restoreDigitalState(); // 增加状态恢复调用
      })
    );

    this.subscriptions.add(
      this.manualTestService.testStatusUpdated$.subscribe(() => {
        this.restoreDigitalState(); // 增加状态恢复调用
        setTimeout(() => {
          this.restoreDigitalState(); // 增加延迟调用确保后端数据异步到达时同步UI
        });
      })
    );
  }

  ngOnDestroy(): void {
    this.subscriptions.unsubscribe();
  }

  /**
   * 获取当前状态
   */
  getCurrentState(): string {
    return this.plcMonitoringService.getFormattedMonitoringValue('currentState', 'DO' as any) || '读取中...';
  }

  /**
   * 获取当前状态颜色
   */
  getCurrentStateColor(): string {
    const state = this.getCurrentState();
    if (state === 'ON' || state === '1' || state === 'true') {
      return '#52c41a'; // 绿色
    } else if (state === 'OFF' || state === '0' || state === 'false') {
      return '#ff4d4f'; // 红色
    }
    return '#1890ff'; // 默认蓝色
  }

  /**
   * 获取状态标签
   */
  getStateLabel(state: string, index: number): string {
    const stepNumber = index + 1;
    switch (state) {
      case 'low': return index === 0 ? '低电平(第1步)' : '低电平(第3步)';
      case 'high': return '高电平(第2步)';
      default: return `${state}(第${stepNumber}步)`;
    }
  }

  /**
   * 根据状态和索引获取对应的采集结果
   */
  getStateResult(state: string, index: number): any {
    const stepNumber = this.getStepNumberByStateAndIndex(state, index);
    const resultKey = `step_${stepNumber}`;
    return this.stateResults[resultKey];
  }

  /**
   * 检查指定状态是否已完成
   */
  isStateCompleted(state: string, index: number): boolean {
    const stepNumber = this.getStepNumberByStateAndIndex(state, index);
    const resultKey = `step_${stepNumber}`;
    const result = this.stateResults[resultKey];
    // 确保结果存在且有有效的采集数据
    return !!(result && result.actualValue !== undefined);
  }

  /**
   * 检查状态按钮是否应该禁用
   */
  isStateButtonDisabled(state: string, index: number): boolean {
    const stepNumber = this.getStepNumberByStateAndIndex(state, index);
    
    // 如果已经采集过，则禁用
    if (this.isStateCompleted(state, index)) {
      return true;
    }

    // 按序列验证：第2步需要第1步完成，第3步需要第1、2步完成
    if (stepNumber === 2 && !this.hasStepCompleted(1)) {
      return true;
    }
    
    if (stepNumber === 3 && (!this.hasStepCompleted(1) || !this.hasStepCompleted(2))) {
      return true;
    }

    return false;
  }

  /**
   * 获取按钮类型 - 提供视觉引导
   */
  getButtonType(state: string, index: number): 'default' | 'primary' | 'text' {
    const stepNumber = this.getStepNumberByStateAndIndex(state, index);
    
    // 已完成的步骤显示为primary（将通过CSS设置为绿色）
    if (this.isStateCompleted(state, index)) {
      return 'primary';
    }
    
    // 当前可点击的步骤显示为default（蓝色边框）
    const nextStep = this.getNextRequiredStep();
    if (stepNumber === nextStep) {
      return 'default';
    }
    
    // 等待的步骤显示为text（灰色）
    return 'text';
  }
  
  /**
   * 获取下一个需要执行的步骤号
   */
  private getNextRequiredStep(): number {
    if (!this.hasStepCompleted(1)) {
      return 1;
    } else if (!this.hasStepCompleted(2)) {
      return 2;
    } else if (!this.hasStepCompleted(3)) {
      return 3;
    }
    return -1; // 所有步骤都完成了
  }

  /**
   * 获取子项状态颜色
   */
  getSubItemStatusColor(subItem: ManualTestSubItem): string {
    const status = this.manualTestService.getSubItemStatus(subItem);
    return MANUAL_TEST_SUB_ITEM_STATUS_COLORS[status];
  }

  /**
   * 获取子项状态文本
   */
  getSubItemStatusText(subItem: ManualTestSubItem): string {
    const status = this.manualTestService.getSubItemStatus(subItem);
    return MANUAL_TEST_SUB_ITEM_LABELS[subItem] || status;
  }

  /**
   * 检查子项是否已完成
   */
  isSubItemCompleted(subItem: ManualTestSubItem): boolean {
    return this.manualTestService.isSubItemCompleted(subItem);
  }

  /**
   * 检查子项是否已通过或跳过（用于按钮disable判断）
   * 只有通过或跳过的项目才禁用按钮，失败的项目允许重新操作
   */
  isSubItemPassedOrSkipped(subItem: ManualTestSubItem): boolean {
    const status = this.manualTestService.getSubItemStatus(subItem);
    return status === ManualTestSubItemStatus.Passed || 
           status === ManualTestSubItemStatus.Skipped;
  }

  /**
   * 检查确认通过按钮是否应该禁用
   */
  isConfirmButtonDisabled(subItem: ManualTestSubItem): boolean {
    if (subItem === ManualTestSubItem.ShowValueCheck) {
      if (!this.areAllStatesCollected()) {
        return true;
      }
      return this.confirmClicked; // true ⇒ 禁用；false ⇒ 可点
    }

    // 其它子项按状态判断
    const status = this.manualTestService.getSubItemStatus(subItem);
    return status === ManualTestSubItemStatus.Passed || status === ManualTestSubItemStatus.Skipped;
  }

  /**
   * 检查是否所有3个状态都已收集完数据
   * 低电平(步骤1) -> 高电平(步骤2) -> 低电平(步骤3)
   */
  areAllStatesCollected(): boolean {
    return this.hasStepCompleted(1) && this.hasStepCompleted(2) && this.hasStepCompleted(3);
  }

  /**
   * 检查状态采集是否全部完成
   */
  isStateCollectionCompleted(): boolean {
    return this.areAllStatesCollected();
  }

  /**
   * 完成子项
   */
  async completeSubItem(subItem: ManualTestSubItem): Promise<void> {
    // 点击确认通过时立刻禁用按钮，避免重复
    if (subItem === ManualTestSubItem.ShowValueCheck) {
      this.confirmClicked = true;
    }
    if (!this.instance) return;

    try {
      await this.manualTestService.completeSubItem(this.instance.instance_id, subItem);
      this.message.success(`${MANUAL_TEST_SUB_ITEM_LABELS[subItem]} 已完成`);
    } catch (error) {
      this.message.error(`完成测试项失败: ${error}`);
    }
  }

  /**
   * 跳过子项
   */
  async skipSubItem(subItem: ManualTestSubItem): Promise<void> {
    if (!this.instance) return;

    this.modal.confirm({
      nzTitle: '确认测试失败',
      nzContent: `确定要将测试标记为失败吗？`,
      nzOnOk: async () => {
        try {
          await this.manualTestService.failSubItem(this.instance!.instance_id, subItem, '用户手动标记失败');
          this.message.error(`${MANUAL_TEST_SUB_ITEM_LABELS[subItem]} 已标记失败`);
        } catch (error) {
          this.message.error(`确认测试失败异常: ${error}`);
        }
      }
    });
  }

  /**
   * 获取已完成数量
   */
  getCompletedCount(): number {
    return this.manualTestService.getCompletedSubItemsCount(this.testConfig.applicableSubItems);
  }

  /**
   * 获取总数量
   */
  getTotalCount(): number {
    return this.testConfig.applicableSubItems.length;
  }

  /**
   * 检查是否全部完成
   */
  isAllCompleted(): boolean {
    return this.manualTestService.areAllSubItemsCompleted(this.testConfig.applicableSubItems);
  }

  /**
   * 完成测试
   */
  private finishTest(): void {
    // 仅通知外部，由 ManualTestModal 统一取消测试并关闭窗口
    this.testCompleted.emit();
    this.completedEmitted = true;
  }

  /**
   * 点击数字状态采集按钮
   */
  async captureDigitalState(state: string, index: number): Promise<void> {
    if (!this.instance) return;

    this.isCapturing = true;
    this.currentCapturingState = state;

    try {
      const stepNumber = this.getStepNumberByStateAndIndex(state, index);
      const expectedState = state === 'high';
      
      const resp = await invoke<any>('capture_do_state_cmd', {
        instanceId: this.instance.instance_id,
        stepNumber: stepNumber,
        expectedState: expectedState
      });

      // 记录采集结果
      const resultKey = `step_${stepNumber}`;
      this.stateResults[resultKey] = {
        actualValue: resp.actual_value,
        timestamp: new Date(),
        stepNumber: stepNumber
      };

      // 触发实例已更新事件，通知父组件刷新
      this.manualTestService.emitInstanceUpdated(this.instance.instance_id);
      
      this.message.success(`采集${this.getStateLabel(state, index)}成功，状态: ${resp.actual_value}`);
    } catch (err: any) {
      this.message.error(`采集${this.getStateLabel(state, index)}失败: ${err}`);
    } finally {
      this.isCapturing = false;
      this.currentCapturingState = null;
    }
  }

  /**
   * 检查指定步骤是否已完成
   */
  private hasStepCompleted(stepNumber: number): boolean {
    const resultKey = `step_${stepNumber}`;
    return !!this.stateResults[resultKey];
  }

  /**
   * 获取状态按钮提示文本
   */
  getStateButtonTooltip(state: string, index: number): string {
    const stepNumber = this.getStepNumberByStateAndIndex(state, index);
    
    if (this.isStateCollectionCompleted()) {
      return `状态采集已完成，采集按钮已禁用以保护数据一致性`;
    }
    
    if (this.isStateCompleted(state, index)) {
      const result = this.getStateResult(state, index);
      return `已采集 - 步骤${stepNumber}: ${result.actualValue}`;
    }

    if (this.isStateButtonDisabled(state, index)) {
      if (stepNumber === 2 && !this.hasStepCompleted(1)) {
        return `请先完成第1步（低电平）的采集`;
      }
      if (stepNumber === 3 && (!this.hasStepCompleted(1) || !this.hasStepCompleted(2))) {
        return `请先完成前面步骤的采集`;
      }
    }
    
    return `点击采集${this.getStateLabel(state, index)}`;
  }

  /**
   * 检查是否有任何状态采集结果
   */
  hasAnyStateResults(): boolean {
    return Object.keys(this.stateResults).length > 0;
  }

  /**
   * 获取确认通过按钮的提示文本
   */
  getConfirmButtonTooltip(): string {
    const status = this.manualTestService.getSubItemStatus(ManualTestSubItem.ShowValueCheck);
    
    if (status === ManualTestSubItemStatus.Passed) {
      return '该项目已通过';
    }
    
    if (status === ManualTestSubItemStatus.Skipped) {
      return '该项目已跳过';
    }
    
    if (!this.areAllStatesCollected()) {
      return '需要先完成所有状态（低-高-低电平）的数据采集';
    }
    
    return '确认显示值与实际输出状态一致';
  }
}