using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Threading;
using System.Threading.Tasks;
using FatFullVersion.IServices;
using FatFullVersion.Models; 

namespace FatFullVersion.Services.ChannelTask 
{
    /// <summary>
    /// AI测试任务实现类
    /// </summary>
    public class AITestTask : TestTask 
    {
        /// <summary>
        /// 创建AI测试任务实例
        /// </summary>
        /// <param name="id">任务ID</param>
        /// <param name="channelMapping">通道映射信息</param>
        /// <param name="testPlcCommunication">测试PLC通信实例</param>
        /// <param name="targetPlcCommunication">被测PLC通信实例</param>
        public AITestTask(
            string id,
            ChannelMapping channelMapping,
            IPlcCommunication testPlcCommunication,
            IPlcCommunication targetPlcCommunication)
            : base(id, channelMapping, testPlcCommunication, targetPlcCommunication)
        {
        }

        /// <summary>
        /// 执行AI硬点测试逻辑。
        /// 此方法通过测试PLC输出5点（0%，25%，50%，75%，100%量程）模拟量信号，
        /// 然后检查被测PLC是否在允许的偏差范围内正确接收这些信号值。
        /// </summary>
        /// <param name="cancellationToken">取消令牌</param>
        /// <returns>一个包含测试是否成功以及详细信息的 HardPointTestRawResult 的任务。</returns>
        protected override async Task<HardPointTestRawResult> ExecuteTestAsync(CancellationToken cancellationToken) 
        {
            StringBuilder detailedTestLog = new StringBuilder();
            bool overallSuccess = true;

            try
            {
                // 确保连接已建立
                if (!TestPlcCommunication.IsConnected)
                {
                    var connectResultTest = await TestPlcCommunication.ConnectAsync();
                    if (!connectResultTest.IsSuccess)
                    {
                        return new HardPointTestRawResult(false, $"测试PLC连接失败: {connectResultTest.ErrorMessage}");
                    }
                }
                if (!TargetPlcCommunication.IsConnected)
                {
                    var connectResultTarget = await TargetPlcCommunication.ConnectAsync();
                    if (!connectResultTarget.IsSuccess)
                    {
                        return new HardPointTestRawResult(false, $"被测PLC连接失败: {connectResultTarget.ErrorMessage}");
                    }
                }

                if (!ChannelMapping.RangeLowerLimitValue.HasValue || !ChannelMapping.RangeUpperLimitValue.HasValue)
                {
                    detailedTestLog.AppendLine("量程上下限未在通道映射中正确设置。");
                    return new HardPointTestRawResult(false, detailedTestLog.ToString());
                }
                float minValue = ChannelMapping.RangeLowerLimitValue.Value;
                float maxValue = ChannelMapping.RangeUpperLimitValue.Value;
                if (maxValue <= minValue)
                {
                    detailedTestLog.AppendLine("量程设置无效（上限必须大于下限）。");
                    return new HardPointTestRawResult(false, detailedTestLog.ToString());
                }
                float range = maxValue - minValue;
                float[] percentages = { 0f, 25f, 50f, 75f, 100f };

                detailedTestLog.AppendLine($"开始AI硬点测试: {ChannelMapping.VariableName} ({ChannelMapping.ChannelTag}). 量程: {minValue} - {maxValue}.");

                for (int i = 0; i < percentages.Length; i++)
                {
                    if (ChannelMapping.VariableName.Contains("PT_2101"))
                    {
                        var percentage = percentages[i];
                        cancellationToken.ThrowIfCancellationRequested();
                        await CheckAndWaitForResumeAsync(cancellationToken); 

                        float percentValue = percentage;
                        float expectedValue = minValue + (range * percentage / 100f);
                        detailedTestLog.AppendLine($"步骤 {i + 1}/5: 测试 {percentage}% 点.");
                        var writeResult = await TestPlcCommunication.WriteAnalogValueAsync(ChannelMapping.TestPLCCommunicationAddress.Substring(1), percentValue);
                        if (!writeResult.IsSuccess)
                        {
                            detailedTestLog.AppendLine($"  写入测试值 ({percentValue}) 到测试PLC失败：{writeResult.ErrorMessage}");
                            overallSuccess = false;
                            break; 
                        }
                        detailedTestLog.AppendLine($"  测试PLC已写入百分比值: {percentValue}% -> 预计工程值 {expectedValue}.");
                    
                        await Task.Delay(3000, cancellationToken); 
                    
                        var readResult = await TargetPlcCommunication.ReadAnalogValueAsync(ChannelMapping.PlcCommunicationAddress.Substring(1));
                        if (!readResult.IsSuccess)
                        {
                            detailedTestLog.AppendLine($"  读取被测PLC值失败：{readResult.ErrorMessage}");
                            overallSuccess = false;
                            break; 
                        }
                    
                        float actualValue = readResult.Data;
                        detailedTestLog.AppendLine($"  被测PLC读取到值: {actualValue}.");

                        switch (percentage)
                        {
                            case 0f: ChannelMapping.Value0Percent = actualValue; break;
                            case 25f: ChannelMapping.Value25Percent = actualValue; break;
                            case 50f: ChannelMapping.Value50Percent = actualValue; break;
                            case 75f: ChannelMapping.Value75Percent = actualValue; break;
                            case 100f: ChannelMapping.Value100Percent = actualValue; break;
                        }
                    
                        float deviation = Math.Abs(actualValue - expectedValue);
                        float deviationPercent = 0f;
                        if (Math.Abs(range) > 1E-6) 
                        {
                            deviationPercent = (deviation / range) * 100f;
                        }
                        else if (Math.Abs(expectedValue) > 1E-6) 
                        {
                            deviationPercent = (deviation / Math.Abs(expectedValue)) * 100f;
                        }
                        else if (Math.Abs(actualValue) > 1E-6) 
                        {
                            deviationPercent = 100.0f; 
                        }
                                        
                        const float allowedRangeDeviationPercent = 1.0f; 

                        if (deviationPercent <= allowedRangeDeviationPercent)
                        {
                            detailedTestLog.AppendLine($"  {percentage}% 点测试通过。期望: {expectedValue}, 实际: {actualValue}, 偏差: {deviation:F3} ({deviationPercent:F2}% of range).");
                        }
                        else
                        {
                            detailedTestLog.AppendLine($"  {percentage}% 点测试失败! 期望: {expectedValue}, 实际: {actualValue}, 偏差: {deviation:F3} ({deviationPercent:F2}% of range). 允许偏差: {allowedRangeDeviationPercent}% of range.");
                            overallSuccess = false;
                        }
                        await Task.Delay(1000, cancellationToken); 
                    }
                    else
                    {
                        var percentage = percentages[i];
                        cancellationToken.ThrowIfCancellationRequested();
                        await CheckAndWaitForResumeAsync(cancellationToken);

                        float percentValue = percentage;
                        float expectedValue = minValue + (range * percentage / 100f);
                        detailedTestLog.AppendLine($"步骤 {i + 1}/5: 测试 {percentage}% 点.");
                        var writeResult = await TestPlcCommunication.WriteAnalogValueAsync(ChannelMapping.TestPLCCommunicationAddress.Substring(1), percentValue);
                        if (!writeResult.IsSuccess)
                        {
                            detailedTestLog.AppendLine($"  写入测试值 ({percentValue}) 到测试PLC失败：{writeResult.ErrorMessage}");
                            overallSuccess = false;
                            break;
                        }
                        detailedTestLog.AppendLine($"  测试PLC已写入百分比值: {percentValue}% -> 预计工程值 {expectedValue}.");

                        await Task.Delay(3000, cancellationToken);

                        var readResult = await TargetPlcCommunication.ReadAnalogValueAsync(ChannelMapping.PlcCommunicationAddress.Substring(1));
                        if (!readResult.IsSuccess)
                        {
                            detailedTestLog.AppendLine($"  读取被测PLC值失败：{readResult.ErrorMessage}");
                            overallSuccess = false;
                            break;
                        }

                        float actualValue = readResult.Data;
                        detailedTestLog.AppendLine($"  被测PLC读取到值: {actualValue}.");

                        switch (percentage)
                        {
                            case 0f: ChannelMapping.Value0Percent = actualValue; break;
                            case 25f: ChannelMapping.Value25Percent = actualValue; break;
                            case 50f: ChannelMapping.Value50Percent = actualValue; break;
                            case 75f: ChannelMapping.Value75Percent = actualValue; break;
                            case 100f: ChannelMapping.Value100Percent = actualValue; break;
                        }

                        float deviation = Math.Abs(actualValue - expectedValue);
                        float deviationPercent = 0f;
                        if (Math.Abs(range) > 1E-6)
                        {
                            deviationPercent = (deviation / range) * 100f;
                        }
                        else if (Math.Abs(expectedValue) > 1E-6)
                        {
                            deviationPercent = (deviation / Math.Abs(expectedValue)) * 100f;
                        }
                        else if (Math.Abs(actualValue) > 1E-6)
                        {
                            deviationPercent = 100.0f;
                        }

                        const float allowedRangeDeviationPercent = 1.0f;

                        if (deviationPercent <= allowedRangeDeviationPercent)
                        {
                            detailedTestLog.AppendLine($"  {percentage}% 点测试通过。期望: {expectedValue}, 实际: {actualValue}, 偏差: {deviation:F3} ({deviationPercent:F2}% of range).");
                        }
                        else
                        {
                            detailedTestLog.AppendLine($"  {percentage}% 点测试失败! 期望: {expectedValue}, 实际: {actualValue}, 偏差: {deviation:F3} ({deviationPercent:F2}% of range). 允许偏差: {allowedRangeDeviationPercent}% of range.");
                            overallSuccess = false;
                        }
                        await Task.Delay(1000, cancellationToken);
                    }
                }
            }
            catch (OperationCanceledException)
            {
                detailedTestLog.AppendLine("测试任务被用户取消。");
                return new HardPointTestRawResult(false, detailedTestLog.ToString());
            }
            catch (Exception ex)
            {
                detailedTestLog.AppendLine($"执行AI硬点测试时发生意外错误：{ex.Message}");
                return new HardPointTestRawResult(false, detailedTestLog.ToString());
            }
            finally
            {
                try
                {
                    float resetValue = ChannelMapping.RangeLowerLimitValue.HasValue ? ChannelMapping.RangeLowerLimitValue.Value : 0f;
                    var resetResultFinal = await TestPlcCommunication.WriteAnalogValueAsync(ChannelMapping.TestPLCCommunicationAddress.Substring(1), resetValue);
                    if (!resetResultFinal.IsSuccess)
                    {
                        detailedTestLog.AppendLine($"警告：测试PLC输出复位失败：{resetResultFinal.ErrorMessage}");
                    }
                    else
                    {
                        detailedTestLog.AppendLine("测试PLC输出已复位。");
                    }
                }
                catch(Exception ex_reset)
                {
                    detailedTestLog.AppendLine($"警告：测试PLC输出复位时发生异常：{ex_reset.Message}");
                }
            }

            if (overallSuccess)
            {
                detailedTestLog.Insert(0, "AI硬点测试所有点均在允许偏差范围内。\n");
            }
            else
            {
                detailedTestLog.Insert(0, "AI硬点测试存在失败项。\n");
            }
            return new HardPointTestRawResult(overallSuccess, detailedTestLog.ToString());
        }
    }
} 