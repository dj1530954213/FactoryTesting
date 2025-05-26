import { Component, OnInit, OnDestroy } from '@angular/core';
import { RouterOutlet, RouterLink, RouterLinkActive } from '@angular/router';
import { CommonModule } from '@angular/common';
import { TauriApiService } from './services/tauri-api.service';
import { SystemStatus } from './models';
import { Subscription } from 'rxjs';

// NG-ZORRO 组件导入
import { NzLayoutModule } from 'ng-zorro-antd/layout';
import { NzMenuModule } from 'ng-zorro-antd/menu';
import { NzIconModule } from 'ng-zorro-antd/icon';
import { NzBadgeModule } from 'ng-zorro-antd/badge';
import { NzTagModule } from 'ng-zorro-antd/tag';
import { NzSpaceModule } from 'ng-zorro-antd/space';
import { NzDividerModule } from 'ng-zorro-antd/divider';

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
    NzDividerModule
  ],
  templateUrl: './app.component.html',
  styleUrl: './app.component.css'
})
export class AppComponent implements OnInit, OnDestroy {
  // 工厂测试系统主组件
  // 当前为基础版本，后续将添加更多功能
  title = 'factory-testing';
  systemStatus: SystemStatus | null = null;
  isConnected = false;
  lastUpdateTime = new Date();
  
  private statusSubscription?: Subscription;

  constructor(private tauriApiService: TauriApiService) {}

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
  }

  ngOnDestroy() {
    // 移除全局拖拽事件处理
    this.removeGlobalDragHandling();
    
    if (this.statusSubscription) {
      this.statusSubscription.unsubscribe();
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
}
