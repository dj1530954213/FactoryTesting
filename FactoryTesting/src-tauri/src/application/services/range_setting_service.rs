//! # 通道量程设置服务 (Channel Range Setting Service)
//!
//! ## 业务说明
//! 通道量程设置服务负责在测试批次切换后将模拟量通道的量程参数写入测试PLC
//! 确保测试PLC能够正确模拟目标通道的工程量程，是AO通道测试的关键组件
//!
//! ## 核心功能
//! - **量程计算**: 根据通道定义计算对应的PLC量程寄存器值
//! - **批量写入**: 支持批量将多个通道的量程参数写入PLC
//! - **动态切换**: 支持运行时动态切换不同的实现策略
//! - **错误处理**: 完善的PLC通信错误处理和重试机制
//!
//! ## 工作原理
//! ```
//! 通道定义 → 量程计算 → 寄存器映射 → PLC写入 → 写入验证
//! ```
//!
//! ## 设计模式
//! - **动态代理模式**: DynamicRangeSettingService支持运行时实现切换
//! - **空对象模式**: NullRangeSettingService用于无PLC连接时的处理
//! - **策略模式**: 支持多种量程设置策略
//!
//! ## 应用场景
//! - **批次切换**: 每次切换测试批次时更新PLC量程设置
//! - **AO通道测试**: AO通道测试前必须设置正确的量程
//! - **校准测试**: 高精度测试时的量程校准
//!
//! ## Rust知识点
//! - **trait对象**: 使用dyn trait实现动态分发
//! - **异步互斥**: 使用tokio::sync::Mutex保护共享状态
//! - **包装器模式**: 通过包装器提供额外功能

use crate::domain::services::{
    IPlcCommunicationService, ConnectionHandle,
};
use crate::domain::services::IRangeRegisterRepository;
use crate::domain::services::range_value_calculator::{IRangeValueCalculator};
use crate::utils::error::{AppError, AppResult};
use crate::models::structs::{ChannelPointDefinition as Channel};
use crate::models::enums::ModuleType;
use std::sync::Arc;
use std::collections::HashMap;
use async_trait::async_trait;
use tokio::sync::Mutex;

#[async_trait]
pub trait IChannelRangeSettingService: Send + Sync {
    async fn set_ranges(&self, channels: &[Channel]) -> AppResult<()>;
}

/// 可动态替换内部实现的量程写入服务（包装器模式）
/// 启动时注入本结构体，内部持有真正的实现，可在运行时替换
pub struct DynamicRangeSettingService {
    inner: Mutex<Arc<dyn IChannelRangeSettingService + Send + Sync>>, // 受 Mutex 保护
}

impl DynamicRangeSettingService {
    pub fn new(initial: Arc<dyn IChannelRangeSettingService + Send + Sync>) -> Self {
        Self { inner: Mutex::new(initial) }
    }

    /// 用新的实现替换内部指针
    pub async fn replace(&self, new_impl: Arc<dyn IChannelRangeSettingService + Send + Sync>) {
        let mut guard = self.inner.lock().await;
        *guard = new_impl;
        log::info!("[DynamicRange] 已替换内部 ChannelRangeSettingService 实现");
    }
}

#[async_trait]
impl IChannelRangeSettingService for DynamicRangeSettingService {
    async fn set_ranges(&self, channels: &[Channel]) -> AppResult<()> {
        let guard = self.inner.lock().await;
        guard.set_ranges(channels).await
    }
}


pub struct ChannelRangeSettingService {
    plc: Arc<dyn IPlcCommunicationService>,
    plc_handle: ConnectionHandle,
    repo: Arc<dyn IRangeRegisterRepository>,
    calculator: Arc<dyn IRangeValueCalculator>,
}

impl ChannelRangeSettingService {
    pub fn new(
        plc: Arc<dyn IPlcCommunicationService>,
        plc_handle: ConnectionHandle,
        repo: Arc<dyn IRangeRegisterRepository>,
        calculator: Arc<dyn IRangeValueCalculator>,
    ) -> Self {
        log::info!("[ChannelRangeSettingService] 创建，新连接: id={}, handle_id={}, protocol={:?}", plc_handle.connection_id, plc_handle.handle_id, plc_handle.protocol);
        Self { plc, plc_handle, repo, calculator }
    }
}

#[derive(Debug, Default)]
pub struct NullRangeSettingService;

#[async_trait]
impl IChannelRangeSettingService for NullRangeSettingService {
    async fn set_ranges(&self, _channels: &[Channel]) -> AppResult<()> {
        Err(AppError::plc_communication_error("PLC连接未初始化".to_string()))
    }
}

#[async_trait]
impl IChannelRangeSettingService for ChannelRangeSettingService {
    async fn set_ranges(&self, channels: &[Channel]) -> AppResult<()> {
        log::info!("[RangeSetting] 开始写入量程，共{}个通道，使用连接 id={} handle={}", channels.len(), self.plc_handle.connection_id, self.plc_handle.handle_id);
        // 获取寄存器映射
        let addr_map = self.repo.all().await?;
        let mut failed: Vec<(String, f32)> = Vec::new();
        for ch in channels.iter().filter(|c| matches!(c.module_type, ModuleType::AI)) {
            // 先用原始 tag 查找寄存器映射；若找不到，则尝试 "{tag}_RANGE"
            let maybe_addr = addr_map.get(&ch.tag).cloned().or_else(|| {
                let range_key = format!("{}_RANGE", ch.tag);
                addr_map.get(&range_key).cloned()
            });
            if let Some(addr) = maybe_addr {
                let val = self.calculator.calc_value(ch);
                log::debug!("[RangeSetting] 计划写入: tag={} addr={} val={}", ch.tag, addr, val);
                failed.push((addr.clone(), val));
                        } else {
                log::warn!("[RangeSetting] 未找到寄存器映射, tag={} 或 {}_RANGE", ch.tag, ch.tag);
            }
        }
        const RETRY: usize = 3;
        let mut attempt = 0;
        while !failed.is_empty() && attempt < RETRY {
            attempt += 1;
            let mut still_fail = Vec::new();
            for (addr, val) in failed.into_iter() {
                match self.plc.write_f32(&self.plc_handle, &addr, val).await {
                    Ok(_) => {
                        log::info!("[RangeSetting] 写入成功: addr={} val={}", addr, val);
                    }
                    Err(e) => {
                        log::error!("[RangeSetting] 写入寄存器失败: {} -> {}, err={:?}", addr, val, e);
                        still_fail.push((addr, val));
                    }
                }
            }
            failed = still_fail;
        }
        if !failed.is_empty() {
            log::error!("[RangeSetting] 写入量程失败，剩余 {} 个寄存器未写入", failed.len());
            return Err(AppError::generic(format!("量程设定失败，{} 个寄存器未写入", failed.len())));
        }
        log::info!("[RangeSetting] 量程写入完成");
        Ok(())
    }
}
