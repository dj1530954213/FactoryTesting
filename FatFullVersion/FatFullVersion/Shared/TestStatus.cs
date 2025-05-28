using System;

namespace FatFullVersion.Shared
{
    /// <summary>
    /// 通用测试状态枚举。
    /// </summary>
    public enum TestStatus
    {
        NotTested = 0,     // 未测试
        Waiting = 1,       // 等待测试
        Testing = 2,       // 测试中
        Passed = 3,        // 通过
        Failed = 4,        // 失败
        Skipped = 5,       // 跳过
        NotApplicable = 6  // 无需测试 (N/A)
    }

    public static class TestStatusExtensions
    {
        /// <summary>
        /// 将 TestStatus 转为中文显示文本。
        /// </summary>
        public static string ToText(this TestStatus status)
        {
            return status switch
            {
                TestStatus.NotTested      => "未测试",
                TestStatus.Waiting        => "等待测试",
                TestStatus.Testing        => "测试中",
                TestStatus.Passed         => "通过",
                TestStatus.Failed         => "失败",
                TestStatus.Skipped        => "跳过",
                TestStatus.NotApplicable  => "N/A",
                _                         => status.ToString()
            };
        }

        /// <summary>
        /// 判断状态是否为 通过 或 无需测试。
        /// </summary>
        public static bool IsPassOrNA(this TestStatus status) => status == TestStatus.Passed || status == TestStatus.NotApplicable;

        /// <summary>
        /// 将中文/英文状态文本解析为枚举；无法识别返回 NotTested。
        /// </summary>
        public static TestStatus Parse(string txt)
        {
            if (string.IsNullOrWhiteSpace(txt)) return TestStatus.NotTested;
            return txt.Trim() switch
            {
                "通过" or "Passed" => TestStatus.Passed,
                "失败" or "Failed" => TestStatus.Failed,
                "跳过" or "Skipped" => TestStatus.Skipped,
                "等待测试" or "Waiting" => TestStatus.Waiting,
                "测试中" or "Testing" => TestStatus.Testing,
                "N/A" or "NotApplicable" => TestStatus.NotApplicable,
                _ => TestStatus.NotTested
            };
        }
    }

    /// <summary>
    /// 整体测试结果状态（替代 ChannelMapping.TestResultStatus int 字段）。
    /// </summary>
    public enum OverallResultStatus
    {
        NotTested = 0,
        Passed = 1,
        Failed = 2,
        Skipped = 3,
        InProgress = 4 // "测试中 / 进行中"
    }

    public static class OverallResultStatusExtensions
    {
        public static string ToText(this OverallResultStatus status)
        {
            return status switch
            {
                OverallResultStatus.Passed => "通过",
                OverallResultStatus.Failed => "失败",
                OverallResultStatus.Skipped => "跳过",
                OverallResultStatus.InProgress => "测试中",
                _ => "未测试"
            };
        }

        public static OverallResultStatus Parse(int code)
        {
            return code switch
            {
                1 => OverallResultStatus.Passed,
                2 => OverallResultStatus.Failed,
                3 => OverallResultStatus.Skipped,
                4 => OverallResultStatus.InProgress,
                _ => OverallResultStatus.NotTested
            };
        }

        public static int ToCode(this OverallResultStatus status) => (int)status == 4 ? 0 : (int)status;
    }
} 