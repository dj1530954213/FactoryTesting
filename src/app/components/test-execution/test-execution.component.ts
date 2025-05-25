import { Component, OnInit, OnDestroy } from '@angular/core';
import { CommonModule } from '@angular/common';
import { FormsModule } from '@angular/forms';
import { Subscription, interval } from 'rxjs';
import { TauriApiService } from '../../services/tauri-api.service';
import { 
  TestBatchInfo, 
  TestProgressUpdate,
  RawTestOutcome,
  OverallTestStatus,
  OVERALL_TEST_STATUS_LABELS,
  SUB_TEST_STATUS_LABELS,
  ApiError
} from '../../models';

@Component({
  selector: 'app-test-execution',
  standalone: true,
  imports: [CommonModule, FormsModule],
  template: `
    <div class="test-execution-container">
      <!-- 页面标题 -->
      <div class="page-header">
        <div class="title-section">
          <h1 class="page-title">测试执行</h1>
          <p class="page-subtitle">监控和管理测试执行过程</p>
        </div>
        <div class="header-actions">
          <button class="btn-secondary" (click)="refreshData()">
            <i class="icon-refresh">🔄</i>
            刷新
          </button>
        </div>
      </div>

      <!-- 执行状态概览 -->
      <div class="status-overview">
        <div class="overview-card">
          <div class="card-icon">⚡</div>
          <div class="card-content">
            <h3>活动任务</h3>
            <p class="status-value">{{activeTaskCount}}</p>
            <small>正在执行的测试</small>
          </div>
        </div>
        
        <div class="overview-card">
          <div class="card-icon">📊</div>
          <div class="card-content">
            <h3>总进度</h3>
            <p class="status-value">{{getTotalProgress()}}%</p>
            <small>所有批次平均进度</small>
          </div>
        </div>
        
        <div class="overview-card">
          <div class="card-icon">✅</div>
          <div class="card-content">
            <h3>通过率</h3>
            <p class="status-value success">{{getTotalPassRate()}}%</p>
            <small>总体测试通过率</small>
          </div>
        </div>
        
        <div class="overview-card">
          <div class="card-icon">⏱️</div>
          <div class="card-content">
            <h3>运行时间</h3>
            <p class="status-value">{{getTotalRuntime()}}</p>
            <small>累计执行时间</small>
          </div>
        </div>
      </div>

      <!-- 正在执行的批次 -->
      <div class="active-batches-section" *ngIf="activeBatches.length > 0">
        <div class="section-header">
          <h2>正在执行的批次 ({{activeBatches.length}})</h2>
          <div class="auto-refresh-toggle">
            <label class="toggle-label">
              <input 
                type="checkbox" 
                [checked]="autoRefresh"
                (change)="onAutoRefreshChange($event)">
              自动刷新 ({{refreshInterval/1000}}s)
            </label>
          </div>
        </div>

        <div class="active-batches-grid">
          <div *ngFor="let batch of activeBatches" class="batch-execution-card">
            <div class="card-header">
              <div class="batch-info">
                <h3 class="batch-title">{{batch.product_model || '未知产品'}}</h3>
                <span class="batch-id">{{batch.batch_id}}</span>
              </div>
              <div class="execution-status">
                <span class="status-indicator running"></span>
                <span class="status-text">执行中</span>
              </div>
            </div>

            <div class="card-content">
              <!-- 整体进度 -->
              <div class="progress-section">
                <div class="progress-header">
                  <span class="progress-label">整体进度</span>
                  <span class="progress-text">{{getBatchProgress(batch)}}%</span>
                </div>
                <div class="progress-bar">
                  <div class="progress-fill" [style.width.%]="getBatchProgress(batch)"></div>
                </div>
                <div class="progress-details">
                  <span>{{batch.tested_points}}/{{batch.total_points}} 通道</span>
                  <span>通过: {{batch.passed_points}} | 失败: {{batch.failed_points}}</span>
                </div>
              </div>

              <!-- 最新进度更新 -->
              <div class="latest-updates" *ngIf="getLatestUpdates(batch.batch_id).length > 0">
                <h4>最新进度</h4>
                <div class="updates-list">
                  <div *ngFor="let update of getLatestUpdates(batch.batch_id).slice(0, 3)" 
                       class="update-item">
                    <div class="update-info">
                      <span class="point-tag">{{update.point_tag}}</span>
                      <span class="update-status" [class]="getStatusClass(update.overall_status)">
                        {{getStatusLabel(update.overall_status)}}
                      </span>
                    </div>
                    <div class="update-progress">
                      {{update.completed_sub_tests}}/{{update.total_sub_tests}} 子测试
                    </div>
                    <div class="update-time">
                      {{formatTime(update.timestamp)}}
                    </div>
                  </div>
                </div>
              </div>
            </div>

            <div class="card-actions">
              <button class="btn-small" (click)="viewBatchDetails(batch)">查看详情</button>
              <button class="btn-small secondary" (click)="pauseBatch(batch)">暂停</button>
              <button class="btn-small danger" (click)="stopBatch(batch)">停止</button>
            </div>
          </div>
        </div>
      </div>

      <!-- 最近完成的批次 -->
      <div class="completed-batches-section">
        <div class="section-header">
          <h2>最近完成的批次</h2>
          <button class="btn-link" (click)="viewAllCompleted()">查看全部</button>
        </div>

        <div *ngIf="completedBatches.length > 0; else noCompleted" class="completed-batches-list">
          <div *ngFor="let batch of completedBatches.slice(0, 5)" class="completed-batch-item">
            <div class="batch-summary">
              <div class="batch-info">
                <h4>{{batch.product_model || '未知产品'}}</h4>
                <span class="batch-id">{{batch.batch_id}}</span>
              </div>
              <div class="batch-results">
                <div class="result-item">
                  <span class="result-label">总点数:</span>
                  <span class="result-value">{{batch.total_points}}</span>
                </div>
                <div class="result-item">
                  <span class="result-label">通过:</span>
                  <span class="result-value success">{{batch.passed_points}}</span>
                </div>
                <div class="result-item">
                  <span class="result-label">失败:</span>
                  <span class="result-value danger">{{batch.failed_points}}</span>
                </div>
                <div class="result-item">
                  <span class="result-label">通过率:</span>
                  <span class="result-value" [class.success]="getBatchPassRate(batch) >= 90" 
                        [class.warning]="getBatchPassRate(batch) >= 70 && getBatchPassRate(batch) < 90"
                        [class.danger]="getBatchPassRate(batch) < 70">
                    {{getBatchPassRate(batch)}}%
                  </span>
                </div>
              </div>
              <div class="batch-timing">
                <div class="timing-item">
                  <span class="timing-label">完成时间:</span>
                  <span class="timing-value">{{formatDate(batch.test_end_time!)}}</span>
                </div>
                <div class="timing-item" *ngIf="batch.total_test_duration_ms">
                  <span class="timing-label">耗时:</span>
                  <span class="timing-value">{{formatDuration(batch.total_test_duration_ms)}}</span>
                </div>
              </div>
            </div>
            <div class="batch-actions">
              <button class="btn-small" (click)="viewBatchResults(batch)">查看结果</button>
              <button class="btn-small secondary" (click)="exportResults(batch)">导出</button>
            </div>
          </div>
        </div>

        <ng-template #noCompleted>
          <div class="empty-state">
            <div class="empty-icon">📋</div>
            <h3>暂无完成的测试</h3>
            <p>完成的测试批次将在这里显示</p>
          </div>
        </ng-template>
      </div>

      <!-- 系统状态 -->
      <div class="system-status-section">
        <div class="section-header">
          <h2>系统状态</h2>
          <div class="status-indicator" [class.healthy]="systemHealthy" [class.unhealthy]="!systemHealthy">
            {{systemHealthy ? '正常' : '异常'}}
          </div>
        </div>
        
        <div class="status-grid">
          <div class="status-item">
            <span class="status-label">系统版本:</span>
            <span class="status-value">{{systemVersion}}</span>
          </div>
          <div class="status-item">
            <span class="status-label">活动任务:</span>
            <span class="status-value">{{activeTaskCount}}</span>
          </div>
          <div class="status-item">
            <span class="status-label">最后更新:</span>
            <span class="status-value">{{formatTime(lastUpdateTime)}}</span>
          </div>
        </div>
      </div>

      <!-- 加载状态 -->
      <div *ngIf="loading" class="loading-overlay">
        <div class="loading-spinner">
          <div class="spinner"></div>
          <p>加载中...</p>
        </div>
      </div>
    </div>
  `,
  styleUrls: ['./test-execution.component.css']
})
export class TestExecutionComponent implements OnInit, OnDestroy {
  // 数据属性
  activeBatches: TestBatchInfo[] = [];
  completedBatches: TestBatchInfo[] = [];
  progressUpdates: TestProgressUpdate[] = [];
  
  // 系统状态
  activeTaskCount = 0;
  systemHealthy = false;
  systemVersion = 'Unknown';
  lastUpdateTime = new Date().toISOString();
  
  // 界面状态
  loading = false;
  autoRefresh = true;
  refreshInterval = 5000; // 5秒
  
  private subscriptions: Subscription[] = [];
  private refreshTimer?: Subscription;

  constructor(private tauriApi: TauriApiService) {}

  ngOnInit(): void {
    this.loadData();
    this.subscribeToSystemStatus();
    this.startAutoRefresh();
  }

  ngOnDestroy(): void {
    this.subscriptions.forEach(sub => sub.unsubscribe());
    this.stopAutoRefresh();
  }

  /**
   * 处理自动刷新切换
   */
  onAutoRefreshChange(event: Event): void {
    const target = event.target as HTMLInputElement;
    this.autoRefresh = target.checked;
    this.toggleAutoRefresh();
  }

  /**
   * 加载数据
   */
  loadData(): void {
    this.loading = true;
    
    // 加载批次信息
    const batchSub = this.tauriApi.getAllBatchInfo().subscribe({
      next: (batches) => {
        this.activeBatches = batches.filter(b => this.isBatchActive(b));
        this.completedBatches = batches
          .filter(b => this.isBatchCompleted(b))
          .sort((a, b) => new Date(b.test_end_time!).getTime() - new Date(a.test_end_time!).getTime());
        
        this.loading = false;
        this.lastUpdateTime = new Date().toISOString();
      },
      error: (error: ApiError) => {
        console.error('加载批次数据失败:', error);
        this.loading = false;
      }
    });

    this.subscriptions.push(batchSub);
  }

  /**
   * 订阅系统状态
   */
  subscribeToSystemStatus(): void {
    const statusSub = this.tauriApi.systemStatus$.subscribe({
      next: (status) => {
        if (status) {
          this.activeTaskCount = status.active_test_tasks;
          this.systemHealthy = status.system_health === 'healthy';
          this.systemVersion = status.version;
        }
      },
      error: (error) => {
        console.error('系统状态订阅失败:', error);
        this.systemHealthy = false;
      }
    });

    this.subscriptions.push(statusSub);
  }

  /**
   * 开始自动刷新
   */
  startAutoRefresh(): void {
    if (this.autoRefresh) {
      this.refreshTimer = interval(this.refreshInterval).subscribe(() => {
        this.loadData();
      });
    }
  }

  /**
   * 停止自动刷新
   */
  stopAutoRefresh(): void {
    if (this.refreshTimer) {
      this.refreshTimer.unsubscribe();
      this.refreshTimer = undefined;
    }
  }

  /**
   * 切换自动刷新
   */
  toggleAutoRefresh(): void {
    this.stopAutoRefresh();
    if (this.autoRefresh) {
      this.startAutoRefresh();
    }
  }

  /**
   * 刷新数据
   */
  refreshData(): void {
    this.loadData();
  }

  /**
   * 批次操作
   */
  pauseBatch(batch: TestBatchInfo): void {
    const sub = this.tauriApi.pauseBatchTesting(batch.batch_id).subscribe({
      next: () => {
        console.log('批次已暂停:', batch.batch_id);
        this.loadData();
      },
      error: (error: ApiError) => {
        console.error('暂停批次失败:', error);
      }
    });
    this.subscriptions.push(sub);
  }

  stopBatch(batch: TestBatchInfo): void {
    if (!confirm(`确定要停止批次 "${batch.batch_id}" 的测试吗？`)) return;

    const sub = this.tauriApi.stopBatchTesting(batch.batch_id).subscribe({
      next: () => {
        console.log('批次已停止:', batch.batch_id);
        this.loadData();
      },
      error: (error: ApiError) => {
        console.error('停止批次失败:', error);
      }
    });
    this.subscriptions.push(sub);
  }

  viewBatchDetails(batch: TestBatchInfo): void {
    // TODO: 跳转到批次详情页面
    console.log('查看批次详情:', batch.batch_id);
  }

  viewBatchResults(batch: TestBatchInfo): void {
    // TODO: 跳转到批次结果页面
    console.log('查看批次结果:', batch.batch_id);
  }

  exportResults(batch: TestBatchInfo): void {
    // TODO: 实现结果导出功能
    console.log('导出批次结果:', batch.batch_id);
  }

  viewAllCompleted(): void {
    // TODO: 跳转到完成批次列表页面
    console.log('查看所有完成的批次');
  }

  /**
   * 工具方法
   */
  isBatchActive(batch: TestBatchInfo): boolean {
    return batch.test_start_time && !batch.test_end_time;
  }

  isBatchCompleted(batch: TestBatchInfo): boolean {
    return !!batch.test_end_time;
  }

  getBatchProgress(batch: TestBatchInfo): number {
    if (batch.total_points === 0) return 0;
    return Math.round((batch.tested_points / batch.total_points) * 100);
  }

  getBatchPassRate(batch: TestBatchInfo): number {
    if (batch.tested_points === 0) return 0;
    return Math.round((batch.passed_points / batch.tested_points) * 100);
  }

  getTotalProgress(): number {
    if (this.activeBatches.length === 0) return 0;
    const totalProgress = this.activeBatches.reduce((sum, batch) => sum + this.getBatchProgress(batch), 0);
    return Math.round(totalProgress / this.activeBatches.length);
  }

  getTotalPassRate(): number {
    const allBatches = [...this.activeBatches, ...this.completedBatches];
    if (allBatches.length === 0) return 0;
    
    const totalTested = allBatches.reduce((sum, batch) => sum + batch.tested_points, 0);
    const totalPassed = allBatches.reduce((sum, batch) => sum + batch.passed_points, 0);
    
    if (totalTested === 0) return 0;
    return Math.round((totalPassed / totalTested) * 100);
  }

  getTotalRuntime(): string {
    const totalMs = this.completedBatches.reduce((sum, batch) => 
      sum + (batch.total_test_duration_ms || 0), 0);
    return this.formatDuration(totalMs);
  }

  getLatestUpdates(batchId: string): TestProgressUpdate[] {
    return this.progressUpdates
      .filter(update => update.batch_id === batchId)
      .sort((a, b) => new Date(b.timestamp).getTime() - new Date(a.timestamp).getTime());
  }

  getStatusClass(status: OverallTestStatus): string {
    return 'status-' + status.toLowerCase();
  }

  getStatusLabel(status: OverallTestStatus): string {
    return OVERALL_TEST_STATUS_LABELS[status] || status;
  }

  formatDate(dateString: string): string {
    const date = new Date(dateString);
    return date.toLocaleDateString('zh-CN', {
      month: 'short',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit'
    });
  }

  formatTime(dateString: string): string {
    const date = new Date(dateString);
    return date.toLocaleTimeString('zh-CN', {
      hour: '2-digit',
      minute: '2-digit',
      second: '2-digit'
    });
  }

  formatDuration(ms: number): string {
    if (ms === 0) return '0秒';
    
    const seconds = Math.floor(ms / 1000);
    const minutes = Math.floor(seconds / 60);
    const hours = Math.floor(minutes / 60);
    
    if (hours > 0) {
      return `${hours}小时${minutes % 60}分钟`;
    } else if (minutes > 0) {
      return `${minutes}分钟${seconds % 60}秒`;
    } else {
      return `${seconds}秒`;
    }
  }
} 