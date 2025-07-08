import { Component, OnInit } from '@angular/core';
import { CommonModule } from '@angular/common';

// NG-ZORRO 组件导入
import { NzCardModule } from 'ng-zorro-antd/card';
import { NzButtonModule } from 'ng-zorro-antd/button';
import { NzIconModule } from 'ng-zorro-antd/icon';
import { NzSpaceModule } from 'ng-zorro-antd/space';
import { NzAlertModule } from 'ng-zorro-antd/alert';
import { NzMessageService } from 'ng-zorro-antd/message';
import { TauriApiService } from '../../services/tauri-api.service';
import { firstValueFrom } from 'rxjs';
// Tauri 对话框 save API
import { save as saveDialog } from '@tauri-apps/plugin-dialog';


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

  constructor(private message: NzMessageService,
              private tauriApi: TauriApiService) {}

  ngOnInit(): void {
    // 初始化组件
  }

  async exportExcel(): Promise<void> {
    // 先让用户选择导出位置
    const selectedPath = await this.openSaveDialog();
    if (!selectedPath || selectedPath.trim().length === 0) {
      // 用户取消或未输入文件名
      return;
    }

    const msgRef = this.message.loading('正在导出测试结果...', { nzDuration: 0 });
    try {
      const filePath = await firstValueFrom(this.tauriApi.exportTestResults(selectedPath));
      msgRef.messageId && this.message.remove(msgRef.messageId);
      this.message.success('导出成功: ' + filePath);
    } catch (err) {
      msgRef.messageId && this.message.remove(msgRef.messageId);
      this.message.error('导出失败: ' + err);
    }
  }

  /**
   * 打开保存对话框，获取用户指定的导出路径
   */
  private async openSaveDialog(): Promise<string | null> {
    const defaultName = `test_results_${new Date().toISOString().slice(0,16).replace(/[:T]/g,'')}.xlsx`;
    return await saveDialog({
      title: '请选择导出位置',
      defaultPath: defaultName,
      filters: [
        { name: 'Excel', extensions: ['xlsx'] }
      ]
    });
  }

  exportPdf(): void {
    this.message.info('正在生成PDF报告...');
    // TODO: 实现PDF报告生成功能
  }
} 