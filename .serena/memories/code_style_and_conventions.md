# 代码风格和约定

## Angular/TypeScript约定
- 使用TypeScript严格模式
- 组件命名：PascalCase (AiManualTestComponent)
- 服务命名：PascalCase + Service后缀 (ManualTestService)
- 变量命名：camelCase
- 常量命名：UPPER_SNAKE_CASE
- 接口命名：PascalCase
- 枚举命名：PascalCase

## Rust约定
- 结构体：PascalCase (ChannelPointDefinition)
- 函数/方法：snake_case (execute_show_value_test)
- 变量：snake_case (range_low_limit)
- 常量：UPPER_SNAKE_CASE
- 模块：snake_case

## 代码组织
- 每个组件都有对应的.ts/.html/.css文件
- 服务使用Angular的Injectable装饰器
- 数据模型定义在models目录下
- 使用barrel pattern统一导出(index.ts)

## 注释约定
- 使用详细的JSDoc注释说明业务用途
- 中文注释解释业务含义
- 英文注释解释技术实现

## UI约定
- 使用ng-zorro-antd组件库
- 统一的卡片布局和间距
- 颜色系统：
  - 成功：绿色 (#52c41a)
  - 警告：橙色 (#faad14) 
  - 错误：红色 (#ff4d4f)
  - 主色：蓝色 (#1890ff)

## 数据流约定
- 使用RxJS Observable进行异步数据处理
- 服务层使用BehaviorSubject管理状态
- 组件间通信使用@Input/@Output或服务