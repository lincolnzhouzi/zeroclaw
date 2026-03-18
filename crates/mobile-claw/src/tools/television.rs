use crate::error::{Error, Result};
use crate::types::DeviceCommand;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub struct TelevisionTool;

impl TelevisionTool {
    pub fn new() -> Self {
        Self
    }

    pub fn name() -> &'static str {
        "television_control"
    }

    pub fn description() -> &'static str {
        "Control smart TVs: power, channel, volume, input source, and playback"
    }

    pub fn schema() -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "device_id": {
                    "type": "string",
                    "description": "TV device ID"
                },
                "action": {
                    "type": "string",
                    "enum": ["power_on", "power_off", "set_channel", "channel_up",
                             "channel_down", "set_volume", "mute", "unmute", "toggle_mute",
                             "set_input", "play", "pause", "stop"],
                    "description": "Action to perform"
                },
                "parameters": {
                    "type": "object",
                    "properties": {
                        "channel": { "type": "integer" },
                        "volume": { "type": "integer", "minimum": 0, "maximum": 100 },
                        "input_source": { "type": "string" }
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
            "power_on" => self.power_on(&command.device_id).await,
            "power_off" => self.power_off(&command.device_id).await,
            "set_channel" => {
                let channel = params.get("channel")
                    .and_then(|v| v.as_u64())
                    .ok_or_else(|| Error::CommandFailed("Missing channel parameter".to_string()))? as u32;
                self.set_channel(&command.device_id, channel).await
            }
            "channel_up" => self.channel_up(&command.device_id).await,
            "channel_down" => self.channel_down(&command.device_id).await,
            "set_volume" => {
                let volume = params.get("volume")
                    .and_then(|v| v.as_u64())
                    .ok_or_else(|| Error::CommandFailed("Missing volume parameter".to_string()))? as u8;
                self.set_volume(&command.device_id, volume).await
            }
            "mute" => self.mute(&command.device_id).await,
            "unmute" => self.unmute(&command.device_id).await,
            "toggle_mute" => self.toggle_mute(&command.device_id).await,
            "set_input" => {
                let input = params.get("input_source")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| Error::CommandFailed("Missing input_source parameter".to_string()))?;
                self.set_input(&command.device_id, input).await
            }
            "play" => self.play(&command.device_id).await,
            "pause" => self.pause(&command.device_id).await,
            "stop" => self.stop(&command.device_id).await,
            _ => Err(Error::CommandFailed(format!("Unknown TV action: {}", action))),
        }
    }

    async fn power_on(&self, device_id: &str) -> Result<serde_json::Value> {
        tracing::info!("TV {} powered on", device_id);
        Ok(serde_json::json!({
            "device_id": device_id,
            "power": true,
            "status": "powered_on"
        }))
    }

    async fn power_off(&self, device_id: &str) -> Result<serde_json::Value> {
        tracing::info!("TV {} powered off", device_id);
        Ok(serde_json::json!({
            "device_id": device_id,
            "power": false,
            "status": "powered_off"
        }))
    }

    async fn set_channel(&self, device_id: &str, channel: u32) -> Result<serde_json::Value> {
        tracing::info!("TV {} channel set to {}", device_id, channel);
        Ok(serde_json::json!({
            "device_id": device_id,
            "channel": channel,
            "status": "channel_set"
        }))
    }

    async fn channel_up(&self, device_id: &str) -> Result<serde_json::Value> {
        tracing::info!("TV {} channel up", device_id);
        Ok(serde_json::json!({
            "device_id": device_id,
            "status": "channel_up"
        }))
    }

    async fn channel_down(&self, device_id: &str) -> Result<serde_json::Value> {
        tracing::info!("TV {} channel down", device_id);
        Ok(serde_json::json!({
            "device_id": device_id,
            "status": "channel_down"
        }))
    }

    async fn set_volume(&self, device_id: &str, volume: u8) -> Result<serde_json::Value> {
        tracing::info!("TV {} volume set to {}", device_id, volume);
        Ok(serde_json::json!({
            "device_id": device_id,
            "volume": volume,
            "muted": false,
            "status": "volume_set"
        }))
    }

    async fn mute(&self, device_id: &str) -> Result<serde_json::Value> {
        tracing::info!("TV {} muted", device_id);
        Ok(serde_json::json!({
            "device_id": device_id,
            "muted": true,
            "status": "muted"
        }))
    }

    async fn unmute(&self, device_id: &str) -> Result<serde_json::Value> {
        tracing::info!("TV {} unmuted", device_id);
        Ok(serde_json::json!({
            "device_id": device_id,
            "muted": false,
            "status": "unmuted"
        }))
    }

    async fn toggle_mute(&self, device_id: &str) -> Result<serde_json::Value> {
        tracing::info!("TV {} mute toggled", device_id);
        Ok(serde_json::json!({
            "device_id": device_id,
            "status": "mute_toggled"
        }))
    }

    async fn set_input(&self, device_id: &str, input: &str) -> Result<serde_json::Value> {
        tracing::info!("TV {} input set to {}", device_id, input);
        Ok(serde_json::json!({
            "device_id": device_id,
            "input_source": input,
            "status": "input_set"
        }))
    }

    async fn play(&self, device_id: &str) -> Result<serde_json::Value> {
        tracing::info!("TV {} playing", device_id);
        Ok(serde_json::json!({
            "device_id": device_id,
            "playing": true,
            "status": "playing"
        }))
    }

    async fn pause(&self, device_id: &str) -> Result<serde_json::Value> {
        tracing::info!("TV {} paused", device_id);
        Ok(serde_json::json!({
            "device_id": device_id,
            "playing": false,
            "status": "paused"
        }))
    }

    async fn stop(&self, device_id: &str) -> Result<serde_json::Value> {
        tracing::info!("TV {} stopped", device_id);
        Ok(serde_json::json!({
            "device_id": device_id,
            "playing": false,
            "status": "stopped"
        }))
    }
}

impl Default for TelevisionTool {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tool_metadata() {
        assert_eq!(TelevisionTool::name(), "television_control");
    }

    #[tokio::test]
    async fn power_on() {
        let tool = TelevisionTool::new();
        let cmd = DeviceCommand {
            device_id: "tv-1".to_string(),
            action: "power_on".to_string(),
            parameters: HashMap::new(),
            correlation_id: None,
        };

        let result = tool.execute(&cmd).await.unwrap();
        assert_eq!(result["power"], true);
    }

    #[tokio::test]
    async fn set_volume() {
        let tool = TelevisionTool::new();
        let mut params = HashMap::new();
        params.insert("volume".to_string(), serde_json::json!(50));

        let cmd = DeviceCommand {
            device_id: "tv-1".to_string(),
            action: "set_volume".to_string(),
            parameters: params,
            correlation_id: None,
        };

        let result = tool.execute(&cmd).await.unwrap();
        assert_eq!(result["volume"], 50);
    }

    #[tokio::test]
    async fn set_channel() {
        let tool = TelevisionTool::new();
        let mut params = HashMap::new();
        params.insert("channel".to_string(), serde_json::json!(5));

        let cmd = DeviceCommand {
            device_id: "tv-1".to_string(),
            action: "set_channel".to_string(),
            parameters: params,
            correlation_id: None,
        };

        let result = tool.execute(&cmd).await.unwrap();
        assert_eq!(result["channel"], 5);
    }
}
