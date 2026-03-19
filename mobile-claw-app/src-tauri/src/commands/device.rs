use tauri::State;
use crate::state::AppState;
use mobile_claw::types::DeviceInfo;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct CommandResult {
    pub success: bool,
    pub message: String,
    pub data: Option<serde_json::Value>,
}

#[tauri::command]
pub async fn discover_devices(
    state: State<'_, AppState>,
) -> Result<Vec<DeviceInfo>, String> {
    let runtime = state.runtime.read().await;
    runtime
        .discover_devices()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_all_devices(
    state: State<'_, AppState>,
) -> Result<Vec<DeviceInfo>, String> {
    let runtime = state.runtime.read().await;
    runtime
        .get_all_devices()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_device_by_id(
    device_id: String,
    state: State<'_, AppState>,
) -> Result<DeviceInfo, String> {
    let runtime = state.runtime.read().await;
    runtime
        .get_device(&device_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn connect_device(
    device_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let runtime = state.runtime.read().await;
    runtime
        .connect_device(&device_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn disconnect_device(
    device_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let runtime = state.runtime.read().await;
    runtime
        .disconnect_device(&device_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn execute_device_command(
    device_id: String,
    action: String,
    parameters: Option<serde_json::Value>,
    state: State<'_, AppState>,
) -> Result<CommandResult, String> {
    let runtime = state.runtime.read().await;
    
    let params: HashMap<String, serde_json::Value> = parameters
        .and_then(|p| serde_json::from_value(p).ok())
        .unwrap_or_default();
    
    let result = runtime
        .execute_device_command(&device_id, &action, params)
        .await
        .map_err(|e| e.to_string())?;
    
    Ok(CommandResult {
        success: result.success,
        message: result.error.unwrap_or_else(|| "Success".to_string()),
        data: result.result,
    })
}
