using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Threading;
using System.Threading.Tasks;
using FatFullVersion.Entities;
using FatFullVersion.IServices;

namespace FatFullVersion.Models
{
    /// <summary>
    /// 测试任务基类，定义测试任务的通用行为和属性
    /// </summary>
    public abstract class TestTask : IDisposable, FatFullVersion.IServices.ITestPausable
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
        /// 用于存储测试过程中收集的测量数据（如ValueXPercent）和详细日志。
        /// 注意：此对象的Status属性不应再用于驱动最终的硬点测试结果，该职责已移交。
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
        /// 启动并执行测试任务的核心逻辑，并返回原始测试结果。
        /// 此方法由 TestTaskManager 调用。
        /// </summary>
        /// <param name="cancellationToken">外部取消令牌。</param>
        /// <returns>包含测试原始结果的 HardPointTestRawResult。</returns>
        public async Task<HardPointTestRawResult> RunTestAsync(CancellationToken cancellationToken)
        {
            HardPointTestRawResult testOutcome;
            lock (TaskLock)
            {
                if (Status == TestTaskStatus.Running && !IsPaused) // 防止重入，除非是暂停后恢复
                {
                    return new HardPointTestRawResult(false, "任务已在运行中。");
                }
                if (IsCompleted || IsCancelled) 
                {
                     return new HardPointTestRawResult(false, $"任务已完成({Status})，无法再次运行。");
                }

                // 合并外部令牌与内部令牌，如果需要单独取消此任务，应使用CancellationTokenSource
                // 但通常 _masterCancellationTokenSource 由 TestTaskManager 控制全局取消
                // 此处cancellationToken主要用于传递 TestTaskManager 的全局取消信号
                Status = TestTaskStatus.Running;
                IsPaused = false;
                IsCancelled = false; // 重置取消状态
                IsCompleted = false; // 重置完成状态
                PauseEvent.Set(); // 确保开始时不是暂停状态
            }

            try
            {
                testOutcome = await ExecuteTestAsync(cancellationToken); // 调用子类实现的具体测试逻辑

                lock (TaskLock)
                {
                    if (!IsCancelled) // 仅当未被外部取消时，才根据测试结果更新状态
                    {
                        Status = testOutcome.IsSuccess ? TestTaskStatus.Completed : TestTaskStatus.Failed;
                        IsCompleted = true; 
                    }
                }
            }
            catch (OperationCanceledException)
            {
                lock (TaskLock)
                {
                    Status = TestTaskStatus.Cancelled;
                    IsCancelled = true;
                }
                testOutcome = new HardPointTestRawResult(false, "测试任务执行被取消。");
            }
            catch (Exception ex)
            {
                lock (TaskLock)
                {
                    Status = TestTaskStatus.Failed;
                    IsCompleted = true; // 即使失败也标记为完成
                }
                testOutcome = new HardPointTestRawResult(false, $"测试任务执行时发生未知异常: {ex.Message}");
            }
            return testOutcome;
        }

        /// <summary>
        /// 停止测试任务。
        /// </summary>
        public virtual async Task StopAsync()
        {
            bool wasCancelled = false;
            lock (TaskLock)
            {
                if (Status == TestTaskStatus.Completed || Status == TestTaskStatus.Cancelled)
                    return;
                
                if(!CancellationTokenSource.IsCancellationRequested) CancellationTokenSource.Cancel();
                PauseEvent.Set(); 
                
                Status = TestTaskStatus.Cancelled;
                IsCancelled = true;
                wasCancelled = true;
            }
            if (wasCancelled) await Task.Delay(50); // 短暂等待异步操作响应取消
        }

        /// <summary>
        /// 暂停测试任务的执行。
        /// </summary>
        public virtual Task PauseAsync() // 实现 IPausableTask
        {
            lock (TaskLock)
            {
                if (Status == TestTaskStatus.Running && !IsPaused)
                {
                    PauseEvent.Reset();
                    Status = TestTaskStatus.Paused;
                    IsPaused = true;
                }
            }
            return Task.CompletedTask;
        }

        /// <summary>
        /// 恢复已暂停的测试任务的执行。
        /// </summary>
        public virtual Task ResumeAsync() // 实现 IPausableTask
        {
            lock (TaskLock)
            {
                if (Status == TestTaskStatus.Paused && IsPaused)
                {
                    PauseEvent.Set();
                    Status = TestTaskStatus.Running;
                    IsPaused = false;
                }
            }
            return Task.CompletedTask;
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

        #region 抽象方法

        /// <summary>
        /// 子类必须实现此方法以定义具体的测试执行逻辑，并返回原始测试结果。
        /// </summary>
        /// <param name="cancellationToken">取消令牌。</param>
        /// <returns>包含测试原始结果的 HardPointTestRawResult。</returns>
        protected abstract Task<HardPointTestRawResult> ExecuteTestAsync(CancellationToken cancellationToken);

        #endregion

        #region 保护方法

        /// <summary>
        /// 在测试步骤中检查是否应该暂停，如果是则等待恢复信号。
        /// </summary>
        /// <param name="cancellationToken">外部取消令牌，如果触发则中断等待。</param>
        protected async Task CheckAndWaitForResumeAsync(CancellationToken cancellationToken)
        {
            await Task.Run(() =>
            {
                try
                {
                    // WaitAny 等待暂停事件被Set，或者外部取消信号
                    WaitHandle.WaitAny(new[] { PauseEvent.WaitHandle, cancellationToken.WaitHandle });
                    cancellationToken.ThrowIfCancellationRequested(); // 如果是由于取消而结束等待，则抛出
                }
                catch (OperationCanceledException)
                {
                    // 此处捕获的取消是方法级别的，重新抛出以通知调用者
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