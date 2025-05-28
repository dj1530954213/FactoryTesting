using System;
using System.Globalization;
using System.Windows.Data;
using System.Windows.Media;

namespace FatFullVersion.Shared.Converters
{
    /// <summary>
    /// 布尔值转颜色转换器
    /// </summary>
    public class BooleanToColorConverter : IValueConverter
    {
        /// <summary>
        /// 将布尔值转换为颜色，参数格式为"TrueColor;FalseColor"，如"#4CAF50;#F44336"
        /// </summary>
        /// <param name="value">布尔值</param>
        /// <param name="targetType">目标类型</param>
        /// <param name="parameter">参数，格式为"TrueColor;FalseColor"</param>
        /// <param name="culture">区域信息</param>
        /// <returns>根据布尔值返回对应的颜色</returns>
        public object Convert(object value, Type targetType, object parameter, CultureInfo culture)
        {
            if (value is bool isConnected)
            {
                string[] colors = parameter.ToString().Split(';');
                if (colors.Length == 2)
                {
                    string colorStr = isConnected ? colors[0] : colors[1];
                    return (SolidColorBrush)new BrushConverter().ConvertFrom(colorStr);
                }
            }
            
            // 默认返回灰色
            return new SolidColorBrush(Colors.Gray);
        }

        /// <summary>
        /// 将颜色转回布尔值，此方法不实现
        /// </summary>
        public object ConvertBack(object value, Type targetType, object parameter, CultureInfo culture)
        {
            throw new NotImplementedException();
        }
    }
} 