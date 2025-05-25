import { Component, OnInit, OnDestroy } from '@angular/core';
import { RouterOutlet, RouterLink, RouterLinkActive } from '@angular/router';
import { CommonModule } from '@angular/common';
import { Subscription } from 'rxjs';
import { TauriApiService } from './services/tauri-api.service';
import { SystemStatus } from './models';

@Component({
  selector: 'app-root',
  standalone: true,
  imports: [RouterOutlet, RouterLink, RouterLinkActive, CommonModule],
  templateUrl: './app.component.html',
  styleUrl: './app.component.css'
})
export class AppComponent implements OnInit, OnDestroy {
  // 系统状态相关属性
  systemVersion = 'Unknown';
  isSystemHealthy = false;
  systemHealthText = '检查中...';
  activeTaskCount = 0;

  private subscriptions: Subscription[] = [];

  constructor(private tauriApi: TauriApiService) {}

  ngOnInit(): void {
    this.initializeSystemStatus();
  }

  ngOnDestroy(): void {
    // 清理订阅
    this.subscriptions.forEach(sub => sub.unsubscribe());
  }

  /**
   * 初始化系统状态监听
   */
  private initializeSystemStatus(): void {
    // 订阅系统状态更新
    const statusSub = this.tauriApi.systemStatus$.subscribe({
      next: (status: SystemStatus | null) => {
        if (status) {
          this.updateSystemStatus(status);
        }
      },
      error: (error) => {
        console.error('系统状态订阅失败:', error);
        this.handleSystemStatusError();
      }
    });

    this.subscriptions.push(statusSub);
  }

  /**
   * 更新系统状态显示
   */
  private updateSystemStatus(status: SystemStatus): void {
    this.systemVersion = status.version;
    this.isSystemHealthy = status.system_health === 'healthy';
    this.activeTaskCount = status.active_test_tasks;
    
    // 更新健康状态文本
    if (this.isSystemHealthy) {
      this.systemHealthText = '系统正常';
    } else {
      this.systemHealthText = '系统异常';
    }
  }

  /**
   * 处理系统状态错误
   */
  private handleSystemStatusError(): void {
    this.isSystemHealthy = false;
    this.systemHealthText = '连接失败';
    this.activeTaskCount = 0;
  }
} 