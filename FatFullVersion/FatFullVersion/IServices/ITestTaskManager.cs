using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Threading.Tasks;
using FatFullVersion.Models;
// using FatFullVersion.ViewModels; // Add this if ViewModels.BatchInfo is used explicitly

namespace FatFullVersion.IServices
{
    /// <summary>
    /// 测试任务管理器接口，定义测试任务的创建、启动、停止和管理功能
    /// </summary>
    public interface ITestTaskManager : IDisposable
    {
        /// <summary>
        /// 获取接线是否已完成的标志
        /// </summary>
        bool IsWiringCompleted { get; }
        
        /// <summary>
        /// 从通道映射集合创建测试任务
        /// </summary>
        /// <param name="channelMappings">需要测试的通道映射集合</param>
        /// <returns>创建的任务ID列表</returns>
        Task<IEnumerable<string>> CreateTestTasksAsync(IEnumerable<ChannelMapping> channelMappings);

        /// <summary>
        /// 启动所有测试任务
        /// </summary>
        /// <param name="channelsToTest">需要测试的通道映射集合</param>
        /// <returns>操作是否成功</returns>
        Task<bool> StartAllTasksAsync(IEnumerable<ChannelMapping> channelsToTest);

        /// <summary>
        /// 停止所有测试任务
        /// </summary>
        /// <returns>操作是否成功</returns>
        Task<bool> StopAllTasksAsync();

        /// <summary>
        /// 暂停所有测试任务
        /// </summary>
        /// <returns>操作是否成功</returns>
        Task<bool> PauseAllTasksAsync();

        /// <summary>
        /// 恢复所有测试任务
        /// </summary>
        /// <returns>操作是否成功</returns>
        Task<bool> ResumeAllTasksAsync();

        /// <summary>
        /// 根据ID获取测试任务
        /// </summary>
        /// <param name="taskId">任务ID</param>
        /// <returns>测试任务实例，如果不存在则返回null</returns>
        TestTask GetTaskById(string taskId);

        /// <summary>
        /// 根据通道映射获取测试任务
        /// </summary>
        /// <param name="channelMapping">通道映射实例</param>
        /// <returns>测试任务实例，如果不存在则返回null</returns>
        TestTask GetTaskByChannel(ChannelMapping channelMapping);

        /// <summary>
        /// 获取所有活跃的测试任务
        /// </summary>
        /// <returns>所有活跃的测试任务集合</returns>
        IEnumerable<TestTask> GetAllTasks();

        /// <summary>
        /// 删除特定ID的测试任务
        /// </summary>
        /// <param name="taskId">待删除的任务ID</param>
        /// <returns>操作是否成功</returns>
        Task<bool> RemoveTaskAsync(string taskId);

        /// <summary>
        /// 添加新的测试任务
        /// </summary>
        /// <param name="task">要添加的测试任务</param>
        /// <returns>操作是否成功</returns>
        bool AddTask(TestTask task);

        /// <summary>
        /// 清空所有测试任务
        /// </summary>
        /// <returns>操作是否成功</returns>
        Task<bool> ClearAllTasksAsync();

        /// <summary>
        /// 确认接线已完成，启用测试功能
        /// </summary>
        /// <param name="batchInfo">批次信息</param>
        /// <param name="isConfirmed">是否确认完成</param>
        /// <param name="testMap">测试映射集合</param>
        /// <returns>确认操作是否成功</returns>
        Task<bool> ConfirmWiringCompleteAsync(BatchInfo batchInfo, bool isConfirmed, IEnumerable<ChannelMapping> testMap);

        /// <summary>
        /// 显示测试进度对话框
        /// </summary>
        /// <param name="isRetestMode">是否为复测模式，默认为false表示全自动测试</param>
        /// <param name="channelInfo">复测的通道信息（复测模式下使用）</param>
        /// <returns>异步任务</returns>
        Task ShowTestProgressDialogAsync(bool isRetestMode = false, ChannelMapping channelInfo = null);
        
        /// <summary>
        /// 更新批次状态为全部已完成
        /// 只有在所有手动测试完成后才调用此方法
        /// </summary>
        /// <returns>操作是否成功</returns>
        Task<bool> CompleteAllTestsAsync();

        /// <summary>
        /// 对单个通道进行复测
        /// </summary>
        /// <param name="channelToRetest">需要复测的通道映射</param>
        /// <returns>操作是否成功</returns>
        Task<bool> RetestChannelAsync(ChannelMapping channelToRetest);

        /// <summary>
        /// 启动所有测试任务（串行执行方式）
        /// </summary>
        /// <returns>操作是否成功</returns>
        Task<bool> StartAllTasksSerialAsync();

        /// <summary>
        /// 对单个通道进行复测（串行执行方式）
        /// </summary>
        /// <param name="channelMapping">需要复测的通道映射</param>
        /// <returns>操作是否成功</returns>
        Task<bool> RetestChannelSerialAsync(ChannelMapping channelMapping);
    }
}
