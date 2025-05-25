import { Component, OnInit, OnDestroy } from '@angular/core';
import { CommonModule } from '@angular/common';
import { FormsModule } from '@angular/forms';
import { ActivatedRoute, Router } from '@angular/router';
import { Subscription, interval } from 'rxjs';
import { TauriApiService } from '../../services/tauri-api.service';
import { 
  ChannelTestInstance, 
  TestBatchInfo, 
  ChannelPointDefinition,
  RawTestOutcome,
  TestProgressUpdate,
  OverallTestStatus,
  SubTestStatus,
  OVERALL_TEST_STATUS_LABELS,
  SUB_TEST_STATUS_LABELS
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
  templateUrl: './test-execution.component.html',
  styleUrl: './test-execution.component.css'
})
export class TestExecutionComponent implements OnInit, OnDestroy {
  // 核心数据
  currentBatchId: string | null = null;
  executionData: TestExecutionData | null = null;
  statistics: TestStatistics = {
    total: 0,
    completed: 0,
    passed: 0,
    failed: 0,
    inProgress: 0,
    notStarted: 0,
    successRate: 0
  };

  // 界面状态
  isLoading = false;
  loadingMessage = '';
  error: string | null = null;
  isTestRunning = false;
  isPaused = false;

  // 筛选和显示选项
  statusFilter = '';
  moduleTypeFilter = '';
  searchTerm = '';
  showOnlyFailed = false;
  autoRefresh = true;
  refreshInterval = 2000; // 2秒刷新一次

  // 分页
  currentPage = 1;
  pageSize = 20;
  totalPages = 1;

  // 订阅管理
  private subscriptions: Subscription[] = [];

  // 枚举引用
  OverallTestStatus = OverallTestStatus;
  SubTestStatus = SubTestStatus;
  OVERALL_TEST_STATUS_LABELS = OVERALL_TEST_STATUS_LABELS;
  SUB_TEST_STATUS_LABELS = SUB_TEST_STATUS_LABELS;
  
  // 工具类引用
  Math = Math;

  constructor(
    private route: ActivatedRoute,
    private router: Router,
    private tauriApi: TauriApiService
  ) {}

  ngOnInit() {
    // 获取路由参数中的批次ID
    this.route.queryParams.subscribe(params => {
      this.currentBatchId = params['batchId'] || null;
      if (this.currentBatchId) {
        this.loadTestExecution();
        this.startAutoRefresh();
      }
    });
  }

  ngOnDestroy() {
    this.subscriptions.forEach(sub => sub.unsubscribe());
  }

  async loadTestExecution() {
    if (!this.currentBatchId) return;

    try {
      this.isLoading = true;
      this.loadingMessage = '加载测试数据...';
      this.error = null;

      // 并行加载所有相关数据
      const [batchInfo, instances, definitions, outcomes, progress] = await Promise.all([
        this.tauriApi.getAllBatchInfo().toPromise(),
        this.tauriApi.getBatchTestInstances(this.currentBatchId).toPromise(),
        this.tauriApi.getAllChannelDefinitions().toPromise(),
        this.tauriApi.getBatchResults(this.currentBatchId).toPromise(),
        this.tauriApi.getBatchProgress(this.currentBatchId).toPromise()
      ]);

      // 找到当前批次
      const currentBatch = (batchInfo || []).find(b => b.batch_id === this.currentBatchId);
      if (!currentBatch) {
        throw new Error('未找到指定的测试批次');
      }

      this.executionData = {
        batch: currentBatch,
        instances: instances || [],
        definitions: definitions || [],
        outcomes: outcomes || [],
        progress: progress || []
      };

      this.updateStatistics();
      this.updateTestStatus();

    } catch (error) {
      console.error('加载测试执行数据失败:', error);
      this.error = '加载数据失败，请稍后重试';
    } finally {
      this.isLoading = false;
    }
  }

  updateStatistics() {
    if (!this.executionData) return;

    const instances = this.executionData.instances;
    this.statistics.total = instances.length;
    this.statistics.completed = instances.filter(i => 
      i.overall_status === OverallTestStatus.Completed
    ).length;
    this.statistics.failed = instances.filter(i => 
      i.overall_status === OverallTestStatus.Failed
    ).length;
    this.statistics.inProgress = instances.filter(i => 
      i.overall_status === OverallTestStatus.InProgress
    ).length;
    this.statistics.notStarted = instances.filter(i => 
      i.overall_status === OverallTestStatus.NotStarted
    ).length;
    this.statistics.passed = this.statistics.completed - this.statistics.failed;
    
    this.statistics.successRate = this.statistics.total > 0 
      ? Math.round((this.statistics.passed / this.statistics.total) * 100) 
      : 0;
  }

  updateTestStatus() {
    if (!this.executionData) return;

    const batch = this.executionData.batch;
    this.isTestRunning = batch.overall_status === OverallTestStatus.InProgress;
    this.isPaused = false; // TODO: 添加暂停状态检测
  }

  startAutoRefresh() {
    if (this.autoRefresh) {
      const refreshSub = interval(this.refreshInterval).subscribe(() => {
        if (this.currentBatchId && this.isTestRunning) {
          this.loadTestExecution();
        }
      });
      this.subscriptions.push(refreshSub);
    }
  }

  // 测试控制方法
  async startTest() {
    if (!this.currentBatchId) return;

    try {
      this.loadingMessage = '启动测试...';
      await this.tauriApi.startBatchTesting(this.currentBatchId).toPromise();
      this.isTestRunning = true;
      this.loadTestExecution();
    } catch (error) {
      console.error('启动测试失败:', error);
      this.error = '启动测试失败';
    }
  }

  async pauseTest() {
    if (!this.currentBatchId) return;

    try {
      this.loadingMessage = '暂停测试...';
      await this.tauriApi.pauseBatchTesting(this.currentBatchId).toPromise();
      this.isPaused = true;
      this.loadTestExecution();
    } catch (error) {
      console.error('暂停测试失败:', error);
      this.error = '暂停测试失败';
    }
  }

  async resumeTest() {
    if (!this.currentBatchId) return;

    try {
      this.loadingMessage = '恢复测试...';
      await this.tauriApi.resumeBatchTesting(this.currentBatchId).toPromise();
      this.isPaused = false;
      this.loadTestExecution();
    } catch (error) {
      console.error('恢复测试失败:', error);
      this.error = '恢复测试失败';
    }
  }

  async stopTest() {
    if (!this.currentBatchId) return;

    if (!confirm('确定要停止当前测试吗？这将终止所有正在进行的测试任务。')) {
      return;
    }

    try {
      this.loadingMessage = '停止测试...';
      await this.tauriApi.stopBatchTesting(this.currentBatchId).toPromise();
      this.isTestRunning = false;
      this.isPaused = false;
      this.loadTestExecution();
    } catch (error) {
      console.error('停止测试失败:', error);
      this.error = '停止测试失败';
    }
  }

  // 数据获取和显示方法
  getFilteredInstances(): ChannelTestInstance[] {
    if (!this.executionData) return [];

    let filtered = this.executionData.instances;

    // 状态筛选
    if (this.statusFilter) {
      filtered = filtered.filter(instance => 
        instance.overall_status === this.statusFilter
      );
    }

    // 模块类型筛选
    if (this.moduleTypeFilter) {
      const definitions = this.executionData.definitions;
      filtered = filtered.filter(instance => {
        const definition = definitions.find(d => d.id === instance.definition_id);
        return definition?.module_type === this.moduleTypeFilter;
      });
    }

    // 搜索筛选
    if (this.searchTerm) {
      const definitions = this.executionData.definitions;
      const term = this.searchTerm.toLowerCase();
      filtered = filtered.filter(instance => {
        const definition = definitions.find(d => d.id === instance.definition_id);
        return definition?.tag.toLowerCase().includes(term) ||
               definition?.description.toLowerCase().includes(term);
      });
    }

    // 只显示失败项
    if (this.showOnlyFailed) {
      filtered = filtered.filter(instance => 
        instance.overall_status === OverallTestStatus.Failed
      );
    }

    return filtered;
  }

  getDefinitionForInstance(instance: ChannelTestInstance): ChannelPointDefinition | undefined {
    return this.executionData?.definitions.find(d => d.id === instance.definition_id);
  }

  getLatestOutcomeForInstance(instance: ChannelTestInstance): RawTestOutcome | undefined {
    if (!this.executionData) return undefined;
    
    return this.executionData.outcomes
      .filter(o => o.instance_id === instance.instance_id)
      .sort((a, b) => new Date(b.test_timestamp).getTime() - new Date(a.test_timestamp).getTime())[0];
  }

  getStatusClass(status: OverallTestStatus): string {
    const classMap: { [key in OverallTestStatus]: string } = {
      [OverallTestStatus.NotStarted]: 'status-not-started',
      [OverallTestStatus.InProgress]: 'status-in-progress',
      [OverallTestStatus.Completed]: 'status-completed',
      [OverallTestStatus.Failed]: 'status-failed',
      [OverallTestStatus.Cancelled]: 'status-cancelled'
    };
    return classMap[status] || 'status-unknown';
  }

  getStatusLabel(status: OverallTestStatus): string {
    return OVERALL_TEST_STATUS_LABELS[status] || '未知';
  }

  formatTimestamp(timestamp: string): string {
    return new Date(timestamp).toLocaleString('zh-CN');
  }

  formatDuration(startTime?: string, endTime?: string): string {
    if (!startTime) return '未开始';
    if (!endTime) return '进行中';
    
    const start = new Date(startTime).getTime();
    const end = new Date(endTime).getTime();
    const duration = Math.round((end - start) / 1000);
    
    if (duration < 60) return `${duration}秒`;
    if (duration < 3600) return `${Math.round(duration / 60)}分钟`;
    return `${Math.round(duration / 3600)}小时`;
  }

  // 导航和操作方法
  goToDashboard() {
    this.router.navigate(['/dashboard']);
  }

  goToBatchManagement() {
    this.router.navigate(['/batch-management']);
  }

  exportResults() {
    if (!this.currentBatchId) return;
    console.log('导出测试结果:', this.currentBatchId);
    // TODO: 实现结果导出功能
  }

  refreshData() {
    this.loadTestExecution();
  }

  toggleAutoRefresh() {
    this.autoRefresh = !this.autoRefresh;
    if (this.autoRefresh) {
      this.startAutoRefresh();
    } else {
      // 停止自动刷新
      this.subscriptions.forEach(sub => sub.unsubscribe());
      this.subscriptions = [];
    }
  }
}
