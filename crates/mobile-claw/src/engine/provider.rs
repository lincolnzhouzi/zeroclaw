use crate::engine::LocalModelEngine;
use crate::error::{Error, Result};
use crate::protocols::mcp::MCPContext;
use crate::types::ModelConfig;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone)]
pub struct ProviderCapabilities {
    pub native_tool_calling: bool,
    pub vision: bool,
    pub streaming: bool,
}

impl Default for ProviderCapabilities {
    fn default() -> Self {
        Self {
            native_tool_calling: true,
            vision: false,
            streaming: true,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ChatResponse {
    pub text: Option<String>,
    pub tool_calls: Vec<ToolCall>,
    pub usage: Option<TokenUsage>,
    pub reasoning_content: Option<String>,
}

impl ChatResponse {
    pub fn text_only(text: impl Into<String>) -> Self {
        Self {
            text: Some(text.into()),
            tool_calls: Vec::new(),
            usage: None,
            reasoning_content: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub arguments: serde_json::Value,
}

#[derive(Debug, Clone, Default)]
pub struct TokenUsage {
    pub input_tokens: Option<u64>,
    pub output_tokens: Option<u64>,
}

#[derive(Debug, Clone)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

impl ChatMessage {
    pub fn system(content: impl Into<String>) -> Self {
        Self {
            role: "system".into(),
            content: content.into(),
        }
    }

    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: "user".into(),
            content: content.into(),
        }
    }

    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: "assistant".into(),
            content: content.into(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct StreamChunk {
    pub delta: String,
    pub is_final: bool,
}

impl StreamChunk {
    pub fn delta(text: impl Into<String>) -> Self {
        Self {
            delta: text.into(),
            is_final: false,
        }
    }

    pub fn final_chunk() -> Self {
        Self {
            delta: String::new(),
            is_final: true,
        }
    }
}

pub struct MNNProvider {
    engine: Arc<RwLock<Option<LocalModelEngine>>>,
    config: ModelConfig,
    capabilities: ProviderCapabilities,
}

impl MNNProvider {
    pub fn new(config: ModelConfig) -> Self {
        Self {
            engine: Arc::new(RwLock::new(None)),
            config,
            capabilities: ProviderCapabilities::default(),
        }
    }

    pub async fn load(&self) -> Result<()> {
        let engine = LocalModelEngine::new(self.config.clone()).await?;
        let mut guard = self.engine.write().await;
        *guard = Some(engine);
        tracing::info!("MNN Provider loaded: {}", self.config.model_name);
        Ok(())
    }

    pub async fn unload(&self) {
        let mut guard = self.engine.write().await;
        if let Some(engine) = guard.take() {
            engine.unload().await;
        }
        tracing::info!("MNN Provider unloaded");
    }

    pub async fn is_loaded(&self) -> bool {
        self.engine.read().await.is_some()
    }

    pub fn capabilities(&self) -> &ProviderCapabilities {
        &self.capabilities
    }

    pub async fn chat_with_system(
        &self,
        system_prompt: Option<&str>,
        message: &str,
        temperature: f64,
    ) -> Result<String> {
        let engine = self.engine.read().await;
        let engine = engine.as_ref().ok_or_else(|| Error::ModelError("Model not loaded".to_string()))?;

        let context = MCPContext::new("single-turn");
        let full_prompt = if let Some(sys) = system_prompt {
            format!("{}\n\n{}", sys, message)
        } else {
            message.to_string()
        };

        engine.generate(&full_prompt, &context).await
    }

    pub async fn chat(
        &self,
        messages: &[ChatMessage],
        temperature: f64,
    ) -> Result<ChatResponse> {
        let engine = self.engine.read().await;
        let engine = engine.as_ref().ok_or_else(|| Error::ModelError("Model not loaded".to_string()))?;

        let context = MCPContext::new("multi-turn");
        let prompt = self.messages_to_prompt(messages);

        let text = engine.generate(&prompt, &context).await?;

        Ok(ChatResponse::text_only(text))
    }

    fn messages_to_prompt(&self, messages: &[ChatMessage]) -> String {
        messages
            .iter()
            .map(|m| format!("{}: {}", m.role, m.content))
            .collect::<Vec<_>>()
            .join("\n")
    }

    pub fn supports_streaming(&self) -> bool {
        self.capabilities.streaming
    }

    pub async fn stream_chat(
        &self,
        messages: &[ChatMessage],
        temperature: f64,
    ) -> Result<tokio::sync::mpsc::Receiver<StreamChunk>> {
        let engine = self.engine.read().await;
        let engine = engine.as_ref().ok_or_else(|| Error::ModelError("Model not loaded".to_string()))?;

        let context = MCPContext::new("streaming");
        let prompt = self.messages_to_prompt(messages);

        let mut rx = engine.generate_stream(&prompt, &context).await?;

        let (tx, out_rx) = tokio::sync::mpsc::channel(32);

        tokio::spawn(async move {
            while let Some(word) = rx.recv().await {
                if tx.send(StreamChunk::delta(word)).await.is_err() {
                    break;
                }
            }
            let _ = tx.send(StreamChunk::final_chunk()).await;
        });

        Ok(out_rx)
    }

    pub async fn generate_with_context(
        &self,
        prompt: &str,
        context: &MCPContext,
    ) -> Result<String> {
        let engine = self.engine.read().await;
        let engine = engine.as_ref().ok_or_else(|| Error::ModelError("Model not loaded".to_string()))?;

        engine.generate(prompt, context).await
    }

    pub fn config(&self) -> &ModelConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn provider_creation() {
        let config = ModelConfig::default();
        let provider = MNNProvider::new(config);
        assert!(!provider.is_loaded().await);
    }

    #[tokio::test]
    async fn provider_load_unload() {
        let config = ModelConfig::default();
        let provider = MNNProvider::new(config);

        provider.load().await.unwrap();
        assert!(provider.is_loaded().await);

        provider.unload().await;
        assert!(!provider.is_loaded().await);
    }

    #[tokio::test]
    async fn chat_with_system() {
        let config = ModelConfig::default();
        let provider = MNNProvider::new(config);
        provider.load().await.unwrap();

        let response = provider
            .chat_with_system(Some("You are helpful"), "Hello", 0.7)
            .await
            .unwrap();
        assert!(!response.is_empty());
    }

    #[tokio::test]
    async fn chat_multi_turn() {
        let config = ModelConfig::default();
        let provider = MNNProvider::new(config);
        provider.load().await.unwrap();

        let messages = vec![
            ChatMessage::system("You are helpful"),
            ChatMessage::user("Hello"),
        ];

        let response = provider.chat(&messages, 0.7).await.unwrap();
        assert!(response.text.is_some());
    }

    #[tokio::test]
    async fn stream_chat() {
        let config = ModelConfig::default();
        let provider = MNNProvider::new(config);
        provider.load().await.unwrap();

        let messages = vec![ChatMessage::user("Hello")];
        let mut rx = provider.stream_chat(&messages, 0.7).await.unwrap();

        let mut chunks = 0;
        while let Some(chunk) = rx.recv().await {
            if chunk.is_final {
                break;
            }
            chunks += 1;
        }
        assert!(chunks > 0);
    }

    #[test]
    fn capabilities_default() {
        let caps = ProviderCapabilities::default();
        assert!(caps.native_tool_calling);
        assert!(caps.streaming);
        assert!(!caps.vision);
    }

    #[test]
    fn chat_response_text_only() {
        let response = ChatResponse::text_only("Hello");
        assert_eq!(response.text, Some("Hello".to_string()));
        assert!(response.tool_calls.is_empty());
    }
}
