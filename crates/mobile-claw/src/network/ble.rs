use crate::error::{Error, Result};
use crate::types::{ConnectionProtocol, DeviceInfo, DeviceState, DeviceType};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BLEDevice {
    pub address: String,
    pub name: Option<String>,
    pub rssi: i32,
    pub services: Vec<String>,
    pub is_connectable: bool,
    pub is_connected: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BLEService {
    pub uuid: String,
    pub characteristics: Vec<BLECharacteristic>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BLECharacteristic {
    pub uuid: String,
    pub properties: Vec<String>,
}

pub struct BluetoothManager {
    scanning: Arc<RwLock<bool>>,
    connected_devices: Arc<RwLock<HashMap<String, BLEDevice>>>,
    discovered_devices: Arc<RwLock<Vec<BLEDevice>>>,
}

impl BluetoothManager {
    pub fn new() -> Self {
        Self {
            scanning: Arc::new(RwLock::new(false)),
            connected_devices: Arc::new(RwLock::new(HashMap::new())),
            discovered_devices: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub async fn start_scan(&self, duration_secs: u64) -> Result<Vec<BLEDevice>> {
        let mut scanning = self.scanning.write().await;
        if *scanning {
            return Err(Error::NetworkError("Already scanning".to_string()));
        }
        *scanning = true;
        drop(scanning);

        tracing::info!("Starting BLE scan for {} seconds", duration_secs);

        tokio::time::sleep(std::time::Duration::from_secs(duration_secs.min(10))).await;

        let devices = vec![
            BLEDevice {
                address: "AA:BB:CC:DD:EE:FF".to_string(),
                name: Some("Smart Lock".to_string()),
                rssi: -50,
                services: vec!["0000180D".to_string()],
                is_connectable: true,
                is_connected: false,
            },
            BLEDevice {
                address: "11:22:33:44:55:66".to_string(),
                name: Some("BLE Light".to_string()),
                rssi: -65,
                services: vec!["00001800".to_string()],
                is_connectable: true,
                is_connected: false,
            },
        ];

        let mut discovered = self.discovered_devices.write().await;
        *discovered = devices.clone();

        let mut scanning = self.scanning.write().await;
        *scanning = false;

        Ok(devices)
    }

    pub async fn stop_scan(&self) -> Result<()> {
        let mut scanning = self.scanning.write().await;
        *scanning = false;
        tracing::info!("BLE scan stopped");
        Ok(())
    }

    pub async fn is_scanning(&self) -> bool {
        *self.scanning.read().await
    }

    pub async fn connect(&self, address: &str) -> Result<BLEDevice> {
        let discovered = self.discovered_devices.read().await;
        let device = discovered
            .iter()
            .find(|d| d.address == address)
            .ok_or_else(|| Error::DeviceNotFound(address.to_string()))?
            .clone();
        drop(discovered);

        tracing::info!("Connecting to BLE device: {}", address);

        let mut connected = self.connected_devices.write().await;
        let mut device = device;
        device.is_connected = true;
        connected.insert(address.to_string(), device.clone());

        Ok(device)
    }

    pub async fn disconnect(&self, address: &str) -> Result<()> {
        let mut connected = self.connected_devices.write().await;
        if let Some(mut device) = connected.remove(address) {
            device.is_connected = false;
            tracing::info!("Disconnected from BLE device: {}", address);
        }
        Ok(())
    }

    pub async fn get_connected_devices(&self) -> Vec<BLEDevice> {
        let connected = self.connected_devices.read().await;
        connected.values().cloned().collect()
    }

    pub async fn read_characteristic(
        &self,
        address: &str,
        service_uuid: &str,
        char_uuid: &str,
    ) -> Result<Vec<u8>> {
        let connected = self.connected_devices.read().await;
        if !connected.contains_key(address) {
            return Err(Error::DeviceNotFound(address.to_string()));
        }

        tracing::debug!("Reading characteristic {} from {}", char_uuid, address);
        Ok(vec![0x01, 0x02, 0x03])
    }

    pub async fn write_characteristic(
        &self,
        address: &str,
        service_uuid: &str,
        char_uuid: &str,
        data: &[u8],
    ) -> Result<()> {
        let connected = self.connected_devices.read().await;
        if !connected.contains_key(address) {
            return Err(Error::DeviceNotFound(address.to_string()));
        }

        tracing::debug!(
            "Writing to characteristic {} on {}: {:02x?}",
            char_uuid,
            address,
            data
        );
        Ok(())
    }

    pub async fn subscribe_characteristic(
        &self,
        address: &str,
        service_uuid: &str,
        char_uuid: &str,
    ) -> Result<tokio::sync::mpsc::Receiver<Vec<u8>>> {
        let connected = self.connected_devices.read().await;
        if !connected.contains_key(address) {
            return Err(Error::DeviceNotFound(address.to_string()));
        }

        let (tx, rx) = tokio::sync::mpsc::channel(16);

        tracing::debug!("Subscribed to characteristic {} on {}", char_uuid, address);

        Ok(rx)
    }

    pub async fn discover_services(&self, address: &str) -> Result<Vec<BLEService>> {
        let connected = self.connected_devices.read().await;
        if !connected.contains_key(address) {
            return Err(Error::DeviceNotFound(address.to_string()));
        }

        Ok(vec![BLEService {
            uuid: "00001800".to_string(),
            characteristics: vec![BLECharacteristic {
                uuid: "00002A00".to_string(),
                properties: vec!["read".to_string()],
            }],
        }])
    }

    pub async fn to_device_info(&self, ble_device: &BLEDevice) -> DeviceInfo {
        let device_type = ble_device
            .name
            .as_ref()
            .map(|n| {
                if n.contains("Lock") {
                    DeviceType::SmartLock
                } else if n.contains("Light") {
                    DeviceType::Light
                } else {
                    DeviceType::Sensor
                }
            })
            .unwrap_or(DeviceType::Sensor);

        DeviceInfo {
            id: format!("ble-{}", ble_device.address.replace(':', "-")),
            name: ble_device
                .name
                .clone()
                .unwrap_or_else(|| ble_device.address.clone()),
            device_type,
            capabilities: vec!["ble".to_string()],
            endpoint: ble_device.address.clone(),
            port: 0,
            protocol: ConnectionProtocol::BLE,
            last_seen: Utc::now(),
            state: DeviceState {
                online: ble_device.is_connected,
                ..Default::default()
            },
        }
    }

    pub async fn scan(&self, duration: std::time::Duration) -> Result<Vec<DeviceInfo>> {
        let ble_devices = self.start_scan(duration.as_secs()).await?;
        let mut devices = Vec::new();
        for ble_device in ble_devices {
            devices.push(self.to_device_info(&ble_device).await);
        }
        Ok(devices)
    }

    pub async fn disconnect_all(&mut self) -> Result<()> {
        let mut connected = self.connected_devices.write().await;
        let addresses: Vec<String> = connected.keys().cloned().collect();
        drop(connected);

        for address in addresses {
            self.disconnect(&address).await?;
        }
        Ok(())
    }
}

impl Default for BluetoothManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn manager_creation() {
        let manager = BluetoothManager::new();
        assert!(!manager.is_scanning().await);
    }

    #[tokio::test]
    async fn scan_devices() {
        let manager = BluetoothManager::new();
        let devices = manager.start_scan(5).await.unwrap();
        assert!(!devices.is_empty());
        assert!(!manager.is_scanning().await);
    }

    #[tokio::test]
    async fn connect_disconnect() {
        let manager = BluetoothManager::new();

        manager.start_scan(1).await.unwrap();

        let device = manager.connect("AA:BB:CC:DD:EE:FF").await.unwrap();
        assert!(device.is_connected);

        let connected = manager.get_connected_devices().await;
        assert_eq!(connected.len(), 1);

        manager.disconnect("AA:BB:CC:DD:EE:FF").await.unwrap();
        let connected = manager.get_connected_devices().await;
        assert!(connected.is_empty());
    }

    #[tokio::test]
    async fn read_write_characteristic() {
        let manager = BluetoothManager::new();
        manager.start_scan(1).await.unwrap();
        manager.connect("AA:BB:CC:DD:EE:FF").await.unwrap();

        let data = manager
            .read_characteristic("AA:BB:CC:DD:EE:FF", "00001800", "00002A00")
            .await
            .unwrap();
        assert!(!data.is_empty());

        manager
            .write_characteristic("AA:BB:CC:DD:EE:FF", "00001800", "00002A00", &[0x01])
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn to_device_info() {
        let manager = BluetoothManager::new();
        let ble_device = BLEDevice {
            address: "AA:BB:CC:DD:EE:FF".to_string(),
            name: Some("Smart Lock".to_string()),
            rssi: -50,
            services: vec![],
            is_connectable: true,
            is_connected: true,
        };

        let device_info = manager.to_device_info(&ble_device).await;
        assert_eq!(device_info.device_type, DeviceType::SmartLock);
        assert_eq!(device_info.protocol, ConnectionProtocol::BLE);
    }
}
