using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Threading.Tasks;

namespace FatFullVersion.IServices
{
    /// <summary>
    /// PLC通信接口，定义与PLC进行通信的方法
    /// </summary>
    public interface IPlcCommunication
    {
        /// <summary>
        /// 连接到PLC
        /// </summary>
        /// <returns>连接操作结果</returns>
        Task<PlcCommunicationResult> ConnectAsync();

        /// <summary>
        /// 断开与PLC的连接
        /// </summary>
        /// <returns>断开连接操作结果</returns>
        Task<PlcCommunicationResult> DisconnectAsync();

        /// <summary>
        /// 读取模拟量值
        /// </summary>
        /// <param name="address">地址</param>
        /// <returns>读取操作结果，包含读取到的模拟量值</returns>
        Task<PlcCommunicationResult<float>> ReadAnalogValueAsync(string address);

        /// <summary>
        /// 写入模拟量值
        /// </summary>
        /// <param name="address">地址</param>
        /// <param name="value">要写入的值</param>
        /// <returns>写入操作结果</returns>
        Task<PlcCommunicationResult> WriteAnalogValueAsync(string address, float value);

        /// <summary>
        /// 读取数字量值
        /// </summary>
        /// <param name="address">地址</param>
        /// <returns>读取操作结果，包含读取到的数字量值，true表示ON，false表示OFF</returns>
        Task<PlcCommunicationResult<bool>> ReadDigitalValueAsync(string address);

        /// <summary>
        /// 写入数字量值
        /// </summary>
        /// <param name="address">地址</param>
        /// <param name="value">要写入的值，true表示ON，false表示OFF</param>
        /// <returns>写入操作结果</returns>
        Task<PlcCommunicationResult> WriteDigitalValueAsync(string address, bool value);

        /// <summary>
        /// 获取连接状态
        /// </summary>
        /// <returns>连接是否正常</returns>
        bool IsConnected { get; }

        /// <summary>
        /// 获取PLC信息
        /// </summary>
        /// <returns>PLC信息操作结果，包含PLC信息字符串</returns>
        PlcCommunicationResult<string> GetPlcInfo();
    }
} 