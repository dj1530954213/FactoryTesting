# 核心问题日志宏使用指南

## 概述

本文档介绍重构后的4类核心问题日志宏的使用方法和集成特性。这些宏已完全集成到新的Logger系统中，提供可见的日志输出、结构化分类和性能优化。

## 设计目标

- ✅ **真正集成到Logger系统**: 调用实际的Logger方法而非简单的log!宏
- ✅ **可见的日志输出**: 确保每个宏调用都产生可见的日志记录
- ✅ **正确的分类**: 使用CoreLogCategory进行结构化分类
- ✅ **文件和行号记录**: 自动记录调用位置信息
- ✅ **双重输出保证**: 同时输出到标准日志系统和直接控制台

## 核心宏列表

### 1. `log_communication_failure!` - 通讯失败日志

**用途**: 记录PLC通讯、网络连接等通讯相关的失败信息

**语法**:
```rust
log_communication_failure!("消息内容");
log_communication_failure!("格式化消息: {}", 参数);
```

**示例**:
```rust
// 基本用法
log_communication_failure!("PLC连接超时");

// 带参数
log_communication_failure!("设备{}连接失败，错误代码: {}", device_id, error_code);

// 复杂场景
log_communication_failure!("网络请求超时: {}ms > 最大超时{}ms", actual_time, max_timeout);
```

**输出格式**:
```
[2024-01-10 12:34:56.789] [ERROR] [通讯失败] [src/main.rs:42] - PLC连接超时
```

### 2. `log_file_parsing_failure!` - 文件解析失败日志

**用途**: 记录导入、导出文件时的解析错误

**语法**:
```rust
log_file_parsing_failure!("消息内容");
log_file_parsing_failure!("格式化消息: {}", 参数);
```

**示例**:
```rust
// CSV解析失败
log_file_parsing_failure!("无效的CSV格式");

// 带行号信息
log_file_parsing_failure!("解析第{}行失败: {}", line_num, error_detail);

// 文件类型错误
log_file_parsing_failure!("不支持的文件格式: {}，期望: CSV", file_extension);
```

**输出格式**:
```
[2024-01-10 12:34:56.789] [ERROR] [文件解析失败] [src/parser.rs:128] - 解析第15行失败: 缺少必需列
```

### 3. `log_test_failure!` - 测试执行失败日志

**用途**: 记录测试过程中的执行失败信息

**语法**:
```rust
log_test_failure!("消息内容");
log_test_failure!("格式化消息: {}", 参数);
```

**示例**:
```rust
// 基本测试失败
log_test_failure!("温度测试超出范围");

// 详细测试结果
log_test_failure!("第{}项测试失败: 期望{}, 实际{}", test_id, expected, actual);

// 安全测试失败
log_test_failure!("压力测试失败: {}Pa > 安全上限{}Pa", measured_pressure, max_safe_pressure);
```

**输出格式**:
```
[2024-01-10 12:34:56.789] [ERROR] [测试执行失败] [src/test_runner.rs:205] - 第5项测试失败: 期望25.0, 实际30.5
```

### 4. `log_user_operation!` - 用户操作日志

**用途**: 记录用户配置和连接操作信息

**语法**:
```rust
log_user_operation!("消息内容");
log_user_operation!("格式化消息: {}", 参数);
```

**示例**:
```rust
// 基本用户操作
log_user_operation!("用户连接PLC设备");

// 带用户信息
log_user_operation!("用户{}修改了配置项: {}", username, config_name);

// 批量操作
log_user_operation!("用户导入{}个测试配置文件，成功{}个", total_files, success_count);
```

**输出格式**:
```
[2024-01-10 12:34:56.789] [INFO] [用户操作] [src/ui_handler.rs:78] - 用户admin修改了配置项: 温度阈值
```

## 集成特性

### 1. 双重输出保证

每个宏调用都会产生两种输出:
- **标准日志系统输出**: 通过log::error!()或log::info!()调用
- **直接控制台输出**: 使用eprintln!()或println!()确保立即可见

### 2. 颜色编码

- **ERROR级别**: 红色显示 (`\x1b[31m...\x1b[0m`)
- **INFO级别**: 正常颜色显示

### 3. 自动位置记录

使用`file!()`和`line!()`宏自动记录调用位置:
- 文件名: 相对于项目根目录的路径
- 行号: 精确的代码行号

### 4. 时间戳格式

统一使用格式: `YYYY-MM-DD HH:MM:SS.fff`

## 最佳实践

### 1. 选择合适的宏

```rust
// ✅ 正确 - 网络/通讯相关错误
log_communication_failure!("PLC设备无响应");

// ❌ 错误 - 不是通讯问题
log_communication_failure!("配置文件不存在");

// ✅ 正确 - 文件解析问题
log_file_parsing_failure!("配置文件格式无效");
```

### 2. 提供足够的上下文信息

```rust
// ✅ 好 - 包含具体信息
log_test_failure!("压力测试失败: 测得{}Pa超出上限{}Pa", measured, limit);

// ❌ 差 - 信息不够具体
log_test_failure!("测试失败");
```

### 3. 合理使用参数化

```rust
// ✅ 好 - 使用格式化参数
log_user_operation!("用户{}在{}时刻执行了{}", user_id, timestamp, action);

// ❌ 差 - 字符串拼接
log_user_operation!(format!("用户{}在{}时刻执行了{}", user_id, timestamp, action));
```

### 4. 避免敏感信息

```rust
// ✅ 好 - 隐藏敏感信息
log_user_operation!("用户{}更改了密码", username);

// ❌ 差 - 泄露敏感信息
log_user_operation!("用户{}密码从{}改为{}", username, old_pass, new_pass);
```

## 测试和验证

### 1. 单元测试

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_macro_functionality() {
        // 测试基本功能
        log_communication_failure!("测试消息");
        
        // 测试参数化
        let error_code = 404;
        log_communication_failure!("错误代码: {}", error_code);
    }
}
```

### 2. 集成测试

运行集成测试来验证与Logger系统的完整集成:

```bash
cargo test integration_test
```

### 3. 手动验证

```rust
// 在main函数或测试中调用
crate::logging::integration_test::manual_integration_verification();
```

## 性能考虑

### 1. 性能基准

- 平均每次宏调用: < 1ms
- 1000次调用总时间: < 100ms
- 内存开销: 最小化

### 2. 异步处理

- 与EnterpriseLogger结合使用时，支持异步批量处理
- 避免阻塞主线程

### 3. 并发安全

- 所有宏都是线程安全的
- 支持多线程并发调用

## 故障排除

### 1. 没有日志输出

**问题**: 调用宏后看不到任何输出

**解决方案**:
1. 检查RUST_LOG环境变量: `export RUST_LOG=debug`
2. 确保Logger已正确初始化
3. 检查日志级别设置

### 2. 文件位置信息不正确

**问题**: 显示的文件名或行号不对

**解决方案**:
1. 确保使用的是直接宏调用，而不是封装函数
2. 检查是否有宏重新导出导致位置信息丢失

### 3. 性能问题

**问题**: 大量日志调用导致性能下降

**解决方案**:
1. 启用异步处理模式
2. 调整日志级别，减少不必要的日志
3. 使用批量处理模式

## 迁移指南

### 从旧版宏迁移

旧版宏调用:
```rust
log::error!("[通讯失败] PLC连接超时");
```

新版宏调用:
```rust
log_communication_failure!("PLC连接超时");
```

### 批量替换

可以使用以下正则表达式进行批量替换:

- `log::error!\("\[通讯失败\] (.+?)"\)` → `log_communication_failure!("$1")`
- `log::error!\("\[文件解析失败\] (.+?)"\)` → `log_file_parsing_failure!("$1")`
- `log::error!\("\[测试执行失败\] (.+?)"\)` → `log_test_failure!("$1")`
- `log::info!\("\[用户操作\] (.+?)"\)` → `log_user_operation!("$1")`

## 总结

重构后的核心问题日志宏提供了:

- ✅ 完整的Logger系统集成
- ✅ 可见和可靠的日志输出
- ✅ 结构化的问题分类
- ✅ 自动化的位置记录
- ✅ 优秀的性能表现
- ✅ 线程安全和并发支持

这些宏简化了日志记录的使用，同时提供了企业级的功能和可靠性。