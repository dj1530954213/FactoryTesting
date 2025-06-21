using System;
using FatFullVersion.Entities.EntitiesEnum;

namespace FatFullVersion.Entities.ValueObject
{
    /// <summary>
    /// 通道比较表，用于配置测试PLC通道与通讯地址的映射关系
    /// </summary>
    public class ComparisonTable : IComparable<ComparisonTable>
    {
        /// <summary>
        /// 主键ID
        /// </summary>
        public int Id { get; set; }

        /// <summary>
        /// 通道地址
        /// </summary>
        public string ChannelAddress { get; set; }

        /// <summary>
        /// 通讯地址
        /// </summary>
        public string CommunicationAddress { get; set; }

        /// <summary>
        /// 通道类型
        /// </summary>
        public TestPlcChannelType ChannelType { get; set; }

        /// <summary>
        /// 供电类型
        /// </summary>
        public string PowerSupplyType { get; set; }

        /// <summary>
        /// 无参构造函数，供EF Core使用
        /// </summary>
        public ComparisonTable()
        {
            ChannelAddress = string.Empty;
            CommunicationAddress = string.Empty;
        }

        /// <summary>
        /// 带参数构造函数
        /// </summary>
        /// <param name="channelAddress">通道地址</param>
        /// <param name="communicationAddress">通讯地址</param>
        /// <param name="channelType">通道类型</param>
        public ComparisonTable(string channelAddress, string communicationAddress, TestPlcChannelType channelType)
        {
            ChannelAddress = channelAddress ?? string.Empty;
            CommunicationAddress = communicationAddress ?? string.Empty;
            ChannelType = channelType;
        }

        /// <summary>
        /// 实现IComparable接口的比较方法
        /// </summary>
        /// <param name="other">要比较的另一个通道比较表项</param>
        /// <returns>比较结果</returns>
        public int CompareTo(ComparisonTable? other)
        {
            if (other == null) return 1;

            // 首先按通道类型比较（使用枚举的整数值）
            int typeComparison = ((int)ChannelType).CompareTo((int)other.ChannelType);
            if (typeComparison != 0) return typeComparison;

            // 然后按通道地址比较
            int channelAddressComparison = string.Compare(ChannelAddress, other.ChannelAddress, StringComparison.Ordinal);
            if (channelAddressComparison != 0) return channelAddressComparison;

            // 最后按通讯地址比较
            return string.Compare(CommunicationAddress, other.CommunicationAddress, StringComparison.Ordinal);
        }

        public override bool Equals(object? obj)
        {
            if (obj is not ComparisonTable other) return false;
            return Id == other.Id &&
                   ChannelAddress == other.ChannelAddress &&
                   CommunicationAddress == other.CommunicationAddress &&
                   ChannelType == other.ChannelType;
        }

        public override int GetHashCode()
        {
            return HashCode.Combine(Id, ChannelAddress, CommunicationAddress, ChannelType);
        }
    }
}
