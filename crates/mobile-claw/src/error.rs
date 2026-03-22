use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Device not found: {0}")]
    DeviceNotFound(String),

    #[error("Device connection failed: {0}")]
    ConnectionFailed(String),

    #[error("Protocol error: {0}")]
    ProtocolError(String),

    #[error("Model error: {0}")]
    ModelError(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("Bluetooth error: {0}")]
    BluetoothError(String),

    #[error("Timeout error: {0}")]
    TimeoutError(String),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Invalid state: {0}")]
    InvalidState(String),

    #[error("Command failed: {0}")]
    CommandFailed(String),

    #[error("Provider error: {0}")]
    ProviderError(String),

    #[error("Memory error: {0}")]
    MemoryError(String),

    #[error("Profile error: {0}")]
    ProfileError(String),

    #[error("Not implemented: {0}")]
    NotImplemented(String),

    #[error("FFI error: {0}")]
    FFIError(String),

    #[error("Unknown error: {0}")]
    Unknown(String),
}

pub type Result<T> = std::result::Result<T, Error>;

impl From<anyhow::Error> for Error {
    fn from(err: anyhow::Error) -> Self {
        Error::Unknown(err.to_string())
    }
}

impl From<std::ffi::NulError> for Error {
    fn from(err: std::ffi::NulError) -> Self {
        Error::FFIError(err.to_string())
    }
}
