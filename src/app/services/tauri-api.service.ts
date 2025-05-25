import { Injectable } from '@angular/core';
import { invoke } from '@tauri-apps/api/tauri';
import { Observable, from, BehaviorSubject } from 'rxjs';
import { map, catchError } from 'rxjs/operators';
import {
  ChannelPointDefinition,
  ChannelTestInstance,
  TestBatchInfo,
  RawTestOutcome,
  TestExecutionRequest,
  TestExecutionResponse,
  TestProgressUpdate,
  SystemStatus,
  ApiError
} from '../models';

/**
 * Tauri API 服务
 * 
 * 封装所有与后端 Rust 代码的通信
 * 提供 Observable 接口供 Angular 组件使用
 */
@Injectable({
  providedIn: 'root'
})
export class TauriApiService {
  
  // 系统状态主题
  private systemStatusSubject = new BehaviorSubject<SystemStatus | null>(null);
  public systemStatus$ = this.systemStatusSubject.asObservable();

  constructor() {
    // 定期更新系统状态
    this.startSystemStatusPolling();
  }

  // ============================================================================
  // 测试协调相关方法
  // ============================================================================

  /**
   * 提交测试执行请求
   */
  submitTestExecution(request: TestExecutionRequest): Observable<TestExecutionResponse> {
    return from(invoke<TestExecutionResponse>('submit_test_execution', { request }))
      .pipe(catchError(this.handleError));
  }

  /**
   * 开始批次测试
   */
  startBatchTesting(batchId: string): Observable<void> {
    return from(invoke<void>('start_batch_testing', { batchId }))
      .pipe(catchError(this.handleError));
  }

  /**
   * 暂停批次测试
   */
  pauseBatchTesting(batchId: string): Observable<void> {
    return from(invoke<void>('pause_batch_testing', { batchId }))
      .pipe(catchError(this.handleError));
  }

  /**
   * 恢复批次测试
   */
  resumeBatchTesting(batchId: string): Observable<void> {
    return from(invoke<void>('resume_batch_testing', { batchId }))
      .pipe(catchError(this.handleError));
  }

  /**
   * 停止批次测试
   */
  stopBatchTesting(batchId: string): Observable<void> {
    return from(invoke<void>('stop_batch_testing', { batchId }))
      .pipe(catchError(this.handleError));
  }

  /**
   * 获取批次测试进度
   */
  getBatchProgress(batchId: string): Observable<TestProgressUpdate[]> {
    return from(invoke<TestProgressUpdate[]>('get_batch_progress', { batchId }))
      .pipe(catchError(this.handleError));
  }

  /**
   * 获取批次测试结果
   */
  getBatchResults(batchId: string): Observable<RawTestOutcome[]> {
    return from(invoke<RawTestOutcome[]>('get_batch_results', { batchId }))
      .pipe(catchError(this.handleError));
  }

  /**
   * 清理完成的批次
   */
  cleanupCompletedBatch(batchId: string): Observable<void> {
    return from(invoke<void>('cleanup_completed_batch', { batchId }))
      .pipe(catchError(this.handleError));
  }

  // ============================================================================
  // 数据管理相关方法
  // ============================================================================

  /**
   * 获取所有通道定义
   */
  getAllChannelDefinitions(): Observable<ChannelPointDefinition[]> {
    return from(invoke<ChannelPointDefinition[]>('get_all_channel_definitions'))
      .pipe(catchError(this.handleError));
  }

  /**
   * 保存通道定义
   */
  saveChannelDefinition(definition: ChannelPointDefinition): Observable<void> {
    return from(invoke<void>('save_channel_definition', { definition }))
      .pipe(catchError(this.handleError));
  }

  /**
   * 删除通道定义
   */
  deleteChannelDefinition(definitionId: string): Observable<void> {
    return from(invoke<void>('delete_channel_definition', { definitionId }))
      .pipe(catchError(this.handleError));
  }

  /**
   * 获取所有批次信息
   */
  getAllBatchInfo(): Observable<TestBatchInfo[]> {
    return from(invoke<TestBatchInfo[]>('get_all_batch_info'))
      .pipe(catchError(this.handleError));
  }

  /**
   * 保存批次信息
   */
  saveBatchInfo(batchInfo: TestBatchInfo): Observable<void> {
    return from(invoke<void>('save_batch_info', { batchInfo }))
      .pipe(catchError(this.handleError));
  }

  /**
   * 获取批次的所有测试实例
   */
  getBatchTestInstances(batchId: string): Observable<ChannelTestInstance[]> {
    return from(invoke<ChannelTestInstance[]>('get_batch_test_instances', { batchId }))
      .pipe(catchError(this.handleError));
  }

  // ============================================================================
  // 通道状态管理相关方法
  // ============================================================================

  /**
   * 创建测试实例
   */
  createTestInstance(definitionId: string, batchId: string): Observable<ChannelTestInstance> {
    return from(invoke<ChannelTestInstance>('create_test_instance', { definitionId, batchId }))
      .pipe(catchError(this.handleError));
  }

  /**
   * 获取实例状态
   */
  getInstanceState(instanceId: string): Observable<ChannelTestInstance> {
    return from(invoke<ChannelTestInstance>('get_instance_state', { instanceId }))
      .pipe(catchError(this.handleError));
  }

  /**
   * 更新测试结果
   */
  updateTestResult(instanceId: string, outcome: RawTestOutcome): Observable<void> {
    return from(invoke<void>('update_test_result', { instanceId, outcome }))
      .pipe(catchError(this.handleError));
  }

  // ============================================================================
  // 系统信息相关方法
  // ============================================================================

  /**
   * 获取系统状态
   */
  getSystemStatus(): Observable<SystemStatus> {
    return from(invoke<SystemStatus>('get_system_status'))
      .pipe(
        map(status => {
          this.systemStatusSubject.next(status);
          return status;
        }),
        catchError(this.handleError)
      );
  }

  // ============================================================================
  // 工具方法
  // ============================================================================

  /**
   * 开始系统状态轮询
   */
  private startSystemStatusPolling(): void {
    // 每5秒更新一次系统状态
    setInterval(() => {
      this.getSystemStatus().subscribe({
        next: () => {}, // 状态已在 getSystemStatus 中更新
        error: (error) => console.warn('系统状态更新失败:', error)
      });
    }, 5000);

    // 立即获取一次状态
    this.getSystemStatus().subscribe();
  }

  /**
   * 错误处理
   */
  private handleError = (error: any): Observable<never> => {
    console.error('Tauri API 调用失败:', error);
    
    // 将错误转换为标准格式
    const apiError: ApiError = {
      error: typeof error === 'string' ? error : error.message || '未知错误',
      code: error.code,
      details: error
    };

    throw apiError;
  }

  // ============================================================================
  // 便捷方法
  // ============================================================================

  /**
   * 检查系统是否健康
   */
  isSystemHealthy(): boolean {
    const status = this.systemStatusSubject.value;
    return status?.system_health === 'healthy';
  }

  /**
   * 获取当前活动任务数
   */
  getActiveTaskCount(): number {
    const status = this.systemStatusSubject.value;
    return status?.active_test_tasks || 0;
  }

  /**
   * 获取系统版本
   */
  getSystemVersion(): string {
    const status = this.systemStatusSubject.value;
    return status?.version || 'Unknown';
  }
} 