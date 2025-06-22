import { Component, Input, Output, EventEmitter, OnInit, OnDestroy, OnChanges, SimpleChanges } from '@angular/core';
import { CommonModule } from '@angular/common';
import { FormBuilder, FormGroup, Validators, ReactiveFormsModule, FormsModule } from '@angular/forms';
import { Subscription } from 'rxjs';

// NG-ZORRO 组件
import { NzModalModule } from 'ng-zorro-antd/modal';
import { NzFormModule } from 'ng-zorro-antd/form';
import { NzInputModule } from 'ng-zorro-antd/input';
import { NzInputNumberModule } from 'ng-zorro-antd/input-number';
import { NzSelectModule } from 'ng-zorro-antd/select';
import { NzSwitchModule } from 'ng-zorro-antd/switch';
import { NzButtonModule } from 'ng-zorro-antd/button';
import { NzIconModule } from 'ng-zorro-antd/icon';
import { NzSpaceModule } from 'ng-zorro-antd/space';
import { NzCardModule } from 'ng-zorro-antd/card';
import { NzAlertModule } from 'ng-zorro-antd/alert';
import { NzSpinModule } from 'ng-zorro-antd/spin';
import { NzDividerModule } from 'ng-zorro-antd/divider';
import { NzMessageService } from 'ng-zorro-antd/message';
import { NzTagModule } from 'ng-zorro-antd/tag';
import { NzResultModule } from 'ng-zorro-antd/result';
import { NzTabsModule } from 'ng-zorro-antd/tabs';
import { NzGridModule } from 'ng-zorro-antd/grid';
import { NzDescriptionsModule } from 'ng-zorro-antd/descriptions';

// 项目类型和服务
import { PlcConnectionConfig, PlcType, ByteOrder, ConnectionStatus } from '../../models/test-plc-config.model';
import { TestPlcConfigService, AddressReadTestResponse } from '../../services/test-plc-config.service';

/**
 * PLC连接测试结果
 */
interface PlcTestResult {
  success: boolean;
  message: string;
  connectionTimeMs?: number;
}

/**
 * 地址读取测试结果
 */
interface AddressReadResult {
  address: string;
  dataType: 'bool' | 'float';
  success: boolean;
  value?: any;
  error?: string;
}

@Component({
  selector: 'app-plc-advanced-test-window',
  standalone: true,
  imports: [
    CommonModule,
    ReactiveFormsModule,
    FormsModule,
    NzModalModule,
    NzFormModule,
    NzInputModule,
    NzInputNumberModule,
    NzSelectModule,
    NzSwitchModule,
    NzButtonModule,
    NzIconModule,
    NzSpaceModule,
    NzCardModule,
    NzAlertModule,
    NzSpinModule,
    NzDividerModule,
    NzTagModule,
    NzResultModule,
    NzTabsModule,
    NzGridModule,
    NzDescriptionsModule
  ],
  template: `
    <nz-modal 
      [(nzVisible)]="visible"
      [nzTitle]="'PLC通讯参数配置与测试'"
      [nzWidth]="900"
      [nzFooter]="modalFooter"
      [nzMaskClosable]="false"
      (nzOnCancel)="closeModal()">
      
      <ng-container *nzModalContent>
        <div class="advanced-test-container">
          <div nz-row [nzGutter]="24">
            <!-- 左侧：参数配置 -->
            <div nz-col [nzSpan]="12">
              <nz-card nzTitle="参数配置" [nzBorderless]="true">
                <form nz-form [formGroup]="connectionForm" nzLayout="vertical">
                  <div nz-row [nzGutter]="16">
                    <div nz-col [nzSpan]="12">
                      <nz-form-item>
                        <nz-form-label nzRequired>IP地址</nz-form-label>
                        <nz-form-control nzErrorTip="请输入IP地址">
                          <input nz-input formControlName="ipAddress" placeholder="192.168.1.100">
                        </nz-form-control>
                      </nz-form-item>
                    </div>
                    <div nz-col [nzSpan]="12">
                      <nz-form-item>
                        <nz-form-label nzRequired>端口</nz-form-label>
                        <nz-form-control nzErrorTip="请输入端口号">
                          <nz-input-number formControlName="port" [nzMin]="1" [nzMax]="65535" style="width: 100%;"></nz-input-number>
                        </nz-form-control>
                      </nz-form-item>
                    </div>
                  </div>
    
                  <div nz-row [nzGutter]="16">
                    <div nz-col [nzSpan]="12">
                      <nz-form-item>
                        <nz-form-label>站地址</nz-form-label>
                        <nz-form-control>
                          <nz-input-number formControlName="slaveId" [nzMin]="1" [nzMax]="255" style="width: 100%;"></nz-input-number>
                        </nz-form-control>
                      </nz-form-item>
                    </div>
                    <div nz-col [nzSpan]="12">
                      <nz-form-item>
                        <nz-form-label>超时时间(ms)</nz-form-label>
                        <nz-form-control>
                          <nz-input-number formControlName="timeout" [nzMin]="1000" [nzMax]="30000" style="width: 100%;"></nz-input-number>
                        </nz-form-control>
                      </nz-form-item>
                    </div>
                  </div>
    
                  <div nz-row [nzGutter]="16">
                    <div nz-col [nzSpan]="12">
                      <nz-form-item>
                        <nz-form-label>重试次数</nz-form-label>
                        <nz-form-control>
                          <nz-input-number formControlName="retryCount" [nzMin]="0" [nzMax]="10" style="width: 100%;"></nz-input-number>
                        </nz-form-control>
                      </nz-form-item>
                    </div>
                    <div nz-col [nzSpan]="12">
                      <nz-form-item>
                        <nz-form-label>字节顺序</nz-form-label>
                        <nz-form-control>
                          <nz-select formControlName="byteOrder" nzPlaceHolder="选择字节顺序">
                            <nz-option nzValue="ABCD" nzLabel="ABCD (大端)"></nz-option>
                            <nz-option nzValue="CDAB" nzLabel="CDAB (小端)"></nz-option>
                            <nz-option nzValue="BADC" nzLabel="BADC (混合1)"></nz-option>
                            <nz-option nzValue="DCBA" nzLabel="DCBA (混合2)"></nz-option>
                          </nz-select>
                        </nz-form-control>
                      </nz-form-item>
                    </div>
                  </div>
    
                  <nz-form-item>
                    <nz-form-label>首地址从0开始</nz-form-label>
                    <nz-form-control>
                      <nz-switch formControlName="zeroBasedAddress" nzCheckedChildren="是" nzUnCheckedChildren="否"></nz-switch>
                    </nz-form-control>
                  </nz-form-item>
                </form>
              </nz-card>
            </div>

            <!-- 右侧：通讯测试 -->
            <div nz-col [nzSpan]="12">
              <nz-card nzTitle="通讯测试" [nzBorderless]="true">
                <!-- 基础连接测试 -->
                <div class="connection-test-section">
                  <div class="test-header">
                    <h4>基础连接测试</h4>
                    <button nz-button nzType="primary" 
                            [nzLoading]="testingConnection"
                            [disabled]="connectionForm.invalid"
                            (click)="testBasicConnection()">
                      <span nz-icon nzType="api" nzTheme="outline"></span>
                      测试连接
                    </button>
                  </div>
                  
                  <div *ngIf="connectionTestResult" class="test-result">
                    <nz-alert 
                      [nzType]="connectionTestResult.success ? 'success' : 'error'"
                      [nzMessage]="connectionTestResult.message"
                      [nzDescription]="connectionTestResult.connectionTimeMs ? 
                        '连接时间: ' + connectionTestResult.connectionTimeMs + 'ms' : null"
                      nzShowIcon>
                    </nz-alert>
                  </div>
                </div>
    
                <nz-divider></nz-divider>
    
                <!-- 地址读写测试区域 -->
                <div class="address-test-section">
                  <nz-descriptions nzTitle="地址读写测试" [nzColumn]="1" nzBordered [nzSize]="'small'">
                    <!-- Float 读取 -->
                    <nz-descriptions-item nzTitle="Float 读取">
                      <div class="test-item-content">
                        <nz-input-group nzCompact>
                          <input nz-input [(ngModel)]="testAddresses.float" placeholder="如: 40001" style="width: 100px;">
                          <button nz-button nzType="default" [nzLoading]="testingAddress.float" [disabled]="!testAddresses.float || connectionForm.invalid" (click)="testReadAddress('float')">读取</button>
                        </nz-input-group>
                        <div class="read-result-box" 
                             [class.is-success]="addressTestResults.float?.success === true" 
                             [class.is-error]="addressTestResults.float?.success === false">
                          <span *ngIf="addressTestResults.float?.success === true" class="value-text">
                            {{ addressTestResults.float?.value | number:'1.6-6' }}
                          </span>
                          <span *ngIf="addressTestResults.float?.success === false" class="error-text" [title]="addressTestResults.float?.error">
                            读取失败
                          </span>
                          <span *ngIf="addressTestResults.float === undefined" class="placeholder-text">---</span>
                        </div>
                      </div>
                    </nz-descriptions-item>

                    <!-- Bool 读取 -->
                    <nz-descriptions-item nzTitle="Bool 读取">
                      <div class="test-item-content">
                        <nz-input-group nzCompact>
                          <input nz-input [(ngModel)]="testAddresses.bool" placeholder="如: 00001" style="width: 100px;">
                          <button nz-button nzType="default" [nzLoading]="testingAddress.bool" [disabled]="!testAddresses.bool || connectionForm.invalid" (click)="testReadAddress('bool')">读取</button>
                        </nz-input-group>
                        <div class="read-result-box"
                             [class.is-success]="addressTestResults.bool?.success === true"
                             [class.is-error]="addressTestResults.bool?.success === false">
                          <nz-tag *ngIf="addressTestResults.bool?.success === true" [nzColor]="addressTestResults.bool?.value ? 'green' : 'red'">
                            {{ addressTestResults.bool?.value ? 'ON' : 'OFF' }}
                          </nz-tag>
                          <span *ngIf="addressTestResults.bool?.success === false" class="error-text" [title]="addressTestResults.bool?.error">
                            读取失败
                          </span>
                          <span *ngIf="addressTestResults.bool === undefined" class="placeholder-text">---</span>
                        </div>
                      </div>
                    </nz-descriptions-item>
                  </nz-descriptions>
                </div>
              </nz-card>
            </div>
          </div>
        </div>
      </ng-container>

      <ng-template #modalFooter>
        <button nz-button nzType="default" (click)="closeModal()">取消</button>
        <button nz-button nzType="primary" (click)="saveConfiguration()" [nzLoading]="savingConfiguration" [disabled]="connectionForm.invalid">
          <span nz-icon nzType="save" nzTheme="outline"></span>
          保存配置
        </button>
      </ng-template>
    </nz-modal>
  `,
  styles: [`
    .advanced-test-container {
      max-height: 70vh;
      overflow-y: auto;
    }

    nz-card {
      height: 100%;
    }

    nz-form-item {
      margin-bottom: 12px;
    }

    .connection-test-section {
      margin-bottom: 16px;
    }

    .test-header {
      display: flex;
      justify-content: space-between;
      align-items: center;
      margin-bottom: 16px;
    }
    
    .test-result {
      margin-top: 16px;
    }

    .address-test-section {
      margin-top: 16px;
    }

    .test-item-content {
      display: flex;
      align-items: center;
      justify-content: space-between;
      gap: 16px;
      width: 100%;
    }

    .read-result-box {
      width: 160px;
      height: 32px;
      border: 1px solid #d9d9d9;
      border-radius: 4px;
      display: flex;
      align-items: center;
      justify-content: center;
      background-color: #fafafa;
      font-family: 'Segoe UI', 'Helvetica Neue', Arial, sans-serif;
      font-weight: 500;
      transition: all 0.3s;
      overflow: hidden;
    }

    .read-result-box.is-success {
      border-color: #b7eb8f;
      background-color: #f6ffed;
    }
    
    .read-result-box .value-text {
      color: #389e0d;
      font-family: 'Courier New', Courier, monospace;
      font-size: 1.1em;
    }

    .read-result-box.is-error {
      border-color: #ffccc7;
      background-color: #fff2f0;
    }
    
    .read-result-box .error-text {
      color: #cf1322;
      font-size: 14px;
      white-space: nowrap;
      overflow: hidden;
      text-overflow: ellipsis;
      padding: 0 8px;
    }

    .read-result-box .placeholder-text {
      color: #aaa;
    }
  `]
})
export class PlcAdvancedTestWindowComponent implements OnInit, OnDestroy, OnChanges {
  @Input() visible = false;
  @Input() originalConnection: PlcConnectionConfig | null = null;
  @Output() visibleChange = new EventEmitter<boolean>();
  @Output() onConfigSaved = new EventEmitter<void>();

  // 表单
  connectionForm!: FormGroup;

  // 测试状态
  testingConnection = false;
  connectionTestResult: PlcTestResult | null = null;
  savingConfiguration = false;

  // 地址测试
  testAddresses = {
    bool: '00001',
    float: '40001'
  };

  testingAddress = {
    bool: false,
    float: false
  };

  addressTestResults: {
    bool?: AddressReadResult;
    float?: AddressReadResult;
  } = {};

  private subscriptions: Subscription[] = [];

  constructor(
    private fb: FormBuilder,
    private testPlcConfigService: TestPlcConfigService,
    private message: NzMessageService
  ) {
    this.initializeForm();
  }

  ngOnInit(): void {
    // 组件初始化时，如果已有传入数据，则加载
    this.loadConnectionData();
  }

  ngOnChanges(changes: SimpleChanges): void {
    // 监听 originalConnection 的变化，当父组件传入新的连接配置时，更新表单
    if (changes['originalConnection'] && !changes['originalConnection'].firstChange) {
      this.loadConnectionData();
    }
  }

  ngOnDestroy(): void {
    this.subscriptions.forEach(sub => sub.unsubscribe());
  }

  private initializeForm(): void {
    this.connectionForm = this.fb.group({
      id: [this.generateId()],
      name: ['临时连接', [Validators.required]],
      plcType: [PlcType.ModbusTcp, [Validators.required]],
      ipAddress: ['192.168.1.100', [Validators.required]],
      port: [502, [Validators.required, Validators.min(1), Validators.max(65535)]],
      slaveId: [1, [Validators.required, Validators.min(1), Validators.max(255)]],
      timeout: [5000, [Validators.required, Validators.min(1000), Validators.max(30000)]],
      retryCount: [3, [Validators.required, Validators.min(0), Validators.max(10)]],
      byteOrder: [ByteOrder.CDAB, [Validators.required]],
      zeroBasedAddress: [false, [Validators.required]],
      // 这些字段用于完整性，但在高级测试窗口中可能不直接编辑
      isTestPlc: [true],
      description: [''],
      isEnabled: [true],
      lastConnected: [null],
      connectionStatus: [ConnectionStatus.Disconnected]
    });
  }

  private loadConnectionData(): void {
    console.log('接收到的原始连接数据:', this.originalConnection);
    if (this.originalConnection) {
      // 使用传入的数据填充表单
      this.connectionForm.patchValue(this.originalConnection);
      this.message.info(`已加载PLC配置: ${this.originalConnection.name}`);
    } else {
      // 如果没有传入数据，则使用默认值初始化
      this.initializeForm();
      console.log('未传入连接数据，使用默认值初始化表单。');
    }
  }

  async testBasicConnection(): Promise<void> {
    if (this.connectionForm.invalid) {
      this.message.warning('请先完善PLC通讯参数配置');
      return;
    }

    this.testingConnection = true;
    this.connectionTestResult = null;

    try {
      // 创建临时连接配置用于测试
      const tempConfig: PlcConnectionConfig = {
        id: 'temp_test_' + Date.now(),
        name: '临时测试连接',
        plcType: PlcType.ModbusTcp,
        ...this.connectionForm.value,
        connectionStatus: ConnectionStatus.Disconnected,
        isTestPlc: true,
        isEnabled: true,
        retryCount: 3,
        description: '临时测试连接配置'
      };

      // 直接使用临时配置进行测试，而不是通过ID查找
      const result = await this.testPlcConfigService.testTempPlcConnection(tempConfig).toPromise();
      
      this.connectionTestResult = {
        success: result?.success || false,
        message: result?.message || '连接测试失败',
        connectionTimeMs: result?.connectionTimeMs
      };

      if (this.connectionTestResult.success) {
        this.message.success('PLC通讯连接测试成功！');
      } else {
        this.message.error('PLC通讯连接测试失败：' + this.connectionTestResult.message);
      }
    } catch (error) {
      console.error('连接测试失败:', error);
      this.connectionTestResult = {
        success: false,
        message: '连接测试失败：' + (error as any)?.message || '未知错误'
      };
      this.message.error('连接测试失败');
    } finally {
      this.testingConnection = false;
    }
  }

  async testReadAddress(dataType: 'bool' | 'float'): Promise<void> {
    if (this.connectionForm.invalid) {
      this.message.warning('请先完善PLC通讯参数配置');
      return;
    }

    const address = this.testAddresses[dataType];
    if (!address) {
      this.message.warning('请输入要测试的地址');
      return;
    }

    this.testingAddress[dataType] = true;
    // delete this.addressTestResults[dataType]; // 注释掉此行以在加载时保留旧值

    try {
      // 创建临时连接配置
      const tempConfig: PlcConnectionConfig = {
        id: 'temp_read_' + Date.now(),
        name: '临时读取测试',
        plcType: PlcType.ModbusTcp,
        ...this.connectionForm.value,
        connectionStatus: ConnectionStatus.Disconnected,
        isTestPlc: true,
        isEnabled: true,
        retryCount: 3,
        description: '临时地址读取测试'
      };

      // 调用后端服务进行地址读取测试
      const result = await this.testPlcConfigService.testAddressRead(tempConfig, address, dataType).toPromise();
      
      this.addressTestResults[dataType] = {
        address,
        dataType,
        success: result?.success || false,
        value: result?.value,
        error: result?.error
      };

      if (result?.success) {
        this.message.success(`地址 ${address} 读取成功`);
      } else {
        this.message.error(`地址 ${address} 读取失败：${result?.error || '未知错误'}`);
      }
    } catch (error) {
      console.error('地址读取测试失败:', error);
      this.addressTestResults[dataType] = {
        address,
        dataType,
        success: false,
        error: (error as any)?.message || '未知错误'
      };
      this.message.error('地址读取测试失败');
    } finally {
      this.testingAddress[dataType] = false;
    }
  }

  private async simulateAddressRead(address: string, dataType: 'bool' | 'float'): Promise<{ success: boolean; value?: any; error?: string }> {
    // 模拟异步操作
    await new Promise(resolve => setTimeout(resolve, 1000));
    
    // 这里应该调用实际的后端服务
    // 目前返回模拟数据
    if (dataType === 'bool') {
      return { success: true, value: Math.random() > 0.5 };
    } else {
      return { success: true, value: Math.random() * 100 };
    }
  }

  async saveConfiguration(): Promise<void> {
    if (this.connectionForm.invalid) {
      this.message.warning('表单数据无效，无法保存配置。');
      return;
    }

    this.savingConfiguration = true;
    const connectionToSave: PlcConnectionConfig = this.connectionForm.value;

    console.log('准备保存的PLC连接配置:', connectionToSave);

    try {
      const savedConnection = await this.testPlcConfigService.savePlcConnection(connectionToSave).toPromise();
      if (savedConnection) {
        this.message.success(`配置 "${savedConnection.name}" 已成功保存到数据库。`);
        this.onConfigSaved.emit(); // 通知父组件刷新数据
        this.closeModal();       // 关闭模态框
      } else {
        throw new Error('保存操作未返回有效的连接对象。');
      }
    } catch (error: any) {
      console.error('保存PLC配置失败:', error);
      const errorMessage = typeof error === 'string' ? error : (error.message || '未知错误');
      this.message.error(`保存配置失败: ${errorMessage}`);
    } finally {
      this.savingConfiguration = false;
    }
  }

  closeModal(): void {
    this.visible = false;
    this.visibleChange.emit(this.visible);
    
    // 重置状态
    this.connectionTestResult = null;
    this.addressTestResults = {};
    this.testingConnection = false;
  }

  getPlcTypeLabel(plcType?: PlcType): string {
    switch (plcType) {
      case PlcType.ModbusTcp:
        return 'Modbus TCP';
      case PlcType.SiemensS7:
        return 'Siemens S7';
      case PlcType.OpcUa:
        return 'OPC UA';
      default:
        return '未知';
    }
  }

  private generateId(): string {
    return Date.now().toString(36) + Math.random().toString(36).substr(2);
  }
} 