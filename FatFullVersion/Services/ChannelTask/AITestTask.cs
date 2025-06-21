using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Threading;
using System.Threading.Tasks;
using FatFullVersion.IServices;

namespace FatFullVersion.Models
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

        #region 原始量程不需要转换的逻辑留存
        ///// <summary>
        ///// 执行AI测试逻辑
        ///// </summary>
        ///// <param name="cancellationToken">取消令牌</param>
        ///// <returns>异步任务</returns>
        //protected override async Task ExecuteTestAsync(CancellationToken cancellationToken)
        //{
        //    // 确保连接已建立
        //    if (!TestPlcCommunication.IsConnected)
        //    {
        //        await TestPlcCommunication.ConnectAsync();
        //    }

        //    if (!TargetPlcCommunication.IsConnected)
        //    {
        //        await TargetPlcCommunication.ConnectAsync();
        //    }

        //    try
        //    {
        //        // AI测试流程：由测试PLC输出模拟量信号，然后检查被测PLC是否正确接收
        //        // 测试多个不同的信号值（0%、25%、50%、75%、100%等）

        //        // 定义测试信号值（根据工程单位和量程计算）
        //        float minValue = ChannelMapping.RangeLowerLimitValue;
        //        float maxValue = ChannelMapping.RangeUpperLimitValue;
        //        float range = maxValue - minValue;

        //        // 依次测试不同百分比的信号值
        //        float[] percentages = { 0, 25, 50, 75, 100 };

        //        bool allTestsPassed = true;
        //        // 测试前清除原来的测试记录
        //        Result.Status = "";
        //        // 保存详细的测试过程记录
        //        StringBuilder detailedTestLog = new StringBuilder();

        //        for (int i = 0; i < percentages.Length; i++)
        //        {
        //            var percentage = percentages[i];

        //            // 取消检查
        //            cancellationToken.ThrowIfCancellationRequested();

        //            // 暂停检查
        //            await CheckAndWaitForResumeAsync(cancellationToken);

        //            // 计算当前测试值
        //            float testValue = minValue + (range * percentage / 100);

        //            // 写入测试值到测试PLC
        //            var writeResult = await TestPlcCommunication.WriteAnalogValueAsync(ChannelMapping.TestPLCCommunicationAddress.Substring(1), testValue);
        //            if (!writeResult.IsSuccess)
        //            {
        //                detailedTestLog.AppendLine($"写入测试值失败：{writeResult.ErrorMessage}");
        //                allTestsPassed = false;
        //                break;
        //            }

        //            // 等待信号稳定(大约3秒)
        //            await Task.Delay(3000, cancellationToken);

        //            // 读取被测PLC的值
        //            var readResult = await TargetPlcCommunication.ReadAnalogValueAsync(ChannelMapping.PlcCommunicationAddress.Substring(1));
        //            if (!readResult.IsSuccess)
        //            {
        //                detailedTestLog.AppendLine($"读取被测PLC值失败：{readResult.ErrorMessage}");
        //                allTestsPassed = false;
        //                break;
        //            }

        //            float actualValue = readResult.Data;

        //            // 存储各个百分比点位的值
        //            switch (percentage)
        //            {
        //                case 0:
        //                    Result.Value0Percent = actualValue;
        //                    Console.WriteLine($"存储0%值: {actualValue}");
        //                    break;
        //                case 25:
        //                    Result.Value25Percent = actualValue;
        //                    Console.WriteLine($"存储25%值: {actualValue}");
        //                    break;
        //                case 50:
        //                    Result.Value50Percent = actualValue;
        //                    Console.WriteLine($"存储50%值: {actualValue}");
        //                    break;
        //                case 75:
        //                    Result.Value75Percent = actualValue;
        //                    Console.WriteLine($"存储75%值: {actualValue}");
        //                    break;
        //                case 100:
        //                    Result.Value100Percent = actualValue;
        //                    Console.WriteLine($"存储100%值: {actualValue}");
        //                    break;
        //            }

        //            // 更新测试结果
        //            Result.ExpectedValue = testValue;
        //            Result.ActualValue = actualValue;

        //            // 计算偏差是否在容许范围内
        //            float deviation = Math.Abs(actualValue - testValue);
        //            float deviationPercent = (testValue != 0) ? (deviation / Math.Abs(testValue)) * 100 : 0;

        //            // 根据偏差判断是否通过测试
        //            // 假设允许偏差为1%
        //            const float allowedDeviation = 1.0f;

        //            if (deviationPercent <= allowedDeviation)
        //            {
        //                detailedTestLog.AppendLine($"{percentage}%测试通过");
        //            }
        //            else
        //            {
        //                detailedTestLog.AppendLine($"{percentage}%测试失败：偏差{deviationPercent:F2}%超出范围");
        //                allTestsPassed = false;
        //                // 不中断测试流程，继续测试其它点位
        //            }

        //            // 短暂延时再进行下一个测试点
        //            await Task.Delay(1000, cancellationToken);
        //        }

        //        // 确保百分比测试值能够持久化到通道映射中
        //        ChannelMapping.Value0Percent = Result.Value0Percent;
        //        ChannelMapping.Value25Percent = Result.Value25Percent;
        //        ChannelMapping.Value50Percent = Result.Value50Percent;
        //        ChannelMapping.Value75Percent = Result.Value75Percent;
        //        ChannelMapping.Value100Percent = Result.Value100Percent;

        //        // 保存详细日志到错误信息字段，便于查看
        //        Result.ErrorMessage = detailedTestLog.ToString();

        //        // 设置最终测试状态 - 只显示通过或失败
        //        if (allTestsPassed)
        //        {
        //            Result.Status = "通过";
        //            ChannelMapping.HardPointTestResult = "通过";
        //        }
        //        else
        //        {
        //            Result.Status = "失败";
        //            ChannelMapping.HardPointTestResult = "失败";
        //            ChannelMapping.TestResultStatus = 2;
        //        }
        //    }
        //    catch (OperationCanceledException)
        //    {
        //        // 任务被取消，不做特殊处理，直接向上抛出
        //        throw;
        //    }
        //    catch (Exception ex)
        //    {
        //        // 其他异常，记录错误消息
        //        Result.Status = "失败";
        //        Result.ErrorMessage = ex.Message;
        //        ChannelMapping.HardPointTestResult = "失败";
        //        throw;
        //    }
        //    finally
        //    {
        //        // 结束测试时，将测试PLC输出复位到0%
        //        try
        //        {
        //            var resetResult = await TestPlcCommunication.WriteAnalogValueAsync(ChannelMapping.TestPLCCommunicationAddress.Substring(1), ChannelMapping.RangeLowerLimitValue);
        //            if (!resetResult.IsSuccess)
        //            {
        //                // 记录复位失败但不影响测试结果
        //                if (string.IsNullOrEmpty(Result.ErrorMessage))
        //                    Result.ErrorMessage = $"复位失败：{resetResult.ErrorMessage}";
        //                else
        //                    Result.ErrorMessage += $"\n复位失败：{resetResult.ErrorMessage}";
        //            }
        //        }
        //        catch
        //        {
        //            // 忽略复位过程中的异常
        //        }
        //    }
        //}

        ///// <summary>
        ///// 写入0%测试值
        ///// </summary>
        ///// <param name="cancellationToken">取消令牌</param>
        ///// <returns>异步任务</returns>
        //public override async Task Write0PercentTestValueAsync(CancellationToken cancellationToken)
        //{
        //    // 取消检查
        //    cancellationToken.ThrowIfCancellationRequested();

        //    // 暂停检查
        //    await CheckAndWaitForResumeAsync(cancellationToken);

        //    try
        //    {
        //        // 确保连接已建立
        //        if (!TestPlcCommunication.IsConnected)
        //        {
        //            await TestPlcCommunication.ConnectAsync();
        //        }

        //        // 计算0%测试值
        //        float minValue = ChannelMapping.RangeLowerLimitValue;
        //        float testValue = minValue;

        //        // 写入测试值到测试PLC
        //        var writeResult = await TestPlcCommunication.WriteAnalogValueAsync(
        //            ChannelMapping.TestPLCCommunicationAddress.Substring(1),
        //            testValue);

        //        if (!writeResult.IsSuccess)
        //        {
        //            Console.WriteLine($"AI写入0%测试值失败：{writeResult.ErrorMessage}");
        //        }
        //    }
        //    catch (Exception ex)
        //    {
        //        Console.WriteLine($"AI写入0%测试值时出错: {ex.Message}");
        //        throw;
        //    }
        //}

        ///// <summary>
        ///// 读取0%测试值
        ///// </summary>
        ///// <param name="cancellationToken">取消令牌</param>
        ///// <returns>异步任务</returns>
        //public override async Task Read0PercentTestValueAsync(CancellationToken cancellationToken)
        //{
        //    // 取消检查
        //    cancellationToken.ThrowIfCancellationRequested();

        //    // 暂停检查
        //    await CheckAndWaitForResumeAsync(cancellationToken);

        //    try
        //    {
        //        // 确保连接已建立
        //        if (!TargetPlcCommunication.IsConnected)
        //        {
        //            await TargetPlcCommunication.ConnectAsync();
        //        }

        //        // 读取被测PLC的值
        //        var readResult = await TargetPlcCommunication.ReadAnalogValueAsync(
        //            ChannelMapping.PlcCommunicationAddress.Substring(1));

        //        if (!readResult.IsSuccess)
        //        {
        //            Console.WriteLine($"AI读取0%测试值失败：{readResult.ErrorMessage}");
        //        }
        //        else
        //        {
        //            float actualValue = readResult.Data;
        //            Result.Value0Percent = actualValue;
        //            Console.WriteLine($"AI存储0%值: {actualValue}");

        //            // 更新测试结果
        //            Result.ExpectedValue = ChannelMapping.RangeLowerLimitValue;
        //            Result.ActualValue = actualValue;
        //        }
        //    }
        //    catch (Exception ex)
        //    {
        //        Console.WriteLine($"AI读取0%测试值时出错: {ex.Message}");
        //        throw;
        //    }
        //}

        ///// <summary>
        ///// 写入25%测试值
        ///// </summary>
        ///// <param name="cancellationToken">取消令牌</param>
        ///// <returns>异步任务</returns>
        //public override async Task Write25PercentTestValueAsync(CancellationToken cancellationToken)
        //{
        //    // 取消检查
        //    cancellationToken.ThrowIfCancellationRequested();

        //    // 暂停检查
        //    await CheckAndWaitForResumeAsync(cancellationToken);

        //    try
        //    {
        //        // 确保连接已建立
        //        if (!TestPlcCommunication.IsConnected)
        //        {
        //            await TestPlcCommunication.ConnectAsync();
        //        }

        //        // 计算25%测试值
        //        float minValue = ChannelMapping.RangeLowerLimitValue;
        //        float maxValue = ChannelMapping.RangeUpperLimitValue;
        //        float range = maxValue - minValue;
        //        float testValue = minValue + (range * 25 / 100);

        //        // 写入测试值到测试PLC
        //        var writeResult = await TestPlcCommunication.WriteAnalogValueAsync(
        //            ChannelMapping.TestPLCCommunicationAddress.Substring(1),
        //            testValue);

        //        if (!writeResult.IsSuccess)
        //        {
        //            Console.WriteLine($"AI写入25%测试值失败：{writeResult.ErrorMessage}");
        //        }
        //    }
        //    catch (Exception ex)
        //    {
        //        Console.WriteLine($"AI写入25%测试值时出错: {ex.Message}");
        //        throw;
        //    }
        //}

        ///// <summary>
        ///// 读取25%测试值
        ///// </summary>
        ///// <param name="cancellationToken">取消令牌</param>
        ///// <returns>异步任务</returns>
        //public override async Task Read25PercentTestValueAsync(CancellationToken cancellationToken)
        //{
        //    // 取消检查
        //    cancellationToken.ThrowIfCancellationRequested();

        //    // 暂停检查
        //    await CheckAndWaitForResumeAsync(cancellationToken);

        //    try
        //    {
        //        // 确保连接已建立
        //        if (!TargetPlcCommunication.IsConnected)
        //        {
        //            await TargetPlcCommunication.ConnectAsync();
        //        }

        //        // 计算预期的25%测试值
        //        float minValue = ChannelMapping.RangeLowerLimitValue;
        //        float maxValue = ChannelMapping.RangeUpperLimitValue;
        //        float range = maxValue - minValue;
        //        float expectedValue = minValue + (range * 25 / 100);

        //        // 读取被测PLC的值
        //        var readResult = await TargetPlcCommunication.ReadAnalogValueAsync(
        //            ChannelMapping.PlcCommunicationAddress.Substring(1));

        //        if (!readResult.IsSuccess)
        //        {
        //            Console.WriteLine($"AI读取25%测试值失败：{readResult.ErrorMessage}");
        //        }
        //        else
        //        {
        //            float actualValue = readResult.Data;
        //            Result.Value25Percent = actualValue;
        //            Console.WriteLine($"AI存储25%值: {actualValue}");

        //            // 更新测试结果
        //            Result.ExpectedValue = expectedValue;
        //            Result.ActualValue = actualValue;
        //        }
        //    }
        //    catch (Exception ex)
        //    {
        //        Console.WriteLine($"AI读取25%测试值时出错: {ex.Message}");
        //        throw;
        //    }
        //}

        ///// <summary>
        ///// 写入50%测试值
        ///// </summary>
        ///// <param name="cancellationToken">取消令牌</param>
        ///// <returns>异步任务</returns>
        //public override async Task Write50PercentTestValueAsync(CancellationToken cancellationToken)
        //{
        //    // 取消检查
        //    cancellationToken.ThrowIfCancellationRequested();

        //    // 暂停检查
        //    await CheckAndWaitForResumeAsync(cancellationToken);

        //    try
        //    {
        //        // 确保连接已建立
        //        if (!TestPlcCommunication.IsConnected)
        //        {
        //            await TestPlcCommunication.ConnectAsync();
        //        }

        //        // 计算50%测试值
        //        float minValue = ChannelMapping.RangeLowerLimitValue;
        //        float maxValue = ChannelMapping.RangeUpperLimitValue;
        //        float range = maxValue - minValue;
        //        float testValue = minValue + (range * 50 / 100);

        //        // 写入测试值到测试PLC
        //        var writeResult = await TestPlcCommunication.WriteAnalogValueAsync(
        //            ChannelMapping.TestPLCCommunicationAddress.Substring(1),
        //            testValue);

        //        if (!writeResult.IsSuccess)
        //        {
        //            Console.WriteLine($"AI写入50%测试值失败：{writeResult.ErrorMessage}");
        //        }
        //    }
        //    catch (Exception ex)
        //    {
        //        Console.WriteLine($"AI写入50%测试值时出错: {ex.Message}");
        //        throw;
        //    }
        //}

        ///// <summary>
        ///// 读取50%测试值
        ///// </summary>
        ///// <param name="cancellationToken">取消令牌</param>
        ///// <returns>异步任务</returns>
        //public override async Task Read50PercentTestValueAsync(CancellationToken cancellationToken)
        //{
        //    // 取消检查
        //    cancellationToken.ThrowIfCancellationRequested();

        //    // 暂停检查
        //    await CheckAndWaitForResumeAsync(cancellationToken);

        //    try
        //    {
        //        // 确保连接已建立
        //        if (!TargetPlcCommunication.IsConnected)
        //        {
        //            await TargetPlcCommunication.ConnectAsync();
        //        }

        //        // 计算预期的50%测试值
        //        float minValue = ChannelMapping.RangeLowerLimitValue;
        //        float maxValue = ChannelMapping.RangeUpperLimitValue;
        //        float range = maxValue - minValue;
        //        float expectedValue = minValue + (range * 50 / 100);

        //        // 读取被测PLC的值
        //        var readResult = await TargetPlcCommunication.ReadAnalogValueAsync(
        //            ChannelMapping.PlcCommunicationAddress.Substring(1));

        //        if (!readResult.IsSuccess)
        //        {
        //            Console.WriteLine($"AI读取50%测试值失败：{readResult.ErrorMessage}");
        //        }
        //        else
        //        {
        //            float actualValue = readResult.Data;
        //            Result.Value50Percent = actualValue;
        //            Console.WriteLine($"AI存储50%值: {actualValue}");

        //            // 更新测试结果
        //            Result.ExpectedValue = expectedValue;
        //            Result.ActualValue = actualValue;
        //        }
        //    }
        //    catch (Exception ex)
        //    {
        //        Console.WriteLine($"AI读取50%测试值时出错: {ex.Message}");
        //        throw;
        //    }
        //}

        ///// <summary>
        ///// 写入75%测试值
        ///// </summary>
        ///// <param name="cancellationToken">取消令牌</param>
        ///// <returns>异步任务</returns>
        //public override async Task Write75PercentTestValueAsync(CancellationToken cancellationToken)
        //{
        //    // 取消检查
        //    cancellationToken.ThrowIfCancellationRequested();

        //    // 暂停检查
        //    await CheckAndWaitForResumeAsync(cancellationToken);

        //    try
        //    {
        //        // 确保连接已建立
        //        if (!TestPlcCommunication.IsConnected)
        //        {
        //            await TestPlcCommunication.ConnectAsync();
        //        }

        //        // 计算75%测试值
        //        float minValue = ChannelMapping.RangeLowerLimitValue;
        //        float maxValue = ChannelMapping.RangeUpperLimitValue;
        //        float range = maxValue - minValue;
        //        float testValue = minValue + (range * 75 / 100);

        //        // 写入测试值到测试PLC
        //        var writeResult = await TestPlcCommunication.WriteAnalogValueAsync(
        //            ChannelMapping.TestPLCCommunicationAddress.Substring(1),
        //            testValue);

        //        if (!writeResult.IsSuccess)
        //        {
        //            Console.WriteLine($"AI写入75%测试值失败：{writeResult.ErrorMessage}");
        //        }
        //    }
        //    catch (Exception ex)
        //    {
        //        Console.WriteLine($"AI写入75%测试值时出错: {ex.Message}");
        //        throw;
        //    }
        //}

        ///// <summary>
        ///// 读取75%测试值
        ///// </summary>
        ///// <param name="cancellationToken">取消令牌</param>
        ///// <returns>异步任务</returns>
        //public override async Task Read75PercentTestValueAsync(CancellationToken cancellationToken)
        //{
        //    // 取消检查
        //    cancellationToken.ThrowIfCancellationRequested();

        //    // 暂停检查
        //    await CheckAndWaitForResumeAsync(cancellationToken);

        //    try
        //    {
        //        // 确保连接已建立
        //        if (!TargetPlcCommunication.IsConnected)
        //        {
        //            await TargetPlcCommunication.ConnectAsync();
        //        }

        //        // 计算预期的75%测试值
        //        float minValue = ChannelMapping.RangeLowerLimitValue;
        //        float maxValue = ChannelMapping.RangeUpperLimitValue;
        //        float range = maxValue - minValue;
        //        float expectedValue = minValue + (range * 75 / 100);

        //        // 读取被测PLC的值
        //        var readResult = await TargetPlcCommunication.ReadAnalogValueAsync(
        //            ChannelMapping.PlcCommunicationAddress.Substring(1));

        //        if (!readResult.IsSuccess)
        //        {
        //            Console.WriteLine($"AI读取75%测试值失败：{readResult.ErrorMessage}");
        //        }
        //        else
        //        {
        //            float actualValue = readResult.Data;
        //            Result.Value75Percent = actualValue;
        //            Console.WriteLine($"AI存储75%值: {actualValue}");

        //            // 更新测试结果
        //            Result.ExpectedValue = expectedValue;
        //            Result.ActualValue = actualValue;
        //        }
        //    }
        //    catch (Exception ex)
        //    {
        //        Console.WriteLine($"AI读取75%测试值时出错: {ex.Message}");
        //        throw;
        //    }
        //}

        ///// <summary>
        ///// 写入100%测试值
        ///// </summary>
        ///// <param name="cancellationToken">取消令牌</param>
        ///// <returns>异步任务</returns>
        //public override async Task Write100PercentTestValueAsync(CancellationToken cancellationToken)
        //{
        //    // 取消检查
        //    cancellationToken.ThrowIfCancellationRequested();

        //    // 暂停检查
        //    await CheckAndWaitForResumeAsync(cancellationToken);

        //    try
        //    {
        //        // 确保连接已建立
        //        if (!TestPlcCommunication.IsConnected)
        //        {
        //            await TestPlcCommunication.ConnectAsync();
        //        }

        //        // 计算100%测试值
        //        float maxValue = ChannelMapping.RangeUpperLimitValue;

        //        // 写入测试值到测试PLC
        //        var writeResult = await TestPlcCommunication.WriteAnalogValueAsync(
        //            ChannelMapping.TestPLCCommunicationAddress.Substring(1),
        //            maxValue);

        //        if (!writeResult.IsSuccess)
        //        {
        //            Console.WriteLine($"AI写入100%测试值失败：{writeResult.ErrorMessage}");
        //        }
        //    }
        //    catch (Exception ex)
        //    {
        //        Console.WriteLine($"AI写入100%测试值时出错: {ex.Message}");
        //        throw;
        //    }
        //}

        ///// <summary>
        ///// 读取100%测试值
        ///// </summary>
        ///// <param name="cancellationToken">取消令牌</param>
        ///// <returns>异步任务</returns>
        //public override async Task Read100PercentTestValueAsync(CancellationToken cancellationToken)
        //{
        //    // 取消检查
        //    cancellationToken.ThrowIfCancellationRequested();

        //    // 暂停检查
        //    await CheckAndWaitForResumeAsync(cancellationToken);

        //    try
        //    {
        //        // 确保连接已建立
        //        if (!TargetPlcCommunication.IsConnected)
        //        {
        //            await TargetPlcCommunication.ConnectAsync();
        //        }

        //        // 使用100%测试值
        //        float maxValue = ChannelMapping.RangeUpperLimitValue;

        //        // 读取被测PLC的值
        //        var readResult = await TargetPlcCommunication.ReadAnalogValueAsync(
        //            ChannelMapping.PlcCommunicationAddress.Substring(1));

        //        if (!readResult.IsSuccess)
        //        {
        //            Console.WriteLine($"AI读取100%测试值失败：{readResult.ErrorMessage}");
        //        }
        //        else
        //        {
        //            float actualValue = readResult.Data;
        //            Result.Value100Percent = actualValue;
        //            Console.WriteLine($"AI存储100%值: {actualValue}");

        //            // 更新测试结果
        //            Result.ExpectedValue = maxValue;
        //            Result.ActualValue = actualValue;

        //            // 判断测试是否通过
        //            // 计算所有测试点的偏差是否在容许范围内
        //            bool allTestsPassed = EvaluateTestResults();

        //            // 设置最终测试状态
        //            if (allTestsPassed)
        //            {
        //                Result.Status = "通过";
        //                ChannelMapping.HardPointTestResult = "通过";
        //            }
        //            else
        //            {
        //                Result.Status = "失败";
        //                ChannelMapping.HardPointTestResult = "失败";
        //                ChannelMapping.TestResultStatus = 2;
        //            }
        //        }
        //    }
        //    catch (Exception ex)
        //    {
        //        Console.WriteLine($"AI读取100%测试值时出错: {ex.Message}");
        //        throw;
        //    }
        //}

        ///// <summary>
        ///// 写入复位值
        ///// </summary>
        ///// <param name="cancellationToken">取消令牌</param>
        ///// <returns>异步任务</returns>
        //public override async Task WriteResetValueAsync(CancellationToken cancellationToken)
        //{
        //    try
        //    {
        //        // 确保连接已建立
        //        if (!TestPlcCommunication.IsConnected)
        //        {
        //            await TestPlcCommunication.ConnectAsync();
        //        }

        //        // 复位为0%值
        //        var resetResult = await TestPlcCommunication.WriteAnalogValueAsync(
        //            ChannelMapping.TestPLCCommunicationAddress.Substring(1),
        //            ChannelMapping.RangeLowerLimitValue);

        //        if (!resetResult.IsSuccess)
        //        {
        //            Console.WriteLine($"AI复位值失败：{resetResult.ErrorMessage}");
        //            if (string.IsNullOrEmpty(Result.ErrorMessage))
        //                Result.ErrorMessage = $"复位失败：{resetResult.ErrorMessage}";
        //            else
        //                Result.ErrorMessage += $"\n复位失败：{resetResult.ErrorMessage}";
        //        }
        //    }
        //    catch (Exception ex)
        //    {
        //        Console.WriteLine($"AI复位值时出错: {ex.Message}");
        //        // 记录错误但不抛出异常，避免中断测试流程
        //        if (string.IsNullOrEmpty(Result.ErrorMessage))
        //            Result.ErrorMessage = $"复位失败：{ex.Message}";
        //        else
        //            Result.ErrorMessage += $"\n复位失败：{ex.Message}";
        //    }
        //}

        ///// <summary>
        ///// 评估所有测试结果
        ///// </summary>
        ///// <returns>是否所有测试都通过</returns>
        //private bool EvaluateTestResults()
        //{
        //    try
        //    {
        //        // 收集所有需要评估的测试点
        //        Dictionary<string, (float Expected, float Actual)> testPoints = new Dictionary<string, (float Expected, float Actual)>();

        //        // 计算各个测试百分比的预期值
        //        float minValue = ChannelMapping.RangeLowerLimitValue;
        //        float maxValue = ChannelMapping.RangeUpperLimitValue;
        //        float range = maxValue - minValue;

        //        // 添加所有测试点
        //        //testPoints.Add("0%", (Expected: minValue, Actual: (float)Result.Value0Percent));
        //        //testPoints.Add("25%", (Expected: minValue + (range * 25 / 100), Actual: (float)Result.Value25Percent));
        //        //testPoints.Add("50%", (Expected: minValue + (range * 50 / 100), Actual: (float)Result.Value50Percent));
        //        //testPoints.Add("75%", (Expected: minValue + (range * 75 / 100), Actual: (float)Result.Value75Percent));
        //        //testPoints.Add("100%", (Expected: maxValue, Actual: (float)Result.Value100Percent));
        //        //添加所有测试点，
        //        testPoints.Add("0%", (Expected: minValue, Actual: (float)Result.Value0Percent));
        //        testPoints.Add("25%", (Expected: minValue + (range * 25 / 100), Actual: (float)Result.Value25Percent));
        //        testPoints.Add("50%", (Expected: minValue + (range * 50 / 100), Actual: (float)Result.Value50Percent));
        //        testPoints.Add("75%", (Expected: minValue + (range * 75 / 100), Actual: (float)Result.Value75Percent));
        //        testPoints.Add("100%", (Expected: maxValue, Actual: (float)Result.Value100Percent));

        //        // 创建详细测试报告
        //        StringBuilder testReport = new StringBuilder();
        //        bool allPassed = true;

        //        // 允许的最大偏差百分比
        //        const float allowedDeviation = 1.0f;

        //        // 评估每个测试点
        //        foreach (var point in testPoints)
        //        {
        //            float expected = point.Value.Expected;
        //            float actual = point.Value.Actual;

        //            // 计算偏差
        //            float deviation = Math.Abs(actual - expected);
        //            float deviationPercent = (expected != 0) ? (deviation / Math.Abs(expected)) * 100 : 0;

        //            // 判断是否通过
        //            bool pointPassed = deviationPercent <= allowedDeviation;
        //            if (!pointPassed)
        //                allPassed = false;

        //            // 添加到报告
        //            testReport.AppendLine($"{point.Key}测试" + (pointPassed ? "通过" : $"失败：偏差{deviationPercent:F2}%超出范围"));
        //        }

        //        // 保存详细报告
        //        Result.ErrorMessage = testReport.ToString();

        //        return allPassed;
        //    }
        //    catch (Exception ex)
        //    {
        //        Console.WriteLine($"评估测试结果时出错: {ex.Message}");
        //        Result.ErrorMessage = $"评估测试结果时出错: {ex.Message}";
        //        return false;
        //    }
        //}


        #endregion

        /// <summary>
        /// 执行AI测试逻辑
        /// </summary>
        /// <param name="cancellationToken">取消令牌</param>
        /// <returns>异步任务</returns>
        protected override async Task ExecuteTestAsync(CancellationToken cancellationToken)
        {
            // 确保连接已建立
            if (!TestPlcCommunication.IsConnected)
            {
                await TestPlcCommunication.ConnectAsync();
            }

            if (!TargetPlcCommunication.IsConnected)
            {
                await TargetPlcCommunication.ConnectAsync();
            }

            try
            {
                // AI测试流程：由测试PLC输出模拟量信号，然后检查被测PLC是否正确接收
                // 测试多个不同的信号值（0%、25%、50%、75%、100%等）

                // 定义测试信号值（根据工程单位和量程计算）
                float minValue = ChannelMapping.RangeLowerLimitValue;
                float maxValue = ChannelMapping.RangeUpperLimitValue;
                float range = maxValue - minValue;

                // 依次测试不同百分比的信号值
                float[] percentages = { 0, 25, 50, 75, 100 };
                
                bool allTestsPassed = true;
                // 测试前清除原来的测试记录
                Result.Status = "";
                // 保存详细的测试过程记录
                StringBuilder detailedTestLog = new StringBuilder();
                
                for (int i = 0; i < percentages.Length; i++)
                {
                    var percentage = percentages[i];
                    
                    // 取消检查
                    cancellationToken.ThrowIfCancellationRequested();
                    
                    // 暂停检查
                    await CheckAndWaitForResumeAsync(cancellationToken);
                    
                    // 计算当前测试值
                    float testValue = minValue + (range * percentage / 100);
                    
                    // 写入测试值到测试PLC
                    var writeResult = await TestPlcCommunication.WriteAnalogValueAsync(ChannelMapping.TestPLCCommunicationAddress.Substring(1), testValue);
                    if (!writeResult.IsSuccess)
                    {
                        detailedTestLog.AppendLine($"写入测试值失败：{writeResult.ErrorMessage}");
                        allTestsPassed = false;
                        break;
                    }
                    
                    // 等待信号稳定(大约3秒)
                    await Task.Delay(3000, cancellationToken);
                    
                    // 读取被测PLC的值
                    var readResult = await TargetPlcCommunication.ReadAnalogValueAsync(ChannelMapping.PlcCommunicationAddress.Substring(1));
                    if (!readResult.IsSuccess)
                    {
                        detailedTestLog.AppendLine($"读取被测PLC值失败：{readResult.ErrorMessage}");
                        allTestsPassed = false;
                        break;
                    }
                    
                    float actualValue = readResult.Data;
                    
                    // 存储各个百分比点位的值
                    switch (percentage)
                    {
                        case 0:
                            Result.Value0Percent = actualValue;
                            Console.WriteLine($"存储0%值: {actualValue}");
                            break;
                        case 25:
                            Result.Value25Percent = actualValue;
                            Console.WriteLine($"存储25%值: {actualValue}");
                            break;
                        case 50:
                            Result.Value50Percent = actualValue;
                            Console.WriteLine($"存储50%值: {actualValue}");
                            break;
                        case 75:
                            Result.Value75Percent = actualValue;
                            Console.WriteLine($"存储75%值: {actualValue}");
                            break;
                        case 100:
                            Result.Value100Percent = actualValue;
                            Console.WriteLine($"存储100%值: {actualValue}");
                            break;
                    }
                    
                    // 更新测试结果
                    Result.ExpectedValue = testValue;
                    Result.ActualValue = actualValue;
                    
                    // 计算偏差是否在容许范围内
                    float deviation = Math.Abs(actualValue - testValue);
                    float deviationPercent = (testValue != 0) ? (deviation / Math.Abs(testValue)) * 100 : 0;
                    
                    // 根据偏差判断是否通过测试
                    // 假设允许偏差为1%
                    const float allowedDeviation = 1.0f;
                    
                    if (deviationPercent <= allowedDeviation)
                    {
                        detailedTestLog.AppendLine($"{percentage}%测试通过");
                    }
                    else
                    {
                        detailedTestLog.AppendLine($"{percentage}%测试失败：偏差{deviationPercent:F2}%超出范围");
                        allTestsPassed = false;
                        // 不中断测试流程，继续测试其它点位
                    }
                    
                    // 短暂延时再进行下一个测试点
                    await Task.Delay(1000, cancellationToken);
                }
                
                // 确保百分比测试值能够持久化到通道映射中
                ChannelMapping.Value0Percent = Result.Value0Percent;
                ChannelMapping.Value25Percent = Result.Value25Percent;
                ChannelMapping.Value50Percent = Result.Value50Percent;
                ChannelMapping.Value75Percent = Result.Value75Percent;
                ChannelMapping.Value100Percent = Result.Value100Percent;
                
                // 保存详细日志到错误信息字段，便于查看
                Result.ErrorMessage = detailedTestLog.ToString();
                
                // 设置最终测试状态 - 只显示通过或失败
                if (allTestsPassed)
                {
                    Result.Status = "通过";
                    ChannelMapping.HardPointTestResult = "通过";
                }
                else
                {
                    Result.Status = "失败";
                    ChannelMapping.HardPointTestResult = "失败";
                    ChannelMapping.TestResultStatus = 2;
                }
            }
            catch (OperationCanceledException)
            {
                // 任务被取消，不做特殊处理，直接向上抛出
                throw;
            }
            catch (Exception ex)
            {
                // 其他异常，记录错误消息
                Result.Status = "失败";
                Result.ErrorMessage = ex.Message;
                ChannelMapping.HardPointTestResult = "失败";
                throw;
            }
            finally
            {
                // 结束测试时，将测试PLC输出复位到0%
                try
                {
                    var resetResult = await TestPlcCommunication.WriteAnalogValueAsync(ChannelMapping.TestPLCCommunicationAddress.Substring(1), ChannelMapping.RangeLowerLimitValue);
                    if (!resetResult.IsSuccess)
                    {
                        // 记录复位失败但不影响测试结果
                        if (string.IsNullOrEmpty(Result.ErrorMessage))
                            Result.ErrorMessage = $"复位失败：{resetResult.ErrorMessage}";
                        else
                            Result.ErrorMessage += $"\n复位失败：{resetResult.ErrorMessage}";
                    }
                }
                catch
                {
                    // 忽略复位过程中的异常
                }
            }
        }

        /// <summary>
        /// 写入0%测试值
        /// </summary>
        /// <param name="cancellationToken">取消令牌</param>
        /// <returns>异步任务</returns>
        public override async Task Write0PercentTestValueAsync(CancellationToken cancellationToken)
        {
            // 取消检查
            cancellationToken.ThrowIfCancellationRequested();
            
            // 暂停检查
            await CheckAndWaitForResumeAsync(cancellationToken);
            
            try
            {
                // 确保连接已建立
                if (!TestPlcCommunication.IsConnected)
                {
                    await TestPlcCommunication.ConnectAsync();
                }

                // 计算0%测试值
                float minValue = ChannelMapping.RangeLowerLimitValue;
                float testValue = minValue;
                
                // 写入测试值到测试PLC
                var writeResult = await TestPlcCommunication.WriteAnalogValueAsync(
                    ChannelMapping.TestPLCCommunicationAddress.Substring(1), 
                    0f);
                    
                if (!writeResult.IsSuccess)
                {
                    Console.WriteLine($"AI写入0%测试值失败：{writeResult.ErrorMessage}");
                }
            }
            catch (Exception ex)
            {
                Console.WriteLine($"AI写入0%测试值时出错: {ex.Message}");
                throw;
            }
        }

        /// <summary>
        /// 读取0%测试值
        /// </summary>
        /// <param name="cancellationToken">取消令牌</param>
        /// <returns>异步任务</returns>
        public override async Task Read0PercentTestValueAsync(CancellationToken cancellationToken)
        {
            // 取消检查
            cancellationToken.ThrowIfCancellationRequested();
            
            // 暂停检查
            await CheckAndWaitForResumeAsync(cancellationToken);
            
            try
            {
                // 确保连接已建立
                if (!TargetPlcCommunication.IsConnected)
                {
                    await TargetPlcCommunication.ConnectAsync();
                }

                // 读取被测PLC的值
                var readResult = await TargetPlcCommunication.ReadAnalogValueAsync(
                    ChannelMapping.PlcCommunicationAddress.Substring(1));
                    
                if (!readResult.IsSuccess)
                {
                    Console.WriteLine($"AI读取0%测试值失败：{readResult.ErrorMessage}");
                }
                else
                {
                    float actualValue = readResult.Data;
                    Result.Value0Percent = actualValue;
                    Console.WriteLine($"AI存储0%值: {actualValue}");
                    
                    // 更新测试结果
                    Result.ExpectedValue = ChannelMapping.RangeLowerLimitValue;
                    Result.ActualValue = actualValue;
                }
            }
            catch (Exception ex)
            {
                Console.WriteLine($"AI读取0%测试值时出错: {ex.Message}");
                throw;
            }
        }

        /// <summary>
        /// 写入25%测试值
        /// </summary>
        /// <param name="cancellationToken">取消令牌</param>
        /// <returns>异步任务</returns>
        public override async Task Write25PercentTestValueAsync(CancellationToken cancellationToken)
        {
            // 取消检查
            cancellationToken.ThrowIfCancellationRequested();
            
            // 暂停检查
            await CheckAndWaitForResumeAsync(cancellationToken);
            
            try
            {
                // 确保连接已建立
                if (!TestPlcCommunication.IsConnected)
                {
                    await TestPlcCommunication.ConnectAsync();
                }

                // 计算25%测试值
                float minValue = ChannelMapping.RangeLowerLimitValue;
                float maxValue = ChannelMapping.RangeUpperLimitValue;
                float range = maxValue - minValue;
                float testValue = minValue + (range * 25 / 100);
                
                // 写入测试值到测试PLC
                var writeResult = await TestPlcCommunication.WriteAnalogValueAsync(
                    ChannelMapping.TestPLCCommunicationAddress.Substring(1), 
                    25f);
                    
                if (!writeResult.IsSuccess)
                {
                    Console.WriteLine($"AI写入25%测试值失败：{writeResult.ErrorMessage}");
                }
            }
            catch (Exception ex)
            {
                Console.WriteLine($"AI写入25%测试值时出错: {ex.Message}");
                throw;
            }
        }

        /// <summary>
        /// 读取25%测试值
        /// </summary>
        /// <param name="cancellationToken">取消令牌</param>
        /// <returns>异步任务</returns>
        public override async Task Read25PercentTestValueAsync(CancellationToken cancellationToken)
        {
            // 取消检查
            cancellationToken.ThrowIfCancellationRequested();
            
            // 暂停检查
            await CheckAndWaitForResumeAsync(cancellationToken);
            
            try
            {
                // 确保连接已建立
                if (!TargetPlcCommunication.IsConnected)
                {
                    await TargetPlcCommunication.ConnectAsync();
                }

                // 计算预期的25%测试值
                float minValue = ChannelMapping.RangeLowerLimitValue;
                float maxValue = ChannelMapping.RangeUpperLimitValue;
                float range = maxValue - minValue;
                float expectedValue = minValue + (range * 25 / 100);

                // 读取被测PLC的值
                var readResult = await TargetPlcCommunication.ReadAnalogValueAsync(
                    ChannelMapping.PlcCommunicationAddress.Substring(1));
                    
                if (!readResult.IsSuccess)
                {
                    Console.WriteLine($"AI读取25%测试值失败：{readResult.ErrorMessage}");
                }
                else
                {
                    float actualValue = readResult.Data;
                    Result.Value25Percent = actualValue;
                    Console.WriteLine($"AI存储25%值: {actualValue}");
                    
                    // 更新测试结果
                    Result.ExpectedValue = expectedValue;
                    Result.ActualValue = actualValue;
                }
            }
            catch (Exception ex)
            {
                Console.WriteLine($"AI读取25%测试值时出错: {ex.Message}");
                throw;
            }
        }

        /// <summary>
        /// 写入50%测试值
        /// </summary>
        /// <param name="cancellationToken">取消令牌</param>
        /// <returns>异步任务</returns>
        public override async Task Write50PercentTestValueAsync(CancellationToken cancellationToken)
        {
            // 取消检查
            cancellationToken.ThrowIfCancellationRequested();
            
            // 暂停检查
            await CheckAndWaitForResumeAsync(cancellationToken);
            
            try
            {
                // 确保连接已建立
                if (!TestPlcCommunication.IsConnected)
                {
                    await TestPlcCommunication.ConnectAsync();
                }

                // 计算50%测试值
                float minValue = ChannelMapping.RangeLowerLimitValue;
                float maxValue = ChannelMapping.RangeUpperLimitValue;
                float range = maxValue - minValue;
                float testValue = minValue + (range * 50 / 100);
                
                // 写入测试值到测试PLC
                var writeResult = await TestPlcCommunication.WriteAnalogValueAsync(
                    ChannelMapping.TestPLCCommunicationAddress.Substring(1), 
                    50f);
                    
                if (!writeResult.IsSuccess)
                {
                    Console.WriteLine($"AI写入50%测试值失败：{writeResult.ErrorMessage}");
                }
            }
            catch (Exception ex)
            {
                Console.WriteLine($"AI写入50%测试值时出错: {ex.Message}");
                throw;
            }
        }

        /// <summary>
        /// 读取50%测试值
        /// </summary>
        /// <param name="cancellationToken">取消令牌</param>
        /// <returns>异步任务</returns>
        public override async Task Read50PercentTestValueAsync(CancellationToken cancellationToken)
        {
            // 取消检查
            cancellationToken.ThrowIfCancellationRequested();
            
            // 暂停检查
            await CheckAndWaitForResumeAsync(cancellationToken);
            
            try
            {
                // 确保连接已建立
                if (!TargetPlcCommunication.IsConnected)
                {
                    await TargetPlcCommunication.ConnectAsync();
                }

                // 计算预期的50%测试值
                float minValue = ChannelMapping.RangeLowerLimitValue;
                float maxValue = ChannelMapping.RangeUpperLimitValue;
                float range = maxValue - minValue;
                float expectedValue = minValue + (range * 50 / 100);

                // 读取被测PLC的值
                var readResult = await TargetPlcCommunication.ReadAnalogValueAsync(
                    ChannelMapping.PlcCommunicationAddress.Substring(1));
                    
                if (!readResult.IsSuccess)
                {
                    Console.WriteLine($"AI读取50%测试值失败：{readResult.ErrorMessage}");
                }
                else
                {
                    float actualValue = readResult.Data;
                    Result.Value50Percent = actualValue;
                    Console.WriteLine($"AI存储50%值: {actualValue}");
                    
                    // 更新测试结果
                    Result.ExpectedValue = expectedValue;
                    Result.ActualValue = actualValue;
                }
            }
            catch (Exception ex)
            {
                Console.WriteLine($"AI读取50%测试值时出错: {ex.Message}");
                throw;
            }
        }

        /// <summary>
        /// 写入75%测试值
        /// </summary>
        /// <param name="cancellationToken">取消令牌</param>
        /// <returns>异步任务</returns>
        public override async Task Write75PercentTestValueAsync(CancellationToken cancellationToken)
        {
            // 取消检查
            cancellationToken.ThrowIfCancellationRequested();
            
            // 暂停检查
            await CheckAndWaitForResumeAsync(cancellationToken);
            
            try
            {
                // 确保连接已建立
                if (!TestPlcCommunication.IsConnected)
                {
                    await TestPlcCommunication.ConnectAsync();
                }

                // 计算75%测试值
                float minValue = ChannelMapping.RangeLowerLimitValue;
                float maxValue = ChannelMapping.RangeUpperLimitValue;
                float range = maxValue - minValue;
                float testValue = minValue + (range * 75 / 100);
                
                // 写入测试值到测试PLC
                var writeResult = await TestPlcCommunication.WriteAnalogValueAsync(
                    ChannelMapping.TestPLCCommunicationAddress.Substring(1), 
                    75f);
                    
                if (!writeResult.IsSuccess)
                {
                    Console.WriteLine($"AI写入75%测试值失败：{writeResult.ErrorMessage}");
                }
            }
            catch (Exception ex)
            {
                Console.WriteLine($"AI写入75%测试值时出错: {ex.Message}");
                throw;
            }
        }

        /// <summary>
        /// 读取75%测试值
        /// </summary>
        /// <param name="cancellationToken">取消令牌</param>
        /// <returns>异步任务</returns>
        public override async Task Read75PercentTestValueAsync(CancellationToken cancellationToken)
        {
            // 取消检查
            cancellationToken.ThrowIfCancellationRequested();
            
            // 暂停检查
            await CheckAndWaitForResumeAsync(cancellationToken);
            
            try
            {
                // 确保连接已建立
                if (!TargetPlcCommunication.IsConnected)
                {
                    await TargetPlcCommunication.ConnectAsync();
                }

                // 计算预期的75%测试值
                float minValue = ChannelMapping.RangeLowerLimitValue;
                float maxValue = ChannelMapping.RangeUpperLimitValue;
                float range = maxValue - minValue;
                float expectedValue = minValue + (range * 75 / 100);

                // 读取被测PLC的值
                var readResult = await TargetPlcCommunication.ReadAnalogValueAsync(
                    ChannelMapping.PlcCommunicationAddress.Substring(1));
                    
                if (!readResult.IsSuccess)
                {
                    Console.WriteLine($"AI读取75%测试值失败：{readResult.ErrorMessage}");
                }
                else
                {
                    float actualValue = readResult.Data;
                    Result.Value75Percent = actualValue;
                    Console.WriteLine($"AI存储75%值: {actualValue}");
                    
                    // 更新测试结果
                    Result.ExpectedValue = expectedValue;
                    Result.ActualValue = actualValue;
                }
            }
            catch (Exception ex)
            {
                Console.WriteLine($"AI读取75%测试值时出错: {ex.Message}");
                throw;
            }
        }

        /// <summary>
        /// 写入100%测试值
        /// </summary>
        /// <param name="cancellationToken">取消令牌</param>
        /// <returns>异步任务</returns>
        public override async Task Write100PercentTestValueAsync(CancellationToken cancellationToken)
        {
            // 取消检查
            cancellationToken.ThrowIfCancellationRequested();
            
            // 暂停检查
            await CheckAndWaitForResumeAsync(cancellationToken);
            
            try
            {
                // 确保连接已建立
                if (!TestPlcCommunication.IsConnected)
                {
                    await TestPlcCommunication.ConnectAsync();
                }

                // 计算100%测试值
                float maxValue = ChannelMapping.RangeUpperLimitValue;
                
                // 写入测试值到测试PLC
                var writeResult = await TestPlcCommunication.WriteAnalogValueAsync(
                    ChannelMapping.TestPLCCommunicationAddress.Substring(1), 
                    100f);
                    
                if (!writeResult.IsSuccess)
                {
                    Console.WriteLine($"AI写入100%测试值失败：{writeResult.ErrorMessage}");
                }
            }
            catch (Exception ex)
            {
                Console.WriteLine($"AI写入100%测试值时出错: {ex.Message}");
                throw;
            }
        }

        /// <summary>
        /// 读取100%测试值
        /// </summary>
        /// <param name="cancellationToken">取消令牌</param>
        /// <returns>异步任务</returns>
        public override async Task Read100PercentTestValueAsync(CancellationToken cancellationToken)
        {
            // 取消检查
            cancellationToken.ThrowIfCancellationRequested();
            
            // 暂停检查
            await CheckAndWaitForResumeAsync(cancellationToken);
            
            try
            {
                // 确保连接已建立
                if (!TargetPlcCommunication.IsConnected)
                {
                    await TargetPlcCommunication.ConnectAsync();
                }

                // 使用100%测试值
                float maxValue = ChannelMapping.RangeUpperLimitValue;

                // 读取被测PLC的值
                var readResult = await TargetPlcCommunication.ReadAnalogValueAsync(
                    ChannelMapping.PlcCommunicationAddress.Substring(1));
                    
                if (!readResult.IsSuccess)
                {
                    Console.WriteLine($"AI读取100%测试值失败：{readResult.ErrorMessage}");
                }
                else
                {
                    float actualValue = readResult.Data;
                    Result.Value100Percent = actualValue;
                    Console.WriteLine($"AI存储100%值: {actualValue}");
                    
                    // 更新测试结果
                    Result.ExpectedValue = maxValue;
                    Result.ActualValue = actualValue;
                    
                    // 判断测试是否通过
                    // 计算所有测试点的偏差是否在容许范围内
                    bool allTestsPassed = EvaluateTestResults();
                    
                    // 设置最终测试状态
                    if (allTestsPassed)
                    {
                        Result.Status = "通过";
                        ChannelMapping.HardPointTestResult = "通过";
                    }
                    else
                    {
                        Result.Status = "失败";
                        ChannelMapping.HardPointTestResult = "失败";
                        ChannelMapping.TestResultStatus = 2;
                    }
                }
            }
            catch (Exception ex)
            {
                Console.WriteLine($"AI读取100%测试值时出错: {ex.Message}");
                throw;
            }
        }

        /// <summary>
        /// 写入复位值
        /// </summary>
        /// <param name="cancellationToken">取消令牌</param>
        /// <returns>异步任务</returns>
        public override async Task WriteResetValueAsync(CancellationToken cancellationToken)
        {
            try
            {
                // 确保连接已建立
                if (!TestPlcCommunication.IsConnected)
                {
                    await TestPlcCommunication.ConnectAsync();
                }

                // 复位为0%值
                var resetResult = await TestPlcCommunication.WriteAnalogValueAsync(
                    ChannelMapping.TestPLCCommunicationAddress.Substring(1), 
                    0f);
                    
                if (!resetResult.IsSuccess)
                {
                    Console.WriteLine($"AI复位值失败：{resetResult.ErrorMessage}");
                    if (string.IsNullOrEmpty(Result.ErrorMessage))
                        Result.ErrorMessage = $"复位失败：{resetResult.ErrorMessage}";
                    else
                        Result.ErrorMessage += $"\n复位失败：{resetResult.ErrorMessage}";
                }
            }
            catch (Exception ex)
            {
                Console.WriteLine($"AI复位值时出错: {ex.Message}");
                // 记录错误但不抛出异常，避免中断测试流程
                if (string.IsNullOrEmpty(Result.ErrorMessage))
                    Result.ErrorMessage = $"复位失败：{ex.Message}";
                else
                    Result.ErrorMessage += $"\n复位失败：{ex.Message}";
            }
        }

        /// <summary>
        /// 评估所有测试结果
        /// </summary>
        /// <returns>是否所有测试都通过</returns>
        private bool EvaluateTestResults()
        {
            try
            {
                // 收集所有需要评估的测试点
                Dictionary<string, (float Expected, float Actual)> testPoints = new Dictionary<string, (float Expected, float Actual)>();
                
                // 计算各个测试百分比的预期值
                float minValue = ChannelMapping.RangeLowerLimitValue;
                float maxValue = ChannelMapping.RangeUpperLimitValue;
                float range = maxValue - minValue;

                // 添加所有测试点
                //testPoints.Add("0%", (Expected: minValue, Actual: (float)Result.Value0Percent));
                //testPoints.Add("25%", (Expected: minValue + (range * 25 / 100), Actual: (float)Result.Value25Percent));
                //testPoints.Add("50%", (Expected: minValue + (range * 50 / 100), Actual: (float)Result.Value50Percent));
                //testPoints.Add("75%", (Expected: minValue + (range * 75 / 100), Actual: (float)Result.Value75Percent));
                //testPoints.Add("100%", (Expected: maxValue, Actual: (float)Result.Value100Percent));
                //添加所有测试点，
                //testPoints.Add("0%", (Expected: 0f, Actual: (float)Result.Value0Percent));
                //testPoints.Add("25%", (Expected: 25f, Actual: (float)Result.Value25Percent));
                //testPoints.Add("50%", (Expected: 50f, Actual: (float)Result.Value50Percent));
                //testPoints.Add("75%", (Expected: 75f, Actual: (float)Result.Value75Percent));
                //testPoints.Add("100%", (Expected: 100f, Actual: (float)Result.Value100Percent));
                //添加所有测试点，
                testPoints.Add("0%", (Expected: minValue, Actual: (float)Result.Value0Percent));
                testPoints.Add("25%", (Expected: minValue + (range * 25 / 100), Actual: (float)Result.Value25Percent));
                testPoints.Add("50%", (Expected: minValue + (range * 50 / 100), Actual: (float)Result.Value50Percent));
                testPoints.Add("75%", (Expected: minValue + (range * 75 / 100), Actual: (float)Result.Value75Percent));
                testPoints.Add("100%", (Expected: maxValue, Actual: (float)Result.Value100Percent));

                // 创建详细测试报告
                StringBuilder testReport = new StringBuilder();
                bool allPassed = true;
                
                // 允许的最大偏差百分比
                const float allowedDeviation = 2.0f;
                
                // 评估每个测试点
                foreach (var point in testPoints)
                {
                    float expected = point.Value.Expected;
                    float actual = point.Value.Actual;
                    
                    // 计算偏差
                    float deviation = Math.Abs(actual - expected);
                    float deviationPercent = (expected != 0) ? (deviation / Math.Abs(expected)) * 100 : 0;
                    
                    // 判断是否通过
                    bool pointPassed = deviationPercent <= allowedDeviation;
                    if (!pointPassed)
                        allPassed = false;
                        
                    // 添加到报告
                    testReport.AppendLine($"{point.Key}测试" + (pointPassed ? "通过" : $"失败：偏差{deviationPercent:F2}%超出范围"));
                }
                
                // 保存详细报告
                Result.ErrorMessage = testReport.ToString();
                
                return allPassed;
            }
            catch (Exception ex)
            {
                Console.WriteLine($"评估测试结果时出错: {ex.Message}");
                Result.ErrorMessage = $"评估测试结果时出错: {ex.Message}";
                return false;
            }
        }
    }
} 