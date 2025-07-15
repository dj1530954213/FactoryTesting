import { Component, OnInit, OnDestroy } from '@angular/core';
import { CommonModule } from '@angular/common';
import { FormsModule } from '@angular/forms';
import { RouterModule } from '@angular/router';
import { Subscription } from 'rxjs';

// Tauri API 导入
import { open } from '@tauri-apps/plugin-dialog';
import { invoke } from '@tauri-apps/api/core';

// NG-ZORRO 组件导入
import { NzCardModule } from 'ng-zorro-antd/card';
import { NzButtonModule } from 'ng-zorro-antd/button';
import { NzMessageService } from 'ng-zorro-antd/message';
import { NzIconModule } from 'ng-zorro-antd/icon';
import { NzSpaceModule } from 'ng-zorro-antd/space';
import { NzTableModule } from 'ng-zorro-antd/table';
import { NzTagModule } from 'ng-zorro-antd/tag';
import { NzAlertModule } from 'ng-zorro-antd/alert';
import { NzInputModule } from 'ng-zorro-antd/input';
import { NzSelectModule } from 'ng-zorro-antd/select';
import { NzFormModule } from 'ng-zorro-antd/form';
import { NzProgressModule } from 'ng-zorro-antd/progress';
import { NzModalModule } from 'ng-zorro-antd/modal';
import { NzStepsModule } from 'ng-zorro-antd/steps';
import { NzResultModule } from 'ng-zorro-antd/result';
import { NzSpinModule } from 'ng-zorro-antd/spin';
import { NzStatisticModule } from 'ng-zorro-antd/statistic';
import { NzDescriptionsModule } from 'ng-zorro-antd/descriptions';
import { NzListModule } from 'ng-zorro-antd/list';
import { NzEmptyModule } from 'ng-zorro-antd/empty';
import { NzDropDownModule } from 'ng-zorro-antd/dropdown';
import { NzToolTipModule } from 'ng-zorro-antd/tooltip';
import { NzPopconfirmModule } from 'ng-zorro-antd/popconfirm';

// 服务导入
import { TauriApiService } from '../../services/tauri-api.service';
import { TestPlcConfigService } from '../../services/test-plc-config.service';
import { DataStateService, ImportState } from '../../services/data-state.service';
import { TestPlcChannelConfig, TestPlcChannelType } from '../../models/test-plc-config.model';
import { BatchSelectionService } from '../../services/batch-selection.service';

@Component({
  selector: 'app-data-management',
  standalone: true,
  imports: [
    CommonModule,
    FormsModule,
    RouterModule,
    NzCardModule,
    NzButtonModule,
    NzIconModule,
    NzSpaceModule,
    NzTableModule,
    NzTagModule,
    NzAlertModule,
    NzInputModule,
    NzSelectModule,
    NzFormModule,
    NzProgressModule,
    NzModalModule,
    NzStepsModule,
    NzResultModule,
    NzSpinModule,
    NzStatisticModule,
    NzDescriptionsModule,
    NzListModule,
    NzEmptyModule,
    NzDropDownModule,
    NzToolTipModule,
    NzPopconfirmModule
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
  selectedFile: any = null;  // 支持File和模拟对象
  selectedFilePath: string = '';  // 存储完整文件路径
  
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
    private dataStateService: DataStateService,
    private batchSelectionService: BatchSelectionService
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

  // 使用Tauri文件对话框选择文件
  async selectFileWithDialog(): Promise<void> {
    try {
      // 强制重新检测Tauri环境
      console.log('=== 开始文件选择操作 ===');
      console.log('强制重新检测Tauri环境...');
      
      // 先检查基本的Tauri对象
      console.log('检查window对象:', typeof window);
      console.log('检查__TAURI__:', !!(window as any).__TAURI__);
      console.log('检查__TAURI_INTERNALS__:', !!(window as any).__TAURI_INTERNALS__);
      console.log('检查__TAURI_METADATA__:', !!(window as any).__TAURI_METADATA__);
      
      // 检查invoke函数
      try {
        console.log('检查invoke函数:', typeof invoke);
        console.log('invoke函数来源:', invoke);
      } catch (e) {
        console.log('invoke函数检查失败:', e);
      }
      
      // 检查当前环境信息
      console.log('当前URL:', window.location.href);
      console.log('当前协议:', window.location.protocol);
      console.log('当前主机:', window.location.hostname);
      console.log('当前端口:', window.location.port);
      console.log('用户代理:', navigator.userAgent);
      
      const isTauriEnv = this.tauriApiService.forceRedetectEnvironment();
      console.log('环境检测结果:', isTauriEnv);
      
      // 如果检测失败，尝试直接调用open函数来验证
      if (!isTauriEnv) {
        console.log('检测到非Tauri环境，尝试直接调用open函数验证...');
        try {
          // 尝试直接调用open函数
          console.log('尝试调用open函数:', typeof open);
          if (typeof open === 'function') {
            console.log('open函数存在，可能是Tauri环境，继续执行...');
          } else {
            console.log('open函数不存在，确认为非Tauri环境');
            this.message.warning('文件对话框功能仅在Tauri环境中可用，请使用文件上传按钮');
            return;
          }
        } catch (e) {
          console.log('open函数调用测试失败:', e);
          this.message.warning('文件对话框功能仅在Tauri环境中可用，请使用文件上传按钮');
          return;
        }
      }

      console.log('Tauri环境检测通过，尝试打开文件对话框...');
      
      // 使用Tauri文件对话框选择Excel文件
      const selected = await open({
        multiple: false,
        filters: [{
          name: 'Excel文件',
          extensions: ['xlsx', 'xls']
        }]
      });

      console.log('文件对话框返回结果:', selected);

      if (selected && typeof selected === 'string') {
        // 创建一个模拟的文件对象
        const fileName = selected.split('\\').pop() || selected.split('/').pop() || 'unknown.xlsx';
        this.selectedFile = {
          uid: Date.now().toString(),
          name: fileName,
          status: 'done',
          size: 0,
          type: 'application/vnd.openxmlformats-officedocument.spreadsheetml.sheet'
        };
        
        // 存储完整的文件路径
        this.selectedFilePath = selected;
        
        // 更新状态
        this.dataStateService.updateImportState({ 
          selectedFile: this.selectedFile 
        });
        
        console.log('文件选择成功:');
        console.log('  文件名:', fileName);
        console.log('  完整路径:', selected);
        this.message.success(`已选择文件: ${fileName}`);
      } else {
        console.log('用户取消了文件选择或选择结果无效');
      }
    } catch (error) {
      console.error('文件选择过程中发生错误:', error);
      
      // 如果是因为open函数不存在导致的错误，给出更明确的提示
      if ((error as Error).toString().includes('open is not defined') || (error as Error).toString().includes('Cannot read properties')) {
        console.log('确认为非Tauri环境，open函数不可用');
        this.message.warning('文件对话框功能仅在Tauri桌面应用中可用，请使用下方的文件上传按钮');
      } else {
        this.message.error(`文件选择失败: ${error}`);
      }
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

  // 完成导入 - 使用新的一键导入和创建批次方法
  completeImport(): void {
    if (!this.selectedFile) {
      this.message.error('没有选择文件');
      return;
    }

    // 🚀 使用新的一键导入Excel并创建批次服务
    console.log('🚀 调用新的一键导入Excel并创建批次服务:', this.selectedFile.name);

    // 使用完整的文件路径（如果有的话）或文件名
    const filePath = this.selectedFilePath || this.selectedFile.name;

    console.log('🚀 使用文件路径:', filePath);

    this.tauriApiService.importExcelAndCreateBatch(
      filePath,
      '自动导入批次',
      this.extractProductModel(),
      '系统操作员'
    ).subscribe({
      next: (result) => {
        console.log('🚀 后端一键导入和创建批次结果:', result);

        // 兼容新版后端返回结构
        const importResultRaw = result.import_result || result.importResult;
        const allocationResultRaw = result.allocation_result || result.allocationResult;

        // 如果两者都不存在，但包含 batch_info && instances，则视为 ImportAndPrepareBatchResponse
        const isPrepareResponse = !importResultRaw && !allocationResultRaw && result.batch_info && result.instances;

        if (isPrepareResponse) {
          console.log('检测到 ImportAndPrepareBatchResponse 响应结构');

          const batchInfoRawPrep = result.batch_info;
          const instancesPrep = result.instances;

          // 统计类型数量
          // 优先使用后端提供的 allocation_summary（Rust 端已新增字段）
          const allocSummaryPrep = result.allocation_summary || result.allocationSummary;
          let typeCountsPrep: any = { AI: 0, AO: 0, DI: 0, DO: 0 };

          if (allocSummaryPrep && allocSummaryPrep.ai_channels !== undefined) {
            // 数字字段命名兼容 snake / camel
            typeCountsPrep = {
              AI: allocSummaryPrep.ai_channels ?? allocSummaryPrep.aiChannels ?? 0,
              AO: allocSummaryPrep.ao_channels ?? allocSummaryPrep.aoChannels ?? 0,
              DI: allocSummaryPrep.di_channels ?? allocSummaryPrep.diChannels ?? 0,
              DO: allocSummaryPrep.do_channels ?? allocSummaryPrep.doChannels ?? 0,
            };
          } else {
            // 回退：根据实例的 test_plc_channel_tag 前缀推断（如 "AI1_1" → AI）
            instancesPrep.forEach((inst: any) => {
              let mt = inst.module_type || inst.moduleType;
              if (!mt && inst.test_plc_channel_tag) {
                mt = inst.test_plc_channel_tag.substring(0, 2).toUpperCase();
              }
              if (typeCountsPrep.hasOwnProperty(mt)) typeCountsPrep[mt]++;
            });
          }

          const totalPrep = instancesPrep.length;

          const importResult = {
            success: true,
            totalChannels: totalPrep,
            successChannels: totalPrep,
            failedChannels: 0,
            message: '导入并创建批次完成',
            timestamp: new Date().toISOString(),
            batchInfo: {
              batch_id: batchInfoRawPrep.batch_id,
              product_model: batchInfoRawPrep.product_model || this.extractProductModel(),
              serial_number: batchInfoRawPrep.serial_number || this.generateSerialNumber(),
              creation_time: batchInfoRawPrep.creation_time || new Date().toISOString(),
              total_points: totalPrep,
              tested_points: 0,
              passed_points: 0,
              failed_points: 0,
              status_summary: '已创建，等待测试',
            },
            isPersisted: true,
            definitions: [],
            allocationResult: {
              batches: [batchInfoRawPrep],
              allocated_instances: instancesPrep,
              allocation_summary: {
                ai_channels: typeCountsPrep.AI,
                ao_channels: typeCountsPrep.AO,
                di_channels: typeCountsPrep.DI,
                do_channels: typeCountsPrep.DO,
                total_channels: totalPrep,
              }
            }
          } as any;

          // 重置批次选择服务，避免混淆旧批次
          this.batchSelectionService.reset();

          this.dataStateService.updateImportState({
            isImporting: false,
            currentStep: 2,
            importResult: importResult
          });

          this.message.success(`一键导入完成：成功导入 ${totalPrep} 个通道，创建 1 个批次`);
          return; // 处理完毕
        }

        if (!importResultRaw || !allocationResultRaw) {
          console.error('后端返回结构不符合预期:', result);
          this.message.error('后端返回结构不符合预期，无法解析导入结果');
          return;
        }

        // 提取批次信息（取第一个批次）
        const batchInfoRaw = (allocationResultRaw.batches && allocationResultRaw.batches.length > 0)
          ? allocationResultRaw.batches[0]
          : (allocationResultRaw.batch_info || {});

        // 计算各类型数量
        const typeCounts: any = { AI: 0, AO: 0, DI: 0, DO: 0 };

        if (allocationResultRaw.allocation_summary && allocationResultRaw.allocation_summary.ai_channels !== undefined) {
          // 新版 numeric 统计
          typeCounts.AI = allocationResultRaw.allocation_summary.ai_channels || 0;
          typeCounts.AO = allocationResultRaw.allocation_summary.ao_channels || 0;
          typeCounts.DI = allocationResultRaw.allocation_summary.di_channels || 0;
          typeCounts.DO = allocationResultRaw.allocation_summary.do_channels || 0;
        } else if (importResultRaw.imported_definitions && importResultRaw.imported_definitions.length > 0) {
          // 回退：统计导入定义
          importResultRaw.imported_definitions.forEach((def: any) => {
            const t = def.module_type || def.moduleType;
            if (t && typeCounts.hasOwnProperty(t)) {
              typeCounts[t]++;
            }
          });
        }

        const totalChannels = importResultRaw.successful_imports || importResultRaw.total_rows || 0;

        const importResult = {
          success: true,
          totalChannels: totalChannels,
          successChannels: totalChannels,
          failedChannels: importResultRaw.failed_imports || 0,
          message: allocationResultRaw.message || '导入并创建批次完成',
          timestamp: new Date().toISOString(),
          batchInfo: {
            batch_id: batchInfoRaw.batch_id,
            product_model: batchInfoRaw.product_model || this.extractProductModel(),
            serial_number: batchInfoRaw.serial_number || this.generateSerialNumber(),
            creation_time: batchInfoRaw.creation_time || new Date().toISOString(),
            total_points: totalChannels,
            tested_points: 0,
            passed_points: 0,
            failed_points: 0,
            status_summary: '已创建，等待测试',
          },
          isPersisted: true,
          definitions: importResultRaw.imported_definitions,
          allocationResult: allocationResultRaw,
        };

        // 重置批次选择服务，避免混淆旧批次
        this.batchSelectionService.reset();

        this.dataStateService.updateImportState({
          isImporting: false,
          currentStep: 2,
          importResult: importResult
        });

        this.message.success(`一键导入完成：成功导入 ${totalChannels} 个通道，创建 ${allocationResultRaw.batches?.length || 1} 个批次`);
      },
      error: (error) => {
        console.error('🚀 后端一键导入失败:', error);

        // 只有在后端服务不可用时才显示错误
        if (this.tauriApiService.isTauriEnvironment()) {
          this.message.error(`一键导入失败: ${error.message || error}`);
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

  // 立即持久化数据 - 已废弃，使用新的一键导入方法
  persistDataNow(): void {
    this.message.warning('此功能已废弃，请使用新的一键导入功能');
    console.warn('persistDataNow() 方法已废弃，请使用 importExcelAndCreateBatch() 方法');

    // 如果用户真的需要持久化，引导他们重新导入
    this.message.info('请重新选择Excel文件并使用"开始导入"功能');
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
      
      // 使用完整的文件路径
      const filePath = this.selectedFilePath || this.selectedFile?.name || 'imported_data.xlsx';
      
      const productModel = this.importResult.batchInfo.product_model;
      const serialNumber = this.importResult.batchInfo.serial_number;
      
      const result = await this.tauriApiService.importExcelAndCreateBatch(
        filePath,
        '自动导入批次',
        productModel,
        '系统操作员'
      ).toPromise();

      if (!result) {
        throw new Error('后端服务返回空结果');
      }

      const allocationResult = result.allocation_result; // 提取分配结果
      
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
      
      // 使用完整的文件路径
      const filePath = this.selectedFilePath || this.selectedFile.name;
      
      const productModel = this.importResult.batchInfo.product_model;
      const serialNumber = this.importResult.batchInfo.serial_number;
      
      console.log('调用后端importExcelAndCreateBatch服务...');
      const result = await this.tauriApiService.importExcelAndCreateBatch(
        filePath,
        '自动导入批次',
        productModel,
        '系统操作员'
      ).toPromise();

      if (!result) {
        throw new Error('后端服务返回空结果');
      }

      const allocationResult = result.allocation_result; // 提取分配结果
      
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
    this.selectedFilePath = '';
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
      if (this.importResult.isPersisted) {
        return `成功解析并保存 ${this.importResult.successChannels} 个通道点，共 ${this.importResult.totalChannels} 个通道。已自动创建测试批次。`;
      } else {
        return `成功解析 ${this.importResult.successChannels} 个通道点，共 ${this.importResult.totalChannels} 个通道。数据已准备就绪，将在开始测试时保存。`;
      }
    } else {
      return `解析失败，请检查文件格式和内容。`;
    }
  }

  // 格式化日期时间
  formatDateTime(dateTimeString: string): string {
    try {
      let date: Date;
      // 无时区信息时按北京时间(+08:00)解析
      const plainPattern = /^\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2}$/;
      if (plainPattern.test(dateTimeString)) {
        date = new Date(dateTimeString.replace(' ', 'T') + '+08:00');
      } else {
        date = new Date(dateTimeString);
      }
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

  // 获取各类型通道数量（AI/AO/DI/DO）
  get channelCounts(): any {
    // 0) 首先尝试新版后端 numeric 字段 (ai_channels/ao_channels/di_channels/do_channels)
    const numericSummary = this.importResult?.allocationResult?.allocation_summary;
    if (numericSummary && numericSummary.ai_channels !== undefined) {
      return {
        AI: numericSummary.ai_channels || 0,
        AO: numericSummary.ao_channels || 0,
        DI: numericSummary.di_channels || 0,
        DO: numericSummary.do_channels || 0,
      };
    }

    // 1) 旧版后端 by_module_type 汇总 (snake / camel)
    let sum = this.importResult?.allocationResult?.allocation_summary?.by_module_type;
    if (!sum) {
      // 兼容驼峰命名
      sum = this.importResult?.allocationResult?.allocationSummary?.by_moduleType;
    }
    if (sum) {
      const c: any = { AI: 0, AO: 0, DI: 0, DO: 0 };
      Object.keys(sum).forEach(k => {
        const key = k as any;
        // 两种可能：直接是数字 或 嵌套 definition_count
        const val = typeof sum[key] === 'number' ? sum[key] : (sum[key]?.definition_count || 0);
        c[key] = val;
      });
      return c;
    }

    // 2) allocation_details.module_distribution
    const dist = this.importResult?.allocationResult?.allocation_details?.module_distribution;
    if (dist) return dist;

    // 3) 统计 definitions 数组
    let defs = this.importResult?.definitions;
    if (!defs) {
      // 兼容其他字段
      defs = this.importResult?.channel_definitions || this.importResult?.channelDefinitions;
    }
    if (defs && Array.isArray(defs)) {
      const counts: any = { AI: 0, AO: 0, DI: 0, DO: 0 };
      defs.forEach((d: any) => {
        const t = d.module_type || d.moduleType;
        if (counts.hasOwnProperty(t)) counts[t]++;
      });
      return counts;
    }
    return null;
  }

  // 获取批次数量（兼容多种返回格式）
  get batchCount(): number {
    // 0) 直接统计字段，兼容 snake / camel 命名（通常最准确）
    const ar: any = this.importResult?.allocationResult || {};
    const firstCandidates = [
      ar.total_batches,
      ar.totalBatches,
      ar.batch_count,
      ar.batchCount
    ];
    for (const v of firstCandidates) {
      if (typeof v === 'number' && v > 0) {
        return v;
      }
    }

    // 1) 批次数组长度（次优）
    const len = ar.batches?.length;
    if (typeof len === 'number' && len > 0) {
      return len;
    }

    // 2) 尝试批次信息中的字段
    const bi: any = this.importResult?.batchInfo || {};
    const biCandidates = [bi.total_batches, bi.totalBatches, bi.batch_count, bi.batchCount];
    for (const v of biCandidates) {
      if (typeof v === 'number' && v > 0) {
        return v;
      }
    }

    // 3) 未获取到有效批次数量时返回 0，避免误导
    return 0;
  }
}