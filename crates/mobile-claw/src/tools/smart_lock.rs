use crate::device::types::TemporaryCode;
use crate::error::{Error, Result};
use crate::types::DeviceCommand;
use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub struct SmartLockTool;

impl SmartLockTool {
    pub fn new() -> Self {
        Self
    }

    pub fn name() -> &'static str {
        "smart_lock_control"
    }

    pub fn description() -> &'static str {
        "Control smart locks: lock, unlock, temporary codes, and access logs"
    }

    pub fn schema() -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "device_id": {
                    "type": "string",
                    "description": "Lock device ID"
                },
                "action": {
                    "type": "string",
                    "enum": ["lock", "unlock", "create_temp_code", "delete_temp_code",
                             "list_temp_codes", "get_status", "get_access_log"],
                    "description": "Action to perform"
                },
                "parameters": {
                    "type": "object",
                    "properties": {
                        "code": { "type": "string" },
                        "expires_hours": { "type": "integer" },
                        "max_uses": { "type": "integer" },
                        "limit": { "type": "integer" }
                    }
                }
            },
            "required": ["device_id", "action"]
        })
    }

    pub async fn execute(&self, command: &DeviceCommand) -> Result<serde_json::Value> {
        let action = command.action.as_str();
        let params = &command.parameters;

        match action {
            "lock" => self.lock(&command.device_id).await,
            "unlock" => self.unlock(&command.device_id).await,
            "create_temp_code" => {
                let generated_code = Self::generate_code();
                let code = params.get("code")
                    .and_then(|v| v.as_str())
                    .unwrap_or(&generated_code);
                let expires_hours = params.get("expires_hours")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(24) as i64;
                let max_uses = params.get("max_uses")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(1) as u32;
                self.create_temp_code(&command.device_id, code, expires_hours, max_uses).await
            }
            "delete_temp_code" => {
                let code = params.get("code")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| Error::CommandFailed("Missing code parameter".to_string()))?;
                self.delete_temp_code(&command.device_id, code).await
            }
            "list_temp_codes" => self.list_temp_codes(&command.device_id).await,
            "get_status" => self.get_status(&command.device_id).await,
            "get_access_log" => {
                let limit = params.get("limit")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(10) as usize;
                self.get_access_log(&command.device_id, limit).await
            }
            _ => Err(Error::CommandFailed(format!("Unknown lock action: {}", action))),
        }
    }

    fn generate_code() -> String {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        format!("{:06}", rng.gen_range(100000..999999))
    }

    async fn lock(&self, device_id: &str) -> Result<serde_json::Value> {
        tracing::info!("Lock {} locked", device_id);
        Ok(serde_json::json!({
            "device_id": device_id,
            "locked": true,
            "status": "locked"
        }))
    }

    async fn unlock(&self, device_id: &str) -> Result<serde_json::Value> {
        tracing::info!("Lock {} unlocked", device_id);
        Ok(serde_json::json!({
            "device_id": device_id,
            "locked": false,
            "unlocked_at": Utc::now().to_rfc3339(),
            "status": "unlocked"
        }))
    }

    async fn create_temp_code(
        &self,
        device_id: &str,
        code: &str,
        expires_hours: i64,
        max_uses: u32,
    ) -> Result<serde_json::Value> {
        let expires_at = Utc::now() + Duration::hours(expires_hours);

        tracing::info!("Lock {} temp code created: {}", device_id, code);
        Ok(serde_json::json!({
            "device_id": device_id,
            "temp_code": {
                "code": code,
                "expires_at": expires_at.to_rfc3339(),
                "max_uses": max_uses
            },
            "status": "temp_code_created"
        }))
    }

    async fn delete_temp_code(&self, device_id: &str, code: &str) -> Result<serde_json::Value> {
        tracing::info!("Lock {} temp code deleted: {}", device_id, code);
        Ok(serde_json::json!({
            "device_id": device_id,
            "deleted_code": code,
            "status": "temp_code_deleted"
        }))
    }

    async fn list_temp_codes(&self, device_id: &str) -> Result<serde_json::Value> {
        Ok(serde_json::json!({
            "device_id": device_id,
            "temp_codes": [],
            "status": "success"
        }))
    }

    async fn get_status(&self, device_id: &str) -> Result<serde_json::Value> {
        Ok(serde_json::json!({
            "device_id": device_id,
            "locked": true,
            "battery_level": 85,
            "last_unlock": null,
            "status": "success"
        }))
    }

    async fn get_access_log(&self, device_id: &str, limit: usize) -> Result<serde_json::Value> {
        Ok(serde_json::json!({
            "device_id": device_id,
            "access_log": [],
            "limit": limit,
            "status": "success"
        }))
    }
}

impl Default for SmartLockTool {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tool_metadata() {
        assert_eq!(SmartLockTool::name(), "smart_lock_control");
    }

    #[tokio::test]
    async fn lock() {
        let tool = SmartLockTool::new();
        let cmd = DeviceCommand {
            device_id: "lock-1".to_string(),
            action: "lock".to_string(),
            parameters: HashMap::new(),
            correlation_id: None,
        };

        let result = tool.execute(&cmd).await.unwrap();
        assert_eq!(result["locked"], true);
    }

    #[tokio::test]
    async fn unlock() {
        let tool = SmartLockTool::new();
        let cmd = DeviceCommand {
            device_id: "lock-1".to_string(),
            action: "unlock".to_string(),
            parameters: HashMap::new(),
            correlation_id: None,
        };

        let result = tool.execute(&cmd).await.unwrap();
        assert_eq!(result["locked"], false);
    }

    #[tokio::test]
    async fn create_temp_code() {
        let tool = SmartLockTool::new();
        let mut params = HashMap::new();
        params.insert("code".to_string(), serde_json::json!("123456"));
        params.insert("expires_hours".to_string(), serde_json::json!(48));
        params.insert("max_uses".to_string(), serde_json::json!(3));

        let cmd = DeviceCommand {
            device_id: "lock-1".to_string(),
            action: "create_temp_code".to_string(),
            parameters: params,
            correlation_id: None,
        };

        let result = tool.execute(&cmd).await.unwrap();
        assert_eq!(result["temp_code"]["code"], "123456");
    }

    #[tokio::test]
    async fn get_status() {
        let tool = SmartLockTool::new();
        let cmd = DeviceCommand {
            device_id: "lock-1".to_string(),
            action: "get_status".to_string(),
            parameters: HashMap::new(),
            correlation_id: None,
        };

        let result = tool.execute(&cmd).await.unwrap();
        assert_eq!(result["locked"], true);
    }
}
