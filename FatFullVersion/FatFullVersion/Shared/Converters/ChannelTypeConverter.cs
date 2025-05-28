using System;
using System.Globalization;
using System.Windows.Data;
using FatFullVersion.Entities.EntitiesEnum;

namespace FatFullVersion.Shared.Converters
{
    /// <summary>
    /// 通道类型转换器
    /// 将TestPlcChannelType枚举转换为可读的字符串
    /// </summary>
    public class ChannelTypeConverter : IValueConverter
    {
        public object Convert(object value, Type targetType, object parameter, CultureInfo culture)
        {
            if (value == null)
                return null;

            // 如果已经是ChannelTypeItem类型，直接返回
            if (value is ChannelTypeItem item)
                return item;

            // 如果是TestPlcChannelType枚举，转换为对应的ChannelTypeItem
            if (value is TestPlcChannelType channelType)
            {
                return channelType switch
                {
                    TestPlcChannelType.AI => new ChannelTypeItem { Value = TestPlcChannelType.AI, DisplayName = "模拟量输入有源" },
                    TestPlcChannelType.AO => new ChannelTypeItem { Value = TestPlcChannelType.AO, DisplayName = "模拟量输出有源" },
                    TestPlcChannelType.DI => new ChannelTypeItem { Value = TestPlcChannelType.DI, DisplayName = "数字量输入有源" },
                    TestPlcChannelType.DO => new ChannelTypeItem { Value = TestPlcChannelType.DO, DisplayName = "数字量输出有源" },
                    TestPlcChannelType.AINone => new ChannelTypeItem { Value = TestPlcChannelType.AINone, DisplayName = "模拟量输入无源" },
                    TestPlcChannelType.AONone => new ChannelTypeItem { Value = TestPlcChannelType.AONone, DisplayName = "模拟量输出无源" },
                    TestPlcChannelType.DINone => new ChannelTypeItem { Value = TestPlcChannelType.DINone, DisplayName = "数字量输入无源" },
                    TestPlcChannelType.DONone => new ChannelTypeItem { Value = TestPlcChannelType.DONone, DisplayName = "数字量输出无源" },
                    _ => new ChannelTypeItem { Value = TestPlcChannelType.DI, DisplayName = "数字量输入有源" }
                };
            }

            return null;
        }

        public object ConvertBack(object value, Type targetType, object parameter, CultureInfo culture)
        {
            if (value == null)
                return null;

            // 如果已经是TestPlcChannelType，直接返回
            if (value is TestPlcChannelType)
                return value;

            // 如果是ChannelTypeItem，返回其Value
            if (value is ChannelTypeItem item)
                return item.Value;

            // 如果是字符串，尝试转换为对应的枚举值
            if (value is string str)
            {
                return str switch
                {
                    "模拟量输入有源" => TestPlcChannelType.AI,
                    "模拟量输出有源" => TestPlcChannelType.AO,
                    "数字量输入有源" => TestPlcChannelType.DI,
                    "数字量输出有源" => TestPlcChannelType.DO,
                    "模拟量输入无源" => TestPlcChannelType.AINone,
                    "模拟量输出无源" => TestPlcChannelType.AONone,
                    "数字量输入无源" => TestPlcChannelType.DINone,
                    "数字量输出无源" => TestPlcChannelType.DONone,
                    _ => TestPlcChannelType.DI
                };
            }

            return TestPlcChannelType.DI;
        }
    }

    /// <summary>
    /// 通道类型项
    /// </summary>
    public class ChannelTypeItem
    {
        public TestPlcChannelType Value { get; set; }
        public string DisplayName { get; set; }
    }
}
