import { Component, OnInit, OnDestroy } from '@angular/core';
import { CommonModule } from '@angular/common';
import { FormsModule } from '@angular/forms';
import { Router } from '@angular/router';
import { Subscription } from 'rxjs';
import { TauriApiService } from '../../services/tauri-api.service';
import {
  TestBatchInfo,
  TestReport,
  ReportTemplate,
  ReportGenerationRequest
} from '../../models';

interface ReportGenerationForm {
  batchId: string;
  templateId: string;
  format: 'PDF' | 'Excel';
  includeCharts: boolean;
  includeDetails: boolean;
  customTitle?: string;
  customDescription?: string;
}

@Component({
  selector: 'app-report-generation',
  standalone: true,
  imports: [CommonModule, FormsModule],
  template: `
    <div class="report-generation-container">
      <!-- 页面标题 -->
      <div class="page-header">
        <h1>报告生成</h1>
        <div class="header-actions">
          <button class="btn btn-secondary" (click)="goToDashboard()">
            返回仪表板
          </button>
        </div>
      </div>

      <!-- 主要内容区域 -->
      <div class="main-content">
        <!-- 左侧：报告生成表单 -->
        <div class="generation-panel">
          <div class="panel-header">
            <h2>生成新报告</h2>
          </div>

          <form class="generation-form" #reportForm="ngForm">
            <!-- 批次选择 -->
            <div class="form-group">
              <label for="batchSelect">选择测试批次 *</label>
              <select 
                id="batchSelect"
                name="batchId"
                class="form-control"
                [(ngModel)]="form.batchId"
                required>
                <option value="">请选择批次</option>
                <option *ngFor="let batch of availableBatches" [value]="batch.batch_id">
                  {{ batch.product_model }} - {{ batch.serial_number }} ({{ batch.created_at | date:'short' }})
                </option>
              </select>
            </div>

            <!-- 模板选择 -->
            <div class="form-group">
              <label for="templateSelect">选择报告模板 *</label>
              <select 
                id="templateSelect"
                name="templateId"
                class="form-control"
                [(ngModel)]="form.templateId"
                required>
                <option value="">请选择模板</option>
                <option *ngFor="let template of availableTemplates" [value]="template.id">
                  {{ template.name }} - {{ template.description }}
                </option>
              </select>
            </div>

            <!-- 报告格式 -->
            <div class="form-group">
              <label>报告格式 *</label>
              <div class="radio-group">
                <label class="radio-label">
                  <input type="radio" name="format" value="PDF" [(ngModel)]="form.format">
                  <span>PDF格式</span>
                </label>
                <label class="radio-label">
                  <input type="radio" name="format" value="Excel" [(ngModel)]="form.format">
                  <span>Excel格式</span>
                </label>
              </div>
            </div>

            <!-- 报告选项 -->
            <div class="form-group">
              <label>报告选项</label>
              <div class="checkbox-group">
                <label class="checkbox-label">
                  <input type="checkbox" [(ngModel)]="form.includeCharts" name="includeCharts">
                  <span>包含图表</span>
                </label>
                <label class="checkbox-label">
                  <input type="checkbox" [(ngModel)]="form.includeDetails" name="includeDetails">
                  <span>包含详细数据</span>
                </label>
              </div>
            </div>

            <!-- 自定义标题 -->
            <div class="form-group">
              <label for="customTitle">自定义标题</label>
              <input 
                type="text"
                id="customTitle"
                name="customTitle"
                class="form-control"
                [(ngModel)]="form.customTitle"
                placeholder="可选：自定义报告标题">
            </div>

            <!-- 自定义描述 -->
            <div class="form-group">
              <label for="customDescription">自定义描述</label>
              <textarea 
                id="customDescription"
                name="customDescription"
                class="form-control"
                rows="3"
                [(ngModel)]="form.customDescription"
                placeholder="可选：自定义报告描述"></textarea>
            </div>

            <!-- 生成按钮 -->
            <div class="form-actions">
              <button 
                type="button"
                class="btn btn-primary"
                [disabled]="!reportForm.valid || isGenerating"
                (click)="generateReport()">
                <span *ngIf="isGenerating">生成中...</span>
                <span *ngIf="!isGenerating">生成报告</span>
              </button>
              <button 
                type="button"
                class="btn btn-secondary"
                (click)="resetForm()">
                重置表单
              </button>
            </div>
          </form>

          <!-- 生成进度 -->
          <div class="generation-progress" *ngIf="isGenerating">
            <div class="progress-bar">
              <div class="progress-fill" [style.width.%]="generationProgress"></div>
            </div>
            <p class="progress-text">{{ progressMessage }}</p>
          </div>

          <!-- 生成结果 -->
          <div class="generation-result" *ngIf="generationResult">
            <div class="result-header" [class]="generationResult.success ? 'success' : 'error'">
              <i class="result-icon" [class]="generationResult.success ? 'icon-success' : 'icon-error'"></i>
              <span>{{ generationResult.success ? '生成成功' : '生成失败' }}</span>
            </div>
            <div class="result-content">
              <p>{{ generationResult.message }}</p>
              <div class="result-actions" *ngIf="generationResult.success && generatedReport">
                <button class="btn btn-primary" (click)="downloadReport()">
                  下载报告
                </button>
                <button class="btn btn-outline" (click)="viewReport()">
                  查看报告
                </button>
              </div>
            </div>
          </div>
        </div>

        <!-- 右侧：报告历史 -->
        <div class="history-panel">
          <div class="panel-header">
            <h2>报告历史</h2>
            <button class="btn btn-sm btn-outline" (click)="loadReportHistory()" [disabled]="isLoading">
              刷新
            </button>
          </div>

          <!-- 历史报告列表 -->
          <div class="report-list" *ngIf="!isLoading">
            <div 
              class="report-item" 
              *ngFor="let report of reportHistory"
              [class.selected]="selectedReport?.id === report.id"
              (click)="selectReport(report)">
              <div class="report-header">
                <span class="report-title">{{ report.title }}</span>
                <span class="report-format">{{ report.format }}</span>
              </div>
              <div class="report-info">
                <div class="report-batch">批次: {{ report.batch_id }}</div>
                <div class="report-date">{{ report.created_at | date:'medium' }}</div>
                <div class="report-size">{{ formatFileSize(report.file_size || 0) }}</div>
              </div>
              <div class="report-actions">
                <button class="btn btn-sm btn-primary" (click)="downloadHistoryReport(report); $event.stopPropagation()">
                  下载
                </button>
                <button class="btn btn-sm btn-danger" (click)="deleteReport(report); $event.stopPropagation()">
                  删除
                </button>
              </div>
            </div>
          </div>

          <!-- 加载状态 -->
          <div class="loading-state" *ngIf="isLoading">
            <div class="spinner"></div>
            <p>正在加载报告历史...</p>
          </div>

          <!-- 空状态 -->
          <div class="empty-state" *ngIf="!isLoading && reportHistory.length === 0">
            <div class="empty-icon">📄</div>
            <p>暂无历史报告</p>
          </div>
        </div>
      </div>

      <!-- 错误提示 -->
      <div class="error-message" *ngIf="error">
        <div class="error-content">
          <i class="error-icon">⚠️</i>
          <span>{{ error }}</span>
          <button class="btn btn-sm btn-outline" (click)="clearError()">关闭</button>
        </div>
      </div>
    </div>
  `,
  styles: [`
    .report-generation-container {
      padding: 20px;
      max-width: 1400px;
      margin: 0 auto;
    }

    .page-header {
      display: flex;
      justify-content: space-between;
      align-items: center;
      margin-bottom: 30px;
      padding-bottom: 20px;
      border-bottom: 2px solid #f0f0f0;
    }

    .page-header h1 {
      margin: 0;
      color: #333;
      font-size: 28px;
      font-weight: 600;
    }

    .main-content {
      display: grid;
      grid-template-columns: 1fr 1fr;
      gap: 30px;
    }

    .generation-panel, .history-panel {
      background: #fff;
      border-radius: 8px;
      box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
      overflow: hidden;
    }

    .panel-header {
      display: flex;
      justify-content: space-between;
      align-items: center;
      padding: 20px;
      background: #f8f9fa;
      border-bottom: 1px solid #e9ecef;
    }

    .panel-header h2 {
      margin: 0;
      font-size: 18px;
      font-weight: 600;
      color: #333;
    }

    .generation-form {
      padding: 20px;
    }

    .form-group {
      margin-bottom: 20px;
    }

    .form-group label {
      display: block;
      margin-bottom: 6px;
      font-weight: 500;
      color: #333;
      font-size: 14px;
    }

    .form-control {
      width: 100%;
      padding: 8px 12px;
      border: 1px solid #d9d9d9;
      border-radius: 6px;
      font-size: 14px;
      transition: border-color 0.3s, box-shadow 0.3s;
      box-sizing: border-box;
    }

    .form-control:focus {
      outline: none;
      border-color: #1890ff;
      box-shadow: 0 0 0 2px rgba(24, 144, 255, 0.2);
    }

    .radio-group, .checkbox-group {
      display: flex;
      flex-direction: column;
      gap: 8px;
    }

    .radio-label, .checkbox-label {
      display: flex;
      align-items: center;
      gap: 8px;
      cursor: pointer;
      font-size: 14px;
    }

    .form-actions {
      display: flex;
      gap: 12px;
      margin-top: 30px;
    }

    .btn {
      padding: 8px 16px;
      border: 1px solid transparent;
      border-radius: 6px;
      font-size: 14px;
      font-weight: 500;
      cursor: pointer;
      transition: all 0.3s;
    }

    .btn:disabled {
      opacity: 0.6;
      cursor: not-allowed;
    }

    .btn-primary {
      background-color: #1890ff;
      border-color: #1890ff;
      color: white;
    }

    .btn-secondary {
      background-color: #f5f5f5;
      border-color: #d9d9d9;
      color: #333;
    }

    .btn-outline {
      background-color: transparent;
      border-color: #d9d9d9;
      color: #333;
    }

    .btn-danger {
      background-color: #ff4d4f;
      border-color: #ff4d4f;
      color: white;
    }

    .btn-sm {
      padding: 4px 8px;
      font-size: 12px;
    }

    .generation-progress {
      margin-top: 20px;
      padding: 20px;
      background: #f8f9fa;
      border-radius: 6px;
    }

    .progress-bar {
      width: 100%;
      height: 8px;
      background: #e9ecef;
      border-radius: 4px;
      overflow: hidden;
      margin-bottom: 10px;
    }

    .progress-fill {
      height: 100%;
      background: #1890ff;
      transition: width 0.3s ease;
    }

    .generation-result {
      margin-top: 20px;
      border-radius: 6px;
      overflow: hidden;
    }

    .result-header {
      display: flex;
      align-items: center;
      gap: 8px;
      padding: 12px 16px;
      font-weight: 500;
    }

    .result-header.success {
      background: #f6ffed;
      color: #389e0d;
      border: 1px solid #b7eb8f;
    }

    .result-header.error {
      background: #fff2f0;
      color: #a8071a;
      border: 1px solid #ffccc7;
    }

    .result-content {
      padding: 16px;
      background: white;
      border: 1px solid #e9ecef;
      border-top: none;
    }

    .result-actions {
      display: flex;
      gap: 8px;
      margin-top: 12px;
    }

    .report-list {
      max-height: 600px;
      overflow-y: auto;
    }

    .report-item {
      padding: 16px 20px;
      border-bottom: 1px solid #e9ecef;
      cursor: pointer;
      transition: background-color 0.2s;
    }

    .report-item:hover {
      background: #f8f9fa;
    }

    .report-item.selected {
      background: #e6f7ff;
      border-color: #91d5ff;
    }

    .report-header {
      display: flex;
      justify-content: space-between;
      align-items: center;
      margin-bottom: 8px;
    }

    .report-title {
      font-weight: 500;
      color: #333;
    }

    .report-format {
      padding: 2px 8px;
      background: #f0f0f0;
      border-radius: 4px;
      font-size: 12px;
      color: #666;
    }

    .report-info {
      font-size: 12px;
      color: #666;
      margin-bottom: 8px;
    }

    .report-actions {
      display: flex;
      gap: 8px;
    }

    .loading-state, .empty-state {
      text-align: center;
      padding: 40px 20px;
      color: #666;
    }

    .spinner {
      width: 32px;
      height: 32px;
      border: 3px solid #f3f3f3;
      border-top: 3px solid #1890ff;
      border-radius: 50%;
      animation: spin 1s linear infinite;
      margin: 0 auto 16px;
    }

    @keyframes spin {
      0% { transform: rotate(0deg); }
      100% { transform: rotate(360deg); }
    }

    .error-message {
      position: fixed;
      top: 20px;
      right: 20px;
      z-index: 1000;
    }

    .error-content {
      display: flex;
      align-items: center;
      gap: 12px;
      padding: 12px 16px;
      background: #fff2f0;
      border: 1px solid #ffccc7;
      border-radius: 6px;
      color: #a8071a;
      box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
    }

    @media (max-width: 768px) {
      .main-content {
        grid-template-columns: 1fr;
      }
    }
  `]
})
export class ReportGenerationComponent implements OnInit, OnDestroy {
  // 数据状态
  availableBatches: TestBatchInfo[] = [];
  availableTemplates: ReportTemplate[] = [];
  reportHistory: TestReport[] = [];
  selectedReport: TestReport | null = null;
  generatedReport: TestReport | null = null;

  // 表单数据
  form: ReportGenerationForm = {
    batchId: '',
    templateId: '',
    format: 'PDF',
    includeCharts: true,
    includeDetails: true,
    customTitle: '',
    customDescription: ''
  };

  // 界面状态
  isLoading = false;
  isGenerating = false;
  generationProgress = 0;
  progressMessage = '';
  error: string | null = null;
  generationResult: { success: boolean; message: string } | null = null;

  // 订阅管理
  private subscriptions: Subscription[] = [];

  constructor(
    private router: Router,
    private tauriApi: TauriApiService
  ) {}

  ngOnInit() {
    this.loadInitialData();
  }

  ngOnDestroy() {
    this.subscriptions.forEach(sub => sub.unsubscribe());
  }

  // 加载初始数据
  async loadInitialData() {
    this.isLoading = true;
    this.error = null;

    try {
      // 并行加载所有数据
      await Promise.all([
        this.loadAvailableBatches(),
        this.loadAvailableTemplates(),
        this.loadReportHistory()
      ]);
    } catch (error) {
      this.error = '加载数据失败: ' + (error as Error).message;
    } finally {
      this.isLoading = false;
    }
  }

  // 加载可用批次
  private async loadAvailableBatches() {
    return new Promise<void>((resolve, reject) => {
      const subscription = this.tauriApi.getAllBatchInfo().subscribe({
        next: (batches) => {
          this.availableBatches = batches;
          resolve();
        },
        error: reject
      });
      this.subscriptions.push(subscription);
    });
  }

  // 加载可用模板
  private async loadAvailableTemplates() {
    // 模拟模板数据，实际应该从后端获取
    this.availableTemplates = [
      {
        id: 'standard',
        name: '标准测试报告',
        description: '包含基本测试结果和统计信息',
        content: '',
        created_at: new Date().toISOString(),
        updated_at: new Date().toISOString()
      },
      {
        id: 'detailed',
        name: '详细测试报告',
        description: '包含完整的测试数据和图表分析',
        content: '',
        created_at: new Date().toISOString(),
        updated_at: new Date().toISOString()
      },
      {
        id: 'summary',
        name: '测试摘要报告',
        description: '仅包含测试结果摘要',
        content: '',
        created_at: new Date().toISOString(),
        updated_at: new Date().toISOString()
      }
    ];
  }

  // 加载报告历史
  loadReportHistory() {
    this.isLoading = true;
    
    // 模拟报告历史数据，实际应该从后端获取
    setTimeout(() => {
      this.reportHistory = [
        {
          id: 'report_1',
          title: '测试报告 - ProductV1.0',
          batch_id: 'batch_001',
          template_id: 'standard',
          format: 'PDF',
          file_path: '/reports/report_1.pdf',
          file_size: 1024000,
          generated_by: 'system',
          created_at: new Date(Date.now() - 86400000).toISOString(),
          updated_at: new Date(Date.now() - 86400000).toISOString()
        },
        {
          id: 'report_2',
          title: '详细测试报告 - ProductV2.0',
          batch_id: 'batch_002',
          template_id: 'detailed',
          format: 'Excel',
          file_path: '/reports/report_2.xlsx',
          file_size: 2048000,
          generated_by: 'system',
          created_at: new Date(Date.now() - 172800000).toISOString(),
          updated_at: new Date(Date.now() - 172800000).toISOString()
        }
      ];
      this.isLoading = false;
    }, 1000);
  }

  // 生成报告
  async generateReport() {
    if (!this.validateForm()) {
      return;
    }

    this.isGenerating = true;
    this.generationProgress = 0;
    this.progressMessage = '准备生成报告...';
    this.generationResult = null;

    try {
      // 创建报告生成请求
      const request: ReportGenerationRequest = {
        batch_id: this.form.batchId,
        template_id: this.form.templateId,
        format: this.form.format,
        options: {
          include_charts: this.form.includeCharts,
          include_details: this.form.includeDetails,
          custom_title: this.form.customTitle,
          custom_description: this.form.customDescription
        }
      };

      // 模拟生成进度
      await this.simulateGenerationProgress();

      // 调用后端生成报告
      const report = await this.callGenerateReportAPI(request);
      
      this.generatedReport = report;
      this.generationResult = {
        success: true,
        message: `报告生成成功！文件大小: ${this.formatFileSize(report.file_size || 0)}`
      };

      // 刷新报告历史
      this.loadReportHistory();

    } catch (error) {
      this.generationResult = {
        success: false,
        message: '报告生成失败: ' + (error as Error).message
      };
    } finally {
      this.isGenerating = false;
      this.generationProgress = 100;
    }
  }

  // 模拟生成进度
  private async simulateGenerationProgress() {
    const steps = [
      { progress: 20, message: '加载测试数据...' },
      { progress: 40, message: '处理测试结果...' },
      { progress: 60, message: '生成图表...' },
      { progress: 80, message: '格式化报告...' },
      { progress: 100, message: '完成生成...' }
    ];

    for (const step of steps) {
      await new Promise(resolve => setTimeout(resolve, 500));
      this.generationProgress = step.progress;
      this.progressMessage = step.message;
    }
  }

  // 调用后端生成报告API
  private async callGenerateReportAPI(request: ReportGenerationRequest): Promise<TestReport> {
    return new Promise((resolve, reject) => {
      const apiCall = this.form.format === 'PDF' 
        ? this.tauriApi.generatePdfReport(request)
        : this.tauriApi.generateExcelReport(request);

      const subscription = apiCall.subscribe({
        next: resolve,
        error: reject
      });
      this.subscriptions.push(subscription);
    });
  }

  // 验证表单
  private validateForm(): boolean {
    if (!this.form.batchId) {
      this.error = '请选择测试批次';
      return false;
    }
    if (!this.form.templateId) {
      this.error = '请选择报告模板';
      return false;
    }
    if (!this.form.format) {
      this.error = '请选择报告格式';
      return false;
    }
    return true;
  }

  // 重置表单
  resetForm() {
    this.form = {
      batchId: '',
      templateId: '',
      format: 'PDF',
      includeCharts: true,
      includeDetails: true,
      customTitle: '',
      customDescription: ''
    };
    this.generationResult = null;
    this.generatedReport = null;
  }

  // 下载报告
  downloadReport() {
    if (this.generatedReport) {
      // 实际应该调用Tauri的文件下载API
      console.log('下载报告:', this.generatedReport.file_path);
      alert('报告下载功能需要集成Tauri文件API');
    }
  }

  // 查看报告
  viewReport() {
    if (this.generatedReport) {
      // 实际应该调用Tauri的文件打开API
      console.log('查看报告:', this.generatedReport.file_path);
      alert('报告查看功能需要集成Tauri文件API');
    }
  }

  // 选择历史报告
  selectReport(report: TestReport) {
    this.selectedReport = report;
  }

  // 下载历史报告
  downloadHistoryReport(report: TestReport) {
    console.log('下载历史报告:', report.file_path);
    alert('报告下载功能需要集成Tauri文件API');
  }

  // 删除报告
  deleteReport(report: TestReport) {
    if (confirm(`确定要删除报告 "${report.title}" 吗？`)) {
      const subscription = this.tauriApi.deleteReport(report.id).subscribe({
        next: () => {
          this.reportHistory = this.reportHistory.filter(r => r.id !== report.id);
          if (this.selectedReport?.id === report.id) {
            this.selectedReport = null;
          }
        },
        error: (error: any) => {
          this.error = '删除报告失败: ' + error.message;
        }
      });
      this.subscriptions.push(subscription);
    }
  }

  // 清除错误
  clearError() {
    this.error = null;
  }

  // 格式化文件大小
  formatFileSize(bytes: number): string {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
  }

  // 导航方法
  goToDashboard() {
    this.router.navigate(['/dashboard']);
  }
} 