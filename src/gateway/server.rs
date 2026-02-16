use axum::{
    middleware,
    routing::get,
    Router,
};
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;
use tracing::info;

use crate::config::{self, OpenClawConfig};
use crate::gateway::{auth, routes, ws, state::GatewayState};

/// Start the gateway server.
pub async fn start_gateway(config: OpenClawConfig) -> Result<(), Box<dyn std::error::Error>> {
    let port = config::resolve_gateway_port(&config);
    let bind_addr = config::resolve_gateway_bind(&config);

    let state = GatewayState::new(config);

    // Register builtin tools
    state.tool_registry.register_builtins().await;

    // Build router
    let app = build_app(state.clone());

    let addr: SocketAddr = format!("{}:{}", bind_addr, port).parse()?;
    info!("🦀 rustyclaw gateway starting on {}", addr);
    info!("  Version: {}", crate::version::VERSION);
    info!("  Engine: rustyclaw (Rust)");
    info!("  Auth: {}", if state.auth_token.is_some() { "token" } else { "none" });

    let listener = TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

/// Build the full application with middleware.
fn build_app(state: GatewayState) -> Router {
    let ws_state = state.clone();

    // Routes that need auth
    let protected = routes::build_router(state.clone())
        .layer(middleware::from_fn_with_state(
            state.clone(),
            auth::auth_middleware,
        ));

    // WebSocket route (auth checked in handler)
    let ws_route = Router::new()
        .route("/ws", get(ws::ws_handler))
        .with_state(ws_state);

    // Combine
    Router::new()
        .merge(ws_route)
        .merge(protected)
        .layer(CorsLayer::permissive())
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::Request;
    use tower::ServiceExt;

    #[tokio::test]
    async fn app_serves_health() {
        let config = OpenClawConfig::default();
        let state = GatewayState::new(config);
        state.tool_registry.register_builtins().await;
        let app = build_app(state);

        let response = app
            .oneshot(Request::builder().uri("/health").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(response.status(), 200);
    }

    #[tokio::test]
    async fn app_auth_required() {
        let json = r#"{"gateway":{"auth":{"token":"secret"}}}"#;
        let config: OpenClawConfig = serde_json::from_str(json).unwrap();
        let state = GatewayState::new(config);
        let app = build_app(state);

        // /v1/status without auth should fail
        let response = app
            .oneshot(Request::builder().uri("/v1/status").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(response.status(), 401);
    }

    #[tokio::test]
    async fn app_auth_with_token() {
        let json = r#"{"gateway":{"auth":{"token":"secret"}}}"#;
        let config: OpenClawConfig = serde_json::from_str(json).unwrap();
        let state = GatewayState::new(config);
        state.tool_registry.register_builtins().await;
        let app = build_app(state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/v1/status")
                    .header("authorization", "Bearer secret")
                    .body(Body::empty())
                    .unwrap()
            )
            .await
            .unwrap();
        assert_eq!(response.status(), 200);
    }

    #[tokio::test]
    async fn health_bypasses_auth() {
        let json = r#"{"gateway":{"auth":{"token":"secret"}}}"#;
        let config: OpenClawConfig = serde_json::from_str(json).unwrap();
        let state = GatewayState::new(config);
        let app = build_app(state);

        // Health should work without auth
        let response = app
            .oneshot(Request::builder().uri("/health").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(response.status(), 200);
    }
}
