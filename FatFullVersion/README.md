# FAT 测试系统

本项目是一个功能验收测试（FAT）系统，用于自动化 PLC 通道的分配和测试。

## 测试 PLC 配置

当前版本已完善了测试 PLC 配置类，主要修改包括：

1. **TestPlcConfig 类**：
   - 位于 `Entities` 命名空间下
   - 包含 PLC 品牌类型（Micro850、HollySys_LKS 等）
   - 包含 IP 地址
   - 包含通道与通讯地址的对应关系表（ComparisonTable 列表）

2. **ComparisonTable 记录类**：
   - 位于 `Entities.ValueObject` 命名空间下
   - 定义为记录（record），包含通道地址、通讯地址和通道类型

3. **测试 PLC 通道类型枚举**：
   - 位于 `Entities.EntitiesEnum` 命名空间下
   - 包含 AI、AO、DI、DO 四种通道类型

4. **通道映射服务更新**：
   - 修改了 `IChannelMappingService` 接口，添加了使用 TestPlcConfig 进行通道分配的方法
   - 更新了 `ChannelMappingService` 实现，使其支持基于 TestPlcConfig 的通道分配
   - 兼容原有的通道分配方法，保证向后兼容性

5. **视图模型更新**：
   - 更新了 `DataEditViewModel` 中的通道分配方法，使用新的 API

## 使用方式

通道分配现在可以基于 TestPlcConfig 对象进行，这提供了更灵活的配置选项：

```csharp
// 创建测试 PLC 配置
var testPlcConfig = new TestPlcConfig
{
    BrandType = PlcBrandTypeEnum.Micro850,
    IpAddress = "192.168.1.1",
    CommentsTables = new List<ComparisonTable>
    {
        new ComparisonTable("AI1_1", "AI1.1", TestPlcChannelType.AI),
        new ComparisonTable("AO1_1", "AO1.1", TestPlcChannelType.AO),
        // 添加更多通道...
    }
};

// 使用配置分配通道
var result = await _channelMappingService.AllocateChannelsAsync(
    aiChannels, aoChannels, diChannels, doChannels, testPlcConfig);
``` 