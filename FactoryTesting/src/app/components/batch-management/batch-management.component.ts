import { Component, OnInit } from '@angular/core';
import { CommonModule } from '@angular/common';
import { FormsModule } from '@angular/forms';
import { Router } from '@angular/router';

interface BatchInfo {
  id: string;
  productModel: string;
  serialNumber: string;
  customerName?: string;
  operatorName?: string;
  status: 'completed' | 'in-progress' | 'failed' | 'cancelled';
  createdAt: Date;
  totalPoints: number;
  completedPoints: number;
  passedPoints: number;
  failedPoints: number;
  duration?: number; // 以分钟为单位
}

@Component({
  selector: 'app-batch-management',
  standalone: true,
  imports: [CommonModule, FormsModule],
  templateUrl: './batch-management.component.html',
  styleUrl: './batch-management.component.css'
})
export class BatchManagementComponent implements OnInit {
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
  allBatches: BatchInfo[] = [];
  filteredBatches: BatchInfo[] = [];

  // 分页
  currentPage = 1;
  pageSize = 10;
  totalPages = 1;

  // 状态
  isLoading = false;
  loadingMessage = '';

  constructor(private router: Router) {}

  ngOnInit() {
    this.loadBatches();
  }

  loadBatches() {
    this.isLoading = true;
    this.loadingMessage = '加载批次数据...';

    // 模拟数据
    setTimeout(() => {
      this.allBatches = [
        {
          id: '1',
          productModel: 'Model-A',
          serialNumber: 'SN001',
          customerName: '客户A',
          operatorName: '张三',
          status: 'completed',
          createdAt: new Date(Date.now() - 86400000),
          totalPoints: 50,
          completedPoints: 50,
          passedPoints: 48,
          failedPoints: 2,
          duration: 120
        },
        {
          id: '2',
          productModel: 'Model-B',
          serialNumber: 'SN002',
          customerName: '客户B',
          operatorName: '李四',
          status: 'in-progress',
          createdAt: new Date(Date.now() - 3600000),
          totalPoints: 75,
          completedPoints: 30,
          passedPoints: 28,
          failedPoints: 2,
          duration: 60
        },
        {
          id: '3',
          productModel: 'Model-C',
          serialNumber: 'SN003',
          customerName: '客户C',
          operatorName: '王五',
          status: 'failed',
          createdAt: new Date(Date.now() - 172800000),
          totalPoints: 40,
          completedPoints: 25,
          passedPoints: 20,
          failedPoints: 5,
          duration: 90
        }
      ];

      this.updateStatistics();
      this.applyFilters();
      this.isLoading = false;
    }, 1000);
  }

  updateStatistics() {
    this.totalBatches = this.allBatches.length;
    this.completedBatches = this.allBatches.filter(b => b.status === 'completed').length;
    this.inProgressBatches = this.allBatches.filter(b => b.status === 'in-progress').length;
    
    const completedBatchesData = this.allBatches.filter(b => b.status === 'completed');
    if (completedBatchesData.length > 0) {
      const totalTests = completedBatchesData.reduce((sum, b) => sum + b.totalPoints, 0);
      const passedTests = completedBatchesData.reduce((sum, b) => sum + b.passedPoints, 0);
      this.successRate = Math.round((passedTests / totalTests) * 100);
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
        batch.productModel.toLowerCase().includes(term) ||
        batch.serialNumber.toLowerCase().includes(term) ||
        (batch.customerName && batch.customerName.toLowerCase().includes(term))
      );
    }

    // 状态筛选
    if (this.statusFilter) {
      filtered = filtered.filter(batch => batch.status === this.statusFilter);
    }

    // 日期筛选
    if (this.dateFilter) {
      const now = new Date();
      const today = new Date(now.getFullYear(), now.getMonth(), now.getDate());
      
      filtered = filtered.filter(batch => {
        const batchDate = new Date(batch.createdAt);
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

  getStatusLabel(status: string): string {
    const labels: { [key: string]: string } = {
      'completed': '已完成',
      'in-progress': '进行中',
      'failed': '失败',
      'cancelled': '已取消'
    };
    return labels[status] || status;
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

  getProgressPercentage(batch: BatchInfo): number {
    return Math.round((batch.completedPoints / batch.totalPoints) * 100);
  }

  navigateToDataImport() {
    this.router.navigate(['/data-import']);
  }

  refreshBatches() {
    this.loadBatches();
  }

  exportBatches() {
    console.log('导出所有批次报告');
  }

  viewBatchDetails(batch: BatchInfo) {
    this.router.navigate(['/test-execution'], { queryParams: { batchId: batch.id } });
  }

  continueBatch(batch: BatchInfo) {
    this.router.navigate(['/test-execution'], { queryParams: { batchId: batch.id, continue: true } });
  }

  exportBatch(batch: BatchInfo) {
    console.log('导出批次报告:', batch.id);
  }

  duplicateBatch(batch: BatchInfo) {
    console.log('复制批次:', batch.id);
  }

  deleteBatch(batch: BatchInfo) {
    if (confirm(`确定要删除批次 "${batch.productModel} - ${batch.serialNumber}" 吗？`)) {
      this.allBatches = this.allBatches.filter(b => b.id !== batch.id);
      this.updateStatistics();
      this.applyFilters();
    }
  }

  goToPage(page: number) {
    if (page >= 1 && page <= this.totalPages) {
      this.currentPage = page;
    }
  }
}
