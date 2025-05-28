using FatFullVersion.Models;
using System;
using System.Collections.Generic;
using System.Threading.Tasks;

namespace FatFullVersion.IServices
{
    /// <summary>
    /// 测试记录服务接口
    /// 负责测试记录的保存、恢复和管理
    /// </summary>
    public interface ITestRecordService
    {
        /// <summary>
        /// 保存测试记录 - 通用方法
        /// </summary>
        /// <param name="channelMappings">通道映射数据集合</param>
        /// <param name="testTag">测试标识，如果为null则使用测试记录中的标识</param>
        /// <returns>操作是否成功</returns>
        Task<bool> SaveTestRecordsAsync(IEnumerable<ChannelMapping> channelMappings, string testTag = null);

        /// <summary>
        /// 保存单个测试记录 - 手动测试场景
        /// </summary>
        /// <param name="channelMapping">通道映射数据</param>
        /// <returns>操作是否成功</returns>
        Task<bool> SaveTestRecordAsync(ChannelMapping channelMapping);

        /// <summary>
        /// 批量保存硬点自动测试完成的记录 - 新增优化方法
        /// </summary>
        /// <param name="channelMappings">通道映射数据集合</param>
        /// <param name="testTag">测试标识</param>
        /// <returns>操作是否成功</returns>
        Task<bool> SaveHardPointTestResultsAsync(IEnumerable<ChannelMapping> channelMappings, string testTag = null);

        /// <summary>
        /// 更新单个通道的复测结果 - 复测场景优化
        /// </summary>
        /// <param name="channelMapping">通道映射数据</param>
        /// <returns>操作是否成功</returns>
        Task<bool> UpdateRetestResultAsync(ChannelMapping channelMapping);

        /// <summary>
        /// 恢复指定测试标识的测试记录
        /// </summary>
        /// <param name="testTag">测试标识</param>
        /// <returns>恢复的测试记录集合</returns>
        Task<List<ChannelMapping>> RestoreTestRecordsAsync(string testTag);

        /// <summary>
        /// 获取所有测试批次标识及其信息
        /// </summary>
        /// <returns>测试批次信息列表</returns>
        Task<List<TestBatchInfo>> GetAllTestBatchesAsync();

        /// <summary>
        /// 删除测试批次
        /// </summary>
        /// <param name="testTag">测试标识</param>
        /// <returns>操作是否成功</returns>
        Task<bool> DeleteTestBatchAsync(string testTag);

        /// <summary>
        /// 异步保存单个通道测试记录（用于避免并发锁竞争）
        /// </summary>
        /// <param name="channelMapping">通道映射数据</param>
        /// <returns>操作是否成功</returns>
        Task<bool> SaveTestRecordAsyncQueued(ChannelMapping channelMapping);
    }
} 