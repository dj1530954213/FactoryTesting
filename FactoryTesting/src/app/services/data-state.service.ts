/**
 * # 数据状态管理服务 - DataStateService
 * 
 * ## 业务功能说明
 * - 管理数据导入流程的状态和进度
 * - 缓存Excel解析结果和批次信息
 * - 提供跨组件的状态共享机制
 * - 支持导入流程的断点续传
 * 
 * ## 前后端调用链
 * - **状态管理**: 纯前端状态管理，不直接调用后端
 * - **数据缓存**: 缓存从TauriApiService获取的数据
 * - **流程控制**: 协调data-import组件的多步骤流程
 * 
 * ## Angular知识点
 * - **Injectable**: 全局单例服务，根级注入
 * - **BehaviorSubject**: 有初始值的响应式状态管理
 * - **Observable**: 响应式数据流，支持订阅模式
 * - **本地存储**: localStorage持久化状态信息
 * 
 * ## 设计模式
 * - **观察者模式**: 通过Observable通知状态变化
 * - **单例模式**: 全局唯一的状态管理实例
 * - **状态机模式**: 管理导入流程的状态转换
 */

import { Injectable } from '@angular/core';
import { BehaviorSubject, Observable } from 'rxjs';

/**
 * 导入状态接口
 * 
 * **业务含义**: Excel文件导入流程的状态信息
 * **状态管理**: 跟踪多步骤导入过程的当前进度
 */
export interface ImportState {
  currentStep: number;    // 当前步骤 (0: 选择文件, 1: 解析预览, 2: 配置批次, 3: 完成导入)
  isImporting: boolean;   // 是否正在导入中
  importProgress: number; // 导入进度百分比 (0-100)
  importResult: any;      // 导入结果数据
  selectedFile?: any;     // 选中的文件信息
}

/**
 * 测试数据状态接口
 * 
 * **业务含义**: 内存中缓存的测试数据状态
 * **用途**: 避免重复解析，提高用户体验
 */
export interface TestDataState {
  parsedDefinitions: any[];  // 解析后的通道定义列表
  suggestedBatchInfo: any;   // 建议的批次信息
  isDataAvailable: boolean;  // 数据是否可用
  lastParseTime?: Date;      // 最后解析时间
}

/**
 * 数据状态管理服务类
 * 
 * **业务作用**: 管理Excel导入流程和测试数据的状态
 * **服务范围**: 全局单例，跨组件状态共享
 * **持久化**: 支持本地存储，刷新页面后状态保持
 */
@Injectable({
  providedIn: 'root'
})
export class DataStateService {
  /** 本地存储键名 */
  private readonly STORAGE_KEY = 'fat_test_import_state';
  
  /** 
   * 初始导入状态
   * **设计原则**: 清空状态，避免预设数据干扰
   */
  private initialState: ImportState = {
    currentStep: 0,        // 从第一步开始
    isImporting: false,    // 初始未在导入中
    importProgress: 0,     // 进度为0
    importResult: null,    // 无导入结果
    selectedFile: null     // 无选中文件
  };

  // === 响应式状态管理 ===
  
  /** 
   * 导入状态主题
   * **业务用途**: 跟踪Excel导入流程的状态变化
   */
  private importStateSubject = new BehaviorSubject<ImportState>(this.initialState);
  
  /** 
   * 导入状态可观察流
   * **订阅方式**: 组件通过订阅获取状态更新
   */
  public importState$ = this.importStateSubject.asObservable();

  /** 
   * 测试数据状态主题
   * **业务用途**: 缓存解析后的测试数据，避免重复处理
   */
  private testDataStateSubject = new BehaviorSubject<TestDataState>({
    parsedDefinitions: [],     // 空的定义列表
    suggestedBatchInfo: null,  // 无建议批次信息
    isDataAvailable: false,    // 数据不可用
    lastParseTime: undefined   // 无解析时间
  });

  /** 
   * 测试数据状态可观察流
   * **用途**: 组件获取缓存的测试数据状态
   */
  public testDataState$ = this.testDataStateSubject.asObservable();

  /**
   * 构造函数
   * **初始化**: 从本地存储恢复状态，保持用户体验连续性
   */
  constructor() {
    // 应用启动时清除所有存储的状态，确保干净的初始状态
    this.clearStoredState();
    this.importStateSubject.next(this.initialState);
  }

  // 更新导入状态
  updateImportState(partialState: Partial<ImportState>): void {
    const currentState = this.importStateSubject.value;
    const newState = { ...currentState, ...partialState };
    
    // 不保存selectedFile到localStorage，因为File对象无法序列化
    const stateToStore: Partial<ImportState> = { ...newState };
    if ('selectedFile' in stateToStore) {
      delete (stateToStore as any).selectedFile;
    }
    
    this.importStateSubject.next(newState);
    this.saveToStorage(stateToStore);
  }

  // 获取当前导入状态
  getCurrentImportState(): ImportState {
    return this.importStateSubject.value;
  }

  // 重置导入状态
  resetImportState(): void {
    this.importStateSubject.next(this.initialState);
    this.clearStoredState();
  }

  // 保存状态到localStorage
  private saveToStorage(state: Partial<ImportState>): void {
    try {
      localStorage.setItem(this.STORAGE_KEY, JSON.stringify(state));
    } catch (error) {
      console.warn('无法保存导入状态到localStorage:', error);
    }
  }

  // 从localStorage加载状态
  private loadFromStorage(): Partial<ImportState> | null {
    try {
      const stored = localStorage.getItem(this.STORAGE_KEY);
      return stored ? JSON.parse(stored) : null;
    } catch (error) {
      console.warn('无法从localStorage加载导入状态:', error);
      return null;
    }
  }

  // 清除存储的状态
  private clearStoredState(): void {
    try {
      localStorage.removeItem(this.STORAGE_KEY);
    } catch (error) {
      console.warn('无法清除localStorage中的导入状态:', error);
    }
  }

  // 恢复状态（仅在用户明确操作时调用）
  restoreStateFromStorage(): void {
    const storedState = this.loadFromStorage();
    if (storedState) {
      const restoredState = { ...this.initialState, ...storedState };
      this.importStateSubject.next(restoredState);
    }
  }

  // 清理所有数据（公共方法）
  clearAllData(): void {
    this.clearStoredState();
    this.importStateSubject.next(this.initialState);
  }

  // 内存中的测试数据管理方法
  setTestData(definitions: any[], batchInfo: any): void {
    this.testDataStateSubject.next({
      parsedDefinitions: definitions,
      suggestedBatchInfo: batchInfo,
      isDataAvailable: true,
      lastParseTime: new Date()
    });
  }

  getTestData(): TestDataState {
    return this.testDataStateSubject.value;
  }

  clearTestData(): void {
    this.testDataStateSubject.next({
      parsedDefinitions: [],
      suggestedBatchInfo: null,
      isDataAvailable: false,
      lastParseTime: undefined
    });
  }

  hasTestData(): boolean {
    return this.testDataStateSubject.value.isDataAvailable && 
           this.testDataStateSubject.value.parsedDefinitions.length > 0;
  }
} 