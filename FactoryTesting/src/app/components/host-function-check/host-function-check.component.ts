import { Component, OnInit, OnDestroy } from '@angular/core';
import { BatchSelectionService } from '../../services/batch-selection.service';
import { filter, Subscription } from 'rxjs';
import { TauriApiService } from '../../services/tauri-api.service';
import { CommonModule } from '@angular/common';
import { FormsModule } from '@angular/forms';
import { NzCardModule } from 'ng-zorro-antd/card';
import { NzButtonModule } from 'ng-zorro-antd/button';
import { NzIconModule } from 'ng-zorro-antd/icon';
import { NzStepsModule } from 'ng-zorro-antd/steps';
import { NzResultModule } from 'ng-zorro-antd/result';
import { NzAlertModule } from 'ng-zorro-antd/alert';
import { NzDividerModule } from 'ng-zorro-antd/divider';
import { NzSpaceModule } from 'ng-zorro-antd/space';
import { NzTagModule } from 'ng-zorro-antd/tag';
import { NzProgressModule } from 'ng-zorro-antd/progress';
import { NzMessageService } from 'ng-zorro-antd/message';
import { NzModalModule, NzModalService } from 'ng-zorro-antd/modal';
import { NzDescriptionsModule } from 'ng-zorro-antd/descriptions';
import { NzListModule } from 'ng-zorro-antd/list';
import { NzEmptyModule } from 'ng-zorro-antd/empty';

export interface FunctionCheckItem {
  id: string;
  name: string;
  description: string;
  instructions: string[];
  status: 'pending' | 'checking' | 'passed' | 'failed';
  icon: string;
  color: string;
  checkTime?: Date;
  notes?: string;
}

@Component({
  selector: 'app-host-function-check',
  standalone: true,
  imports: [
    CommonModule,
    FormsModule,
    NzCardModule,
    NzButtonModule,
    NzIconModule,
    NzStepsModule,
    NzResultModule,
    NzAlertModule,
    NzDividerModule,
    NzSpaceModule,
    NzTagModule,
    NzProgressModule,
    NzModalModule,
    NzDescriptionsModule,
    NzListModule,
    NzEmptyModule
  ],
  templateUrl: './host-function-check.component.html',
  styleUrls: ['./host-function-check.component.css']
})
export class HostFunctionCheckComponent implements OnInit, OnDestroy {

  private batchSub?: Subscription;

  // 功能检查项目列表
  checkItems: FunctionCheckItem[] = [
    {
      id: 'history-trend',
      name: '历史趋势功能',
      description: '检查历史数据趋势图显示是否正常',
      instructions: [
        '1. 打开历史趋势界面',
        '2. 选择任意测试点位',
        '3. 设置时间范围（建议选择最近1小时）',
        '4. 检查趋势曲线是否正常显示',
        '5. 验证数据缩放、平移功能是否正常',
        '6. 检查数据导出功能是否可用'
      ],
      status: 'pending',
      icon: 'line-chart',
      color: 'blue'
    },
    {
      id: 'realtime-trend',
      name: '实时趋势功能',
      description: '检查实时数据趋势图更新是否正常',
      instructions: [
        '1. 打开实时趋势界面',
        '2. 选择正在测试的点位',
        '3. 观察趋势曲线是否实时更新',
        '4. 检查数据刷新频率是否合理',
        '5. 验证暂停/恢复功能是否正常',
        '6. 测试多点位同时显示功能'
      ],
      status: 'pending',
      icon: 'stock',
      color: 'green'
    },
    {
      id: 'report-function',
      name: '报表功能',
      description: '检查测试报表生成和导出功能',
      instructions: [
        '1. 进入报表生成界面',
        '2. 选择测试批次和时间范围',
        '3. 生成测试报表',
        '4. 检查报表内容完整性',
        '5. 验证PDF/Excel导出功能',
        '6. 确认报表格式符合要求'
      ],
      status: 'pending',
      icon: 'file-text',
      color: 'orange'
    },
    {
      id: 'alarm-function',
      name: '报警级别与声音',
      description: '检查报警系统的级别设置和声音提示',
      instructions: [
        '1. 触发一个测试报警（可通过设置异常值）',
        '2. 检查报警级别显示是否正确',
        '3. 验证报警声音是否正常播放',
        '4. 测试不同级别报警的声音区别',
        '5. 检查报警确认功能',
        '6. 验证报警历史记录功能'
      ],
      status: 'pending',
      icon: 'sound',
      color: 'red'
    },
    {
      id: 'operation-log',
      name: '操作日志记录',
      description: '检查系统操作日志记录功能',
      instructions: [
        '1. 执行几个系统操作（如导入数据、开始测试）',
        '2. 打开操作日志界面',
        '3. 检查操作记录是否完整',
        '4. 验证日志时间戳准确性',
        '5. 测试日志搜索和筛选功能',
        '6. 检查日志导出功能'
      ],
      status: 'pending',
      icon: 'file-search',
      color: 'purple'
    }
  ];

  currentCheckIndex = 0;
  isCheckingInProgress = false;
  overallProgress = 0;

  stationName = '';  
  // 是否已成功加载到后端状态
  hasStatusData = false;
  importTime = '';

  /** 后端 function_key -> 前端 card id 映射 */
  /**
   * 后端 function_key 可能为 SNAKE_CASE 或 CamelCase，均做映射
   */
  private readonly functionKeyMap: Record<string, string> = {
    HISTORICAL_TREND: 'history-trend',
    HISTORICALTREND: 'history-trend',
    REALTIME_TREND: 'realtime-trend',
    REALTIMETREND: 'realtime-trend',
    REPORT: 'report-function',
    ALARM_LEVEL_SOUND: 'alarm-function',
    ALARMLEVELSOUND: 'alarm-function',
    OPERATION_LOG: 'operation-log',
    OPERATIONLOG: 'operation-log'
  };

  constructor(
    private message: NzMessageService,
    private modal: NzModalService,
    private api: TauriApiService,
    private batchService: BatchSelectionService
  ) {}

  ngOnInit(): void {
    // 订阅当前批次
    this.batchSub = this.batchService.selectedBatch$
      .pipe(filter(b => !!b))
      .subscribe((batch: any) => {
        if (!batch) { return; }
        this.stationName = batch.station_name ?? '';
        this.importTime = batch.creation_time ?? batch.last_updated_time ?? '';
        // 若批次包含 import_time 字段（兼容后端返回）则优先使用
        // @ts-ignore
        if (batch.import_time) { this.importTime = (batch as any).import_time; }
        this.loadStatuses();
      });
  }
    // 加载已有功能测试状态
  private loadStatuses(): void {
    this.hasStatusData = false;
    if (!this.stationName) { return; }
    // 传空字符串让后端自动回退到最新 import_time
    console.log('[GFT] loadStatuses 调用', this.stationName);
    this.api.getGlobalFunctionTests(this.stationName, '').subscribe({
      next: (records: any[]) => {
        console.log('[GFT] 接收到状态记录', records);
        if (!records || records.length === 0) {
          // 未获取到任何状态，保持提示用户导入数据
          this.hasStatusData = false;
          this.updateProgress();
          return;
        }
        this.hasStatusData = true;
        // 若当前 importTime 为空或与返回记录不一致，则以返回记录为准，确保后续 update 时命中
        if (records.length > 0 && records[0].import_time && this.importTime !== records[0].import_time) {
          this.importTime = records[0].import_time;
          console.log('[GFT] 同步 importTime 为最新批次', this.importTime);
        }
        records.forEach(rec => {
          const id = this.functionKeyMap[ (rec.function_key || '').toUpperCase() ];
          if (!id) { return; }
          const item = this.checkItems.find(i => i.id === id);
          if (item) {
            item.status = rec.status === 'TestCompletedPassed' ? 'passed' : rec.status === 'TestCompletedFailed' ? 'failed' : 'pending';
            if (rec.start_time) { item.checkTime = new Date(rec.start_time); }
          }
        });
        this.updateProgress();
      },
      error: err => console.error('[GFT] 加载失败', err)
    });
  }

  ngOnDestroy(): void {
    this.batchSub?.unsubscribe();
    // 清理资源
  }

  /**
   * 开始检查指定项目
   */
  startCheck(item: FunctionCheckItem): void {
    this.currentCheckIndex = this.checkItems.findIndex(i => i.id === item.id);
    item.status = 'checking';
    this.isCheckingInProgress = true;
    
    this.message.info(`开始检查: ${item.name}`);
    
    // 显示检查指导模态框
    this.showCheckInstructions(item);
  }

  /**
   * 显示检查指导模态框
   */
  showCheckInstructions(item: FunctionCheckItem): void {
    this.modal.create({
      nzTitle: `${item.name} - 检查指导`,
      nzContent: `
        <div class="check-instructions">
          <p><strong>功能描述：</strong>${item.description}</p>
          <div class="instructions-list">
            <h4>检查步骤：</h4>
            <ol>
              ${item.instructions.map(instruction => `<li>${instruction}</li>`).join('')}
            </ol>
          </div>
          <div class="check-tips">
            <p><strong>注意事项：</strong></p>
            <ul>
              <li>请按照步骤逐一检查</li>
              <li>如发现异常，请记录具体问题</li>
              <li>检查完成后选择相应的结果</li>
            </ul>
          </div>
        </div>
      `,
      nzWidth: 600,
      nzFooter: [
        {
          label: '检查通过',
          type: 'primary',
          onClick: () => this.completeCheck(item, 'passed')
        },
        {
          label: '检查失败',
          type: 'default',
          danger: true,
          onClick: () => this.completeCheck(item, 'failed')
        },
        {
          label: '取消检查',
          type: 'default',
          onClick: () => this.cancelCheck(item)
        }
      ]
    });
  }

  /**
   * 完成检查
   */
  completeCheck(item: FunctionCheckItem, result: 'passed' | 'failed'): void {
    if (!this.stationName || !this.importTime) {
      console.warn('[GFT] 批次信息缺失，忽略更新');
      return;
    }
    if (!this.importTime) {
      this.importTime = new Date().toISOString();
    }
    // 更新后端状态
    this.api.updateGlobalFunctionTest({
      station_name: this.stationName,
      import_time: this.importTime,
      function_key: this.mapIdToKey(item.id),
      status: result === 'passed' ? 'TestCompletedPassed' : 'TestCompletedFailed',
      start_time: item.checkTime ? item.checkTime.toISOString() : new Date().toISOString(),
      end_time: new Date().toISOString()
    }).subscribe({
      error: err => console.error('[GFT] 更新状态失败', err)
    });
    item.status = result;
    item.checkTime = new Date();
    this.isCheckingInProgress = false;
    
    this.updateProgress();
    
    if (result === 'passed') {
      this.message.success(`${item.name} 检查通过`);
    } else {
      this.message.error(`${item.name} 检查失败`);
    }
    
    this.modal.closeAll();
  }

  /**
   * 取消检查
   */
  cancelCheck(item: FunctionCheckItem): void {
    item.status = 'pending';
    this.isCheckingInProgress = false;
    this.modal.closeAll();
  }

  /**
   * 重置检查项目
   */
  resetCheck(item: FunctionCheckItem): void {
    item.status = 'pending';
    item.checkTime = undefined;
    item.notes = undefined;
    this.updateProgress();
    this.message.info(`已重置 ${item.name} 的检查状态`);
  }

  /**
   * 重置所有检查
   */
  resetAllChecks(): void {
    if (!this.stationName || !this.importTime) {
      console.warn('[GFT] 批次信息缺失，忽略重置');
      return;
    }
    if (!this.importTime) {
      this.importTime = new Date().toISOString();
    }
    this.api.resetGlobalFunctionTests(this.stationName, this.importTime).subscribe({
      error: err => console.error('[GFT] 重置状态失败', err)
    });
    this.modal.confirm({
      nzTitle: '确认重置',
      nzContent: '确定要重置所有检查项目吗？',
      nzOnOk: () => {
        this.hasStatusData = false;
    this.checkItems.forEach(item => {
          item.status = 'pending';
          item.checkTime = undefined;
          item.notes = undefined;
        });
        this.updateProgress();
        this.message.success('已重置所有检查项目');
      }
    });
  }

  /**
   * 将组件id映射为后端 GlobalFunctionKey
   */
  private mapIdToKey(id: string): string {
    switch (id) {
      case 'history-trend': return 'HistoricalTrend';
      case 'realtime-trend': return 'RealTimeTrend';
      case 'report-function': return 'Report';
      case 'alarm-function': return 'AlarmLevelSound';
      case 'operation-log': return 'OperationLog';
      default: return id;
    }
  }

  /**
   * 更新整体进度
   */
  updateProgress(): void {
    const completedItems = this.checkItems.filter(item => 
      item.status === 'passed' || item.status === 'failed'
    ).length;
    this.overallProgress = Math.round((completedItems / this.checkItems.length) * 100);
  }

  /**
   * 获取状态标签颜色
   */
  getStatusColor(status: string): string {
    switch (status) {
      case 'passed': return 'success';
      case 'failed': return 'error';
      case 'checking': return 'processing';
      default: return 'default';
    }
  }

  /**
   * 获取状态文本
   */
  getStatusText(status: string): string {
    switch (status) {
      case 'passed': return '通过';
      case 'failed': return '失败';
      case 'checking': return '检查中';
      default: return '待检查';
    }
  }

  /**
   * 获取通过的检查项数量
   */
  getPassedCount(): number {
    return this.checkItems.filter(item => item.status === 'passed').length;
  }

  /**
   * 获取失败的检查项数量
   */
  getFailedCount(): number {
    return this.checkItems.filter(item => item.status === 'failed').length;
  }

  /**
   * 检查是否所有项目都已完成
   */
  isAllCompleted(): boolean {
    return this.checkItems.every(item => 
      item.status === 'passed' || item.status === 'failed'
    );
  }

  /**
   * 生成检查报告
   */
  generateReport(): void {
    if (!this.isAllCompleted()) {
      this.message.warning('请完成所有检查项目后再生成报告');
      return;
    }

    // 这里可以调用后端API生成报告
    this.message.success('检查报告生成功能开发中...');
  }
}
