use crate::device::DeviceManager;
use crate::engine::LocalModelEngine;
use crate::error::Result;
use crate::network::{BluetoothManager, WiFiManager};
use crate::profile::{RecommendationEngine, UserProfileEngine};
use crate::protocols::{A2AProtocol, ACPProtocol, MCPProtocol};
use crate::runtime::{config::RuntimeConfig, MobileClawRuntime};
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct MobileClawRuntimeBuilder {
    config: RuntimeConfig,
    device_manager: Option<DeviceManager>,
    local_model: Option<LocalModelEngine>,
    user_profile: Option<UserProfileEngine>,
    recommendation: Option<RecommendationEngine>,
    wifi_manager: Option<WiFiManager>,
    ble_manager: Option<BluetoothManager>,
}

impl MobileClawRuntimeBuilder {
    pub fn new() -> Self {
        Self {
            config: RuntimeConfig::default(),
            device_manager: None,
            local_model: None,
            user_profile: None,
            recommendation: None,
            wifi_manager: None,
            ble_manager: None,
        }
    }

    pub fn config(mut self, config: RuntimeConfig) -> Self {
        self.config = config;
        self
    }

    pub fn device_manager(mut self, manager: DeviceManager) -> Self {
        self.device_manager = Some(manager);
        self
    }

    pub fn local_model(mut self, engine: LocalModelEngine) -> Self {
        self.local_model = Some(engine);
        self
    }

    pub fn user_profile(mut self, engine: UserProfileEngine) -> Self {
        self.user_profile = Some(engine);
        self
    }

    pub fn recommendation(mut self, engine: RecommendationEngine) -> Self {
        self.recommendation = Some(engine);
        self
    }

    pub fn wifi_manager(mut self, manager: WiFiManager) -> Self {
        self.wifi_manager = Some(manager);
        self
    }

    pub fn ble_manager(mut self, manager: BluetoothManager) -> Self {
        self.ble_manager = Some(manager);
        self
    }

    pub fn with_model(mut self, model_config: crate::runtime::config::ModelConfig) -> Self {
        self.config.model = Some(model_config);
        self
    }

    pub fn with_discovery(mut self, discovery_config: crate::runtime::config::DiscoveryConfig) -> Self {
        self.config.discovery = discovery_config;
        self
    }

    pub fn with_network(mut self, network_config: crate::runtime::config::NetworkConfig) -> Self {
        self.config.network = network_config;
        self
    }

    pub fn with_security(mut self, security_config: crate::runtime::config::SecurityConfig) -> Self {
        self.config.security = security_config;
        self
    }

    pub fn build(self) -> Result<MobileClawRuntime> {
        Ok(MobileClawRuntime {
            config: self.config,
            device_manager: Arc::new(RwLock::new(
                self.device_manager.unwrap_or_else(DeviceManager::new),
            )),
            a2a_protocol: Arc::new(RwLock::new(A2AProtocol::new())),
            acp_protocol: Arc::new(RwLock::new(ACPProtocol::new())),
            mcp_protocol: Arc::new(RwLock::new(MCPProtocol::new())),
            local_model: Arc::new(RwLock::new(self.local_model)),
            user_profile: Arc::new(RwLock::new(
                self.user_profile.unwrap_or_else(UserProfileEngine::new),
            )),
            recommendation: Arc::new(RwLock::new(
                self.recommendation.unwrap_or_else(RecommendationEngine::with_default_profile),
            )),
            wifi_manager: Arc::new(RwLock::new(self.wifi_manager)),
            ble_manager: Arc::new(RwLock::new(self.ble_manager)),
            running: Arc::new(RwLock::new(false)),
        })
    }
}

impl Default for MobileClawRuntimeBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn builder_creates_runtime() {
        let runtime = MobileClawRuntimeBuilder::new().build().unwrap();
        assert!(!runtime.is_running().await);
    }

    #[tokio::test]
    async fn builder_with_custom_config() {
        let config = RuntimeConfig {
            node_id: "test-node".to_string(),
            ..Default::default()
        };
        let runtime = MobileClawRuntimeBuilder::new()
            .config(config)
            .build()
            .unwrap();
        assert!(!runtime.is_running().await);
    }
}
