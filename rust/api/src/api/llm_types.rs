use std::collections::HashMap;

use flutter_rust_bridge::frb;
use kelivo_core::llm::{
    ChatDelta, ChatEvent, ChatMessage, ChatRequest, ChatResponse, ChatRole, ChatUsage,
    MessageContent,
};
use serde::{Deserialize, Serialize};

#[frb]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FrbChatRole {
    User,
    Assistant,
    System,
    Tool,
}

impl From<FrbChatRole> for ChatRole {
    fn from(value: FrbChatRole) -> Self {
        match value {
            FrbChatRole::User => ChatRole::User,
            FrbChatRole::Assistant => ChatRole::Assistant,
            FrbChatRole::System => ChatRole::System,
            FrbChatRole::Tool => ChatRole::Tool,
        }
    }
}

impl From<ChatRole> for FrbChatRole {
    fn from(value: ChatRole) -> Self {
        match value {
            ChatRole::User => FrbChatRole::User,
            ChatRole::Assistant => FrbChatRole::Assistant,
            ChatRole::System => FrbChatRole::System,
            ChatRole::Tool => FrbChatRole::Tool,
        }
    }
}

#[frb]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrbChatMessage {
    pub role: FrbChatRole,
    pub parts: Vec<String>,
}

impl From<FrbChatMessage> for ChatMessage {
    fn from(value: FrbChatMessage) -> Self {
        let content = value.parts.into_iter().map(MessageContent::Text).collect();
        ChatMessage {
            role: value.role.into(),
            content,
        }
    }
}

impl From<ChatMessage> for FrbChatMessage {
    fn from(value: ChatMessage) -> Self {
        let parts = value
            .content
            .into_iter()
            .filter_map(|part| match part {
                MessageContent::Text(text) => Some(text),
            })
            .collect();
        FrbChatMessage {
            role: value.role.into(),
            parts,
        }
    }
}

#[frb]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrbChatRequest {
    pub request_id: String,
    pub provider: Option<String>,
    pub model: String,
    pub messages: Vec<FrbChatMessage>,
    pub temperature: Option<f32>,
    pub top_p: Option<f32>,
    pub max_output_tokens: Option<u32>,
    pub metadata: HashMap<String, String>,
}

impl From<FrbChatRequest> for ChatRequest {
    fn from(value: FrbChatRequest) -> Self {
        ChatRequest {
            request_id: value.request_id,
            provider: value.provider,
            model: value.model,
            messages: value.messages.into_iter().map(Into::into).collect(),
            temperature: value.temperature,
            top_p: value.top_p,
            max_output_tokens: value.max_output_tokens,
            metadata: value.metadata,
        }
    }
}

#[frb]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrbChatUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

impl From<ChatUsage> for FrbChatUsage {
    fn from(value: ChatUsage) -> Self {
        FrbChatUsage {
            prompt_tokens: value.prompt_tokens,
            completion_tokens: value.completion_tokens,
            total_tokens: value.total_tokens,
        }
    }
}

impl From<FrbChatUsage> for ChatUsage {
    fn from(value: FrbChatUsage) -> Self {
        ChatUsage {
            prompt_tokens: value.prompt_tokens,
            completion_tokens: value.completion_tokens,
            total_tokens: value.total_tokens,
        }
    }
}

#[frb]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrbChatResponse {
    pub request_id: String,
    pub provider_id: String,
    pub model: String,
    pub output_text: String,
    pub usage: FrbChatUsage,
}

impl From<ChatResponse> for FrbChatResponse {
    fn from(value: ChatResponse) -> Self {
        FrbChatResponse {
            request_id: value.request_id,
            provider_id: value.provider_id,
            model: value.model,
            output_text: value.output_text,
            usage: value.usage.into(),
        }
    }
}

#[frb]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrbChatDelta {
    pub request_id: String,
    pub provider_id: String,
    pub content: String,
}

impl From<ChatDelta> for FrbChatDelta {
    fn from(value: ChatDelta) -> Self {
        FrbChatDelta {
            request_id: value.request_id,
            provider_id: value.provider_id,
            content: value.content,
        }
    }
}

#[frb]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum FrbChatEvent {
    Delta(FrbChatDelta),
    Usage(FrbChatUsage),
    Completed(FrbChatResponse),
    Error(String),
    Cancelled,
}

impl From<ChatEvent> for FrbChatEvent {
    fn from(value: ChatEvent) -> Self {
        match value {
            ChatEvent::Delta(delta) => FrbChatEvent::Delta(delta.into()),
            ChatEvent::Usage(usage) => FrbChatEvent::Usage(usage.into()),
            ChatEvent::Completed(resp) => FrbChatEvent::Completed(resp.into()),
            ChatEvent::Error { message, .. } => FrbChatEvent::Error(message),
            ChatEvent::Cancelled { .. } => FrbChatEvent::Cancelled,
        }
    }
}
