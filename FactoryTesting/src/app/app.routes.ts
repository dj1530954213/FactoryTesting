import { Routes } from '@angular/router';

export const routes: Routes = [
  {
    path: '',
    redirectTo: '/dashboard',
    pathMatch: 'full'
  },
  {
    path: 'dashboard',
    loadComponent: () => import('./components/dashboard/dashboard.component').then(m => m.DashboardComponent)
  },
  {
    path: 'data-import',
    loadComponent: () => import('./components/data-import/data-import.component').then(m => m.DataImportComponent)
  },
  {
    path: 'test-execution',
    loadComponent: () => import('./components/test-execution/test-execution.component').then(m => m.TestExecutionComponent)
  },
  {
    path: 'batch-management',
    loadComponent: () => import('./components/batch-management/batch-management.component').then(m => m.BatchManagementComponent)
  },
  {
    path: 'manual-test',
    loadComponent: () => import('./components/manual-test/manual-test.component').then(m => m.ManualTestComponent)
  },
  {
    path: '**',
    loadComponent: () => import('./components/shared/not-found.component').then(m => m.NotFoundComponent)
  }
];
