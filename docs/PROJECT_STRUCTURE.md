# FAT_TEST 项目目录结构

## 项目结构概览

本文档描述了 FAT_TEST 项目的完整目录结构，基于 Rust + Angular + Tauri 技术栈。

## 根目录结构

```
FactoryTesting/                                # Git 仓库根目录
├── FactoryTesting/                            # Tauri 项目根目录
│   ├── README.md                              # 项目说明文档
│   ├── package.json                           # npm 依赖和脚本配置
│   ├── package-lock.json                      # npm 依赖锁定文件
│   ├── angular.json                           # Angular 项目配置
│   ├── tsconfig.json                          # TypeScript 配置
│   ├── tsconfig.app.json                      # Angular 应用 TypeScript 配置
│   ├── tsconfig.spec.json                     # 测试 TypeScript 配置
│   ├── .editorconfig                          # 编辑器配置
│   ├── .gitignore                             # Git 忽略文件配置
│   │
│   ├── src/                                   # Angular 前端源代码
│   ├── src-tauri/                             # Rust 后端 + Tauri 集成
│   ├── docs/                                  # 项目文档
│   ├── scripts/                               # 构建和部署脚本
│   ├── tests/                                 # 端到端集成测试
│   ├── config/                                # 配置文件
│   ├── data/                                  # 运行时数据目录
│   ├── public/                                # 静态资源
│   ├── node_modules/                          # Node.js 依赖
│   ├── .angular/                              # Angular 构建缓存
│   └── .vscode/                               # VS Code 配置
├── Notes/                                     # 技术栈迁移文档
├── modbus通讯示例代码/                        # 现有的示例代码
└── .gitignore                                 # Git 忽略文件配置
```

## Rust 后端结构 (src-tauri/)

```
src-tauri/
├── Cargo.toml                                 # Rust 依赖配置
├── Cargo.lock                                 # Rust 依赖锁定文件
├── tauri.conf.json                            # Tauri 应用配置
├── build.rs                                   # 构建脚本
├── icons/                                     # 应用图标资源
├── capabilities/                              # Tauri 权限配置
├── gen/                                       # 生成的代码
├── target/                                    # Rust 构建输出
│
└── src/                                       # Rust 源代码
    ├── main.rs                                # Tauri 应用入口点
    ├── lib.rs                                 # 库根模块
    │
    ├── commands/                              # Tauri Commands (前端调用接口)
    │   ├── mod.rs                             # 命令模块根文件
    │   ├── batch_management.rs                # 批次管理相关命令
    │   ├── test_orchestration.rs              # 测试编排相关命令
    │   ├── manual_test.rs                     # 手动测试相关命令
    │   ├── data_management.rs                 # 数据管理相关命令
    │   ├── channel_config.rs                  # 通道配置相关命令
    │   └── system_settings.rs                 # 系统设置相关命令
    │
    ├── events/                                # Tauri Events (后端推送给前端)
    │   ├── mod.rs                             # 事件模块根文件
    │   ├── channel_state_events.rs            # 通道状态变更事件
    │   ├── batch_progress_events.rs           # 批次进度事件
    │   ├── log_events.rs                      # 日志事件
    │   └── system_events.rs                   # 系统状态事件
    │
    ├── state/                                 # Tauri 状态管理
    │   ├── mod.rs                             # 状态模块根文件
    │   ├── app_state.rs                       # 应用全局状态
    │   └── service_container.rs               # 服务容器/依赖注入
    │
    ├── models/                                # 数据模型层
    │   ├── mod.rs                             # 模型模块根文件
    │   ├── enums.rs                           # 核心枚举定义
    │   ├── structs.rs                         # 核心结构体定义
    │   ├── command_payloads.rs                # Tauri Command 参数/响应模型
    │   ├── event_payloads.rs                  # Tauri Event Payload 模型
    │   └── settings.rs                        # 应用设置数据模型
    │
    ├── services/                              # 业务服务层
    │   ├── mod.rs                             # 服务模块根文件
    │   │
    │   ├── application/                       # 应用层服务
    │   │   ├── mod.rs                         # 应用层模块根文件
    │   │   ├── test_orchestration_service.rs  # 测试编排服务
    │   │   ├── data_management_service.rs     # 数据管理服务
    │   │   ├── manual_test_service.rs         # 手动测试服务
    │   │   ├── channel_config_service.rs      # 通道配置服务
    │   │   └── notification_service.rs        # 通知/事件发布服务
    │   │
    │   ├── domain/                            # 领域服务层
    │   │   ├── mod.rs                         # 领域层模块根文件
    │   │   ├── channel_state_manager.rs       # 通道状态管理器
    │   │   ├── test_execution_engine.rs       # 测试执行引擎
    │   │   ├── specific_test_executors/       # 特定测试步骤执行器
    │   │   │   ├── mod.rs
    │   │   │   ├── ai_test_executor.rs        # AI点测试执行器
    │   │   │   ├── ao_test_executor.rs        # AO点测试执行器
    │   │   │   ├── di_test_executor.rs        # DI点测试执行器
    │   │   │   ├── do_test_executor.rs        # DO点测试执行器
    │   │   │   ├── communication_test_executor.rs # 通信测试
    │   │   │   └── generic_read_write_executor.rs # 通用读写执行器
    │   │   ├── statistics_service.rs          # 统计服务
    │   │   └── test_record_service.rs         # 测试记录服务
    │   │
    │   └── infrastructure/                    # 基础设施层
    │       ├── mod.rs                         # 基础设施层模块根文件
    │       ├── plc/                           # PLC 通信相关
    │       │   ├── mod.rs
    │       │   ├── plc_communication_service.rs  # PLC通信服务接口
    │       │   ├── mock_plc_service.rs           # Mock PLC服务
    │       │   ├── modbus_plc_service.rs         # Modbus TCP PLC服务
    │       │   ├── s7_plc_service.rs             # Siemens S7 PLC服务
    │       │   └── opcua_plc_service.rs          # OPC UA PLC服务
    │       ├── persistence/                   # 数据持久化相关
    │       │   ├── mod.rs
    │       │   ├── persistence_service.rs       # 持久化服务接口
    │       │   ├── json_file_persistence.rs     # JSON文件持久化
    │       │   ├── sqlite_persistence.rs        # SQLite数据库持久化
    │       │   └── mock_persistence.rs          # Mock持久化服务
    │       └── excel/                         # Excel 处理相关
    │           ├── mod.rs
    │           ├── excel_importer.rs            # Excel导入器
    │           └── excel_exporter.rs            # Excel导出器
    │
    ├── utils/                                 # 工具模块
    │   ├── mod.rs                             # 工具模块根文件
    │   ├── error.rs                           # 错误类型定义
    │   ├── logging.rs                         # 日志配置
    │   ├── config.rs                          # 配置加载/管理
    │   └── conversion.rs                      # 数据类型转换工具
    │
    └── tests/                                 # 集成测试
        ├── mod.rs
        ├── command_tests.rs                   # Tauri Command 测试
        ├── service_integration_tests.rs       # 服务集成测试
        └── mock_data.rs                       # 测试数据生成
```

## Angular 前端结构 (src/)

```
src/
├── main.ts                                    # Angular 应用启动入口
├── index.html                                 # HTML 入口
├── styles.css                                 # 全局样式
│
└── app/                                       # Angular 应用根模块
    ├── app.component.ts                       # 根组件
    ├── app.component.html                     # 根组件模板
    ├── app.component.css                      # 根组件样式
    ├── app.component.spec.ts                  # 根组件测试
    ├── app.config.ts                          # 应用配置
    ├── app.routes.ts                          # 路由配置
    │
    ├── models/                                # TypeScript 数据模型
    │   ├── index.ts                           # 模型导出索引
    │   ├── backend-models.ts                  # 后端数据模型接口
    │   ├── command-models.ts                  # Tauri Command 参数/响应接口
    │   ├── event-models.ts                    # Tauri Event Payload 接口
    │   ├── ui-models.ts                       # 前端特有的UI模型
    │   └── form-models.ts                     # 表单数据模型
    │
    ├── services/                              # Angular 服务层
    │   ├── index.ts                           # 服务导出索引
    │   ├── backend-comms.service.ts           # Tauri 通信封装服务
    │   ├── test-orchestration-api.service.ts # 测试编排API服务
    │   ├── data-management-api.service.ts    # 数据管理API服务
    │   ├── manual-test-api.service.ts        # 手动测试API服务
    │   ├── channel-config-api.service.ts     # 通道配置API服务
    │   ├── event-listener.service.ts         # Tauri事件监听服务
    │   ├── ui-state.service.ts               # UI状态管理服务
    │   ├── notification.service.ts           # 通知/提示服务
    │   └── settings.service.ts               # 设置管理服务
    │
    ├── components/                            # Angular 组件
    │   ├── shared/                            # 共享组件
    │   │   ├── header/                        # 头部组件
    │   │   ├── sidebar/                       # 侧边栏组件
    │   │   ├── loading-spinner/               # 加载动画组件
    │   │   ├── confirmation-dialog/           # 确认对话框组件
    │   │   ├── status-indicator/              # 状态指示器组件
    │   │   ├── progress-bar/                  # 进度条组件
    │   │   └── data-table/                    # 通用数据表格组件
    │   │
    │   ├── batch-management/                  # 批次管理页面
    │   │   ├── batch-list/                    # 批次列表子组件
    │   │   ├── batch-create/                  # 创建批次子组件
    │   │   └── batch-details/                 # 批次详情子组件
    │   │
    │   ├── data-import/                       # 数据导入页面
    │   │   ├── excel-upload/                  # Excel文件上传子组件
    │   │   ├── point-preview/                 # 点位预览子组件
    │   │   └── import-progress/               # 导入进度子组件
    │   │
    │   ├── test-execution/                    # 测试执行页面
    │   │   ├── channel-list/                  # 通道列表子组件
    │   │   ├── channel-detail/                # 通道详情子组件
    │   │   ├── test-control-panel/            # 测试控制面板
    │   │   ├── batch-progress/                # 批次进度显示
    │   │   └── real-time-status/              # 实时状态监控
    │   │
    │   ├── manual-test/                       # 手动测试页面
    │   │   ├── manual-test-panel/             # 手动测试操作面板
    │   │   ├── value-input/                   # 数值输入子组件
    │   │   ├── alarm-test/                    # 报警测试子组件
    │   │   └── reading-display/               # 读数显示子组件
    │   │
    │   └── channel-configuration/             # 通道配置页面
    │       └── channel-editor/                # 通道编辑器
    │
    ├── pipes/                                 # Angular 管道
    │   ├── index.ts
    │   ├── test-status.pipe.ts                # 测试状态显示管道
    │   ├── duration.pipe.ts                   # 时长格式化管道
    │   ├── engineering-unit.pipe.ts           # 工程单位格式化管道
    │   └── safe-html.pipe.ts                  # 安全HTML管道
    │
    ├── directives/                            # Angular 指令
    │   ├── index.ts
    │   ├── auto-focus.directive.ts            # 自动聚焦指令
    │   └── number-only.directive.ts           # 仅数字输入指令
    │
    ├── guards/                                # 路由守卫
    │   ├── index.ts
    │   ├── batch-loaded.guard.ts              # 批次加载完成守卫
    │   └── unsaved-changes.guard.ts           # 未保存更改守卫
    │
    └── interceptors/                          # HTTP 拦截器
        ├── index.ts
        ├── error-handler.interceptor.ts       # 错误处理拦截器
        └── loading.interceptor.ts             # 加载状态拦截器
```

## 配置和数据目录

```
config/                                        # 配置文件目录
├── app_settings.json                          # 应用默认设置
├── plc_configurations/                        # PLC配置文件模板
│   ├── modbus_tcp_template.json               # Modbus TCP 模板
│   ├── s7_template.json                       # Siemens S7 模板
│   └── opcua_template.json                    # OPC UA 模板
└── test_parameters/                           # 测试参数配置
    ├── ai_test_params.json                    # AI点测试参数
    ├── ao_test_params.json                    # AO点测试参数
    ├── di_test_params.json                    # DI点测试参数
    └── do_test_params.json                    # DO点测试参数

data/                                          # 运行时数据目录
├── batches/                                   # 批次数据存储
├── configurations/                            # 保存的配置
├── logs/                                      # 应用日志文件
└── exports/                                   # 导出的测试报告

docs/                                          # 项目文档
├── README.md                                  # 项目总体文档
├── ARCHITECTURE.md                            # 架构设计文档
├── PROJECT_STRUCTURE.md                       # 项目结构说明 (本文档)
├── API.md                                     # API接口文档
├── DEPLOYMENT.md                              # 部署说明文档
├── USER_MANUAL.md                             # 用户使用手册
├── DEVELOPMENT.md                             # 开发环境搭建指南
└── CHANGELOG.md                               # 版本更新日志

scripts/                                       # 构建和部署脚本
├── build-all.sh                               # 完整构建脚本
├── dev-setup.sh                              # 开发环境配置脚本
├── test-all.sh                                # 运行所有测试脚本
└── release.sh                                 # 发布脚本

tests/                                         # 端到端集成测试
├── fixtures/                                  # 测试数据和文件
│   ├── sample_excel_imports/                  # 示例Excel导入文件
│   ├── mock_plc_data/                         # Mock PLC数据
│   └── test_configurations/                   # 测试配置文件
├── integration/                               # 集成测试用例
│   ├── full_workflow_tests.rs
│   ├── plc_communication_tests.rs
│   └── data_persistence_tests.rs
└── performance/                               # 性能测试
    ├── load_tests.rs
    └── stress_tests.rs
```

## 说明

1. **已创建的目录**: 所有上述目录结构已经按照技术栈迁移详细实施步骤文档的要求创建完成。

2. **模块化设计**: 项目采用清晰的模块化设计，前后端分离，职责明确。

3. **分层架构**: Rust 后端采用分层架构（应用层、领域层、基础设施层），便于维护和扩展。

4. **配置管理**: 提供了完整的配置文件模板和运行时数据目录。

5. **文档完整**: 包含了完整的项目文档结构，便于团队协作和维护。

6. **测试支持**: 提供了完整的测试目录结构，支持单元测试、集成测试和性能测试。

这个项目结构为后续的开发工作提供了坚实的基础，严格按照技术栈迁移文档的要求进行组织。 