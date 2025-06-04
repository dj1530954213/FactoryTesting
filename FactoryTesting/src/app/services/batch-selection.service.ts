import { Injectable } from '@angular/core';
import { BehaviorSubject, Observable } from 'rxjs';
import { TestBatchInfo } from '../models';

/**
 * 批次选择状态管理服务
 * 负责管理用户选择的批次状态，并在页面跳转时保持状态
 */
@Injectable({
  providedIn: 'root'
})
export class BatchSelectionService {
  private readonly STORAGE_KEY = 'fat_test_selected_batch_id';
  
  // 当前选中的批次
  private selectedBatchSubject = new BehaviorSubject<TestBatchInfo | null>(null);
  public selectedBatch$ = this.selectedBatchSubject.asObservable();
  
  // 可用批次列表
  private availableBatchesSubject = new BehaviorSubject<TestBatchInfo[]>([]);
  public availableBatches$ = this.availableBatchesSubject.asObservable();

  constructor() {
    console.log('🔧 [BATCH_SELECTION] 批次选择服务初始化');
  }

  /**
   * 设置可用批次列表
   */
  setAvailableBatches(batches: TestBatchInfo[]): void {
    console.log('📋 [BATCH_SELECTION] 设置可用批次列表，数量:', batches.length);
    this.availableBatchesSubject.next(batches);
    
    // 尝试恢复之前选择的批次
    this.restoreSelectedBatch(batches);
  }

  /**
   * 获取当前可用批次列表
   */
  getAvailableBatches(): TestBatchInfo[] {
    return this.availableBatchesSubject.value;
  }

  /**
   * 选择批次
   */
  selectBatch(batch: TestBatchInfo): void {
    console.log('✅ [BATCH_SELECTION] 选择批次:', batch.batch_id, batch.batch_name);
    this.selectedBatchSubject.next(batch);
    
    // 保存到本地存储
    this.saveSelectedBatchId(batch.batch_id);
  }

  /**
   * 获取当前选中的批次
   */
  getSelectedBatch(): TestBatchInfo | null {
    return this.selectedBatchSubject.value;
  }

  /**
   * 清除选择
   */
  clearSelection(): void {
    console.log('🧹 [BATCH_SELECTION] 清除批次选择');
    this.selectedBatchSubject.next(null);
    this.clearStoredBatchId();
  }

  /**
   * 自动选择第一个批次（如果没有选择任何批次）
   */
  autoSelectFirstBatch(): void {
    const batches = this.getAvailableBatches();
    const currentSelection = this.getSelectedBatch();
    
    if (batches.length > 0 && !currentSelection) {
      console.log('🎯 [BATCH_SELECTION] 自动选择第一个批次:', batches[0].batch_id);
      this.selectBatch(batches[0]);
    }
  }

  /**
   * 检查选中的批次是否仍然存在于可用列表中
   */
  validateSelectedBatch(): boolean {
    const selectedBatch = this.getSelectedBatch();
    const availableBatches = this.getAvailableBatches();
    
    if (!selectedBatch) {
      return false;
    }
    
    const exists = availableBatches.some(batch => batch.batch_id === selectedBatch.batch_id);
    
    if (!exists) {
      console.log('⚠️ [BATCH_SELECTION] 选中的批次不再存在，清除选择');
      this.clearSelection();
      return false;
    }
    
    return true;
  }

  /**
   * 保存选中的批次ID到本地存储
   */
  private saveSelectedBatchId(batchId: string): void {
    try {
      localStorage.setItem(this.STORAGE_KEY, batchId);
      console.log('💾 [BATCH_SELECTION] 已保存批次ID到本地存储:', batchId);
    } catch (error) {
      console.warn('⚠️ [BATCH_SELECTION] 保存批次ID失败:', error);
    }
  }

  /**
   * 从本地存储获取批次ID
   */
  private getStoredBatchId(): string | null {
    try {
      const batchId = localStorage.getItem(this.STORAGE_KEY);
      console.log('📖 [BATCH_SELECTION] 从本地存储读取批次ID:', batchId);
      return batchId;
    } catch (error) {
      console.warn('⚠️ [BATCH_SELECTION] 读取批次ID失败:', error);
      return null;
    }
  }

  /**
   * 清除本地存储的批次ID
   */
  private clearStoredBatchId(): void {
    try {
      localStorage.removeItem(this.STORAGE_KEY);
      console.log('🗑️ [BATCH_SELECTION] 已清除本地存储的批次ID');
    } catch (error) {
      console.warn('⚠️ [BATCH_SELECTION] 清除批次ID失败:', error);
    }
  }

  /**
   * 恢复之前选择的批次
   */
  private restoreSelectedBatch(batches: TestBatchInfo[]): void {
    const storedBatchId = this.getStoredBatchId();
    
    if (storedBatchId) {
      // 查找存储的批次ID对应的批次
      const batch = batches.find(b => b.batch_id === storedBatchId);
      
      if (batch) {
        console.log('🔄 [BATCH_SELECTION] 恢复之前选择的批次:', batch.batch_id);
        this.selectedBatchSubject.next(batch);
        return;
      } else {
        console.log('⚠️ [BATCH_SELECTION] 存储的批次ID不存在于当前列表中，清除存储');
        this.clearStoredBatchId();
      }
    }
    
    // 如果没有存储的批次或存储的批次不存在，自动选择第一个
    if (batches.length > 0) {
      console.log('🎯 [BATCH_SELECTION] 自动选择第一个批次:', batches[0].batch_id);
      this.selectBatch(batches[0]);
    }
  }

  /**
   * 重置服务状态
   */
  reset(): void {
    console.log('🔄 [BATCH_SELECTION] 重置服务状态');
    this.selectedBatchSubject.next(null);
    this.availableBatchesSubject.next([]);
    this.clearStoredBatchId();
  }
}
