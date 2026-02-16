use crate::config::OpenClawConfig;
use crate::session::SessionManager;
use crate::tools::ToolRegistry;
use crate::channel::ChannelManager;
use crate::cron_system::CronService;
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::{DateTime, Utc};

/// Shared gateway state, accessible from all request handlers.
#[derive(Clone)]
pub struct GatewayState {
    pub config: Arc<RwLock<OpenClawConfig>>,
    pub session_manager: SessionManager,
    pub tool_registry: ToolRegistry,
    pub channel_manager: Arc<RwLock<ChannelManager>>,
    pub cron_service: Arc<RwLock<Option<CronService>>>,
    pub start_time: DateTime<Utc>,
    pub auth_token: Option<String>,
    pub workspace_dir: String,
}

impl GatewayState {
    pub fn new(config: OpenClawConfig) -> Self {
        let auth_token = crate::config::resolve_gateway_auth_token(&config);
        let workspace_dir = config.workspace_dir()
            .unwrap_or("~/.openclaw/workspace")
            .to_string();

        let tool_deny = config.tools.as_ref()
            .and_then(|t| t.deny.clone())
            .unwrap_or_default();
        let tool_allow = config.tools.as_ref()
            .and_then(|t| t.allow.clone())
            .unwrap_or_default();

        Self {
            config: Arc::new(RwLock::new(config)),
            session_manager: SessionManager::new(1000),
            tool_registry: ToolRegistry::with_policy(tool_deny, tool_allow),
            channel_manager: Arc::new(RwLock::new(ChannelManager::new())),
            cron_service: Arc::new(RwLock::new(None)),
            start_time: Utc::now(),
            auth_token,
            workspace_dir,
        }
    }

    /// Gateway uptime in seconds.
    pub fn uptime_secs(&self) -> i64 {
        (Utc::now() - self.start_time).num_seconds()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gateway_state_creation() {
        let config = OpenClawConfig::default();
        let state = GatewayState::new(config);
        assert!(state.auth_token.is_none());
        assert!(state.uptime_secs() >= 0);
    }

    #[test]
    fn gateway_state_with_auth() {
        let json = r#"{"gateway":{"auth":{"token":"secret123"}}}"#;
        let config: OpenClawConfig = serde_json::from_str(json).unwrap();
        let state = GatewayState::new(config);
        assert_eq!(state.auth_token, Some("secret123".into()));
    }
}
