use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::types::{MNNBackendType, MNNQuantization, PowerMode, ModelConfig as EngineModelConfig};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeConfig {
    pub node_id: String,
    pub model: Option<ModelConfig>,
    pub discovery: DiscoveryConfig,
    pub network: NetworkConfig,
    pub profile: ProfileConfig,
    pub security: SecurityConfig,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            node_id: uuid::Uuid::new_v4().to_string(),
            model: None,
            discovery: DiscoveryConfig::default(),
            network: NetworkConfig::default(),
            profile: ProfileConfig::default(),
            security: SecurityConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    pub model_path: PathBuf,
    pub model_name: String,
    pub quantization: MNNQuantization,
    pub context_length: usize,
    pub backend_type: MNNBackendType,
    pub thread_count: usize,
    pub power_mode: PowerMode,
    pub auto_load: bool,
}

impl Default for ModelConfig {
    fn default() -> Self {
        Self {
            model_path: PathBuf::from("models/qwen-3b.mnn"),
            model_name: "Qwen2.5-3B-Instruct".to_string(),
            quantization: MNNQuantization::INT8,
            context_length: 4096,
            backend_type: MNNBackendType::Auto,
            thread_count: 4,
            power_mode: PowerMode::Balanced,
            auto_load: true,
        }
    }
}

impl From<ModelConfig> for EngineModelConfig {
    fn from(config: ModelConfig) -> Self {
        Self {
            model_path: config.model_path,
            model_name: config.model_name,
            quantization: config.quantization,
            context_length: config.context_length,
            backend_type: config.backend_type,
            thread_count: config.thread_count,
            power_mode: config.power_mode,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryConfig {
    pub enable_wifi: bool,
    pub enable_ble: bool,
    pub scan_interval_secs: u64,
    pub device_timeout_secs: u64,
    pub auto_connect: bool,
}

impl Default for DiscoveryConfig {
    fn default() -> Self {
        Self {
            enable_wifi: true,
            enable_ble: true,
            scan_interval_secs: 60,
            device_timeout_secs: 300,
            auto_connect: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    pub wifi_enabled: bool,
    pub ble_enabled: bool,
    pub ble_scan_duration_secs: u64,
    pub connection_timeout_secs: u64,
    pub retry_count: u32,
    pub retry_delay_ms: u64,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            wifi_enabled: true,
            ble_enabled: true,
            ble_scan_duration_secs: 5,
            connection_timeout_secs: 10,
            retry_count: 3,
            retry_delay_ms: 1000,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileConfig {
    pub enable_learning: bool,
    pub learning_interval_secs: u64,
    pub max_history_entries: usize,
    pub preference_update_threshold: f32,
}

impl Default for ProfileConfig {
    fn default() -> Self {
        Self {
            enable_learning: true,
            learning_interval_secs: 3600,
            max_history_entries: 10000,
            preference_update_threshold: 0.7,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    pub require_pairing: bool,
    pub allow_unknown_devices: bool,
    pub encryption_enabled: bool,
    pub audit_logging: bool,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            require_pairing: true,
            allow_unknown_devices: false,
            encryption_enabled: true,
            audit_logging: true,
        }
    }
}

impl RuntimeConfig {
    pub fn from_file(path: &std::path::Path) -> crate::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: Self = toml::from_str(&content)
            .map_err(|e| crate::Error::ConfigError(format!("Failed to parse config: {}", e)))?;
        Ok(config)
    }

    pub fn to_file(&self, path: &std::path::Path) -> crate::Result<()> {
        let content = toml::to_string_pretty(self)
            .map_err(|e| crate::Error::ConfigError(format!("Failed to serialize config: {}", e)))?;
        std::fs::write(path, content)?;
        Ok(())
    }

    pub fn from_toml(toml_str: &str) -> crate::Result<Self> {
        toml::from_str(toml_str)
            .map_err(|e| crate::Error::ConfigError(format!("Failed to parse config: {}", e)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_is_valid() {
        let config = RuntimeConfig::default();
        assert!(!config.node_id.is_empty());
        assert!(config.discovery.enable_wifi);
        assert!(config.discovery.enable_ble);
    }

    #[test]
    fn config_serialization_roundtrip() {
        let config = RuntimeConfig::default();
        let toml_str = toml::to_string(&config).unwrap();
        let parsed: RuntimeConfig = toml::from_str(&toml_str).unwrap();
        assert_eq!(config.node_id, parsed.node_id);
    }

    #[test]
    fn model_config_defaults() {
        let model = ModelConfig::default();
        assert_eq!(model.context_length, 4096);
        assert_eq!(model.thread_count, 4);
    }
}
