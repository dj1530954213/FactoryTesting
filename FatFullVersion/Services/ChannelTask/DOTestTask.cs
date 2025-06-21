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
        /// 执行DO测试逻辑
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
                // DO测试：由被测PLC设置数字量输出，然后测试PLC检测该信号
                
                // 取消检查
                cancellationToken.ThrowIfCancellationRequested();
                    
                // 暂停检查
                await CheckAndWaitForResumeAsync(cancellationToken);
                
                bool allTestsPassed = true;
                Result.Status = "";
                
                // 创建详细测试日志
                StringBuilder detailedTestLog = new StringBuilder();
                
                // 测试信号为1（闭合）
                var writeHighResult = await TargetPlcCommunication.WriteDigitalValueAsync(
                    ChannelMapping.PlcCommunicationAddress.Substring(1), 
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
                    
                    // 读取测试PLC的值
                    var readHighResult = await TestPlcCommunication.ReadDigitalValueAsync(
                        ChannelMapping.TestPLCCommunicationAddress.Substring(1));
                        
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
                
                // 测试信号为0（断开）
                if (allTestsPassed)
                {
                    // 取消检查
                    cancellationToken.ThrowIfCancellationRequested();
                        
                    // 暂停检查
                    await CheckAndWaitForResumeAsync(cancellationToken);
                    
                    var writeLowResult = await TargetPlcCommunication.WriteDigitalValueAsync(
                        ChannelMapping.PlcCommunicationAddress.Substring(1), 
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
                        
                        // 读取测试PLC的值
                        var readLowResult = await TestPlcCommunication.ReadDigitalValueAsync(
                            ChannelMapping.TestPLCCommunicationAddress.Substring(1));
                            
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
                }
                
                // 保存详细测试日志
                Result.ErrorMessage = detailedTestLog.ToString();
                
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
                // 测试完成后，将被测PLC的DO输出复位为0
                try
                {
                    await TargetPlcCommunication.WriteDigitalValueAsync(
                        ChannelMapping.PlcCommunicationAddress.Substring(1), 
                        false);
                }
                catch (Exception ex)
                {
                    Console.WriteLine($"复位DO通道失败: {ex.Message}");
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
                if (!TargetPlcCommunication.IsConnected)
                {
                    await TargetPlcCommunication.ConnectAsync();
                }

                // 写入高信号（true）到被测PLC
                var writeHighResult = await TargetPlcCommunication.WriteDigitalValueAsync(
                    ChannelMapping.PlcCommunicationAddress.Substring(1), 
                    true);
                    
                if (!writeHighResult.IsSuccess)
                {
                    Console.WriteLine($"DO写入高信号失败: {writeHighResult.ErrorMessage}");
                }
            }
            catch (Exception ex)
            {
                Console.WriteLine($"DO写入高信号时出错: {ex.Message}");
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
                if (!TestPlcCommunication.IsConnected)
                {
                    await TestPlcCommunication.ConnectAsync();
                }

                // 读取测试PLC的值
                var readHighResult = await TestPlcCommunication.ReadDigitalValueAsync(
                    ChannelMapping.TestPLCCommunicationAddress.Substring(1));
                    
                if (!readHighResult.IsSuccess)
                {
                    Console.WriteLine($"DO读取高信号失败: {readHighResult.ErrorMessage}");
                }
                else
                {
                    bool actualHighValue = readHighResult.Data;
                    
                    // 记录实际值
                    if (actualHighValue)
                    {
                        Console.WriteLine("DO高信号测试通过");
                    }
                    else
                    {
                        Console.WriteLine("DO高信号测试失败: 期望值为true，实际值为false");
                    }
                    
                    // 更新测试结果
                    Result.ExpectedValue = 1;
                    Result.ActualValue = actualHighValue ? 1 : 0;
                }
            }
            catch (Exception ex)
            {
                Console.WriteLine($"DO读取高信号时出错: {ex.Message}");
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
                if (!TargetPlcCommunication.IsConnected)
                {
                    await TargetPlcCommunication.ConnectAsync();
                }

                // 写入低信号（false）到被测PLC
                var writeLowResult = await TargetPlcCommunication.WriteDigitalValueAsync(
                    ChannelMapping.PlcCommunicationAddress.Substring(1), 
                    false);
                    
                if (!writeLowResult.IsSuccess)
                {
                    Console.WriteLine($"DO写入低信号失败: {writeLowResult.ErrorMessage}");
                }
            }
            catch (Exception ex)
            {
                Console.WriteLine($"DO写入低信号时出错: {ex.Message}");
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
                if (!TestPlcCommunication.IsConnected)
                {
                    await TestPlcCommunication.ConnectAsync();
                }

                // 读取测试PLC的值
                var readLowResult = await TestPlcCommunication.ReadDigitalValueAsync(
                    ChannelMapping.TestPLCCommunicationAddress.Substring(1));
                    
                if (!readLowResult.IsSuccess)
                {
                    Console.WriteLine($"DO读取低信号失败: {readLowResult.ErrorMessage}");
                }
                else
                {
                    bool actualLowValue = readLowResult.Data;
                    
                    // 记录实际值 - 低信号测试需要值为false才通过
                    if (!actualLowValue)
                    {
                        Console.WriteLine("DO低信号测试通过");
                    }
                    else
                    {
                        Console.WriteLine("DO低信号测试失败: 期望值为false，实际值为true");
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
                Console.WriteLine($"DO读取低信号时出错: {ex.Message}");
                throw;
            }
        }

        /// <summary>
        /// 写入0%测试值
        /// </summary>
        /// <param name="cancellationToken">取消令牌</param>
        /// <returns>异步任务</returns>
        public override async Task Write0PercentTestValueAsync(CancellationToken cancellationToken) => 
            await WriteLowSignalAsync(cancellationToken);

        /// <summary>
        /// 读取0%测试值
        /// </summary>
        /// <param name="cancellationToken">取消令牌</param>
        /// <returns>异步任务</returns>
        public override async Task Read0PercentTestValueAsync(CancellationToken cancellationToken) => 
            await ReadLowSignalAsync(cancellationToken);

        /// <summary>
        /// 写入25%测试值
        /// </summary>
        /// <param name="cancellationToken">取消令牌</param>
        /// <returns>异步任务</returns>
        public override async Task Write25PercentTestValueAsync(CancellationToken cancellationToken) => 
            await WriteHighSignalAsync(cancellationToken);

        /// <summary>
        /// 读取25%测试值
        /// </summary>
        /// <param name="cancellationToken">取消令牌</param>
        /// <returns>异步任务</returns>
        public override async Task Read25PercentTestValueAsync(CancellationToken cancellationToken) => 
            await ReadHighSignalAsync(cancellationToken);

        /// <summary>
        /// 写入复位值
        /// </summary>
        /// <param name="cancellationToken">取消令牌</param>
        /// <returns>异步任务</returns>
        public override async Task WriteResetValueAsync(CancellationToken cancellationToken) => 
            await WriteLowSignalAsync(cancellationToken);

        /// <summary>
        /// 写入50%测试值
        /// </summary>
        /// <param name="cancellationToken">取消令牌</param>
        /// <returns>异步任务</returns>
        public override async Task Write50PercentTestValueAsync(CancellationToken cancellationToken) => 
            await Task.CompletedTask;

        /// <summary>
        /// 读取50%测试值
        /// </summary>
        /// <param name="cancellationToken">取消令牌</param>
        /// <returns>异步任务</returns>
        public override async Task Read50PercentTestValueAsync(CancellationToken cancellationToken) => 
            await Task.CompletedTask;

        /// <summary>
        /// 写入75%测试值
        /// </summary>
        /// <param name="cancellationToken">取消令牌</param>
        /// <returns>异步任务</returns>
        public override async Task Write75PercentTestValueAsync(CancellationToken cancellationToken) => 
            await Task.CompletedTask;

        /// <summary>
        /// 读取75%测试值
        /// </summary>
        /// <param name="cancellationToken">取消令牌</param>
        /// <returns>异步任务</returns>
        public override async Task Read75PercentTestValueAsync(CancellationToken cancellationToken) => 
            await Task.CompletedTask;

        /// <summary>
        /// 写入100%测试值
        /// </summary>
        /// <param name="cancellationToken">取消令牌</param>
        /// <returns>异步任务</returns>
        public override async Task Write100PercentTestValueAsync(CancellationToken cancellationToken) => 
            await Task.CompletedTask;

        /// <summary>
        /// 读取100%测试值
        /// </summary>
        /// <param name="cancellationToken">取消令牌</param>
        /// <returns>异步任务</returns>
        public override async Task Read100PercentTestValueAsync(CancellationToken cancellationToken) => 
            await Task.CompletedTask;
    }
}