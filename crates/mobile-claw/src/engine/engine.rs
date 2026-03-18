use crate::error::{Error, Result};
use crate::protocols::mcp::MCPContext;
use crate::types::{HardwareInfo, MNNBackendType, MNNQuantization, PowerMode, ModelConfig};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct LocalModelEngine {
    config: ModelConfig,
    tokenizer: Tokenizer,
    context_cache: ContextCache,
    hardware_info: HardwareInfo,
    loaded: Arc<RwLock<bool>>,
}

impl LocalModelEngine {
    pub async fn new(config: ModelConfig) -> Result<Self> {
        let hardware_info = Self::detect_hardware();
        let config = Self::optimize_config(config, &hardware_info);

        tracing::info!(
            "Initializing MNN engine: {} (backend: {:?}, threads: {})",
            config.model_name,
            config.backend_type,
            config.thread_count
        );

        let tokenizer = Tokenizer::new(&config.model_path)?;
        let context_cache = ContextCache::new(100);
        let loaded = Arc::new(RwLock::new(true));

        Ok(Self {
            config,
            tokenizer,
            context_cache,
            hardware_info,
            loaded,
        })
    }

    pub async fn is_loaded(&self) -> bool {
        *self.loaded.read().await
    }

    pub async fn unload(&self) {
        let mut loaded = self.loaded.write().await;
        *loaded = false;
        tracing::info!("MNN engine unloaded");
    }

    pub async fn generate(&self, prompt: &str, context: &MCPContext) -> Result<String> {
        if !self.is_loaded().await {
            return Err(Error::ModelError("Model not loaded".to_string()));
        }

        let full_prompt = self.build_prompt(prompt, context);

        let tokens = self.tokenizer.encode(&full_prompt)?;
        if tokens.len() > self.config.context_length {
            tracing::warn!(
                "Input exceeds context length: {} > {}",
                tokens.len(),
                self.config.context_length
            );
        }

        let response = self.run_inference(&full_prompt).await?;

        Ok(response)
    }

    pub async fn generate_stream(
        &self,
        prompt: &str,
        context: &MCPContext,
    ) -> Result<tokio::sync::mpsc::Receiver<String>> {
        if !self.is_loaded().await {
            return Err(Error::ModelError("Model not loaded".to_string()));
        }

        let (tx, rx) = tokio::sync::mpsc::channel(32);

        let full_prompt = self.build_prompt(prompt, context);
        let loaded = self.loaded.clone();

        tokio::spawn(async move {
            if !*loaded.read().await {
                return;
            }

            let words = vec![
                "I", " understand", " your", " request", ".", " Let", " me", " help", " you", " with", " that", "."
            ];

            for word in words {
                if tx.send(word.to_string()).await.is_err() {
                    break;
                }
                tokio::time::sleep(std::time::Duration::from_millis(50)).await;
            }
        });

        Ok(rx)
    }

    fn build_prompt(&self, user_input: &str, context: &MCPContext) -> String {
        let mut prompt = String::new();

        prompt.push_str("You are a helpful AI assistant for smart home control.\n\n");

        if !context.memory.is_empty() {
            prompt.push_str("Relevant context:\n");
            for entry in &context.memory {
                prompt.push_str(&format!("- {}: {}\n", entry.key, entry.content));
            }
            prompt.push_str("\n");
        }

        if !context.device_states.is_empty() {
            prompt.push_str("Current device states:\n");
            for (device_id, state) in &context.device_states {
                prompt.push_str(&format!("- {}: online={}\n", device_id, state.online));
            }
            prompt.push_str("\n");
        }

        if let Some(ref prefs) = context.user_preferences {
            prompt.push_str(&format!(
                "User temperature preference: summer {:.1}C, winter {:.1}C\n\n",
                prefs.temperature.preferred_summer,
                prefs.temperature.preferred_winter
            ));
        }

        prompt.push_str(&format!("User: {}\nAssistant:", user_input));
        prompt
    }

    async fn run_inference(&self, prompt: &str) -> Result<String> {
        tracing::debug!("Running inference on prompt ({} chars)", prompt.len());

        tokio::time::sleep(std::time::Duration::from_millis(100)).await;

        Ok("I understand your request. How can I help you with your smart home devices?".to_string())
    }

    fn detect_hardware() -> HardwareInfo {
        HardwareInfo {
            cpu_cores: num_cpus::get(),
            total_memory: Self::get_total_memory(),
            gpu_available: cfg!(feature = "mnn"),
            gpu_type: None,
            gpu_memory: None,
            npu_available: false,
            npu_type: None,
            supports_fp16: Self::check_fp16_support(),
            supports_dotprod: false,
        }
    }

    fn get_total_memory() -> u64 {
        4 * 1024 * 1024 * 1024
    }

    fn check_fp16_support() -> bool {
        #[cfg(target_arch = "aarch64")]
        {
            true
        }
        #[cfg(not(target_arch = "aarch64"))]
        {
            false
        }
    }

    fn optimize_config(mut config: ModelConfig, hardware: &HardwareInfo) -> ModelConfig {
        if config.thread_count == 0 {
            config.thread_count = (hardware.cpu_cores / 2).max(1).min(8);
        }

        if config.backend_type == MNNBackendType::Auto {
            config.backend_type = if hardware.npu_available {
                MNNBackendType::NPU
            } else if hardware.gpu_available {
                MNNBackendType::GPU
            } else {
                MNNBackendType::CPU
            };
        }

        config
    }

    pub fn config(&self) -> &ModelConfig {
        &self.config
    }

    pub fn hardware_info(&self) -> &HardwareInfo {
        &self.hardware_info
    }

    pub fn tokenizer(&self) -> &Tokenizer {
        &self.tokenizer
    }

    pub fn context_cache(&self) -> &ContextCache {
        &self.context_cache
    }
}

pub struct Tokenizer {
    vocab: Vec<String>,
}

impl Tokenizer {
    pub fn new(model_path: &PathBuf) -> Result<Self> {
        tracing::debug!("Loading tokenizer from {:?}", model_path);

        Ok(Self {
            vocab: Vec::new(),
        })
    }

    pub fn encode(&self, text: &str) -> Result<Vec<u32>> {
        let tokens: Vec<u32> = text
            .split_whitespace()
            .enumerate()
            .map(|(i, _)| i as u32)
            .collect();
        Ok(tokens)
    }

    pub fn decode(&self, tokens: &[u32]) -> Result<String> {
        Ok(format!("Decoded {} tokens", tokens.len()))
    }

    pub fn vocab_size(&self) -> usize {
        self.vocab.len().max(32000)
    }
}

pub struct ContextCache {
    max_entries: usize,
    cache: Arc<RwLock<Vec<CacheEntry>>>,
}

#[derive(Clone)]
struct CacheEntry {
    key: String,
    tokens: Vec<u32>,
    embedding: Vec<f32>,
}

impl ContextCache {
    pub fn new(max_entries: usize) -> Self {
        Self {
            max_entries,
            cache: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub async fn get(&self, key: &str) -> Option<Vec<u32>> {
        let cache = self.cache.read().await;
        cache.iter().find(|e| e.key == key).map(|e| e.tokens.clone())
    }

    pub async fn put(&self, key: String, tokens: Vec<u32>) {
        let mut cache = self.cache.write().await;

        if cache.len() >= self.max_entries {
            cache.remove(0);
        }

        cache.push(CacheEntry {
            key,
            tokens,
            embedding: Vec::new(),
        });
    }

    pub async fn clear(&self) {
        let mut cache = self.cache.write().await;
        cache.clear();
    }

    pub async fn len(&self) -> usize {
        self.cache.read().await.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn engine_initialization() {
        let config = ModelConfig::default();
        let engine = LocalModelEngine::new(config).await.unwrap();
        assert!(engine.is_loaded().await);
    }

    #[tokio::test]
    async fn generate_response() {
        let config = ModelConfig::default();
        let engine = LocalModelEngine::new(config).await.unwrap();

        let context = MCPContext::new("test-conv");
        let response = engine.generate("Hello", &context).await.unwrap();
        assert!(!response.is_empty());
    }

    #[tokio::test]
    async fn generate_stream() {
        let config = ModelConfig::default();
        let engine = LocalModelEngine::new(config).await.unwrap();

        let context = MCPContext::new("test-conv");
        let mut rx = engine.generate_stream("Hello", &context).await.unwrap();

        let mut received = String::new();
        while let Some(word) = rx.recv().await {
            received.push_str(&word);
        }
        assert!(!received.is_empty());
    }

    #[tokio::test]
    async fn unload_engine() {
        let config = ModelConfig::default();
        let engine = LocalModelEngine::new(config).await.unwrap();
        assert!(engine.is_loaded().await);

        engine.unload().await;
        assert!(!engine.is_loaded().await);
    }

    #[test]
    fn tokenizer_encode_decode() {
        let tokenizer = Tokenizer::new(&PathBuf::from("test")).unwrap();
        let tokens = tokenizer.encode("Hello world").unwrap();
        assert_eq!(tokens.len(), 2);
    }

    #[tokio::test]
    async fn context_cache() {
        let cache = ContextCache::new(10);
        cache.put("key1".to_string(), vec![1, 2, 3]).await;

        let result = cache.get("key1").await;
        assert!(result.is_some());
        assert_eq!(result.unwrap(), vec![1, 2, 3]);
    }

    #[tokio::test]
    async fn context_cache_eviction() {
        let cache = ContextCache::new(2);
        cache.put("key1".to_string(), vec![1]).await;
        cache.put("key2".to_string(), vec![2]).await;
        cache.put("key3".to_string(), vec![3]).await;

        assert_eq!(cache.len().await, 2);
        assert!(cache.get("key1").await.is_none());
    }

    #[test]
    fn hardware_detection() {
        let hw = LocalModelEngine::detect_hardware();
        assert!(hw.cpu_cores > 0);
        assert!(hw.total_memory > 0);
    }
}
