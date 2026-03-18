use crate::device::types::{ACMode, AirConditionerState, FanSpeed};
use crate::error::{Error, Result};
use crate::types::DeviceCommand;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub struct AirConditionerTool;

impl AirConditionerTool {
    pub fn new() -> Self {
        Self
    }

    pub fn name() -> &'static str {
        "air_conditioner_control"
    }

    pub fn description() -> &'static str {
        "Control air conditioners: power, temperature, mode, fan speed, and timer"
    }

    pub fn schema() -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "device_id": {
                    "type": "string",
                    "description": "Air conditioner device ID"
                },
                "action": {
                    "type": "string",
                    "enum": ["power_on", "power_off", "set_temperature", "set_mode",
                             "set_fan_speed", "set_timer", "enable_eco", "disable_eco"],
                    "description": "Action to perform"
                },
                "parameters": {
                    "type": "object",
                    "properties": {
                        "temperature": { "type": "number", "minimum": 16, "maximum": 30 },
                        "mode": { "type": "string", "enum": ["cool", "heat", "dehumidify", "fan", "auto"] },
                        "fan_speed": { "type": "string", "enum": ["low", "medium", "high", "auto"] },
                        "timer_minutes": { "type": "integer" }
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
            "set_temperature" => {
                let temp = params.get("temperature")
                    .and_then(|v| v.as_f64())
                    .ok_or_else(|| Error::CommandFailed("Missing temperature parameter".to_string()))?;
                self.set_temperature(&command.device_id, temp as f32).await
            }
            "set_mode" => {
                let mode = params.get("mode")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| Error::CommandFailed("Missing mode parameter".to_string()))?;
                self.set_mode(&command.device_id, mode).await
            }
            "set_fan_speed" => {
                let speed = params.get("fan_speed")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| Error::CommandFailed("Missing fan_speed parameter".to_string()))?;
                self.set_fan_speed(&command.device_id, speed).await
            }
            "set_timer" => {
                let minutes = params.get("timer_minutes")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0) as u32;
                self.set_timer(&command.device_id, minutes).await
            }
            "enable_eco" => self.enable_eco(&command.device_id).await,
            "disable_eco" => self.disable_eco(&command.device_id).await,
            _ => Err(Error::CommandFailed(format!("Unknown AC action: {}", action))),
        }
    }

    async fn power_on(&self, device_id: &str) -> Result<serde_json::Value> {
        tracing::info!("AC {} powered on", device_id);
        Ok(serde_json::json!({
            "device_id": device_id,
            "power": true,
            "status": "powered_on"
        }))
    }

    async fn power_off(&self, device_id: &str) -> Result<serde_json::Value> {
        tracing::info!("AC {} powered off", device_id);
        Ok(serde_json::json!({
            "device_id": device_id,
            "power": false,
            "status": "powered_off"
        }))
    }

    async fn set_temperature(&self, device_id: &str, temperature: f32) -> Result<serde_json::Value> {
        if !(16.0..=30.0).contains(&temperature) {
            return Err(Error::CommandFailed(
                format!("Temperature {} out of range (16-30)", temperature)
            ));
        }

        tracing::info!("AC {} temperature set to {}°C", device_id, temperature);
        Ok(serde_json::json!({
            "device_id": device_id,
            "temperature": temperature,
            "status": "temperature_set"
        }))
    }

    async fn set_mode(&self, device_id: &str, mode: &str) -> Result<serde_json::Value> {
        let ac_mode = match mode.to_lowercase().as_str() {
            "cool" => ACMode::Cool,
            "heat" => ACMode::Heat,
            "dehumidify" => ACMode::Dehumidify,
            "fan" => ACMode::Fan,
            "auto" => ACMode::Auto,
            _ => return Err(Error::CommandFailed(format!("Invalid mode: {}", mode))),
        };

        tracing::info!("AC {} mode set to {:?}", device_id, ac_mode);
        Ok(serde_json::json!({
            "device_id": device_id,
            "mode": mode,
            "status": "mode_set"
        }))
    }

    async fn set_fan_speed(&self, device_id: &str, speed: &str) -> Result<serde_json::Value> {
        let fan_speed = match speed.to_lowercase().as_str() {
            "low" => FanSpeed::Low,
            "medium" => FanSpeed::Medium,
            "high" => FanSpeed::High,
            "auto" => FanSpeed::Auto,
            _ => return Err(Error::CommandFailed(format!("Invalid fan speed: {}", speed))),
        };

        tracing::info!("AC {} fan speed set to {:?}", device_id, fan_speed);
        Ok(serde_json::json!({
            "device_id": device_id,
            "fan_speed": speed,
            "status": "fan_speed_set"
        }))
    }

    async fn set_timer(&self, device_id: &str, minutes: u32) -> Result<serde_json::Value> {
        tracing::info!("AC {} timer set to {} minutes", device_id, minutes);
        Ok(serde_json::json!({
            "device_id": device_id,
            "timer_minutes": minutes,
            "timer_enabled": minutes > 0,
            "status": "timer_set"
        }))
    }

    async fn enable_eco(&self, device_id: &str) -> Result<serde_json::Value> {
        tracing::info!("AC {} eco mode enabled", device_id);
        Ok(serde_json::json!({
            "device_id": device_id,
            "eco_mode": true,
            "status": "eco_enabled"
        }))
    }

    async fn disable_eco(&self, device_id: &str) -> Result<serde_json::Value> {
        tracing::info!("AC {} eco mode disabled", device_id);
        Ok(serde_json::json!({
            "device_id": device_id,
            "eco_mode": false,
            "status": "eco_disabled"
        }))
    }
}

impl Default for AirConditionerTool {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tool_metadata() {
        assert_eq!(AirConditionerTool::name(), "air_conditioner_control");
    }

    #[tokio::test]
    async fn power_on() {
        let tool = AirConditionerTool::new();
        let cmd = DeviceCommand {
            device_id: "ac-1".to_string(),
            action: "power_on".to_string(),
            parameters: HashMap::new(),
            correlation_id: None,
        };

        let result = tool.execute(&cmd).await.unwrap();
        assert_eq!(result["power"], true);
    }

    #[tokio::test]
    async fn set_temperature() {
        let tool = AirConditionerTool::new();
        let mut params = HashMap::new();
        params.insert("temperature".to_string(), serde_json::json!(24.5));

        let cmd = DeviceCommand {
            device_id: "ac-1".to_string(),
            action: "set_temperature".to_string(),
            parameters: params,
            correlation_id: None,
        };

        let result = tool.execute(&cmd).await.unwrap();
        assert_eq!(result["temperature"], 24.5);
    }

    #[tokio::test]
    async fn set_temperature_out_of_range() {
        let tool = AirConditionerTool::new();
        let mut params = HashMap::new();
        params.insert("temperature".to_string(), serde_json::json!(35.0));

        let cmd = DeviceCommand {
            device_id: "ac-1".to_string(),
            action: "set_temperature".to_string(),
            parameters: params,
            correlation_id: None,
        };

        assert!(tool.execute(&cmd).await.is_err());
    }

    #[tokio::test]
    async fn set_mode() {
        let tool = AirConditionerTool::new();
        let mut params = HashMap::new();
        params.insert("mode".to_string(), serde_json::json!("cool"));

        let cmd = DeviceCommand {
            device_id: "ac-1".to_string(),
            action: "set_mode".to_string(),
            parameters: params,
            correlation_id: None,
        };

        let result = tool.execute(&cmd).await.unwrap();
        assert_eq!(result["mode"], "cool");
    }
}
