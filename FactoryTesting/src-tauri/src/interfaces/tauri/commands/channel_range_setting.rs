//! Tauri 命令：批次切换时写入AI量程

use std::sync::Arc;
use crate::application::services::range_setting_service::IChannelRangeSettingService;
use crate::utils::error::AppResult;
use crate::domain::services::IPersistenceService;
use crate::models::structs::{ChannelPointDefinition as Channel, ChannelTestInstance};

#[tauri::command]
pub async fn apply_channel_range_setting_cmd(
    batch_name: String,
    range_service: tauri::State<'_, Arc<dyn IChannelRangeSettingService>>,
    persistence: tauri::State<'_, Arc<dyn IPersistenceService>>,
) -> AppResult<()> {
    log::info!("[TauriCmd] 执行 apply_channel_range_setting_cmd，批次：{}", batch_name);
    // 先加载批次的测试实例，再取 definition_id 关联通道定义
    let instances = persistence.load_test_instances_by_batch(&batch_name).await?;
    log::info!("[TauriCmd] 批次 {} 获取到 {} 个测试实例", batch_name, instances.len());
    let def_id_set: std::collections::HashSet<String> = instances.iter().map(|i| i.definition_id.clone()).collect();

    let all_definitions = persistence.load_all_channel_definitions().await?;
    // 构建 definition_id -> 通道定义
    let def_map: std::collections::HashMap<String, Channel> = all_definitions
        .into_iter()
        .filter(|c| def_id_set.contains(&c.id))
        .map(|c| (c.id.clone(), c))
        .collect();

    // 构建 definition_id -> instance
    let inst_map: std::collections::HashMap<String, ChannelTestInstance> = instances
        .into_iter()
        .map(|ins| (ins.definition_id.clone(), ins))
        .collect();

    // 根据 instance 中的测试PLC通道标签构造新的 RANGE 标签
    let mut write_channels: Vec<Channel> = Vec::new();
    for (def_id, def) in def_map {
        match inst_map.get(&def_id).and_then(|ins| ins.test_plc_channel_tag.clone()) {
            Some(plc_tag) => {
                let mut new_def = def.clone();
                new_def.tag = format!("{}_RANGE", plc_tag);
                write_channels.push(new_def);
            }
            None => {
                log::warn!("[TauriCmd] 定义 {} 未分配 test_plc_channel_tag，跳过", def_id);
            }
        }
    }

    log::info!("[TauriCmd] 批次 {} 最终需要下发 {} 条通道量程", batch_name, write_channels.len());

    if write_channels.is_empty() {
        log::warn!("[TauriCmd] 批次 {} 没有可写入的通道，终止量程写入", batch_name);
        return Err(crate::utils::error::AppError::generic("当前批次没有可写入的通道，请先分配通道".to_string()));
    }

    let res = range_service.set_ranges(&write_channels).await;
    match &res {
        Ok(_) => log::info!("[TauriCmd] 批次 {} 量程写入成功", batch_name),
        Err(e) => log::error!("[TauriCmd] 批次 {} 量程写入失败: {:?}", batch_name, e),
    }
    res
}
