using System;
using System.Threading.Tasks;
using FatFullVersion.Models;

namespace FatFullVersion.IServices
{
    /// <summary>
    /// Manual Test I/O Service
    /// 负责手动测试场景下与PLC进行数据交互，例如：
    /// 1. 周期性读取 AI/AO/DO/DI 等通道的反馈或设定值（从目标PLC读取）。
    /// 2. 发送数字/模拟量命令以驱动被测或测试设备（写入测试PLC）。
    /// 本阶段实现 AI 报警设定值监控（从目标PLC读取）、测试值下发功能（写入测试PLC）、AO 数值监控、DI 测试信号控制和 DO 数值监控。
    /// </summary>
    public interface IManualTestIoService
    {
        /// <summary>
        /// 启动对指定 AI 通道报警设定值的监控，周期 0.5s（从目标PLC读取）。
        /// </summary>
        /// <param name="channel">目标通道</param>
        /// <param name="updateAction">读取结果回调 (SL, SLL, SH, SHH)</param>
        void StartAlarmValueMonitoring(ChannelMapping channel, Action<float?, float?, float?, float?> updateAction);

        /// <summary>
        /// 启动对指定 AO 通道数值的监控，周期 0.5s（从测试PLC读取）。
        /// </summary>
        /// <param name="channel">目标 AO 通道</param>
        /// <param name="updateAction">读取结果回调 (当前工程值)</param>
        void StartAOValueMonitoring(ChannelMapping channel, Action<string> updateAction);

        /// <summary>
        /// 启动对指定 DO 通道数值的监控，周期 0.5s（从测试PLC读取）。
        /// </summary>
        /// <param name="channel">目标 DO 通道</param>
        /// <param name="updateAction">读取结果回调 (当前状态值："ON"/"OFF"/"读取失败"等)</param>
        void StartDOValueMonitoring(ChannelMapping channel, Action<string> updateAction);

        /// <summary>
        /// 发送 AI 测试值到测试 PLC。
        /// </summary>
        /// <param name="channel">目标 AI 通道</param>
        /// <param name="testValue">要发送的测试值（工程值）</param>
        /// <returns>发送操作的结果</returns>
        Task<bool> SendAITestValueAsync(ChannelMapping channel, float testValue);

        /// <summary>
        /// 发送 DI 测试信号到测试 PLC（设置为激活状态）。
        /// </summary>
        /// <param name="channel">目标 DI 通道</param>
        /// <returns>发送操作的结果</returns>
        Task<bool> SendDITestSignalAsync(ChannelMapping channel);

        /// <summary>
        /// 复位 DI 测试信号（设置为非激活状态）。
        /// </summary>
        /// <param name="channel">目标 DI 通道</param>
        /// <returns>复位操作的结果</returns>
        Task<bool> ResetDITestSignalAsync(ChannelMapping channel);

        /// <summary>
        /// 停止所有监控/交互任务。
        /// </summary>
        void StopAll();
    }
} 