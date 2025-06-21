using System;
using System.Collections.Generic;
using System.Threading.Tasks;
using FatFullVersion.Models;

namespace FatFullVersion.IServices
{
    /// <summary>
    /// 测试结果导出服务接口
    /// </summary>
    public interface ITestResultExportService
    {
        /// <summary>
        /// 导出测试结果到Excel文件
        /// </summary>
        /// <param name="testResults">测试结果数据</param>
        /// <param name="filePath">导出文件路径，如果为null则通过文件对话框选择</param>
        /// <returns>导出是否成功</returns>
        Task<bool> ExportToExcelAsync(IEnumerable<ChannelMapping> testResults, string filePath = null);
        
        /// <summary>
        /// 检查是否所有测试点位都已通过
        /// </summary>
        /// <param name="testResults">测试结果数据</param>
        /// <returns>是否所有点位都已通过测试</returns>
        bool AreAllTestsPassed(IEnumerable<ChannelMapping> testResults);

        /// <summary>
        /// 导出通道映射到Excel文件
        /// </summary>
        /// <param name="channelMappings">通道映射数据</param>
        /// <param name="filePath">导出文件路径，如果为null则通过文件对话框选择</param>
        /// <returns>导出是否成功</returns>
        Task<bool> ExportChannelMapToExcelAsync(IEnumerable<ChannelMapping> channelMappings, string filePath = null);
    }
} 