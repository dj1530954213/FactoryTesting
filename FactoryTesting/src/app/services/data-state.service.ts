import { Injectable } from '@angular/core';
import { BehaviorSubject, Observable } from 'rxjs';

export interface ImportState {
  currentStep: number;
  isImporting: boolean;
  importProgress: number;
  importResult: any | null;
  selectedFile: File | null;
}

@Injectable({
  providedIn: 'root'
})
export class DataStateService {
  private readonly STORAGE_KEY = 'fat_test_import_state';
  
  // 初始状态 - 完全清空，没有任何预设数据
  private initialState: ImportState = {
    currentStep: 0,
    isImporting: false,
    importProgress: 0,
    importResult: null,
    selectedFile: null
  };

  private importStateSubject = new BehaviorSubject<ImportState>(this.initialState);
  public importState$ = this.importStateSubject.asObservable();

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
} 