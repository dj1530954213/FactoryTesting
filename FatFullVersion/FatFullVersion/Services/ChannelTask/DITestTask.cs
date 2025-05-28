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
    /// DI测试任务实现类
    /// </summary>
    public class DITestTask : TestTask
    {
        /// <summary>
        /// 创建DI测试任务实例
        /// </summary>
        /// <param name="id">任务ID</param>
        /// <param name="channelMapping">通道映射信息</param>
        /// <param name="testPlcCommunication">测试PLC通信实例</param>
        /// <param name="targetPlcCommunication">被测PLC通信实例</param>
        public DITestTask(
            string id,
            ChannelMapping channelMapping,
            IPlcCommunication testPlcCommunication,
            IPlcCommunication targetPlcCommunication)
            : base(id, channelMapping, testPlcCommunication, targetPlcCommunication)
        {
        }

        /// <summary>
        /// 执行DI硬点测试逻辑。
        /// 此方法通过测试PLC输出高、低两种信号，然后检查被测PLC的DI点是否正确响应。
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

                detailedTestLog.AppendLine($"开始DI硬点测试: {ChannelMapping.VariableName} ({ChannelMapping.ChannelTag}). 接线方式: {ChannelMapping.WireSystem ?? "未指定"}.");

                // --- 测试高信号 (期望被测PLC DI为ON) ---
                detailedTestLog.AppendLine("步骤 1/2: 测试高信号 (ON).");
                var writeHighResult = await TestPlcCommunication.WriteDigitalValueAsync(ChannelMapping.TestPLCCommunicationAddress.Substring(1), true);
                if (!writeHighResult.IsSuccess)
                {
                    detailedTestLog.AppendLine($"  写入高信号到测试PLC失败: {writeHighResult.ErrorMessage}");
                    overallSuccess = false;
                }
                else
                {
                    detailedTestLog.AppendLine("  测试PLC已输出高信号 (true).");
                    await Task.Delay(3000, cancellationToken); // 等待信号稳定

                    var readHighResult = await TargetPlcCommunication.ReadDigitalValueAsync(ChannelMapping.PlcCommunicationAddress.Substring(1));
                    if (!readHighResult.IsSuccess)
                    {
                        detailedTestLog.AppendLine($"  读取被测PLC DI状态失败: {readHighResult.ErrorMessage}");
                        overallSuccess = false;
                    }
                    else
                    {
                        bool expectedHighState = (ChannelMapping.WireSystem == "常闭") ? false : true;
                        bool actualHighState = readHighResult.Data;
                        detailedTestLog.AppendLine($"  被测PLC DI读取到状态: {actualHighState} (期望根据接线方式为: {expectedHighState}).");

                        if (actualHighState == expectedHighState)
                        {
                            detailedTestLog.AppendLine("  高信号点测试通过。");
                        }
                        else
                        {
                            detailedTestLog.AppendLine($"  高信号点测试失败! 期望: {expectedHighState}, 实际: {actualHighState}.");
                            overallSuccess = false;
                        }
                    }
                }
                cancellationToken.ThrowIfCancellationRequested();
                await Task.Delay(1000, cancellationToken);

                // --- 测试低信号 (期望被测PLC DI为OFF) ---
                if (overallSuccess) // 如果高信号测试已失败，可能无需继续或标记后续测试也受影响
                {
                    detailedTestLog.AppendLine("步骤 2/2: 测试低信号 (OFF).");
                    var writeLowResult = await TestPlcCommunication.WriteDigitalValueAsync(ChannelMapping.TestPLCCommunicationAddress.Substring(1), false);
                    if (!writeLowResult.IsSuccess)
                    {
                        detailedTestLog.AppendLine($"  写入低信号到测试PLC失败: {writeLowResult.ErrorMessage}");
                        overallSuccess = false;
                    }
                    else
                    {
                        detailedTestLog.AppendLine("  测试PLC已输出低信号 (false).");
                        await Task.Delay(3000, cancellationToken);

                        var readLowResult = await TargetPlcCommunication.ReadDigitalValueAsync(ChannelMapping.PlcCommunicationAddress.Substring(1));
                        if (!readLowResult.IsSuccess)
                        {
                            detailedTestLog.AppendLine($"  读取被测PLC DI状态失败: {readLowResult.ErrorMessage}");
                            overallSuccess = false;
                        }
                        else
                        {
                            bool expectedLowState = (ChannelMapping.WireSystem == "常闭") ? true : false;
                            bool actualLowState = readLowResult.Data;
                            detailedTestLog.AppendLine($"  被测PLC DI读取到状态: {actualLowState} (期望根据接线方式为: {expectedLowState}).");

                            if (actualLowState == expectedLowState)
                            {
                                detailedTestLog.AppendLine("  低信号点测试通过。");
                            }
                            else
                            {
                                detailedTestLog.AppendLine($"  低信号点测试失败! 期望: {expectedLowState}, 实际: {actualLowState}.");
                                overallSuccess = false;
                            }
                        }
                    }
                }
                else
                {
                    detailedTestLog.AppendLine("步骤 2/2: 因高信号测试失败，低信号测试跳过或标记为受影响。");
                    // overallSuccess 已经是 false
                }
            }
            catch (OperationCanceledException)
            {
                detailedTestLog.AppendLine("DI测试任务被用户取消。");
                return new HardPointTestRawResult(false, detailedTestLog.ToString());
            }
            catch (Exception ex)
            {
                detailedTestLog.AppendLine($"执行DI硬点测试时发生意外错误：{ex.Message}");
                return new HardPointTestRawResult(false, detailedTestLog.ToString());
            }
            finally
            {
                try
                {
                    var resetResult = await TestPlcCommunication.WriteDigitalValueAsync(ChannelMapping.TestPLCCommunicationAddress.Substring(1), false);
                    if (!resetResult.IsSuccess)
                    {
                        detailedTestLog.AppendLine($"警告：测试PLC输出复位失败：{resetResult.ErrorMessage}");
                    }
                    else
                    {
                        detailedTestLog.AppendLine("测试PLC输出已复位到低信号 (false)。");
                    }
                }
                catch (Exception ex_reset)
                {
                    detailedTestLog.AppendLine($"警告：测试PLC输出复位时发生异常：{ex_reset.Message}");
                }
            }

            if (overallSuccess)
            {
                detailedTestLog.Insert(0, "DI硬点测试所有步骤均通过。\n");
            }
            else
            {
                detailedTestLog.Insert(0, "DI硬点测试存在失败项。\n");
            }
            return new HardPointTestRawResult(overallSuccess, detailedTestLog.ToString());
        }
    }
}