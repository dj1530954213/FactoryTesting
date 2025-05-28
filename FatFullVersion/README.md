# FAT 测试系统

## 1. 项目概述

FAT 测试系统是一个用于自动化 PLC 通道分配和测试的功能验收测试系统。系统采用现代化的架构设计，支持多种 PLC 品牌和通信协议，提供完整的测试流程管理功能。

## 2. 系统架构

### 2.1 整体架构
- **MVVM 架构模式**：采用 WPF + Prism 实现
- **依赖注入**：使用 DryIoc 容器管理服务依赖
- **模块化设计**：基于 Prism 的模块化功能
- **分层架构**：
  - 表现层（Views）
  - 业务逻辑层（ViewModels）
  - 服务层（Services）
  - 数据访问层（Repository）
  - 实体层（Entities）

### 2.2 技术栈
- **开发框架**：.NET 8.0
- **UI 框架**：WPF + MaterialDesignThemes
- **依赖注入**：Prism + DryIoc
- **数据访问**：Entity Framework Core
- **Excel 处理**：NPOI
- **通信协议**：Modbus TCP

## 3. 核心服务与功能

### 3.1 通道映射服务 (ChannelMappingService)
- **功能描述**：
  - PLC 通道的自动分配和管理
  - 支持多种 PLC 品牌（Micro850、HollySys_LKS 等）
  - 通道类型管理（AI、AO、DI、DO）
  - Excel 数据导入导出
- **依赖服务**：
  - IRepository：数据持久化
  - IMessageService：消息通知

### 3.2 测试任务管理器 (TestTaskManager)
- **功能描述**：
  - 测试任务的创建、执行和监控
  - 测试流程的状态管理
  - 测试结果的实时更新
  - 批量测试任务处理
- **依赖服务**：
  - IPlcCommunication：PLC 通信
  - IChannelMappingService：通道映射
  - ITestRecordService：测试记录

### 3.3 PLC 通信服务 (PlcCommunication)
- **功能描述**：
  - 基于 Modbus TCP 协议的 PLC 通信
  - 支持多种 PLC 品牌
  - 通信状态监控
  - 数据读写操作
- **依赖服务**：
  - IMessageService：通信状态通知

### 3.4 测试记录服务 (TestRecordService)
- **功能描述**：
  - 测试数据的记录和存储
  - 测试历史查询
  - 测试报告生成
- **依赖服务**：
  - IRepository：数据持久化
  - ITestResultExportService：结果导出

### 3.5 数据点服务 (PointDataService)
- **功能描述**：
  - 支持 Excel 和 API 两种数据源
  - 数据点的导入和管理
  - 数据验证和转换
- **依赖服务**：
  - IRepository：数据持久化

## 4. 服务协作流程

### 4.1 测试任务执行流程
1. **任务创建**：
   - TestTaskManager 创建测试任务
   - ChannelMappingService 分配通道
   - PointDataService 加载测试数据

2. **任务执行**：
   - PlcCommunication 建立通信连接
   - TestTaskManager 控制测试流程
   - TestRecordService 记录测试数据

3. **结果处理**：
   - TestRecordService 保存测试结果
   - TestResultExportService 生成报告

### 4.2 通道映射流程
1. **数据导入**：
   - PointDataService 导入 Excel 数据
   - ChannelMappingService 创建映射关系

2. **通道分配**：
   - ChannelMappingService 根据 PLC 配置分配通道
   - 生成通道映射表

3. **数据导出**：
   - ChannelMappingService 导出映射结果
   - 生成配置文档

## 5. 项目结构

```
FatFullVersion/
├── Views/                # 视图层
├── ViewModels/           # 视图模型层
├── Services/             # 服务实现
├── IServices/            # 服务接口
├── Entities/             # 实体类
├── Models/               # 数据模型
├── Data/                 # 数据访问层
├── Common/               # 公共组件
├── Events/               # 事件定义
└── Shared/               # 共享资源
```

## 6. 开发规范

- 所有服务必须实现对应的接口
- 使用依赖注入管理服务实例
- 遵循 MVVM 模式进行开发
- 保持代码注释的完整性
- 使用异步编程模式处理耗时操作