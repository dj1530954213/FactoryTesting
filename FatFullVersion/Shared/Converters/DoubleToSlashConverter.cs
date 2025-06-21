using System;
using System.Globalization;
using System.Windows.Data;

namespace FatFullVersion.Shared.Converters
{
    /// <summary>
    /// 将double.NaN值转换为"/"的转换器，对有效数值保留小数点后三位
    /// </summary>
    public class DoubleToSlashConverter : IValueConverter
    {
        /// <summary>
        /// 将数值转换为显示值
        /// </summary>
        /// <param name="value">数值</param>
        /// <param name="targetType">目标类型</param>
        /// <param name="parameter">参数</param>
        /// <param name="culture">区域信息</param>
        /// <returns>显示值</returns>
        public object Convert(object value, Type targetType, object parameter, CultureInfo culture)
        {
            if (value is double doubleValue)
            {
                if (double.IsNaN(doubleValue))
                {
                    return "/";
                }
                // 格式化浮点数，保留三位小数
                return Math.Round(doubleValue, 3).ToString("F3", culture);
            }
            else if (value is float floatValue)
            {
                if (float.IsNaN(floatValue))
                {
                    return "/";
                }
                // 格式化浮点数，保留三位小数
                return Math.Round(floatValue, 3).ToString("F3", culture);
            }
            return value;
        }

        /// <summary>
        /// 不支持反向转换
        /// </summary>
        public object ConvertBack(object value, Type targetType, object parameter, CultureInfo culture)
        {
            throw new NotImplementedException();
        }
    }
} 