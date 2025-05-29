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
        <div class="button-group">
          <button (click)="createTestData()" [disabled]="isLoading">
            {{ isLoading ? '创建中...' : '创建测试数据' }}
          </button>
          <button (click)="createAndSubmitBatch()" [disabled]="isLoading">
            {{ isLoading ? '创建中...' : '创建并提交测试' }}
          </button>
        </div>
      </div>

      <!-- 所有批次显示区域 -->
      <div class="section" *ngIf="sessionBatches.length > 0">
        <h3>当前会话批次列表 ({{ sessionBatches.length }} 个批次)</h3>
        <div class="batch-list">
          <div *ngFor="let batch of sessionBatches; trackBy: trackByBatchId" 
               class="batch-item" 
               [class.selected]="batch.batch_id === currentBatchId"
               (click)="selectBatch(batch.batch_id)">
            <div class="batch-header">
              <h4>{{ batch.batch_name || ('批次 ' + extractBatchNumber(batch.batch_id)) }}</h4>
              <span class="batch-status" [class]="getBatchStatusClass(batch.overall_status)">
                {{ getStatusLabel(batch.overall_status) }}
              </span>
            </div>
            <div class="batch-details">
              <p><strong>批次ID:</strong> {{ batch.batch_id }}</p>
              <p><strong>产品型号:</strong> {{ batch.product_model || '未指定' }}</p>
              <p><strong>序列号:</strong> {{ batch.serial_number || '未指定' }}</p>
              <p><strong>总点位:</strong> {{ batch.total_points }}</p>
              <p><strong>通过/失败:</strong> {{ batch.passed_points }}/{{ batch.failed_points }}</p>
              <p><strong>创建时间:</strong> {{ formatTimestamp(batch.creation_time?.toString() || '') }}</p>
            </div>
            <div class="batch-controls" *ngIf="batch.batch_id === currentBatchId">
              <button (click)="startBatch($event)" [disabled]="!canStart()">开始</button>
              <button (click)="pauseBatch($event)" [disabled]="!canPause()">暂停</button>
              <button (click)="resumeBatch($event)" [disabled]="!canResume()">恢复</button>
              <button (click)="stopBatch($event)" [disabled]="!canStop()">停止</button>
              <button (click)="cleanupBatch($event)" [disabled]="!canCleanup()">清理</button>
            </div>
          </div>
        </div>
        <div class="batch-summary" *ngIf="sessionBatches.length > 1">
          <h4>批次汇总统计</h4>
          <p><strong>总批次数:</strong> {{ sessionBatches.length }}</p>
          <p><strong>总点位数:</strong> {{ getTotalPoints() }}</p>
          <p><strong>已完成批次:</strong> {{ getCompletedBatches() }}</p>
          <p><strong>整体通过率:</strong> {{ getOverallPassRate() }}%</p>
        </div>
      </div>

      <!-- 当前批次控制区域 -->
      <div class="section" *ngIf="currentBatchId">
        <h3>当前选中批次详情 - {{ getCurrentBatchName() }}</h3>
        <div class="selected-batch-info">
          <p><strong>批次ID:</strong> {{ currentBatchId }}</p>
          <p><strong>状态:</strong> {{ currentBatchStatus }}</p>
          <p><strong>实例数量:</strong> {{ currentInstanceCount }}</p>
          <button (click)="refreshCurrentBatchData()" [disabled]="isLoading">
            {{ isLoading ? '刷新中...' : '刷新数据' }}
          </button>
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

    /* 批次列表样式 */
    .batch-list {
      display: grid;
      gap: 15px;
    }

    .batch-item {
      border: 2px solid #ddd;
      border-radius: 8px;
      padding: 15px;
      background: white;
      cursor: pointer;
      transition: all 0.3s ease;
    }

    .batch-item:hover {
      border-color: #2196f3;
      box-shadow: 0 2px 8px rgba(33, 150, 243, 0.2);
    }

    .batch-item.selected {
      border-color: #4caf50;
      background: #f8fff8;
      box-shadow: 0 2px 8px rgba(76, 175, 80, 0.3);
    }

    .batch-header {
      display: flex;
      justify-content: space-between;
      align-items: center;
      margin-bottom: 10px;
      border-bottom: 1px solid #eee;
      padding-bottom: 8px;
    }

    .batch-header h4 {
      margin: 0;
      color: #333;
      font-size: 16px;
    }

    .batch-status {
      padding: 4px 8px;
      border-radius: 4px;
      font-size: 12px;
      font-weight: bold;
      text-transform: uppercase;
    }

    .batch-status.not-tested { 
      background: #f5f5f5; 
      color: #666; 
    }

    .batch-status.hard-point-testing { 
      background: #fff3e0; 
      color: #f57c00; 
    }

    .batch-status.test-completed-passed { 
      background: #e8f5e8; 
      color: #2e7d32; 
    }

    .batch-status.test-completed-failed { 
      background: #ffebee; 
      color: #c62828; 
    }

    .batch-details {
      display: grid;
      grid-template-columns: 1fr 1fr;
      gap: 8px;
      margin-bottom: 10px;
    }

    .batch-details p {
      margin: 0;
      font-size: 14px;
      color: #555;
    }

    .batch-controls {
      display: flex;
      gap: 8px;
      padding-top: 10px;
      border-top: 1px solid #eee;
    }

    .batch-controls button {
      padding: 6px 12px;
      font-size: 12px;
      border: none;
      border-radius: 4px;
      cursor: pointer;
      font-weight: bold;
    }

    .batch-summary {
      background: #f0f8ff;
      border: 1px solid #b3d9ff;
      border-radius: 8px;
      padding: 15px;
      margin-top: 20px;
    }

    .batch-summary h4 {
      margin: 0 0 10px 0;
      color: #1565c0;
    }

    .selected-batch-info {
      background: #f8f9fa;
      border: 1px solid #dee2e6;
      border-radius: 6px;
      padding: 15px;
    }

    .selected-batch-info p {
      margin: 5px 0;
    }

    .selected-batch-info button {
      margin-top: 10px;
      padding: 8px 16px;
      background: #007bff;
      color: white;
      border: none;
      border-radius: 4px;
      cursor: pointer;
    }

    .selected-batch-info button:hover {
      background: #0056b3;
    }

    .selected-batch-info button:disabled {
      background: #6c757d;
      cursor: not-allowed;
    }

    .button-group {
      display: flex;
      gap: 10px;
      align-items: center;
    }

    .button-group button {
      padding: 10px 15px;
      border: none;
      border-radius: 4px;
      cursor: pointer;
      font-weight: bold;
    }

    .button-group button:disabled {
      opacity: 0.5;
      cursor: not-allowed;
    }
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
  sessionBatches: TestBatchInfo[] = [];

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

    // 加载当前会话的所有批次
    this.loadSessionBatches();
  }

  ngOnDestroy() {
    this.subscriptions.forEach(sub => sub.unsubscribe());
    if (this.progressPolling) {
      this.progressPolling.unsubscribe();
    }
  }

  // 加载当前会话的所有批次
  loadSessionBatches() {
    this.tauriApi.getSessionBatches().subscribe({
      next: (batches) => {
        this.sessionBatches = batches;
        this.addLog('info', `加载了 ${batches.length} 个会话批次`);
      },
      error: (error) => {
        this.addLog('error', `加载会话批次失败: ${error}`);
      }
    });
  }

  // 创建测试数据
  createTestData() {
    this.isLoading = true;
    this.addLog('info', '开始创建测试数据...');

    this.tauriApi.createTestData().subscribe({
      next: (definitions) => {
        this.isLoading = false;
        this.addLog('success', `成功创建了 ${definitions.length} 个测试通道定义`);
        
        // 显示创建的通道详情
        const counts = definitions.reduce((acc, def) => {
          const key = def.module_type;
          acc[key] = (acc[key] || 0) + 1;
          return acc;
        }, {} as Record<string, number>);
        
        Object.entries(counts).forEach(([type, count]) => {
          this.addLog('info', `  ${type}: ${count} 个通道`);
        });
      },
      error: (error) => {
        this.isLoading = false;
        this.addLog('error', `创建测试数据失败: ${error}`);
      }
    });
  }

  // 创建并提交测试批次
  createAndSubmitBatch() {
    this.isLoading = true;
    this.addLog('info', '开始创建测试批次...');

    // 首先获取所有通道定义
    this.tauriApi.getAllChannelDefinitions().subscribe({
      next: (definitions) => {
        if (definitions.length === 0) {
          this.isLoading = false;
          this.addLog('error', '没有找到通道定义，请先导入Excel文件');
          return;
        }

        this.addLog('info', `找到 ${definitions.length} 个通道定义`);

        // 创建批次信息
        const batchInfo: TestBatchInfo = {
          batch_id: 'batch_' + Date.now(),
          product_model: this.newBatch.product_model,
          serial_number: this.newBatch.serial_number,
          operator_name: this.newBatch.operator_name,
          total_points: definitions.length,
          passed_points: 0,
          failed_points: 0,
          overall_status: OverallTestStatus.NotTested,
          creation_time: new Date().toISOString(),
          last_updated_time: new Date().toISOString(),
          created_at: new Date().toISOString(),
          updated_at: new Date().toISOString()
        };

        // 创建测试执行请求
        const request: TestExecutionRequest = {
          batch_info: batchInfo,
          channel_definitions: definitions,
          max_concurrent_tests: 3,
          auto_start: this.autoStart
        };

        this.addLog('info', `创建测试批次，包含 ${definitions.length} 个通道定义`);

        // 提交请求
        this.tauriApi.submitTestExecution(request).subscribe({
          next: (response: TestExecutionResponse) => {
            this.isLoading = false;
            
            // 记录所有生成的批次信息
            this.addLog('success', `批次创建成功: ${response.message}`);
            this.addLog('info', `生成了 ${response.all_batches.length} 个批次`);
            
            // 显示所有批次的详细信息
            response.all_batches.forEach((batch, index) => {
              this.addLog('info', `批次${index + 1}: ${batch.batch_name} (${batch.total_points}个点位)`);
            });
            
            // 选择第一个批次作为当前批次
            this.currentBatchId = response.batch_id;
            this.currentBatchStatus = response.status;
            this.currentInstanceCount = response.instance_count;
            
            // 重新加载会话批次列表
            this.loadSessionBatches();
            
            if (this.autoStart) {
              this.startProgressPolling();
            }
          },
          error: (error) => {
            this.isLoading = false;
            this.addLog('error', `批次创建失败: ${error}`);
          }
        });
      },
      error: (error) => {
        this.isLoading = false;
        this.addLog('error', `获取通道定义失败: ${error}`);
      }
    });
  }

  // 开始批次测试
  startBatch(event?: Event) {
    if (event) event.stopPropagation();
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
  pauseBatch(event?: Event) {
    if (event) event.stopPropagation();
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
  resumeBatch(event?: Event) {
    if (event) event.stopPropagation();
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
  stopBatch(event?: Event) {
    if (event) event.stopPropagation();
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
  cleanupBatch(event?: Event) {
    if (event) event.stopPropagation();
    if (!this.currentBatchId) return;

    this.addLog('info', '清理批次...');
    this.tauriApi.cleanupCompletedBatch(this.currentBatchId).subscribe({
      next: () => {
        this.addLog('success', '批次已清理');
        this.resetState();
        this.loadSessionBatches(); // 重新加载批次列表
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

  // 新添加的方法
  getCurrentBatchName(): string {
    return this.currentBatchId ? ('批次 ' + this.extractBatchNumber(this.currentBatchId)) : '未命名批次';
  }

  extractBatchNumber(batchId: string): string {
    const parts = batchId.split('_');
    return parts.length > 1 ? parts[parts.length - 1] : '1';
  }

  getBatchStatusClass(status: OverallTestStatus): string {
    return status.toString().toLowerCase().replace(/_/g, '-');
  }

  getTotalPoints(): number {
    return this.sessionBatches.reduce((total, batch) => total + batch.total_points, 0);
  }

  getCompletedBatches(): number {
    return this.sessionBatches.filter(batch => batch.overall_status === OverallTestStatus.TestCompletedPassed).length;
  }

  getOverallPassRate(): number {
    const totalCompleted = this.getCompletedBatches();
    const totalBatches = this.sessionBatches.length;
    if (totalBatches === 0) return 0;
    return Math.round((totalCompleted / totalBatches) * 100);
  }

  selectBatch(batchId: string) {
    this.currentBatchId = batchId;
    this.refreshCurrentBatchData();
  }

  refreshCurrentBatchData() {
    if (!this.currentBatchId) return;
    
    this.isLoading = true;
    this.addLog('info', '刷新当前批次数据...');

    // 先获取进度更新
    this.tauriApi.getBatchProgress(this.currentBatchId).subscribe({
      next: (progress) => {
        this.progressUpdates = progress;
        this.addLog('success', '批次进度数据刷新成功');
      },
      error: (error) => {
        this.addLog('error', `获取批次进度失败: ${error}`);
      }
    });

    // 再获取测试结果
    this.tauriApi.getBatchResults(this.currentBatchId).subscribe({
      next: (results) => {
        this.testResults = results;
        this.isLoading = false;
        this.addLog('success', '批次结果数据刷新成功');
      },
      error: (error) => {
        this.isLoading = false;
        this.addLog('error', `获取批次结果失败: ${error}`);
      }
    });
  }

  trackByBatchId(index: number, batch: TestBatchInfo) {
    return batch.batch_id;
  }
}
