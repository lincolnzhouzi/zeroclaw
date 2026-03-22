use crate::engine::mnn::ffi::*;
use crate::error::{Error, Result};
use crate::types::{MNNBackendType, MNNQuantization};
use std::ffi::{c_int, c_void, CStr, CString};
use std::path::Path;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};

pub struct MNNLlm {
    #[allow(dead_code)]
    inner: Arc<RwLock<Option<*mut MNNLlmOpaque>>>,
    model_path: std::path::PathBuf,
    loaded: Arc<RwLock<bool>>,
}

unsafe impl Send for MNNLlm {}
unsafe impl Sync for MNNLlm {}

impl MNNLlm {
    pub fn new<P: AsRef<Path>>(model_path: P) -> Self {
        Self {
            inner: Arc::new(RwLock::new(None)),
            model_path: model_path.as_ref().to_path_buf(),
            loaded: Arc::new(RwLock::new(false)),
        }
    }

    #[cfg(all(feature = "mnn", mnn_linked))]
    pub async fn load(&self) -> Result<()> {
        let config_path = self.model_path.join("llm_config.json");
        let config_path_str = config_path.to_string_lossy().into_owned();
        let config_cstr = CString::new(config_path_str)?;

        unsafe {
            let llm = Llm_createLLM(config_cstr.as_ptr());
            if llm.is_null() {
                return Err(Error::ModelError("Failed to create LLM".to_string()));
            }

            let ret = Llm_load(llm);
            if ret != 0 {
                Llm_destroy(llm);
                return Err(Error::ModelError("Failed to load LLM model".to_string()));
            }

            let mut inner = self.inner.write().await;
            *inner = Some(llm);
            let mut loaded = self.loaded.write().await;
            *loaded = true;
        }

        tracing::info!("MNN LLM loaded from {:?}", self.model_path);
        Ok(())
    }

    #[cfg(not(all(feature = "mnn", mnn_linked)))]
    pub async fn load(&self) -> Result<()> {
        let mut loaded = self.loaded.write().await;
        *loaded = true;
        tracing::info!("MNN LLM stub loaded from {:?}", self.model_path);
        Ok(())
    }

    #[cfg(all(feature = "mnn", mnn_linked))]
    pub async fn unload(&self) {
        let mut inner = self.inner.write().await;
        if let Some(llm) = inner.take() {
            unsafe {
                Llm_destroy(llm);
            }
        }
        let mut loaded = self.loaded.write().await;
        *loaded = false;
        tracing::info!("MNN LLM unloaded");
    }

    #[cfg(not(all(feature = "mnn", mnn_linked)))]
    pub async fn unload(&self) {
        let mut loaded = self.loaded.write().await;
        *loaded = false;
        tracing::info!("MNN LLM stub unloaded");
    }

    pub async fn is_loaded(&self) -> bool {
        *self.loaded.read().await
    }

    #[cfg(all(feature = "mnn", mnn_linked))]
    pub async fn encode(&self, text: &str) -> Result<Vec<i32>> {
        let inner = self.inner.read().await;
        let llm = inner.ok_or_else(|| Error::ModelError("LLM not loaded".into()))?;

        let text_cstr = CString::new(text)?;
        let mut tokens = vec![0i32; 4096];
        let mut len = 4096i32;

        unsafe {
            let ret = Llm_tokenizer_encode(llm, text_cstr.as_ptr(), tokens.as_mut_ptr(), &mut len);
            if ret != 0 {
                return Err(Error::ModelError("Tokenizer encode failed".into()));
            }
        }

        tokens.truncate(len as usize);
        Ok(tokens)
    }

    #[cfg(not(all(feature = "mnn", mnn_linked)))]
    pub async fn encode(&self, text: &str) -> Result<Vec<i32>> {
        let tokens: Vec<i32> = text.chars().map(|c| c as i32).collect();
        Ok(tokens)
    }

    #[cfg(all(feature = "mnn", mnn_linked))]
    pub async fn decode(&self, token: i32) -> Result<String> {
        let inner = self.inner.read().await;
        let llm = inner.ok_or_else(|| Error::ModelError("LLM not loaded".into()))?;

        let mut buffer = vec![0u8; 256];

        unsafe {
            let ret = Llm_tokenizer_decode(llm, token, buffer.as_mut_ptr() as *mut i8, 256);
            if ret != 0 {
                return Err(Error::ModelError("Tokenizer decode failed".into()));
            }
        }

        let end = buffer.iter().position(|&b| b == 0).unwrap_or(buffer.len());
        String::from_utf8(buffer[..end].to_vec())
            .map_err(|e| Error::ModelError(format!("UTF-8 decode error: {}", e)))
    }

    #[cfg(not(all(feature = "mnn", mnn_linked)))]
    pub async fn decode(&self, token: i32) -> Result<String> {
        if token > 0 && token < 0x10FFFF {
            Ok(String::from(char::from_u32(token as u32).unwrap_or('?')))
        } else {
            Ok(String::new())
        }
    }

    #[cfg(all(feature = "mnn", mnn_linked))]
    pub async fn generate_stream(&self, prompt: &str) -> Result<mpsc::Receiver<String>> {
        let inner = self.inner.read().await;
        let llm = *inner
            .as_ref()
            .ok_or_else(|| Error::ModelError("LLM not loaded".into()))?;

        let (tx, rx) = mpsc::channel(64);
        let tokens = self.encode(prompt).await?;

        struct CallbackData {
            tx: mpsc::Sender<String>,
        }

        let data = Box::new(CallbackData { tx: tx.clone() });
        let data_ptr = Box::into_raw(data) as *mut c_void;

        extern "C" fn callback(text: *const i8, _is_end: i32, user_data: *mut c_void) {
            unsafe {
                let data = &*(user_data as *const CallbackData);
                if text.is_null() {
                    return;
                }
                let c_str = CStr::from_ptr(text);
                if let Ok(s) = c_str.to_str() {
                    let _ = data.tx.blocking_send(s.to_string());
                }
            }
        }

        unsafe {
            Llm_response(
                llm,
                tokens.as_ptr(),
                tokens.len() as i32,
                callback,
                data_ptr,
            );
        }

        Ok(rx)
    }

    #[cfg(not(all(feature = "mnn", mnn_linked)))]
    pub async fn generate_stream(&self, prompt: &str) -> Result<mpsc::Receiver<String>> {
        let (tx, rx) = mpsc::channel(64);

        let response = format!("Echo: {}", prompt);
        let _ = tx.send(response).await;

        Ok(rx)
    }

    #[cfg(all(feature = "mnn", mnn_linked))]
    pub async fn set_config(&self, key: &str, value: &str) -> Result<()> {
        let inner = self.inner.read().await;
        let llm = inner.ok_or_else(|| Error::ModelError("LLM not loaded".into()))?;

        let key_cstr = CString::new(key)?;
        let value_cstr = CString::new(value)?;

        unsafe {
            let ret = Llm_set_config(llm, key_cstr.as_ptr(), value_cstr.as_ptr());
            if ret != 0 {
                return Err(Error::ModelError(format!(
                    "Failed to set config: {}={}",
                    key, value
                )));
            }
        }
        Ok(())
    }

    #[cfg(not(all(feature = "mnn", mnn_linked)))]
    pub async fn set_config(&self, _key: &str, _value: &str) -> Result<()> {
        Ok(())
    }

    #[cfg(all(feature = "mnn", mnn_linked))]
    pub async fn reset(&self) {
        let inner = self.inner.read().await;
        if let Some(llm) = inner.as_ref() {
            unsafe {
                Llm_reset(*llm);
            }
        }
    }

    #[cfg(not(all(feature = "mnn", mnn_linked)))]
    pub async fn reset(&self) {
        tracing::info!("MNN LLM stub reset");
    }
}

impl Drop for MNNLlm {
    fn drop(&mut self) {}
}

#[derive(Debug, Clone)]
pub struct LlmConfig {
    pub model_dir: std::path::PathBuf,
    pub backend_type: MNNBackendType,
    pub thread_num: usize,
    pub precision: String,
    pub memory: String,
    pub quantization: MNNQuantization,
}

impl Default for LlmConfig {
    fn default() -> Self {
        Self {
            model_dir: std::path::PathBuf::from("models/default"),
            backend_type: MNNBackendType::Auto,
            thread_num: 4,
            precision: "normal".to_string(),
            memory: "normal".to_string(),
            quantization: MNNQuantization::INT8,
        }
    }
}

#[derive(Debug, Clone)]
pub struct GenerationConfig {
    pub max_new_tokens: usize,
    pub temperature: f32,
    pub top_k: usize,
    pub top_p: f32,
    pub repetition_penalty: f32,
}

impl Default for GenerationConfig {
    fn default() -> Self {
        Self {
            max_new_tokens: 512,
            temperature: 0.7,
            top_k: 40,
            top_p: 0.9,
            repetition_penalty: 1.1,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_llm_config_default() {
        let config = LlmConfig::default();
        assert_eq!(config.thread_num, 4);
        assert!(matches!(config.backend_type, MNNBackendType::Auto));
    }

    #[test]
    fn test_generation_config_default() {
        let config = GenerationConfig::default();
        assert_eq!(config.max_new_tokens, 512);
        assert!((config.temperature - 0.7).abs() < 0.001);
        assert_eq!(config.top_k, 40);
    }

    #[test]
    fn test_llm_creation() {
        let llm = MNNLlm::new("models/test");
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            assert!(!llm.is_loaded().await);
        });
    }

    #[test]
    fn test_generation_config_custom() {
        let config = GenerationConfig {
            max_new_tokens: 1024,
            temperature: 0.8,
            top_k: 50,
            top_p: 0.95,
            repetition_penalty: 1.2,
        };
        assert_eq!(config.max_new_tokens, 1024);
        assert!((config.temperature - 0.8).abs() < 0.001);
    }
}
