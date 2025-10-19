use std::sync::Arc;

use flutter_rust_bridge::for_generated::DcoCodec;
use flutter_rust_bridge::{frb, Rust2DartSendError};
use futures::StreamExt;
use kelivo_core::llm::{mock::MockProvider, ChatRequest, LlmProvider, LlmService};
use once_cell::sync::Lazy;
use serde_json;

use crate::frb_generated::StreamSink;

use super::llm_types::{FrbChatEvent, FrbChatRequest, FrbChatResponse};

static LLM_SERVICE: Lazy<Arc<LlmService>> = Lazy::new(|| {
    let service = LlmService::new("mock");
    let provider: Arc<dyn LlmProvider> = Arc::new(MockProvider::new("mock"));
    service.register_provider("mock", provider);
    Arc::new(service)
});

fn llm_service() -> &'static Arc<LlmService> {
    &LLM_SERVICE
}

#[frb]
pub async fn llm_chat(request: FrbChatRequest) -> Result<FrbChatResponse, String> {
    let core_request: ChatRequest = request.into();
    let response = llm_service()
        .chat(core_request)
        .await
        .map_err(|err| err.message().to_string())?;
    Ok(response.into())
}

#[frb]
pub async fn llm_chat_stream(
    request: FrbChatRequest,
    sink: StreamSink<String, DcoCodec>,
) -> Result<(), String> {
    let core_request: ChatRequest = request.into();
    let mut stream = llm_service().chat_stream(core_request);
    while let Some(event) = stream.next().await {
        let frb_event: FrbChatEvent = event.into();
        let payload =
            serde_json::to_string(&frb_event).map_err(|err| format!("serialize error: {err}"))?;
        sink.add(payload).map_err(stream_error_to_string)?;
    }
    Ok(())
}

#[frb]
pub async fn llm_cancel(request_id: String) -> bool {
    llm_service().cancel(&request_id).await
}

fn stream_error_to_string(err: Rust2DartSendError) -> String {
    format!("failed to deliver stream event: {err}")
}
