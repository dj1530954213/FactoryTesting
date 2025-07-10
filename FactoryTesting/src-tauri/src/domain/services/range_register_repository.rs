//! 量程寄存器仓储接口
//! 仅暴露读取功能，写入/更新暂未使用

use async_trait::async_trait;
use std::collections::HashMap;
use crate::utils::error::AppResult;

#[async_trait]
pub trait IRangeRegisterRepository: Send + Sync {
    /// 获取所有寄存器 ChannelTag -> Address
    async fn all(&self) -> AppResult<HashMap<String, String>>;

    /// 获取单个寄存器地址
    async fn get_addr(&self, tag: &str) -> AppResult<Option<String>> {
        let map = self.all().await?;
        Ok(map.get(tag).cloned())
    }
}
