using System;
using System.Threading;
using System.Threading.Tasks;
using FatFullVersion.IServices;
using FatFullVersion.Models;
using FatFullVersion.Services.Interfaces; // IPlcCommunication

namespace FatFullVersion.Services
{
    /// <summary>
    /// 实现定时读取 AI 通道报警设定值 (SL/SLL/SH/SHH) 的监控服务。
    /// </summary>
    public class AlarmValueMonitorService : IAlarmValueMonitorService
    {
        private readonly IPlcCommunication _plc;
        private CancellationTokenSource _cts;
        private Task _monitorTask;
        private ChannelMapping _channel;
        private Action<float?, float?, float?, float?> _updateAction;

        public AlarmValueMonitorService(IPlcCommunication targetPlc)
        {
            _plc = targetPlc ?? throw new ArgumentNullException(nameof(targetPlc));
        }

        public void StartMonitoring(ChannelMapping channel, Action<float?, float?, float?, float?> updateAction)
        {
            StopMonitoring(); // 确保之前的任务停止
            if (channel == null || updateAction == null) return;

            _channel = channel;
            _updateAction = updateAction;
            _cts = new CancellationTokenSource();
            _monitorTask = Task.Run(() => MonitorLoopAsync(_cts.Token));
        }

        public void StopMonitoring()
        {
            if (_cts != null)
            {
                try { _cts.Cancel(); } catch { }
                _cts = null;
            }
        }

        private async Task MonitorLoopAsync(CancellationToken token)
        {
            while (!token.IsCancellationRequested)
            {
                try
                {
                    float? sl  = await ReadAnalogAsync(_channel?.SLSetPointCommAddress);
                    float? sll = await ReadAnalogAsync(_channel?.SLLSetPointCommAddress);
                    float? sh  = await ReadAnalogAsync(_channel?.SHSetPointCommAddress);
                    float? shh = await ReadAnalogAsync(_channel?.SHHSetPointCommAddress);

                    _updateAction?.Invoke(sl, sll, sh, shh);
                }
                catch (Exception ex)
                {
                    System.Diagnostics.Debug.WriteLine($"AlarmValueMonitorService Error: {ex.Message}");
                }
                await Task.Delay(500, token).ContinueWith(_ => { });
            }
        }

        private async Task<float?> ReadAnalogAsync(string address)
        {
            try
            {
                if (string.IsNullOrEmpty(address)) return null;
                var result = await _plc.ReadAnalogValueAsync(address.Substring(1));
                return result.IsSuccess ? result.Data : (float?)null;
            }
            catch
            {
                return null;
            }
        }
    }
} 