import { Component, OnInit } from '@angular/core';
import { CommonModule } from '@angular/common';
import { RouterModule } from '@angular/router';

// NG-ZORRO 组件导入
import { NzCardModule } from 'ng-zorro-antd/card';
import { NzButtonModule } from 'ng-zorro-antd/button';
import { NzIconModule } from 'ng-zorro-antd/icon';
import { NzSpaceModule } from 'ng-zorro-antd/space';
import { NzStepsModule } from 'ng-zorro-antd/steps';
import { NzTabsModule } from 'ng-zorro-antd/tabs';
import { NzTableModule } from 'ng-zorro-antd/table';
import { NzTagModule } from 'ng-zorro-antd/tag';
import { NzProgressModule } from 'ng-zorro-antd/progress';
import { NzAlertModule } from 'ng-zorro-antd/alert';
import { NzListModule } from 'ng-zorro-antd/list';
import { NzEmptyModule } from 'ng-zorro-antd/empty';
import { NzMessageService } from 'ng-zorro-antd/message';

// 服务导入
import { TauriApiService } from '../../services/tauri-api.service';

@Component({
  selector: 'app-test-area',
  standalone: true,
  imports: [
    CommonModule,
    RouterModule,
    NzCardModule,
    NzButtonModule,
    NzIconModule,
    NzSpaceModule,
    NzStepsModule,
    NzTabsModule,
    NzTableModule,
    NzTagModule,
    NzProgressModule,
    NzAlertModule,
    NzListModule,
    NzEmptyModule
  ],
  template: `
    <div class="test-area-container">
      <div class="page-header">
        <h2>
          <span nz-icon nzType="experiment" nzTheme="outline"></span>
          测试区域
        </h2>
        <p>执行测试工作流程：选择批次 → 确认接线 → 自动测试 → 手动测试</p>
      </div>

      <!-- 测试工作流程步骤 -->
      <nz-card nzTitle="测试工作流程" class="workflow-card">
        <nz-steps [nzCurrent]="currentStep" nzDirection="horizontal">
          <nz-step nzTitle="选择测试批次" nzDescription="从可用批次中选择要测试的产品批次"></nz-step>
          <nz-step nzTitle="确认接线" nzDescription="验证所有测试点的物理连接"></nz-step>
          <nz-step nzTitle="自动通道测试" nzDescription="执行自动化测试流程"></nz-step>
          <nz-step nzTitle="手动测试" nzDescription="执行需要人工干预的测试项目"></nz-step>
        </nz-steps>
      </nz-card>

      <!-- 功能区域标签页 -->
      <nz-tabset nzType="card" class="test-tabs">
        <nz-tab nzTitle="批次管理">
          <div class="tab-content">
            <nz-alert 
              nzType="info" 
              nzMessage="批次选择" 
              nzDescription="请选择要进行测试的产品批次。系统将加载对应的测试配置和点表数据。"
              nzShowIcon>
            </nz-alert>
            
            <!-- 已选择的批次信息 -->
            <div *ngIf="selectedBatch" class="selected-batch-info">
              <h4>
                <span nz-icon nzType="check-circle" nzTheme="outline"></span>
                已选择批次
              </h4>
              <nz-card nzSize="small" class="selected-batch-card">
                <div class="batch-details">
                  <div class="batch-detail-item">
                    <span class="label">批次ID:</span>
                    <nz-tag nzColor="purple">{{ selectedBatch.batch_id }}</nz-tag>
                  </div>
                  <div class="batch-detail-item">
                    <span class="label">产品型号:</span>
                    <nz-tag nzColor="blue">{{ selectedBatch.product_model }}</nz-tag>
                  </div>
                  <div class="batch-detail-item">
                    <span class="label">序列号:</span>
                    <nz-tag nzColor="cyan">{{ selectedBatch.serial_number }}</nz-tag>
                  </div>
                  <div class="batch-detail-item">
                    <span class="label">总点数:</span>
                    <nz-tag nzColor="green">{{ selectedBatch.total_points }}</nz-tag>
                  </div>
                  <div class="batch-detail-item">
                    <span class="label">状态:</span>
                    <nz-tag [nzColor]="getStatusColor(selectedBatch.status_summary)">{{ selectedBatch.status_summary }}</nz-tag>
                  </div>
                </div>
              </nz-card>
            </div>

            <!-- 可用批次列表 -->
            <div class="available-batches">
              <h4>
                <span nz-icon nzType="container" nzTheme="outline"></span>
                可用测试批次
              </h4>
              
              <div *ngIf="isLoadingBatches" class="loading-state">
                <nz-alert nzType="info" nzMessage="正在加载批次列表..." nzShowIcon></nz-alert>
              </div>
              
              <div *ngIf="!isLoadingBatches && availableBatches.length === 0" class="empty-state">
                <nz-empty nzNotFoundContent="暂无可用批次，请先导入点表数据"></nz-empty>
              </div>
              
              <nz-list *ngIf="!isLoadingBatches && availableBatches.length > 0" 
                       [nzDataSource]="availableBatches" 
                       nzBordered>
                <nz-list-item *ngFor="let batch of availableBatches">
                  <nz-list-item-meta
                    [nzTitle]="batch.batch_id"
                    [nzDescription]="'产品: ' + batch.product_model + ' | 序列号: ' + batch.serial_number + ' | 创建时间: ' + formatDateTime(batch.creation_time)">
                  </nz-list-item-meta>
                  
                  <ul nz-list-item-actions>
                    <nz-list-item-action>
                      <nz-tag [nzColor]="getStatusColor(batch.status_summary)">
                        {{ batch.status_summary }}
                      </nz-tag>
                    </nz-list-item-action>
                    <nz-list-item-action>
                      <span class="points-info">{{ batch.total_points }}点</span>
                    </nz-list-item-action>
                    <nz-list-item-action>
                      <button nz-button 
                              nzType="primary" 
                              nzSize="small" 
                              [disabled]="selectedBatch?.batch_id === batch.batch_id"
                              (click)="selectBatch(batch)">
                        {{ selectedBatch?.batch_id === batch.batch_id ? '已选择' : '选择' }}
                      </button>
                    </nz-list-item-action>
                  </ul>
                </nz-list-item>
              </nz-list>
            </div>
            
            <div class="action-buttons">
              <nz-space>
                <button *nzSpaceItem nz-button nzType="default" (click)="refreshBatches()">
                  <span nz-icon nzType="reload" nzTheme="outline"></span>
                  刷新批次列表
                </button>
                <button *nzSpaceItem nz-button nzType="primary" routerLink="/data-management">
                  <span nz-icon nzType="import" nzTheme="outline"></span>
                  导入新点表
                </button>
              </nz-space>
            </div>
          </div>
        </nz-tab>

        <nz-tab nzTitle="接线确认">
          <div class="tab-content">
            <nz-alert 
              nzType="warning" 
              nzMessage="接线验证" 
              nzDescription="在开始自动测试前，请确认所有测试点的物理连接正确。这是确保测试结果准确性的重要步骤。"
              nzShowIcon>
            </nz-alert>
            
            <div class="action-buttons">
              <nz-space>
                <button *nzSpaceItem nz-button nzType="primary">
                  <span nz-icon nzType="check-circle" nzTheme="outline"></span>
                  开始接线检查
                </button>
                <button *nzSpaceItem nz-button nzType="default">
                  <span nz-icon nzType="eye" nzTheme="outline"></span>
                  查看接线图
                </button>
              </nz-space>
            </div>
          </div>
        </nz-tab>

        <nz-tab nzTitle="自动测试">
          <div class="tab-content">
            <nz-alert 
              nzType="success" 
              nzMessage="自动化测试" 
              nzDescription="系统将自动执行所有配置的测试项目，包括AI/AO/DI/DO点的功能验证和报警测试。"
              nzShowIcon>
            </nz-alert>
            
            <div class="test-progress">
              <h4>测试进度</h4>
              <nz-progress [nzPercent]="testProgress" nzStatus="active"></nz-progress>
            </div>
            
            <div class="action-buttons">
              <nz-space>
                <button *nzSpaceItem nz-button nzType="primary" routerLink="/test-execution">
                  <span nz-icon nzType="play-circle" nzTheme="outline"></span>
                  开始自动测试
                </button>
                <button *nzSpaceItem nz-button nzType="default">
                  <span nz-icon nzType="pause-circle" nzTheme="outline"></span>
                  暂停测试
                </button>
                <button *nzSpaceItem nz-button nzType="primary" nzDanger>
                  <span nz-icon nzType="stop" nzTheme="outline"></span>
                  停止测试
                </button>
              </nz-space>
            </div>
          </div>
        </nz-tab>

        <nz-tab nzTitle="手动测试">
          <div class="tab-content">
            <nz-alert 
              nzType="info" 
              nzMessage="手动测试" 
              nzDescription="对于需要人工干预或特殊验证的测试项目，可以在此进行手动测试操作。"
              nzShowIcon>
            </nz-alert>
            
            <div class="action-buttons">
              <nz-space>
                <button *nzSpaceItem nz-button nzType="primary" routerLink="/manual-test">
                  <span nz-icon nzType="tool" nzTheme="outline"></span>
                  手动测试工具
                </button>
                <button *nzSpaceItem nz-button nzType="default">
                  <span nz-icon nzType="form" nzTheme="outline"></span>
                  测试记录
                </button>
              </nz-space>
            </div>
          </div>
        </nz-tab>
      </nz-tabset>
    </div>
  `,
  styles: [`
    .test-area-container {
      padding: 24px;
      background: #f5f5f5;
      min-height: 100vh;
    }

    .page-header {
      margin-bottom: 24px;
    }

    .page-header h2 {
      margin: 0;
      color: #1890ff;
      font-size: 24px;
      font-weight: 600;
    }

    .page-header h2 span {
      margin-right: 8px;
    }

    .page-header p {
      margin: 8px 0 0 0;
      color: #666;
      font-size: 14px;
    }

    .workflow-card {
      margin-bottom: 24px;
      box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
    }

    .test-tabs {
      background: #fff;
      border-radius: 8px;
      box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
    }

    .tab-content {
      padding: 24px;
    }

    .action-buttons {
      margin-top: 24px;
    }

    .test-progress {
      margin: 24px 0;
    }

    .test-progress h4 {
      margin-bottom: 12px;
      color: #262626;
    }

    .selected-batch-info {
      margin-bottom: 24px;
    }

    .selected-batch-card {
      margin-top: 12px;
    }

    .batch-details {
      display: flex;
      flex-wrap: wrap;
    }

    .batch-detail-item {
      width: 50%;
      margin-bottom: 8px;
    }

    .label {
      font-weight: 600;
    }

    .available-batches {
      margin-bottom: 24px;
    }

    .loading-state {
      margin-bottom: 12px;
    }

    .empty-state {
      margin-bottom: 12px;
    }

    .points-info {
      margin-left: 8px;
    }
  `]
})
export class TestAreaComponent implements OnInit {
  
  currentStep = 0;
  testProgress = 0;
  availableBatches: any[] = [];
  selectedBatch: any = null;
  isLoadingBatches = false;

  constructor(private tauriApiService: TauriApiService, private message: NzMessageService) {}

  ngOnInit(): void {
    this.loadAvailableBatches();
  }

  // 加载可用批次
  async loadAvailableBatches(): Promise<void> {
    this.isLoadingBatches = true;
    try {
      if (this.tauriApiService.isTauriEnvironment()) {
        // 调用后端获取批次列表
        const result = await this.tauriApiService.getBatchList().toPromise();
        this.availableBatches = result || [];
      } else {
        // 模拟数据
        this.availableBatches = this.getMockBatches();
      }
      console.log('加载的批次列表:', this.availableBatches);
    } catch (error) {
      console.error('加载批次列表失败:', error);
      this.message.error('加载批次列表失败');
      // 使用模拟数据作为回退
      this.availableBatches = this.getMockBatches();
    } finally {
      this.isLoadingBatches = false;
    }
  }

  // 获取模拟批次数据
  private getMockBatches(): any[] {
    return [
      {
        batch_id: 'BATCH_20241201_1430',
        product_model: 'TEST_MODEL_A',
        serial_number: 'SN20241201001',
        creation_time: new Date().toISOString(),
        total_points: 88,
        tested_points: 0,
        passed_points: 0,
        failed_points: 0,
        status_summary: '已创建，等待测试'
      },
      {
        batch_id: 'BATCH_20241130_1645',
        product_model: 'TEST_MODEL_B',
        serial_number: 'SN20241130002',
        creation_time: new Date(Date.now() - 24 * 60 * 60 * 1000).toISOString(),
        total_points: 156,
        tested_points: 78,
        passed_points: 75,
        failed_points: 3,
        status_summary: '测试中 - 50%'
      }
    ];
  }

  // 选择批次
  selectBatch(batch: any): void {
    this.selectedBatch = batch;
    this.message.success(`已选择批次: ${batch.batch_id}`);
    console.log('选择的批次:', batch);
  }

  // 刷新批次列表
  refreshBatches(): void {
    this.loadAvailableBatches();
    this.message.info('正在刷新批次列表...');
  }

  // 格式化日期时间
  formatDateTime(dateTimeString: string): string {
    try {
      const date = new Date(dateTimeString);
      return date.toLocaleString('zh-CN', {
        year: 'numeric',
        month: '2-digit',
        day: '2-digit',
        hour: '2-digit',
        minute: '2-digit'
      });
    } catch (error) {
      return dateTimeString;
    }
  }

  // 获取状态标签颜色
  getStatusColor(status: string): string {
    if (status.includes('已创建')) return 'blue';
    if (status.includes('测试中')) return 'orange';
    if (status.includes('已完成')) return 'green';
    if (status.includes('失败')) return 'red';
    return 'default';
  }
} 