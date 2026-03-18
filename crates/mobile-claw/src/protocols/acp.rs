use crate::error::{Error, Result};
use crate::types::{CommandResult, DeviceCommand, DeviceId};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ACPCommand {
    pub id: String,
    pub device_id: DeviceId,
    pub action: String,
    pub parameters: HashMap<String, serde_json::Value>,
    pub timestamp: DateTime<Utc>,
    pub timeout: Duration,
    pub priority: CommandPriority,
    pub retry_count: u32,
}

impl From<DeviceCommand> for ACPCommand {
    fn from(cmd: DeviceCommand) -> Self {
        Self {
            id: cmd.correlation_id.unwrap_or_else(|| uuid::Uuid::new_v4().to_string()),
            device_id: cmd.device_id,
            action: cmd.action,
            parameters: cmd.parameters,
            timestamp: Utc::now(),
            timeout: Duration::from_secs(30),
            priority: CommandPriority::Normal,
            retry_count: 0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ACPResponse {
    pub command_id: String,
    pub status: CommandStatus,
    pub result: Option<serde_json::Value>,
    pub error: Option<ACPError>,
    pub timestamp: DateTime<Utc>,
    pub execution_time: Duration,
}

impl From<ACPResponse> for CommandResult {
    fn from(resp: ACPResponse) -> Self {
        Self {
            device_id: String::new(),
            success: resp.status == CommandStatus::Completed,
            result: resp.result,
            error: resp.error.map(|e| e.message),
            execution_time_ms: resp.execution_time.as_millis() as u64,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum CommandStatus {
    Pending,
    Queued,
    Executing,
    Completed,
    Failed,
    Timeout,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum CommandPriority {
    Low,
    Normal,
    High,
    Critical,
}

impl Default for CommandPriority {
    fn default() -> Self {
        Self::Normal
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ACPError {
    pub code: String,
    pub message: String,
    pub details: Option<serde_json::Value>,
}

impl ACPError {
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            details: None,
        }
    }

    pub fn timeout(timeout: Duration) -> Self {
        Self {
            code: "TIMEOUT".to_string(),
            message: format!("Command timed out after {:?}", timeout),
            details: None,
        }
    }

    pub fn device_not_found(device_id: &str) -> Self {
        Self {
            code: "DEVICE_NOT_FOUND".to_string(),
            message: format!("Device not found: {}", device_id),
            details: None,
        }
    }

    pub fn execution_failed(reason: impl Into<String>) -> Self {
        Self {
            code: "EXECUTION_FAILED".to_string(),
            message: reason.into(),
            details: None,
        }
    }
}

struct CommandQueueEntry {
    command: ACPCommand,
    created_at: DateTime<Utc>,
}

pub struct ACPProtocol {
    command_queue: Arc<RwLock<Vec<CommandQueueEntry>>>,
    response_cache: Arc<RwLock<HashMap<String, ACPResponse>>>,
    max_queue_size: usize,
    default_timeout: Duration,
    max_retries: u32,
}

impl ACPProtocol {
    pub fn new() -> Self {
        Self {
            command_queue: Arc::new(RwLock::new(Vec::new())),
            response_cache: Arc::new(RwLock::new(HashMap::new())),
            max_queue_size: 100,
            default_timeout: Duration::from_secs(30),
            max_retries: 3,
        }
    }

    pub fn with_config(max_queue_size: usize, default_timeout: Duration, max_retries: u32) -> Self {
        Self {
            command_queue: Arc::new(RwLock::new(Vec::new())),
            response_cache: Arc::new(RwLock::new(HashMap::new())),
            max_queue_size,
            default_timeout,
            max_retries,
        }
    }

    pub async fn enqueue_command(&self, mut command: ACPCommand) -> Result<()> {
        let mut queue = self.command_queue.write().await;

        if queue.len() >= self.max_queue_size {
            queue.sort_by(|a, b| {
                let priority_order = |p: &CommandPriority| match p {
                    CommandPriority::Critical => 0,
                    CommandPriority::High => 1,
                    CommandPriority::Normal => 2,
                    CommandPriority::Low => 3,
                };
                priority_order(&a.command.priority).cmp(&priority_order(&b.command.priority))
            });

            if let Some(lowest) = queue.last() {
                if lowest.command.priority == CommandPriority::Low {
                    queue.pop();
                }
            }
        }

        if command.timeout.is_zero() {
            command.timeout = self.default_timeout;
        }

        queue.push(CommandQueueEntry {
            command,
            created_at: Utc::now(),
        });

        Ok(())
    }

    pub async fn execute_command(&self, command: DeviceCommand) -> Result<CommandResult> {
        let acp_command: ACPCommand = command.into();
        self.execute_acp_command(acp_command).await
    }

    async fn execute_acp_command(&self, command: ACPCommand) -> Result<CommandResult> {
        self.enqueue_command(command.clone()).await?;

        let start = std::time::Instant::now();

        let response = self.process_command(&command).await;

        let execution_time = start.elapsed();

        let acp_response = match response {
            Ok(result) => ACPResponse {
                command_id: command.id.clone(),
                status: CommandStatus::Completed,
                result: Some(result),
                error: None,
                timestamp: Utc::now(),
                execution_time,
            },
            Err(e) => ACPResponse {
                command_id: command.id.clone(),
                status: CommandStatus::Failed,
                result: None,
                error: Some(ACPError::execution_failed(e.to_string())),
                timestamp: Utc::now(),
                execution_time,
            },
        };

        {
            let mut cache = self.response_cache.write().await;
            cache.insert(command.id.clone(), acp_response.clone());
        }

        {
            let mut queue = self.command_queue.write().await;
            queue.retain(|entry| entry.command.id != command.id);
        }

        Ok(acp_response.into())
    }

    async fn process_command(&self, command: &ACPCommand) -> Result<serde_json::Value> {
        match command.action.as_str() {
            "power_on" => Ok(serde_json::json!({"status": "powered_on"})),
            "power_off" => Ok(serde_json::json!({"status": "powered_off"})),
            "set_temperature" => {
                let temp = command.parameters.get("temperature")
                    .and_then(|v| v.as_f64())
                    .ok_or_else(|| Error::CommandFailed("Missing temperature parameter".to_string()))?;
                Ok(serde_json::json!({"temperature": temp, "status": "set"}))
            }
            "set_brightness" => {
                let brightness = command.parameters.get("brightness")
                    .and_then(|v| v.as_u64())
                    .ok_or_else(|| Error::CommandFailed("Missing brightness parameter".to_string()))?;
                Ok(serde_json::json!({"brightness": brightness, "status": "set"}))
            }
            "set_position" => {
                let position = command.parameters.get("position")
                    .and_then(|v| v.as_u64())
                    .ok_or_else(|| Error::CommandFailed("Missing position parameter".to_string()))?;
                Ok(serde_json::json!({"position": position, "status": "set"}))
            }
            "get_status" => Ok(serde_json::json!({"online": true})),
            _ => Err(Error::CommandFailed(format!("Unknown action: {}", command.action))),
        }
    }

    pub async fn get_command_status(&self, command_id: &str) -> Option<ACPResponse> {
        let cache = self.response_cache.write().await;
        cache.get(command_id).cloned()
    }

    pub async fn cancel_command(&self, command_id: &str) -> Result<bool> {
        let mut queue = self.command_queue.write().await;
        let initial_len = queue.len();
        queue.retain(|entry| entry.command.id != command_id);
        Ok(queue.len() < initial_len)
    }

    pub async fn get_pending_commands(&self) -> Vec<ACPCommand> {
        let queue = self.command_queue.read().await;
        queue.iter().map(|entry| entry.command.clone()).collect()
    }

    pub async fn clear_completed(&self) {
        let mut cache = self.response_cache.write().await;
        cache.clear();
    }

    pub async fn queue_size(&self) -> usize {
        self.command_queue.read().await.len()
    }
}

impl Default for ACPProtocol {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn protocol_initialization() {
        let protocol = ACPProtocol::new();
        assert_eq!(protocol.queue_size().await, 0);
    }

    #[tokio::test]
    async fn enqueue_command() {
        let protocol = ACPProtocol::new();
        let command = ACPCommand {
            id: "cmd-1".to_string(),
            device_id: "device-1".to_string(),
            action: "power_on".to_string(),
            parameters: HashMap::new(),
            timestamp: Utc::now(),
            timeout: Duration::from_secs(30),
            priority: CommandPriority::Normal,
            retry_count: 0,
        };

        protocol.enqueue_command(command).await.unwrap();
        assert_eq!(protocol.queue_size().await, 1);
    }

    #[tokio::test]
    async fn execute_power_command() {
        let protocol = ACPProtocol::new();
        let command = DeviceCommand {
            device_id: "device-1".to_string(),
            action: "power_on".to_string(),
            parameters: HashMap::new(),
            correlation_id: Some("cmd-1".to_string()),
        };

        let result = protocol.execute_command(command).await.unwrap();
        assert!(result.success);
    }

    #[tokio::test]
    async fn execute_temperature_command() {
        let protocol = ACPProtocol::new();
        let mut params = HashMap::new();
        params.insert("temperature".to_string(), serde_json::json!(24.5));

        let command = DeviceCommand {
            device_id: "ac-1".to_string(),
            action: "set_temperature".to_string(),
            parameters: params,
            correlation_id: Some("cmd-2".to_string()),
        };

        let result = protocol.execute_command(command).await.unwrap();
        assert!(result.success);
        assert!(result.result.is_some());
    }

    #[tokio::test]
    async fn unknown_action_fails() {
        let protocol = ACPProtocol::new();
        let command = DeviceCommand {
            device_id: "device-1".to_string(),
            action: "unknown_action".to_string(),
            parameters: HashMap::new(),
            correlation_id: Some("cmd-3".to_string()),
        };

        let result = protocol.execute_command(command).await.unwrap();
        assert!(!result.success);
        assert!(result.error.is_some());
    }

    #[tokio::test]
    async fn cancel_command() {
        let protocol = ACPProtocol::new();
        let command = ACPCommand {
            id: "cmd-cancel".to_string(),
            device_id: "device-1".to_string(),
            action: "power_on".to_string(),
            parameters: HashMap::new(),
            timestamp: Utc::now(),
            timeout: Duration::from_secs(30),
            priority: CommandPriority::Low,
            retry_count: 0,
        };

        protocol.enqueue_command(command).await.unwrap();
        let cancelled = protocol.cancel_command("cmd-cancel").await.unwrap();
        assert!(cancelled);
        assert_eq!(protocol.queue_size().await, 0);
    }

    #[tokio::test]
    async fn priority_queue_ordering() {
        let protocol = ACPProtocol::new();

        let low_cmd = ACPCommand {
            id: "low".to_string(),
            device_id: "d".to_string(),
            action: "test".to_string(),
            parameters: HashMap::new(),
            timestamp: Utc::now(),
            timeout: Duration::from_secs(30),
            priority: CommandPriority::Low,
            retry_count: 0,
        };

        let critical_cmd = ACPCommand {
            id: "critical".to_string(),
            device_id: "d".to_string(),
            action: "test".to_string(),
            parameters: HashMap::new(),
            timestamp: Utc::now(),
            timeout: Duration::from_secs(30),
            priority: CommandPriority::Critical,
            retry_count: 0,
        };

        protocol.enqueue_command(low_cmd).await.unwrap();
        protocol.enqueue_command(critical_cmd).await.unwrap();

        let pending = protocol.get_pending_commands().await;
        assert_eq!(pending.len(), 2);
    }
}
