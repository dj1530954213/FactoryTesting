using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Threading.Tasks;

namespace FatFullVersion.Entities.EntitiesEnum
{
    /// <summary>
    /// 测试PLC通道类型枚举
    /// </summary>
    public enum TestPlcChannelType
    {
        /// <summary>
        /// 模拟量输入有源
        /// </summary>
        AI = 0,

        /// <summary>
        /// 模拟量输出有源
        /// </summary>
        AO = 1,

        /// <summary>
        /// 数字量输入有源
        /// </summary>
        DI = 2,

        /// <summary>
        /// 数字量输出有源
        /// </summary>
        DO = 3,

        /// <summary>
        /// 模拟量输入无源
        /// </summary>
        AINone = 4,

        /// <summary>
        /// 模拟量输出无源
        /// </summary>
        AONone = 5,

        /// <summary>
        /// 数字量输入无源
        /// </summary>
        DINone = 6,

        /// <summary>
        /// 数字量输出无源
        /// </summary>
        DONone = 7
    }

    /// <summary>
    /// TestPlcChannelType 的扩展方法
    /// </summary>
    public static class TestPlcChannelTypeExtensions
    {
        /// <summary>
        /// 比较两个 TestPlcChannelType 值
        /// </summary>
        /// <param name="value">当前值</param>
        /// <param name="other">要比较的值</param>
        /// <returns>比较结果</returns>
        public static int CompareTo(this TestPlcChannelType value, TestPlcChannelType other)
        {
            return ((int)value).CompareTo((int)other);
        }
    }
}
