using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Threading.Tasks;
using System.Windows;
using FatFullVersion.Models;
using FatFullVersion.IServices;

namespace FatFullVersion.IServices
{
    /// <summary>
    /// 通道量程设定服务接口
    /// </summary>
    public interface IChannelRangerSettingService
    {
        /// <summary>
        /// 获取批次量程设定寄存器和值（仅返回不写）
        /// </summary>
        Dictionary<string, float> SetChannelRangeValue(IEnumerable<ChannelMapping> instances, string batchName);

        /// <summary>
        /// 直接根据批次信息写入量程设定
        /// </summary>
        Task SetChannelRangeAsync(IEnumerable<ChannelMapping> instances, string batchName, IPlcCommunication plc);
    }
}
