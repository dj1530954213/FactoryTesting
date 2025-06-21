using System;
using System.Globalization;
using System.Windows;
using System.Windows.Data;

namespace FatFullVersion.Shared.Converters
{
    /// <summary>
    /// 根据通道类型控制UI元素的显示和隐藏
    /// </summary>
    public class ModuleTypeToVisibilityConverter : IValueConverter
    {
        /// <summary>
        /// 将通道类型转换为可见性
        /// </summary>
        /// <param name="value">通道类型</param>
        /// <param name="targetType">目标类型</param>
        /// <param name="parameter">参数</param>
        /// <param name="culture">区域信息</param>
        /// <returns>可见性</returns>
        public object Convert(object value, Type targetType, object parameter, CultureInfo culture)
        {
            if (value is string moduleType)
            {
                // 所有通道类型都显示手动测试按钮
                string lowerModuleType = moduleType?.ToLower();
                return lowerModuleType switch
                {
                    "ai" => Visibility.Visible,
                    "di" => Visibility.Visible,
                    "do" => Visibility.Visible,
                    "ao" => Visibility.Visible,
                    "ainone" => Visibility.Visible,
                    "dinone" => Visibility.Visible,
                    "donone" => Visibility.Visible,
                    "aonone" => Visibility.Visible,
                    _ => Visibility.Collapsed
                };
            }
            return Visibility.Collapsed;
        }

        /// <summary>
        /// 反向转换（未实现）
        /// </summary>
        public object ConvertBack(object value, Type targetType, object parameter, CultureInfo culture)
        {
            throw new NotImplementedException();
        }
    }
} 