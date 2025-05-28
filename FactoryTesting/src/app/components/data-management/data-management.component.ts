import { Component, OnInit, OnDestroy } from '@angular/core';
import { CommonModule } from '@angular/common';
import { FormsModule } from '@angular/forms';
import { RouterModule } from '@angular/router';
import { Subscription } from 'rxjs';

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
import { DataStateService, ImportState } from '../../services/data-state.service';
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
export class DataManagementComponent implements OnInit, OnDestroy {

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

  // 订阅管理
  private subscriptions: Subscription[] = [];

  constructor(
    private message: NzMessageService, 
    private tauriApiService: TauriApiService, 
    private testPlcConfigService: TestPlcConfigService,
    private dataStateService: DataStateService
  ) {}

  ngOnInit(): void {
    // 不加载历史数据，确保应用启动时没有预设数据
    this.subscribeToImportState();
  }

  ngOnDestroy(): void {
    this.subscriptions.forEach(sub => sub.unsubscribe());
  }

  // 订阅导入状态
  private subscribeToImportState(): void {
    const subscription = this.dataStateService.importState$.subscribe(state => {
      this.currentStep = state.currentStep;
      this.importProgress = state.importProgress;
      this.importResult = state.importResult;
      this.isImporting = state.isImporting;
      // 不恢复selectedFile，因为File对象无法序列化
    });
    this.subscriptions.push(subscription);
  }

  // 加载历史数据 - 仅在用户明确请求时加载
  loadHistoricalData(): void {
    // 清空历史数据，不提供任何预设数据
    this.historicalData = [];
  }

  // 文件选择处理
  onFileSelected(event: any): void {
    const file = event.target.files[0];
    if (file) {
      this.selectedFile = file;
      // 更新状态但不自动跳转步骤
      this.dataStateService.updateImportState({ 
        selectedFile: file 
      });
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
    this.dataStateService.updateImportState({
      isImporting: true,
      currentStep: 1,
      importProgress: 0
    });

    console.log('导入开始，切换到步骤1');

    try {
      // 优化导入进度 - 加快一倍速度
      const progressInterval = setInterval(() => {
        const currentProgress = this.dataStateService.getCurrentImportState().importProgress;
        const newProgress = currentProgress + 10; // 每次增加10%
        
        this.dataStateService.updateImportState({
          importProgress: newProgress
        });
        
        console.log('导入进度:', newProgress + '%');
        
        // 在某些关键点添加稍长的停顿，模拟真实的处理过程
        if (newProgress === 30) {
          setTimeout(() => {}, 150); // 模拟文件解析
        } else if (newProgress === 80) {
          setTimeout(() => {}, 250); // 模拟数据验证
        }
        
        if (newProgress >= 100) {
          clearInterval(progressInterval);
          console.log('导入进度完成，准备切换到完成步骤');
          // 稍微延迟完成，让用户看到100%
          setTimeout(() => {
            this.completeImport();
          }, 150);
        }
      }, 200); // 200ms间隔
      
    } catch (error) {
      this.message.error('导入失败');
      console.error('导入错误:', error);
      this.dataStateService.updateImportState({
        isImporting: false,
        currentStep: 0
      });
    }
  }

  // 完成导入
  completeImport(): void {
    if (!this.selectedFile) {
      this.message.error('没有选择文件');
      return;
    }

    // 调用真实的后端Excel解析服务
    console.log('调用后端Excel解析服务:', this.selectedFile.name);
    
    this.tauriApiService.parseExcelAndCreateBatch(this.selectedFile.name, this.selectedFile.name)
      .subscribe({
        next: (result) => {
          console.log('后端Excel解析结果:', result);
          
          // 使用后端返回的真实结果
          const importResult = {
            success: true,
            totalChannels: result.total_channels || result.channel_count || 0,
            message: result.message || '数据导入成功',
            timestamp: new Date().toISOString(),
            batchInfo: result.batch_info || {
              batch_id: result.batch_id,
              product_model: result.product_model || this.extractProductModel(),
              serial_number: result.serial_number || this.generateSerialNumber(),
              creation_time: new Date().toISOString(),
              total_points: result.total_channels || result.channel_count || 0,
              tested_points: 0,
              passed_points: 0,
              failed_points: 0,
              status_summary: '已创建，等待测试'
            },
            // 如果后端返回了分配结果，使用真实结果
            allocationResult: result.allocation_result || null
          };
          
          this.dataStateService.updateImportState({
            isImporting: false,
            currentStep: 2,
            importResult: importResult
          });
          
          this.message.success(`数据导入完成：${result.message || '成功解析Excel文件'}`);
        },
        error: (error) => {
          console.error('后端Excel解析失败:', error);
          
          // 只有在后端服务不可用时才显示错误
          if (this.tauriApiService.isTauriEnvironment()) {
            this.message.error(`Excel解析失败: ${error.message || error}`);
            this.dataStateService.updateImportState({
              isImporting: false,
              currentStep: 0
            });
          } else {
            // 开发环境：提示用户需要启动后端服务
            this.message.warning('开发环境：需要启动Tauri后端服务才能解析Excel文件');
            this.dataStateService.updateImportState({
              isImporting: false,
              currentStep: 0
            });
          }
        }
      });
  }

  // 自动分配逻辑
  private async performAutoAllocation(): Promise<void> {
    // 只有在Tauri环境中才执行分配逻辑
    if (!this.tauriApiService.isTauriEnvironment()) {
      console.log('开发环境：跳过自动分配');
      return;
    }

    try {
      console.log('开始执行自动分配逻辑...');
      
      // 调用真实的后端自动分配服务
      console.log('使用Tauri后端服务进行自动分配...');
      
      // 使用现有的导入Excel并分配通道命令
      const filePath = this.selectedFile?.name || 'imported_data.xlsx';
      const productModel = this.importResult.batchInfo.product_model;
      const serialNumber = this.importResult.batchInfo.serial_number;
      
      const allocationResult = await this.tauriApiService.importExcelAndAllocateChannels(
        filePath,
        productModel,
        serialNumber
      ).toPromise();
      
      console.log('后端自动分配结果:', allocationResult);
      
      if (allocationResult && allocationResult.success) {
        this.message.success(`自动分配完成：成功分配 ${allocationResult.allocated_count || 0} 个通道`);
        
        // 更新导入结果
        const updatedResult = {
          ...this.importResult,
          allocationResult: allocationResult
        };
        updatedResult.batchInfo.allocated_count = allocationResult.allocated_count || 0;
        updatedResult.batchInfo.batch_id = allocationResult.batch_id || updatedResult.batchInfo.batch_id;
        
        this.dataStateService.updateImportState({
          importResult: updatedResult
        });
      } else {
        throw new Error(allocationResult?.message || '分配失败');
      }
    } catch (error) {
      console.error('自动分配失败:', error);
      this.message.warning('自动分配失败，请在测试区域手动查看批次信息');
    }
  }

  // 智能分配逻辑 - 调用后端服务进行真正的Excel解析和通道分配
  private async performIntelligentAllocation(): Promise<void> {
    console.log('执行基于后端服务的智能分配逻辑...');
    
    try {
      if (!this.selectedFile) {
        throw new Error('没有选择Excel文件');
      }
      
      console.log(`开始调用后端服务解析Excel文件: ${this.selectedFile.name}`);
      
      // 只有在Tauri环境中才调用后端服务
      if (!this.tauriApiService.isTauriEnvironment()) {
        console.log('开发环境：跳过后端Excel解析服务调用');
        return;
      }
      
      // 调用真实的后端Excel解析和分配服务
      const filePath = this.selectedFile.name;
      const productModel = this.importResult.batchInfo.product_model;
      const serialNumber = this.importResult.batchInfo.serial_number;
      
      console.log('调用后端importExcelAndAllocateChannels服务...');
      const allocationResult = await this.tauriApiService.importExcelAndAllocateChannels(
        filePath,
        productModel,
        serialNumber
      ).toPromise();
      
      console.log('后端分配结果:', allocationResult);
      
      if (allocationResult && allocationResult.success) {
        // 使用后端返回的真实分配结果
        const updatedResult = {
          ...this.importResult,
          allocationResult: {
            success: true,
            allocated_count: allocationResult.allocated_count,
            conflict_count: allocationResult.conflict_count || 0,
            total_count: allocationResult.total_count || this.importResult.totalChannels,
            total_batches: allocationResult.total_batches || 1,
            message: allocationResult.message || '智能分配完成',
            allocation_details: {
              source: 'backend_service',
              excel_file_name: this.selectedFile.name,
              allocation_algorithm: '后端Excel解析和通道分配服务',
              backend_result: allocationResult
            }
          }
        };
        
        updatedResult.batchInfo.allocated_count = allocationResult.allocated_count;
        updatedResult.batchInfo.conflict_count = allocationResult.conflict_count || 0;
        updatedResult.batchInfo.total_batches = allocationResult.total_batches || 1;
        updatedResult.batchInfo.batch_id = allocationResult.batch_id || updatedResult.batchInfo.batch_id;
        
        this.dataStateService.updateImportState({
          importResult: updatedResult
        });
        
        this.message.success(`智能分配完成：${allocationResult.message || '成功分配通道'}`);
      } else {
        throw new Error(allocationResult?.message || '后端分配服务返回失败');
      }
      
    } catch (error) {
      console.error('智能分配过程中发生错误:', error);
      // 在Tauri环境中才显示错误，开发环境中静默处理
      if (this.tauriApiService.isTauriEnvironment()) {
        this.message.warning('智能分配失败，请在测试区域手动查看批次信息');
      }
      throw error;
    }
  }

  // 生成批次信息 - 修正Excel列映射关系
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
      status_summary: '已创建，等待测试',
      // 添加Excel列映射说明
      excel_column_mapping: {
        '变量名称(HMI)': '点位名称',
        '变量描述': '通道位号', 
        '通道位号': '被测PLC通道号',
        'channel_address': '测试PLC通道号'
      }
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
    this.selectedFile = null;
    this.dataStateService.resetImportState();
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
  }

  // 获取分配率
  getAllocationRate(): number {
    if (!this.importResult?.allocationResult) {
      return 0;
    }
    
    const total = this.importResult.allocationResult.total_count || this.importResult.totalChannels;
    const allocated = this.importResult.allocationResult.allocated_count || 0;
    
    if (total === 0) {
      return 0;
    }
    
    return Math.round((allocated / total) * 100);
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