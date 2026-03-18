pub mod runtime;
pub mod protocols;
pub mod engine;
pub mod device;
pub mod tools;
pub mod profile;
pub mod network;
pub mod types;
pub mod error;

pub use runtime::MobileClawRuntime;
pub use error::{Error, Result};
pub use types::*;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const NAME: &str = "Mobile Claw";
