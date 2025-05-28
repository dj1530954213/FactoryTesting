using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Threading.Tasks;
using FatFullVersion.Entities.EntitiesEnum;
using FatFullVersion.Entities.ValueObject;

namespace FatFullVersion.Entities
{
    /// <summary>
    /// 测试PLC相关配置信息
    /// </summary>
    public class TestPlcConfig
    {
        //测试PLC型号
        public PlcBrandTypeEnum BrandType { get; set; }
        //IP地址
        public string IpAddress { get; set; }
        //通道与通讯地址的对应关系表
        public List<ComparisonTable> CommentsTables { get; set; }
    }
}
