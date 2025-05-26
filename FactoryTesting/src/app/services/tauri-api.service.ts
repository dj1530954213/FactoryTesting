import { Injectable } from '@angular/core';
import { Observable, from, BehaviorSubject, interval } from 'rxjs';
import { map, catchError, switchMap } from 'rxjs/operators';
import { invoke } from '@tauri-apps/api/core';
import {
  ChannelPointDefinition,
  TestBatchInfo,
  ChannelTestInstance,
  RawTestOutcome,
  TestExecutionRequest,
  TestExecutionResponse,
  TestProgressUpdate,
  SystemStatus,
  ParseExcelResponse,
  CreateBatchRequest,
  CreateBatchResponse
} from '../models';

@Injectable({
  providedIn: 'root'
})
export class TauriApiService {
  private systemStatusSubject = new BehaviorSubject<SystemStatus | null>(null);
  public systemStatus$ = this.systemStatusSubject.asObservable();

  constructor() {
    // 启动系统状态实时轮询（每5秒更新一次）
    this.startSystemStatusPolling();
  }

  // ============================================================================
  // 测试协调相关命令
  // ============================================================================

  /**
   * 提交测试执行请求
   */
  submitTestExecution(request: TestExecutionRequest): Observable<TestExecutionResponse> {
    return from(invoke<TestExecutionResponse>('submit_test_execution', { request }));
  }

  /**
   * 开始批次测试
   */
  startBatchTesting(batchId: string): Observable<void> {
    return from(invoke<void>('start_batch_testing', { batchId }));
  }

  /**
   * 暂停批次测试
   */
  pauseBatchTesting(batchId: string): Observable<void> {
    return from(invoke<void>('pause_batch_testing', { batchId }));
  }

  /**
   * 恢复批次测试
   */
  resumeBatchTesting(batchId: string): Observable<void> {
    return from(invoke<void>('resume_batch_testing', { batchId }));
  }

  /**
   * 停止批次测试
   */
  stopBatchTesting(batchId: string): Observable<void> {
    return from(invoke<void>('stop_batch_testing', { batchId }));
  }

  /**
   * 获取批次测试进度
   */
  getBatchProgress(batchId: string): Observable<TestProgressUpdate[]> {
    return from(invoke<TestProgressUpdate[]>('get_batch_progress', { batchId }));
  }

  /**
   * 获取批次测试结果
   */
  getBatchResults(batchId: string): Observable<RawTestOutcome[]> {
    return from(invoke<RawTestOutcome[]>('get_batch_results', { batchId }));
  }

  /**
   * 清理完成的批次
   */
  cleanupCompletedBatch(batchId: string): Observable<void> {
    return from(invoke<void>('cleanup_completed_batch', { batchId }));
  }

  // ============================================================================
  // 数据管理相关命令
  // ============================================================================

  /**
   * 导入Excel文件并解析通道定义
   */
  importExcelFile(filePath: string): Observable<ChannelPointDefinition[]> {
    return from(invoke<ChannelPointDefinition[]>('import_excel_file', { filePath }));
  }

  /**
   * 创建测试批次并保存通道定义
   */
  createTestBatchWithDefinitions(batchInfo: TestBatchInfo, definitions: ChannelPointDefinition[]): Observable<string> {
    return from(invoke<string>('create_test_batch_with_definitions', { batchInfo, definitions }));
  }

  /**
   * 获取所有通道定义
   */
  getAllChannelDefinitions(): Observable<ChannelPointDefinition[]> {
    return from(invoke<ChannelPointDefinition[]>('get_all_channel_definitions'));
  }

  /**
   * 保存通道定义
   */
  saveChannelDefinition(definition: ChannelPointDefinition): Observable<void> {
    return from(invoke<void>('save_channel_definition', { definition }));
  }

  /**
   * 删除通道定义
   */
  deleteChannelDefinition(definitionId: string): Observable<void> {
    return from(invoke<void>('delete_channel_definition', { definitionId }));
  }

  /**
   * 获取所有批次信息
   */
  getAllBatchInfo(): Observable<TestBatchInfo[]> {
    return from(invoke<TestBatchInfo[]>('get_all_batch_info'));
  }

  /**
   * 保存批次信息
   */
  saveBatchInfo(batchInfo: TestBatchInfo): Observable<void> {
    return from(invoke<void>('save_batch_info', { batchInfo }));
  }

  /**
   * 获取批次测试实例
   */
  getBatchTestInstances(batchId: string): Observable<ChannelTestInstance[]> {
    return from(invoke<ChannelTestInstance[]>('get_batch_test_instances', { batchId }));
  }

  // ============================================================================
  // 通道状态管理相关命令
  // ============================================================================

  /**
   * 创建测试实例
   */
  createTestInstance(definitionId: string, batchId: string): Observable<ChannelTestInstance> {
    return from(invoke<ChannelTestInstance>('create_test_instance', { definitionId, batchId }));
  }

  /**
   * 获取实例状态
   */
  getInstanceState(instanceId: string): Observable<ChannelTestInstance> {
    return from(invoke<ChannelTestInstance>('get_instance_state', { instanceId }));
  }

  /**
   * 更新测试结果
   */
  updateTestResult(instanceId: string, outcome: RawTestOutcome): Observable<void> {
    return from(invoke<void>('update_test_result', { instanceId, outcome }));
  }

  // ============================================================================
  // 系统信息相关命令
  // ============================================================================

  /**
   * 获取系统状态
   */
  getSystemStatus(): Observable<SystemStatus> {
    return from(invoke<SystemStatus>('get_system_status'));
  }

  // ============================================================================
  // 文件处理相关命令
  // ============================================================================

  /**
   * 解析Excel文件
   */
  parseExcelFile(filePath: string): Observable<ParseExcelResponse> {
    return from(invoke<ParseExcelResponse>('parse_excel_file', { filePath }));
  }

  /**
   * 创建测试批次
   */
  createTestBatch(batchData: CreateBatchRequest): Observable<CreateBatchResponse> {
    return from(invoke<CreateBatchResponse>('create_test_batch', { batchData }));
  }

  /**
   * 获取批次列表
   */
  getBatchList(): Observable<TestBatchInfo[]> {
    return from(invoke<TestBatchInfo[]>('get_batch_list'));
  }

  /**
   * 获取批次的通道定义列表
   */
  getBatchChannelDefinitions(batchId: string): Observable<ChannelPointDefinition[]> {
    return from(invoke<ChannelPointDefinition[]>('get_batch_channel_definitions', { batchId }));
  }

  /**
   * 检查是否在Tauri环境中运行
   */
  isTauriEnvironment(): boolean {
    const hasTauri = typeof window !== 'undefined' && window.__TAURI__;
    const hasInvoke = typeof window !== 'undefined' && window.__TAURI__ && window.__TAURI__.core && window.__TAURI__.core.invoke;
    
    console.log('Tauri环境检测:');
    console.log('  window存在:', typeof window !== 'undefined');
    console.log('  __TAURI__存在:', typeof window !== 'undefined' && !!window.__TAURI__);
    console.log('  invoke函数存在:', hasInvoke);
    console.log('  最终结果:', hasTauri);
    
    return hasTauri;
  }

  // ============================================================================
  // 便捷方法
  // ============================================================================

  /**
   * 检查系统健康状态
   */
  isSystemHealthy(): Observable<boolean> {
    return this.getSystemStatus().pipe(
      map(status => status.system_health === 'healthy'),
      catchError(() => from([false]))
    );
  }

  /**
   * 获取活动任务数
   */
  getActiveTaskCount(): Observable<number> {
    return this.getSystemStatus().pipe(
      map(status => status.active_test_tasks),
      catchError(() => from([0]))
    );
  }

  /**
   * 获取系统版本
   */
  getSystemVersion(): Observable<string> {
    return this.getSystemStatus().pipe(
      map(status => status.version),
      catchError(() => from(['未知']))
    );
  }

  // ============================================================================
  // 私有方法
  // ============================================================================

  /**
   * 启动系统状态实时轮询
   */
  private startSystemStatusPolling(): void {
    interval(5000) // 每5秒轮询一次
      .pipe(
        switchMap(() => this.getSystemStatus()),
        catchError(error => {
          console.error('系统状态轮询失败:', error);
          return from([null]);
        })
      )
      .subscribe(status => {
        this.systemStatusSubject.next(status);
      });
  }
} 