import { Component, OnInit, OnDestroy } from '@angular/core';
import { CommonModule } from '@angular/common';
import { FormsModule } from '@angular/forms';
import { Router } from '@angular/router';
import { Subscription } from 'rxjs';
import { TauriApiService } from '../../services/tauri-api.service';
import { TestBatchInfo, ChannelTestInstance, OverallTestStatus } from '../../models';

// 扩展的批次信息接口，包含统计数据
interface ExtendedBatchInfo extends TestBatchInfo {
  totalPoints: number;
  completedPoints: number;
  passedPoints: number;
  failedPoints: number;
  duration?: number; // 以分钟为单位
  progressPercentage: number;
  successRate: number;
}

@Component({
  selector: 'app-batch-management',
  standalone: true,
  imports: [CommonModule, FormsModule],
  templateUrl: './batch-management.component.html',
  styleUrl: './batch-management.component.css'
})
export class BatchManagementComponent implements OnInit, OnDestroy {
  // 统计数据
  totalBatches = 0;
  completedBatches = 0;
  inProgressBatches = 0;
  successRate = 0;

  // 筛选和搜索
  searchTerm = '';
  statusFilter = '';
  dateFilter = '';

  // 批次数据
  allBatches: ExtendedBatchInfo[] = [];
  filteredBatches: ExtendedBatchInfo[] = [];

  // 分页
  currentPage = 1;
  pageSize = 10;
  totalPages = 1;

  // 状态
  isLoading = false;
  loadingMessage = '';
  error: string | null = null;

  // 订阅管理
  private subscriptions: Subscription[] = [];

  constructor(
    private router: Router,
    private tauriApi: TauriApiService
  ) {}

  ngOnInit() {
    this.loadBatches();
  }

  ngOnDestroy() {
    this.subscriptions.forEach(sub => sub.unsubscribe());
  }

  async loadBatches() {
    this.isLoading = true;
    this.loadingMessage = '加载批次数据...';
    this.error = null;

    try {
      // 获取所有批次信息
      const batches = await this.getAllBatchInfo();
      
      // 为每个批次计算统计数据
      this.allBatches = await Promise.all(
        batches.map(batch => this.calculateBatchStatistics(batch))
      );

      this.updateStatistics();
      this.applyFilters();
    } catch (error) {
      this.error = '加载批次数据失败: ' + (error as Error).message;
      console.error('加载批次失败:', error);
    } finally {
      this.isLoading = false;
    }
  }

  // 获取所有批次信息
  private getAllBatchInfo(): Promise<TestBatchInfo[]> {
    return new Promise((resolve, reject) => {
      const subscription = this.tauriApi.getAllBatchInfo().subscribe({
        next: resolve,
        error: reject
      });
      this.subscriptions.push(subscription);
    });
  }

  // 计算批次统计数据
  private async calculateBatchStatistics(batch: TestBatchInfo): Promise<ExtendedBatchInfo> {
    try {
      // 获取批次的测试实例
      const instances = await this.getBatchTestInstances(batch.batch_id);
      
      const totalPoints = instances.length;
      const completedPoints = instances.filter(instance => 
        instance.overall_status !== OverallTestStatus.NotTested
      ).length;
      const passedPoints = instances.filter(instance => 
        instance.overall_status === OverallTestStatus.TestCompletedPassed
      ).length;
      const failedPoints = instances.filter(instance => 
        instance.overall_status === OverallTestStatus.TestCompletedFailed
      ).length;

      const progressPercentage = totalPoints > 0 ? Math.round((completedPoints / totalPoints) * 100) : 0;
      const successRate = completedPoints > 0 ? Math.round((passedPoints / completedPoints) * 100) : 0;

      // 计算持续时间（如果有开始和结束时间）
      let duration: number | undefined;
      if (batch.test_start_time && batch.test_end_time) {
        const start = new Date(batch.test_start_time);
        const end = new Date(batch.test_end_time);
        duration = Math.round((end.getTime() - start.getTime()) / (1000 * 60)); // 转换为分钟
      }

      return {
        ...batch,
        totalPoints,
        completedPoints,
        passedPoints,
        failedPoints,
        duration,
        progressPercentage,
        successRate
      };
    } catch (error) {
      console.error(`计算批次 ${batch.batch_id} 统计数据失败:`, error);
      // 返回默认值
      return {
        ...batch,
        totalPoints: 0,
        completedPoints: 0,
        passedPoints: 0,
        failedPoints: 0,
        progressPercentage: 0,
        successRate: 0
      };
    }
  }

  // 获取批次测试实例
  private getBatchTestInstances(batchId: string): Promise<ChannelTestInstance[]> {
    return new Promise((resolve, reject) => {
      const subscription = this.tauriApi.getBatchTestInstances(batchId).subscribe({
        next: resolve,
        error: reject
      });
      this.subscriptions.push(subscription);
    });
  }

  updateStatistics() {
    this.totalBatches = this.allBatches.length;
    this.completedBatches = this.allBatches.filter(b => 
      b.overall_status === OverallTestStatus.TestCompletedPassed || 
      b.overall_status === OverallTestStatus.TestCompletedFailed
    ).length;
    this.inProgressBatches = this.allBatches.filter(b => 
      b.overall_status === OverallTestStatus.HardPointTesting ||
      b.overall_status === OverallTestStatus.AlarmTesting
    ).length;
    
    const completedBatchesData = this.allBatches.filter(b => 
      b.overall_status === OverallTestStatus.TestCompletedPassed || 
      b.overall_status === OverallTestStatus.TestCompletedFailed
    );
    if (completedBatchesData.length > 0) {
      const totalTests = completedBatchesData.reduce((sum, b) => sum + b.totalPoints, 0);
      const passedTests = completedBatchesData.reduce((sum, b) => sum + b.passedPoints, 0);
      this.successRate = totalTests > 0 ? Math.round((passedTests / totalTests) * 100) : 0;
    } else {
      this.successRate = 0;
    }
  }

  applyFilters() {
    let filtered = [...this.allBatches];

    // 搜索筛选
    if (this.searchTerm) {
      const term = this.searchTerm.toLowerCase();
      filtered = filtered.filter(batch => 
        (batch.product_model && batch.product_model.toLowerCase().includes(term)) ||
        (batch.serial_number && batch.serial_number.toLowerCase().includes(term)) ||
        (batch.operator_name && batch.operator_name.toLowerCase().includes(term))
      );
    }

    // 状态筛选
    if (this.statusFilter) {
      filtered = filtered.filter(batch => batch.overall_status.toString() === this.statusFilter);
    }

    // 日期筛选
    if (this.dateFilter) {
      const now = new Date();
      const today = new Date(now.getFullYear(), now.getMonth(), now.getDate());
      
      filtered = filtered.filter(batch => {
        const batchDate = new Date(batch.created_at);
        switch (this.dateFilter) {
          case 'today':
            return batchDate >= today;
          case 'week':
            const weekAgo = new Date(today.getTime() - 7 * 24 * 60 * 60 * 1000);
            return batchDate >= weekAgo;
          case 'month':
            const monthAgo = new Date(today.getFullYear(), today.getMonth() - 1, today.getDate());
            return batchDate >= monthAgo;
          case 'quarter':
            const quarterAgo = new Date(today.getFullYear(), today.getMonth() - 3, today.getDate());
            return batchDate >= quarterAgo;
          default:
            return true;
        }
      });
    }

    this.filteredBatches = filtered;
    this.totalPages = Math.ceil(filtered.length / this.pageSize);
    this.currentPage = 1;
  }

  getStatusLabel(status: OverallTestStatus): string {
    const labels: { [key in OverallTestStatus]: string } = {
      [OverallTestStatus.NotTested]: '未测试',
      [OverallTestStatus.HardPointTesting]: '硬点测试中',
      [OverallTestStatus.AlarmTesting]: '报警测试中',
      [OverallTestStatus.TestCompletedPassed]: '测试完成并通过',
      [OverallTestStatus.TestCompletedFailed]: '测试完成并失败'
    };
    return labels[status] || status.toString();
  }

  getStatusClass(status: OverallTestStatus): string {
    const classes: { [key in OverallTestStatus]: string } = {
      [OverallTestStatus.NotTested]: 'status-not-tested',
      [OverallTestStatus.HardPointTesting]: 'status-in-progress',
      [OverallTestStatus.AlarmTesting]: 'status-in-progress',
      [OverallTestStatus.TestCompletedPassed]: 'status-completed',
      [OverallTestStatus.TestCompletedFailed]: 'status-failed'
    };
    return classes[status] || 'status-unknown';
  }

  formatDuration(minutes?: number): string {
    if (!minutes) return '未知';
    const hours = Math.floor(minutes / 60);
    const mins = minutes % 60;
    if (hours > 0) {
      return `${hours}小时${mins}分钟`;
    }
    return `${mins}分钟`;
  }

  getProgressPercentage(batch: ExtendedBatchInfo): number {
    return batch.progressPercentage;
  }

  // 导航方法
  navigateToDataImport() {
    this.router.navigate(['/data-import']);
  }

  // 刷新批次数据
  refreshBatches() {
    this.loadBatches();
  }

  // 导出批次数据
  exportBatches() {
    // TODO: 实现批次数据导出功能
    alert('批次数据导出功能开发中...');
  }

  // 查看批次详情
  viewBatchDetails(batch: ExtendedBatchInfo) {
    // 导航到测试执行页面，并传递批次ID
    this.router.navigate(['/test-execution'], { 
      queryParams: { batchId: batch.batch_id } 
    });
  }

  // 继续批次测试
  async continueBatch(batch: ExtendedBatchInfo) {
    if (batch.overall_status !== OverallTestStatus.HardPointTesting && 
        batch.overall_status !== OverallTestStatus.AlarmTesting) {
      alert('只能继续进行中的批次测试');
      return;
    }

    try {
      const subscription = this.tauriApi.resumeBatchTesting(batch.batch_id).subscribe({
        next: () => {
          alert('批次测试已恢复');
          this.refreshBatches();
        },
        error: (error: any) => {
          alert('恢复批次测试失败: ' + error.message);
        }
      });
      this.subscriptions.push(subscription);
    } catch (error) {
      alert('恢复批次测试失败: ' + (error as Error).message);
    }
  }

  // 导出单个批次
  exportBatch(batch: ExtendedBatchInfo) {
    // TODO: 实现单个批次导出功能
    alert(`导出批次 ${batch.serial_number} 功能开发中...`);
  }

  // 复制批次
  duplicateBatch(batch: ExtendedBatchInfo) {
    // 导航到数据导入页面，预填充批次信息
    this.router.navigate(['/data-import'], {
      queryParams: {
        duplicateFrom: batch.batch_id,
        productModel: batch.product_model
      }
    });
  }

  // 删除批次
  deleteBatch(batch: ExtendedBatchInfo) {
    if (batch.overall_status === OverallTestStatus.HardPointTesting || 
        batch.overall_status === OverallTestStatus.AlarmTesting) {
      alert('无法删除进行中的批次，请先停止测试');
      return;
    }

    if (confirm(`确定要删除批次 "${batch.serial_number}" 吗？此操作不可撤销。`)) {
      // TODO: 实现批次删除功能
      alert('批次删除功能开发中...');
    }
  }

  // 分页方法
  goToPage(page: number) {
    if (page >= 1 && page <= this.totalPages) {
      this.currentPage = page;
    }
  }

  get paginatedBatches(): ExtendedBatchInfo[] {
    const startIndex = (this.currentPage - 1) * this.pageSize;
    const endIndex = startIndex + this.pageSize;
    return this.filteredBatches.slice(startIndex, endIndex);
  }

  get pageNumbers(): number[] {
    const pages: number[] = [];
    const maxVisiblePages = 5;
    const halfVisible = Math.floor(maxVisiblePages / 2);
    
    let startPage = Math.max(1, this.currentPage - halfVisible);
    let endPage = Math.min(this.totalPages, startPage + maxVisiblePages - 1);
    
    if (endPage - startPage < maxVisiblePages - 1) {
      startPage = Math.max(1, endPage - maxVisiblePages + 1);
    }
    
    for (let i = startPage; i <= endPage; i++) {
      pages.push(i);
    }
    
    return pages;
  }

  // 清除错误
  clearError() {
    this.error = null;
  }
}
