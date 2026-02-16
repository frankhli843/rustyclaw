use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A message in a conversation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: MessageRole,
    pub content: MessageContent,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    System,
    User,
    Assistant,
    Tool,
}

/// Message content — can be plain text or structured content blocks.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MessageContent {
    Text(String),
    Blocks(Vec<ContentBlock>),
}

impl MessageContent {
    pub fn as_text(&self) -> Option<&str> {
        match self {
            MessageContent::Text(s) => Some(s),
            MessageContent::Blocks(blocks) => {
                // Return text of first text block
                blocks.iter().find_map(|b| {
                    if let ContentBlock::Text { text } = b {
                        Some(text.as_str())
                    } else {
                        None
                    }
                })
            }
        }
    }

    pub fn to_text(&self) -> String {
        match self {
            MessageContent::Text(s) => s.clone(),
            MessageContent::Blocks(blocks) => {
                blocks.iter().filter_map(|b| {
                    match b {
                        ContentBlock::Text { text } => Some(text.clone()),
                        _ => None,
                    }
                }).collect::<Vec<_>>().join("\n")
            }
        }
    }
}

/// Content block types used in Anthropic Messages API.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ContentBlock {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "image")]
    Image {
        source: ImageSource,
    },
    #[serde(rename = "tool_use")]
    ToolUse {
        id: String,
        name: String,
        input: serde_json::Value,
    },
    #[serde(rename = "tool_result")]
    ToolResult {
        tool_use_id: String,
        content: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        is_error: Option<bool>,
    },
    #[serde(rename = "thinking")]
    Thinking {
        thinking: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageSource {
    #[serde(rename = "type")]
    pub source_type: String,
    pub media_type: String,
    pub data: String,
}

/// Tool definition for function calling.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
}

/// A chat completion request.
#[derive(Debug, Clone)]
pub struct CompletionRequest {
    pub model: String,
    pub system: Option<String>,
    pub messages: Vec<Message>,
    pub tools: Vec<ToolDefinition>,
    pub max_tokens: u32,
    pub temperature: Option<f64>,
    pub stream: bool,
    pub stop_sequences: Vec<String>,
    pub metadata: HashMap<String, String>,
}

impl Default for CompletionRequest {
    fn default() -> Self {
        Self {
            model: "claude-sonnet-4-20250514".to_string(),
            system: None,
            messages: Vec::new(),
            tools: Vec::new(),
            max_tokens: 8192,
            temperature: None,
            stream: false,
            stop_sequences: Vec::new(),
            metadata: HashMap::new(),
        }
    }
}

/// A completion response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionResponse {
    pub id: String,
    pub model: String,
    pub content: Vec<ContentBlock>,
    pub stop_reason: Option<String>,
    pub usage: Usage,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Usage {
    pub input_tokens: u64,
    pub output_tokens: u64,
    #[serde(default)]
    pub cache_creation_input_tokens: u64,
    #[serde(default)]
    pub cache_read_input_tokens: u64,
}

/// A streaming event from the provider.
#[derive(Debug, Clone)]
pub enum StreamEvent {
    /// Message started
    MessageStart { id: String, model: String },
    /// Content block started
    ContentBlockStart { index: usize, content_block: ContentBlock },
    /// Text delta
    ContentBlockDelta { index: usize, delta: ContentDelta },
    /// Content block finished
    ContentBlockStop { index: usize },
    /// Message delta (stop reason, usage)
    MessageDelta { stop_reason: Option<String>, usage: Option<Usage> },
    /// Message finished
    MessageStop,
    /// Ping
    Ping,
    /// Error
    Error { message: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ContentDelta {
    #[serde(rename = "text_delta")]
    TextDelta { text: String },
    #[serde(rename = "input_json_delta")]
    InputJsonDelta { partial_json: String },
    #[serde(rename = "thinking_delta")]
    ThinkingDelta { thinking: String },
}

/// Provider trait — abstraction over LLM providers.
#[async_trait::async_trait]
pub trait Provider: Send + Sync {
    /// Get a non-streaming completion.
    async fn complete(&self, request: &CompletionRequest) -> Result<CompletionResponse, ProviderError>;

    /// Get a streaming completion.
    async fn stream(&self, request: &CompletionRequest) -> Result<
        tokio::sync::mpsc::Receiver<StreamEvent>,
        ProviderError,
    >;

    /// Provider name.
    fn name(&self) -> &str;
}

#[derive(Debug, thiserror::Error)]
pub enum ProviderError {
    #[error("API error: {status} - {message}")]
    ApiError { status: u16, message: String },
    #[error("Authentication error: {0}")]
    AuthError(String),
    #[error("Rate limited: retry after {retry_after_ms}ms")]
    RateLimited { retry_after_ms: u64 },
    #[error("Network error: {0}")]
    NetworkError(String),
    #[error("Invalid request: {0}")]
    InvalidRequest(String),
    #[error("Provider error: {0}")]
    Other(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn message_content_text() {
        let content = MessageContent::Text("hello".into());
        assert_eq!(content.as_text(), Some("hello"));
        assert_eq!(content.to_text(), "hello");
    }

    #[test]
    fn message_content_blocks() {
        let content = MessageContent::Blocks(vec![
            ContentBlock::Text { text: "hello".into() },
            ContentBlock::Text { text: "world".into() },
        ]);
        assert_eq!(content.as_text(), Some("hello"));
        assert_eq!(content.to_text(), "hello\nworld");
    }

    #[test]
    fn tool_definition_serializes() {
        let tool = ToolDefinition {
            name: "read_file".into(),
            description: "Read a file".into(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string" }
                },
                "required": ["path"]
            }),
        };
        let json = serde_json::to_string(&tool).unwrap();
        assert!(json.contains("read_file"));
    }

    #[test]
    fn completion_request_defaults() {
        let req = CompletionRequest::default();
        assert_eq!(req.max_tokens, 8192);
        assert!(!req.stream);
    }

    #[test]
    fn parse_model_id() {
        let (p, m) = crate::config::OpenClawConfig::parse_model_id("anthropic/claude-opus-4-6");
        assert_eq!(p, "anthropic");
        assert_eq!(m, "claude-opus-4-6");
    }
}
