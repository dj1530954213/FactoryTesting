using System;
using System.Threading;
using System.Threading.Tasks;
using FatFullVersion.IServices;
using FatFullVersion.Models;
using FatFullVersion.Services.Interfaces;

namespace FatFullVersion.Services
{
    /// <summary>
    /// ManualTestIoService 实现周期性 PLC 读写，用于手动测试阶段的数据交互。
    /// 当前实现 AI 报警设定值监控功能（从目标PLC读取）、测试值下发功能（写入测试PLC）、AO 数值监控（从测试PLC读取）、DI 测试信号控制和 DO 数值监控。
    /// </summary>
    public class ManualTestIoService : IManualTestIoService
    {
        private readonly IPlcCommunication _targetPlc; // 用于读取报警设定值
        private readonly IPlcCommunication _testPlc;   // 用于写入测试值和读取AO值
        
        // AI 报警设定值监控相关字段
        private CancellationTokenSource _alarmMonitorCts;
        private Task _alarmMonitorTask;
        private ChannelMapping _alarmMonitorChannel;
        private Action<float?, float?, float?, float?> _alarmUpdateAction;
        
        // AO 数值监控相关字段
        private CancellationTokenSource _aoMonitorCts;
        private Task _aoMonitorTask;
        private ChannelMapping _aoMonitorChannel;
        private Action<string> _aoUpdateAction;

        // DO 数值监控相关字段
        private CancellationTokenSource _doMonitorCts;
        private Task _doMonitorTask;
        private ChannelMapping _doMonitorChannel;
        private Action<string> _doUpdateAction;

        public ManualTestIoService(IPlcCommunication targetPlc, IPlcCommunication testPlc)
        {
            _targetPlc = targetPlc ?? throw new ArgumentNullException(nameof(targetPlc));
            _testPlc = testPlc ?? throw new ArgumentNullException(nameof(testPlc));
        }

        public void StartAlarmValueMonitoring(ChannelMapping channel, Action<float?, float?, float?, float?> updateAction)
        {
            StopAlarmValueMonitoring();
            if (channel == null || updateAction == null) return;

            _alarmMonitorChannel = channel;
            _alarmUpdateAction = updateAction;
            _alarmMonitorCts = new CancellationTokenSource();
            _alarmMonitorTask = Task.Run(() => AlarmMonitorLoopAsync(_alarmMonitorCts.Token));
        }

        public void StartAOValueMonitoring(ChannelMapping channel, Action<string> updateAction)
        {
            StopAOValueMonitoring();
            if (channel == null || updateAction == null) return;

            _aoMonitorChannel = channel;
            _aoUpdateAction = updateAction;
            _aoMonitorCts = new CancellationTokenSource();
            _aoMonitorTask = Task.Run(() => AOMonitorLoopAsync(_aoMonitorCts.Token));
        }

        public void StartDOValueMonitoring(ChannelMapping channel, Action<string> updateAction)
        {
            StopDOValueMonitoring();
            if (channel == null || updateAction == null) return;

            _doMonitorChannel = channel;
            _doUpdateAction = updateAction;
            _doMonitorCts = new CancellationTokenSource();
            _doMonitorTask = Task.Run(() => DOMonitorLoopAsync(_doMonitorCts.Token));
        }

        public async Task<bool> SendAITestValueAsync(ChannelMapping channel, float testValue)
        {
            try
            {
                if (channel == null)
                {
                    System.Diagnostics.Debug.WriteLine("SendAITestValueAsync: 通道参数为空");
                    return false;
                }

                if (string.IsNullOrEmpty(channel.TestPLCCommunicationAddress))
                {
                    System.Diagnostics.Debug.WriteLine($"SendAITestValueAsync: 通道 {channel.VariableName} 的测试PLC通信地址为空");
                    return false;
                }

                // 将工程值转换为百分比值
                float percentageValue = ConvertRealValueToPercentage(channel, testValue);
                
                // 移除地址前缀并写入测试PLC
                string address = channel.TestPLCCommunicationAddress.Substring(1);
                var writeResult = await _testPlc.WriteAnalogValueAsync(address, percentageValue);

                if (!writeResult.IsSuccess)
                {
                    System.Diagnostics.Debug.WriteLine($"SendAITestValueAsync: 写入测试PLC失败 - {writeResult.ErrorMessage}");
                    return false;
                }

                System.Diagnostics.Debug.WriteLine($"SendAITestValueAsync: 成功发送测试值 {testValue} -> {percentageValue}% 到地址 {address}");
                return true;
            }
            catch (Exception ex)
            {
                System.Diagnostics.Debug.WriteLine($"SendAITestValueAsync 异常: {ex.Message}");
                return false;
            }
        }

        public async Task<bool> SendDITestSignalAsync(ChannelMapping channel)
        {
            try
            {
                if (channel == null)
                {
                    System.Diagnostics.Debug.WriteLine("SendDITestSignalAsync: 通道参数为空");
                    return false;
                }

                if (string.IsNullOrEmpty(channel.TestPLCCommunicationAddress))
                {
                    System.Diagnostics.Debug.WriteLine($"SendDITestSignalAsync: 通道 {channel.VariableName} 的测试PLC通信地址为空");
                    return false;
                }

                // 将DI信号设置为激活状态（true）
                var writeResult = await _testPlc.WriteDigitalValueAsync(channel.TestPLCCommunicationAddress, true);

                if (!writeResult.IsSuccess)
                {
                    System.Diagnostics.Debug.WriteLine($"SendDITestSignalAsync: 写入测试PLC失败 - {writeResult.ErrorMessage}");
                    return false;
                }

                System.Diagnostics.Debug.WriteLine($"SendDITestSignalAsync: 成功发送DI测试信号（激活）到地址 {channel.TestPLCCommunicationAddress}");
                return true;
            }
            catch (Exception ex)
            {
                System.Diagnostics.Debug.WriteLine($"SendDITestSignalAsync 异常: {ex.Message}");
                return false;
            }
        }

        public async Task<bool> ResetDITestSignalAsync(ChannelMapping channel)
        {
            try
            {
                if (channel == null)
                {
                    System.Diagnostics.Debug.WriteLine("ResetDITestSignalAsync: 通道参数为空");
                    return false;
                }

                if (string.IsNullOrEmpty(channel.TestPLCCommunicationAddress))
                {
                    System.Diagnostics.Debug.WriteLine($"ResetDITestSignalAsync: 通道 {channel.VariableName} 的测试PLC通信地址为空");
                    return false;
                }

                // 将DI信号设置为非激活状态（false）
                var writeResult = await _testPlc.WriteDigitalValueAsync(channel.TestPLCCommunicationAddress, false);

                if (!writeResult.IsSuccess)
                {
                    System.Diagnostics.Debug.WriteLine($"ResetDITestSignalAsync: 写入测试PLC失败 - {writeResult.ErrorMessage}");
                    return false;
                }

                System.Diagnostics.Debug.WriteLine($"ResetDITestSignalAsync: 成功复位DI测试信号（非激活）到地址 {channel.TestPLCCommunicationAddress}");
                return true;
            }
            catch (Exception ex)
            {
                System.Diagnostics.Debug.WriteLine($"ResetDITestSignalAsync 异常: {ex.Message}");
                return false;
            }
        }

        public void StopAll()
        {
            StopAlarmValueMonitoring();
            StopAOValueMonitoring();
            StopDOValueMonitoring();
        }

        private void StopAlarmValueMonitoring()
        {
            if (_alarmMonitorCts != null)
            {
                try { _alarmMonitorCts.Cancel(); } catch { }
                _alarmMonitorCts = null;
            }
        }

        private void StopAOValueMonitoring()
        {
            if (_aoMonitorCts != null)
            {
                try { _aoMonitorCts.Cancel(); } catch { }
                _aoMonitorCts = null;
            }
        }

        private void StopDOValueMonitoring()
        {
            if (_doMonitorCts != null)
            {
                try { _doMonitorCts.Cancel(); } catch { }
                _doMonitorCts = null;
            }
        }

        private async Task AlarmMonitorLoopAsync(CancellationToken token)
        {
            while (!token.IsCancellationRequested)
            {
                try
                {
                    float? sl  = await ReadAnalogAsync(_alarmMonitorChannel?.SLSetPointCommAddress, _targetPlc);
                    float? sll = await ReadAnalogAsync(_alarmMonitorChannel?.SLLSetPointCommAddress, _targetPlc);
                    float? sh  = await ReadAnalogAsync(_alarmMonitorChannel?.SHSetPointCommAddress, _targetPlc);
                    float? shh = await ReadAnalogAsync(_alarmMonitorChannel?.SHHSetPointCommAddress, _targetPlc);

                    _alarmUpdateAction?.Invoke(sl, sll, sh, shh);
                }
                catch (Exception ex)
                {
                    System.Diagnostics.Debug.WriteLine($"AlarmMonitorLoopAsync Error: {ex.Message}");
                }
                await Task.Delay(500, token).ContinueWith(_ => { });
            }
        }

        private async Task AOMonitorLoopAsync(CancellationToken token)
        {
            while (!token.IsCancellationRequested)
            {
                try
                {
                    string aoValue = await ReadAOValueAsync(_aoMonitorChannel);
                    _aoUpdateAction?.Invoke(aoValue);
                }
                catch (Exception ex)
                {
                    System.Diagnostics.Debug.WriteLine($"AOMonitorLoopAsync Error: {ex.Message}");
                    _aoUpdateAction?.Invoke("监控异常");
                }
                await Task.Delay(500, token).ContinueWith(_ => { });
            }
        }

        private async Task<string> ReadAOValueAsync(ChannelMapping channel)
        {
            try
            {
                if (channel == null)
                {
                    return "通道参数为空";
                }

                if (string.IsNullOrEmpty(channel.TestPLCCommunicationAddress))
                {
                    return "反馈点地址无效";
                }

                // 从测试PLC读取AO的百分比值
                string address = channel.TestPLCCommunicationAddress.Substring(1);
                var readResult = await _testPlc.ReadAnalogValueAsync(address);

                if (!readResult.IsSuccess)
                {
                    return "读取失败";
                }

                // 将百分比值转换为工程值
                float engineeringValue = ConvertPercentageToRealValue(channel, readResult.Data);
                return engineeringValue.ToString("F3");
            }
            catch (Exception ex)
            {
                System.Diagnostics.Debug.WriteLine($"ReadAOValueAsync 异常: {ex.Message}");
                return "读取异常";
            }
        }

        private async Task<float?> ReadAnalogAsync(string address, IPlcCommunication plc)
        {
            try
            {
                if (string.IsNullOrEmpty(address)) return null;
                var result = await plc.ReadAnalogValueAsync(address.Substring(1));
                return result.IsSuccess ? result.Data : (float?)null;
            }
            catch
            {
                return null;
            }
        }

        /// <summary>
        /// 将工程值转换为百分比值
        /// </summary>
        /// <param name="channel">通道映射信息</param>
        /// <param name="realValue">工程值</param>
        /// <returns>百分比值</returns>
        private float ConvertRealValueToPercentage(ChannelMapping channel, float realValue)
        {
            try
            {
                if (!channel.RangeLowerLimitValue.HasValue || !channel.RangeUpperLimitValue.HasValue)
                {
                    System.Diagnostics.Debug.WriteLine($"ConvertRealValueToPercentage: 通道 {channel.VariableName} 的量程值未设置");
                    return realValue; // 如果没有量程信息，直接返回原值
                }

                float lowerLimit = channel.RangeLowerLimitValue.Value;
                float upperLimit = channel.RangeUpperLimitValue.Value;
                float range = upperLimit - lowerLimit;

                if (Math.Abs(range) < 1e-6f) // 避免除零
                {
                    System.Diagnostics.Debug.WriteLine($"ConvertRealValueToPercentage: 通道 {channel.VariableName} 的量程范围为零");
                    return 0f;
                }

                // 计算百分比值
                float percentage = ((realValue - lowerLimit) / range) * 100.0f;
                
                System.Diagnostics.Debug.WriteLine($"ConvertRealValueToPercentage: {realValue} -> {percentage}% (范围: {lowerLimit} - {upperLimit})");
                return percentage;
            }
            catch (Exception ex)
            {
                System.Diagnostics.Debug.WriteLine($"ConvertRealValueToPercentage 异常: {ex.Message}");
                return realValue; // 出错时返回原值
            }
        }

        /// <summary>
        /// 将百分比值转换为工程值
        /// </summary>
        /// <param name="channel">通道映射信息</param>
        /// <param name="percentageValue">百分比值</param>
        /// <returns>工程值</returns>
        private float ConvertPercentageToRealValue(ChannelMapping channel, float percentageValue)
        {
            try
            {
                if (!channel.RangeLowerLimitValue.HasValue || !channel.RangeUpperLimitValue.HasValue)
                {
                    System.Diagnostics.Debug.WriteLine($"ConvertPercentageToRealValue: 通道 {channel.VariableName} 的量程值未设置");
                    return percentageValue; // 如果没有量程信息，直接返回原值
                }

                float lowerLimit = channel.RangeLowerLimitValue.Value;
                float upperLimit = channel.RangeUpperLimitValue.Value;
                float range = upperLimit - lowerLimit;

                // 计算工程值
                float engineeringValue = lowerLimit + (range * percentageValue / 100.0f);
                
                System.Diagnostics.Debug.WriteLine($"ConvertPercentageToRealValue: {percentageValue}% -> {engineeringValue} (范围: {lowerLimit} - {upperLimit})");
                return engineeringValue;
            }
            catch (Exception ex)
            {
                System.Diagnostics.Debug.WriteLine($"ConvertPercentageToRealValue 异常: {ex.Message}");
                return percentageValue; // 出错时返回原值
            }
        }

        private async Task DOMonitorLoopAsync(CancellationToken token)
        {
            while (!token.IsCancellationRequested)
            {
                try
                {
                    string doValue = await ReadDOValueAsync(_doMonitorChannel);
                    _doUpdateAction?.Invoke(doValue);
                }
                catch (Exception ex)
                {
                    System.Diagnostics.Debug.WriteLine($"DOMonitorLoopAsync Error: {ex.Message}");
                    _doUpdateAction?.Invoke("监控异常");
                }
                await Task.Delay(500, token).ContinueWith(_ => { });
            }
        }

        private async Task<string> ReadDOValueAsync(ChannelMapping channel)
        {
            try
            {
                if (channel == null)
                {
                    return "通道参数为空";
                }

                if (string.IsNullOrEmpty(channel.TestPLCCommunicationAddress))
                {
                    return "反馈点地址无效";
                }

                // 从测试PLC读取DO的反馈状态（通常是读取测试PLC上连接到此DO输出的DI点）
                string address = channel.TestPLCCommunicationAddress.Substring(1);
                var readResult = await _testPlc.ReadDigitalValueAsync(address);

                if (!readResult.IsSuccess)
                {
                    return "读取失败";
                }

                // 将数字量值转换为状态字符串
                return readResult.Data ? "ON" : "OFF";
            }
            catch (Exception ex)
            {
                System.Diagnostics.Debug.WriteLine($"ReadDOValueAsync 异常: {ex.Message}");
                return "读取异常";
            }
        }
    }
} 