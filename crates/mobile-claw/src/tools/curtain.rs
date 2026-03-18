use crate::error::{Error, Result};
use crate::types::DeviceCommand;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub struct CurtainTool;

impl CurtainTool {
    pub fn new() -> Self {
        Self
    }

    pub fn name() -> &'static str {
        "curtain_control"
    }

    pub fn description() -> &'static str {
        "Control smart curtains: open, close, position, and scenes"
    }

    pub fn schema() -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "device_id": {
                    "type": "string",
                    "description": "Curtain device ID"
                },
                "action": {
                    "type": "string",
                    "enum": ["open", "close", "set_position", "stop", "set_scene"],
                    "description": "Action to perform"
                },
                "parameters": {
                    "type": "object",
                    "properties": {
                        "position": { "type": "integer", "minimum": 0, "maximum": 100 },
                        "scene": { "type": "string" }
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
            "open" => self.open(&command.device_id).await,
            "close" => self.close(&command.device_id).await,
            "set_position" => {
                let position = params.get("position")
                    .and_then(|v| v.as_u64())
                    .ok_or_else(|| Error::CommandFailed("Missing position parameter".to_string()))? as u8;
                self.set_position(&command.device_id, position).await
            }
            "stop" => self.stop(&command.device_id).await,
            "set_scene" => {
                let scene = params.get("scene")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| Error::CommandFailed("Missing scene parameter".to_string()))?;
                self.set_scene(&command.device_id, scene).await
            }
            _ => Err(Error::CommandFailed(format!("Unknown curtain action: {}", action))),
        }
    }

    async fn open(&self, device_id: &str) -> Result<serde_json::Value> {
        tracing::info!("Curtain {} opened", device_id);
        Ok(serde_json::json!({
            "device_id": device_id,
            "position": 100,
            "moving": false,
            "status": "opened"
        }))
    }

    async fn close(&self, device_id: &str) -> Result<serde_json::Value> {
        tracing::info!("Curtain {} closed", device_id);
        Ok(serde_json::json!({
            "device_id": device_id,
            "position": 0,
            "moving": false,
            "status": "closed"
        }))
    }

    async fn set_position(&self, device_id: &str, position: u8) -> Result<serde_json::Value> {
        tracing::info!("Curtain {} position set to {}%", device_id, position);
        Ok(serde_json::json!({
            "device_id": device_id,
            "position": position,
            "moving": false,
            "status": "position_set"
        }))
    }

    async fn stop(&self, device_id: &str) -> Result<serde_json::Value> {
        tracing::info!("Curtain {} stopped", device_id);
        Ok(serde_json::json!({
            "device_id": device_id,
            "moving": false,
            "status": "stopped"
        }))
    }

    async fn set_scene(&self, device_id: &str, scene: &str) -> Result<serde_json::Value> {
        let position = match scene.to_lowercase().as_str() {
            "morning" => 100,
            "day" => 50,
            "evening" => 30,
            "night" => 0,
            "privacy" => 100,
            _ => return Err(Error::CommandFailed(format!("Unknown scene: {}", scene))),
        };

        tracing::info!("Curtain {} scene set to {} (position: {}%)", device_id, scene, position);
        Ok(serde_json::json!({
            "device_id": device_id,
            "scene": scene,
            "position": position,
            "status": "scene_set"
        }))
    }
}

impl Default for CurtainTool {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tool_metadata() {
        assert_eq!(CurtainTool::name(), "curtain_control");
    }

    #[tokio::test]
    async fn open() {
        let tool = CurtainTool::new();
        let cmd = DeviceCommand {
            device_id: "curtain-1".to_string(),
            action: "open".to_string(),
            parameters: HashMap::new(),
            correlation_id: None,
        };

        let result = tool.execute(&cmd).await.unwrap();
        assert_eq!(result["position"], 100);
    }

    #[tokio::test]
    async fn close() {
        let tool = CurtainTool::new();
        let cmd = DeviceCommand {
            device_id: "curtain-1".to_string(),
            action: "close".to_string(),
            parameters: HashMap::new(),
            correlation_id: None,
        };

        let result = tool.execute(&cmd).await.unwrap();
        assert_eq!(result["position"], 0);
    }

    #[tokio::test]
    async fn set_position() {
        let tool = CurtainTool::new();
        let mut params = HashMap::new();
        params.insert("position".to_string(), serde_json::json!(75));

        let cmd = DeviceCommand {
            device_id: "curtain-1".to_string(),
            action: "set_position".to_string(),
            parameters: params,
            correlation_id: None,
        };

        let result = tool.execute(&cmd).await.unwrap();
        assert_eq!(result["position"], 75);
    }

    #[tokio::test]
    async fn set_scene() {
        let tool = CurtainTool::new();
        let mut params = HashMap::new();
        params.insert("scene".to_string(), serde_json::json!("morning"));

        let cmd = DeviceCommand {
            device_id: "curtain-1".to_string(),
            action: "set_scene".to_string(),
            parameters: params,
            correlation_id: None,
        };

        let result = tool.execute(&cmd).await.unwrap();
        assert_eq!(result["scene"], "morning");
        assert_eq!(result["position"], 100);
    }
}
