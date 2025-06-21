using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Threading.Tasks;
using FatFullVersion.Models;
using FatFullVersion.IServices;
using System.Windows;

namespace FatFullVersion.Services
{
    public class ChannelRangerSettingService : IChannelRangerSettingService
    {
        private readonly IMessageService _messageService;

        public ChannelRangerSettingService(IMessageService messageService)
        {
            _messageService = messageService;
        }

        static Dictionary<string,string> RangerTabel = new Dictionary<string,string>
        {
            { "AO1_1_RANGE", "5601" },
            { "AO1_2_RANGE", "5603" },
            { "AO1_3_RANGE", "5605" },
            { "AO1_4_RANGE", "5607" },
            { "AO1_5_RANGE", "5609" },
            { "AO1_6_RANGE", "5611" },
            { "AO1_7_RANGE", "5613" },
            { "AO1_8_RANGE", "5615" },
            { "AO2_1_RANGE", "5617" },
            { "AO2_2_RANGE", "5619" },
            { "AO2_3_RANGE", "5621" },
            { "AO2_4_RANGE", "5623" },
            { "AO2_5_RANGE", "5625" },
            { "AO2_6_RANGE", "5627" },
            { "AO2_7_RANGE", "5629" },
            { "AO2_8_RANGE", "5631" }
        };

        public Dictionary<string, float> SetChannelRangeValue(IEnumerable<ChannelMapping> Instances,string batchName)
        {
            var aiInstances = Instances.Where(c => c.TestBatch.Contains(batchName) && c.ModuleType.Contains("AI"));
            Dictionary<string, float> channelRangesValue = new Dictionary<string, float>();
            foreach (var instance in aiInstances)
            {
                if(instance.ModuleName.Contains("S"))
                {
                    channelRangesValue.Add(RangerTabel.First(r=>r.Key.Contains(instance.TestPLCChannelTag)).Value,33300.0f);
                }
                else
                {
                    channelRangesValue.Add(RangerTabel.First(r => r.Key.Contains(instance.TestPLCChannelTag)).Value, 27647.0f);
                }
            }
            return channelRangesValue;
        }

        /// <summary>
        /// 根据当前批次的AI通道安全型信息直接写入量程设定值
        /// </summary>
        /// <param name="instances">当前批次的通道集合</param>
        /// <param name="batchName">批次名称</param>
        /// <param name="plc">测试PLC通信实例</param>
        /// <returns></returns>
        public async Task SetChannelRangeAsync(IEnumerable<ChannelMapping> instances, string batchName, IPlcCommunication plc)
        {
            if (instances == null || plc == null) return;

            var aiInstances = instances.Where(c => c.TestBatch.Contains(batchName) && c.ModuleType.Contains("AI"));

            var failedList = new List<(string register,float value)>();

            foreach (var inst in aiInstances)
            {
                var reg = RangerTabel.First(r=>r.Key.Contains(inst.TestPLCChannelTag)).Value;
                float val = inst.ModuleName.Contains("S")?33300.0f:27647.0f;
                failedList.Add((reg,val));
            }

            const int maxRetry = 3;
            int attempt = 0;
            while (failedList.Count>0 && attempt<maxRetry)
            {
                attempt++;
                var stillFail = new List<(string register,float value)>();
                foreach(var item in failedList)
                {
                    try
                    {
                        await plc.WriteAnalogValueAsync(item.register,item.value);
                    }
                    catch
                    {
                        stillFail.Add(item);
                    }
                }
                failedList = stillFail;
            }

            if(failedList.Count>0)
            {
                // 弹窗提示失败信息
                await _messageService.ShowAsync("量程设定失败","仍有"+failedList.Count+"个寄存器写入失败，请检查连接并重试。",MessageBoxButton.OK);
            }
        }
    }
}
