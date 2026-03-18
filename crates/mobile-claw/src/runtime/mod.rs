pub mod config;
pub mod builder;

pub use builder::MobileClawRuntimeBuilder;

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
            tracing::info!("Local model engine initialized: {}", model_config.model_name);
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
}
