//! Tauri 命令：PLC 连接（确认接线）
//!
//! 当前实现仅做日志记录并返回成功。
//! 后续可在此处调用实际的 PLC 通讯服务，检查连接并返回结果。

use crate::utils::error::AppResult;

#[derive(serde::Serialize)]
pub struct ConnectPlcResult {
    pub success: bool,
    pub message: Option<String>,
}

#[tauri::command]
pub async fn connect_plc_cmd() -> AppResult<ConnectPlcResult> {
    log::info!("[TauriCmd] 执行 connect_plc_cmd (确认接线)");

    // TODO: 调用实际 PLC 通讯逻辑。
    // 目前先返回成功，避免前端调用失败。

    Ok(ConnectPlcResult {
        success: true,
        message: None,
    })
}
