import { Component, OnInit, OnDestroy } from '@angular/core';
import { CommonModule } from '@angular/common';
import { FormsModule } from '@angular/forms';
import { Router } from '@angular/router';
import { TauriApiService } from '../../services/tauri-api.service';

// 导入Tauri API
declare global {
  interface Window {
    __TAURI__: any;
  }
}

interface RecentFile {
  name: string;
  path: string;
  lastUsed: Date;
}

interface PreviewDataItem {
  tag: string;
  description: string;
  moduleType: string;
  channelNumber: string;
  plcAddress: string;
}

interface BatchInfo {
  productModel: string;
  serialNumber: string;
  customerName: string;
  operatorName: string;
}

@Component({
  selector: 'app-data-import',
  standalone: true,
  imports: [CommonModule, FormsModule],
  templateUrl: './data-import.component.html',
  styleUrl: './data-import.component.css'
})
export class DataImportComponent implements OnInit, OnDestroy {
  // 步骤控制
  currentStep = 1;
  
  // 文件相关
  selectedFileName = '';
  selectedFilePath = '';
  isDragOver = false;
  recentFiles: RecentFile[] = [];
  
  // 预览数据
  previewData: PreviewDataItem[] = [];
  
  // 批次信息
  batchInfo: BatchInfo = {
    productModel: '',
    serialNumber: '',
    customerName: '',
    operatorName: ''
  };
  
  // 状态控制
  isLoading = false;
  isCreatingBatch = false;
  loadingMessage = '';
  error: string | null = null;

  constructor(
    private router: Router,
    private tauriApi: TauriApiService
  ) {}

  ngOnInit() {
    this.loadRecentFiles();
    this.setupDragAndDrop();
  }

  ngOnDestroy() {
    this.removeDragAndDropListeners();
  }

  // 加载最近使用的文件
  loadRecentFiles() {
    // 从本地存储加载最近使用的文件
    const recentFilesJson = localStorage.getItem('recentFiles');
    if (recentFilesJson) {
      try {
        this.recentFiles = JSON.parse(recentFilesJson).map((file: any) => ({
          ...file,
          lastUsed: new Date(file.lastUsed)
        }));
      } catch (error) {
        console.error('加载最近文件失败:', error);
        this.recentFiles = [];
      }
    }
  }

  // 保存最近使用的文件
  saveRecentFile(fileName: string, filePath: string) {
    const newFile: RecentFile = {
      name: fileName,
      path: filePath,
      lastUsed: new Date()
    };

    // 移除重复的文件
    this.recentFiles = this.recentFiles.filter(file => file.path !== filePath);
    
    // 添加到开头
    this.recentFiles.unshift(newFile);
    
    // 只保留最近5个文件
    this.recentFiles = this.recentFiles.slice(0, 5);
    
    // 保存到本地存储
    localStorage.setItem('recentFiles', JSON.stringify(this.recentFiles));
  }

  // 文件选择功能
  async selectFile() {
    try {
      this.error = null;
      
      // 检查是否在Tauri环境中
      if (typeof window !== 'undefined' && window.__TAURI__) {
        const { open } = window.__TAURI__.dialog;
        
        const selected = await open({
          multiple: false,
          filters: [
            {
              name: 'Excel文件',
              extensions: ['xlsx', 'xls']
            }
          ]
        });

        if (selected && typeof selected === 'string') {
          await this.handleFileSelection(selected);
        }
      } else {
        // 在开发环境中，使用HTML5文件选择
        const input = document.createElement('input');
        input.type = 'file';
        input.accept = '.xlsx,.xls';
        input.onchange = async (event: any) => {
          const file = event.target.files[0];
          if (file) {
            await this.handleFileSelection(file.path || file.name, file);
          }
        };
        input.click();
      }
    } catch (error) {
      console.error('文件选择失败:', error);
      this.error = '文件选择失败，请重试';
    }
  }

  // 处理文件选择
  async handleFileSelection(filePath: string, file?: File) {
    try {
      this.isLoading = true;
      this.loadingMessage = '正在读取文件...';
      this.error = null;

      // 提取文件名
      const fileName = filePath.split(/[/\\]/).pop() || filePath;
      
      this.selectedFileName = fileName;
      this.selectedFilePath = filePath;

      // 保存到最近使用的文件
      this.saveRecentFile(fileName, filePath);

      // 读取和解析文件
      await this.loadPreviewData(filePath, file);

    } catch (error) {
      console.error('处理文件失败:', error);
      this.error = '文件处理失败: ' + (error as Error).message;
    } finally {
      this.isLoading = false;
      this.loadingMessage = '';
    }
  }

  // 选择最近使用的文件
  async selectRecentFile(file: RecentFile) {
    await this.handleFileSelection(file.path);
  }

  // 加载预览数据
  async loadPreviewData(filePath: string, file?: File) {
    this.isLoading = true;
    this.loadingMessage = '正在解析Excel文件...';
    this.error = null;

    try {
      if (this.tauriApi.isTauriEnvironment()) {
        // 在Tauri环境中，调用后端API解析文件
        try {
          const result = await this.tauriApi.parseExcelFile(filePath).toPromise();
          
          if (result?.success && result.data) {
            // 转换数据格式
            this.previewData = result.data.map(item => ({
              tag: item.tag,
              description: item.description,
              moduleType: item.module_type,
              channelNumber: item.channel_number,
              plcAddress: item.plc_communication_address
            }));
            
            console.log(`成功解析Excel文件，共${result.total_count}个通道定义`);
            this.loadingMessage = `成功解析${result.total_count}个通道定义`;
          } else {
            throw new Error(result?.message || '解析Excel文件失败');
          }
        } catch (error) {
          console.error('调用Tauri API失败:', error);
          throw error;
        }
      } else {
        // 在开发环境中，使用模拟数据
        console.log('开发环境：使用模拟数据');
        await new Promise(resolve => setTimeout(resolve, 1000)); // 模拟加载时间
        this.previewData = this.generateMockPreviewData();
        this.loadingMessage = '开发环境：显示模拟数据';
      }

      if (this.previewData.length > 0) {
        this.currentStep = 2; // 自动进入下一步
      } else {
        this.error = '文件中没有找到有效的测试点数据';
      }

    } catch (error) {
      console.error('解析文件失败:', error);
      this.error = error instanceof Error ? error.message : '文件解析失败，请检查文件格式是否正确';
    } finally {
      this.isLoading = false;
      this.loadingMessage = '';
    }
  }

  // 生成模拟预览数据
  generateMockPreviewData(): PreviewDataItem[] {
    return [
      {
        tag: 'AI001',
        description: '温度传感器1',
        moduleType: 'AI',
        channelNumber: 'CH01',
        plcAddress: 'DB1.DBD0'
      },
      {
        tag: 'AI002',
        description: '压力传感器1',
        moduleType: 'AI',
        channelNumber: 'CH02',
        plcAddress: 'DB1.DBD4'
      },
      {
        tag: 'DI001',
        description: '开关状态1',
        moduleType: 'DI',
        channelNumber: 'CH01',
        plcAddress: 'I0.0'
      },
      {
        tag: 'DI002',
        description: '开关状态2',
        moduleType: 'DI',
        channelNumber: 'CH02',
        plcAddress: 'I0.1'
      },
      {
        tag: 'AO001',
        description: '控制阀1',
        moduleType: 'AO',
        channelNumber: 'CH01',
        plcAddress: 'QB0'
      },
      {
        tag: 'DO001',
        description: '指示灯1',
        moduleType: 'DO',
        channelNumber: 'CH01',
        plcAddress: 'Q0.0'
      }
    ];
  }

  // 设置拖拽功能
  setupDragAndDrop() {
    if (typeof window !== 'undefined') {
      // 阻止整个窗口的默认拖拽行为
      window.addEventListener('dragover', this.preventDefaults, false);
      window.addEventListener('drop', this.preventDefaults, false);
      window.addEventListener('dragenter', this.preventDefaults, false);
      window.addEventListener('dragleave', this.preventDefaults, false);
      
      // 阻止文档级别的拖拽行为
      document.addEventListener('dragover', this.preventDefaults, false);
      document.addEventListener('drop', this.preventDefaults, false);
      document.addEventListener('dragenter', this.preventDefaults, false);
      document.addEventListener('dragleave', this.preventDefaults, false);
    }
  }

  // 移除拖拽监听器
  removeDragAndDropListeners() {
    if (typeof window !== 'undefined') {
      window.removeEventListener('dragover', this.preventDefaults, false);
      window.removeEventListener('drop', this.preventDefaults, false);
      window.removeEventListener('dragenter', this.preventDefaults, false);
      window.removeEventListener('dragleave', this.preventDefaults, false);
      
      document.removeEventListener('dragover', this.preventDefaults, false);
      document.removeEventListener('drop', this.preventDefaults, false);
      document.removeEventListener('dragenter', this.preventDefaults, false);
      document.removeEventListener('dragleave', this.preventDefaults, false);
    }
  }

  // 阻止默认行为
  preventDefaults = (e: Event) => {
    e.preventDefault();
    e.stopPropagation();
  }

  // 拖拽进入
  onDragEnter(event: DragEvent) {
    event.preventDefault();
    event.stopPropagation();
    
    // 设置允许拖拽
    if (event.dataTransfer) {
      event.dataTransfer.dropEffect = 'copy';
      event.dataTransfer.effectAllowed = 'copy';
    }
    
    this.isDragOver = true;
  }

  // 拖拽离开
  onDragLeave(event: DragEvent) {
    event.preventDefault();
    event.stopPropagation();
    
    // 只有当离开整个拖拽区域时才设置为false
    const rect = (event.currentTarget as HTMLElement).getBoundingClientRect();
    const x = event.clientX;
    const y = event.clientY;
    
    if (x < rect.left || x > rect.right || y < rect.top || y > rect.bottom) {
      this.isDragOver = false;
    }
  }

  // 拖拽悬停
  onDragOver(event: DragEvent) {
    event.preventDefault();
    event.stopPropagation();
    
    // 设置拖拽效果
    if (event.dataTransfer) {
      event.dataTransfer.dropEffect = 'copy';
      event.dataTransfer.effectAllowed = 'copy';
    }
    
    this.isDragOver = true;
  }

  // 文件放置
  async onDrop(event: DragEvent) {
    event.preventDefault();
    event.stopPropagation();
    this.isDragOver = false;

    const files = event.dataTransfer?.files;
    if (files && files.length > 0) {
      const file = files[0];
      
      // 检查文件类型
      if (file.name.endsWith('.xlsx') || file.name.endsWith('.xls')) {
        // 清除之前的错误
        this.error = null;
        await this.handleFileSelection(file.name, file);
      } else {
        this.error = '请选择Excel文件 (.xlsx 或 .xls)';
        // 3秒后自动清除错误
        setTimeout(() => {
          this.error = null;
        }, 3000);
      }
    }
  }

  getModuleTypeCount(moduleType: string): number {
    return this.previewData.filter(item => item.moduleType === moduleType).length;
  }

  nextStep() {
    if (this.canProceedToNext()) {
      this.currentStep++;
    }
  }

  previousStep() {
    if (this.currentStep > 1) {
      this.currentStep--;
      this.error = null; // 清除错误信息
    }
  }

  canProceedToNext(): boolean {
    switch (this.currentStep) {
      case 1:
        return this.selectedFileName !== '' && this.previewData.length > 0;
      case 2:
        return this.previewData.length > 0;
      case 3:
        return this.batchInfo.productModel !== '' && this.batchInfo.serialNumber !== '';
      default:
        return false;
    }
  }

  async startTesting() {
    this.isCreatingBatch = true;
    this.loadingMessage = '正在创建测试批次...';
    this.error = null;
    
    try {
      if (this.tauriApi.isTauriEnvironment()) {
        // 在Tauri环境中，调用后端API创建批次
        try {
          // 转换PreviewDataItem为ChannelPointDefinition格式
          const channelDefinitions = this.previewData.map(item => ({
            id: '', // 后端会生成
            tag: item.tag,
            variable_name: item.tag, // 使用tag作为变量名
            description: item.description,
            station_name: 'Station1', // 默认值
            module_name: 'Module1', // 默认值
            module_type: item.moduleType as any,
            channel_number: item.channelNumber,
            point_data_type: 'Float' as any, // 默认值
            plc_communication_address: item.plcAddress,
            created_at: new Date().toISOString(),
            updated_at: new Date().toISOString()
          }));
          
          const batchData = {
            file_name: this.selectedFileName,
            file_path: this.selectedFilePath,
            preview_data: channelDefinitions,
            batch_info: {
              product_model: this.batchInfo.productModel,
              serial_number: this.batchInfo.serialNumber,
              customer_name: this.batchInfo.customerName,
              operator_name: this.batchInfo.operatorName
            }
          };
          
          const result = await this.tauriApi.createTestBatch(batchData).toPromise();
          
          if (result?.success) {
            console.log('批次创建成功:', result.batch_id);
            this.loadingMessage = '批次创建成功，正在跳转...';
          } else {
            throw new Error(result?.message || '创建批次失败');
          }
        } catch (error) {
          console.error('调用Tauri API创建批次失败:', error);
          throw error;
        }
      } else {
        // 开发环境：模拟批次创建
        console.log('开发环境：模拟批次创建');
        await new Promise(resolve => setTimeout(resolve, 2000));
        this.loadingMessage = '开发环境：模拟批次创建成功';
      }
      
      // 导航到测试执行页面
      this.router.navigate(['/test-execution']);
      
    } catch (error) {
      console.error('创建批次失败:', error);
      this.error = '创建批次失败: ' + (error as Error).message;
    } finally {
      this.isCreatingBatch = false;
      this.loadingMessage = '';
    }
  }

  // 返回仪表板
  goToDashboard() {
    this.router.navigate(['/dashboard']);
  }

  // 清除错误
  clearError() {
    this.error = null;
  }
}
