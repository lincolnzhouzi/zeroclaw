use tauri::State;
use crate::state::AppState;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub theme: String,
    pub language: String,
    pub auto_discover: bool,
    pub discovery_interval: u64,
    pub power_mode: String,
    pub notifications_enabled: bool,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            theme: "system".to_string(),
            language: "zh-CN".to_string(),
            auto_discover: true,
            discovery_interval: 30,
            power_mode: "balanced".to_string(),
            notifications_enabled: true,
        }
    }
}

#[tauri::command]
pub async fn get_settings() -> Result<AppSettings, String> {
    Ok(AppSettings::default())
}

#[tauri::command]
pub async fn update_settings(
    settings: AppSettings,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let mut runtime = state.runtime.write().await;
    runtime
        .update_settings(mobile_claw::runtime::AppSettings {
            theme: Some(settings.theme),
            language: Some(settings.language),
            auto_discover: settings.auto_discover,
            discovery_interval_secs: settings.discovery_interval,
            power_mode: match settings.power_mode.as_str() {
                "performance" => mobile_claw::types::PowerMode::Performance,
                "powersaving" => mobile_claw::types::PowerMode::PowerSaving,
                _ => mobile_claw::types::PowerMode::Balanced,
            },
            notifications_enabled: settings.notifications_enabled,
        })
        .await
        .map_err(|e| e.to_string())
}
