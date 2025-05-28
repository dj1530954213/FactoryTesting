using System;
using System.Globalization;
using System.Windows.Data;
using System.Windows.Media;
using FatFullVersion.Shared;

namespace FatFullVersion.Shared.Converters
{
    /// <summary>
    /// OverallResultStatus → 行背景色 转换器。
    /// </summary>
    public class OverallStatusToColorConverter : IValueConverter
    {
        public object Convert(object value, Type targetType, object parameter, CultureInfo culture)
        {
            if (value is OverallResultStatus status)
            {
                return status switch
                {
                    OverallResultStatus.Passed      => new SolidColorBrush(Color.FromRgb(144, 238, 144)), // LightGreen
                    OverallResultStatus.Failed      => new SolidColorBrush(Color.FromRgb(255, 182, 193)), // LightPink
                    OverallResultStatus.Skipped     => new SolidColorBrush(Color.FromRgb(211, 211, 211)), // LightGray
                    OverallResultStatus.InProgress  => new SolidColorBrush(Color.FromRgb(255, 255, 224)), // LightYellow
                    _                               => new SolidColorBrush(Colors.White)
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