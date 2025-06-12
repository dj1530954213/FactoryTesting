import { Component, OnInit, OnDestroy, ViewChild } from '@angular/core';
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
import { NzProgressModule } from 'ng-zorro-antd/progress';
import { NzModalModule, NzModalService } from 'ng-zorro-antd/modal';

import { TauriApiService } from '../../services/tauri-api.service';
import { DataStateService } from '../../services/data-state.service';
import { BatchSelectionService } from '../../services/batch-selection.service';
import { Subscription, firstValueFrom } from 'rxjs';
import { listen } from '@tauri-apps/api/event';
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
import { ErrorDetailModalComponent } from './error-detail-modal.component';
import { ManualTestModalComponent } from '../manual-test/manual-test-modal.component';

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
    NzCollapseModule,
    NzProgressModule,
    NzModalModule,
    ErrorDetailModalComponent,
    ManualTestModalComponent
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

  // 测试进度相关
  testProgress = {
    totalPoints: 0,
    completedPoints: 0,
    successPoints: 0,
    failedPoints: 0,
    progressPercentage: 0,
    currentPoint: undefined as string | undefined,
    estimatedTimeRemaining: undefined as string | undefined
  };

  // 🔧 优化：数据刷新防抖机制
  private refreshTimeouts = new Map<string, any>();
  private lastRefreshTime = 0;
  private readonly MIN_REFRESH_INTERVAL = 1000; // 最小刷新间隔1秒

  // 测试状态
  isTestCompleted = false;
  recentTestResults: Array<{
    pointTag: string;
    success: boolean;
    message: string;
    timestamp: Date;
  }> = [];



  // 筛选和搜索相关
  selectedModuleTypes: ModuleType[] = [];
  private _searchText = '';
  showOnlyTested = false;
  showOnlyFailed = false;

  // 🔧 性能优化：缓存过滤结果
  private _filteredInstances: ChannelTestInstance[] = [];
  private _lastFilterHash = '';

  // 🔧 性能优化：缓存定义映射
  private _definitionMap = new Map<string, ChannelPointDefinition>();

  // 🔧 性能优化：防抖处理
  private _searchDebounceTimer: any = null;
  private _statsUpdateTimer: any = null;

  // 模块类型选项
  moduleTypeOptions = [
    { label: MODULE_TYPE_LABELS[ModuleType.AI], value: ModuleType.AI, count: 0 },
    { label: MODULE_TYPE_LABELS[ModuleType.AO], value: ModuleType.AO, count: 0 },
    { label: MODULE_TYPE_LABELS[ModuleType.DI], value: ModuleType.DI, count: 0 },
    { label: MODULE_TYPE_LABELS[ModuleType.DO], value: ModuleType.DO, count: 0 }
  ];

  // 错误详情模态框相关
  errorDetailModalVisible = false;
  selectedErrorInstance: ChannelTestInstance | null = null;
  selectedErrorDefinition: ChannelPointDefinition | null = null;

  // 手动测试模态框相关
  manualTestModalVisible = false;
  selectedManualTestInstance: ChannelTestInstance | null = null;
  selectedManualTestDefinition: ChannelPointDefinition | null = null;

  constructor(
    private tauriApiService: TauriApiService,
    private message: NzMessageService,
    private dataStateService: DataStateService,
    private batchSelectionService: BatchSelectionService,
    private modal: NzModalService
  ) {}

  ngOnInit(): void {
    this.loadAvailableBatches();
    this.checkForUnpersistedData();
    this.subscribeToSelectedBatch();
    this.setupTestResultListener(); // 异步调用，不需要等待

    // 初始化测试进度
    this.initializeTestProgress();
  }

  ngOnDestroy(): void {
    // 🔧 优化：组件销毁时清理所有定时器
    this.refreshTimeouts.forEach(timeoutId => clearTimeout(timeoutId));
    this.refreshTimeouts.clear();
    // console.log('🔧 [TEST_AREA] 组件销毁，已清理所有定时器');

    // 清理订阅
    this.subscriptions.unsubscribe();

    // 🔧 性能优化：清理缓存和定时器
    this._definitionMap.clear();
    this._filteredInstances = [];
    if (this._searchDebounceTimer) {
      clearTimeout(this._searchDebounceTimer);
      this._searchDebounceTimer = null;
    }
    if (this._statsUpdateTimer) {
      clearTimeout(this._statsUpdateTimer);
      this._statsUpdateTimer = null;
    }
  }

  // 🔧 性能优化：trackBy函数
  trackByInstanceId(index: number, instance: ChannelTestInstance): string {
    return instance.instance_id;
  }

  trackByBatchId(index: number, batch: TestBatchInfo): string {
    return batch.batch_id;
  }

  // 🔧 性能优化：搜索文本的getter和setter，实现防抖
  get searchText(): string {
    return this._searchText;
  }

  set searchText(value: string) {
    this._searchText = value;

    // 清除之前的定时器
    if (this._searchDebounceTimer) {
      clearTimeout(this._searchDebounceTimer);
    }

    // 设置新的防抖定时器
    this._searchDebounceTimer = setTimeout(() => {
      // 清理过滤缓存，触发重新计算
      this._filteredInstances = [];
      this._lastFilterHash = '';
    }, 300); // 300ms防抖延迟
  }

  // 🔧 性能优化：延迟统计更新，避免频繁调用
  private scheduleStatsUpdate(): void {
    if (this._statsUpdateTimer) {
      clearTimeout(this._statsUpdateTimer);
    }

    this._statsUpdateTimer = setTimeout(() => {
      this.updateModuleTypeStats();
    }, 100); // 100ms延迟
  }

  // 🔧 性能优化：模块类型过滤变化处理
  onModuleTypeFilterChange(): void {
    this.onFilterChange();
  }

  // 🔧 性能优化：通用过滤变化处理
  onFilterChange(): void {
    // 清理过滤缓存，触发重新计算
    this._filteredInstances = [];
    this._lastFilterHash = '';
  }

  /**
   * 订阅选中的批次变化
   */
  private subscribeToSelectedBatch(): void {
    const subscription = this.batchSelectionService.selectedBatch$.subscribe(batch => {
      this.selectedBatch = batch;
      if (batch) {
        // console.log('🎯 [TEST_AREA] 批次选择变化:', batch.batch_id);
        this.loadBatchDetails();
      } else {
        this.batchDetails = null;
        // 重置测试进度
        this.initializeTestProgress();
      }
    });
    this.subscriptions.add(subscription);
  }

  /**
   * 🔧 优化：智能数据刷新调度器，避免频繁刷新
   */
  private scheduleDataRefresh(reason: string, delay: number = 1000): void {
    // 清除之前的定时器
    if (this.refreshTimeouts.has(reason)) {
      clearTimeout(this.refreshTimeouts.get(reason));
    }

    // 检查最小刷新间隔
    const now = Date.now();
    if (now - this.lastRefreshTime < this.MIN_REFRESH_INTERVAL) {
      delay = Math.max(delay, this.MIN_REFRESH_INTERVAL - (now - this.lastRefreshTime));
    }

    // 设置新的定时器
    const timeoutId = setTimeout(async () => {
      this.lastRefreshTime = Date.now();
      this.refreshTimeouts.delete(reason);

      // console.log(`🔄 [TEST_AREA] 执行数据刷新 (原因: ${reason})`);
      await this.loadBatchDetails();
    }, delay);

    this.refreshTimeouts.set(reason, timeoutId);
  }

  /**
   * 设置测试结果实时监听
   */
  private async setupTestResultListener(): Promise<void> {
    // console.log('🎧 [TEST_AREA] 设置测试结果实时监听');

    try {
      // 监听后端发布的测试完成事件
      const unlistenCompleted = await listen('test-completed', (event) => {
        // console.log('🎉 [TEST_AREA] 收到测试完成事件:', event.payload);

        // 解析事件数据
        const testResult = event.payload as {
          instanceId: string;
          success: boolean;
          subTestItem: string;
          message: string;
          rawValue?: number;
          engValue?: number;
          pointTag?: string;
        };

        console.log(`🔄 [TEST_AREA] 处理测试结果: ${testResult.instanceId} - ${testResult.success ? '通过' : '失败'}`);

        // 更新本地状态
        this.updateInstanceStatus(testResult);

        // 更新测试进度
        this.updateTestProgressFromResult(testResult);

        // 🔧 优化：测试完成后延迟刷新，避免频繁调用
        this.scheduleDataRefresh('test-completed', 1000);

        // 显示通知
        if (testResult.success) {
          console.log(`✅ [TEST_AREA] 测试通过: ${testResult.instanceId}`);
        } else {
          console.log(`❌ [TEST_AREA] 测试失败: ${testResult.instanceId} - ${testResult.message}`);
        }
      });

      // 监听后端发布的测试状态变化事件
      const unlistenStatusChanged = await listen('test-status-changed', (event) => {
        console.log('🔄 [TEST_AREA] 收到测试状态变化事件:', event.payload);

        // 解析事件数据
        const statusChange = event.payload as {
          instanceId: string;
          oldStatus: OverallTestStatus;
          newStatus: OverallTestStatus;
          timestamp: string;
          pointTag?: string;
        };

        console.log(`🔄 [TEST_AREA] 状态变化: ${statusChange.instanceId} - ${statusChange.oldStatus} -> ${statusChange.newStatus}`);

        // 更新本地状态
        this.updateInstanceStatusDirect(statusChange.instanceId, statusChange.newStatus);

        // 🔧 优化：测试状态变化后智能刷新
        if (statusChange.newStatus === OverallTestStatus.TestCompletedPassed ||
            statusChange.newStatus === OverallTestStatus.TestCompletedFailed) {
          this.scheduleDataRefresh('status-changed', 500);
        }

        // 更新当前测试点位
        if (statusChange.newStatus === OverallTestStatus.HardPointTesting && statusChange.pointTag) {
          this.testProgress.currentPoint = statusChange.pointTag;
        }
      });

      // 监听测试进度更新事件
      const unlistenProgressUpdate = await listen('test-progress-update', (event) => {
        console.log('📊 [TEST_AREA] 收到测试进度更新事件:', event.payload);

        const progressData = event.payload as {
          batchId: string;
          totalPoints: number;
          completedPoints: number;
          successPoints: number;
          failedPoints: number;
          progressPercentage: number;
          currentPoint?: string;
        };

        // 只有当批次ID匹配时才更新进度
        if (progressData.batchId === this.selectedBatch?.batch_id) {
          this.testProgress.totalPoints = progressData.totalPoints;
          this.testProgress.completedPoints = progressData.completedPoints;
          this.testProgress.successPoints = progressData.successPoints;
          this.testProgress.failedPoints = progressData.failedPoints;
          this.testProgress.progressPercentage = progressData.progressPercentage;
          this.testProgress.currentPoint = progressData.currentPoint;

          // 检查是否完成
          if (this.testProgress.progressPercentage >= 100 && !this.isTestCompleted) {
            this.isTestCompleted = true;
            this.isAutoTesting = false;
            this.testProgress.currentPoint = undefined;
            this.message.success('批次测试已完成！', { nzDuration: 5000 });
          }

          console.log('📊 [TEST_AREA] 测试进度已更新:', this.testProgress);
        }
      });

      // 监听批次状态变化事件
      const unlistenBatchStatusChanged = await listen('batch-status-changed', (event) => {
        console.log('📋 [TEST_AREA] 收到批次状态变化事件:', event.payload);

        const batchStatusData = event.payload as {
          batchId: string;
          status: string;
          statistics: {
            total_channels: number;
            tested_channels: number;
            passed_channels: number;
            failed_channels: number;
            skipped_channels: number;
            in_progress_channels: number;
          };
        };

        // 只有当批次ID匹配时才更新状态
        if (batchStatusData.batchId === this.selectedBatch?.batch_id) {
          console.log('📋 [TEST_AREA] 更新批次状态:', batchStatusData.status);

          // 更新测试进度
          this.testProgress.totalPoints = batchStatusData.statistics.total_channels;
          this.testProgress.completedPoints = batchStatusData.statistics.tested_channels;
          this.testProgress.successPoints = batchStatusData.statistics.passed_channels;
          this.testProgress.failedPoints = batchStatusData.statistics.failed_channels;
          this.testProgress.progressPercentage = this.testProgress.totalPoints > 0
            ? (this.testProgress.completedPoints / this.testProgress.totalPoints) * 100
            : 0;

          // 检查批次是否完成
          if (batchStatusData.status === 'completed' && !this.isTestCompleted) {
            this.isTestCompleted = true;
            this.isAutoTesting = false;
            this.testProgress.currentPoint = undefined;
            this.message.success('批次测试已完成！', { nzDuration: 5000 });

            // 🔧 优化：批次完成后智能刷新
            this.scheduleDataRefresh('batch-completed', 1200);
          }

          console.log('📋 [TEST_AREA] 批次状态已更新:', this.testProgress);
        }
      });

      // 在组件销毁时清理事件监听器
      this.subscriptions.add({
        unsubscribe: () => {
          unlistenCompleted();
          unlistenStatusChanged();
          unlistenProgressUpdate();
          unlistenBatchStatusChanged();
        }
      });

      console.log('✅ [TEST_AREA] 测试事件监听器设置成功');
    } catch (error) {
      console.error('❌ [TEST_AREA] 设置事件监听器失败:', error);

      // 如果事件监听失败，回退到定时器轮询
      this.setupPollingFallback();
    }
  }

  /**
   * 回退到定时器轮询方式
   */
  private setupPollingFallback(): void {
    console.log('🔄 [TEST_AREA] 定时器轮询已禁用，避免无限循环');

    // 暂时禁用定时器轮询，避免无限循环
    // TODO: 重新设计轮询机制，只在必要时启用
    /*
    const intervalId = setInterval(async () => {
      if (this.selectedBatch && this.isAutoTesting) {
        // 移除频繁的日志输出，避免控制台噪音
        // console.log('🔄 [TEST_AREA] 定时刷新批次状态');
        await this.loadBatchDetails();
      }
    }, 2000); // 每2秒刷新一次

    // 在组件销毁时清理定时器
    this.subscriptions.add({
      unsubscribe: () => clearInterval(intervalId)
    });
    */
  }

  /**
   * 更新实例状态
   */
  private updateInstanceStatus(testResult: any): void {
    if (!this.batchDetails?.instances) return;

    // 查找对应的实例
    const instance = this.batchDetails.instances.find(inst =>
      inst.instance_id === testResult.instanceId
    );

    if (instance) {
      // 更新状态
      if (testResult.success) {
        instance.overall_status = OverallTestStatus.TestCompletedPassed;
      } else {
        instance.overall_status = OverallTestStatus.TestCompletedFailed;
      }

      // 🔧 性能优化：延迟更新统计信息
      this.scheduleStatsUpdate();

      console.log(`🔄 [TEST_AREA] 已更新实例状态: ${testResult.instanceId} -> ${instance.overall_status}`);
    } else {
      console.warn(`⚠️ [TEST_AREA] 未找到实例: ${testResult.instanceId}`);
    }
  }

  /**
   * 更新测试进度
   */
  private updateTestProgressFromResult(testResult: any): void {
    if (!this.batchDetails?.instances) return;

    // 添加到最近测试结果
    const definition = this.getDefinitionByInstanceId(testResult.instanceId);
    if (definition) {
      this.recentTestResults.push({
        pointTag: definition.tag || testResult.instanceId,
        success: testResult.success,
        message: testResult.message || '',
        timestamp: new Date()
      });

      // 只保留最近10个结果
      if (this.recentTestResults.length > 10) {
        this.recentTestResults = this.recentTestResults.slice(-10);
      }
    }

    // 重新计算进度统计
    this.calculateTestProgress();
  }

  /**
   * 计算测试进度
   */
  private calculateTestProgress(): void {
    if (!this.batchDetails?.instances) {
      this.testProgress = {
        totalPoints: 0,
        completedPoints: 0,
        successPoints: 0,
        failedPoints: 0,
        progressPercentage: 0,
        currentPoint: undefined,
        estimatedTimeRemaining: undefined
      };
      return;
    }

    const instances = this.batchDetails.instances;
    const totalPoints = instances.length;
    let completedPoints = 0;
    let successPoints = 0;
    let failedPoints = 0;
    let testingPoints = 0;

    // 统计各种状态的点位数量
    instances.forEach(instance => {
      switch (instance.overall_status) {
        case OverallTestStatus.TestCompletedPassed:
          completedPoints++;
          successPoints++;
          break;
        case OverallTestStatus.TestCompletedFailed:
          completedPoints++;
          failedPoints++;
          break;
        case OverallTestStatus.HardPointTesting:
        case OverallTestStatus.AlarmTesting:
          testingPoints++;
          break;
        case OverallTestStatus.WiringConfirmed:
        case OverallTestStatus.NotTested:
        default:
          // 这些状态不计入已完成
          break;
      }
    });

    const progressPercentage = totalPoints > 0 ? (completedPoints / totalPoints) * 100 : 0;

    this.testProgress = {
      totalPoints,
      completedPoints,
      successPoints,
      failedPoints,
      progressPercentage,
      currentPoint: this.testProgress.currentPoint,
      estimatedTimeRemaining: this.testProgress.estimatedTimeRemaining
    };

    // 检查是否完成 - 只有当所有点位都测试完成时才算完成
    const allCompleted = completedPoints === totalPoints && testingPoints === 0;
    if (allCompleted && !this.isTestCompleted) {
      this.isTestCompleted = true;
      this.isAutoTesting = false;
      this.testProgress.currentPoint = undefined;
      console.log('🎉 [TEST_AREA] 批次测试已完成！成功:', successPoints, '失败:', failedPoints);
    } else if (testingPoints === 0 && this.isAutoTesting && completedPoints > 0) {
      // 如果没有正在测试的点位，但还有未完成的，可能测试已停止
      this.isAutoTesting = false;
      console.log('⚠️ [TEST_AREA] 测试已停止，但可能未完全完成');
    }

    console.log('📊 [TEST_AREA] 测试进度统计:', {
      totalPoints,
      completedPoints,
      successPoints,
      failedPoints,
      testingPoints,
      progressPercentage: progressPercentage.toFixed(1) + '%'
    });
  }

  /**
   * 直接更新实例状态（用于状态变化事件）
   */
  private updateInstanceStatusDirect(instanceId: string, newStatus: OverallTestStatus): void {
    if (!this.batchDetails?.instances) return;

    // 查找对应的实例
    const instance = this.batchDetails.instances.find(inst =>
      inst.instance_id === instanceId
    );

    if (instance) {
      // 直接更新状态
      instance.overall_status = newStatus;

      // 🔧 性能优化：延迟更新统计信息
      this.scheduleStatsUpdate();

      console.log(`🔄 [TEST_AREA] 直接更新实例状态: ${instanceId} -> ${newStatus}`);
    } else {
      console.warn(`⚠️ [TEST_AREA] 未找到实例: ${instanceId}`);
    }
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
      // 初始化测试进度
      this.initializeTestProgress();

      // 调用后端API开始自动测试
      const result = await this.tauriApiService.startBatchAutoTest(this.selectedBatch.batch_id).toPromise();

      if (result && result.success) {
        this.message.success('🚀 通道自动测试已启动', { nzDuration: 2000 });
        console.log('✅ [TEST_AREA] 通道自动测试启动成功');

        // 🔧 优化：测试启动后智能刷新
        this.scheduleDataRefresh('test-started', 800);

        // 测试启动成功，保持 isAutoTesting = true，直到测试完成
        console.log('✅ [TEST_AREA] 测试已启动，等待测试完成...');
      } else {
        throw new Error((result && result.message) || '启动自动测试失败');
      }
    } catch (error) {
      console.error('❌ [TEST_AREA] 启动自动测试失败:', error);
      this.message.error('启动自动测试失败: ' + error);

      // 启动失败时重置状态
      this.isAutoTesting = false;
      this.isTestCompleted = false;
    }
  }

  /**
   * 手动刷新批次详情
   */
  async refreshBatchDetails(): Promise<void> {
    if (!this.selectedBatch) {
      this.message.warning('请先选择一个测试批次');
      return;
    }

    console.log('🔄 [TEST_AREA] 手动刷新批次详情');
    this.message.info('正在刷新批次状态...', { nzDuration: 1000 });

    try {
      // 🔧 优化：清除所有待执行的刷新，立即执行手动刷新
      this.refreshTimeouts.forEach(timeoutId => clearTimeout(timeoutId));
      this.refreshTimeouts.clear();

      await this.loadBatchDetails();
      this.lastRefreshTime = Date.now(); // 更新最后刷新时间
      this.message.success('批次状态已刷新', { nzDuration: 2000 });
    } catch (error) {
      console.error('❌ [TEST_AREA] 刷新批次详情失败:', error);
      this.message.error('刷新失败: ' + error);
    }
  }

  /**
   * 检查测试完成状态
   */
  private checkTestCompletionStatus(): void {
    if (!this.batchDetails?.instances) return;

    const instances = this.batchDetails.instances;
    const totalPoints = instances.length;
    let completedPoints = 0;
    let testingPoints = 0;

    // 统计各种状态的点位数量
    instances.forEach(instance => {
      switch (instance.overall_status) {
        case OverallTestStatus.TestCompletedPassed:
        case OverallTestStatus.TestCompletedFailed:
          completedPoints++;
          break;
        case OverallTestStatus.HardPointTesting:
        case OverallTestStatus.AlarmTesting:
          testingPoints++;
          break;
      }
    });

    console.log('🔍 [TEST_AREA] 检查测试完成状态:', {
      totalPoints,
      completedPoints,
      testingPoints,
      isTestCompleted: this.isTestCompleted,
      isAutoTesting: this.isAutoTesting
    });

    // 如果所有点位都已完成测试，且当前状态不是已完成
    if (completedPoints === totalPoints && testingPoints === 0 && !this.isTestCompleted) {
      console.log('🎉 [TEST_AREA] 检测到测试已完成，更新状态');
      this.isTestCompleted = true;
      this.isAutoTesting = false;
      this.testProgress.currentPoint = undefined;
      this.message.success('批次测试已完成！', { nzDuration: 5000 });
    }

    // 如果没有正在测试的点位，但还有未完成的，可能测试已停止
    else if (testingPoints === 0 && this.isAutoTesting && completedPoints < totalPoints) {
      console.log('⚠️ [TEST_AREA] 检测到测试已停止，但未完全完成');
      this.isAutoTesting = false;
    }
  }





  /**
   * 初始化测试进度
   */
  private initializeTestProgress(): void {
    const totalPoints = this.batchDetails?.instances?.length || 0;

    this.testProgress = {
      totalPoints,
      completedPoints: 0,
      successPoints: 0,
      failedPoints: 0,
      progressPercentage: 0,
      currentPoint: undefined,
      estimatedTimeRemaining: undefined
    };

    this.isTestCompleted = false;
    this.recentTestResults = [];

    // 计算当前进度（可能有已完成的测试）
    this.calculateTestProgress();

    console.log('🔧 [TEST_AREA] 测试进度已初始化:', this.testProgress);
  }

  /**
   * 获取测试状态颜色
   */
  getTestStatusColor(): string {
    if (this.isTestCompleted) {
      return this.testProgress.failedPoints > 0 ? 'warning' : 'success';
    } else if (this.isAutoTesting) {
      return 'processing';
    } else {
      return 'default';
    }
  }

  /**
   * 获取测试状态文本
   */
  getTestStatusText(): string {
    if (this.isTestCompleted) {
      return this.testProgress.failedPoints > 0 ? '测试完成(有失败)' : '测试完成(全部通过)';
    } else if (this.isAutoTesting) {
      return '测试进行中';
    } else {
      return '等待开始';
    }
  }

  /**
   * 获取进度条状态
   */
  getProgressStatus(): 'success' | 'exception' | 'active' | 'normal' {
    if (this.isTestCompleted) {
      return this.testProgress.failedPoints > 0 ? 'exception' : 'success';
    } else if (this.isAutoTesting) {
      return 'active';
    } else {
      return 'normal';
    }
  }

  /**
   * 获取进度条颜色
   */
  getProgressColor(): string {
    if (this.isTestCompleted) {
      return this.testProgress.failedPoints > 0 ? '#ff4d4f' : '#52c41a';
    } else if (this.isAutoTesting) {
      return '#1890ff';
    } else {
      return '#d9d9d9';
    }
  }

  /**
   * 获取正在测试的点位数量
   */
  getTestingCount(): number {
    if (!this.batchDetails?.instances) return 0;

    return this.batchDetails.instances.filter(instance =>
      instance.overall_status === OverallTestStatus.HardPointTesting ||
      instance.overall_status === OverallTestStatus.AlarmTesting
    ).length;
  }

  async loadBatchDetails(): Promise<void> {
    if (!this.selectedBatch) {
      // console.log('⚠️ [TEST_AREA] 没有选择批次，无法加载详情');
      this.message.warning('请先选择一个测试批次');
      return;
    }

    // console.log('📊 [TEST_AREA] 开始加载批次详情');
    // console.log('📊 [TEST_AREA] 选中的批次ID:', this.selectedBatch.batch_id);
    this.isLoadingDetails = true;
    try {
      // 调用真实的后端API获取批次详情
      // console.log('📊 [TEST_AREA] 调用后端API: getBatchDetails()');

      // 🔧 优化：直接获取数据，避免重试导致的双倍请求
      const details = await firstValueFrom(this.tauriApiService.getBatchDetails(this.selectedBatch.batch_id));

      // console.log('📊 [TEST_AREA] 后端返回的详情数据:', details);

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

        // console.log('✅ [TEST_AREA] 批次详情加载成功');
        // console.log('✅ [TEST_AREA] 实例数量:', this.batchDetails.instances.length);
        // console.log('✅ [TEST_AREA] 定义数量:', this.batchDetails.definitions.length);

        // 移除成功消息，因为这个方法会被定时器频繁调用
        // this.message.success('批次详情加载成功');

        // 🔧 性能优化：重建定义缓存和清理过滤缓存
        this.rebuildDefinitionCache();
        this._filteredInstances = [];
        this._lastFilterHash = '';

        this.updateModuleTypeStats();

        // 更新测试进度
        this.calculateTestProgress();

        // 强制检查测试完成状态
        this.checkTestCompletionStatus();
      } else {
        // console.error('❌ [TEST_AREA] 后端返回空的详情数据');
        throw new Error('未找到批次详情数据');
      }
    } catch (error) {
      // console.error('❌ [TEST_AREA] 加载批次详情失败:', error);
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

  // 🔧 性能优化：缓存的定义查找方法
  private getDefinitionByInstanceIdCached(instanceId: string): ChannelPointDefinition | undefined {
    // 如果缓存为空，重建缓存
    if (this._definitionMap.size === 0 && this.batchDetails?.definitions && this.batchDetails?.instances) {
      this.rebuildDefinitionCache();
    }

    return this._definitionMap.get(instanceId);
  }

  // 🔧 性能优化：重建定义缓存
  private rebuildDefinitionCache(): void {
    this._definitionMap.clear();
    if (this.batchDetails?.definitions && this.batchDetails?.instances) {
      // 建立 instanceId -> definition 的映射
      this.batchDetails.instances.forEach(instance => {
        const definition = this.batchDetails!.definitions!.find(def => def.id === instance.definition_id);
        if (definition) {
          this._definitionMap.set(instance.instance_id, definition);
        }
      });
    }
  }

  getInstanceStatusColor(status: OverallTestStatus): string {
    switch (status) {
      case OverallTestStatus.NotTested: return 'default';
      case OverallTestStatus.WiringConfirmed: return 'blue';
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
      case OverallTestStatus.WiringConfirmed: return '接线确认';
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

  // 🔧 性能优化：使用getter返回缓存的过滤结果，保持数据一致性
  get filteredInstances(): ChannelTestInstance[] {
    if (!this.batchDetails?.instances) return [];

    // 计算当前过滤条件的哈希值
    const currentFilterHash = this.calculateFilterHash();

    // 如果过滤条件没有变化，返回缓存的结果
    if (currentFilterHash === this._lastFilterHash && this._filteredInstances.length > 0) {
      return this._filteredInstances;
    }

    // 重新计算过滤结果 - 保持原有逻辑完全一致
    this._filteredInstances = this.computeFilteredInstances();
    this._lastFilterHash = currentFilterHash;

    return this._filteredInstances;
  }

  // 🔧 性能优化：计算过滤条件哈希值
  private calculateFilterHash(): string {
    return JSON.stringify({
      selectedModuleTypes: this.selectedModuleTypes.sort(),
      searchText: this.searchText.trim().toLowerCase(),
      showOnlyTested: this.showOnlyTested,
      showOnlyFailed: this.showOnlyFailed,
      instancesLength: this.batchDetails?.instances?.length || 0,
      // 添加实例状态变化的检测
      instancesHash: this.batchDetails?.instances?.map(i => `${i.instance_id}:${i.overall_status}`).join(',') || ''
    });
  }

  // 🔧 性能优化：实际的过滤计算逻辑 - 保持原有逻辑完全一致
  private computeFilteredInstances(): ChannelTestInstance[] {
    if (!this.batchDetails?.instances) return [];

    return this.batchDetails.instances.filter(instance => {
      const definition = this.getDefinitionByInstanceIdCached(instance.instance_id);

      // 模块类型筛选 - 保持原有逻辑
      if (this.selectedModuleTypes.length > 0 && definition) {
        if (!this.selectedModuleTypes.includes(definition.module_type)) {
          return false;
        }
      }

      // 搜索文本筛选 - 保持原有逻辑
      if (this.searchText.trim()) {
        const searchLower = this.searchText.toLowerCase();
        const matchesTag = definition?.tag?.toLowerCase().includes(searchLower);
        const matchesVariable = definition?.variable_name?.toLowerCase().includes(searchLower);
        const matchesDescription = definition?.description?.toLowerCase().includes(searchLower);

        if (!matchesTag && !matchesVariable && !matchesDescription) {
          return false;
        }
      }

      // 测试状态筛选 - 保持原有逻辑
      if (this.showOnlyTested) {
        if (instance.overall_status === OverallTestStatus.NotTested) {
          return false;
        }
      }

      // 失败状态筛选 - 保持原有逻辑
      if (this.showOnlyFailed) {
        if (instance.overall_status !== OverallTestStatus.TestCompletedFailed) {
          return false;
        }
      }

      return true;
    });
  }

  // 🔧 保持向后兼容的方法
  getFilteredInstances(): ChannelTestInstance[] {
    return this.filteredInstances;
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
    // console.log('🔍 [TEST_AREA] 检查是否有未持久化的数据');
    const testData = this.dataStateService.getTestData();

    if (testData.isDataAvailable && testData.parsedDefinitions.length > 0) {
      // console.log('⚠️ [TEST_AREA] 检测到未持久化的测试数据');
      // console.log('⚠️ [TEST_AREA] 这表明点表导入流程可能没有正确完成批次分配');

      // 清理内存中的数据，因为批次应该已经在导入时创建
      this.dataStateService.clearTestData();
      this.message.warning('检测到未完成的导入流程，请重新导入点表以创建批次');

      // 重新加载批次列表，查看是否有新创建的批次
      this.loadAvailableBatches();
    } else {
      // console.log('✅ [TEST_AREA] 没有未持久化的数据，正常加载批次列表');
    }
  }

  /**
   * 显示错误详情
   */
  showErrorDetail(instance: ChannelTestInstance): void {
    // console.log('🔍 [TEST_AREA] DJDJDJDJ');
    // console.log('🔍 [TEST_AREA] 显示错误详情:', instance.instance_id);
    // console.log('🔍 [TEST_AREA] 实例完整数据:', instance);
    // console.log('🔍 [TEST_AREA] digital_test_steps 字段:', instance.digital_test_steps);
    // console.log('🔍 [TEST_AREA] digital_test_steps 长度:', instance.digital_test_steps?.length);

    // 查找对应的通道定义
    const definition = this.getDefinitionByInstanceId(instance.instance_id);
    if (!definition) {
      this.message.error('未找到通道定义信息');
      return;
    }

    // console.log('🔍 [TEST_AREA] 找到定义:', definition);
    // console.log('🔍 [TEST_AREA] 定义模块类型:', definition.module_type);

    this.selectedErrorInstance = instance;
    this.selectedErrorDefinition = definition;
    this.errorDetailModalVisible = true;
  }

  /**
   * 关闭错误详情模态框
   */
  closeErrorDetailModal(): void {
    this.errorDetailModalVisible = false;
    this.selectedErrorInstance = null;
    this.selectedErrorDefinition = null;
  }

  /**
   * 检查是否有错误详情可显示
   */
  hasErrorDetails(instance: ChannelTestInstance): boolean {
    // console.log('------------------------');
    // console.log('🔍 [TEST_AREA] hasErrorDetails 检查:', instance.instance_id);
    // console.log('🔍 [TEST_AREA] error_messageaa:', instance.error_message);
    // console.log('🔍 [TEST_AREA] overall_status:', instance.overall_status);
    // console.log('🔍 [TEST_AREA] sub_test_results:', instance.sub_test_results);

    // 检查是否有错误信息或失败的子测试结果
    if (instance.error_message && instance.error_message.trim()) {
      // console.log('🔍 [TEST_AREA] 有错误信息，返回 true');
      return true;
    }

    // 检查是否有失败的子测试结果
    if (instance.sub_test_results) {
      for (const [testItem, result] of Object.entries(instance.sub_test_results)) {
        // console.log(`🔍 [TEST_AREA] 检查子测试 ${testItem}:`, result);
        if (result.status === SubTestStatus.Failed && result.details) {
          // console.log('🔍 [TEST_AREA] 找到失败的子测试，返回 true');
          return true;
        }
      }
    }

    // 如果状态是失败但没有具体错误信息，也显示按钮
    const shouldShow = instance.overall_status === OverallTestStatus.TestCompletedFailed;
    // console.log('🔍 [TEST_AREA] 最终判断结果:', shouldShow);
    return shouldShow;
  }

  /**
   * 检查单个通道测试按钮是否应该禁用
   */
  isChannelTestDisabled(instance: ChannelTestInstance): boolean {
    // 当状态为"通过"或"测试中"时禁用按钮
    return instance.overall_status === OverallTestStatus.TestCompletedPassed ||
           instance.overall_status === OverallTestStatus.HardPointTesting;
  }

  /**
   * 获取单个通道测试按钮的文本
   */
  getChannelTestButtonText(instance: ChannelTestInstance): string {
    if (instance.overall_status === OverallTestStatus.HardPointTesting) {
      return '测试中...';
    }
    return '硬点重测';
  }

  /**
   * 开始单个通道的硬点测试
   */
  async startSingleChannelTest(instance: ChannelTestInstance): Promise<void> {
    try {
      console.log('🚀 [TEST_AREA] 开始单个通道硬点测试:', instance.instance_id);

      // 调用后端API开始单个通道测试
      await firstValueFrom(this.tauriApiService.startSingleChannelTest(instance.instance_id));

      console.log('✅ [TEST_AREA] 单个通道硬点测试已启动:', instance.instance_id);

      // 可选：显示成功消息
      // this.message.success('硬点测试已启动');

    } catch (error) {
      console.error('❌ [TEST_AREA] 启动单个通道硬点测试失败:', error);
      this.message.error(`启动硬点测试失败: ${error}`);
    }
  }

  /**
   * 获取表格行的CSS类名（用于整行颜色变更）
   */
  getRowClassName = (data: ChannelTestInstance, index: number): string => {
    // 硬点测试失败 → 红色
    if (data.overall_status === OverallTestStatus.TestCompletedFailed) {
      return 'row-failed';
    }

    // 测试完成且通过 → 绿色
    if (data.overall_status === OverallTestStatus.TestCompletedPassed) {
      return 'row-passed';
    }

    return '';
  }

  /**
   * 检查手动测试按钮是否启用
   */
  isManualTestEnabled(instance: ChannelTestInstance): boolean {
    // 只有硬点测试通过后才允许手动测试
    return instance.overall_status === OverallTestStatus.HardPointTestCompleted ||
           instance.overall_status === OverallTestStatus.TestCompletedPassed ||
           instance.overall_status === OverallTestStatus.ManualTesting;
  }

  /**
   * 开始手动测试
   */
  async startManualTest(instance: ChannelTestInstance): Promise<void> {
    try {
      console.log('🔧 [TEST_AREA] 开始手动测试:', instance.instance_id);

      // 获取通道定义信息
      const definition = this.getDefinitionByInstanceId(instance.instance_id);
      if (!definition) {
        this.message.error('无法找到通道定义信息');
        return;
      }

      // 设置选中的实例和定义
      this.selectedManualTestInstance = instance;
      this.selectedManualTestDefinition = definition;

      // 打开手动测试模态框
      this.manualTestModalVisible = true;

      console.log('✅ [TEST_AREA] 手动测试模态框已打开');

    } catch (error) {
      console.error('❌ [TEST_AREA] 启动手动测试失败:', error);
      this.message.error(`启动手动测试失败: ${error}`);
    }
  }

  /**
   * 手动测试完成处理
   */
  onManualTestCompleted(): void {
    console.log('🎉 [TEST_AREA] 手动测试完成');
    this.closeManualTestModal();

    // 刷新批次详情以获取最新状态
    this.loadBatchDetails();
  }

  /**
   * 关闭手动测试模态框
   */
  closeManualTestModal(): void {
    this.manualTestModalVisible = false;
    this.selectedManualTestInstance = null;
    this.selectedManualTestDefinition = null;
  }


}