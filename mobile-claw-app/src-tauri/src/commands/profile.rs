use tauri::State;
use crate::state::AppState;
use mobile_claw::types::UserPreferences;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfile {
    pub user_id: String,
    pub name: String,
    pub preferences: UserPreferences,
    pub device_usage_count: usize,
    pub patterns_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recommendation {
    pub id: String,
    pub title: String,
    pub description: String,
    pub action_type: String,
    pub device_id: Option<String>,
    pub confidence: f32,
}

#[tauri::command]
pub async fn get_user_profile(
    user_id: Option<String>,
    state: State<'_, AppState>,
) -> Result<UserProfile, String> {
    let runtime = state.runtime.read().await;
    let uid = user_id.unwrap_or_else(|| "default".to_string());
    
    let profile = runtime
        .get_user_profile(&uid)
        .await
        .map_err(|e| e.to_string())?;
    
    Ok(UserProfile {
        user_id: profile.user_id,
        name: profile.name,
        preferences: profile.preferences,
        device_usage_count: profile.device_usage.len(),
        patterns_count: profile.behavior_patterns.len(),
    })
}

#[tauri::command]
pub async fn update_user_preferences(
    user_id: String,
    preferences: UserPreferences,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let mut runtime = state.runtime.write().await;
    runtime
        .update_user_preferences(&user_id, preferences)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_recommendations(
    user_id: Option<String>,
    state: State<'_, AppState>,
) -> Result<Vec<Recommendation>, String> {
    let runtime = state.runtime.read().await;
    let uid = user_id.unwrap_or_else(|| "default".to_string());
    
    let recs = runtime
        .get_recommendations(&uid)
        .await
        .map_err(|e| e.to_string())?;
    
    Ok(recs
        .into_iter()
        .map(|r| Recommendation {
            id: r.id,
            title: r.title,
            description: r.description,
            action_type: r.action_type,
            device_id: r.device_id,
            confidence: r.confidence,
        })
        .collect())
}
