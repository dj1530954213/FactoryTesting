//! 量程值计算器
//! 根据通道是否安全型决定写入值

use crate::models::structs::ChannelPointDefinition as Channel;

pub trait IRangeValueCalculator: Send + Sync {
    fn calc_value(&self, channel: &Channel) -> f32;
}

pub struct DefaultRangeValueCalculator;

impl IRangeValueCalculator for DefaultRangeValueCalculator {
    fn calc_value(&self, channel: &Channel) -> f32 {
        if channel.module_name.contains('S') { 33300.0 } else { 27647.0 }
    }
}
