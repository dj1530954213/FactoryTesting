import { Component, Input, Output, EventEmitter } from '@angular/core';
import { CommonModule } from '@angular/common';
import { NzCollapseModule } from 'ng-zorro-antd/collapse';
import { NzListModule } from 'ng-zorro-antd/list';
import { NzButtonModule } from 'ng-zorro-antd/button';
import { NzIconModule } from 'ng-zorro-antd/icon';
import { NzSpaceModule } from 'ng-zorro-antd/space';
import { NzTagModule } from 'ng-zorro-antd/tag';
import { NzProgressModule } from 'ng-zorro-antd/progress';

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
    NzProgressModule
  ],
  template: `
    <nz-collapse nzAccordion>
      <nz-collapse-panel
        *ngFor="let session of sessions"
        [nzHeader]="'导入时间: ' + session.timestamp + ' | 站场: ' + session.stations.join(', ') + '  (批次 ' + session.total_batches + ' 个)'">
        <!-- 操作按钮 -->
        <div class="session-actions">
          <button nz-button nzSize="small" nzType="link" (click)="deleteSession.emit(session)">
            <span nz-icon nzType="delete"></span> 删除
          </button>
          <button nz-button nzSize="small" nzType="link" (click)="restoreSession.emit(session)">
            <span nz-icon nzType="rollback"></span> 恢复
          </button>
        </div>

        <!-- 批次列表 -->
        <nz-list [nzDataSource]="session.batches" nzItemLayout="horizontal">
          <nz-list-item *ngFor="let batch of session.batches">
            <nz-space nzAlign="center">
              <nz-tag [nzColor]="batch.isCurrentSession ? 'processing' : 'default'" style="min-width:80px;text-align:center">
                {{ batch.name }}
              </nz-tag>
              <span>{{ batch.createdAt | date:'MM-dd HH:mm' }}</span>
              <span>{{ batch.totalPoints }} 点</span>

              <!-- 测试进度条 -->
              <nz-progress
                nzSize="small"
                [nzPercent]="batch.totalPoints ? (batch.testedCount / batch.totalPoints) * 100 : 0"
                [nzStrokeColor]="batch.failureCount > 0 ? '#ff4d4f' : '#52c41a'"
                [nzStrokeLinecap]="'square'"
                style="width:120px">
              </nz-progress>

              <!-- 通过率 -->
              <nz-tag [nzColor]="(batch.successCount || 0) / ((batch.successCount || 0) + (batch.failureCount || 0) || 1) >= 0.9 ? 'green' : (batch.failureCount || 0) > 0 ? 'red' : 'default'">
                {{ (batch.totalPoints ? ((batch.successCount || 0) / batch.totalPoints * 100) : 0) | number:'1.0-0' }}%
              </nz-tag>

              <!-- "查看"按钮已移除，根据业务要求不再显示批次详情按钮 -->
            </nz-space>
          </nz-list-item>
        </nz-list>
      </nz-collapse-panel>
    </nz-collapse>
  `,
  styles: [`
    .session-actions {
      text-align: right;
      margin-bottom: 8px;
    }
    nz-list-item {
      padding: 4px 0;
    }
    nz-space {
      width: 100%;
      display: flex;
      justify-content: space-between;
      align-items: center;
    }
  `]
})
export class BatchSessionListComponent {
  @Input() sessions: ImportSessionGroup[] = [];
  @Output() viewBatch = new EventEmitter<any>();
  @Output() deleteSession = new EventEmitter<any>();
  @Output() restoreSession = new EventEmitter<any>();
} 