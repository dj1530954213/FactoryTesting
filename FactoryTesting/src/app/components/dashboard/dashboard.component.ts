import { Component, OnInit, OnDestroy } from '@angular/core';
import { CommonModule } from '@angular/common';
import { RouterModule, Router } from '@angular/router';
import { FormsModule } from '@angular/forms';
import { Subscription, interval } from 'rxjs';
import { TauriApiService } from '../../services/tauri-api.service';
import { SystemStatus, TestBatchInfo, OverallTestStatus, DashboardBatchInfo, DeleteBatchResponse } from '../../models';

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
import { NzModalModule, NzModalService } from 'ng-zorro-antd/modal';
import { NzMessageModule, NzMessageService } from 'ng-zorro-antd/message';

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

// 仪表盘显示的批次信息接口 - 包含模板中使用的所有字段
interface DashboardBatchDisplay {
  // 新的字段名（用于某些显示）
  id: string;
  name: string;
  station: string;
  createdAt: string;
  totalPoints: number;
  testedCount: number;
  untestedCount: number;
  successCount: number;
  failureCount: number;
  status: OverallTestStatus;
  isCurrentSession: boolean;

  // 原始字段名（模板中使用的）
  batch_id: string;
  batch_name: string;
  product_model?: string;
  serial_number?: string;
  station_name?: string;
  creation_time?: string;
  last_updated_time?: string;
  total_points: number;
  tested_points: number;
  passed_points: number;
  failed_points: number;
  skipped_points: number;
  overall_status: OverallTestStatus;
  operator_name?: string;
  created_at?: string;
  updated_at?: string;
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
    NzModalModule,
    NzMessageModule,
    // ECharts 模块
    NgxEchartsModule
  ],
  templateUrl: './dashboard.component.html',
  styleUrls: ['./dashboard.component.css']
})
export class DashboardComponent implements OnInit, OnDestroy {
  // 系统状态
  systemStatus: SystemStatus | null = null;
  recentBatches: DashboardBatchDisplay[] = []; // 🔧 修复：使用正确的类型
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
    private router: Router,
    private modal: NzModalService,
    private message: NzMessageService
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

      console.log('📊 [DASHBOARD] 开始加载仪表盘数据');

      // 🔧 修复：使用新的仪表盘批次列表API，获取所有批次并标识当前会话批次
      const [systemStatus, dashboardBatches, allChannels] = await Promise.all([
        this.tauriApi.getSystemStatus().toPromise(),
        this.tauriApi.getDashboardBatchList().toPromise(), // 🔧 使用新的仪表盘API
        this.tauriApi.getAllChannelDefinitions().toPromise()
      ]);

      console.log('📊 [DASHBOARD] 获取到的仪表盘批次数据:', dashboardBatches);

      this.systemStatus = systemStatus || null;
      this.totalChannels = allChannels?.length || 0;
      this.totalBatches = dashboardBatches?.length || 0;

      // 🔧 修复：由于后端使用了 #[serde(flatten)]，dashboardBatches 本身就是展平的数据
      // 不需要提取 batch_info，直接使用 dashboardBatches
      const allBatches = dashboardBatches?.filter(db => {
        // 确保 db 存在且有必要的字段
        if (!db || !db.batch_id) {
          console.warn('📊 [DASHBOARD] 发现无效的批次数据:', db);
          return false;
        }
        return true;
      }) || [];

      console.log('📊 [DASHBOARD] 提取的批次数据:', allBatches);
      console.log('📊 [DASHBOARD] 原始仪表盘批次数据:', dashboardBatches);

      // 计算待测批次数量
      this.pendingBatches = allBatches.filter(batch =>
        batch.overall_status === OverallTestStatus.NotTested
      ).length;

      // 计算总体成功率
      const completedBatches = allBatches.filter(batch =>
        batch.overall_status === OverallTestStatus.TestCompletedPassed ||
        batch.overall_status === OverallTestStatus.TestCompletedFailed
      );
      if (completedBatches.length > 0) {
        const totalTests = completedBatches.reduce((sum, batch) => sum + (batch.total_points || 0), 0);
        const passedTests = completedBatches.reduce((sum, batch) => sum + (batch.passed_points || 0), 0);
        this.overallSuccessRate = totalTests > 0 ? Math.round((passedTests / totalTests) * 100) : 0;
      }

      // 🔧 处理最近批次数据，转换为前端需要的格式 - 使用最保守的方法
      const validBatches = allBatches.filter(batch => {
        return batch &&
               typeof batch === 'object' &&
               batch.batch_id &&
               typeof batch.batch_id === 'string';
      });

      console.log('📊 [DASHBOARD] 有效批次数量:', validBatches.length);

      this.recentBatches = validBatches
        .sort((a: DashboardBatchInfo, b: DashboardBatchInfo) => {
          // 🔧 修复：使用正确的类型，因为现在 validBatches 是 DashboardBatchInfo[]
          const timeA = a.creation_time ? new Date(a.creation_time).getTime() : 0;
          const timeB = b.creation_time ? new Date(b.creation_time).getTime() : 0;
          return timeB - timeA; // 最新的在前
        })
        .slice(0, 10)
        .map(batch => {
          try {
            console.log('📊 [DASHBOARD] 处理批次:', batch.batch_id, '站场:', batch.station_name, '当前会话:', batch.is_current_session);

            // 🔧 修复：直接使用 batch 的会话信息，因为它本身就是 DashboardBatchInfo
            const isCurrentSession = batch.is_current_session || false;

            // 🔧 安全地获取站场信息
            const stationName = batch.station_name || '未知站场';

          return {
            // 新的字段名
            id: batch.batch_id,
            name: batch.batch_name || '未命名批次',
            station: stationName,
            createdAt: batch.creation_time || batch.created_at || new Date().toISOString(),
            totalPoints: batch.total_points || 0,
            testedCount: batch.tested_points || 0,
            untestedCount: (batch.total_points || 0) - (batch.tested_points || 0),
            successCount: batch.passed_points || 0,
            failureCount: batch.failed_points || 0,
            status: this.getStatusFromProgress(batch.tested_points || 0, batch.total_points || 0),
            isCurrentSession: isCurrentSession,

            // 原始字段名（保持兼容性）
            batch_id: batch.batch_id,
            batch_name: batch.batch_name || '未命名批次',
            product_model: batch.product_model,
            serial_number: batch.serial_number,
            station_name: stationName,
            creation_time: batch.creation_time,
            last_updated_time: batch.last_updated_time,
            total_points: batch.total_points || 0,
            tested_points: batch.tested_points || 0,
            passed_points: batch.passed_points || 0,
            failed_points: batch.failed_points || 0,
            skipped_points: batch.skipped_points || 0,
            overall_status: this.getStatusFromProgress(batch.tested_points || 0, batch.total_points || 0),
            operator_name: batch.operator_name,
            created_at: batch.created_at,
            updated_at: batch.updated_at
          };
          } catch (error) {
            console.error('📊 [DASHBOARD] 处理批次数据时发生错误:', error, '批次:', batch);
            return null;
          }
        })
        .filter(batch => batch !== null); // 🔧 过滤掉null值

      // 🔍 调试：检查站场信息
      console.log('📊 [DASHBOARD] 最终的recentBatches数组:', this.recentBatches);
      this.recentBatches.forEach((batch, index) => {
        console.log(`📊 [DASHBOARD] 批次${index + 1}:`, {
          id: batch.id,
          station: batch.station,
          station_name: batch.station_name,
          isCurrentSession: batch.isCurrentSession,
          batch对象: batch
        });
      });

      // 检查是否有导入的数据
      this.hasImportedData = this.totalBatches > 0;

      console.log('📊 [DASHBOARD] 仪表盘数据加载完成');
      console.log('📊 [DASHBOARD] 总批次数:', this.totalBatches);
      console.log('📊 [DASHBOARD] 最近批次数:', this.recentBatches.length);

    } catch (error) {
      console.error('📊 [DASHBOARD] 加载仪表板数据失败:', error);
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

  // 页面操作方法
  onRefresh() {
    this.loadDashboardData();
    this.loadAvailableBatches();
  }

  viewBatchDetails(batch: DashboardBatchDisplay) {
    console.log('🔍 [viewBatchDetails] 输入参数:', batch);
    console.log('🔍 [viewBatchDetails] 批次类型:', typeof batch);
    console.log('🔍 [viewBatchDetails] 批次属性:', Object.keys(batch || {}));

    if (batch && batch.station_name) {
      console.log('🔍 [viewBatchDetails] 站场信息:', batch.station_name);
    } else {
      console.log('🔍 [viewBatchDetails] ⚠️ 站场信息缺失');
    }

    console.log('🔍 [viewBatchDetails] 导航到测试区域，批次ID:', batch?.id);
    this.router.navigate(['/test-area'], {
      queryParams: { batchId: batch?.id }
    });
  }

  /**
   * 删除批次 - 级联删除三张表中的所有关联数据
   * @param batch 要删除的批次信息
   */
  deleteBatch(batch: DashboardBatchDisplay) {
    console.log('🗑️ [DELETE_BATCH] 准备删除批次:', batch.id, batch.name);

    // 优化的确认对话框 - 使用更简洁的内容和更好的动画
    const modal = this.modal.confirm({
      nzTitle: '⚠️ 确认删除批次',
      nzContent: this.createDeleteConfirmContent(batch),
      nzOkText: '🗑️ 确认删除',
      nzOkType: 'primary',
      nzOkDanger: true,
      nzCancelText: '✖️ 取消',
      nzWidth: 480,
      nzMaskClosable: false,
      nzKeyboard: true,
      nzCentered: true,
      nzMaskStyle: {
        'backdrop-filter': 'blur(4px)',
        'background-color': 'rgba(0, 0, 0, 0.45)'
      },
      nzBodyStyle: {
        'padding': '24px',
        'line-height': '1.6'
      },
      nzOnOk: () => {
        // 立即关闭对话框，提供即时反馈
        modal.close();
        return this.performBatchDeletion(batch);
      },
      nzOnCancel: () => {
        console.log('🚫 [DELETE_BATCH] 用户取消删除操作');
      }
    });

    // 添加对话框打开动画
    setTimeout(() => {
      const modalElement = document.querySelector('.ant-modal');
      if (modalElement) {
        modalElement.classList.add('modal-fade-in');
      }
    }, 10);
  }

  /**
   * 创建删除确认对话框的内容
   * @param batch 批次信息
   * @returns HTML内容字符串
   */
  private createDeleteConfirmContent(batch: DashboardBatchDisplay): string {
    return `
      <div style="font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;">
        <div style="margin-bottom: 16px;">
          <p style="font-size: 16px; margin: 0 0 12px 0; color: #262626;">
            您确定要删除批次 <strong style="color: #1890ff;">"${batch.name}"</strong> 吗？
          </p>
          <div style="background: #fff2e8; border: 1px solid #ffbb96; border-radius: 6px; padding: 12px; margin: 12px 0;">
            <p style="margin: 0 0 8px 0; color: #fa541c; font-weight: 500; display: flex; align-items: center;">
              <span style="margin-right: 6px;">⚠️</span>
              此操作将永久删除以下数据：
            </p>
            <ul style="margin: 8px 0 0 0; padding-left: 20px; color: #595959; line-height: 1.8;">
              <li>📊 批次信息 (test_batch_info 表)</li>
              <li>🧪 测试实例 (channel_test_instances 表)</li>
              <li>📋 通道定义 (channel_point_definitions 表)</li>
            </ul>
          </div>
          <p style="color: #ff4d4f; font-weight: 600; margin: 16px 0 0 0; text-align: center; font-size: 14px;">
            🚨 此操作不可撤销！
          </p>
        </div>
      </div>
    `;
  }

  /**
   * 执行批次删除操作 - 优化版本，提供更流畅的用户体验
   * @param batch 要删除的批次信息
   */
  private async performBatchDeletion(batch: DashboardBatchDisplay): Promise<void> {
    // 显示优化的加载消息
    const loadingMessage = this.message.loading(
      `🗑️ 正在删除批次 "${batch.name}"...`,
      { nzDuration: 0 }
    );

    try {
      console.log('🗑️ [DELETE_BATCH] 开始执行删除操作:', batch.id);

      // 添加短暂延迟，让用户看到加载状态
      await new Promise(resolve => setTimeout(resolve, 300));

      // 调用后端API删除批次
      const result = await this.tauriApi.deleteBatch(batch.id).toPromise();

      console.log('✅ [DELETE_BATCH] 删除操作完成:', result);

      // 关闭加载消息
      this.message.remove(loadingMessage.messageId);

      if (result && result.success) {
        // 删除成功 - 显示优化的成功消息
        this.message.success(
          `🎉 批次 "${batch.name}" 删除成功！已清理 ${result.deleted_definitions_count} 个通道定义和 ${result.deleted_instances_count} 个测试实例`,
          { nzDuration: 4000 }
        );

        // 添加视觉反馈 - 先从列表中移除该项
        this.recentBatches = this.recentBatches.filter(b => b.id !== batch.id);
        this.totalBatches = Math.max(0, this.totalBatches - 1);

        // 延迟刷新数据，让用户看到即时的视觉反馈
        setTimeout(async () => {
          await this.loadDashboardData();
          console.log('✅ [DELETE_BATCH] 仪表盘数据已刷新');
        }, 500);

      } else {
        // 删除失败或结果为空
        const errorMessage = result?.message || '删除操作返回空结果';
        this.message.error(
          `❌ 删除批次失败: ${errorMessage}`,
          { nzDuration: 6000 }
        );
        console.error('❌ [DELETE_BATCH] 删除失败:', errorMessage);
      }

    } catch (error) {
      console.error('❌ [DELETE_BATCH] 删除批次时发生错误:', error);

      // 关闭加载消息
      this.message.remove(loadingMessage.messageId);

      // 显示优化的错误消息
      const errorMsg = error instanceof Error ? error.message : '未知错误';
      this.message.error(
        `💥 删除批次时发生错误: ${errorMsg}`,
        { nzDuration: 8000 }
      );
    }
  }

  // 批次相关方法 - 支持两种类型
  calculatePassRate(batch: TestBatchInfo | DashboardBatchDisplay | null | undefined): number {
    console.log('🔍 [calculatePassRate] 输入参数:', batch);

    if (!batch) {
      console.log('🔍 [calculatePassRate] 批次为空，返回0');
      return 0;
    }

    // 由于 DashboardBatchDisplay 包含了所有字段，直接使用原始字段名
    const batchData = batch as any;
    const totalPoints = batchData.total_points || batchData.totalPoints || 0;
    const passedPoints = batchData.passed_points || batchData.successCount || 0;

    console.log('🔍 [calculatePassRate] 解析数据:', { totalPoints, passedPoints, batchData });

    if (!totalPoints || totalPoints === 0) {
      console.log('🔍 [calculatePassRate] 总点位为0，返回0');
      return 0;
    }

    const result = Math.round((passedPoints / totalPoints) * 100);
    console.log('🔍 [calculatePassRate] 计算结果:', result);
    return result;
  }

  formatTime(dateString: string): string {
    if (!dateString) {
      return '未知时间';
    }

    const date = new Date(dateString);
    if (isNaN(date.getTime())) {
      return '无效时间';
    }

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

  // 格式化详细时间显示
  formatDetailedTime(dateString: string): string {
    if (!dateString) {
      return '未知时间';
    }

    const date = new Date(dateString);
    if (isNaN(date.getTime())) {
      return '无效时间';
    }

    // 格式化为 YYYY-MM-DD HH:mm:ss
    const year = date.getFullYear();
    const month = String(date.getMonth() + 1).padStart(2, '0');
    const day = String(date.getDate()).padStart(2, '0');
    const hours = String(date.getHours()).padStart(2, '0');
    const minutes = String(date.getMinutes()).padStart(2, '0');
    const seconds = String(date.getSeconds()).padStart(2, '0');

    return `${year}-${month}-${day} ${hours}:${minutes}:${seconds}`;
  }

  // 获取未测试点位数量 - 支持两种类型
  getUntestedPoints(batch: TestBatchInfo | DashboardBatchDisplay | null | undefined): number {
    console.log('🔍 [getUntestedPoints] 输入参数:', batch);

    if (!batch) {
      console.log('🔍 [getUntestedPoints] 批次为空，返回0');
      return 0;
    }

    const batchData = batch as any;
    const tested = batchData.tested_points || batchData.testedCount || 0;
    const total = batchData.total_points || batchData.totalPoints || 0;

    console.log('🔍 [getUntestedPoints] 解析数据:', { tested, total, batchData });

    const result = Math.max(0, total - tested);
    console.log('🔍 [getUntestedPoints] 计算结果:', result);
    return result;
  }

  // 获取测试进度百分比 - 支持两种类型
  getTestProgress(batch: TestBatchInfo | DashboardBatchDisplay | null | undefined): number {
    console.log('🔍 [getTestProgress] 输入参数:', batch);

    if (!batch) {
      console.log('🔍 [getTestProgress] 批次为空，返回0');
      return 0;
    }

    const batchData = batch as any;
    const total = batchData.total_points || batchData.totalPoints || 0;
    const tested = batchData.tested_points || batchData.testedCount || 0;

    console.log('🔍 [getTestProgress] 解析数据:', { total, tested, batchData });

    if (!total || total === 0) {
      console.log('🔍 [getTestProgress] 总点位为0，返回0');
      return 0;
    }

    const result = Math.round((tested / total) * 100);
    console.log('🔍 [getTestProgress] 计算结果:', result);
    return result;
  }

  // 获取进度条状态 - 支持两种类型
  getProgressStatus(batch: TestBatchInfo | DashboardBatchDisplay | null | undefined): 'success' | 'exception' | 'active' | 'normal' {
    console.log('🔍 [getProgressStatus] 输入参数:', batch);

    if (!batch) {
      console.log('🔍 [getProgressStatus] 批次为空，返回normal');
      return 'normal';
    }

    const progress = this.getTestProgress(batch);
    const passRate = this.calculatePassRate(batch);

    console.log('🔍 [getProgressStatus] 计算数据:', { progress, passRate });

    if (progress === 100) {
      const result = passRate >= 90 ? 'success' : 'exception';
      console.log('🔍 [getProgressStatus] 测试完成，结果:', result);
      return result;
    } else if (progress > 0) {
      console.log('🔍 [getProgressStatus] 测试进行中，返回active');
      return 'active';
    }

    console.log('🔍 [getProgressStatus] 未开始测试，返回normal');
    return 'normal';
  }

  getBatchStatusText(status: string | OverallTestStatus | undefined): string {
    console.log('🔍 [getBatchStatusText] 输入参数:', status);

    if (!status) {
      console.log('🔍 [getBatchStatusText] 状态为空，返回未知状态');
      return '未知状态';
    }

    if (typeof status === 'string') {
      const statusMap: { [key: string]: string } = {
        'pending': '待开始',
        'ready': '准备就绪',
        'running': '进行中',
        'completed': '已完成',
        'failed': '失败',
        'cancelled': '已取消'
      };
      const result = statusMap[status] || status;
      console.log('🔍 [getBatchStatusText] 字符串状态转换结果:', result);
      return result;
    }

    // 处理 OverallTestStatus 枚举
    const overallStatusMap: { [key in OverallTestStatus]: string } = {
      [OverallTestStatus.NotTested]: '未测试',
      [OverallTestStatus.HardPointTesting]: '硬点测试中',
      [OverallTestStatus.AlarmTesting]: '报警测试中',
      [OverallTestStatus.TestCompletedPassed]: '测试完成并通过',
      [OverallTestStatus.TestCompletedFailed]: '测试完成并失败'
    };
    const result = overallStatusMap[status] || '未知状态';
    console.log('🔍 [getBatchStatusText] 枚举状态转换结果:', result);
    return result;
  }

  getBatchStatusColor(status: string | OverallTestStatus | undefined): string {
    console.log('🔍 [getBatchStatusColor] 输入参数:', status);

    if (!status) {
      console.log('🔍 [getBatchStatusColor] 状态为空，返回默认颜色');
      return '#d9d9d9';
    }

    if (typeof status === 'string') {
      const colorMap: { [key: string]: string } = {
        'pending': '#d9d9d9',
        'ready': '#1890ff',
        'running': '#fa8c16',
        'completed': '#52c41a',
        'failed': '#ff4d4f',
        'cancelled': '#8c8c8c'
      };
      const result = colorMap[status] || '#d9d9d9';
      console.log('🔍 [getBatchStatusColor] 字符串状态颜色结果:', result);
      return result;
    }

    // 处理 OverallTestStatus 枚举
    const overallColorMap: { [key in OverallTestStatus]: string } = {
      [OverallTestStatus.NotTested]: '#d9d9d9',
      [OverallTestStatus.HardPointTesting]: '#1890ff',
      [OverallTestStatus.AlarmTesting]: '#fa8c16',
      [OverallTestStatus.TestCompletedPassed]: '#52c41a',
      [OverallTestStatus.TestCompletedFailed]: '#ff4d4f'
    };
    const result = overallColorMap[status] || '#d9d9d9';
    console.log('🔍 [getBatchStatusColor] 枚举状态颜色结果:', result);
    return result;
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

  // 创建测试数据用于演示
  async createTestData() {
    try {
      console.log('开始创建测试数据...');

      // 创建多个测试批次，使用真实的站场名称
      const testBatches = [
        {
          batch_name: '樟洋电厂-批次001',
          product_model: 'DCS-X1000',
          serial_number: 'ZY20241201001',
          station_name: '樟洋电厂',
          operator_name: '张三',
          total_points: 48,
          tested_points: 48,
          passed_points: 45,
          failed_points: 3,
          skipped_points: 0
        },
        {
          batch_name: '樟洋电厂-批次002',
          product_model: 'DCS-Y2000',
          serial_number: 'ZY20241201002',
          station_name: '樟洋电厂',
          operator_name: '李四',
          total_points: 32,
          tested_points: 28,
          passed_points: 26,
          failed_points: 2,
          skipped_points: 0
        },
        {
          batch_name: '樟洋电厂-批次003',
          product_model: 'DCS-Z3000',
          serial_number: 'ZY20241201003',
          station_name: '樟洋电厂',
          operator_name: '王五',
          total_points: 64,
          tested_points: 15,
          passed_points: 14,
          failed_points: 1,
          skipped_points: 0
        },
        {
          batch_name: '樟洋电厂-批次004',
          product_model: 'DCS-A4000',
          serial_number: 'ZY20241201004',
          station_name: '樟洋电厂',
          operator_name: '赵六',
          total_points: 24,
          tested_points: 0,
          passed_points: 0,
          failed_points: 0,
          skipped_points: 0
        }
      ];

      for (const batchData of testBatches) {
        // 创建TestBatchInfo对象
        const now = new Date().toISOString();
        const testBatch: TestBatchInfo = {
          batch_id: `batch_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`,
          batch_name: batchData.batch_name,
          product_model: batchData.product_model,
          serial_number: batchData.serial_number,
          station_name: batchData.station_name,
          operator_name: batchData.operator_name,
          creation_time: now,
          last_updated_time: now,
          total_points: batchData.total_points,
          tested_points: batchData.tested_points,
          passed_points: batchData.passed_points,
          failed_points: batchData.failed_points,
          skipped_points: batchData.skipped_points,
          overall_status: this.getStatusFromProgress(batchData.tested_points, batchData.total_points),
          status_summary: this.generateStatusSummary(batchData),
          created_at: now,
          updated_at: now
        };

        // 创建一些示例通道定义
        const definitions = this.generateSampleDefinitions(batchData.total_points, testBatch.batch_id, batchData.station_name);

        // 调用后端API保存数据
        try {
          console.log('🔧 准备调用后端API创建测试批次:', batchData.batch_name);
          console.log('🔧 批次信息:', testBatch);
          console.log('🔧 通道定义数量:', definitions.length);

          const result = await this.tauriApi.createTestBatchWithDefinitions(testBatch, definitions).toPromise();
          console.log(`✅ 成功创建测试批次: ${batchData.batch_name}, 结果:`, result);
        } catch (error) {
          console.error(`❌ 创建测试批次失败: ${batchData.batch_name}`, error);
          throw error; // 重新抛出错误以便外层catch处理
        }
      }

      console.log('测试数据创建完成');
      // 重新加载仪表盘数据
      await this.loadDashboardData();

    } catch (error) {
      console.error('创建测试数据失败:', error);
    }
  }

  private getStatusFromProgress(tested: number, total: number): OverallTestStatus {
    if (tested === 0) {
      return OverallTestStatus.NotTested;
    } else if (tested < total) {
      return OverallTestStatus.HardPointTesting;
    } else {
      return OverallTestStatus.TestCompletedPassed;
    }
  }

  private generateStatusSummary(batchData: any): string {
    if (batchData.tested_points === 0) {
      return '未开始测试';
    } else if (batchData.tested_points < batchData.total_points) {
      return `测试进行中 - ${batchData.tested_points}/${batchData.total_points}`;
    } else {
      const passRate = Math.round((batchData.passed_points / batchData.total_points) * 100);
      return `测试完成 - 通过率 ${passRate}%`;
    }
  }

  private generateSampleDefinitions(count: number, batchId: string, stationName: string): any[] {
    const definitions = [];
    for (let i = 1; i <= count; i++) {
      definitions.push({
        id: `def_${batchId}_${i}`,
        tag: `CH${i.toString().padStart(3, '0')}`,
        variable_name: `VAR_${i.toString().padStart(3, '0')}`,
        variable_description: `测试点位 ${i}`,
        module_type: i % 2 === 0 ? 'AI' : 'DI',
        plc_communication_address: `DB1.DBD${i * 4}`,
        station_name: stationName, // 使用传入的站场名称
        module_name: `模块${Math.floor((i - 1) / 8) + 1}`,
        channel_tag_in_module: `CH${i % 8}`,
        data_type: i % 2 === 0 ? 'Float' : 'Bool',
        power_supply_type: '有源',
        wire_system: i % 2 === 0 ? '4线制' : '2线制',
        test_batch_id: batchId
      });
    }
    return definitions;
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
