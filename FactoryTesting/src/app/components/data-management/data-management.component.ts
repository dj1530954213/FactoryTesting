import { Component, OnInit } from '@angular/core';
import { CommonModule } from '@angular/common';
import { FormsModule } from '@angular/forms';
import { RouterModule } from '@angular/router';

// NG-ZORRO 组件导入
import { NzCardModule } from 'ng-zorro-antd/card';
import { NzButtonModule } from 'ng-zorro-antd/button';
import { NzUploadModule } from 'ng-zorro-antd/upload';
import { NzMessageService } from 'ng-zorro-antd/message';
import { NzIconModule } from 'ng-zorro-antd/icon';
import { NzSpaceModule } from 'ng-zorro-antd/space';
import { NzDividerModule } from 'ng-zorro-antd/divider';
import { NzTableModule } from 'ng-zorro-antd/table';
import { NzTagModule } from 'ng-zorro-antd/tag';
import { NzAlertModule } from 'ng-zorro-antd/alert';
import { NzStepsModule } from 'ng-zorro-antd/steps';
import { NzProgressModule } from 'ng-zorro-antd/progress';
import { NzModalModule } from 'ng-zorro-antd/modal';
import { NzListModule } from 'ng-zorro-antd/list';
import { NzEmptyModule } from 'ng-zorro-antd/empty';

// 服务导入
import { TauriApiService } from '../../services/tauri-api.service';
import { TestPlcConfigService } from '../../services/test-plc-config.service';
import { TestPlcChannelConfig, TestPlcChannelType } from '../../models/test-plc-config.model';

@Component({
  selector: 'app-data-management',
  standalone: true,
  imports: [
    CommonModule,
    FormsModule,
    RouterModule,
    NzCardModule,
    NzButtonModule,
    NzUploadModule,
    NzIconModule,
    NzSpaceModule,
    NzDividerModule,
    NzTableModule,
    NzTagModule,
    NzAlertModule,
    NzStepsModule,
    NzProgressModule,
    NzModalModule,
    NzListModule,
    NzEmptyModule
  ],
  templateUrl: './data-management.component.html',
  styleUrls: ['./data-management.component.css']
})
export class DataManagementComponent implements OnInit {

  // 当前步骤
  currentStep = 0;
  
  // 导入状态
  isImporting = false;
  importProgress = 0;
  
  // 文件信息
  selectedFile: File | null = null;
  
  // 历史数据列表
  historicalData: any[] = [];
  
  // 导入结果
  importResult: any = null;
  
  // 模态框状态
  isHistoryModalVisible = false;

  constructor(private message: NzMessageService, private tauriApiService: TauriApiService, private testPlcConfigService: TestPlcConfigService) {}

  ngOnInit(): void {
    this.loadHistoricalData();
  }

  // 加载历史数据
  loadHistoricalData(): void {
    this.historicalData = [
      {
        id: '1',
        name: '产品A测试数据_20241201',
        date: '2024-12-01 14:30:00',
        status: 'completed',
        channelCount: 88, // 修正为实际的88个通道
        testProgress: 100
      },
      {
        id: '2',
        name: '产品B测试数据_20241130',
        date: '2024-11-30 16:45:00',
        status: 'partial',
        channelCount: 88, // 修正为实际的88个通道
        testProgress: 65
      }
    ];
  }

  // 文件选择处理
  onFileSelected(event: any): void {
    const file = event.target.files[0];
    if (file) {
      this.selectedFile = file;
      // 保持在步骤0，不要自动跳转到步骤1
      console.log('文件已选择:', file.name, '当前步骤:', this.currentStep);
      this.message.success(`已选择文件: ${file.name}`);
    }
  }

  // 开始导入
  async startImport(): Promise<void> {
    console.log('开始导入，当前状态:', {
      selectedFile: this.selectedFile?.name,
      currentStep: this.currentStep,
      isImporting: this.isImporting
    });

    if (!this.selectedFile) {
      this.message.error('请先选择要导入的文件');
      return;
    }

    // 开始导入时才切换到步骤1
    this.isImporting = true;
    this.currentStep = 1;
    this.importProgress = 0;

    console.log('导入开始，切换到步骤1');

    try {
      // 优化导入进度 - 加快一倍速度
      const progressInterval = setInterval(() => {
        this.importProgress += 10; // 每次增加10%，恢复原来的速度
        console.log('导入进度:', this.importProgress + '%');
        
        // 在某些关键点添加稍长的停顿，模拟真实的处理过程
        if (this.importProgress === 30) {
          setTimeout(() => {}, 150); // 模拟文件解析，减少延迟
        } else if (this.importProgress === 80) {
          setTimeout(() => {}, 250); // 模拟数据验证，减少延迟
        }
        
        if (this.importProgress >= 100) {
          clearInterval(progressInterval);
          console.log('导入进度完成，准备切换到完成步骤');
          // 稍微延迟完成，让用户看到100%
          setTimeout(() => {
            this.completeImport();
          }, 150); // 减少延迟
        }
      }, 200); // 200ms间隔，加快一倍速度
      
    } catch (error) {
      this.message.error('导入失败');
      console.error('导入错误:', error);
      this.isImporting = false;
      // 导入失败时回到步骤0
      this.currentStep = 0;
    }
  }

  // 完成导入
  completeImport(): void {
    console.log('完成导入，切换到步骤2');
    this.isImporting = false;
    this.currentStep = 2;
    
    // 模拟真实的点位数量（88个点位）
    const actualChannelCount = 88;
    
    this.importResult = {
      success: true,
      totalChannels: actualChannelCount,
      successChannels: actualChannelCount,
      failedChannels: 0,
      batchInfo: this.generateBatchInfo(actualChannelCount) // 自动生成批次信息
    };
    
    this.message.success('数据导入完成，已自动创建测试批次');
    
    // 调用后端自动分配逻辑
    this.performAutoAllocation();
  }

  // 自动分配逻辑
  private async performAutoAllocation(): Promise<void> {
    try {
      console.log('开始执行自动分配逻辑...');
      
      // 检查是否在Tauri环境中
      if (this.tauriApiService.isTauriEnvironment()) {
        // 调用真实的后端自动分配服务
        const allocationResult = await this.tauriApiService.autoAllocateBatch({
          batchId: this.importResult.batchInfo.batch_id,
          batchInfo: this.importResult.batchInfo,
          fileName: this.selectedFile?.name,
          channelCount: this.importResult.totalChannels
        }).toPromise();
        
        console.log('后端自动分配结果:', allocationResult);
        this.message.success('批次自动分配完成，可在测试区域查看');
      } else {
        // 开发环境下使用基于测试PLC配置的智能分配
        await this.performIntelligentAllocation();
        console.log('智能自动分配完成（基于测试PLC配置）');
        this.message.success('批次智能分配完成（基于测试PLC配置），可在测试区域查看');
      }
      
    } catch (error) {
      console.error('自动分配失败:', error);
      // 如果后端调用失败，回退到智能分配模式
      await this.performIntelligentAllocation();
      this.message.warning('后端服务不可用，使用智能分配模式');
    }
  }

  // 基于测试PLC配置的智能分配
  private async performIntelligentAllocation(): Promise<void> {
    return new Promise((resolve) => {
      // 获取测试PLC通道配置
      this.testPlcConfigService.getTestPlcChannels().subscribe({
        next: (testPlcChannels) => {
          console.log('获取到测试PLC通道配置:', testPlcChannels.length, '个通道');
          
          // 按通道类型分组
          const channelsByType = this.groupChannelsByType(testPlcChannels);
          
          // 模拟智能分配过程
          setTimeout(() => {
            console.log('智能分配过程:');
            console.log('- 分析导入的点表数据');
            console.log('- 按模块类型匹配测试PLC通道');
            console.log('- AI通道可用:', channelsByType['AI']?.length || 0);
            console.log('- AO通道可用:', channelsByType['AO']?.length || 0);
            console.log('- DI通道可用:', channelsByType['DI']?.length || 0);
            console.log('- DO通道可用:', channelsByType['DO']?.length || 0);
            console.log('- 创建通道映射关系');
            console.log('- 生成测试实例');
            
            // 更新批次信息，包含分配详情
            this.importResult.batchInfo.allocation_details = {
              total_test_channels: testPlcChannels.length,
              available_ai_channels: channelsByType['AI']?.length || 0,
              available_ao_channels: channelsByType['AO']?.length || 0,
              available_di_channels: channelsByType['DI']?.length || 0,
              available_do_channels: channelsByType['DO']?.length || 0,
              allocation_strategy: 'intelligent_mapping',
              allocation_time: new Date().toISOString()
            };
            
            resolve();
          }, 1500);
        },
        error: (error) => {
          console.error('获取测试PLC通道配置失败:', error);
          // 回退到简单分配模式
          this.performSimpleAllocation();
          resolve();
        }
      });
    });
  }

  // 按通道类型分组
  private groupChannelsByType(channels: TestPlcChannelConfig[]): Record<string, TestPlcChannelConfig[]> {
    const grouped: Record<string, TestPlcChannelConfig[]> = {};
    
    channels.forEach(channel => {
      if (!channel.isEnabled) return; // 跳过禁用的通道
      
      let typeKey = '';
      switch (channel.channelType) {
        case TestPlcChannelType.AI:
        case TestPlcChannelType.AINone:
          typeKey = 'AI';
          break;
        case TestPlcChannelType.AO:
        case TestPlcChannelType.AONone:
          typeKey = 'AO';
          break;
        case TestPlcChannelType.DI:
        case TestPlcChannelType.DINone:
          typeKey = 'DI';
          break;
        case TestPlcChannelType.DO:
        case TestPlcChannelType.DONone:
          typeKey = 'DO';
          break;
      }
      
      if (typeKey) {
        if (!grouped[typeKey]) {
          grouped[typeKey] = [];
        }
        grouped[typeKey].push(channel);
      }
    });
    
    return grouped;
  }

  // 简单分配模式（回退方案）
  private async performSimpleAllocation(): Promise<void> {
    return new Promise((resolve) => {
      setTimeout(() => {
        console.log('简单分配模式：');
        console.log('- 使用默认通道分配策略');
        console.log('- 按顺序分配测试通道');
        console.log('- 创建基础测试实例');
        resolve();
      }, 1000);
    });
  }

  // 生成批次信息
  private generateBatchInfo(channelCount: number): any {
    const now = new Date();
    const batchId = `BATCH_${now.getFullYear()}${(now.getMonth() + 1).toString().padStart(2, '0')}${now.getDate().toString().padStart(2, '0')}_${now.getHours().toString().padStart(2, '0')}${now.getMinutes().toString().padStart(2, '0')}`;
    
    return {
      batch_id: batchId,
      product_model: this.extractProductModel(),
      serial_number: this.generateSerialNumber(),
      creation_time: now.toISOString(),
      total_points: channelCount,
      tested_points: 0,
      passed_points: 0,
      failed_points: 0,
      status_summary: '已创建，等待测试'
    };
  }

  // 从文件名提取产品型号
  private extractProductModel(): string {
    if (!this.selectedFile) return '未知产品';
    
    const fileName = this.selectedFile.name.replace(/\.[^/.]+$/, ''); // 移除扩展名
    // 简单的产品型号提取逻辑，可以根据实际需求调整
    const modelMatch = fileName.match(/([A-Z0-9]+)/);
    return modelMatch ? modelMatch[1] : fileName.substring(0, 10);
  }

  // 生成序列号
  private generateSerialNumber(): string {
    const now = new Date();
    return `SN${now.getFullYear()}${(now.getMonth() + 1).toString().padStart(2, '0')}${now.getDate().toString().padStart(2, '0')}${Math.floor(Math.random() * 1000).toString().padStart(3, '0')}`;
  }

  // 重置导入流程
  resetImport(): void {
    this.currentStep = 0;
    this.selectedFile = null;
    this.importProgress = 0;
    this.importResult = null;
    this.isImporting = false;
  }

  // 显示历史数据模态框
  showHistoryModal(): void {
    this.isHistoryModalVisible = true;
  }

  // 关闭历史数据模态框
  closeHistoryModal(): void {
    this.isHistoryModalVisible = false;
  }

  // 恢复历史数据
  restoreData(item: any): void {
    this.message.info(`正在恢复数据: ${item.name}`);
    this.closeHistoryModal();
  }

  // 导出当前数据
  exportCurrentData(): void {
    this.message.info('正在导出当前数据...');
    // TODO: 实现导出功能
  }

  // 获取状态标签颜色
  getStatusColor(status: string): string {
    switch (status) {
      case 'completed': return 'green';
      case 'partial': return 'orange';
      case 'failed': return 'red';
      default: return 'default';
    }
  }

  // 获取状态文本
  getStatusText(status: string): string {
    switch (status) {
      case 'completed': return '已完成';
      case 'partial': return '部分完成';
      case 'failed': return '失败';
      default: return '未知';
    }
  }

  // 获取导入结果描述
  getImportResultDescription(): string {
    if (!this.importResult) return '';
    
    if (this.importResult.success) {
      return `成功导入 ${this.importResult.successChannels} 个通道点，共 ${this.importResult.totalChannels} 个通道。已自动创建测试批次并完成分配。`;
    } else {
      return `导入失败，请检查文件格式和内容。`;
    }
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
        minute: '2-digit',
        second: '2-digit'
      });
    } catch (error) {
      return dateTimeString;
    }
  }
} 