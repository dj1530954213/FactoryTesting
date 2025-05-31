import { ApplicationConfig, provideZoneChangeDetection, importProvidersFrom, ErrorHandler, Injectable } from '@angular/core';
import { provideRouter } from '@angular/router';

import { routes } from './app.routes';
import { provideAnimationsAsync } from '@angular/platform-browser/animations/async';
import { zh_CN, provideNzI18n } from 'ng-zorro-antd/i18n';
import { registerLocaleData } from '@angular/common';
import zh from '@angular/common/locales/zh';
import { FormsModule } from '@angular/forms';
import { provideHttpClient } from '@angular/common/http';

// ECharts 配置
import { provideEchartsCore } from 'ngx-echarts';

registerLocaleData(zh);

@Injectable()
export class GlobalErrorHandler implements ErrorHandler {
  handleError(error: any): void {
    console.error('全局错误处理器捕获到错误:', error);
    console.error('错误堆栈:', error.stack);
    console.error('错误详情:', {
      message: error.message,
      name: error.name,
      cause: error.cause,
      stack: error.stack
    });

    // 不重新抛出错误，避免应用崩溃
    // throw error;
  }
}

export const appConfig: ApplicationConfig = {
  providers: [
    provideZoneChangeDetection({ eventCoalescing: true }),
    provideRouter(routes),
    provideAnimationsAsync(),
    provideNzI18n(zh_CN),
    importProvidersFrom(FormsModule),
    provideHttpClient(),
    provideEchartsCore({
      echarts: () => import('echarts')
    }),
    { provide: ErrorHandler, useClass: GlobalErrorHandler }
  ]
};
