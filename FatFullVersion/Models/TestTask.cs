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
    /// 测试任务基类，定义测试任务的通用行为和属性
    /// </summary>
    public abstract class TestTask : IDisposable
    {
        #region 属性与字段

        /// <summary>
        /// 任务ID
        /// </summary>
        public string Id { get; }

        /// <summary>
        /// 通道映射信息
        /// </summary>
        public ChannelMapping ChannelMapping { get; }

        /// <summary>
        /// 任务状态
        /// </summary>
        public TestTaskStatus Status { get; protected set; }

        /// <summary>
        /// 测试PLC通信实例
        /// </summary>
        protected readonly IPlcCommunication TestPlcCommunication;

        /// <summary>
        /// 被测PLC通信实例
        /// </summary>
        protected readonly IPlcCommunication TargetPlcCommunication;

        /// <summary>
        /// 测试结果
        /// </summary>
        public ChannelMapping Result { get; protected set; }

        /// <summary>
        /// 是否已完成
        /// </summary>
        public bool IsCompleted { get; protected set; }

        /// <summary>
        /// 是否已取消
        /// </summary>
        public bool IsCancelled { get; protected set; }

        /// <summary>
        /// 是否已暂停
        /// </summary>
        public bool IsPaused { get; protected set; }

        /// <summary>
        /// 任务锁对象，用于同步操作
        /// </summary>
        protected readonly object TaskLock = new object();

        /// <summary>
        /// 用于取消任务的取消令牌源
        /// </summary>
        protected CancellationTokenSource CancellationTokenSource;

        /// <summary>
        /// 暂停任务的信号量
        /// </summary>
        protected readonly ManualResetEventSlim PauseEvent;

        #endregion

        #region 构造函数

        /// <summary>
        /// 创建测试任务实例
        /// </summary>
        /// <param name="id">任务ID</param>
        /// <param name="channelMapping">通道映射信息</param>
        /// <param name="testPlcCommunication">测试PLC通信实例</param>
        /// <param name="targetPlcCommunication">被测PLC通信实例</param>
        protected TestTask(
            string id,
            ChannelMapping channelMapping,
            IPlcCommunication testPlcCommunication,
            IPlcCommunication targetPlcCommunication)
        {
            Id = id ?? throw new ArgumentNullException(nameof(id));
            ChannelMapping = channelMapping ?? throw new ArgumentNullException(nameof(channelMapping));
            TestPlcCommunication = testPlcCommunication ?? throw new ArgumentNullException(nameof(testPlcCommunication));
            TargetPlcCommunication = targetPlcCommunication ?? throw new ArgumentNullException(nameof(targetPlcCommunication));

            Status = TestTaskStatus.Created;
            Result = new ChannelMapping
            {
                VariableName = channelMapping.VariableName,
                TestPLCChannelTag = channelMapping.TestPLCChannelTag,
                StartTime = DateTime.Now
            };

            IsCompleted = false;
            IsCancelled = false;
            IsPaused = false;

            CancellationTokenSource = new CancellationTokenSource();
            PauseEvent = new ManualResetEventSlim(true); // 初始为非暂停状态
        }

        #endregion

        #region 公共方法

        /// <summary>
        /// 启动测试任务
        /// </summary>
        /// <param name="cancellationToken">外部取消令牌</param>
        /// <returns>异步任务</returns>
        public virtual async Task StartAsync(CancellationToken cancellationToken = default)
        {
            lock (TaskLock)
            {
                if (Status == TestTaskStatus.Running)
                    return;

                if (IsCompleted || IsCancelled)
                    return;

                // 确保取消令牌源是有效的
                if (CancellationTokenSource.IsCancellationRequested)
                {
                    CancellationTokenSource.Dispose();
                    CancellationTokenSource = new CancellationTokenSource();
                }

                // 确保PauseEvent是设置状态（非暂停）
                PauseEvent.Set();

                // 合并外部令牌与内部令牌
                var linkedTokenSource = CancellationTokenSource.Token.Register(() => { });
                
                Status = TestTaskStatus.Running;
                IsPaused = false;
            }

            try
            {
                // 更新开始时间
                Result.StartTime = DateTime.Now;

                // 执行实际的测试逻辑
                await ExecuteTestAsync(cancellationToken);

                // 如果测试完成且未取消，则更新任务状态
                if (!IsCancelled)
                {
                    lock (TaskLock)
                    {
                        Status = TestTaskStatus.Completed;
                        IsCompleted = true;
                        Result.EndTime = DateTime.Now;
                    }
                }
            }
            catch (OperationCanceledException)
            {
                // 任务被取消，设置状态
                lock (TaskLock)
                {
                    Status = TestTaskStatus.Cancelled;
                    IsCancelled = true;
                    Result.EndTime = DateTime.Now;
                    Result.Status = "已取消";
                }
            }
            catch (Exception ex)
            {
                // 任务执行出错，设置状态和错误信息
                lock (TaskLock)
                {
                    Status = TestTaskStatus.Failed;
                    Result.EndTime = DateTime.Now;
                    Result.Status = "失败";
                    Result.ErrorMessage = ex.Message;
                }
            }
        }

        /// <summary>
        /// 停止测试任务
        /// </summary>
        /// <returns>异步任务</returns>
        public virtual async Task StopAsync()
        {
            lock (TaskLock)
            {
                if (Status == TestTaskStatus.Completed || Status == TestTaskStatus.Cancelled)
                    return;

                // 请求取消任务
                CancellationTokenSource.Cancel();
                
                // 确保暂停状态解除，以便任务能够处理取消
                PauseEvent.Set();
                
                Status = TestTaskStatus.Cancelled;
                IsCancelled = true;
                Result.EndTime = DateTime.Now;
                Result.Status = "已取消";
            }

            // 给予异步操作一些时间来响应取消
            await Task.Delay(100);
        }

        /// <summary>
        /// 暂停测试任务
        /// </summary>
        /// <returns>异步任务</returns>
        public virtual async Task PauseAsync()
        {
            lock (TaskLock)
            {
                if (Status != TestTaskStatus.Running || IsPaused || IsCompleted || IsCancelled)
                    return;

                // 重置事件，使任务暂停
                PauseEvent.Reset();
                
                Status = TestTaskStatus.Paused;
                IsPaused = true;
            }

            await Task.CompletedTask;
        }

        /// <summary>
        /// 恢复测试任务
        /// </summary>
        /// <returns>异步任务</returns>
        public virtual async Task ResumeAsync()
        {
            lock (TaskLock)
            {
                if (Status != TestTaskStatus.Paused || !IsPaused || IsCompleted || IsCancelled)
                    return;

                // 设置事件，使任务继续执行
                PauseEvent.Set();
                
                Status = TestTaskStatus.Running;
                IsPaused = false;
            }

            await Task.CompletedTask;
        }

        /// <summary>
        /// 释放资源
        /// </summary>
        public virtual void Dispose()
        {
            // 确保任务已停止
            StopAsync().Wait();

            // 释放资源
            CancellationTokenSource.Dispose();
            PauseEvent.Dispose();
        }

        #endregion

        #region 串行测试接口方法

        /// <summary>
        /// 写入0%测试值
        /// </summary>
        /// <param name="cancellationToken">取消令牌</param>
        /// <returns>异步任务</returns>
        public virtual async Task Write0PercentTestValueAsync(CancellationToken cancellationToken)
        {
            await Task.CompletedTask;
        }

        /// <summary>
        /// 读取0%测试值
        /// </summary>
        /// <param name="cancellationToken">取消令牌</param>
        /// <returns>异步任务</returns>
        public virtual async Task Read0PercentTestValueAsync(CancellationToken cancellationToken)
        {
            await Task.CompletedTask;
        }

        /// <summary>
        /// 写入25%测试值
        /// </summary>
        /// <param name="cancellationToken">取消令牌</param>
        /// <returns>异步任务</returns>
        public virtual async Task Write25PercentTestValueAsync(CancellationToken cancellationToken)
        {
            await Task.CompletedTask;
        }

        /// <summary>
        /// 读取25%测试值
        /// </summary>
        /// <param name="cancellationToken">取消令牌</param>
        /// <returns>异步任务</returns>
        public virtual async Task Read25PercentTestValueAsync(CancellationToken cancellationToken)
        {
            await Task.CompletedTask;
        }

        /// <summary>
        /// 写入50%测试值
        /// </summary>
        /// <param name="cancellationToken">取消令牌</param>
        /// <returns>异步任务</returns>
        public virtual async Task Write50PercentTestValueAsync(CancellationToken cancellationToken)
        {
            await Task.CompletedTask;
        }

        /// <summary>
        /// 读取50%测试值
        /// </summary>
        /// <param name="cancellationToken">取消令牌</param>
        /// <returns>异步任务</returns>
        public virtual async Task Read50PercentTestValueAsync(CancellationToken cancellationToken)
        {
            await Task.CompletedTask;
        }

        /// <summary>
        /// 写入75%测试值
        /// </summary>
        /// <param name="cancellationToken">取消令牌</param>
        /// <returns>异步任务</returns>
        public virtual async Task Write75PercentTestValueAsync(CancellationToken cancellationToken)
        {
            await Task.CompletedTask;
        }

        /// <summary>
        /// 读取75%测试值
        /// </summary>
        /// <param name="cancellationToken">取消令牌</param>
        /// <returns>异步任务</returns>
        public virtual async Task Read75PercentTestValueAsync(CancellationToken cancellationToken)
        {
            await Task.CompletedTask;
        }

        /// <summary>
        /// 写入100%测试值
        /// </summary>
        /// <param name="cancellationToken">取消令牌</param>
        /// <returns>异步任务</returns>
        public virtual async Task Write100PercentTestValueAsync(CancellationToken cancellationToken)
        {
            await Task.CompletedTask;
        }

        /// <summary>
        /// 读取100%测试值
        /// </summary>
        /// <param name="cancellationToken">取消令牌</param>
        /// <returns>异步任务</returns>
        public virtual async Task Read100PercentTestValueAsync(CancellationToken cancellationToken)
        {
            await Task.CompletedTask;
        }

        /// <summary>
        /// 写入高信号（true）
        /// </summary>
        /// <param name="cancellationToken">取消令牌</param>
        /// <returns>异步任务</returns>
        public virtual async Task WriteHighSignalAsync(CancellationToken cancellationToken)
        {
            await Task.CompletedTask;
        }

        /// <summary>
        /// 读取高信号（true）
        /// </summary>
        /// <param name="cancellationToken">取消令牌</param>
        /// <returns>异步任务</returns>
        public virtual async Task ReadHighSignalAsync(CancellationToken cancellationToken)
        {
            await Task.CompletedTask;
        }

        /// <summary>
        /// 写入低信号（false）
        /// </summary>
        /// <param name="cancellationToken">取消令牌</param>
        /// <returns>异步任务</returns>
        public virtual async Task WriteLowSignalAsync(CancellationToken cancellationToken)
        {
            await Task.CompletedTask;
        }

        /// <summary>
        /// 读取低信号（false）
        /// </summary>
        /// <param name="cancellationToken">取消令牌</param>
        /// <returns>异步任务</returns>
        public virtual async Task ReadLowSignalAsync(CancellationToken cancellationToken)
        {
            await Task.CompletedTask;
        }

        /// <summary>
        /// 写入复位值
        /// </summary>
        /// <param name="cancellationToken">取消令牌</param>
        /// <returns>异步任务</returns>
        public virtual async Task WriteResetValueAsync(CancellationToken cancellationToken)
        {
            await Task.CompletedTask;
        }

        #endregion

        #region 抽象方法

        /// <summary>
        /// 执行测试逻辑，由派生类实现
        /// </summary>
        /// <param name="cancellationToken">取消令牌</param>
        /// <returns>异步任务</returns>
        protected abstract Task ExecuteTestAsync(CancellationToken cancellationToken);

        #endregion

        #region 保护方法

        /// <summary>
        /// 检查是否应该暂停，如果是则等待恢复
        /// </summary>
        /// <param name="cancellationToken">取消令牌</param>
        protected async Task CheckAndWaitForResumeAsync(CancellationToken cancellationToken)
        {
            // 等待PauseEvent被设置（非暂停状态）或者取消令牌被触发
            await Task.Run(() =>
            {
                try
                {
                    PauseEvent.Wait(cancellationToken);
                }
                catch (OperationCanceledException)
                {
                    // 捕获取消异常，重新抛出以便调用方知道任务已取消
                    throw;
                }
            });
        }

        #endregion
    }
    /// <summary>
    /// 测试任务状态枚举
    /// </summary>
    public enum TestTaskStatus
    {
        /// <summary>
        /// 已创建但未启动
        /// </summary>
        Created,

        /// <summary>
        /// 正在运行
        /// </summary>
        Running,

        /// <summary>
        /// 已暂停
        /// </summary>
        Paused,

        /// <summary>
        /// 已完成
        /// </summary>
        Completed,

        /// <summary>
        /// 已取消
        /// </summary>
        Cancelled,

        /// <summary>
        /// 执行失败
        /// </summary>
        Failed
    }
} 