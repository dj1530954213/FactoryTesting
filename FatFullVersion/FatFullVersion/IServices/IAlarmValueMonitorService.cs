using System;
using System.Threading.Tasks;
using FatFullVersion.Models;

namespace FatFullVersion.IServices
{
    /// <summary>
    /// 报警设定值监控服务接口，用于在指定通道上定时读取 SL/ SLL/ SH/ SHH 四个设定值。
    /// </summary>
    public interface IAlarmValueMonitorService
    {
        /// <summary>
        /// 启动监控。
        /// </summary>
        /// <param name="channel">要监控的通道 (AI)</param>
        /// <param name="updateAction">读取到值后的回调 (low, lowLow, high, highHigh)</param>
        void StartMonitoring(ChannelMapping channel, Action<float?, float?, float?, float?> updateAction);

        /// <summary>
        /// 停止监控。
        /// </summary>
        void StopMonitoring();
    }
} 