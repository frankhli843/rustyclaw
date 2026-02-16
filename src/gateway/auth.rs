use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use crate::gateway::state::GatewayState;

/// Extract bearer token from Authorization header.
pub fn extract_bearer_token(auth_header: &str) -> Option<&str> {
    auth_header.strip_prefix("Bearer ").or_else(|| auth_header.strip_prefix("bearer "))
}

/// Verify a token against the expected token using constant-time comparison.
pub fn verify_token(provided: &str, expected: &str) -> bool {
    crate::security::secret_equal::safe_equal_secret(Some(provided), Some(expected))
}

/// Auth middleware for axum.
pub async fn auth_middleware(
    axum::extract::State(state): axum::extract::State<GatewayState>,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // If no auth token configured, allow all
    let expected_token = match &state.auth_token {
        Some(t) => t,
        None => return Ok(next.run(request).await),
    };

    // Skip auth for health endpoint
    if request.uri().path() == "/health" || request.uri().path() == "/v1/health" {
        return Ok(next.run(request).await);
    }

    // Check Authorization header
    let auth_header = request.headers()
        .get("authorization")
        .and_then(|v| v.to_str().ok());

    match auth_header {
        Some(header) => {
            if let Some(token) = extract_bearer_token(header) {
                if verify_token(token, expected_token) {
                    return Ok(next.run(request).await);
                }
            }
            Err(StatusCode::UNAUTHORIZED)
        }
        None => {
            // Also check query param ?token=
            let uri = request.uri();
            if let Some(query) = uri.query() {
                for param in query.split('&') {
                    if let Some(token) = param.strip_prefix("token=") {
                        if verify_token(token, expected_token) {
                            return Ok(next.run(request).await);
                        }
                    }
                }
            }
            Err(StatusCode::UNAUTHORIZED)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_bearer() {
        assert_eq!(extract_bearer_token("Bearer abc123"), Some("abc123"));
        assert_eq!(extract_bearer_token("bearer abc123"), Some("abc123"));
        assert_eq!(extract_bearer_token("Basic abc"), None);
    }

    #[test]
    fn verify_token_works() {
        assert!(verify_token("secret", "secret"));
        assert!(!verify_token("wrong", "secret"));
    }
}
