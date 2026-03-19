use tauri::State;
use crate::state::AppState;
use mobile_claw::types::ModelConfig;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelStatus {
    pub loaded: bool,
    pub name: String,
    pub backend: String,
    pub quantization: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub id: String,
    pub name: String,
    pub size_mb: u64,
    pub quantization: String,
    pub context_length: usize,
    pub downloaded: bool,
}

#[tauri::command]
pub async fn get_model_status(
    state: State<'_, AppState>,
) -> Result<ModelStatus, String> {
    let runtime = state.runtime.read().await;
    let status = runtime.get_model_status().await;
    
    Ok(ModelStatus {
        loaded: status.loaded,
        name: status.name,
        backend: status.backend,
        quantization: status.quantization,
    })
}

#[tauri::command]
pub async fn load_model(
    config: ModelConfig,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let mut runtime = state.runtime.write().await;
    runtime
        .load_model(config)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn unload_model(
    state: State<'_, AppState>,
) -> Result<(), String> {
    let mut runtime = state.runtime.write().await;
    runtime
        .unload_model()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_available_models(
    state: State<'_, AppState>,
) -> Result<Vec<ModelInfo>, String> {
    let runtime = state.runtime.read().await;
    let models = runtime.get_available_models().await;
    
    Ok(models
        .into_iter()
        .map(|m| ModelInfo {
            id: m.id,
            name: m.name,
            size_mb: m.size_bytes / (1024 * 1024),
            quantization: format!("{:?}", m.quantization),
            context_length: m.context_length,
            downloaded: m.downloaded,
        })
        .collect())
}
