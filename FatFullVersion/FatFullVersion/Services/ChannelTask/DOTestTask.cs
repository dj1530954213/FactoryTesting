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
    /// DO测试任务实现类
    /// </summary>
    public class DOTestTask : TestTask
    {
        /// <summary>
        /// 创建DO测试任务实例
        /// </summary>
        /// <param name="id">任务ID</param>
        /// <param name="channelMapping">通道映射信息</param>
        /// <param name="testPlcCommunication">测试PLC通信实例</param>
        /// <param name="targetPlcCommunication">被测PLC通信实例</param>
        public DOTestTask(
            string id,
            ChannelMapping channelMapping,
            IPlcCommunication testPlcCommunication,
            IPlcCommunication targetPlcCommunication)
            : base(id, channelMapping, testPlcCommunication, targetPlcCommunication)
        {
        }

        /// <summary>
        /// 执行DO硬点测试逻辑。
        /// 此方法通过被测PLC输出高、低两种信号，然后由测试PLC读取这些信号并验证。
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

                detailedTestLog.AppendLine($"开始DO硬点测试: {ChannelMapping.VariableName} ({ChannelMapping.ChannelTag}).");

                // --- 测试高信号 (期望测试PLC DI为ON) ---
                detailedTestLog.AppendLine("步骤 1/2: 测试高信号 (被测PLC DO输出ON).");
                var writeHighResult = await TargetPlcCommunication.WriteDigitalValueAsync(ChannelMapping.PlcCommunicationAddress.Substring(1), true);
                if (!writeHighResult.IsSuccess)
                {
                    detailedTestLog.AppendLine($"  控制被测PLC DO输出高信号失败: {writeHighResult.ErrorMessage}");
                    overallSuccess = false;
                }
                else
                {
                    detailedTestLog.AppendLine("  被测PLC DO已输出高信号 (true).");
                    await Task.Delay(3000, cancellationToken);

                    var readHighResult = await TestPlcCommunication.ReadDigitalValueAsync(ChannelMapping.TestPLCCommunicationAddress.Substring(1));
                    if (!readHighResult.IsSuccess)
                    {
                        detailedTestLog.AppendLine($"  读取测试PLC DI状态失败: {readHighResult.ErrorMessage}");
                        overallSuccess = false;
                    }
                    else
                    {
                        bool expectedHighState = true;
                        bool actualHighState = readHighResult.Data;
                        detailedTestLog.AppendLine($"  测试PLC DI读取到状态: {actualHighState} (期望: {expectedHighState}).");

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

                // --- 测试低信号 (期望测试PLC DI为OFF) ---
                detailedTestLog.AppendLine("步骤 2/2: 测试低信号 (被测PLC DO输出OFF).");
                var writeLowResult = await TargetPlcCommunication.WriteDigitalValueAsync(ChannelMapping.PlcCommunicationAddress.Substring(1), false);
                if (!writeLowResult.IsSuccess)
                {
                    detailedTestLog.AppendLine($"  控制被测PLC DO输出低信号失败: {writeLowResult.ErrorMessage}");
                    overallSuccess = false;
                }
                else
                {
                    detailedTestLog.AppendLine("  被测PLC DO已输出低信号 (false).");
                    await Task.Delay(3000, cancellationToken);

                    var readLowResult = await TestPlcCommunication.ReadDigitalValueAsync(ChannelMapping.TestPLCCommunicationAddress.Substring(1));
                    if (!readLowResult.IsSuccess)
                    {
                        detailedTestLog.AppendLine($"  读取测试PLC DI状态失败: {readLowResult.ErrorMessage}");
                        overallSuccess = false;
                    }
                    else
                    {
                        bool expectedLowState = false;
                        bool actualLowState = readLowResult.Data;
                        detailedTestLog.AppendLine($"  测试PLC DI读取到状态: {actualLowState} (期望: {expectedLowState}).");

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
            catch (OperationCanceledException)
            {
                detailedTestLog.AppendLine("DO测试任务被用户取消。");
                return new HardPointTestRawResult(false, detailedTestLog.ToString());
            }
            catch (Exception ex)
            {
                detailedTestLog.AppendLine($"执行DO硬点测试时发生意外错误：{ex.Message}");
                return new HardPointTestRawResult(false, detailedTestLog.ToString());
            }
            finally
            {
                try
                {
                    var resetResult = await TargetPlcCommunication.WriteDigitalValueAsync(ChannelMapping.PlcCommunicationAddress.Substring(1), false);
                    if (!resetResult.IsSuccess)
                    {
                        detailedTestLog.AppendLine($"警告：被测PLC DO输出复位失败：{resetResult.ErrorMessage}");
                    }
                    else
                    {
                        detailedTestLog.AppendLine("被测PLC DO输出已复位到低信号 (false)。");
                    }
                }
                catch (Exception ex_reset)
                {
                    detailedTestLog.AppendLine($"警告：被测PLC DO输出复位时发生异常：{ex_reset.Message}");
                }
            }

            if (overallSuccess)
            {
                detailedTestLog.Insert(0, "DO硬点测试所有步骤均通过。\n");
            }
            else
            {
                detailedTestLog.Insert(0, "DO硬点测试存在失败项。\n");
            }
            return new HardPointTestRawResult(overallSuccess, detailedTestLog.ToString());
        }
    }
}