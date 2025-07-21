import { Component, OnInit, OnDestroy, ViewChild, ChangeDetectorRef } from '@angular/core';
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
import { NzModalModule, NzModalService, NzModalRef } from 'ng-zorro-antd/modal';
import { listen } from '@tauri-apps/api/event';
// Tauri 对话框 API：按需导入 save 方法
import { save as saveDialog } from '@tauri-apps/plugin-dialog';

import { TauriApiService } from '../../services/tauri-api.service';
import { ManualTestService } from '../../services/manual-test.service';
import { DataStateService } from '../../services/data-state.service';
import { BatchSelectionService } from '../../services/batch-selection.service';
import { Subscription, firstValueFrom } from 'rxjs';
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
import { ManualTestStatus } from '../../models/manual-test.types';
import { ErrorDetailModalComponent } from './error-detail-modal.component';
import { ManualTestModalComponent } from '../manual-test/manual-test-modal.component';
import { ErrorNotesModalComponent } from './error-notes-modal.component';

// 批次测试统计接口
interface BatchTestStats {
  totalPoints: number;
  pendingPoints: number;
  testedPoints: number;
  successPoints: number;
  failedPoints: number;
  skippedPoints: number;
  startedPoints: number; // 已开始测试的点位数（包括中间状态）
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
    ManualTestModalComponent,
    ErrorNotesModalComponent
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
  /** 失败点位重测状态 */
  isRetestingFailed = false;
  /** 当前处于硬点/报警测试状态的实例数量 */
  activeHardpointTests = 0;

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
  private readonly MIN_REFRESH_INTERVAL = 300; // 最小刷新间隔优化为300ms
  private readonly CRITICAL_REFRESH_INTERVAL = 100; // 关键状态变化立即刷新间隔100ms
  private readonly INSTANT_UPDATE_INTERVAL = 50; // 即时更新间隔50ms

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
  showOnlyPassed = false;
  showOnlyHardPointPassed = false;
  showOnlyNotTested = false;

  // 🔧 性能优化：缓存过滤结果
  private _filteredInstances: ChannelTestInstance[] = [];
  private _lastFilterHash = '';

  // 🔧 性能优化：缓存定义映射
  private _definitionMap = new Map<string, ChannelPointDefinition>();

  // 🔧 性能优化：防抖处理
  private _searchDebounceTimer: any = null;
  private _statsUpdateTimer: any = null;
  private _progressUpdateTimer: any = null; // 🔧 新增：进度更新防抖定时器

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
  private hardPointModalRef?: NzModalRef;
  manualTestModalVisible = false;
  selectedManualTestInstance: ChannelTestInstance | null = null;
  selectedManualTestDefinition: ChannelPointDefinition | null = null;

  // 错误备注模态框相关
  errorNotesModalVisible = false;
  selectedErrorNotesInstance: ChannelTestInstance | null = null;
  selectedErrorNotesDefinition: ChannelPointDefinition | null = null;

  constructor(
    private tauriApiService: TauriApiService,
    private message: NzMessageService,
    private dataStateService: DataStateService,
    private batchSelectionService: BatchSelectionService,
    private modal: NzModalService,
    private manualTestService: ManualTestService,
    private cdr: ChangeDetectorRef
  ) {}

  ngOnInit(): void {
    // 订阅手动测试状态变化，实时刷新实例状态
    // 订阅当前手动测试状态（启动测试时会推送一次）
    this.subscriptions.add(
      this.manualTestService.currentTestStatus$.subscribe(status => {
        this.applyManualTestStatus(status);
      })
    );

    // 订阅实时状态更新流，实现 UI 实时刷新
    this.subscriptions.add(
      this.manualTestService.testStatusUpdated$.subscribe(status => {
        this.applyManualTestStatus(status);
      })
    );
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
    if (this._progressUpdateTimer) {
      clearTimeout(this._progressUpdateTimer);
      this._progressUpdateTimer = null;
    }
  }

  /**
   * 根据手动测试状态更新对应实例，触发变更检测
   */
  private applyManualTestStatus(status: ManualTestStatus | null): void {
    if (!status || !this.batchDetails) {
      return;
    }
    const inst = this.batchDetails.instances?.find((i: ChannelTestInstance) => i.instance_id === status.instanceId);
    if (inst) {
      const oldStatus = inst.overall_status;
      inst.overall_status = status.overallStatus as any;
      
      // 如果有错误信息也同步更新
      if (status.errorMessage !== undefined) {
        (inst as any).error_message = status.errorMessage;
      }
      
      console.log(`🔄 [TEST_AREA] 手动测试状态更新: ${status.instanceId} - ${oldStatus} -> ${status.overallStatus}`);
      
      // 🔧 新增：强制清理缓存确保状态变化立即反映
      this.smartCacheRefresh('status');
      
      // 🔧 新增：立即刷新数据以确保持久化状态同步
      this.scheduleDataRefresh('manual-test-status-changed', this.INSTANT_UPDATE_INTERVAL);
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
        // 批次切换时重置 PLC 连接状态，确保“确认接线”按钮可点击
        this.isConnected = false;
        this.isConnecting = false;
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
      
      // 🔧 新增：刷新完成后触发变更检测
      this.cdr.detectChanges();
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

        // 🔧 去重检查：避免重复计数同一个实例的完成事件
        if (this.completedTestInstances.has(testResult.instanceId)) {
          console.log(`⚠️ [TEST_AREA] 实例 ${testResult.instanceId} 已处理过，跳过重复计数`);
          return;
        }

        // 添加到已完成集合
        this.completedTestInstances.add(testResult.instanceId);

        // 更新本地状态
        this.updateInstanceStatus(testResult);

        // 更新测试进度
        this.updateTestProgressFromResult(testResult);

        // 🔧 新增：测试完成计数逻辑
        this.completedTestCount++;
        console.log(`📊 [TEST_AREA] 测试完成计数: ${this.completedTestCount}/${this.expectedTestCount} (实例: ${testResult.instanceId})`);
        console.log(`📊 [TEST_AREA] 已完成实例列表: [${Array.from(this.completedTestInstances).join(', ')}]`);

        // 🔧 防护检查：确保计数不会超过预期值
        if (this.completedTestCount > this.expectedTestCount) {
          console.warn(`⚠️ [TEST_AREA] 异常：完成计数 ${this.completedTestCount} 超过预期 ${this.expectedTestCount}`);
        }

        // 🔧 检查是否所有测试都已完成
        if (this.isTestingModalVisible && this.expectedTestCount > 0 && this.completedTestCount >= this.expectedTestCount) {
          console.log('🎉 [TEST_AREA] 所有测试已完成，关闭弹窗');
          this.closeTestingModal();
        }

        // 🔧 新增：智能缓存刷新确保数据一致性
        this.smartCacheRefresh('complete');

        // 🔧 优化：测试完成后即时刷新，确保UI及时更新
        this.scheduleDataRefresh('test-completed', this.INSTANT_UPDATE_INTERVAL);

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

        // 🔧 简化弹窗控制：只用于显示弹窗，不用于关闭
        if (statusChange.newStatus === OverallTestStatus.HardPointTesting || statusChange.newStatus === OverallTestStatus.AlarmTesting) {
          // 如果弹窗未显示，则显示弹窗
          if (!this.isTestingModalVisible) {
            console.log('🔧 [TEST_AREA] 检测到硬点测试开始，显示弹窗');
            this.openTestingModal();
          }
        }

        // 更新整体进度
        this.updateOverallProgress();

        // 🔧 新增：智能缓存刷新确保状态变化及时反映
        this.smartCacheRefresh('status');

        // 🔧 优化：测试状态变化后即时刷新
        this.scheduleDataRefresh('test-status-changed', this.INSTANT_UPDATE_INTERVAL);
      });

      // 🔧 新增：监听错误状态变化事件，实时更新错误信息
      const unlistenErrorStatusChanged = await listen('error-status-changed', (event) => {
        console.log('🚨 [TEST_AREA] 收到错误状态变化事件:', event.payload);

        const errorChange = event.payload as {
          instanceId: string;
          errorMessage?: string;
          hasError: boolean;
          timestamp: string;
        };

        // 更新本地实例的错误信息
        const inst = this.batchDetails?.instances?.find(i => i.instance_id === errorChange.instanceId);
        if (inst) {
          (inst as any).error_message = errorChange.hasError ? errorChange.errorMessage : null;
          
          console.log(`🚨 [TEST_AREA] 错误状态已更新: ${errorChange.instanceId} - ${errorChange.hasError ? '有错误' : '无错误'}`);
        }

        // 🔧 新增：智能缓存刷新确保错误信息及时显示
        this.smartCacheRefresh('error');

        // 立即刷新数据以确保一致性
        this.scheduleDataRefresh('error-status-changed', this.INSTANT_UPDATE_INTERVAL);
      });

      // 🔧 新增：监听测试进度变化事件，实时更新进度信息
      const unlistenProgressChanged = await listen('test-progress-changed', (event) => {
        console.log('📊 [TEST_AREA] 收到测试进度变化事件:', event.payload);

        const progressChange = event.payload as {
          batchId: string;
          completedCount: number;
          totalCount: number;
          progressPercentage: number;
        };

        // 只更新当前批次的进度
        if (this.selectedBatch && this.selectedBatch.batch_id === progressChange.batchId) {
          this.testProgress.completedPoints = progressChange.completedCount;
          this.testProgress.totalPoints = progressChange.totalCount;
          this.testProgress.progressPercentage = progressChange.progressPercentage;
          
          // 立即触发变更检测
          this.cdr.detectChanges();
        }
      });

      // 🔧 新增：监听实例详情变化事件，实时更新实例详情
      const unlistenInstanceDetailChanged = await listen('instance-detail-changed', (event) => {
        console.log('🔄 [TEST_AREA] 收到实例详情变化事件:', event.payload);

        const detailChange = event.payload as {
          instanceId: string;
          field: string;
          value: any;
          timestamp: string;
        };

        // 更新本地实例的具体字段
        const inst = this.batchDetails?.instances?.find(i => i.instance_id === detailChange.instanceId);
        if (inst) {
          (inst as any)[detailChange.field] = detailChange.value;
          
          console.log(`🔄 [TEST_AREA] 实例详情已更新: ${detailChange.instanceId}.${detailChange.field} = ${detailChange.value}`);
          
          // 根据字段类型智能刷新
          if (detailChange.field.includes('status')) {
            this.smartCacheRefresh('status');
          } else if (detailChange.field.includes('error')) {
            this.smartCacheRefresh('error');
          } else {
            this.smartCacheRefresh('complete');
          }
        }
      });

      // 组件销毁时自动注销事件监听
      this.subscriptions.add({
        unsubscribe: () => {
          unlistenCompleted();
          unlistenStatusChanged();
          unlistenErrorStatusChanged();
          unlistenProgressChanged();
          unlistenInstanceDetailChanged();
        }
      });
    } catch (error) {
      console.error('❌ [TEST_AREA] 设置测试结果监听失败:', error);
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

      // 如果当前没有选中批次，自动选择第一个批次，避免通道表格区域为空
      this.batchSelectionService.autoSelectFirstBatch();

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

  /**
   * 在切换批次时立即重置进度并加载详情
   */
  selectBatch(batch: TestBatchInfo): void {
    // 更新批次选择服务
    this.batchSelectionService.selectBatch(batch);
    this.message.success(`已选择批次: ${batch.batch_name || batch.batch_id}`);

    // 1. 立即重置，避免显示上一批次数据
    this.resetProgress();
    this.batchDetails = null;

    // 2. 异步加载新批次详情
    this.loadBatchDetails();
  }

  /**
   * 重置总体测试进度对象
   */
  private resetProgress(): void {
    this.testProgress = {
      totalPoints: 0,
      completedPoints: 0,
      successPoints: 0,
      failedPoints: 0,
      progressPercentage: 0,
      currentPoint: undefined,
      estimatedTimeRemaining: undefined
    };
    this.isTestCompleted = false;
  }

  /**
   * 判断是否存在正在进行的硬点/报警测试
   */
  isHardpointTesting(): boolean {
    return !!this.batchDetails?.instances?.some(inst =>
      inst.overall_status === OverallTestStatus.HardPointTesting ||
      inst.overall_status === OverallTestStatus.AlarmTesting
    );
  }

  // ========= 硬点测试实例跟踪 =========
  private hardpointTestingSet = new Set<string>();
  
  // ========= 测试完成计数机制 =========
  private expectedTestCount = 0;  // 预期要完成的测试数量
  private completedTestCount = 0; // 已完成的测试数量
  private isTestingModalVisible = false; // 测试弹窗是否可见
  private completedTestInstances = new Set<string>(); // 已完成测试的实例ID，用于去重

  // ========= 硬点测试弹窗控制 =========
  private openHardPointTestingModal(): void {
    if (!this.hardPointModalRef) {
      this.hardPointModalRef = this.modal.create({
        nzTitle: '硬点通道自动测试',
        nzContent: '正在进行硬点通道测试，请稍候……',
        nzClosable: false,
        nzMaskClosable: false
      });
    }
  }

  private closeHardPointTestingModal(): void {
    if (this.hardPointModalRef) {
      this.hardPointModalRef.close();
      this.hardPointModalRef = undefined;
    }
  }

  // ========= 新的测试弹窗控制（基于完成计数） =========
  /**
   * 打开测试弹窗
   */
  private openTestingModal(): void {
    if (!this.hardPointModalRef) {
      this.hardPointModalRef = this.modal.create({
        nzTitle: '硬点通道自动测试',
        nzContent: '正在进行硬点通道测试，请稍候……',
        nzClosable: false,
        nzMaskClosable: false
      });
      this.isTestingModalVisible = true;
      console.log('🔧 [TEST_AREA] 测试弹窗已打开');
    }
  }

  /**
   * 关闭测试弹窗
   */
  private closeTestingModal(): void {
    if (this.hardPointModalRef) {
      this.hardPointModalRef.close();
      this.hardPointModalRef = undefined;
      this.isTestingModalVisible = false;
      
      // 重置计数器和去重集合
      this.expectedTestCount = 0;
      this.completedTestCount = 0;
      this.completedTestInstances.clear();
      
      console.log('🔧 [TEST_AREA] 测试弹窗已关闭，计数器和去重集合已重置');
    }
  }

  /**
   * 初始化测试计数器
   */
  private initializeTestCounter(testCount: number): void {
    const oldExpected = this.expectedTestCount;
    const oldCompleted = this.completedTestCount;
    
    this.expectedTestCount = testCount;
    this.completedTestCount = 0;
    this.completedTestInstances.clear(); // 清理去重集合
    
    console.log(`🔧 [TEST_AREA] 初始化测试计数器：${oldCompleted}/${oldExpected} → 0/${testCount}`);
    console.log(`🔧 [TEST_AREA] 已清理去重集合，当前弹窗状态：${this.isTestingModalVisible ? '显示' : '隐藏'}`);
  }

  /**
   * 根据当前实例状态刷新硬点测试弹窗（已废弃，使用新的计数机制）
   * 打开条件：至少有一个实例处于 HardPointTesting 或 AlarmTesting
   * 关闭条件：全部实例不在以上状态
   */
  private refreshHardPointModal(): void {
    // 此方法已废弃，现在使用基于 test-completed 事件的计数机制
    console.log('⚠️ [TEST_AREA] refreshHardPointModal 方法已废弃');
  }

  // ========= 进度更新辅助 =========
  private updateTestProgressFromResult(_result: { instanceId: string; success: boolean }): void {
    // 统一调用整体进度更新
    this.updateOverallProgress();
  }






  /**
   * 根据 batchDetails 重新计算总体进度并更新 testProgress
   * 🔧 修复：添加防抖机制，避免频繁更新导致闪烁
   */
  private updateOverallProgress(): void {
    // 清除之前的定时器
    if (this._progressUpdateTimer) {
      clearTimeout(this._progressUpdateTimer);
    }

    // 使用防抖机制，延迟100ms更新，避免频繁闪烁
    this._progressUpdateTimer = setTimeout(() => {
      this.doUpdateOverallProgress();
    }, 100);
  }

  /**
   * 实际执行进度更新的方法
   */
  private doUpdateOverallProgress(): void {
    const stats = this.calculateTestStatsFromDetails();

    this.testProgress.totalPoints = stats.totalPoints;
    this.testProgress.completedPoints = stats.testedPoints;
    this.testProgress.successPoints = stats.successPoints;
    this.testProgress.failedPoints = stats.failedPoints;

    this.testProgress.progressPercentage = stats.totalPoints === 0 ? 0 : Math.round((stats.testedPoints / stats.totalPoints) * 100);

    // --- 同步更新批次统计信息，避免切换批次后数据显示为初始值 ---
    if (this.selectedBatch) {
      this.selectedBatch.total_points = stats.totalPoints;
      this.selectedBatch.tested_points = stats.testedPoints;
      this.selectedBatch.passed_points = stats.successPoints;
      this.selectedBatch.failed_points = stats.failedPoints;
      this.selectedBatch.skipped_points = stats.skippedPoints;
      this.selectedBatch.started_points = stats.startedPoints; // 🔧 新增：更新已开始测试点位数

      // 同时更新 availableBatches 列表中的同批次对象（仅更新统计数据）
      const idx = this.availableBatches.findIndex(b => b.batch_id === this.selectedBatch!.batch_id);
      if (idx !== -1) {
        this.availableBatches[idx] = { ...this.availableBatches[idx], ...{
          total_points: stats.totalPoints,
          tested_points: stats.testedPoints,
          passed_points: stats.successPoints,
          failed_points: stats.failedPoints,
          skipped_points: stats.skippedPoints,
          started_points: stats.startedPoints // 🔧 新增：更新已开始测试点位数
          // 🔧 移除：状态字段不再同步，批次选择区域使用独立的状态逻辑
        } } as TestBatchInfo;
      }
    }

    // 弹窗开关逻辑已统一由 hardpointTestingSet（事件监听驱动）控制，此处不再处理

  }

  /**
   * 根据当前批次详情计算整体进度（兼容旧调用）
   */
  private calculateTestProgress(): void {
    this.updateOverallProgress();
  }

  /**
   * 更新指定实例的整体状态（来自 test-completed 事件）
   */
  private updateInstanceStatus(testResult: { instanceId: string; success: boolean }): void {
    const inst = this.batchDetails?.instances?.find(i => i.instance_id === testResult.instanceId);
    if (inst) {
      inst.overall_status = testResult.success ? OverallTestStatus.TestCompletedPassed : OverallTestStatus.TestCompletedFailed;
    }
    this.updateOverallProgress();
  }

  /**
   * 直接更新实例状态（来自 test-status-changed 事件）
   */
  private updateInstanceStatusDirect(instanceId: string, newStatus: OverallTestStatus): void {
    const inst = this.batchDetails?.instances?.find(i => i.instance_id === instanceId);
    if (inst) {
      inst.overall_status = newStatus;
    }
    this.updateOverallProgress();
  }

  /**

  /**
   * 确认接线 - 连接测试PLC和被测PLC
   */
  async confirmWiring(): Promise<void> {
    if (!this.selectedBatch) {
      this.message.warning('请先选择一个测试批次');
      return;
    }

    console.log(`🔗 [TEST_AREA] 开始确认接线，连接PLC，批次ID: ${this.selectedBatch.batch_id}`);
    this.isConnecting = true;

    try {
      // 调用后端API连接PLC，并在连接成功后自动下发量程
      const batchId = this.selectedBatch.batch_id;
      const result = await this.tauriApiService.connectPlc(batchId).toPromise();

      if (result && result.success) {
        this.isConnected = true;
        this.message.success('PLC连接成功，量程下发完成');
        console.log('✅ [TEST_AREA] PLC连接+量程下发成功');
      } else {
        // 可能是PLC连接失败，也可能是量程下发失败
        const errMsg = (result && result.message) || 'PLC连接或量程下发失败';
        throw new Error(errMsg);
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

    if (this.hasAnyHardPointTested()) {
      this.message.warning('当前批次已有硬点测试完成，无法再次进行自动测试。请切换到其他批次进行测试。');
      return;
    }

    console.log('🚀 [TEST_AREA] 开始通道自动测试');
    this.isAutoTesting = true;

    try {
      // 初始化测试进度
      this.initializeTestProgress();

      // 🔧 新增：在API调用之前初始化测试计数器，避免时序问题
      // 计算实际需要执行的测试数量（排除已经被标记为跳过的实例）
      const instancesNeedingTest = this.batchDetails?.instances?.filter(inst => 
        inst.overall_status !== OverallTestStatus.Skipped &&
        inst.overall_status !== OverallTestStatus.TestCompletedPassed &&
        inst.overall_status !== OverallTestStatus.TestCompletedFailed
      ) || [];
      const testCountToExecute = instancesNeedingTest.length;
      console.log(`🔧 [TEST_AREA] 批次共有 ${this.batchDetails?.instances?.length || 0} 个实例，其中 ${testCountToExecute} 个需要执行测试`);
      this.initializeTestCounter(testCountToExecute);

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

      // 启动失败时重置状态和计数器
      this.isAutoTesting = false;
      this.isTestCompleted = false;
      this.expectedTestCount = 0;
      this.completedTestCount = 0;
      this.completedTestInstances.clear();
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
    let failedPoints = 0;

    // 统计各种状态的点位数量
    instances.forEach(instance => {
      switch (instance.overall_status) {
        case OverallTestStatus.TestCompletedPassed:
          completedPoints++;
          break;
        case OverallTestStatus.TestCompletedFailed:
          // 仍视为未完成，等待重测
          failedPoints++;
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
      failedPoints,
      isTestCompleted: this.isTestCompleted,
      isAutoTesting: this.isAutoTesting
    });

    // 弹窗显示/隐藏逻辑已迁移至事件监听中的 hardpointTestingSet 管理，这里不再根据 testingPoints 控制弹窗，以避免早关的问题

    // 如果所有点位都已完成测试，且当前状态不是已完成
    if (failedPoints === 0 && completedPoints + failedPoints === totalPoints && testingPoints === 0 && !this.isTestCompleted) {
      console.log('🎉 [TEST_AREA] 检测到测试已完成，更新状态');
      this.isTestCompleted = true;
      this.isAutoTesting = false;
      this.testProgress.currentPoint = undefined;
      // 弹窗关闭由 hardpointTestingSet 逻辑统一处理
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
    // 重置总体进度对象
    this.testProgress = {
      totalPoints: 0,
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
   * 🔧 修复：使用与批次选择区域一致的状态逻辑
   */
  getTestStatusColor(): string {
    if (!this.selectedBatch) {
      return 'default';
    }
    
    // 使用批次选择区域的状态逻辑保持一致
    const status = this.getBatchSelectionStatus(this.selectedBatch);
    
    switch (status.color) {
      case 'success': return 'success';
      case 'error': return 'warning';
      case 'processing': return 'processing';
      default: return 'default';
    }
  }

  /**
   * 获取进度条状态
   */
  getProgressStatus(): 'success' | 'exception' | 'active' | 'normal' {
    const { totalPoints, completedPoints, failedPoints } = this.testProgress;
    const allDone = totalPoints > 0 && completedPoints === totalPoints;

    if (allDone) {
      return failedPoints > 0 ? 'exception' : 'success';
    } else if (completedPoints > 0) {
      return 'active';
    } else {
      return 'normal';
    }
  }

  /**
   * 获取进度条颜色
   */
  getProgressColor(): string {
    switch (this.getProgressStatus()) {
      case 'success':
        return '#52c41a';
      case 'exception':
        return '#ff4d4f';
      case 'active':
        return '#1890ff';
      default:
        return '#d9d9d9';
    }
  }

  /**
   * 获取测试状态文本
   * 🔧 修复：使用与批次选择区域一致的状态逻辑
   */
  getTestStatusText(): string {
    if (!this.selectedBatch) {
      return '等待开始';
    }
    
    // 使用批次选择区域的状态逻辑保持一致
    const status = this.getBatchSelectionStatus(this.selectedBatch);
    
    // 根据状态返回对应的文本
    switch (status.status) {
      case '未开始': return '等待开始';
      case '测试中': return '测试进行中';
      case '已完成': 
        const { failedPoints } = this.testProgress;
        return failedPoints > 0 ? '测试完成(有失败)' : '测试完成(全部通过)';
      default: return '等待开始';
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

  // 🔧 新增：强制清理所有缓存，用于关键状态变化时
  private forceClearAllCaches(): void {
    this._filteredInstances = [];
    this._lastFilterHash = '';
    this._definitionMap.clear();
    
    // 立即重建定义缓存
    this.rebuildDefinitionCache();
    
    // 🔧 新增：强制更新统计信息
    this.updateModuleTypeStats();
    
    // 立即触发变更检测
    this.cdr.detectChanges();
  }

  // 🔧 新增：智能缓存刷新，根据变化类型选择刷新策略
  private smartCacheRefresh(changeType: 'status' | 'error' | 'complete'): void {
    switch (changeType) {
      case 'status':
        // 状态变化：清理过滤缓存，保留定义缓存
        this._filteredInstances = [];
        this._lastFilterHash = '';
        this.updateModuleTypeStats();
        break;
      case 'error':
        // 错误变化：只清理过滤缓存
        this._filteredInstances = [];
        this._lastFilterHash = '';
        break;
      case 'complete':
        // 完成变化：全量刷新
        this.forceClearAllCaches();
        break;
    }
    
    // 立即触发变更检测
    this.cdr.detectChanges();
  }

  getInstanceStatusColor(status: OverallTestStatus): string {
    switch (status) {
      case OverallTestStatus.NotTested: return 'default';
      case OverallTestStatus.WiringConfirmed: return 'blue';
      case OverallTestStatus.HardPointTesting: return 'processing';
      case OverallTestStatus.HardPointTestCompleted: return 'blue';
      case OverallTestStatus.ManualTestInProgress:
      case OverallTestStatus.ManualTesting: return 'processing';
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
      case OverallTestStatus.HardPointTestCompleted: return '硬点测试完成';
      case OverallTestStatus.ManualTestInProgress:
      case OverallTestStatus.ManualTesting: return '手动测试中';
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
      showOnlyPassed: this.showOnlyPassed,
      showOnlyFailed: this.showOnlyFailed,
      showOnlyHardPointPassed: this.showOnlyHardPointPassed,
      showOnlyNotTested: this.showOnlyNotTested,
      instancesLength: this.batchDetails?.instances?.length || 0,
      // 添加实例状态变化的检测
      instancesHash: this.batchDetails?.instances?.map(i => `${i.instance_id}:${i.overall_status}`).join(',') || ''
    });
  }

  // 🔧 性能优化：实际的过滤计算逻辑 - 保持原有逻辑完全一致
  private computeFilteredInstances(): ChannelTestInstance[] {
    if (!this.batchDetails?.instances) return [];

    const filtered = this.batchDetails.instances.filter(instance => {
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

      // 已通过状态筛选
      if (this.showOnlyPassed) {
        if (instance.overall_status !== OverallTestStatus.TestCompletedPassed) {
          return false;
        }
      }

      // 硬点测试通过筛选
      if (this.showOnlyHardPointPassed) {
        if (instance.overall_status !== OverallTestStatus.HardPointTestCompleted) {
          return false;
        }
      }

      // 未测试状态筛选
      if (this.showOnlyNotTested) {
        if (instance.overall_status !== OverallTestStatus.NotTested) {
          return false;
        }
      }

      return true;
    });

    // 按序号排序：有序号的在前，按序号升序；无序号的在后，按tag排序
    return filtered.sort((a, b) => {
      const defA = this.getDefinitionByInstanceIdCached(a.instance_id);
      const defB = this.getDefinitionByInstanceIdCached(b.instance_id);
      
      const seqA = defA?.sequenceNumber;
      const seqB = defB?.sequenceNumber;
      
      // 如果都有序号，按序号排序
      if (seqA !== undefined && seqB !== undefined) {
        return seqA - seqB;
      }
      
      // 有序号的排在无序号的前面
      if (seqA !== undefined && seqB === undefined) {
        return -1;
      }
      if (seqA === undefined && seqB !== undefined) {
        return 1;
      }
      
      // 都没有序号时，按tag排序
      const tagA = defA?.tag || '';
      const tagB = defB?.tag || '';
      return tagA.localeCompare(tagB);
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
    this.showOnlyPassed = false;
    this.showOnlyFailed = false;
    this.showOnlyHardPointPassed = false;
    this.showOnlyNotTested = false;
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
      let date: Date;
      // 如果字符串不包含时区信息，按北京时间(+08:00)解析
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
      case OverallTestStatus.HardPointTestCompleted: return 'blue';
      case OverallTestStatus.ManualTestInProgress:
      case OverallTestStatus.ManualTesting: return 'processing';
      case OverallTestStatus.AlarmTesting: return 'warning';
      case OverallTestStatus.TestCompletedPassed: return 'green';
      case OverallTestStatus.TestCompletedFailed: return 'red';
      default: return 'default';
    }
  }

  readonly OverallTestStatus = OverallTestStatus;
  readonly ModuleType = ModuleType;

  /**
   * 判断当前批次是否存在硬点测试失败的点位
   */
  hasFailedHardPoints(): boolean {
    if (this.batchDetails) {
      return this.batchDetails.instances.some(inst => inst.overall_status === OverallTestStatus.TestCompletedFailed);
    }
    // 如果没有详情，回退到批次摘要信息
    return (this.selectedBatch?.failed_points || 0) > 0;
  }

  /**
   * 判断当前批次是否有任何硬点测试已完成（不论成功失败）
   * 用于控制自动测试按钮的可用性
   */
  hasAnyHardPointTested(): boolean {
    if (this.batchDetails) {
      return this.batchDetails.instances.some(inst => 
        inst.overall_status === OverallTestStatus.HardPointTestCompleted ||
        inst.overall_status === OverallTestStatus.TestCompletedPassed ||
        inst.overall_status === OverallTestStatus.TestCompletedFailed
      );
    }
    // 如果没有详情，回退到批次摘要信息
    return (this.selectedBatch?.tested_points || 0) > 0;
  }

  /**
   * 硬点测试弹窗控制
   */
  private showHardPointModal(): void {
    if (this.hardPointModalRef) return;
    this.hardPointModalRef = this.modal.create({
      nzTitle: '硬点通道自动测试',
      nzContent: '正在进行硬点通道测试，请稍候……',
      nzClosable: false,
      nzMaskClosable: false,
      nzFooter: null
    });
  }

  private closeHardPointModal(): void {
    if (this.hardPointModalRef) {
      this.hardPointModalRef.destroy();
      this.hardPointModalRef = undefined;
    }
  }

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
      testedPoints: batch.tested_points || 0,
      pendingPoints: (batch.total_points || 0) - (batch.tested_points || 0),
      successPoints: batch.passed_points || 0,
      failedPoints: batch.failed_points || 0,
      skippedPoints: batch.skipped_points || 0,
      startedPoints: batch.started_points || 0 // 使用批次中保存的已开始测试点位数
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
        skippedPoints: 0,
        startedPoints: 0
      };
    }

    const instances = this.batchDetails.instances;
    const totalPoints = instances.length;

    let successPoints = 0;
    let failedPoints = 0;
    let skippedPoints = 0;

    instances.forEach(instance => {
      switch (instance.overall_status) {
        case OverallTestStatus.TestCompletedPassed:
          successPoints++;
          break;
        case OverallTestStatus.TestCompletedFailed:
          failedPoints++;
          break;
        case OverallTestStatus.Skipped:
          skippedPoints++;
          break;
        default:
          // 其它状态（未测试、测试中等）不计入已测，待测将在后续由总数计算得出
          break;
      }
    });

    const testedPoints = successPoints + failedPoints + skippedPoints; // 跳过的也计入已测（因为它们不需要执行）
    const pendingPoints = totalPoints - testedPoints;

    // 🔧 新增：计算已开始测试的点位数（用于批次状态显示）
    let startedPoints = 0;
    instances.forEach(instance => {
      const status = instance.overall_status;
      // 统计已开始测试的点位（包括中间状态）
      if (status !== OverallTestStatus.NotTested &&
          status !== OverallTestStatus.WiringConfirmationRequired) {
        startedPoints++;
      }
    });

    // 🔧 移除：不再在统计计算中更新批次状态，避免与批次选择区域状态冲突
    // 批次选择区域现在使用独立的 getBatchSelectionStatus() 方法

    return {
      totalPoints,
      pendingPoints,
      testedPoints,
      successPoints,
      failedPoints,
      skippedPoints,
      startedPoints // 新增字段
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
    // 正在测试中时禁用按钮
    if (instance.overall_status === OverallTestStatus.HardPointTesting ||
        instance.overall_status === OverallTestStatus.ManualTesting ||
        instance.overall_status === OverallTestStatus.ManualTestInProgress) {
      return true;
    }

    // 整体测试通过时禁用按钮
    if (instance.overall_status === OverallTestStatus.TestCompletedPassed) {
      return true;
    }

    // 硬点测试完成且未失败时禁用按钮
    if (instance.overall_status === OverallTestStatus.HardPointTestCompleted) {
      return true;
    }

    // 如果整体状态是失败，只有硬点测试失败时才启用硬点重测按钮
    if (instance.overall_status === OverallTestStatus.TestCompletedFailed) {
      return !this.isHardPointTestFailed(instance);
    }

    // 其他情况启用按钮（未测试、接线确认等）
    return false;
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

      // 🔧 新增：为单个通道测试初始化计数器
      this.initializeTestCounter(1);

      // 调用后端API开始单个通道测试
      await firstValueFrom(this.tauriApiService.startSingleChannelTest(instance.instance_id));

      console.log('✅ [TEST_AREA] 单个通道硬点测试已启动:', instance.instance_id);

      // 显示成功消息
      this.message.success('硬点重测已启动');

      // 弹窗的显示依赖后端发送 HardPointTesting 状态事件
      // 弹窗的关闭依赖 test-completed 事件计数

    } catch (error) {
      console.error('❌ [TEST_AREA] 启动单个通道硬点测试失败:', error);
      this.message.error(`启动硬点测试失败: ${error}`);
      
      // API调用失败时重置计数器
      this.expectedTestCount = 0;
      this.completedTestCount = 0;
    }
  }

  /**
   * 获取表格行的CSS类名（用于整行颜色变更）
   */
  getRowClassName = (data: ChannelTestInstance, index: number): string => {
    // 1) 硬点测试失败 / 最终失败 → 红色
    if (data.overall_status === OverallTestStatus.TestCompletedFailed) {
      return 'row-failed';
    }

    // 2) 仅硬点通过（淡蓝色）
    if (data.overall_status === OverallTestStatus.HardPointTestCompleted) {
      return 'row-hardpoint-passed';
    }

    // 3) 测试全部通过 → 淡绿色（不再校验子项，后端已确保条件）
    if (data.overall_status === OverallTestStatus.TestCompletedPassed) {
      return 'row-passed';
    }

    // 4) 未测试或其他状态 → 默认白色
    return '';
  }

  /**
   * 检查是否为硬点测试失败
   * 通过检查硬点测试的具体状态来判断失败原因
   */
  private isHardPointTestFailed(instance: ChannelTestInstance): boolean {
    // 检查是否存在硬点测试结果且状态为失败
    if (instance.sub_test_results) {
      for (const [subTestItem, result] of Object.entries(instance.sub_test_results)) {
        // 硬点测试相关的子测试项
        if (subTestItem === 'HardPoint' || 
            subTestItem === 'Output0Percent' || 
            subTestItem === 'Output25Percent' || 
            subTestItem === 'Output50Percent' || 
            subTestItem === 'Output75Percent' || 
            subTestItem === 'Output100Percent') {
          if ((result as any).status === 'Failed') {
            return true;
          }
        }
      }
    }
    return false;
  }

  /**
   * 检查是否为手动测试失败
   * 当整体状态为失败但硬点测试通过时，判断为手动测试失败
   */
  public isManualTestFailed(instance: ChannelTestInstance): boolean {
    // 如果整体状态不是失败，则不是手动测试失败
    if (instance.overall_status !== OverallTestStatus.TestCompletedFailed) {
      return false;
    }
    
    // 如果是硬点测试失败，则不是手动测试失败
    if (this.isHardPointTestFailed(instance)) {
      return false;
    }
    
    // 检查是否存在手动测试项失败
    if (instance.sub_test_results) {
      for (const [subTestItem, result] of Object.entries(instance.sub_test_results)) {
        // 手动测试相关的子测试项
        if (subTestItem === 'LowLowAlarm' || 
            subTestItem === 'LowAlarm' || 
            subTestItem === 'HighAlarm' || 
            subTestItem === 'HighHighAlarm' || 
            subTestItem === 'Maintenance' || 
            subTestItem === 'Trend' || 
            subTestItem === 'Report' ||
            subTestItem === 'StateDisplay') {
          if ((result as any).status === 'Failed') {
            return true;
          }
        }
      }
    }
    
    return false;
  }

  /**
   * 检查手动测试按钮是否启用
   * 新逻辑：硬点测试失败时禁用，手动测试失败时允许重测
   */
  isManualTestEnabled(instance: ChannelTestInstance): boolean {
    // 情况1：硬点测试完成，允许手动测试
    if (instance.overall_status === OverallTestStatus.HardPointTestCompleted ||
        instance.overall_status === OverallTestStatus.TestCompletedPassed ||
        instance.overall_status === OverallTestStatus.ManualTesting) {
      return true;
    }
    
    // 情况2：测试失败时，区分失败类型
    if (instance.overall_status === OverallTestStatus.TestCompletedFailed) {
      // 如果是硬点测试失败，禁用手动测试
      if (this.isHardPointTestFailed(instance)) {
        return false;
      }
      
      // 如果是手动测试失败，允许重新打开手动测试
      if (this.isManualTestFailed(instance)) {
        return true;
      }
    }
    
    // 其他情况禁用
    return false;
  }

  /**
   * 获取手动测试按钮文本
   * 根据测试状态显示不同的按钮文本
   */
  getManualTestButtonText(instance: ChannelTestInstance): string {
    // 如果正在手动测试中
    if (instance.overall_status === OverallTestStatus.ManualTesting) {
      return '测试中...';
    }
    
    // 如果是手动测试失败，显示重测
    if (this.isManualTestFailed(instance)) {
      return '重新测试';
    }
    
    // 如果是硬点测试失败，显示禁用状态
    if (this.isHardPointTestFailed(instance)) {
      return '硬点测试失败';
    }
    
    // 如果已测试通过
    if (instance.overall_status === OverallTestStatus.TestCompletedPassed) {
      return '重新测试';
    }
    
    // 默认情况
    return '上位机测试';
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

    // 🔧 新增：强制清理缓存确保状态更新立即显示
    this.smartCacheRefresh('complete');
    
    // 🔧 优化：立即刷新批次详情以获取最新状态
    this.scheduleDataRefresh('manual-test-completed', this.INSTANT_UPDATE_INTERVAL);
  }

  /**
   * 关闭手动测试模态框
   */
  closeManualTestModal(): void {
    this.manualTestModalVisible = false;
    this.selectedManualTestInstance = null;
    this.selectedManualTestDefinition = null;
  }

  /**
   * 检查是否显示错误备注按钮
   * 当通道整体状态为失败时显示错误备注按钮
   */
  showErrorNotesButton(instance: ChannelTestInstance): boolean {
    return instance.overall_status === OverallTestStatus.TestCompletedFailed;
  }

  /**
   * 打开错误备注模态框
   */
  openErrorNotesModal(instance: ChannelTestInstance): void {
    try {
      console.log('📝 [TEST_AREA] 打开错误备注模态框:', instance.instance_id);

      // 获取通道定义信息
      const definition = this.getDefinitionByInstanceId(instance.instance_id);
      if (!definition) {
        this.message.error('无法找到通道定义信息');
        return;
      }

      // 设置选中的实例和定义
      this.selectedErrorNotesInstance = instance;
      this.selectedErrorNotesDefinition = definition;

      // 打开错误备注模态框
      this.errorNotesModalVisible = true;

      console.log('✅ [TEST_AREA] 错误备注模态框已打开');

    } catch (error) {
      console.error('❌ [TEST_AREA] 打开错误备注模态框失败:', error);
      this.message.error(`打开错误备注模态框失败: ${error}`);
    }
  }

  /**
   * 错误备注保存完成处理
   */
  onErrorNotesSaved(): void {
    console.log('💾 [TEST_AREA] 错误备注保存完成');
    this.closeErrorNotesModal();
    
    // 强制清理缓存确保状态更新立即显示
    this.smartCacheRefresh('complete');
    
    // 立即刷新批次详情以获取最新状态
    this.scheduleDataRefresh('error-notes-saved', this.INSTANT_UPDATE_INTERVAL);
  }

  /**
   * 关闭错误备注模态框
   */
  closeErrorNotesModal(): void {
    this.errorNotesModalVisible = false;
    this.selectedErrorNotesInstance = null;
    this.selectedErrorNotesDefinition = null;
  }

  /**
   * 重新测试当前批次硬点测试失败的点位
   */
  async retestFailedHardPoints(): Promise<void> {
    if (!this.selectedBatch) {
      this.message.warning('请先选择一个测试批次');
      return;
    }
    if (!this.isConnected) {
      this.message.warning('请先确认接线并连接PLC');
      return;
    }
    if (!this.hasFailedHardPoints()) {
      this.message.info('当前批次没有硬点失败，无需重测');
      return;
    }

    this.isRetestingFailed = true;

    try {
      // 收集失败的硬点实例
      if (!this.batchDetails) {
        await this.loadBatchDetails();
      }
      const failedInstances = (this.batchDetails?.instances || []).filter(inst => inst.overall_status === OverallTestStatus.TestCompletedFailed);
      
      if (failedInstances.length === 0) {
        this.message.info('当前批次没有硬点失败，无需重测');
        return;
      }

      console.log('🔄 [TEST_AREA] 开始批量重测失败点位，共', failedInstances.length, '个');

      // 🔧 新增：为批量重测初始化计数器
      this.initializeTestCounter(failedInstances.length);

      // 并行启动所有失败实例重测
      let successCount = 0;
      const startPromises = failedInstances.map(async (inst) => {
        try {
          await firstValueFrom(this.tauriApiService.startSingleChannelTest(inst.instance_id));
          console.log('✅ [TEST_AREA] 失败点位重测启动成功:', inst.instance_id);
          successCount++;
        } catch (error) {
          console.error('❌ [TEST_AREA] 启动单通道重测失败:', inst.instance_id, error);
        }
      });

      // 等待所有启动操作完成
      await Promise.allSettled(startPromises);

      if (successCount === 0) {
        this.message.error('所有失败点位重测启动都失败');
        // 如果没有成功启动的测试，重置计数器
        this.expectedTestCount = 0;
        this.completedTestCount = 0;
      } else {
        this.message.success(`已启动 ${successCount} 个失败点位重测`);
        // 🔧 修正：根据实际成功启动的数量调整预期计数
        this.expectedTestCount = successCount;
      }

      // 弹窗的显示依赖后端发送 HardPointTesting 状态事件
      // 弹窗的关闭依赖 test-completed 事件计数

      // 启动后刷新数据
      this.scheduleDataRefresh('failed-retest-started', 800);
    } catch (error) {
      console.error('❌ [TEST_AREA] 启动失败点位重测失败:', error);
      this.message.error('启动失败点位重测失败: ' + error);
    } finally {
      this.isRetestingFailed = false;
    }
  }

  // ======================= 导出通道分配 =========================
  async exportChannelAllocation(): Promise<void> {
    if (!this.selectedBatch) {
      this.message.warning('请先选择批次');
      return;
    }
    console.log('📤 [TEST_AREA] 用户点击导出通道分配表按钮');
    // 弹出文件保存对话框
    const selectedPath = await this.openSaveDialog();
    console.log('📤 [TEST_AREA] 用户选择的导出路径:', selectedPath);

    // 用户取消或未输入文件名都直接返回
    if (!selectedPath || selectedPath.trim().length === 0) {
      return;
    }

    const msgRef = this.message.loading('正在导出通道分配表...', { nzDuration: 0 });
    try {
      const filePath = await firstValueFrom(this.tauriApiService.exportChannelAllocation(selectedPath));
      msgRef.messageId && this.message.remove(msgRef.messageId);
      this.message.success('导出成功: ' + filePath, { nzDuration: 3000 });
    } catch (error) {
      msgRef.messageId && this.message.remove(msgRef.messageId);
      console.error('导出失败', error);
      this.message.error('导出失败，请查看日志');
    }
  }

  async openSaveDialog(): Promise<string | null> {
    console.log('📤 [TEST_AREA] 打开保存对话框');
    const defaultName = `${this.selectedBatch?.station_name || 'station'}_${new Date().toISOString().slice(0,16).replace(/[:T]/g,'')}_通道分配表.xlsx`;
    return await saveDialog({
      title: '请选择导出位置',
      defaultPath: defaultName,
      filters: [
        { name: 'Excel', extensions: ['xlsx'] }
      ]
    });
  }

  /**
   * 🔧 新增：专门用于批次选择区域的状态判断
   * 
   * 与通道详情区域的进度统计分离，解决状态冲突问题
   * 
   * 判断逻辑：
   * - 只要有任何点位开始过硬点测试，就显示"测试中"
   * - 所有点位完成测试后，显示"已完成"
   * - 从未开始测试，显示"未开始"
   */
  getBatchSelectionStatus(batch: TestBatchInfo): { status: string; color: string } {
    // 如果不是当前选中的批次，使用基础统计
    if (!this.selectedBatch || this.selectedBatch.batch_id !== batch.batch_id || !this.batchDetails?.instances) {
      const testedPoints = batch.tested_points || 0; // 完全完成测试的点位
      const startedPoints = batch.started_points || 0; // 已开始测试的点位（包括中间状态）
      const totalPoints = batch.total_points || 0;
      
      // 🔧 修复：使用 startedPoints 判断是否开始测试，解决硬点测试完成后切换批次状态变为"未测试"的问题
      if (startedPoints === 0) {
        return { status: '未开始', color: 'default' };
      } else if (testedPoints < totalPoints) {
        return { status: '测试中', color: 'processing' };
      } else {
        const failedPoints = batch.failed_points || 0;
        return { 
          status: '已完成', 
          color: failedPoints === 0 ? 'success' : 'error' 
        };
      }
    }

    // 当前选中批次，使用详细状态判断
    const instances = this.batchDetails.instances;
    const totalPoints = instances.length;
    
    // 统计不同状态的点位数量
    let completedPoints = 0;      // 完全完成测试的点位
    let startedTestingPoints = 0; // 开始过测试的点位（包括正在测试和已完成）
    let failedPoints = 0;         // 失败的点位
    
    instances.forEach(instance => {
      const status = instance.overall_status;
      
      // 统计完全完成的点位
      if (status === OverallTestStatus.TestCompletedPassed || 
          status === OverallTestStatus.TestCompletedFailed ||
          status === OverallTestStatus.Skipped) {
        completedPoints++;
      }
      
      // 统计失败点位
      if (status === OverallTestStatus.TestCompletedFailed) {
        failedPoints++;
      }
      
      // 统计开始过测试的点位（关键：包括正在测试的状态）
      if (status !== OverallTestStatus.NotTested &&
          status !== OverallTestStatus.WiringConfirmationRequired) {
        startedTestingPoints++;
      }
    });

    // 批次选择区域的状态判断逻辑
    if (startedTestingPoints === 0) {
      // 没有任何点位开始测试
      return { status: '未开始', color: 'default' };
    } else if (completedPoints < totalPoints) {
      // 有点位开始测试但还未全部完成
      return { status: '测试中', color: 'processing' };
    } else {
      // 所有点位完成测试
      return { 
        status: '已完成', 
        color: failedPoints === 0 ? 'success' : 'error' 
      };
    }
  }


}