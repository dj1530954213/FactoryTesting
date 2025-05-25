import { Component, OnInit, OnDestroy } from '@angular/core';
import { CommonModule } from '@angular/common';
import { FormsModule, ReactiveFormsModule, FormBuilder, FormGroup, Validators } from '@angular/forms';
import { Subscription } from 'rxjs';
import { TauriApiService } from '../../services/tauri-api.service';
import { 
  ChannelPointDefinition, 
  ModuleType, 
  PointDataType,
  MODULE_TYPE_LABELS,
  ApiError
} from '../../models';

@Component({
  selector: 'app-channel-configuration',
  standalone: true,
  imports: [CommonModule, FormsModule, ReactiveFormsModule],
  template: `
    <div class="channel-config-container">
      <!-- 页面标题 -->
      <div class="page-header">
        <h1 class="page-title">通道配置管理</h1>
        <p class="page-subtitle">配置和管理测试通道参数</p>
        <div class="header-actions">
          <button class="btn-primary" (click)="showAddForm()">
            <i class="icon-add"></i>
            添加通道
          </button>
          <button class="btn-secondary" (click)="refreshChannels()">
            <i class="icon-refresh"></i>
            刷新
          </button>
        </div>
      </div>

      <!-- 搜索和筛选 -->
      <div class="filter-section">
        <div class="search-box">
          <input 
            type="text" 
            placeholder="搜索通道位号、变量名或描述..." 
            [(ngModel)]="searchTerm"
            (input)="filterChannels()"
            class="search-input">
          <i class="search-icon">🔍</i>
        </div>
        
        <div class="filter-controls">
          <select [(ngModel)]="selectedModuleType" (change)="filterChannels()" class="filter-select">
            <option value="">所有模块类型</option>
            <option *ngFor="let type of moduleTypes" [value]="type">
              {{getModuleTypeLabel(type)}}
            </option>
          </select>
          
          <select [(ngModel)]="selectedDataType" (change)="filterChannels()" class="filter-select">
            <option value="">所有数据类型</option>
            <option *ngFor="let type of dataTypes" [value]="type">{{type}}</option>
          </select>
        </div>
      </div>

      <!-- 通道列表 -->
      <div class="channels-section">
        <div class="section-header">
          <h2>通道列表 ({{filteredChannels.length}})</h2>
          <div class="view-controls">
            <button 
              class="view-btn" 
              [class.active]="viewMode === 'grid'"
              (click)="viewMode = 'grid'">
              <i class="icon-grid">⊞</i>
            </button>
            <button 
              class="view-btn" 
              [class.active]="viewMode === 'table'"
              (click)="viewMode = 'table'">
              <i class="icon-table">☰</i>
            </button>
          </div>
        </div>

        <!-- 网格视图 -->
        <div *ngIf="viewMode === 'grid'" class="channels-grid">
          <div *ngFor="let channel of filteredChannels" class="channel-card">
            <div class="card-header">
              <div class="channel-info">
                <h3 class="channel-tag">{{channel.tag}}</h3>
                <span class="module-type" [class]="'type-' + channel.module_type.toLowerCase()">
                  {{getModuleTypeLabel(channel.module_type)}}
                </span>
              </div>
              <div class="card-actions">
                <button class="btn-icon" (click)="editChannel(channel)" title="编辑">
                  <i class="icon-edit">✏️</i>
                </button>
                <button class="btn-icon danger" (click)="deleteChannel(channel)" title="删除">
                  <i class="icon-delete">🗑️</i>
                </button>
              </div>
            </div>
            
            <div class="card-content">
              <div class="info-row">
                <span class="label">变量名:</span>
                <span class="value">{{channel.variable_name}}</span>
              </div>
              <div class="info-row">
                <span class="label">描述:</span>
                <span class="value">{{channel.variable_description}}</span>
              </div>
              <div class="info-row">
                <span class="label">站点:</span>
                <span class="value">{{channel.station_name}}</span>
              </div>
              <div class="info-row">
                <span class="label">模块:</span>
                <span class="value">{{channel.module_name}}</span>
              </div>
              <div class="info-row">
                <span class="label">PLC地址:</span>
                <span class="value code">{{channel.plc_communication_address}}</span>
              </div>
              <div class="info-row" *ngIf="channel.range_lower_limit !== undefined && channel.range_upper_limit !== undefined">
                <span class="label">量程:</span>
                <span class="value">{{channel.range_lower_limit}} ~ {{channel.range_upper_limit}} {{channel.engineering_unit || ''}}</span>
              </div>
            </div>
          </div>
        </div>

        <!-- 表格视图 -->
        <div *ngIf="viewMode === 'table'" class="channels-table-container">
          <table class="channels-table">
            <thead>
              <tr>
                <th>位号</th>
                <th>变量名</th>
                <th>描述</th>
                <th>模块类型</th>
                <th>站点</th>
                <th>模块</th>
                <th>PLC地址</th>
                <th>数据类型</th>
                <th>操作</th>
              </tr>
            </thead>
            <tbody>
              <tr *ngFor="let channel of filteredChannels">
                <td class="tag-cell">{{channel.tag}}</td>
                <td>{{channel.variable_name}}</td>
                <td class="description-cell">{{channel.variable_description}}</td>
                <td>
                  <span class="module-type-badge" [class]="'type-' + channel.module_type.toLowerCase()">
                    {{getModuleTypeLabel(channel.module_type)}}
                  </span>
                </td>
                <td>{{channel.station_name}}</td>
                <td>{{channel.module_name}}</td>
                <td class="code">{{channel.plc_communication_address}}</td>
                <td>{{channel.data_type}}</td>
                <td class="actions-cell">
                  <button class="btn-small" (click)="editChannel(channel)">编辑</button>
                  <button class="btn-small danger" (click)="deleteChannel(channel)">删除</button>
                </td>
              </tr>
            </tbody>
          </table>
        </div>

        <!-- 空状态 -->
        <div *ngIf="filteredChannels.length === 0" class="empty-state">
          <div class="empty-icon">📊</div>
          <h3>{{channels.length === 0 ? '暂无通道配置' : '未找到匹配的通道'}}</h3>
          <p>{{channels.length === 0 ? '开始添加您的第一个测试通道' : '尝试调整搜索条件或筛选器'}}</p>
          <button *ngIf="channels.length === 0" class="btn-primary" (click)="showAddForm()">添加通道</button>
        </div>
      </div>

      <!-- 添加/编辑表单模态框 -->
      <div *ngIf="showForm" class="modal-overlay" (click)="hideForm()">
        <div class="modal-content" (click)="$event.stopPropagation()">
          <div class="modal-header">
            <h2>{{isEditing ? '编辑通道' : '添加通道'}}</h2>
            <button class="btn-close" (click)="hideForm()">×</button>
          </div>
          
          <form [formGroup]="channelForm" (ngSubmit)="saveChannel()" class="channel-form">
            <div class="form-grid">
              <!-- 基本信息 -->
              <div class="form-section">
                <h3>基本信息</h3>
                
                <div class="form-group">
                  <label for="tag">位号 *</label>
                  <input 
                    id="tag" 
                    type="text" 
                    formControlName="tag" 
                    placeholder="例如: AI001"
                    class="form-input">
                  <div *ngIf="channelForm.get('tag')?.invalid && channelForm.get('tag')?.touched" class="error-message">
                    位号为必填项
                  </div>
                </div>

                <div class="form-group">
                  <label for="variable_name">变量名 *</label>
                  <input 
                    id="variable_name" 
                    type="text" 
                    formControlName="variable_name" 
                    placeholder="例如: Temperature_1"
                    class="form-input">
                  <div *ngIf="channelForm.get('variable_name')?.invalid && channelForm.get('variable_name')?.touched" class="error-message">
                    变量名为必填项
                  </div>
                </div>

                <div class="form-group">
                  <label for="variable_description">描述 *</label>
                  <input 
                    id="variable_description" 
                    type="text" 
                    formControlName="variable_description" 
                    placeholder="例如: 反应器温度"
                    class="form-input">
                  <div *ngIf="channelForm.get('variable_description')?.invalid && channelForm.get('variable_description')?.touched" class="error-message">
                    描述为必填项
                  </div>
                </div>
              </div>

              <!-- 模块信息 -->
              <div class="form-section">
                <h3>模块信息</h3>
                
                <div class="form-group">
                  <label for="station_name">站点名称 *</label>
                  <input 
                    id="station_name" 
                    type="text" 
                    formControlName="station_name" 
                    placeholder="例如: Station1"
                    class="form-input">
                </div>

                <div class="form-group">
                  <label for="module_name">模块名称 *</label>
                  <input 
                    id="module_name" 
                    type="text" 
                    formControlName="module_name" 
                    placeholder="例如: Module1"
                    class="form-input">
                </div>

                <div class="form-group">
                  <label for="module_type">模块类型 *</label>
                  <select id="module_type" formControlName="module_type" class="form-select">
                    <option value="">请选择模块类型</option>
                    <option *ngFor="let type of moduleTypes" [value]="type">
                      {{getModuleTypeLabel(type)}}
                    </option>
                  </select>
                </div>

                <div class="form-group">
                  <label for="channel_tag_in_module">模块内通道标识 *</label>
                  <input 
                    id="channel_tag_in_module" 
                    type="text" 
                    formControlName="channel_tag_in_module" 
                    placeholder="例如: CH01"
                    class="form-input">
                </div>

                <div class="form-group">
                  <label for="data_type">数据类型 *</label>
                  <select id="data_type" formControlName="data_type" class="form-select">
                    <option value="">请选择数据类型</option>
                    <option *ngFor="let type of dataTypes" [value]="type">{{type}}</option>
                  </select>
                </div>
              </div>

              <!-- PLC通信 -->
              <div class="form-section">
                <h3>PLC通信</h3>
                
                <div class="form-group">
                  <label for="plc_communication_address">PLC通信地址 *</label>
                  <input 
                    id="plc_communication_address" 
                    type="text" 
                    formControlName="plc_communication_address" 
                    placeholder="例如: DB1.DBD0"
                    class="form-input">
                </div>

                <div class="form-group">
                  <label for="test_rig_plc_address">测试台PLC地址</label>
                  <input 
                    id="test_rig_plc_address" 
                    type="text" 
                    formControlName="test_rig_plc_address" 
                    placeholder="例如: DB2.DBD0"
                    class="form-input">
                </div>
              </div>

              <!-- 量程设置 -->
              <div class="form-section" *ngIf="isAnalogModule()">
                <h3>量程设置</h3>
                
                <div class="form-row">
                  <div class="form-group">
                    <label for="range_lower_limit">下限值</label>
                    <input 
                      id="range_lower_limit" 
                      type="number" 
                      formControlName="range_lower_limit" 
                      placeholder="0"
                      class="form-input">
                  </div>

                  <div class="form-group">
                    <label for="range_upper_limit">上限值</label>
                    <input 
                      id="range_upper_limit" 
                      type="number" 
                      formControlName="range_upper_limit" 
                      placeholder="100"
                      class="form-input">
                  </div>

                  <div class="form-group">
                    <label for="engineering_unit">工程单位</label>
                    <input 
                      id="engineering_unit" 
                      type="text" 
                      formControlName="engineering_unit" 
                      placeholder="例如: °C"
                      class="form-input">
                  </div>
                </div>
              </div>
            </div>

            <div class="form-actions">
              <button type="button" class="btn-secondary" (click)="hideForm()">取消</button>
              <button type="submit" class="btn-primary" [disabled]="channelForm.invalid || saving">
                {{saving ? '保存中...' : (isEditing ? '更新' : '添加')}}
              </button>
            </div>
          </form>
        </div>
      </div>

      <!-- 加载状态 -->
      <div *ngIf="loading" class="loading-overlay">
        <div class="loading-spinner">
          <div class="spinner"></div>
          <p>加载中...</p>
        </div>
      </div>
    </div>
  `,
  styleUrls: ['./channel-configuration.component.css']
})
export class ChannelConfigurationComponent implements OnInit, OnDestroy {
  // 数据属性
  channels: ChannelPointDefinition[] = [];
  filteredChannels: ChannelPointDefinition[] = [];
  
  // 界面状态
  loading = false;
  saving = false;
  showForm = false;
  isEditing = false;
  viewMode: 'grid' | 'table' = 'grid';
  
  // 搜索和筛选
  searchTerm = '';
  selectedModuleType = '';
  selectedDataType = '';
  
  // 表单
  channelForm: FormGroup;
  editingChannel: ChannelPointDefinition | null = null;
  
  // 枚举数据
  moduleTypes = Object.values(ModuleType);
  dataTypes = Object.values(PointDataType);
  
  private subscriptions: Subscription[] = [];

  constructor(
    private tauriApi: TauriApiService,
    private fb: FormBuilder
  ) {
    this.channelForm = this.createForm();
  }

  ngOnInit(): void {
    this.loadChannels();
  }

  ngOnDestroy(): void {
    this.subscriptions.forEach(sub => sub.unsubscribe());
  }

  /**
   * 创建表单
   */
  private createForm(): FormGroup {
    return this.fb.group({
      tag: ['', Validators.required],
      variable_name: ['', Validators.required],
      variable_description: ['', Validators.required],
      station_name: ['', Validators.required],
      module_name: ['', Validators.required],
      module_type: ['', Validators.required],
      channel_tag_in_module: ['', Validators.required],
      data_type: ['', Validators.required],
      power_supply_type: [''],
      wire_system: [''],
      plc_communication_address: ['', Validators.required],
      test_rig_plc_address: [''],
      range_lower_limit: [null],
      range_upper_limit: [null],
      engineering_unit: ['']
    });
  }

  /**
   * 加载通道列表
   */
  loadChannels(): void {
    this.loading = true;
    const sub = this.tauriApi.getAllChannelDefinitions().subscribe({
      next: (channels) => {
        this.channels = channels;
        this.filterChannels();
        this.loading = false;
      },
      error: (error: ApiError) => {
        console.error('加载通道失败:', error);
        this.loading = false;
        // TODO: 显示错误提示
      }
    });
    this.subscriptions.push(sub);
  }

  /**
   * 刷新通道列表
   */
  refreshChannels(): void {
    this.loadChannels();
  }

  /**
   * 筛选通道
   */
  filterChannels(): void {
    let filtered = [...this.channels];

    // 搜索筛选
    if (this.searchTerm.trim()) {
      const term = this.searchTerm.toLowerCase();
      filtered = filtered.filter(channel => 
        channel.tag.toLowerCase().includes(term) ||
        channel.variable_name.toLowerCase().includes(term) ||
        channel.variable_description.toLowerCase().includes(term)
      );
    }

    // 模块类型筛选
    if (this.selectedModuleType) {
      filtered = filtered.filter(channel => channel.module_type === this.selectedModuleType);
    }

    // 数据类型筛选
    if (this.selectedDataType) {
      filtered = filtered.filter(channel => channel.data_type === this.selectedDataType);
    }

    this.filteredChannels = filtered;
  }

  /**
   * 显示添加表单
   */
  showAddForm(): void {
    this.isEditing = false;
    this.editingChannel = null;
    this.channelForm.reset();
    this.showForm = true;
  }

  /**
   * 编辑通道
   */
  editChannel(channel: ChannelPointDefinition): void {
    this.isEditing = true;
    this.editingChannel = channel;
    this.channelForm.patchValue(channel);
    this.showForm = true;
  }

  /**
   * 隐藏表单
   */
  hideForm(): void {
    this.showForm = false;
    this.isEditing = false;
    this.editingChannel = null;
    this.channelForm.reset();
  }

  /**
   * 保存通道
   */
  saveChannel(): void {
    if (this.channelForm.invalid) return;

    this.saving = true;
    const formValue = this.channelForm.value;
    
    const channelData: ChannelPointDefinition = {
      id: this.isEditing ? this.editingChannel!.id : this.generateId(),
      ...formValue
    };

    const sub = this.tauriApi.saveChannelDefinition(channelData).subscribe({
      next: () => {
        this.saving = false;
        this.hideForm();
        this.loadChannels();
        // TODO: 显示成功提示
      },
      error: (error: ApiError) => {
        console.error('保存通道失败:', error);
        this.saving = false;
        // TODO: 显示错误提示
      }
    });
    this.subscriptions.push(sub);
  }

  /**
   * 删除通道
   */
  deleteChannel(channel: ChannelPointDefinition): void {
    if (!confirm(`确定要删除通道 "${channel.tag}" 吗？`)) return;

    const sub = this.tauriApi.deleteChannelDefinition(channel.id).subscribe({
      next: () => {
        this.loadChannels();
        // TODO: 显示成功提示
      },
      error: (error: ApiError) => {
        console.error('删除通道失败:', error);
        // TODO: 显示错误提示
      }
    });
    this.subscriptions.push(sub);
  }

  /**
   * 获取模块类型标签
   */
  getModuleTypeLabel(type: ModuleType): string {
    return MODULE_TYPE_LABELS[type] || type;
  }

  /**
   * 检查是否为模拟量模块
   */
  isAnalogModule(): boolean {
    const moduleType = this.channelForm.get('module_type')?.value;
    return moduleType === ModuleType.AI || moduleType === ModuleType.AO;
  }

  /**
   * 生成ID
   */
  private generateId(): string {
    return 'channel_' + Date.now() + '_' + Math.random().toString(36).substr(2, 9);
  }
} 