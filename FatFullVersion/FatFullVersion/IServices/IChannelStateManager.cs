using System;
using FatFullVersion.Models; // Assumed ExcelPointData and ChannelMapping are here or in Entities
// using FatFullVersion.Entities; // Assumed ChannelMapping might be here

namespace FatFullVersion.IServices
{
    public enum ManualTestItem
    {
        ShowValue,          // 显示值核对 (适用于 AI, AO, DI, DO)
        LowLowAlarm,        // 低低报测试 (AI)
        LowAlarm,           // 低报测试 (AI)
        HighAlarm,          // 高报测试 (AI)
        HighHighAlarm,      // 高高报测试 (AI)
        AlarmValueSet,      // 报警值设定核对 (AI)
        MaintenanceFunction,// 维护功能测试 (AI, AO - AO可能默认通过)
        TrendCheck,         // 趋势检查 (AI, AO)
        ReportCheck         // 报表检查 (AI, AO)
    }

    public struct HardPointTestRawResult
    {
        public bool IsSuccess { get; }
        public string Detail { get; } // 成功时可为空，失败时为原因
        // public float? RawValue { get; } // 如果需要传递原始数值结果，应为 float?
        // 可根据需要添加其他原始结果信息，如错误码

        public HardPointTestRawResult(bool isSuccess, string detail = null /*, float? rawValue = null*/)
        {
            IsSuccess = isSuccess;
            Detail = detail;
            // RawValue = rawValue;
        }
    }

    public interface IChannelStateManager
    {
        /// <summary>
        /// 根据从Excel导入的数据初始化ChannelMapping对象的状态。
        /// </summary>
        void InitializeChannelFromImport(ChannelMapping channel, ExcelPointData pointData, DateTime importTime);

        /// <summary>
        /// 应用通道分配信息到ChannelMapping对象，并可能重置测试状态。
        /// </summary>
        void ApplyAllocationInfo(ChannelMapping channel, string testBatch, string testPlcChannelTag, string testPlcCommAddress);

        /// <summary>
        /// 清除ChannelMapping对象的通道分配信息，并重置测试状态到初始。
        /// </summary>
        void ClearAllocationInfo(ChannelMapping channel);

        /// <summary>
        /// 将ChannelMapping对象标记为已跳过测试。
        /// </summary>
        void MarkAsSkipped(ChannelMapping channel, string reason, DateTime skipTime);

        /// <summary>
        /// 准备ChannelMapping对象以进行接线确认（通常设置为等待测试）。
        /// </summary>
        void PrepareForWiringConfirmation(ChannelMapping channel, DateTime confirmTime);

        /// <summary>
        /// 开始硬点测试前，设置ChannelMapping的初始测试中状态。
        /// </summary>
        void BeginHardPointTest(ChannelMapping channel, DateTime startTime);

        /// <summary>
        /// 根据原始的硬点测试结果，更新ChannelMapping的状态。
        /// 内部会调用EvaluateOverallStatus。
        /// </summary>
        void SetHardPointTestOutcome(ChannelMapping channel, HardPointTestRawResult rawOutcome, DateTime outcomeTime);

        /// <summary>
        /// 开始手动测试前，准备ChannelMapping的状态（如重置子项为未测试）。
        /// </summary>
        void BeginManualTest(ChannelMapping channel);

        /// <summary>
        /// 记录手动测试中某个子项的测试结果，并重新评估整体状态。
        /// 内部会调用EvaluateOverallStatus。
        /// </summary>
        void SetManualSubTestOutcome(ChannelMapping channel, ManualTestItem itemType, bool isSuccess, DateTime outcomeTime, string details = null);

        /// <summary>
        /// 重置ChannelMapping的状态以便进行重新测试。
        /// </summary>
        void ResetForRetest(ChannelMapping channel);
    }
} 