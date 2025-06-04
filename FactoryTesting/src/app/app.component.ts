import { Component, OnInit, OnDestroy } from '@angular/core';
import { RouterOutlet, RouterLink, RouterLinkActive } from '@angular/router';
import { CommonModule } from '@angular/common';
import { TauriApiService } from './services/tauri-api.service';
import { SystemStatus } from './models';
import { PlcConnectionStatus } from './models/plc-connection-status.model';
import { Subscription, interval } from 'rxjs';

// NG-ZORRO 组件导入
import { NzLayoutModule } from 'ng-zorro-antd/layout';
import { NzMenuModule } from 'ng-zorro-antd/menu';
import { NzIconModule } from 'ng-zorro-antd/icon';
import { NzBadgeModule } from 'ng-zorro-antd/badge';
import { NzTagModule } from 'ng-zorro-antd/tag';
import { NzSpaceModule } from 'ng-zorro-antd/space';
import { NzDividerModule } from 'ng-zorro-antd/divider';
import { NzMessageService } from 'ng-zorro-antd/message';
import { NzMessageModule } from 'ng-zorro-antd/message';

@Component({
  selector: 'app-root',
  standalone: true,
  imports: [
    RouterOutlet, 
    RouterLink, 
    RouterLinkActive, 
    CommonModule,
    // NG-ZORRO 模块
    NzLayoutModule,
    NzMenuModule,
    NzIconModule,
    NzBadgeModule,
    NzTagModule,
    NzSpaceModule,
    NzDividerModule,
    NzMessageModule
  ],
  templateUrl: './app.component.html',
  styleUrl: './app.component.css'
})
export class AppComponent implements OnInit, OnDestroy {
  // 工厂测试系统主组件
  // 当前为基础版本，后续将添加更多功能
  title = 'factory-testing';
  systemStatus: SystemStatus | null = null;
  plcConnectionStatus: PlcConnectionStatus | null = null;
  isConnected = false;
  lastUpdateTime = new Date();

  private statusSubscription?: Subscription;
  private plcStatusSubscription?: Subscription;

  constructor(private tauriApiService: TauriApiService, private messageService: NzMessageService) {}

  ngOnInit() {
    // 设置全局拖拽事件处理
    this.setupGlobalDragHandling();

    // 订阅系统状态更新
    this.statusSubscription = this.tauriApiService.getSystemStatus().subscribe({
      next: (status) => {
        this.systemStatus = status;
        this.isConnected = status.system_health === 'healthy';
        this.lastUpdateTime = new Date();
      },
      error: (error) => {
        console.error('获取系统状态失败:', error);
        this.isConnected = false;
      }
    });

    // 启动PLC连接状态轮询（每5秒检查一次）
    this.startPlcStatusPolling();
  }

  ngOnDestroy() {
    // 移除全局拖拽事件处理
    this.removeGlobalDragHandling();

    if (this.statusSubscription) {
      this.statusSubscription.unsubscribe();
    }

    if (this.plcStatusSubscription) {
      this.plcStatusSubscription.unsubscribe();
    }
  }

  // 设置全局拖拽事件处理
  private setupGlobalDragHandling() {
    if (typeof window !== 'undefined') {
      // 阻止整个应用的默认拖拽行为
      document.addEventListener('dragover', this.preventDefaults, false);
      document.addEventListener('drop', this.preventDefaults, false);
      document.addEventListener('dragenter', this.preventDefaults, false);
      
      // 阻止窗口级别的拖拽行为
      window.addEventListener('dragover', this.preventDefaults, false);
      window.addEventListener('drop', this.preventDefaults, false);
    }
  }

  // 移除全局拖拽事件处理
  private removeGlobalDragHandling() {
    if (typeof window !== 'undefined') {
      document.removeEventListener('dragover', this.preventDefaults, false);
      document.removeEventListener('drop', this.preventDefaults, false);
      document.removeEventListener('dragenter', this.preventDefaults, false);
      
      window.removeEventListener('dragover', this.preventDefaults, false);
      window.removeEventListener('drop', this.preventDefaults, false);
    }
  }

  // 阻止默认拖拽行为
  private preventDefaults = (e: Event) => {
    e.preventDefault();
    e.stopPropagation();
  }

  getCurrentTime(): string {
    return new Date().toLocaleTimeString();
  }

  // 获取测试进度百分比
  getTestProgress(): number {
    if (!this.systemStatus) {
      return 0;
    }

    // 基于活动任务数量计算进度
    // 这里可以根据实际业务逻辑调整计算方式
    const activeTasks = this.systemStatus.active_test_tasks || 0;
    const maxTasks = 100; // 假设最大任务数为100

    if (activeTasks === 0) {
      return 0;
    }

    // 简单的进度计算，实际应用中可能需要更复杂的逻辑
    return Math.min(Math.round((activeTasks / maxTasks) * 100), 100);
  }

  // 启动PLC连接状态轮询
  private startPlcStatusPolling() {
    // 立即检查一次
    this.checkPlcConnectionStatus();

    // 每5秒检查一次PLC连接状态
    this.plcStatusSubscription = interval(5000).subscribe(() => {
      this.checkPlcConnectionStatus();
    });
  }

  // 检查PLC连接状态
  private checkPlcConnectionStatus() {
    if (this.tauriApiService.isTauriEnvironment()) {
      this.tauriApiService.getPlcConnectionStatus().subscribe({
        next: (status) => {
          this.plcConnectionStatus = status;
        },
        error: (error) => {
          console.error('获取PLC连接状态失败:', error);
          // 设置默认状态
          this.plcConnectionStatus = {
            testPlcConnected: false,
            targetPlcConnected: false,
            testPlcName: undefined,
            targetPlcName: undefined,
            lastCheckTime: new Date().toISOString()
          };
        }
      });
    } else {
      // 非Tauri环境，设置模拟状态
      this.plcConnectionStatus = {
        testPlcConnected: false,
        targetPlcConnected: false,
        testPlcName: '模拟测试PLC',
        targetPlcName: '模拟被测PLC',
        lastCheckTime: new Date().toISOString()
      };
    }
  }
}
