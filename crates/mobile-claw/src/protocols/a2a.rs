use crate::error::{Error, Result};
use crate::types::{DeviceInfo, PeerId, PeerInfo};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};

const PROTOCOL_VERSION: &str = "1.0.0";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum A2AMessage {
    Hello {
        node_id: String,
        capabilities: Vec<String>,
        protocol_version: String,
    },
    DeviceDiscovery {
        query: String,
        filters: Option<DeviceFilters>,
    },
    DeviceControl {
        device_id: String,
        command: DeviceCommandPayload,
        correlation_id: String,
    },
    Telemetry {
        device_id: String,
        data: serde_json::Value,
        timestamp: DateTime<Utc>,
    },
    Heartbeat {
        node_id: String,
        timestamp: DateTime<Utc>,
    },
    Bye {
        node_id: String,
        reason: Option<String>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceFilters {
    pub device_types: Option<Vec<String>>,
    pub capabilities: Option<Vec<String>>,
    pub online_only: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceCommandPayload {
    pub action: String,
    pub parameters: HashMap<String, serde_json::Value>,
}

pub struct A2AProtocol {
    version: String,
    node_id: String,
    peers: Arc<RwLock<HashMap<PeerId, PeerInfo>>>,
    discovery_active: Arc<RwLock<bool>>,
    message_sender: broadcast::Sender<A2AMessage>,
    message_receiver: Arc<RwLock<Option<broadcast::Receiver<A2AMessage>>>>,
}

impl A2AProtocol {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(256);
        Self {
            version: PROTOCOL_VERSION.to_string(),
            node_id: uuid::Uuid::new_v4().to_string(),
            peers: Arc::new(RwLock::new(HashMap::new())),
            discovery_active: Arc::new(RwLock::new(false)),
            message_sender: tx,
            message_receiver: Arc::new(RwLock::new(None)),
        }
    }

    pub fn with_node_id(node_id: impl Into<String>) -> Self {
        let mut protocol = Self::new();
        protocol.node_id = node_id.into();
        protocol
    }

    pub async fn start_discovery(&self) -> Result<()> {
        let mut active = self.discovery_active.write().await;
        if *active {
            return Ok(());
        }

        tracing::info!("Starting A2A discovery service");

        self.broadcast_hello().await?;

        *active = true;
        Ok(())
    }

    pub async fn stop_discovery(&self) -> Result<()> {
        let mut active = self.discovery_active.write().await;
        if !*active {
            return Ok(());
        }

        self.broadcast_bye(None).await?;

        *active = false;
        tracing::info!("A2A discovery service stopped");
        Ok(())
    }

    async fn broadcast_hello(&self) -> Result<()> {
        let message = A2AMessage::Hello {
            node_id: self.node_id.clone(),
            capabilities: vec![
                "device_control".to_string(),
                "telemetry".to_string(),
                "multimodal".to_string(),
            ],
            protocol_version: self.version.clone(),
        };
        self.broadcast(&message).await
    }

    async fn broadcast_bye(&self, reason: Option<String>) -> Result<()> {
        let message = A2AMessage::Bye {
            node_id: self.node_id.clone(),
            reason,
        };
        self.broadcast(&message).await
    }

    async fn broadcast(&self, message: &A2AMessage) -> Result<()> {
        let _ = self.message_sender.send(message.clone());
        tracing::debug!("Broadcasting A2A message: {:?}", message);
        Ok(())
    }

    pub async fn discover_peers(&self) -> Result<Vec<PeerInfo>> {
        let peers = self.peers.read().await;
        Ok(peers.values().cloned().collect())
    }

    pub async fn register_peer(&self, peer: PeerInfo) -> Result<()> {
        let mut peers = self.peers.write().await;
        peers.insert(peer.id.clone(), peer.clone());
        tracing::info!("Registered peer: {} ({})", peer.id, peer.endpoint);
        Ok(())
    }

    pub async fn unregister_peer(&self, peer_id: &str) -> Result<()> {
        let mut peers = self.peers.write().await;
        if peers.remove(peer_id).is_some() {
            tracing::info!("Unregistered peer: {}", peer_id);
        }
        Ok(())
    }

    pub async fn send_to_peer(&self, peer_id: &str, message: &A2AMessage) -> Result<()> {
        let peers = self.peers.read().await;
        if !peers.contains_key(peer_id) {
            return Err(Error::ProtocolError(format!("Unknown peer: {}", peer_id)));
        }
        self.broadcast(message).await
    }

    pub async fn broadcast_device_discovery(
        &self,
        query: &str,
        filters: Option<DeviceFilters>,
    ) -> Result<()> {
        let message = A2AMessage::DeviceDiscovery {
            query: query.to_string(),
            filters,
        };
        self.broadcast(&message).await
    }

    pub async fn send_heartbeat(&self) -> Result<()> {
        let message = A2AMessage::Heartbeat {
            node_id: self.node_id.clone(),
            timestamp: Utc::now(),
        };
        self.broadcast(&message).await
    }

    pub async fn send_telemetry(
        &self,
        device_id: &str,
        data: serde_json::Value,
    ) -> Result<()> {
        let message = A2AMessage::Telemetry {
            device_id: device_id.to_string(),
            data,
            timestamp: Utc::now(),
        };
        self.broadcast(&message).await
    }

    pub fn subscribe(&self) -> broadcast::Receiver<A2AMessage> {
        self.message_sender.subscribe()
    }

    pub async fn handle_message(&self, message: A2AMessage) -> Result<()> {
        match message {
            A2AMessage::Hello {
                node_id,
                capabilities,
                protocol_version,
            } => {
                let peer = PeerInfo {
                    id: node_id.clone(),
                    endpoint: String::new(),
                    capabilities,
                    last_seen: Utc::now(),
                    protocol_version,
                };
                self.register_peer(peer).await?;
            }
            A2AMessage::Heartbeat { node_id, timestamp } => {
                let mut peers = self.peers.write().await;
                if let Some(peer) = peers.get_mut(&node_id) {
                    peer.last_seen = timestamp;
                }
            }
            A2AMessage::Bye { node_id, reason } => {
                if let Some(r) = reason {
                    tracing::info!("Peer {} left: {}", node_id, r);
                }
                self.unregister_peer(&node_id).await?;
            }
            _ => {}
        }
        Ok(())
    }

    pub async fn cleanup_stale_peers(&self, timeout_secs: i64) -> Result<()> {
        let mut peers = self.peers.write().await;
        let now = Utc::now();
        let timeout = chrono::Duration::seconds(timeout_secs);

        peers.retain(|id, peer| {
            let is_stale = now - peer.last_seen > timeout;
            if is_stale {
                tracing::warn!("Removing stale peer: {}", id);
            }
            !is_stale
        });

        Ok(())
    }

    pub fn node_id(&self) -> &str {
        &self.node_id
    }

    pub fn version(&self) -> &str {
        &self.version
    }
}

impl Default for A2AProtocol {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn protocol_initialization() {
        let protocol = A2AProtocol::new();
        assert_eq!(protocol.version(), "1.0.0");
        assert!(!protocol.node_id().is_empty());
    }

    #[tokio::test]
    async fn start_discovery() {
        let protocol = A2AProtocol::new();
        protocol.start_discovery().await.unwrap();
        
        let active = protocol.discovery_active.read().await;
        assert!(*active);
    }

    #[tokio::test]
    async fn register_and_discover_peers() {
        let protocol = A2AProtocol::new();
        
        let peer = PeerInfo {
            id: "peer-1".to_string(),
            endpoint: "192.168.1.100:8080".to_string(),
            capabilities: vec!["device_control".to_string()],
            last_seen: Utc::now(),
            protocol_version: "1.0.0".to_string(),
        };
        
        protocol.register_peer(peer).await.unwrap();
        
        let peers = protocol.discover_peers().await.unwrap();
        assert_eq!(peers.len(), 1);
        assert_eq!(peers[0].id, "peer-1");
    }

    #[tokio::test]
    async fn unregister_peer() {
        let protocol = A2AProtocol::new();
        
        let peer = PeerInfo {
            id: "peer-2".to_string(),
            endpoint: "192.168.1.101:8080".to_string(),
            capabilities: vec![],
            last_seen: Utc::now(),
            protocol_version: "1.0.0".to_string(),
        };
        
        protocol.register_peer(peer).await.unwrap();
        protocol.unregister_peer("peer-2").await.unwrap();
        
        let peers = protocol.discover_peers().await.unwrap();
        assert!(peers.is_empty());
    }

    #[tokio::test]
    async fn handle_hello_message() {
        let protocol = A2AProtocol::new();
        
        let message = A2AMessage::Hello {
            node_id: "new-peer".to_string(),
            capabilities: vec!["test".to_string()],
            protocol_version: "1.0.0".to_string(),
        };
        
        protocol.handle_message(message).await.unwrap();
        
        let peers = protocol.discover_peers().await.unwrap();
        assert_eq!(peers.len(), 1);
    }

    #[tokio::test]
    async fn cleanup_stale_peers() {
        let protocol = A2AProtocol::new();
        
        let old_peer = PeerInfo {
            id: "old-peer".to_string(),
            endpoint: "".to_string(),
            capabilities: vec![],
            last_seen: Utc::now() - chrono::Duration::seconds(100),
            protocol_version: "1.0.0".to_string(),
        };
        
        let new_peer = PeerInfo {
            id: "new-peer".to_string(),
            endpoint: "".to_string(),
            capabilities: vec![],
            last_seen: Utc::now(),
            protocol_version: "1.0.0".to_string(),
        };
        
        protocol.register_peer(old_peer).await.unwrap();
        protocol.register_peer(new_peer).await.unwrap();
        
        protocol.cleanup_stale_peers(60).await.unwrap();
        
        let peers = protocol.discover_peers().await.unwrap();
        assert_eq!(peers.len(), 1);
        assert_eq!(peers[0].id, "new-peer");
    }
}
