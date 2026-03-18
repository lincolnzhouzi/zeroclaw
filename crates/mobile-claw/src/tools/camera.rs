use crate::device::types::CameraState;
use crate::error::{Error, Result};
use crate::types::DeviceCommand;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CameraControlRequest {
    pub device_id: String,
    pub action: CameraAction,
    pub parameters: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CameraAction {
    PowerOn,
    PowerOff,
    StartRecording,
    StopRecording,
    SetResolution,
    PTZMove,
    PTZReset,
    EnableMotionDetection,
    DisableMotionDetection,
    Snapshot,
}

pub struct CameraTool;

impl CameraTool {
    pub fn new() -> Self {
        Self
    }

    pub fn name() -> &'static str {
        "camera_control"
    }

    pub fn description() -> &'static str {
        "Control smart cameras: power, recording, PTZ, motion detection, and snapshots"
    }

    pub fn schema() -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "device_id": {
                    "type": "string",
                    "description": "Camera device ID"
                },
                "action": {
                    "type": "string",
                    "enum": ["power_on", "power_off", "start_recording", "stop_recording",
                             "set_resolution", "ptz_move", "ptz_reset", "enable_motion",
                             "disable_motion", "snapshot"],
                    "description": "Action to perform"
                },
                "parameters": {
                    "type": "object",
                    "properties": {
                        "resolution": { "type": "string" },
                        "pan": { "type": "number" },
                        "tilt": { "type": "number" },
                        "zoom": { "type": "number" }
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
            "start_recording" => self.start_recording(&command.device_id).await,
            "stop_recording" => self.stop_recording(&command.device_id).await,
            "set_resolution" => {
                let width = params.get("width").and_then(|v| v.as_u64()).unwrap_or(1920) as u32;
                let height = params.get("height").and_then(|v| v.as_u64()).unwrap_or(1080) as u32;
                self.set_resolution(&command.device_id, width, height).await
            }
            "ptz_move" => {
                let pan = params.get("pan").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;
                let tilt = params.get("tilt").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;
                let zoom = params.get("zoom").and_then(|v| v.as_f64()).unwrap_or(1.0) as f32;
                self.ptz_move(&command.device_id, pan, tilt, zoom).await
            }
            "ptz_reset" => self.ptz_reset(&command.device_id).await,
            "enable_motion" => self.enable_motion_detection(&command.device_id).await,
            "disable_motion" => self.disable_motion_detection(&command.device_id).await,
            "snapshot" => self.snapshot(&command.device_id).await,
            _ => Err(Error::CommandFailed(format!("Unknown camera action: {}", action))),
        }
    }

    async fn power_on(&self, device_id: &str) -> Result<serde_json::Value> {
        tracing::info!("Camera {} powered on", device_id);
        Ok(serde_json::json!({
            "device_id": device_id,
            "status": "powered_on",
            "power": true
        }))
    }

    async fn power_off(&self, device_id: &str) -> Result<serde_json::Value> {
        tracing::info!("Camera {} powered off", device_id);
        Ok(serde_json::json!({
            "device_id": device_id,
            "status": "powered_off",
            "power": false
        }))
    }

    async fn start_recording(&self, device_id: &str) -> Result<serde_json::Value> {
        tracing::info!("Camera {} started recording", device_id);
        Ok(serde_json::json!({
            "device_id": device_id,
            "recording": true,
            "status": "recording_started"
        }))
    }

    async fn stop_recording(&self, device_id: &str) -> Result<serde_json::Value> {
        tracing::info!("Camera {} stopped recording", device_id);
        Ok(serde_json::json!({
            "device_id": device_id,
            "recording": false,
            "status": "recording_stopped"
        }))
    }

    async fn set_resolution(&self, device_id: &str, width: u32, height: u32) -> Result<serde_json::Value> {
        tracing::info!("Camera {} resolution set to {}x{}", device_id, width, height);
        Ok(serde_json::json!({
            "device_id": device_id,
            "resolution": {
                "width": width,
                "height": height
            },
            "status": "resolution_set"
        }))
    }

    async fn ptz_move(&self, device_id: &str, pan: f32, tilt: f32, zoom: f32) -> Result<serde_json::Value> {
        tracing::info!("Camera {} PTZ: pan={}, tilt={}, zoom={}", device_id, pan, tilt, zoom);
        Ok(serde_json::json!({
            "device_id": device_id,
            "ptz": {
                "pan": pan,
                "tilt": tilt,
                "zoom": zoom
            },
            "status": "ptz_moved"
        }))
    }

    async fn ptz_reset(&self, device_id: &str) -> Result<serde_json::Value> {
        tracing::info!("Camera {} PTZ reset", device_id);
        Ok(serde_json::json!({
            "device_id": device_id,
            "ptz": {
                "pan": 0.0,
                "tilt": 0.0,
                "zoom": 1.0
            },
            "status": "ptz_reset"
        }))
    }

    async fn enable_motion_detection(&self, device_id: &str) -> Result<serde_json::Value> {
        tracing::info!("Camera {} motion detection enabled", device_id);
        Ok(serde_json::json!({
            "device_id": device_id,
            "motion_detection": true,
            "status": "motion_detection_enabled"
        }))
    }

    async fn disable_motion_detection(&self, device_id: &str) -> Result<serde_json::Value> {
        tracing::info!("Camera {} motion detection disabled", device_id);
        Ok(serde_json::json!({
            "device_id": device_id,
            "motion_detection": false,
            "status": "motion_detection_disabled"
        }))
    }

    async fn snapshot(&self, device_id: &str) -> Result<serde_json::Value> {
        tracing::info!("Camera {} snapshot taken", device_id);
        Ok(serde_json::json!({
            "device_id": device_id,
            "snapshot_url": format!("/api/devices/{}/snapshot/{}", device_id, chrono::Utc::now().timestamp()),
            "status": "snapshot_taken"
        }))
    }
}

impl Default for CameraTool {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tool_metadata() {
        assert_eq!(CameraTool::name(), "camera_control");
        assert!(!CameraTool::description().is_empty());
    }

    #[tokio::test]
    async fn power_on() {
        let tool = CameraTool::new();
        let cmd = DeviceCommand {
            device_id: "cam-1".to_string(),
            action: "power_on".to_string(),
            parameters: HashMap::new(),
            correlation_id: None,
        };

        let result = tool.execute(&cmd).await.unwrap();
        assert_eq!(result["power"], true);
    }

    #[tokio::test]
    async fn start_recording() {
        let tool = CameraTool::new();
        let cmd = DeviceCommand {
            device_id: "cam-1".to_string(),
            action: "start_recording".to_string(),
            parameters: HashMap::new(),
            correlation_id: None,
        };

        let result = tool.execute(&cmd).await.unwrap();
        assert_eq!(result["recording"], true);
    }

    #[tokio::test]
    async fn ptz_move() {
        let tool = CameraTool::new();
        let mut params = HashMap::new();
        params.insert("pan".to_string(), serde_json::json!(45.0));
        params.insert("tilt".to_string(), serde_json::json!(-15.0));
        params.insert("zoom".to_string(), serde_json::json!(2.0));

        let cmd = DeviceCommand {
            device_id: "cam-1".to_string(),
            action: "ptz_move".to_string(),
            parameters: params,
            correlation_id: None,
        };

        let result = tool.execute(&cmd).await.unwrap();
        assert_eq!(result["ptz"]["pan"], 45.0);
    }
}
