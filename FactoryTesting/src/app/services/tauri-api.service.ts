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
  AppSettings,
  ParseExcelResponse,
  CreateBatchRequest,
  CreateBatchResponse,
  PrepareTestInstancesRequest,
  PrepareTestInstancesResponse,
  BatchDetailsPayload
} from '../models';

@Injectable({
  providedIn: 'root'
})
export class TauriApiService {
  private systemStatusSubject = new BehaviorSubject<SystemStatus | null>(null);
  public systemStatus$ = this.systemStatusSubject.asObservable();
  
  // 缓存Tauri环境检测结果，避免重复检测
  private _isTauriEnvironment: boolean | null = null;
  private _environmentChecked = false;

  constructor() {
    // 重置环境检测缓存，确保每次启动都重新检测
    this._environmentChecked = false;
    this._isTauriEnvironment = null;
    
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
   * 获取当前会话的所有批次信息
   */
  getSessionBatches(): Observable<TestBatchInfo[]> {
    return from(invoke<TestBatchInfo[]>('get_session_batches'));
  }

  /**
   * 清理完成的批次
   */
  cleanupCompletedBatch(batchId: string): Observable<void> {
    return from(invoke<void>('cleanup_completed_batch', { batchId }));
  }

  /**
   * 创建测试数据 - 用于调试批次分配功能
   */
  createTestData(): Observable<ChannelPointDefinition[]> {
    return from(invoke<ChannelPointDefinition[]>('create_test_data'));
  }

  // ============================================================================
  // 数据管理相关命令
  // ============================================================================

  /**
   * 导入Excel文件并解析通道定义
   */
  importExcelFile(filePath: string): Observable<ChannelPointDefinition[]> {
    return from(invoke<ChannelPointDefinition[]>('import_excel_file', { file_path: filePath }));
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
    return from(invoke<ParseExcelResponse>('parse_excel_file', { file_path: filePath }));
  }

  /**
   * 创建测试批次
   */
  createTestBatch(batchData: CreateBatchRequest): Observable<CreateBatchResponse> {
    return from(invoke<CreateBatchResponse>('create_test_batch', { batch_data: batchData }));
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
    return from(invoke<ChannelPointDefinition[]>('get_batch_channel_definitions', { batch_id: batchId }));
  }

  /**
   * 检查是否在Tauri环境中运行
   */
  isTauriEnvironment(): boolean {
    // 如果已经检测过，直接返回缓存结果
    if (this._environmentChecked) {
      return this._isTauriEnvironment!;
    }

    // 检查是否在浏览器环境中
    if (typeof window === 'undefined') {
      console.log('Tauri环境检测: 不在浏览器环境中');
      this._isTauriEnvironment = false;
      this._environmentChecked = true;
      return false;
    }

    // 多重检测逻辑
    // 1. 检查__TAURI__对象是否存在
    const hasTauri = !!(window as any).__TAURI__;
    
    // 2. 检查是否为tauri协议
    const isTauriProtocol = window.location.protocol === 'tauri:';
    
    // 3. 检查是否为file协议（Tauri应用在某些情况下使用file协议）
    const isFileProtocol = window.location.protocol === 'file:';
    
    // 4. 检查用户代理字符串是否包含Tauri标识
    const userAgent = navigator.userAgent || '';
    const hasTauriUserAgent = userAgent.includes('Tauri') || userAgent.includes('tauri');
    
    // 5. 检查是否存在Tauri特有的全局对象
    const hasTauriGlobals = !!(window as any).__TAURI_INTERNALS__ || !!(window as any).__TAURI_METADATA__;
    
    // 6. 检查窗口对象的特殊属性
    const hasWindowTauri = !!(window as any).window && !!(window as any).window.__TAURI__;
    
    // 7. 尝试检测Tauri的invoke函数
    let hasInvokeFunction = false;
    try {
      hasInvokeFunction = typeof invoke === 'function';
    } catch (e) {
      hasInvokeFunction = false;
    }
    
    // 8. 检查是否在localhost但不是标准的开发端口
    const isLocalhost = window.location.hostname === 'localhost' || window.location.hostname === '127.0.0.1';
    const isStandardDevPort = window.location.port === '4200' || window.location.port === '3000' || window.location.port === '8080';
    const isPossibleTauriPort = isLocalhost && !isStandardDevPort;
    
    // 输出详细检测信息
    console.log('=== Tauri环境检测详情 ===');
    console.log('  当前URL:', window.location.href);
    console.log('  协议:', window.location.protocol);
    console.log('  主机:', window.location.hostname);
    console.log('  端口:', window.location.port);
    console.log('  __TAURI__对象存在:', hasTauri);
    console.log('  __TAURI_INTERNALS__存在:', !!(window as any).__TAURI_INTERNALS__);
    console.log('  __TAURI_METADATA__存在:', !!(window as any).__TAURI_METADATA__);
    console.log('  window.__TAURI__存在:', hasWindowTauri);
    console.log('  invoke函数可用:', hasInvokeFunction);
    console.log('  是否Tauri协议:', isTauriProtocol);
    console.log('  是否文件协议:', isFileProtocol);
    console.log('  用户代理:', userAgent);
    console.log('  用户代理包含Tauri:', hasTauriUserAgent);
    console.log('  Tauri全局对象存在:', hasTauriGlobals);
    console.log('  可能的Tauri端口:', isPossibleTauriPort);
    
    // 如果满足以下任一条件，认为是Tauri环境：
    // 1. __TAURI__对象存在
    // 2. 使用tauri协议
    // 3. 使用file协议且用户代理包含Tauri标识
    // 4. 存在Tauri特有的全局对象
    // 5. invoke函数可用
    // 6. 在localhost的非标准开发端口（可能是Tauri应用）
    const result = hasTauri || isTauriProtocol || (isFileProtocol && hasTauriUserAgent) || 
                   hasTauriGlobals || hasInvokeFunction || hasWindowTauri || 
                   (isPossibleTauriPort && (hasTauriUserAgent || hasTauriGlobals));
    
    console.log('  最终检测结果:', result);
    console.log('========================');
    
    // 如果检测失败，尝试延迟检测
    if (!result && isLocalhost) {
      console.log('首次检测失败，将在500ms后重新检测...');
      setTimeout(() => {
        this._environmentChecked = false;
        this._isTauriEnvironment = null;
        const retryResult = this.isTauriEnvironment();
        console.log('延迟检测结果:', retryResult);
      }, 500);
    }
    
    // 缓存结果
    this._isTauriEnvironment = result;
    this._environmentChecked = true;
    
    return result;
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
  // 应用配置相关命令
  // ============================================================================

  /**
   * 加载应用配置
   */
  loadAppSettings(): Observable<AppSettings> {
    return from(invoke<AppSettings>('load_app_settings_cmd')).pipe(
      catchError(error => {
        console.error('加载应用配置失败:', error);
        // 返回默认配置
        const defaultSettings: AppSettings = {
          id: 'default_settings',
          theme: 'light',
          plc_ip_address: '127.0.0.1',
          plc_port: 502,
          default_operator_name: undefined,
          auto_save_interval_minutes: 5,
          recent_projects: [],
          last_backup_time: undefined
        };
        return from([defaultSettings]);
      })
    );
  }

  /**
   * 保存应用配置
   */
  saveAppSettings(settings: AppSettings): Observable<void> {
    return from(invoke<void>('save_app_settings_cmd', { settings })).pipe(
      catchError(error => {
        console.error('保存应用配置失败:', error);
        throw error;
      })
    );
  }

  // ============================================================================
  // 报告生成相关方法
  // ============================================================================

  generatePdfReport(request: any): Observable<any> {
    return from(invoke('generate_pdf_report', { request }));
  }

  generateExcelReport(request: any): Observable<any> {
    return from(invoke('generate_excel_report', { request }));
  }

  deleteReport(reportId: string): Observable<any> {
    return from(invoke('delete_report', { reportId }));
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

  /**
   * 自动分配批次 - 根据导入的通道定义自动创建测试批次和实例
   */
  autoAllocateBatch(batchData: any): Observable<any> {
    return from(invoke('import_excel_and_prepare_batch_cmd', { 
      args: {
        file_path_str: batchData.filePath,
        product_model: batchData.productModel,
        serial_number: batchData.serialNumber
      }
    }));
  }

  /**
   * 解析Excel文件但不持久化数据
   */
  parseExcelWithoutPersistence(filePath: string, fileName: string): Observable<any> {
    return from(invoke('parse_excel_without_persistence_cmd', { 
      args: {
        file_path: filePath, 
        file_name: fileName 
      }
    }));
  }

  /**
   * 创建批次并持久化数据（在开始测试时调用）
   */
  createBatchAndPersistData(batchInfo: any, definitions: any[]): Observable<any> {
    return from(invoke('create_batch_and_persist_data_cmd', { 
      request: {
        batch_info: batchInfo,
        definitions: definitions
      }
    }));
  }

  /**
   * 解析Excel文件并自动创建批次
   */
  parseExcelAndCreateBatch(filePath: string, fileName: string): Observable<any> {
    return from(invoke('parse_excel_and_create_batch_cmd', { 
      args: {
        file_path: filePath, 
        file_name: fileName 
      }
    }));
  }

  /**
   * 准备批次测试实例 - 自动分配逻辑
   */
  prepareTestInstancesForBatch(request: PrepareTestInstancesRequest): Observable<PrepareTestInstancesResponse> {
    return from(invoke<PrepareTestInstancesResponse>('prepare_test_instances_for_batch_cmd', {
      args: {
        batch_id: request.batch_id,
        definition_ids: request.definition_ids
      }
    }));
  }

  /**
   * 自动分配测试实例 - 为选中的批次分配测试实例
   */
  allocateTestInstances(batchId: string): Observable<PrepareTestInstancesResponse> {
    return this.prepareTestInstancesForBatch({ batch_id: batchId });
  }

  /**
   * 获取批次详情
   */
  getBatchDetails(batchId: string): Observable<BatchDetailsPayload> {
    return from(invoke<BatchDetailsPayload>('get_batch_status_cmd', { 
      batch_id: batchId 
    }));
  }

  /**
   * 获取通道映射配置
   */
  getChannelMappings(): Observable<any[]> {
    return from(invoke<any[]>('get_channel_mappings_cmd'));
  }

  /**
   * 强制重新检测Tauri环境
   */
  forceRedetectEnvironment(): boolean {
    this._environmentChecked = false;
    this._isTauriEnvironment = null;
    return this.isTauriEnvironment();
  }

  /**
   * 导入Excel文件并自动分配通道
   */
  importExcelAndAllocateChannels(
    filePath: string, 
    productModel?: string, 
    serialNumber?: string
  ): Observable<any> {
    return from(invoke('import_excel_and_allocate_channels_cmd', { 
      file_path: filePath, 
      product_model: productModel, 
      serial_number: serialNumber 
    }));
  }

  /**
   * 清理当前会话数据
   */
  clearSessionData(): Observable<string> {
    return from(invoke<string>('clear_session_data'));
  }
} 