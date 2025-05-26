import { Component, OnInit, OnDestroy } from '@angular/core';
import { CommonModule } from '@angular/common';
import { RouterModule, Router } from '@angular/router';
import { FormsModule } from '@angular/forms';
import { Subscription, interval } from 'rxjs';
import { TauriApiService } from '../../services/tauri-api.service';
import { SystemStatus, TestBatchInfo, OverallTestStatus } from '../../models';

interface AvailableBatch {
  id: string;
  productModel: string;
  serialNumber: string;
  totalPoints: number;
  analogPoints: number;
  digitalPoints: number;
  status: string;
}

interface TestProgress {
  total: number;
  completed: number;
  passed: number;
  failed: number;
  pending: number;
}

interface FinalResults {
  passed: number;
  failed: number;
}

@Component({
  selector: 'app-dashboard',
  standalone: true,
  imports: [CommonModule, RouterModule, FormsModule],
  templateUrl: './dashboard.component.html',
  styleUrls: ['./dashboard.component.css']
})
export class DashboardComponent implements OnInit, OnDestroy {
  // 系统状态
  systemStatus: SystemStatus | null = null;
  recentBatches: TestBatchInfo[] = [];
  totalChannels = 0;
  totalBatches = 0;
  pendingBatches = 0;
  overallSuccessRate = 0;
  loading = true;
  loadingMessage = '正在加载系统数据...';
  error: string | null = null;

  // 工作流程状态
  hasImportedData = false;
  selectedBatchId = '';
  selectedBatch: AvailableBatch | null = null;
  availableBatches: AvailableBatch[] = [];
  wiringConfirmed = false;
  testInProgress = false;
  testCompleted = false;
  resultExported = false;

  // 测试进度
  currentTestProgress: TestProgress = {
    total: 0,
    completed: 0,
    passed: 0,
    failed: 0,
    pending: 0
  };

  finalResults: FinalResults = {
    passed: 0,
    failed: 0
  };

  private subscriptions: Subscription[] = [];

  constructor(
    private tauriApi: TauriApiService,
    private router: Router
  ) {}

  ngOnInit() {
    this.loadDashboardData();
    this.loadAvailableBatches();
    
    // 每30秒自动刷新数据
    const refreshSubscription = interval(30000).subscribe(() => {
      this.loadDashboardData();
      if (this.testInProgress) {
        this.updateTestProgress();
      }
    });
    this.subscriptions.push(refreshSubscription);
  }

  ngOnDestroy() {
    this.subscriptions.forEach(sub => sub.unsubscribe());
  }

  async loadDashboardData() {
    try {
      this.loading = true;
      this.error = null;

      // 并行加载所有数据
      const [systemStatus, allBatches, allChannels] = await Promise.all([
        this.tauriApi.getSystemStatus().toPromise(),
        this.tauriApi.getAllBatchInfo().toPromise(),
        this.tauriApi.getAllChannelDefinitions().toPromise()
      ]);

      this.systemStatus = systemStatus || null;
      this.totalChannels = allChannels?.length || 0;
      this.totalBatches = allBatches?.length || 0;
      
      // 计算待测批次数量
      this.pendingBatches = (allBatches || []).filter(batch => 
        batch.overall_status === OverallTestStatus.NotTested
      ).length;
      
      // 计算总体成功率
      const completedBatches = (allBatches || []).filter(batch => 
        batch.overall_status === OverallTestStatus.TestCompletedPassed ||
        batch.overall_status === OverallTestStatus.TestCompletedFailed
      );
      if (completedBatches.length > 0) {
        const totalTests = completedBatches.reduce((sum, batch) => sum + (batch.total_points || 0), 0);
        const passedTests = completedBatches.reduce((sum, batch) => sum + (batch.passed_points || 0), 0);
        this.overallSuccessRate = totalTests > 0 ? Math.round((passedTests / totalTests) * 100) : 0;
      }
      
      // 获取最近的批次
      this.recentBatches = (allBatches || [])
        .sort((a: TestBatchInfo, b: TestBatchInfo) => new Date(b.created_at).getTime() - new Date(a.created_at).getTime())
        .slice(0, 5);

      // 检查是否有导入的数据
      this.hasImportedData = this.totalBatches > 0;

    } catch (error) {
      console.error('加载仪表板数据失败:', error);
      this.error = '加载数据失败，请稍后重试';
    } finally {
      this.loading = false;
    }
  }

  async loadAvailableBatches() {
    try {
      // 模拟加载可用批次数据
      this.availableBatches = [
        {
          id: '1',
          productModel: 'Model-A',
          serialNumber: 'SN001',
          totalPoints: 50,
          analogPoints: 20,
          digitalPoints: 30,
          status: 'ready'
        },
        {
          id: '2',
          productModel: 'Model-B',
          serialNumber: 'SN002',
          totalPoints: 75,
          analogPoints: 35,
          digitalPoints: 40,
          status: 'ready'
        }
      ];
    } catch (error) {
      console.error('加载可用批次失败:', error);
    }
  }

  // 系统状态相关方法
  getSystemHealthText(): string {
    if (!this.systemStatus?.system_health) return '未知';
    return this.systemStatus.system_health === 'healthy' ? '正常' : '异常';
  }

  // 工作流程方法
  navigateToDataImport() {
    this.router.navigate(['/data-import']);
  }

  navigateToManualTest() {
    this.router.navigate(['/manual-test']);
  }

  onBatchSelected() {
    this.selectedBatch = this.availableBatches.find(batch => batch.id === this.selectedBatchId) || null;
    if (this.selectedBatch) {
      this.wiringConfirmed = false;
      this.testInProgress = false;
      this.testCompleted = false;
      this.resultExported = false;
    }
  }

  getAnalogPointsCount(): number {
    return this.selectedBatch?.analogPoints || 0;
  }

  getDigitalPointsCount(): number {
    return this.selectedBatch?.digitalPoints || 0;
  }

  confirmWiring() {
    this.wiringConfirmed = true;
  }

  startTest() {
    if (!this.selectedBatch) return;
    
    this.testInProgress = true;
    this.testCompleted = false;
    
    // 初始化测试进度
    this.currentTestProgress = {
      total: this.selectedBatch.totalPoints,
      completed: 0,
      passed: 0,
      failed: 0,
      pending: this.selectedBatch.totalPoints
    };
    
    // 模拟测试进度
    this.simulateTestProgress();
  }

  private simulateTestProgress() {
    const interval = setInterval(() => {
      if (this.currentTestProgress.completed < this.currentTestProgress.total) {
        this.currentTestProgress.completed++;
        this.currentTestProgress.pending--;
        
        // 随机分配通过或失败
        if (Math.random() > 0.1) { // 90% 通过率
          this.currentTestProgress.passed++;
        } else {
          this.currentTestProgress.failed++;
        }
      } else {
        // 测试完成
        clearInterval(interval);
        this.testInProgress = false;
        this.testCompleted = true;
        this.finalResults = {
          passed: this.currentTestProgress.passed,
          failed: this.currentTestProgress.failed
        };
      }
    }, 1000); // 每秒更新一次
  }

  updateTestProgress() {
    // TODO: 从后端获取实际测试进度
  }

  getTestButtonText(): string {
    if (this.testInProgress) return '测试进行中...';
    if (this.testCompleted) return '测试已完成';
    return '开始测试';
  }

  getTestStatusClass(): string {
    if (this.testInProgress) return 'status-running';
    if (this.testCompleted) return 'status-completed';
    return 'status-ready';
  }

  getTestStatusText(): string {
    if (this.testInProgress) return '测试进行中';
    if (this.testCompleted) return '测试已完成';
    return '准备就绪';
  }

  getProgressPercentage(): number {
    if (this.currentTestProgress.total === 0) return 0;
    return Math.round((this.currentTestProgress.completed / this.currentTestProgress.total) * 100);
  }

  getFinalSuccessRate(): number {
    const total = this.finalResults.passed + this.finalResults.failed;
    if (total === 0) return 0;
    return Math.round((this.finalResults.passed / total) * 100);
  }

  viewTestDetails() {
    this.router.navigate(['/test-execution'], { 
      queryParams: { batchId: this.selectedBatchId } 
    });
  }

  exportResults() {
    console.log('导出测试结果');
    this.resultExported = true;
  }

  resetWorkflow() {
    this.selectedBatchId = '';
    this.selectedBatch = null;
    this.wiringConfirmed = false;
    this.testInProgress = false;
    this.testCompleted = false;
    this.resultExported = false;
    this.loadAvailableBatches();
  }

  // 批次相关方法
  calculatePassRate(batch: TestBatchInfo): number {
    if (!batch.total_points || batch.total_points === 0) return 0;
    const passedPoints = batch.passed_points || 0;
    return Math.round((passedPoints / batch.total_points) * 100);
  }

  formatTime(dateString: string): string {
    const date = new Date(dateString);
    const now = new Date();
    const diffMs = now.getTime() - date.getTime();
    const diffHours = Math.floor(diffMs / (1000 * 60 * 60));
    const diffDays = Math.floor(diffHours / 24);

    if (diffDays > 0) {
      return `${diffDays}天前`;
    } else if (diffHours > 0) {
      return `${diffHours}小时前`;
    } else {
      const diffMinutes = Math.floor(diffMs / (1000 * 60));
      return diffMinutes > 0 ? `${diffMinutes}分钟前` : '刚刚';
    }
  }

  getBatchStatusText(status: string): string {
    const statusMap: { [key: string]: string } = {
      'pending': '待开始',
      'ready': '准备就绪',
      'running': '进行中',
      'completed': '已完成',
      'failed': '失败',
      'cancelled': '已取消'
    };
    return statusMap[status] || status;
  }

  getBatchStatusClass(status: string): string {
    const classMap: { [key: string]: string } = {
      'pending': 'status-pending',
      'ready': 'status-ready',
      'running': 'status-running',
      'completed': 'status-completed',
      'failed': 'status-failed',
      'cancelled': 'status-cancelled'
    };
    return classMap[status] || 'status-unknown';
  }

  viewBatchDetails(batch: TestBatchInfo) {
    this.router.navigate(['/test-execution'], { 
      queryParams: { batchId: batch.batch_id } 
    });
  }

  exportBatchReport(batch: TestBatchInfo) {
    console.log('导出批次报告:', batch.batch_id);
  }

  // 刷新数据
  onRefresh() {
    this.loadDashboardData();
    this.loadAvailableBatches();
  }
}
