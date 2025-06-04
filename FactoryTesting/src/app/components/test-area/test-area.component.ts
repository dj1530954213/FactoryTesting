import { Component, OnInit, OnDestroy } from '@angular/core';
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
import { NzCollapseModule } from 'ng-zorro-antd/collapse';
import { TauriApiService } from '../../services/tauri-api.service';
import { DataStateService } from '../../services/data-state.service';
import { BatchSelectionService } from '../../services/batch-selection.service';
import { Subscription } from 'rxjs';
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

// 批次测试统计接口
interface BatchTestStats {
  totalPoints: number;
  pendingPoints: number;
  testedPoints: number;
  successPoints: number;
  failedPoints: number;
  skippedPoints: number;
}

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
    NzMenuModule,
    NzCollapseModule
  ],
  templateUrl: './test-area.component.html',
  styleUrls: ['./test-area.component.css']
})
export class TestAreaComponent implements OnInit, OnDestroy {
  // 订阅管理
  private subscriptions = new Subscription();

  // 批次管理相关
  availableBatches: TestBatchInfo[] = [];
  selectedBatch: TestBatchInfo | null = null;
  isLoadingBatches = false;
  batchDetails: PrepareTestInstancesResponse | null = null;
  isLoadingDetails = false;

  // 批次面板折叠状态
  batchPanelExpanded = false;

  // PLC连接和测试状态
  isConnecting = false;
  isConnected = false;
  isAutoTesting = false;

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
    private dataStateService: DataStateService,
    private batchSelectionService: BatchSelectionService
  ) {}

  ngOnInit(): void {
    this.loadAvailableBatches();
    this.checkForUnpersistedData();
    this.subscribeToSelectedBatch();
  }

  ngOnDestroy(): void {
    this.subscriptions.unsubscribe();
  }

  /**
   * 订阅选中的批次变化
   */
  private subscribeToSelectedBatch(): void {
    const subscription = this.batchSelectionService.selectedBatch$.subscribe(batch => {
      this.selectedBatch = batch;
      if (batch) {
        console.log('🎯 [TEST_AREA] 批次选择变化:', batch.batch_id);
        this.loadBatchDetails();
      } else {
        this.batchDetails = null;
      }
    });
    this.subscriptions.add(subscription);
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

      // 更新批次选择服务
      this.batchSelectionService.setAvailableBatches(this.availableBatches);

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
      this.batchSelectionService.setAvailableBatches([]);
    } finally {
      this.isLoadingBatches = false;
    }
  }

  selectBatch(batch: TestBatchInfo): void {
    // 使用批次选择服务来管理状态
    this.batchSelectionService.selectBatch(batch);
    this.message.success(`已选择批次: ${batch.batch_name || batch.batch_id}`);
  }

  /**
   * 确认接线 - 连接测试PLC和被测PLC
   */
  async confirmWiring(): Promise<void> {
    if (!this.selectedBatch) {
      this.message.warning('请先选择一个测试批次');
      return;
    }

    console.log('🔗 [TEST_AREA] 开始确认接线，连接PLC');
    this.isConnecting = true;

    try {
      // 调用后端API连接PLC
      const result = await this.tauriApiService.connectPlc().toPromise();

      if (result && result.success) {
        this.isConnected = true;
        this.message.success('PLC连接成功，接线确认完成');
        console.log('✅ [TEST_AREA] PLC连接成功');
      } else {
        throw new Error((result && result.message) || 'PLC连接失败');
      }
    } catch (error) {
      console.error('❌ [TEST_AREA] PLC连接失败:', error);
      this.message.error('PLC连接失败: ' + error);
      this.isConnected = false;
    } finally {
      this.isConnecting = false;
    }
  }

  /**
   * 开始通道自动测试
   */
  async startAutoTest(): Promise<void> {
    if (!this.selectedBatch) {
      this.message.warning('请先选择一个测试批次');
      return;
    }

    if (!this.isConnected) {
      this.message.warning('请先确认接线连接PLC');
      return;
    }

    console.log('🚀 [TEST_AREA] 开始通道自动测试');
    this.isAutoTesting = true;

    try {
      // 调用后端API开始自动测试
      const result = await this.tauriApiService.startBatchAutoTest(this.selectedBatch.batch_id).toPromise();

      if (result && result.success) {
        this.message.success('通道自动测试已启动');
        console.log('✅ [TEST_AREA] 通道自动测试启动成功');

        // 重新加载批次详情以获取最新状态
        await this.loadBatchDetails();
      } else {
        throw new Error((result && result.message) || '启动自动测试失败');
      }
    } catch (error) {
      console.error('❌ [TEST_AREA] 启动自动测试失败:', error);
      this.message.error('启动自动测试失败: ' + error);
    } finally {
      this.isAutoTesting = false;
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
   * 批次面板折叠状态切换
   */
  onBatchPanelToggle(expanded: boolean): void {
    this.batchPanelExpanded = expanded;
  }

  /**
   * 获取批次面板标题
   */
  getBatchPanelHeader(): string {
    if (this.selectedBatch) {
      const stats = this.getBatchTestStats(this.selectedBatch);
      return `当前批次: ${this.selectedBatch.batch_name || this.selectedBatch.batch_id} (${stats.totalPoints}个点位)`;
    }
    return '选择测试批次';
  }

  /**
   * 获取连接状态颜色
   */
  getConnectionStatusColor(): string {
    if (this.isConnecting) return 'processing';
    if (this.isConnected) return 'success';
    return 'default';
  }

  /**
   * 获取连接状态图标
   */
  getConnectionStatusIcon(): string {
    if (this.isConnecting) return 'loading';
    if (this.isConnected) return 'check-circle';
    return 'disconnect';
  }

  /**
   * 获取连接状态文本
   */
  getConnectionStatusText(): string {
    if (this.isConnecting) return '连接中...';
    if (this.isConnected) return 'PLC已连接';
    return 'PLC未连接';
  }

  /**
   * 获取批次的测试统计信息
   */
  getBatchTestStats(batch: TestBatchInfo): BatchTestStats {
    // 如果是当前选中的批次且有详情数据，使用详情数据计算
    if (this.selectedBatch?.batch_id === batch.batch_id && this.batchDetails) {
      return this.calculateTestStatsFromDetails();
    }

    // 否则返回基础统计信息
    return {
      totalPoints: batch.total_points || 0,
      pendingPoints: batch.total_points || 0, // 假设所有点都是待测
      testedPoints: 0,
      successPoints: 0,
      failedPoints: 0,
      skippedPoints: 0
    };
  }

  /**
   * 从批次详情计算测试统计信息
   */
  private calculateTestStatsFromDetails(): BatchTestStats {
    if (!this.batchDetails?.instances) {
      return {
        totalPoints: 0,
        pendingPoints: 0,
        testedPoints: 0,
        successPoints: 0,
        failedPoints: 0,
        skippedPoints: 0
      };
    }

    const instances = this.batchDetails.instances;
    const totalPoints = instances.length;

    let pendingPoints = 0;
    let testedPoints = 0;
    let successPoints = 0;
    let failedPoints = 0;
    let skippedPoints = 0;

    instances.forEach(instance => {
      switch (instance.overall_status) {
        case OverallTestStatus.NotTested:
          pendingPoints++;
          break;
        case OverallTestStatus.HardPointTesting:
        case OverallTestStatus.AlarmTesting:
          testedPoints++;
          break;
        case OverallTestStatus.TestCompletedPassed:
          testedPoints++;
          successPoints++;
          break;
        case OverallTestStatus.TestCompletedFailed:
          testedPoints++;
          failedPoints++;
          break;
        default:
          // 其他状态视为跳过
          skippedPoints++;
          break;
      }
    });

    return {
      totalPoints,
      pendingPoints,
      testedPoints,
      successPoints,
      failedPoints,
      skippedPoints
    };
  }

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