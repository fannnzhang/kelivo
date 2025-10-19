use std::sync::Arc;

use flutter_rust_bridge::for_generated::DcoCodec;
use flutter_rust_bridge::{frb, Rust2DartSendError};
use futures::StreamExt;
use kelivo_core::llm::{
    mock::MockProvider,
    openai::OpenAiAdapter,
    ChatRequest, LlmProvider, LlmService,
};
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

fn join_endpoint(base_url: &str, chat_path: &str) -> String {
    let base = base_url.trim_end_matches('/');
    let path = if chat_path.starts_with('/') {
        &chat_path[1..]
    } else {
        chat_path
    };
    format!("{}/{}", base, path)
}

fn build_provider_from_request(
    req: &ChatRequest,
) -> Result<(String, Arc<dyn LlmProvider>), String> {
    let meta = &req.metadata;
    let provider_id = req
        .provider
        .clone()
        .or_else(|| meta.get("provider_id").cloned())
        .unwrap_or_else(|| "openai".to_string());

    let base_url = meta
        .get("base_url")
        .cloned()
        .ok_or_else(|| "missing required metadata: base_url".to_string())?;
    let chat_path = meta
        .get("chat_path")
        .cloned()
        .unwrap_or_else(|| "/chat/completions".to_string());
    let api_key = meta
        .get("api_key")
        .cloned()
        .ok_or_else(|| "missing required metadata: api_key".to_string())?;

    let endpoint = join_endpoint(&base_url, &chat_path);
    let adapter = OpenAiAdapter::with_provider(provider_id.clone(), endpoint, api_key)
        .map_err(|err| err.message().to_string())?;
    Ok((provider_id, Arc::new(adapter) as Arc<dyn LlmProvider>))
}

#[frb]
pub async fn llm_chat(request: FrbChatRequest) -> Result<FrbChatResponse, String> {
    let core_request: ChatRequest = request.into();
    // Build provider from request metadata and register as default for this call
    let (provider_id, provider) = build_provider_from_request(&core_request)?;
    let service = llm_service();
    service.register_provider(&provider_id, provider);
    service.set_default_provider(&provider_id);

    let response = service
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

    let (provider_id, provider) = build_provider_from_request(&core_request)?;
    let service = llm_service();
    service.register_provider(&provider_id, provider);
    service.set_default_provider(&provider_id);

    let mut stream = service.chat_stream(core_request);
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

#[cfg(test)]
mod tests {
    use super::*;
    use kelivo_core::llm::{ChatMessage, ChatRole, MessageContent};
    use std::collections::HashMap;

    fn sample_request_with_meta(meta: HashMap<String, String>) -> ChatRequest {
        ChatRequest {
            request_id: "req-test".into(),
            provider: None,
            model: "gpt-4o-mini".into(),
            messages: vec![ChatMessage {
                role: ChatRole::User,
                content: vec![MessageContent::Text("hello".into())],
            }],
            temperature: None,
            top_p: None,
            max_output_tokens: None,
            metadata: meta,
        }
    }

    #[test]
    fn join_endpoint_trims_and_joins() {
        assert_eq!(
            join_endpoint("https://api.example.com/", "/chat/completions"),
            "https://api.example.com/chat/completions"
        );
        assert_eq!(
            join_endpoint("https://api.example.com", "responses"),
            "https://api.example.com/responses"
        );
        assert_eq!(
            join_endpoint("https://api.example.com/", "responses"),
            "https://api.example.com/responses"
        );
    }

    #[test]
    fn build_provider_uses_default_path_and_metadata_provider() {
        let mut meta = HashMap::new();
        meta.insert("base_url".into(), "https://openai.example.com".into());
        meta.insert("api_key".into(), "sk-test".into());
        meta.insert("provider_id".into(), "openrouter".into());

        let req = sample_request_with_meta(meta);
        let (provider_id, provider) = build_provider_from_request(&req).expect("provider");
        // provider_id should come from metadata when request.provider is None
        assert_eq!(provider_id, "openrouter");
        // provider.id() must equal returned id
        assert_eq!(provider.id(), provider_id);
    }

    #[test]
    fn build_provider_request_provider_overrides_metadata() {
        let mut meta = HashMap::new();
        meta.insert("base_url".into(), "https://openai.example.com/".into());
        meta.insert("api_key".into(), "sk-test".into());
        meta.insert("provider_id".into(), "openrouter".into());

        let mut req = sample_request_with_meta(meta);
        req.provider = Some("anthropic".into());
        let (provider_id, provider) = build_provider_from_request(&req).expect("provider");
        assert_eq!(provider_id, "anthropic");
        assert_eq!(provider.id(), provider_id);
    }

    #[test]
    fn build_provider_missing_base_url_errors() {
        let mut meta = HashMap::new();
        meta.insert("api_key".into(), "sk-test".into());
        let req = sample_request_with_meta(meta);
        let res = build_provider_from_request(&req);
        assert!(res.is_err());
        let err = res.err().unwrap();
        assert!(err.contains("missing required metadata: base_url"));
    }

    #[test]
    fn build_provider_missing_api_key_errors() {
        let mut meta = HashMap::new();
        meta.insert("base_url".into(), "https://openai.example.com".into());
        let req = sample_request_with_meta(meta);
        let res = build_provider_from_request(&req);
        assert!(res.is_err());
        let err = res.err().unwrap();
        assert!(err.contains("missing required metadata: api_key"));
    }
}
