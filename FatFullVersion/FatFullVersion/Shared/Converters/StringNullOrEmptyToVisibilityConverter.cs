using System;
using System.Globalization;
using System.Windows;
using System.Windows.Data;

namespace FatFullVersion.Shared.Converters
{
    /// <summary>
    /// 将字符串是否为空转换为Visibility。非空->Visible，空或仅空白->Collapsed。
    /// </summary>
    public class StringNullOrEmptyToVisibilityConverter : IValueConverter
    {
        public object Convert(object value, Type targetType, object parameter, CultureInfo culture)
        {
            bool isNullOrEmpty = value == null || string.IsNullOrWhiteSpace(value.ToString());
            // 如果传入参数为"Invert"，则取反逻辑
            if (parameter != null && parameter.ToString()?.Equals("Invert", StringComparison.OrdinalIgnoreCase) == true)
            {
                isNullOrEmpty = !isNullOrEmpty;
            }
            return isNullOrEmpty ? Visibility.Collapsed : Visibility.Visible;
        }

        public object ConvertBack(object value, Type targetType, object parameter, CultureInfo culture)
        {
            throw new NotImplementedException();
        }
    }
} 