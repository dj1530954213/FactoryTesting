using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.Linq;
using System.Reflection;
using System.Text;
using System.Threading.Tasks;
using System.Windows;
using FatFullVersion.Data;
using FatFullVersion.Entities;
using FatFullVersion.Entities.EntitiesEnum;
using FatFullVersion.Entities.ValueObject;
using FatFullVersion.IServices;
using FatFullVersion.Models;
using Microsoft.EntityFrameworkCore;
using Microsoft.EntityFrameworkCore.Infrastructure;
using Microsoft.Extensions.Logging;

namespace FatFullVersion.Services
{
    /// <summary>
    /// 数据访问仓储类
    /// 提供对数据库的CRUD操作
    /// </summary>
    public class Repository : IRepository
    {
        private readonly ApplicationDbContext _context;
        private readonly ILogger<Repository> _logger;

        public Repository(ApplicationDbContext context)
        {
            _context = context;
            _context.Database.EnsureCreated(); // 确保数据库和表已创建
            // 检查连接字符串配置
            var connectionString = _context.Database.GetConnectionString();
            Console.WriteLine($"当前连接字符串: {connectionString}");
            var tableExists = _context.Database.ExecuteSqlRawAsync("SELECT count(*) FROM sqlite_master WHERE type='table' AND name='ChannelMappings'");
            Console.WriteLine($"ChannelMappings表存在: {tableExists.Result > 0}");
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
            catch (Exception e)
            {
                MessageBox.Show($"保存数据出现错误:{e.Message}");
                return false;
            }
        }

        public async Task<List<PlcConnectionConfig>> GetAllPlcConnectionConfigsAsync()
        {
            return await _context.PlcConnections.ToListAsync();
        }

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
                return await _context.SaveChangesAsync()>=0;
            }
            catch (Exception e)
            {
                return false;
            }
        }

        #region 测试记录操作

        /// <summary>
        /// 保存测试记录集合
        /// </summary>
        /// <param name="records">测试记录集合</param>
        /// <returns>保存操作是否成功</returns>
        public async Task<bool> SaveTestRecordsAsync(IEnumerable<ChannelMapping> records)
        {
            try
            {
                if (records == null || !records.Any())
                    return true;

                foreach (var record in records)
                {
                    // 确保每条记录都有Guid作为Id
                    if (record.Id == Guid.Empty)
                    {
                        record.Id = Guid.NewGuid();
                    }

                    // 更新时间戳
                    record.UpdatedTime = DateTime.Now;

                    // 处理可能存在的NaN值，将其转换为null
                    // 检查所有数值类型的属性，替换NaN值
                    ProcessNanValues(record);

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
                }

                return await _context.SaveChangesAsync() > 0;
            }
            catch (Exception ex)
            {
                MessageBox.Show($"保存测试记录时出错: {ex.Message}", "错误", MessageBoxButton.OK, MessageBoxImage.Error);
                return false;
            }
            finally
            {
                foreach (var record in records)
                {
                    RestoreNanValues(record);
                }
            }
        }
        /// <summary>
        /// 保存单个测试记录
        /// </summary>
        /// <param name="record">测试记录</param>
        /// <returns>保存操作是否成功</returns>
        public async Task<bool> SaveTestRecordAsync(ChannelMapping record)
        {
            foreach (var prop in typeof(ChannelMapping).GetProperties())
            {
                var value = prop.GetValue(record);
                if (value is float f && float.IsNaN(f))
                    Console.WriteLine($"字段 {prop.Name} 是 NaN");
                if (value is double d && double.IsNaN(d))
                    Console.WriteLine($"字段 {prop.Name} 是 NaN");
            }
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
                ProcessNanValues(record);
                
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
                
                // 将数据库中的null值转换回NaN
                foreach (var record in records)
                {
                    RestoreNanValues(record);
                }
                
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

                // 构建参数化的SQL DELETE语句
                // 假设表名为 ChannelMappings，如果您的表名不同，请在此处修改
                var sql = "DELETE FROM ChannelMappings WHERE TestTag = {0}";

                // 执行SQL语句
                int affectedRows = await _context.Database.ExecuteSqlRawAsync(sql, testTag);

                // 如果 affectedRows >= 0，表示命令成功执行（即使没有行被删除也是成功的）
                return affectedRows >= 0;
            }
            catch (Exception ex)
            {
                MessageBox.Show($"使用SQL删除测试记录时出错: {ex.Message}", "数据库错误", MessageBoxButton.OK, MessageBoxImage.Error);
                return false;
            }
        }
        
        #endregion

        /// <summary>
        /// 处理ChannelMapping对象中的NaN值，将其转换为-999999999以便存储到数据库
        /// </summary>
        /// <param name="record">待处理的通道映射对象</param>
        private void ProcessNanValues(ChannelMapping record)
        {

            var props = typeof(ChannelMapping).GetProperties(BindingFlags.Public | BindingFlags.Instance);

            foreach (var prop in props)
            {
                if (!prop.CanRead || !prop.CanWrite)
                    continue;

                var type = prop.PropertyType;
                var value = prop.GetValue(record);

                if (type == typeof(float) && value is float f && float.IsNaN(f))
                {
                    prop.SetValue(record, -999999999f);
                }
                else if (type == typeof(double) && value is double d && double.IsNaN(d))
                {
                    prop.SetValue(record, -999999999d);
                }
            }
        }
        
        /// <summary>
        /// 将数据库中读取的ChannelMapping对象中的-999999999值转换回NaN
        /// </summary>
        /// <param name="record">待处理的通道映射对象</param>
        private void RestoreNanValues(ChannelMapping record)
        {
            // 将-999999999转换回NaN
            // 处理float类型的字段
            //if (record.RangeLowerLimitValue == -999999999)
            //    record.RangeLowerLimitValue = float.NaN;

            //if (record.RangeUpperLimitValue == -999999999)
            //    record.RangeUpperLimitValue = float.NaN;

            //if (record.SLLSetValueNumber == -999999999)
            //    record.SLLSetValueNumber = float.NaN;

            //if (record.SLSetValueNumber == -999999999)
            //    record.SLSetValueNumber = float.NaN;

            //if (record.SHSetValueNumber == -999999999)
            //    record.SHSetValueNumber = float.NaN;

            //if (record.SHHSetValueNumber == -999999999)
            //    record.SHHSetValueNumber = float.NaN;

            //// 处理double类型的字段
            //if (record.ExpectedValue == -999999999)
            //    record.ExpectedValue = double.NaN;

            //if (record.ActualValue == -999999999)
            //    record.ActualValue = double.NaN;

            //if (record.Value0Percent == -999999999)
            //    record.Value0Percent = double.NaN;

            //if (record.Value25Percent == -999999999)
            //    record.Value25Percent = double.NaN;

            //if (record.Value50Percent == -999999999)
            //    record.Value50Percent = double.NaN;

            //if (record.Value75Percent == -999999999)
            //    record.Value75Percent = double.NaN;

            //if (record.Value100Percent == -999999999)
            //    record.Value100Percent = double.NaN;

            var props = typeof(ChannelMapping).GetProperties(BindingFlags.Public | BindingFlags.Instance);

            foreach (var prop in props)
            {
                if (!prop.CanRead || !prop.CanWrite)
                    continue;

                var type = prop.PropertyType;
                var value = prop.GetValue(record);

                if (type == typeof(float) && value is float f && f == -999999999f)
                {
                    prop.SetValue(record, float.NaN);
                }
                else if (type == typeof(double) && value is double d && d == -999999999d)
                {
                    prop.SetValue(record, double.NaN);
                }
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
                
                // 将数据库中的null值转换回NaN
                foreach (var record in records)
                {
                    RestoreNanValues(record);
                }
                
                return records;
            }
            catch (Exception ex)
            {
                MessageBox.Show($"获取所有测试记录时出错: {ex.Message}", "错误", MessageBoxButton.OK, MessageBoxImage.Error);
                return new List<ChannelMapping>();
            }
        }

        private ChannelMapping CloneAndCleanNan(ChannelMapping original)
        {
            var clone = new ChannelMapping
            {
                Id = original.Id,
                TestTag = original.TestTag,
                UpdatedTime = original.UpdatedTime,
                // ...复制所有字段...
                RangeLowerLimitValue = float.IsNaN(original.RangeLowerLimitValue) ? -999999999f : original.RangeLowerLimitValue,
                // 对其他字段重复这一判断
                // ...
            };
            return clone;
        }

        /// <summary>
        /// 使用原生SQL语句保存测试记录
        /// </summary>
        /// <param name="record">测试记录</param>
        /// <returns>保存操作是否成功</returns>
        public async Task<bool> SaveTestRecordWithSqlAsync(ChannelMapping record)
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
                ProcessNanValues(record);

                // 检查记录是否已存在
                var existing = await _context.ChannelMappings.FirstOrDefaultAsync(c => c.Id == record.Id);
                
                if (existing != null)
                {
                    // 如果记录存在，使用SQL更新
                    var query = @"
                    UPDATE ChannelMappings
                    SET TestTag = @TestTag,
                        ModuleName = @ModuleName,
                        ModuleType = @ModuleType,
                        PowerSupplyType = @PowerSupplyType,
                        WireSystem = @WireSystem,
                        Tag = @Tag,
                        StationName = @StationName,
                        VariableName = @VariableName,
                        VariableDescription = @VariableDescription,
                        DataType = @DataType,
                        ChannelTag = @ChannelTag,
                        AccessProperty = @AccessProperty,
                        SaveHistory = @SaveHistory,
                        PowerFailureProtection = @PowerFailureProtection,
                        RangeLowerLimit = @RangeLowerLimit,
                        RangeLowerLimitValue = @RangeLowerLimitValue,
                        RangeUpperLimit = @RangeUpperLimit,
                        RangeUpperLimitValue = @RangeUpperLimitValue,
                        SLLSetValue = @SLLSetValue,
                        SLLSetValueNumber = @SLLSetValueNumber,
                        SLLSetPoint = @SLLSetPoint,
                        SLLSetPointPLCAddress = @SLLSetPointPLCAddress,
                        SLLSetPointCommAddress = @SLLSetPointCommAddress,
                        SLSetValue = @SLSetValue,
                        SLSetValueNumber = @SLSetValueNumber,
                        SLSetPoint = @SLSetPoint,
                        SLSetPointPLCAddress = @SLSetPointPLCAddress,
                        SLSetPointCommAddress = @SLSetPointCommAddress,
                        SHSetValue = @SHSetValue,
                        SHSetValueNumber = @SHSetValueNumber,
                        SHSetPoint = @SHSetPoint,
                        SHSetPointPLCAddress = @SHSetPointPLCAddress,
                        SHSetPointCommAddress = @SHSetPointCommAddress,
                        SHHSetValue = @SHHSetValue,
                        SHHSetValueNumber = @SHHSetValueNumber,
                        SHHSetPoint = @SHHSetPoint,
                        SHHSetPointPLCAddress = @SHHSetPointPLCAddress,
                        SHHSetPointCommAddress = @SHHSetPointCommAddress,
                        LLAlarm = @LLAlarm,
                        LLAlarmPLCAddress = @LLAlarmPLCAddress,
                        LLAlarmCommAddress = @LLAlarmCommAddress,
                        LAlarm = @LAlarm,
                        LAlarmPLCAddress = @LAlarmPLCAddress,
                        LAlarmCommAddress = @LAlarmCommAddress,
                        HAlarm = @HAlarm,
                        HAlarmPLCAddress = @HAlarmPLCAddress,
                        HAlarmCommAddress = @HAlarmCommAddress,
                        HHAlarm = @HHAlarm,
                        HHAlarmPLCAddress = @HHAlarmPLCAddress,
                        HHAlarmCommAddress = @HHAlarmCommAddress,
                        MaintenanceValueSetting = @MaintenanceValueSetting,
                        MaintenanceValueSetPoint = @MaintenanceValueSetPoint,
                        MaintenanceValueSetPointPLCAddress = @MaintenanceValueSetPointPLCAddress,
                        MaintenanceValueSetPointCommAddress = @MaintenanceValueSetPointCommAddress,
                        MaintenanceEnableSwitchPoint = @MaintenanceEnableSwitchPoint,
                        MaintenanceEnableSwitchPointPLCAddress = @MaintenanceEnableSwitchPointPLCAddress,
                        MaintenanceEnableSwitchPointCommAddress = @MaintenanceEnableSwitchPointCommAddress,
                        PLCAbsoluteAddress = @PLCAbsoluteAddress,
                        PlcCommunicationAddress = @PlcCommunicationAddress,
                        UpdatedTime = @UpdatedTime,
                        TestBatch = @TestBatch,
                        TestPLCChannelTag = @TestPLCChannelTag,
                        TestPLCCommunicationAddress = @TestPLCCommunicationAddress,
                        MonitorStatus = @MonitorStatus,
                        TestId = @TestId,
                        TestResultStatus = @TestResultStatus,
                        ResultText = @ResultText,
                        HardPointTestResult = @HardPointTestResult,
                        TestTime = @TestTime,
                        FinalTestTime = @FinalTestTime,
                        Status = @Status,
                        StartTime = @StartTime,
                        EndTime = @EndTime,
                        ExpectedValue = @ExpectedValue,
                        ActualValue = @ActualValue,
                        Value0Percent = @Value0Percent,
                        Value25Percent = @Value25Percent,
                        Value50Percent = @Value50Percent,
                        Value75Percent = @Value75Percent,
                        Value100Percent = @Value100Percent,
                        LowLowAlarmStatus = @LowLowAlarmStatus,
                        LowAlarmStatus = @LowAlarmStatus,
                        HighAlarmStatus = @HighAlarmStatus,
                        HighHighAlarmStatus = @HighHighAlarmStatus,
                        MaintenanceFunction = @MaintenanceFunction,
                        ErrorMessage = @ErrorMessage,
                        CurrentValue = @CurrentValue,
                        ShowValueStatus = @ShowValueStatus,
                        AlarmValueSetStatus = @AlarmValueSetStatus,
                        TrendCheck = @TrendCheck,
                        ReportCheck = @ReportCheck
                    WHERE Id = @Id";

                    var parameters = CreateSqlParameters(record);

                    // 执行SQL语句
                    var result = await _context.Database.ExecuteSqlRawAsync(query, parameters.Select(p => 
                        new Microsoft.Data.Sqlite.SqliteParameter(p.Key, p.Value)).ToArray());

                    // 恢复NaN值
                    RestoreNanValues(record);
                    
                    return result > 0;
                }
                else
                {
                    // 如果记录不存在，使用SQL插入
                    var query = @"
                    INSERT INTO ChannelMappings (
                        Id, TestTag, ModuleName, ModuleType, PowerSupplyType, 
                        WireSystem, Tag, StationName, VariableName, VariableDescription, 
                        DataType, ChannelTag, AccessProperty, SaveHistory, PowerFailureProtection, 
                        RangeLowerLimit, RangeLowerLimitValue, RangeUpperLimit, RangeUpperLimitValue, 
                        SLLSetValue, SLLSetValueNumber, SLLSetPoint, SLLSetPointPLCAddress, SLLSetPointCommAddress, 
                        SLSetValue, SLSetValueNumber, SLSetPoint, SLSetPointPLCAddress, SLSetPointCommAddress, 
                        SHSetValue, SHSetValueNumber, SHSetPoint, SHSetPointPLCAddress, SHSetPointCommAddress, 
                        SHHSetValue, SHHSetValueNumber, SHHSetPoint, SHHSetPointPLCAddress, SHHSetPointCommAddress, 
                        LLAlarm, LLAlarmPLCAddress, LLAlarmCommAddress, 
                        LAlarm, LAlarmPLCAddress, LAlarmCommAddress, 
                        HAlarm, HAlarmPLCAddress, HAlarmCommAddress, 
                        HHAlarm, HHAlarmPLCAddress, HHAlarmCommAddress, 
                        MaintenanceValueSetting, MaintenanceValueSetPoint, MaintenanceValueSetPointPLCAddress, MaintenanceValueSetPointCommAddress, 
                        MaintenanceEnableSwitchPoint, MaintenanceEnableSwitchPointPLCAddress, MaintenanceEnableSwitchPointCommAddress, 
                        PLCAbsoluteAddress, PlcCommunicationAddress, 
                        CreatedTime, UpdatedTime, 
                        TestBatch, TestPLCChannelTag, TestPLCCommunicationAddress, 
                        MonitorStatus, TestId, TestResultStatus, ResultText, HardPointTestResult, 
                        TestTime, FinalTestTime, Status, StartTime, EndTime, 
                        ExpectedValue, ActualValue, 
                        Value0Percent, Value25Percent, Value50Percent, Value75Percent, Value100Percent, 
                        LowLowAlarmStatus, LowAlarmStatus, HighAlarmStatus, HighHighAlarmStatus, 
                        MaintenanceFunction, ErrorMessage, CurrentValue, ShowValueStatus, AlarmValueSetStatus,
                        TrendCheck, ReportCheck
                    ) VALUES (
                        @Id, @TestTag, @ModuleName, @ModuleType, @PowerSupplyType, 
                        @WireSystem, @Tag, @StationName, @VariableName, @VariableDescription, 
                        @DataType, @ChannelTag, @AccessProperty, @SaveHistory, @PowerFailureProtection, 
                        @RangeLowerLimit, @RangeLowerLimitValue, @RangeUpperLimit, @RangeUpperLimitValue, 
                        @SLLSetValue, @SLLSetValueNumber, @SLLSetPoint, @SLLSetPointPLCAddress, @SLLSetPointCommAddress, 
                        @SLSetValue, @SLSetValueNumber, @SLSetPoint, @SLSetPointPLCAddress, @SLSetPointCommAddress, 
                        @SHSetValue, @SHSetValueNumber, @SHSetPoint, @SHSetPointPLCAddress, @SHSetPointCommAddress, 
                        @SHHSetValue, @SHHSetValueNumber, @SHHSetPoint, @SHHSetPointPLCAddress, @SHHSetPointCommAddress, 
                        @LLAlarm, @LLAlarmPLCAddress, @LLAlarmCommAddress, 
                        @LAlarm, @LAlarmPLCAddress, @LAlarmCommAddress, 
                        @HAlarm, @HAlarmPLCAddress, @HAlarmCommAddress, 
                        @HHAlarm, @HHAlarmPLCAddress, @HHAlarmCommAddress, 
                        @MaintenanceValueSetting, @MaintenanceValueSetPoint, @MaintenanceValueSetPointPLCAddress, @MaintenanceValueSetPointCommAddress, 
                        @MaintenanceEnableSwitchPoint, @MaintenanceEnableSwitchPointPLCAddress, @MaintenanceEnableSwitchPointCommAddress, 
                        @PLCAbsoluteAddress, @PlcCommunicationAddress, 
                        @CreatedTime, @UpdatedTime, 
                        @TestBatch, @TestPLCChannelTag, @TestPLCCommunicationAddress, 
                        @MonitorStatus, @TestId, @TestResultStatus, @ResultText, @HardPointTestResult, 
                        @TestTime, @FinalTestTime, @Status, @StartTime, @EndTime, 
                        @ExpectedValue, @ActualValue, 
                        @Value0Percent, @Value25Percent, @Value50Percent, @Value75Percent, @Value100Percent, 
                        @LowLowAlarmStatus, @LowAlarmStatus, @HighAlarmStatus, @HighHighAlarmStatus, 
                        @MaintenanceFunction, @ErrorMessage, @CurrentValue, @ShowValueStatus, @AlarmValueSetStatus,
                        @TrendCheck, @ReportCheck
                    )";

                    var parameters = CreateSqlParameters(record);
                    parameters.Add("@CreatedTime", record.CreatedTime);

                    // 执行SQL语句
                    var result = await _context.Database.ExecuteSqlRawAsync(query, parameters.Select(p => 
                        new Microsoft.Data.Sqlite.SqliteParameter(p.Key, p.Value)).ToArray());

                    // 恢复NaN值
                    RestoreNanValues(record);
                    
                    return result > 0;
                }
            }
            catch (Exception ex)
            {
                MessageBox.Show($"保存测试记录时出错: {ex.Message}", "错误", MessageBoxButton.OK, MessageBoxImage.Error);
                return false;
            }
        }

        /// <summary>
        /// 创建SQL参数列表
        /// </summary>
        /// <param name="record">通道映射记录</param>
        /// <returns>参数字典</returns>
        private Dictionary<string, object> CreateSqlParameters(ChannelMapping record)
        {
            return new Dictionary<string, object>
            {
                { "@Id", record.Id },
                { "@TestTag", record.TestTag ?? (object)DBNull.Value },
                { "@ModuleName", record.ModuleName ?? (object)DBNull.Value },
                { "@ModuleType", record.ModuleType ?? (object)DBNull.Value },
                { "@PowerSupplyType", record.PowerSupplyType ?? (object)DBNull.Value },
                { "@WireSystem", record.WireSystem ?? (object)DBNull.Value },
                { "@Tag", record.Tag ?? (object)DBNull.Value },
                { "@StationName", record.StationName ?? (object)DBNull.Value },
                { "@VariableName", record.VariableName ?? (object)DBNull.Value },
                { "@VariableDescription", record.VariableDescription ?? (object)DBNull.Value },
                { "@DataType", record.DataType ?? (object)DBNull.Value },
                { "@ChannelTag", record.ChannelTag ?? (object)DBNull.Value },
                { "@AccessProperty", record.AccessProperty ?? (object)DBNull.Value },
                { "@SaveHistory", record.SaveHistory ?? (object)DBNull.Value },
                { "@PowerFailureProtection", record.PowerFailureProtection ?? (object)DBNull.Value },
                { "@RangeLowerLimit", record.RangeLowerLimit ?? (object)DBNull.Value },
                { "@RangeLowerLimitValue", record.RangeLowerLimitValue },
                { "@RangeUpperLimit", record.RangeUpperLimit ?? (object)DBNull.Value },
                { "@RangeUpperLimitValue", record.RangeUpperLimitValue },
                { "@SLLSetValue", record.SLLSetValue ?? (object)DBNull.Value },
                { "@SLLSetValueNumber", record.SLLSetValueNumber },
                { "@SLLSetPoint", record.SLLSetPoint ?? (object)DBNull.Value },
                { "@SLLSetPointPLCAddress", record.SLLSetPointPLCAddress ?? (object)DBNull.Value },
                { "@SLLSetPointCommAddress", record.SLLSetPointCommAddress ?? (object)DBNull.Value },
                { "@SLSetValue", record.SLSetValue ?? (object)DBNull.Value },
                { "@SLSetValueNumber", record.SLSetValueNumber },
                { "@SLSetPoint", record.SLSetPoint ?? (object)DBNull.Value },
                { "@SLSetPointPLCAddress", record.SLSetPointPLCAddress ?? (object)DBNull.Value },
                { "@SLSetPointCommAddress", record.SLSetPointCommAddress ?? (object)DBNull.Value },
                { "@SHSetValue", record.SHSetValue ?? (object)DBNull.Value },
                { "@SHSetValueNumber", record.SHSetValueNumber },
                { "@SHSetPoint", record.SHSetPoint ?? (object)DBNull.Value },
                { "@SHSetPointPLCAddress", record.SHSetPointPLCAddress ?? (object)DBNull.Value },
                { "@SHSetPointCommAddress", record.SHSetPointCommAddress ?? (object)DBNull.Value },
                { "@SHHSetValue", record.SHHSetValue ?? (object)DBNull.Value },
                { "@SHHSetValueNumber", record.SHHSetValueNumber },
                { "@SHHSetPoint", record.SHHSetPoint ?? (object)DBNull.Value },
                { "@SHHSetPointPLCAddress", record.SHHSetPointPLCAddress ?? (object)DBNull.Value },
                { "@SHHSetPointCommAddress", record.SHHSetPointCommAddress ?? (object)DBNull.Value },
                { "@LLAlarm", record.LLAlarm ?? (object)DBNull.Value },
                { "@LLAlarmPLCAddress", record.LLAlarmPLCAddress ?? (object)DBNull.Value },
                { "@LLAlarmCommAddress", record.LLAlarmCommAddress ?? (object)DBNull.Value },
                { "@LAlarm", record.LAlarm ?? (object)DBNull.Value },
                { "@LAlarmPLCAddress", record.LAlarmPLCAddress ?? (object)DBNull.Value },
                { "@LAlarmCommAddress", record.LAlarmCommAddress ?? (object)DBNull.Value },
                { "@HAlarm", record.HAlarm ?? (object)DBNull.Value },
                { "@HAlarmPLCAddress", record.HAlarmPLCAddress ?? (object)DBNull.Value },
                { "@HAlarmCommAddress", record.HAlarmCommAddress ?? (object)DBNull.Value },
                { "@HHAlarm", record.HHAlarm ?? (object)DBNull.Value },
                { "@HHAlarmPLCAddress", record.HHAlarmPLCAddress ?? (object)DBNull.Value },
                { "@HHAlarmCommAddress", record.HHAlarmCommAddress ?? (object)DBNull.Value },
                { "@MaintenanceValueSetting", record.MaintenanceValueSetting ?? (object)DBNull.Value },
                { "@MaintenanceValueSetPoint", record.MaintenanceValueSetPoint ?? (object)DBNull.Value },
                { "@MaintenanceValueSetPointPLCAddress", record.MaintenanceValueSetPointPLCAddress ?? (object)DBNull.Value },
                { "@MaintenanceValueSetPointCommAddress", record.MaintenanceValueSetPointCommAddress ?? (object)DBNull.Value },
                { "@MaintenanceEnableSwitchPoint", record.MaintenanceEnableSwitchPoint ?? (object)DBNull.Value },
                { "@MaintenanceEnableSwitchPointPLCAddress", record.MaintenanceEnableSwitchPointPLCAddress ?? (object)DBNull.Value },
                { "@MaintenanceEnableSwitchPointCommAddress", record.MaintenanceEnableSwitchPointCommAddress ?? (object)DBNull.Value },
                { "@PLCAbsoluteAddress", record.PLCAbsoluteAddress ?? (object)DBNull.Value },
                { "@PlcCommunicationAddress", record.PlcCommunicationAddress ?? (object)DBNull.Value },
                { "@UpdatedTime", record.UpdatedTime },
                { "@TestBatch", record.TestBatch ?? (object)DBNull.Value },
                { "@TestPLCChannelTag", record.TestPLCChannelTag ?? (object)DBNull.Value },
                { "@TestPLCCommunicationAddress", record.TestPLCCommunicationAddress ?? (object)DBNull.Value },
                { "@MonitorStatus", record.MonitorStatus ?? (object)DBNull.Value },
                { "@TestId", record.TestId },
                { "@TestResultStatus", record.TestResultStatus },
                { "@ResultText", record.ResultText ?? (object)DBNull.Value },
                { "@HardPointTestResult", record.HardPointTestResult ?? (object)DBNull.Value },
                { "@TestTime", record.TestTime.HasValue ? (object)record.TestTime.Value : DBNull.Value },
                { "@FinalTestTime", record.FinalTestTime.HasValue ? (object)record.FinalTestTime.Value : DBNull.Value },
                { "@Status", record.Status ?? (object)DBNull.Value },
                { "@StartTime", record.StartTime },
                { "@EndTime", record.EndTime },
                { "@ExpectedValue", record.ExpectedValue },
                { "@ActualValue", record.ActualValue },
                { "@Value0Percent", record.Value0Percent },
                { "@Value25Percent", record.Value25Percent },
                { "@Value50Percent", record.Value50Percent },
                { "@Value75Percent", record.Value75Percent },
                { "@Value100Percent", record.Value100Percent },
                { "@LowLowAlarmStatus", record.LowLowAlarmStatus ?? (object)DBNull.Value },
                { "@LowAlarmStatus", record.LowAlarmStatus ?? (object)DBNull.Value },
                { "@HighAlarmStatus", record.HighAlarmStatus ?? (object)DBNull.Value },
                { "@HighHighAlarmStatus", record.HighHighAlarmStatus ?? (object)DBNull.Value },
                { "@MaintenanceFunction", record.MaintenanceFunction ?? (object)DBNull.Value },
                { "@ErrorMessage", record.ErrorMessage ?? (object)DBNull.Value },
                { "@CurrentValue", record.CurrentValue ?? (object)DBNull.Value },
                { "@ShowValueStatus", record.ShowValueStatus ?? (object)DBNull.Value },
                { "@AlarmValueSetStatus", record.AlarmValueSetStatus ?? (object)DBNull.Value },
                { "@TrendCheck", record.TrendCheck ?? (object)DBNull.Value },
                { "@ReportCheck", record.ReportCheck ?? (object)DBNull.Value }
            };
        }

        /// <summary>
        /// 使用原生SQL语句批量保存多条测试记录
        /// </summary>
        /// <param name="records">测试记录集合</param>
        /// <returns>保存操作是否成功</returns>
        public async Task<bool> SaveTestRecordsWithSqlAsync(IEnumerable<ChannelMapping> records)
        {
            try
            {
                if (records == null || !records.Any())
                    return true;

                int successCount = 0;
                foreach (var record in records)
                {
                    // 调用单个记录的SQL保存方法
                    bool success = await SaveTestRecordWithSqlAsync(record);
                    if (success)
                    {
                        successCount++;
                    }
                }

                // 如果有任何记录保存成功，就返回true
                return successCount > 0;
            }
            catch (Exception ex)
            {
                MessageBox.Show($"批量保存测试记录时出错: {ex.Message}", "错误", MessageBoxButton.OK, MessageBoxImage.Error);
                return false;
            }
        }
    }
}
