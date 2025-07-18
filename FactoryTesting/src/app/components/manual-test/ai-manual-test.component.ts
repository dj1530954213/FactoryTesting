import { Component, Input, Output, EventEmitter, OnInit, OnDestroy, ChangeDetectorRef } from '@angular/core';
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
import { ModuleType } from '../../models';

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
            [nzValue]="realtimeCurrentValue" 
            nzSuffix="工程单位"
            [nzValueStyle]="{ color: '#1890ff' }">
          </nz-statistic>
          <nz-statistic 
            nzTitle="SLL设定值" 
            [nzValue]="realtimeSllSetPoint" 
            [nzValueStyle]="{ color: '#ff4d4f' }">
          </nz-statistic>
          <nz-statistic 
            nzTitle="SL设定值" 
            [nzValue]="realtimeSlSetPoint" 
            [nzValueStyle]="{ color: '#faad14' }">
          </nz-statistic>
          <nz-statistic 
            nzTitle="SH设定值" 
            [nzValue]="realtimeShSetPoint" 
            [nzValueStyle]="{ color: '#faad14' }">
          </nz-statistic>
          <nz-statistic 
            nzTitle="SHH设定值" 
            [nzValue]="realtimeShhSetPoint" 
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
                      step="0.001">
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
                  [disabled]="isSubItemPassedOrSkipped(ManualTestSubItem.ShowValueCheck)"
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

          <!-- 低低报警测试 -->
          <nz-card nzSize="small" class="test-item-card" *ngIf="shouldShowSubItem(ManualTestSubItem.LowLowAlarmTest)">
            <div class="test-item-header">
              <span class="test-item-title">低低报警测试</span>
              <nz-tag [nzColor]="getSubItemStatusColor(ManualTestSubItem.LowLowAlarmTest)">
                {{ getSubItemStatusText(ManualTestSubItem.LowLowAlarmTest) }}
              </nz-tag>
            </div>
            <div class="test-item-content">
              <p>设定值: {{ getStaticSllSetValue() }}</p>
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
                  [disabled]="isSubItemPassedOrSkipped(ManualTestSubItem.LowLowAlarmTest)"
                  (click)="completeSubItem(ManualTestSubItem.LowLowAlarmTest)">
                  <i nz-icon nzType="check"></i>
                  确认通过
                </button>
                <button
                  nz-button
                  nzSize="small"
                  [disabled]="isSubItemPassedOrSkipped(ManualTestSubItem.LowLowAlarmTest)"
                  (click)="skipSubItem(ManualTestSubItem.LowLowAlarmTest)">
                  <i nz-icon nzType="close"></i>
                  测试失败
                </button>
              </div>
            </div>
          </nz-card>

          <!-- 低报警测试 -->
          <nz-card nzSize="small" class="test-item-card" *ngIf="shouldShowSubItem(ManualTestSubItem.LowAlarmTest)">
            <div class="test-item-header">
              <span class="test-item-title">低报警测试</span>
              <nz-tag [nzColor]="getSubItemStatusColor(ManualTestSubItem.LowAlarmTest)">
                {{ getSubItemStatusText(ManualTestSubItem.LowAlarmTest) }}
              </nz-tag>
            </div>
            <div class="test-item-content">
              <p>设定值: {{ getStaticSlSetValue() }}</p>
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
                  [disabled]="isSubItemPassedOrSkipped(ManualTestSubItem.LowAlarmTest)"
                  (click)="completeSubItem(ManualTestSubItem.LowAlarmTest)">
                  <i nz-icon nzType="check"></i>
                  确认通过
                </button>
                <button
                  nz-button
                  nzSize="small"
                  [disabled]="isSubItemPassedOrSkipped(ManualTestSubItem.LowAlarmTest)"
                  (click)="skipSubItem(ManualTestSubItem.LowAlarmTest)">
                  <i nz-icon nzType="close"></i>
                  测试失败
                </button>
              </div>
            </div>
          </nz-card>

          <!-- 高报警测试 -->
          <nz-card nzSize="small" class="test-item-card" *ngIf="shouldShowSubItem(ManualTestSubItem.HighAlarmTest)">
            <div class="test-item-header">
              <span class="test-item-title">高报警测试</span>
              <nz-tag [nzColor]="getSubItemStatusColor(ManualTestSubItem.HighAlarmTest)">
                {{ getSubItemStatusText(ManualTestSubItem.HighAlarmTest) }}
              </nz-tag>
            </div>
            <div class="test-item-content">
              <p>设定值: {{ getStaticShSetValue() }}</p>
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
                  [disabled]="isSubItemPassedOrSkipped(ManualTestSubItem.HighAlarmTest)"
                  (click)="completeSubItem(ManualTestSubItem.HighAlarmTest)">
                  <i nz-icon nzType="check"></i>
                  确认通过
                </button>
                <button
                  nz-button
                  nzSize="small"
                  [disabled]="isSubItemPassedOrSkipped(ManualTestSubItem.HighAlarmTest)"
                  (click)="skipSubItem(ManualTestSubItem.HighAlarmTest)">
                  <i nz-icon nzType="close"></i>
                  测试失败
                </button>
              </div>
            </div>
          </nz-card>

          <!-- 高高报警测试 -->
          <nz-card nzSize="small" class="test-item-card" *ngIf="shouldShowSubItem(ManualTestSubItem.HighHighAlarmTest)">
            <div class="test-item-header">
              <span class="test-item-title">高高报警测试</span>
              <nz-tag [nzColor]="getSubItemStatusColor(ManualTestSubItem.HighHighAlarmTest)">
                {{ getSubItemStatusText(ManualTestSubItem.HighHighAlarmTest) }}
              </nz-tag>
            </div>
            <div class="test-item-content">
              <p>设定值: {{ getStaticShhSetValue() }}</p>
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
                  [disabled]="isSubItemPassedOrSkipped(ManualTestSubItem.HighHighAlarmTest)"
                  (click)="completeSubItem(ManualTestSubItem.HighHighAlarmTest)">
                  <i nz-icon nzType="check"></i>
                  确认通过
                </button>
                <button
                  nz-button
                  nzSize="small"
                  [disabled]="isSubItemPassedOrSkipped(ManualTestSubItem.HighHighAlarmTest)"
                  (click)="skipSubItem(ManualTestSubItem.HighHighAlarmTest)">
                  <i nz-icon nzType="close"></i>
                  测试失败
                </button>
              </div>
            </div>
          </nz-card>

          <!-- 维护功能测试 -->
          <nz-card nzSize="small" class="test-item-card" *ngIf="shouldShowSubItem(ManualTestSubItem.MaintenanceFunction)">
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

  // 实时监控数据显示属性
  realtimeCurrentValue: string = '读取中...';
  realtimeSllSetPoint: string = '读取中...';
  realtimeSlSetPoint: string = '读取中...';
  realtimeShSetPoint: string = '读取中...';
  realtimeShhSetPoint: string = '读取中...';

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
    private modal: NzModalService,
    private cdr: ChangeDetectorRef
  ) {}

  ngOnInit(): void {
    // 初始化显示值
    this.updateRealtimeValues();
    
    // 订阅PLC监控数据
    this.subscriptions.add(
      this.manualTestService.testStatusUpdated$.subscribe(() => {
        // 后端测试状态更新后刷新视图，确保按钮禁用状态及时生效
        this.cdr.markForCheck();
      })
    );

    // 订阅PLC监控数据
    this.subscriptions.add(
      this.plcMonitoringService.currentMonitoringData$.subscribe(data => {
        this.updateRealtimeValues();
      })
    );
  }

  /**
   * 更新实时显示值
   */
  private updateRealtimeValues(): void {
    this.realtimeCurrentValue = this.plcMonitoringService.getFormattedMonitoringValue('currentValue', ModuleType.AI);
    this.realtimeSllSetPoint = this.plcMonitoringService.getFormattedMonitoringValue('sllSetPoint', ModuleType.AI);
    this.realtimeSlSetPoint = this.plcMonitoringService.getFormattedMonitoringValue('slSetPoint', ModuleType.AI);
    this.realtimeShSetPoint = this.plcMonitoringService.getFormattedMonitoringValue('shSetPoint', ModuleType.AI);
    this.realtimeShhSetPoint = this.plcMonitoringService.getFormattedMonitoringValue('shhSetPoint', ModuleType.AI);
  }

  ngOnDestroy(): void {
    this.subscriptions.unsubscribe();
  }

  /**
   * 获取当前值
   */
  getCurrentValue(): string {
    return this.plcMonitoringService.getFormattedMonitoringValue('currentValue', ModuleType.AI) || '读取中...';
  }

  /**
   * 获取SLL设定值（来自PLC实时监控）
   */
  getSllSetPoint(): string {
    return this.plcMonitoringService.getFormattedMonitoringValue('sllSetPoint', ModuleType.AI);
  }

  /**
   * 获取SL设定值（来自PLC实时监控）
   */
  getSlSetPoint(): string {
    return this.plcMonitoringService.getFormattedMonitoringValue('slSetPoint', ModuleType.AI);
  }

  /**
   * 获取SH设定值（来自PLC实时监控）
   */
  getShSetPoint(): string {
    return this.plcMonitoringService.getFormattedMonitoringValue('shSetPoint', ModuleType.AI);
  }

  /**
   * 获取SHH设定值（来自PLC实时监控）
   */
  getShhSetPoint(): string {
    return this.plcMonitoringService.getFormattedMonitoringValue('shhSetPoint', ModuleType.AI);
  }

  /**
   * 获取SLL静态设定值（来自点表）
   */
  getStaticSllSetValue(): string {
    if (this.definition && this.definition.sll_set_value !== undefined && this.definition.sll_set_value !== null) {
      return this.definition.sll_set_value.toFixed(3);
    }
    return '--';
  }

  /**
   * 获取SL静态设定值（来自点表）
   */
  getStaticSlSetValue(): string {
    if (this.definition && this.definition.sl_set_value !== undefined && this.definition.sl_set_value !== null) {
      return this.definition.sl_set_value.toFixed(3);
    }
    return '--';
  }

  /**
   * 获取SH静态设定值（来自点表）
   */
  getStaticShSetValue(): string {
    if (this.definition && this.definition.sh_set_value !== undefined && this.definition.sh_set_value !== null) {
      return this.definition.sh_set_value.toFixed(3);
    }
    return '--';
  }

  /**
   * 获取SHH静态设定值（来自点表）
   */
  getStaticShhSetValue(): string {
    if (this.definition && this.definition.shh_set_value !== undefined && this.definition.shh_set_value !== null) {
      return this.definition.shh_set_value.toFixed(3);
    }
    return '--';
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
    console.log('🎯 [前端] 点击生成随机值按钮:', this.instance?.instance_id);
    if (!this.instance) {
      this.message.error('测试实例不存在');
      return;
    }

    try {
      const response = await this.manualTestService.generateRandomDisplayValue(this.instance.instance_id);
      if (response.success) {
        this.displayTestValue = parseFloat(response.randomValue.toFixed(3));
        this.message.success(`已生成随机值: ${response.randomValue.toFixed(3)}`);
        console.log('✅ [前端] 生成随机值成功:', response.randomValue);
      } else {
        this.message.error(response.message || '生成随机值失败');
        console.error('❌ [前端] 生成随机值失败:', response.message);
      }
    } catch (error) {
      console.error('❌ [前端] 生成随机值异常:', error);
      this.message.error(`生成随机值失败: ${error}`);
    }
  }

  /**
   * 执行显示值核对测试
   */
  async executeDisplayValueTest(): Promise<void> {
    console.log('🎯 [前端] 点击显示值核对测试按钮:', this.instance?.instance_id, '测试值:', this.displayTestValue);
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
        console.log('✅ [前端] 显示值测试成功:', response.message);
      } else {
        this.message.error(response.message || '显示值测试失败');
        console.error('❌ [前端] 显示值测试失败:', response.message);
      }
    } catch (error) {
      console.error('❌ [前端] 显示值测试异常:', error);
      this.message.error(`显示值测试失败: ${error}`);
    } finally {
      this.isDisplayValueTesting = false;
    }
  }

  /**
   * 执行报警测试
   */
  async executeAlarmTest(alarmType: string): Promise<void> {
    console.log('🎯 [前端] 点击报警测试按钮:', alarmType, '实例ID:', this.instance?.instance_id);
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
        console.log('✅ [前端] 报警测试成功:', alarmType, response.message);
      } else {
        this.message.error(response.message || `${alarmType}报警测试失败`);
        console.error('❌ [前端] 报警测试失败:', alarmType, response.message);
      }
    } catch (error) {
      console.error('❌ [前端] 报警测试异常:', alarmType, error);
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
        this.message.success(`${alarmType}报警复位成功，已恢复到显示值: ${this.displayTestValue.toFixed(3)}`);
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
    console.log('🎯 [前端] 点击维护功能测试按钮:', enable ? '启用' : '复位', '实例ID:', this.instance?.instance_id);
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
        console.log('✅ [前端] 维护功能测试成功:', action, response.message);
      } else {
        this.message.error(response.message || '维护功能测试失败');
        console.error('❌ [前端] 维护功能测试失败:', response.message);
      }
    } catch (error) {
      console.error('❌ [前端] 维护功能测试异常:', error);
      this.message.error(`维护功能测试失败: ${error}`);
    } finally {
      this.isMaintenanceTesting = false;
    }
  }
}
