use crate::device::types::{LightState, RGBColor};
use crate::error::{Error, Result};
use crate::types::DeviceCommand;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub struct LightTool;

impl LightTool {
    pub fn new() -> Self {
        Self
    }

    pub fn name() -> &'static str {
        "light_control"
    }

    pub fn description() -> &'static str {
        "Control smart lights: power, brightness, color temperature, RGB color, and scenes"
    }

    pub fn schema() -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "device_id": {
                    "type": "string",
                    "description": "Light device ID"
                },
                "action": {
                    "type": "string",
                    "enum": ["power_on", "power_off", "set_brightness", "set_color_temp",
                             "set_rgb", "set_scene", "dim", "brighten"],
                    "description": "Action to perform"
                },
                "parameters": {
                    "type": "object",
                    "properties": {
                        "brightness": { "type": "integer", "minimum": 0, "maximum": 100 },
                        "color_temp": { "type": "integer", "minimum": 2700, "maximum": 6500 },
                        "r": { "type": "integer", "minimum": 0, "maximum": 255 },
                        "g": { "type": "integer", "minimum": 0, "maximum": 255 },
                        "b": { "type": "integer", "minimum": 0, "maximum": 255 },
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
            "power_on" => self.power_on(&command.device_id).await,
            "power_off" => self.power_off(&command.device_id).await,
            "set_brightness" => {
                let brightness = params.get("brightness")
                    .and_then(|v| v.as_u64())
                    .ok_or_else(|| Error::CommandFailed("Missing brightness parameter".to_string()))? as u8;
                self.set_brightness(&command.device_id, brightness).await
            }
            "set_color_temp" => {
                let temp = params.get("color_temp")
                    .and_then(|v| v.as_u64())
                    .ok_or_else(|| Error::CommandFailed("Missing color_temp parameter".to_string()))? as u16;
                self.set_color_temp(&command.device_id, temp).await
            }
            "set_rgb" => {
                let r = params.get("r").and_then(|v| v.as_u64()).unwrap_or(255) as u8;
                let g = params.get("g").and_then(|v| v.as_u64()).unwrap_or(255) as u8;
                let b = params.get("b").and_then(|v| v.as_u64()).unwrap_or(255) as u8;
                self.set_rgb(&command.device_id, r, g, b).await
            }
            "set_scene" => {
                let scene = params.get("scene")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| Error::CommandFailed("Missing scene parameter".to_string()))?;
                self.set_scene(&command.device_id, scene).await
            }
            "dim" => self.dim(&command.device_id).await,
            "brighten" => self.brighten(&command.device_id).await,
            _ => Err(Error::CommandFailed(format!("Unknown light action: {}", action))),
        }
    }

    async fn power_on(&self, device_id: &str) -> Result<serde_json::Value> {
        tracing::info!("Light {} powered on", device_id);
        Ok(serde_json::json!({
            "device_id": device_id,
            "power": true,
            "status": "powered_on"
        }))
    }

    async fn power_off(&self, device_id: &str) -> Result<serde_json::Value> {
        tracing::info!("Light {} powered off", device_id);
        Ok(serde_json::json!({
            "device_id": device_id,
            "power": false,
            "status": "powered_off"
        }))
    }

    async fn set_brightness(&self, device_id: &str, brightness: u8) -> Result<serde_json::Value> {
        tracing::info!("Light {} brightness set to {}%", device_id, brightness);
        Ok(serde_json::json!({
            "device_id": device_id,
            "brightness": brightness,
            "power": brightness > 0,
            "status": "brightness_set"
        }))
    }

    async fn set_color_temp(&self, device_id: &str, temp: u16) -> Result<serde_json::Value> {
        if !(2700..=6500).contains(&temp) {
            return Err(Error::CommandFailed(
                format!("Color temperature {} out of range (2700-6500K)", temp)
            ));
        }

        tracing::info!("Light {} color temperature set to {}K", device_id, temp);
        Ok(serde_json::json!({
            "device_id": device_id,
            "color_temperature": temp,
            "status": "color_temp_set"
        }))
    }

    async fn set_rgb(&self, device_id: &str, r: u8, g: u8, b: u8) -> Result<serde_json::Value> {
        tracing::info!("Light {} RGB set to ({}, {}, {})", device_id, r, g, b);
        Ok(serde_json::json!({
            "device_id": device_id,
            "rgb": {
                "r": r,
                "g": g,
                "b": b
            },
            "status": "rgb_set"
        }))
    }

    async fn set_scene(&self, device_id: &str, scene: &str) -> Result<serde_json::Value> {
        let scene_settings = match scene.to_lowercase().as_str() {
            "reading" => serde_json::json!({
                "brightness": 100,
                "color_temp": 4000
            }),
            "movie" => serde_json::json!({
                "brightness": 30,
                "color_temp": 2700
            }),
            "night" => serde_json::json!({
                "brightness": 10,
                "color_temp": 2700
            }),
            "daylight" => serde_json::json!({
                "brightness": 100,
                "color_temp": 6500
            }),
            _ => serde_json::json!({})
        };

        tracing::info!("Light {} scene set to {}", device_id, scene);
        Ok(serde_json::json!({
            "device_id": device_id,
            "scene": scene,
            "settings": scene_settings,
            "status": "scene_set"
        }))
    }

    async fn dim(&self, device_id: &str) -> Result<serde_json::Value> {
        tracing::info!("Light {} dimmed", device_id);
        Ok(serde_json::json!({
            "device_id": device_id,
            "brightness_change": -20,
            "status": "dimmed"
        }))
    }

    async fn brighten(&self, device_id: &str) -> Result<serde_json::Value> {
        tracing::info!("Light {} brightened", device_id);
        Ok(serde_json::json!({
            "device_id": device_id,
            "brightness_change": 20,
            "status": "brightened"
        }))
    }
}

impl Default for LightTool {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tool_metadata() {
        assert_eq!(LightTool::name(), "light_control");
    }

    #[tokio::test]
    async fn power_on() {
        let tool = LightTool::new();
        let cmd = DeviceCommand {
            device_id: "light-1".to_string(),
            action: "power_on".to_string(),
            parameters: HashMap::new(),
            correlation_id: None,
        };

        let result = tool.execute(&cmd).await.unwrap();
        assert_eq!(result["power"], true);
    }

    #[tokio::test]
    async fn set_brightness() {
        let tool = LightTool::new();
        let mut params = HashMap::new();
        params.insert("brightness".to_string(), serde_json::json!(75));

        let cmd = DeviceCommand {
            device_id: "light-1".to_string(),
            action: "set_brightness".to_string(),
            parameters: params,
            correlation_id: None,
        };

        let result = tool.execute(&cmd).await.unwrap();
        assert_eq!(result["brightness"], 75);
    }

    #[tokio::test]
    async fn set_rgb() {
        let tool = LightTool::new();
        let mut params = HashMap::new();
        params.insert("r".to_string(), serde_json::json!(255));
        params.insert("g".to_string(), serde_json::json!(128));
        params.insert("b".to_string(), serde_json::json!(0));

        let cmd = DeviceCommand {
            device_id: "light-1".to_string(),
            action: "set_rgb".to_string(),
            parameters: params,
            correlation_id: None,
        };

        let result = tool.execute(&cmd).await.unwrap();
        assert_eq!(result["rgb"]["r"], 255);
    }

    #[tokio::test]
    async fn set_scene() {
        let tool = LightTool::new();
        let mut params = HashMap::new();
        params.insert("scene".to_string(), serde_json::json!("movie"));

        let cmd = DeviceCommand {
            device_id: "light-1".to_string(),
            action: "set_scene".to_string(),
            parameters: params,
            correlation_id: None,
        };

        let result = tool.execute(&cmd).await.unwrap();
        assert_eq!(result["scene"], "movie");
    }
}
