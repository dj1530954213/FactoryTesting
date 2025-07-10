//! Channel Range Setting Service
//! 负责在批次切换后将量程值写入测试 PLC

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

#[async_trait]
pub trait IChannelRangeSettingService: Send + Sync {
    async fn set_ranges(&self, channels: &[Channel]) -> AppResult<()>;
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
        log::info!("[RangeSetting] 开始写入量程，共{}个通道", channels.len());
        // 获取寄存器映射
        let addr_map = self.repo.all().await?;
        let mut failed: Vec<(String, f32)> = Vec::new();
        for ch in channels.iter().filter(|c| matches!(c.module_type, ModuleType::AI)) {
            if let Some(addr) = addr_map.get(&ch.tag) {
                let val = self.calculator.calc_value(ch);
                failed.push((addr.clone(), val));
            }
        }
        const RETRY: usize = 3;
        let mut attempt = 0;
        while !failed.is_empty() && attempt < RETRY {
            attempt += 1;
            let mut still_fail = Vec::new();
            for (addr, val) in failed.into_iter() {
                if let Err(e) = self.plc.write_f32(&self.plc_handle, &addr, val).await {
                    log::error!("[RangeSetting] 写入寄存器失败: {} -> {}, err={:?}", addr, val, e);
                    still_fail.push((addr, val));
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
