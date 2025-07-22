import { Component, Input, Output, EventEmitter, OnInit, OnDestroy } from '@angular/core';
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
 * AO点位手动测试组件
 * 包含3个测试项：显示值核对 + 趋势检查 + 报表检查
 */
@Component({
  selector: 'app-ao-manual-test',
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
    <div class="ao-manual-test">
      
      <!-- 实时监控数据显示 -->
      <nz-card nzTitle="实时监控数据" nzSize="small" class="monitoring-card">
        <div class="monitoring-grid">
          <nz-statistic 
            nzTitle="当前输出值" 
            [nzValue]="getCurrentOutput()" 
            nzSuffix="工程单位"
            [nzValueStyle]="{ color: '#1890ff' }">
          </nz-statistic>
        </div>
      </nz-card>

      <nz-divider></nz-divider>

      <!-- AO 采集按钮 -->
      <nz-card nzTitle="采集输出百分比测试" nzSize="small" class="capture-card">
        
        <!-- 测试完成状态提示 -->
        <div *ngIf="isTestCompleted()" 
             style="margin-bottom: 12px; padding: 8px 12px; background-color: #f6ffed; border: 1px solid #b7eb8f; border-radius: 4px; display: flex; align-items: center; gap: 8px; color: #52c41a; font-size: 12px;">
          <i nz-icon nzType="check-circle" nzTheme="twotone" [nzTwotoneColor]="'#52c41a'"></i>
          <span>测试已完成，采集按钮已禁用以保护数据一致性</span>
        </div>

        <div class="capture-buttons">
          <button *ngFor="let pct of percentPoints"
                  nz-button
                  nzType="default"
                  [disabled]="captureCompleted[pct] || isTestCompleted()"
                  [nzLoading]="isCapturing && currentCapturingPercent === pct"
                  (click)="captureAoPoint(pct)"
                  [title]="getButtonTooltip(pct)">
            <span class="button-text">{{ pct }}%</span>
            <span *ngIf="captureResults[pct]" class="deviation-text">
              偏差: {{ captureResults[pct].deviation.toFixed(1) }}%
            </span>
          </button>
        </div>

        <!-- 采集结果汇总 -->
        <div *ngIf="hasAnyResults()" class="capture-summary">
          <nz-divider nzText="采集结果汇总" nzOrientation="left"></nz-divider>
          <div class="summary-grid">
            <div *ngFor="let pct of percentPoints" class="summary-item" [class.completed]="captureCompleted[pct]">
              <span class="summary-label">{{ pct }}%:</span>
              <span *ngIf="captureResults[pct]" class="summary-value">
                {{ captureResults[pct].value.toFixed(2) }} ({{ captureResults[pct].deviation > 0 ? '+' : '' }}{{ captureResults[pct].deviation.toFixed(1) }}%)
              </span>
              <span *ngIf="!captureResults[pct]" class="summary-pending">待采集</span>
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
              <p>请确认HMI界面显示的输出值与实际输出一致</p>
              
              <!-- 检查点收集状态提示 -->
              <div *ngIf="!areAllCheckpointsCollected() && !isSubItemPassedOrSkipped(ManualTestSubItem.ShowValueCheck)" 
                   style="margin: 8px 0; padding: 8px 12px; background-color: #fffbe6; border: 1px solid #fadb14; border-radius: 4px; display: flex; align-items: center; gap: 8px; color: #faad14; font-size: 12px;">
                <i nz-icon nzType="exclamation-circle" nzTheme="twotone" [nzTwotoneColor]="'#faad14'"></i>
                <span>确认通过需要先完成所有检查点 (0%, 25%, 50%, 75%, 100%) 的数据采集</span>
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

          <!-- 维护功能测试 -->
          <!--nz-card nzSize="small" class="test-item-card" *ngIf="shouldShowSubItem(ManualTestSubItem.MaintenanceFunction)">
            <div class="test-item-header">
              <span class="test-item-title">维护功能测试</span>
              <nz-tag [nzColor]="getSubItemStatusColor(ManualTestSubItem.MaintenanceFunction)">
                {{ getSubItemStatusText(ManualTestSubItem.MaintenanceFunction) }}
              </nz-tag>
            </div>
            <div class="test-item-content">
              <p>请确认维护开关和维护值设定功能正常</p>
              <div class="test-item-actions">
                <button 
                  nz-button 
                  nzType="primary" 
                  nzSize="small"
                  [disabled]="isSubItemPassedOrSkipped(ManualTestSubItem.MaintenanceFunction)"
                  (click)="completeSubItem(ManualTestSubItem.MaintenanceFunction)">
                  <i nz-icon nzType="check"></i>
                  确认通过
                </button>
                <button 
                  nz-button 
                  nzSize="small"
                  [disabled]="isSubItemPassedOrSkipped(ManualTestSubItem.MaintenanceFunction)"
                  (click)="skipSubItem(ManualTestSubItem.MaintenanceFunction)">
                  <i nz-icon nzType="close"></i>
                  测试失败
                </button>
              </div>
            </div>
          </nz-card-->

        </div>
      </div>

      <nz-divider></nz-divider>

      <!-- 测试进度 -->
      <div class="test-progress-section">
        <span>测试进度: {{ getCompletedCount() }} / {{ getTotalCount() }}</span>
      </div>

    </div>
  `,
  styleUrls: ['./ai-manual-test.component.css'] // 复用AI组件的样式
})
export class AoManualTestComponent implements OnInit, OnDestroy {
  @Input() instance: ChannelTestInstance | null = null;
  @Input() definition: ChannelPointDefinition | null = null;
  @Input() testStatus: ManualTestStatus | null = null;
  @Output() testCompleted = new EventEmitter<void>();
  @Output() testCancelled = new EventEmitter<void>();

  // 测试配置
  private testConfig = getManualTestConfig('AO' as any);
  
  // 订阅管理
  private subscriptions = new Subscription();

  // AO 采集相关
  percentPoints: number[] = [0, 25, 50, 75, 100];
  captureCompleted: Record<number, boolean> = {0:false,25:false,50:false,75:false,100:false};
  captureResults: Record<number, { value: number; deviation: number }> = {};
  isCapturing: boolean = false;
  currentCapturingPercent: number | null = null;

  // 枚举引用（用于模板）
  ManualTestSubItem = ManualTestSubItem;


  constructor(
    private manualTestService: ManualTestService,
    private plcMonitoringService: PlcMonitoringService,
    private message: NzMessageService,
    private modal: NzModalService
  ) {}

  // 已触发完成事件标志，避免重复执行
  private completedEmitted = false;
  private statusInitialized = false;
  private previousCompleted = false;

  ngOnInit(): void {
    // PLC 监控刷新
    this.subscriptions.add(
      this.plcMonitoringService.currentMonitoringData$.subscribe(() => {})
    );

    // 监听测试状态：仅在仍处于 ManualTest 阶段且全部子项完成时触发 finishTest()
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
      })
    );
  }

  ngOnDestroy(): void {
    this.subscriptions.unsubscribe();
  }

  /**
   * 获取当前输出值
   */
  getCurrentOutput(): string {
    return this.plcMonitoringService.getFormattedMonitoringValue('currentOutput', 'AO' as any) || '读取中...';
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
   * 注意：这个方法现在只用于"测试失败"按钮，"确认通过"按钮有单独的判断逻辑
   */
  isSubItemPassedOrSkipped(subItem: ManualTestSubItem): boolean {
    const status = this.manualTestService.getSubItemStatus(subItem);
    return status === ManualTestSubItemStatus.Passed || 
           status === ManualTestSubItemStatus.Skipped;
  }

  /**
   * 检查确认通过按钮是否应该禁用
   * 对于AO点位的确认通过按钮，需要满足两个条件：
   * 1. 该项目未通过或跳过
   * 2. 所有5个检查点都已收集完数据
   */
  isConfirmButtonDisabled(subItem: ManualTestSubItem): boolean {
    const status = this.manualTestService.getSubItemStatus(subItem);
    
    // 如果已经通过或跳过，禁用按钮
    if (status === ManualTestSubItemStatus.Passed || status === ManualTestSubItemStatus.Skipped) {
      return true;
    }

    // 对于ShowValueCheck（显示值核对），需要检查5个检查点是否都收集完数据
    if (subItem === ManualTestSubItem.ShowValueCheck) {
      return !this.areAllCheckpointsCollected();
    }
    
    return false;
  }

  /**
   * 检查是否所有5个检查点都已收集完数据
   * 0%, 25%, 50%, 75%, 100%
   */
  areAllCheckpointsCollected(): boolean {
    const allCollected = this.percentPoints.every(percent => this.captureCompleted[percent]);
    
    // 调试输出
    if (!allCollected) {
      const missingPoints = this.percentPoints.filter(percent => !this.captureCompleted[percent]);
      console.log('🔍 [AO_MANUAL_TEST] 检查点收集状态:', {
        allCollected: allCollected,
        completedPoints: this.percentPoints.filter(percent => this.captureCompleted[percent]),
        missingPoints: missingPoints,
        captureCompleted: this.captureCompleted
      });
    }
    
    return allCollected;
  }

  /**
   * 检查子项是否应该显示（未被后端标记为跳过）
   * 如果后端在初始化时就将某个子项标记为Skipped，说明该项应该被隐藏
   */
  shouldShowSubItem(subItem: ManualTestSubItem): boolean {
    // 如果没有测试状态，显示所有项目（向后兼容）
    if (!this.testStatus) return true;
    
    // 检查后端返回的状态
    const subItemResult = this.testStatus.subItemResults[subItem];
    if (!subItemResult) return true;
    
    // 如果后端初始化时就标记为Skipped，且有相关的跳过原因，则不显示
    // 跳过原因存储在operatorNotes字段中（后端映射关系：details -> operatorNotes）
    if (subItemResult.status === ManualTestSubItemStatus.Skipped && 
        subItemResult.operatorNotes && 
        (subItemResult.operatorNotes.includes('预留点位测试') ||
         subItemResult.operatorNotes.includes('设定值为空') ||
         subItemResult.operatorNotes.includes('无报警设定值'))) {
      return false;
    }
    
    return true;
  }

  /**
   * 完成子项
   */
  async completeSubItem(subItem: ManualTestSubItem): Promise<void> {
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
   * 检查测试是否已完成（通过或失败）
   * 用于控制采集按钮的禁用状态，保护数据一致性
   */
  isTestCompleted(): boolean {
    if (!this.instance) return false;
    
    // 方法1：如果整体状态明确显示为测试完成，则禁用采集按钮（最权威的判断）
    const isOverallCompleted = this.instance.overall_status === OverallTestStatus.TestCompletedPassed ||
                              this.instance.overall_status === OverallTestStatus.TestCompletedFailed;
    
    if (isOverallCompleted) {
      console.log('🔍 [AO_MANUAL_TEST] 测试状态已完成，禁用采集按钮:', this.instance.overall_status);
      return true;
    }
    
    // 方法2：检查所有手动测试子项是否都已完成（适用于查看详情模式）
    // 这种情况下completedEmitted可能是false，但子项状态已完成
    const allSubItemsCompleted = this.isAllCompleted();
    if (allSubItemsCompleted) {
      console.log('🔍 [AO_MANUAL_TEST] 所有手动测试子项已完成，禁用采集按钮');
      return true;
    }
    
    // 方法3：如果已经发出完成事件，也禁用采集按钮（防止状态更新延迟）
    if (this.completedEmitted) {
      console.log('🔍 [AO_MANUAL_TEST] 测试完成事件已发出，禁用采集按钮');
      return true;
    }
    
    // 调试输出当前状态
    if (Math.random() < 0.1) { // 10%概率输出调试信息
      console.log('🔍 [AO_MANUAL_TEST] isTestCompleted检查:', {
        instanceStatus: this.instance.overall_status,
        completedEmitted: this.completedEmitted,
        allCompleted: allSubItemsCompleted,
        result: false
      });
    }
    
    return false;
  }

  /**
   * 完成测试
   */
  // 当用户点击 “完成测试” 或程序检测到测试完成时调用
  finishTest(): void {
    // 只发出完成事件供外部（ManualTestModal）关闭
    this.testCompleted.emit();
    this.completedEmitted = true;
  }

  /**
   * 取消测试
   */
  /**
   * 点击采集按钮
   */
  async captureAoPoint(percent: number): Promise<void> {
    if (!this.instance) return;

    this.isCapturing = true;
    this.currentCapturingPercent = percent;

    try {
      const resp = await invoke<any>('capture_ao_point_cmd', {
        instanceId: this.instance.instance_id,
        checkpointPercent: percent
      });
      this.captureCompleted[percent] = true;
      this.captureResults[percent] = {
        value: resp.actual_value,
        deviation: resp.deviation_percent
      };
      this.message.success(`采集 ${percent}% 成功，偏差 ${resp.deviation_percent.toFixed(2)}%`);
    } catch (err: any) {
      this.message.error(`采集 ${percent}% 失败: ${err}`);
    } finally {
      this.isCapturing = false;
      this.currentCapturingPercent = null;
    }
  }

  /**
   * 获取按钮提示文本
   */
  getButtonTooltip(percent: number): string {
    // 如果测试已完成，显示保护性提示
    if (this.isTestCompleted()) {
      // 区分不同的完成情况
      if (this.instance && (this.instance.overall_status === OverallTestStatus.TestCompletedPassed ||
                           this.instance.overall_status === OverallTestStatus.TestCompletedFailed)) {
        return `整体测试已完成（${this.instance.overall_status === OverallTestStatus.TestCompletedPassed ? '通过' : '失败'}），采集按钮已禁用以保护数据一致性`;
      } else if (this.isAllCompleted()) {
        return `所有测试项已完成，采集按钮已禁用以保护数据一致性`;
      } else {
        return `测试已完成，采集按钮已禁用以保护数据一致性`;
      }
    }
    
    if (this.captureCompleted[percent]) {
      const result = this.captureResults[percent];
      return `已采集 - 实际值: ${result.value.toFixed(2)}, 偏差: ${result.deviation.toFixed(1)}%`;
    }
    return `点击采集 ${percent}% 输出点`;
  }

  /**
   * 检查是否有任何采集结果
   */
  hasAnyResults(): boolean {
    return Object.keys(this.captureResults).length > 0;
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
    
    if (!this.areAllCheckpointsCollected()) {
      const missingPoints = this.percentPoints.filter(percent => !this.captureCompleted[percent]);
      return `需要先完成所有检查点的数据采集。缺失检查点: ${missingPoints.join(', ')}%`;
    }
    
    return '确认显示值与实际输出一致';
  }

  cancelTest(): void {
    this.modal.confirm({
      nzTitle: '确认取消',
      nzContent: '确定要取消手动测试吗？已完成的测试项将会保存。',
      nzOnOk: () => {
        this.testCancelled.emit();
      }
    });
  }
}
