use crate::error::{Error, Result};
use crate::types::{ConnectionProtocol, DeviceInfo, DeviceState, DeviceType};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WiFiNetwork {
    pub ssid: String,
    pub signal_strength: i32,
    pub security: WiFiSecurity,
    pub frequency: WiFiFrequency,
    pub bssid: String,
    pub is_connected: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum WiFiSecurity {
    Open,
    WPA2,
    WPA3,
    WPA2WPA3,
    WEP,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum WiFiFrequency {
    TwoPointFourGHz,
    FiveGHz,
    SixGHz,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WiFiConfig {
    pub ssid: String,
    pub password: Option<String>,
    pub auto_connect: bool,
    pub priority: u8,
}

pub struct WiFiManager {
    connected: Arc<RwLock<Option<String>>>,
    saved_networks: Arc<RwLock<HashMap<String, WiFiConfig>>>,
    scan_results: Arc<RwLock<Vec<WiFiNetwork>>>,
}

impl WiFiManager {
    pub fn new() -> Self {
        Self {
            connected: Arc::new(RwLock::new(None)),
            saved_networks: Arc::new(RwLock::new(HashMap::new())),
            scan_results: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub async fn scan(&self) -> Result<Vec<WiFiNetwork>> {
        tracing::info!("Scanning WiFi networks");

        let networks = vec![
            WiFiNetwork {
                ssid: "HomeNetwork".to_string(),
                signal_strength: -45,
                security: WiFiSecurity::WPA2,
                frequency: WiFiFrequency::FiveGHz,
                bssid: "AA:BB:CC:DD:EE:01".to_string(),
                is_connected: false,
            },
            WiFiNetwork {
                ssid: "Guest_Network".to_string(),
                signal_strength: -60,
                security: WiFiSecurity::WPA2,
                frequency: WiFiFrequency::TwoPointFourGHz,
                bssid: "AA:BB:CC:DD:EE:02".to_string(),
                is_connected: false,
            },
        ];

        let mut results = self.scan_results.write().await;
        *results = networks.clone();

        Ok(networks)
    }

    pub async fn connect(&self, ssid: &str, password: Option<&str>) -> Result<()> {
        tracing::info!("Connecting to WiFi network: {}", ssid);

        let mut connected = self.connected.write().await;
        *connected = Some(ssid.to_string());

        let mut saved = self.saved_networks.write().await;
        saved.insert(
            ssid.to_string(),
            WiFiConfig {
                ssid: ssid.to_string(),
                password: password.map(|s| s.to_string()),
                auto_connect: true,
                priority: 1,
            },
        );

        tracing::info!("Connected to WiFi: {}", ssid);
        Ok(())
    }

    pub async fn disconnect(&self) -> Result<()> {
        let mut connected = self.connected.write().await;
        let ssid = connected.take();

        if let Some(ssid) = ssid {
            tracing::info!("Disconnected from WiFi: {}", ssid);
        }

        Ok(())
    }

    pub async fn get_connected_network(&self) -> Option<String> {
        self.connected.read().await.clone()
    }

    pub async fn is_connected(&self) -> bool {
        self.connected.read().await.is_some()
    }

    pub async fn get_signal_strength(&self) -> Option<i32> {
        if self.is_connected().await {
            Some(-45)
        } else {
            None
        }
    }

    pub async fn save_network(&self, config: WiFiConfig) -> Result<()> {
        let mut saved = self.saved_networks.write().await;
        saved.insert(config.ssid.clone(), config);
        Ok(())
    }

    pub async fn forget_network(&self, ssid: &str) -> Result<()> {
        let mut saved = self.saved_networks.write().await;
        saved.remove(ssid);
        tracing::info!("Forgot WiFi network: {}", ssid);
        Ok(())
    }

    pub async fn list_saved_networks(&self) -> Vec<WiFiConfig> {
        let saved = self.saved_networks.read().await;
        saved.values().cloned().collect()
    }

    pub async fn discover_devices(&self) -> Result<Vec<DeviceInfo>> {
        if !self.is_connected().await {
            return Err(Error::NetworkError("Not connected to WiFi".to_string()));
        }

        tracing::info!("Discovering devices on WiFi network");

        let devices = vec![DeviceInfo {
            id: "wifi-device-1".to_string(),
            name: "Living Room Light".to_string(),
            device_type: DeviceType::Light,
            capabilities: vec!["power".to_string(), "brightness".to_string()],
            endpoint: "192.168.1.100:8080".to_string(),
            port: 8080,
            protocol: ConnectionProtocol::HTTP,
            last_seen: Utc::now(),
            state: DeviceState {
                online: true,
                ..Default::default()
            },
        }];

        Ok(devices)
    }

    pub async fn get_local_ip(&self) -> Option<String> {
        if self.is_connected().await {
            Some("192.168.1.50".to_string())
        } else {
            None
        }
    }

    pub async fn disconnect_all(&mut self) -> Result<()> {
        self.disconnect().await
    }
}

impl Default for WiFiManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn manager_creation() {
        let manager = WiFiManager::new();
        assert!(!manager.is_connected().await);
    }

    #[tokio::test]
    async fn scan_networks() {
        let manager = WiFiManager::new();
        let networks = manager.scan().await.unwrap();
        assert!(!networks.is_empty());
    }

    #[tokio::test]
    async fn connect_disconnect() {
        let manager = WiFiManager::new();

        manager
            .connect("TestNetwork", Some("password"))
            .await
            .unwrap();
        assert!(manager.is_connected().await);
        assert_eq!(
            manager.get_connected_network().await,
            Some("TestNetwork".to_string())
        );

        manager.disconnect().await.unwrap();
        assert!(!manager.is_connected().await);
    }

    #[tokio::test]
    async fn save_forget_network() {
        let manager = WiFiManager::new();

        let config = WiFiConfig {
            ssid: "SavedNetwork".to_string(),
            password: Some("pass".to_string()),
            auto_connect: true,
            priority: 1,
        };

        manager.save_network(config).await.unwrap();
        let saved = manager.list_saved_networks().await;
        assert_eq!(saved.len(), 1);

        manager.forget_network("SavedNetwork").await.unwrap();
        let saved = manager.list_saved_networks().await;
        assert!(saved.is_empty());
    }

    #[tokio::test]
    async fn discover_devices() {
        let manager = WiFiManager::new();

        let result = manager.discover_devices().await;
        assert!(result.is_err());

        manager.connect("TestNetwork", None).await.unwrap();
        let devices = manager.discover_devices().await.unwrap();
        assert!(!devices.is_empty());
    }
}
