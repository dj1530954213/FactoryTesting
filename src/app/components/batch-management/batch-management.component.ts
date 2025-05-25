import { Component, OnInit, OnDestroy } from '@angular/core';
import { CommonModule } from '@angular/common';
import { FormsModule, ReactiveFormsModule, FormBuilder, FormGroup, Validators } from '@angular/forms';
import { Subscription } from 'rxjs';
import { TauriApiService } from '../../services/tauri-api.service';
import { 
  TestBatchInfo, 
  ChannelPointDefinition,
  TestExecutionRequest,
  ApiError
} from '../../models';

@Component({
  selector: 'app-batch-management',
  standalone: true,
  imports: [CommonModule, FormsModule, ReactiveFormsModule],
  template: `
    <div class="batch-management-container">
      <!-- 页面标题 -->
      <div class="page-header">
        <div class="title-section">
          <h1 class="page-title">批次管理</h1>
          <p class="page-subtitle">创建和管理测试批次</p>
        </div>
        <div class="header-actions">
          <button class="btn-primary" (click)="showCreateForm()">
            <i class="icon-add">➕</i>
            创建批次
          </button>
          <button class="btn-secondary" (click)="refreshBatches()">
            <i class="icon-refresh">🔄</i>
            刷新
          </button>
        </div>
      </div>

      <!-- 统计卡片 -->
      <div class="stats-section">
        <div class="stat-card">
          <div class="stat-icon">📦</div>
          <div class="stat-content">
            <h3>总批次数</h3>
            <p class="stat-value">{{batches.length}}</p>
          </div>
        </div>
        
        <div class="stat-card">
          <div class="stat-icon">⏳</div>
          <div class="stat-content">
            <h3>进行中</h3>
            <p class="stat-value">{{getInProgressCount()}}</p>
          </div>
        </div>
        
        <div class="stat-card">
          <div class="stat-icon">✅</div>
          <div class="stat-content">
            <h3>已完成</h3>
            <p class="stat-value">{{getCompletedCount()}}</p>
          </div>
        </div>
        
        <div class="stat-card">
          <div class="stat-icon">📊</div>
          <div class="stat-content">
            <h3>总通道数</h3>
            <p class="stat-value">{{totalChannels}}</p>
          </div>
        </div>
      </div>

      <!-- 搜索和筛选 -->
      <div class="filter-section">
        <div class="search-box">
          <input 
            type="text" 
            placeholder="搜索批次ID、产品型号或序列号..." 
            [(ngModel)]="searchTerm"
            (input)="filterBatches()"
            class="search-input">
          <i class="search-icon">🔍</i>
        </div>
        
        <div class="filter-controls">
          <select [(ngModel)]="selectedStatus" (change)="filterBatches()" class="filter-select">
            <option value="">所有状态</option>
            <option value="in-progress">进行中</option>
            <option value="completed">已完成</option>
            <option value="pending">待开始</option>
          </select>
          
          <select [(ngModel)]="sortBy" (change)="sortBatches()" class="filter-select">
            <option value="created_at_desc">创建时间 (最新)</option>
            <option value="created_at_asc">创建时间 (最早)</option>
            <option value="product_model">产品型号</option>
            <option value="total_points">总点数</option>
          </select>
        </div>
      </div>

      <!-- 批次列表 -->
      <div class="batches-section">
        <div class="section-header">
          <h2>批次列表 ({{filteredBatches.length}})</h2>
        </div>

        <div *ngIf="filteredBatches.length > 0; else noBatches" class="batches-grid">
          <div *ngFor="let batch of filteredBatches" class="batch-card">
            <div class="card-header">
              <div class="batch-info">
                <h3 class="batch-title">{{batch.product_model || '未知产品'}}</h3>
                <span class="batch-id">{{batch.batch_id}}</span>
              </div>
              <div class="batch-status" [class]="getBatchStatusClass(batch)">
                {{getBatchStatusText(batch)}}
              </div>
            </div>
            
            <div class="card-content">
              <div class="info-grid">
                <div class="info-item">
                  <span class="label">序列号:</span>
                  <span class="value">{{batch.serial_number || 'N/A'}}</span>
                </div>
                <div class="info-item">
                  <span class="label">操作员:</span>
                  <span class="value">{{batch.operator_name || 'N/A'}}</span>
                </div>
                <div class="info-item">
                  <span class="label">总点数:</span>
                  <span class="value">{{batch.total_points}}</span>
                </div>
                <div class="info-item">
                  <span class="label">已测试:</span>
                  <span class="value">{{batch.tested_points}}</span>
                </div>
                <div class="info-item">
                  <span class="label">通过:</span>
                  <span class="value success">{{batch.passed_points}}</span>
                </div>
                <div class="info-item">
                  <span class="label">失败:</span>
                  <span class="value danger">{{batch.failed_points}}</span>
                </div>
              </div>
              
              <!-- 进度条 -->
              <div class="progress-section">
                <div class="progress-info">
                  <span class="progress-label">测试进度</span>
                  <span class="progress-text">{{getProgressText(batch)}}</span>
                </div>
                <div class="progress-bar">
                  <div class="progress-fill" [style.width.%]="getProgressPercentage(batch)"></div>
                </div>
              </div>
              
              <!-- 时间信息 -->
              <div class="time-info">
                <div class="time-item">
                  <span class="time-label">创建时间:</span>
                  <span class="time-value">{{formatDate(batch.created_at)}}</span>
                </div>
                <div class="time-item" *ngIf="batch.test_start_time">
                  <span class="time-label">开始时间:</span>
                  <span class="time-value">{{formatDate(batch.test_start_time)}}</span>
                </div>
                <div class="time-item" *ngIf="batch.test_end_time">
                  <span class="time-label">完成时间:</span>
                  <span class="time-value">{{formatDate(batch.test_end_time)}}</span>
                </div>
                <div class="time-item" *ngIf="batch.total_test_duration_ms">
                  <span class="time-label">耗时:</span>
                  <span class="time-value">{{formatDuration(batch.total_test_duration_ms)}}</span>
                </div>
              </div>
            </div>
            
            <div class="card-actions">
              <button class="btn-small" (click)="viewBatchDetails(batch)">查看详情</button>
              <button class="btn-small" (click)="startBatchTest(batch)" 
                      [disabled]="isBatchInProgress(batch)">
                {{isBatchInProgress(batch) ? '测试中' : '开始测试'}}
              </button>
              <button class="btn-small secondary" (click)="editBatch(batch)">编辑</button>
              <button class="btn-small danger" (click)="deleteBatch(batch)">删除</button>
            </div>
          </div>
        </div>

        <ng-template #noBatches>
          <div class="empty-state">
            <div class="empty-icon">📦</div>
            <h3>{{batches.length === 0 ? '暂无测试批次' : '未找到匹配的批次'}}</h3>
            <p>{{batches.length === 0 ? '创建您的第一个测试批次开始测试' : '尝试调整搜索条件或筛选器'}}</p>
            <button *ngIf="batches.length === 0" class="btn-primary" (click)="showCreateForm()">创建批次</button>
          </div>
        </ng-template>
      </div>

      <!-- 创建/编辑批次模态框 -->
      <div *ngIf="showForm" class="modal-overlay" (click)="hideForm()">
        <div class="modal-content" (click)="$event.stopPropagation()">
          <div class="modal-header">
            <h2>{{isEditing ? '编辑批次' : '创建测试批次'}}</h2>
            <button class="btn-close" (click)="hideForm()">×</button>
          </div>
          
          <form [formGroup]="batchForm" (ngSubmit)="saveBatch()" class="batch-form">
            <div class="form-sections">
              <!-- 基本信息 -->
              <div class="form-section">
                <h3>基本信息</h3>
                
                <div class="form-group">
                  <label for="product_model">产品型号 *</label>
                  <input 
                    id="product_model" 
                    type="text" 
                    formControlName="product_model" 
                    placeholder="例如: ProductV1.0"
                    class="form-input">
                  <div *ngIf="batchForm.get('product_model')?.invalid && batchForm.get('product_model')?.touched" class="error-message">
                    产品型号为必填项
                  </div>
                </div>

                <div class="form-group">
                  <label for="serial_number">序列号</label>
                  <input 
                    id="serial_number" 
                    type="text" 
                    formControlName="serial_number" 
                    placeholder="例如: SN123456"
                    class="form-input">
                </div>

                <div class="form-group">
                  <label for="operator_name">操作员</label>
                  <input 
                    id="operator_name" 
                    type="text" 
                    formControlName="operator_name" 
                    placeholder="例如: 张三"
                    class="form-input">
                </div>

                <div class="form-group">
                  <label for="notes">备注</label>
                  <textarea 
                    id="notes" 
                    formControlName="notes" 
                    placeholder="批次备注信息..."
                    class="form-textarea"
                    rows="3"></textarea>
                </div>
              </div>

              <!-- 通道选择 -->
              <div class="form-section">
                <h3>通道选择</h3>
                <p class="section-description">选择要包含在此批次中的测试通道</p>
                
                <div class="channel-selection">
                  <div class="selection-header">
                    <label class="checkbox-label">
                      <input 
                        type="checkbox" 
                        [checked]="allChannelsSelected()"
                        [indeterminate]="someChannelsSelected()"
                        (change)="toggleAllChannels($event)">
                      全选 ({{selectedChannels.length}}/{{availableChannels.length}})
                    </label>
                    
                    <div class="selection-filters">
                      <input 
                        type="text" 
                        placeholder="搜索通道..." 
                        [(ngModel)]="channelSearchTerm"
                        [ngModelOptions]="{standalone: true}"
                        (input)="filterAvailableChannels()"
                        class="channel-search">
                    </div>
                  </div>
                  
                  <div class="channels-list">
                    <div *ngFor="let channel of filteredAvailableChannels" class="channel-item">
                      <label class="checkbox-label">
                        <input 
                          type="checkbox" 
                          [checked]="isChannelSelected(channel)"
                          (change)="toggleChannel(channel, $event)">
                        <div class="channel-info">
                          <span class="channel-tag">{{channel.tag}}</span>
                          <span class="channel-desc">{{channel.variable_description}}</span>
                          <span class="channel-type">{{channel.module_type}}</span>
                        </div>
                      </label>
                    </div>
                  </div>
                </div>
              </div>
            </div>

            <div class="form-actions">
              <button type="button" class="btn-secondary" (click)="hideForm()">取消</button>
              <button type="submit" class="btn-primary" [disabled]="batchForm.invalid || saving || selectedChannels.length === 0">
                {{saving ? '保存中...' : (isEditing ? '更新批次' : '创建批次')}}
              </button>
            </div>
          </form>
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
  styleUrls: ['./batch-management.component.css']
})
export class BatchManagementComponent implements OnInit, OnDestroy {
  // 数据属性
  batches: TestBatchInfo[] = [];
  filteredBatches: TestBatchInfo[] = [];
  availableChannels: ChannelPointDefinition[] = [];
  filteredAvailableChannels: ChannelPointDefinition[] = [];
  selectedChannels: ChannelPointDefinition[] = [];
  totalChannels = 0;
  
  // 界面状态
  loading = false;
  saving = false;
  showForm = false;
  isEditing = false;
  
  // 搜索和筛选
  searchTerm = '';
  selectedStatus = '';
  sortBy = 'created_at_desc';
  channelSearchTerm = '';
  
  // 表单
  batchForm: FormGroup;
  editingBatch: TestBatchInfo | null = null;
  
  private subscriptions: Subscription[] = [];

  constructor(
    private tauriApi: TauriApiService,
    private fb: FormBuilder
  ) {
    this.batchForm = this.createForm();
  }

  ngOnInit(): void {
    this.loadData();
  }

  ngOnDestroy(): void {
    this.subscriptions.forEach(sub => sub.unsubscribe());
  }

  /**
   * 创建表单
   */
  private createForm(): FormGroup {
    return this.fb.group({
      product_model: ['', Validators.required],
      serial_number: [''],
      operator_name: [''],
      notes: ['']
    });
  }

  /**
   * 加载数据
   */
  loadData(): void {
    this.loading = true;
    
    // 并行加载批次和通道数据
    const batchSub = this.tauriApi.getAllBatchInfo().subscribe({
      next: (batches) => {
        this.batches = batches;
        this.filterBatches();
      },
      error: (error: ApiError) => {
        console.error('加载批次失败:', error);
      }
    });

    const channelSub = this.tauriApi.getAllChannelDefinitions().subscribe({
      next: (channels) => {
        this.availableChannels = channels;
        this.filteredAvailableChannels = [...channels];
        this.totalChannels = channels.length;
        this.loading = false;
      },
      error: (error: ApiError) => {
        console.error('加载通道失败:', error);
        this.loading = false;
      }
    });

    this.subscriptions.push(batchSub, channelSub);
  }

  /**
   * 刷新批次列表
   */
  refreshBatches(): void {
    this.loadData();
  }

  /**
   * 筛选批次
   */
  filterBatches(): void {
    let filtered = [...this.batches];

    // 搜索筛选
    if (this.searchTerm.trim()) {
      const term = this.searchTerm.toLowerCase();
      filtered = filtered.filter(batch => 
        batch.batch_id.toLowerCase().includes(term) ||
        (batch.product_model && batch.product_model.toLowerCase().includes(term)) ||
        (batch.serial_number && batch.serial_number.toLowerCase().includes(term))
      );
    }

    // 状态筛选
    if (this.selectedStatus) {
      filtered = filtered.filter(batch => {
        const status = this.getBatchStatus(batch);
        return status === this.selectedStatus;
      });
    }

    this.filteredBatches = filtered;
    this.sortBatches();
  }

  /**
   * 排序批次
   */
  sortBatches(): void {
    this.filteredBatches.sort((a, b) => {
      switch (this.sortBy) {
        case 'created_at_desc':
          return new Date(b.created_at).getTime() - new Date(a.created_at).getTime();
        case 'created_at_asc':
          return new Date(a.created_at).getTime() - new Date(b.created_at).getTime();
        case 'product_model':
          return (a.product_model || '').localeCompare(b.product_model || '');
        case 'total_points':
          return b.total_points - a.total_points;
        default:
          return 0;
      }
    });
  }

  /**
   * 筛选可用通道
   */
  filterAvailableChannels(): void {
    if (!this.channelSearchTerm.trim()) {
      this.filteredAvailableChannels = [...this.availableChannels];
      return;
    }

    const term = this.channelSearchTerm.toLowerCase();
    this.filteredAvailableChannels = this.availableChannels.filter(channel =>
      channel.tag.toLowerCase().includes(term) ||
      channel.variable_name.toLowerCase().includes(term) ||
      channel.variable_description.toLowerCase().includes(term)
    );
  }

  /**
   * 显示创建表单
   */
  showCreateForm(): void {
    this.isEditing = false;
    this.editingBatch = null;
    this.batchForm.reset();
    this.selectedChannels = [];
    this.channelSearchTerm = '';
    this.filterAvailableChannels();
    this.showForm = true;
  }

  /**
   * 编辑批次
   */
  editBatch(batch: TestBatchInfo): void {
    this.isEditing = true;
    this.editingBatch = batch;
    this.batchForm.patchValue(batch);
    // TODO: 加载批次关联的通道
    this.selectedChannels = [];
    this.showForm = true;
  }

  /**
   * 隐藏表单
   */
  hideForm(): void {
    this.showForm = false;
    this.isEditing = false;
    this.editingBatch = null;
    this.batchForm.reset();
    this.selectedChannels = [];
  }

  /**
   * 保存批次
   */
  saveBatch(): void {
    if (this.batchForm.invalid || this.selectedChannels.length === 0) return;

    this.saving = true;
    const formValue = this.batchForm.value;
    
    const batchData: TestBatchInfo = {
      batch_id: this.isEditing ? this.editingBatch!.batch_id : this.generateBatchId(),
      product_model: formValue.product_model,
      serial_number: formValue.serial_number,
      operator_name: formValue.operator_name,
      notes: formValue.notes,
      total_points: this.selectedChannels.length,
      tested_points: 0,
      passed_points: 0,
      failed_points: 0,
      skipped_points: 0,
      created_at: this.isEditing ? this.editingBatch!.created_at : new Date().toISOString(),
      updated_at: new Date().toISOString()
    };

    const sub = this.tauriApi.saveBatchInfo(batchData).subscribe({
      next: () => {
        this.saving = false;
        this.hideForm();
        this.loadData();
        // TODO: 显示成功提示
      },
      error: (error: ApiError) => {
        console.error('保存批次失败:', error);
        this.saving = false;
        // TODO: 显示错误提示
      }
    });
    this.subscriptions.push(sub);
  }

  /**
   * 删除批次
   */
  deleteBatch(batch: TestBatchInfo): void {
    if (!confirm(`确定要删除批次 "${batch.batch_id}" 吗？`)) return;

    // TODO: 实现删除批次API
    console.log('删除批次:', batch.batch_id);
  }

  /**
   * 开始批次测试
   */
  startBatchTest(batch: TestBatchInfo): void {
    if (this.isBatchInProgress(batch)) return;

    const request: TestExecutionRequest = {
      batch_info: batch,
      channel_definitions: [], // TODO: 获取批次关联的通道
      auto_start: true
    };

    const sub = this.tauriApi.submitTestExecution(request).subscribe({
      next: (response) => {
        console.log('测试提交成功:', response);
        this.loadData();
        // TODO: 显示成功提示并跳转到测试执行页面
      },
      error: (error: ApiError) => {
        console.error('提交测试失败:', error);
        // TODO: 显示错误提示
      }
    });
    this.subscriptions.push(sub);
  }

  /**
   * 查看批次详情
   */
  viewBatchDetails(batch: TestBatchInfo): void {
    // TODO: 跳转到批次详情页面
    console.log('查看批次详情:', batch.batch_id);
  }

  /**
   * 通道选择相关方法
   */
  allChannelsSelected(): boolean {
    return this.selectedChannels.length === this.availableChannels.length;
  }

  someChannelsSelected(): boolean {
    return this.selectedChannels.length > 0 && this.selectedChannels.length < this.availableChannels.length;
  }

  toggleAllChannels(event: any): void {
    if (event.target.checked) {
      this.selectedChannels = [...this.availableChannels];
    } else {
      this.selectedChannels = [];
    }
  }

  isChannelSelected(channel: ChannelPointDefinition): boolean {
    return this.selectedChannels.some(c => c.id === channel.id);
  }

  toggleChannel(channel: ChannelPointDefinition, event: any): void {
    if (event.target.checked) {
      this.selectedChannels.push(channel);
    } else {
      this.selectedChannels = this.selectedChannels.filter(c => c.id !== channel.id);
    }
  }

  /**
   * 工具方法
   */
  getBatchStatus(batch: TestBatchInfo): string {
    if (batch.test_end_time) return 'completed';
    if (batch.test_start_time) return 'in-progress';
    return 'pending';
  }

  getBatchStatusClass(batch: TestBatchInfo): string {
    return 'status-' + this.getBatchStatus(batch);
  }

  getBatchStatusText(batch: TestBatchInfo): string {
    const status = this.getBatchStatus(batch);
    switch (status) {
      case 'completed': return '已完成';
      case 'in-progress': return '进行中';
      case 'pending': return '待开始';
      default: return '未知';
    }
  }

  isBatchInProgress(batch: TestBatchInfo): boolean {
    return this.getBatchStatus(batch) === 'in-progress';
  }

  getInProgressCount(): number {
    return this.batches.filter(b => this.getBatchStatus(b) === 'in-progress').length;
  }

  getCompletedCount(): number {
    return this.batches.filter(b => this.getBatchStatus(b) === 'completed').length;
  }

  getProgressPercentage(batch: TestBatchInfo): number {
    if (batch.total_points === 0) return 0;
    return Math.round((batch.tested_points / batch.total_points) * 100);
  }

  getProgressText(batch: TestBatchInfo): string {
    return `${batch.tested_points}/${batch.total_points} (${this.getProgressPercentage(batch)}%)`;
  }

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

  formatDuration(ms: number): string {
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

  private generateBatchId(): string {
    const now = new Date();
    const dateStr = now.toISOString().slice(0, 10).replace(/-/g, '');
    const timeStr = now.toTimeString().slice(0, 8).replace(/:/g, '');
    return `BATCH_${dateStr}_${timeStr}`;
  }
} 