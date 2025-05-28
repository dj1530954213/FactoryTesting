using System;
using System.Globalization;
using System.Windows.Data;
using System.Windows.Media;

namespace FatFullVersion.Shared.Converters
{
    /// <summary>
    /// 批次状态颜色转换器，将不同的批次状态转换为对应的颜色
    /// </summary>
    public class BatchStatusToColorConverter : IValueConverter
    {
        public object Convert(object value, Type targetType, object parameter, CultureInfo culture)
        {
            if (value is string status)
            {
                switch (status)
                {
                    case "已完成":
                        return new SolidColorBrush(Colors.Green);
                    case "进行中":
                        return new SolidColorBrush(Colors.Blue);
                    case "未开始":
                        return new SolidColorBrush(Colors.Orange);
                    case "已取消":
                        return new SolidColorBrush(Colors.Red);
                    default:
                        return new SolidColorBrush(Colors.Black);
                }
            }
            
            return new SolidColorBrush(Colors.Black);
        }

        public object ConvertBack(object value, Type targetType, object parameter, CultureInfo culture)
        {
            throw new NotImplementedException();
        }
    }
} 