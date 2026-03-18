use crate::error::{Error, Result};
use crate::types::{DeviceId, DeviceInfo, DeviceState, DeviceType, ConnectionProtocol};
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct DeviceManager {
    devices: Arc<RwLock<HashMap<DeviceId, DeviceInfo>>>,
    connections: Arc<RwLock<HashMap<DeviceId, DeviceConnection>>>,
    groups: Arc<RwLock<HashMap<String, Vec<DeviceId>>>>,
}

#[derive(Debug, Clone)]
pub struct DeviceConnection {
    pub device_id: DeviceId,
    pub status: ConnectionStatus,
    pub last_active: DateTime<Utc>,
    pub retry_count: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConnectionStatus {
    Disconnected,
    Connecting,
    Connected,
    Reconnecting,
    Error(String),
}

impl DeviceManager {
    pub fn new() -> Self {
        Self {
            devices: Arc::new(RwLock::new(HashMap::new())),
            connections: Arc::new(RwLock::new(HashMap::new())),
            groups: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn register_device(&self, device: DeviceInfo) -> Result<()> {
        let device_id = device.id.clone();
        tracing::info!("Registering device: {} ({:?})", device_id, device.device_type);

        let mut devices = self.devices.write().await;
        devices.insert(device_id.clone(), device);

        let mut connections = self.connections.write().await;
        connections.insert(
            device_id.clone(),
            DeviceConnection {
                device_id,
                status: ConnectionStatus::Disconnected,
                last_active: Utc::now(),
                retry_count: 0,
            },
        );

        Ok(())
    }

    pub async fn unregister_device(&self, device_id: &str) -> Result<()> {
        tracing::info!("Unregistering device: {}", device_id);

        let mut devices = self.devices.write().await;
        devices.remove(device_id);

        let mut connections = self.connections.write().await;
        connections.remove(device_id);

        Ok(())
    }

    pub async fn get_device(&self, device_id: &str) -> Option<DeviceInfo> {
        let devices = self.devices.read().await;
        devices.get(device_id).cloned()
    }

    pub async fn list_devices(&self) -> Vec<DeviceInfo> {
        let devices = self.devices.read().await;
        devices.values().cloned().collect()
    }

    pub async fn list_devices_by_type(&self, device_type: DeviceType) -> Vec<DeviceInfo> {
        let devices = self.devices.read().await;
        devices
            .values()
            .filter(|d| d.device_type == device_type)
            .cloned()
            .collect()
    }

    pub async fn list_online_devices(&self) -> Vec<DeviceInfo> {
        let devices = self.devices.read().await;
        devices
            .values()
            .filter(|d| d.state.online)
            .cloned()
            .collect()
    }

    pub async fn update_device_state(&self, device_id: &str, state: DeviceState) -> Result<()> {
        let mut devices = self.devices.write().await;
        if let Some(device) = devices.get_mut(device_id) {
            device.state = state;
            device.last_seen = Utc::now();
            Ok(())
        } else {
            Err(Error::DeviceNotFound(device_id.to_string()))
        }
    }

    pub async fn connect(&self, device_id: &str) -> Result<()> {
        let devices = self.devices.read().await;
        if !devices.contains_key(device_id) {
            return Err(Error::DeviceNotFound(device_id.to_string()));
        }
        drop(devices);

        let mut connections = self.connections.write().await;
        if let Some(conn) = connections.get_mut(device_id) {
            conn.status = ConnectionStatus::Connected;
            conn.last_active = Utc::now();
            conn.retry_count = 0;
        }

        let mut devices = self.devices.write().await;
        if let Some(device) = devices.get_mut(device_id) {
            device.state.online = true;
        }

        tracing::info!("Connected to device: {}", device_id);
        Ok(())
    }

    pub async fn disconnect(&self, device_id: &str) -> Result<()> {
        let mut connections = self.connections.write().await;
        if let Some(conn) = connections.get_mut(device_id) {
            conn.status = ConnectionStatus::Disconnected;
        }

        let mut devices = self.devices.write().await;
        if let Some(device) = devices.get_mut(device_id) {
            device.state.online = false;
        }

        tracing::info!("Disconnected from device: {}", device_id);
        Ok(())
    }

    pub async fn get_connection_status(&self, device_id: &str) -> Option<ConnectionStatus> {
        let connections = self.connections.read().await;
        connections.get(device_id).map(|c| c.status.clone())
    }

    pub async fn create_group(&self, group_name: &str, device_ids: Vec<DeviceId>) -> Result<()> {
        let mut groups = self.groups.write().await;
        groups.insert(group_name.to_string(), device_ids);
        tracing::info!("Created device group: {}", group_name);
        Ok(())
    }

    pub async fn add_to_group(&self, group_name: &str, device_id: DeviceId) -> Result<()> {
        let mut groups = self.groups.write().await;
        groups
            .entry(group_name.to_string())
            .or_default()
            .push(device_id);
        Ok(())
    }

    pub async fn remove_from_group(&self, group_name: &str, device_id: &str) -> Result<()> {
        let mut groups = self.groups.write().await;
        if let Some(devices) = groups.get_mut(group_name) {
            devices.retain(|id| id != device_id);
        }
        Ok(())
    }

    pub async fn get_group_devices(&self, group_name: &str) -> Option<Vec<DeviceInfo>> {
        let groups = self.groups.read().await;
        let device_ids = groups.get(group_name)?;

        let devices = self.devices.read().await;
        Some(
            device_ids
                .iter()
                .filter_map(|id| devices.get(id).cloned())
                .collect(),
        )
    }

    pub async fn list_groups(&self) -> Vec<String> {
        let groups = self.groups.read().await;
        groups.keys().cloned().collect()
    }

    pub async fn device_count(&self) -> usize {
        self.devices.read().await.len()
    }

    pub async fn online_count(&self) -> usize {
        let devices = self.devices.read().await;
        devices.values().filter(|d| d.state.online).count()
    }

    pub async fn refresh_device(&self, device_id: &str) -> Result<()> {
        let mut devices = self.devices.write().await;
        if let Some(device) = devices.get_mut(device_id) {
            device.last_seen = Utc::now();
            Ok(())
        } else {
            Err(Error::DeviceNotFound(device_id.to_string()))
        }
    }

    pub async fn cleanup_stale_devices(&self, timeout_secs: i64) -> Result<Vec<DeviceId>> {
        let now = Utc::now();
        let timeout = chrono::Duration::seconds(timeout_secs);

        let mut devices = self.devices.write().await;
        let mut connections = self.connections.write().await;

        let stale: Vec<DeviceId> = devices
            .iter()
            .filter(|(_, device)| now - device.last_seen > timeout)
            .map(|(id, _)| id.clone())
            .collect();

        for id in &stale {
            devices.remove(id);
            connections.remove(id);
            tracing::warn!("Removed stale device: {}", id);
        }

        Ok(stale)
    }
}

impl Default for DeviceManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_device(id: &str, device_type: DeviceType) -> DeviceInfo {
        DeviceInfo {
            id: id.to_string(),
            name: format!("Test {}", id),
            device_type,
            capabilities: vec!["power".to_string()],
            endpoint: "192.168.1.100:8080".to_string(),
            port: 8080,
            protocol: ConnectionProtocol::HTTP,
            last_seen: Utc::now(),
            state: DeviceState::default(),
        }
    }

    #[tokio::test]
    async fn register_and_get_device() {
        let manager = DeviceManager::new();
        let device = create_test_device("light-1", DeviceType::Light);

        manager.register_device(device).await.unwrap();

        let retrieved = manager.get_device("light-1").await;
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().name, "Test light-1");
    }

    #[tokio::test]
    async fn unregister_device() {
        let manager = DeviceManager::new();
        let device = create_test_device("light-2", DeviceType::Light);

        manager.register_device(device).await.unwrap();
        manager.unregister_device("light-2").await.unwrap();

        let retrieved = manager.get_device("light-2").await;
        assert!(retrieved.is_none());
    }

    #[tokio::test]
    async fn list_devices_by_type() {
        let manager = DeviceManager::new();

        manager
            .register_device(create_test_device("light-3", DeviceType::Light))
            .await
            .unwrap();
        manager
            .register_device(create_test_device("ac-1", DeviceType::AirConditioner))
            .await
            .unwrap();

        let lights = manager.list_devices_by_type(DeviceType::Light).await;
        assert_eq!(lights.len(), 1);

        let acs = manager.list_devices_by_type(DeviceType::AirConditioner).await;
        assert_eq!(acs.len(), 1);
    }

    #[tokio::test]
    async fn connect_disconnect() {
        let manager = DeviceManager::new();
        manager
            .register_device(create_test_device("light-4", DeviceType::Light))
            .await
            .unwrap();

        manager.connect("light-4").await.unwrap();
        let status = manager.get_connection_status("light-4").await;
        assert_eq!(status, Some(ConnectionStatus::Connected));

        manager.disconnect("light-4").await.unwrap();
        let status = manager.get_connection_status("light-4").await;
        assert_eq!(status, Some(ConnectionStatus::Disconnected));
    }

    #[tokio::test]
    async fn device_groups() {
        let manager = DeviceManager::new();
        manager
            .register_device(create_test_device("light-5", DeviceType::Light))
            .await
            .unwrap();
        manager
            .register_device(create_test_device("light-6", DeviceType::Light))
            .await
            .unwrap();

        manager
            .create_group("living_room", vec!["light-5".to_string(), "light-6".to_string()])
            .await
            .unwrap();

        let group_devices = manager.get_group_devices("living_room").await;
        assert!(group_devices.is_some());
        assert_eq!(group_devices.unwrap().len(), 2);

        let groups = manager.list_groups().await;
        assert_eq!(groups.len(), 1);
    }

    #[tokio::test]
    async fn update_device_state() {
        let manager = DeviceManager::new();
        manager
            .register_device(create_test_device("ac-2", DeviceType::AirConditioner))
            .await
            .unwrap();

        let new_state = DeviceState {
            online: true,
            power: Some(true),
            temperature: Some(24.0),
            ..Default::default()
        };

        manager.update_device_state("ac-2", new_state).await.unwrap();

        let device = manager.get_device("ac-2").await.unwrap();
        assert_eq!(device.state.temperature, Some(24.0));
    }

    #[tokio::test]
    async fn device_count() {
        let manager = DeviceManager::new();
        assert_eq!(manager.device_count().await, 0);

        manager
            .register_device(create_test_device("d1", DeviceType::Light))
            .await
            .unwrap();
        assert_eq!(manager.device_count().await, 1);
    }
}
