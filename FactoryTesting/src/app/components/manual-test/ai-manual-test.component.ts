import { Component, Input, Output, EventEmitter, OnInit, OnDestroy } from '@angular/core';
import { CommonModule } from '@angular/common';
import { FormsModule } from '@angular/forms';
import { NzCardModule } from 'ng-zorro-antd/card';
import { NzButtonModule } from 'ng-zorro-antd/button';
import { NzTagModule } from 'ng-zorro-antd/tag';
import { NzIconModule } from 'ng-zorro-antd/icon';
import { NzInputModule } from 'ng-zorro-antd/input';
import { NzMessageService } from 'ng-zorro-antd/message';
import { NzModalService } from 'ng-zorro-antd/modal';
import { NzDividerModule } from 'ng-zorro-antd/divider';
import { NzSpaceModule } from 'ng-zorro-antd/space';
import { NzStatisticModule } from 'ng-zorro-antd/statistic';
import { Subscription } from 'rxjs';

import { ChannelTestInstance, ChannelPointDefinition } from '../../models';
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
 * AI点位手动测试组件
 * 包含8个测试项：显示值核对 + 4个报警测试 + 趋势检查 + 报表检查 + 维护功能
 */
@Component({
  selector: 'app-ai-manual-test',
  standalone: true,
  imports: [
    CommonModule,
    FormsModule,
    NzCardModule,
    NzButtonModule,
    NzTagModule,
    NzIconModule,
    NzInputModule,
    NzDividerModule,
    NzSpaceModule,
    NzStatisticModule
  ],
  template: `
    <div class="ai-manual-test">
      
      <!-- 实时监控数据显示 -->
      <nz-card nzTitle="实时监控数据" nzSize="small" class="monitoring-card">
        <div class="monitoring-grid">
          <nz-statistic 
            nzTitle="当前值" 
            [nzValue]="getCurrentValue()" 
            nzSuffix="工程单位"
            [nzValueStyle]="{ color: '#1890ff' }">
          </nz-statistic>
          <nz-statistic 
            nzTitle="SLL设定值" 
            [nzValue]="getSllSetPoint()" 
            [nzValueStyle]="{ color: '#ff4d4f' }">
          </nz-statistic>
          <nz-statistic 
            nzTitle="SL设定值" 
            [nzValue]="getSlSetPoint()" 
            [nzValueStyle]="{ color: '#faad14' }">
          </nz-statistic>
          <nz-statistic 
            nzTitle="SH设定值" 
            [nzValue]="getShSetPoint()" 
            [nzValueStyle]="{ color: '#faad14' }">
          </nz-statistic>
          <nz-statistic 
            nzTitle="SHH设定值" 
            [nzValue]="getShhSetPoint()" 
            [nzValueStyle]="{ color: '#ff4d4f' }">
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
              <p>请确认HMI界面显示的数值与实际测量值一致</p>

              <!-- 显示值输入和操作 -->
              <div class="display-value-section">
                <div class="value-input-group">
                  <label>测试值:</label>
                  <nz-input-group nzCompact>
                    <input
                      nz-input
                      style="width: 120px"
                      [(ngModel)]="displayTestValue"
                      [placeholder]="getDisplayValuePlaceholder()"
                      type="number"
                      step="0.01">
                    <button
                      nz-button
                      nzType="default"
                      (click)="generateRandomDisplayValue()"
                      title="生成随机值">
                      <i nz-icon nzType="reload"></i>
                    </button>
                  </nz-input-group>
                </div>

                <div class="test-actions">
                  <button
                    nz-button
                    nzType="primary"
                    nzSize="small"
                    [nzLoading]="isDisplayValueTesting"
                    (click)="executeDisplayValueTest()">
                    <i nz-icon nzType="play-circle"></i>
                    显示值核对测试
                  </button>
                </div>
              </div>

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
                  跳过
                </button>
              </div>
            </div>
          </nz-card>

          <!-- 低低报警测试 -->
          <nz-card nzSize="small" class="test-item-card">
            <div class="test-item-header">
              <span class="test-item-title">低低报警测试</span>
              <nz-tag [nzColor]="getSubItemStatusColor(ManualTestSubItem.LowLowAlarmTest)">
                {{ getSubItemStatusText(ManualTestSubItem.LowLowAlarmTest) }}
              </nz-tag>
            </div>
            <div class="test-item-content">
              <p>设定值: {{ getSllSetPoint() }}</p>
              <p>请确认当数值低于设定值时触发低低报警</p>

              <!-- 报警测试操作 -->
              <div class="alarm-test-section">
                <div class="test-actions">
                  <button
                    nz-button
                    nzType="primary"
                    nzSize="small"
                    [nzLoading]="isAlarmTesting && currentAlarmType === 'LL'"
                    (click)="executeAlarmTest('LL')">
                    <i nz-icon nzType="play-circle"></i>
                    低低报警测试
                  </button>
                  <button
                    nz-button
                    nzType="default"
                    nzSize="small"
                    [nzLoading]="isAlarmTesting && currentAlarmType === 'LL_RESET'"
                    (click)="resetAlarmTest('LL')">
                    <i nz-icon nzType="reload"></i>
                    复位
                  </button>
                </div>
              </div>

              <div class="test-item-actions">
                <button
                  nz-button
                  nzType="primary"
                  nzSize="small"
                  [disabled]="isSubItemCompleted(ManualTestSubItem.LowLowAlarmTest)"
                  (click)="completeSubItem(ManualTestSubItem.LowLowAlarmTest)">
                  <i nz-icon nzType="check"></i>
                  确认通过
                </button>
                <button
                  nz-button
                  nzSize="small"
                  [disabled]="isSubItemCompleted(ManualTestSubItem.LowLowAlarmTest)"
                  (click)="skipSubItem(ManualTestSubItem.LowLowAlarmTest)">
                  <i nz-icon nzType="forward"></i>
                  跳过
                </button>
              </div>
            </div>
          </nz-card>

          <!-- 低报警测试 -->
          <nz-card nzSize="small" class="test-item-card">
            <div class="test-item-header">
              <span class="test-item-title">低报警测试</span>
              <nz-tag [nzColor]="getSubItemStatusColor(ManualTestSubItem.LowAlarmTest)">
                {{ getSubItemStatusText(ManualTestSubItem.LowAlarmTest) }}
              </nz-tag>
            </div>
            <div class="test-item-content">
              <p>设定值: {{ getSlSetPoint() }}</p>
              <p>请确认当数值低于设定值时触发低报警</p>

              <!-- 报警测试操作 -->
              <div class="alarm-test-section">
                <div class="test-actions">
                  <button
                    nz-button
                    nzType="primary"
                    nzSize="small"
                    [nzLoading]="isAlarmTesting && currentAlarmType === 'L'"
                    (click)="executeAlarmTest('L')">
                    <i nz-icon nzType="play-circle"></i>
                    低报警测试
                  </button>
                  <button
                    nz-button
                    nzType="default"
                    nzSize="small"
                    [nzLoading]="isAlarmTesting && currentAlarmType === 'L_RESET'"
                    (click)="resetAlarmTest('L')">
                    <i nz-icon nzType="reload"></i>
                    复位
                  </button>
                </div>
              </div>

              <div class="test-item-actions">
                <button
                  nz-button
                  nzType="primary"
                  nzSize="small"
                  [disabled]="isSubItemCompleted(ManualTestSubItem.LowAlarmTest)"
                  (click)="completeSubItem(ManualTestSubItem.LowAlarmTest)">
                  <i nz-icon nzType="check"></i>
                  确认通过
                </button>
                <button
                  nz-button
                  nzSize="small"
                  [disabled]="isSubItemCompleted(ManualTestSubItem.LowAlarmTest)"
                  (click)="skipSubItem(ManualTestSubItem.LowAlarmTest)">
                  <i nz-icon nzType="forward"></i>
                  跳过
                </button>
              </div>
            </div>
          </nz-card>

          <!-- 高报警测试 -->
          <nz-card nzSize="small" class="test-item-card">
            <div class="test-item-header">
              <span class="test-item-title">高报警测试</span>
              <nz-tag [nzColor]="getSubItemStatusColor(ManualTestSubItem.HighAlarmTest)">
                {{ getSubItemStatusText(ManualTestSubItem.HighAlarmTest) }}
              </nz-tag>
            </div>
            <div class="test-item-content">
              <p>设定值: {{ getShSetPoint() }}</p>
              <p>请确认当数值高于设定值时触发高报警</p>

              <!-- 报警测试操作 -->
              <div class="alarm-test-section">
                <div class="test-actions">
                  <button
                    nz-button
                    nzType="primary"
                    nzSize="small"
                    [nzLoading]="isAlarmTesting && currentAlarmType === 'H'"
                    (click)="executeAlarmTest('H')">
                    <i nz-icon nzType="play-circle"></i>
                    高报警测试
                  </button>
                  <button
                    nz-button
                    nzType="default"
                    nzSize="small"
                    [nzLoading]="isAlarmTesting && currentAlarmType === 'H_RESET'"
                    (click)="resetAlarmTest('H')">
                    <i nz-icon nzType="reload"></i>
                    复位
                  </button>
                </div>
              </div>

              <div class="test-item-actions">
                <button
                  nz-button
                  nzType="primary"
                  nzSize="small"
                  [disabled]="isSubItemCompleted(ManualTestSubItem.HighAlarmTest)"
                  (click)="completeSubItem(ManualTestSubItem.HighAlarmTest)">
                  <i nz-icon nzType="check"></i>
                  确认通过
                </button>
                <button
                  nz-button
                  nzSize="small"
                  [disabled]="isSubItemCompleted(ManualTestSubItem.HighAlarmTest)"
                  (click)="skipSubItem(ManualTestSubItem.HighAlarmTest)">
                  <i nz-icon nzType="forward"></i>
                  跳过
                </button>
              </div>
            </div>
          </nz-card>

          <!-- 高高报警测试 -->
          <nz-card nzSize="small" class="test-item-card">
            <div class="test-item-header">
              <span class="test-item-title">高高报警测试</span>
              <nz-tag [nzColor]="getSubItemStatusColor(ManualTestSubItem.HighHighAlarmTest)">
                {{ getSubItemStatusText(ManualTestSubItem.HighHighAlarmTest) }}
              </nz-tag>
            </div>
            <div class="test-item-content">
              <p>设定值: {{ getShhSetPoint() }}</p>
              <p>请确认当数值高于设定值时触发高高报警</p>

              <!-- 报警测试操作 -->
              <div class="alarm-test-section">
                <div class="test-actions">
                  <button
                    nz-button
                    nzType="primary"
                    nzSize="small"
                    [nzLoading]="isAlarmTesting && currentAlarmType === 'HH'"
                    (click)="executeAlarmTest('HH')">
                    <i nz-icon nzType="play-circle"></i>
                    高高报警测试
                  </button>
                  <button
                    nz-button
                    nzType="default"
                    nzSize="small"
                    [nzLoading]="isAlarmTesting && currentAlarmType === 'HH_RESET'"
                    (click)="resetAlarmTest('HH')">
                    <i nz-icon nzType="reload"></i>
                    复位
                  </button>
                </div>
              </div>

              <div class="test-item-actions">
                <button
                  nz-button
                  nzType="primary"
                  nzSize="small"
                  [disabled]="isSubItemCompleted(ManualTestSubItem.HighHighAlarmTest)"
                  (click)="completeSubItem(ManualTestSubItem.HighHighAlarmTest)">
                  <i nz-icon nzType="check"></i>
                  确认通过
                </button>
                <button
                  nz-button
                  nzSize="small"
                  [disabled]="isSubItemCompleted(ManualTestSubItem.HighHighAlarmTest)"
                  (click)="skipSubItem(ManualTestSubItem.HighHighAlarmTest)">
                  <i nz-icon nzType="forward"></i>
                  跳过
                </button>
              </div>
            </div>
          </nz-card>

          <!-- 趋势检查 -->
          <nz-card nzSize="small" class="test-item-card">
            <div class="test-item-header">
              <span class="test-item-title">趋势检查</span>
              <nz-tag [nzColor]="getSubItemStatusColor(ManualTestSubItem.TrendCheck)">
                {{ getSubItemStatusText(ManualTestSubItem.TrendCheck) }}
              </nz-tag>
            </div>
            <div class="test-item-content">
              <p>请确认HMI界面的趋势图显示正常</p>
              <div class="test-item-actions">
                <button 
                  nz-button 
                  nzType="primary" 
                  nzSize="small"
                  [disabled]="isSubItemCompleted(ManualTestSubItem.TrendCheck)"
                  (click)="completeSubItem(ManualTestSubItem.TrendCheck)">
                  <i nz-icon nzType="check"></i>
                  确认通过
                </button>
                <button 
                  nz-button 
                  nzSize="small"
                  [disabled]="isSubItemCompleted(ManualTestSubItem.TrendCheck)"
                  (click)="skipSubItem(ManualTestSubItem.TrendCheck)">
                  <i nz-icon nzType="forward"></i>
                  跳过
                </button>
              </div>
            </div>
          </nz-card>

          <!-- 报表检查 -->
          <nz-card nzSize="small" class="test-item-card">
            <div class="test-item-header">
              <span class="test-item-title">报表检查</span>
              <nz-tag [nzColor]="getSubItemStatusColor(ManualTestSubItem.ReportCheck)">
                {{ getSubItemStatusText(ManualTestSubItem.ReportCheck) }}
              </nz-tag>
            </div>
            <div class="test-item-content">
              <p>请确认相关报表生成和显示正常</p>
              <div class="test-item-actions">
                <button 
                  nz-button 
                  nzType="primary" 
                  nzSize="small"
                  [disabled]="isSubItemCompleted(ManualTestSubItem.ReportCheck)"
                  (click)="completeSubItem(ManualTestSubItem.ReportCheck)">
                  <i nz-icon nzType="check"></i>
                  确认通过
                </button>
                <button 
                  nz-button 
                  nzSize="small"
                  [disabled]="isSubItemCompleted(ManualTestSubItem.ReportCheck)"
                  (click)="skipSubItem(ManualTestSubItem.ReportCheck)">
                  <i nz-icon nzType="forward"></i>
                  跳过
                </button>
              </div>
            </div>
          </nz-card>

          <!-- 维护功能测试 -->
          <nz-card nzSize="small" class="test-item-card">
            <div class="test-item-header">
              <span class="test-item-title">维护功能测试</span>
              <nz-tag [nzColor]="getSubItemStatusColor(ManualTestSubItem.MaintenanceFunction)">
                {{ getSubItemStatusText(ManualTestSubItem.MaintenanceFunction) }}
              </nz-tag>
            </div>
            <div class="test-item-content">
              <p>请确认维护开关和维护值设定功能正常</p>

              <!-- 维护功能测试操作 -->
              <div class="maintenance-test-section">
                <div class="test-actions">
                  <button
                    nz-button
                    nzType="primary"
                    nzSize="small"
                    [nzLoading]="isMaintenanceTesting"
                    (click)="executeMaintenanceTest(true)">
                    <i nz-icon nzType="play-circle"></i>
                    维护功能启用
                  </button>
                  <button
                    nz-button
                    nzType="default"
                    nzSize="small"
                    [nzLoading]="isMaintenanceTesting"
                    (click)="executeMaintenanceTest(false)">
                    <i nz-icon nzType="reload"></i>
                    维护功能复位
                  </button>
                </div>
              </div>

              <div class="test-item-actions">
                <button
                  nz-button
                  nzType="primary"
                  nzSize="small"
                  [disabled]="isSubItemCompleted(ManualTestSubItem.MaintenanceFunction)"
                  (click)="completeSubItem(ManualTestSubItem.MaintenanceFunction)">
                  <i nz-icon nzType="check"></i>
                  确认通过
                </button>
                <button
                  nz-button
                  nzSize="small"
                  [disabled]="isSubItemCompleted(ManualTestSubItem.MaintenanceFunction)"
                  (click)="skipSubItem(ManualTestSubItem.MaintenanceFunction)">
                  <i nz-icon nzType="forward"></i>
                  跳过
                </button>
              </div>
            </div>
          </nz-card>

        </div>
      </div>

      <nz-divider></nz-divider>

      <!-- 测试进度和操作 -->
      <div class="test-progress-section">
        <div class="progress-info">
          <span>测试进度: {{ getCompletedCount() }} / {{ getTotalCount() }}</span>
          <div class="progress-actions">
            <button 
              nz-button 
              nzType="primary"
              [disabled]="!isAllCompleted()"
              (click)="finishTest()">
              <i nz-icon nzType="check-circle"></i>
              完成测试
            </button>
            <button 
              nz-button 
              (click)="cancelTest()">
              <i nz-icon nzType="close"></i>
              取消测试
            </button>
          </div>
        </div>
      </div>

    </div>
  `,
  styleUrls: ['./ai-manual-test.component.css']
})
export class AiManualTestComponent implements OnInit, OnDestroy {
  @Input() instance: ChannelTestInstance | null = null;
  @Input() definition: ChannelPointDefinition | null = null;
  @Input() testStatus: ManualTestStatus | null = null;
  @Output() testCompleted = new EventEmitter<void>();
  @Output() testCancelled = new EventEmitter<void>();

  // 测试配置
  private testConfig = getManualTestConfig('AI' as any);
  
  // 订阅管理
  private subscriptions = new Subscription();

  // 枚举引用（用于模板）
  ManualTestSubItem = ManualTestSubItem;

  // 显示值测试相关
  displayTestValue: number = 0;
  isDisplayValueTesting: boolean = false;

  // 报警测试状态
  isAlarmTesting: boolean = false;
  currentAlarmType: string = '';

  // 维护功能测试状态
  isMaintenanceTesting: boolean = false;

  constructor(
    private manualTestService: ManualTestService,
    private plcMonitoringService: PlcMonitoringService,
    private message: NzMessageService,
    private modal: NzModalService
  ) {}

  ngOnInit(): void {
    // 订阅PLC监控数据
    this.subscriptions.add(
      this.plcMonitoringService.currentMonitoringData$.subscribe(data => {
        // PLC数据更新时，界面会自动刷新
      })
    );
  }

  ngOnDestroy(): void {
    this.subscriptions.unsubscribe();
  }

  /**
   * 获取当前值
   */
  getCurrentValue(): string {
    return this.plcMonitoringService.getFormattedMonitoringValue('currentValue', 'AI' as any) || '读取中...';
  }

  /**
   * 获取SLL设定值
   */
  getSllSetPoint(): string {
    return this.plcMonitoringService.getFormattedMonitoringValue('sllSetPoint', 'AI' as any) || '读取中...';
  }

  /**
   * 获取SL设定值
   */
  getSlSetPoint(): string {
    return this.plcMonitoringService.getFormattedMonitoringValue('slSetPoint', 'AI' as any) || '读取中...';
  }

  /**
   * 获取SH设定值
   */
  getShSetPoint(): string {
    return this.plcMonitoringService.getFormattedMonitoringValue('shSetPoint', 'AI' as any) || '读取中...';
  }

  /**
   * 获取SHH设定值
   */
  getShhSetPoint(): string {
    return this.plcMonitoringService.getFormattedMonitoringValue('shhSetPoint', 'AI' as any) || '读取中...';
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
      nzTitle: '确认跳过',
      nzContent: `确定要跳过 "${MANUAL_TEST_SUB_ITEM_LABELS[subItem]}" 吗？`,
      nzOnOk: async () => {
        try {
          await this.manualTestService.skipSubItem(this.instance!.instance_id, subItem, '用户手动跳过');
          this.message.info(`${MANUAL_TEST_SUB_ITEM_LABELS[subItem]} 已跳过`);
        } catch (error) {
          this.message.error(`跳过测试项失败: ${error}`);
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
  finishTest(): void {
    this.testCompleted.emit();
  }

  /**
   * 取消测试
   */
  cancelTest(): void {
    this.modal.confirm({
      nzTitle: '确认取消',
      nzContent: '确定要取消手动测试吗？已完成的测试项将会保存。',
      nzOnOk: () => {
        this.testCancelled.emit();
      }
    });
  }

  // ==================== 新增的AI手动测试方法 ====================

  /**
   * 获取显示值占位符
   */
  getDisplayValuePlaceholder(): string {
    if (!this.definition) return '请输入测试值';
    const low = this.definition.range_low_limit || 0;
    const high = this.definition.range_high_limit || 100;
    return `${low} - ${high}`;
  }

  /**
   * 生成随机显示值
   */
  async generateRandomDisplayValue(): Promise<void> {
    if (!this.instance) {
      this.message.error('测试实例不存在');
      return;
    }

    try {
      const response = await this.manualTestService.generateRandomDisplayValue(this.instance.instance_id);
      if (response.success) {
        this.displayTestValue = response.randomValue;
        this.message.success(`已生成随机值: ${response.randomValue.toFixed(2)}`);
      } else {
        this.message.error(response.message || '生成随机值失败');
      }
    } catch (error) {
      this.message.error(`生成随机值失败: ${error}`);
    }
  }

  /**
   * 执行显示值核对测试
   */
  async executeDisplayValueTest(): Promise<void> {
    if (!this.instance) {
      this.message.error('测试实例不存在');
      return;
    }

    if (this.displayTestValue === null || this.displayTestValue === undefined) {
      this.message.error('请输入测试值');
      return;
    }

    this.isDisplayValueTesting = true;
    try {
      const response = await this.manualTestService.executeShowValueTest(
        this.instance.instance_id,
        this.displayTestValue
      );

      if (response.success) {
        this.message.success(`显示值测试成功: ${response.message}`);
      } else {
        this.message.error(response.message || '显示值测试失败');
      }
    } catch (error) {
      this.message.error(`显示值测试失败: ${error}`);
    } finally {
      this.isDisplayValueTesting = false;
    }
  }

  /**
   * 执行报警测试
   */
  async executeAlarmTest(alarmType: string): Promise<void> {
    if (!this.instance) {
      this.message.error('测试实例不存在');
      return;
    }

    this.isAlarmTesting = true;
    this.currentAlarmType = alarmType;

    try {
      const response = await this.manualTestService.executeAlarmTest(
        this.instance.instance_id,
        alarmType
      );

      if (response.success) {
        this.message.success(`${alarmType}报警测试成功: ${response.message}`);
      } else {
        this.message.error(response.message || `${alarmType}报警测试失败`);
      }
    } catch (error) {
      this.message.error(`${alarmType}报警测试失败: ${error}`);
    } finally {
      this.isAlarmTesting = false;
      this.currentAlarmType = '';
    }
  }

  /**
   * 复位报警测试（恢复到显示值核对的值）
   */
  async resetAlarmTest(alarmType: string): Promise<void> {
    if (!this.instance) {
      this.message.error('测试实例不存在');
      return;
    }

    if (this.displayTestValue === null || this.displayTestValue === undefined) {
      this.message.error('请先执行显示值核对测试');
      return;
    }

    this.isAlarmTesting = true;
    this.currentAlarmType = `${alarmType}_RESET`;

    try {
      const response = await this.manualTestService.executeShowValueTest(
        this.instance.instance_id,
        this.displayTestValue
      );

      if (response.success) {
        this.message.success(`${alarmType}报警复位成功，已恢复到显示值: ${this.displayTestValue.toFixed(2)}`);
      } else {
        this.message.error(response.message || `${alarmType}报警复位失败`);
      }
    } catch (error) {
      this.message.error(`${alarmType}报警复位失败: ${error}`);
    } finally {
      this.isAlarmTesting = false;
      this.currentAlarmType = '';
    }
  }

  /**
   * 执行维护功能测试
   */
  async executeMaintenanceTest(enable: boolean): Promise<void> {
    if (!this.instance) {
      this.message.error('测试实例不存在');
      return;
    }

    this.isMaintenanceTesting = true;

    try {
      const response = await this.manualTestService.executeMaintenanceTest(
        this.instance.instance_id,
        enable
      );

      if (response.success) {
        const action = enable ? '启用' : '复位';
        this.message.success(`维护功能${action}成功: ${response.message}`);
      } else {
        this.message.error(response.message || '维护功能测试失败');
      }
    } catch (error) {
      this.message.error(`维护功能测试失败: ${error}`);
    } finally {
      this.isMaintenanceTesting = false;
    }
  }
}
