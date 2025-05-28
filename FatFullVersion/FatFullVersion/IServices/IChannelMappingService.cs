using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Threading.Tasks;
using FatFullVersion.Models;
using FatFullVersion.Entities;
using FatFullVersion.ViewModels;
using FatFullVersion.Entities.EntitiesEnum;
using System.Collections.ObjectModel;
using FatFullVersion.Shared;

namespace FatFullVersion.IServices
{
    /// <summary>
    /// 通道映射服务接口
    /// 负责处理通道映射相关的业务逻辑，包括数据转换、批次管理和分配逻辑
    /// </summary>
    public interface IChannelMappingService
    {
        /// <summary>
        /// 获取所有通道映射
        /// </summary>
        /// <returns>通道映射列表</returns>
        Task<IEnumerable<ChannelMapping>> GetAllChannelMappingsAsync();

        /// <summary>
        /// 根据批次名称获取通道映射
        /// </summary>
        /// <param name="batchName">批次名称</param>
        /// <returns>该批次的通道映射列表</returns>
        Task<IEnumerable<ChannelMapping>> GetChannelMappingsByBatchNameAsync(string batchName);

        /// <summary>
        /// 根据通道类型获取通道映射
        /// </summary>
        /// <param name="moduleType">通道类型</param>
        /// <returns>该类型的通道映射列表</returns>
        Task<IEnumerable<ChannelMapping>> GetChannelMappingsByModuleTypeAsync(string moduleType);

        /// <summary>
        /// 添加通道映射
        /// </summary>
        /// <param name="channelMapping">通道映射实体</param>
        /// <returns>操作结果</returns>
        Task<bool> AddChannelMappingAsync(ChannelMapping channelMapping);

        /// <summary>
        /// 批量添加通道映射
        /// </summary>
        /// <param name="channelMappings">通道映射列表</param>
        /// <returns>操作结果</returns>
        Task<bool> AddChannelMappingsAsync(IEnumerable<ChannelMapping> channelMappings);

        /// <summary>
        /// 更新通道映射
        /// </summary>
        /// <param name="channelMapping">通道映射实体</param>
        /// <returns>操作结果</returns>
        Task<bool> UpdateChannelMappingAsync(ChannelMapping channelMapping);

        /// <summary>
        /// 批量更新通道映射
        /// </summary>
        /// <param name="channelMappings">通道映射列表</param>
        /// <returns>操作结果</returns>
        Task<bool> UpdateChannelMappingsAsync(IEnumerable<ChannelMapping> channelMappings);

        /// <summary>
        /// 删除通道映射
        /// </summary>
        /// <param name="id">通道映射ID</param>
        /// <returns>操作结果</returns>
        Task<bool> DeleteChannelMappingAsync(string id);

        /// <summary>
        /// 批量删除通道映射
        /// </summary>
        /// <param name="ids">通道映射ID列表</param>
        /// <returns>操作结果</returns>
        Task<bool> DeleteChannelMappingsAsync(IEnumerable<string> ids);

        /// <summary>
        /// 分配通道测试关系（唯一入口）
        /// </summary>
        /// <param name="channels">ObservableCollection 通道集合</param>
        /// <returns>分配后同一引用的集合</returns>
        Task<ObservableCollection<ChannelMapping>> AllocateChannelsTestAsync(ObservableCollection<ChannelMapping> channels);

        /// <summary>
        /// 将通道分配结果同步到通道集合中
        /// </summary>
        /// <param name="channels">通道映射集合</param>
        void SyncChannelAllocation(IEnumerable<ChannelMapping> channels);

        /// <summary>
        /// 设置当前使用的测试PLC配置
        /// </summary>
        /// <param name="config">配置信息</param>
        void SetTestPlcConfig(TestPlcConfig config);

        /// <summary>
        /// 更新批次状态信息，根据通道数据更新批次的状态
        /// </summary>
        /// <param name="batches">批次信息集合</param>
        /// <param name="channels">通道数据集合</param>
        /// <returns>更新后的批次信息集合</returns>
        Task<IEnumerable<BatchInfo>> UpdateBatchStatusAsync(
            IEnumerable<BatchInfo> batches,
            IEnumerable<ChannelMapping> channels);

        /// <summary>
        /// 获取特定类型的通道列表
        /// </summary>
        /// <param name="moduleType">通道类型</param>
        /// <param name="allChannels">所有通道集合</param>
        /// <returns>过滤后的通道列表</returns>
        IEnumerable<ChannelMapping> GetChannelsByType(string moduleType,string powerType, IEnumerable<ChannelMapping> allChannels);

        /// <summary>
        /// 从Excel导入的数据创建通道映射
        /// </summary>
        /// <param name="excelData">Excel导入的数据</param>
        /// <returns>创建的通道映射集合</returns>
        Task<IEnumerable<ChannelMapping>> CreateChannelMappingsFromExcelAsync(IEnumerable<ExcelPointData> excelData);

        /// <summary>
        /// 异步分配PLC通道映射关系
        /// </summary>
        /// <param name="aiChannels">AI通道列表</param>
        /// <param name="aoChannels">AO通道列表</param>
        /// <param name="diChannels">DI通道列表</param>
        /// <param name="doChannels">DO通道列表</param>
        /// <param name="testPlcConfig">测试PLC的配置信息</param>
        /// <returns>分配结果，包含所有类型的通道</returns>
        Task<(IEnumerable<ChannelMapping> AI, IEnumerable<ChannelMapping> AO, IEnumerable<ChannelMapping> DI, IEnumerable<ChannelMapping> DO)> 
            AllocateChannelsAsync(
                IEnumerable<ChannelMapping> aiChannels,
                IEnumerable<ChannelMapping> aoChannels, 
                IEnumerable<ChannelMapping> diChannels,
                IEnumerable<ChannelMapping> doChannels,
                TestPlcConfig testPlcConfig);

        /// <summary>
        /// 获取所有批次
        /// </summary>
        /// <returns>批次信息列表</returns>
        Task<IEnumerable<BatchInfo>> GetAllBatchesAsync();

        /// <summary>
        /// 获取或创建批次信息
        /// </summary>
        /// <param name="batchName">批次名称</param>
        /// <param name="itemCount">批次包含的项目数量</param>
        /// <returns>批次信息</returns>
        Task<BatchInfo> GetOrCreateBatchAsync(string batchName, int itemCount);

        /// <summary>
        /// 从通道映射信息中提取批次信息
        /// </summary>
        /// <param name="aiChannels">AI通道列表</param>
        /// <param name="aoChannels">AO通道列表</param>
        /// <param name="diChannels">DI通道列表</param>
        /// <param name="doChannels">DO通道列表</param>
        /// <returns>提取的批次信息集合</returns>
        Task<IEnumerable<BatchInfo>> ExtractBatchInfoAsync(
            IEnumerable<ChannelMapping> aiChannels,
            IEnumerable<ChannelMapping> aoChannels,
            IEnumerable<ChannelMapping> diChannels,
            IEnumerable<ChannelMapping> doChannels);

        /// <summary>
        /// 从通道映射信息中提取批次信息
        /// </summary>
        /// <param name="allChannels">所有通道集合</param>
        /// <returns>提取的批次信息集合</returns>
        Task<IEnumerable<BatchInfo>> ExtractBatchInfoAsync(
            IEnumerable<ChannelMapping> allChannels);

        /// <summary>
        /// 清除所有通道分配信息
        /// </summary>
        /// <param name="channels">需要清除分配信息的通道集合</param>
        /// <returns>清除分配信息后的通道集合</returns>
        Task<IEnumerable<ChannelMapping>> ClearAllChannelAllocationsAsync(IEnumerable<ChannelMapping> channels);

        /// <summary>
        /// 创建并初始化通道映射
        /// </summary>
        /// <param name="pointDataList">点数据列表</param>
        /// <returns>创建的通道映射集合</returns>
        Task<IEnumerable<ChannelMapping>> CreateAndInitializeChannelMappingsAsync(IEnumerable<ExcelPointData> pointDataList);

        /// <summary>
        /// 验证通道映射数据的完整性和正确性
        /// </summary>
        /// <param name="channels">需要验证的通道集合</param>
        /// <returns>验证结果，包含错误信息列表</returns>
        /// <remarks>
        /// 该方法检查通道数据的必填字段、数值范围、重复性等，确保数据质量
        /// </remarks>
        Task<ValidationResult> ValidateChannelMappingsAsync(IEnumerable<ChannelMapping> channels);

        /// <summary>
        /// 获取指定模块类型的通道数量统计
        /// </summary>
        /// <param name="channels">通道数据集合</param>
        /// <param name="moduleType">模块类型（如AI、AO、DI、DO等）</param>
        /// <returns>指定类型的通道数量</returns>
        int GetChannelCountByType(IEnumerable<ChannelMapping> channels, string moduleType);

        /// <summary>
        /// 按照模块类型分组获取通道
        /// </summary>
        /// <param name="channels">通道数据集合</param>
        /// <returns>按模块类型分组的通道字典</returns>
        Dictionary<string, List<ChannelMapping>> GroupChannelsByModuleType(IEnumerable<ChannelMapping> channels);

        /// <summary>
        /// 从通道集合中提取批次信息的重载方法
        /// </summary>
        /// <param name="channels">通道数据集合</param>
        /// <returns>提取的批次信息集合</returns>
        Task<IEnumerable<BatchInfo>> ExtractBatchInfoAsync(
            ObservableCollection<ChannelMapping> channels);

        /// <summary>
        /// 从通道集合中提取批次信息的泛型重载方法
        /// </summary>
        /// <param name="channels">通道数据集合</param>
        /// <returns>提取的批次信息集合</returns>
        Task<IEnumerable<BatchInfo>> ExtractBatchInfoAsync(
            ICollection<ChannelMapping> channels);
    }
}
