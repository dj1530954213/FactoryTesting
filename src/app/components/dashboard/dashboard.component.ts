import { Component, OnInit, OnDestroy } from '@angular/core';
import { CommonModule } from '@angular/common';
import { Subscription } from 'rxjs';
import { TauriApiService } from '../../services/tauri-api.service';
import { 
  SystemStatus, 
  TestBatchInfo, 
  ChannelPointDefinition,
  OVERALL_TEST_STATUS_LABELS,
  OverallTestStatus 
} from '../../models';

@Component({
  selector: 'app-dashboard',
  standalone: true,
  imports: [CommonModule],
  template: `
    <div class="dashboard-container">
      <!-- 页面标题 -->
      <div class="page-header">
        <h1 class="page-title">系统仪表板</h1>
        <p class="page-subtitle">工厂测试系统运行状态概览</p>
      </div>

      <!-- 系统状态卡片 -->
      <div class="status-cards">
        <div class="status-card system-health" [class.healthy]="systemStatus?.system_health === 'healthy'">
          <div class="card-icon">🏥</div>
          <div class="card-content">
            <h3>系统健康</h3>
            <p class="status-value">{{systemStatus?.system_health === 'healthy' ? '正常' : '异常'}}</p>
            <small>版本: {{systemStatus?.version || 'Unknown'}}</small>
          </div>
        </div>

        <div class="status-card active-tasks">
          <div class="card-icon">⚡</div>
          <div class="card-content">
            <h3>活动任务</h3>
            <p class="status-value">{{systemStatus?.active_test_tasks || 0}}</p>
            <small>正在执行的测试任务</small>
          </div>
        </div>

        <div class="status-card total-channels">
          <div class="card-icon">📊</div>
          <div class="card-content">
            <h3>通道总数</h3>
            <p class="status-value">{{totalChannels}}</p>
            <small>已配置的测试通道</small>
          </div>
        </div>

        <div class="status-card total-batches">
          <div class="card-icon">📦</div>
          <div class="card-content">
            <h3>批次总数</h3>
            <p class="status-value">{{totalBatches}}</p>
            <small>历史测试批次</small>
          </div>
        </div>
      </div>

      <!-- 最近批次 -->
      <div class="recent-section">
        <div class="section-header">
          <h2>最近测试批次</h2>
          <button class="btn-link" (click)="navigateToBatchManagement()">查看全部</button>
        </div>
        
        <div class="recent-batches" *ngIf="recentBatches.length > 0; else noBatches">
          <div class="batch-card" *ngFor="let batch of recentBatches">
            <div class="batch-header">
              <h4>{{batch.product_model || '未知产品'}}</h4>
              <span class="batch-id">{{batch.batch_id}}</span>
            </div>
            <div class="batch-info">
              <div class="info-item">
                <span class="label">序列号:</span>
                <span class="value">{{batch.serial_number || 'N/A'}}</span>
              </div>
              <div class="info-item">
                <span class="label">总点数:</span>
                <span class="value">{{batch.total_points}}</span>
              </div>
              <div class="info-item">
                <span class="label">通过率:</span>
                <span class="value success">{{getPassRate(batch)}}%</span>
              </div>
            </div>
            <div class="batch-footer">
              <span class="timestamp">{{formatDate(batch.created_at)}}</span>
              <button class="btn-small" (click)="viewBatchDetails(batch.batch_id)">查看详情</button>
            </div>
          </div>
        </div>

        <ng-template #noBatches>
          <div class="empty-state">
            <div class="empty-icon">📭</div>
            <h3>暂无测试批次</h3>
            <p>开始创建您的第一个测试批次</p>
            <button class="btn-primary" (click)="navigateToBatchManagement()">创建批次</button>
          </div>
        </ng-template>
      </div>

      <!-- 快速操作 -->
      <div class="quick-actions">
        <div class="section-header">
          <h2>快速操作</h2>
        </div>
        
        <div class="action-grid">
          <button class="action-card" (click)="navigateToChannelConfig()">
            <div class="action-icon">⚙️</div>
            <h4>通道配置</h4>
            <p>配置测试通道参数</p>
          </button>
          
          <button class="action-card" (click)="navigateToTestExecution()">
            <div class="action-icon">🧪</div>
            <h4>开始测试</h4>
            <p>执行自动化测试</p>
          </button>
          
          <button class="action-card" (click)="navigateToManualTest()">
            <div class="action-icon">🔧</div>
            <h4>手动测试</h4>
            <p>进行手动测试操作</p>
          </button>
          
          <button class="action-card" (click)="navigateToDataImport()">
            <div class="action-icon">📥</div>
            <h4>数据导入</h4>
            <p>导入通道配置数据</p>
          </button>
        </div>
      </div>
    </div>
  `,
  styleUrls: ['./dashboard.component.css']
})
export class DashboardComponent implements OnInit, OnDestroy {
  systemStatus: SystemStatus | null = null;
  recentBatches: TestBatchInfo[] = [];
  totalChannels = 0;
  totalBatches = 0;
  
  private subscriptions: Subscription[] = [];

  constructor(private tauriApi: TauriApiService) {}

  ngOnInit(): void {
    this.loadDashboardData();
    this.subscribeToSystemStatus();
  }

  ngOnDestroy(): void {
    this.subscriptions.forEach(sub => sub.unsubscribe());
  }

  /**
   * 加载仪表板数据
   */
  private loadDashboardData(): void {
    // 加载批次信息
    const batchSub = this.tauriApi.getAllBatchInfo().subscribe({
      next: (batches) => {
        this.totalBatches = batches.length;
        this.recentBatches = batches
          .sort((a, b) => new Date(b.created_at).getTime() - new Date(a.created_at).getTime())
          .slice(0, 5); // 只显示最近5个批次
      },
      error: (error) => {
        console.error('加载批次信息失败:', error);
      }
    });

    // 加载通道定义
    const channelSub = this.tauriApi.getAllChannelDefinitions().subscribe({
      next: (channels) => {
        this.totalChannels = channels.length;
      },
      error: (error) => {
        console.error('加载通道定义失败:', error);
      }
    });

    this.subscriptions.push(batchSub, channelSub);
  }

  /**
   * 订阅系统状态更新
   */
  private subscribeToSystemStatus(): void {
    const statusSub = this.tauriApi.systemStatus$.subscribe({
      next: (status) => {
        this.systemStatus = status;
      },
      error: (error) => {
        console.error('系统状态订阅失败:', error);
      }
    });

    this.subscriptions.push(statusSub);
  }

  /**
   * 计算批次通过率
   */
  getPassRate(batch: TestBatchInfo): number {
    if (batch.total_points === 0) return 0;
    return Math.round((batch.passed_points / batch.total_points) * 100);
  }

  /**
   * 格式化日期
   */
  formatDate(dateString: string): string {
    const date = new Date(dateString);
    return date.toLocaleDateString('zh-CN', {
      year: 'numeric',
      month: 'short',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit'
    });
  }

  /**
   * 导航方法
   */
  navigateToBatchManagement(): void {
    // TODO: 实现路由导航
    console.log('导航到批次管理');
  }

  navigateToChannelConfig(): void {
    // TODO: 实现路由导航
    console.log('导航到通道配置');
  }

  navigateToTestExecution(): void {
    // TODO: 实现路由导航
    console.log('导航到测试执行');
  }

  navigateToManualTest(): void {
    // TODO: 实现路由导航
    console.log('导航到手动测试');
  }

  navigateToDataImport(): void {
    // TODO: 实现路由导航
    console.log('导航到数据导入');
  }

  viewBatchDetails(batchId: string): void {
    // TODO: 实现批次详情查看
    console.log('查看批次详情:', batchId);
  }
} 