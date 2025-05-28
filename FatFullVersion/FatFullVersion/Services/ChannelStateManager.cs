using System;
using System.Globalization;
using FatFullVersion.Models;
using FatFullVersion.IServices;
using FatFullVersion.Shared;
using FatFullVersion.Entities;
using System.Linq;

namespace FatFullVersion.Services
{
    /// <summary>
    /// 负责管理 ChannelMapping 对象的核心业务状态转换和规则计算。
    /// </summary>
    public class ChannelStateManager : IChannelStateManager
    {
        // 字符串常量，用于标准化的状态文本
        private const string StatusNotTested = "未测试";
        private const string StatusWaiting = "等待测试";
        private const string StatusTesting = "测试中";
        private const string StatusPassed = "通过";
        private const string StatusFailed = "失败";
        private const string StatusSkipped = "跳过";
        private const string StatusNotApplicable = "N/A";
        private const string StatusManualTesting = "手动测试中";
        private const string StatusHardPointTesting = "硬点通道测试中";
        private const string StatusHardPointPassed = "硬点通道测试成功";
        private const string StatusHardPointFailed = "硬点通道测试失败";

        /// <summary>
        /// 根据从Excel导入的数据初始化ChannelMapping对象的状态。
        /// </summary>
        /// <param name="channel">要初始化的通道映射对象。</param>
        /// <param name="pointData">从Excel读取的点位数据。</param>
        /// <param name="importTime">导入操作的时间戳。</param>
        public void InitializeChannelFromImport(ChannelMapping channel, ExcelPointData pointData, DateTime importTime)
        {
            if (channel == null || pointData == null)
            {
                return;
            }

            // 1. 基础属性映射 (假设部分由调用方完成，或在此补充)
            // channel.VariableName = pointData.VariableName;
            // channel.ModuleType = pointData.ModuleType;
            // channel.TestTag = $"{pointData.StationName}|创建时间:{importTime:yyyy年MM月dd日HH时mm分}";
            channel.Id = Guid.NewGuid(); // 确保每个通道有唯一ID

            // 2. 处理空值与数值类型转换 (float?)
            channel.SLLSetValueNumber = TryParseFloatNullable(pointData.SLLSetValue);
            channel.SLSetValueNumber = TryParseFloatNullable(pointData.SLSetValue);
            channel.SHSetValueNumber = TryParseFloatNullable(pointData.SHSetValue);
            channel.SHHSetValueNumber = TryParseFloatNullable(pointData.SHHSetValue);

            // 2.1 报警设定点通讯地址映射
            channel.SLLSetPointCommAddress = pointData.SLLSetPointCommAddress;
            channel.SLSetPointCommAddress  = pointData.SLSetPointCommAddress;
            channel.SHSetPointCommAddress  = pointData.SHSetPointCommAddress;
            channel.SHHSetPointCommAddress = pointData.SHHSetPointCommAddress;

            // 量程处理：如果Excel中无效或为空，则默认为 0.0-100.0
            // 假设 pointData.RangeLowerLimitValue 和 pointData.RangeUpperLimitValue 是 double 类型
            // 并且 ExcelPointData 模型中对空值已有处理（比如空字符串解析为0，或者有专门的标记）
            // 或者 ExcelPointData 中这两个属性本身就是 nullable double?

            // 更安全的做法是尝试从 pointData 的原始文本（如果可用）解析，或者检查其数值的有效性
            // 这里我们先简化处理，假设 pointData.RangeLowerLimitValue 如果是无效的会是0或者某个标记
            // 我们需要一种方式判断 pointData 中的量程是否真的"未提供"
            
            // 方案：我们假设 ExcelPointData 的量程属性如果是string，我们可以用TryParseFloatNullable
            // 如果 ExcelPointData 的量程属性是double，我们需要一个方法判断它是否是"有效"的初始值（非0，或非某个特定标记）
            // 为了与 SxxSetValue 等保持一致性，我们假设 ExcelPointData 有 RangeLowerLimit 和 RangeUpperLimit 字符串属性
            float? parsedRangeLower = TryParseFloatNullable(pointData.RangeLowerLimit); // 假设 ExcelPointData 有 RangeLowerLimit 字符串属性
            float? parsedRangeUpper = TryParseFloatNullable(pointData.RangeUpperLimit); // 假设 ExcelPointData 有 RangeUpperLimit 字符串属性

            channel.RangeLowerLimitValue = parsedRangeLower ?? 0.0f;
            channel.RangeUpperLimitValue = parsedRangeUpper ?? 100.0f;
            
            // 如果解析后上限仍小于等于下限 (例如都为null导致都为0，或只提供了上限导致下限为0，上限为一个有效值但小于0)
            // 或者是原始数据本身就不合理，这里再校正一次确保上限大于下限
            if (channel.RangeUpperLimitValue <= channel.RangeLowerLimitValue)
            {
                // 如果是因为解析为null导致的，上面已经设了0和100，这里会跳过
                // 如果是原始数据不合理，比如上限填了20，下限填了50，则重置为0-100
                // 或者如果只提供了其中一个，另一个为默认值但导致了不合理，也重置
                if (!(parsedRangeLower == null && parsedRangeUpper == null)) // 避免覆盖纯粹由null导致的0-100默认值
                {
                    // 记录一个警告或日志：量程上下限设置不合理或缺失，已重置为0-100
                    Console.WriteLine($"警告：通道 {channel.VariableName} 量程上下限不合理({pointData.RangeLowerLimit} - {pointData.RangeUpperLimit})，已重置为0-100。");
                    channel.RangeLowerLimitValue = 0.0f;
                    channel.RangeUpperLimitValue = 100.0f;
                }
            }

            // 3. 初始化子测试状态属性
            string moduleTypeUpper = channel.ModuleType?.ToUpper();

            // 默认所有手动子测试项为"未测试"
            channel.ShowValueStatus = StatusNotTested;
            channel.LowLowAlarmStatus = StatusNotTested;
            channel.LowAlarmStatus = StatusNotTested;
            channel.HighAlarmStatus = StatusNotTested;
            channel.HighHighAlarmStatus = StatusNotTested;
            channel.AlarmValueSetStatusEnum = Shared.TestStatus.NotTested;
            channel.MaintenanceFunctionEnum = Shared.TestStatus.NotTested;
            channel.TrendCheckEnum = Shared.TestStatus.NotTested;
            channel.ReportCheckEnum = Shared.TestStatus.NotTested;

            switch (moduleTypeUpper)
            {
                case "AI":
                    // AI 报警状态的特殊初始化逻辑
                    if (channel.SLLSetValueNumber == null && channel.SLSetValueNumber == null)
                    {
                        channel.LowLowAlarmStatusEnum = Shared.TestStatus.NotApplicable;
                        channel.LowAlarmStatusEnum = Shared.TestStatus.NotApplicable;
                    }
                    else
                    {
                        channel.LowLowAlarmStatusEnum = (channel.SLLSetValueNumber == null) ? Shared.TestStatus.NotApplicable : Shared.TestStatus.NotTested;
                        channel.LowAlarmStatusEnum = (channel.SLSetValueNumber == null) ? Shared.TestStatus.NotApplicable : Shared.TestStatus.NotTested;
                    }

                    if (channel.SHSetValueNumber == null && channel.SHHSetValueNumber == null)
                    {
                        channel.HighAlarmStatusEnum = Shared.TestStatus.NotApplicable;
                        channel.HighHighAlarmStatusEnum = Shared.TestStatus.NotApplicable;
                    }
                    else
                    {
                        channel.HighAlarmStatusEnum = (channel.SHSetValueNumber == null) ? Shared.TestStatus.NotApplicable : Shared.TestStatus.NotTested;
                        channel.HighHighAlarmStatusEnum = (channel.SHHSetValueNumber == null) ? Shared.TestStatus.NotApplicable : Shared.TestStatus.NotTested;
                    }

                    // AlarmValueSetStatus 判定
                    bool allAlarmsConfiguredAndPassedOrNAForAI =
                        (channel.LowLowAlarmStatusEnum == Shared.TestStatus.Passed || channel.LowLowAlarmStatusEnum == Shared.TestStatus.NotApplicable) &&
                        (channel.LowAlarmStatusEnum == Shared.TestStatus.Passed || channel.LowAlarmStatusEnum == Shared.TestStatus.NotApplicable) &&
                        (channel.HighAlarmStatusEnum == Shared.TestStatus.Passed || channel.HighAlarmStatusEnum == Shared.TestStatus.NotApplicable) &&
                        (channel.HighHighAlarmStatusEnum == Shared.TestStatus.Passed || channel.HighHighAlarmStatusEnum == Shared.TestStatus.NotApplicable);
                    
                    // 仅当所有相关报警项都明确配置（即其数值不为null导致状态为Passed/NA）时，才认为报警值设定需要测试
                    // 或者更简单的规则：如果 Excel 中所有报警设定值文本都为空，则 AlarmValueSetStatus 通过
                    bool allRawAlarmSettingsEmptyForAI = string.IsNullOrWhiteSpace(pointData.SLLSetValue) &&
                                                   string.IsNullOrWhiteSpace(pointData.SLSetValue) &&
                                                   string.IsNullOrWhiteSpace(pointData.SHSetValue) &&
                                                   string.IsNullOrWhiteSpace(pointData.SHHSetValue);

                    if (allRawAlarmSettingsEmptyForAI) // 如果原始excel文本都为空，则报警设定通过
                    {
                        channel.AlarmValueSetStatusEnum = Shared.TestStatus.NotApplicable;
                    }
                    // （如果上一个条件不满足，它会保持 StatusNotTested）
                    break;

                case "AO":
                    channel.LowLowAlarmStatusEnum = Shared.TestStatus.NotApplicable;
                    channel.LowAlarmStatusEnum = Shared.TestStatus.NotApplicable;
                    channel.HighAlarmStatusEnum = Shared.TestStatus.NotApplicable;
                    channel.HighHighAlarmStatusEnum = Shared.TestStatus.NotApplicable;
                    channel.AlarmValueSetStatusEnum = Shared.TestStatus.NotApplicable;
                    // 根据 DataEditViewModel 旧逻辑，AO的MaintenanceFunction在导入时即为通过
                    channel.MaintenanceFunctionEnum = Shared.TestStatus.Passed;
                    break;

                case "DI":
                case "DINone":
                case "DO":
                case "DONone":
                    channel.LowLowAlarmStatusEnum = Shared.TestStatus.NotApplicable;
                    channel.LowAlarmStatusEnum = Shared.TestStatus.NotApplicable;
                    channel.HighAlarmStatusEnum = Shared.TestStatus.NotApplicable;
                    channel.HighHighAlarmStatusEnum = Shared.TestStatus.NotApplicable;
                    channel.AlarmValueSetStatusEnum = Shared.TestStatus.NotApplicable;
                    channel.MaintenanceFunctionEnum = Shared.TestStatus.NotApplicable;
                    channel.TrendCheckEnum = Shared.TestStatus.NotApplicable;
                    channel.ReportCheckEnum = Shared.TestStatus.NotApplicable;
                    // DI/DO 的 RangeLowerLimitValue 和 RangeUpperLimitValue 通常是无效的
                    // DataEditViewModel 中设为了 float.NaN。我们可以在这里也这样做，或者依赖于 pointData 中的值。
                    // 如果pointData中这些已经是NaN，那么channel中也会是NaN。
                    // 或者，如果我们希望在这里强制，可以：
                    // channel.RangeLowerLimitValue = float.NaN;
                    // channel.RangeUpperLimitValue = float.NaN;
                    // 同时 ValueXPercent 系列也应为N/A或null (已为float?)
                    channel.Value0Percent = float.NaN;
                    channel.Value25Percent = float.NaN;
                    channel.Value50Percent = float.NaN;
                    channel.Value75Percent = float.NaN;
                    channel.Value100Percent = float.NaN;
                    channel.RangeLowerLimitValue = float.NaN;
                    channel.RangeUpperLimitValue = float.NaN;
                    channel.RangeLowerLimit = "N/A";
                    channel.RangeUpperLimit = "N/A";
                    break;
                
                // TODO: 根据需要为 AINone, AONone, DINone, DONone 等添加特定初始化逻辑
            }

            // 4. 初始化核心测试状态
            channel.HardPointStatus = Shared.TestStatus.NotTested;
            channel.ResultText = StatusNotTested; 
            channel.OverallStatus = OverallResultStatus.NotTested;
            if (channel.Status != StatusSkipped) // 如果之前由于某种原因被跳过，则不覆盖已有的Status
            {
                channel.Status = StatusNotTested; // 如果Status属性保留并使用
            }

            // 5. 初始化时间戳
            channel.StartTime = null;
            channel.TestTime = null;
            channel.FinalTestTime = null;
            
            // 6. 调用EvaluateOverallStatus
            // 在初始化阶段，这通常只是确认状态为"未测试"，但调用它以保持流程一致性
            EvaluateOverallStatus(channel); 

            // --- 预留点位（变量名含 YLDW）特殊处理 -----------------------------------------
            if (!string.IsNullOrEmpty(channel.VariableName) && channel.VariableName.IndexOf("YLDW", StringComparison.OrdinalIgnoreCase) >= 0)
            {
                // 仅保留 0%~100% 硬点测试，其他手动测试全部设为 N/A
                channel.ShowValueStatus = StatusNotApplicable;
                channel.LowLowAlarmStatusEnum = Shared.TestStatus.NotApplicable;
                channel.LowAlarmStatusEnum = Shared.TestStatus.NotApplicable;
                channel.HighAlarmStatusEnum = Shared.TestStatus.NotApplicable;
                channel.HighHighAlarmStatusEnum = Shared.TestStatus.NotApplicable;
                channel.AlarmValueSetStatusEnum = Shared.TestStatus.NotApplicable;
                channel.MaintenanceFunctionEnum = Shared.TestStatus.NotApplicable;
                channel.TrendCheckEnum = Shared.TestStatus.NotApplicable;
                channel.ReportCheckEnum = Shared.TestStatus.NotApplicable;
            }
        }

        /// <summary>
        /// 将通道的测试相关属性重置为初始导入后的状态（不包括从Excel直接读取的配置型状态）。
        /// </summary>
        private void ResetTestRelatedPropertiesToInitial(ChannelMapping channel)
        {
            if (channel == null) return;

            // 保留由 InitializeChannelFromImport 根据 ExcelPointData 设置的子状态的初始值（如某些报警为N/A或Passed）
            // 但清除那些在测试过程中会被改变的状态或结果。

            // 核心测试状态
            channel.HardPointStatus = Shared.TestStatus.NotTested;
            channel.ResultText = StatusNotTested;
            channel.OverallStatus = OverallResultStatus.NotTested;

            // 时间戳
            channel.StartTime = null;
            channel.TestTime = null;
            channel.FinalTestTime = null;

            // 手动测试子状态 - 通常在 InitializeChannelFromImport 时已根据配置设定。
            // 如果分配操作意味着所有子测试也需要重新开始，则重置为 NotTested，
            // 除非它们在初始化时因配置（如报警未配置）而被设为 N/A 或 Passed。
            // 为简单起见，这里我们假设分配后，如果这些状态不是N/A，就应该回到NotTested。
            // InitializeChannelFromImport 已经处理了基于Excel配置的初始N/A或Passed。
            // 所以这里主要是确保那些可能在测试中被改变的状态回到 NotTested。

            Action<Func<string>, Action<string>> resetIfNotNA = (getter, setter) =>
            {
                if (getter() != StatusNotApplicable)
                {
                    setter(StatusNotTested);
                }
            };
            
            // resetIfNotNA(() => channel.ShowValueStatus, status => channel.ShowValueStatus = status); // ShowValueStatus 总会被 Initialize 设为 NotTested
            // InitializeChannelFromImport 会设置这些，所以这里可能不需要再次重置，除非分配逻辑需要更彻底的清除。
            // 但为了确保分配后是一个干净的待测状态（对于那些可测项），我们仅重置那些非永久N/A的状态。
            // 简化：我们依赖 InitializeChannelFromImport 来设定好哪些是 NotApplicable。
            // 分配操作本身不改变这些"配置性"状态，只重置"测试过程"状态。
            // 因此，上面对HardPointTestResult, ResultText, TestResultStatus, 时间戳的重置是主要的。
            // 如果一个通道被重新分配，它的子测试项的"通过/失败"结果也应该被清除，回到"未测试"。
            // 但那些本来就是"N/A"的（因为未配置）应该保持"N/A"。

            // 一个更精细的重置方式是重新调用Initialize的部分逻辑，但不重新解析Excel
            // 或者，我们可以在InitializeChannelFromImport中设置好所有初始状态，
            // 然后这里只重置那些肯定会被测试流程修改的顶层状态。

            // 决定：ApplyAllocationInfo 和 ClearAllocationInfo 将会重置测试结果，
            // 并且子项状态也应该回到"未测试"，除非它们一开始就是N/A。
            // InitializeChannelFromImport 已经负责根据Excel设定哪些是N/A。
            // 所以，我们这里也需要将非N/A的子项重置回NotTested。

            channel.ShowValueStatus = channel.ShowValueStatus == StatusNotApplicable ? StatusNotApplicable : StatusNotTested;

            channel.LowLowAlarmStatusEnum = channel.LowLowAlarmStatusEnum == Shared.TestStatus.NotApplicable ? Shared.TestStatus.NotApplicable : Shared.TestStatus.NotTested;
            channel.LowAlarmStatusEnum = channel.LowAlarmStatusEnum == Shared.TestStatus.NotApplicable ? Shared.TestStatus.NotApplicable : Shared.TestStatus.NotTested;
            channel.HighAlarmStatusEnum = channel.HighAlarmStatusEnum == Shared.TestStatus.NotApplicable ? Shared.TestStatus.NotApplicable : Shared.TestStatus.NotTested;
            channel.HighHighAlarmStatusEnum = channel.HighHighAlarmStatusEnum == Shared.TestStatus.NotApplicable ? Shared.TestStatus.NotApplicable : Shared.TestStatus.NotTested;
            channel.AlarmValueSetStatusEnum = channel.AlarmValueSetStatusEnum == Shared.TestStatus.NotApplicable ? Shared.TestStatus.NotApplicable : Shared.TestStatus.NotTested;
            
            // MaintenanceFunction 对于AO可能是Passed，对于AI是NotTested或N/A
            if(channel.ModuleType?.ToUpper() == "AO" && channel.MaintenanceFunctionEnum == Shared.TestStatus.Passed) {}
            else if (channel.MaintenanceFunctionEnum != Shared.TestStatus.NotApplicable) { channel.MaintenanceFunctionEnum = Shared.TestStatus.NotTested; }
            
            channel.TrendCheckEnum = channel.TrendCheckEnum == Shared.TestStatus.NotApplicable ? Shared.TestStatus.NotApplicable : Shared.TestStatus.NotTested;
            channel.ReportCheckEnum = channel.ReportCheckEnum == Shared.TestStatus.NotApplicable ? Shared.TestStatus.NotApplicable : Shared.TestStatus.NotTested;
        }

        /// <summary>
        /// 应用通道分配信息到ChannelMapping对象，并重置测试状态。
        /// </summary>
        /// <param name="channel">要应用分配的通道映射对象。</param>
        /// <param name="testBatch">分配的测试批次名称。</param>
        /// <param name="testPlcChannelTag">分配的测试PLC通道标签。</param>
        /// <param name="testPlcCommAddress">分配的测试PLC通讯地址。</param>
        public void ApplyAllocationInfo(ChannelMapping channel, string testBatch, string testPlcChannelTag, string testPlcCommAddress)
        {
            if (channel == null) return;

            channel.TestBatch = testBatch;
            channel.TestPLCChannelTag = testPlcChannelTag;
            channel.TestPLCCommunicationAddress = testPlcCommAddress;

            // 应用新的分配信息，意味着之前的测试结果和进度无效，需要重置
            ResetTestRelatedPropertiesToInitial(channel);
            
            // 重新评估整体状态，通常会是"未测试"
            EvaluateOverallStatus(channel);
        }

        /// <summary>
        /// 清除ChannelMapping对象的通道分配信息，并重置测试状态到初始导入状态。
        /// </summary>
        /// <param name="channel">要清除分配的通道映射对象。</param>
        public void ClearAllocationInfo(ChannelMapping channel)
        {
            if (channel == null) return;

            channel.TestBatch = string.Empty; // 或者 null，取决于模型的喜好和数据库如何处理
            channel.TestPLCChannelTag = string.Empty;
            channel.TestPLCCommunicationAddress = string.Empty;

            // 清除分配信息后，通道回到完全未测试的初始状态
            ResetTestRelatedPropertiesToInitial(channel);

            // 重新评估整体状态，应为"未测试"
            EvaluateOverallStatus(channel);
        }

        /// <summary>
        /// 将ChannelMapping对象标记为已跳过测试。
        /// </summary>
        /// <param name="channel">要标记为跳过的通道映射对象。</param>
        /// <param name="reason">跳过的原因。</param>
        /// <param name="skipTime">执行跳过操作的时间戳。</param>
        public void MarkAsSkipped(ChannelMapping channel, string reason, DateTime skipTime)
        {
            if (channel == null) return;

            // 设置整体状态为跳过
            channel.OverallStatus = OverallResultStatus.Skipped;
            
            // 设置硬点测试状态为跳过，这会自动更新HardPointTestResult
            channel.HardPointStatus = Shared.TestStatus.Skipped;
            
            channel.ResultText = string.IsNullOrWhiteSpace(reason) ? "已跳过测试" : $"已跳过测试，原因: {reason}";
            channel.FinalTestTime = skipTime;
            
            if (channel.Status != StatusSkipped) // 如果 Status 属性保留并使用
            {
                channel.Status = StatusSkipped;
            }

            // 将所有手动子测试状态设置为不适用，因为测试被跳过了
            channel.ShowValueStatus = StatusNotApplicable;
            channel.LowLowAlarmStatusEnum = Shared.TestStatus.NotApplicable;
            channel.LowAlarmStatusEnum = Shared.TestStatus.NotApplicable;
            channel.HighAlarmStatusEnum = Shared.TestStatus.NotApplicable;
            channel.HighHighAlarmStatusEnum = Shared.TestStatus.NotApplicable;
            channel.AlarmValueSetStatusEnum = Shared.TestStatus.NotApplicable;
            channel.MaintenanceFunctionEnum = Shared.TestStatus.NotApplicable;
            channel.TrendCheckEnum = Shared.TestStatus.NotApplicable;
            channel.ReportCheckEnum = Shared.TestStatus.NotApplicable;

            // 时间戳，StartTime 和 TestTime 如果之前有值，可以保留，也可以清除，取决于业务定义
            // 通常跳过测试意味着之前的测试过程（如果有）也中止了。
            // 为了清晰，这里也清空 StartTime 和 TestTime，因为 FinalTestTime 已被设置。
            // channel.StartTime = null; // 可选，取决于是否要保留开始跳过前的信息
            // channel.TestTime = null;  // 可选

            // 调用 EvaluateOverallStatus 主要是为了代码流程的统一性，
            // 实际上 EvaluateOverallStatus 的规则0会直接处理 TestResultStatus = 3 的情况。
            EvaluateOverallStatus(channel, skipTime);
        }

        /// <summary>
        /// 准备ChannelMapping对象以进行接线确认（通常设置为等待测试）。
        /// </summary>
        /// <param name="channel">要准备接线确认的通道映射对象。</param>
        /// <param name="confirmTime">执行接线确认的时间戳。</param>
        public void PrepareForWiringConfirmation(ChannelMapping channel, DateTime confirmTime)
        {
            if (channel == null) return;

            // 只有当通道当前是"未测试"状态时，才将其更新为"等待测试"
            // 如果已经是其他状态（如"测试中"、"失败"、"跳过"等），则不应更改
            if (channel.HardPointStatus == Shared.TestStatus.NotTested || string.IsNullOrEmpty(channel.ResultText))
            {
                channel.HardPointStatus = Shared.TestStatus.Waiting;
                channel.ResultText = StatusWaiting; // 或者更具体的如 "接线已确认，等待测试开始"
                // TestResultStatus 保持为 0 (未测试)
            }
            // 如果通道已分配但未进行任何测试（例如 OverallStatus 为 NotTested 且 HardPointStatus 为 NotTested）
            // 也应该更新为等待测试
            else if (channel.OverallStatus == OverallResultStatus.NotTested && 
                     (channel.HardPointStatus == Shared.TestStatus.NotTested || string.IsNullOrEmpty(channel.ResultText)))
            {
                channel.HardPointStatus = Shared.TestStatus.Waiting;
                channel.ResultText = StatusWaiting; 
            }

            // 调用 EvaluateOverallStatus 以确保状态一致性，尽管此操作本身不应改变 TestResultStatus 从0到其他终态
            // 但它可以帮助标准化 ResultText (如果 EvaluateOverallStatus 中有相应逻辑)
            EvaluateOverallStatus(channel, null); // confirmTime 不是测试完成时间，所以不传递给 eventTimeForFinalTest
        }

        /// <summary>
        /// 开始硬点测试前，设置ChannelMapping的初始测试中状态。
        /// </summary>
        /// <param name="channel">要开始硬点测试的通道映射对象。</param>
        /// <param name="startTime">硬点测试开始的时间戳。</param>
        public void BeginHardPointTest(ChannelMapping channel, DateTime startTime)
        {
            if (channel == null) return;

            channel.TestTime = startTime;
            if (channel.StartTime == null || channel.StartTime == DateTime.MinValue) // DateTime不是nullable，所以检查MinValue
            {
                channel.StartTime = startTime;
            }
            channel.FinalTestTime = null; 

            channel.HardPointStatus = Shared.TestStatus.Testing;
            
            // 更新ResultText，如果已经是某种"测试中"的状态，避免重复或不当覆盖
            if (channel.ResultText == StatusWaiting || channel.ResultText == StatusNotTested || string.IsNullOrEmpty(channel.ResultText))
            {
                channel.ResultText = StatusHardPointTesting;
            }
            else if (!channel.ResultText.Contains(StatusTesting)) // 如果不包含任何"测试中"的字样
            {
                channel.ResultText = $"{channel.ResultText}, {StatusHardPointTesting}";
            }
            // else: 如果ResultText已经是更具体的测试中状态，例如手动测试中，则暂时不覆盖，后续SetHardPointTestOutcome会处理
            // 但通常硬点测试优先于或独立于手动测试的文本状态

            channel.OverallStatus = OverallResultStatus.InProgress;

            EvaluateOverallStatus(channel, null);
        }

        /// <summary>
        /// 根据原始的硬点测试结果，更新ChannelMapping的状态。
        /// 此方法内部会调用 EvaluateOverallStatus 来重新评估整体状态。
        /// </summary>
        /// <param name="channel">要更新硬点测试结果的通道映射对象。</param>
        /// <param name="rawOutcome">原始的硬点测试结果。</param>
        /// <param name="outcomeTime">硬点测试结果产生的时间戳。</param>
        public void SetHardPointTestOutcome(ChannelMapping channel, HardPointTestRawResult rawOutcome, DateTime outcomeTime)
        {
            if (channel == null) return;

            if (rawOutcome.IsSuccess)
            {
                channel.HardPointStatus = Shared.TestStatus.Passed;
                channel.HardPointErrorDetail = null;
            }
            else
            {
                channel.HardPointStatus = Shared.TestStatus.Failed;
                channel.HardPointErrorDetail = rawOutcome.Detail;
            }

            channel.TestTime = outcomeTime; // 记录硬点测试结果的确切时间

            // EvaluateOverallStatus 会根据新的 HardPointTestResult 和其他子项状态来决定 TestResultStatus, ResultText 和 FinalTestTime
            EvaluateOverallStatus(channel, outcomeTime);
        }

        /// <summary>
        /// 开始手动测试前，准备ChannelMapping的状态（例如，重置相关子测试项为"未测试"）。
        /// </summary>
        /// <param name="channel">要开始手动测试的通道映射对象。</param>
        public void BeginManualTest(ChannelMapping channel)
        {
            if (channel == null) return;
            // 如果通道已经是最终失败或跳过状态，则不应开始手动测试或重置状态
            if (channel.OverallStatus == OverallResultStatus.Failed || channel.OverallStatus == OverallResultStatus.Skipped)
            {
                return;
            }

            string moduleTypeUpper = channel.ModuleType?.ToUpper();

            // 1. 重置非最终状态的子测试项为"未测试"
            // 保留已经明确为 Passed 或 NotApplicable 的状态
            Action<Func<string>, Action<string>> resetSubTest = (getter, setter) =>
            {
                var currentStatus = getter();
                if (currentStatus != StatusPassed && currentStatus != StatusNotApplicable)
                {
                    setter(StatusNotTested);
                }
            };
            
            resetSubTest(() => channel.ShowValueStatus, status => channel.ShowValueStatus = status);

            if (moduleTypeUpper == "AI")
            {
                resetSubTest(() => channel.LowLowAlarmStatus, status => channel.LowLowAlarmStatus = status);
                resetSubTest(() => channel.LowAlarmStatus, status => channel.LowAlarmStatus = status);
                resetSubTest(() => channel.HighAlarmStatus, status => channel.HighAlarmStatus = status);
                resetSubTest(() => channel.HighHighAlarmStatus, status => channel.HighHighAlarmStatus = status);
                resetSubTest(() => channel.AlarmValueSetStatus, status => channel.AlarmValueSetStatus = status);
                resetSubTest(() => channel.MaintenanceFunction, status => channel.MaintenanceFunction = status);
                resetSubTest(() => channel.TrendCheck, status => channel.TrendCheck = status);
                resetSubTest(() => channel.ReportCheck, status => channel.ReportCheck = status);
            }
            else if (moduleTypeUpper == "AO")
            {
                // AO的MaintenanceFunction在Initialize时可能已设为Passed，这里不应重置为NotTested，除非逻辑要求
                if (channel.MaintenanceFunctionEnum != Shared.TestStatus.Passed && channel.MaintenanceFunctionEnum != Shared.TestStatus.NotApplicable) 
                { 
                    channel.MaintenanceFunctionEnum = Shared.TestStatus.NotTested; 
                }
                resetSubTest(() => channel.TrendCheck, status => channel.TrendCheck = status);
                resetSubTest(() => channel.ReportCheck, status => channel.ReportCheck = status);
            }
            // DI/DO 只有 ShowValueStatus，已处理

            // 2. 更新 ResultText
            if (channel.HardPointStatus == Shared.TestStatus.Passed || channel.HardPointStatus == Shared.TestStatus.NotApplicable || string.IsNullOrEmpty(channel.ResultText))
            {
                // 如果硬点OK或不适用，则主要显示手动测试中
                channel.ResultText = StatusManualTesting;
            }
            else if (channel.HardPointStatus == Shared.TestStatus.Testing || channel.HardPointStatus == Shared.TestStatus.Waiting)
            {
                // 如果硬点仍在进行，ResultText可能已是HardPointTesting，暂时不覆盖或附加
                // 或者根据优先级决定是否要显示 "硬点测试中，准备手动测试"
                 channel.ResultText = $"{channel.HardPointStatus}, {StatusManualTesting}";
            }
            // 如果硬点失败，则 ResultText 应已反映失败，不应被"手动测试中"覆盖。
            // 此情况已在方法开头通过检查 TestResultStatus == 2 避免。
            
            // 3. 清除最终测试时间
            channel.FinalTestTime = null;

            // 4. TestResultStatus 保持 0 (或当前非终态)，因为手动测试刚开始
            // 如果之前是硬点通过但整体未完成，TestResultStatus 可能是0。如果硬点失败，则为2。
            // 我们在方法开始时检查了 TestResultStatus 是否为2或3，所以到这里它应该是0或1 (如果硬点通过且无手动项)
            // 如果是1，现在开始手动测试，应将其暂时拨回0。
            if(channel.OverallStatus == OverallResultStatus.Passed) //之前是全通过，现在开始手动（或部分重测手动）
            {
                channel.OverallStatus = OverallResultStatus.NotTested;
            }
            
            // 5. 调用 EvaluateOverallStatus
            EvaluateOverallStatus(channel, null);
        }

        /// <summary>
        /// 记录手动测试中某个子项的测试结果，并重新评估整体状态。
        /// 此方法内部会调用 EvaluateOverallStatus 来重新评估整体状态。
        /// </summary>
        /// <param name="channel">要记录手动子测试结果的通道映射对象。</param>
        /// <param name="itemType">手动测试的子项类型。</param>
        /// <param name="isSuccess">该子项测试是否通过。</param>
        /// <param name="outcomeTime">手动子测试结果产生的时间戳。</param>
        /// <param name="details">如果测试失败，相关的详细信息（可选）。</param>
        public void SetManualSubTestOutcome(ChannelMapping channel, ManualTestItem itemType, bool isSuccess, DateTime outcomeTime, string details = null)
        {
            if (channel == null) return;

            // 如果通道已经是最终失败或跳过状态，则不允许修改子测试结果
            if (channel.OverallStatus == OverallResultStatus.Failed || channel.OverallStatus == OverallResultStatus.Skipped)
            {
                // 可以考虑记录一个警告日志：尝试修改已处于终态的通道的子测试结果
                return;
            }

            string outcomeStatus = isSuccess ? StatusPassed : StatusFailed;
            string previousResultText = channel.ResultText; // 保存一下，可能后面用到

            switch (itemType)
            {
                case ManualTestItem.ShowValue:
                    channel.ShowValueStatusEnum = isSuccess ? Shared.TestStatus.Passed : Shared.TestStatus.Failed;
                    break;
                case ManualTestItem.LowLowAlarm:
                    channel.LowLowAlarmStatusEnum = isSuccess ? Shared.TestStatus.Passed : Shared.TestStatus.Failed;
                    break;
                case ManualTestItem.LowAlarm:
                    channel.LowAlarmStatusEnum = isSuccess ? Shared.TestStatus.Passed : Shared.TestStatus.Failed;
                    break;
                case ManualTestItem.HighAlarm:
                    channel.HighAlarmStatusEnum = isSuccess ? Shared.TestStatus.Passed : Shared.TestStatus.Failed;
                    break;
                case ManualTestItem.HighHighAlarm:
                    channel.HighHighAlarmStatusEnum = isSuccess ? Shared.TestStatus.Passed : Shared.TestStatus.Failed;
                    break;
                case ManualTestItem.AlarmValueSet:
                    channel.AlarmValueSetStatusEnum = isSuccess ? Shared.TestStatus.Passed : Shared.TestStatus.Failed;
                    break;
                case ManualTestItem.MaintenanceFunction:
                    channel.MaintenanceFunctionEnum = isSuccess ? Shared.TestStatus.Passed : Shared.TestStatus.Failed;
                    break;
                case ManualTestItem.TrendCheck:
                    channel.TrendCheckEnum = isSuccess ? Shared.TestStatus.Passed : Shared.TestStatus.Failed;
                    break;
                case ManualTestItem.ReportCheck:
                    channel.ReportCheckEnum = isSuccess ? Shared.TestStatus.Passed : Shared.TestStatus.Failed;
                    break;
                default:
                    // 无效的 itemType，可以记录错误或忽略
                    return;
            }
            
            // 如果失败，并且提供了details，可以考虑将其附加到ResultText的某个地方，
            // 但EvaluateOverallStatus中的ConstructFailedResultText通常会生成更全面的失败描述
            // if (!isSuccess && !string.IsNullOrWhiteSpace(details)) {
            //     channel.ResultText = details; // 或者附加到 general failure reason
            // }

            channel.TestTime = outcomeTime; // 记录本次子测试项确认的时间

            // 核心：重新评估整体状态
            // EvaluateOverallStatus 会根据所有子项（包括刚更新的这个）和硬点状态来设定 TestResultStatus, ResultText, FinalTestTime
            EvaluateOverallStatus(channel, outcomeTime);

            // 如果更新了 ShowValueStatus，则同步枚举字段
            if(itemType == ManualTestItem.ShowValue)
            {
                channel.ShowValueStatusEnum = isSuccess ? Shared.TestStatus.Passed : Shared.TestStatus.Failed;
            }
        }

        /// <summary>
        /// 重置ChannelMapping的状态以便进行重新测试。
        /// </summary>
        /// <param name="channel">要重置状态以进行复测的通道映射对象。</param>
        public void ResetForRetest(ChannelMapping channel)
        {
            if (channel == null) return;

            // 调用辅助方法重置所有测试相关属性到初始状态
            ResetTestRelatedPropertiesToInitial(channel);

            // 重新评估整体状态。此时 TestResultStatus 应该为 0 (未测试)，
            // ResultText 应该为 StatusNotTested。
            EvaluateOverallStatus(channel, null); // 传入 null 是因为复测开始并不代表最终完成时间
        }

        /// <summary>
        /// 评估并设置通道的整体测试状态。
        /// 这是所有状态变更方法最终调用的核心逻辑。
        /// </summary>
        /// <param name="channel">需要评估的通道对象。</param>
        /// <param name="eventTimeForFinalTest">如果测试因此评估而最终完成（通过、失败、跳过），则使用此时间作为FinalTestTime。传入null则不主动设置FinalTestTime，除非状态明确结束。</param>
        private void EvaluateOverallStatus(ChannelMapping channel, DateTime? eventTimeForFinalTest = null)
        {
            if (channel == null) return;

            bool IsNA(string status) => string.IsNullOrWhiteSpace(status) || status == StatusNotApplicable;

            // 规则 0: 如果已被标记为跳过 (TestResultStatus = 3)
            // 跳过状态由 MarkAsSkipped 方法直接设定，通常包含最终时间和结果文本。
            // EvaluateOverallStatus 在此情况下主要用于确保不会覆盖已明确的跳过状态。
            if (channel.OverallStatus == OverallResultStatus.Skipped) // 3 代表"跳过"
            {
                // 通常 MarkAsSkipped 已设置 FinalTestTime 和 ResultText。
                // 若 Status 属性保留, channel.Status = StatusSkipped;
                return;
            }

            // 在评估开始前，不应急于重置 TestResultStatus 为0，除非是特定的重置流程。
            // 保留当前状态，让后续逻辑决定是否覆盖。
            // bool currentOverallStatusIsTerminal = channel.TestResultStatus == 1 || channel.TestResultStatus == 2 || channel.TestResultStatus == 3;

            bool anyManualSubTestFailed = false;
            bool anyManualSubTestNotTested = false;

            var manualStatuses = new System.Collections.Generic.List<TestStatus>
            {
                channel.ShowValueStatusEnum
            };

            string moduleTypeUpper = channel.ModuleType?.ToUpper();

            if (moduleTypeUpper == "AI")
            {
                manualStatuses.AddRange(new[]{
                    channel.LowLowAlarmStatusEnum,
                    channel.LowAlarmStatusEnum,
                    channel.HighAlarmStatusEnum,
                    channel.HighHighAlarmStatusEnum,
                    channel.AlarmValueSetStatusEnum,
                    channel.MaintenanceFunctionEnum,
                    channel.TrendCheckEnum,
                    channel.ReportCheckEnum
                });
            }
            else if (moduleTypeUpper == "AO")
            {
                manualStatuses.AddRange(new[]{
                    channel.MaintenanceFunctionEnum,
                    channel.TrendCheckEnum,
                    channel.ReportCheckEnum
                });
            }

            // DI/DO 无额外子项

            anyManualSubTestFailed = manualStatuses.Exists(s => s == TestStatus.Failed);
            anyManualSubTestNotTested = manualStatuses.Exists(s => s == TestStatus.NotTested || s == TestStatus.Waiting || s == TestStatus.Testing);

            // --- 判定整体状态 --- 

            // 规则 1: 任何一个手动子测试失败，则整体测试失败。
            if (anyManualSubTestFailed)
            {
                channel.OverallStatus = OverallResultStatus.Failed;
                channel.HardPointErrorDetail = ConstructFailedResultText(channel, "手动测试不通过");
                channel.ResultText = StatusFailed;
                channel.FinalTestTime = eventTimeForFinalTest ?? DateTime.Now; 
                return;
            }

            // 规则 2: 硬点测试失败，则整体测试失败。
            // (HardPointTestResult 的失败状态由 SetHardPointTestOutcome 方法设置)
            if (channel.HardPointStatus == Shared.TestStatus.Failed || (channel.HardPointStatus != null && channel.ResultText.StartsWith(StatusFailed))) // 更稳健的失败检查
            {
                channel.OverallStatus = OverallResultStatus.Failed;
                channel.ResultText = channel.ResultText;
                channel.FinalTestTime = eventTimeForFinalTest ?? DateTime.Now;
                return;
            }

            // 规则 3: 所有测试环节均已完成且无失败。
            bool hardPointIsOkOrNotApplicable = 
                channel.HardPointStatus == Shared.TestStatus.Passed || 
                channel.HardPointStatus == Shared.TestStatus.Skipped || // 跳过的硬点也视为OK，因为跳过是种完成状态
                channel.HardPointStatus == Shared.TestStatus.NotApplicable || 
                string.IsNullOrEmpty(channel.ResultText) || // 对于没有硬点测试环节的类型
                channel.HardPointStatus == Shared.TestStatus.NotTested; // 如果硬点未测，但所有手动项都通过了 (这个逻辑需审慎，通常硬点是前提)
                                                              // 更安全的做法是：如果一个类型有硬点测试要求，则 HardPointTestResult 必须是 StatusPassed 或 StatusNotApplicable
            
            // DI/DO 仍需等待硬点测试明确为 Passed 或标记为 N/A，
            // 因此不再强制将 strictHardPointPassed 设置为 true。
            
            // 硬点测试必须先通过，之后才允许整体通过
            bool strictHardPointPassed = channel.HardPointStatus == Shared.TestStatus.Passed;
            if (strictHardPointPassed && !anyManualSubTestNotTested)
            {
                channel.OverallStatus = OverallResultStatus.Passed;
                channel.ResultText = StatusPassed;
                channel.FinalTestTime = eventTimeForFinalTest ?? DateTime.Now;
                return;
            }
            
            // 规则 4: 测试仍在进行中。
            // 如果硬点测试是"测试中"或"等待测试"
            if (channel.HardPointStatus == Shared.TestStatus.Testing || channel.HardPointStatus == Shared.TestStatus.Waiting)
            {
                channel.OverallStatus = OverallResultStatus.InProgress;
                // ResultText 已由 BeginHardPointTest 或 PrepareForWiringConfirmation 设置
                channel.FinalTestTime = null; 
                return;
            }
            // 如果手动测试项有未完成的 (不是Pass/Fail/NA，即还是NotTested)
            if (anyManualSubTestNotTested && !anyManualSubTestFailed && channel.HardPointStatus != Shared.TestStatus.NotTested) 
            {
                channel.OverallStatus = OverallResultStatus.InProgress;
                channel.ResultText = StatusManualTesting; 
                // 可以构建更详细的 ResultText, e.g., "手动测试中: XXX项未完成"
                channel.FinalTestTime = null; 
                return;
            }

            // 规则 4.1: 硬点未测试且尚无失败时保持"未测试"/初始状态
            if (channel.HardPointStatus == Shared.TestStatus.NotTested && !anyManualSubTestFailed)
            {
                channel.OverallStatus = OverallResultStatus.NotTested;
                channel.ResultText = StatusNotTested;
                channel.FinalTestTime = null;
                return;
            }

            // 规则 5: 回退或默认情况，例如硬点测试通过，但手动部分尚未全部完成且无失败，或刚初始化。
            // 此时 TestResultStatus 保持或设为 0。
            // ResultText 应反映当前所处的阶段。
            channel.OverallStatus = OverallResultStatus.InProgress;
            if (strictHardPointPassed && anyManualSubTestNotTested)
            {
                channel.ResultText = $"{StatusHardPointPassed}, 等待{StatusManualTesting}";
            }
            else if (channel.HardPointStatus == Shared.TestStatus.NotTested && anyManualSubTestNotTested)
            {
                channel.ResultText = StatusManualTesting; // 假设可以直接开始手动测试
            }
            else if (channel.HardPointStatus == Shared.TestStatus.NotTested && !anyManualSubTestFailed && anyManualSubTestNotTested)
            {
                // 硬点未测，手动项已完成但并非全部通过（也非全部失败，否则前面已返回）
                // 这种情况理论上不应发生，因为 anyManualSubTestNotTested 为 false 意味着有未测试项
                // 或者是 anyManualSubTestNotTested 为true (有未测试项）
                // 这里更像是状态不一致，应倾向于标记为测试中或根据未通过项标记失败。
                // 为安全起见，标记为测试中。
                channel.ResultText = StatusTesting; 
            }
            // 如果是刚初始化的状态，ResultText 应该已经是 StatusNotTested
            // 如果没有特定条件匹配，保留由前置步骤（如BeginManualTest）设定的ResultText，或设为通用测试中
            else if (string.IsNullOrEmpty(channel.ResultText) || channel.ResultText == StatusPassed || channel.ResultText == StatusFailed)
            {
                 // 如果ResultText是终态，但我们到这里说明还没完全结束，则修正它
                 channel.ResultText = StatusTesting;
            }
            
            channel.FinalTestTime = null; // 明确测试尚未最终完成
        }

        /// <summary>
        /// 辅助方法（占位），用于构建详细的失败结果文本。
        /// </summary>
        private string ConstructFailedResultText(ChannelMapping channel, string generalFailureReason)
        {
            // TODO: 根据channel中具体失败的子项来构建更详细的失败原因。
            // 例如: "手动测试不通过 (低报测试:失败;趋势检查:失败)"
            var failedItems = new System.Text.StringBuilder();
            Action<string, string> appendIfFailed = (itemName, status) => {
                if (status == StatusFailed) {
                    if (failedItems.Length > 0) failedItems.Append(", ");
                    failedItems.Append($"{itemName}:{StatusFailed}");
                }
            };

            string moduleTypeUpper = channel.ModuleType?.ToUpper();
            appendIfFailed("显示值核对", channel.ShowValueStatusEnum.ToText());

            if (moduleTypeUpper == "AI")
            {
                appendIfFailed("低低报", channel.LowLowAlarmStatus);
                appendIfFailed("低报", channel.LowAlarmStatus);
                appendIfFailed("高报", channel.HighAlarmStatus);
                appendIfFailed("高高报", channel.HighHighAlarmStatus);
                appendIfFailed("报警值设定", channel.AlarmValueSetStatusEnum.ToText());
                appendIfFailed("维护功能", channel.MaintenanceFunctionEnum.ToText());
                appendIfFailed("趋势检查", channel.TrendCheckEnum.ToText());
                appendIfFailed("报表检查", channel.ReportCheckEnum.ToText());
            }
            else if (moduleTypeUpper == "AO")
            {
                appendIfFailed("维护功能", channel.MaintenanceFunctionEnum.ToText());
                appendIfFailed("趋势检查", channel.TrendCheckEnum.ToText());
                appendIfFailed("报表检查", channel.ReportCheckEnum.ToText());
            }
            // DI/DO 只有 ShowValueStatus，已在前面处理

            if (failedItems.Length > 0)
            {
                return $"{generalFailureReason} ({failedItems.ToString()})";
            }
            return generalFailureReason;
        }

        /// <summary>
        /// 尝试将字符串解析为可空的float?。
        /// </summary>
        private float? TryParseFloatNullable(string value)
        {
            if (string.IsNullOrWhiteSpace(value))
            {
                return null;
            }
            if (float.TryParse(value, NumberStyles.Any, CultureInfo.InvariantCulture, out float result))
            {
                return result;
            }
            return null; // 或记录一个解析错误，或返回一个特定的错误指示值
        }
    }
} 