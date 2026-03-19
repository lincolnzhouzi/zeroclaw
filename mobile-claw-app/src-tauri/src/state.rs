use std::sync::Arc;
use tokio::sync::RwLock;
use mobile_claw::runtime::MobileClawRuntime;
use mobile_claw::runtime::config::RuntimeConfig;

#[derive(Clone)]
pub struct AppState {
    pub runtime: Arc<RwLock<MobileClawRuntime>>,
    pub config: RuntimeConfig,
}

impl AppState {
    pub fn new(config: RuntimeConfig) -> Self {
        let runtime = MobileClawRuntime::new(config.clone());
        Self {
            runtime: Arc::new(RwLock::new(runtime)),
            config,
        }
    }

    pub async fn initialize(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let runtime = self.runtime.write().await;
        runtime.start().await?;
        Ok(())
    }

    pub async fn shutdown(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let runtime = self.runtime.write().await;
        runtime.stop().await?;
        Ok(())
    }
}
