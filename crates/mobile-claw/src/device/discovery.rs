use crate::device::DeviceManager;
use crate::error::Result;
use crate::types::{ConnectionProtocol, DeviceInfo, DeviceState, DeviceType};
use chrono::Utc;
use std::net::{IpAddr, Ipv4Addr};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

pub struct DeviceDiscovery {
    manager: Arc<RwLock<DeviceManager>>,
    discovery_active: Arc<RwLock<bool>>,
    scan_interval: Duration,
    timeout: Duration,
}

impl DeviceDiscovery {
    pub fn new(manager: Arc<RwLock<DeviceManager>>) -> Self {
        Self {
            manager,
            discovery_active: Arc::new(RwLock::new(false)),
            scan_interval: Duration::from_secs(30),
            timeout: Duration::from_secs(5),
        }
    }

    pub fn with_config(
        manager: Arc<RwLock<DeviceManager>>,
        scan_interval: Duration,
        timeout: Duration,
    ) -> Self {
        Self {
            manager,
            discovery_active: Arc::new(RwLock::new(false)),
            scan_interval,
            timeout,
        }
    }

    pub async fn start_discovery(&self) -> Result<()> {
        let mut active = self.discovery_active.write().await;
        if *active {
            return Ok(());
        }
        *active = true;
        drop(active);

        tracing::info!("Starting device discovery");

        self.scan_network().await?;

        Ok(())
    }

    pub async fn stop_discovery(&self) {
        let mut active = self.discovery_active.write().await;
        *active = false;
        tracing::info!("Stopped device discovery");
    }

    pub async fn is_discovering(&self) -> bool {
        *self.discovery_active.read().await
    }

    async fn scan_network(&self) -> Result<Vec<DeviceInfo>> {
        let mut discovered = Vec::new();

        let base_ip = "192.168.1";
        for i in 1..=10u8 {
            let ip = format!("{}.{}", base_ip, i);
            if let Some(device) = self.probe_device(&ip).await {
                discovered.push(device);
            }
        }

        for device in &discovered {
            let mut manager = self.manager.write().await;
            manager.register_device(device.clone()).await?;
        }

        tracing::info!("Discovered {} devices", discovered.len());
        Ok(discovered)
    }

    async fn probe_device(&self, ip: &str) -> Option<DeviceInfo> {
        tokio::time::sleep(Duration::from_millis(10)).await;

        if ip.ends_with(".1") || ip.ends_with(".100") {
            return None;
        }

        let device_type = self.detect_device_type(ip).await?;

        Some(DeviceInfo {
            id: format!("device-{}", ip.replace('.', "-")),
            name: format!("Device at {}", ip),
            device_type,
            capabilities: vec!["power".to_string()],
            endpoint: format!("{}:8080", ip),
            port: 8080,
            protocol: ConnectionProtocol::HTTP,
            last_seen: Utc::now(),
            state: DeviceState {
                online: true,
                ..Default::default()
            },
        })
    }

    async fn detect_device_type(&self, ip: &str) -> Option<DeviceType> {
        let last_octet: u8 = ip.split('.').last()?.parse().ok()?;

        match last_octet % 6 {
            0 => Some(DeviceType::Light),
            1 => Some(DeviceType::AirConditioner),
            2 => Some(DeviceType::Camera),
            3 => Some(DeviceType::Television),
            4 => Some(DeviceType::SmartLock),
            5 => Some(DeviceType::Curtain),
            _ => Some(DeviceType::Sensor),
        }
    }

    pub async fn discover_by_type(&self, device_type: DeviceType) -> Result<Vec<DeviceInfo>> {
        let devices = self.scan_network().await?;
        Ok(devices
            .into_iter()
            .filter(|d| d.device_type == device_type)
            .collect())
    }

    pub async fn discover_mdns(&self) -> Result<Vec<DeviceInfo>> {
        tracing::debug!("Scanning via mDNS");
        Ok(Vec::new())
    }

    pub async fn discover_ssdp(&self) -> Result<Vec<DeviceInfo>> {
        tracing::debug!("Scanning via SSDP/UPnP");
        Ok(Vec::new())
    }

    pub async fn discover_bluetooth(&self) -> Result<Vec<DeviceInfo>> {
        tracing::debug!("Scanning via Bluetooth");
        Ok(Vec::new())
    }

    pub async fn refresh_device(&self, device_id: &str) -> Result<Option<DeviceInfo>> {
        let manager = self.manager.read().await;
        if let Some(device) = manager.get_device(device_id).await {
            let refreshed = DeviceInfo {
                last_seen: Utc::now(),
                ..device
            };
            drop(manager);

            let mut manager = self.manager.write().await;
            manager.register_device(refreshed.clone()).await?;
            return Ok(Some(refreshed));
        }
        Ok(None)
    }
}

#[derive(Debug, Clone)]
pub struct DiscoveryConfig {
    pub enable_mdns: bool,
    pub enable_ssdp: bool,
    pub enable_bluetooth: bool,
    pub scan_interval_secs: u64,
    pub timeout_secs: u64,
    pub network_prefix: String,
}

impl Default for DiscoveryConfig {
    fn default() -> Self {
        Self {
            enable_mdns: true,
            enable_ssdp: true,
            enable_bluetooth: true,
            scan_interval_secs: 30,
            timeout_secs: 5,
            network_prefix: "192.168.1".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn discovery_creation() {
        let manager = Arc::new(RwLock::new(DeviceManager::new()));
        let discovery = DeviceDiscovery::new(manager);
        assert!(!discovery.is_discovering().await);
    }

    #[tokio::test]
    async fn start_stop_discovery() {
        let manager = Arc::new(RwLock::new(DeviceManager::new()));
        let discovery = DeviceDiscovery::new(manager);

        discovery.start_discovery().await.unwrap();
        assert!(discovery.is_discovering().await);

        discovery.stop_discovery().await;
        assert!(!discovery.is_discovering().await);
    }

    #[test]
    fn discovery_config_default() {
        let config = DiscoveryConfig::default();
        assert!(config.enable_mdns);
        assert!(config.enable_ssdp);
        assert!(config.enable_bluetooth);
    }
}
