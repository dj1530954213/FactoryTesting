using System;
using System.Globalization;
using System.Windows.Data;

namespace FatFullVersion.Shared.Converters
{
    /// <summary>
    /// 将HardPointTestResult字符串转换为按钮的启用状态的转换器
    /// </summary>
    public class HardPointTestResultToEnabledConverter : IValueConverter
    {
        /// <summary>
        /// 将测试结果字符串转换为布尔值
        /// 如果字符串包含"失败"，则返回false；否则返回true
        /// </summary>
        /// <param name="value">测试结果字符串</param>
        /// <param name="targetType">目标类型</param>
        /// <param name="parameter">参数</param>
        /// <param name="culture">区域信息</param>
        /// <returns>按钮是否启用</returns>
        public object Convert(object value, Type targetType, object parameter, CultureInfo culture)
        {
            if (value is string result)
            {
                // 如果结果为空或包含"失败"字样，按钮禁用
                return !string.IsNullOrEmpty(result) && !result.Contains("失败") && !result.Contains("跳过");
            }
            // 默认禁用
            return false;
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