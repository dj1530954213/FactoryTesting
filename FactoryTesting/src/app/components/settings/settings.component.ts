import { Component, OnInit, OnDestroy } from '@angular/core';
import { CommonModule } from '@angular/common';
import { FormsModule } from '@angular/forms';
import { Router } from '@angular/router';
import { Subscription } from 'rxjs';
import { TauriApiService } from '../../services/tauri-api.service';
import { AppSettings } from '../../models';

@Component({
  selector: 'app-settings',
  standalone: true,
  imports: [CommonModule, FormsModule],
  templateUrl: './settings.component.html',
  styleUrl: './settings.component.css'
})
export class SettingsComponent implements OnInit, OnDestroy {
  // 配置数据
  settings: AppSettings = {
    id: 'default_settings',
    theme: 'light',
    plc_ip_address: '127.0.0.1',
    plc_port: 502,
    default_operator_name: undefined,
    auto_save_interval_minutes: 5,
    recent_projects: [],
    last_backup_time: undefined
  };

  // 界面状态
  isLoading = false;
  isSaving = false;
  error: string | null = null;
  successMessage: string | null = null;
  hasUnsavedChanges = false;

  // 原始配置（用于检测变更）
  private originalSettings: AppSettings | null = null;

  // 主题选项
  themeOptions = [
    { value: 'light', label: '浅色主题' },
    { value: 'dark', label: '深色主题' },
    { value: 'auto', label: '跟随系统' }
  ];

  // 自动保存间隔选项（分钟）
  autoSaveOptions = [
    { value: 1, label: '1分钟' },
    { value: 5, label: '5分钟' },
    { value: 10, label: '10分钟' },
    { value: 30, label: '30分钟' },
    { value: 0, label: '禁用自动保存' }
  ];

  // 订阅管理
  private subscriptions: Subscription[] = [];

  constructor(
    private tauriApi: TauriApiService,
    private router: Router
  ) {}

  ngOnInit() {
    this.loadSettings();
  }

  ngOnDestroy() {
    // 清理订阅
    this.subscriptions.forEach(sub => sub.unsubscribe());
  }

  /**
   * 加载应用配置
   */
  loadSettings() {
    this.isLoading = true;
    this.error = null;

    const subscription = this.tauriApi.loadAppSettings().subscribe({
      next: (settings) => {
        this.settings = { ...settings };
        this.originalSettings = { ...settings };
        this.hasUnsavedChanges = false;
        console.log('应用配置加载成功:', settings);
      },
      error: (error) => {
        this.error = '加载应用配置失败: ' + error.message;
        console.error('加载应用配置失败:', error);
      },
      complete: () => {
        this.isLoading = false;
      }
    });

    this.subscriptions.push(subscription);
  }

  /**
   * 保存应用配置
   */
  saveSettings() {
    if (!this.hasUnsavedChanges) {
      this.showSuccessMessage('配置没有变更');
      return;
    }

    this.isSaving = true;
    this.error = null;
    this.successMessage = null;

    const subscription = this.tauriApi.saveAppSettings(this.settings).subscribe({
      next: () => {
        this.originalSettings = { ...this.settings };
        this.hasUnsavedChanges = false;
        this.showSuccessMessage('应用配置保存成功');
        console.log('应用配置保存成功');
      },
      error: (error) => {
        this.error = '保存应用配置失败: ' + error.message;
        console.error('保存应用配置失败:', error);
      },
      complete: () => {
        this.isSaving = false;
      }
    });

    this.subscriptions.push(subscription);
  }

  /**
   * 重置为默认配置
   */
  resetToDefaults() {
    if (confirm('确定要重置为默认配置吗？这将丢失所有自定义设置。')) {
      this.settings = {
        id: 'default_settings',
        theme: 'light',
        plc_ip_address: '127.0.0.1',
        plc_port: 502,
        default_operator_name: undefined,
        auto_save_interval_minutes: 5,
        recent_projects: [],
        last_backup_time: undefined
      };
      this.checkForChanges();
    }
  }

  /**
   * 取消更改
   */
  cancelChanges() {
    if (this.originalSettings) {
      this.settings = { ...this.originalSettings };
      this.hasUnsavedChanges = false;
      this.showSuccessMessage('已取消更改');
    }
  }

  /**
   * 检测配置变更
   */
  onSettingsChange() {
    this.checkForChanges();
    this.clearMessages();
  }

  /**
   * 检查是否有未保存的变更
   */
  private checkForChanges() {
    if (!this.originalSettings) {
      this.hasUnsavedChanges = false;
      return;
    }

    this.hasUnsavedChanges = JSON.stringify(this.settings) !== JSON.stringify(this.originalSettings);
  }

  /**
   * 清除最近项目列表
   */
  clearRecentProjects() {
    if (confirm('确定要清除最近项目列表吗？')) {
      this.settings.recent_projects = [];
      this.checkForChanges();
    }
  }

  /**
   * 删除最近项目
   */
  removeRecentProject(index: number) {
    this.settings.recent_projects.splice(index, 1);
    this.checkForChanges();
  }

  /**
   * 测试PLC连接
   */
  testPlcConnection() {
    if (!this.settings.plc_ip_address || !this.settings.plc_port) {
      this.error = '请先设置PLC IP地址和端口';
      return;
    }

    // 这里可以添加实际的PLC连接测试逻辑
    this.showSuccessMessage(`PLC连接测试: ${this.settings.plc_ip_address}:${this.settings.plc_port} (模拟测试)`);
  }

  /**
   * 导航到其他页面
   */
  goToDashboard() {
    if (this.hasUnsavedChanges) {
      if (confirm('有未保存的更改，确定要离开吗？')) {
        this.router.navigate(['/dashboard']);
      }
    } else {
      this.router.navigate(['/dashboard']);
    }
  }

  /**
   * 显示成功消息
   */
  private showSuccessMessage(message: string) {
    this.successMessage = message;
    setTimeout(() => {
      this.successMessage = null;
    }, 3000);
  }

  /**
   * 清除消息
   */
  private clearMessages() {
    this.error = null;
    this.successMessage = null;
  }

  /**
   * 获取主题标签
   */
  getThemeLabel(value: string): string {
    const option = this.themeOptions.find(opt => opt.value === value);
    return option ? option.label : value;
  }

  /**
   * 获取自动保存间隔标签
   */
  getAutoSaveLabel(value: number): string {
    const option = this.autoSaveOptions.find(opt => opt.value === value);
    return option ? option.label : `${value}分钟`;
  }

  /**
   * 格式化最后备份时间
   */
  formatLastBackupTime(): string {
    if (!this.settings.last_backup_time) {
      return '从未备份';
    }
    return new Date(this.settings.last_backup_time).toLocaleString('zh-CN');
  }
}
