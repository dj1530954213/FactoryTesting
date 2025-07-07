import { Routes } from '@angular/router';

export const routes: Routes = [
  {
    path: '',
    redirectTo: '/dashboard',
    pathMatch: 'full'
  },
  // 主要功能区域路由
  {
    path: 'dashboard',
    loadComponent: () => import('./components/dashboard/dashboard.component').then(m => m.DashboardComponent)
  },
  {
    path: 'test-plc-config',
    loadComponent: () => import('./components/test-plc-config/test-plc-config.component').then(m => m.TestPlcConfigComponent)
  },
  {
    path: 'data-management',
    loadComponent: () => import('./components/data-management/data-management.component').then(m => m.DataManagementComponent)
  },
  {
    path: 'test-area',
    loadComponent: () => import('./components/test-area/test-area.component').then(m => m.TestAreaComponent)
  },
  {
    path: 'host-function-check',
    loadComponent: () => import('./components/host-function-check/host-function-check.component').then(m => m.HostFunctionCheckComponent)
  },
  {
    path: 'result-export',
    loadComponent: () => import('./components/result-export/result-export.component').then(m => m.ResultExportComponent)
  },
  {
    path: 'system-settings',
    loadComponent: () => import('./components/system-settings/system-settings.component').then(m => m.SystemSettingsComponent)
  },
  
  // 向后兼容的路由重定向
  {
    path: 'data-import',
    redirectTo: '/data-management',
    pathMatch: 'full'
  },
  {
    path: 'test-execution',
    redirectTo: '/test-area',
    pathMatch: 'full'
  },
  {
    path: 'batch-management',
    redirectTo: '/test-area',
    pathMatch: 'full'
  },
  {
    path: 'manual-test',
    redirectTo: '/test-area',
    pathMatch: 'full'
  },
  {
    path: 'settings',
    redirectTo: '/system-settings',
    pathMatch: 'full'
  },
  {
    path: 'report-generation',
    redirectTo: '/result-export',
    pathMatch: 'full'
  },
  {
    path: 'system-monitor',
    redirectTo: '/dashboard',
    pathMatch: 'full'
  },
  
  // 404页面
  {
    path: '**',
    loadComponent: () => import('./components/shared/not-found.component').then(m => m.NotFoundComponent)
  }
];
