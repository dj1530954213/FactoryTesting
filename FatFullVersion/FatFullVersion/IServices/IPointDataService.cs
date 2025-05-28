using System;
using System.Collections.Generic;
using System.Threading.Tasks;
using FatFullVersion.IServices;
using FatFullVersion.Models;

namespace FatFullVersion.Services.Interfaces
{
    /// <summary>
    /// 点表数据服务接口，用于处理点表数据的导入和处理
    /// </summary>
    public interface IPointDataService
    {
        /// <summary>
        /// 获取点表数据并转换为点表对象列表
        /// </summary>
        /// <param name="parameters">获取参数</param>
        /// <returns>点表对象列表</returns>
        Task<IEnumerable<ExcelPointData>> GetPointDataAsync(Dictionary<string, object> parameters = null);
        
        /// <summary>
        /// 验证点表数据的有效性
        /// </summary>
        /// <param name="pointDataList">需要验证的点表数据列表</param>
        /// <returns>验证结果，包含错误信息</returns>
        Task<ValidationResult> ValidatePointDataAsync(IEnumerable<ExcelPointData> pointDataList);
        
        /// <summary>
        /// 保存点表数据
        /// </summary>
        /// <param name="pointDataList">点表数据列表</param>
        /// <returns>操作结果</returns>
        Task<bool> SavePointDataAsync(IEnumerable<ExcelPointData> pointDataList);

        /// <summary>
        /// 导入点表配置
        /// </summary>
        /// <param name="importAction">导入完成后的回调动作，参数为导入的点表数据列表</param>
        /// <returns></returns>
        Task ImportPointConfigurationAsync(Action<IEnumerable<ExcelPointData>> importAction = null);
    }

    
} 