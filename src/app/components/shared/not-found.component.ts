import { Component } from '@angular/core';
import { CommonModule } from '@angular/common';
import { RouterLink } from '@angular/router';

@Component({
  selector: 'app-not-found',
  standalone: true,
  imports: [CommonModule, RouterLink],
  template: `
    <div class="not-found-container">
      <div class="not-found-content">
        <div class="error-code">404</div>
        <h1>页面未找到</h1>
        <p>抱歉，您访问的页面不存在或已被移动。</p>
        <div class="actions">
          <a routerLink="/dashboard" class="btn-primary">返回首页</a>
          <button class="btn-secondary" (click)="goBack()">返回上页</button>
        </div>
      </div>
    </div>
  `,
  styles: [`
    .not-found-container {
      display: flex;
      align-items: center;
      justify-content: center;
      min-height: 60vh;
      text-align: center;
    }

    .not-found-content {
      max-width: 500px;
      padding: 2rem;
    }

    .error-code {
      font-size: 8rem;
      font-weight: 700;
      color: #e74c3c;
      margin-bottom: 1rem;
      line-height: 1;
    }

    h1 {
      font-size: 2rem;
      color: #2c3e50;
      margin-bottom: 1rem;
    }

    p {
      color: #7f8c8d;
      font-size: 1.1rem;
      margin-bottom: 2rem;
    }

    .actions {
      display: flex;
      gap: 1rem;
      justify-content: center;
      flex-wrap: wrap;
    }

    .btn-primary, .btn-secondary {
      padding: 0.75rem 1.5rem;
      border-radius: 8px;
      font-weight: 500;
      text-decoration: none;
      border: none;
      cursor: pointer;
      transition: all 0.3s ease;
    }

    .btn-primary {
      background: #3498db;
      color: white;
    }

    .btn-primary:hover {
      background: #2980b9;
      transform: translateY(-1px);
    }

    .btn-secondary {
      background: #ecf0f1;
      color: #2c3e50;
    }

    .btn-secondary:hover {
      background: #d5dbdb;
      transform: translateY(-1px);
    }
  `]
})
export class NotFoundComponent {
  goBack(): void {
    window.history.back();
  }
} 