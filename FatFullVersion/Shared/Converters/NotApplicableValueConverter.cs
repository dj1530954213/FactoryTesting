using System;
using System.Diagnostics;
using System.Globalization;
using System.Windows.Data;

namespace FatFullVersion.Shared.Converters
{
    /// <summary>
    /// 根据点位类型将不适用的数值显示为"/"的转换器
    /// </summary>
    public class NotApplicableValueConverter : IMultiValueConverter
    {
        /// <summary>
        /// 将数值根据点位类型转换为显示值
        /// </summary>
        /// <param name="values">第一个参数是显示值，第二个参数是点位类型</param>
        /// <param name="targetType">目标类型</param>
        /// <param name="parameter">可选参数</param>
        /// <param name="culture">区域信息</param>
        /// <returns>适当的显示值</returns>
        public object Convert(object[] values, Type targetType, object parameter, CultureInfo culture)
        {
            // 只完成最基本的检查，减少出错可能
            try
            {
                // 对于空值或不足两个值的情况，直接返回第一个值（如果存在）
                if (values == null)
                    return null;
                
                if (values.Length < 2)
                    return values.Length > 0 ? values[0] : null;

                var value = values[0];
                var pointType = values[1] as string;

                // 如果值为null，直接返回null
                if (value == null)
                    return null;

                // 如果点位类型为null，直接返回原始值
                if (pointType == null)
                    return value;

                // 将点位类型转为小写并规范化
                pointType = pointType.ToLower().Trim();

                // 只有DI和DO点位的特定字段显示为"/"
                if (pointType == "di" || pointType == "do")
                {
                    if (parameter != null)
                    {
                        string fieldName = parameter.ToString();
                        if (fieldName == "Range" || fieldName == "Percent" || fieldName == "Alarm")
                        {
                            return "/";
                        }
                    }
                }

                // 其他所有情况返回原始值
                return value;
            }
            catch (Exception ex)
            {
                Debug.WriteLine($"NotApplicableValueConverter异常: {ex.Message}");
                return values?.Length > 0 ? values[0] : null;
            }
        }

        /// <summary>
        /// 不支持反向转换
        /// </summary>
        public object[] ConvertBack(object value, Type[] targetTypes, object parameter, CultureInfo culture)
        {
            throw new NotImplementedException();
        }
    }
} 