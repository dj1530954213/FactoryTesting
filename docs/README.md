# FAT_TEST 桌面应用程序

## 项目概述

FAT_TEST 是一个基于 Rust + Angular + Tauri 技术栈的工厂验收测试桌面应用程序。该项目从原有的 C# WPF 技术栈迁移而来，旨在提供更好的性能、跨平台支持和现代化的用户界面。

## 技术栈

- **后端**: Rust + Tauri
- **前端**: Angular + TypeScript
- **桌面应用**: Tauri (跨平台)
- **通信协议**: Modbus TCP, Siemens S7, OPC UA
- **数据存储**: SQLite, JSON文件

## 项目结构

```
fat-test-desktop-app/
├── src-tauri/          # Rust 后端代码
├── src/                # Angular 前端代码
├── docs/               # 项目文档
├── scripts/            # 构建脚本
├── tests/              # 测试文件
├── config/             # 配置文件
└── data/               # 运行时数据
```

## 快速开始

### 环境要求

- Node.js 18+
- Rust 1.70+
- Tauri CLI

### 安装依赖

```bash
npm install
```

### 开发模式

```bash
npm run tauri dev
```

### 构建应用

```bash
npm run tauri build
```

## 功能特性

- 批次管理和测试编排
- 多种PLC通信协议支持
- 实时测试状态监控
- 手动测试功能
- 测试结果报告和导出
- 通道配置管理
- 系统设置和日志查看

## 贡献指南

请参考 [DEVELOPMENT.md](./DEVELOPMENT.md) 了解开发环境搭建和贡献流程。

## 许可证

[MIT License](../LICENSE) 