import { Component, OnInit, OnDestroy } from '@angular/core';
import { CommonModule } from '@angular/common';
import { FormsModule } from '@angular/forms';
import { ActivatedRoute, Router } from '@angular/router';
import { Subscription, interval } from 'rxjs';
import { TauriApiService } from '../../services/tauri-api.service';
import {
  TestExecutionRequest,
  TestExecutionResponse,
  TestProgressUpdate,
  ChannelPointDefinition,
  ChannelTestInstance,
  TestBatchInfo,
  RawTestOutcome,
  ModuleType,
  PointDataType,
  OverallTestStatus
} from '../../models';

interface TestExecutionData {
  batch: TestBatchInfo;
  instances: ChannelTestInstance[];
  definitions: ChannelPointDefinition[];
  outcomes: RawTestOutcome[];
  progress: TestProgressUpdate[];
}

interface TestStatistics {
  total: number;
  completed: number;
  passed: number;
  failed: number;
  inProgress: number;
  notStarted: number;
  successRate: number;
}

@Component({
  selector: 'app-test-execution',
  standalone: true,
  imports: [CommonModule, FormsModule],
  template: `
    <div class="test-execution-container">
      <h2>测试执行管理</h2>
      
      <!-- 批次创建区域 -->
      <div class="section">
        <h3>创建测试批次</h3>
        <div class="form-group">
          <label>产品型号:</label>
          <input [(ngModel)]="newBatch.product_model" placeholder="输入产品型号">
        </div>
        <div class="form-group">
          <label>序列号:</label>
          <input [(ngModel)]="newBatch.serial_number" placeholder="输入序列号">
        </div>
        <div class="form-group">
          <label>操作员:</label>
          <input [(ngModel)]="newBatch.operator_name" placeholder="输入操作员姓名">
        </div>
        <div class="form-group">
          <label>自动开始:</label>
          <input type="checkbox" [(ngModel)]="autoStart">
        </div>
        <button (click)="createAndSubmitBatch()" [disabled]="isLoading">
          {{ isLoading ? '创建中...' : '创建并提交测试' }}
        </button>
      </div>

      <!-- 当前批次控制区域 -->
      <div class="section" *ngIf="currentBatchId">
        <h3>批次控制 - {{ currentBatchId }}</h3>
        <div class="control-buttons">
          <button (click)="startBatch()" [disabled]="!canStart()">开始测试</button>
          <button (click)="pauseBatch()" [disabled]="!canPause()">暂停测试</button>
          <button (click)="resumeBatch()" [disabled]="!canResume()">恢复测试</button>
          <button (click)="stopBatch()" [disabled]="!canStop()">停止测试</button>
          <button (click)="cleanupBatch()" [disabled]="!canCleanup()">清理批次</button>
        </div>
        <div class="status-info">
          <p>状态: {{ currentBatchStatus }}</p>
          <p>实例数量: {{ currentInstanceCount }}</p>
        </div>
      </div>

      <!-- 进度显示区域 -->
      <div class="section" *ngIf="currentBatchId">
        <h3>测试进度</h3>
        <div class="progress-container">
          <div class="progress-summary">
            <p>总进度: {{ getOverallProgress() }}%</p>
            <p>完成实例: {{ getCompletedInstances() }} / {{ progressUpdates.length }}</p>
          </div>
          <div class="progress-list">
            <div *ngFor="let progress of progressUpdates" class="progress-item">
              <div class="progress-header">
                <span class="point-tag">{{ progress.point_tag }}</span>
                <span class="status" [class]="getStatusClass(progress.overall_status)">
                  {{ getStatusLabel(progress.overall_status) }}
                </span>
              </div>
              <div class="progress-bar">
                <div class="progress-fill" 
                     [style.width.%]="getProgressPercentage(progress)">
                </div>
              </div>
              <div class="progress-details">
                <span>{{ progress.completed_sub_tests }} / {{ progress.total_sub_tests }} 子测试</span>
                <span class="timestamp">{{ formatTimestamp(progress.timestamp) }}</span>
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- 测试结果区域 -->
      <div class="section" *ngIf="currentBatchId">
        <h3>测试结果</h3>
        <div class="results-summary">
          <p>总结果: {{ testResults.length }} 个</p>
          <p>成功: {{ getSuccessCount() }}</p>
          <p>失败: {{ getFailureCount() }}</p>
        </div>
        <div class="results-list">
          <div *ngFor="let result of testResults" class="result-item">
            <div class="result-header">
              <span class="instance-id">{{ result.channel_instance_id }}</span>
              <span class="success-status" [class]="result.success ? 'success' : 'failure'">
                {{ result.success ? '成功' : '失败' }}
              </span>
            </div>
            <div class="result-details">
              <p>子测试项: {{ getSubTestItemLabel(result.sub_test_item) }}</p>
              <p *ngIf="result.message">消息: {{ result.message }}</p>
              <p>时间: {{ formatTimestamp(result.start_time) }}</p>
            </div>
          </div>
        </div>
      </div>

      <!-- 系统状态区域 -->
      <div class="section">
        <h3>系统状态</h3>
        <div class="system-status" *ngIf="systemStatus">
          <p>活动任务: {{ systemStatus.active_test_tasks }}</p>
          <p>系统健康: {{ systemStatus.system_health }}</p>
          <p>版本: {{ systemStatus.version }}</p>
        </div>
      </div>

      <!-- 日志区域 -->
      <div class="section">
        <h3>操作日志</h3>
        <div class="log-container">
          <div *ngFor="let log of logs" class="log-item" [class]="log.level">
            <span class="timestamp">{{ formatTimestamp(log.timestamp) }}</span>
            <span class="message">{{ log.message }}</span>
          </div>
        </div>
      </div>
    </div>
  `,
  styles: [`
    .test-execution-container {
      padding: 20px;
      max-width: 1200px;
      margin: 0 auto;
    }

    .section {
      margin-bottom: 30px;
      padding: 20px;
      border: 1px solid #ddd;
      border-radius: 8px;
      background: #f9f9f9;
    }

    .form-group {
      margin-bottom: 15px;
    }

    .form-group label {
      display: inline-block;
      width: 100px;
      font-weight: bold;
    }

    .form-group input {
      padding: 8px;
      border: 1px solid #ccc;
      border-radius: 4px;
      width: 200px;
    }

    .control-buttons {
      display: flex;
      gap: 10px;
      margin-bottom: 15px;
    }

    .control-buttons button {
      padding: 10px 15px;
      border: none;
      border-radius: 4px;
      cursor: pointer;
      font-weight: bold;
    }

    .control-buttons button:disabled {
      opacity: 0.5;
      cursor: not-allowed;
    }

    .progress-item {
      margin-bottom: 15px;
      padding: 10px;
      border: 1px solid #ccc;
      border-radius: 4px;
      background: white;
    }

    .progress-header {
      display: flex;
      justify-content: space-between;
      margin-bottom: 8px;
    }

    .progress-bar {
      width: 100%;
      height: 20px;
      background: #eee;
      border-radius: 10px;
      overflow: hidden;
      margin-bottom: 8px;
    }

    .progress-fill {
      height: 100%;
      background: #4caf50;
      transition: width 0.3s ease;
    }

    .progress-details {
      display: flex;
      justify-content: space-between;
      font-size: 12px;
      color: #666;
    }

    .status.not-tested { color: #999; }
    .status.hard-point-testing { color: #ff9800; }
    .status.test-completed-passed { color: #4caf50; }
    .status.test-completed-failed { color: #f44336; }

    .result-item {
      margin-bottom: 10px;
      padding: 10px;
      border: 1px solid #ccc;
      border-radius: 4px;
      background: white;
    }

    .result-header {
      display: flex;
      justify-content: space-between;
      margin-bottom: 8px;
    }

    .success { color: #4caf50; font-weight: bold; }
    .failure { color: #f44336; font-weight: bold; }

    .log-container {
      max-height: 300px;
      overflow-y: auto;
      border: 1px solid #ccc;
      padding: 10px;
      background: white;
    }

    .log-item {
      margin-bottom: 5px;
      font-family: monospace;
      font-size: 12px;
    }

    .log-item.info { color: #2196f3; }
    .log-item.success { color: #4caf50; }
    .log-item.error { color: #f44336; }
    .log-item.warning { color: #ff9800; }
  `]
})
export class TestExecutionComponent implements OnInit, OnDestroy {
  // 状态变量
  isLoading = false;
  currentBatchId: string | null = null;
  currentBatchStatus = 'unknown';
  currentInstanceCount = 0;
  autoStart = false;

  // 数据
  newBatch: Partial<TestBatchInfo> = {
    product_model: 'TestProduct_V1.0',
    serial_number: 'SN' + Date.now(),
    operator_name: '测试操作员'
  };

  progressUpdates: TestProgressUpdate[] = [];
  testResults: RawTestOutcome[] = [];
  systemStatus: any = null;
  logs: Array<{timestamp: string, level: string, message: string}> = [];

  // 订阅
  private subscriptions: Subscription[] = [];
  private progressPolling: Subscription | null = null;

  constructor(private tauriApi: TauriApiService) {}

  ngOnInit() {
    this.addLog('info', '测试执行组件已初始化');
    
    // 订阅系统状态
    this.subscriptions.push(
      this.tauriApi.systemStatus$.subscribe(status => {
        this.systemStatus = status;
      })
    );
  }

  ngOnDestroy() {
    this.subscriptions.forEach(sub => sub.unsubscribe());
    if (this.progressPolling) {
      this.progressPolling.unsubscribe();
    }
  }

  // 创建并提交测试批次
  createAndSubmitBatch() {
    this.isLoading = true;
    this.addLog('info', '开始创建测试批次...');

    // 创建示例通道定义
    const sampleDefinitions: ChannelPointDefinition[] = [
      {
        id: 'def_001',
        tag: 'AI001',
        variable_name: 'Temperature_1',
        description: '温度传感器1',
        station_name: 'Station1',
        module_name: 'Module1',
        module_type: ModuleType.AI,
        channel_number: 'CH01',
        point_data_type: PointDataType.Float,
        plc_communication_address: 'DB1.DBD0',
        analog_range_min: 0,
        analog_range_max: 100,
        created_at: new Date().toISOString(),
        updated_at: new Date().toISOString()
      },
      {
        id: 'def_002',
        tag: 'DI001',
        variable_name: 'Switch_1',
        description: '开关状态1',
        station_name: 'Station1',
        module_name: 'Module2',
        module_type: ModuleType.DI,
        channel_number: 'CH01',
        point_data_type: PointDataType.Bool,
        plc_communication_address: 'I0.0',
        created_at: new Date().toISOString(),
        updated_at: new Date().toISOString()
      }
    ];

    // 创建批次信息
    const batchInfo: TestBatchInfo = {
      batch_id: 'batch_' + Date.now(),
      product_model: this.newBatch.product_model,
      serial_number: this.newBatch.serial_number,
      operator_name: this.newBatch.operator_name,
      total_points: sampleDefinitions.length,
      passed_points: 0,
      failed_points: 0,
      overall_status: OverallTestStatus.NotTested,
      created_at: new Date().toISOString(),
      updated_at: new Date().toISOString()
    };

    // 创建测试执行请求
    const request: TestExecutionRequest = {
      batch_info: batchInfo,
      channel_definitions: sampleDefinitions,
      max_concurrent_tests: 3,
      auto_start: this.autoStart
    };

    // 提交请求
    this.tauriApi.submitTestExecution(request).subscribe({
      next: (response: TestExecutionResponse) => {
        this.isLoading = false;
        this.currentBatchId = response.batch_id;
        this.currentBatchStatus = response.status;
        this.currentInstanceCount = response.instance_count;
        
        this.addLog('success', `批次创建成功: ${response.message}`);
        
        if (this.autoStart) {
          this.startProgressPolling();
        }
      },
      error: (error) => {
        this.isLoading = false;
        this.addLog('error', `批次创建失败: ${error}`);
      }
    });
  }

  // 开始批次测试
  startBatch() {
    if (!this.currentBatchId) return;

    this.addLog('info', '开始批次测试...');
    this.tauriApi.startBatchTesting(this.currentBatchId).subscribe({
      next: () => {
        this.currentBatchStatus = 'running';
        this.addLog('success', '批次测试已开始');
        this.startProgressPolling();
      },
      error: (error) => {
        this.addLog('error', `开始测试失败: ${error}`);
      }
    });
  }

  // 暂停批次测试
  pauseBatch() {
    if (!this.currentBatchId) return;

    this.addLog('info', '暂停批次测试...');
    this.tauriApi.pauseBatchTesting(this.currentBatchId).subscribe({
      next: () => {
        this.currentBatchStatus = 'paused';
        this.addLog('warning', '批次测试已暂停');
        this.stopProgressPolling();
      },
      error: (error) => {
        this.addLog('error', `暂停测试失败: ${error}`);
      }
    });
  }

  // 恢复批次测试
  resumeBatch() {
    if (!this.currentBatchId) return;

    this.addLog('info', '恢复批次测试...');
    this.tauriApi.resumeBatchTesting(this.currentBatchId).subscribe({
      next: () => {
        this.currentBatchStatus = 'running';
        this.addLog('success', '批次测试已恢复');
        this.startProgressPolling();
      },
      error: (error) => {
        this.addLog('error', `恢复测试失败: ${error}`);
      }
    });
  }

  // 停止批次测试
  stopBatch() {
    if (!this.currentBatchId) return;

    this.addLog('info', '停止批次测试...');
    this.tauriApi.stopBatchTesting(this.currentBatchId).subscribe({
      next: () => {
        this.currentBatchStatus = 'stopped';
        this.addLog('warning', '批次测试已停止');
        this.stopProgressPolling();
      },
      error: (error) => {
        this.addLog('error', `停止测试失败: ${error}`);
      }
    });
  }

  // 清理批次
  cleanupBatch() {
    if (!this.currentBatchId) return;

    this.addLog('info', '清理批次...');
    this.tauriApi.cleanupCompletedBatch(this.currentBatchId).subscribe({
      next: () => {
        this.addLog('success', '批次已清理');
        this.resetState();
      },
      error: (error) => {
        this.addLog('error', `清理批次失败: ${error}`);
      }
    });
  }

  // 开始进度轮询
  private startProgressPolling() {
    if (this.progressPolling) {
      this.progressPolling.unsubscribe();
    }

    this.progressPolling = interval(2000).subscribe(() => {
      if (this.currentBatchId) {
        this.updateProgress();
        this.updateResults();
      }
    });
  }

  // 停止进度轮询
  private stopProgressPolling() {
    if (this.progressPolling) {
      this.progressPolling.unsubscribe();
      this.progressPolling = null;
    }
  }

  // 更新进度
  private updateProgress() {
    if (!this.currentBatchId) return;

    this.tauriApi.getBatchProgress(this.currentBatchId).subscribe({
      next: (progress) => {
        this.progressUpdates = progress;
      },
      error: (error) => {
        console.error('获取进度失败:', error);
      }
    });
  }

  // 更新结果
  private updateResults() {
    if (!this.currentBatchId) return;

    this.tauriApi.getBatchResults(this.currentBatchId).subscribe({
      next: (results) => {
        this.testResults = results;
      },
      error: (error) => {
        console.error('获取结果失败:', error);
      }
    });
  }

  // 重置状态
  private resetState() {
    this.currentBatchId = null;
    this.currentBatchStatus = 'unknown';
    this.currentInstanceCount = 0;
    this.progressUpdates = [];
    this.testResults = [];
    this.stopProgressPolling();
  }

  // 添加日志
  private addLog(level: string, message: string) {
    this.logs.unshift({
      timestamp: new Date().toISOString(),
      level,
      message
    });

    // 限制日志数量
    if (this.logs.length > 100) {
      this.logs = this.logs.slice(0, 100);
    }
  }

  // 辅助方法
  canStart(): boolean {
    return this.currentBatchId !== null && 
           (this.currentBatchStatus === 'submitted' || this.currentBatchStatus === 'paused');
  }

  canPause(): boolean {
    return this.currentBatchId !== null && this.currentBatchStatus === 'running';
  }

  canResume(): boolean {
    return this.currentBatchId !== null && this.currentBatchStatus === 'paused';
  }

  canStop(): boolean {
    return this.currentBatchId !== null && 
           (this.currentBatchStatus === 'running' || this.currentBatchStatus === 'paused');
  }

  canCleanup(): boolean {
    return this.currentBatchId !== null && 
           (this.currentBatchStatus === 'stopped' || this.currentBatchStatus === 'completed');
  }

  getOverallProgress(): number {
    if (this.progressUpdates.length === 0) return 0;
    
    const totalProgress = this.progressUpdates.reduce((sum, progress) => {
      return sum + this.getProgressPercentage(progress);
    }, 0);
    
    return Math.round(totalProgress / this.progressUpdates.length);
  }

  getCompletedInstances(): number {
    return this.progressUpdates.filter(p => 
      p.completed_sub_tests >= p.total_sub_tests
    ).length;
  }

  getProgressPercentage(progress: TestProgressUpdate): number {
    if (progress.total_sub_tests === 0) return 0;
    return Math.round((progress.completed_sub_tests / progress.total_sub_tests) * 100);
  }

  getStatusClass(status: OverallTestStatus): string {
    return status.toString().toLowerCase().replace(/_/g, '-');
  }

  getStatusLabel(status: OverallTestStatus): string {
    const labels: {[key: string]: string} = {
      'NotTested': '未测试',
      'HardPointTesting': '硬点测试中',
      'TestCompletedPassed': '测试通过',
      'TestCompletedFailed': '测试失败'
    };
    return labels[status] || status;
  }

  getSubTestItemLabel(item: string): string {
    const labels: {[key: string]: string} = {
      'HardPoint': '硬点测试',
      'LowLowAlarm': '低低报警',
      'LowAlarm': '低报警',
      'HighAlarm': '高报警',
      'HighHighAlarm': '高高报警',
      'StateDisplay': '状态显示'
    };
    return labels[item] || item;
  }

  getSuccessCount(): number {
    return this.testResults.filter(r => r.success).length;
  }

  getFailureCount(): number {
    return this.testResults.filter(r => !r.success).length;
  }

  formatTimestamp(timestamp: string): string {
    return new Date(timestamp).toLocaleString('zh-CN');
  }
}
