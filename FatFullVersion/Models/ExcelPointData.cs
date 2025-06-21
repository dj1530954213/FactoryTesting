using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Threading.Tasks;

namespace FatFullVersion.Models
{
    /// <summary>
    /// Excel点表数据模型，用于从导入的Excel文件映射点位数据
    /// 获取完数据之后需要直接存入数据库，后续的操作只在这个模型中增加字段
    /// </summary>
    public class ExcelPointData
    {
        /// <summary>
        /// 序号
        /// </summary>
        public int SerialNumber { get; set; }
        
        /// <summary>
        /// 模块名称
        /// </summary>
        public string ModuleName { get; set; }
        
        /// <summary>
        /// 模块类型
        /// </summary>
        public string ModuleType { get; set; }
        
        /// <summary>
        /// 供电类型（有源/无源）
        /// </summary>
        public string PowerSupplyType { get; set; }
        
        /// <summary>
        /// 线制
        /// </summary>
        public string WireSystem { get; set; }
        
        /// <summary>
        /// 通道位号
        /// </summary>
        public string ChannelTag { get; set; }
        
        /// <summary>
        /// 位号
        /// </summary>
        public string Tag { get; set; }
        
        /// <summary>
        /// 场站名
        /// </summary>
        public string StationName { get; set; }
        
        /// <summary>
        /// 变量名称（HMI）
        /// </summary>
        public string VariableName { get; set; }
        
        /// <summary>
        /// 变量描述
        /// </summary>
        public string VariableDescription { get; set; }
        
        /// <summary>
        /// 数据类型
        /// </summary>
        public string DataType { get; set; }
        
        /// <summary>
        /// 读写属性
        /// </summary>
        public string AccessProperty { get; set; }
        
        /// <summary>
        /// 保存历史
        /// </summary>
        public string SaveHistory { get; set; }
        
        /// <summary>
        /// 掉电保护
        /// </summary>
        public string PowerFailureProtection { get; set; }
        
        /// <summary>
        /// 量程低限
        /// </summary>
        public string RangeLowerLimit { get; set; }
        
        /// <summary>
        /// 量程低限数值
        /// </summary>
        public float RangeLowerLimitValue 
        { 
            get 
            {
                if (float.TryParse(RangeLowerLimit, out float result))
                {
                    return result;
                }
                return 0;
            } 
        }
        
        /// <summary>
        /// 量程高限
        /// </summary>
        public string RangeUpperLimit { get; set; }
        
        /// <summary>
        /// 量程高限数值
        /// </summary>
        public float RangeUpperLimitValue 
        { 
            get 
            {
                if (float.TryParse(RangeUpperLimit, out float result))
                {
                    return result;
                }
                return 100;
            } 
        }
        
        /// <summary>
        /// SLL设定值
        /// </summary>
        public string SLLSetValue { get; set; }
        
        /// <summary>
        /// SLL设定值数值
        /// </summary>
        public float SLLSetValueNumber
        {
            get
            {
                if (float.TryParse(SLLSetValue, out float result))
                {
                    return result;
                }
                return 0;
            }
        }
        
        /// <summary>
        /// SLL设定点位
        /// </summary>
        public string SLLSetPoint { get; set; }
        
        /// <summary>
        /// SLL设定点位_PLC地址
        /// </summary>
        public string SLLSetPointPLCAddress { get; set; }
        
        /// <summary>
        /// SLL设定点位_通讯地址
        /// </summary>
        public string SLLSetPointCommAddress { get; set; }
        
        /// <summary>
        /// SL设定值
        /// </summary>
        public string SLSetValue { get; set; }
        
        /// <summary>
        /// SL设定值数值
        /// </summary>
        public float SLSetValueNumber
        {
            get
            {
                if (float.TryParse(SLSetValue, out float result))
                {
                    return result;
                }
                return 0;
            }
        }
        
        /// <summary>
        /// SL设定点位
        /// </summary>
        public string SLSetPoint { get; set; }
        
        /// <summary>
        /// SL设定点位_PLC地址
        /// </summary>
        public string SLSetPointPLCAddress { get; set; }
        
        /// <summary>
        /// SL设定点位_通讯地址
        /// </summary>
        public string SLSetPointCommAddress { get; set; }
        
        /// <summary>
        /// SH设定值
        /// </summary>
        public string SHSetValue { get; set; }
        
        /// <summary>
        /// SH设定值数值
        /// </summary>
        public float SHSetValueNumber
        {
            get
            {
                if (float.TryParse(SHSetValue, out float result))
                {
                    return result;
                }
                return 0;
            }
        }
        
        /// <summary>
        /// SH设定点位
        /// </summary>
        public string SHSetPoint { get; set; }
        
        /// <summary>
        /// SH设定点位_PLC地址
        /// </summary>
        public string SHSetPointPLCAddress { get; set; }
        
        /// <summary>
        /// SH设定点位_通讯地址
        /// </summary>
        public string SHSetPointCommAddress { get; set; }
        
        /// <summary>
        /// SHH设定值
        /// </summary>
        public string SHHSetValue { get; set; }
        
        /// <summary>
        /// SHH设定值数值
        /// </summary>
        public float SHHSetValueNumber
        {
            get
            {
                if (float.TryParse(SHHSetValue, out float result))
                {
                    return result;
                }
                return 0;
            }
        }
        
        /// <summary>
        /// SHH设定点位
        /// </summary>
        public string SHHSetPoint { get; set; }
        
        /// <summary>
        /// SHH设定点位_PLC地址
        /// </summary>
        public string SHHSetPointPLCAddress { get; set; }
        
        /// <summary>
        /// SHH设定点位_通讯地址
        /// </summary>
        public string SHHSetPointCommAddress { get; set; }
        
        /// <summary>
        /// LL报警
        /// </summary>
        public string LLAlarm { get; set; }
        
        /// <summary>
        /// LL报警_PLC地址
        /// </summary>
        public string LLAlarmPLCAddress { get; set; }
        
        /// <summary>
        /// LL报警_通讯地址
        /// </summary>
        public string LLAlarmCommAddress { get; set; }
        
        /// <summary>
        /// L报警
        /// </summary>
        public string LAlarm { get; set; }
        
        /// <summary>
        /// L报警_PLC地址
        /// </summary>
        public string LAlarmPLCAddress { get; set; }
        
        /// <summary>
        /// L报警_通讯地址
        /// </summary>
        public string LAlarmCommAddress { get; set; }
        
        /// <summary>
        /// H报警
        /// </summary>
        public string HAlarm { get; set; }
        
        /// <summary>
        /// H报警_PLC地址
        /// </summary>
        public string HAlarmPLCAddress { get; set; }
        
        /// <summary>
        /// H报警_通讯地址
        /// </summary>
        public string HAlarmCommAddress { get; set; }
        
        /// <summary>
        /// HH报警
        /// </summary>
        public string HHAlarm { get; set; }
        
        /// <summary>
        /// HH报警_PLC地址
        /// </summary>
        public string HHAlarmPLCAddress { get; set; }
        
        /// <summary>
        /// HH报警_通讯地址
        /// </summary>
        public string HHAlarmCommAddress { get; set; }
        
        /// <summary>
        /// 维护值设定
        /// </summary>
        public string MaintenanceValueSetting { get; set; }
        
        /// <summary>
        /// 维护值设定点位
        /// </summary>
        public string MaintenanceValueSetPoint { get; set; }
        
        /// <summary>
        /// 维护值设定点位_PLC地址
        /// </summary>
        public string MaintenanceValueSetPointPLCAddress { get; set; }
        
        /// <summary>
        /// 维护值设定点位_通讯地址
        /// </summary>
        public string MaintenanceValueSetPointCommAddress { get; set; }
        
        /// <summary>
        /// 维护使能开关点位
        /// </summary>
        public string MaintenanceEnableSwitchPoint { get; set; }
        
        /// <summary>
        /// 维护使能开关点位_PLC地址
        /// </summary>
        public string MaintenanceEnableSwitchPointPLCAddress { get; set; }
        
        /// <summary>
        /// 维护使能开关点位_通讯地址
        /// </summary>
        public string MaintenanceEnableSwitchPointCommAddress { get; set; }
        
        /// <summary>
        /// PLC绝对地址
        /// </summary>
        public string PLCAbsoluteAddress { get; set; }
        
        /// <summary>
        /// 上位机通讯地址
        /// </summary>
        public string CommunicationAddress { get; set; }
        
        /// <summary>
        /// 创建时间
        /// </summary>
        public DateTime CreatedTime { get; set; } = DateTime.Now;
        
        /// <summary>
        /// 更新时间
        /// </summary>
        public DateTime? UpdatedTime { get; set; }

        /// <summary>
        /// 获取低低限值
        /// </summary>
        public float LowLowLimit => SLLSetValueNumber;

        /// <summary>
        /// 获取低限值
        /// </summary>
        public float LowLimit => SLSetValueNumber;

        /// <summary>
        /// 获取高限值
        /// </summary>
        public float HighLimit => SHSetValueNumber;

        /// <summary>
        /// 获取高高限值
        /// </summary>
        public float HighHighLimit => SHHSetValueNumber;
    }
}
