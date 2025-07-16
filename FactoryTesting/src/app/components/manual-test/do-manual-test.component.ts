import { Component, Input, Output, EventEmitter, OnInit, OnDestroy, ChangeDetectorRef } from '@angular/core';
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
import { Subscription } from 'rxjs';

import { ChannelTestInstance, ChannelPointDefinition } from '../../models';
import { ManualTestService } from '../../services/manual-test.service';
import { PlcMonitoringService } from '../../services/plc-monitoring.service';
import {
  ManualTestStatus,
  ManualTestSubItem,
  MANUAL_TEST_SUB_ITEM_LABELS,
  MANUAL_TEST_SUB_ITEM_STATUS_COLORS,
  getManualTestConfig
} from '../../models/manual-test.types';

/**
 * DO点位手动测试组件
 * 包含1个测试项：显示值核对
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
              <div class="test-item-actions">
                <button 
                  nz-button 
                  nzType="primary" 
                  nzSize="small"
                  [disabled]="isSubItemCompleted(ManualTestSubItem.ShowValueCheck)"
                  (click)="completeSubItem(ManualTestSubItem.ShowValueCheck)">
                  <i nz-icon nzType="check"></i>
                  确认通过
                </button>
                <button 
                  nz-button 
                  nzSize="small"
                  [disabled]="isSubItemCompleted(ManualTestSubItem.ShowValueCheck)"
                  (click)="skipSubItem(ManualTestSubItem.ShowValueCheck)">
                  <i nz-icon nzType="forward"></i>
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
  styleUrls: ['./ai-manual-test.component.css'] // 复用AI组件的样式
})
export class DoManualTestComponent implements OnInit, OnDestroy {
  @Input() instance: ChannelTestInstance | null = null;
  @Input() definition: ChannelPointDefinition | null = null;
  @Input() testStatus: ManualTestStatus | null = null;

  @Output() testCompleted = new EventEmitter<void>();
  @Output() testCancelled = new EventEmitter<void>();

  // 测试配置
  private testConfig = getManualTestConfig('DO' as any);
  
  // 订阅管理
  private subscriptions = new Subscription();

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

  ngOnInit(): void {
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
          this.statusInitialized = true;
          this.previousCompleted = allCompleted;
          return;
        }
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
   * 完成测试
   */
  private finishTest(): void {
    // 仅通知外部，由 ManualTestModal 统一取消测试并关闭窗口
    this.testCompleted.emit();
    this.completedEmitted = true;
  }
}
