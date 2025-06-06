import { Component, Input, Output, EventEmitter, OnInit, OnDestroy } from '@angular/core';
import { CommonModule } from '@angular/common';
import { NzModalModule } from 'ng-zorro-antd/modal';
import { NzProgressModule } from 'ng-zorro-antd/progress';
import { NzStatisticModule } from 'ng-zorro-antd/statistic';
import { NzTagModule } from 'ng-zorro-antd/tag';
import { NzIconModule } from 'ng-zorro-antd/icon';
import { NzSpinModule } from 'ng-zorro-antd/spin';
import { NzButtonModule } from 'ng-zorro-antd/button';
import { NzDividerModule } from 'ng-zorro-antd/divider';
import { NzGridModule } from 'ng-zorro-antd/grid';
import { Subscription } from 'rxjs';
import { listen } from '@tauri-apps/api/event';

// 测试进度统计接口
interface TestProgressStats {
  totalPoints: number;
  completedPoints: number;
  successPoints: number;
  failedPoints: number;
  currentPoint?: string;
  progressPercentage: number;
  estimatedTimeRemaining?: string;
}

@Component({
  selector: 'app-test-progress-modal',
  standalone: true,
  imports: [
    CommonModule,
    NzModalModule,
    NzProgressModule,
    NzStatisticModule,
    NzTagModule,
    NzIconModule,
    NzSpinModule,
    NzButtonModule,
    NzDividerModule,
    NzGridModule
  ],
  template: `
    <nz-modal
      [(nzVisible)]="visible"
      nzTitle="批次自动测试进度"
      [nzClosable]="false"
      [nzMaskClosable]="false"
      [nzFooter]="footerTemplate"
      nzWidth="600px">
      
      <div class="test-progress-content">
        <!-- 整体进度 -->
        <div class="overall-progress">
          <div class="progress-header">
            <h4>测试进度</h4>
            <span class="progress-percentage">{{ stats.progressPercentage.toFixed(1) }}%</span>
          </div>

          <nz-progress
            [nzPercent]="stats.progressPercentage"
            [nzStatus]="isTestCompleted ? 'success' : 'active'"
            [nzStrokeColor]="getProgressColor()"
            nzSize="default"
            [nzShowInfo]="true">
          </nz-progress>

          <div class="progress-info">
            <span>{{ stats.completedPoints }} / {{ stats.totalPoints }} 个点位已完成</span>
            <span *ngIf="stats.estimatedTimeRemaining" class="time-remaining">
              预计剩余时间: {{ stats.estimatedTimeRemaining }}
            </span>
            <span *ngIf="isTestCompleted" class="completion-status">
              ✅ 测试已完成
            </span>
          </div>
        </div>

        <nz-divider></nz-divider>

        <!-- 当前测试点位 -->
        <div class="current-test" *ngIf="stats.currentPoint">
          <div class="current-test-header">
            <i nz-icon nzType="play-circle" nzTheme="outline"></i>
            <span>当前测试点位</span>
          </div>
          <div class="current-point">
            <nz-tag nzColor="processing">{{ stats.currentPoint }}</nz-tag>
          </div>
        </div>

        <nz-divider></nz-divider>

        <!-- 测试统计 -->
        <div class="test-statistics">
          <nz-row [nzGutter]="16">
            <nz-col nzSpan="6">
              <nz-statistic
                nzTitle="总点位数"
                [nzValue]="stats.totalPoints"
                [nzValueStyle]="{ color: '#1890ff' }">
              </nz-statistic>
            </nz-col>
            <nz-col nzSpan="6">
              <nz-statistic
                nzTitle="已完成"
                [nzValue]="stats.completedPoints"
                [nzValueStyle]="{ color: '#52c41a' }">
              </nz-statistic>
            </nz-col>
            <nz-col nzSpan="6">
              <nz-statistic
                nzTitle="测试通过"
                [nzValue]="stats.successPoints"
                [nzValueStyle]="{ color: '#52c41a' }">
              </nz-statistic>
            </nz-col>
            <nz-col nzSpan="6">
              <nz-statistic
                nzTitle="测试失败"
                [nzValue]="stats.failedPoints"
                [nzValueStyle]="{ color: '#ff4d4f' }">
              </nz-statistic>
            </nz-col>
          </nz-row>
        </div>

        <!-- 最近的测试结果 -->
        <div class="recent-results" *ngIf="recentResults.length > 0">
          <nz-divider></nz-divider>
          <h5>最近测试结果</h5>
          <div class="result-list">
            <div 
              *ngFor="let result of recentResults.slice(-5)" 
              class="result-item"
              [class.success]="result.success"
              [class.failed]="!result.success">
              <nz-tag [nzColor]="result.success ? 'success' : 'error'">
                {{ result.pointTag }}
              </nz-tag>
              <span class="result-message">{{ result.message }}</span>
              <span class="result-time">{{ formatTime(result.timestamp) }}</span>
            </div>
          </div>
        </div>
      </div>

      <ng-template #footerTemplate>
        <button
          nz-button
          nzType="default"
          (click)="closeModal()">
          {{ isTestCompleted ? '关闭' : '强制关闭' }}
        </button>
        <button
          *ngIf="isTestCompleted"
          nz-button
          nzType="primary"
          (click)="closeModal()">
          完成
        </button>
      </ng-template>
    </nz-modal>
  `,
  styleUrls: ['./test-progress-modal.component.css']
})
export class TestProgressModalComponent implements OnInit, OnDestroy {
  @Input()
  set visible(value: boolean) {
    console.log('🔧 [TestProgressModal] visible 属性变化:', value);
    this._visible = value;
  }
  get visible(): boolean {
    return this._visible;
  }
  private _visible = false;

  @Input() batchId = '';
  @Output() visibleChange = new EventEmitter<boolean>();
  @Output() testCompleted = new EventEmitter<void>();

  // 测试进度统计
  stats: TestProgressStats = {
    totalPoints: 0,
    completedPoints: 0,
    successPoints: 0,
    failedPoints: 0,
    progressPercentage: 0
  };

  // 最近的测试结果
  recentResults: Array<{
    pointTag: string;
    success: boolean;
    message: string;
    timestamp: Date;
  }> = [];

  // 测试是否完成
  isTestCompleted = false;

  // 订阅管理
  private subscriptions = new Subscription();

  // 🔧 添加完成检测相关属性
  private lastProgressUpdateTime = 0;
  private completionCheckTimer?: any;
  private batchStatusCheckTimer?: any;

  ngOnInit(): void {
    this.setupEventListeners();
  }

  ngOnDestroy(): void {
    this.subscriptions.unsubscribe();

    // 🔧 清理完成检测定时器
    if (this.completionCheckTimer) {
      clearTimeout(this.completionCheckTimer);
    }

    // 🔧 清理批次状态检查定时器
    if (this.batchStatusCheckTimer) {
      clearInterval(this.batchStatusCheckTimer);
    }
  }

  /**
   * 设置事件监听器
   */
  private async setupEventListeners(): Promise<void> {
    try {
      // 监听测试完成事件
      const unlistenCompleted = await listen('test-completed', (event) => {
        console.log('🎉 [TestProgressModal] 收到测试完成事件:', event.payload);

        const testResult = event.payload as {
          instanceId: string;
          success: boolean;
          subTestItem: string;
          message: string;
          pointTag?: string;
        };

        this.updateProgress(testResult);
      });

      // 监听测试状态变化事件
      const unlistenStatusChanged = await listen('test-status-changed', (event) => {
        console.log('🔄 [TestProgressModal] 收到测试状态变化事件:', event.payload);

        const statusChange = event.payload as {
          instanceId: string;
          oldStatus: string;
          newStatus: string;
          pointTag?: string;
        };

        this.updateCurrentTest(statusChange);
      });

      // 监听测试进度更新事件
      const unlistenProgressUpdate = await listen('test-progress-update', (event) => {
        console.log('📊 [TestProgressModal] 收到测试进度更新事件:', event.payload);

        const progressData = event.payload as {
          batchId: string;
          totalPoints: number;
          completedPoints: number;
          successPoints: number;
          failedPoints: number;
          progressPercentage: number;
          currentPoint?: string;
        };

        // 只有当批次ID匹配时才更新进度
        if (progressData.batchId === this.batchId) {
          // 直接更新进度统计
          this.stats.totalPoints = progressData.totalPoints;
          this.stats.completedPoints = progressData.completedPoints;
          this.stats.successPoints = progressData.successPoints;
          this.stats.failedPoints = progressData.failedPoints;
          this.stats.progressPercentage = progressData.progressPercentage;
          this.stats.currentPoint = progressData.currentPoint;

          // 更新最后进度更新时间
          this.lastProgressUpdateTime = Date.now();

          console.log('📊 [TestProgressModal] 进度统计已更新:', this.stats);

          // 检查是否完成
          if (this.stats.progressPercentage >= 100 && !this.isTestCompleted) {
            this.startCompletionDetection();
          }
        } else {
          console.log('🔄 [TestProgressModal] 忽略其他批次的进度更新:', progressData.batchId);
        }
      });

      // 🔧 添加批次状态变化监听器
      const unlistenBatchStatusChanged = await listen('batch-status-changed', (event) => {
        console.log('🎯 [TestProgressModal] 收到批次状态变化事件:', event.payload);

        const batchStatus = event.payload as {
          batchId: string;
          status: string;
          statistics?: {
            totalChannels: number;
            testedChannels: number;
            passedChannels: number;
            failedChannels: number;
            skippedChannels: number;
            inProgressChannels: number;
            progressPercentage: number;
          };
        };

        // 更新进度统计信息
        if (batchStatus.statistics) {
          const stats = batchStatus.statistics;
          this.stats.totalPoints = stats.totalChannels;
          this.stats.completedPoints = stats.testedChannels;
          this.stats.successPoints = stats.passedChannels;
          this.stats.failedPoints = stats.failedChannels;
          this.stats.progressPercentage = stats.progressPercentage;

          console.log('🔄 [TestProgressModal] 从批次状态更新进度:', this.stats);
        }

        // 如果批次状态变为完成，则标记测试完成
        if (batchStatus.status === 'completed' || batchStatus.status === 'finished') {
          console.log('🎉 [TestProgressModal] 批次测试完成');
          this.isTestCompleted = true;
          this.stats.currentPoint = undefined;
          this.stats.progressPercentage = 100;
          this.testCompleted.emit();

          // 清理定时器
          if (this.batchStatusCheckTimer) {
            clearInterval(this.batchStatusCheckTimer);
            this.batchStatusCheckTimer = undefined;
          }
        }
      });

      // 在组件销毁时清理事件监听器
      this.subscriptions.add({
        unsubscribe: () => {
          unlistenCompleted();
          unlistenStatusChanged();
          unlistenProgressUpdate();
          unlistenBatchStatusChanged();
        }
      });

    } catch (error) {
      console.error('❌ [TestProgressModal] 设置事件监听器失败:', error);
    }
  }

  /**
   * 更新测试进度
   */
  private updateProgress(testResult: any): void {
    console.log('🔄 [TestProgressModal] 收到测试结果:', testResult);

    // 🔧 防止重复计算同一个测试实例
    const existingResult = this.recentResults.find(r =>
      r.pointTag === (testResult.pointTag || testResult.instanceId)
    );

    if (!existingResult) {
      // 更新完成点位数
      this.stats.completedPoints++;

      // 更新成功/失败统计
      if (testResult.success) {
        this.stats.successPoints++;
      } else {
        this.stats.failedPoints++;
      }

      // 添加到最近结果
      this.recentResults.push({
        pointTag: testResult.pointTag || testResult.instanceId,
        success: testResult.success,
        message: testResult.message || (testResult.success ? '测试通过' : '测试失败'),
        timestamp: new Date()
      });
    } else {
      console.log('🔄 [TestProgressModal] 跳过重复的测试结果:', testResult.instanceId);
    }

    // 🔧 安全计算进度百分比，避免除零错误
    if (this.stats.totalPoints > 0) {
      this.stats.progressPercentage = Math.min(100, (this.stats.completedPoints / this.stats.totalPoints) * 100);
    } else {
      this.stats.progressPercentage = 100; // 如果总点位数为0，设为100%
    }

    // 🔧 更新最后进度更新时间
    this.lastProgressUpdateTime = Date.now();

    console.log('🔄 [TestProgressModal] 进度更新:', this.stats);
    console.log('🔄 [TestProgressModal] 当前进度:', `${this.stats.completedPoints}/${this.stats.totalPoints} (${this.stats.progressPercentage.toFixed(1)}%)`);

    // 🔧 启动完成检测机制
    this.startCompletionDetection();
  }

  /**
   * 🔧 启动完成检测机制
   * 如果进度达到100%且在一定时间内没有新的更新，则认为测试完成
   */
  private startCompletionDetection(): void {
    // 清除之前的定时器
    if (this.completionCheckTimer) {
      clearTimeout(this.completionCheckTimer);
    }

    // 🔧 如果进度已经达到100%，立即标记为完成
    if (this.stats.progressPercentage >= 100 && !this.isTestCompleted) {
      console.log('🎉 [TestProgressModal] 进度达到100%，立即标记为完成');
      this.isTestCompleted = true;
      this.stats.currentPoint = undefined;
      this.testCompleted.emit();

      // 🔧 不自动关闭模态框，等待用户手动关闭
      console.log('🔧 [TestProgressModal] 测试完成，等待用户手动关闭');
    }
  }

  /**
   * 更新当前测试点位
   */
  private updateCurrentTest(statusChange: any): void {
    if (statusChange.newStatus.includes('Testing') || statusChange.newStatus.includes('测试中')) {
      this.stats.currentPoint = statusChange.pointTag || statusChange.instanceId;
    }
  }

  /**
   * 初始化进度统计
   */
  initializeProgress(totalPoints: number): void {
    console.log('🔧 [TestProgressModal] 初始化进度统计，总点位数:', totalPoints);

    // 🔧 确保总点位数至少为1，避免除零错误
    const safeTotal = Math.max(1, totalPoints);

    this.stats = {
      totalPoints: safeTotal,
      completedPoints: 0,
      successPoints: 0,
      failedPoints: 0,
      progressPercentage: 0,
      currentPoint: undefined
    };
    this.recentResults = [];
    this.isTestCompleted = false;

    // 🔧 重置完成检测相关状态
    this.lastProgressUpdateTime = Date.now();
    if (this.completionCheckTimer) {
      clearTimeout(this.completionCheckTimer);
      this.completionCheckTimer = undefined;
    }

    // 🔧 如果原始总点位数为0，直接标记为完成
    if (totalPoints === 0) {
      console.log('⚠️ [TestProgressModal] 总点位数为0，直接标记为完成');
      this.isTestCompleted = true;
      this.stats.progressPercentage = 100;
      this.testCompleted.emit();
    }

    // 🔧 设置定期检查批次状态的定时器
    this.setupBatchStatusCheckTimer();

    console.log('✅ [TestProgressModal] 进度统计初始化完成:', this.stats);
  }

  /**
   * 设置批次状态检查定时器
   * 即使没有收到测试完成事件，也会定期检查批次状态
   */
  private setupBatchStatusCheckTimer(): void {
    // 清除之前的定时器
    if (this.batchStatusCheckTimer) {
      clearInterval(this.batchStatusCheckTimer);
    }

    // 每5秒检查一次批次状态
    this.batchStatusCheckTimer = setInterval(() => {
      // 如果已经标记为完成，则停止检查
      if (this.isTestCompleted) {
        clearInterval(this.batchStatusCheckTimer);
        return;
      }

      // 如果超过30秒没有进度更新，则检查批次状态
      const currentTime = Date.now();
      const timeSinceLastUpdate = currentTime - this.lastProgressUpdateTime;

      if (timeSinceLastUpdate > 30000) {
        console.log('⚠️ [TestProgressModal] 超过30秒没有进度更新，检查批次状态');

        // 如果进度已经达到一定程度，可以认为测试已完成
        if (this.stats.progressPercentage > 90) {
          console.log('🎉 [TestProgressModal] 进度已达到90%以上，标记为完成');
          this.isTestCompleted = true;
          this.stats.progressPercentage = 100;
          this.testCompleted.emit();
          clearInterval(this.batchStatusCheckTimer);
          this.batchStatusCheckTimer = undefined;
        }
      }
    }, 5000);
  }

  /**
   * 获取进度条颜色
   */
  getProgressColor(): string {
    if (this.stats.progressPercentage < 30) return '#ff4d4f';
    if (this.stats.progressPercentage < 70) return '#faad14';
    return '#52c41a';
  }

  /**
   * 格式化时间
   */
  formatTime(timestamp: Date): string {
    return timestamp.toLocaleTimeString('zh-CN', {
      hour: '2-digit',
      minute: '2-digit',
      second: '2-digit'
    });
  }



  /**
   * 关闭模态框
   */
  closeModal(): void {
    console.log('🔧 [TestProgressModal] 用户点击关闭按钮');
    this.visible = false;
    this.visibleChange.emit(false);

    // 🔧 清理所有定时器
    if (this.completionCheckTimer) {
      clearTimeout(this.completionCheckTimer);
      this.completionCheckTimer = undefined;
    }

    if (this.batchStatusCheckTimer) {
      clearInterval(this.batchStatusCheckTimer);
      this.batchStatusCheckTimer = undefined;
    }
  }
}
