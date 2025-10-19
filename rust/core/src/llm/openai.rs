use async_stream::stream;
use async_trait::async_trait;
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use tokio_util::sync::CancellationToken;

use super::{
    ChatDelta, ChatEvent, ChatMessage, ChatRequest, ChatResponse, ChatRole, ChatStream, ChatUsage,
    LlmProvider,
};
use crate::net::HttpClient;
use crate::{KelivoError, KelivoResult};

#[derive(Clone)]
pub struct OpenAiAdapter {
    client: HttpClient,
    endpoint: String,
    api_key: String,
    organization: Option<String>,
    provider_id: String,
}

impl OpenAiAdapter {
    pub fn new(endpoint: impl Into<String>, api_key: impl Into<String>) -> KelivoResult<Self> {
        Self::with_provider("openai", endpoint, api_key)
    }

    pub fn with_provider(
        provider_id: impl Into<String>,
        endpoint: impl Into<String>,
        api_key: impl Into<String>,
    ) -> KelivoResult<Self> {
        Ok(Self {
            client: HttpClient::with_default()?,
            endpoint: endpoint.into(),
            api_key: api_key.into(),
            organization: None,
            provider_id: provider_id.into(),
        })
    }

    pub fn with_client(
        client: HttpClient,
        endpoint: impl Into<String>,
        api_key: impl Into<String>,
    ) -> Self {
        Self::with_client_and_provider(client, "openai", endpoint, api_key)
    }

    pub fn with_client_and_provider(
        client: HttpClient,
        provider_id: impl Into<String>,
        endpoint: impl Into<String>,
        api_key: impl Into<String>,
    ) -> Self {
        Self {
            client,
            endpoint: endpoint.into(),
            api_key: api_key.into(),
            organization: None,
            provider_id: provider_id.into(),
        }
    }

    pub fn with_organization(mut self, value: impl Into<String>) -> Self {
        self.organization = Some(value.into());
        self
    }

    fn headers(&self) -> HashMap<String, String> {
        let mut headers = HashMap::new();
        headers.insert(
            "Authorization".to_string(),
            format!("Bearer {}", self.api_key),
        );
        if let Some(ref org) = self.organization {
            headers.insert("OpenAI-Organization".to_string(), org.clone());
        }
        headers
    }

    fn map_messages(&self, messages: &[ChatMessage]) -> Vec<OpenAiMessage> {
        messages
            .iter()
            .map(|msg| OpenAiMessage {
                role: map_role(&msg.role).to_string(),
                content: msg
                    .content
                    .iter()
                    .map(|part| match part {
                        super::MessageContent::Text(text) => text.as_str(),
                    })
                    .collect::<Vec<_>>()
                    .join("\n"),
            })
            .collect()
    }
}

fn map_role(role: &ChatRole) -> &'static str {
    match role {
        ChatRole::User => "user",
        ChatRole::Assistant => "assistant",
        ChatRole::System => "system",
        ChatRole::Tool => "tool",
    }
}

#[async_trait]
impl LlmProvider for OpenAiAdapter {
    fn id(&self) -> &str {
        &self.provider_id
    }

    async fn chat(
        &self,
        request: ChatRequest,
        cancel: CancellationToken,
    ) -> KelivoResult<ChatResponse> {
        if cancel.is_cancelled() {
            return Err(KelivoError::new("request cancelled"));
        }

        let payload = json!({
            "model": request.model,
            "messages": self.map_messages(&request.messages),
            "temperature": request.temperature,
            "top_p": request.top_p,
            "max_tokens": request.max_output_tokens,
            "stream": false,
        });

        let value = self
            .client
            .json_post(&self.endpoint, &self.headers(), &payload)
            .await
            .map_err(|err| KelivoError::new(format!("openai chat failed: {}", err.message())))?;
        let response: OpenAiChatCompletion = serde_json::from_value(value).map_err(|err| {
            KelivoError::new(format!("failed to decode openai chat response: {err}"))
        })?;

        let choice = response
            .choices
            .into_iter()
            .next()
            .ok_or_else(|| KelivoError::new("openai response missing choices"))?;
        let output_text = choice.message.map(|msg| msg.content).unwrap_or_default();
        let usage = response.usage.unwrap_or_default();

        Ok(ChatResponse {
            request_id: request.request_id,
            provider_id: self.id().to_string(),
            model: request.model,
            output_text,
            usage: usage.into(),
        })
    }

    fn chat_stream(&self, request: ChatRequest, cancel: CancellationToken) -> ChatStream {
        let headers = self.headers();
        let client = self.client.clone();
        let endpoint = self.endpoint.clone();
        let provider_id = self.provider_id.clone();
        let payload = json!({
            "model": request.model,
            "messages": self.map_messages(&request.messages),
            "temperature": request.temperature,
            "top_p": request.top_p,
            "max_tokens": request.max_output_tokens,
            "stream": true,
        });
        let request_id = request.request_id.clone();
        stream! {
            let mut aggregated = String::new();
            let mut usage = ChatUsage::default();
            match client.stream_sse(&endpoint, &headers, Some(payload)).await {
                Ok(mut sse) => {
                    let mut stream_cancelled = false;
                    while let Some(event) = sse.next().await {
                        match event {
                            Ok(chunk) => {
                                if chunk.trim() == "[DONE]" {
                                    break;
                                }
                                match serde_json::from_str::<OpenAiStreamChunk>(&chunk) {
                                    Ok(chunk) => {
                                        if let Some(delta) = chunk.choices.into_iter().next() {
                                            if let Some(content) = delta.delta.and_then(|d| d.content) {
                                                if cancel.is_cancelled() {
                                                    stream_cancelled = true;
                                                    break;
                                                }
                                                aggregated.push_str(&content);
                                                yield ChatEvent::Delta(ChatDelta {
                                                    request_id: request_id.clone(),
                                                    provider_id: provider_id.clone(),
                                                    content,
                                                });
                                            }
                                            if let Some(chunk_usage) = delta.usage {
                                                usage = usage.combine(chunk_usage.into());
                                            }
                                        }
                                    }
                                    Err(err) => {
                                        yield ChatEvent::Error {
                                            request_id: request_id.clone(),
                                            message: format!("failed to parse stream chunk: {err}"),
                                        };
                                        stream_cancelled = true;
                                        break;
                                    }
                                }
                            }
                            Err(err) => {
                                yield ChatEvent::Error {
                                    request_id: request_id.clone(),
                                    message: err.message().to_string(),
                                };
                                stream_cancelled = true;
                                break;
                            }
                        }
                    }

                    if !stream_cancelled && !cancel.is_cancelled() {
                        yield ChatEvent::Usage(usage.clone());
                        yield ChatEvent::Completed(ChatResponse {
                            request_id,
                            provider_id,
                            model: request.model,
                            output_text: aggregated,
                            usage,
                        });
                    }
                }
                Err(err) => {
                    yield ChatEvent::Error {
                        request_id,
                        message: err.message().to_string(),
                    };
                }
            }
        }
        .boxed()
    }
}

#[derive(Debug, Deserialize)]
struct OpenAiChatCompletion {
    choices: Vec<OpenAiChoice>,
    usage: Option<OpenAiUsage>,
}

#[derive(Debug, Deserialize)]
struct OpenAiChoice {
    message: Option<OpenAiMessage>,
}

#[derive(Debug, Deserialize)]
struct OpenAiUsage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}

impl Default for OpenAiUsage {
    fn default() -> Self {
        Self {
            prompt_tokens: 0,
            completion_tokens: 0,
            total_tokens: 0,
        }
    }
}

impl From<OpenAiUsage> for ChatUsage {
    fn from(value: OpenAiUsage) -> Self {
        Self {
            prompt_tokens: value.prompt_tokens,
            completion_tokens: value.completion_tokens,
            total_tokens: value.total_tokens,
        }
    }
}

impl ChatUsage {
    fn combine(mut self, other: ChatUsage) -> Self {
        self.prompt_tokens += other.prompt_tokens;
        self.completion_tokens += other.completion_tokens;
        self.total_tokens += other.total_tokens;
        self
    }
}

#[derive(Debug, Deserialize)]
struct OpenAiStreamChunk {
    choices: Vec<OpenAiStreamChoice>,
}

#[derive(Debug, Deserialize)]
struct OpenAiStreamChoice {
    delta: Option<OpenAiDelta>,
    usage: Option<OpenAiUsage>,
}

#[derive(Debug, Deserialize)]
struct OpenAiDelta {
    content: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct OpenAiMessage {
    role: String,
    content: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn maps_messages() {
        let adapter = OpenAiAdapter::new("https://example.com", "token").unwrap();
        let request = ChatRequest {
            request_id: "req".into(),
            provider: None,
            model: "gpt-4".into(),
            messages: vec![ChatMessage {
                role: ChatRole::User,
                content: vec![super::super::MessageContent::Text("Hello".into())],
            }],
            temperature: Some(0.5),
            top_p: None,
            max_output_tokens: None,
            metadata: HashMap::new(),
        };

        let mapped = adapter.map_messages(&request.messages);
        assert_eq!(mapped.len(), 1);
        assert_eq!(mapped[0].role, "user");
        assert_eq!(mapped[0].content, "Hello");
    }
}
