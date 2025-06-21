using System;
using System.Globalization;
using System.Windows.Data;
using Prism.Commands;

namespace FatFullVersion.Shared.Converters
{
    /// <summary>
    /// 根据通道类型选择对应的手动测试命令
    /// </summary>
    public class ModuleTypeToCommandConverter : IMultiValueConverter
    {
        /// <summary>
        /// 将通道类型转换为对应的命令
        /// </summary>
        /// <param name="values">值数组，第一个是通道类型，第二个是ViewModel</param>
        /// <param name="targetType">目标类型</param>
        /// <param name="parameter">参数</param>
        /// <param name="culture">区域信息</param>
        /// <returns>对应的命令</returns>
        public object Convert(object[] values, Type targetType, object parameter, CultureInfo culture)
        {
            if (values.Length >= 2 && values[0] is string moduleType && values[1] is ViewModels.DataEditViewModel viewModel)
            {
                string lowerModuleType = moduleType?.ToLower();
                
                // 处理无源模块类型，转换为对应的基本类型
                if (lowerModuleType?.Contains("none") == true)
                {
                    lowerModuleType = lowerModuleType.Replace("none", "");
                }
                
                return lowerModuleType switch
                {
                    "ai" => viewModel.OpenAIManualTestCommand,
                    "di" => viewModel.OpenDIManualTestCommand,
                    "do" => viewModel.OpenDOManualTestCommand,
                    "ao" => viewModel.OpenAOManualTestCommand,
                    _ => null
                };
            }
            return null;
        }

        /// <summary>
        /// 反向转换（未实现）
        /// </summary>
        public object[] ConvertBack(object value, Type[] targetTypes, object parameter, CultureInfo culture)
        {
            throw new NotImplementedException();
        }
    }
} 