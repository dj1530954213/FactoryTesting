# 枚举类型分析

## ModuleType (模块类型)

### 定义
表示不同类型的PLC模块，用于区分输入输出和模拟数字信号类型。

### 枚举值
- **AI**: 模拟量输入 (Analog Input)
- **AO**: 模拟量输出 (Analog Output)  
- **DI**: 数字量输入 (Digital Input)
- **DO**: 数字量输出 (Digital Output)
- **AINone**: 模拟量输入（无源）
- **AONone**: 模拟量输出（无源）
- **DINone**: 数字量输入（无源）
- **DONone**: 数字量输出（无源）
- **Communication**: 通信模块
- **Other(String)**: 其他特殊模块类型

### 业务方法
```rust
impl ModuleType {
    pub fn is_analog(&self) -> bool;      // 是否为模拟量
    pub fn is_digital(&self) -> bool;     // 是否为数字量
    pub fn is_input(&self) -> bool;       // 是否为输入
    pub fn is_output(&self) -> bool;      // 是否为输出
    pub fn is_passive(&self) -> bool;     // 是否为无源
    pub fn description(&self) -> &str;    // 中文描述
}
```

## PointDataType (点位数据类型)

### 定义
表示PLC点位的数据类型，用于数据读写和类型转换。

### 枚举值
- **Bool**: 布尔类型 (true/false)
- **Float**: 单精度浮点数
- **Double**: 双精度浮点数
- **Int**: 32位整数
- **Int16**: 16位整数
- **Int32**: 32位整数
- **UInt16**: 无符号16位整数
- **UInt32**: 无符号32位整数
- **String**: 字符串类型

### 业务方法
```rust
impl PointDataType {
    pub fn size_in_bytes(&self) -> usize;     // 数据大小(字节)
    pub fn is_numeric(&self) -> bool;         // 是否为数值类型
    pub fn default_value(&self) -> String;    // 默认值
    pub fn parse_from_string(&self, s: &str) -> Result<Value, ParseError>;
}
```

## OverallTestStatus (整体测试状态)

### 定义
表示一个通道测试实例的总体状态，反映测试进度和结果。

### 枚举值
- **NotTested**: 未测试
- **Skipped**: 跳过测试
- **WiringConfirmationRequired**: 接线确认需要
- **WiringConfirmed**: 接线已确认，等待开始硬点或手动测试
- **HardPointTestInProgress**: 硬点测试进行中
- **HardPointTesting**: 硬点测试进行中（别名）
- **HardPointTestCompleted**: 硬点测试已完成
- **ManualTestInProgress**: 手动测试进行中
- **ManualTesting**: 手动测试进行中（别名）
- **TestCompletedPassed**: 测试完成且通过
- **TestCompletedFailed**: 测试完成但失败
- **Retesting**: 重新测试中

### 状态转换规则
```
NotTested → WiringConfirmationRequired → WiringConfirmed 
         → HardPointTesting → HardPointTestCompleted 
         → ManualTesting → TestCompletedPassed/TestCompletedFailed
```

### 业务方法
```rust
impl OverallTestStatus {
    pub fn is_completed(&self) -> bool;           // 是否完成
    pub fn is_testing(&self) -> bool;             // 是否测试中
    pub fn is_passed(&self) -> bool;              // 是否通过
    pub fn is_failed(&self) -> bool;              // 是否失败
    pub fn next_possible_states(&self) -> Vec<Self>; // 下一步可能状态
    pub fn color_code(&self) -> &str;             // 状态颜色代码
    pub fn description(&self) -> &str;            // 中文描述
}
```

## SubTestStatus (子测试状态)

### 定义
表示单个子测试项的状态。

### 枚举值
- **NotTested**: 未测试
- **Testing**: 测试中
- **Passed**: 测试通过
- **Failed**: 测试失败
- **NotApplicable**: 不适用（该测试项对当前模块类型不适用）
- **Skipped**: 跳过测试

### 业务方法
```rust
impl SubTestStatus {
    pub fn is_final(&self) -> bool;        // 是否为最终状态
    pub fn is_success(&self) -> bool;      // 是否成功
    pub fn priority(&self) -> u8;          // 状态优先级
}
```

## SubTestItem (子测试项目)

### 定义
对应原ChannelMapping.cs中的各种子测试项，定义了具体的测试内容。

### 通用测试项
- **HardPoint**: 硬点回路测试（核心测试项）
- **TrendCheck/Trend**: 趋势检查（AI/AO模块）
- **ReportCheck/Report**: 报表检查（AI/AO模块）
- **Maintenance**: 维护功能测试（AI/AO模块）

### AI模块特有测试项
- **LowLowAlarm**: 低低报警测试
- **LowAlarm**: 低报警测试
- **HighAlarm**: 高报警测试
- **HighHighAlarm**: 高高报警测试
- **AlarmValueSetting**: 报警值设定整体状态
- **MaintenanceFunction**: 维护功能测试

### DI/DO模块特有测试项
- **StateDisplay**: 状态显示/回读测试

### AO模块特有测试项
- **Output0Percent**: 输出0%测试
- **Output25Percent**: 输出25%测试
- **Output50Percent**: 输出50%测试
- **Output75Percent**: 输出75%测试
- **Output100Percent**: 输出100%测试

### 通信测试项
- **CommunicationTest**: 通信连接测试

### 自定义测试项
- **Custom(String)**: 自定义测试项（支持扩展）

### 业务方法
```rust
impl SubTestItem {
    pub fn applicable_to_module(&self, module_type: &ModuleType) -> bool; // 是否适用于模块
    pub fn execution_order(&self) -> u8;                                  // 执行顺序
    pub fn estimated_duration_ms(&self) -> u32;                          // 预估耗时
    pub fn description(&self) -> &str;                                    // 中文描述
    pub fn is_critical(&self) -> bool;                                    // 是否为关键测试
}
```

## LogLevel (日志级别)

### 定义
用于系统日志记录的级别控制。

### 枚举值
- **Debug**: 调试级别
- **Info**: 信息级别
- **Warning**: 警告级别
- **Error**: 错误级别
- **Fatal**: 致命错误级别

### 业务方法
```rust
impl LogLevel {
    pub fn priority(&self) -> u8;          // 级别优先级
    pub fn color_code(&self) -> &str;      // 颜色代码
    pub fn should_log(&self, min_level: &LogLevel) -> bool; // 是否应该记录
}
```

## 枚举设计原则

### 1. 可扩展性
- 使用 `Other(String)` 和 `Custom(String)` 变体支持未来扩展
- 避免硬编码限制业务发展

### 2. 类型安全
- 使用强类型枚举替代字符串常量
- 编译时检查防止无效状态

### 3. 序列化兼容
- 所有枚举实现 `Serialize` 和 `Deserialize`
- 与前端JSON通信兼容
- 与数据库存储兼容

### 4. 业务语义
- 枚举值名称反映业务含义
- 提供中文描述方法
- 包含业务逻辑方法

### 5. 状态机支持
- `OverallTestStatus` 支持状态转换验证
- `SubTestStatus` 支持状态优先级
- 防止非法状态转换

## 使用示例

```rust
// 模块类型判断
let module = ModuleType::AI;
if module.is_analog() && module.is_input() {
    println!("这是模拟量输入模块");
}

// 状态转换
let current_status = OverallTestStatus::WiringConfirmed;
let next_states = current_status.next_possible_states();
assert!(next_states.contains(&OverallTestStatus::HardPointTesting));

// 测试项适用性
let test_item = SubTestItem::LowAlarm;
let module_type = ModuleType::AI;
if test_item.applicable_to_module(&module_type) {
    println!("低报警测试适用于AI模块");
}

// 序列化
let status = OverallTestStatus::TestCompletedPassed;
let json = serde_json::to_string(&status).unwrap();
let deserialized: OverallTestStatus = serde_json::from_str(&json).unwrap();
assert_eq!(status, deserialized);
```
