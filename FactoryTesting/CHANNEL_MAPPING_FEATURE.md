# 通道映射查看功能实现总结

## 功能概述

为工厂验收测试系统的测试区域组件添加了完整的通道映射查看功能，用户现在可以查看系统在Excel导入时自动生成的智能通道映射关系。

## 实现的功能

### 1. 通道映射模态框
- **触发方式**: 在测试区域选择批次后，点击"查看通道映射"按钮
- **显示内容**: 详细的通道映射关系表格
- **加载状态**: 带有加载动画的数据获取过程
- **响应式设计**: 1200px宽度的大型模态框，适合显示详细信息

### 2. 映射信息展示
- **被测通道信息**: 位号、类型、供电方式
- **测试通道信息**: PLC地址、类型、供电方式
- **映射关系**: 直接映射、反向映射等类型
- **状态显示**: 激活/禁用状态
- **智能备注**: 自动生成的映射说明

### 3. 数据可视化
- **颜色编码**: 不同通道类型使用不同颜色标签
- **供电类型**: 有源(红色)、无源(蓝色)区分
- **映射类型**: 直接映射(绿色)、反向映射(蓝色)等
- **状态指示**: 激活状态用绿色标签显示

## 技术实现

### 前端组件 (Angular)

#### 1. 状态管理
```typescript
// 通道映射相关状态
isChannelMappingModalVisible = false;
channelMappings: any[] = [];
isLoadingMappings = false;
```

#### 2. 核心方法
- `viewChannelMappings()`: 打开模态框并加载数据
- `loadChannelMappings()`: 从后端获取映射数据
- `closeChannelMappingModal()`: 关闭模态框并清理数据
- `getMockChannelMappings()`: 提供模拟数据支持

#### 3. 辅助方法
- `getMappingTypeLabel()`: 映射类型中文标签
- `getMappingTypeColor()`: 映射类型颜色编码
- `getChannelTypeColor()`: 通道类型颜色编码

### 后端服务集成

#### 1. API服务方法
```typescript
/**
 * 获取通道映射配置
 */
getChannelMappings(): Observable<any[]> {
  return from(invoke<any[]>('get_channel_mappings_cmd'));
}
```

#### 2. 数据结构
```typescript
interface ChannelMapping {
  id: string;
  target_channel_id: string;
  target_channel_tag: string;
  target_channel_type: string;
  target_power_type: string;
  test_plc_channel_id: string;
  test_plc_channel_address: string;
  test_plc_channel_type: string;
  test_plc_power_type: string;
  mapping_type: string;
  is_active: boolean;
  notes: string;
  created_at: string;
}
```

### UI组件使用

#### 1. NG-ZORRO组件
- `nz-modal`: 模态框容器
- `nz-table`: 映射数据表格
- `nz-tag`: 状态和类型标签
- `nz-spin`: 加载动画
- `nz-alert`: 信息提示
- `nz-empty`: 空数据状态

#### 2. 样式设计
```scss
.loading-container {
  display: flex;
  justify-content: center;
  align-items: center;
  height: 100%;
}

.mapping-table {
  margin-top: 24px;
}

.mapping-notes {
  white-space: pre-wrap;
}

.empty-mappings {
  margin-top: 24px;
}
```

## 智能映射规则展示

### 1. 有源/无源匹配
- **AI有源** → **AI无源测试通道** (反向映射)
- **AI无源** → **AI有源测试通道** (直接映射)
- **AO有源** → **AO无源测试通道** (反向映射)
- **AO无源** → **AO有源测试通道** (直接映射)
- **DI/DO** 同理

### 2. 映射类型说明
- **直接映射**: 同类型同供电方式
- **反向映射**: 同类型相反供电方式
- **比例映射**: 带有数值转换的映射
- **自定义映射**: 用户手动配置的映射

## 用户体验

### 1. 操作流程
1. 在测试区域选择一个批次
2. 点击"查看通道映射"按钮
3. 系统自动加载并显示映射关系
4. 用户可以查看详细的映射信息
5. 关闭模态框返回主界面

### 2. 信息展示
- **清晰的表格布局**: 9列信息完整展示
- **颜色编码**: 快速识别不同类型和状态
- **分页支持**: 大量数据的友好展示
- **空状态处理**: 无数据时的友好提示

### 3. 响应式设计
- **大屏幕**: 1200px宽度充分利用空间
- **小屏幕**: 表格自动适应并支持横向滚动
- **加载状态**: 平滑的加载动画体验

## 模拟数据示例

系统提供了完整的模拟数据支持，包括：
- AI001 有源 → AI1_1 无源 (反向映射)
- AO001 无源 → AO1_1 有源 (直接映射)
- DI001 有源 → DI1_1 无源 (反向映射)
- DO001 无源 → DO1_1 有源 (直接映射)

每个映射都包含详细的备注说明，帮助用户理解映射逻辑。

## 后续扩展

### 1. 功能增强
- 映射关系编辑功能
- 映射冲突检测和解决
- 批量映射操作
- 映射关系导出

### 2. 性能优化
- 虚拟滚动支持大量数据
- 映射数据缓存机制
- 增量更新和实时同步

### 3. 用户体验
- 映射关系可视化图表
- 搜索和过滤功能
- 映射历史记录
- 操作日志追踪

## 总结

通过实现完整的通道映射查看功能，用户现在可以：
- 清晰地了解系统的智能映射逻辑
- 验证映射关系的正确性
- 快速定位映射问题
- 为后续的测试执行提供参考

这个功能大大提升了系统的透明度和可维护性，为工厂验收测试提供了重要的支持工具。 