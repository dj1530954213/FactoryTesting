import { Injectable } from '@angular/core';
import { Observable, BehaviorSubject, from, of } from 'rxjs';
import { map, catchError, tap } from 'rxjs/operators';
import { invoke } from '@tauri-apps/api/core';
import { 
  TestPlcChannelConfig, 
  PlcConnectionConfig, 
  ChannelMappingConfig,
  TestPlcChannelType,
  PlcType,
  ConnectionStatus,
  TestPlcChannelTypeLabels,
  GenerateChannelMappingsRequest,
  GenerateChannelMappingsResponse,
  TestPlcConnectionResponse
} from '../models/test-plc-config.model';
import { TauriApiService } from './tauri-api.service';

@Injectable({
  providedIn: 'root'
})
export class TestPlcConfigService {
  
  // 状态管理
  private testPlcChannelsSubject = new BehaviorSubject<TestPlcChannelConfig[]>([]);
  private plcConnectionsSubject = new BehaviorSubject<PlcConnectionConfig[]>([]);
  private channelMappingsSubject = new BehaviorSubject<ChannelMappingConfig[]>([]);
  
  // 公开的Observable
  public testPlcChannels$ = this.testPlcChannelsSubject.asObservable();
  public plcConnections$ = this.plcConnectionsSubject.asObservable();
  public channelMappings$ = this.channelMappingsSubject.asObservable();

  constructor(private tauriApiService: TauriApiService) {
    this.initializeDefaultData();
  }

  // ============================================================================
  // 测试PLC通道配置管理
  // ============================================================================

  /**
   * 获取测试PLC通道配置
   */
  getTestPlcChannels(
    channelTypeFilter?: TestPlcChannelType,
    enabledOnly?: boolean
  ): Observable<TestPlcChannelConfig[]> {
    return from(
      invoke<TestPlcChannelConfig[]>('get_test_plc_channels_cmd', {
        channelTypeFilter,
        enabledOnly
      })
    );
  }

  /**
   * 保存测试PLC通道配置
   */
  saveTestPlcChannel(channel: TestPlcChannelConfig): Observable<TestPlcChannelConfig> {
    console.log('开始保存测试PLC通道配置:', channel);
    
    return from(
      invoke<TestPlcChannelConfig>('save_test_plc_channel_cmd', { channel })
    ).pipe(
      tap(result => {
        console.log('保存测试PLC通道配置成功:', result);
      }),
      catchError(error => {
        console.error('保存测试PLC通道配置失败:', error);
        console.error('错误详情:', {
          message: error.message,
          stack: error.stack,
          name: error.name,
          cause: error.cause
        });
        throw error;
      })
    );
  }

  /**
   * 删除测试PLC通道配置
   */
  deleteTestPlcChannel(channelId: string): Observable<void> {
    return from(
      invoke<void>('delete_test_plc_channel_cmd', { channelId })
    );
  }

  // ============================================================================
  // PLC连接配置管理
  // ============================================================================

  /**
   * 获取PLC连接配置
   */
  getPlcConnections(): Observable<PlcConnectionConfig[]> {
    return from(
      invoke<PlcConnectionConfig[]>('get_plc_connections_cmd')
    );
  }

  /**
   * 保存PLC连接配置
   */
  savePlcConnection(connection: PlcConnectionConfig): Observable<PlcConnectionConfig> {
    return from(
      invoke<PlcConnectionConfig>('save_plc_connection_cmd', { connection })
    );
  }

  /**
   * 测试PLC连接
   */
  testPlcConnection(connectionId: string): Observable<TestPlcConnectionResponse> {
    return from(
      invoke<TestPlcConnectionResponse>('test_plc_connection_cmd', { connectionId })
    );
  }

  // ============================================================================
  // 通道映射配置管理
  // ============================================================================

  /**
   * 获取通道映射配置
   */
  getChannelMappings(): Observable<ChannelMappingConfig[]> {
    return from(
      invoke<ChannelMappingConfig[]>('get_channel_mappings_cmd')
    );
  }

  /**
   * 自动生成通道映射
   */
  generateChannelMappings(request: GenerateChannelMappingsRequest): Observable<GenerateChannelMappingsResponse> {
    return from(
      invoke<GenerateChannelMappingsResponse>('generate_channel_mappings_cmd', { request })
    );
  }

  // ============================================================================
  // 私有方法
  // ============================================================================

  /**
   * 初始化默认数据
   */
  private initializeDefaultData(): void {
    // 现在数据从数据库加载，不需要硬编码的默认数据
    // 初始化时调用后端的初始化方法
    this.initializeDefaultTestPlcChannels().subscribe({
      next: () => {
        console.log('默认测试PLC通道配置初始化完成');
        this.refreshTestPlcChannels();
        this.refreshPlcConnections();
      },
      error: (error) => {
        console.error('初始化默认测试PLC通道配置失败:', error);
      }
    });
  }

  /**
   * 刷新测试PLC通道列表
   */
  private refreshTestPlcChannels(): void {
    this.getTestPlcChannels().subscribe(channels => {
      this.testPlcChannelsSubject.next(channels);
    });
  }

  /**
   * 刷新PLC连接列表
   */
  private refreshPlcConnections(): void {
    this.getPlcConnections().subscribe(connections => {
      this.plcConnectionsSubject.next(connections);
    });
  }

  /**
   * 刷新通道映射列表
   */
  private refreshChannelMappings(): void {
    this.getChannelMappings().subscribe(mappings => {
      this.channelMappingsSubject.next(mappings);
    });
  }

  /**
   * 生成唯一ID
   */
  private generateId(): string {
    return Date.now().toString(36) + Math.random().toString(36).substr(2);
  }

  /**
   * 初始化默认测试PLC通道配置
   */
  initializeDefaultTestPlcChannels(): Observable<void> {
    return from(
      invoke<void>('initialize_default_test_plc_channels_cmd')
    );
  }

  /**
   * 按通道类型分组通道配置
   */
  groupChannelsByType(channels: TestPlcChannelConfig[]): Record<string, TestPlcChannelConfig[]> {
    return channels.reduce((groups, channel) => {
      const type = channel.channelType.toString();
      if (!groups[type]) {
        groups[type] = [];
      }
      groups[type].push(channel);
      return groups;
    }, {} as Record<string, TestPlcChannelConfig[]>);
  }

  /**
   * 获取可用的测试PLC通道统计
   */
  getChannelStatistics(channels: TestPlcChannelConfig[]): {
    total: number;
    enabled: number;
    byType: Record<TestPlcChannelType, number>;
  } {
    const stats = {
      total: channels.length,
      enabled: channels.filter(c => c.isEnabled).length,
      byType: {} as Record<TestPlcChannelType, number>
    };

    // 初始化各类型计数
    Object.values(TestPlcChannelType).forEach(type => {
      if (typeof type === 'number') {
        stats.byType[type] = 0;
      }
    });

    // 统计各类型数量
    channels.forEach(channel => {
      stats.byType[channel.channelType]++;
    });

    return stats;
  }

  /**
   * 验证通道配置
   */
  validateChannelConfig(channel: TestPlcChannelConfig): string[] {
    const errors: string[] = [];

    if (!channel.channelAddress?.trim()) {
      errors.push('通道位号不能为空');
    }

    if (!channel.communicationAddress?.trim()) {
      errors.push('通讯地址不能为空');
    }

    if (!channel.powerSupplyType?.trim()) {
      errors.push('供电类型不能为空');
    }

    if (channel.channelType === undefined || channel.channelType === null) {
      errors.push('通道类型不能为空');
    }

    return errors;
  }

  /**
   * 创建默认的通道配置
   */
  createDefaultChannelConfig(): TestPlcChannelConfig {
    return {
      channelAddress: '',
      channelType: TestPlcChannelType.AI,
      communicationAddress: '',
      powerSupplyType: '24V DC',
      description: '',
      isEnabled: true
    };
  }
} 