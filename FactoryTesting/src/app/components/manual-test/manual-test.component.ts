import { Component, OnInit, OnDestroy } from '@angular/core';
import { CommonModule } from '@angular/common';
import { FormsModule } from '@angular/forms';
import { Router } from '@angular/router';
import { Subscription } from 'rxjs';

// 数据模型接口
interface ChannelTestInstance {
  instanceId: string;
  batchId: string;
  channelLabel: string;
  description: string;
  moduleType: string;
  overallStatus: string;
  lastUpdated: string;
  definition: ChannelPointDefinition;
}

interface ChannelPointDefinition {
  channelLabel: string;
  description: string;
  moduleType: string;
  pointDataType: string;
  plcAddress: string;
  expectedValue?: number;
  tolerance?: number;
  highAlarmThreshold?: number;
  lowAlarmThreshold?: number;
}

interface SubTestItem {
  name: string;
  displayName: string;
  description: string;
  applicable: boolean;
}

interface ManualTestResult {
  success: boolean;
  message: string;
  value?: any;
  timestamp: string;
}

interface TestStatistics {
  totalChannels: number;
  readyForTest: number;
  inProgress: number;
  completed: number;
  failed: number;
}

@Component({
  selector: 'app-manual-test',
  standalone: true,
  imports: [CommonModule, FormsModule],
  templateUrl: './manual-test.component.html',
  styleUrl: './manual-test.component.css'
})
export class ManualTestComponent implements OnInit, OnDestroy {
  // 数据状态
  channels: ChannelTestInstance[] = [];
  filteredChannels: ChannelTestInstance[] = [];
  selectedChannel: ChannelTestInstance | null = null;
  availableSubTests: SubTestItem[] = [];
  selectedSubTest: SubTestItem | null = null;
  statistics: TestStatistics = {
    totalChannels: 0,
    readyForTest: 0,
    inProgress: 0,
    completed: 0,
    failed: 0
  };

  // 界面状态
  isLoading = false;
  error: string | null = null;
  searchTerm = '';
  selectedModuleType = '';
  selectedStatus = '';

  // 手动测试状态
  isTestInProgress = false;
  currentValue: any = null;
  inputValue: any = null;
  testResult: ManualTestResult | null = null;

  // 筛选选项
  moduleTypes = ['AI', 'AO', 'DI', 'DO', 'COMMUNICATION'];
  statusOptions = ['Ready', 'InProgress', 'Completed', 'Failed'];

  // 订阅管理
  private subscriptions: Subscription[] = [];

  constructor(private router: Router) {}

  ngOnInit() {
    this.loadChannels();
    this.initializeSubTests();
  }

  ngOnDestroy() {
    // 清理订阅
    this.subscriptions.forEach(sub => sub.unsubscribe());
  }

  // 加载通道数据
  async loadChannels() {
    this.isLoading = true;
    this.error = null;

    try {
      // 模拟API调用 - 实际应该调用Tauri命令
      await this.simulateApiCall();
      
      // 模拟数据
      this.channels = this.generateMockChannels();
      this.filteredChannels = [...this.channels];
      this.updateStatistics();
    } catch (error) {
      this.error = '加载通道数据失败: ' + (error as Error).message;
    } finally {
      this.isLoading = false;
    }
  }

  // 初始化子测试项
  initializeSubTests() {
    this.availableSubTests = [
      { name: 'ReadValue', displayName: '读取数值', description: '读取当前通道的实时数值', applicable: true },
      { name: 'WriteValue', displayName: '写入数值', description: '向通道写入指定数值', applicable: true },
      { name: 'HighAlarmTest', displayName: '高报警测试', description: '测试高报警阈值功能', applicable: true },
      { name: 'LowAlarmTest', displayName: '低报警测试', description: '测试低报警阈值功能', applicable: true },
      { name: 'CommunicationTest', displayName: '通信测试', description: '测试通道通信状态', applicable: true },
      { name: 'CalibrationTest', displayName: '校准测试', description: '执行通道校准程序', applicable: true }
    ];
  }

  // 筛选通道
  filterChannels() {
    this.filteredChannels = this.channels.filter(channel => {
      const matchesSearch = !this.searchTerm || 
        channel.channelLabel.toLowerCase().includes(this.searchTerm.toLowerCase()) ||
        channel.description.toLowerCase().includes(this.searchTerm.toLowerCase());
      
      const matchesModule = !this.selectedModuleType || 
        channel.moduleType === this.selectedModuleType;
      
      const matchesStatus = !this.selectedStatus || 
        channel.overallStatus === this.selectedStatus;

      return matchesSearch && matchesModule && matchesStatus;
    });
  }

  // 选择通道
  selectChannel(channel: ChannelTestInstance) {
    this.selectedChannel = channel;
    this.selectedSubTest = null;
    this.testResult = null;
    this.currentValue = null;
    this.inputValue = null;
    this.updateAvailableSubTests();
  }

  // 更新可用的子测试项
  updateAvailableSubTests() {
    if (!this.selectedChannel) return;

    const moduleType = this.selectedChannel.moduleType;
    
    // 根据模块类型更新子测试项的适用性
    this.availableSubTests.forEach(subTest => {
      switch (subTest.name) {
        case 'WriteValue':
          subTest.applicable = ['AO', 'DO'].includes(moduleType);
          break;
        case 'HighAlarmTest':
        case 'LowAlarmTest':
          subTest.applicable = ['AI', 'AO'].includes(moduleType);
          break;
        case 'CommunicationTest':
          subTest.applicable = moduleType === 'COMMUNICATION';
          break;
        case 'CalibrationTest':
          subTest.applicable = ['AI', 'AO'].includes(moduleType);
          break;
        default:
          subTest.applicable = true;
      }
    });
  }

  // 选择子测试项
  selectSubTest(subTest: SubTestItem) {
    this.selectedSubTest = subTest;
    this.testResult = null;
    this.inputValue = null;
    
    // 如果是读取数值，自动执行
    if (subTest.name === 'ReadValue') {
      this.executeReadValue();
    }
  }

  // 执行读取数值
  async executeReadValue() {
    if (!this.selectedChannel) return;

    this.isTestInProgress = true;
    this.testResult = null;

    try {
      // 模拟API调用 - 实际应该调用Tauri命令
      await this.simulateApiCall(500);
      
      // 模拟读取结果
      const mockValue = this.generateMockValue(this.selectedChannel.moduleType);
      
      this.currentValue = mockValue;
      this.testResult = {
        success: true,
        message: '读取成功',
        value: mockValue,
        timestamp: new Date().toLocaleString()
      };
    } catch (error) {
      this.testResult = {
        success: false,
        message: '读取失败: ' + (error as Error).message,
        timestamp: new Date().toLocaleString()
      };
    } finally {
      this.isTestInProgress = false;
    }
  }

  // 执行写入数值
  async executeWriteValue() {
    if (!this.selectedChannel || !this.selectedSubTest || this.inputValue === null) return;

    this.isTestInProgress = true;
    this.testResult = null;

    try {
      // 验证输入值
      if (!this.validateInputValue()) {
        throw new Error('输入值无效');
      }

      // 模拟API调用 - 实际应该调用Tauri命令
      await this.simulateApiCall(800);
      
      this.testResult = {
        success: true,
        message: `写入成功: ${this.inputValue}`,
        value: this.inputValue,
        timestamp: new Date().toLocaleString()
      };
    } catch (error) {
      this.testResult = {
        success: false,
        message: '写入失败: ' + (error as Error).message,
        timestamp: new Date().toLocaleString()
      };
    } finally {
      this.isTestInProgress = false;
    }
  }

  // 执行特定测试
  async executeSpecificTest() {
    if (!this.selectedChannel || !this.selectedSubTest) return;

    this.isTestInProgress = true;
    this.testResult = null;

    try {
      // 模拟API调用 - 实际应该调用Tauri命令
      await this.simulateApiCall(1200);
      
      const testName = this.selectedSubTest.displayName;
      this.testResult = {
        success: Math.random() > 0.2, // 80%成功率
        message: `${testName}执行完成`,
        timestamp: new Date().toLocaleString()
      };
    } catch (error) {
      this.testResult = {
        success: false,
        message: '测试执行失败: ' + (error as Error).message,
        timestamp: new Date().toLocaleString()
      };
    } finally {
      this.isTestInProgress = false;
    }
  }

  // 验证输入值
  validateInputValue(): boolean {
    if (!this.selectedChannel || this.inputValue === null) return false;

    const moduleType = this.selectedChannel.moduleType;
    
    switch (moduleType) {
      case 'AO':
        return typeof this.inputValue === 'number' && 
               this.inputValue >= 0 && this.inputValue <= 100;
      case 'DO':
        return typeof this.inputValue === 'boolean' || 
               this.inputValue === 0 || this.inputValue === 1;
      default:
        return true;
    }
  }

  // 获取输入类型
  getInputType(): string {
    if (!this.selectedChannel) return 'text';
    
    switch (this.selectedChannel.moduleType) {
      case 'AO':
        return 'number';
      case 'DO':
        return 'checkbox';
      default:
        return 'text';
    }
  }

  // 获取输入占位符
  getInputPlaceholder(): string {
    if (!this.selectedChannel) return '';
    
    switch (this.selectedChannel.moduleType) {
      case 'AO':
        return '请输入0-100之间的数值';
      case 'DO':
        return '选择开关状态';
      default:
        return '请输入值';
    }
  }

  // 更新统计信息
  updateStatistics() {
    this.statistics = {
      totalChannels: this.channels.length,
      readyForTest: this.channels.filter(c => c.overallStatus === 'Ready').length,
      inProgress: this.channels.filter(c => c.overallStatus === 'InProgress').length,
      completed: this.channels.filter(c => c.overallStatus === 'Completed').length,
      failed: this.channels.filter(c => c.overallStatus === 'Failed').length
    };
  }

  // 导航方法
  goToDashboard() {
    this.router.navigate(['/dashboard']);
  }

  goToTestExecution() {
    this.router.navigate(['/test-execution']);
  }

  // 工具方法
  private async simulateApiCall(delay: number = 1000): Promise<void> {
    return new Promise(resolve => setTimeout(resolve, delay));
  }

  private generateMockChannels(): ChannelTestInstance[] {
    const channels: ChannelTestInstance[] = [];
    const moduleTypes = ['AI', 'AO', 'DI', 'DO', 'COMMUNICATION'];
    const statuses = ['Ready', 'InProgress', 'Completed', 'Failed'];

    for (let i = 1; i <= 20; i++) {
      const moduleType = moduleTypes[Math.floor(Math.random() * moduleTypes.length)];
      const status = statuses[Math.floor(Math.random() * statuses.length)];
      
      channels.push({
        instanceId: `instance_${i}`,
        batchId: 'batch_001',
        channelLabel: `CH_${i.toString().padStart(3, '0')}`,
        description: `测试通道 ${i} - ${moduleType}类型`,
        moduleType,
        overallStatus: status,
        lastUpdated: new Date().toISOString(),
        definition: {
          channelLabel: `CH_${i.toString().padStart(3, '0')}`,
          description: `测试通道 ${i} - ${moduleType}类型`,
          moduleType,
          pointDataType: moduleType === 'AI' || moduleType === 'AO' ? 'Float32' : 'Boolean',
          plcAddress: `DB1.DBX${i}.0`,
          expectedValue: moduleType === 'AI' || moduleType === 'AO' ? 50.0 : undefined,
          tolerance: moduleType === 'AI' || moduleType === 'AO' ? 2.0 : undefined,
          highAlarmThreshold: moduleType === 'AI' ? 80.0 : undefined,
          lowAlarmThreshold: moduleType === 'AI' ? 20.0 : undefined
        }
      });
    }

    return channels;
  }

  private generateMockValue(moduleType: string): any {
    switch (moduleType) {
      case 'AI':
      case 'AO':
        return Math.round((Math.random() * 100) * 100) / 100;
      case 'DI':
      case 'DO':
        return Math.random() > 0.5;
      case 'COMMUNICATION':
        return Math.random() > 0.8 ? 'Connected' : 'Disconnected';
      default:
        return 'Unknown';
    }
  }
}
