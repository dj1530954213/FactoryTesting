import { Component, OnInit, OnDestroy } from '@angular/core';
import { CommonModule } from '@angular/common';
import { FormsModule, ReactiveFormsModule, FormBuilder, FormGroup, Validators } from '@angular/forms';

// NG-ZORRO 组件导入
import { NzCardModule } from 'ng-zorro-antd/card';
import { NzButtonModule } from 'ng-zorro-antd/button';
import { NzIconModule } from 'ng-zorro-antd/icon';
import { NzSpaceModule } from 'ng-zorro-antd/space';
import { NzTableModule } from 'ng-zorro-antd/table';
import { NzTagModule } from 'ng-zorro-antd/tag';
import { NzModalModule } from 'ng-zorro-antd/modal';
import { NzFormModule } from 'ng-zorro-antd/form';
import { NzInputModule } from 'ng-zorro-antd/input';
import { NzSelectModule } from 'ng-zorro-antd/select';
import { NzSwitchModule } from 'ng-zorro-antd/switch';
import { NzMessageService } from 'ng-zorro-antd/message';
import { NzTabsModule } from 'ng-zorro-antd/tabs';
import { NzDividerModule } from 'ng-zorro-antd/divider';
import { NzInputNumberModule } from 'ng-zorro-antd/input-number';
import { NzAlertModule } from 'ng-zorro-antd/alert';
import { NzSpinModule } from 'ng-zorro-antd/spin';

// 服务和模型导入
import { TestPlcConfigService } from '../../services/test-plc-config.service';
import { 
  TestPlcChannelConfig, 
  PlcConnectionConfig,
  TestPlcChannelType,
  PlcType,
  ConnectionStatus,
  TestPlcChannelTypeLabels,
  PlcTypeLabels,
  ConnectionStatusLabels,
  getChannelTypeColor,
  getConnectionStatusColor,
  TestPlcConnectionResponse
} from '../../models/test-plc-config.model';
import { Subscription } from 'rxjs';

@Component({
  selector: 'app-test-plc-config',
  standalone: true,
  imports: [
    CommonModule,
    FormsModule,
    ReactiveFormsModule,
    NzCardModule,
    NzButtonModule,
    NzIconModule,
    NzSpaceModule,
    NzTableModule,
    NzTagModule,
    NzModalModule,
    NzFormModule,
    NzInputModule,
    NzSelectModule,
    NzSwitchModule,
    NzTabsModule,
    NzDividerModule,
    NzInputNumberModule,
    NzAlertModule,
    NzSpinModule
  ],
  template: `
    <div class="test-plc-config-container">
      <div class="page-header">
        <h2>
          <span nz-icon nzType="setting" nzTheme="outline"></span>
          测试PLC配置管理
        </h2>
        <p>管理测试PLC和被测PLC的连接配置以及测试PLC通道配置</p>
      </div>

      <!-- PLC连接配置区域 -->
      <nz-card nzTitle="PLC连接配置" class="config-section">
        <nz-alert 
          nzType="info" 
          nzMessage="PLC连接管理" 
          nzDescription="配置测试PLC和被测PLC的连接参数，确保通讯正常。支持Modbus TCP、Siemens S7、OPC UA等协议。"
          nzShowIcon>
        </nz-alert>
        
        <div class="action-buttons">
          <button nz-button nzType="primary" (click)="showAddConnectionModal()">
            <span nz-icon nzType="plus" nzTheme="outline"></span>
            添加PLC连接
          </button>
        </div>

        <!-- PLC连接列表 -->
        <nz-table #plcConnectionTable [nzData]="plcConnections" nzBordered nzSize="middle">
          <thead>
            <tr>
              <th>连接名称</th>
              <th>PLC类型</th>
              <th>IP地址</th>
              <th>端口</th>
              <th>用途</th>
              <th>连接状态</th>
              <th>操作</th>
            </tr>
          </thead>
          <tbody>
            <tr *ngFor="let connection of plcConnectionTable.data">
              <td>{{ connection.name }}</td>
              <td>
                <nz-tag nzColor="blue">{{ getPlcTypeLabel(connection.plcType) }}</nz-tag>
              </td>
              <td>{{ connection.ipAddress }}</td>
              <td>{{ connection.port }}</td>
              <td>
                <nz-tag [nzColor]="connection.isTestPlc ? 'green' : 'orange'">
                  {{ connection.isTestPlc ? '测试PLC' : '被测PLC' }}
                </nz-tag>
              </td>
              <td>
                <nz-tag [nzColor]="getConnectionStatusColor(connection.connectionStatus)">
                  {{ getConnectionStatusLabel(connection.connectionStatus) }}
                </nz-tag>
              </td>
              <td>
                <nz-space>
                  <button *nzSpaceItem nz-button nzType="primary" nzSize="small" 
                          [nzLoading]="testingConnections.has(connection.id)"
                          (click)="testConnection(connection.id)">
                    <span nz-icon nzType="api" nzTheme="outline"></span>
                    测试连接
                  </button>
                  <button *nzSpaceItem nz-button nzType="default" nzSize="small" 
                          (click)="editConnection(connection)">
                    <span nz-icon nzType="edit" nzTheme="outline"></span>
                    编辑
                  </button>
                </nz-space>
              </td>
            </tr>
          </tbody>
        </nz-table>
      </nz-card>

      <!-- 测试PLC通道配置区域 -->
      <nz-card nzTitle="测试PLC通道配置" class="config-section">
        <nz-alert 
          nzType="info" 
          nzMessage="测试PLC通道管理" 
          nzDescription="配置测试PLC的所有通道信息，包括通道位号、类型、通讯地址和供电类型。系统预置88个标准通道配置。"
          nzShowIcon>
        </nz-alert>
        
        <!-- 筛选和搜索 -->
        <div class="filter-section">
          <nz-space>
            <nz-select *nzSpaceItem nzPlaceHolder="选择通道类型" 
                       [(ngModel)]="selectedChannelType" 
                       (ngModelChange)="filterChannels()"
                       nzAllowClear
                       style="width: 200px;">
              <nz-option nzValue="" nzLabel="所有类型"></nz-option>
              <nz-option *ngFor="let type of channelTypes" 
                         [nzValue]="type" 
                         [nzLabel]="getChannelTypeLabel(type)">
              </nz-option>
            </nz-select>
            
            <nz-input-group *nzSpaceItem nzPrefixIcon="search" style="width: 300px;">
              <input nz-input placeholder="搜索通道位号或地址..." 
                     [(ngModel)]="searchTerm" 
                     (ngModelChange)="filterChannels()">
            </nz-input-group>
            
            <span *nzSpaceItem class="filter-info">
              共 {{ testPlcChannels.length }} 个通道，显示 {{ filteredChannels.length }} 个
            </span>
          </nz-space>
        </div>

        <div class="action-buttons">
          <nz-space>
            <button *nzSpaceItem nz-button nzType="primary" (click)="showAddChannelModal()">
              <span nz-icon nzType="plus" nzTheme="outline"></span>
              添加通道
            </button>
            <button *nzSpaceItem nz-button nzType="default" (click)="refreshChannels()">
              <span nz-icon nzType="reload" nzTheme="outline"></span>
              刷新列表
            </button>
          </nz-space>
        </div>

        <!-- 测试PLC通道列表 -->
        <nz-table #channelTable [nzData]="filteredChannels" nzBordered nzSize="middle" 
                  [nzPageSize]="20" [nzShowPagination]="true">
          <thead>
            <tr>
              <th>通道位号</th>
              <th>通道类型</th>
              <th>通讯地址</th>
              <th>供电类型</th>
              <th>状态</th>
              <th>描述</th>
              <th>操作</th>
            </tr>
          </thead>
          <tbody>
            <tr *ngFor="let channel of channelTable.data">
              <td>{{ channel.channelAddress }}</td>
              <td>
                <nz-tag [nzColor]="getChannelTypeColor(channel.channelType)">
                  {{ getChannelTypeLabel(channel.channelType) }}
                </nz-tag>
              </td>
              <td>{{ channel.communicationAddress }}</td>
              <td>
                <nz-tag nzColor="purple">{{ channel.powerSupplyType || '未设置' }}</nz-tag>
              </td>
              <td>
                <nz-tag [nzColor]="channel.isEnabled ? 'green' : 'default'">
                  {{ channel.isEnabled ? '启用' : '禁用' }}
                </nz-tag>
              </td>
              <td>{{ channel.description || '-' }}</td>
              <td>
                <nz-space>
                  <button *nzSpaceItem nz-button nzType="default" nzSize="small" 
                          (click)="editChannel(channel)">
                    <span nz-icon nzType="edit" nzTheme="outline"></span>
                    编辑
                  </button>
                  <button *nzSpaceItem nz-button nzType="primary" nzDanger nzSize="small" 
                          (click)="deleteChannel(channel.id!)">
                    <span nz-icon nzType="delete" nzTheme="outline"></span>
                    删除
                  </button>
                </nz-space>
              </td>
            </tr>
          </tbody>
        </nz-table>
      </nz-card>

      <!-- PLC连接配置模态框 -->
      <nz-modal 
        [(nzVisible)]="isConnectionModalVisible"
        [nzTitle]="isEditingConnection ? '编辑PLC连接' : '添加PLC连接'"
        [nzFooter]="null"
        (nzOnCancel)="closeConnectionModal()">
        
        <ng-container *nzModalContent>
          <form nz-form [formGroup]="connectionForm" (ngSubmit)="saveConnection()">
            <nz-form-item>
              <nz-form-label [nzSpan]="6" nzRequired>连接名称</nz-form-label>
              <nz-form-control [nzSpan]="18">
                <input nz-input formControlName="name" placeholder="请输入连接名称">
              </nz-form-control>
            </nz-form-item>

            <nz-form-item>
              <nz-form-label [nzSpan]="6" nzRequired>PLC类型</nz-form-label>
              <nz-form-control [nzSpan]="18">
                <nz-select formControlName="plcType" nzPlaceHolder="选择PLC类型">
                  <nz-option *ngFor="let type of plcTypes" 
                             [nzValue]="type" 
                             [nzLabel]="getPlcTypeLabel(type)">
                  </nz-option>
                </nz-select>
              </nz-form-control>
            </nz-form-item>

            <nz-form-item>
              <nz-form-label [nzSpan]="6" nzRequired>IP地址</nz-form-label>
              <nz-form-control [nzSpan]="18">
                <input nz-input formControlName="ipAddress" placeholder="192.168.1.100">
              </nz-form-control>
            </nz-form-item>

            <nz-form-item>
              <nz-form-label [nzSpan]="6" nzRequired>端口</nz-form-label>
              <nz-form-control [nzSpan]="18">
                <nz-input-number formControlName="port" [nzMin]="1" [nzMax]="65535" 
                                 nzPlaceHolder="502" style="width: 100%;">
                </nz-input-number>
              </nz-form-control>
            </nz-form-item>

            <nz-form-item>
              <nz-form-label [nzSpan]="6">超时时间(ms)</nz-form-label>
              <nz-form-control [nzSpan]="18">
                <nz-input-number formControlName="timeout" [nzMin]="1000" [nzMax]="30000" 
                                 nzPlaceHolder="5000" style="width: 100%;">
                </nz-input-number>
              </nz-form-control>
            </nz-form-item>

            <nz-form-item>
              <nz-form-label [nzSpan]="6">重试次数</nz-form-label>
              <nz-form-control [nzSpan]="18">
                <nz-input-number formControlName="retryCount" [nzMin]="0" [nzMax]="10" 
                                 nzPlaceHolder="3" style="width: 100%;">
                </nz-input-number>
              </nz-form-control>
            </nz-form-item>

            <nz-form-item>
              <nz-form-label [nzSpan]="6">PLC用途</nz-form-label>
              <nz-form-control [nzSpan]="18">
                <nz-switch formControlName="isTestPlc" 
                           nzCheckedChildren="测试PLC" 
                           nzUnCheckedChildren="被测PLC">
                </nz-switch>
              </nz-form-control>
            </nz-form-item>

            <nz-form-item>
              <nz-form-label [nzSpan]="6">描述</nz-form-label>
              <nz-form-control [nzSpan]="18">
                <textarea nz-input formControlName="description" 
                          placeholder="请输入描述信息" rows="3">
                </textarea>
              </nz-form-control>
            </nz-form-item>

            <nz-form-item>
              <nz-form-control [nzSpan]="24" style="text-align: right;">
                <nz-space>
                  <button *nzSpaceItem nz-button nzType="default" (click)="closeConnectionModal()">
                    取消
                  </button>
                  <button *nzSpaceItem nz-button nzType="primary" 
                          [nzLoading]="saving" 
                          [disabled]="connectionForm.invalid">
                    {{ isEditingConnection ? '更新' : '添加' }}
                  </button>
                </nz-space>
              </nz-form-control>
            </nz-form-item>
          </form>
        </ng-container>
      </nz-modal>

      <!-- 通道配置模态框 -->
      <nz-modal 
        [(nzVisible)]="isChannelModalVisible"
        [nzTitle]="isEditingChannel ? '编辑通道配置' : '添加通道配置'"
        [nzFooter]="null"
        (nzOnCancel)="closeChannelModal()">
        
        <ng-container *nzModalContent>
          <form nz-form [formGroup]="channelForm" (ngSubmit)="saveChannel()">
            <nz-form-item>
              <nz-form-label [nzSpan]="6" nzRequired>通道位号</nz-form-label>
              <nz-form-control [nzSpan]="18">
                <input nz-input formControlName="channelAddress" placeholder="AI1_1">
              </nz-form-control>
            </nz-form-item>

            <nz-form-item>
              <nz-form-label [nzSpan]="6" nzRequired>通道类型</nz-form-label>
              <nz-form-control [nzSpan]="18">
                <nz-select formControlName="channelType" nzPlaceHolder="选择通道类型">
                  <nz-option *ngFor="let type of channelTypes" 
                             [nzValue]="type" 
                             [nzLabel]="getChannelTypeLabel(type)">
                  </nz-option>
                </nz-select>
              </nz-form-control>
            </nz-form-item>

            <nz-form-item>
              <nz-form-label [nzSpan]="6" nzRequired>通讯地址</nz-form-label>
              <nz-form-control [nzSpan]="18">
                <input nz-input formControlName="communicationAddress" placeholder="40101">
              </nz-form-control>
            </nz-form-item>

            <nz-form-item>
              <nz-form-label [nzSpan]="6" nzRequired>供电类型</nz-form-label>
              <nz-form-control [nzSpan]="18">
                <nz-select formControlName="powerSupplyType" nzPlaceHolder="选择供电类型">
                  <nz-option nzValue="24V DC" nzLabel="24V DC"></nz-option>
                  <nz-option nzValue="无源" nzLabel="无源"></nz-option>
                  <nz-option nzValue="220V AC" nzLabel="220V AC"></nz-option>
                  <nz-option nzValue="其他" nzLabel="其他"></nz-option>
                </nz-select>
              </nz-form-control>
            </nz-form-item>

            <nz-form-item>
              <nz-form-label [nzSpan]="6">描述</nz-form-label>
              <nz-form-control [nzSpan]="18">
                <textarea nz-input formControlName="description" 
                          placeholder="请输入描述信息" rows="3">
                </textarea>
              </nz-form-control>
            </nz-form-item>

            <nz-form-item>
              <nz-form-label [nzSpan]="6">启用状态</nz-form-label>
              <nz-form-control [nzSpan]="18">
                <nz-switch formControlName="isEnabled" 
                           nzCheckedChildren="启用" 
                           nzUnCheckedChildren="禁用">
                </nz-switch>
              </nz-form-control>
            </nz-form-item>

            <nz-form-item>
              <nz-form-control [nzSpan]="24" style="text-align: right;">
                <nz-space>
                  <button *nzSpaceItem nz-button nzType="default" (click)="closeChannelModal()">
                    取消
                  </button>
                  <button *nzSpaceItem nz-button nzType="primary" 
                          [nzLoading]="saving" 
                          [disabled]="channelForm.invalid">
                    {{ isEditingChannel ? '更新' : '添加' }}
                  </button>
                </nz-space>
              </nz-form-control>
            </nz-form-item>
          </form>
        </ng-container>
      </nz-modal>
    </div>
  `,
  styles: [`
    .test-plc-config-container {
      padding: 24px;
      background: #f5f5f5;
      min-height: 100vh;
    }

    .page-header {
      margin-bottom: 24px;
    }

    .page-header h2 {
      margin: 0;
      color: #1890ff;
      font-size: 24px;
      font-weight: 600;
    }

    .page-header h2 span {
      margin-right: 8px;
    }

    .page-header p {
      margin: 8px 0 0 0;
      color: #666;
      font-size: 14px;
    }

    .config-section {
      margin-bottom: 32px;
      border-radius: 8px;
      box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
    }

    .config-section:last-child {
      margin-bottom: 0;
    }

    .action-buttons {
      margin: 24px 0;
    }

    .filter-section {
      margin: 16px 0;
      padding: 16px;
      background: #fafafa;
      border-radius: 6px;
    }

    .filter-info {
      color: #666;
      font-size: 12px;
      line-height: 32px;
    }

    .ant-table {
      margin-top: 16px;
    }

    .ant-form-item {
      margin-bottom: 16px;
    }

    /* 响应式设计 */
    @media (max-width: 768px) {
      .test-plc-config-container {
        padding: 16px;
      }
      
      .page-header h2 {
        font-size: 20px;
      }
      
      .config-section {
        margin-bottom: 24px;
      }
    }
  `]
})
export class TestPlcConfigComponent implements OnInit, OnDestroy {
  
  // 数据属性
  testPlcChannels: TestPlcChannelConfig[] = [];
  filteredChannels: TestPlcChannelConfig[] = [];
  plcConnections: PlcConnectionConfig[] = [];
  
  // 界面状态
  loading = false;
  saving = false;
  
  // 模态框状态
  isConnectionModalVisible = false;
  isChannelModalVisible = false;
  isEditingConnection = false;
  isEditingChannel = false;
  
  // 筛选和搜索
  searchTerm = '';
  selectedChannelType: TestPlcChannelType | '' = '';
  
  // 表单
  connectionForm!: FormGroup;
  channelForm!: FormGroup;
  editingConnection: PlcConnectionConfig | null = null;
  editingChannel: TestPlcChannelConfig | null = null;
  
  // 枚举数据
  channelTypes = [
    TestPlcChannelType.AI,
    TestPlcChannelType.AO,
    TestPlcChannelType.DI,
    TestPlcChannelType.DO,
    TestPlcChannelType.AINone,
    TestPlcChannelType.AONone,
    TestPlcChannelType.DINone,
    TestPlcChannelType.DONone
  ];
  plcTypes = Object.values(PlcType);
  
  // 测试连接状态
  testingConnections = new Set<string>();

  private subscriptions: Subscription[] = [];

  constructor(
    private testPlcConfigService: TestPlcConfigService,
    private message: NzMessageService,
    private fb: FormBuilder
  ) {
    this.initializeForms();
  }

  ngOnInit(): void {
    this.loadData();
  }

  ngOnDestroy(): void {
    this.subscriptions.forEach(sub => sub.unsubscribe());
  }

  // ============================================================================
  // 初始化方法
  // ============================================================================

  private initializeForms(): void {
    this.connectionForm = this.fb.group({
      name: ['', [Validators.required]],
      plcType: [PlcType.ModbusTcp, [Validators.required]],
      ipAddress: ['', [Validators.required]],
      port: [502, [Validators.required, Validators.min(1), Validators.max(65535)]],
      timeout: [5000, [Validators.min(1000), Validators.max(30000)]],
      retryCount: [3, [Validators.min(0), Validators.max(10)]],
      isTestPlc: [true],
      description: [''],
      isEnabled: [true]
    });

    this.channelForm = this.fb.group({
      channelAddress: ['', [Validators.required]],
      channelType: ['', [Validators.required]],
      communicationAddress: ['', [Validators.required]],
      powerSupplyType: ['', [Validators.required]],
      description: [''],
      isEnabled: [true]
    });
  }

  private loadData(): void {
    this.loading = true;
    
    // 加载测试PLC通道配置
    const channelsSub = this.testPlcConfigService.getTestPlcChannels().subscribe({
      next: (channels) => {
        console.log('加载的通道数据:', channels.slice(0, 3)); // 打印前3个数据样本
        this.testPlcChannels = channels;
        this.filterChannels();
        this.loading = false;
      },
      error: (error) => {
        console.error('加载测试PLC通道配置失败:', error);
        this.message.error('加载测试PLC通道配置失败');
        this.loading = false;
      }
    });

    // 加载PLC连接配置
    const connectionsSub = this.testPlcConfigService.getPlcConnections().subscribe({
      next: (connections) => {
        this.plcConnections = connections;
      },
      error: (error) => {
        console.error('加载PLC连接配置失败:', error);
        this.message.error('加载PLC连接配置失败');
      }
    });

    this.subscriptions.push(channelsSub, connectionsSub);
  }

  // ============================================================================
  // 筛选和搜索方法
  // ============================================================================

  filterChannels(): void {
    let filtered = [...this.testPlcChannels];

    // 按通道类型筛选
    if (this.selectedChannelType !== '') {
      // 确保类型比较正确 - 将选中的类型转换为数字进行比较
      const selectedType = Number(this.selectedChannelType);
      filtered = filtered.filter(channel => {
        const channelType = Number(channel.channelType);
        return channelType === selectedType;
      });
    }

    // 按搜索词筛选
    if (this.searchTerm.trim()) {
      const term = this.searchTerm.toLowerCase();
      filtered = filtered.filter(channel => 
        channel.channelAddress.toLowerCase().includes(term) ||
        channel.communicationAddress.toLowerCase().includes(term) ||
        (channel.description && channel.description.toLowerCase().includes(term))
      );
    }

    this.filteredChannels = filtered;
    console.log('筛选结果:', {
      selectedType: this.selectedChannelType,
      totalChannels: this.testPlcChannels.length,
      filteredChannels: filtered.length,
      sampleChannel: filtered[0]
    });
  }

  refreshChannels(): void {
    this.loadData();
    this.message.info('正在刷新通道列表...');
  }

  // ============================================================================
  // PLC连接管理方法
  // ============================================================================

  showAddConnectionModal(): void {
    this.isEditingConnection = false;
    this.editingConnection = null;
    this.connectionForm.reset({
      plcType: PlcType.ModbusTcp,
      port: 502,
      timeout: 5000,
      retryCount: 3,
      isTestPlc: true,
      isEnabled: true
    });
    this.isConnectionModalVisible = true;
  }

  editConnection(connection: PlcConnectionConfig): void {
    this.isEditingConnection = true;
    this.editingConnection = connection;
    this.connectionForm.patchValue(connection);
    this.isConnectionModalVisible = true;
  }

  closeConnectionModal(): void {
    this.isConnectionModalVisible = false;
    this.isEditingConnection = false;
    this.editingConnection = null;
    this.connectionForm.reset();
  }

  saveConnection(): void {
    if (this.connectionForm.invalid) return;

    this.saving = true;
    const formValue = this.connectionForm.value;
    
    const connectionData: PlcConnectionConfig = {
      id: this.isEditingConnection ? this.editingConnection!.id : this.generateId(),
      ...formValue,
      connectionStatus: ConnectionStatus.Disconnected
    };

    this.testPlcConfigService.savePlcConnection(connectionData).subscribe({
      next: () => {
        this.message.success(this.isEditingConnection ? 'PLC连接配置更新成功' : 'PLC连接配置添加成功');
        this.closeConnectionModal();
        this.loadData();
        this.saving = false;
      },
      error: (error) => {
        console.error('保存PLC连接配置失败:', error);
        this.message.error('保存PLC连接配置失败');
        this.saving = false;
      }
    });
  }

  testConnection(connectionId: string): void {
    this.testingConnections.add(connectionId);
    
    this.testPlcConfigService.testPlcConnection(connectionId).subscribe({
      next: (response: TestPlcConnectionResponse) => {
        const connection = this.plcConnections.find(c => c.id === connectionId);
        if (connection) {
          connection.connectionStatus = response.success ? ConnectionStatus.Connected : ConnectionStatus.Error;
        }

        if (response.success) {
          this.message.success(response.message || 'PLC连接测试成功');
        } else {
          this.message.error(response.message || 'PLC连接测试失败');
        }
        
        this.testingConnections.delete(connectionId);
        this.plcConnections = [...this.plcConnections];
      },
      error: (err) => {
        console.error('测试PLC连接失败:', err);
        this.message.error('测试PLC连接失败: 调用服务时发生错误');
        
        const connection = this.plcConnections.find(c => c.id === connectionId);
        if (connection) {
          connection.connectionStatus = ConnectionStatus.Error;
        }
        this.testingConnections.delete(connectionId);
        this.plcConnections = [...this.plcConnections];
      }
    });
  }

  // ============================================================================
  // 通道配置管理方法
  // ============================================================================

  showAddChannelModal(): void {
    this.isEditingChannel = false;
    this.editingChannel = null;
    this.channelForm.reset({
      isEnabled: true
    });
    this.isChannelModalVisible = true;
  }

  editChannel(channel: TestPlcChannelConfig): void {
    this.isEditingChannel = true;
    this.editingChannel = channel;
    this.channelForm.patchValue(channel);
    this.isChannelModalVisible = true;
  }

  closeChannelModal(): void {
    this.isChannelModalVisible = false;
    this.isEditingChannel = false;
    this.editingChannel = null;
    this.channelForm.reset();
  }

  saveChannel(): void {
    if (this.channelForm.invalid) {
      this.message.error('请填写所有必填字段');
      return;
    }

    // 防止重复提交
    if (this.saving) {
      console.log('正在保存中，忽略重复请求');
      return;
    }

    try {
      this.saving = true;
      const formValue = this.channelForm.value;
      
      const channelData: TestPlcChannelConfig = {
        id: this.isEditingChannel ? this.editingChannel!.id : undefined,
        channelAddress: formValue.channelAddress,
        channelType: formValue.channelType,
        communicationAddress: formValue.communicationAddress,
        powerSupplyType: formValue.powerSupplyType,
        description: formValue.description || '',
        isEnabled: formValue.isEnabled
      };

      console.log('保存通道配置数据:', channelData);
      console.log('当前编辑状态:', {
        isEditingChannel: this.isEditingChannel,
        editingChannel: this.editingChannel
      });

      // 添加超时处理
      const saveOperation = this.testPlcConfigService.saveTestPlcChannel(channelData);
      
      // 设置30秒超时
      const timeoutPromise = new Promise((_, reject) => {
        setTimeout(() => reject(new Error('保存操作超时')), 30000);
      });

      Promise.race([
        saveOperation.toPromise(),
        timeoutPromise
      ]).then((savedChannel) => {
        console.log('通道配置保存成功，返回数据:', savedChannel);
        this.message.success(this.isEditingChannel ? '通道配置更新成功' : '通道配置添加成功');
        this.closeChannelModal();
        
        // 延迟刷新数据，避免立即操作导致问题
        setTimeout(() => {
          this.loadData();
        }, 500);
        
        this.saving = false;
      }).catch((error) => {
        console.error('保存通道配置失败:', error);
        console.error('错误类型:', typeof error);
        console.error('错误内容:', JSON.stringify(error, null, 2));
        
        let errorMessage = '保存通道配置失败';
        if (error && typeof error === 'string') {
          errorMessage += `: ${error}`;
        } else if (error && error.message) {
          errorMessage += `: ${error.message}`;
        }
        
        this.message.error(errorMessage);
        this.saving = false;
      });
    } catch (error) {
      console.error('saveChannel方法执行异常:', error);
      this.message.error('保存操作执行异常');
      this.saving = false;
    }
  }

  deleteChannel(channelId: string): void {
    this.testPlcConfigService.deleteTestPlcChannel(channelId).subscribe({
      next: () => {
        this.message.success('通道配置删除成功');
        this.loadData();
      },
      error: (error) => {
        console.error('删除通道配置失败:', error);
        this.message.error('删除通道配置失败');
      }
    });
  }

  // ============================================================================
  // 辅助方法
  // ============================================================================

  getChannelTypeLabel(type: TestPlcChannelType | number): string {
    // 确保类型转换正确
    let channelType: TestPlcChannelType;
    
    if (typeof type === 'string') {
      // 如果是字符串，尝试解析为数字
      channelType = parseInt(type) as TestPlcChannelType;
    } else if (typeof type === 'number') {
      channelType = type as TestPlcChannelType;
    } else {
      channelType = type;
    }
    
    // 调试日志
    console.log('通道类型转换:', { 
      原始类型: type, 
      转换后类型: channelType, 
      标签: TestPlcChannelTypeLabels[channelType] 
    });
    
    return TestPlcChannelTypeLabels[channelType] || `未知类型(${type})`;
  }

  getPlcTypeLabel(type: PlcType): string {
    return PlcTypeLabels[type] || '未知类型';
  }

  getConnectionStatusLabel(status: ConnectionStatus): string {
    return ConnectionStatusLabels[status] || '未知状态';
  }

  getChannelTypeColor(type: TestPlcChannelType | number): string {
    // 确保类型转换正确
    let channelType: TestPlcChannelType;
    
    if (typeof type === 'string') {
      channelType = parseInt(type) as TestPlcChannelType;
    } else if (typeof type === 'number') {
      channelType = type as TestPlcChannelType;
    } else {
      channelType = type;
    }
    
    return getChannelTypeColor(channelType);
  }

  getConnectionStatusColor(status: ConnectionStatus): string {
    return getConnectionStatusColor(status);
  }

  private generateId(): string {
    return Date.now().toString(36) + Math.random().toString(36).substr(2);
  }
} 