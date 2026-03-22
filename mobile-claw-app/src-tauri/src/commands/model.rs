use crate::state::AppState;
use mobile_claw::types::{HardwareInfo, MNNBackendType, MNNQuantization, ModelConfig, PowerMode};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tauri::State;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelStatus {
    pub loaded: bool,
    pub name: String,
    pub backend: String,
    pub quantization: String,
    pub context_length: usize,
    pub thread_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfoResponse {
    pub id: String,
    pub name: String,
    pub size_bytes: u64,
    pub quantization: String,
    pub context_length: usize,
    pub downloaded: bool,
    pub model_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareInfoResponse {
    pub cpu_cores: usize,
    pub total_memory: u64,
    pub gpu_available: bool,
    pub gpu_type: Option<String>,
    pub gpu_memory: Option<u64>,
    pub npu_available: bool,
    pub npu_type: Option<String>,
    pub supports_fp16: bool,
    pub supports_dotprod: bool,
}

impl From<HardwareInfo> for HardwareInfoResponse {
    fn from(hw: HardwareInfo) -> Self {
        Self {
            cpu_cores: hw.cpu_cores,
            total_memory: hw.total_memory,
            gpu_available: hw.gpu_available,
            gpu_type: hw.gpu_type.map(|t| format!("{:?}", t)),
            gpu_memory: hw.gpu_memory,
            npu_available: hw.npu_available,
            npu_type: hw.npu_type.map(|t| format!("{:?}", t)),
            supports_fp16: hw.supports_fp16,
            supports_dotprod: hw.supports_dotprod,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadModelRequest {
    pub model_id: String,
    pub model_path: String,
    pub model_name: String,
    pub quantization: String,
    pub context_length: usize,
    pub backend_type: String,
    pub thread_count: usize,
    pub power_mode: String,
}

impl TryFrom<LoadModelRequest> for ModelConfig {
    type Error = String;

    fn try_from(req: LoadModelRequest) -> Result<Self, Self::Error> {
        let quantization = match req.quantization.to_lowercase().as_str() {
            "fp32" => MNNQuantization::FP32,
            "fp16" => MNNQuantization::FP16,
            "int8" | "q8" => MNNQuantization::INT8,
            "bf16" => MNNQuantization::BF16,
            _ => MNNQuantization::INT8,
        };

        let backend_type = match req.backend_type.to_lowercase().as_str() {
            "cpu" => MNNBackendType::CPU,
            "gpu" => MNNBackendType::GPU,
            "npu" => MNNBackendType::NPU,
            "auto" => MNNBackendType::Auto,
            _ => MNNBackendType::Auto,
        };

        let power_mode = match req.power_mode.to_lowercase().as_str() {
            "performance" => PowerMode::Performance,
            "powersaving" | "power_saving" => PowerMode::PowerSaving,
            _ => PowerMode::Balanced,
        };

        Ok(ModelConfig {
            model_path: std::path::PathBuf::from(&req.model_path),
            model_name: req.model_name,
            quantization,
            context_length: req.context_length,
            backend_type,
            thread_count: req.thread_count,
            power_mode,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadProgress {
    pub model_id: String,
    pub status: String,
    pub percentage: u8,
    pub downloaded_bytes: u64,
    pub total_bytes: u64,
    pub error: Option<String>,
}

#[tauri::command]
pub async fn get_model_status(state: State<'_, AppState>) -> Result<ModelStatus, String> {
    let runtime = state.runtime.read().await;
    let status = runtime.get_model_status().await;

    Ok(ModelStatus {
        loaded: status.loaded,
        name: status.name,
        backend: status.backend,
        quantization: status.quantization,
        context_length: status.context_length.unwrap_or(4096),
        thread_count: status.thread_count.unwrap_or(4),
    })
}

#[tauri::command]
pub async fn load_model(
    request: LoadModelRequest,
    state: State<'_, AppState>,
) -> Result<ModelStatus, String> {
    let config = ModelConfig::try_from(request)?;

    let mut runtime = state.runtime.write().await;
    runtime
        .load_model(config)
        .await
        .map_err(|e| e.to_string())?;

    let status = runtime.get_model_status().await;

    Ok(ModelStatus {
        loaded: status.loaded,
        name: status.name,
        backend: status.backend,
        quantization: status.quantization,
        context_length: status.context_length.unwrap_or(4096),
        thread_count: status.thread_count.unwrap_or(4),
    })
}

#[tauri::command]
pub async fn unload_model(state: State<'_, AppState>) -> Result<(), String> {
    let mut runtime = state.runtime.write().await;
    runtime.unload_model().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_available_models(
    state: State<'_, AppState>,
) -> Result<Vec<ModelInfoResponse>, String> {
    let runtime = state.runtime.read().await;
    let models = runtime.get_available_models().await;

    Ok(models
        .into_iter()
        .map(|m| ModelInfoResponse {
            id: m.id,
            name: m.name,
            size_bytes: m.size_bytes,
            quantization: format!("{:?}", m.quantization),
            context_length: m.context_length,
            downloaded: m.downloaded,
            model_type: format!("{:?}", m.model_type),
        })
        .collect())
}

#[tauri::command]
pub async fn get_hardware_info(state: State<'_, AppState>) -> Result<HardwareInfoResponse, String> {
    let runtime = state.runtime.read().await;
    let hw = runtime.get_hardware_info().await;

    Ok(HardwareInfoResponse::from(hw))
}

#[tauri::command]
pub async fn download_model(
    model_id: String,
    _state: State<'_, AppState>,
    _app: tauri::AppHandle,
) -> Result<(), String> {
    Err("Model download not implemented yet".to_string())
}

#[tauri::command]
pub async fn cancel_download(model_id: String, _state: State<'_, AppState>) -> Result<(), String> {
    let _ = model_id;
    Ok(())
}

#[tauri::command]
pub async fn delete_model(model_id: String, _state: State<'_, AppState>) -> Result<(), String> {
    let _ = model_id;
    Ok(())
}

#[tauri::command]
pub async fn get_model_config(
    _state: State<'_, AppState>,
) -> Result<HashMap<String, serde_json::Value>, String> {
    Ok(HashMap::new())
}

#[tauri::command]
pub async fn set_model_config(
    key: String,
    value: serde_json::Value,
    _state: State<'_, AppState>,
) -> Result<(), String> {
    let _ = (key, value);
    Ok(())
}
