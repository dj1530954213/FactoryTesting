using System;
using System.Globalization;
using System.Windows.Data;

namespace FatFullVersion.Shared.Converters
{
    /// <summary>
    /// 布尔值转状态文本转换器
    /// </summary>
    public class BooleanToStatusConverter : IValueConverter
    {
        /// <summary>
        /// 将布尔值转换为状态文本，参数格式为"TrueText;FalseText"，如"已连接;未连接"
        /// </summary>
        /// <param name="value">布尔值</param>
        /// <param name="targetType">目标类型</param>
        /// <param name="parameter">参数，格式为"TrueText;FalseText"</param>
        /// <param name="culture">区域信息</param>
        /// <returns>根据布尔值返回对应的状态文本</returns>
        public object Convert(object value, Type targetType, object parameter, CultureInfo culture)
        {
            if (value is bool isConnected && parameter is string paramStr)
            {
                string[] texts = paramStr.Split(';');
                if (texts.Length == 2)
                {
                    return isConnected ? texts[0] : texts[1];
                }
            }
            
            // 默认返回未知状态
            return "未知状态";
        }

        /// <summary>
        /// 将状态文本转回布尔值，此方法不实现
        /// </summary>
        public object ConvertBack(object value, Type targetType, object parameter, CultureInfo culture)
        {
            throw new NotImplementedException();
        }
    }
} 