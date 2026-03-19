use tauri::{State, Window, Emitter};
use crate::state::AppState;
use serde::{Deserialize, Serialize};
use futures_util::stream::StreamExt;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatResponse {
    pub response: String,
    pub conversation_id: String,
}

#[tauri::command]
pub async fn send_message(
    message: String,
    conversation_id: Option<String>,
    state: State<'_, AppState>,
) -> Result<ChatResponse, String> {
    let runtime = state.runtime.read().await;
    let conv_id = conversation_id.unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
    
    let response = runtime
        .process_user_input(&message)
        .await
        .map_err(|e| e.to_string())?;
    
    Ok(ChatResponse {
        response,
        conversation_id: conv_id,
    })
}

#[tauri::command]
pub async fn stream_message(
    message: String,
    conversation_id: String,
    state: State<'_, AppState>,
    window: Window,
) -> Result<(), String> {
    let runtime = state.runtime.read().await;
    
    let stream = runtime
        .stream_generate(&message)
        .await
        .map_err(|e| e.to_string())?;
    
    let conv_id = conversation_id.clone();
    tokio::spawn(async move {
        let mut stream = stream;
        while let Some(chunk_result) = stream.next().await {
            match chunk_result {
                Ok(text) => {
                    let _ = window.emit("chat:chunk", &text);
                }
                Err(e) => {
                    let _ = window.emit("chat:error", e.to_string());
                    break;
                }
            }
        }
        let _ = window.emit("chat:complete", &conv_id);
    });
    
    Ok(())
}

#[tauri::command]
pub async fn get_conversation_history(
    conversation_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<ChatMessage>, String> {
    let runtime = state.runtime.read().await;
    let history = runtime.get_conversation_history(&conversation_id).await;
    
    Ok(history
        .into_iter()
        .map(|m| ChatMessage {
            role: m.role,
            content: m.content,
            timestamp: chrono::Utc::now().to_rfc3339(),
        })
        .collect())
}

#[tauri::command]
pub async fn clear_conversation(
    conversation_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let mut runtime = state.runtime.write().await;
    runtime
        .clear_conversation(&conversation_id)
        .await
        .map_err(|e| e.to_string())
}
