using System;
using System.ComponentModel;

namespace FatFullVersion.Models
{
    /// <summary>
    /// 批次测试状态枚举
    /// </summary>
    public enum BatchTestStatus
    {
        /// <summary>
        /// 未开始
        /// </summary>
        [Description("未开始")]
        NotStarted,

        /// <summary>
        /// 进行中
        /// </summary>
        [Description("进行中")]
        InProgress,

        /// <summary>
        /// 已完成
        /// </summary>
        [Description("已完成")]
        Completed,

        /// <summary>
        /// 已取消
        /// </summary>
        [Description("已取消")]
        Canceled
    }

    /// <summary>
    /// 批次测试状态扩展方法
    /// </summary>
    public static class BatchTestStatusExtensions
    {
        /// <summary>
        /// 获取批次测试状态的描述文本
        /// </summary>
        /// <param name="status">批次测试状态</param>
        /// <returns>状态描述文本</returns>
        public static string GetDescription(this BatchTestStatus status)
        {
            var field = status.GetType().GetField(status.ToString());
            if (field == null) return status.ToString();

            var attribute = (DescriptionAttribute)Attribute.GetCustomAttribute(field, typeof(DescriptionAttribute));
            return attribute == null ? status.ToString() : attribute.Description;
        }
    }
} 