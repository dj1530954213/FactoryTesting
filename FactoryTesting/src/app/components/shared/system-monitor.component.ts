import { Component, OnInit, OnDestroy } from '@angular/core';
import { CommonModule } from '@angular/common';
import { FormsModule } from '@angular/forms';
import { Router } from '@angular/router';
import { Subscription, interval } from 'rxjs';
import { TauriApiService } from '../../services/tauri-api.service';
import { SystemStatus } from '../../models';

interface SystemMetrics {
  cpuUsage: number;
  memoryUsage: number;
  diskUsage: number;
  networkStatus: string;
  lastUpdateTime: string;
}

interface PerformanceData {
  timestamp: string;
  activeTests: number;
  responseTime: number;
  errorRate: number;
}

@Component({
  selector: 'app-system-monitor',
  standalone: true,
  imports: [CommonModule, FormsModule],
  template: `
    <div class="system-monitor-container">
      <!-- 页面标题 -->
      <div class="page-header">
        <h1>🖥️ 系统监控</h1>
        <div class="header-actions">
          <button class="btn btn-secondary" (click)="goToDashboard()">
            返回仪表板
          </button>
          <button class="btn btn-primary" (click)="refreshData()" [disabled]="isLoading">
            <span *ngIf="isLoading">刷新中...</span>
            <span *ngIf="!isLoading">🔄 刷新</span>
          </button>
        </div>
      </div>

      <!-- 系统状态概览 -->
      <div class="status-overview">
        <div class="status-card system-health" [class.healthy]="systemStatus?.system_health === 'healthy'">
          <div class="card-icon">{{ systemStatus?.system_health === 'healthy' ? '✅' : '⚠️' }}</div>
          <div class="card-content">
            <h3>系统健康状态</h3>
            <p class="status-value">{{ getSystemHealthText() }}</p>
            <small>最后检查: {{ lastUpdateTime | date:'HH:mm:ss' }}</small>
          </div>
        </div>

        <div class="status-card active-tasks">
          <div class="card-icon">⚡</div>
          <div class="card-content">
            <h3>活动测试任务</h3>
            <p class="status-value">{{ systemStatus?.active_test_tasks || 0 }}</p>
            <small>正在执行的测试数量</small>
          </div>
        </div>

        <div class="status-card version-info">
          <div class="card-icon">📋</div>
          <div class="card-content">
            <h3>系统版本</h3>
            <p class="status-value">{{ systemStatus?.version || 'v1.0.0' }}</p>
            <small>当前运行版本</small>
          </div>
        </div>

        <div class="status-card uptime">
          <div class="card-icon">⏱️</div>
          <div class="card-content">
            <h3>运行时间</h3>
            <p class="status-value">{{ formatUptime() }}</p>
            <small>系统连续运行时间</small>
          </div>
        </div>
      </div>

      <!-- 性能指标 -->
      <div class="performance-section">
        <h2>性能指标</h2>
        <div class="metrics-grid">
          <div class="metric-card">
            <h4>CPU 使用率</h4>
            <div class="progress-bar">
              <div class="progress-fill" [style.width.%]="metrics.cpuUsage"></div>
            </div>
            <span class="metric-value">{{ metrics.cpuUsage }}%</span>
          </div>

          <div class="metric-card">
            <h4>内存使用率</h4>
            <div class="progress-bar">
              <div class="progress-fill" [style.width.%]="metrics.memoryUsage"></div>
            </div>
            <span class="metric-value">{{ metrics.memoryUsage }}%</span>
          </div>

          <div class="metric-card">
            <h4>磁盘使用率</h4>
            <div class="progress-bar">
              <div class="progress-fill" [style.width.%]="metrics.diskUsage"></div>
            </div>
            <span class="metric-value">{{ metrics.diskUsage }}%</span>
          </div>

          <div class="metric-card">
            <h4>网络状态</h4>
            <div class="network-status" [class]="getNetworkStatusClass()">
              {{ metrics.networkStatus }}
            </div>
          </div>
        </div>
      </div>

      <!-- 实时监控图表 -->
      <div class="charts-section">
        <h2>实时监控</h2>
        <div class="charts-grid">
          <div class="chart-card">
            <h4>活动测试数量趋势</h4>
            <div class="chart-placeholder">
              <div class="chart-line">
                <div *ngFor="let data of performanceHistory" 
                     class="chart-point" 
                     [style.height.%]="(data.activeTests / maxActiveTests) * 100"
                     [title]="data.timestamp + ': ' + data.activeTests + ' 个测试'">
                </div>
              </div>
            </div>
          </div>

          <div class="chart-card">
            <h4>系统响应时间</h4>
            <div class="chart-placeholder">
              <div class="chart-line">
                <div *ngFor="let data of performanceHistory" 
                     class="chart-point response-time" 
                     [style.height.%]="(data.responseTime / maxResponseTime) * 100"
                     [title]="data.timestamp + ': ' + data.responseTime + 'ms'">
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- 系统日志 -->
      <div class="logs-section">
        <h2>系统日志</h2>
        <div class="log-controls">
          <select [(ngModel)]="logLevel" (change)="filterLogs()">
            <option value="">所有级别</option>
            <option value="info">信息</option>
            <option value="warning">警告</option>
            <option value="error">错误</option>
          </select>
          <button class="btn btn-outline" (click)="clearLogs()">清除日志</button>
        </div>
        <div class="log-container">
          <div *ngFor="let log of filteredLogs" class="log-entry" [class]="log.level">
            <span class="log-timestamp">{{ log.timestamp | date:'HH:mm:ss' }}</span>
            <span class="log-level">{{ log.level.toUpperCase() }}</span>
            <span class="log-message">{{ log.message }}</span>
          </div>
        </div>
      </div>

      <!-- 错误提示 -->
      <div class="error-message" *ngIf="error">
        <div class="error-content">
          <i class="error-icon">⚠️</i>
          <span>{{ error }}</span>
          <button class="btn btn-sm btn-outline" (click)="clearError()">关闭</button>
        </div>
      </div>
    </div>
  `,
  styles: [`
    .system-monitor-container {
      padding: 20px;
      max-width: 1400px;
      margin: 0 auto;
      background: #f8f9fa;
      min-height: 100vh;
    }

    .page-header {
      display: flex;
      justify-content: space-between;
      align-items: center;
      margin-bottom: 30px;
      padding: 20px;
      background: white;
      border-radius: 8px;
      box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
    }

    .page-header h1 {
      margin: 0;
      color: #333;
      font-size: 28px;
      font-weight: 600;
    }

    .header-actions {
      display: flex;
      gap: 12px;
    }

    .status-overview {
      display: grid;
      grid-template-columns: repeat(auto-fit, minmax(280px, 1fr));
      gap: 20px;
      margin-bottom: 30px;
    }

    .status-card {
      background: white;
      border-radius: 8px;
      padding: 20px;
      box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
      display: flex;
      align-items: center;
      transition: transform 0.2s ease;
    }

    .status-card:hover {
      transform: translateY(-2px);
    }

    .status-card.healthy {
      border-left: 4px solid #28a745;
    }

    .card-icon {
      font-size: 32px;
      margin-right: 16px;
    }

    .card-content h3 {
      margin: 0 0 8px 0;
      font-size: 16px;
      color: #666;
    }

    .status-value {
      font-size: 24px;
      font-weight: 600;
      color: #333;
      margin-bottom: 4px;
    }

    .performance-section, .charts-section, .logs-section {
      background: white;
      border-radius: 8px;
      padding: 20px;
      margin-bottom: 20px;
      box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
    }

    .performance-section h2, .charts-section h2, .logs-section h2 {
      margin: 0 0 20px 0;
      color: #333;
      font-size: 20px;
    }

    .metrics-grid {
      display: grid;
      grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
      gap: 20px;
    }

    .metric-card {
      padding: 16px;
      border: 1px solid #e9ecef;
      border-radius: 6px;
      background: #f8f9fa;
    }

    .metric-card h4 {
      margin: 0 0 12px 0;
      font-size: 14px;
      color: #666;
    }

    .progress-bar {
      width: 100%;
      height: 8px;
      background: #e9ecef;
      border-radius: 4px;
      overflow: hidden;
      margin-bottom: 8px;
    }

    .progress-fill {
      height: 100%;
      background: linear-gradient(90deg, #28a745, #20c997);
      transition: width 0.3s ease;
    }

    .metric-value {
      font-weight: 600;
      color: #333;
    }

    .network-status {
      padding: 4px 8px;
      border-radius: 4px;
      font-weight: 500;
      text-align: center;
    }

    .network-status.connected {
      background: #d4edda;
      color: #155724;
    }

    .network-status.disconnected {
      background: #f8d7da;
      color: #721c24;
    }

    .charts-grid {
      display: grid;
      grid-template-columns: repeat(auto-fit, minmax(400px, 1fr));
      gap: 20px;
    }

    .chart-card {
      border: 1px solid #e9ecef;
      border-radius: 6px;
      padding: 16px;
    }

    .chart-card h4 {
      margin: 0 0 16px 0;
      font-size: 16px;
      color: #333;
    }

    .chart-placeholder {
      height: 200px;
      background: #f8f9fa;
      border-radius: 4px;
      position: relative;
      overflow: hidden;
    }

    .chart-line {
      display: flex;
      align-items: end;
      height: 100%;
      padding: 10px;
      gap: 2px;
    }

    .chart-point {
      flex: 1;
      background: #007bff;
      min-height: 2px;
      border-radius: 2px 2px 0 0;
      transition: all 0.3s ease;
    }

    .chart-point.response-time {
      background: #28a745;
    }

    .chart-point:hover {
      opacity: 0.8;
    }

    .log-controls {
      display: flex;
      gap: 12px;
      margin-bottom: 16px;
    }

    .log-controls select {
      padding: 6px 12px;
      border: 1px solid #d9d9d9;
      border-radius: 4px;
    }

    .log-container {
      max-height: 300px;
      overflow-y: auto;
      border: 1px solid #e9ecef;
      border-radius: 4px;
      background: #f8f9fa;
    }

    .log-entry {
      display: flex;
      gap: 12px;
      padding: 8px 12px;
      border-bottom: 1px solid #e9ecef;
      font-family: monospace;
      font-size: 12px;
    }

    .log-entry.error {
      background: #fff5f5;
      color: #c53030;
    }

    .log-entry.warning {
      background: #fffbf0;
      color: #d69e2e;
    }

    .log-entry.info {
      background: #f0f8ff;
      color: #3182ce;
    }

    .log-timestamp {
      color: #666;
      min-width: 80px;
    }

    .log-level {
      font-weight: 600;
      min-width: 60px;
    }

    .log-message {
      flex: 1;
    }

    .btn {
      padding: 8px 16px;
      border: 1px solid transparent;
      border-radius: 4px;
      font-size: 14px;
      cursor: pointer;
      transition: all 0.3s;
    }

    .btn:disabled {
      opacity: 0.6;
      cursor: not-allowed;
    }

    .btn-primary {
      background-color: #007bff;
      border-color: #007bff;
      color: white;
    }

    .btn-secondary {
      background-color: #6c757d;
      border-color: #6c757d;
      color: white;
    }

    .btn-outline {
      background-color: transparent;
      border-color: #d9d9d9;
      color: #333;
    }

    .btn-sm {
      padding: 4px 8px;
      font-size: 12px;
    }

    .error-message {
      position: fixed;
      top: 20px;
      right: 20px;
      z-index: 1000;
    }

    .error-content {
      display: flex;
      align-items: center;
      gap: 12px;
      padding: 12px 16px;
      background: #fff2f0;
      border: 1px solid #ffccc7;
      border-radius: 6px;
      color: #a8071a;
      box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
    }
  `]
})
export class SystemMonitorComponent implements OnInit, OnDestroy {
  // 系统状态
  systemStatus: SystemStatus | null = null;
  lastUpdateTime = new Date();
  isLoading = false;
  error: string | null = null;

  // 性能指标
  metrics: SystemMetrics = {
    cpuUsage: 0,
    memoryUsage: 0,
    diskUsage: 0,
    networkStatus: 'unknown',
    lastUpdateTime: new Date().toISOString()
  };

  // 性能历史数据
  performanceHistory: PerformanceData[] = [];
  maxActiveTests = 10;
  maxResponseTime = 1000;

  // 日志管理
  logs: Array<{timestamp: string, level: string, message: string}> = [];
  filteredLogs: Array<{timestamp: string, level: string, message: string}> = [];
  logLevel = '';

  // 订阅管理
  private subscriptions: Subscription[] = [];
  private startTime = new Date();

  constructor(
    private router: Router,
    private tauriApi: TauriApiService
  ) {}

  ngOnInit() {
    this.loadSystemData();
    this.startRealTimeMonitoring();
    this.initializeLogs();
  }

  ngOnDestroy() {
    this.subscriptions.forEach(sub => sub.unsubscribe());
  }

  // 加载系统数据
  async loadSystemData() {
    this.isLoading = true;
    this.error = null;

    try {
      // 获取系统状态
      const systemStatus = await this.tauriApi.getSystemStatus().toPromise();
      this.systemStatus = systemStatus || null;
      this.lastUpdateTime = new Date();

      // 模拟性能指标（实际应该从后端获取）
      this.updateMetrics();
      
      this.addLog('info', '系统数据加载完成');
    } catch (error) {
      this.error = '加载系统数据失败: ' + (error as Error).message;
      this.addLog('error', '系统数据加载失败: ' + (error as Error).message);
    } finally {
      this.isLoading = false;
    }
  }

  // 开始实时监控
  startRealTimeMonitoring() {
    // 每5秒更新一次系统状态
    const statusSubscription = interval(5000).subscribe(() => {
      this.loadSystemData();
    });

    // 每2秒更新一次性能数据
    const metricsSubscription = interval(2000).subscribe(() => {
      this.updateMetrics();
      this.updatePerformanceHistory();
    });

    this.subscriptions.push(statusSubscription, metricsSubscription);
  }

  // 更新性能指标
  updateMetrics() {
    // 模拟性能数据（实际应该从系统API获取）
    this.metrics = {
      cpuUsage: Math.round(Math.random() * 30 + 10), // 10-40%
      memoryUsage: Math.round(Math.random() * 20 + 40), // 40-60%
      diskUsage: Math.round(Math.random() * 10 + 70), // 70-80%
      networkStatus: Math.random() > 0.1 ? 'connected' : 'disconnected',
      lastUpdateTime: new Date().toISOString()
    };
  }

  // 更新性能历史数据
  updatePerformanceHistory() {
    const newData: PerformanceData = {
      timestamp: new Date().toISOString(),
      activeTests: this.systemStatus?.active_test_tasks || 0,
      responseTime: Math.round(Math.random() * 200 + 50), // 50-250ms
      errorRate: Math.round(Math.random() * 5) // 0-5%
    };

    this.performanceHistory.push(newData);
    
    // 只保留最近50个数据点
    if (this.performanceHistory.length > 50) {
      this.performanceHistory.shift();
    }

    // 更新最大值用于图表缩放
    this.maxActiveTests = Math.max(this.maxActiveTests, newData.activeTests);
    this.maxResponseTime = Math.max(this.maxResponseTime, newData.responseTime);
  }

  // 初始化日志
  initializeLogs() {
    this.addLog('info', '系统监控组件已启动');
    this.addLog('info', '开始实时监控系统状态');
  }

  // 添加日志
  addLog(level: string, message: string) {
    const log = {
      timestamp: new Date().toISOString(),
      level: level,
      message: message
    };
    
    this.logs.unshift(log);
    
    // 只保留最近100条日志
    if (this.logs.length > 100) {
      this.logs.pop();
    }
    
    this.filterLogs();
  }

  // 筛选日志
  filterLogs() {
    if (!this.logLevel) {
      this.filteredLogs = [...this.logs];
    } else {
      this.filteredLogs = this.logs.filter(log => log.level === this.logLevel);
    }
  }

  // 清除日志
  clearLogs() {
    this.logs = [];
    this.filteredLogs = [];
    this.addLog('info', '日志已清除');
  }

  // 刷新数据
  refreshData() {
    this.loadSystemData();
    this.addLog('info', '手动刷新系统数据');
  }

  // 获取系统健康状态文本
  getSystemHealthText(): string {
    if (!this.systemStatus?.system_health) return '未知';
    return this.systemStatus.system_health === 'healthy' ? '正常' : '异常';
  }

  // 获取网络状态样式类
  getNetworkStatusClass(): string {
    return this.metrics.networkStatus === 'connected' ? 'connected' : 'disconnected';
  }

  // 格式化运行时间
  formatUptime(): string {
    const now = new Date();
    const diff = now.getTime() - this.startTime.getTime();
    const hours = Math.floor(diff / (1000 * 60 * 60));
    const minutes = Math.floor((diff % (1000 * 60 * 60)) / (1000 * 60));
    return `${hours}小时${minutes}分钟`;
  }

  // 清除错误
  clearError() {
    this.error = null;
  }

  // 导航方法
  goToDashboard() {
    this.router.navigate(['/dashboard']);
  }
} 