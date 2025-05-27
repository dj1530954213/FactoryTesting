import { Component, OnInit } from '@angular/core';
import { CommonModule } from '@angular/common';

// NG-ZORRO 组件导入
import { NzCardModule } from 'ng-zorro-antd/card';
import { NzButtonModule } from 'ng-zorro-antd/button';
import { NzIconModule } from 'ng-zorro-antd/icon';
import { NzSpaceModule } from 'ng-zorro-antd/space';
import { NzAlertModule } from 'ng-zorro-antd/alert';
import { NzTabsModule } from 'ng-zorro-antd/tabs';
import { NzMessageService } from 'ng-zorro-antd/message';

@Component({
  selector: 'app-system-settings',
  standalone: true,
  imports: [
    CommonModule,
    NzCardModule,
    NzButtonModule,
    NzIconModule,
    NzSpaceModule,
    NzAlertModule,
    NzTabsModule
  ],
  template: `
    <div class="system-settings-container">
      <div class="page-header">
        <h2>
          <span nz-icon nzType="setting" nzTheme="outline"></span>
          系统设置
        </h2>
        <p>配置系统参数和应用设置</p>
      </div>

      <nz-tabset nzType="card" class="settings-tabs">
        <nz-tab nzTitle="通用设置">
          <div class="tab-content">
            <nz-card nzTitle="应用配置" class="settings-card">
              <p>配置应用的基本参数，如超时时间、重试次数等。</p>
              <nz-space>
                <button *nzSpaceItem nz-button nzType="primary" (click)="saveGeneralSettings()">
                  <span nz-icon nzType="save" nzTheme="outline"></span>
                  保存设置
                </button>
              </nz-space>
            </nz-card>
          </div>
        </nz-tab>

        <nz-tab nzTitle="测试配置">
          <div class="tab-content">
            <nz-card nzTitle="测试参数" class="settings-card">
              <p>配置测试相关的参数，如并发数量、测试超时等。</p>
              <nz-space>
                <button *nzSpaceItem nz-button nzType="primary" (click)="saveTestSettings()">
                  <span nz-icon nzType="save" nzTheme="outline"></span>
                  保存设置
                </button>
              </nz-space>
            </nz-card>
          </div>
        </nz-tab>

        <nz-tab nzTitle="日志设置">
          <div class="tab-content">
            <nz-card nzTitle="日志配置" class="settings-card">
              <p>配置日志级别和输出设置。</p>
              <nz-space>
                <button *nzSpaceItem nz-button nzType="primary" (click)="saveLogSettings()">
                  <span nz-icon nzType="save" nzTheme="outline"></span>
                  保存设置
                </button>
              </nz-space>
            </nz-card>
          </div>
        </nz-tab>

        <nz-tab nzTitle="关于">
          <div class="tab-content">
            <nz-card nzTitle="系统信息" class="settings-card">
              <p>工厂验收测试系统 v1.0.0</p>
              <p>基于 Rust + Angular + Tauri 技术栈开发</p>
              <p>© 2024 工厂验收测试系统</p>
            </nz-card>
          </div>
        </nz-tab>
      </nz-tabset>
    </div>
  `,
  styles: [`
    .system-settings-container {
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

    .settings-tabs {
      background: #fff;
      border-radius: 8px;
      box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
    }

    .tab-content {
      padding: 24px;
    }

    .settings-card {
      margin-bottom: 16px;
    }
  `]
})
export class SystemSettingsComponent implements OnInit {

  constructor(private message: NzMessageService) {}

  ngOnInit(): void {
    // 初始化组件
  }

  saveGeneralSettings(): void {
    this.message.success('通用设置已保存');
  }

  saveTestSettings(): void {
    this.message.success('测试设置已保存');
  }

  saveLogSettings(): void {
    this.message.success('日志设置已保存');
  }
} 