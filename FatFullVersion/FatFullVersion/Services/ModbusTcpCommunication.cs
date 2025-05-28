using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.Linq;
using System.Text;
using System.Threading.Tasks;
using System.Windows;
using DCMCAJ.ModBus;
using DCMCAJ.Core.Pipe;
using DCMCAJ;
using FatFullVersion.IServices;
using FatFullVersion.Entities;
using DataFormat = DCMCAJ.Core.DataFormat;
using Prism.Mvvm;

namespace FatFullVersion.Services
{
    /// <summary>
    /// Modbus TCP通信实现类，提供与PLC通信的具体实现
    /// </summary>
    public class ModbusTcpCommunication : BindableBase, IPlcCommunication
    {
        /// <summary>
        /// Modbus TCP通信对象
        /// </summary>
        private DCMCAJ.ModBus.ModbusTcpNet _modbus;
        
        /// <summary>
        /// 仓储层接口
        /// </summary>
        private readonly IRepository _repository;
        
        /// <summary>
        /// PLC连接配置
        /// </summary>
        private PlcConnectionConfig _connectionConfig;

        /// <summary>
        /// 是否是测试PLC
        /// </summary>
        private readonly bool _isTestPlc;

        /// <summary>
        /// 跳出循环读取使用
        /// </summary>
        private bool _manualDisConnect;

        private bool _isConnected;
        /// <summary>
        /// 连接状态
        /// </summary>
        public bool IsConnected 
        { 
            get => _isConnected; 
            private set => SetProperty(ref _isConnected, value); 
        }

        /// <summary>
        /// 构造函数
        /// </summary>
        /// <param name="repository">仓储层接口</param>
        /// <param name="isTestPlc">是否是测试PLC，true表示测试PLC，false表示被测PLC</param>
        public ModbusTcpCommunication(IRepository repository, bool isTestPlc = true)
        {
            _repository = repository ?? throw new ArgumentNullException(nameof(repository));
            _isTestPlc = isTestPlc;
            _modbus = new DCMCAJ.ModBus.ModbusTcpNet();
            IsConnected = false;
        }

        /// <summary>
        /// 连接到PLC
        /// </summary>
        /// <returns>连接操作结果</returns>
        public async Task<PlcCommunicationResult> ConnectAsync()
        {
            try
            {
                // 根据PLC类型从仓储层获取对应的连接配置
                _connectionConfig = _isTestPlc
                    ? await _repository.GetTestPlcConnectionConfigAsync()
                    : await _repository.GetTargetPlcConnectionConfigAsync();
                
                // 配置Modbus连接参数
                _modbus.Station = _connectionConfig.Station;
                _modbus.AddressStartWithZero = _connectionConfig.AddressStartWithZero;
                _modbus.IsCheckMessageId = _connectionConfig.IsCheckMessageId;
                _modbus.IsStringReverse = _connectionConfig.IsStringReverse;
                
                // 设置数据格式
                _modbus.DataFormat = ConvertToDataFormat(_connectionConfig.DataFormat);
                
                // 配置通信管道
                _modbus.CommunicationPipe = new DCMCAJ.Core.Pipe.PipeTcpNet(_connectionConfig.IpAddress, _connectionConfig.Port)
                {
                    ConnectTimeOut = _connectionConfig.ConnectTimeOut,
                    ReceiveTimeOut = _connectionConfig.ReceiveTimeOut,
                    SleepTime = _connectionConfig.SleepTime,
                    SocketKeepAliveTime = _connectionConfig.SocketKeepAliveTime,
                    IsPersistentConnection = _connectionConfig.IsPersistentConnection,
                };

                // 执行连接
                var result = await _modbus.ConnectServerAsync();
                if (result.IsSuccess)
                {
                    IsConnected = true;
                    CheckConnection(_connectionConfig.KeepConnectionAliveTag);
                    return PlcCommunicationResult.CreateSuccessResult();
                }
                else
                {
                    IsConnected = false;
                    return PlcCommunicationResult.CreateFailedResult(result.Message, result.ErrorCode);
                }
            }
            catch (Exception ex)
            {
                IsConnected = false;
                return PlcCommunicationResult.CreateExceptionResult(ex);
            }
        }
        /// <summary>
        /// 重连接
        /// </summary>
        /// <returns></returns>
        public async Task<PlcCommunicationResult> ReConnectAsync()
        {
            var result = await _modbus.ConnectServerAsync();
            if (result.IsSuccess)
            {
                IsConnected = true;
                _manualDisConnect = false;
                return PlcCommunicationResult.CreateSuccessResult();
            }
            else
            {
                IsConnected = false;
                return PlcCommunicationResult.CreateFailedResult(result.Message, result.ErrorCode);
            }
        }
        /// <summary>
        /// 循环检查连接是否正常
        /// </summary>
        /// <param name="defultAddress"></param>
        public void CheckConnection(string keepAliveTag)
        {
            Task.Run(async () =>
            {
                string address = keepAliveTag;
                while (true)
                {
                    var result = await ReadDigitalValueAsync(address);
                    bool connected = result.IsSuccess;
                    
                    // 在UI线程上更新属性以确保正确触发通知
                    if (connected != IsConnected)
                    {
                        // 使用Dispatcher确保在UI线程上更新属性
                        System.Windows.Application.Current.Dispatcher.Invoke(() =>
                        {
                            IsConnected = connected;
                        });
                    }
                    
                    await Task.Delay(500);
                    if (_manualDisConnect)
                    {
                        break;
                    }
                }
            });
        }

        /// <summary>
        /// 断开与PLC的连接
        /// </summary>
        /// <returns>断开连接操作结果</returns>
        public async Task<PlcCommunicationResult> DisconnectAsync()
        {
            try
            {
                if (IsConnected)
                {
                    var result = await _modbus.ConnectCloseAsync();
                    if (result.IsSuccess)
                    {
                        IsConnected = false;
                        _manualDisConnect = true;
                        return PlcCommunicationResult.CreateSuccessResult();
                    }
                    else
                    {
                        return PlcCommunicationResult.CreateFailedResult(result.Message, result.ErrorCode);
                    }
                }
                else
                {
                    return PlcCommunicationResult.CreateSuccessResult();
                }
            }
            catch (Exception ex)
            {
                return PlcCommunicationResult.CreateExceptionResult(ex);
            }
            finally
            {
                // 无论如何都确保连接状态更新为断开
                IsConnected = false;
            }
        }

        /// <summary>
        /// 读取模拟量值
        /// </summary>
        /// <param name="address">地址</param>
        /// <returns>读取操作结果，包含读取到的模拟量值</returns>
        public async Task<PlcCommunicationResult<float>> ReadAnalogValueAsync(string address)
        {
            try
            {
                if (!IsConnected)
                {
                    var connectResult = await ReConnectAsync();
                    if (!connectResult.IsSuccess)
                    {
                        return PlcCommunicationResult<float>.CreateFailedResult(
                            $"读取失败: 未连接到PLC。连接错误: {connectResult.ErrorMessage}", 
                            connectResult.ErrorCode);
                    }
                }

                var result = await _modbus.ReadFloatAsync(address);
                if (result.IsSuccess)
                {
                    return PlcCommunicationResult<float>.CreateSuccessResult(result.Content);
                }
                else
                {
                    return PlcCommunicationResult<float>.CreateFailedResult($"读取失败: {result.Message}", result.ErrorCode);
                }
            }
            catch (Exception ex)
            {
                IsConnected = false; // 发生异常时假设连接已断开
                return PlcCommunicationResult<float>.CreateExceptionResult(ex);
            }
        }

        /// <summary>
        /// 写入模拟量值
        /// </summary>
        /// <param name="address">地址</param>
        /// <param name="value">要写入的值</param>
        /// <returns>写入操作结果</returns>
        public async Task<PlcCommunicationResult> WriteAnalogValueAsync(string address, float value)
        {
            try
            {
                if (!IsConnected)
                {
                    var connectResult = await ReConnectAsync();
                    if (!connectResult.IsSuccess)
                    {
                        return PlcCommunicationResult.CreateFailedResult(
                            $"写入失败: 未连接到PLC。连接错误: {connectResult.ErrorMessage}", 
                            connectResult.ErrorCode);
                    }
                }

                var result = await _modbus.WriteAsync(address, value);
                if (result.IsSuccess)
                {
                    return PlcCommunicationResult.CreateSuccessResult();
                }
                else
                {
                    return PlcCommunicationResult.CreateFailedResult($"写入失败: {result.Message}", result.ErrorCode);
                }
            }
            catch (Exception ex)
            {
                IsConnected = false; // 发生异常时假设连接已断开
                return PlcCommunicationResult.CreateExceptionResult(ex);
            }
        }

        /// <summary>
        /// 读取数字量值
        /// </summary>
        /// <param name="address">地址</param>
        /// <returns>读取操作结果，包含读取到的数字量值</returns>
        public async Task<PlcCommunicationResult<bool>> ReadDigitalValueAsync(string address)
        {
            try
            {
                if (!IsConnected)
                {
                    var connectResult = await ReConnectAsync();
                    if (!connectResult.IsSuccess)
                    {
                        return PlcCommunicationResult<bool>.CreateFailedResult(
                            $"读取失败: 未连接到PLC。连接错误: {connectResult.ErrorMessage}", 
                            connectResult.ErrorCode);
                    }
                }

                var result = await _modbus.ReadBoolAsync(address);
                if (result.IsSuccess)
                {
                    return PlcCommunicationResult<bool>.CreateSuccessResult(result.Content);
                }
                else
                {
                    return PlcCommunicationResult<bool>.CreateFailedResult($"读取失败: {result.Message}", result.ErrorCode);
                }
            }
            catch (Exception ex)
            {
                IsConnected = false; // 发生异常时假设连接已断开
                return PlcCommunicationResult<bool>.CreateExceptionResult(ex);
            }
        }

        /// <summary>
        /// 写入数字量值
        /// </summary>
        /// <param name="address">地址</param>
        /// <param name="value">要写入的值</param>
        /// <returns>写入操作结果</returns>
        public async Task<PlcCommunicationResult> WriteDigitalValueAsync(string address, bool value)
        {
            try
            {
                if (!IsConnected)
                {
                    var connectResult = await ReConnectAsync();
                    if (!connectResult.IsSuccess)
                    {
                        return PlcCommunicationResult.CreateFailedResult(
                            $"写入失败: 未连接到PLC。连接错误: {connectResult.ErrorMessage}", 
                            connectResult.ErrorCode);
                    }
                }

                var result = await _modbus.WriteAsync(address, value);
                if (result.IsSuccess)
                {
                    return PlcCommunicationResult.CreateSuccessResult();
                }
                else
                {
                    return PlcCommunicationResult.CreateFailedResult($"写入失败: {result.Message}", result.ErrorCode);
                }
            }
            catch (Exception ex)
            {
                IsConnected = false; // 发生异常时假设连接已断开
                return PlcCommunicationResult.CreateExceptionResult(ex);
            }
        }

        /// <summary>
        /// 获取PLC信息
        /// </summary>
        /// <returns>PLC信息操作结果，包含PLC信息字符串</returns>
        public PlcCommunicationResult<string> GetPlcInfo()
        {
            try
            {
                if (!IsConnected)
                {
                    return PlcCommunicationResult<string>.CreateFailedResult("未连接到PLC");
                }

                string info = $"PLC连接信息 - IP地址: {_connectionConfig.IpAddress}, 端口: {_connectionConfig.Port}, " +
                             $"站号: {_connectionConfig.Station}, 连接状态: {(IsConnected ? "已连接" : "未连接")}";
                
                return PlcCommunicationResult<string>.CreateSuccessResult(info);
            }
            catch (Exception ex)
            {
                return PlcCommunicationResult<string>.CreateExceptionResult(ex);
            }
        }

        /// <summary>
        /// 将字符串格式的数据格式转换为DCMCAJ.Core.DataFormat枚举
        /// </summary>
        /// <param name="dataFormatString">数据格式字符串</param>
        /// <returns>DataFormat枚举值</returns>
        private DataFormat ConvertToDataFormat(string dataFormatString)
        {
            switch (dataFormatString?.ToUpper())
            {
                case "ABCD":
                    return DataFormat.ABCD;
                case "BADC":
                    return DataFormat.BADC;
                case "CDAB":
                    return DataFormat.CDAB;
                case "DCBA":
                    return DataFormat.DCBA;
                default:
                    return DataFormat.ABCD; // 默认格式
            }
        }
    }
}
