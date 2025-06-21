using System;
using System.Collections.Generic;
using System.Linq;
using System.Threading.Tasks;
using System.Windows;
using FatFullVersion.IServices;
using FatFullVersion.Models;

namespace FatFullVersion.Services
{
    /// <summary>
    /// 测试记录服务实现类
    /// 负责测试记录的保存、恢复和管理
    /// </summary>
    public class TestRecordService : ITestRecordService
    {
        private readonly IRepository _repository;

        /// <summary>
        /// 构造函数
        /// </summary>
        /// <param name="repository">数据仓储服务</param>
        public TestRecordService(IRepository repository)
        {
            _repository = repository ?? throw new ArgumentNullException(nameof(repository));
        }

        /// <summary>
        /// 保存测试记录
        /// </summary>
        /// <param name="channelMappings">通道映射数据集合</param>
        /// <param name="testTag">测试标识，如果为null则使用测试记录中的标识</param>
        /// <returns>操作是否成功</returns>
        public async Task<bool> SaveTestRecordsAsync(IEnumerable<ChannelMapping> channelMappings, string testTag = null)
        {
            try
            {
                // 没有记录时返回成功
                if (channelMappings == null || !channelMappings.Any())
                    return true;

                // 如果提供了测试标识，为所有记录设置统一的标识
                var records = channelMappings.ToList();
                if (!string.IsNullOrEmpty(testTag))
                {
                    foreach (var record in records)
                    {
                        record.TestTag = testTag;
                    }
                }

                // 检查所有记录是否都有测试标识
                //if (records.Any(r => string.IsNullOrEmpty(r.TestTag)))
                //{
                //    // 为没有测试标识的记录设置统一的标识
                //    var newTag = GenerateTestTag();
                //    foreach (var record in records.Where(r => string.IsNullOrEmpty(r.TestTag)))
                //    {
                //        record.TestTag = newTag;
                //    }
                //}

                //// 确保所有记录都有唯一Id
                //foreach (var record in records.Where(r => string.IsNullOrEmpty(r.Id)))
                //{
                //    record.Id = Guid.NewGuid().ToString("N");
                //}

                // 保存记录
                //return await _repository.SaveTestRecordsAsync(records);

                return await _repository.SaveTestRecordsWithSqlAsync(records);
            }
            catch (Exception ex)
            {
                MessageBox.Show($"保存测试记录时出错: {ex.Message}", "错误", MessageBoxButton.OK, MessageBoxImage.Error);
                return false;
            }
        }

        /// <summary>
        /// 保存单个测试记录
        /// </summary>
        /// <param name="channelMapping">通道映射数据</param>
        /// <returns>操作是否成功</returns>
        public async Task<bool> SaveTestRecordAsync(ChannelMapping channelMapping)
        {
            try
            {
                if (channelMapping == null)
                    return false;

                // 如果没有测试标识，生成一个新的
                //if (string.IsNullOrEmpty(channelMapping.TestTag))
                //{
                //    channelMapping.TestTag = GenerateTestTag();
                //}

                // 确保有唯一Id
                //if (string.IsNullOrEmpty(channelMapping.Id))
                //{
                //    channelMapping.Id = Guid.NewGuid().ToString("N");
                //}

                return await _repository.SaveTestRecordWithSqlAsync(channelMapping);
            }
            catch (Exception ex)
            {
                MessageBox.Show($"保存单个测试记录时出错: {ex.Message}", "错误", MessageBoxButton.OK, MessageBoxImage.Error);
                return false;
            }
        }

        /// <summary>
        /// 恢复指定测试标识的测试记录
        /// </summary>
        /// <param name="testTag">测试标识</param>
        /// <returns>恢复的测试记录集合</returns>
        public async Task<List<ChannelMapping>> RestoreTestRecordsAsync(string testTag)
        {
            try
            {
                if (string.IsNullOrEmpty(testTag))
                    return new List<ChannelMapping>();

                return await _repository.GetTestRecordsByTagAsync(testTag);
            }
            catch (Exception ex)
            {
                MessageBox.Show($"恢复测试记录时出错: {ex.Message}", "错误", MessageBoxButton.OK, MessageBoxImage.Error);
                return new List<ChannelMapping>();
            }
        }

        /// <summary>
        /// 获取所有测试批次标识及其信息
        /// </summary>
        /// <returns>测试批次信息列表</returns>
        public async Task<List<TestBatchInfo>> GetAllTestBatchesAsync()
        {
            try
            {
                // 获取所有测试标识
                var testTags = await _repository.GetAllTestTagsAsync();
                var result = new List<TestBatchInfo>();

                // 获取每个测试批次的详细信息
                foreach (var tag in testTags)
                {
                    var records = await _repository.GetTestRecordsByTagAsync(tag);
                    if (!records.Any()) continue;

                    var batchInfo = new TestBatchInfo
                    {
                        TestTag = tag,
                        CreatedTime = records.Min(r => r.CreatedTime),
                        LastUpdatedTime = records.Max(r => r.UpdatedTime),
                        TotalCount = records.Count,
                        TestedCount = records.Count(r => r.TestResultStatus > 0),
                        PassedCount = records.Count(r => r.TestResultStatus == 1),
                        FailedCount = records.Count(r => r.TestResultStatus == 2)
                    };

                    result.Add(batchInfo);
                }

                return result;
            }
            catch (Exception ex)
            {
                MessageBox.Show($"获取测试批次信息时出错: {ex.Message}", "错误", MessageBoxButton.OK, MessageBoxImage.Error);
                return new List<TestBatchInfo>();
            }
        }

        /// <summary>
        /// 删除测试批次
        /// </summary>
        /// <param name="testTag">测试标识</param>
        /// <returns>操作是否成功</returns>
        public async Task<bool> DeleteTestBatchAsync(string testTag)
        {
            try
            {
                if (string.IsNullOrEmpty(testTag))
                    return false;

                return await _repository.DeleteTestRecordsByTagAsync(testTag);
            }
            catch (Exception ex)
            {
                MessageBox.Show($"删除测试批次时出错: {ex.Message}", "错误", MessageBoxButton.OK, MessageBoxImage.Error);
                return false;
            }
        }
    }
} 