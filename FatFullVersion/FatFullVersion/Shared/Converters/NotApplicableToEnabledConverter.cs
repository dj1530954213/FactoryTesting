using System;
using System.Globalization;
using System.Windows.Data;

namespace FatFullVersion.Shared.Converters
{
    /// <summary>
    /// 如果绑定值等于 "N/A"（无需测试）则返回 false (不可点击)，否则返回 true。
    /// 适用于 Button.IsEnabled 绑定。
    /// </summary>
    public class NotApplicableToEnabledConverter : IValueConverter
    {
        public object Convert(object value, Type targetType, object parameter, CultureInfo culture)
        {
            string status = value?.ToString() ?? string.Empty;
            return !string.Equals(status, "N/A", StringComparison.OrdinalIgnoreCase);
        }

        public object ConvertBack(object value, Type targetType, object parameter, CultureInfo culture)
        {
            throw new NotImplementedException();
        }
    }
} 