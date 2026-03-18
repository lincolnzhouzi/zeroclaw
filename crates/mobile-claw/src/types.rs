use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::IpAddr;
use std::path::PathBuf;
use std::time::Duration;

pub type DeviceId = String;
pub type SessionId = String;
pub type ConversationId = String;
pub type PeerId = String;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceInfo {
    pub id: DeviceId,
    pub name: String,
    pub device_type: DeviceType,
    pub capabilities: Vec<String>,
    pub endpoint: String,
    pub port: u16,
    pub protocol: ConnectionProtocol,
    pub last_seen: DateTime<Utc>,
    pub state: DeviceState,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum DeviceType {
    Camera,
    AirConditioner,
    Television,
    Light,
    SmartLock,
    Curtain,
    Speaker,
    Thermostat,
    Sensor,
    RobotVacuum,
    SmartPlug,
    Gateway,
    Other(String),
}

impl std::fmt::Display for DeviceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Camera => write!(f, "camera"),
            Self::AirConditioner => write!(f, "air_conditioner"),
            Self::Television => write!(f, "television"),
            Self::Light => write!(f, "light"),
            Self::SmartLock => write!(f, "smart_lock"),
            Self::Curtain => write!(f, "curtain"),
            Self::Speaker => write!(f, "speaker"),
            Self::Thermostat => write!(f, "thermostat"),
            Self::Sensor => write!(f, "sensor"),
            Self::RobotVacuum => write!(f, "robot_vacuum"),
            Self::SmartPlug => write!(f, "smart_plug"),
            Self::Gateway => write!(f, "gateway"),
            Self::Other(name) => write!(f, "{}", name),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DeviceState {
    pub online: bool,
    pub power: Option<bool>,
    pub temperature: Option<f32>,
    pub brightness: Option<u8>,
    pub position: Option<u8>,
    pub volume: Option<u8>,
    pub mode: Option<String>,
    pub custom: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ConnectionProtocol {
    HTTP,
    WebSocket,
    MQTT,
    CoAP,
    BLE,
    USB,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerInfo {
    pub id: PeerId,
    pub endpoint: String,
    pub capabilities: Vec<String>,
    pub last_seen: DateTime<Utc>,
    pub protocol_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceCommand {
    pub device_id: DeviceId,
    pub action: String,
    pub parameters: HashMap<String, serde_json::Value>,
    pub correlation_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandResult {
    pub device_id: DeviceId,
    pub success: bool,
    pub result: Option<serde_json::Value>,
    pub error: Option<String>,
    pub execution_time_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    pub model_path: PathBuf,
    pub model_name: String,
    pub quantization: MNNQuantization,
    pub context_length: usize,
    pub backend_type: MNNBackendType,
    pub thread_count: usize,
    pub power_mode: PowerMode,
}

impl Default for ModelConfig {
    fn default() -> Self {
        Self {
            model_path: PathBuf::from("models/default.mnn"),
            model_name: "default".to_string(),
            quantization: MNNQuantization::INT8,
            context_length: 4096,
            backend_type: MNNBackendType::Auto,
            thread_count: 4,
            power_mode: PowerMode::Balanced,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum MNNQuantization {
    FP32,
    FP16,
    #[default]
    INT8,
    BF16,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub enum MNNBackendType {
    CPU,
    GPU,
    NPU,
    #[default]
    Auto,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum PowerMode {
    Performance,
    #[default]
    Balanced,
    PowerSaving,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub name: String,
    pub variant: String,
    pub quantization: MNNQuantization,
    pub size_bytes: u64,
    pub context_length: usize,
    pub download_url: Option<String>,
    pub local_path: PathBuf,
    pub checksum: String,
    pub model_type: MNNModelType,
    pub multimodal: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MNNModelType {
    TextLLM,
    VisionLLM,
    AudioLLM,
    OmniLLM,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPreferences {
    pub temperature: TemperaturePreference,
    pub entertainment: EntertainmentPreference,
    pub lighting: LightingPreference,
    pub security: SecurityPreference,
}

impl Default for UserPreferences {
    fn default() -> Self {
        Self {
            temperature: TemperaturePreference::default(),
            entertainment: EntertainmentPreference::default(),
            lighting: LightingPreference::default(),
            security: SecurityPreference::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemperaturePreference {
    pub summer_comfortable_range: (f32, f32),
    pub winter_comfortable_range: (f32, f32),
    pub preferred_summer: f32,
    pub preferred_winter: f32,
    pub auto_adjust: bool,
}

impl Default for TemperaturePreference {
    fn default() -> Self {
        Self {
            summer_comfortable_range: (24.0, 28.0),
            winter_comfortable_range: (20.0, 24.0),
            preferred_summer: 26.0,
            preferred_winter: 22.0,
            auto_adjust: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EntertainmentPreference {
    pub preferred_genres: Vec<String>,
    pub preferred_music_style: Vec<String>,
    pub volume_preference: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LightingPreference {
    pub brightness_day: u8,
    pub brightness_night: u8,
    pub color_temperature_day: u16,
    pub color_temperature_night: u16,
    pub auto_dim: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SecurityPreference {
    pub auto_lock: bool,
    pub lock_delay_minutes: u32,
    pub alert_on_unauthorized: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoodContext {
    pub mood_type: MoodType,
    pub confidence: f32,
    pub suggested_actions: Vec<DeviceAction>,
    pub content_recommendations: Vec<ContentRecommendation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MoodType {
    Happy,
    Sad,
    Stressed,
    Anxious,
    Angry,
    Lonely,
    Bored,
    Neutral,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceAction {
    pub device_id: DeviceId,
    pub command: DeviceCommand,
    pub priority: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentRecommendation {
    pub content_type: ContentType,
    pub id: String,
    pub title: String,
    pub description: String,
    pub confidence: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContentType {
    TVShow,
    Movie,
    Music,
    Podcast,
    Comedy,
    Relaxation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareInfo {
    pub cpu_cores: usize,
    pub total_memory: u64,
    pub gpu_available: bool,
    pub gpu_type: Option<GpuType>,
    pub gpu_memory: Option<u64>,
    pub npu_available: bool,
    pub npu_type: Option<NpuType>,
    pub supports_fp16: bool,
    pub supports_dotprod: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GpuType {
    Metal,
    OpenCL,
    Vulkan,
    CUDA,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NpuType {
    CoreML,
    NNAPI,
    HIAI,
    QNN,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WiFiConnection {
    pub device_id: DeviceId,
    pub ip_address: IpAddr,
    pub port: u16,
    pub protocol: ConnectionProtocol,
    pub last_seen: DateTime<Utc>,
    pub latency: Duration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BLEConnection {
    pub device_id: DeviceId,
    pub mac_address: String,
    pub services: Vec<uuid::Uuid>,
    pub characteristics: HashMap<uuid::Uuid, BLECharacteristic>,
    pub rssi: i16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BLECharacteristic {
    pub uuid: uuid::Uuid,
    pub properties: CharacteristicProperties,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacteristicProperties {
    pub read: bool,
    pub write: bool,
    pub notify: bool,
    pub indicate: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultimodalInput {
    pub text: Option<String>,
    pub images: Vec<Vec<u8>>,
    pub audio: Vec<Vec<u8>>,
}

impl MultimodalInput {
    pub fn text_only(text: impl Into<String>) -> Self {
        Self {
            text: Some(text.into()),
            images: Vec::new(),
            audio: Vec::new(),
        }
    }

    pub fn with_image(mut self, image: Vec<u8>) -> Self {
        self.images.push(image);
        self
    }

    pub fn with_audio(mut self, audio: Vec<u8>) -> Self {
        self.audio.push(audio);
        self
    }

    pub fn is_multimodal(&self) -> bool {
        !self.images.is_empty() || !self.audio.is_empty()
    }
}
