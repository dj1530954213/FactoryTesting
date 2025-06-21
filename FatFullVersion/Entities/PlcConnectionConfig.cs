using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Threading.Tasks;

namespace FatFullVersion.Entities
{
    /// <summary>
    /// PLC连接配置
    /// </summary>
    public class PlcConnectionConfig : IComparable<PlcConnectionConfig>
    {
        public int Id { get; set; }
        public string IpAddress { get; set; } = "127.0.0.1";
        public int Port { get; set; } = 502;
        public byte Station { get; set; } = 1;
        public bool AddressStartWithZero { get; set; } = false;
        public bool IsCheckMessageId { get; set; } = true;
        public bool IsStringReverse { get; set; } = false;
        public string DataFormat { get; set; } = "ABCD";
        public int ConnectTimeOut { get; set; } = 5000;
        public int ReceiveTimeOut { get; set; } = 10000;
        public int SleepTime { get; set; } = 0;
        public int SocketKeepAliveTime { get; set; } = -1;
        public bool IsTestPlc { get; set; }
        public bool IsPersistentConnection { get; set; } = true;
        public string KeepConnectionAliveTag { get; set; } = "101";

        public int CompareTo(PlcConnectionConfig? other)
        {
            if (other == null) return 1;
            int idComparison = Id.CompareTo(other.Id);
            if (idComparison != 0) return idComparison;
            int ipComparison = string.Compare(IpAddress, other.IpAddress, StringComparison.Ordinal);
            if (ipComparison != 0) return ipComparison;
            return Port.CompareTo(other.Port);
        }
    }
}