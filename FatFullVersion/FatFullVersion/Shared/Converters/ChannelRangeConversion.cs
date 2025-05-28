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
        public static float PercentageToRealValue(ChannelMapping channel, float percentage)
        {
            if (!channel.RangeLowerLimitValue.HasValue || !channel.RangeUpperLimitValue.HasValue)
            {
                // Or throw an exception, or return a specific error indicator like float.NaN
                System.Diagnostics.Debug.WriteLine($"Warning: Range limits not set for channel {channel.VariableName} in PercentageToRealValue.");
                return float.NaN; // Or handle error appropriately
            }
            float minValue = channel.RangeLowerLimitValue.Value;
            float maxValue = channel.RangeUpperLimitValue.Value;
            if (maxValue <= minValue) return minValue; // Avoid division by zero or incorrect calculation
            float realValue = minValue + (maxValue - minValue) * percentage / 100f;
            return realValue;
        }

        public static float RealValueToPercentage(ChannelMapping channel, float realValue)
        {
            if (!channel.RangeLowerLimitValue.HasValue || !channel.RangeUpperLimitValue.HasValue)
            {
                System.Diagnostics.Debug.WriteLine($"Warning: Range limits not set for channel {channel.VariableName} in RealValueToPercentage.");
                return float.NaN;
            }
            float minValue = channel.RangeLowerLimitValue.Value;
            float maxValue = channel.RangeUpperLimitValue.Value;
            if (maxValue <= minValue) return 0f; // Avoid division by zero, return 0% or NaN
            float percent = (realValue - minValue) / (maxValue - minValue) * 100f;
            return percent;
        }

        public static float RealValueToPercentage(ChannelMapping channel, string realValueString)
        {
            if (!channel.RangeLowerLimitValue.HasValue || !channel.RangeUpperLimitValue.HasValue)
            {
                System.Diagnostics.Debug.WriteLine($"Warning: Range limits not set for channel {channel.VariableName} in RealValueToPercentage (string overload).");
                return float.NaN;
            }
            if (!float.TryParse(realValueString, out float realValue))
            {
                System.Diagnostics.Debug.WriteLine($"Warning: Could not parse realValueString '{realValueString}' to float.");
                return float.NaN;
            }

            float minValue = channel.RangeLowerLimitValue.Value;
            float maxValue = channel.RangeUpperLimitValue.Value;
            if (maxValue <= minValue) return 0f;
            float percent = (realValue - minValue) / (maxValue - minValue) * 100f;
            return percent;
        }
    }
}
