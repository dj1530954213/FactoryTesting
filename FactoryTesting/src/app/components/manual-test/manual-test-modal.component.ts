import { Component, Input, Output, EventEmitter, OnInit, OnDestroy, OnChanges, SimpleChanges } from '@angular/core';
import { CommonModule } from '@angular/common';
import { FormsModule } from '@angular/forms';
import { NzModalModule } from 'ng-zorro-antd/modal';
import { NzButtonModule } from 'ng-zorro-antd/button';
import { NzIconModule } from 'ng-zorro-antd/icon';
import { NzMessageService } from 'ng-zorro-antd/message';
import { NzSpinModule } from 'ng-zorro-antd/spin';
import { Subscription } from 'rxjs';

import { ChannelTestInstance, ChannelPointDefinition, ModuleType } from '../../models';
import { ManualTestService } from '../../services/manual-test.service';
import { PlcMonitoringService } from '../../services/plc-monitoring.service';
import {
  ManualTestStatus,
  getManualTestConfig,
  StartPlcMonitoringRequest
} from '../../models/manual-test.types';

// 导入具体的手动测试组件
import { AiManualTestComponent } from './ai-manual-test.component';
import { AoManualTestComponent } from './ao-manual-test.component';
import { DiManualTestComponent } from './di-manual-test.component';
import { DoManualTestComponent } from './do-manual-test.component';

/**
 * 手动测试模态框组件
 * 根据点位类型动态显示对应的手动测试界面
 */
@Component({
  selector: 'app-manual-test-modal',
  standalone: true,
  imports: [
    CommonModule,
    FormsModule,
    NzModalModule,
    NzButtonModule,
    NzIconModule,
    NzSpinModule,
    AiManualTestComponent,
    AoManualTestComponent,
    DiManualTestComponent,
    DoManualTestComponent
  ],
  template: `
    <nz-modal
      [(nzVisible)]="visible"
      [nzTitle]="getModalTitle()"
      [nzWidth]="getModalWidth()"
      [nzFooter]="null"
      [nzClosable]="true"
      [nzMaskClosable]="false"
      (nzOnCancel)="onCancel()">
      
      <ng-container *nzModalContent>
        <nz-spin [nzSpinning]="isLoading" nzTip="正在初始化手动测试...">
          <div class="manual-test-content">
            
            <!-- 通道基本信息 -->
            <div class="channel-info-section">
              <h4>通道信息</h4>
              <div class="info-grid">
                <div class="info-item">
                  <label>点位名称:</label>
                  <span>{{ definition?.tag || 'N/A' }}</span>
                </div>
                <div class="info-item">
                  <label>变量描述:</label>
                  <span>{{ definition?.description || 'N/A' }}</span>
                </div>
                <div class="info-item">
                  <label>模块类型:</label>
                  <span>{{ definition?.module_type || 'N/A' }}</span>
                </div>
                <div class="info-item">
                  <label>通信地址:</label>
                  <span>{{ definition?.plc_communication_address || 'N/A' }}</span>
                </div>
              </div>
            </div>

            <!-- 根据模块类型显示对应的手动测试组件 -->
            <div class="test-content-section">
              <app-ai-manual-test
                *ngIf="definition?.module_type === ModuleType.AI"
                [instance]="instance"
                [definition]="definition"
                [testStatus]="currentTestStatus"
                (testCompleted)="onTestCompleted()"
                (testCancelled)="onTestCancelled()">
              </app-ai-manual-test>

              <app-ao-manual-test
                *ngIf="definition?.module_type === ModuleType.AO"
                [instance]="instance"
                [definition]="definition"
                [testStatus]="currentTestStatus"
                (testCompleted)="onTestCompleted()"
                (testCancelled)="onTestCancelled()">
              </app-ao-manual-test>

              <app-di-manual-test
                *ngIf="definition?.module_type === ModuleType.DI"
                [instance]="instance"
                [definition]="definition"
                [testStatus]="currentTestStatus"
                (testCompleted)="onTestCompleted()"
                (testCancelled)="onTestCancelled()">
              </app-di-manual-test>

              <app-do-manual-test
                *ngIf="definition?.module_type === ModuleType.DO"
                [instance]="instance"
                [definition]="definition"
                [testStatus]="currentTestStatus"
                (testCompleted)="onTestCompleted()"
                (testCancelled)="onTestCancelled()">
              </app-do-manual-test>

              <!-- 不支持的模块类型 -->
              <div *ngIf="!isSupportedModuleType()" class="unsupported-type">
                <span nz-icon nzType="exclamation-circle" nzTheme="outline" class="warning-icon"></span>
                <p>不支持的模块类型: {{ definition?.module_type }}</p>
              </div>
            </div>

          </div>
        </nz-spin>
      </ng-container>
    </nz-modal>
  `,
  styleUrls: ['./manual-test-modal.component.css']
})
export class ManualTestModalComponent implements OnInit, OnDestroy, OnChanges {
  @Input() visible = false;
  @Input() instance: ChannelTestInstance | null = null;
  @Input() definition: ChannelPointDefinition | null = null;
  @Output() visibleChange = new EventEmitter<boolean>();
  @Output() testCompleted = new EventEmitter<void>();

  // 组件状态
  isLoading = false;
  currentTestStatus: ManualTestStatus | null = null;

  // 订阅管理
  private subscriptions = new Subscription();

  // 模块类型枚举（用于模板）
  ModuleType = ModuleType;

  constructor(
    private manualTestService: ManualTestService,
    private plcMonitoringService: PlcMonitoringService,
    private message: NzMessageService
  ) {}

  ngOnInit(): void {
    // 订阅手动测试状态变化
    this.subscriptions.add(
      this.manualTestService.currentTestStatus$.subscribe(status => {
        this.currentTestStatus = status;
      })
    );

    // 订阅手动测试完成事件
    this.subscriptions.add(
      this.manualTestService.testCompleted$.subscribe(() => {
        this.onTestCompleted();
      })
    );
  }

  ngOnDestroy(): void {
    this.subscriptions.unsubscribe();
    // 清理PLC监控
    this.plcMonitoringService.stopMonitoring().catch(error => {
      console.error('停止PLC监控失败:', error);
    });
  }

  /**
   * 监听visible变化，当模态框打开时初始化测试
   */
  ngOnChanges(changes: SimpleChanges): void {
    if (changes['visible'] && this.visible && this.instance && this.definition) {
      this.initializeManualTest();
    } else if (changes['visible'] && !this.visible) {
      this.cleanup();
    }
  }

  /**
   * 初始化手动测试
   */
  private async initializeManualTest(): Promise<void> {
    if (!this.instance || !this.definition) return;

    try {
      this.isLoading = true;
      console.log('🔧 [MANUAL_TEST_MODAL] 初始化手动测试:', this.instance.instance_id);

      // 启动手动测试
      const response = await this.manualTestService.startManualTest({
        instanceId: this.instance.instance_id,
        moduleType: this.definition.module_type as ModuleType,
        operatorName: '当前操作员' // TODO: 从用户服务获取
      });

      if (!response.success) {
        throw new Error(response.message || '启动手动测试失败');
      }

      // 启动PLC监控
      await this.startPlcMonitoring();

      console.log('✅ [MANUAL_TEST_MODAL] 手动测试初始化成功');
    } catch (error) {
      console.error('❌ [MANUAL_TEST_MODAL] 初始化手动测试失败:', error);
      this.message.error(`初始化手动测试失败: ${error}`);
      this.onCancel();
    } finally {
      this.isLoading = false;
    }
  }

  /**
   * 启动PLC监控
   */
  private async startPlcMonitoring(): Promise<void> {
    if (!this.instance || !this.definition) return;

    try {
      const config = getManualTestConfig(this.definition.module_type as ModuleType);
      if (!config.plcMonitoringRequired) return;

      // 根据模块类型确定监控地址
      const monitoringAddresses = this.getMonitoringAddresses();
      if (monitoringAddresses.length === 0) return;

      // 构建地址→键名映射
      const addressKeyMap: Record<string, string> = {};
      const moduleType = this.definition.module_type as ModuleType;
      const baseAddress = this.definition.plc_communication_address;
      if (!baseAddress) {
        console.warn('⚠️ [MANUAL_TEST_MODAL] 通道定义缺少PLC通信地址:', this.definition.tag);
        return;
      }

      const sllAddr = this.definition.sll_set_point_communication_address || this.definition.sll_set_point_plc_address;
      if (sllAddr) {
        addressKeyMap[sllAddr] = 'sllSetPoint';
        console.log('📊 [MANUAL_TEST_MODAL] 添加SLL设定值地址:', this.definition.sll_set_point_communication_address);
      }
      const slAddr = this.definition.sl_set_point_communication_address || this.definition.sl_set_point_plc_address;
      if (slAddr) {
        addressKeyMap[slAddr] = 'slSetPoint';
        console.log('📊 [MANUAL_TEST_MODAL] 添加SL设定值地址:', this.definition.sl_set_point_communication_address);
      }
      const shAddr = this.definition.sh_set_point_communication_address || this.definition.sh_set_point_plc_address;
      if (shAddr) {
        addressKeyMap[shAddr] = 'shSetPoint';
        console.log('📊 [MANUAL_TEST_MODAL] 添加SH设定值地址:', this.definition.sh_set_point_communication_address);
      }
      const shhAddr = this.definition.shh_set_point_communication_address || this.definition.shh_set_point_plc_address;
      if (shhAddr) {
        addressKeyMap[shhAddr] = 'shhSetPoint';
        console.log('📊 [MANUAL_TEST_MODAL] 添加SHH设定值地址:', this.definition.shh_set_point_communication_address);
      }

      if (moduleType === ModuleType.AI) {
        addressKeyMap[baseAddress] = 'currentValue';
      } else if (moduleType === ModuleType.AO) {
        addressKeyMap[baseAddress] = 'currentOutput';
      } else if (moduleType === ModuleType.DI || moduleType === ModuleType.DO) {
        addressKeyMap[baseAddress] = 'currentState';
      }

      const request: StartPlcMonitoringRequest = {
        instanceId: this.instance.instance_id,
        moduleType: this.definition.module_type as ModuleType,
        monitoringAddresses,
        addressKeyMap
      };

      const response = await this.plcMonitoringService.startMonitoring(request);
      if (!response.success) {
        console.warn('⚠️ [MANUAL_TEST_MODAL] PLC监控启动失败:', response.message);
        // PLC监控失败不阻止手动测试继续
      }
    } catch (error) {
      console.error('❌ [MANUAL_TEST_MODAL] 启动PLC监控失败:', error);
      // PLC监控失败不阻止手动测试继续
    }
  }

  /**
   * 获取需要监控的PLC地址
   */
  private getMonitoringAddresses(): string[] {
    if (!this.definition) return [];

    const addresses: string[] = [];
    const moduleType = this.definition.module_type as ModuleType;

    // 优先使用实例分配的测试PLC通信地址（TestPlcChannelConfig.communication_address）
    let baseAddress = this.instance?.test_plc_communication_address;
    if (!baseAddress) {
      // 回退到通道定义自身的 PLC 通信地址
      baseAddress = this.definition.plc_communication_address;
    }
    if (!baseAddress) {
      console.warn('⚠️ [MANUAL_TEST_MODAL] 实例和定义均缺少PLC通信地址:', this.definition.tag);
      return [];
    }

    console.log('🔧 [MANUAL_TEST_MODAL] 获取监控地址 - 点位:', this.definition.tag, '地址:', baseAddress, '类型:', moduleType);

    switch (moduleType) {
      case ModuleType.AI:
        // AI点位监控当前值
        addresses.push(baseAddress);

        // 添加AI点位的报警设定值地址
        const sllAddr = this.definition.sll_set_point_communication_address || this.definition.sll_set_point_plc_address;
        if (sllAddr) {
          addresses.push(sllAddr);
          console.log('📊 [MANUAL_TEST_MODAL] 添加SLL设定值地址:', this.definition.sll_set_point_communication_address);
        }
        const slAddr = this.definition.sl_set_point_communication_address || this.definition.sl_set_point_plc_address;
        if (slAddr) {
          addresses.push(slAddr);
          console.log('📊 [MANUAL_TEST_MODAL] 添加SL设定值地址:', this.definition.sl_set_point_communication_address);
        }
        const shAddr = this.definition.sh_set_point_communication_address || this.definition.sh_set_point_plc_address;
        if (shAddr) {
          addresses.push(shAddr);
          console.log('📊 [MANUAL_TEST_MODAL] 添加SH设定值地址:', this.definition.sh_set_point_communication_address);
        }
        const shhAddr = this.definition.shh_set_point_communication_address || this.definition.shh_set_point_plc_address;
        if (shhAddr) {
          addresses.push(shhAddr);
          console.log('📊 [MANUAL_TEST_MODAL] 添加SHH设定值地址:', this.definition.shh_set_point_communication_address);
        }

        console.log('📊 [MANUAL_TEST_MODAL] AI点位监控地址列表:', addresses);
        break;

      case ModuleType.AO:
        // AO点位监控当前输出值
        addresses.push(baseAddress);
        console.log('📊 [MANUAL_TEST_MODAL] AO点位监控地址:', baseAddress);
        break;

      case ModuleType.DI:
      case ModuleType.DO:
        // DI/DO点位监控当前状态
        addresses.push(baseAddress);
        console.log('📊 [MANUAL_TEST_MODAL] 数字量点位监控地址:', baseAddress);
        break;

      default:
        console.warn('⚠️ [MANUAL_TEST_MODAL] 不支持的模块类型:', moduleType);
        break;
    }

    console.log('✅ [MANUAL_TEST_MODAL] 最终监控地址列表:', addresses);
    return addresses;
  }

  /**
   * 获取模态框标题
   */
  getModalTitle(): string {
    if (!this.definition) return '手动测试';
    return `${this.definition.module_type} 点位手动测试 - ${this.definition.tag}`;
  }

  /**
   * 获取模态框宽度
   */
  getModalWidth(): string {
    if (!this.definition) return '800px';
    
    const moduleType = this.definition.module_type as ModuleType;
    switch (moduleType) {
      case ModuleType.AI:
        return '1000px'; // AI点位测试项较多，需要更宽的窗口
      case ModuleType.AO:
        return '900px';
      case ModuleType.DI:
      case ModuleType.DO:
        return '700px'; // DI/DO点位测试项较少，窗口可以小一些
      default:
        return '800px';
    }
  }

  /**
   * 检查是否为支持的模块类型
   */
  isSupportedModuleType(): boolean {
    if (!this.definition) return false;
    const moduleType = this.definition.module_type as ModuleType;
    return [ModuleType.AI, ModuleType.AO, ModuleType.DI, ModuleType.DO].includes(moduleType);
  }

  /**
   * 测试完成处理
   */
  onTestCompleted(): void {
    console.log('🎉 [MANUAL_TEST_MODAL] 手动测试完成');
    this.message.success('手动测试已完成！');
    this.testCompleted.emit();
    this.closeModal();
  }

  /**
   * 测试取消处理
   */
  onTestCancelled(): void {
    console.log('🔧 [MANUAL_TEST_MODAL] 手动测试已取消');
    this.closeModal();
  }

  /**
   * 取消按钮处理
   */
  onCancel(): void {
    this.manualTestService.cancelCurrentTest();
    this.closeModal();
  }

  /**
   * 关闭模态框
   */
  private closeModal(): void {
    this.visible = false;
    this.visibleChange.emit(false);
    this.cleanup();
  }

  /**
   * 清理资源
   */
  private cleanup(): void {
    // 重置手动测试服务状态，确保下次可重新启动
    this.manualTestService.cancelCurrentTest();

    // 停止PLC监控
    this.plcMonitoringService.stopMonitoring().catch(error => {
      console.error('停止PLC监控失败:', error);
    });
  }
}
