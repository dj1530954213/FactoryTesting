/**
 * # 应用配置模块 - AppConfig
 * 
 * ## 业务功能说明
 * - 配置Angular应用的全局服务提供者
 * - 设置国际化、路由、动画等核心功能
 * - 配置全局错误处理机制
 * - 集成第三方库配置 (NG-ZORRO、ECharts)
 * 
 * ## 前后端调用链
 * - **HTTP客户端**: 通过provideHttpClient提供HTTP服务用于API调用
 * - **路由配置**: 配置前端路由，支持单页应用导航
 * 
 * ## Angular知识点
 * - **ApplicationConfig**: Angular 17+ 的配置方式，替代传统的NgModule
 * - **依赖注入**: 使用providers数组配置全局服务
 * - **惰性加载**: provideAnimationsAsync支持动画的按需加载
 * - **国际化**: 配置中文本地化支持
 * 
 * ## 第三方集成
 * - **NG-ZORRO**: Ant Design的Angular实现，提供UI组件
 * - **ECharts**: 图表库，用于数据可视化
 * - **HttpClient**: Angular的HTTP客户端，用于与后端通信
 * 
 * ## 错误处理策略
 * - 全局错误处理器捕获所有未处理的异常
 * - 避免应用崩溃，提供更好的用户体验
 */

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

// 注册中文本地化数据，支持日期、数字等格式的中文显示
registerLocaleData(zh);

/**
 * 全局错误处理器
 * 
 * **业务作用**: 捕获所有未处理的异常，防止应用崩溃
 * **处理策略**: 记录详细错误信息但不重新抛出
 * **适用场景**: 生产环境的错误监控和调试
 */
@Injectable()
export class GlobalErrorHandler implements ErrorHandler {
  /**
   * 处理全局错误
   * 
   * **业务流程**:
   * 1. 记录错误的基本信息
   * 2. 记录错误堆栈用于调试
   * 3. 记录结构化错误详情
   * 4. 不重新抛出，避免应用崩溃
   * 
   * **错误信息包含**:
   * - 错误消息
   * - 错误名称和类型
   * - 错误原因和堆栈
   * 
   * @param error 捕获到的错误对象
   */
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
    // 在生产环境中，可以考虑将错误发送到远程日志服务
    // throw error;
  }
}

/**
 * 应用配置对象
 * 
 * **配置说明**: 使用Angular 17+的standalone方式配置应用
 * **配置项目**:
 * - 变更检测优化
 * - 路由配置
 * - 动画支持
 * - 国际化设置
 * - HTTP客户端
 * - 图表库配置
 * - 全局错误处理
 */
export const appConfig: ApplicationConfig = {
  providers: [
    // 变更检测优化：启用事件合并以提高性能
    provideZoneChangeDetection({ eventCoalescing: true }),
    
    // 路由配置：提供应用的路由服务
    provideRouter(routes),
    
    // 动画支持：异步加载动画模块，减少初始包体积
    provideAnimationsAsync(),
    
    // 国际化配置：设置为中文环境
    provideNzI18n(zh_CN),
    
    // 表单模块：支持模板驱动表单和响应式表单
    importProvidersFrom(FormsModule),
    
    // HTTP客户端：用于与后端API通信
    provideHttpClient(),
    
    // ECharts配置：图表库的动态导入配置
    provideEchartsCore({
      echarts: () => import('echarts')
    }),
    
    // 全局错误处理：注册自定义错误处理器
    { provide: ErrorHandler, useClass: GlobalErrorHandler }
  ]
};
