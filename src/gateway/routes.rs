use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde_json::{json, Value};
use crate::gateway::state::GatewayState;
use crate::version::VERSION;

/// Build the HTTP router with all routes.
pub fn build_router(state: GatewayState) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/v1/health", get(health))
        .route("/v1/status", get(status))
        .route("/v1/config", get(get_config))
        .route("/v1/sessions", get(list_sessions))
        .route("/v1/tools", get(list_tools))
        .route("/v1/chat/completions", post(chat_completions))
        .with_state(state)
}

async fn health() -> Json<Value> {
    Json(json!({
        "status": "ok",
        "version": VERSION,
        "engine": "rustyclaw"
    }))
}

async fn status(State(state): State<GatewayState>) -> Json<Value> {
    let config = state.config.read().await;
    let session_count = state.session_manager.count().await;
    let tools = state.tool_registry.list_definitions().await;
    let channels = state.channel_manager.read().await.list_channels()
        .iter().map(|s| Value::String(s.to_string())).collect::<Vec<_>>();

    Json(json!({
        "status": "running",
        "version": VERSION,
        "engine": "rustyclaw",
        "uptime_seconds": state.uptime_secs(),
        "sessions": session_count,
        "tools": tools.len(),
        "channels": channels,
        "model": config.primary_model(),
        "workspace": config.workspace_dir(),
    }))
}

async fn get_config(State(state): State<GatewayState>) -> Json<Value> {
    let config = state.config.read().await;
    // Return a sanitized config (no secrets)
    Json(json!({
        "gateway": {
            "port": config.gateway.as_ref().and_then(|g| g.port),
            "bind": config.gateway.as_ref().and_then(|g| g.bind.clone()),
            "mode": config.gateway.as_ref().and_then(|g| g.mode.clone()),
        },
        "model": config.primary_model(),
        "workspace": config.workspace_dir(),
        "plugins": config.plugins.as_ref().and_then(|p| p.entries.as_ref())
            .map(|e| e.keys().cloned().collect::<Vec<_>>()),
    }))
}

async fn list_sessions(State(state): State<GatewayState>) -> Json<Value> {
    let keys = state.session_manager.list_keys().await;
    Json(json!({
        "count": keys.len(),
        "sessions": keys,
    }))
}

async fn list_tools(State(state): State<GatewayState>) -> Json<Value> {
    let tools = state.tool_registry.list_definitions().await;
    let tool_list: Vec<Value> = tools.iter().map(|t| {
        json!({
            "name": t.name,
            "description": t.description,
        })
    }).collect();
    Json(json!({
        "count": tool_list.len(),
        "tools": tool_list,
    }))
}

/// OpenAI-compatible chat completions endpoint (stub).
async fn chat_completions(
    State(_state): State<GatewayState>,
    Json(body): Json<Value>,
) -> Result<Json<Value>, StatusCode> {
    let model = body["model"].as_str().unwrap_or("claude-sonnet-4-20250514");
    let messages = body["messages"].as_array()
        .ok_or(StatusCode::BAD_REQUEST)?;

    // For now, return a structured response indicating the request was received
    Ok(Json(json!({
        "id": format!("chatcmpl-{}", uuid::Uuid::new_v4()),
        "object": "chat.completion",
        "model": model,
        "choices": [{
            "index": 0,
            "message": {
                "role": "assistant",
                "content": format!("rustyclaw received {} messages for model {}", messages.len(), model)
            },
            "finish_reason": "stop"
        }],
        "usage": {
            "prompt_tokens": 0,
            "completion_tokens": 0,
            "total_tokens": 0
        }
    })))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::Request;
    use tower::ServiceExt;

    fn test_state() -> GatewayState {
        GatewayState::new(crate::config::OpenClawConfig::default())
    }

    #[tokio::test]
    async fn health_endpoint() {
        let app = build_router(test_state());
        let response = app
            .oneshot(Request::builder().uri("/health").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn status_endpoint() {
        let app = build_router(test_state());
        let response = app
            .oneshot(Request::builder().uri("/v1/status").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn sessions_endpoint() {
        let app = build_router(test_state());
        let response = app
            .oneshot(Request::builder().uri("/v1/sessions").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn tools_endpoint() {
        let app = build_router(test_state());
        let response = app
            .oneshot(Request::builder().uri("/v1/tools").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }
}
