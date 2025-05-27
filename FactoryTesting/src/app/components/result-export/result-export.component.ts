import { Component, OnInit } from '@angular/core';
import { CommonModule } from '@angular/common';

// NG-ZORRO 组件导入
import { NzCardModule } from 'ng-zorro-antd/card';
import { NzButtonModule } from 'ng-zorro-antd/button';
import { NzIconModule } from 'ng-zorro-antd/icon';
import { NzSpaceModule } from 'ng-zorro-antd/space';
import { NzAlertModule } from 'ng-zorro-antd/alert';
import { NzMessageService } from 'ng-zorro-antd/message';

@Component({
  selector: 'app-result-export',
  standalone: true,
  imports: [
    CommonModule,
    NzCardModule,
    NzButtonModule,
    NzIconModule,
    NzSpaceModule,
    NzAlertModule
  ],
  template: `
    <div class="result-export-container">
      <div class="page-header">
        <h2>
          <span nz-icon nzType="export" nzTheme="outline"></span>
          结果导出
        </h2>
        <p>导出测试结果和生成测试报告</p>
      </div>

      <nz-alert 
        nzType="info" 
        nzMessage="导出说明" 
        nzDescription="可以导出Excel格式的测试结果，或生成PDF格式的测试报告。"
        nzShowIcon
        class="export-alert">
      </nz-alert>

      <div class="export-cards">
        <nz-card nzTitle="Excel导出" class="export-card">
          <p>导出详细的测试数据到Excel文件，包含所有测试点的结果和统计信息。</p>
          <nz-space>
            <button *nzSpaceItem nz-button nzType="primary" (click)="exportExcel()">
              <span nz-icon nzType="file-excel" nzTheme="outline"></span>
              导出Excel
            </button>
          </nz-space>
        </nz-card>

        <nz-card nzTitle="PDF报告" class="export-card">
          <p>生成格式化的PDF测试报告，适合打印和存档。</p>
          <nz-space>
            <button *nzSpaceItem nz-button nzType="primary" (click)="exportPdf()">
              <span nz-icon nzType="file-pdf" nzTheme="outline"></span>
              生成PDF报告
            </button>
          </nz-space>
        </nz-card>
      </div>
    </div>
  `,
  styles: [`
    .result-export-container {
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

    .export-alert {
      margin-bottom: 24px;
    }

    .export-cards {
      display: grid;
      grid-template-columns: 1fr 1fr;
      gap: 24px;
    }

    .export-card {
      background: #fff;
      border-radius: 8px;
      box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
    }

    @media (max-width: 768px) {
      .export-cards {
        grid-template-columns: 1fr;
      }
    }
  `]
})
export class ResultExportComponent implements OnInit {

  constructor(private message: NzMessageService) {}

  ngOnInit(): void {
    // 初始化组件
  }

  exportExcel(): void {
    this.message.info('正在导出Excel文件...');
    // TODO: 实现Excel导出功能
  }

  exportPdf(): void {
    this.message.info('正在生成PDF报告...');
    // TODO: 实现PDF报告生成功能
  }
} 