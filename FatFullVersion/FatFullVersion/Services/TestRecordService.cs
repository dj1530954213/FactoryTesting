using System;
using System.Collections.Generic;
using System.Linq;
using System.Threading.Tasks;
using System.Windows;
using FatFullVersion.IServices;
using FatFullVersion.Models;
using FatFullVersion.Shared;
using System.Collections.Concurrent;
using System.Threading;

namespace FatFullVersion.Services
{
    /// <summary>
    /// 测试记录服务实现类
    /// 负责测试记录的保存、恢复和管理
    /// </summary>
    public class TestRecordService : ITestRecordService, IDisposable
    {
        private readonly IRepository _repository;
        
        // 异步保存队列相关
        private readonly ConcurrentQueue<ChannelMapping> _saveQueue;
        private readonly SemaphoreSlim _saveQueueSemaphore;
        private readonly CancellationTokenSource _cancellationTokenSource;
        private readonly Task _saveQueueProcessor;

        /// <summary>
        /// 构造函数
        /// </summary>
        /// <param name="repository">数据仓储服务</param>
        public TestRecordService(IRepository repository)
        {
            _repository = repository ?? throw new ArgumentNullException(nameof(repository));
            
            // 初始化异步保存队列
            _saveQueue = new ConcurrentQueue<ChannelMapping>();
            _saveQueueSemaphore = new SemaphoreSlim(0);
            _cancellationTokenSource = new CancellationTokenSource();
            
            // 启动后台保存处理器
            _saveQueueProcessor = Task.Run(SaveQueueProcessorAsync);
        }

        /// <summary>
        /// 异步保存队列处理器（后台运行）
        /// </summary>
        private async Task SaveQueueProcessorAsync()
        {
            var batchSaveInterval = TimeSpan.FromMilliseconds(500); // 500ms批处理间隔
            var maxBatchSize = 10; // 最大批处理数量
            
            while (!_cancellationTokenSource.Token.IsCancellationRequested)
            {
                try
                {
                    // 等待队列有数据或超时
                    await _saveQueueSemaphore.WaitAsync(_cancellationTokenSource.Token);
                    
                    var recordsToSave = new List<ChannelMapping>();
                    
                    // 收集批处理的记录
                    while (recordsToSave.Count < maxBatchSize && _saveQueue.TryDequeue(out var record))
                    {
                        recordsToSave.Add(record);
                    }
                    
                    // 等待一小段时间，收集更多记录进行批处理
                    if (recordsToSave.Count < maxBatchSize)
                    {
                        await Task.Delay(batchSaveInterval, _cancellationTokenSource.Token);
                        
                        // 再次收集记录
                        while (recordsToSave.Count < maxBatchSize && _saveQueue.TryDequeue(out var record))
                        {
                            recordsToSave.Add(record);
                        }
                    }
                    
                    // 批量保存记录
                    if (recordsToSave.Any())
                    {
                        await _repository.SaveTestRecordsAsync(recordsToSave);
                        System.Diagnostics.Debug.WriteLine($"异步队列批量保存了 {recordsToSave.Count} 条记录");
                    }
                }
                catch (OperationCanceledException)
                {
                    // 正常的取消操作，退出循环
                    break;
                }
                catch (Exception ex)
                {
                    System.Diagnostics.Debug.WriteLine($"异步保存队列处理出错: {ex.Message}");
                    // 继续处理，不中断队列
                }
            }
        }

        /// <summary>
        /// 异步保存单个通道测试记录（用于避免并发锁竞争）
        /// </summary>
        /// <param name="channelMapping">通道映射数据</param>
        /// <returns>操作是否成功</returns>
        public async Task<bool> SaveTestRecordAsyncQueued(ChannelMapping channelMapping)
        {
            try
            {
                if (channelMapping == null)
                    return false;

                // 添加到异步保存队列
                _saveQueue.Enqueue(channelMapping);
                _saveQueueSemaphore.Release();
                
                System.Diagnostics.Debug.WriteLine($"通道 {channelMapping.VariableName} 已加入异步保存队列");
                return true;
            }
            catch (Exception ex)
            {
                System.Diagnostics.Debug.WriteLine($"添加到异步保存队列失败: {ex.Message}");
                return false;
            }
        }

        /// <summary>
        /// 保存测试记录 - 通用方法
        /// </summary>
        /// <param name="channelMappings">通道映射数据集合</param>
        /// <param name="testTag">测试标识，如果为null则使用测试记录中的标识</param>
        /// <returns>操作是否成功</returns>
        public async Task<bool> SaveTestRecordsAsync(IEnumerable<ChannelMapping> channelMappings, string testTag = null)
        {
            try
            {
                if (channelMappings == null || !channelMappings.Any())
                {
                    System.Diagnostics.Debug.WriteLine("SaveTestRecordsAsync: 通道映射集合为空，跳过保存。");
                    return true;
                }

                var mappingsList = channelMappings.ToList();
                System.Diagnostics.Debug.WriteLine($"SaveTestRecordsAsync: 开始保存 {mappingsList.Count} 条测试记录");

                // 直接传递ChannelMapping对象，让Repository处理转换
                bool success = await _repository.SaveTestRecordsAsync(mappingsList);
                
                if (success)
                {
                    System.Diagnostics.Debug.WriteLine($"SaveTestRecordsAsync: 成功保存 {mappingsList.Count} 条测试记录");
                }
                else
                {
                    System.Diagnostics.Debug.WriteLine($"SaveTestRecordsAsync: 保存 {mappingsList.Count} 条测试记录失败");
                }

                return success;
            }
            catch (Exception ex)
            {
                System.Diagnostics.Debug.WriteLine($"SaveTestRecordsAsync: 保存测试记录时发生异常 - {ex.Message}");
                return false;
            }
        }

        /// <summary>
        /// 保存单个测试记录 - 手动测试场景
        /// </summary>
        /// <param name="channelMapping">通道映射数据</param>
        /// <returns>操作是否成功</returns>
        public async Task<bool> SaveTestRecordAsync(ChannelMapping channelMapping)
        {
            if (channelMapping == null)
            {
                System.Diagnostics.Debug.WriteLine("SaveTestRecordAsync: 通道映射对象为空，跳过保存。");
                return false;
            }

            System.Diagnostics.Debug.WriteLine($"SaveTestRecordAsync: 开始保存单个测试记录 - {channelMapping.VariableName}");

            try
            {
                // 直接传递ChannelMapping对象，让Repository处理转换
                bool success = await _repository.SaveTestRecordAsync(channelMapping);
                
                if (success)
                {
                    System.Diagnostics.Debug.WriteLine($"SaveTestRecordAsync: 成功保存单个测试记录 - {channelMapping.VariableName}");
                }
                else
                {
                    System.Diagnostics.Debug.WriteLine($"SaveTestRecordAsync: 保存单个测试记录失败 - {channelMapping.VariableName}");
                }

                return success;
            }
            catch (Exception ex)
            {
                System.Diagnostics.Debug.WriteLine($"SaveTestRecordAsync: 保存单个测试记录时发生异常 - {ex.Message}");
                return false;
            }
        }

        /// <summary>
        /// 【关键节点1】硬点自动测试完成批量存储
        /// </summary>
        /// <param name="channelMappings">所有需要保存的通道集合（包括测试和未测试的）</param>
        /// <param name="batchName">批次名称</param>
        /// <returns>操作是否成功</returns>
        public async Task<bool> SaveHardPointTestResultsAsync(IEnumerable<ChannelMapping> channelMappings, string batchName)
        {
            try
            {
                if (channelMappings == null || !channelMappings.Any())
                {
                    System.Diagnostics.Debug.WriteLine("SaveHardPointTestResultsAsync: 通道集合为空，跳过保存。");
                    return true;
                }

                var mappingsList = channelMappings.ToList();
                System.Diagnostics.Debug.WriteLine($"SaveHardPointTestResultsAsync: 开始批量保存硬点测试结果，共 {mappingsList.Count} 个通道，批次：{batchName}");

                // 使用Repository的批量保存方法
                bool success = await _repository.SaveHardPointTestResultsAsync(mappingsList);
                
                if (success)
                {
                    System.Diagnostics.Debug.WriteLine($"SaveHardPointTestResultsAsync: 成功批量保存 {mappingsList.Count} 条硬点测试记录");
                }
                else
                {
                    System.Diagnostics.Debug.WriteLine($"SaveHardPointTestResultsAsync: 批量保存 {mappingsList.Count} 条硬点测试记录失败");
                }

                return success;
            }
            catch (Exception ex)
            {
                System.Diagnostics.Debug.WriteLine($"SaveHardPointTestResultsAsync: 批量保存硬点测试结果时发生异常 - {ex.Message}");
                return false;
            }
        }

        /// <summary>
        /// 更新单个通道的复测结果 - 复测场景优化
        /// </summary>
        /// <param name="channelMapping">通道映射数据</param>
        /// <returns>操作是否成功</returns>
        public async Task<bool> UpdateRetestResultAsync(ChannelMapping channelMapping)
        {
            try
            {
                if (channelMapping == null)
                {
                    System.Diagnostics.Debug.WriteLine("UpdateRetestResultAsync: 通道映射对象为空，跳过更新。");
                    return false;
                }

                System.Diagnostics.Debug.WriteLine($"UpdateRetestResultAsync: 开始更新复测结果 - {channelMapping.VariableName}");

                // 使用Repository的复测更新方法
                bool success = await _repository.UpdateRetestResultAsync(channelMapping);
                
                if (success)
                {
                    System.Diagnostics.Debug.WriteLine($"UpdateRetestResultAsync: 成功更新复测结果 - {channelMapping.VariableName}");
                }
                else
                {
                    System.Diagnostics.Debug.WriteLine($"UpdateRetestResultAsync: 更新复测结果失败 - {channelMapping.VariableName}");
                }

                return success;
            }
            catch (Exception ex)
            {
                System.Diagnostics.Debug.WriteLine($"UpdateRetestResultAsync: 更新复测结果时发生异常 - {ex.Message}");
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
                        LastUpdatedTime = records.Max(r => r.UpdatedTime) ?? DateTime.Now,
                        TotalCount = records.Count,
                        TestedCount = records.Count(r => r.OverallStatus != OverallResultStatus.NotTested && r.OverallStatus != OverallResultStatus.InProgress),
                        PassedCount = records.Count(r => r.OverallStatus == OverallResultStatus.Passed),
                        FailedCount = records.Count(r => r.OverallStatus == OverallResultStatus.Failed)
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

        /// <summary>
        /// 释放资源
        /// </summary>
        public void Dispose()
        {
            _cancellationTokenSource?.Cancel();
            
            try
            {
                _saveQueueProcessor?.Wait(5000); // 等待5秒钟让队列处理完成
            }
            catch (Exception ex)
            {
                System.Diagnostics.Debug.WriteLine($"等待保存队列完成时出错: {ex.Message}");
            }
            
            _saveQueueSemaphore?.Dispose();
            _cancellationTokenSource?.Dispose();
        }

        /// <summary>
        /// 将NaN值转换为null以便存储到数据库
        /// </summary>
        /// <param name="value">原始值</param>
        /// <returns>处理后的值</returns>
        private double? ProcessNanValuesForStorage(double value)
        {
            return double.IsNaN(value) ? null : value;
        }
    }
} 