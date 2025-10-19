use std::collections::HashMap;
use std::pin::Pin;
use std::time::Duration;

use async_stream::stream;
use futures::Stream;
use futures::StreamExt;
use reqwest::{Client, Method, RequestBuilder, Response, StatusCode};
use serde_json::Value;

use crate::{KelivoError, KelivoResult};

#[derive(Debug, Clone)]
pub struct HttpClientConfig {
    pub user_agent: Option<String>,
    pub timeout: Duration,
    pub connect_timeout: Duration,
}

impl Default for HttpClientConfig {
    fn default() -> Self {
        Self {
            user_agent: Some("Kelivo-Rust/1.0".to_string()),
            timeout: Duration::from_secs(30),
            connect_timeout: Duration::from_secs(10),
        }
    }
}

#[derive(Clone)]
pub struct HttpClient {
    client: Client,
    user_agent: Option<String>,
}

impl HttpClient {
    pub fn new(config: HttpClientConfig) -> KelivoResult<Self> {
        let mut builder = Client::builder()
            .timeout(config.timeout)
            .connect_timeout(config.connect_timeout);

        if let Some(ref agent) = config.user_agent {
            builder = builder.user_agent(agent);
        }

        let client = builder
            .build()
            .map_err(|err| KelivoError::new(format!("failed to build http client: {err}")))?;

        Ok(Self {
            client,
            user_agent: config.user_agent,
        })
    }

    pub fn with_default() -> KelivoResult<Self> {
        Self::new(HttpClientConfig::default())
    }

    pub fn request(&self, method: Method, url: &str) -> RequestBuilder {
        let mut builder = self.client.request(method, url);
        if let Some(ref agent) = self.user_agent {
            builder = builder.header("User-Agent", agent);
        }
        builder
    }

    pub async fn json_post(
        &self,
        url: &str,
        headers: &HashMap<String, String>,
        payload: &Value,
    ) -> KelivoResult<Value> {
        let mut builder = self.request(Method::POST, url).json(payload);
        for (key, value) in headers {
            builder = builder.header(key, value);
        }

        let response = builder
            .send()
            .await
            .map_err(|err| KelivoError::new(format!("http post failed: {err}")))?;

        Self::map_json_response(response).await
    }

    pub async fn json_get(
        &self,
        url: &str,
        headers: &HashMap<String, String>,
    ) -> KelivoResult<Value> {
        let mut builder = self.request(Method::GET, url);
        for (key, value) in headers {
            builder = builder.header(key, value);
        }

        let response = builder
            .send()
            .await
            .map_err(|err| KelivoError::new(format!("http get failed: {err}")))?;

        Self::map_json_response(response).await
    }

    async fn map_json_response(response: Response) -> KelivoResult<Value> {
        let status = response.status();
        let text = response
            .text()
            .await
            .map_err(|err| KelivoError::new(format!("failed to read body: {err}")))?;

        if !status.is_success() {
            return Err(KelivoError::new(format!(
                "http error {}: {}",
                status.as_u16(),
                text
            )));
        }

        serde_json::from_str(&text)
            .map_err(|err| KelivoError::new(format!("failed to parse json: {err}")))
    }

    pub async fn stream_sse(
        &self,
        url: &str,
        headers: &HashMap<String, String>,
        body: Option<Value>,
    ) -> KelivoResult<SseStream> {
        let mut builder = self.request(Method::POST, url);
        for (key, value) in headers {
            builder = builder.header(key, value);
        }

        if let Some(payload) = body {
            builder = builder.json(&payload);
        }

        let response = builder
            .send()
            .await
            .map_err(|err| KelivoError::new(format!("failed to open SSE stream: {err}")))?;

        if response.status() != StatusCode::OK {
            let status = response.status();
            let text = response
                .text()
                .await
                .unwrap_or_else(|_| "<unreadable>".to_string());
            return Err(KelivoError::new(format!(
                "SSE request failed: status={} body={}",
                status.as_u16(),
                text
            )));
        }

        Ok(SseStream::new(response))
    }
}

pub struct SseStream {
    inner: Pin<Box<dyn Stream<Item = KelivoResult<String>> + Send>>,
}

impl SseStream {
    fn new(response: Response) -> Self {
        let stream = response.bytes_stream();
        let mut buffer = Vec::new();
        let inner = stream! {
            futures::pin_mut!(stream);
            while let Some(chunk) = stream.next().await {
                let chunk = match chunk {
                    Ok(bytes) => bytes,
                    Err(err) => {
                        yield Err(KelivoError::new(format!("sse chunk error: {err}")));
                        break;
                    }
                };

                buffer.extend_from_slice(&chunk);

                while let Some(pos) = buffer.iter().position(|b| *b == b'\n') {
                    let line = buffer.drain(..=pos).collect::<Vec<u8>>();
                    if line == b"\n" || line == b"\r\n" {
                        continue;
                    }

                    if let Some(payload) = parse_sse_line(&line) {
                        yield Ok(payload);
                    }
                }
            }
        };

        Self { inner: Box::pin(inner) }
    }
}

impl Stream for SseStream {
    type Item = KelivoResult<String>;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        self.inner.as_mut().poll_next(cx)
    }
}

fn parse_sse_line(line: &[u8]) -> Option<String> {
    let trimmed = line.trim_ascii();
    if trimmed.starts_with(b"data:") {
        let payload = &trimmed[5..];
        let payload = payload.strip_prefix(b" ").unwrap_or(payload);
        String::from_utf8(payload.to_vec()).ok()
    } else {
        None
    }
}



#[cfg(test)]
mod tests {
    use super::*;
    use httpmock::Method::POST;
    use httpmock::MockServer;
    use serde_json::json;
    use std::collections::HashMap;
    use tokio::runtime::Runtime;

    #[test]
    fn posts_json() {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let server = MockServer::start_async().await;
            let mock = server
                .mock(|when, then| {
                    when.method(POST).path("/json");
                    then.status(200)
                        .header("Content-Type", "application/json")
                        .body(r#"{"ok":true}"#);
                });

            let client = HttpClient::with_default().unwrap();
            let response = client
                .json_post(
                    &format!("{}/json", server.base_url()),
                    &HashMap::new(),
                    &json!({"foo":"bar"}),
                )
                .await
                .expect("json response");
            assert_eq!(response["ok"], json!(true));

            mock.assert_async().await;
        });
    }

    #[test]
    fn parses_sse_lines() {
        let input = b"data: hello\n\n";
        let event = parse_sse_line(input);
        assert_eq!(event, Some("hello".to_string()));
    }
}
