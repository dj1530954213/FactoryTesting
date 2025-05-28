using System;
using FatFullVersion.Entities.EntitiesEnum;
using FatFullVersion.IServices;

namespace FatFullVersion.Services
{
    /// <summary>
    /// PLC通信工厂类，用于创建和管理不同类型的PLC通信实例
    /// </summary>
    public class PlcCommunicationFactory
    {
        private readonly IRepository _repository;
        private readonly PlcType _plcType;

        /// <summary>
        /// 构造函数
        /// </summary>
        /// <param name="repository">仓储层接口</param>
        /// <param name="plcType">PLC类型</param>
        public PlcCommunicationFactory(IRepository repository, PlcType plcType)
        {
            _repository = repository ?? throw new ArgumentNullException(nameof(repository));
            _plcType = plcType;
        }

        /// <summary>
        /// 创建PLC通信实例
        /// </summary>
        /// <returns>PLC通信实例</returns>
        public IPlcCommunication CreatePlcCommunication()
        {
            // 根据PLC类型创建不同的通信实例
            switch (_plcType)
            {
                case PlcType.TestPlc:
                    // 测试PLC使用测试PLC的配置
                    return new ModbusTcpCommunication(_repository, isTestPlc: true);
                case PlcType.TargetPlc:
                    // 被测PLC使用被测PLC的配置
                    return new ModbusTcpCommunication(_repository, isTestPlc: false);
                default:
                    throw new ArgumentException($"不支持的PLC类型: {_plcType}");
            }
        }
    }
} 