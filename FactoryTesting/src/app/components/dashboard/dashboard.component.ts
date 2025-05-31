import { Component, OnInit, OnDestroy } from '@angular/core';
import { CommonModule } from '@angular/common';
import { RouterModule, Router } from '@angular/router';
import { FormsModule } from '@angular/forms';
import { Subscription, interval } from 'rxjs';
import { TauriApiService } from '../../services/tauri-api.service';
import { SystemStatus, TestBatchInfo, OverallTestStatus } from '../../models';

// NG-ZORRO 组件导入
import { NzCardModule } from 'ng-zorro-antd/card';
import { NzStatisticModule } from 'ng-zorro-antd/statistic';
import { NzGridModule } from 'ng-zorro-antd/grid';
import { NzIconModule } from 'ng-zorro-antd/icon';
import { NzButtonModule } from 'ng-zorro-antd/button';
import { NzSpinModule } from 'ng-zorro-antd/spin';
import { NzAlertModule } from 'ng-zorro-antd/alert';
import { NzTagModule } from 'ng-zorro-antd/tag';
import { NzProgressModule } from 'ng-zorro-antd/progress';
import { NzListModule } from 'ng-zorro-antd/list';
import { NzAvatarModule } from 'ng-zorro-antd/avatar';
import { NzDividerModule } from 'ng-zorro-antd/divider';
import { NzSpaceModule } from 'ng-zorro-antd/space';

// ECharts 导入
import { NgxEchartsModule } from 'ngx-echarts';
import { EChartsOption } from 'echarts';

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

interface RecentActivity {
  icon: string;
  title: string;
  description: string;
  timestamp: Date;
}

@Component({
  selector: 'app-dashboard',
  standalone: true,
  imports: [
    CommonModule,
    RouterModule,
    FormsModule,
    // NG-ZORRO 模块
    NzCardModule,
    NzStatisticModule,
    NzGridModule,
    NzIconModule,
    NzButtonModule,
    NzSpinModule,
    NzAlertModule,
    NzTagModule,
    NzProgressModule,
    NzListModule,
    NzAvatarModule,
    NzDividerModule,
    NzSpaceModule,
    // ECharts 模块
    NgxEchartsModule
  ],
  templateUrl: './dashboard.component.html',
  styleUrls: ['./dashboard.component.css']
})
export class DashboardComponent implements OnInit, OnDestroy {
  // 系统状态
  systemStatus: SystemStatus | null = null;
  recentBatches: TestBatchInfo[] = [];
  recentActivities: RecentActivity[] = [];
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

  // ECharts 图表配置
  testProgressChartOption: EChartsOption = {};
  systemStatusChartOption: EChartsOption = {};
  batchStatusChartOption: EChartsOption = {};

  private subscriptions: Subscription[] = [];

  constructor(
    private tauriApi: TauriApiService,
    private router: Router
  ) {}

  ngOnInit() {
    this.loadDashboardData();
    this.loadAvailableBatches();
    this.initializeCharts();

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

  navigateToTestArea() {
    this.router.navigate(['/test-area']);
  }

  navigateToReports() {
    this.router.navigate(['/reports']);
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

  getBatchStatusText(status: string | OverallTestStatus): string {
    if (typeof status === 'string') {
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

    // 处理 OverallTestStatus 枚举
    const overallStatusMap: { [key in OverallTestStatus]: string } = {
      [OverallTestStatus.NotTested]: '未测试',
      [OverallTestStatus.HardPointTesting]: '硬点测试中',
      [OverallTestStatus.AlarmTesting]: '报警测试中',
      [OverallTestStatus.TestCompletedPassed]: '测试完成并通过',
      [OverallTestStatus.TestCompletedFailed]: '测试完成并失败'
    };
    return overallStatusMap[status] || '未知状态';
  }

  getBatchStatusColor(status: string | OverallTestStatus): string {
    if (typeof status === 'string') {
      const colorMap: { [key: string]: string } = {
        'pending': '#d9d9d9',
        'ready': '#1890ff',
        'running': '#fa8c16',
        'completed': '#52c41a',
        'failed': '#ff4d4f',
        'cancelled': '#8c8c8c'
      };
      return colorMap[status] || '#d9d9d9';
    }

    // 处理 OverallTestStatus 枚举
    const overallColorMap: { [key in OverallTestStatus]: string } = {
      [OverallTestStatus.NotTested]: '#d9d9d9',
      [OverallTestStatus.HardPointTesting]: '#1890ff',
      [OverallTestStatus.AlarmTesting]: '#fa8c16',
      [OverallTestStatus.TestCompletedPassed]: '#52c41a',
      [OverallTestStatus.TestCompletedFailed]: '#ff4d4f'
    };
    return overallColorMap[status] || '#d9d9d9';
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

  // 初始化图表
  private initializeCharts() {
    this.initTestProgressChart();
    this.initSystemStatusChart();
    this.initBatchStatusChart();
  }

  // 初始化测试进度图表
  private initTestProgressChart() {
    this.testProgressChartOption = {
      title: {
        text: '测试进度',
        left: 'center',
        textStyle: {
          fontSize: 14,
          fontWeight: 'normal'
        }
      },
      tooltip: {
        trigger: 'item',
        formatter: '{a} <br/>{b}: {c} ({d}%)'
      },
      legend: {
        orient: 'vertical',
        left: 'left',
        data: ['已完成', '进行中', '待测试']
      },
      series: [
        {
          name: '测试进度',
          type: 'pie',
          radius: ['40%', '70%'],
          center: ['50%', '60%'],
          data: [
            { value: this.currentTestProgress.completed, name: '已完成', itemStyle: { color: '#52c41a' } },
            { value: this.testInProgress ? 1 : 0, name: '进行中', itemStyle: { color: '#1890ff' } },
            { value: this.currentTestProgress.pending, name: '待测试', itemStyle: { color: '#d9d9d9' } }
          ],
          emphasis: {
            itemStyle: {
              shadowBlur: 10,
              shadowOffsetX: 0,
              shadowColor: 'rgba(0, 0, 0, 0.5)'
            }
          }
        }
      ]
    };
  }

  // 初始化系统状态图表
  private initSystemStatusChart() {
    this.systemStatusChartOption = {
      title: {
        text: '系统状态监控',
        left: 'center',
        textStyle: {
          fontSize: 14,
          fontWeight: 'normal'
        }
      },
      tooltip: {
        trigger: 'axis'
      },
      xAxis: {
        type: 'category',
        data: ['CPU', '内存', 'PLC连接', '数据库', '网络']
      },
      yAxis: {
        type: 'value',
        max: 100,
        axisLabel: {
          formatter: '{value}%'
        }
      },
      series: [
        {
          name: '状态',
          type: 'bar',
          data: [
            { value: 85, itemStyle: { color: '#52c41a' } },
            { value: 72, itemStyle: { color: '#52c41a' } },
            { value: this.systemStatus?.system_health === 'healthy' ? 100 : 0, itemStyle: { color: this.systemStatus?.system_health === 'healthy' ? '#52c41a' : '#ff4d4f' } },
            { value: 95, itemStyle: { color: '#52c41a' } },
            { value: 88, itemStyle: { color: '#52c41a' } }
          ]
        }
      ]
    };
  }

  // 初始化批次状态图表
  private initBatchStatusChart() {
    const statusCounts = this.calculateBatchStatusCounts();

    this.batchStatusChartOption = {
      title: {
        text: '批次状态分布',
        left: 'center',
        textStyle: {
          fontSize: 14,
          fontWeight: 'normal'
        }
      },
      tooltip: {
        trigger: 'item',
        formatter: '{a} <br/>{b}: {c} ({d}%)'
      },
      legend: {
        orient: 'horizontal',
        bottom: '0%',
        data: ['未测试', '测试中', '已完成', '失败']
      },
      series: [
        {
          name: '批次状态',
          type: 'pie',
          radius: '60%',
          center: ['50%', '45%'],
          data: [
            { value: statusCounts.notTested, name: '未测试', itemStyle: { color: '#d9d9d9' } },
            { value: statusCounts.testing, name: '测试中', itemStyle: { color: '#1890ff' } },
            { value: statusCounts.passed, name: '已完成', itemStyle: { color: '#52c41a' } },
            { value: statusCounts.failed, name: '失败', itemStyle: { color: '#ff4d4f' } }
          ]
        }
      ]
    };
  }

  // 计算批次状态统计
  private calculateBatchStatusCounts() {
    const counts = {
      notTested: 0,
      testing: 0,
      passed: 0,
      failed: 0
    };

    this.recentBatches.forEach(batch => {
      switch (batch.overall_status) {
        case OverallTestStatus.NotTested:
          counts.notTested++;
          break;
        case OverallTestStatus.HardPointTesting:
        case OverallTestStatus.AlarmTesting:
          counts.testing++;
          break;
        case OverallTestStatus.TestCompletedPassed:
          counts.passed++;
          break;
        case OverallTestStatus.TestCompletedFailed:
          counts.failed++;
          break;
      }
    });

    return counts;
  }

  // 更新图表数据
  updateCharts() {
    this.initTestProgressChart();
    this.initSystemStatusChart();
    this.initBatchStatusChart();
  }


}
