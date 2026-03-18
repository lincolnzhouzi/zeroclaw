use crate::types::{DeviceInfo, DeviceType, ConnectionProtocol, DeviceState};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CameraState {
    pub power: bool,
    pub recording: bool,
    pub resolution: Resolution,
    pub ptz: Option<PTZState>,
    pub motion_detection: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Resolution {
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PTZState {
    pub pan: f32,
    pub tilt: f32,
    pub zoom: f32,
}

impl Default for CameraState {
    fn default() -> Self {
        Self {
            power: false,
            recording: false,
            resolution: Resolution {
                width: 1920,
                height: 1080,
            },
            ptz: None,
            motion_detection: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AirConditionerState {
    pub power: bool,
    pub temperature: f32,
    pub mode: ACMode,
    pub fan_speed: FanSpeed,
    pub timer: Option<TimerSetting>,
    pub eco_mode: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ACMode {
    Cool,
    Heat,
    Dehumidify,
    Fan,
    Auto,
}

impl std::fmt::Display for ACMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Cool => write!(f, "cool"),
            Self::Heat => write!(f, "heat"),
            Self::Dehumidify => write!(f, "dehumidify"),
            Self::Fan => write!(f, "fan"),
            Self::Auto => write!(f, "auto"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum FanSpeed {
    Low,
    Medium,
    High,
    Auto,
}

impl Default for AirConditionerState {
    fn default() -> Self {
        Self {
            power: false,
            temperature: 24.0,
            mode: ACMode::Auto,
            fan_speed: FanSpeed::Auto,
            timer: None,
            eco_mode: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimerSetting {
    pub enabled: bool,
    pub duration_minutes: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelevisionState {
    pub power: bool,
    pub channel: Option<u32>,
    pub volume: u8,
    pub muted: bool,
    pub input_source: String,
    pub playing: bool,
}

impl Default for TelevisionState {
    fn default() -> Self {
        Self {
            power: false,
            channel: Some(1),
            volume: 50,
            muted: false,
            input_source: "HDMI1".to_string(),
            playing: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LightState {
    pub power: bool,
    pub brightness: u8,
    pub color_temperature: Option<u16>,
    pub rgb_color: Option<RGBColor>,
    pub scene: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RGBColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Default for LightState {
    fn default() -> Self {
        Self {
            power: false,
            brightness: 100,
            color_temperature: Some(4000),
            rgb_color: None,
            scene: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmartLockState {
    pub locked: bool,
    pub battery_level: Option<u8>,
    pub last_unlock: Option<DateTime<Utc>>,
    pub temp_codes: Vec<TemporaryCode>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporaryCode {
    pub code: String,
    pub expires_at: DateTime<Utc>,
    pub uses_remaining: u32,
}

impl Default for SmartLockState {
    fn default() -> Self {
        Self {
            locked: true,
            battery_level: Some(100),
            last_unlock: None,
            temp_codes: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurtainState {
    pub position: u8,
    pub moving: bool,
    pub scene: Option<String>,
}

impl Default for CurtainState {
    fn default() -> Self {
        Self {
            position: 0,
            moving: false,
            scene: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensorState {
    pub value: f64,
    pub unit: String,
    pub last_update: DateTime<Utc>,
}

impl Default for SensorState {
    fn default() -> Self {
        Self {
            value: 0.0,
            unit: "".to_string(),
            last_update: Utc::now(),
        }
    }
}

pub fn create_camera_device(id: &str, name: &str, endpoint: &str) -> DeviceInfo {
    DeviceInfo {
        id: id.to_string(),
        name: name.to_string(),
        device_type: DeviceType::Camera,
        capabilities: vec![
            "power".to_string(),
            "recording".to_string(),
            "ptz".to_string(),
            "motion_detection".to_string(),
        ],
        endpoint: endpoint.to_string(),
        port: 8080,
        protocol: ConnectionProtocol::HTTP,
        last_seen: Utc::now(),
        state: DeviceState::default(),
    }
}

pub fn create_ac_device(id: &str, name: &str, endpoint: &str) -> DeviceInfo {
    DeviceInfo {
        id: id.to_string(),
        name: name.to_string(),
        device_type: DeviceType::AirConditioner,
        capabilities: vec![
            "power".to_string(),
            "temperature".to_string(),
            "mode".to_string(),
            "fan_speed".to_string(),
            "timer".to_string(),
        ],
        endpoint: endpoint.to_string(),
        port: 8080,
        protocol: ConnectionProtocol::HTTP,
        last_seen: Utc::now(),
        state: DeviceState::default(),
    }
}

pub fn create_light_device(id: &str, name: &str, endpoint: &str) -> DeviceInfo {
    DeviceInfo {
        id: id.to_string(),
        name: name.to_string(),
        device_type: DeviceType::Light,
        capabilities: vec![
            "power".to_string(),
            "brightness".to_string(),
            "color_temperature".to_string(),
            "rgb".to_string(),
            "scene".to_string(),
        ],
        endpoint: endpoint.to_string(),
        port: 8080,
        protocol: ConnectionProtocol::HTTP,
        last_seen: Utc::now(),
        state: DeviceState::default(),
    }
}

pub fn create_tv_device(id: &str, name: &str, endpoint: &str) -> DeviceInfo {
    DeviceInfo {
        id: id.to_string(),
        name: name.to_string(),
        device_type: DeviceType::Television,
        capabilities: vec![
            "power".to_string(),
            "channel".to_string(),
            "volume".to_string(),
            "input".to_string(),
            "playback".to_string(),
        ],
        endpoint: endpoint.to_string(),
        port: 8080,
        protocol: ConnectionProtocol::HTTP,
        last_seen: Utc::now(),
        state: DeviceState::default(),
    }
}

pub fn create_lock_device(id: &str, name: &str, endpoint: &str) -> DeviceInfo {
    DeviceInfo {
        id: id.to_string(),
        name: name.to_string(),
        device_type: DeviceType::SmartLock,
        capabilities: vec![
            "lock".to_string(),
            "unlock".to_string(),
            "temp_code".to_string(),
            "access_log".to_string(),
        ],
        endpoint: endpoint.to_string(),
        port: 8080,
        protocol: ConnectionProtocol::HTTP,
        last_seen: Utc::now(),
        state: DeviceState::default(),
    }
}

pub fn create_curtain_device(id: &str, name: &str, endpoint: &str) -> DeviceInfo {
    DeviceInfo {
        id: id.to_string(),
        name: name.to_string(),
        device_type: DeviceType::Curtain,
        capabilities: vec![
            "open".to_string(),
            "close".to_string(),
            "position".to_string(),
            "scene".to_string(),
        ],
        endpoint: endpoint.to_string(),
        port: 8080,
        protocol: ConnectionProtocol::HTTP,
        last_seen: Utc::now(),
        state: DeviceState::default(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_camera_device_test() {
        let device = create_camera_device("cam-1", "Living Room Camera", "192.168.1.100");
        assert_eq!(device.id, "cam-1");
        assert_eq!(device.device_type, DeviceType::Camera);
        assert!(device.capabilities.contains(&"recording".to_string()));
    }

    #[test]
    fn create_ac_device_test() {
        let device = create_ac_device("ac-1", "Bedroom AC", "192.168.1.101");
        assert_eq!(device.device_type, DeviceType::AirConditioner);
        assert!(device.capabilities.contains(&"temperature".to_string()));
    }

    #[test]
    fn create_light_device_test() {
        let device = create_light_device("light-1", "Kitchen Light", "192.168.1.102");
        assert_eq!(device.device_type, DeviceType::Light);
        assert!(device.capabilities.contains(&"brightness".to_string()));
    }

    #[test]
    fn ac_mode_display() {
        assert_eq!(ACMode::Cool.to_string(), "cool");
        assert_eq!(ACMode::Heat.to_string(), "heat");
        assert_eq!(ACMode::Auto.to_string(), "auto");
    }

    #[test]
    fn default_states() {
        let camera = CameraState::default();
        assert!(!camera.power);
        assert!(!camera.recording);

        let ac = AirConditionerState::default();
        assert_eq!(ac.temperature, 24.0);
        assert_eq!(ac.mode, ACMode::Auto);

        let light = LightState::default();
        assert_eq!(light.brightness, 100);

        let lock = SmartLockState::default();
        assert!(lock.locked);
    }
}
