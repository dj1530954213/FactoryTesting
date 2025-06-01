import { Component, OnInit } from '@angular/core';
import { CommonModule } from '@angular/common';
import { FormsModule } from '@angular/forms';
import { NzMessageService } from 'ng-zorro-antd/message';
import { NzCardModule } from 'ng-zorro-antd/card';
import { NzDividerModule } from 'ng-zorro-antd/divider';
import { NzButtonModule } from 'ng-zorro-antd/button';
import { NzTableModule } from 'ng-zorro-antd/table';
import { NzTagModule } from 'ng-zorro-antd/tag';
import { NzSelectModule } from 'ng-zorro-antd/select';
import { NzInputModule } from 'ng-zorro-antd/input';
import { NzCheckboxModule } from 'ng-zorro-antd/checkbox';
import { NzSpinModule } from 'ng-zorro-antd/spin';
import { NzStatisticModule } from 'ng-zorro-antd/statistic';
import { NzGridModule } from 'ng-zorro-antd/grid';
import { NzSpaceModule } from 'ng-zorro-antd/space';
import { NzIconModule } from 'ng-zorro-antd/icon';
import { NzEmptyModule } from 'ng-zorro-antd/empty';
import { NzDropDownModule } from 'ng-zorro-antd/dropdown';
import { NzMenuModule } from 'ng-zorro-antd/menu';
import { TauriApiService } from '../../services/tauri-api.service';
import { DataStateService } from '../../services/data-state.service';
import {
  TestBatchInfo,
  ChannelTestInstance,
  ChannelPointDefinition,
  PrepareTestInstancesResponse,
  OverallTestStatus,
  ModuleType,
  PointDataType,
  AllocationSummary,
  SubTestItem,
  SubTestStatus,
  OVERALL_TEST_STATUS_LABELS,
  MODULE_TYPE_LABELS,
  POINT_DATA_TYPE_LABELS
} from '../../models';

@Component({
  selector: 'app-test-area',
  standalone: true,
  imports: [
    CommonModule,
    FormsModule,
    NzCardModule,
    NzDividerModule,
    NzButtonModule,
    NzTableModule,
    NzTagModule,
    NzSelectModule,
    NzInputModule,
    NzCheckboxModule,
    NzSpinModule,
    NzStatisticModule,
    NzGridModule,
    NzSpaceModule,
    NzIconModule,
    NzEmptyModule,
    NzDropDownModule,
    NzMenuModule
  ],
  templateUrl: './test-area.component.html',
  styleUrls: ['./test-area.component.css']
})
export class TestAreaComponent implements OnInit {
  // 批次管理相关
  availableBatches: TestBatchInfo[] = [];
  selectedBatch: TestBatchInfo | null = null;
  isLoadingBatches = false;
  batchDetails: PrepareTestInstancesResponse | null = null;
  isLoadingDetails = false;

  // 筛选和搜索相关
  selectedModuleTypes: ModuleType[] = [];
  searchText = '';
  showOnlyTested = false;
  showOnlyFailed = false;

  // 模块类型选项
  moduleTypeOptions = [
    { label: MODULE_TYPE_LABELS[ModuleType.AI], value: ModuleType.AI, count: 0 },
    { label: MODULE_TYPE_LABELS[ModuleType.AO], value: ModuleType.AO, count: 0 },
    { label: MODULE_TYPE_LABELS[ModuleType.DI], value: ModuleType.DI, count: 0 },
    { label: MODULE_TYPE_LABELS[ModuleType.DO], value: ModuleType.DO, count: 0 }
  ];

  constructor(
    private tauriApiService: TauriApiService,
    private message: NzMessageService,
    private dataStateService: DataStateService
  ) {}

  ngOnInit(): void {
    this.loadAvailableBatches();
    this.checkForUnpersistedData();
  }

  async loadAvailableBatches(): Promise<void> {
    console.log('📋 [TEST_AREA] 开始加载可用批次列表');
    this.isLoadingBatches = true;
    try {
      // 调用真实的后端API获取批次列表
      console.log('📋 [TEST_AREA] 调用后端API: getBatchList()');
      const batches = await this.tauriApiService.getBatchList().toPromise();
      this.availableBatches = batches || [];

      console.log('✅ [TEST_AREA] 成功从后端获取批次列表');
      console.log('✅ [TEST_AREA] 批次数量:', this.availableBatches.length);

      if (this.availableBatches.length > 0) {
        console.log('✅ [TEST_AREA] 批次详情:');
        this.availableBatches.forEach((batch, index) => {
          console.log(`  批次${index + 1}: ID=${batch.batch_id}, 名称=${batch.batch_name}, 点位数=${batch.total_points}`);
        });
      } else {
        console.log('⚠️ [TEST_AREA] 没有找到任何批次，可能需要先导入点表');
        this.message.info('暂无可用的测试批次，请先导入Excel文件创建批次');
      }
    } catch (error) {
      console.error('❌ [TEST_AREA] 加载批次列表失败:', error);
      this.message.error('加载批次列表失败: ' + error);
      this.availableBatches = [];
    } finally {
      this.isLoadingBatches = false;
    }
  }

  selectBatch(batch: TestBatchInfo): void {
    this.selectedBatch = batch;
    this.batchDetails = null;
    this.message.success(`已选择批次: ${batch.batch_name || batch.batch_id}`);
    
    // 自动加载批次详情
    this.loadBatchDetails();
  }

  refreshBatches(): void {
    this.loadAvailableBatches();
    this.message.info('正在刷新批次列表...');
  }

  /**
   * 清理当前会话数据
   */
  async clearSessionData(): Promise<void> {
    try {
      const result = await this.tauriApiService.clearSessionData().toPromise();
      this.message.success(result || '会话数据清理完成');
      
      // 清理本地状态
      this.availableBatches = [];
      this.selectedBatch = null;
      this.batchDetails = null;
      
      // 清理数据状态服务
      this.dataStateService.clearAllData();
      
      console.log('会话数据已清理，界面已重置');
    } catch (error) {
      console.error('清理会话数据失败:', error);
      this.message.error('清理会话数据失败: ' + error);
    }
  }

  async loadBatchDetails(): Promise<void> {
    if (!this.selectedBatch) {
      console.log('⚠️ [TEST_AREA] 没有选择批次，无法加载详情');
      this.message.warning('请先选择一个测试批次');
      return;
    }

    console.log('📊 [TEST_AREA] 开始加载批次详情');
    console.log('📊 [TEST_AREA] 选中的批次ID:', this.selectedBatch.batch_id);
    this.isLoadingDetails = true;
    try {
      // 调用真实的后端API获取批次详情
      console.log('📊 [TEST_AREA] 调用后端API: getBatchDetails()');
      const details = await this.tauriApiService.getBatchDetails(this.selectedBatch.batch_id).toPromise();

      console.log('📊 [TEST_AREA] 后端返回的详情数据:', details);

      if (details) {
        // 使用后端返回的真实数据
        this.batchDetails = {
          batch_info: details.batch_info,
          instances: details.instances,
          definitions: details.definitions || [],
          allocation_summary: details.allocation_summary || {
            total_definitions: details.instances.length,
            allocated_instances: details.instances.length,
            skipped_definitions: 0,
            allocation_errors: []
          }
        };

        console.log('✅ [TEST_AREA] 批次详情加载成功');
        console.log('✅ [TEST_AREA] 实例数量:', this.batchDetails.instances.length);
        console.log('✅ [TEST_AREA] 定义数量:', this.batchDetails.definitions.length);

        this.message.success('批次详情加载成功');
        this.updateModuleTypeStats();
      } else {
        console.error('❌ [TEST_AREA] 后端返回空的详情数据');
        throw new Error('未找到批次详情数据');
      }
    } catch (error) {
      console.error('❌ [TEST_AREA] 加载批次详情失败:', error);
      this.message.error('加载批次详情失败: ' + error);
      this.batchDetails = null;
    } finally {
      this.isLoadingDetails = false;
    }
  }

  getDefinitionByInstanceId(instanceId: string): ChannelPointDefinition | undefined {
    return this.batchDetails?.definitions?.find(def => 
      this.batchDetails?.instances?.find(inst => inst.instance_id === instanceId)?.definition_id === def.id
    );
  }

  getInstanceStatusColor(status: OverallTestStatus): string {
    switch (status) {
      case OverallTestStatus.NotTested: return 'default';
      case OverallTestStatus.HardPointTesting: return 'processing';
      case OverallTestStatus.AlarmTesting: return 'warning';
      case OverallTestStatus.TestCompletedPassed: return 'success';
      case OverallTestStatus.TestCompletedFailed: return 'error';
      default: return 'default';
    }
  }

  getInstanceStatusText(status: OverallTestStatus): string {
    switch (status) {
      case OverallTestStatus.NotTested: return '未测试';
      case OverallTestStatus.HardPointTesting: return '硬点测试中';
      case OverallTestStatus.AlarmTesting: return '报警测试中';
      case OverallTestStatus.TestCompletedPassed: return '测试通过';
      case OverallTestStatus.TestCompletedFailed: return '测试失败';
      default: return '未知状态';
    }
  }

  getAllocationErrorCount(): number {
    return this.batchDetails?.allocation_summary?.allocation_errors?.length || 0;
  }

  getFilteredInstances(): ChannelTestInstance[] {
    if (!this.batchDetails?.instances) return [];

    return this.batchDetails.instances.filter(instance => {
      const definition = this.getDefinitionByInstanceId(instance.instance_id);
      
      // 模块类型筛选
      if (this.selectedModuleTypes.length > 0 && definition) {
        if (!this.selectedModuleTypes.includes(definition.module_type)) {
          return false;
        }
      }

      // 搜索文本筛选
      if (this.searchText.trim()) {
        const searchLower = this.searchText.toLowerCase();
        const matchesTag = definition?.tag?.toLowerCase().includes(searchLower);
        const matchesVariable = definition?.variable_name?.toLowerCase().includes(searchLower);
        const matchesDescription = definition?.description?.toLowerCase().includes(searchLower);
        
        if (!matchesTag && !matchesVariable && !matchesDescription) {
          return false;
        }
      }

      // 测试状态筛选
      if (this.showOnlyTested) {
        if (instance.overall_status === OverallTestStatus.NotTested) {
          return false;
        }
      }

      // 失败状态筛选
      if (this.showOnlyFailed) {
        if (instance.overall_status !== OverallTestStatus.TestCompletedFailed) {
          return false;
        }
      }

      return true;
    });
  }

  updateModuleTypeStats(): void {
    if (!this.batchDetails?.instances) return;

    // 重置计数
    this.moduleTypeOptions.forEach(option => option.count = 0);

    // 统计每种模块类型的数量
    this.batchDetails.instances.forEach(instance => {
      const definition = this.getDefinitionByInstanceId(instance.instance_id);
      if (definition) {
        const option = this.moduleTypeOptions.find(opt => opt.value === definition.module_type);
        if (option) {
          option.count++;
        }
      }
    });
  }

  clearAllFilters(): void {
    this.selectedModuleTypes = [];
    this.searchText = '';
    this.showOnlyTested = false;
    this.showOnlyFailed = false;
  }

  getFilterStatusText(): string {
    const total = this.batchDetails?.instances?.length || 0;
    const filtered = this.getFilteredInstances().length;
    
    if (total === filtered) {
      return `显示全部 ${total} 个通道`;
    } else {
      return `显示 ${filtered} / ${total} 个通道`;
    }
  }

  getModuleTypeColor(moduleType: ModuleType | undefined): string {
    if (!moduleType) return 'default';
    switch (moduleType) {
      case ModuleType.AI: return 'blue';
      case ModuleType.AO: return 'green';
      case ModuleType.DI: return 'orange';
      case ModuleType.DO: return 'purple';
      default: return 'default';
    }
  }

  getPointDataTypeColor(dataType: PointDataType | undefined): string {
    if (!dataType) return 'default';
    switch (dataType) {
      case PointDataType.Bool: return 'purple';
      case PointDataType.Int: return 'orange';
      case PointDataType.Float: return 'blue';
      case PointDataType.String: return 'green';
      default: return 'default';
    }
  }

  getPointDataTypeLabel(dataType: PointDataType | undefined): string {
    if (!dataType) return 'N/A';
    return POINT_DATA_TYPE_LABELS[dataType] || dataType;
  }

  formatDateTime(dateTimeString: string | undefined): string {
    if (!dateTimeString) return 'N/A';
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

  getStatusColor(status: OverallTestStatus | string): string {
    if (typeof status === 'string') {
      if (status.includes('已创建')) return 'blue';
      if (status.includes('测试中')) return 'orange';
      if (status.includes('已完成')) return 'green';
      if (status.includes('失败')) return 'red';
      return 'default';
    }
    
    switch (status) {
      case OverallTestStatus.NotTested: return 'default';
      case OverallTestStatus.HardPointTesting: return 'orange';
      case OverallTestStatus.AlarmTesting: return 'warning';
      case OverallTestStatus.TestCompletedPassed: return 'green';
      case OverallTestStatus.TestCompletedFailed: return 'red';
      default: return 'default';
    }
  }

  readonly OverallTestStatus = OverallTestStatus;
  readonly ModuleType = ModuleType;

  /**
   * 检查是否有未持久化的数据
   *
   * ⚠️ 重要修改：测试区域不再创建批次，只获取已存在的数据
   * 批次创建应该在点表导入时完成
   */
  private checkForUnpersistedData(): void {
    console.log('🔍 [TEST_AREA] 检查是否有未持久化的数据');
    const testData = this.dataStateService.getTestData();

    if (testData.isDataAvailable && testData.parsedDefinitions.length > 0) {
      console.log('⚠️ [TEST_AREA] 检测到未持久化的测试数据');
      console.log('⚠️ [TEST_AREA] 这表明点表导入流程可能没有正确完成批次分配');

      // 清理内存中的数据，因为批次应该已经在导入时创建
      this.dataStateService.clearTestData();
      this.message.warning('检测到未完成的导入流程，请重新导入点表以创建批次');

      // 重新加载批次列表，查看是否有新创建的批次
      this.loadAvailableBatches();
    } else {
      console.log('✅ [TEST_AREA] 没有未持久化的数据，正常加载批次列表');
    }
  }
} 