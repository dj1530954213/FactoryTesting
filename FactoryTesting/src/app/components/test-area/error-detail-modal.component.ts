import { Component, Input, Output, EventEmitter, OnInit, OnChanges } from '@angular/core';
import { CommonModule } from '@angular/common';
import { NzModalModule } from 'ng-zorro-antd/modal';
import { NzTableModule } from 'ng-zorro-antd/table';
import { NzTagModule } from 'ng-zorro-antd/tag';
import { NzDescriptionsModule } from 'ng-zorro-antd/descriptions';
import { NzDividerModule } from 'ng-zorro-antd/divider';
import { NzButtonModule } from 'ng-zorro-antd/button';
import { NzIconModule } from 'ng-zorro-antd/icon';
import { NzTypographyModule } from 'ng-zorro-antd/typography';
import { ChannelTestInstance, ChannelPointDefinition, SubTestItem, SubTestExecutionResult, SubTestStatus } from '../../models';

/**
 * 错误详情查看模态框组件
 * 用于显示通道测试的详细错误信息
 */
@Component({
  selector: 'app-error-detail-modal',
  standalone: true,
  imports: [
    CommonModule,
    NzModalModule,
    NzTableModule,
    NzTagModule,
    NzDescriptionsModule,
    NzDividerModule,
    NzButtonModule,
    NzIconModule,
    NzTypographyModule
  ],
  template: `
    <nz-modal
      [(nzVisible)]="visible"
      [nzTitle]="modalTitle"
      [nzWidth]="800"
      [nzFooter]="null"
      (nzOnCancel)="onCancel()">
      
      <ng-container *nzModalContent>
        <div class="error-detail-content" *ngIf="instance && definition">
          
          <!-- 基本信息 -->
          <nz-descriptions nzBordered [nzColumn]="2" nzSize="small">
            <nz-descriptions-item nzTitle="点位名称">{{ definition.tag }}</nz-descriptions-item>
            <nz-descriptions-item nzTitle="变量名称">{{ definition.variable_name }}</nz-descriptions-item>
            <nz-descriptions-item nzTitle="模块类型">
              <nz-tag [nzColor]="getModuleTypeColor(definition.module_type)">
                {{ definition.module_type }}
              </nz-tag>
            </nz-descriptions-item>
            <nz-descriptions-item nzTitle="数据类型">{{ definition.point_data_type }}</nz-descriptions-item>
            <nz-descriptions-item nzTitle="PLC通信地址">{{ definition.plc_communication_address }}</nz-descriptions-item>
            <nz-descriptions-item nzTitle="测试状态">
              <nz-tag [nzColor]="getStatusColor(instance.overall_status)">
                {{ getStatusText(instance.overall_status) }}
              </nz-tag>
            </nz-descriptions-item>
          </nz-descriptions>

          <nz-divider></nz-divider>

          <!-- 错误摘要 -->
          <div class="error-summary" *ngIf="instance.error_message">
            <h4><span nz-icon nzType="exclamation-circle" nzTheme="fill" style="color: #ff4d4f;"></span> 错误摘要</h4>
            <div class="error-message-box">
              {{ instance.error_message }}
            </div>
          </div>

          <nz-divider></nz-divider>

          <!-- 详细测试结果 -->
          <div class="test-results-section">
            <h4><span nz-icon nzType="unordered-list"></span> 详细测试结果</h4>

            <!-- AI/AO点位显示百分比测试结果 -->
            <div *ngIf="isAnalogType(definition?.module_type)" class="analog-test-results">
              <nz-table
                [nzData]="getPercentageTestResults()"
                [nzPageSize]="0"
                [nzShowPagination]="false"
                nzSize="small">
                <thead>
                  <tr>
                    <th>测试点</th>
                    <th>设定值</th>
                    <th>期望值</th>
                    <th>实际值</th>
                    <th>偏差</th>
                    <th>结果</th>
                  </tr>
                </thead>
                <tbody>
                  <tr *ngFor="let result of getPercentageTestResults()">
                    <td>
                      <nz-tag [nzColor]="'blue'">{{ result.percentage }}%</nz-tag>
                    </td>
                    <td>
                      <span class="value-text">{{ result.setValue | number:'1.2-2' }}</span>
                    </td>
                    <td>
                      <span class="value-text">{{ result.expectedValue | number:'1.2-2' }}</span>
                    </td>
                    <td>
                      <span class="value-text">{{ result.actualValue | number:'1.2-2' }}</span>
                    </td>
                    <td>
                      <span class="deviation-text" [class.deviation-error]="result.deviation > 2">
                        {{ result.deviation | number:'1.2-2' }}%
                      </span>
                    </td>
                    <td>
                      <nz-tag [nzColor]="result.deviation <= 2 ? 'success' : 'error'">
                        {{ result.deviation <= 2 ? '通过' : '失败' }}
                      </nz-tag>
                    </td>
                  </tr>
                </tbody>
              </nz-table>
            </div>

            <!-- DI/DO点位显示简化的测试结果 -->
            <div *ngIf="!isAnalogType(definition?.module_type)" class="digital-test-results">
              <nz-table
                [nzData]="getDigitalTestResults()"
                [nzPageSize]="0"
                [nzShowPagination]="false"
                nzSize="small">
                <thead>
                  <tr>
                    <th>测试项</th>
                    <th>结果</th>
                  </tr>
                </thead>
                <tbody>
                  <tr *ngFor="let result of getDigitalTestResults()">
                    <td>
                      <nz-tag [nzColor]="getTestItemColor(result.testItem)">
                        {{ getTestItemText(result.testItem) }}
                      </nz-tag>
                    </td>
                    <td>
                      <nz-tag [nzColor]="result.status === 'Passed' ? 'success' : result.status === 'Failed' ? 'error' : 'default'">
                        {{ getSubTestStatusText(result.status) }}
                      </nz-tag>
                    </td>
                  </tr>
                </tbody>
              </nz-table>
            </div>
          </div>

          <!-- 针对不同点位类型的专门信息 -->
          <nz-divider></nz-divider>
          <div class="type-specific-info">
            <h4><span nz-icon nzType="info-circle"></span> {{ definition.module_type }} 类型测试说明</h4>
            <div [ngSwitch]="definition.module_type">
              
              <!-- AI类型说明 -->
              <div *ngSwitchCase="'AI'" class="ai-info">
                <p><strong>模拟量输入(AI)测试流程：</strong></p>
                <ol>
                  <li>测试PLC AO按序输出: 0%, 25%, 50%, 75%, 100%</li>
                  <li>被测PLC AI采集对应数值</li>
                  <li>验证采集值与期望值的偏差在允许范围内(≤2%)</li>
                </ol>
                <p><strong>量程范围：</strong> {{ definition.range_low_limit || definition.analog_range_min || 0 }} ~ {{ definition.range_high_limit || definition.analog_range_max || 100 }}</p>
                <p><strong>允许偏差：</strong> ≤2%</p>
              </div>

              <!-- AO类型说明 -->
              <div *ngSwitchCase="'AO'" class="ao-info">
                <p><strong>模拟量输出(AO)测试流程：</strong></p>
                <ol>
                  <li>被测PLC AO按序输出: 0%, 25%, 50%, 75%, 100%</li>
                  <li>测试PLC AI采集对应数值</li>
                  <li>验证采集值与期望值的偏差在允许范围内(≤5%)</li>
                </ol>
                <p><strong>量程范围：</strong> {{ definition.analog_range_min || 'N/A' }} ~ {{ definition.analog_range_max || 'N/A' }}</p>
              </div>

              <!-- DI类型说明 -->
              <div *ngSwitchCase="'DI'" class="di-info">
                <p><strong>数字量输入(DI)测试流程：</strong></p>
                <ol>
                  <li>测试PLC DO输出低电平，检查被测PLC DI是否显示"断开"</li>
                  <li>测试PLC DO输出高电平，检查被测PLC DI是否显示"接通"</li>
                  <li>测试PLC DO输出低电平，检查被测PLC DI是否显示"断开"</li>
                </ol>
                <p><strong>接线方式：</strong> 标准接线</p>
              </div>

              <!-- DO类型说明 -->
              <div *ngSwitchCase="'DO'" class="do-info">
                <p><strong>数字量输出(DO)测试流程：</strong></p>
                <ol>
                  <li>被测PLC DO输出低电平，检查测试PLC DI是否显示"断开"</li>
                  <li>被测PLC DO输出高电平，检查测试PLC DI是否显示"接通"</li>
                  <li>被测PLC DO输出低电平，检查测试PLC DI是否显示"断开"</li>
                </ol>
                <p><strong>接线方式：</strong> 标准接线</p>
              </div>

              <!-- 默认情况 -->
              <div *ngSwitchDefault>
                <p>暂无该类型的详细测试说明。</p>
              </div>
            </div>
          </div>

        </div>
      </ng-container>
    </nz-modal>
  `,
  styles: [`
    .error-detail-content {
      padding: 16px 0;
    }

    .error-summary h4 {
      margin-bottom: 12px;
      color: #ff4d4f;
      font-weight: 600;
    }

    .error-message-box {
      background: #fff2f0;
      border: 1px solid #ffccc7;
      border-radius: 6px;
      padding: 12px;
      color: #a8071a;
      font-family: 'Courier New', monospace;
      white-space: pre-wrap;
      word-break: break-word;
    }

    .test-results-section h4,
    .type-specific-info h4 {
      margin-bottom: 16px;
      color: #1890ff;
      font-weight: 600;
    }

    .value-text {
      font-family: 'Courier New', monospace;
      font-weight: 500;
    }

    .description-text {
      max-width: 200px;
      overflow: hidden;
      text-overflow: ellipsis;
      white-space: nowrap;
      display: inline-block;
    }

    .ai-info, .ao-info, .di-info, .do-info {
      background: #f6ffed;
      border: 1px solid #b7eb8f;
      border-radius: 6px;
      padding: 16px;
    }

    .ai-info ol, .ao-info ol, .di-info ol, .do-info ol {
      margin: 8px 0;
      padding-left: 20px;
    }

    .ai-info li, .ao-info li, .di-info li, .do-info li {
      margin: 4px 0;
    }

    .ai-info p, .ao-info p, .di-info p, .do-info p {
      margin: 8px 0;
    }

    .ai-info strong, .ao-info strong, .di-info strong, .do-info strong {
      color: #389e0d;
    }

    .deviation-text {
      font-family: 'Courier New', monospace;
      font-weight: 600;
      color: #52c41a;
    }

    .deviation-text.deviation-error {
      color: #ff4d4f;
    }

    .analog-test-results .ant-table-tbody > tr > td {
      padding: 8px 12px;
    }

    .digital-test-results .ant-table-tbody > tr > td {
      padding: 12px;
    }
  `]
})
export class ErrorDetailModalComponent implements OnInit, OnChanges {
  @Input() visible = false;
  @Input() instance: ChannelTestInstance | null = null;
  @Input() definition: ChannelPointDefinition | null = null;
  @Output() visibleChange = new EventEmitter<boolean>();

  modalTitle = '';

  ngOnInit() {
    this.updateModalTitle();
  }

  ngOnChanges() {
    this.updateModalTitle();
  }

  private updateModalTitle() {
    if (this.definition) {
      this.modalTitle = `错误详情 - ${this.definition.tag} (${this.definition.module_type})`;
    } else {
      this.modalTitle = '错误详情';
    }
  }

  onCancel() {
    this.visible = false;
    this.visibleChange.emit(false);
  }

  getModuleTypeColor(moduleType: string): string {
    const colors: { [key: string]: string } = {
      'AI': 'blue',
      'AO': 'green', 
      'DI': 'orange',
      'DO': 'purple'
    };
    return colors[moduleType] || 'default';
  }

  getStatusColor(status: string): string {
    if (status.includes('Passed')) return 'success';
    if (status.includes('Failed')) return 'error';
    if (status.includes('Testing')) return 'processing';
    return 'default';
  }

  getStatusText(status: string): string {
    const statusMap: { [key: string]: string } = {
      'NotTested': '未测试',
      'WiringConfirmed': '接线确认',
      'HardPointTesting': '硬点测试中',
      'TestCompletedPassed': '测试通过',
      'TestCompletedFailed': '测试失败',
      'ManualTesting': '手动测试中',
      'AllCompleted': '全部完成'
    };
    return statusMap[status] || status;
  }

  getTestItemColor(testItem: string): string {
    const colors: { [key: string]: string } = {
      'HardPoint': 'red',
      'HighHighAlarm': 'volcano',
      'HighAlarm': 'orange',
      'LowAlarm': 'gold',
      'LowLowAlarm': 'lime',
      'ShowValue': 'green',
      'TrendCheck': 'cyan',
      'ReportCheck': 'blue',
      'MaintenanceFunction': 'geekblue'
    };
    return colors[testItem] || 'default';
  }

  getTestItemText(testItem: string): string {
    const itemMap: { [key: string]: string } = {
      'HardPoint': '硬点测试',
      'HighHighAlarm': '高高报警',
      'HighAlarm': '高报警', 
      'LowAlarm': '低报警',
      'LowLowAlarm': '低低报警',
      'ShowValue': '显示值',
      'TrendCheck': '趋势检查',
      'ReportCheck': '报表检查',
      'MaintenanceFunction': '维护功能'
    };
    return itemMap[testItem] || testItem;
  }

  getSubTestStatusText(status: string): string {
    const statusMap: { [key: string]: string } = {
      'NotTested': '未测试',
      'Passed': '通过',
      'Failed': '失败',
      'NotApplicable': '不适用'
    };
    return statusMap[status] || status;
  }

  getSubTestResults(): any[] {
    if (!this.instance?.sub_test_results) return [];

    return Object.entries(this.instance.sub_test_results).map(([testItem, result]) => ({
      testItem,
      status: result.status,
      expectedValue: result.expected_value,
      actualValue: result.actual_value,
      details: result.details
    }));
  }

  /**
   * 判断是否为模拟量类型
   */
  isAnalogType(moduleType?: string): boolean {
    return moduleType === 'AI' || moduleType === 'AO';
  }

  /**
   * 获取百分比测试结果（AI/AO点位）
   */
  getPercentageTestResults(): any[] {
    if (!this.instance || !this.definition || !this.isAnalogType(this.definition.module_type)) {
      return [];
    }

    const rangeLow = this.definition.range_low_limit || this.definition.analog_range_min || 0;
    const rangeHigh = this.definition.range_high_limit || this.definition.analog_range_max || 100;
    const rangeSpan = rangeHigh - rangeLow;

    const percentages = [0, 25, 50, 75, 100];
    const results = [];

    for (let i = 0; i < percentages.length; i++) {
      const percentage = percentages[i];
      const setValue = rangeLow + (rangeSpan * percentage / 100);
      const expectedValue = setValue; // 对于AI/AO，期望值就是设定值

      // 从实例的百分比测试结果字段中获取实际值
      let actualValue = 0;
      switch (percentage) {
        case 0:
          actualValue = (this.instance as any).test_result_0_percent || 0;
          break;
        case 25:
          actualValue = (this.instance as any).test_result_25_percent || 0;
          break;
        case 50:
          actualValue = (this.instance as any).test_result_50_percent || 0;
          break;
        case 75:
          actualValue = (this.instance as any).test_result_75_percent || 0;
          break;
        case 100:
          actualValue = (this.instance as any).test_result_100_percent || 0;
          break;
      }

      // 计算偏差百分比
      const deviation = rangeSpan > 0 ? Math.abs(actualValue - expectedValue) / rangeSpan * 100 : 0;

      results.push({
        percentage,
        setValue,
        expectedValue,
        actualValue,
        deviation
      });
    }

    return results;
  }

  /**
   * 获取数字量测试结果（DI/DO点位）
   */
  getDigitalTestResults(): any[] {
    if (!this.instance?.sub_test_results || this.isAnalogType(this.definition?.module_type)) {
      return [];
    }

    return Object.entries(this.instance.sub_test_results)
      .filter(([testItem, result]) => {
        // 只显示硬点测试，不显示结果和描述
        return testItem === 'HardPoint';
      })
      .map(([testItem, result]) => ({
        testItem,
        status: result.status
      }));
  }
}
