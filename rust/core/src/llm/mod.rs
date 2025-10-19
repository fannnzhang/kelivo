use std::collections::HashMap;
use std::pin::Pin;
use std::sync::Arc;

use async_stream::stream;
use async_trait::async_trait;
use futures::{Stream, StreamExt};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;

use crate::{KelivoError, KelivoResult};

pub mod anthropic;
pub mod google;
pub mod mock;
pub mod openai;
pub mod openrouter;

pub type ChatStream = Pin<Box<dyn Stream<Item = ChatEvent> + Send>>;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ChatRole {
    User,
    Assistant,
    System,
    Tool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ChatMessage {
    pub role: ChatRole,
    pub content: Vec<MessageContent>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", content = "data")]
pub enum MessageContent {
    Text(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct ChatUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ChatResponse {
    pub request_id: String,
    pub provider_id: String,
    pub model: String,
    pub output_text: String,
    pub usage: ChatUsage,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ChatDelta {
    pub request_id: String,
    pub provider_id: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ChatEvent {
    Delta(ChatDelta),
    Usage(ChatUsage),
    Completed(ChatResponse),
    Error { request_id: String, message: String },
    Cancelled { request_id: String },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ChatRequest {
    pub request_id: String,
    pub provider: Option<String>,
    pub model: String,
    pub messages: Vec<ChatMessage>,
    pub temperature: Option<f32>,
    pub top_p: Option<f32>,
    pub max_output_tokens: Option<u32>,
    pub metadata: HashMap<String, String>,
}

impl ChatRequest {
    pub fn last_user_message(&self) -> Option<&str> {
        self.messages
            .iter()
            .rev()
            .find(|msg| msg.role == ChatRole::User)
            .and_then(|msg| {
                msg.content.iter().rev().find_map(|content| match content {
                    MessageContent::Text(text) => Some(text.as_str()),
                })
            })
    }
}

#[async_trait]
pub trait LlmProvider: Send + Sync {
    fn id(&self) -> &str;
    async fn chat(
        &self,
        request: ChatRequest,
        cancel: CancellationToken,
    ) -> KelivoResult<ChatResponse>;
    fn chat_stream(&self, request: ChatRequest, cancel: CancellationToken) -> ChatStream;
}

#[derive(Clone)]
pub struct LlmService {
    inner: Arc<LlmServiceInner>,
}

struct LlmServiceInner {
    providers: RwLock<HashMap<String, Arc<dyn LlmProvider>>>,
    default_provider: RwLock<String>,
    inflight: Mutex<HashMap<String, CancellationToken>>,
}

impl LlmService {
    pub fn new(default_provider: impl Into<String>) -> Self {
        Self {
            inner: Arc::new(LlmServiceInner {
                providers: RwLock::new(HashMap::new()),
                default_provider: RwLock::new(default_provider.into()),
                inflight: Mutex::new(HashMap::new()),
            }),
        }
    }

    pub fn register_provider(
        &self,
        provider_id: impl Into<String>,
        provider: Arc<dyn LlmProvider>,
    ) {
        self.inner
            .providers
            .write()
            .insert(provider_id.into(), provider);
    }

    pub fn set_default_provider(&self, provider_id: impl Into<String>) {
        *self.inner.default_provider.write() = provider_id.into();
    }

    fn provider(&self, id: Option<&str>) -> KelivoResult<Arc<dyn LlmProvider>> {
        let default = self.inner.default_provider.read().clone();
        let lookup = id.unwrap_or(&default);
        self.inner
            .providers
            .read()
            .get(lookup)
            .cloned()
            .ok_or_else(|| KelivoError::new(format!("unknown provider: {lookup}")))
    }

    async fn track(&self, request_id: &str, cancel: CancellationToken) {
        let mut inflight = self.inner.inflight.lock().await;
        inflight.insert(request_id.to_string(), cancel);
    }

    async fn untrack(&self, request_id: &str) {
        let mut inflight = self.inner.inflight.lock().await;
        inflight.remove(request_id);
    }

    pub async fn chat(&self, request: ChatRequest) -> KelivoResult<ChatResponse> {
        let provider = self.provider(request.provider.as_deref())?;
        let request_id = request.request_id.clone();
        let cancel = CancellationToken::new();
        self.track(&request_id, cancel.clone()).await;
        let result = provider.chat(request, cancel.clone()).await;
        self.untrack(&request_id).await;
        result
    }

    pub fn chat_stream(&self, request: ChatRequest) -> ChatStream {
        let service = self.clone();
        let provider_hint = request.provider.clone();
        stream! {
            let request_id = request.request_id.clone();
            let provider = match service.provider(provider_hint.as_deref()) {
                Ok(provider) => provider,
                Err(err) => {
                    yield ChatEvent::Error { request_id, message: err.message().to_string() };
                    return;
                }
            };

            let cancel = CancellationToken::new();
            service.track(&request_id, cancel.clone()).await;
            let mut cancelled = false;
            let mut inner = provider.chat_stream(request.clone(), cancel.clone());
            while let Some(event) = inner.next().await {
                if cancel.is_cancelled() {
                    yield ChatEvent::Cancelled { request_id: request_id.clone() };
                    cancelled = true;
                    break;
                }
                yield event;
            }
            if !cancelled && cancel.is_cancelled() {
                yield ChatEvent::Cancelled { request_id: request_id.clone() };
            }
            service.untrack(&request_id).await;
        }
        .boxed()
    }

    pub async fn cancel(&self, request_id: &str) -> bool {
        let mut inflight = self.inner.inflight.lock().await;
        if let Some(token) = inflight.remove(request_id) {
            token.cancel();
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::mock::MockProvider;
    use super::*;
    use std::collections::HashMap;
    use std::sync::Arc;

    fn sample_messages() -> Vec<ChatMessage> {
        vec![ChatMessage {
            role: ChatRole::User,
            content: vec![MessageContent::Text("Hello from test".into())],
        }]
    }

    #[tokio::test]
    async fn mock_provider_chat() {
        let service = LlmService::new("mock");
        service.register_provider("mock", Arc::new(MockProvider::new("mock")));

        let request = ChatRequest {
            request_id: "req-1".into(),
            provider: None,
            model: "mock-mini".into(),
            messages: sample_messages(),
            temperature: Some(0.2),
            top_p: None,
            max_output_tokens: Some(32),
            metadata: HashMap::new(),
        };

        let response = service.chat(request).await.expect("chat result");
        assert!(response.output_text.contains("Hello"));
        assert_eq!(response.provider_id, "mock");
    }

    #[tokio::test]
    async fn mock_provider_stream_and_cancel() {
        let service = LlmService::new("mock");
        service.register_provider("mock", Arc::new(MockProvider::new("mock")));

        let request = ChatRequest {
            request_id: "req-stream".into(),
            provider: Some("mock".into()),
            model: "mock-mini".into(),
            messages: sample_messages(),
            temperature: None,
            top_p: None,
            max_output_tokens: None,
            metadata: HashMap::new(),
        };

        let mut stream = service.chat_stream(request.clone());
        let mut deltas = String::new();
        while let Some(event) = stream.next().await {
            match event {
                ChatEvent::Delta(delta) => {
                    deltas.push_str(&delta.content);
                    if deltas.len() >= 5 {
                        service.cancel(&request.request_id).await;
                    }
                }
                ChatEvent::Cancelled { request_id } => {
                    assert_eq!(request_id, request.request_id);
                    break;
                }
                _ => {}
            }
        }
    }

    #[tokio::test]
    async fn cancel_nonexistent_request_returns_false() {
        let service = LlmService::new("mock");
        assert!(!service.cancel("missing").await);
    }
}
