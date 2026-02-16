use crate::provider::types::*;
use reqwest::Client;
use serde_json::Value;
use tracing::debug;

const ANTHROPIC_API_URL: &str = "https://api.anthropic.com/v1/messages";
const ANTHROPIC_API_VERSION: &str = "2023-06-01";

/// Anthropic Claude provider implementation.
pub struct AnthropicProvider {
    client: Client,
    api_key: String,
    base_url: String,
}

impl AnthropicProvider {
    pub fn new(api_key: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
            base_url: ANTHROPIC_API_URL.to_string(),
        }
    }

    pub fn with_base_url(mut self, url: String) -> Self {
        self.base_url = url;
        self
    }

    /// Resolve API key from environment or config.
    pub fn api_key_from_env() -> Option<String> {
        std::env::var("ANTHROPIC_API_KEY").ok()
    }

    fn build_request_body(&self, request: &CompletionRequest) -> Value {
        let mut messages: Vec<Value> = Vec::new();

        for msg in &request.messages {
            let role = match msg.role {
                MessageRole::User => "user",
                MessageRole::Assistant => "assistant",
                MessageRole::Tool => "user", // Tool results go as user messages
                MessageRole::System => continue, // System handled separately
            };

            let content = match &msg.content {
                MessageContent::Text(text) => Value::String(text.clone()),
                MessageContent::Blocks(blocks) => {
                    let block_values: Vec<Value> = blocks.iter().map(|b| {
                        serde_json::to_value(b).unwrap_or(Value::Null)
                    }).collect();
                    Value::Array(block_values)
                }
            };

            messages.push(serde_json::json!({
                "role": role,
                "content": content,
            }));
        }

        let mut body = serde_json::json!({
            "model": request.model,
            "messages": messages,
            "max_tokens": request.max_tokens,
        });

        if let Some(system) = &request.system {
            body["system"] = Value::String(system.clone());
        }

        if let Some(temp) = request.temperature {
            body["temperature"] = Value::from(temp);
        }

        if !request.tools.is_empty() {
            let tools: Vec<Value> = request.tools.iter().map(|t| {
                serde_json::json!({
                    "name": t.name,
                    "description": t.description,
                    "input_schema": t.input_schema,
                })
            }).collect();
            body["tools"] = Value::Array(tools);
        }

        if !request.stop_sequences.is_empty() {
            body["stop_sequences"] = Value::Array(
                request.stop_sequences.iter().map(|s| Value::String(s.clone())).collect()
            );
        }

        if request.stream {
            body["stream"] = Value::Bool(true);
        }

        body
    }

    fn parse_response(&self, body: &Value) -> Result<CompletionResponse, ProviderError> {
        let id = body["id"].as_str().unwrap_or("").to_string();
        let model = body["model"].as_str().unwrap_or("").to_string();
        let stop_reason = body["stop_reason"].as_str().map(|s| s.to_string());

        let content = if let Some(content_arr) = body["content"].as_array() {
            content_arr.iter().filter_map(|block| {
                let block_type = block["type"].as_str()?;
                match block_type {
                    "text" => Some(ContentBlock::Text {
                        text: block["text"].as_str().unwrap_or("").to_string(),
                    }),
                    "tool_use" => Some(ContentBlock::ToolUse {
                        id: block["id"].as_str().unwrap_or("").to_string(),
                        name: block["name"].as_str().unwrap_or("").to_string(),
                        input: block["input"].clone(),
                    }),
                    "thinking" => Some(ContentBlock::Thinking {
                        thinking: block["thinking"].as_str().unwrap_or("").to_string(),
                    }),
                    _ => None,
                }
            }).collect()
        } else {
            vec![]
        };

        let usage = Usage {
            input_tokens: body["usage"]["input_tokens"].as_u64().unwrap_or(0),
            output_tokens: body["usage"]["output_tokens"].as_u64().unwrap_or(0),
            cache_creation_input_tokens: body["usage"]["cache_creation_input_tokens"].as_u64().unwrap_or(0),
            cache_read_input_tokens: body["usage"]["cache_read_input_tokens"].as_u64().unwrap_or(0),
        };

        Ok(CompletionResponse {
            id,
            model,
            content,
            stop_reason,
            usage,
        })
    }

    pub fn parse_sse_event(line: &str) -> Option<(String, String)> {
        // SSE format: "event: <type>\ndata: <json>"
        // We accumulate event type and data
        if let Some(event_type) = line.strip_prefix("event: ") {
            return Some(("event".to_string(), event_type.trim().to_string()));
        }
        if let Some(data) = line.strip_prefix("data: ") {
            return Some(("data".to_string(), data.trim().to_string()));
        }
        None
    }
}

#[async_trait::async_trait]
impl Provider for AnthropicProvider {
    async fn complete(&self, request: &CompletionRequest) -> Result<CompletionResponse, ProviderError> {
        let body = self.build_request_body(request);

        debug!("Anthropic request: model={}", request.model);

        let response = self.client
            .post(&self.base_url)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", ANTHROPIC_API_VERSION)
            .header("content-type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| ProviderError::NetworkError(e.to_string()))?;

        let status = response.status().as_u16();
        if status == 401 || status == 403 {
            let text = response.text().await.unwrap_or_default();
            return Err(ProviderError::AuthError(text));
        }
        if status == 429 {
            let retry_after = response.headers()
                .get("retry-after")
                .and_then(|v| v.to_str().ok())
                .and_then(|v| v.parse::<u64>().ok())
                .unwrap_or(60)
                * 1000;
            return Err(ProviderError::RateLimited { retry_after_ms: retry_after });
        }
        if status >= 400 {
            let text = response.text().await.unwrap_or_default();
            return Err(ProviderError::ApiError { status, message: text });
        }

        let resp_body: Value = response.json().await
            .map_err(|e| ProviderError::Other(format!("Failed to parse response: {}", e)))?;

        self.parse_response(&resp_body)
    }

    async fn stream(&self, request: &CompletionRequest) -> Result<
        tokio::sync::mpsc::Receiver<StreamEvent>,
        ProviderError,
    > {
        let mut stream_request = request.clone();
        stream_request.stream = true;
        let body = self.build_request_body(&stream_request);

        debug!("Anthropic stream request: model={}", request.model);

        let response = self.client
            .post(&self.base_url)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", ANTHROPIC_API_VERSION)
            .header("content-type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| ProviderError::NetworkError(e.to_string()))?;

        let status = response.status().as_u16();
        if status >= 400 {
            let text = response.text().await.unwrap_or_default();
            return Err(ProviderError::ApiError { status, message: text });
        }

        let (tx, rx) = tokio::sync::mpsc::channel(100);

        tokio::spawn(async move {
            use futures::StreamExt;
            let mut stream = response.bytes_stream();
            let mut buffer = String::new();
            let mut current_event_type = String::new();

            while let Some(chunk_result) = stream.next().await {
                let chunk = match chunk_result {
                    Ok(c) => c,
                    Err(e) => {
                        let _ = tx.send(StreamEvent::Error { message: e.to_string() }).await;
                        break;
                    }
                };

                buffer.push_str(&String::from_utf8_lossy(&chunk));

                // Process complete lines
                while let Some(newline_pos) = buffer.find('\n') {
                    let line = buffer[..newline_pos].trim().to_string();
                    buffer = buffer[newline_pos + 1..].to_string();

                    if line.is_empty() {
                        continue;
                    }

                    if let Some(event_type) = line.strip_prefix("event: ") {
                        current_event_type = event_type.trim().to_string();
                        continue;
                    }

                    if let Some(data) = line.strip_prefix("data: ") {
                        let data = data.trim();
                        if data == "[DONE]" {
                            let _ = tx.send(StreamEvent::MessageStop).await;
                            return;
                        }

                        if let Ok(json) = serde_json::from_str::<Value>(data) {
                            let event = match current_event_type.as_str() {
                                "message_start" => {
                                    let msg = &json["message"];
                                    Some(StreamEvent::MessageStart {
                                        id: msg["id"].as_str().unwrap_or("").to_string(),
                                        model: msg["model"].as_str().unwrap_or("").to_string(),
                                    })
                                }
                                "content_block_start" => {
                                    let index = json["index"].as_u64().unwrap_or(0) as usize;
                                    let cb = &json["content_block"];
                                    let block_type = cb["type"].as_str().unwrap_or("text");
                                    let content_block = match block_type {
                                        "text" => ContentBlock::Text {
                                            text: cb["text"].as_str().unwrap_or("").to_string(),
                                        },
                                        "tool_use" => ContentBlock::ToolUse {
                                            id: cb["id"].as_str().unwrap_or("").to_string(),
                                            name: cb["name"].as_str().unwrap_or("").to_string(),
                                            input: Value::Object(serde_json::Map::new()),
                                        },
                                        "thinking" => ContentBlock::Thinking {
                                            thinking: String::new(),
                                        },
                                        _ => ContentBlock::Text { text: String::new() },
                                    };
                                    Some(StreamEvent::ContentBlockStart { index, content_block })
                                }
                                "content_block_delta" => {
                                    let index = json["index"].as_u64().unwrap_or(0) as usize;
                                    let delta = &json["delta"];
                                    let delta_type = delta["type"].as_str().unwrap_or("");
                                    let content_delta = match delta_type {
                                        "text_delta" => ContentDelta::TextDelta {
                                            text: delta["text"].as_str().unwrap_or("").to_string(),
                                        },
                                        "input_json_delta" => ContentDelta::InputJsonDelta {
                                            partial_json: delta["partial_json"].as_str().unwrap_or("").to_string(),
                                        },
                                        "thinking_delta" => ContentDelta::ThinkingDelta {
                                            thinking: delta["thinking"].as_str().unwrap_or("").to_string(),
                                        },
                                        _ => ContentDelta::TextDelta { text: String::new() },
                                    };
                                    Some(StreamEvent::ContentBlockDelta { index, delta: content_delta })
                                }
                                "content_block_stop" => {
                                    let index = json["index"].as_u64().unwrap_or(0) as usize;
                                    Some(StreamEvent::ContentBlockStop { index })
                                }
                                "message_delta" => {
                                    let delta = &json["delta"];
                                    let stop_reason = delta["stop_reason"].as_str().map(String::from);
                                    let usage = json.get("usage").map(|u| Usage {
                                        input_tokens: u["input_tokens"].as_u64().unwrap_or(0),
                                        output_tokens: u["output_tokens"].as_u64().unwrap_or(0),
                                        cache_creation_input_tokens: 0,
                                        cache_read_input_tokens: 0,
                                    });
                                    Some(StreamEvent::MessageDelta { stop_reason, usage })
                                }
                                "message_stop" => Some(StreamEvent::MessageStop),
                                "ping" => Some(StreamEvent::Ping),
                                "error" => {
                                    let msg = json["error"]["message"].as_str()
                                        .unwrap_or("Unknown error").to_string();
                                    Some(StreamEvent::Error { message: msg })
                                }
                                _ => None,
                            };

                            if let Some(event) = event {
                                if tx.send(event).await.is_err() {
                                    return; // Receiver dropped
                                }
                            }
                        }
                    }
                }
            }
        });

        Ok(rx)
    }

    fn name(&self) -> &str {
        "anthropic"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_request_body() {
        let provider = AnthropicProvider::new("test-key".into());
        let request = CompletionRequest {
            model: "claude-sonnet-4-20250514".into(),
            system: Some("You are helpful.".into()),
            messages: vec![
                Message {
                    role: MessageRole::User,
                    content: MessageContent::Text("Hello".into()),
                },
            ],
            max_tokens: 1024,
            temperature: Some(0.7),
            ..Default::default()
        };
        let body = provider.build_request_body(&request);
        assert_eq!(body["model"], "claude-sonnet-4-20250514");
        assert_eq!(body["system"], "You are helpful.");
        assert_eq!(body["max_tokens"], 1024);
        assert_eq!(body["temperature"], 0.7);
        assert_eq!(body["messages"][0]["role"], "user");
        assert_eq!(body["messages"][0]["content"], "Hello");
    }

    #[test]
    fn builds_request_with_tools() {
        let provider = AnthropicProvider::new("test-key".into());
        let request = CompletionRequest {
            model: "claude-sonnet-4-20250514".into(),
            messages: vec![Message {
                role: MessageRole::User,
                content: MessageContent::Text("Read file.txt".into()),
            }],
            tools: vec![ToolDefinition {
                name: "read_file".into(),
                description: "Read a file".into(),
                input_schema: serde_json::json!({"type": "object", "properties": {"path": {"type": "string"}}}),
            }],
            ..Default::default()
        };
        let body = provider.build_request_body(&request);
        assert!(body["tools"].is_array());
        assert_eq!(body["tools"][0]["name"], "read_file");
    }

    #[test]
    fn parses_response() {
        let provider = AnthropicProvider::new("test-key".into());
        let body = serde_json::json!({
            "id": "msg_123",
            "model": "claude-sonnet-4-20250514",
            "content": [
                {"type": "text", "text": "Hello!"}
            ],
            "stop_reason": "end_turn",
            "usage": {
                "input_tokens": 10,
                "output_tokens": 5
            }
        });
        let response = provider.parse_response(&body).unwrap();
        assert_eq!(response.id, "msg_123");
        assert_eq!(response.usage.input_tokens, 10);
        assert_eq!(response.stop_reason, Some("end_turn".into()));
        assert_eq!(response.content.len(), 1);
    }

    #[test]
    fn parses_tool_use_response() {
        let provider = AnthropicProvider::new("test-key".into());
        let body = serde_json::json!({
            "id": "msg_456",
            "model": "claude-sonnet-4-20250514",
            "content": [
                {
                    "type": "tool_use",
                    "id": "tu_789",
                    "name": "read_file",
                    "input": {"path": "/tmp/test.txt"}
                }
            ],
            "stop_reason": "tool_use",
            "usage": {"input_tokens": 20, "output_tokens": 15}
        });
        let response = provider.parse_response(&body).unwrap();
        assert_eq!(response.stop_reason, Some("tool_use".into()));
        match &response.content[0] {
            ContentBlock::ToolUse { id, name, input } => {
                assert_eq!(id, "tu_789");
                assert_eq!(name, "read_file");
                assert_eq!(input["path"], "/tmp/test.txt");
            }
            _ => panic!("Expected ToolUse block"),
        }
    }

    #[test]
    fn sse_event_parsing() {
        assert_eq!(
            AnthropicProvider::parse_sse_event("event: message_start"),
            Some(("event".into(), "message_start".into()))
        );
        assert_eq!(
            AnthropicProvider::parse_sse_event("data: {\"test\": true}"),
            Some(("data".into(), "{\"test\": true}".into()))
        );
        assert_eq!(
            AnthropicProvider::parse_sse_event("random line"),
            None
        );
    }
}
