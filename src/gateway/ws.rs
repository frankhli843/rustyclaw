use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::Response,
};
use futures::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tracing::{error, info, warn};
use crate::gateway::state::GatewayState;

/// WebSocket protocol version.
pub const PROTOCOL_VERSION: u32 = 1;

/// JSON-RPC style message for the gateway WebSocket protocol.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WsMessage {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub method: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<Value>,
}

/// Handle WebSocket upgrade.
pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<GatewayState>,
) -> Response {
    ws.on_upgrade(move |socket| handle_ws_connection(socket, state))
}

async fn handle_ws_connection(socket: WebSocket, state: GatewayState) {
    let (mut sender, mut receiver) = socket.split();
    info!("WebSocket client connected");

    // Send welcome message
    let welcome = json!({
        "method": "gateway.hello",
        "params": {
            "version": crate::version::VERSION,
            "protocol": PROTOCOL_VERSION,
            "engine": "rustyclaw",
        }
    });
    if let Err(e) = sender.send(Message::Text(welcome.to_string().into())).await {
        error!("Failed to send welcome: {}", e);
        return;
    }

    while let Some(msg_result) = receiver.next().await {
        let msg = match msg_result {
            Ok(m) => m,
            Err(e) => {
                warn!("WebSocket receive error: {}", e);
                break;
            }
        };

        match msg {
            Message::Text(text) => {
                let text_str: &str = &text;
                match serde_json::from_str::<WsMessage>(text_str) {
                    Ok(ws_msg) => {
                        let response = handle_ws_method(&state, &ws_msg).await;
                        if let Some(resp) = response {
                            let json_str = serde_json::to_string(&resp).unwrap_or_default();
                            if let Err(e) = sender.send(Message::Text(json_str.into())).await {
                                error!("Failed to send response: {}", e);
                                break;
                            }
                        }
                    }
                    Err(e) => {
                        warn!("Invalid WS message: {}", e);
                        let error_resp = json!({
                            "error": { "code": -32700, "message": "Parse error" }
                        });
                        let _ = sender.send(Message::Text(error_resp.to_string().into())).await;
                    }
                }
            }
            Message::Ping(data) => {
                let _ = sender.send(Message::Pong(data)).await;
            }
            Message::Close(_) => {
                info!("WebSocket client disconnected");
                break;
            }
            _ => {}
        }
    }
}

async fn handle_ws_method(state: &GatewayState, msg: &WsMessage) -> Option<WsMessage> {
    let method = msg.method.as_deref().unwrap_or("");

    let result = match method {
        "gateway.status" => {
            let session_count = state.session_manager.count().await;
            json!({
                "status": "running",
                "version": crate::version::VERSION,
                "uptime": state.uptime_secs(),
                "sessions": session_count,
            })
        }
        "gateway.health" => {
            json!({ "status": "ok" })
        }
        "sessions.list" => {
            let keys = state.session_manager.list_keys().await;
            json!({ "sessions": keys })
        }
        "config.get" => {
            let config = state.config.read().await;
            json!({
                "model": config.primary_model(),
                "workspace": config.workspace_dir(),
            })
        }
        "tools.list" => {
            let tools = state.tool_registry.list_definitions().await;
            let names: Vec<&str> = tools.iter().map(|t| t.name.as_str()).collect();
            json!({ "tools": names })
        }
        _ => {
            return Some(WsMessage {
                id: msg.id.clone(),
                method: None,
                params: None,
                result: None,
                error: Some(json!({
                    "code": -32601,
                    "message": format!("Method not found: {}", method)
                })),
            });
        }
    };

    Some(WsMessage {
        id: msg.id.clone(),
        method: None,
        params: None,
        result: Some(result),
        error: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ws_message_serialization() {
        let msg = WsMessage {
            id: Some("1".into()),
            method: Some("gateway.status".into()),
            params: None,
            result: None,
            error: None,
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("gateway.status"));
        // Verify None fields are skipped
        assert!(!json.contains("params"));
    }

    #[test]
    fn ws_message_deserialization() {
        let json = r#"{"id":"1","method":"gateway.status"}"#;
        let msg: WsMessage = serde_json::from_str(json).unwrap();
        assert_eq!(msg.id, Some("1".into()));
        assert_eq!(msg.method, Some("gateway.status".into()));
    }
}
