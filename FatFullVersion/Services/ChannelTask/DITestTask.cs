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
        /// 执行DI测试逻辑
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
                // DI测试：测试PLC设置开关量信号，然后检查被测PLC是否正确接收
                
                // 取消检查
                cancellationToken.ThrowIfCancellationRequested();
                    
                // 暂停检查
                await CheckAndWaitForResumeAsync(cancellationToken);
                
                bool allTestsPassed = true;
                Result.Status = "";
                
                // 创建详细测试日志
                StringBuilder detailedTestLog = new StringBuilder();
                
                // 测试信号为1（闭合/接通）
                var writeHighResult = await TestPlcCommunication.WriteDigitalValueAsync(
                    ChannelMapping.TestPLCCommunicationAddress.Substring(1), 
                    true);
                    
                if (!writeHighResult.IsSuccess)
                {
                    detailedTestLog.AppendLine($"写入高信号失败: {writeHighResult.ErrorMessage}");
                    allTestsPassed = false;
                }
                else
                {
                    // 等待信号稳定
                    await Task.Delay(3000, cancellationToken);
                    
                    // 读取被测PLC的值
                    var readHighResult = await TargetPlcCommunication.ReadDigitalValueAsync(
                        ChannelMapping.PlcCommunicationAddress.Substring(1));
                        
                    if (!readHighResult.IsSuccess)
                    {
                        detailedTestLog.AppendLine($"读取高信号失败: {readHighResult.ErrorMessage}");
                        allTestsPassed = false;
                    }
                    else
                    {
                        bool actualHighValue = readHighResult.Data;
                        
                        if (actualHighValue)
                        {
                            detailedTestLog.AppendLine("高信号测试通过");
                        }
                        else
                        {
                            detailedTestLog.AppendLine("高信号测试失败: 期望值为true，实际值为false");
                            allTestsPassed = false;
                        }
                        
                        // 更新测试结果
                        Result.ExpectedValue = 1;
                        Result.ActualValue = actualHighValue ? 1 : 0;
                    }
                }

                await Task.Delay(1000, cancellationToken);
                // 测试信号为0（断开）
                if (allTestsPassed)
                {
                    // 取消检查
                    cancellationToken.ThrowIfCancellationRequested();
                        
                    // 暂停检查
                    await CheckAndWaitForResumeAsync(cancellationToken);
                    
                    var writeLowResult = await TestPlcCommunication.WriteDigitalValueAsync(
                        ChannelMapping.TestPLCCommunicationAddress.Substring(1), 
                        false);
                        
                    if (!writeLowResult.IsSuccess)
                    {
                        detailedTestLog.AppendLine($"写入低信号失败: {writeLowResult.ErrorMessage}");
                        allTestsPassed = false;
                    }
                    else
                    {
                        // 等待信号稳定
                        await Task.Delay(3000, cancellationToken);
                        
                        // 读取被测PLC的值
                        var readLowResult = await TargetPlcCommunication.ReadDigitalValueAsync(
                            ChannelMapping.PlcCommunicationAddress.Substring(1));
                            
                        if (!readLowResult.IsSuccess)
                        {
                            detailedTestLog.AppendLine($"读取低信号失败: {readLowResult.ErrorMessage}");
                            allTestsPassed = false;
                        }
                        else
                        {
                            bool actualLowValue = readLowResult.Data;
                            
                            if (!actualLowValue)
                            {
                                detailedTestLog.AppendLine("低信号测试通过");
                            }
                            else
                            {
                                detailedTestLog.AppendLine("低信号测试失败: 期望值为false，实际值为true");
                                allTestsPassed = false;
                            }
                            
                            // 更新测试结果
                            Result.ExpectedValue = 0;
                            Result.ActualValue = actualLowValue ? 1 : 0;
                        }
                    }
                }
                await Task.Delay(1000, cancellationToken);
                // 保存详细测试日志
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
                // 任务被取消
                throw;
            }
            catch (Exception ex)
            {
                // 其他异常
                Result.Status = "失败";
                Result.ErrorMessage = ex.Message;
                ChannelMapping.HardPointTestResult = "失败";
                throw;
            }
            finally
            {
                // 测试完成后，将测试信号复位为0
                try
                {
                    await TestPlcCommunication.WriteDigitalValueAsync(
                        ChannelMapping.TestPLCCommunicationAddress.Substring(1), 
                        false);
                }
                catch (Exception ex)
                {
                    Console.WriteLine($"复位DI通道失败: {ex.Message}");
                    if (string.IsNullOrEmpty(Result.ErrorMessage))
                        Result.ErrorMessage = $"复位失败: {ex.Message}";
                    else
                        Result.ErrorMessage += $"\n复位失败: {ex.Message}";
                }
            }
        }

        /// <summary>
        /// 写入高信号（true）
        /// </summary>
        /// <param name="cancellationToken">取消令牌</param>
        /// <returns>异步任务</returns>
        public override async Task WriteHighSignalAsync(CancellationToken cancellationToken)
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

                // 写入高信号（true）到测试PLC
                var writeHighResult = await TestPlcCommunication.WriteDigitalValueAsync(
                    ChannelMapping.TestPLCCommunicationAddress.Substring(1), 
                    true);
                    
                if (!writeHighResult.IsSuccess)
                {
                    Console.WriteLine($"DI写入高信号失败: {writeHighResult.ErrorMessage}");
                }
            }
            catch (Exception ex)
            {
                Console.WriteLine($"DI写入高信号时出错: {ex.Message}");
                throw;
            }
        }

        /// <summary>
        /// 读取高信号（true）
        /// </summary>
        /// <param name="cancellationToken">取消令牌</param>
        /// <returns>异步任务</returns>
        public override async Task ReadHighSignalAsync(CancellationToken cancellationToken)
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
                    var result = await TargetPlcCommunication.ConnectAsync();
                }

                // 读取被测PLC的值
                var readHighResult = await TargetPlcCommunication.ReadDigitalValueAsync(
                    ChannelMapping.PlcCommunicationAddress);
                if (!readHighResult.IsSuccess)
                {
                    Console.WriteLine($"DI读取高信号失败: {readHighResult.ErrorMessage}");
                }
                else
                {
                    bool actualHighValue;
                    //对常闭点位进行取反处理
                    if (ChannelMapping.WireSystem == "常闭")
                    {
                        //目前先不使用取反的逻辑
                        //actualHighValue = !readHighResult.Data;
                        actualHighValue = readHighResult.Data;
                    }
                    else
                    {
                        actualHighValue = readHighResult.Data;
                    }

                    // 记录实际值
                    if (actualHighValue)
                    {
                        Console.WriteLine("DI高信号测试通过");
                    }
                    else
                    {
                        Console.WriteLine("DI高信号测试失败: 期望值为true，实际值为false");
                    }
                    
                    // 更新测试结果
                    Result.ExpectedValue = 1;
                    Result.ActualValue = actualHighValue ? 1 : 0;
                }
            }
            catch (Exception ex)
            {
                Console.WriteLine($"DI读取高信号时出错: {ex.Message}");
                throw;
            }
        }

        /// <summary>
        /// 写入低信号（false）
        /// </summary>
        /// <param name="cancellationToken">取消令牌</param>
        /// <returns>异步任务</returns>
        public override async Task WriteLowSignalAsync(CancellationToken cancellationToken)
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

                // 写入低信号（false）到测试PLC
                var writeLowResult = await TestPlcCommunication.WriteDigitalValueAsync(
                    ChannelMapping.TestPLCCommunicationAddress.Substring(1), 
                    false);
                    
                if (!writeLowResult.IsSuccess)
                {
                    Console.WriteLine($"DI写入低信号失败: {writeLowResult.ErrorMessage}");
                }
            }
            catch (Exception ex)
            {
                Console.WriteLine($"DI写入低信号时出错: {ex.Message}");
                throw;
            }
        }

        /// <summary>
        /// 读取低信号（false）
        /// </summary>
        /// <param name="cancellationToken">取消令牌</param>
        /// <returns>异步任务</returns>
        public override async Task ReadLowSignalAsync(CancellationToken cancellationToken)
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
                var readLowResult = await TargetPlcCommunication.ReadDigitalValueAsync(
                    ChannelMapping.PlcCommunicationAddress);
                    
                if (!readLowResult.IsSuccess)
                {
                    Console.WriteLine($"DI读取低信号失败: {readLowResult.ErrorMessage}");
                }
                else
                {
                    bool actualLowValue;
                    if (ChannelMapping.WireSystem == "常闭")
                    {
                        //目前先不使用取反的逻辑
                        //actualLowValue = !readLowResult.Data;
                        actualLowValue = readLowResult.Data;
                    }
                    else
                    {
                        actualLowValue = readLowResult.Data;
                    }

                    // 记录实际值 - 低信号测试需要值为false才通过
                    if (!actualLowValue)
                    {
                        Console.WriteLine("DI低信号测试通过");
                    }
                    else
                    {
                        Console.WriteLine("DI低信号测试失败: 期望值为false，实际值为true");
                    }
                    
                    // 保存低信号测试结果到临时变量，而不是直接覆盖Result
                    double lowSignalExpectedValue = 0;
                    double lowSignalActualValue = actualLowValue ? 1 : 0;
                    
                    // 获取之前保存的高信号测试结果
                    double highSignalExpectedValue = Result.ExpectedValue;
                    double highSignalActualValue = Result.ActualValue;
                    
                    // 评估测试结果，使用正确的变量比较
                    bool highSignalPassed = (highSignalExpectedValue == 1) && (highSignalActualValue == 1);
                    bool lowSignalPassed = (lowSignalExpectedValue == 0) && (lowSignalActualValue == 0);
                    StringBuilder testReport = new StringBuilder();
                    
                    // 添加高信号测试结果
                    testReport.AppendLine(highSignalPassed ? "高信号测试通过" : "高信号测试失败: 期望值为true，实际值为false");
                    // 添加低信号测试结果
                    testReport.AppendLine(lowSignalPassed ? "低信号测试通过" : "低信号测试失败: 期望值为false，实际值为true");
                    
                    // 保存测试报告
                    Result.ErrorMessage = testReport.ToString();
                    
                    // 更新Result值为低信号结果 (这里更新是可以的，因为高信号结果已经在上面的逻辑中使用过了)
                    Result.ExpectedValue = lowSignalExpectedValue;
                    Result.ActualValue = lowSignalActualValue;
                    
                    // 设置最终测试状态
                    if (highSignalPassed && lowSignalPassed)
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
                Console.WriteLine($"DI读取低信号时出错: {ex.Message}");
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

                // 将测试PLC的DI输出复位为false
                var resetResult = await TestPlcCommunication.WriteDigitalValueAsync(
                    ChannelMapping.TestPLCCommunicationAddress.Substring(1), 
                    false);
                    
                if (!resetResult.IsSuccess)
                {
                    Console.WriteLine($"复位DI通道失败: {resetResult.ErrorMessage}");
                    if (string.IsNullOrEmpty(Result.ErrorMessage))
                        Result.ErrorMessage = $"复位失败: {resetResult.ErrorMessage}";
                    else
                        Result.ErrorMessage += $"\n复位失败: {resetResult.ErrorMessage}";
                }
            }
            catch (Exception ex)
            {
                Console.WriteLine($"复位DI通道失败: {ex.Message}");
                // 记录错误但不抛出异常，避免中断测试流程
                if (string.IsNullOrEmpty(Result.ErrorMessage))
                    Result.ErrorMessage = $"复位失败: {ex.Message}";
                else
                    Result.ErrorMessage += $"\n复位失败: {ex.Message}";
            }
        }

        /// <summary>
        /// 使用50%的测试点无需实际操作，仅返回
        /// </summary>
        public override async Task Write50PercentTestValueAsync(CancellationToken cancellationToken) => 
            await Task.CompletedTask;

        /// <summary>
        /// 使用50%的测试点无需实际操作，仅返回
        /// </summary>
        public override async Task Read50PercentTestValueAsync(CancellationToken cancellationToken) => 
            await Task.CompletedTask;

        /// <summary>
        /// 使用75%的测试点无需实际操作，仅返回
        /// </summary>
        public override async Task Write75PercentTestValueAsync(CancellationToken cancellationToken) => 
            await Task.CompletedTask;

        /// <summary>
        /// 使用75%的测试点无需实际操作，仅返回
        /// </summary>
        public override async Task Read75PercentTestValueAsync(CancellationToken cancellationToken) => 
            await Task.CompletedTask;

        /// <summary>
        /// 使用100%的测试点无需实际操作，仅返回
        /// </summary>
        public override async Task Write100PercentTestValueAsync(CancellationToken cancellationToken) => 
            await Task.CompletedTask;

        /// <summary>
        /// 使用100%的测试点无需实际操作，仅返回
        /// </summary>
        public override async Task Read100PercentTestValueAsync(CancellationToken cancellationToken) => 
            await Task.CompletedTask;

        /// <summary>
        /// 写入0%测试值
        /// </summary>
        /// <param name="cancellationToken">取消令牌</param>
        /// <returns>异步任务</returns>
        public override async Task Write0PercentTestValueAsync(CancellationToken cancellationToken)
        {
            // 对于DI测试，0%相当于写入低信号(false)
            await WriteLowSignalAsync(cancellationToken);
        }

        /// <summary>
        /// 读取0%测试值
        /// </summary>
        /// <param name="cancellationToken">取消令牌</param>
        /// <returns>异步任务</returns>
        public override async Task Read0PercentTestValueAsync(CancellationToken cancellationToken)
        {
            // 对于DI测试，0%相当于读取低信号(false)
            await ReadLowSignalAsync(cancellationToken);
        }

        /// <summary>
        /// 写入25%测试值
        /// </summary>
        /// <param name="cancellationToken">取消令牌</param>
        /// <returns>异步任务</returns>
        public override async Task Write25PercentTestValueAsync(CancellationToken cancellationToken)
        {
            // 对于DI测试，25%也可视为写入高信号(true)
            await WriteHighSignalAsync(cancellationToken);
        }

        /// <summary>
        /// 读取25%测试值
        /// </summary>
        /// <param name="cancellationToken">取消令牌</param>
        /// <returns>异步任务</returns>
        public override async Task Read25PercentTestValueAsync(CancellationToken cancellationToken)
        {
            // 对于DI测试，25%也可视为读取高信号(true)
            await ReadHighSignalAsync(cancellationToken);
        }
    }
}