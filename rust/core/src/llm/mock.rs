use async_stream::stream;
use async_trait::async_trait;
use futures::StreamExt;
use tokio_util::sync::CancellationToken;

use super::{ChatDelta, ChatEvent, ChatRequest, ChatResponse, ChatStream, ChatUsage, LlmProvider};
use crate::{KelivoError, KelivoResult};

#[derive(Debug)]
pub struct MockProvider {
    id: String,
}

impl MockProvider {
    pub fn new(id: impl Into<String>) -> Self {
        Self { id: id.into() }
    }

    fn build_response(&self, request: &ChatRequest) -> (String, ChatUsage) {
        let prompt = request.last_user_message().unwrap_or("");
        let completion = if prompt.is_empty() {
            format!("{}: (no input)", self.id)
        } else {
            format!("{} echo: {}", self.id, prompt)
        };
        let prompt_tokens = prompt.split_whitespace().count() as u32;
        let completion_tokens = completion.split_whitespace().count() as u32;
        let usage = ChatUsage {
            prompt_tokens,
            completion_tokens,
            total_tokens: prompt_tokens + completion_tokens,
        };
        (completion, usage)
    }
}

#[async_trait]
impl LlmProvider for MockProvider {
    fn id(&self) -> &str {
        &self.id
    }

    async fn chat(
        &self,
        request: ChatRequest,
        cancel: CancellationToken,
    ) -> KelivoResult<ChatResponse> {
        if cancel.is_cancelled() {
            return Err(KelivoError::new("request cancelled"));
        }
        let (output_text, usage) = self.build_response(&request);
        Ok(ChatResponse {
            request_id: request.request_id,
            provider_id: self.id.clone(),
            model: request.model,
            output_text,
            usage,
        })
    }

    fn chat_stream(&self, request: ChatRequest, cancel: CancellationToken) -> ChatStream {
        let provider_id = self.id.clone();
        let (output_text, usage) = self.build_response(&request);
        stream! {
            let request_id = request.request_id.clone();
            for chunk in output_text.split_whitespace() {
                if cancel.is_cancelled() {
                    return;
                }
                yield ChatEvent::Delta(ChatDelta {
                    request_id: request_id.clone(),
                    provider_id: provider_id.clone(),
                    content: format!("{chunk} "),
                });
            }
            if !cancel.is_cancelled() {
                yield ChatEvent::Usage(usage.clone());
                yield ChatEvent::Completed(ChatResponse {
                    request_id,
                    provider_id,
                    model: request.model,
                    output_text,
                    usage,
                });
            }
        }
        .boxed()
    }
}
