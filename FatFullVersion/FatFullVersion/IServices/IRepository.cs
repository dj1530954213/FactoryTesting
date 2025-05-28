using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Threading.Tasks;
using FatFullVersion.Entities;
using FatFullVersion.Entities.ValueObject;
using FatFullVersion.Models;

namespace FatFullVersion.IServices
{
    /// <summary>
    /// 仓储层接口，提供数据持久化服务
    /// </summary>
    public interface IRepository
    {
        /// <summary>
        /// 初始化数据库
        /// </summary>
        /// <returns>初始化是否成功</returns>
        Task<bool> InitializeDatabaseAsync();

        #region PLC连接配置操作

        /// <summary>
        /// 获取测试PLC连接配置
        /// </summary>
        /// <returns>PLC连接配置</returns>
        Task<PlcConnectionConfig> GetTestPlcConnectionConfigAsync();

        /// <summary>
        /// 获取被测PLC连接配置
        /// </summary>
        /// <returns>PLC连接配置</returns>
        Task<PlcConnectionConfig> GetTargetPlcConnectionConfigAsync();

        /// <summary>
        /// 保存PLC连接配置
        /// </summary>
        /// <param name="config">PLC连接配置</param>
        /// <returns>保存操作是否成功</returns>
        Task<bool> SavePlcConnectionConfigAsync(PlcConnectionConfig config);

        /// <summary>
        /// 获取所有PLC连接配置
        /// </summary>
        /// <returns>PLC连接配置列表</returns>
        Task<List<PlcConnectionConfig>> GetAllPlcConnectionConfigsAsync();

        #endregion

        #region 通道比较表操作

        /// <summary>
        /// 通过通道位号获取通讯地址
        /// </summary>
        /// <param name="channelTag">通道位号</param>
        /// <returns>通讯地址</returns>
        Task<string> GetPlcCommunicationAddress(string channelTag);

        /// <summary>
        /// 获取所有测试PLC的通道与通讯地址的对应关系
        /// </summary>
        /// <returns>通道比较表列表</returns>
        Task<List<ComparisonTable>> GetComparisonTablesAsync();

        /// <summary>
        /// 添加通道比较表项
        /// </summary>
        /// <param name="comparisonTable">通道比较表项</param>
        /// <returns>添加操作是否成功</returns>
        Task<bool> AddComparisonTableAsync(ComparisonTable comparisonTable);

        /// <summary>
        /// 添加多个通道比较表项
        /// </summary>
        /// <param name="comparisonTables">通道比较表项列表</param>
        /// <returns>添加操作是否成功</returns>
        Task<bool> AddComparisonTablesAsync(List<ComparisonTable> comparisonTables);

        /// <summary>
        /// 更新通道比较表项
        /// </summary>
        /// <param name="comparisonTable">通道比较表项</param>
        /// <returns>更新操作是否成功</returns>
        Task<bool> UpdateComparisonTableAsync(ComparisonTable comparisonTable);

        /// <summary>
        /// 删除通道比较表项
        /// </summary>
        /// <param name="id">通道比较表项ID</param>
        /// <returns>删除操作是否成功</returns>
        Task<bool> DeleteComparisonTableAsync(int id);

        /// <summary>
        /// 保存所有通道比较表项（删除旧数据并添加新数据）
        /// </summary>
        /// <param name="comparisonTables">通道比较表项列表</param>
        /// <returns>保存操作是否成功</returns>
        Task<bool> SaveAllComparisonTablesAsync(List<ComparisonTable> comparisonTables);

        #endregion
        
        #region 测试记录操作 - 重构优化
        
        /// <summary>
        /// 保存测试记录集合 - 通用方法
        /// </summary>
        /// <param name="records">测试记录集合</param>
        /// <returns>保存操作是否成功</returns>
        Task<bool> SaveTestRecordsAsync(IEnumerable<ChannelMapping> records);

        /// <summary>
        /// 保存单个测试记录 - 手动测试场景优化
        /// </summary>
        /// <param name="record">测试记录</param>
        /// <returns>保存操作是否成功</returns>
        Task<bool> SaveTestRecordAsync(ChannelMapping record);

        /// <summary>
        /// 批量保存硬点自动测试完成的记录 - 新增优化方法
        /// </summary>
        /// <param name="records">测试记录集合</param>
        /// <returns>保存操作是否成功</returns>
        Task<bool> SaveHardPointTestResultsAsync(IEnumerable<ChannelMapping> records);

        /// <summary>
        /// 更新单个通道的复测结果 - 复测场景优化
        /// </summary>
        /// <param name="record">测试记录</param>
        /// <returns>保存操作是否成功</returns>
        Task<bool> UpdateRetestResultAsync(ChannelMapping record);

        /// <summary>
        /// 根据测试标识获取测试记录
        /// </summary>
        /// <param name="testTag">测试标识</param>
        /// <returns>测试记录集合</returns>
        Task<List<ChannelMapping>> GetTestRecordsByTagAsync(string testTag);
        
        /// <summary>
        /// 获取所有不同的测试标识
        /// </summary>
        /// <returns>测试标识集合</returns>
        Task<List<string>> GetAllTestTagsAsync();
        
        /// <summary>
        /// 根据测试标识删除测试记录
        /// </summary>
        /// <param name="testTag">测试标识</param>
        /// <returns>删除操作是否成功</returns>
        Task<bool> DeleteTestRecordsByTagAsync(string testTag);
        
        /// <summary>
        /// 获取所有测试记录
        /// </summary>
        /// <returns>所有测试记录集合</returns>
        Task<List<ChannelMapping>> GetAllTestRecordsAsync();
        
        #endregion

        #region 废弃的方法 - 保留向后兼容性
        
        /// <summary>
        /// 批量保存测试记录 - 已废弃，请使用SaveTestRecordsAsync
        /// </summary>
        [Obsolete("已废弃，请使用SaveTestRecordsAsync")]
        Task<bool> SaveTestRecordsWithSqlAsync(IEnumerable<ChannelMapping> records);
        
        /// <summary>
        /// 保存单个测试记录 - 已废弃，请使用SaveTestRecordAsync
        /// </summary>
        [Obsolete("已废弃，请使用SaveTestRecordAsync")]
        Task<bool> SaveTestRecordWithSqlAsync(ChannelMapping record);
        
        #endregion
    }
}
