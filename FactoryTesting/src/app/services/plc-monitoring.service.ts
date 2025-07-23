import { Injectable } from '@angular/core';
import { Observable, BehaviorSubject, Subject } from 'rxjs';
import { invoke } from '@tauri-apps/api/core';
import { listen, UnlistenFn } from '@tauri-apps/api/event';
import {
  StartPlcMonitoringRequest,
  StartPlcMonitoringResponse,
  StopPlcMonitoringRequest,
  PlcMonitoringData,
  AiPlcMonitoringData,
  AoPlcMonitoringData,
  DigitalPlcMonitoringData
} from '../models/manual-test.types';
import { ModuleType } from '../models';

/**
 * PLC监控服务
 * 负责管理PLC变量的实时监控
 */
@Injectable({
  providedIn: 'root'
})
export class PlcMonitoringService {
  // 当前监控数据
  private currentMonitoringData = new BehaviorSubject<PlcMonitoringData | null>(null);
  public currentMonitoringData$ = this.currentMonitoringData.asObservable();

  // 监控状态
  private isMonitoring = new BehaviorSubject<boolean>(false);
  public isMonitoring$ = this.isMonitoring.asObservable();

  // 监控错误事件
  private monitoringError = new Subject<string>();
  public monitoringError$ = this.monitoringError.asObservable();

  // 当前监控ID
  private currentMonitoringId: string | null = null;
  private currentInstanceId: string | null = null;

  // 事件监听器
  private eventUnlisteners: UnlistenFn[] = [];

  constructor() {
    this.setupEventListeners();
  }

  /**
   * 设置事件监听器
   */
  private async setupEventListeners(): Promise<void> {
    try {
      // 监听PLC监控数据更新事件
      const unlistenData = await listen<PlcMonitoringData>('plc-monitoring-data', (event) => {
        //console.log('📊 [PLC_MONITORING_SERVICE] 收到PLC监控数据:', event.payload);
        this.currentMonitoringData.next(event.payload);
      });

      // 监听PLC监控错误事件
      const unlistenError = await listen<{ instanceId: string; error: string }>('plc-monitoring-error', (event) => {
        console.error('❌ [PLC_MONITORING_SERVICE] PLC监控错误:', event.payload);
        this.monitoringError.next(event.payload.error);
      });

      // 监听PLC监控停止事件
      const unlistenStopped = await listen<{ instanceId: string; reason: string }>('plc-monitoring-stopped', (event) => {
        console.log('🛑 [PLC_MONITORING_SERVICE] PLC监控已停止:', event.payload);
        if (event.payload.instanceId === this.currentInstanceId) {
          this.stopMonitoringInternal();
        }
      });

      this.eventUnlisteners = [unlistenData, unlistenError, unlistenStopped];
      console.log('✅ [PLC_MONITORING_SERVICE] 事件监听器设置成功');
    } catch (error) {
      console.error('❌ [PLC_MONITORING_SERVICE] 设置事件监听器失败:', error);
    }
  }

  /**
   * 开始PLC监控
   */
  async startMonitoring(request: StartPlcMonitoringRequest): Promise<StartPlcMonitoringResponse> {
    try {
      console.log('🔧 [PLC_MONITORING_SERVICE] 开始PLC监控:', request);

            // 启动前保险：先尝试停止所有可能存在的监控（即使本地认为未监控）
      await this.forceStopMonitoring(request.instanceId);

      // 如果已有监控在运行，先停止（包含其他实例）
      if (this.isMonitoring.value) {
        await this.stopMonitoring();
      }

      const response = await invoke<StartPlcMonitoringResponse>('start_plc_monitoring_cmd', {
        request
      });

      if (response.success) {
        this.currentMonitoringId = response.monitoringId || null;
        this.currentInstanceId = request.instanceId;
        this.isMonitoring.next(true);
        console.log('✅ [PLC_MONITORING_SERVICE] PLC监控已启动:', response.monitoringId);
      }

      return response;
    } catch (error) {
      console.error('❌ [PLC_MONITORING_SERVICE] 启动PLC监控失败:', error);
      throw new Error(`启动PLC监控失败: ${error}`);
    }
  }

  /**
   * 停止PLC监控
   */
  /**
   * 强制停止指定实例的所有监控（不携带 monitoringId）。
   * 后端应当根据 instanceId 清理全部相关任务。
   */
  private async forceStopMonitoring(instanceId: string): Promise<void> {
    try {
      const request: StopPlcMonitoringRequest = { instanceId };
      await invoke('stop_plc_monitoring_cmd', { request });
    } catch (error) {
      console.warn('⚠️ [PLC_MONITORING_SERVICE] 强制停止监控失败(可以忽略):', error);
    }
  }

  async stopMonitoring(): Promise<void> {
    try {
      if (!this.currentInstanceId) {
        console.log('🔧 [PLC_MONITORING_SERVICE] 没有活跃的监控，无需停止');
        return;
      }

      console.log('🔧 [PLC_MONITORING_SERVICE] 停止PLC监控:', this.currentInstanceId);

      const request: StopPlcMonitoringRequest = {
        instanceId: this.currentInstanceId,
        monitoringId: this.currentMonitoringId || undefined
      };

      await invoke('stop_plc_monitoring_cmd', { request });

      // 额外保险：再发一次不带 monitoringId 的停止请求，防止之前启动失败导致 id 不匹配
      await this.forceStopMonitoring(this.currentInstanceId);

      this.stopMonitoringInternal();
      console.log('✅ [PLC_MONITORING_SERVICE] PLC监控已停止');
    } catch (error) {
      console.error('❌ [PLC_MONITORING_SERVICE] 停止PLC监控失败:', error);
      // 即使停止失败，也要清理本地状态
      this.stopMonitoringInternal();
    }
  }

  /**
   * 内部停止监控方法
   */
  private stopMonitoringInternal(): void {
    this.isMonitoring.next(false);
    this.currentMonitoringData.next(null);
    this.currentMonitoringId = null;
    this.currentInstanceId = null;
  }

  /**
   * 获取AI点位的监控数据
   */
  getAiMonitoringData(): AiPlcMonitoringData | null {
    const data = this.currentMonitoringData.value;
    if (!data) return null;

    return data as AiPlcMonitoringData;
  }

  /**
   * 获取AO点位的监控数据
   */
  getAoMonitoringData(): AoPlcMonitoringData | null {
    const data = this.currentMonitoringData.value;
    if (!data) return null;

    return data as AoPlcMonitoringData;
  }

  /**
   * 获取数字量点位的监控数据
   */
  getDigitalMonitoringData(): DigitalPlcMonitoringData | null {
    const data = this.currentMonitoringData.value;
    if (!data) return null;

    return data as DigitalPlcMonitoringData;
  }

  /**
   * 检查是否正在监控指定实例
   */
  isMonitoringInstance(instanceId: string): boolean {
    return this.isMonitoring.value && this.currentInstanceId === instanceId;
  }

  /**
   * 获取监控数据的特定值
   */
  getMonitoringValue(key: string): any {
    const data = this.currentMonitoringData.value;
    if (!data || !data.values) return undefined; // 没有监控数据时返回undefined

    return data.values[key]; // 可能返回null（空地址）或具体值
  }

  /**
   * 获取格式化的监控值文本
   */
  getFormattedMonitoringValue(key: string, moduleType: ModuleType): string {
    const value = this.getMonitoringValue(key);
    
    // 如果值为null（对应空地址），显示'--'
    if (value === null) {
      return '--';
    }
    
    // 如果值为undefined（监控数据还未到达），显示'读取中...'
    if (value === undefined) {
      return '读取中...';
    }

    switch (moduleType) {
      case ModuleType.AI:
      case ModuleType.AO:
        if (typeof value === 'number') {
          return value.toFixed(3);
        }
        break;
      
      case ModuleType.DI:
      case ModuleType.DO:
        if (typeof value === 'boolean') {
          return value ? 'ON' : 'OFF';
        }
        if (typeof value === 'string') {
          return value;
        }
        break;
    }

    return String(value);
  }

  /**
   * 重置服务状态（用于组件销毁时清理）
   */
  async reset(): Promise<void> {
    console.log('🔧 [PLC_MONITORING_SERVICE] 重置服务状态');
    
    // 停止监控
    await this.stopMonitoring();
    
    // 清理事件监听器
    this.eventUnlisteners.forEach(unlisten => {
      try {
        unlisten();
      } catch (error) {
        console.error('清理事件监听器失败:', error);
      }
    });
    this.eventUnlisteners = [];
  }

  /**
   * 获取当前监控状态信息
   */
  getCurrentMonitoringInfo(): {
    isMonitoring: boolean;
    instanceId: string | null;
    monitoringId: string | null;
    hasData: boolean;
  } {
    return {
      isMonitoring: this.isMonitoring.value,
      instanceId: this.currentInstanceId,
      monitoringId: this.currentMonitoringId,
      hasData: this.currentMonitoringData.value !== null
    };
  }
}
