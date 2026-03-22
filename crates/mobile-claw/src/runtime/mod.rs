pub mod builder;
pub mod config;

pub use builder::MobileClawRuntimeBuilder;

use serde::{Deserialize, Serialize};

use crate::device::DeviceManager;
use crate::engine::LocalModelEngine;
use crate::error::{Error, Result};
use crate::network::{BluetoothManager, WiFiManager};
use crate::profile::{RecommendationEngine, UserProfileEngine};
use crate::protocols::{A2AProtocol, ACPProtocol, MCPProtocol};
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct MobileClawRuntime {
    config: config::RuntimeConfig,
    device_manager: Arc<RwLock<DeviceManager>>,
    a2a_protocol: Arc<RwLock<A2AProtocol>>,
    acp_protocol: Arc<RwLock<ACPProtocol>>,
    mcp_protocol: Arc<RwLock<MCPProtocol>>,
    local_model: Arc<RwLock<Option<LocalModelEngine>>>,
    user_profile: Arc<RwLock<UserProfileEngine>>,
    recommendation: Arc<RwLock<RecommendationEngine>>,
    wifi_manager: Arc<RwLock<Option<WiFiManager>>>,
    ble_manager: Arc<RwLock<Option<BluetoothManager>>>,
    running: Arc<RwLock<bool>>,
}

impl MobileClawRuntime {
    pub fn builder() -> MobileClawRuntimeBuilder {
        MobileClawRuntimeBuilder::new()
    }

    pub fn new(config: config::RuntimeConfig) -> Self {
        Self {
            config,
            device_manager: Arc::new(RwLock::new(DeviceManager::new())),
            a2a_protocol: Arc::new(RwLock::new(A2AProtocol::new())),
            acp_protocol: Arc::new(RwLock::new(ACPProtocol::new())),
            mcp_protocol: Arc::new(RwLock::new(MCPProtocol::new())),
            local_model: Arc::new(RwLock::new(None)),
            user_profile: Arc::new(RwLock::new(UserProfileEngine::new())),
            recommendation: Arc::new(RwLock::new(RecommendationEngine::with_default_profile())),
            wifi_manager: Arc::new(RwLock::new(None)),
            ble_manager: Arc::new(RwLock::new(None)),
            running: Arc::new(RwLock::new(false)),
        }
    }

    pub async fn start(&self) -> Result<()> {
        let mut running = self.running.write().await;
        if *running {
            return Err(Error::InvalidState("Runtime already running".to_string()));
        }

        tracing::info!("Starting Mobile Claw Runtime v{}", crate::VERSION);

        self.initialize_network_managers().await?;

        self.start_discovery_services().await?;

        if let Some(ref model_config) = self.config.model {
            let mut local_model = self.local_model.write().await;
            let engine_config: crate::types::ModelConfig = model_config.clone().into();
            let engine = LocalModelEngine::new(engine_config).await?;
            *local_model = Some(engine);
            tracing::info!(
                "Local model engine initialized: {}",
                model_config.model_name
            );
        }

        *running = true;
        tracing::info!("Mobile Claw Runtime started successfully");
        Ok(())
    }

    pub async fn stop(&self) -> Result<()> {
        let mut running = self.running.write().await;
        if !*running {
            return Err(Error::InvalidState("Runtime not running".to_string()));
        }

        tracing::info!("Stopping Mobile Claw Runtime");

        {
            let mut wifi = self.wifi_manager.write().await;
            if let Some(ref mut manager) = *wifi {
                manager.disconnect_all().await?;
            }
            *wifi = None;
        }

        {
            let mut ble = self.ble_manager.write().await;
            if let Some(ref mut manager) = *ble {
                manager.disconnect_all().await?;
            }
            *ble = None;
        }

        *running = false;
        tracing::info!("Mobile Claw Runtime stopped");
        Ok(())
    }

    pub async fn is_running(&self) -> bool {
        *self.running.read().await
    }

    async fn initialize_network_managers(&self) -> Result<()> {
        #[cfg(feature = "wifi")]
        {
            let mut wifi = self.wifi_manager.write().await;
            *wifi = Some(WiFiManager::new());
            tracing::debug!("WiFi manager initialized");
        }

        #[cfg(feature = "ble")]
        {
            let mut ble = self.ble_manager.write().await;
            *ble = Some(BluetoothManager::new());
            tracing::debug!("Bluetooth manager initialized");
        }

        #[cfg(not(any(feature = "wifi", feature = "ble")))]
        {
            let _ = &self.wifi_manager;
            let _ = &self.ble_manager;
        }

        Ok(())
    }

    async fn start_discovery_services(&self) -> Result<()> {
        {
            let mut a2a = self.a2a_protocol.write().await;
            a2a.start_discovery().await?;
        }

        {
            let wifi = self.wifi_manager.read().await;
            if let Some(ref manager) = *wifi {
                let devices = manager.discover_devices().await?;
                let mut dm = self.device_manager.write().await;
                for device in devices {
                    dm.register_device(device).await?;
                }
            }
        }

        {
            let ble = self.ble_manager.read().await;
            if let Some(ref manager) = *ble {
                let devices = manager.scan(std::time::Duration::from_secs(5)).await?;
                let mut dm = self.device_manager.write().await;
                for device in devices {
                    dm.register_device(device).await?;
                }
            }
        }

        Ok(())
    }

    pub async fn device_manager(&self) -> tokio::sync::RwLockWriteGuard<'_, DeviceManager> {
        self.device_manager.write().await
    }

    pub async fn a2a_protocol(&self) -> tokio::sync::RwLockWriteGuard<'_, A2AProtocol> {
        self.a2a_protocol.write().await
    }

    pub async fn acp_protocol(&self) -> tokio::sync::RwLockWriteGuard<'_, ACPProtocol> {
        self.acp_protocol.write().await
    }

    pub async fn mcp_protocol(&self) -> tokio::sync::RwLockWriteGuard<'_, MCPProtocol> {
        self.mcp_protocol.write().await
    }

    pub async fn local_model(&self) -> tokio::sync::RwLockWriteGuard<'_, Option<LocalModelEngine>> {
        self.local_model.write().await
    }

    pub async fn user_profile(&self) -> tokio::sync::RwLockWriteGuard<'_, UserProfileEngine> {
        self.user_profile.write().await
    }

    pub async fn recommendation(&self) -> tokio::sync::RwLockWriteGuard<'_, RecommendationEngine> {
        self.recommendation.write().await
    }

    pub async fn process_user_input(&self, input: &str) -> Result<String> {
        let mcp = self.mcp_protocol.read().await;
        let context = mcp.build_context().await;

        let model = self.local_model.read().await;
        if let Some(ref engine) = *model {
            let response = engine.generate(input, &context).await?;
            Ok(response)
        } else {
            Err(Error::ModelError("No local model loaded".to_string()))
        }
    }

    pub async fn execute_device_command(
        &self,
        device_id: &str,
        action: &str,
        parameters: std::collections::HashMap<String, serde_json::Value>,
    ) -> Result<crate::types::CommandResult> {
        let acp = self.acp_protocol.read().await;
        let command = crate::types::DeviceCommand {
            device_id: device_id.to_string(),
            action: action.to_string(),
            parameters,
            correlation_id: Some(uuid::Uuid::new_v4().to_string()),
        };
        acp.execute_command(command).await
    }

    pub async fn discover_devices(&self) -> Result<Vec<crate::types::DeviceInfo>> {
        let mut all_devices = Vec::new();

        {
            let wifi = self.wifi_manager.read().await;
            if let Some(ref manager) = *wifi {
                let devices = manager.discover_devices().await?;
                all_devices.extend(devices);
            }
        }

        {
            let ble = self.ble_manager.read().await;
            if let Some(ref manager) = *ble {
                let devices = manager.scan(std::time::Duration::from_secs(3)).await?;
                all_devices.extend(devices);
            }
        }

        Ok(all_devices)
    }

    pub async fn get_all_devices(&self) -> Result<Vec<crate::types::DeviceInfo>> {
        let device_manager = self.device_manager.read().await;
        Ok(device_manager.list_devices().await)
    }

    pub async fn get_device(&self, device_id: &str) -> Result<crate::types::DeviceInfo> {
        let device_manager = self.device_manager.read().await;
        device_manager
            .get_device(device_id)
            .await
            .ok_or_else(|| Error::DeviceNotFound(device_id.to_string()))
    }

    pub async fn connect_device(&self, device_id: &str) -> Result<()> {
        let device_manager = self.device_manager.read().await;
        let device = device_manager
            .get_device(device_id)
            .await
            .ok_or_else(|| Error::DeviceNotFound(device_id.to_string()))?;

        match device.protocol {
            crate::types::ConnectionProtocol::HTTP
            | crate::types::ConnectionProtocol::WebSocket => {
                let wifi = self.wifi_manager.read().await;
                if let Some(ref manager) = *wifi {
                    manager.connect(&device.endpoint, None).await?;
                }
            }
            crate::types::ConnectionProtocol::BLE => {
                let ble = self.ble_manager.read().await;
                if let Some(ref manager) = *ble {
                    manager.connect(&device.endpoint).await?;
                }
            }
            _ => {}
        }

        let dm = self.device_manager.write().await;
        dm.connect(device_id).await?;
        Ok(())
    }

    pub async fn disconnect_device(&self, device_id: &str) -> Result<()> {
        let device_manager = self.device_manager.read().await;
        let device = device_manager
            .get_device(device_id)
            .await
            .ok_or_else(|| Error::DeviceNotFound(device_id.to_string()))?;

        match device.protocol {
            crate::types::ConnectionProtocol::HTTP
            | crate::types::ConnectionProtocol::WebSocket => {
                let wifi = self.wifi_manager.read().await;
                if let Some(ref manager) = *wifi {
                    manager.disconnect().await?;
                }
            }
            crate::types::ConnectionProtocol::BLE => {
                let ble = self.ble_manager.read().await;
                if let Some(ref manager) = *ble {
                    manager.disconnect(&device.endpoint).await?;
                }
            }
            _ => {}
        }

        let mut dm = self.device_manager.write().await;
        dm.disconnect(device_id).await?;
        Ok(())
    }

    pub async fn load_model(&mut self, config: crate::types::ModelConfig) -> Result<()> {
        let engine = LocalModelEngine::new(config).await?;
        let mut local_model = self.local_model.write().await;
        *local_model = Some(engine);
        Ok(())
    }

    pub async fn unload_model(&mut self) -> Result<()> {
        let mut local_model = self.local_model.write().await;
        if let Some(ref mut engine) = *local_model {
            engine.unload().await;
        }
        *local_model = None;
        Ok(())
    }

    pub async fn stream_generate(
        &self,
        prompt: &str,
    ) -> Result<futures_util::stream::BoxStream<'static, Result<String>>> {
        let model = self.local_model.read().await;
        if let Some(ref engine) = *model {
            engine.stream_generate(prompt).await
        } else {
            Err(Error::ModelError("No local model loaded".to_string()))
        }
    }

    pub fn config(&self) -> &config::RuntimeConfig {
        &self.config
    }

    pub async fn update_settings(&mut self, settings: AppSettings) -> Result<()> {
        self.config.discovery.scan_interval_secs = settings.discovery_interval_secs;
        self.config.discovery.enable_wifi = true;
        self.config.discovery.enable_ble = true;
        Ok(())
    }

    pub async fn get_model_status(&self) -> ModelStatus {
        let model = self.local_model.read().await;
        match &*model {
            Some(engine) => ModelStatus {
                loaded: engine.is_loaded().await,
                name: engine.model_name().to_string(),
                backend: format!("{:?}", engine.backend_type()),
                quantization: format!("{:?}", engine.quantization()),
                context_length: Some(engine.context_length()),
                thread_count: Some(engine.thread_count()),
            },
            None => ModelStatus {
                loaded: false,
                name: "None".to_string(),
                backend: "None".to_string(),
                quantization: "None".to_string(),
                context_length: None,
                thread_count: None,
            },
        }
    }

    pub async fn get_available_models(&self) -> Vec<ModelInfo> {
        vec![
            ModelInfo {
                id: "qwen-3b".to_string(),
                name: "Qwen2.5-3B-Instruct".to_string(),
                size_bytes: 1_800_000_000,
                quantization: crate::types::MNNQuantization::INT8,
                context_length: 4096,
                downloaded: false,
                model_type: "causal-lm".to_string(),
            },
            ModelInfo {
                id: "qwen-7b".to_string(),
                name: "Qwen2.5-7B-Instruct".to_string(),
                size_bytes: 4_200_000_000,
                quantization: crate::types::MNNQuantization::INT8,
                context_length: 8192,
                downloaded: false,
                model_type: "causal-lm".to_string(),
            },
        ]
    }

    pub async fn get_hardware_info(&self) -> crate::types::HardwareInfo {
        crate::types::HardwareInfo {
            cpu_cores: num_cpus::get(),
            total_memory: 8 * 1024 * 1024 * 1024,
            gpu_available: false,
            gpu_type: None,
            gpu_memory: None,
            npu_available: false,
            npu_type: None,
            supports_fp16: true,
            supports_dotprod: cfg!(target_arch = "aarch64"),
        }
    }

    pub async fn get_user_profile(&self, user_id: &str) -> Result<UserProfileInfo> {
        let profile = self.user_profile.read().await;
        let user_profile = profile
            .get_profile(user_id)
            .await
            .ok_or_else(|| Error::ProfileError(format!("User {} not found", user_id)))?;
        let name = user_profile.user_id.clone();
        Ok(UserProfileInfo {
            user_id: user_profile.user_id,
            name,
            preferences: user_profile.preferences,
            device_usage: user_profile.device_usage,
            behavior_patterns: user_profile.behavior_patterns,
        })
    }

    pub async fn update_user_preferences(
        &mut self,
        user_id: &str,
        preferences: crate::types::UserPreferences,
    ) -> Result<()> {
        let profile = self.user_profile.write().await;
        profile.update_preferences(user_id, preferences).await
    }

    pub async fn get_recommendations(&self, user_id: &str) -> Result<Vec<RecommendationInfo>> {
        let devices = self.get_all_devices().await?;
        let rec = self.recommendation.read().await;
        let recommendations = rec.generate_recommendations(user_id, &devices).await?;
        Ok(recommendations
            .into_iter()
            .map(|r| {
                let action_type = format!("{:?}", r.recommendation_type);
                let device_id = r.actions.first().map(|a| a.device_id.clone());
                RecommendationInfo {
                    id: r.id,
                    title: r.title,
                    description: r.description,
                    action_type,
                    device_id,
                    confidence: r.confidence,
                }
            })
            .collect())
    }

    pub async fn get_conversation_history(&self, _conversation_id: &str) -> Vec<ChatMessageInfo> {
        let mcp = self.mcp_protocol.read().await;
        mcp.get_history()
    }

    pub async fn clear_conversation(&mut self, _conversation_id: &str) -> Result<()> {
        let mut mcp = self.mcp_protocol.write().await;
        mcp.clear_history();
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelStatus {
    pub loaded: bool,
    pub name: String,
    pub backend: String,
    pub quantization: String,
    pub context_length: Option<usize>,
    pub thread_count: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub id: String,
    pub name: String,
    pub size_bytes: u64,
    pub quantization: crate::types::MNNQuantization,
    pub context_length: usize,
    pub downloaded: bool,
    pub model_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfileInfo {
    pub user_id: String,
    pub name: String,
    pub preferences: crate::types::UserPreferences,
    pub device_usage: std::collections::HashMap<String, crate::profile::DeviceUsageStats>,
    pub behavior_patterns: Vec<crate::profile::BehaviorPattern>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecommendationInfo {
    pub id: String,
    pub title: String,
    pub description: String,
    pub action_type: String,
    pub device_id: Option<String>,
    pub confidence: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessageInfo {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub theme: Option<String>,
    pub language: Option<String>,
    pub auto_discover: bool,
    pub discovery_interval_secs: u64,
    pub power_mode: crate::types::PowerMode,
    pub notifications_enabled: bool,
}
