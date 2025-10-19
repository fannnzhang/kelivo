use async_trait::async_trait;
use tokio_util::sync::CancellationToken;

use super::openai::OpenAiAdapter;
use super::{ChatRequest, ChatResponse, ChatStream, LlmProvider};
use crate::net::HttpClient;
use crate::KelivoResult;

#[derive(Clone)]
pub struct AnthropicAdapter {
    inner: OpenAiAdapter,
}

impl AnthropicAdapter {
    pub fn new(endpoint: impl Into<String>, api_key: impl Into<String>) -> KelivoResult<Self> {
        Ok(Self {
            inner: OpenAiAdapter::with_provider("anthropic", endpoint, api_key)?,
        })
    }

    pub fn with_client(
        client: HttpClient,
        endpoint: impl Into<String>,
        api_key: impl Into<String>,
    ) -> Self {
        Self {
            inner: OpenAiAdapter::with_client_and_provider(client, "anthropic", endpoint, api_key),
        }
    }
}

#[async_trait]
impl LlmProvider for AnthropicAdapter {
    fn id(&self) -> &str {
        self.inner.id()
    }

    async fn chat(
        &self,
        request: ChatRequest,
        cancel: CancellationToken,
    ) -> KelivoResult<ChatResponse> {
        self.inner.chat(request, cancel).await
    }

    fn chat_stream(&self, request: ChatRequest, cancel: CancellationToken) -> ChatStream {
        self.inner.chat_stream(request, cancel)
    }
}
