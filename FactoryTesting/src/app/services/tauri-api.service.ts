/**
 * # Tauri API服务 - TauriApiService
 * 
 * ## 业务功能说明
 * - 前端与Rust后端通信的核心服务
 * - 封装所有Tauri命令调用，提供类型安全的API接口
 * - 管理系统状态的实时轮询和缓存
 * - 处理环境检测和兼容性适配
 * 
 * ## 前后端调用链架构
 * ```
 * Angular组件 → TauriApiService → @tauri-apps/api/core.invoke → Tauri框架 → Rust后端命令 → 业务逻辑处理
 * ```
 * 
 * ## 主要功能模块
 * - **数据管理**: Excel导入、批次创建、通道定义管理
 * - **测试协调**: 批次测试执行、进度监控、结果获取
 * - **PLC通信**: 连接管理、状态监控、数据读写
 * - **系统监控**: 状态轮询、健康检查、环境检测
 * - **文件处理**: Excel解析、报告生成、数据导出
 * 
 * ## Angular知识点
 * - **Injectable装饰器**: 标记为可注入的服务，单例模式
 * - **RxJS操作符**: from、map、catchError、switchMap、tap用于异步操作
 * - **BehaviorSubject**: 状态管理，保存最新值并支持订阅
 * - **Observable模式**: 所有API调用返回Observable，支持响应式编程
 * 
 * ## Tauri集成
 * - **invoke函数**: Tauri提供的前后端通信桥梁
 * - **环境检测**: 多重检测机制判断运行环境
 * - **类型安全**: TypeScript接口确保调用参数和返回值的类型正确性
 * 
 * ## 设计模式
 * - **门面模式**: 为复杂的后端API提供简化接口
 * - **观察者模式**: 状态轮询和事件订阅
 * - **适配器模式**: 环境适配，支持Tauri和浏览器环境
 */

import { Injectable } from '@angular/core';
import { Observable, from, BehaviorSubject, interval } from 'rxjs';
import { map, catchError, switchMap, tap } from 'rxjs/operators';
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
  BatchDetailsPayload,
  ImportExcelAndCreateBatchResponse,
  DashboardBatchInfo,
  DeleteBatchResponse
} from '../models';
import { PlcConnectionStatus } from '../models/plc-connection-status.model';

/**
 * Tauri API服务类
 * 
 * **业务作用**: 前端与Rust后端通信的单一入口点
 * **服务范围**: 全局单例服务，在根模块注入
 * **核心职责**: 
 * - 封装所有后端API调用
 * - 管理系统状态轮询
 * - 处理环境检测和适配
 */
@Injectable({
  providedIn: 'root'
})
export class TauriApiService {
  // === 状态管理 ===
  /** 
   * 系统状态主题 - 使用BehaviorSubject保存最新状态
   * **数据来源**: 后端get_system_status命令
   * **更新机制**: 每5秒轮询一次
   */
  private systemStatusSubject = new BehaviorSubject<SystemStatus | null>(null);
  
  /** 
   * 系统状态Observable - 供组件订阅
   * **使用方式**: 组件通过订阅获取实时状态更新
   */
  public systemStatus$ = this.systemStatusSubject.asObservable();

  // === 环境检测缓存 ===
  /** Tauri环境检测结果缓存，避免重复检测 */
  private _isTauriEnvironment: boolean | null = null;
  
  /** 环境检测是否已完成标志 */
  private _environmentChecked = false;

  /**
   * 构造函数
   * 
   * **初始化流程**:
   * 1. 重置环境检测缓存
   * 2. 启动系统状态轮询
   * 
   * **设计理念**: 服务启动即开始监控，确保状态实时性
   */
  constructor() {
    // 重置环境检测缓存，确保每次启动都重新检测
    this._environmentChecked = false;
    this._isTauriEnvironment = null;

    // 启动系统状态实时轮询（每5秒更新一次）
    this.startSystemStatusPolling();
  }

  // ============================================================================
  // 全局功能测试项相关命令
  // ============================================================================

  /**
   * 获取全局功能测试状态
   * 
   * **业务功能**: 获取指定站场和导入时间的功能测试状态列表
   * **应用场景**: 仪表盘显示各功能模块的测试状态
   * **调用链**: getGlobalFunctionTests → invoke → get_global_function_tests_cmd → 后端状态查询
   * 
   * @param stationName 站场名称
   * @param importTime 导入时间 (YYYY-MM-DDTHH:MM:SS格式)
   * @returns Observable<any[]> 返回功能测试状态数组
   */
  getGlobalFunctionTests(stationName: string, importTime: string) {
    console.log('[API] getGlobalFunctionTests →', stationName, importTime);

    // 构建兼容新旧版本的参数对象
    const payload = {
      stationName: stationName,
      importTime: importTime,
      // 兼容旧版后端参数名（snake_case格式）
      station_name: stationName,
      import_time: importTime
    } as any;
    console.log('[API] invoke get_global_function_tests_cmd payload', payload);
    
    // 调用后端命令获取功能测试状态
    return from(invoke<any[]>('get_global_function_tests_cmd', payload));
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
   * 开始单个通道的硬点测试
   */
  startSingleChannelTest(instanceId: string): Observable<void> {
    return from(invoke<void>('start_single_channel_test', { instanceId }));
  }

  /**
   * 创建测试数据 - 用于调试批次分配功能
   */
  createTestData(): Observable<ChannelPointDefinition[]> {
    return from(invoke<ChannelPointDefinition[]>('create_test_data'));
  }

  /**
   * 连接PLC并下发量程设置
   * 
   * **业务功能**: 
   * 1. 建立与PLC的连接通道
   * 2. 可选地为指定批次下发通道量程配置
   * 
   * **业务场景**: 测试开始前的PLC准备工作
   * **调用链**: connectPlc → connect_plc_cmd → PLC连接管理器 → Modbus连接
   * **后续操作**: 连接成功后可选调用apply_channel_range_setting_cmd下发量程
   * 
   * **技术说明**:
   * - connect_plc_cmd返回 {success, message?}
   * - apply_channel_range_setting_cmd返回Unit（在JS中为null）
   * - 使用switchMap链式调用，确保连接成功后再下发量程
   * 
   * @param batchId 可选的批次ID，提供时会自动下发该批次的量程配置
   * @returns Observable<{success: boolean, message?: string}> PLC连接结果
   */
  connectPlc(batchId?: string): Observable<{ success: boolean; message?: string }> {
    console.log('🔗 [TAURI_API] 调用连接PLC API');
    return from(invoke<{ success: boolean; message?: string }>('connect_plc_cmd')).pipe(
      tap(result => {
        if (result.success) {
          console.log('✅ [TAURI_API] PLC连接成功');
        } else {
          console.error('❌ [TAURI_API] PLC连接失败:', result.message);
        }
      }),
      switchMap(result => {
        // 如果连接失败或未提供批次名，则直接返回结果
        if (!result.success || !batchId) {
          return from([result]);
        }
        // 成功连接且提供批次名，继续设置通道量程
        console.log('📝 [TAURI_API] 开始下发通道量程，批次ID:', batchId);
        // 量程下发命令仅在成功时返回 null/undefined，失败时抛异常
        // 注意：Rust 侧命令参数为 snake_case 的 batch_name
        const paramObj = { batchName: String(batchId) };
        console.log('📤 [TAURI_API] 下发量程参数对象:', paramObj);
        return from(invoke<void>('apply_channel_range_setting_cmd', paramObj)).pipe(
          tap(() => {
            console.log('✅ [TAURI_API] 通道量程下发成功');
          }),
          map(() => ({ success: true as const })),
          catchError(error => {
            console.error('❌ [TAURI_API] 通道量程下发命令调用失败:', error);
            throw error;
          })
        );
      }),
      catchError(error => {
        console.error('❌ [TAURI_API] PLC连接API调用失败:', error);
        throw error;
      })
    );
  }

  /**
   * 开始批次自动测试
   */
  startBatchAutoTest(batchId: string): Observable<{ success: boolean; message?: string }> {
    console.log('🚀 [TAURI_API] 调用开始批次自动测试API');
    console.log('🚀 [TAURI_API] 批次ID:', batchId);
    return from(invoke<{ success: boolean; message?: string }>('start_batch_auto_test_cmd', {
      args: { batch_id: batchId }
    })).pipe(
      tap(result => {
        if (result.success) {
          console.log('✅ [TAURI_API] 批次自动测试启动成功');
        } else {
          console.error('❌ [TAURI_API] 批次自动测试启动失败:', result.message);
        }
      }),
      catchError(error => {
        console.error('❌ [TAURI_API] 批次自动测试API调用失败:', error);
        throw error;
      })
    );
  }

  /**
   * 重新测试批次中硬点测试失败的点位
   */
  retestFailedHardPoints(batchId: string): Observable<{ success: boolean; message?: string }> {
    console.log('🚀 [TAURI_API] 调用重新测试失败硬点API, 批次ID:', batchId);
    return from(invoke<{ success: boolean; message?: string }>('retest_failed_hardpoints_cmd', {
      args: { batch_id: batchId }
    })).pipe(
      tap(result => {
        if (result.success) {
          console.log('✅ [TAURI_API] 重新测试失败硬点启动成功');
        } else {
          console.error('❌ [TAURI_API] 重新测试失败硬点启动失败:', result.message);
        }
      }),
      catchError(error => {
        console.error('❌ [TAURI_API] 重新测试失败硬点API调用失败:', error);
        throw error;
      })
    );
  }

  /**
   * 获取PLC连接状态
   */
  getPlcConnectionStatus(): Observable<PlcConnectionStatus> {
    return from(invoke<PlcConnectionStatus>('get_plc_connection_status_cmd')).pipe(
      catchError(error => {
        console.error('❌ [TAURI_API] 获取PLC连接状态失败:', error);
        throw error;
      })
    );
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
    return from(invoke<string>('create_test_batch_with_definitions_cmd', { batch_info: batchInfo, definitions }));
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
  updateTestResult(outcome: RawTestOutcome): Observable<void> {
    return from(invoke<void>('update_test_result', { outcome }));
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
   * @deprecated 已废弃 - 请使用 autoAllocateBatch 替代
   * 这个方法已经不再使用，批次创建应该在点表导入时自动完成
   */
  createTestBatch(batchData: CreateBatchRequest): Observable<CreateBatchResponse> {
    console.error('❌ [TAURI_API] createTestBatch 已废弃，请使用 autoAllocateBatch 进行完整的导入和批次创建流程');
    throw new Error('createTestBatch 已废弃，请使用 autoAllocateBatch 方法');
  }

  /**
   * 获取批次列表 - 从状态管理器获取已分配的批次（测试区域专用）
   */
  getBatchList(): Observable<TestBatchInfo[]> {
    console.log('📋 [TAURI_API] 调用获取批次列表API - 测试区域专用');
    return from(invoke<TestBatchInfo[]>('get_batch_list')).pipe(
      tap(batches => {
        console.log('✅ [TAURI_API] 成功获取批次列表');
        console.log('✅ [TAURI_API] 批次数量:', batches.length);
        if (batches.length > 0) {
          batches.forEach((batch, index) => {
            console.log(`  批次${index + 1}: ID=${batch.batch_id}, 名称=${batch.batch_name}, 点位数=${batch.total_points}`);
          });
        }
      }),
      catchError(error => {
        console.error('❌ [TAURI_API] 获取批次列表失败:', error);
        throw error;
      })
    );
  }

  /**
   * 获取仪表盘批次列表 - 从数据库获取所有批次并标识当前会话批次
   */
  getDashboardBatchList(): Observable<DashboardBatchInfo[]> {
    return from(invoke<DashboardBatchInfo[]>('get_dashboard_batch_list')).pipe(
      tap(dashboardBatches => {
        // 静默获取数据，不输出日志

        // 🔍 调试站场信息 - 修复：由于后端使用了 #[serde(flatten)]，直接访问字段
        dashboardBatches.forEach((dashboardBatch, index) => {
          if (dashboardBatch.station_name) {
            console.log(`✅ [TAURI_API] 批次${index + 1} 站场信息: ${dashboardBatch.station_name}`);
          } else {
            console.warn(`⚠️ [TAURI_API] 批次${index + 1} 缺少站场信息: ${dashboardBatch.batch_name}`);
          }
        });
      }),
      catchError(error => {
        console.error('❌ [TAURI_API] 获取仪表盘批次列表失败:', error);
        throw error;
      })
    );
  }

  /**
   * 获取批次的通道定义列表
   */
  getBatchChannelDefinitions(batchId: string): Observable<ChannelPointDefinition[]> {
    return from(invoke<ChannelPointDefinition[]>('get_batch_channel_definitions', { batch_id: batchId }));
  }

  /**
   * 检查是否在Tauri环境中运行
   * 
   * **业务功能**: 智能检测应用运行环境，确保API调用的正确性
   * **检测场景**:
   * - Tauri桌面环境：调用真实的后端API
   * - 浏览器环境：返回模拟数据用于开发调试
   * 
   * **检测策略**: 多重检测机制，提高检测准确性
   * 1. __TAURI__全局对象检测
   * 2. tauri://协议检测
   * 3. file://协议 + User-Agent检测
   * 4. Tauri内部对象检测
   * 5. invoke函数可用性检测
   * 6. 网络端口分析
   * 
   * **缓存机制**: 首次检测后缓存结果，避免重复检测开销
   * **容错处理**: 检测失败时支持延迟重试
   * 
   * @returns boolean true表示Tauri环境，false表示浏览器环境
   */
  isTauriEnvironment(): boolean {
    // 如果已经检测过，直接返回缓存结果，提高性能
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

  /**
   * 强制重新检测Tauri环境
   */
  forceRedetectEnvironment(): boolean {
    this._environmentChecked = false;
    this._isTauriEnvironment = null;
    return this.isTauriEnvironment();
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
  // 全局功能测试状态相关方法
  // ============================================================================



  /** 更新某个功能测试状态 */
  updateGlobalFunctionTest(request: {
    station_name: string;
    import_time: string;
    function_key: string;
    status: string;
    start_time?: string;
    end_time?: string;
  }): Observable<any> {
    return from(invoke('update_global_function_test_cmd', { request }));
  }

  /** 重置某站场的全部功能测试状态 */
  resetGlobalFunctionTests(stationName: string, importTime: string): Observable<any> {
    return from(invoke('reset_global_function_tests_cmd', { station_name: stationName, import_time: importTime }));
  }

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
   * 自动分配批次 - 点表导入和批次创建的核心方法
   * 
   * **业务功能**: 
   * 1. 解析Excel点表文件
   * 2. 自动创建测试批次
   * 3. 分配测试实例
   * 4. 存储到状态管理器
   * 
   * **业务场景**: 用户导入Excel点表后的自动化处理流程
   * **调用链**: autoAllocateBatch → import_excel_and_prepare_batch_cmd → Excel解析器 → 批次管理器 → 数据库存储
   * 
   * **处理步骤**:
   * 1. 解析Excel文件获取通道定义
   * 2. 根据产品型号和序列号创建批次
   * 3. 为每个通道定义创建测试实例
   * 4. 存储批次信息到内存状态管理器
   * 
   * **参数说明**:
   * - filePath: Excel文件路径
   * - productModel: 产品型号
   * - serialNumber: 产品序列号
   * 
   * @param batchData 包含文件路径和产品信息的批次数据
   * @returns Observable<any> 返回创建的批次信息和分配结果
   */
  autoAllocateBatch(batchData: any): Observable<any> {
    console.log('🚀 [TAURI_API] 调用自动分配批次API');
    console.log('🚀 [TAURI_API] 参数:', batchData);

    return from(invoke('import_excel_and_prepare_batch_cmd', {
      args: {
        file_path_str: batchData.filePath,
        product_model: batchData.productModel,
        serial_number: batchData.serialNumber
      }
    })).pipe(
      tap(response => {
        console.log('✅ [TAURI_API] 自动分配批次成功');
        console.log('✅ [TAURI_API] 响应数据:', response);
      }),
      catchError(error => {
        console.error('❌ [TAURI_API] 自动分配批次失败:', error);
        throw error;
      })
    );
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
   * @deprecated 已废弃 - 测试区域不应该创建批次
   * 批次创建应该在点表导入时自动完成，测试区域只获取已存在的数据
   */
  createBatchAndPersistData(batchInfo: any, definitions: any[]): Observable<any> {
    console.error('❌ [TAURI_API] createBatchAndPersistData 已废弃，测试区域不应该创建批次');
    throw new Error('createBatchAndPersistData 已废弃，批次应该在点表导入时创建');
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
   * 🔧 新的统一导入和分配流程 - 符合架构设计
   *
   * 第一步：导入Excel到数据库（清空旧数据）
   * 第二步：创建批次（仅内存操作）
   */
  importExcelAndCreateBatch(filePath: string, batchName: string, productModel?: string, operatorName?: string): Observable<any> {
    console.log('🚀 [TAURI_API] 调用导入Excel并准备批次API (修复版)');
    console.log('🚀 [TAURI_API] 文件路径:', filePath);
    console.log('🚀 [TAURI_API] 产品型号:', productModel);

    return from(invoke('import_excel_and_prepare_batch_cmd', {
      args: {
        file_path_str: filePath,
        product_model: productModel,
        serial_number: operatorName // 使用操作员名称作为序列号
      }
    })).pipe(
      tap(result => {
        console.log('✅ [TAURI_API] 导入Excel并准备批次成功');
        console.log('✅ [TAURI_API] 结果:', result);
      }),
      catchError(error => {
        console.error('❌ [TAURI_API] 导入Excel并准备批次失败:', error);
        throw error;
      })
    );
  }

  /**
   * @deprecated 使用 importExcelAndCreateBatch 替代
   * 导入Excel文件并分配通道 - 完整的导入和分配流程
   */
  importExcelAndAllocateChannels(filePath: string, productModel?: string, serialNumber?: string): Observable<any> {
    console.warn('importExcelAndAllocateChannels 已废弃，请使用 importExcelAndCreateBatch');
    return from(invoke('import_excel_and_prepare_batch_cmd', {
      args: {
        file_path_str: filePath,
        product_model: productModel,
        serial_number: serialNumber
      }
    }));
  }

  /**
   * 获取批次详情和状态 - 从状态管理器获取批次的详细信息
   */
  getBatchDetails(batchId: string): Observable<BatchDetailsPayload> {
    console.log('📊 [TAURI_API] 调用获取批次详情API');
    console.log('📊 [TAURI_API] 批次ID:', batchId);
    return from(invoke<BatchDetailsPayload>('get_batch_status_cmd', {
      args: { batch_id: batchId }
    })).pipe(
      tap(details => {
        console.log('✅ [TAURI_API] 成功获取批次详情');
        console.log('✅ [TAURI_API] 批次信息:', details.batch_info);
        console.log('✅ [TAURI_API] 实例数量:', details.instances?.length || 0);
        console.log('✅ [TAURI_API] 定义数量:', details.definitions?.length || 0);
      }),
      catchError(error => {
        console.error('❌ [TAURI_API] 获取批次详情失败:', error);
        throw error;
      })
    );
  }

  /**
   * 准备批次测试实例
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
   * 获取通道映射配置
   */
  getChannelMappings(): Observable<any[]> {
    return from(invoke<any[]>('get_channel_mappings_cmd'));
  }

  /**
   * 清理当前会话数据
   */
  clearSessionData(): Observable<string> {
    return from(invoke<string>('clear_session_data'));
  }

  /**
   * 删除单个批次及其相关数据
   */
  deleteBatch(batchId: string): Observable<DeleteBatchResponse> {
    return from(invoke<DeleteBatchResponse>('delete_batch_cmd', {
      request: { batch_id: batchId }
    }));
  }

  /**
   * 恢复会话数据
   *
   * @param sessionKey 会话键（可选，19 位时间戳，形如 YYYY-MM-DDTHH:MM:SS）
   * @param batchId    可选批次ID，如果同时提供 batchId 与 sessionKey，后端会优先使用 batchId
   */
  restoreSession(sessionKey?: string, batchId?: string): Observable<TestBatchInfo[]> {
    console.log('🔄 [TAURI_API] 调用恢复会话 API', { sessionKey, batchId });
    const payload: any = {
      batch_id: batchId,
      session_key: sessionKey,
      batchId: batchId,
      sessionKey: sessionKey
    };
    console.log('[TAURI_API] restoreSession payload', payload);
    return from(invoke<TestBatchInfo[]>('restore_session_cmd', payload)).pipe(
      tap(list => console.log(`🔄 [TAURI_API] 恢复完成，加载 ${list.length} 个批次`)),
      catchError(err => {
        console.error('❌ [TAURI_API] 恢复会话失败:', err);
        throw err;
      })
    );
  }

  /**
   * 导出当前批次的通道分配表
   */
  exportChannelAllocation(targetPath: string | null | undefined): Observable<string> {
    const cleanedPath = (!targetPath || targetPath.trim().length === 0) ? null : targetPath.trim();
    console.log('📤 [TAURI_API] 准备导出通道分配表, cleanedPath=', cleanedPath);
    const payload = cleanedPath ? { targetPath: cleanedPath } : {};
    return from(invoke<string>('export_channel_allocation_cmd', payload)).pipe(
      tap(path => console.log('✅ [TAURI_API] 通道分配表导出成功, 文件路径:', path)),
      catchError(err => {
        console.error('❌ [TAURI_API] 通道分配表导出失败:', err);
        throw err;
      })
    );
  }

  /**
   * 导出测试结果 Excel
   */
  exportTestResults(targetPath: string | null | undefined): Observable<string> {
    const cleanedPath = (!targetPath || targetPath.trim().length === 0) ? null : targetPath.trim();
    console.log('📤 [TAURI_API] 准备导出测试结果, cleanedPath=', cleanedPath);
    const payload = cleanedPath ? { targetPath: cleanedPath } : {};
    return from(invoke<string>('export_test_results_cmd', payload)).pipe(
      tap(path => console.log('✅ [TAURI_API] 测试结果导出成功, 文件路径:', path)),
      catchError(err => {
        console.error('❌ [TAURI_API] 测试结果导出失败:', err);
        throw err;
      })
    );
  }

  // ============================================================================
  // 错误备注管理相关命令
  // ============================================================================

  /**
   * 保存通道测试实例的错误备注
   * @param instanceId 通道测试实例ID
   * @param integrationNotes 集成错误备注
   * @param plcNotes PLC编程错误备注  
   * @param hmiNotes 上位机组态错误备注
   */
  saveErrorNotes(
    instanceId: string,
    integrationNotes: string | null,
    plcNotes: string | null,
    hmiNotes: string | null
  ): Observable<void> {
    console.log('💾 [TAURI_API] 保存错误备注:', {
      instanceId,
      integrationNotes,
      plcNotes,
      hmiNotes
    });

    return from(invoke<void>('save_error_notes_cmd', {
      instanceId,
      integrationErrorNotes: integrationNotes,
      plcProgrammingErrorNotes: plcNotes,
      hmiConfigurationErrorNotes: hmiNotes
    })).pipe(
      tap(() => console.log('✅ [TAURI_API] 错误备注保存成功:', instanceId)),
      catchError(err => {
        console.error('❌ [TAURI_API] 错误备注保存失败:', err);
        throw err;
      })
    );
  }

  /**
   * 获取测试实例详情
   * 从数据库获取最新的测试实例数据，确保包含最新的错误备注信息
   * 
   * @param instanceId 测试实例ID
   * @returns Observable<ChannelTestInstance | null>
   */
  getTestInstanceDetails(instanceId: string): Observable<ChannelTestInstance | null> {
    console.log('📋 [TAURI_API] 获取测试实例详情:', instanceId);

    return from(invoke<ChannelTestInstance | null>('get_test_instance_details_cmd', {
      instanceId
    })).pipe(
      tap(instance => {
        if (instance) {
          console.log('✅ [TAURI_API] 测试实例详情获取成功:', instanceId);
        } else {
          console.warn('⚠️ [TAURI_API] 测试实例不存在:', instanceId);
        }
      }),
      catchError(err => {
        console.error('❌ [TAURI_API] 测试实例详情获取失败:', err);
        throw err;
      })
    );
  }

}