//! Tauri 命令：批次切换时写入AI量程

use std::sync::Arc;
use crate::application::services::range_setting_service::IChannelRangeSettingService;
use crate::utils::error::AppResult;
use crate::domain::services::IPersistenceService;
use crate::models::structs::ChannelPointDefinition as Channel;

#[tauri::command]
pub async fn apply_channel_range_setting_cmd(
    batch_name: String,
    range_service: tauri::State<'_, Arc<dyn IChannelRangeSettingService>>,
    persistence: tauri::State<'_, Arc<dyn IPersistenceService>>,
) -> AppResult<()> {
    log::info!("[TauriCmd] 执行 apply_channel_range_setting_cmd，批次：{}", batch_name);
    // 加载所有通道定义并按批次过滤
    let channels: Vec<Channel> = persistence
        .load_all_channel_definitions()
        .await?
        .into_iter()
        .filter(|c| c.batch_id.as_deref() == Some(&batch_name))
        .collect();

    let res = range_service.set_ranges(&channels).await;
    match &res {
        Ok(_) => log::info!("[TauriCmd] 批次 {} 量程写入成功", batch_name),
        Err(e) => log::error!("[TauriCmd] 批次 {} 量程写入失败: {:?}", batch_name, e),
    }
    res
}
