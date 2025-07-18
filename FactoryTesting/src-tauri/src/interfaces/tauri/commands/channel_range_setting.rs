/// 通道量程设置命令模块
/// 
/// 业务说明：
/// 本模块负责在批次切换时自动配置AI（模拟量输入）通道的量程
/// 量程设置是测试准备的关键步骤，确保PLC能正确解析模拟信号
/// 
/// 技术背景：
/// - AI通道需要设置量程才能将电流/电压信号转换为工程值
/// - 量程信息存储在专门的寄存器中（通道地址_RANGE）
/// - 批次切换时需要根据通道定义重新配置量程
/// 
/// 架构定位：
/// - 位于接口层(interfaces)，处理前端的量程设置请求
/// - 通过IChannelRangeSettingService应用服务执行量程写入
/// - 使用IPersistenceService查询通道配置数据

use std::sync::Arc;
use crate::application::services::range_setting_service::IChannelRangeSettingService;
use crate::utils::error::AppResult;
use crate::domain::services::IPersistenceService;
use crate::models::structs::{ChannelPointDefinition as Channel, ChannelTestInstance};

/// 应用通道量程设置命令
/// 
/// 业务说明：
/// - 在批次测试开始前，自动配置所有AI通道的量程
/// - 从数据库获取批次的通道配置，生成量程写入指令
/// - 通过PLC通信服务将量程值写入到对应的寄存器
/// 
/// 执行流程：
/// 1. 加载批次的所有测试实例
/// 2. 获取实例对应的通道定义
/// 3. 根据测试PLC通道标签生成量程寄存器地址
/// 4. 调用量程服务执行批量写入
/// 
/// 参数：
/// - batch_name: 批次名称
/// - range_service: 量程设置服务（通过Tauri状态注入）
/// - persistence: 持久化服务（通过Tauri状态注入）
/// 
/// 返回：
/// - Ok(()): 量程设置成功
/// - Err: 设置失败的错误信息
/// 
/// 调用链：
/// 前端批次切换 -> apply_channel_range_setting_cmd -> IChannelRangeSettingService -> PLC通信
/// 
/// Rust知识点：
/// - #[tauri::command] 标记为Tauri命令
/// - tauri::State<'_, T> 是Tauri的依赖注入机制
/// - Arc<dyn Trait> 用于共享trait对象的所有权
/// - ? 操作符用于错误传播
#[tauri::command]
pub async fn apply_channel_range_setting_cmd(
    batch_name: String,
    range_service: tauri::State<'_, Arc<dyn IChannelRangeSettingService>>,
    persistence: tauri::State<'_, Arc<dyn IPersistenceService>>,
) -> AppResult<()> {
    log::info!("[TauriCmd] 执行 apply_channel_range_setting_cmd，批次：{}", batch_name);
    
    // 先加载批次的测试实例，再取 definition_id 关联通道定义
    // 业务说明：测试实例包含了通道分配信息，是量程设置的数据来源
    let instances = persistence.load_test_instances_by_batch(&batch_name).await?;
    log::info!("[TauriCmd] 批次 {} 获取到 {} 个测试实例", batch_name, instances.len());
    
    // 收集所有实例的定义ID，用于后续过滤
    // Rust知识点：HashSet用于去重，iter().map().collect()是函数式编程范式
    let def_id_set: std::collections::HashSet<String> = instances.iter().map(|i| i.definition_id.clone()).collect();

    // 加载所有通道定义，然后过滤出当前批次需要的
    let all_definitions = persistence.load_all_channel_definitions().await?;
    
    // 构建 definition_id -> 通道定义 的映射
    // 业务说明：只保留当前批次用到的通道定义，减少处理数据量
    // Rust知识点：HashMap提供O(1)的查找性能
    let def_map: std::collections::HashMap<String, Channel> = all_definitions
        .into_iter()
        .filter(|c| def_id_set.contains(&c.id))
        .map(|c| (c.id.clone(), c))
        .collect();

    // 构建 definition_id -> instance 的映射
    // 业务说明：方便根据定义ID快速查找对应的测试实例
    let inst_map: std::collections::HashMap<String, ChannelTestInstance> = instances
        .into_iter()
        .map(|ins| (ins.definition_id.clone(), ins))
        .collect();

    // 根据 instance 中的测试PLC通道标签构造新的 RANGE 标签
    // 业务说明：量程寄存器的地址规则是：通道地址_RANGE
    // 例如：AI01 的量程寄存器是 AI01_RANGE
    let mut write_channels: Vec<Channel> = Vec::new();
    for (def_id, def) in def_map {
        // Rust知识点：Option的链式调用，and_then用于处理嵌套的Option
        match inst_map.get(&def_id).and_then(|ins| ins.test_plc_channel_tag.clone()) {
            Some(plc_tag) => {
                // 克隆通道定义并修改标签为量程寄存器地址
                let mut new_def = def.clone();
                new_def.tag = format!("{}_RANGE", plc_tag);
                write_channels.push(new_def);
            }
            None => {
                // 未分配测试PLC通道的定义跳过
                log::warn!("[TauriCmd] 定义 {} 未分配 test_plc_channel_tag，跳过", def_id);
            }
        }
    }

    log::info!("[TauriCmd] 批次 {} 最终需要下发 {} 条通道量程", batch_name, write_channels.len());

    // 检查是否有需要写入的通道
    if write_channels.is_empty() {
        log::warn!("[TauriCmd] 批次 {} 没有可写入的通道，终止量程写入", batch_name);
        // 业务说明：如果批次没有分配通道，提示用户先进行通道分配
        return Err(crate::utils::error::AppError::generic("当前批次没有可写入的通道，请先分配通道".to_string()));
    }

    // 调用量程服务执行批量写入
    // 业务说明：set_ranges会通过PLC通信服务将量程值写入到对应的寄存器
    let res = range_service.set_ranges(&write_channels).await;
    
    // 记录执行结果
    match &res {
        Ok(_) => log::info!("[TauriCmd] 批次 {} 量程写入成功", batch_name),
        Err(e) => log::error!("[TauriCmd] 批次 {} 量程写入失败: {:?}", batch_name, e),
    }
    
    // 直接返回服务层的结果
    res
}
