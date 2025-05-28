using System;
using System.Globalization;
using System.Linq;
using System.Windows.Data;
using System.Windows.Media;

namespace FatFullVersion.Shared.Converters
{
    /// <summary>
    /// 报警成对状态（高报+高高报，低报+低低报）转换为颜色。
    /// 逻辑：
    /// 1. 只要其中任意一个状态为 "失败" ➜ 红色。
    /// 2. 否则，只要任意一个状态为 "未测试"/"等待测试"/"测试中" ➜ 灰色（待确认）。
    /// 3. 否则，只要任意一个状态为 "通过" ➜ 绿色。
    /// 4. 都为 "N/A" ➜ 默认蓝色（表示无需测试）。
    /// </summary>
    public class AlarmPairStatusToColorConverter : IMultiValueConverter
    {
        private static readonly string[] WaitingStates = { "未测试", "等待测试", "测试中" };

        public object Convert(object[] values, Type targetType, object parameter, CultureInfo culture)
        {
            // values[0] = 高报(或低报)状态, values[1] = 高高报(或低低报)状态
            var states = values?.Select(v => v?.ToString() ?? string.Empty).ToArray() ?? Array.Empty<string>();

            if (states.Length == 0) return new SolidColorBrush(Color.FromRgb(0x44, 0x72, 0xC4)); // 默认蓝色

            // 1. 任意失败
            if (states.Any(s => s.Contains("失败")))
            {
                return new SolidColorBrush(Colors.Red);
            }

            // 2. 任意待测试
            if (states.Any(s => WaitingStates.Contains(s)))
            {
                return new SolidColorBrush(Colors.Gray);
            }

            // 3. 任意通过
            if (states.Any(s => s.Contains("通过")))
            {
                return new SolidColorBrush(Colors.Green);
            }

            // 4. 全部 N/A 或其他 ➜ 默认蓝色
            return new SolidColorBrush(Color.FromRgb(0x44, 0x72, 0xC4));
        }

        public object[] ConvertBack(object value, Type[] targetTypes, object parameter, CultureInfo culture)
        {
            throw new NotImplementedException();
        }
    }
} 