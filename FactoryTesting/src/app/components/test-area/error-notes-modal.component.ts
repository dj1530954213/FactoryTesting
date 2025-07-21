import { Component, Input, Output, EventEmitter, OnInit, OnDestroy, OnChanges, SimpleChanges } from '@angular/core';
import { CommonModule } from '@angular/common';
import { FormsModule } from '@angular/forms';
import { NzModalModule } from 'ng-zorro-antd/modal';
import { NzFormModule } from 'ng-zorro-antd/form';
import { NzInputModule } from 'ng-zorro-antd/input';
import { NzButtonModule } from 'ng-zorro-antd/button';
import { NzDividerModule } from 'ng-zorro-antd/divider';
import { NzMessageService } from 'ng-zorro-antd/message';
import { NzIconModule } from 'ng-zorro-antd/icon';
import { NzTagModule } from 'ng-zorro-antd/tag';
import { TauriApiService } from '../../services/tauri-api.service';
import { ChannelTestInstance, ChannelPointDefinition } from '../../models';
import { firstValueFrom } from 'rxjs';

@Component({
  selector: 'app-error-notes-modal',
  standalone: true,
  imports: [
    CommonModule,
    FormsModule,
    NzModalModule,
    NzFormModule,
    NzInputModule,
    NzButtonModule,
    NzDividerModule,
    NzIconModule,
    NzTagModule
  ],
  template: `
    <!-- 自定义模态框覆盖层 -->
    <div *ngIf="visible" class="modal-overlay" (click)="onCancel()">
      <div class="modal-container" (click)="$event.stopPropagation()">
        <!-- 标题栏 -->
        <div class="modal-header">
          <h3>错误备注</h3>
          <button class="close-btn" (click)="onCancel()">×</button>
        </div>
        
        <!-- 内容区域 -->
        <div class="modal-body">
          

          <!-- 测试错误信息 -->
          <div *ngIf="hasAnyErrors()" class="error-section">
            <h3>测试错误信息</h3>
            
            <!-- 硬点测试错误 -->
            <div *ngIf="getHardPointError()" class="error-item danger">
              <div class="error-icon">
                <i nz-icon nzType="close-circle" nzTheme="fill"></i>
              </div>
              <div class="error-content">
                <div class="error-title">硬点测试错误</div>
                <div class="error-detail">{{ getHardPointError() }}</div>
              </div>
            </div>

            <!-- 手动测试错误 -->
            <div *ngIf="getManualTestError()" class="error-item warning">
              <div class="error-icon">
                <i nz-icon nzType="warning" nzTheme="fill"></i>
              </div>
              <div class="error-content">
                <div class="error-title">手动测试错误</div>
                <div class="error-detail">{{ getManualTestError() }}</div>
              </div>
            </div>

            <!-- 测试失败项目小卡片 -->
            <div *ngIf="getFailedTestItems().length > 0" class="failed-tests-section">
              <h4>失败的测试项目 <span class="failure-count">({{ getFailedTestItems().length }}项)</span></h4>
              <div class="failed-test-cards">
                <div *ngFor="let item of getFailedTestItems(); let i = index" 
                     class="test-item-card" 
                     [style.animation-delay]="(i * 0.1) + 's'">
                  <div class="test-item-header">
                    <div class="test-item-icon">
                      <i nz-icon nzType="close" nzTheme="outline"></i>
                    </div>
                    <div class="test-item-content">
                      <div class="test-item-name">{{ item }}</div>
                      <div class="test-item-status">
                        <i nz-icon nzType="exclamation-circle" nzTheme="fill"></i>
                        测试失败
                      </div>
                    </div>
                  </div>
                  
                </div>
              </div>
            </div>
          </div>

          <!-- 错误备注录入卡片 -->
          <div class="notes-input-card">
            <div class="card-header notes">
              <i nz-icon nzType="edit" nzTheme="fill"></i>
              <span>错误备注分类录入</span>
            </div>
            <div class="notes-form">
              <!-- 集成错误备注 -->
              <div class="note-section integration">
                <div class="note-header">
                  <div class="note-icon">
                    <i nz-icon nzType="cluster" nzTheme="outline"></i>
                  </div>
                  <div class="note-info">
                    <div class="note-title">集成错误备注</div>
                    <div class="note-subtitle">系统集成相关问题</div>
                  </div>
                </div>
                <textarea 
                  nz-input
                  [(ngModel)]="errorNotes.integration"
                  placeholder="记录与系统集成相关的错误信息，如通信故障、连接问题、配置错误等..."
                  [nzAutosize]="{ minRows: 3, maxRows: 6 }"
                  class="note-textarea">
                </textarea>
              </div>

              <!-- PLC编程错误备注 -->
              <div class="note-section plc">
                <div class="note-header">
                  <div class="note-icon">
                    <i nz-icon nzType="code" nzTheme="outline"></i>
                  </div>
                  <div class="note-info">
                    <div class="note-title">PLC编程错误备注</div>
                    <div class="note-subtitle">PLC程序相关问题</div>
                  </div>
                </div>
                <textarea 
                  nz-input
                  [(ngModel)]="errorNotes.plc"
                  placeholder="记录与PLC编程相关的错误信息，如逻辑错误、地址配置、程序段问题等..."
                  [nzAutosize]="{ minRows: 3, maxRows: 6 }"
                  class="note-textarea">
                </textarea>
              </div>

              <!-- 上位机组态错误备注 -->
              <div class="note-section hmi">
                <div class="note-header">
                  <div class="note-icon">
                    <i nz-icon nzType="desktop" nzTheme="outline"></i>
                  </div>
                  <div class="note-info">
                    <div class="note-title">上位机组态错误备注</div>
                    <div class="note-subtitle">HMI/SCADA组态相关问题</div>
                  </div>
                </div>
                <textarea 
                  nz-input
                  [(ngModel)]="errorNotes.hmi"
                  placeholder="记录与上位机组态相关的错误信息，如界面配置、数据绑定、显示异常等..."
                  [nzAutosize]="{ minRows: 3, maxRows: 6 }"
                  class="note-textarea">
                </textarea>
              </div>
            </div>
          </div>
        </div>
        
        <!-- 底部按钮 -->
        <div class="modal-footer">
          <button nz-button nzType="default" (click)="onCancel()">取消</button>
          <button nz-button nzType="primary" [nzLoading]="isSaving" (click)="onSave()">保存</button>
        </div>
      </div>
    </div>
  `,
  styles: [`
    /* 模态框基础样式 */
    .modal-overlay {
      position: fixed;
      top: 0;
      left: 0;
      width: 100%;
      height: 100%;
      background: rgba(0, 0, 0, 0.6);
      display: flex;
      justify-content: center;
      align-items: flex-start;
      z-index: 1000;
      padding-top: 30px;
      backdrop-filter: blur(2px);
    }

    .modal-container {
      background: linear-gradient(135deg, #ffffff 0%, #fafbfc 100%);
      border-radius: 16px;
      box-shadow: 0 20px 60px rgba(0, 0, 0, 0.3), 0 0 0 1px rgba(255, 255, 255, 0.1);
      width: 1000px;
      max-height: 90vh;
      display: flex;
      flex-direction: column;
      border: 1px solid rgba(255, 255, 255, 0.2);
    }

    .modal-header {
      padding: 20px 28px;
      border-bottom: 1px solid rgba(240, 240, 240, 0.8);
      display: flex;
      justify-content: space-between;
      align-items: center;
      background: linear-gradient(135deg, #f8f9fa 0%, #ffffff 100%);
      border-radius: 16px 16px 0 0;
    }

    .modal-header h3 {
      margin: 0;
      color: #1c1c1c;
      font-weight: 700;
      font-size: 18px;
      letter-spacing: -0.5px;
    }

    .close-btn {
      background: rgba(255, 255, 255, 0.8);
      border: 1px solid rgba(0, 0, 0, 0.05);
      font-size: 20px;
      cursor: pointer;
      color: #666;
      width: 36px;
      height: 36px;
      border-radius: 50%;
      display: flex;
      align-items: center;
      justify-content: center;
      transition: all 0.2s ease;
      backdrop-filter: blur(4px);
    }

    .close-btn:hover {
      background: rgba(244, 244, 245, 0.9);
      color: #333;
      transform: scale(1.05);
    }

    .modal-body {
      padding: 28px;
      overflow-y: auto;
      flex: 1;
      max-height: calc(90vh - 140px);
    }

    .modal-footer {
      padding: 16px 28px;
      border-top: 1px solid rgba(240, 240, 240, 0.8);
      display: flex;
      justify-content: flex-end;
      gap: 12px;
      background: linear-gradient(135deg, #f8f9fa 0%, #ffffff 100%);
      border-radius: 0 0 16px 16px;
    }

    /* 卡片基础样式 */
    .info-card, .error-summary-card, .notes-input-card {
      background: #ffffff;
      border-radius: 12px;
      margin-bottom: 24px;
      box-shadow: 0 2px 8px rgba(0, 0, 0, 0.04), 0 1px 2px rgba(0, 0, 0, 0.06);
      border: 1px solid rgba(240, 240, 240, 0.6);
      overflow: hidden;
    }

    /* 错误信息区域 */
    .error-section {
      margin-bottom: 32px;
    }

    .error-section h3 {
      color: #cf1322;
      font-size: 18px;
      font-weight: 700;
      margin-bottom: 20px;
      display: flex;
      align-items: center;
      gap: 8px;
    }

    .error-section h3::before {
      content: '⚠️';
      font-size: 20px;
    }

    /* 错误项目样式 */
    .error-item {
      display: flex;
      gap: 16px;
      padding: 16px;
      border-radius: 8px;
      margin-bottom: 16px;
      border: 1px solid #f0f0f0;
      transition: all 0.2s ease;
    }

    .error-item:hover {
      transform: translateY(-1px);
      box-shadow: 0 4px 12px rgba(0, 0, 0, 0.08);
    }

    .error-item.danger {
      border-left: 4px solid #ff4d4f;
      background: linear-gradient(135deg, #ffffff 0%, #fff2f0 100%);
    }

    .error-item.warning {
      border-left: 4px solid #fa8c16;
      background: linear-gradient(135deg, #ffffff 0%, #fff7e6 100%);
    }

    .error-icon {
      flex-shrink: 0;
      width: 24px;
      height: 24px;
      display: flex;
      align-items: center;
      justify-content: center;
      border-radius: 50%;
    }

    .error-item.danger .error-icon {
      background: #ff4d4f;
      color: white;
    }

    .error-item.warning .error-icon {
      background: #fa8c16;
      color: white;
    }

    .error-content {
      flex: 1;
    }

    .error-title {
      font-weight: 600;
      color: #262626;
      margin-bottom: 6px;
      font-size: 14px;
    }

    .error-detail {
      color: #595959;
      line-height: 1.5;
      font-size: 13px;
    }

    /* 失败测试项目区域 */
    .failed-tests-section {
      margin-top: 24px;
    }

    .failed-tests-section h4 {
      color: #262626;
      font-size: 16px;
      font-weight: 600;
      margin-bottom: 20px;
      display: flex;
      align-items: center;
      gap: 8px;
    }

    .failed-tests-section h4::before {
      content: '🔴';
      font-size: 18px;
    }

    .failure-count {
      font-size: 14px;
      color: #8c8c8c;
      font-weight: 400;
      margin-left: 8px;
    }

    .failed-test-cards {
      display: grid;
      grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
      gap: 16px;
    }

    .test-item-card {
      background: linear-gradient(135deg, #ffffff 0%, #fff1f0 100%);
      border: 1px solid #ffccc7;
      border-left: 4px solid #ff4d4f;
      border-radius: 12px;
      padding: 16px;
      text-align: left;
      transition: all 0.3s ease;
      box-shadow: 0 2px 8px rgba(255, 77, 79, 0.08);
      position: relative;
      overflow: hidden;
    }

    .test-item-card::before {
      content: '';
      position: absolute;
      top: 0;
      left: 0;
      right: 0;
      height: 2px;
      background: linear-gradient(90deg, #ff4d4f 0%, #ff7875 100%);
    }

    .test-item-card:hover {
      transform: translateY(-3px);
      box-shadow: 0 8px 25px rgba(255, 77, 79, 0.15);
      border-color: #ff4d4f;
      background: linear-gradient(135deg, #ffffff 0%, #fff2f0 100%);
    }

    .test-item-header {
      display: flex;
      align-items: center;
      gap: 12px;
      margin-bottom: 8px;
    }

    .test-item-icon {
      width: 32px;
      height: 32px;
      background: linear-gradient(135deg, #ff4d4f 0%, #ff7875 100%);
      border-radius: 8px;
      display: flex;
      align-items: center;
      justify-content: center;
      color: white;
      font-size: 14px;
      flex-shrink: 0;
      box-shadow: 0 2px 6px rgba(255, 77, 79, 0.3);
    }

    .test-item-content {
      flex: 1;
    }

    .test-item-name {
      font-size: 14px;
      color: #262626;
      font-weight: 600;
      line-height: 1.4;
      margin-bottom: 4px;
    }

    .test-item-status {
      font-size: 12px;
      color: #ff4d4f;
      font-weight: 500;
      display: flex;
      align-items: center;
      gap: 4px;
    }

    .test-item-status i {
      font-size: 12px;
      margin-right: 4px;
    }

    .test-item-footer {
      margin-top: 12px;
      padding-top: 12px;
      border-top: 1px solid rgba(255, 77, 79, 0.1);
      display: flex;
      justify-content: flex-end;
    }

    .severity-indicator {
      font-size: 11px;
      padding: 2px 8px;
      border-radius: 12px;
      font-weight: 500;
      text-transform: uppercase;
      letter-spacing: 0.5px;
    }

    .severity-indicator.high {
      background: linear-gradient(135deg, #ff4d4f 0%, #ff7875 100%);
      color: white;
      box-shadow: 0 1px 3px rgba(255, 77, 79, 0.3);
    }

    .severity-indicator.medium {
      background: linear-gradient(135deg, #fa8c16 0%, #ffa940 100%);
      color: white;
      box-shadow: 0 1px 3px rgba(250, 140, 22, 0.3);
    }

    .severity-indicator.low {
      background: linear-gradient(135deg, #faad14 0%, #ffd666 100%);
      color: white;
      box-shadow: 0 1px 3px rgba(250, 173, 20, 0.3);
    }

    /* 卡片动画效果 */
    .test-item-card {
      animation: fadeInUp 0.3s ease-out;
    }

    @keyframes fadeInUp {
      from {
        opacity: 0;
        transform: translateY(20px);
      }
      to {
        opacity: 1;
        transform: translateY(0);
      }
    }

    /* 不同测试项目的图标样式 */
    .test-item-card:nth-child(1) .test-item-icon {
      background: linear-gradient(135deg, #ff4d4f 0%, #ff7875 100%);
    }

    .test-item-card:nth-child(2) .test-item-icon {
      background: linear-gradient(135deg, #fa541c 0%, #ff7a45 100%);
    }

    .test-item-card:nth-child(3) .test-item-icon {
      background: linear-gradient(135deg, #fa8c16 0%, #ffa940 100%);
    }

    .test-item-card:nth-child(4) .test-item-icon {
      background: linear-gradient(135deg, #faad14 0%, #ffd666 100%);
    }

    .test-item-card:nth-child(5) .test-item-icon {
      background: linear-gradient(135deg, #722ed1 0%, #9254de 100%);
    }

    .test-item-card:nth-child(6) .test-item-icon {
      background: linear-gradient(135deg, #eb2f96 0%, #f759ab 100%);
    }

    /* 响应式优化 */
    @media (max-width: 768px) {
      .failed-test-cards {
        grid-template-columns: repeat(auto-fill, minmax(160px, 1fr));
        gap: 12px;
      }
      
      .test-item-card {
        padding: 12px;
      }
      
      .test-item-icon {
        width: 28px;
        height: 28px;
        font-size: 12px;
      }
      
      .test-item-name {
        font-size: 13px;
      }
      
      .test-item-status {
        font-size: 11px;
      }
    }

    /* 大屏幕优化 */
    @media (min-width: 1200px) {
      .failed-test-cards {
        grid-template-columns: repeat(auto-fill, minmax(220px, 1fr));
        gap: 18px;
      }
    }

    .card-header {
      padding: 16px 20px;
      border-bottom: 1px solid rgba(240, 240, 240, 0.6);
      display: flex;
      align-items: center;
      gap: 10px;
      font-weight: 600;
      font-size: 15px;
      background: linear-gradient(135deg, #fafbfc 0%, #f8f9fa 100%);
    }

    .card-header i {
      font-size: 16px;
    }

    .card-header.error {
      background: linear-gradient(135deg, #fff2f0 0%, #ffebe8 100%);
      color: #cf1322;
      border-bottom-color: #ffccc7;
    }

    .card-header.notes {
      background: linear-gradient(135deg, #f6ffed 0%, #eef9e6 100%);
      color: #389e0d;
      border-bottom-color: #b7eb8f;
    }

    /* 通道信息网格 */
    .info-grid {
      padding: 20px;
      display: grid;
      grid-template-columns: repeat(2, 1fr);
      gap: 16px;
    }

    .info-item {
      display: flex;
      flex-direction: column;
      gap: 4px;
    }

    .info-label {
      font-size: 12px;
      color: #8c8c8c;
      font-weight: 500;
      text-transform: uppercase;
      letter-spacing: 0.5px;
    }

    .info-value {
      font-size: 14px;
      color: #262626;
      font-weight: 500;
    }

    .info-value.primary {
      color: #1890ff;
      font-weight: 600;
      font-size: 15px;
    }

    .info-value.mono {
      font-family: 'Consolas', 'Monaco', 'Courier New', monospace;
      background: #f5f5f5;
      padding: 4px 8px;
      border-radius: 4px;
      font-size: 13px;
    }

    /* 模块类型徽章 */
    .module-type-badge {
      display: inline-block;
      padding: 4px 12px;
      border-radius: 12px;
      font-size: 12px;
      font-weight: 600;
      text-transform: uppercase;
      letter-spacing: 0.5px;
    }

    .module-type-badge[data-type="AI"] {
      background: linear-gradient(135deg, #e6f7ff 0%, #bae7ff 100%);
      color: #1890ff;
      border: 1px solid #91d5ff;
    }

    .module-type-badge[data-type="AO"] {
      background: linear-gradient(135deg, #f6ffed 0%, #d9f7be 100%);
      color: #52c41a;
      border: 1px solid #95de64;
    }

    .module-type-badge[data-type="DI"] {
      background: linear-gradient(135deg, #fff7e6 0%, #ffd591 100%);
      color: #fa8c16;
      border: 1px solid #ffc069;
    }

    .module-type-badge[data-type="DO"] {
      background: linear-gradient(135deg, #fff2f0 0%, #ffccc7 100%);
      color: #ff4d4f;
      border: 1px solid #ff7875;
    }

    /* 状态徽章 */
    .status-badge {
      display: inline-block;
      padding: 4px 12px;
      border-radius: 12px;
      font-size: 12px;
      font-weight: 600;
      letter-spacing: 0.3px;
    }

    .status-badge[data-status="TestCompletedPassed"] {
      background: linear-gradient(135deg, #f6ffed 0%, #d9f7be 100%);
      color: #52c41a;
      border: 1px solid #95de64;
    }

    .status-badge[data-status="TestCompletedFailed"] {
      background: linear-gradient(135deg, #fff2f0 0%, #ffccc7 100%);
      color: #ff4d4f;
      border: 1px solid #ff7875;
    }

    .status-badge[data-status="NotTested"] {
      background: linear-gradient(135deg, #f5f5f5 0%, #e8e8e8 100%);
      color: #8c8c8c;
      border: 1px solid #d9d9d9;
    }

    .status-badge[data-status="Skipped"] {
      background: linear-gradient(135deg, #f5f5f5 0%, #e8e8e8 100%);
      color: #8c8c8c;
      border: 1px solid #d9d9d9;
    }

    .status-badge[data-status="WiringConfirmationRequired"],
    .status-badge[data-status="ManualTestInProgress"],
    .status-badge[data-status="ManualTesting"] {
      background: linear-gradient(135deg, #fff7e6 0%, #ffd591 100%);
      color: #fa8c16;
      border: 1px solid #ffc069;
    }

    .status-badge[data-status="WiringConfirmed"],
    .status-badge[data-status="HardPointTestInProgress"],
    .status-badge[data-status="HardPointTesting"] {
      background: linear-gradient(135deg, #e6f7ff 0%, #bae7ff 100%);
      color: #1890ff;
      border: 1px solid #91d5ff;
    }

    .status-badge[data-status="HardPointTestCompleted"] {
      background: linear-gradient(135deg, #f6ffed 0%, #d9f7be 100%);
      color: #52c41a;
      border: 1px solid #95de64;
    }

    .status-badge[data-status="AlarmTesting"] {
      background: linear-gradient(135deg, #f9f0ff 0%, #efdbff 100%);
      color: #722ed1;
      border: 1px solid #b37feb;
    }

    .status-badge[data-status="Retesting"] {
      background: linear-gradient(135deg, #fffbe6 0%, #fff1b8 100%);
      color: #faad14;
      border: 1px solid #ffd666;
    }

    /* 错误列表 */
    .error-list {
      padding: 20px;
      display: flex;
      flex-direction: column;
      gap: 16px;
    }

    .error-item {
      display: flex;
      gap: 16px;
      padding: 16px;
      border-radius: 8px;
      background: #ffffff;
      border: 1px solid #f0f0f0;
      transition: all 0.2s ease;
    }

    .error-item:hover {
      transform: translateY(-1px);
      box-shadow: 0 4px 12px rgba(0, 0, 0, 0.08);
    }

    .error-item.danger {
      border-left: 4px solid #ff4d4f;
      background: linear-gradient(135deg, #ffffff 0%, #fff2f0 100%);
    }

    .error-item.warning {
      border-left: 4px solid #fa8c16;
      background: linear-gradient(135deg, #ffffff 0%, #fff7e6 100%);
    }

    .error-item.system {
      border-left: 4px solid #722ed1;
      background: linear-gradient(135deg, #ffffff 0%, #f9f0ff 100%);
    }

    .error-item.info {
      border-left: 4px solid #1890ff;
      background: linear-gradient(135deg, #ffffff 0%, #e6f7ff 100%);
    }

    .error-icon {
      flex-shrink: 0;
      width: 24px;
      height: 24px;
      display: flex;
      align-items: center;
      justify-content: center;
      border-radius: 50%;
    }

    .error-item.danger .error-icon {
      background: #ff4d4f;
      color: white;
    }

    .error-item.warning .error-icon {
      background: #fa8c16;
      color: white;
    }

    .error-item.system .error-icon {
      background: #722ed1;
      color: white;
    }

    .error-item.info .error-icon {
      background: #1890ff;
      color: white;
    }

    .error-content {
      flex: 1;
    }

    .error-title {
      font-weight: 600;
      color: #262626;
      margin-bottom: 6px;
      font-size: 14px;
    }

    .error-detail {
      color: #595959;
      line-height: 1.5;
      margin-bottom: 8px;
      font-size: 13px;
    }

    .error-time {
      display: flex;
      align-items: center;
      gap: 6px;
      color: #8c8c8c;
      font-size: 12px;
    }

    /* 错误备注表单 */
    .notes-form {
      padding: 20px;
      display: flex;
      flex-direction: column;
      gap: 24px;
    }

    .note-section {
      border-radius: 8px;
      border: 1px solid #f0f0f0;
      transition: all 0.2s ease;
      overflow: hidden;
    }

    .note-section:hover {
      border-color: #d9d9d9;
      box-shadow: 0 2px 8px rgba(0, 0, 0, 0.06);
    }

    .note-section.integration {
      border-left: 4px solid #52c41a;
    }

    .note-section.plc {
      border-left: 4px solid #fa8c16;
    }

    .note-section.hmi {
      border-left: 4px solid #1890ff;
    }

    .note-header {
      padding: 16px 20px;
      background: linear-gradient(135deg, #fafbfc 0%, #f8f9fa 100%);
      border-bottom: 1px solid #f0f0f0;
      display: flex;
      align-items: center;
      gap: 12px;
    }

    .note-icon {
      width: 32px;
      height: 32px;
      border-radius: 8px;
      display: flex;
      align-items: center;
      justify-content: center;
      font-size: 16px;
    }

    .note-section.integration .note-icon {
      background: linear-gradient(135deg, #f6ffed 0%, #d9f7be 100%);
      color: #52c41a;
    }

    .note-section.plc .note-icon {
      background: linear-gradient(135deg, #fff7e6 0%, #ffd591 100%);
      color: #fa8c16;
    }

    .note-section.hmi .note-icon {
      background: linear-gradient(135deg, #e6f7ff 0%, #bae7ff 100%);
      color: #1890ff;
    }

    .note-info {
      flex: 1;
    }

    .note-title {
      font-weight: 600;
      color: #262626;
      font-size: 14px;
      margin-bottom: 2px;
    }

    .note-subtitle {
      color: #8c8c8c;
      font-size: 12px;
    }

    .note-textarea {
      border: none !important;
      border-radius: 0 !important;
      box-shadow: none !important;
      padding: 16px 20px !important;
      font-size: 14px !important;
      line-height: 1.6 !important;
      resize: none !important;
      background: #ffffff !important;
    }

    .note-textarea:focus {
      box-shadow: none !important;
      background: #fafbfc !important;
    }

    /* 响应式设计 */
    @media (max-width: 768px) {
      .modal-container {
        width: 95vw;
        margin: 20px;
      }
      
      .info-grid {
        grid-template-columns: 1fr;
        gap: 12px;
      }
      
      .modal-body {
        padding: 20px;
      }
      
      .error-item {
        padding: 12px;
      }
    }

    .info-item {
      display: flex;
      align-items: center;
      gap: 8px;
    }

    .info-item .label {
      color: #8c8c8c;
      font-size: 12px;
      min-width: 80px;
    }

    .error-notes-section h4 {
      color: #1890ff;
      font-weight: 600;
      margin-bottom: 8px;
    }

    .section-description {
      color: #595959;
      font-size: 13px;
      margin-bottom: 20px;
      font-style: italic;
    }

    .notes-region {
      margin-bottom: 24px;
      border: 1px solid #f0f0f0;
      border-radius: 8px;
      padding: 20px;
      transition: all 0.3s ease;
      min-height: 120px;
    }

    .notes-region:hover {
      border-color: #d9d9d9;
      box-shadow: 0 2px 8px rgba(0, 0, 0, 0.05);
    }

    .integration-region {
      background: linear-gradient(135deg, #f6ffed 0%, #f9f9f9 100%);
      border-left: 4px solid #52c41a;
    }

    .plc-region {
      background: linear-gradient(135deg, #fff7e6 0%, #f9f9f9 100%);
      border-left: 4px solid #fa8c16;
    }

    .hmi-region {
      background: linear-gradient(135deg, #e6f7ff 0%, #f9f9f9 100%);
      border-left: 4px solid #1890ff;
    }

    .region-header {
      display: flex;
      align-items: center;
      gap: 8px;
      margin-bottom: 12px;
    }

    .region-header i {
      font-size: 16px;
    }

    .integration-region .region-header i {
      color: #52c41a;
    }

    .plc-region .region-header i {
      color: #fa8c16;
    }

    .hmi-region .region-header i {
      color: #1890ff;
    }

    .region-title {
      font-weight: 600;
      color: #262626;
      font-size: 14px;
    }

    .region-subtitle {
      color: #8c8c8c;
      font-size: 12px;
      font-style: italic;
    }

    .notes-textarea {
      width: 100%;
      border: 1px solid #d9d9d9;
      border-radius: 6px;
      font-size: 14px;
      line-height: 1.6;
      padding: 12px;
      font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
    }

    .notes-textarea:focus {
      border-color: #40a9ff;
      box-shadow: 0 0 0 2px rgba(24, 144, 255, 0.2);
    }

    /* 响应式设计 */
    @media (max-width: 768px) {
      .info-grid {
        grid-template-columns: 1fr;
      }
      .notes-region {
        padding: 16px;
        margin-bottom: 20px;
      }
      .notes-textarea {
        font-size: 13px;
        padding: 10px;
      }
    }

    /* 大屏幕优化 */
    @media (min-width: 1200px) {
      .notes-region {
        padding: 24px;
      }
      .channel-info-section {
        padding: 24px;
      }
    }
  `]
})
export class ErrorNotesModalComponent implements OnInit, OnDestroy, OnChanges {
  @Input() visible = false;
  @Input() instance: ChannelTestInstance | null = null;
  @Input() definition: ChannelPointDefinition | null = null;
  
  @Output() visibleChange = new EventEmitter<boolean>();
  @Output() notesSaved = new EventEmitter<void>();

  // 错误备注数据
  errorNotes = {
    integration: '',  // 集成错误备注
    plc: '',         // PLC编程错误备注
    hmi: ''          // 上位机组态错误备注
  };

  isSaving = false;

  constructor(
    private tauriApiService: TauriApiService,
    private message: NzMessageService
  ) {}

  ngOnInit(): void {
    // 组件初始化时加载现有的错误备注
    this.loadExistingNotes().catch(console.error);
  }

  ngOnChanges(changes: SimpleChanges): void {
    // 当输入属性变化时重新加载数据
    if (changes['visible'] && changes['visible'].currentValue) {
      this.loadExistingNotes().catch(console.error);
    }
    if (changes['instance'] || changes['definition']) {
      this.loadExistingNotes().catch(console.error);
    }
  }

  ngOnDestroy(): void {
    // 组件销毁时重置数据
    this.resetForm();
  }

  get modalTitle(): string {
    return this.definition ? `错误备注 - ${this.definition.tag}` : '错误备注';
  }

  /**
   * 加载现有的错误备注（从服务端获取最新数据）
   */
  private async loadExistingNotes(): Promise<void> {
    if (!this.instance) {
      return;
    }

    try {
      console.log('🔄 [ERROR_NOTES_MODAL] 从服务端加载最新错误备注:', this.instance.instance_id);
      
      // 从服务端获取最新的实例数据，确保备注信息是最新的
      const latestInstance = await firstValueFrom(
        this.tauriApiService.getTestInstanceDetails(this.instance.instance_id)
      );
      
      if (latestInstance) {
        // 使用最新数据更新错误备注
        this.errorNotes.integration = latestInstance.integration_error_notes || '';
        this.errorNotes.plc = latestInstance.plc_programming_error_notes || '';
        this.errorNotes.hmi = latestInstance.hmi_configuration_error_notes || '';
        
        console.log('✅ [ERROR_NOTES_MODAL] 最新错误备注加载成功:', this.errorNotes);
      } else {
        // 如果获取失败，使用传入的instance数据作为备选
        console.warn('⚠️ [ERROR_NOTES_MODAL] 无法获取最新数据，使用缓存数据');
        this.errorNotes.integration = this.instance.integration_error_notes || '';
        this.errorNotes.plc = this.instance.plc_programming_error_notes || '';
        this.errorNotes.hmi = this.instance.hmi_configuration_error_notes || '';
      }
    } catch (error) {
      console.error('❌ [ERROR_NOTES_MODAL] 加载错误备注失败，使用缓存数据:', error);
      // 如果发生错误，使用传入的instance数据作为备选
      this.errorNotes.integration = this.instance.integration_error_notes || '';
      this.errorNotes.plc = this.instance.plc_programming_error_notes || '';
      this.errorNotes.hmi = this.instance.hmi_configuration_error_notes || '';
    }
  }

  /**
   * 重置表单数据
   */
  private resetForm(): void {
    this.errorNotes = {
      integration: '',
      plc: '',
      hmi: ''
    };
    this.isSaving = false;
  }

  /**
   * 获取模块类型颜色
   */
  getModuleTypeColor(moduleType: string): string {
    const colorMap: { [key: string]: string } = {
      'AI': 'blue',
      'AO': 'green', 
      'DI': 'orange',
      'DO': 'red',
      'AINone': 'default',
      'DINone': 'default'
    };
    return colorMap[moduleType] || 'default';
  }

  /**
   * 获取状态颜色
   */
  getStatusColor(status: string): string {
    const colorMap: { [key: string]: string } = {
      'TestCompletedPassed': '#52c41a',
      'TestCompletedFailed': '#ff4d4f',
      'NotTested': '#d9d9d9',
      'Skipped': '#8c8c8c',
      'WiringConfirmationRequired': '#fa8c16',
      'WiringConfirmed': '#1890ff',
      'HardPointTestInProgress': '#1890ff',
      'HardPointTesting': '#1890ff',
      'HardPointTestCompleted': '#52c41a',
      'ManualTestInProgress': '#fa8c16',
      'ManualTesting': '#fa8c16',
      'AlarmTesting': '#722ed1',
      'Retesting': '#faad14'
    };
    return colorMap[status] || '#8c8c8c';
  }

  /**
   * 获取状态标签
   */
  getStatusLabel(status: string): string {
    const labelMap: { [key: string]: string } = {
      'TestCompletedPassed': '测试通过',
      'TestCompletedFailed': '测试失败',
      'NotTested': '未测试',
      'Skipped': '已跳过',
      'WiringConfirmationRequired': '待确认接线',
      'WiringConfirmed': '接线已确认',
      'HardPointTestInProgress': '硬点测试进行中',
      'HardPointTesting': '硬点测试中',
      'HardPointTestCompleted': '硬点测试完成',
      'ManualTestInProgress': '手动测试进行中',
      'ManualTesting': '手动测试中',
      'AlarmTesting': '报警测试中',
      'Retesting': '重新测试中'
    };
    return labelMap[status] || status;
  }

  /**
   * 检查是否有任何错误
   */
  hasAnyErrors(): boolean {
    if (!this.instance) return false;
    
    return !!(
      this.getHardPointError() ||
      this.getManualTestError() ||
      this.getFailedTestItems().length > 0
    );
  }

  /**
   * 获取失败的测试项目列表
   */
  getFailedTestItems(): string[] {
    if (!this.instance?.error_message) return [];
    
    const errorMessage = this.instance.error_message;
    
    // 如果错误信息包含"测试失败:"，则提取失败的测试项目
    if (errorMessage.includes('测试失败:')) {
      const failedPart = errorMessage.split('测试失败:')[1];
      if (failedPart) {
        // 按逗号分割并清理空格
        return failedPart.split(',').map(item => item.trim()).filter(item => item.length > 0);
      }
    }
    
    return [];
  }

  /**
   * 获取硬点测试错误信息
   */
  getHardPointError(): string | null {
    if (!this.instance?.sub_test_results) return null;
    
    // 从sub_test_results中查找硬点测试的错误
    for (const [testItem, result] of Object.entries(this.instance.sub_test_results)) {
      if (testItem === 'HardPoint' && result.status === 'Failed') {
        return result.details || '硬点测试失败';
      }
    }
    
    // 也检查hardpoint相关的错误字段  
    if ((this.instance as any).hard_point_error_detail) {
      return (this.instance as any).hard_point_error_detail;
    }
    
    return null;
  }

  /**
   * 获取硬点测试错误时间
   */
  getHardPointErrorTime(): string {
    if (!this.instance?.sub_test_results) return '';
    
    for (const [testItem, result] of Object.entries(this.instance.sub_test_results)) {
      if (testItem === 'HardPoint' && result.status === 'Failed') {
        return result.timestamp ? new Date(result.timestamp).toLocaleString('zh-CN') : '';
      }
    }
    
    return '';
  }

  /**
   * 获取硬点测试实际值
   */
  getHardPointActualValue(): string | null {
    if (!this.instance?.sub_test_results) return null;
    
    for (const [testItem, result] of Object.entries(this.instance.sub_test_results)) {
      if (testItem === 'HardPoint' && result.actual_value) {
        return result.actual_value;
      }
    }
    
    return null;
  }

  /**
   * 获取硬点测试期望值
   */
  getHardPointExpectedValue(): string | null {
    if (!this.instance?.sub_test_results) return null;
    
    for (const [testItem, result] of Object.entries(this.instance.sub_test_results)) {
      if (testItem === 'HardPoint' && result.expected_value) {
        return result.expected_value;
      }
    }
    
    return null;
  }

  /**
   * 获取手动测试错误信息
   */
  getManualTestError(): string | null {
    if (!this.instance?.sub_test_results) return null;
    
    // 从sub_test_results中查找手动测试的错误
    for (const [testItem, result] of Object.entries(this.instance.sub_test_results)) {
      if (testItem === 'ManualTest' && result.status === 'Failed') {
        return result.details || '手动测试失败';
      }
    }
    
    return null;
  }

  /**
   * 获取手动测试错误时间
   */
  getManualTestErrorTime(): string {
    if (!this.instance?.sub_test_results) return '';
    
    for (const [testItem, result] of Object.entries(this.instance.sub_test_results)) {
      if (testItem === 'ManualTest' && result.status === 'Failed') {
        return result.timestamp ? new Date(result.timestamp).toLocaleString('zh-CN') : '';
      }
    }
    
    return '';
  }

  /**
   * 获取错误数量
   */
  getErrorCount(): number {
    let count = 0;
    
    if (this.getHardPointError()) count++;
    if (this.getManualTestError()) count++;
    if (this.getFailedTestItems().length > 0) count++;
    
    return count;
  }


  /**
   * 格式化日期时间
   */
  formatDateTime(dateTime?: string | Date | null): string {
    if (!dateTime) return '';
    
    const date = typeof dateTime === 'string' ? new Date(dateTime) : dateTime;
    return date.toLocaleString('zh-CN', {
      year: 'numeric',
      month: '2-digit',
      day: '2-digit',
      hour: '2-digit',
      minute: '2-digit',
      second: '2-digit'
    });
  }

  /**
   * 获取测试阶段
   */
  getTestPhase(): string {
    if (!this.instance) return '未知';
    
    const status = this.instance.overall_status;
    
    if (status === 'NotTested') {
      return '测试准备阶段';
    } else if (status === 'WiringConfirmationRequired') {
      return '接线确认阶段';
    } else if (status === 'WiringConfirmed') {
      return '接线已确认';
    } else if (status === 'HardPointTestInProgress' || status === 'HardPointTesting') {
      return '硬点测试阶段';
    } else if (status === 'HardPointTestCompleted') {
      return '硬点测试完成';
    } else if (status === 'ManualTestInProgress' || status === 'ManualTesting') {
      return '手动测试阶段';
    } else if (status === 'AlarmTesting') {
      return '报警测试阶段';
    } else if (status === 'TestCompletedPassed') {
      return '测试完成（通过）';
    } else if (status === 'TestCompletedFailed') {
      return '测试完成（失败）';
    } else if (status === 'Retesting') {
      return '重新测试';
    } else if (status === 'Skipped') {
      return '已跳过';
    } else {
      return '未知阶段';
    }
  }

  /**
   * 取消按钮点击事件
   */
  onCancel(): void {
    this.resetForm();
    this.visible = false;
    this.visibleChange.emit(false);
  }

  /**
   * 保存错误备注
   */
  async onSave(): Promise<void> {
    if (!this.instance) {
      this.message.error('实例信息缺失');
      return;
    }

    // 检查是否至少有一个备注不为空
    const hasNotes = this.errorNotes.integration.trim() || 
                     this.errorNotes.plc.trim() || 
                     this.errorNotes.hmi.trim();

    if (!hasNotes) {
      this.message.warning('请至少输入一条错误备注');
      return;
    }

    this.isSaving = true;

    try {
      console.log('💾 [ERROR_NOTES_MODAL] 保存错误备注:', this.instance.instance_id);
      console.log('📝 [ERROR_NOTES_MODAL] 备注内容:', this.errorNotes);

      // 调用后端API保存错误备注
      await firstValueFrom(this.tauriApiService.saveErrorNotes(
        this.instance.instance_id,
        this.errorNotes.integration.trim() || null,
        this.errorNotes.plc.trim() || null,
        this.errorNotes.hmi.trim() || null
      ));

      console.log('✅ [ERROR_NOTES_MODAL] 错误备注保存成功');
      this.message.success('错误备注保存成功');
      
      // 发射保存完成事件
      this.notesSaved.emit();
      
      // 关闭模态框
      this.onCancel();

    } catch (error) {
      console.error('❌ [ERROR_NOTES_MODAL] 保存错误备注失败:', error);
      this.message.error(`保存错误备注失败: ${error}`);
    } finally {
      this.isSaving = false;
    }
  }
}