using System;
using System.Globalization;
using System.Windows;
using System.Windows.Data;
using System.Windows.Media;

namespace FatFullVersion.Shared.Converters
{
    /// <summary>
    /// 测试结果状态到背景色的转换器
    /// </summary>
    public class TestResultStatusToColorConverter : IValueConverter
    {
        /// <summary>
        /// 将测试结果状态转换为对应的背景色
        /// </summary>
        public object Convert(object value, Type targetType, object parameter, CultureInfo culture)
        {
            // 添加调试输出
            //System.Diagnostics.Debug.WriteLine($"TestResultStatusToColorConverter: value={value}");
            
            if (value is int status)
            {
                return status switch
                {
                    0 => new SolidColorBrush(Colors.White), // 未测试
                    1 => new SolidColorBrush(Color.FromRgb(144, 238, 144)), // 更明亮的绿色 (LightGreen)
                    2 => new SolidColorBrush(Color.FromRgb(255, 182, 193)), // 更明亮的红色 (LightPink)
                    _ => new SolidColorBrush(Colors.White)
                };
            }
            return new SolidColorBrush(Colors.White);
        }

        public object ConvertBack(object value, Type targetType, object parameter, CultureInfo culture)
        {
            throw new NotImplementedException();
        }
    }
}