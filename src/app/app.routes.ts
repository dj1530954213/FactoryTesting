import { Routes } from '@angular/router';

export const routes: Routes = [
  // 默认路由重定向到仪表板
  {
    path: '',
    redirectTo: '/dashboard',
    pathMatch: 'full'
  },
  
  // 仪表板页面
  {
    path: 'dashboard',
    loadComponent: () => import('./components/dashboard/dashboard.component').then(m => m.DashboardComponent),
    title: '仪表板 - FAT_TEST'
  },
  
  // 通道配置页面
  {
    path: 'channel-config',
    loadComponent: () => import('./components/channel-configuration/channel-configuration.component').then(m => m.ChannelConfigurationComponent),
    title: '通道配置 - FAT_TEST'
  },
  
  // 批次管理页面
  {
    path: 'batch-management',
    loadComponent: () => import('./components/batch-management/batch-management.component').then(m => m.BatchManagementComponent),
    title: '批次管理 - FAT_TEST'
  },
  
  // 测试执行页面
  {
    path: 'test-execution',
    loadComponent: () => import('./components/test-execution/test-execution.component').then(m => m.TestExecutionComponent),
    title: '测试执行 - FAT_TEST'
  },
  
  // 手动测试页面
  {
    path: 'manual-test',
    loadComponent: () => import('./components/manual-test/manual-test.component').then(m => m.ManualTestComponent),
    title: '手动测试 - FAT_TEST'
  },
  
  // 数据导入页面
  {
    path: 'data-import',
    loadComponent: () => import('./components/data-import/data-import.component').then(m => m.DataImportComponent),
    title: '数据导入 - FAT_TEST'
  },
  
  // 测试执行详情页面（带参数）
  {
    path: 'test-execution/:batchId',
    loadComponent: () => import('./components/test-execution/test-execution-detail.component').then(m => m.TestExecutionDetailComponent),
    title: '测试执行详情 - FAT_TEST'
  },
  
  // 通道详情页面（带参数）
  {
    path: 'channel/:channelId',
    loadComponent: () => import('./components/channel-configuration/channel-detail.component').then(m => m.ChannelDetailComponent),
    title: '通道详情 - FAT_TEST'
  },
  
  // 404 页面
  {
    path: '**',
    loadComponent: () => import('./components/shared/not-found.component').then(m => m.NotFoundComponent),
    title: '页面未找到 - FAT_TEST'
  }
]; 