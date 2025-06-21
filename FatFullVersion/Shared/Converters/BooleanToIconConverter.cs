using System;
using System.Globalization;
using System.Windows.Data;

namespace FatFullVersion.Shared.Converters
{
    /// <summary>
    /// 布尔值转MaterialDesign图标转换器
    /// </summary>
    public class BooleanToIconConverter : IValueConverter
    {
        /// <summary>
        /// 将布尔值转换为MaterialDesign图标，参数格式为"TrueIcon;FalseIcon"，如"Check;Close"
        /// </summary>
        /// <param name="value">布尔值</param>
        /// <param name="targetType">目标类型</param>
        /// <param name="parameter">参数，格式为"TrueIcon;FalseIcon"</param>
        /// <param name="culture">区域信息</param>
        /// <returns>根据布尔值返回对应的图标</returns>
        public object Convert(object value, Type targetType, object parameter, CultureInfo culture)
        {
            if (value is bool isConnected && parameter is string paramStr)
            {
                string[] icons = paramStr.Split(';');
                if (icons.Length == 2)
                {
                    return isConnected ? icons[0] : icons[1];
                }
            }
            
            // 默认返回问号图标
            return "Help";
        }

        /// <summary>
        /// 将图标转回布尔值，此方法不实现
        /// </summary>
        public object ConvertBack(object value, Type targetType, object parameter, CultureInfo culture)
        {
            throw new NotImplementedException();
        }
    }
} 