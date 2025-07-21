/**
 * # 应用主组件 - AppComponent
 * 
 * ## 业务功能说明
 * - 工厂测试系统的根组件，负责整体应用框架的构建
 * - 提供应用的主导航布局和顶级状态管理
 * - 实时监控系统状态和PLC连接状态
 * - 处理全局的拖拽事件防护
 * 
 * ## 前后端调用链
 * - **调用后端**: get_system_status、get_plc_connection_status_cmd
 * - **数据流向**: Angular组件 → TauriApiService → Tauri → Rust后端 → 状态管理器/PLC服务
 * - **关键接口**: 系统状态轮询、PLC连接状态检查
 * 
 * ## Angular知识点
 * - **框架特性**: Component装饰器、standalone组件、OnInit/OnDestroy生命周期
 * - **生命周期**: ngOnInit初始化、ngOnDestroy清理资源
 * - **依赖注入**: TauriApiService、NzMessageService
 * - **响应式编程**: interval轮询、subscription订阅管理
 * 
 * ## NG-ZORRO组件
 * - NzLayoutModule: 布局框架
 * - NzMenuModule: 导航菜单
 * - NzBadgeModule: 状态徽章
 * - NzTagModule: 状态标签
 * - NzMessageModule: 消息提示
 * 
 * ## 设计模式
 * - 观察者模式：状态轮询和事件订阅
 * - 单例模式：全局服务注入
 */

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
/**
 * 工厂测试系统应用主组件
 * 
 * **业务作用**: 系统的根组件，负责顶级布局和全局状态管理
 * **调用后端**: 通过TauriApiService调用后端状态服务
 * **Angular特性**: 实现OnInit和OnDestroy生命周期接口
 */
export class AppComponent implements OnInit, OnDestroy {
  // === 组件基础属性 ===
  /** 应用标题 */
  title = 'factory-testing';
  
  // === 系统状态相关属性 ===
  /** 
   * 系统状态信息
   * **数据来源**: 后端get_system_status命令
   * **更新频率**: 每5秒轮询一次
   */
  systemStatus: SystemStatus | null = null;
  
  /** 
   * PLC连接状态信息
   * **数据来源**: 后端get_plc_connection_status_cmd命令  
   * **更新频率**: 每1秒轮询一次
   */
  plcConnectionStatus: PlcConnectionStatus | null = null;
  
  /** 
   * 系统连接状态标志
   * **业务含义**: 标识系统整体健康状态
   */
  isConnected = false;
  
  /** 最后更新时间 */
  lastUpdateTime = new Date();

  // === 订阅管理 ===
  /** 系统状态订阅 - 用于取消订阅防止内存泄漏 */
  private statusSubscription?: Subscription;
  
  /** PLC状态订阅 - 用于取消订阅防止内存泄漏 */
  private plcStatusSubscription?: Subscription;

  /**
   * 构造函数 - 依赖注入
   * 
   * **注入服务**:
   * - TauriApiService: 与后端通信的API服务
   * - NzMessageService: Ant Design消息提示服务
   */
  constructor(private tauriApiService: TauriApiService, private messageService: NzMessageService) {}

  /**
   * 组件初始化生命周期钩子
   * 
   * **业务流程**:
   * 1. 设置全局拖拽事件防护
   * 2. 启动系统状态实时监控
   * 3. 启动PLC连接状态轮询
   * 
   * **调用链**: ngOnInit → TauriApiService → 后端状态服务
   */
  ngOnInit() {
    // 设置全局拖拽事件处理，防止文件拖拽影响应用
    this.setupGlobalDragHandling();

    // 订阅系统状态更新 - 建立与后端的实时连接
    this.statusSubscription = this.tauriApiService.getSystemStatus().subscribe({
      next: (status) => {
        this.systemStatus = status;
        // 根据后端返回的健康状态更新连接标志
        this.isConnected = status.system_health === 'healthy';
        this.lastUpdateTime = new Date();
      },
      error: (error) => {
        console.error('获取系统状态失败:', error);
        this.isConnected = false;
      }
    });

    // 启动PLC连接状态轮询（每1秒检查一次，与后端心跳保持同步）
    this.startPlcStatusPolling();
  }

  /**
   * 组件销毁生命周期钩子
   * 
   * **业务功能**: 清理资源，防止内存泄漏
   * **清理项目**:
   * 1. 移除全局拖拽事件监听器
   * 2. 取消系统状态订阅
   * 3. 取消PLC状态订阅
   * 
   * **Angular最佳实践**: 在OnDestroy中清理所有订阅
   */
  ngOnDestroy() {
    // 移除全局拖拽事件处理，避免内存泄漏
    this.removeGlobalDragHandling();

    // 取消系统状态订阅，防止组件销毁后继续执行回调
    if (this.statusSubscription) {
      this.statusSubscription.unsubscribe();
    }

    // 取消PLC状态订阅，防止内存泄漏
    if (this.plcStatusSubscription) {
      this.plcStatusSubscription.unsubscribe();
    }
  }

  /**
   * 设置全局拖拽事件处理
   * 
   * **业务目的**: 防止用户意外拖拽文件到应用导致页面跳转
   * **技术实现**: 全局监听拖拽事件并阻止默认行为
   * **适用场景**: 桌面应用中防止文件拖拽干扰
   */
  private setupGlobalDragHandling() {
    // 检查是否在浏览器环境中，避免服务端渲染错误
    if (typeof window !== 'undefined') {
      // 阻止整个应用的默认拖拽行为，防止文件拖拽导致页面跳转
      document.addEventListener('dragover', this.preventDefaults, false);
      document.addEventListener('drop', this.preventDefaults, false);
      document.addEventListener('dragenter', this.preventDefaults, false);
      
      // 阻止窗口级别的拖拽行为，提供双重保护
      window.addEventListener('dragover', this.preventDefaults, false);
      window.addEventListener('drop', this.preventDefaults, false);
    }
  }

  /**
   * 移除全局拖拽事件处理
   * 
   * **业务目的**: 组件销毁时清理事件监听器，防止内存泄漏
   * **调用时机**: 在ngOnDestroy中调用
   */
  private removeGlobalDragHandling() {
    if (typeof window !== 'undefined') {
      // 移除document级别的事件监听器
      document.removeEventListener('dragover', this.preventDefaults, false);
      document.removeEventListener('drop', this.preventDefaults, false);
      document.removeEventListener('dragenter', this.preventDefaults, false);
      
      // 移除window级别的事件监听器
      window.removeEventListener('dragover', this.preventDefaults, false);
      window.removeEventListener('drop', this.preventDefaults, false);
    }
  }

  /**
   * 阻止默认拖拽行为的事件处理器
   * 
   * **技术实现**: 使用箭头函数保持this上下文
   * **功能**: 阻止事件冒泡和默认行为
   */
  private preventDefaults = (e: Event) => {
    e.preventDefault();  // 阻止默认行为
    e.stopPropagation(); // 阻止事件冒泡
  }

  /**
   * 获取当前时间字符串
   * 
   * **业务用途**: 在UI中显示实时时间
   * **返回格式**: 本地化时间字符串 (HH:MM:SS)
   */
  getCurrentTime(): string {
    return new Date().toLocaleTimeString();
  }

  /**
   * 获取测试进度百分比
   * 
   * **业务场景**: 在UI中显示当前测试任务的整体进度
   * **计算逻辑**: 基于活动任务数量计算相对进度
   * **数据来源**: systemStatus.active_test_tasks
   * 
   * @returns 0-100的进度百分比
   */
  getTestProgress(): number {
    // 如果系统状态不可用，返回0进度
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

  /**
   * 启动PLC连接状态轮询
   * 
   * **业务需求**: 实时监控PLC连接状态，及时发现连接异常
   * **轮询频率**: 每1秒检查一次，与后端心跳检测保持同步
   * **调用链**: startPlcStatusPolling → checkPlcConnectionStatus → TauriApiService → 后端get_plc_connection_status_cmd
   */
  private startPlcStatusPolling() {
    // 立即检查一次，避免等待轮询间隔
    this.checkPlcConnectionStatus();

    // 每1秒检查一次PLC连接状态，与后端心跳检测保持同步
    this.plcStatusSubscription = interval(1000).subscribe(() => {
      this.checkPlcConnectionStatus();
    });
  }

  /**
   * 检查PLC连接状态
   * 
   * **业务功能**: 获取测试PLC和被测PLC的连接状态
   * **环境适配**: 
   * - Tauri环境: 调用后端真实状态
   * - 浏览器环境: 返回模拟状态用于开发调试
   * 
   * **调用链**: checkPlcConnectionStatus → TauriApiService.getPlcConnectionStatus → 后端get_plc_connection_status_cmd
   * **错误处理**: 连接失败时设置默认断开状态
   */
  private checkPlcConnectionStatus() {
    // 检查是否在Tauri桌面应用环境中
    if (this.tauriApiService.isTauriEnvironment()) {
      // 真实环境：调用后端API获取PLC状态
      this.tauriApiService.getPlcConnectionStatus().subscribe({
        next: (status) => {
          this.plcConnectionStatus = status;
        },
        error: (error) => {
          console.error('获取PLC连接状态失败:', error);
          // 设置默认断开状态，确保UI有明确的错误指示
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
      // 非Tauri环境（浏览器开发环境），设置模拟状态用于UI调试
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
