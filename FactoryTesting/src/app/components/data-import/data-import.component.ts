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
  variableName: string;
  stationName: string;
  moduleName: string;
  dataType: string;
  analogRangeMin?: number;
  analogRangeMax?: number;
}

@Component({
  selector: 'app-data-import',
  standalone: true,
  imports: [CommonModule, FormsModule],
  templateUrl: './data-import.component.html',
  styleUrl: './data-import.component.css'
})
export class DataImportComponent implements OnInit, OnDestroy {
  // 当前步骤：1=选择文件, 2=预览数据, 3=开始测试（删除批次信息步骤）
  currentStep = 1;
  
  // 文件相关
  selectedFileName = '';
  selectedFilePath = '';
  isDragOver = false;
  recentFiles: RecentFile[] = [];
  
  // 预览数据
  previewData: PreviewDataItem[] = [];
  
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
    // 清除旧的模拟数据，重新开始
    localStorage.removeItem('recentFiles');
    this.recentFiles = [];
    
    // 从本地存储加载最近使用的文件（现在应该是空的）
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
      
      console.log('开始文件选择流程');
      console.log('Tauri环境检测结果:', this.tauriApi.isTauriEnvironment());
      
      // 在Tauri环境中，使用Tauri文件对话框
      if (this.tauriApi.isTauriEnvironment() && typeof window !== 'undefined' && window.__TAURI__) {
        console.log('使用Tauri文件对话框选择文件');
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

        console.log('Tauri文件对话框返回:', selected);
        
        if (selected && typeof selected === 'string') {
          await this.handleFileSelection(selected);
        }
      } else {
        console.log('非Tauri环境，使用HTML文件选择器');
        // 在开发环境中，创建一个隐藏的文件输入元素
        const input = document.createElement('input');
        input.type = 'file';
        input.accept = '.xlsx,.xls';
        input.style.display = 'none';
        
        input.onchange = async (event: any) => {
          const file = event.target.files[0];
          if (file) {
            // 在开发环境中，使用测试文件路径（修正路径）
            const testFilePath = 'D:\\GIT\\Git\\code\\FactoryTesting\\测试文件\\测试IO.xlsx';
            console.log('开发环境：使用测试文件路径:', testFilePath);
            await this.handleFileSelection(testFilePath, file);
          }
        };
        
        document.body.appendChild(input);
        input.click();
        document.body.removeChild(input);
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
    console.log('选择最近使用的文件:', file.name, file.path);
    
    // 检查是否是测试文件，使用正确的路径
    if (file.name === '测试IO.xlsx') {
      const testFilePath = 'D:\\GIT\\Git\\code\\FactoryTesting\\测试文件\\测试IO.xlsx';
      console.log('识别为测试文件，使用完整路径:', testFilePath);
      await this.handleFileSelection(testFilePath);
    } else {
      // 对于其他文件，直接使用保存的路径
      await this.handleFileSelection(file.path);
    }
  }

  // 加载预览数据
  async loadPreviewData(filePath: string, file?: File) {
    this.isLoading = true;
    this.loadingMessage = '正在解析Excel文件...';
    this.error = null;

    try {
      console.log('开始解析Excel文件:', filePath);
      console.log('Tauri环境检测:', this.tauriApi.isTauriEnvironment());
      
      // 强制尝试使用Tauri API，即使在开发环境中
      const forceUseTauriApi = true;
      
      if (this.tauriApi.isTauriEnvironment() || forceUseTauriApi) {
        // 尝试调用后端API解析文件
        try {
          console.log('尝试调用Tauri API解析Excel文件:', filePath);
          const definitions = await this.tauriApi.importExcelFile(filePath).toPromise();
          
          console.log('Tauri API返回结果:', definitions);
          
          if (definitions && definitions.length > 0) {
            // 转换数据格式为前端预览格式
            this.previewData = definitions.map(def => ({
              tag: def.tag,
              description: def.description || '',
              moduleType: def.module_type,
              channelNumber: def.channel_number,
              plcAddress: def.plc_communication_address,
              variableName: def.variable_name,
              stationName: def.station_name,
              moduleName: def.module_name,
              dataType: def.point_data_type,
              analogRangeMin: def.analog_range_min,
              analogRangeMax: def.analog_range_max
            }));
            
            console.log(`成功解析Excel文件，共${definitions.length}个通道定义`);
            console.log('转换后的预览数据:', this.previewData.slice(0, 3)); // 只显示前3个
            this.loadingMessage = `成功解析${definitions.length}个通道定义`;
            
            // 显示统计信息
            const aiCount = this.getModuleTypeCount('AI');
            const aoCount = this.getModuleTypeCount('AO');
            const diCount = this.getModuleTypeCount('DI');
            const doCount = this.getModuleTypeCount('DO');
            console.log(`模块类型统计: AI:${aiCount}, AO:${aoCount}, DI:${diCount}, DO:${doCount}`);
          } else {
            throw new Error('Excel文件中没有找到有效的通道定义');
          }
        } catch (error) {
          console.error('调用Tauri API失败:', error);
          
          // 如果是在开发环境且API调用失败，抛出错误而不是使用模拟数据
          if (!this.tauriApi.isTauriEnvironment()) {
            console.log('Tauri API调用失败，开发环境中无法获取真实数据');
            throw new Error('无法连接到后端服务，请确保使用正确的启动命令：npm run tauri:dev');
          } else {
            throw error;
          }
        }
      } else {
        // 在开发环境中，提示用户使用正确的启动方式
        console.log('开发环境：需要使用Tauri环境');
        throw new Error('请使用正确的启动命令：npm run tauri:dev');
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
        
        console.log('拖拽文件:', file.name);
        
        // 在Tauri环境中，拖拽文件无法获取完整路径，需要特殊处理
        if (this.tauriApi.isTauriEnvironment()) {
          console.log('Tauri环境中的拖拽文件处理');
          
          // 检查是否是测试文件
          if (file.name === '测试IO.xlsx') {
            const testFilePath = 'D:\\GIT\\Git\\code\\FactoryTesting\\测试文件\\测试IO.xlsx';
            console.log('识别为测试文件，使用完整路径:', testFilePath);
            await this.handleFileSelection(testFilePath, file);
          } else {
            // 对于其他文件，提示用户使用文件选择器
            this.error = '请使用"浏览文件"按钮选择Excel文件，以确保能获取完整的文件路径';
            setTimeout(() => {
              this.error = null;
            }, 5000);
          }
        } else {
          console.log('开发环境中的拖拽文件，使用测试文件路径');
          const testFilePath = 'D:\\GIT\\Git\\code\\FactoryTesting\\测试文件\\测试IO.xlsx';
          await this.handleFileSelection(testFilePath, file);
        }
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

  getDataTypeCount(dataType: string): number {
    return this.previewData.filter(item => item.dataType === dataType).length;
  }

  getUniqueStations(): string[] {
    const stations = [...new Set(this.previewData.map(item => item.stationName))];
    return stations.filter(station => station && station.trim() !== '');
  }

  getStationCount(stationName: string): number {
    return this.previewData.filter(item => item.stationName === stationName).length;
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
        return this.previewData.length > 0;
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
        // 在Tauri环境中，调用后端API自动创建批次
        try {
          console.log('调用后端API自动创建测试批次');
          
          // 转换PreviewDataItem为ChannelPointDefinition格式
          const channelDefinitions = this.previewData.map(item => ({
            id: '', // 后端会生成
            tag: item.tag,
            variable_name: item.variableName,
            description: item.description,
            station_name: item.stationName,
            module_name: item.moduleName,
            module_type: item.moduleType as any,
            channel_number: item.channelNumber,
            point_data_type: item.dataType as any,
            plc_communication_address: item.plcAddress,
            analog_range_min: item.analogRangeMin,
            analog_range_max: item.analogRangeMax,
            created_at: new Date().toISOString(),
            updated_at: new Date().toISOString()
          }));
          
          // 调用后端API，让后端自动分配批次信息
          const createBatchRequest = {
            file_name: this.selectedFileName,
            file_path: this.selectedFilePath,
            preview_data: channelDefinitions,
            batch_info: {
              product_model: '', // 后端会自动生成或使用默认值
              serial_number: '', // 后端会自动生成
              customer_name: '',
              operator_name: ''
            }
          };
          
          const response = await this.tauriApi.createTestBatch(createBatchRequest).toPromise();
          
          if (response && response.success && response.batch_id) {
            console.log('批次创建成功:', response.batch_id);
            this.loadingMessage = '批次创建成功，正在跳转...';
            
            // 导航到测试执行页面，传递批次ID
            setTimeout(() => {
              this.router.navigate(['/test-execution'], { queryParams: { batchId: response.batch_id } });
            }, 1000);
          } else {
            throw new Error(response?.message || '创建批次失败：未返回有效响应');
          }
        } catch (error) {
          console.error('调用Tauri API创建批次失败:', error);
          throw error;
        }
      } else {
        // 开发环境：提示使用正确的启动方式
        throw new Error('请使用正确的启动命令：npm run tauri:dev');
      }
      
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

  // 测试Tauri API功能
  async testTauriApi() {
    console.log('=== 测试Tauri API功能 ===');
    
    try {
      // 使用正确的测试文件路径
      const testFilePath = 'D:\\GIT\\Git\\code\\FactoryTesting\\测试文件\\测试IO.xlsx';
      console.log('测试调用Tauri API解析Excel文件:', testFilePath);
      
      // 正确处理Observable返回类型
      this.tauriApi.parseExcelFile(testFilePath).subscribe({
        next: (result) => {
          console.log('Tauri API调用成功，解析结果:', result);
          
          if (result && result.success && result.data && result.data.length > 0) {
            console.log(`成功解析到 ${result.data.length} 个数据点位`);
            console.log('前5个数据点位:', result.data.slice(0, 5));
            
            // 更新预览数据
            this.previewData = result.data.map((item: any) => ({
              tag: item.tag || '',
              description: item.description || '',
              moduleType: item.module_type || '',
              channelNumber: item.channel_number || '',
              plcAddress: item.plc_communication_address || '',
              variableName: item.variable_name || '',
              stationName: item.station_name || '',
              moduleName: item.module_name || '',
              dataType: item.point_data_type || '',
              analogRangeMin: item.analog_range_min,
              analogRangeMax: item.analog_range_max
            }));
            
            this.selectedFileName = '测试IO.xlsx';
            this.selectedFilePath = testFilePath;
            
            console.log('预览数据已更新，数据点位数量:', this.previewData.length);
          } else {
            console.log('API返回空数据或无效数据，结果:', result);
          }
        },
        error: (error) => {
          console.error('测试Tauri API失败:', error);
        }
      });
    } catch (error) {
      console.error('测试Tauri API异常:', error);
    }
  }
}
