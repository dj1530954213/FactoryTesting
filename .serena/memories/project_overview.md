# FactoryTesting项目概述

## 项目用途
这是一个**工厂测试系统**，用于自动化测试工厂PLC点位的功能。系统支持AI/AO/DI/DO四种类型的点位测试，包括硬点测试、手动测试、报警测试等完整的测试流程。

## 技术架构
- **前端**: Angular 18 + ng-zorro-antd UI组件库
- **后端**: Rust + Tauri 2.0 桌面应用框架  
- **数据库**: SQLite + SeaORM
- **通信**: Modbus协议与PLC通信
- **报告**: PDF/Excel报告生成

## 核心业务流程
1. **数据导入**: 从Excel导入点位定义
2. **批次管理**: 创建测试批次，组织相关点位
3. **通道分配**: 为测试点位分配测试PLC通道
4. **测试执行**: 
   - 硬点测试：基础功能测试
   - 手动测试：AI显示值核对、报警测试等
   - 自动测试：批量执行测试流程
5. **结果管理**: 记录测试结果，生成报告

## 核心数据模型
- **ChannelPointDefinition**: 通道点位定义（53个字段）
- **ChannelTestInstance**: 通道测试实例
- **TestBatchInfo**: 测试批次信息
- **ManualTestStatus**: 手动测试状态

## 目录结构
```
FactoryTesting/
├── src/                    # Angular前端源码
│   ├── app/
│   │   ├── components/     # 组件
│   │   ├── services/       # 服务
│   │   └── models/         # 数据模型
├── src-tauri/              # Rust后端源码
│   ├── src/
│   │   ├── models/         # 数据模型
│   │   ├── services/       # 业务服务
│   │   ├── interfaces/     # API接口
│   │   └── infrastructure/ # 基础设施
```