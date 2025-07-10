//! SeaORM 实现的 RangeRegisterRepository

use std::collections::HashMap;
use async_trait::async_trait;
use sea_orm::{DatabaseConnection, EntityTrait};
use crate::utils::error::{AppError, AppResult};
use crate::domain::services::IRangeRegisterRepository;
use crate::models::entities::range_register;

pub struct RangeRegisterRepository {
    db: DatabaseConnection,
    // 简单缓存，避免频繁查询
    cache: tokio::sync::RwLock<Option<HashMap<String, String>>>,
}

impl RangeRegisterRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db, cache: tokio::sync::RwLock::new(None) }
    }

    async fn load_cache(&self) -> AppResult<HashMap<String, String>> {
        let models = range_register::Entity::find().all(&self.db).await.map_err(|e| AppError::persistence_error(e.to_string()))?;
        log::info!("[RangeRegisterRepository] 加载寄存器映射 {} 条", models.len());
        let map: HashMap<String, String> = models.into_iter().map(|m| (m.channel_tag, m.register)).collect();
        Ok(map)
    }
}

#[async_trait]
impl IRangeRegisterRepository for RangeRegisterRepository {
    async fn all(&self) -> AppResult<HashMap<String, String>> {
        {
            let guard = self.cache.read().await;
            if let Some(ref cached) = *guard {
                return Ok(cached.clone());
            }
        }
        // 未命中缓存，加载
        let map = self.load_cache().await?;
        let mut guard = self.cache.write().await;
        *guard = Some(map.clone());
        Ok(map)
    }
}
