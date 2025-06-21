using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Threading.Channels;
using System.Threading.Tasks;
using FatFullVersion.Models;

namespace FatFullVersion.Shared.Converters
{
    public static class ChannelRangeConversion
    {
        public static float PercentageToRealValue(ChannelMapping channel,float percentage)
        {
            float minValue = channel.RangeLowerLimitValue;
            float maxValue = channel.RangeUpperLimitValue;
            float realValue = minValue + (maxValue - minValue) * percentage/100f;
            return realValue;
        }

        public static float RealValueToPercentage(ChannelMapping channel, float realValue)
        {
            float minValue = channel.RangeLowerLimitValue;
            float maxValue = channel.RangeUpperLimitValue;
            float percent = (realValue - minValue) / (maxValue - minValue) * 100f;
            return percent;
        }

        public static float RealValueToPercentage(ChannelMapping channel, string realValue)
        {
            float minValue = channel.RangeLowerLimitValue;
            float maxValue = channel.RangeUpperLimitValue;
            float percent = (Convert.ToSingle(realValue) - minValue) / (maxValue - minValue) * 100f;
            return percent;
        }
    }
}
