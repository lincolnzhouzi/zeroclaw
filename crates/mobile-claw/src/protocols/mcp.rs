use crate::types::{ConversationId, DeviceState, UserPreferences};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPMessage {
    pub role: String,
    pub content: String,
    pub context: Option<MCPContext>,
    pub tools: Vec<MCPToolCall>,
}

impl MCPMessage {
    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: "user".to_string(),
            content: content.into(),
            context: None,
            tools: Vec::new(),
        }
    }

    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: "assistant".to_string(),
            content: content.into(),
            context: None,
            tools: Vec::new(),
        }
    }

    pub fn system(content: impl Into<String>) -> Self {
        Self {
            role: "system".to_string(),
            content: content.into(),
            context: None,
            tools: Vec::new(),
        }
    }

    pub fn with_context(mut self, context: MCPContext) -> Self {
        self.context = Some(context);
        self
    }

    pub fn with_tool_call(mut self, tool_call: MCPToolCall) -> Self {
        self.tools.push(tool_call);
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPContext {
    pub conversation_id: ConversationId,
    pub memory: Vec<MemoryEntry>,
    pub device_states: HashMap<String, DeviceState>,
    pub user_preferences: Option<UserPreferences>,
    pub timestamp: DateTime<Utc>,
}

impl MCPContext {
    pub fn new(conversation_id: impl Into<String>) -> Self {
        Self {
            conversation_id: conversation_id.into(),
            memory: Vec::new(),
            device_states: HashMap::new(),
            user_preferences: None,
            timestamp: Utc::now(),
        }
    }

    pub fn with_memory(mut self, memory: Vec<MemoryEntry>) -> Self {
        self.memory = memory;
        self
    }

    pub fn with_device_states(mut self, states: HashMap<String, DeviceState>) -> Self {
        self.device_states = states;
        self
    }

    pub fn with_user_preferences(mut self, prefs: UserPreferences) -> Self {
        self.user_preferences = Some(prefs);
        self
    }

    pub fn add_memory(&mut self, entry: MemoryEntry) {
        self.memory.push(entry);
    }

    pub fn update_device_state(&mut self, device_id: &str, state: DeviceState) {
        self.device_states.insert(device_id.to_string(), state);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryEntry {
    pub id: String,
    pub key: String,
    pub content: String,
    pub category: MemoryCategory,
    pub timestamp: DateTime<Utc>,
    pub relevance_score: Option<f32>,
}

impl MemoryEntry {
    pub fn new(
        key: impl Into<String>,
        content: impl Into<String>,
        category: MemoryCategory,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            key: key.into(),
            content: content.into(),
            category,
            timestamp: Utc::now(),
            relevance_score: None,
        }
    }

    pub fn with_relevance(mut self, score: f32) -> Self {
        self.relevance_score = Some(score);
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum MemoryCategory {
    Core,
    Daily,
    Conversation,
    Custom(String),
}

impl std::fmt::Display for MemoryCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Core => write!(f, "core"),
            Self::Daily => write!(f, "daily"),
            Self::Conversation => write!(f, "conversation"),
            Self::Custom(name) => write!(f, "{}", name),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPToolCall {
    pub id: String,
    pub name: String,
    pub arguments: serde_json::Value,
}

impl MCPToolCall {
    pub fn new(name: impl Into<String>, arguments: serde_json::Value) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name: name.into(),
            arguments,
        }
    }

    pub fn device_control(
        device_id: &str,
        action: &str,
        params: HashMap<String, serde_json::Value>,
    ) -> Self {
        Self::new(
            "device_control",
            serde_json::json!({
                "device_id": device_id,
                "action": action,
                "parameters": params
            }),
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPToolResult {
    pub tool_call_id: String,
    pub name: String,
    pub result: serde_json::Value,
    pub success: bool,
}

pub struct MCPProtocol {
    context_manager: ContextManager,
    tool_registry: ToolRegistry,
    current_conversation: Arc<RwLock<Option<String>>>,
}

impl MCPProtocol {
    pub fn new() -> Self {
        Self {
            context_manager: ContextManager::new(),
            tool_registry: ToolRegistry::new(),
            current_conversation: Arc::new(RwLock::new(None)),
        }
    }

    pub async fn start_conversation(&self) -> ConversationId {
        let conversation_id = uuid::Uuid::new_v4().to_string();
        let mut current = self.current_conversation.write().await;
        *current = Some(conversation_id.clone());
        conversation_id
    }

    pub async fn end_conversation(&self) {
        let mut current = self.current_conversation.write().await;
        *current = None;
    }

    pub async fn build_context(&self) -> MCPContext {
        let conversation_id = self
            .current_conversation
            .read()
            .await
            .clone()
            .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

        let memory = self
            .context_manager
            .get_relevant_memory(&conversation_id)
            .await;
        let device_states = self.get_all_device_states().await;

        MCPContext {
            conversation_id,
            memory,
            device_states,
            user_preferences: None,
            timestamp: Utc::now(),
        }
    }

    pub async fn build_context_with_prefs(&self, prefs: UserPreferences) -> MCPContext {
        let mut context = self.build_context().await;
        context.user_preferences = Some(prefs);
        context
    }

    async fn get_all_device_states(&self) -> HashMap<String, DeviceState> {
        HashMap::new()
    }

    pub async fn store_memory(&self, key: &str, content: &str, category: MemoryCategory) {
        self.context_manager.store(key, content, category).await;
    }

    pub async fn recall_memory(&self, query: &str, limit: usize) -> Vec<MemoryEntry> {
        self.context_manager.recall(query, limit).await
    }

    pub fn register_tool(&mut self, name: &str, description: &str, schema: serde_json::Value) {
        self.tool_registry.register(name, description, schema);
    }

    pub fn get_tool_specs(&self) -> Vec<ToolSpec> {
        self.tool_registry.list_tools()
    }

    pub fn to_provider_messages(
        &self,
        mcp_messages: &[MCPMessage],
    ) -> Vec<zeroclaw::providers::ChatMessage> {
        mcp_messages
            .iter()
            .map(|m| zeroclaw::providers::ChatMessage {
                role: m.role.clone(),
                content: m.content.clone(),
            })
            .collect()
    }

    pub async fn process_tool_call(&self, tool_call: &MCPToolCall) -> MCPToolResult {
        match tool_call.name.as_str() {
            "device_control" => {
                let result = self.execute_device_control(&tool_call.arguments).await;
                MCPToolResult {
                    tool_call_id: tool_call.id.clone(),
                    name: tool_call.name.clone(),
                    result,
                    success: true,
                }
            }
            _ => MCPToolResult {
                tool_call_id: tool_call.id.clone(),
                name: tool_call.name.clone(),
                result: serde_json::json!({"error": "Unknown tool"}),
                success: false,
            },
        }
    }

    async fn execute_device_control(&self, args: &serde_json::Value) -> serde_json::Value {
        let device_id = args.get("device_id").and_then(|v| v.as_str()).unwrap_or("");
        let action = args.get("action").and_then(|v| v.as_str()).unwrap_or("");
        let params = args
            .get("parameters")
            .cloned()
            .unwrap_or(serde_json::json!({}));

        serde_json::json!({
            "device_id": device_id,
            "action": action,
            "status": "executed",
            "parameters": params
        })
    }

    pub fn get_history(&self) -> Vec<crate::runtime::ChatMessageInfo> {
        Vec::new()
    }

    pub fn clear_history(&mut self) {}
}

impl Default for MCPProtocol {
    fn default() -> Self {
        Self::new()
    }
}

struct ContextManager {
    memory_store: Arc<RwLock<HashMap<String, Vec<MemoryEntry>>>>,
    conversation_memory: Arc<RwLock<HashMap<String, Vec<MemoryEntry>>>>,
}

impl ContextManager {
    fn new() -> Self {
        Self {
            memory_store: Arc::new(RwLock::new(HashMap::new())),
            conversation_memory: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    async fn store(&self, key: &str, content: &str, category: MemoryCategory) {
        let entry = MemoryEntry::new(key, content, category);
        let mut store = self.memory_store.write().await;
        store.entry(key.to_string()).or_default().push(entry);
    }

    async fn recall(&self, query: &str, limit: usize) -> Vec<MemoryEntry> {
        let store = self.memory_store.read().await;
        let mut results: Vec<MemoryEntry> = store
            .values()
            .flat_map(|entries| entries.iter())
            .filter(|entry| {
                entry.content.to_lowercase().contains(&query.to_lowercase())
                    || entry.key.to_lowercase().contains(&query.to_lowercase())
            })
            .cloned()
            .take(limit)
            .collect();

        results.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        results
    }

    async fn get_relevant_memory(&self, conversation_id: &str) -> Vec<MemoryEntry> {
        let conv_mem = self.conversation_memory.read().await;
        conv_mem.get(conversation_id).cloned().unwrap_or_default()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolSpec {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}

struct ToolRegistry {
    tools: HashMap<String, ToolSpec>,
}

impl ToolRegistry {
    fn new() -> Self {
        let mut registry = Self {
            tools: HashMap::new(),
        };

        registry.register_default_tools();
        registry
    }

    fn register(&mut self, name: &str, description: &str, parameters: serde_json::Value) {
        self.tools.insert(
            name.to_string(),
            ToolSpec {
                name: name.to_string(),
                description: description.to_string(),
                parameters,
            },
        );
    }

    fn list_tools(&self) -> Vec<ToolSpec> {
        self.tools.values().cloned().collect()
    }

    fn register_default_tools(&mut self) {
        self.register(
            "device_control",
            "Control a smart device",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "device_id": { "type": "string", "description": "Device ID to control" },
                    "action": { "type": "string", "description": "Action to perform" },
                    "parameters": { "type": "object", "description": "Action parameters" }
                },
                "required": ["device_id", "action"]
            }),
        );

        self.register(
            "get_device_status",
            "Get the current status of a device",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "device_id": { "type": "string", "description": "Device ID to query" }
                },
                "required": ["device_id"]
            }),
        );

        self.register(
            "discover_devices",
            "Discover available smart devices",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "device_type": { "type": "string", "description": "Filter by device type" }
                }
            }),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn protocol_initialization() {
        let protocol = MCPProtocol::new();
        let context = protocol.build_context().await;
        assert!(!context.conversation_id.is_empty());
    }

    #[tokio::test]
    async fn start_conversation() {
        let protocol = MCPProtocol::new();
        let conv_id = protocol.start_conversation().await;
        assert!(!conv_id.is_empty());
    }

    #[test]
    fn mcp_message_constructors() {
        let user = MCPMessage::user("Hello");
        assert_eq!(user.role, "user");
        assert_eq!(user.content, "Hello");

        let assistant = MCPMessage::assistant("Hi there");
        assert_eq!(assistant.role, "assistant");

        let system = MCPMessage::system("You are helpful");
        assert_eq!(system.role, "system");
    }

    #[test]
    fn mcp_context_builder() {
        let context = MCPContext::new("conv-1").with_memory(vec![MemoryEntry::new(
            "key1",
            "content1",
            MemoryCategory::Core,
        )]);

        assert_eq!(context.conversation_id, "conv-1");
        assert_eq!(context.memory.len(), 1);
    }

    #[test]
    fn tool_call_creation() {
        let tool_call = MCPToolCall::new("test_tool", serde_json::json!({"param": "value"}));
        assert_eq!(tool_call.name, "test_tool");
        assert!(!tool_call.id.is_empty());
    }

    #[tokio::test]
    async fn store_and_recall_memory() {
        let protocol = MCPProtocol::new();
        protocol
            .store_memory("pref_temp", "User prefers 24 degrees", MemoryCategory::Core)
            .await;

        let results = protocol.recall_memory("temp", 10).await;
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].key, "pref_temp");
    }

    #[test]
    fn default_tools_registered() {
        let protocol = MCPProtocol::new();
        let tools = protocol.get_tool_specs();
        assert!(!tools.is_empty());
        assert!(tools.iter().any(|t| t.name == "device_control"));
    }

    #[tokio::test]
    async fn process_tool_call() {
        let protocol = MCPProtocol::new();
        let tool_call = MCPToolCall::device_control("device-1", "power_on", HashMap::new());

        let result = protocol.process_tool_call(&tool_call).await;
        assert!(result.success);
    }
}
