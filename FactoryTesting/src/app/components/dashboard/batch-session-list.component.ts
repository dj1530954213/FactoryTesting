import { Component, Input, Output, EventEmitter, OnInit, OnChanges } from '@angular/core';
import { CommonModule } from '@angular/common';
import { NzCollapseModule } from 'ng-zorro-antd/collapse';
import { NzListModule } from 'ng-zorro-antd/list';
import { NzButtonModule } from 'ng-zorro-antd/button';
import { NzIconModule } from 'ng-zorro-antd/icon';
import { NzSpaceModule } from 'ng-zorro-antd/space';
import { NzTagModule } from 'ng-zorro-antd/tag';
import { NzProgressModule } from 'ng-zorro-antd/progress';
import { NzCardModule } from 'ng-zorro-antd/card';
import { NzDividerModule } from 'ng-zorro-antd/divider';

import { OverallTestStatus } from '../../models';

interface DashboardBatchDisplay {
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
  // ... 已省略原始字段
}

interface ImportSessionGroup {
  sessionKey: string;
  timestamp: string;
  batches: DashboardBatchDisplay[];
  total_batches: number;
  stations: string[];
}

@Component({
  selector: 'app-batch-session-list',
  standalone: true,
  imports: [
    CommonModule,
    NzCollapseModule,
    NzListModule,
    NzButtonModule,
    NzIconModule,
    NzSpaceModule,
    NzTagModule,
    NzProgressModule,
    NzCardModule,
    NzDividerModule
  ],
  template: `
    <!-- 美化的批次会话列表 -->
    <div class="enhanced-batch-list">
      <!-- 美化的会话折叠面板 -->
      <nz-collapse nzAccordion [nzBordered]="false" class="session-collapse">
        <nz-collapse-panel
          *ngFor="let session of sessions"
          class="session-panel"
          [nzShowArrow]="true"
          [nzHeader]="'📅 ' + formatSessionTime(session.timestamp) + ' | 🏭 ' + session.stations.join(', ') + ' | 📦 ' + session.total_batches + '个批次'">
          
          <!-- 面板内容 -->
          <div class="session-content">
            <!-- 会话详细信息 -->
            <div class="session-detail-info">
              <div class="detail-row">
                <span class="detail-label">导入时间:</span>
                <span class="detail-value">{{ session.timestamp }}</span>
              </div>
              <div class="detail-row">
                <span class="detail-label">站场信息:</span>
                <span class="detail-value">{{ session.stations.join(', ') }}</span>
              </div>
              <div class="detail-row">
                <span class="detail-label">批次预览:</span>
                <span class="detail-value">
                  <nz-tag 
                    *ngFor="let batch of session.batches.slice(0, 3)" 
                    nzSize="small"
                    [nzColor]="batch.isCurrentSession ? 'processing' : 'default'">
                    {{ batch.name }}
                  </nz-tag>
                  <span *ngIf="session.batches.length > 3" style="color: #8c8c8c; font-style: italic;">
                    +{{ session.batches.length - 3 }} 个更多...
                  </span>
                </span>
              </div>
            </div>
            
            <!-- 操作按钮区域 -->
            <div class="session-actions">
              <button 
                nz-button 
                nzSize="small" 
                nzType="primary" 
                nzGhost 
                class="action-btn restore-btn"
                (click)="restoreSession.emit(session)">
                <span nz-icon nzType="rollback" nzTheme="outline"></span>
                恢复会话
              </button>
              <button 
                nz-button 
                nzSize="small" 
                nzType="primary" 
                nzDanger 
                class="action-btn delete-btn"
                (click)="deleteSession.emit(session)">
                <span nz-icon nzType="delete" nzTheme="outline"></span>
                删除会话
              </button>
            </div>

            <nz-divider nzDashed></nz-divider>

            <!-- 美化的批次卡片网格 -->
            <div class="batch-cards-grid">
              <nz-card 
                *ngFor="let batch of session.batches; trackBy: trackByBatch"
                nzHoverable
                [nzBodyStyle]="{ padding: '20px' }"
                class="batch-card"
                [class.current-session]="batch.isCurrentSession">
                
                <!-- 批次头部信息 -->
                <div class="batch-header">
                  <div class="batch-title-section">
                    <nz-tag 
                      [nzColor]="batch.isCurrentSession ? 'processing' : 'default'"
                      class="batch-name-tag">
                      <span nz-icon nzType="experiment" nzTheme="outline"></span>
                      {{ batch.name }}
                    </nz-tag>
                    <div class="batch-subtitle">
                      <span class="batch-time">
                        <span nz-icon nzType="calendar" nzTheme="outline"></span>
                        {{ batch.createdAt | date:'MM-dd HH:mm' }}
                      </span>
                    </div>
                  </div>
                </div>

                <!-- 批次统计信息 -->
                <div class="batch-stats">
                  <div class="stats-grid">
                    <div class="stat-item total">
                      <div class="stat-number">{{ batch.totalPoints }}</div>
                      <div class="stat-label">总点数</div>
                    </div>
                    
                    <div class="stat-item tested" *ngIf="batch.testedCount > 0">
                      <div class="stat-number">{{ batch.testedCount }}</div>
                      <div class="stat-label">已测试</div>
                    </div>
                    
                    <div class="stat-item success" *ngIf="batch.successCount > 0">
                      <div class="stat-number">{{ batch.successCount }}</div>
                      <div class="stat-label">成功</div>
                    </div>
                    
                    <div class="stat-item failure" *ngIf="batch.failureCount > 0">
                      <div class="stat-number">{{ batch.failureCount }}</div>
                      <div class="stat-label">失败</div>
                    </div>
                  </div>
                </div>

                <!-- 测试进度区域 -->
                <div class="progress-section">
                  <div class="progress-header">
                    <span class="progress-label">
                      <span nz-icon nzType="pie-chart" nzTheme="outline"></span>
                      测试进度
                    </span>
                    <span class="progress-percent">
                      {{ getProgressPercent(batch) }}%
                    </span>
                  </div>
                  <nz-progress
                    nzSize="small"
                    [nzPercent]="getProgressPercent(batch)"
                    [nzStrokeColor]="getProgressColor(batch)"
                    [nzShowInfo]="false"
                    [nzStrokeLinecap]="'round'"
                    class="progress-bar">
                  </nz-progress>
                </div>

                <!-- 通过率标签 -->
                <div class="success-rate-section">
                  <nz-tag 
                    [nzColor]="getSuccessRateColor(batch)" 
                    class="success-rate-tag">
                    <span nz-icon nzType="check-circle" nzTheme="outline"></span>
                    通过率: {{ getSuccessRate(batch) }}%
                  </nz-tag>
                </div>
              </nz-card>
            </div>
          </div>
        </nz-collapse-panel>
      </nz-collapse>
    </div>
  `,
  styles: [`
    /* 整体容器样式 */
    .enhanced-batch-list {
      background: linear-gradient(135deg, #f8fafc 0%, #f1f5f9 100%);
      border-radius: 12px;
      padding: 24px;
      box-shadow: 0 4px 12px rgba(0, 0, 0, 0.05);
    }

    /* 会话折叠面板容器 */
    .session-collapse {
      background: transparent;
    }

    .session-collapse ::ng-deep .ant-collapse {
      background: transparent;
      border: none;
    }

    /* 会话面板样式 */
    .session-panel {
      margin-bottom: 20px;
      border-radius: 12px;
      background: white;
      box-shadow: 0 2px 12px rgba(0, 0, 0, 0.08);
      overflow: hidden;
      transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
      border: 1px solid #e8e8e8;
    }

    .session-panel:hover {
      box-shadow: 0 4px 20px rgba(0, 0, 0, 0.12);
      transform: translateY(-2px);
    }

    .session-panel ::ng-deep .ant-collapse-header {
      padding: 20px 24px;
      background: linear-gradient(135deg, #ffffff 0%, #f8fafc 100%);
      border-bottom: 1px solid #f0f0f0;
    }

    .session-panel ::ng-deep .ant-collapse-content-box {
      padding: 0;
    }

    /* 会话头部样式 */
    .session-header {
      width: 100%;
    }

    .session-main-info {
      display: flex;
      flex-direction: column;
      gap: 16px;
    }

    /* 主要信息行 */
    .session-primary-row {
      display: flex;
      align-items: center;
      gap: 24px;
      flex-wrap: wrap;
    }

    .session-time-info {
      display: flex;
      align-items: center;
      gap: 8px;
      font-size: 16px;
      font-weight: 600;
      color: #262626;
      min-width: 200px;
    }

    .session-station-info {
      display: flex;
      align-items: center;
      gap: 8px;
      font-size: 15px;
      font-weight: 500;
      color: #1890ff;
      min-width: 150px;
    }

    .session-batch-info {
      display: flex;
      align-items: center;
    }

    .time-icon {
      color: #1890ff;
      font-size: 18px;
    }

    .station-icon {
      color: #52c41a;
      font-size: 16px;
    }

    .time-text {
      font-family: 'SF Pro Display', -apple-system, BlinkMacSystemFont, sans-serif;
    }

    .station-text {
      font-family: 'SF Pro Display', -apple-system, BlinkMacSystemFont, sans-serif;
      font-weight: 600;
    }

    /* 次要信息行 */
    .session-secondary-row {
      display: flex;
      align-items: center;
      padding-top: 8px;
      border-top: 1px solid #f0f0f0;
    }

    .batch-preview {
      display: flex;
      align-items: center;
      gap: 12px;
      width: 100%;
      flex-wrap: wrap;
    }

    .preview-label {
      font-size: 13px;
      color: #8c8c8c;
      font-weight: 500;
      min-width: 80px;
    }

    .batch-names {
      display: flex;
      align-items: center;
      gap: 8px;
      flex-wrap: wrap;
      flex: 1;
    }

    .preview-batch-tag {
      font-size: 11px;
      padding: 2px 8px;
      border-radius: 8px;
      font-weight: 500;
      max-width: 120px;
      overflow: hidden;
      text-overflow: ellipsis;
      white-space: nowrap;
    }

    .more-batches {
      font-size: 12px;
      color: #8c8c8c;
      font-style: italic;
      padding: 2px 8px;
      background: #f5f5f5;
      border-radius: 8px;
      border: 1px dashed #d9d9d9;
    }

    .batch-count-tag {
      font-weight: 500;
      font-size: 13px;
      padding: 4px 12px;
      border-radius: 16px;
      display: flex;
      align-items: center;
      gap: 6px;
    }

    /* 会话内容区域 */
    .session-content {
      padding: 24px;
    }

    /* 会话详细信息样式 */
    .session-detail-info {
      background: #f8fafc;
      border-radius: 8px;
      padding: 16px;
      margin-bottom: 20px;
      border: 1px solid #e8e8e8;
    }

    .detail-row {
      display: flex;
      align-items: center;
      margin-bottom: 12px;
      flex-wrap: wrap;
      gap: 8px;
    }

    .detail-row:last-child {
      margin-bottom: 0;
    }

    .detail-label {
      font-weight: 600;
      color: #595959;
      min-width: 80px;
      font-size: 13px;
    }

    .detail-value {
      color: #262626;
      flex: 1;
      font-size: 13px;
      display: flex;
      align-items: center;
      gap: 6px;
      flex-wrap: wrap;
    }

    /* 操作按钮区域 */
    .session-actions {
      display: flex;
      gap: 16px;
      margin-bottom: 20px;
      justify-content: flex-start;
    }

    .action-btn {
      height: 36px;
      padding: 0 16px;
      border-radius: 8px;
      font-weight: 500;
      display: flex;
      align-items: center;
      gap: 6px;
      transition: all 0.3s ease;
      box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
    }

    .restore-btn:hover {
      transform: translateY(-1px);
      box-shadow: 0 4px 8px rgba(24, 144, 255, 0.3);
    }

    .delete-btn:hover {
      transform: translateY(-1px);
      box-shadow: 0 4px 8px rgba(255, 77, 79, 0.3);
    }

    /* 批次卡片网格 */
    .batch-cards-grid {
      display: grid;
      grid-template-columns: repeat(auto-fill, minmax(320px, 1fr));
      gap: 20px;
      margin-top: 16px;
    }

    /* 批次卡片样式 */
    .batch-card {
      border: 1px solid #e8e8e8;
      border-radius: 12px;
      transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
      background: white;
      position: relative;
      overflow: hidden;
    }

    .batch-card::before {
      content: '';
      position: absolute;
      top: 0;
      left: 0;
      right: 0;
      height: 3px;
      background: linear-gradient(90deg, #e8e8e8 0%, #e8e8e8 100%);
      transition: all 0.3s ease;
    }

    .batch-card.current-session::before {
      background: linear-gradient(90deg, #1890ff 0%, #722ed1 100%);
    }

    .batch-card:hover {
      border-color: #1890ff;
      box-shadow: 0 6px 16px rgba(24, 144, 255, 0.15);
      transform: translateY(-3px);
    }

    .batch-card:hover::before {
      height: 4px;
    }

    /* 批次头部 */
    .batch-header {
      margin-bottom: 16px;
    }

    .batch-title-section {
      display: flex;
      flex-direction: column;
      gap: 8px;
    }

    .batch-name-tag {
      font-weight: 600;
      font-size: 14px;
      padding: 6px 12px;
      border-radius: 8px;
      display: flex;
      align-items: center;
      gap: 6px;
      align-self: flex-start;
    }

    .batch-subtitle {
      display: flex;
      align-items: center;
      gap: 8px;
    }

    .batch-time {
      color: #8c8c8c;
      font-size: 12px;
      display: flex;
      align-items: center;
      gap: 4px;
    }

    /* 批次统计信息 */
    .batch-stats {
      margin-bottom: 20px;
    }

    .stats-grid {
      display: grid;
      grid-template-columns: repeat(auto-fit, minmax(70px, 1fr));
      gap: 12px;
    }

    .stat-item {
      text-align: center;
      padding: 12px 8px;
      border-radius: 8px;
      background: #f8fafc;
      border: 1px solid #e8e8e8;
      transition: all 0.3s ease;
    }

    .stat-item:hover {
      transform: translateY(-2px);
      box-shadow: 0 4px 8px rgba(0, 0, 0, 0.1);
    }

    .stat-item.total {
      background: linear-gradient(135deg, #e6f7ff 0%, #f0f9ff 100%);
      border-color: #b3e0ff;
    }

    .stat-item.tested {
      background: linear-gradient(135deg, #f0f9ff 0%, #e6f7ff 100%);
      border-color: #91d5ff;
    }

    .stat-item.success {
      background: linear-gradient(135deg, #f6ffed 0%, #f0fff4 100%);
      border-color: #b7eb8f;
    }

    .stat-item.failure {
      background: linear-gradient(135deg, #fff2e8 0%, #fff1f0 100%);
      border-color: #ffccc7;
    }

    .stat-number {
      font-size: 18px;
      font-weight: 700;
      color: #262626;
      margin-bottom: 4px;
      font-family: 'SF Pro Display', -apple-system, BlinkMacSystemFont, sans-serif;
    }

    .stat-label {
      font-size: 11px;
      color: #8c8c8c;
      font-weight: 500;
      text-transform: uppercase;
      letter-spacing: 0.5px;
    }

    /* 测试进度区域 */
    .progress-section {
      margin-bottom: 16px;
      padding: 16px;
      background: #f8fafc;
      border-radius: 8px;
      border: 1px solid #e8e8e8;
    }

    .progress-header {
      display: flex;
      justify-content: space-between;
      align-items: center;
      margin-bottom: 12px;
    }

    .progress-label {
      font-size: 13px;
      font-weight: 500;
      color: #595959;
      display: flex;
      align-items: center;
      gap: 6px;
    }

    .progress-percent {
      font-size: 14px;
      font-weight: 700;
      color: #1890ff;
      font-family: 'SF Pro Display', -apple-system, BlinkMacSystemFont, sans-serif;
    }

    .progress-bar {
      margin: 0;
    }

    .progress-bar ::ng-deep .ant-progress-bg {
      transition: all 0.3s ease;
    }

    /* 通过率区域 */
    .success-rate-section {
      text-align: center;
    }

    .success-rate-tag {
      font-size: 12px;
      border-radius: 16px;
      padding: 6px 12px;
      font-weight: 600;
      display: inline-flex;
      align-items: center;
      gap: 4px;
      box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
    }

    /* 响应式设计 */
    @media (max-width: 1200px) {
      .batch-cards-grid {
        grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
        gap: 16px;
      }
    }

    @media (max-width: 768px) {
      .enhanced-batch-list {
        padding: 16px;
      }

      .batch-cards-grid {
        grid-template-columns: 1fr;
        gap: 16px;
      }
      
      .session-actions {
        flex-direction: column;
        gap: 12px;
      }

      .action-btn {
        width: 100%;
        justify-content: center;
      }
      
      .session-content {
        padding: 16px;
      }

      .session-panel ::ng-deep .ant-collapse-header {
        padding: 16px;
      }

      /* 移动端头部布局调整 */
      .session-primary-row {
        flex-direction: column;
        align-items: flex-start;
        gap: 12px;
      }

      .session-time-info,
      .session-station-info {
        min-width: auto;
        width: 100%;
      }

      .session-secondary-row {
        padding-top: 12px;
      }

      .batch-preview {
        flex-direction: column;
        align-items: flex-start;
        gap: 8px;
      }

      .preview-label {
        min-width: auto;
      }

      .batch-names {
        justify-content: flex-start;
      }

      .preview-batch-tag {
        max-width: 100px;
      }

      .stats-grid {
        grid-template-columns: repeat(2, 1fr);
      }
    }

    @media (max-width: 480px) {
      /* 超小屏幕进一步优化 */
      .session-time-info {
        font-size: 14px;
      }

      .session-station-info {
        font-size: 13px;
      }

      .batch-count-tag {
        font-size: 12px;
        padding: 3px 10px;
      }

      .preview-batch-tag {
        font-size: 10px;
        max-width: 80px;
      }

      .stats-grid {
        grid-template-columns: 1fr 1fr;
        gap: 8px;
      }

      .stat-item {
        padding: 8px 4px;
      }

      .stat-number {
        font-size: 16px;
      }

      .stat-label {
        font-size: 10px;
      }
    }

    /* 动画效果 */
    @keyframes slideIn {
      from {
        opacity: 0;
        transform: translateY(20px);
      }
      to {
        opacity: 1;
        transform: translateY(0);
      }
    }

    .batch-card {
      animation: slideIn 0.3s ease-out;
    }

    .session-panel {
      animation: slideIn 0.4s ease-out;
    }

    /* 当前会话特殊样式 */
    .batch-card.current-session {
      background: linear-gradient(135deg, #ffffff 0%, #f0f9ff 100%);
      border-color: #1890ff;
    }

    .batch-card.current-session .batch-name-tag {
      background: linear-gradient(135deg, #1890ff 0%, #722ed1 100%);
      color: white;
      box-shadow: 0 2px 8px rgba(24, 144, 255, 0.3);
    }

    /* 悬停效果增强 */
    .session-panel ::ng-deep .ant-collapse-header:hover {
      background: linear-gradient(135deg, #f8fafc 0%, #f0f9ff 100%);
    }

    /* 分割线样式 */
    .session-content ::ng-deep .ant-divider {
      margin: 16px 0;
      border-color: #e8e8e8;
    }

    .session-content ::ng-deep .ant-divider-dashed {
      border-style: dashed;
    }
  `]
})
export class BatchSessionListComponent implements OnInit, OnChanges {
  @Input() sessions: ImportSessionGroup[] = [];
  @Output() viewBatch = new EventEmitter<any>();
  @Output() deleteSession = new EventEmitter<any>();
  @Output() restoreSession = new EventEmitter<any>();

  ngOnInit() {
    // 组件初始化
  }

  ngOnChanges() {
    // 数据变更处理
  }

  formatSessionTime(timestamp: string): string {
    const date = new Date(timestamp);
    return date.toLocaleString('zh-CN', {
      month: '2-digit',
      day: '2-digit',
      hour: '2-digit',
      minute: '2-digit'
    });
  }

  getProgressPercent(batch: DashboardBatchDisplay): number {
    if (!batch.totalPoints) return 0;
    return Math.round((batch.testedCount / batch.totalPoints) * 100);
  }

  getProgressColor(batch: DashboardBatchDisplay): string {
    if (batch.failureCount > 0) return '#ff4d4f';
    if (batch.testedCount === batch.totalPoints) return '#52c41a';
    return '#1890ff';
  }

  getSuccessRate(batch: DashboardBatchDisplay): number {
    if (!batch.totalPoints) return 0;
    return Math.round(((batch.successCount || 0) / batch.totalPoints) * 100);
  }

  getSuccessRateColor(batch: DashboardBatchDisplay): string {
    const rate = this.getSuccessRate(batch);
    if (rate >= 95) return 'green';
    if (rate >= 80) return 'orange';
    if (rate > 0) return 'red';
    return 'default';
  }

  trackByBatch(index: number, batch: DashboardBatchDisplay): string {
    return batch.id;
  }
} 