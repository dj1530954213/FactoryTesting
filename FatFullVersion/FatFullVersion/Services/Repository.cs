using System;
using System.Collections.Generic;
using System.Linq;
using System.Threading.Tasks;
using System.Windows;
using FatFullVersion.Data;
using FatFullVersion.Entities;
using FatFullVersion.Entities.EntitiesEnum;
using FatFullVersion.Entities.ValueObject;
using FatFullVersion.IServices;
using FatFullVersion.Models;
using Microsoft.EntityFrameworkCore;

namespace FatFullVersion.Services
{
    /// <summary>
    /// 数据访问仓储类
    /// 提供对数据库的CRUD操作
    /// </summary>
    public class Repository : IRepository
    {
        private readonly ApplicationDbContext _context;

        public Repository(ApplicationDbContext context)
        {
            _context = context;
            _context.Database.EnsureCreated(); // 确保数据库和表已创建
        }

        public async Task<bool> InitializeDatabaseAsync()
        {
            try
            {
                return await _context.Database.EnsureCreatedAsync();
            }
            catch
            {
                return false;
            }
        }

        #region PLC连接配置操作 - 保持不变

        public async Task<PlcConnectionConfig> GetTestPlcConnectionConfigAsync()
        {
            return await _context.PlcConnections.FirstOrDefaultAsync(p => p.IsTestPlc) ?? new();
        }

        public async Task<PlcConnectionConfig> GetTargetPlcConnectionConfigAsync()
        {
            return await _context.PlcConnections.FirstOrDefaultAsync(p => !p.IsTestPlc) ?? new();
        }

        public async Task<bool> SavePlcConnectionConfigAsync(PlcConnectionConfig config)
        {
            try
            {
                var existing = await _context.PlcConnections.FindAsync(config.Id);
                if (existing == null)
                    _context.PlcConnections.Add(config);
                else
                    _context.Entry(existing).CurrentValues.SetValues(config);

                return await _context.SaveChangesAsync() > 0;
            }
            catch (Exception ex)
            {
                MessageBox.Show($"保存数据出现错误:{ex.Message}");
                return false;
            }
        }

        public async Task<List<PlcConnectionConfig>> GetAllPlcConnectionConfigsAsync()
        {
            return await _context.PlcConnections.ToListAsync();
        }

        #endregion

        #region 通道比较表操作 - 保持不变

        public async Task<string> GetPlcCommunicationAddress(string channelTag)
        {
            return (await _context.ComparisonTables
                .FirstOrDefaultAsync(c => c.ChannelAddress == channelTag))?.CommunicationAddress ?? string.Empty;
        }

        public async Task<List<ComparisonTable>> GetComparisonTablesAsync()
        {
            return await _context.ComparisonTables.ToListAsync();
        }

        public async Task<bool> AddComparisonTableAsync(ComparisonTable comparisonTable)
        {
            _context.ComparisonTables.Add(comparisonTable);
            return await _context.SaveChangesAsync() > 0;
        }

        public async Task<bool> AddComparisonTablesAsync(List<ComparisonTable> comparisonTables)
        {
            _context.ComparisonTables.AddRange(comparisonTables);
            return await _context.SaveChangesAsync() > 0;
        }

        public async Task<bool> UpdateComparisonTableAsync(ComparisonTable comparisonTable)
        {
            var existing = await _context.ComparisonTables.FindAsync(comparisonTable.Id);
            if (existing == null) return false;

            _context.Entry(existing).CurrentValues.SetValues(comparisonTable);
            return await _context.SaveChangesAsync() > 0;
        }

        public async Task<bool> DeleteComparisonTableAsync(int id)
        {
            var item = await _context.ComparisonTables.FindAsync(id);
            if (item == null) return false;

            _context.ComparisonTables.Remove(item);
            return await _context.SaveChangesAsync() > 0;
        }

        public async Task<bool> SaveAllComparisonTablesAsync(List<ComparisonTable> comparisonTables)
        {
            try
            {
                // 获取数据库中所有的记录
                var existingTables = await _context.ComparisonTables.ToListAsync();

                // 删除不存在于 comparisonTables 中的记录
                var tablesToRemove = existingTables.Where(existingTable => !comparisonTables.Any(newTable => newTable.Id == existingTable.Id)).ToList();
                _context.ComparisonTables.RemoveRange(tablesToRemove);

                // 更新已有的记录和插入新记录
                foreach (var comparisonTable in comparisonTables)
                {
                    var existingTable = existingTables.FirstOrDefault(x => x.Id == comparisonTable.Id);
                    if (existingTable != null)
                    {
                        // 如果表格已经存在，更新它
                        _context.Entry(existingTable).CurrentValues.SetValues(comparisonTable);
                    }
                    else
                    {
                        // 如果表格不存在，插入新记录
                        _context.ComparisonTables.Add(comparisonTable);
                    }
                }

                // 保存更改
                return await _context.SaveChangesAsync() >= 0;
            }
            catch (Exception)
            {
                return false;
            }
        }

        #endregion

        #region 测试记录操作 - 重构简化

        /// <summary>
        /// 保存测试记录集合 - 使用EF Core批量操作优化
        /// </summary>
        /// <param name="records">测试记录集合</param>
        /// <returns>保存操作是否成功</returns>
        public async Task<bool> SaveTestRecordsAsync(IEnumerable<ChannelMapping> records)
        {
            try
            {
                if (records == null || !records.Any())
                    return true;

                var recordsList = records.ToList();
                
                // 批量处理：分别处理新增和更新
                var existingIds = recordsList.Where(r => r.Id != Guid.Empty).Select(r => r.Id).ToList();
                var existingRecords = await _context.ChannelMappings
                    .Where(c => existingIds.Contains(c.Id))
                    .ToListAsync();

                foreach (var record in recordsList)
                {
                    // 确保每条记录都有Guid作为Id
                    if (record.Id == Guid.Empty)
                    {
                        record.Id = Guid.NewGuid();
                    }

                    // 更新时间戳
                    record.UpdatedTime = DateTime.Now;

                    // 处理NaN值：直接转换为null，无需复杂转换
                    ProcessNanValuesForStorage(record);

                    // 检查记录是否已存在
                    var existing = existingRecords.FirstOrDefault(e => e.Id == record.Id);
                    if (existing != null)
                    {
                        // 更新现有记录
                        _context.Entry(existing).CurrentValues.SetValues(record);
                    }
                    else
                    {
                        // 添加新记录
                        _context.ChannelMappings.Add(record);
                    }
                }

                return await _context.SaveChangesAsync() > 0;
            }
            catch (Exception ex)
            {
                MessageBox.Show($"保存测试记录时出错: {ex.Message}", "错误", MessageBoxButton.OK, MessageBoxImage.Error);
                return false;
            }
        }

        /// <summary>
        /// 保存单个测试记录 - 手动测试场景优化
        /// </summary>
        /// <param name="record">测试记录</param>
        /// <returns>保存操作是否成功</returns>
        public async Task<bool> SaveTestRecordAsync(ChannelMapping record)
        {
            try
            {
                if (record == null)
                    return false;
                
                // 确保记录有Guid作为Id
                if (record.Id == Guid.Empty)
                {
                    record.Id = Guid.NewGuid();
                }
                
                // 更新时间戳
                record.UpdatedTime = DateTime.Now;
                
                // 处理NaN值
                ProcessNanValuesForStorage(record);
                
                // 检查记录是否已存在
                var existing = await _context.ChannelMappings.FindAsync(record.Id);
                if (existing != null)
                {
                    // 更新现有记录
                    _context.Entry(existing).CurrentValues.SetValues(record);
                }
                else
                {
                    // 添加新记录
                    _context.ChannelMappings.Add(record);
                }
                
                return await _context.SaveChangesAsync() > 0;
            }
            catch (Exception ex)
            {
                MessageBox.Show($"保存单个测试记录时出错: {ex.Message}", "错误", MessageBoxButton.OK, MessageBoxImage.Error);
                return false;
            }
        }

        /// <summary>
        /// 批量保存硬点自动测试完成的记录 - 新增优化方法
        /// </summary>
        /// <param name="records">测试记录集合</param>
        /// <returns>保存操作是否成功</returns>
        public async Task<bool> SaveHardPointTestResultsAsync(IEnumerable<ChannelMapping> records)
        {
            try
            {
                if (records == null || !records.Any())
                    return true;

                var recordsList = records.ToList();
                var recordIds = recordsList.Select(r => r.Id).ToList();
                
                // 一次性批量查询所有需要更新的记录，避免循环中的数据库查询
                var existingRecords = await _context.ChannelMappings
                    .Where(c => recordIds.Contains(c.Id))
                    .ToListAsync();
                
                // 使用字典来优化查找性能
                var existingDict = existingRecords.ToDictionary(r => r.Id);
                
                foreach (var record in recordsList)
                {
                    record.UpdatedTime = DateTime.Now;
                    ProcessNanValuesForStorage(record);
                    
                    if (existingDict.TryGetValue(record.Id, out var existing))
                    {
                        // 只更新测试结果相关字段，提高性能
                        existing.HardPointTestResult = record.HardPointTestResult;
                        existing.TestResultStatus = record.TestResultStatus;
                        existing.ResultText = record.ResultText;
                        existing.FinalTestTime = record.FinalTestTime;
                        existing.TestTime = record.TestTime;
                        existing.StartTime = record.StartTime;
                        existing.UpdatedTime = record.UpdatedTime;
                        
                        // 更新测试数据字段
                        existing.ExpectedValue = record.ExpectedValue;
                        existing.ActualValue = record.ActualValue;
                        existing.Value0Percent = record.Value0Percent;
                        existing.Value25Percent = record.Value25Percent;
                        existing.Value50Percent = record.Value50Percent;
                        existing.Value75Percent = record.Value75Percent;
                        existing.Value100Percent = record.Value100Percent;
                    }
                    else
                    {
                        // 如果记录不存在，添加新记录
                        _context.ChannelMappings.Add(record);
                    }
                }

                return await _context.SaveChangesAsync() > 0;
            }
            catch (Exception ex)
            {
                MessageBox.Show($"批量保存硬点测试结果时出错: {ex.Message}", "错误", MessageBoxButton.OK, MessageBoxImage.Error);
                return false;
            }
        }

        /// <summary>
        /// 更新单个通道的复测结果 - 复测场景优化
        /// </summary>
        /// <param name="record">测试记录</param>
        /// <returns>保存操作是否成功</returns>
        public async Task<bool> UpdateRetestResultAsync(ChannelMapping record)
        {
            try
            {
                if (record == null)
                    return false;
                
                record.UpdatedTime = DateTime.Now;
                ProcessNanValuesForStorage(record);
                
                var existing = await _context.ChannelMappings.FindAsync(record.Id);
                if (existing != null)
                {
                    // 复测场景：更新所有测试相关字段
                    existing.HardPointTestResult = record.HardPointTestResult;
                    existing.TestResultStatus = record.TestResultStatus;
                    existing.ResultText = record.ResultText;
                    existing.FinalTestTime = record.FinalTestTime;
                    existing.TestTime = record.TestTime;
                    existing.StartTime = record.StartTime;
                    existing.UpdatedTime = record.UpdatedTime;
                    
                    // 更新手动测试状态
                    existing.ShowValueStatus = record.ShowValueStatus;
                    existing.LowLowAlarmStatus = record.LowLowAlarmStatus;
                    existing.LowAlarmStatus = record.LowAlarmStatus;
                    existing.HighAlarmStatus = record.HighAlarmStatus;
                    existing.HighHighAlarmStatus = record.HighHighAlarmStatus;
                    existing.AlarmValueSetStatus = record.AlarmValueSetStatus;
                    existing.MaintenanceFunction = record.MaintenanceFunction;
                    existing.TrendCheck = record.TrendCheck;
                    existing.ReportCheck = record.ReportCheck;
                    
                    // 更新测试数据
                    existing.ExpectedValue = record.ExpectedValue;
                    existing.ActualValue = record.ActualValue;
                    existing.Value0Percent = record.Value0Percent;
                    existing.Value25Percent = record.Value25Percent;
                    existing.Value50Percent = record.Value50Percent;
                    existing.Value75Percent = record.Value75Percent;
                    existing.Value100Percent = record.Value100Percent;
                    
                    return await _context.SaveChangesAsync() > 0;
                }
                
                return false;
            }
            catch (Exception ex)
            {
                MessageBox.Show($"更新复测结果时出错: {ex.Message}", "错误", MessageBoxButton.OK, MessageBoxImage.Error);
                return false;
            }
        }
        
        /// <summary>
        /// 根据测试标识获取测试记录
        /// </summary>
        /// <param name="testTag">测试标识</param>
        /// <returns>测试记录集合</returns>
        public async Task<List<ChannelMapping>> GetTestRecordsByTagAsync(string testTag)
        {
            try
            {
                if (string.IsNullOrEmpty(testTag))
                    return new List<ChannelMapping>();
                
                var records = await _context.ChannelMappings
                    .Where(c => c.TestTag == testTag)
                    .ToListAsync();
                
                return records;
            }
            catch (Exception ex)
            {
                MessageBox.Show($"获取测试记录时出错: {ex.Message}", "错误", MessageBoxButton.OK, MessageBoxImage.Error);
                return new List<ChannelMapping>();
            }
        }
        
        /// <summary>
        /// 获取所有不同的测试标识
        /// </summary>
        /// <returns>测试标识集合</returns>
        public async Task<List<string>> GetAllTestTagsAsync()
        {
            try
            {
                return await _context.ChannelMappings
                    .Where(c => !string.IsNullOrEmpty(c.TestTag))
                    .Select(c => c.TestTag)
                    .Distinct()
                    .OrderByDescending(tag => tag) // 按照标签降序排序(通常新的测试记录标签更大)
                    .ToListAsync();
            }
            catch (Exception ex)
            {
                MessageBox.Show($"获取测试标识时出错: {ex.Message}", "错误", MessageBoxButton.OK, MessageBoxImage.Error);
                return new List<string>();
            }
        }
        
        /// <summary>
        /// 根据测试标识删除测试记录
        /// </summary>
        /// <param name="testTag">测试标识</param>
        /// <returns>删除操作是否成功</returns>
        public async Task<bool> DeleteTestRecordsByTagAsync(string testTag)
        {
            try
            {
                if (string.IsNullOrEmpty(testTag))
                    return false;

                var recordsToDelete = await _context.ChannelMappings
                    .Where(c => c.TestTag == testTag)
                    .ToListAsync();

                if (recordsToDelete.Any())
                {
                    _context.ChannelMappings.RemoveRange(recordsToDelete);
                    return await _context.SaveChangesAsync() > 0;
                }

                return true; // 没有记录需要删除也算成功
            }
            catch (Exception ex)
            {
                MessageBox.Show($"删除测试记录时出错: {ex.Message}", "数据库错误", MessageBoxButton.OK, MessageBoxImage.Error);
                return false;
            }
        }

        /// <summary>
        /// 获取所有测试记录
        /// </summary>
        /// <returns>所有测试记录集合</returns>
        public async Task<List<ChannelMapping>> GetAllTestRecordsAsync()
        {
            try
            {
                var records = await _context.ChannelMappings.ToListAsync();
                return records;
            }
            catch (Exception ex)
            {
                MessageBox.Show($"获取所有测试记录时出错: {ex.Message}", "错误", MessageBoxButton.OK, MessageBoxImage.Error);
                return new List<ChannelMapping>();
            }
        }

        #endregion

        #region 辅助方法

        /// <summary>
        /// 简化的NaN值处理：直接转换为null存储
        /// </summary>
        /// <param name="record">待处理的通道映射对象</param>
        private void ProcessNanValuesForStorage(ChannelMapping record)
        {
            // 处理数值字段的NaN值，转换为null以便存储
            if (record.RangeLowerLimitValue.HasValue && float.IsNaN(record.RangeLowerLimitValue.Value))
                record.RangeLowerLimitValue = null;
                
            if (record.RangeUpperLimitValue.HasValue && float.IsNaN(record.RangeUpperLimitValue.Value))
                record.RangeUpperLimitValue = null;
                
            if (record.SLLSetValueNumber.HasValue && float.IsNaN(record.SLLSetValueNumber.Value))
                record.SLLSetValueNumber = null;
                
            if (record.SLSetValueNumber.HasValue && float.IsNaN(record.SLSetValueNumber.Value))
                record.SLSetValueNumber = null;
                
            if (record.SHSetValueNumber.HasValue && float.IsNaN(record.SHSetValueNumber.Value))
                record.SHSetValueNumber = null;
                
            if (record.SHHSetValueNumber.HasValue && float.IsNaN(record.SHHSetValueNumber.Value))
                record.SHHSetValueNumber = null;
                
            if (record.ExpectedValue.HasValue && float.IsNaN(record.ExpectedValue.Value))
                record.ExpectedValue = null;
                
            if (record.ActualValue.HasValue && float.IsNaN(record.ActualValue.Value))
                record.ActualValue = null;
                
            if (record.Value0Percent.HasValue && float.IsNaN(record.Value0Percent.Value))
                record.Value0Percent = null;
                
            if (record.Value25Percent.HasValue && float.IsNaN(record.Value25Percent.Value))
                record.Value25Percent = null;
                
            if (record.Value50Percent.HasValue && float.IsNaN(record.Value50Percent.Value))
                record.Value50Percent = null;
                
            if (record.Value75Percent.HasValue && float.IsNaN(record.Value75Percent.Value))
                record.Value75Percent = null;
                
            if (record.Value100Percent.HasValue && float.IsNaN(record.Value100Percent.Value))
                record.Value100Percent = null;
        }

        #endregion

        #region 废弃的方法 - 保留接口兼容性

        [Obsolete("已废弃，请使用SaveTestRecordsAsync")]
        public async Task<bool> SaveTestRecordsWithSqlAsync(IEnumerable<ChannelMapping> records)
        {
            return await SaveTestRecordsAsync(records);
        }

        [Obsolete("已废弃，请使用SaveTestRecordAsync")]
        public async Task<bool> SaveTestRecordWithSqlAsync(ChannelMapping record)
        {
            return await SaveTestRecordAsync(record);
        }

        #endregion
    }
}
