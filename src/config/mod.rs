use std::path::PathBuf;
use serde::{Deserialize, Serialize};

use crate::utils::resolve_config_dir;

/// Core OpenClaw configuration structure.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OpenClawConfig {
    #[serde(default)]
    pub channels: ChannelsConfig,
    #[serde(default)]
    pub models: ModelsConfig,
    #[serde(default)]
    pub gateway: GatewayConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ChannelsConfig {
    pub whatsapp: Option<WhatsAppConfig>,
    pub telegram: Option<TelegramConfig>,
    pub discord: Option<DiscordConfig>,
    pub slack: Option<SlackConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WhatsAppConfig {
    pub allow_from: Option<Vec<String>>,
    pub phone: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TelegramConfig {
    pub bot_token: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DiscordConfig {
    pub bot_token: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SlackConfig {
    pub bot_token: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ModelsConfig {
    pub default: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GatewayConfig {
    pub port: Option<u16>,
    pub host: Option<String>,
}

/// Resolve the path to the config file.
pub fn resolve_config_path() -> PathBuf {
    resolve_config_dir().join("openclaw.json")
}

/// Load configuration from the default config path.
pub fn load_config() -> Result<OpenClawConfig, Box<dyn std::error::Error>> {
    let config_path = resolve_config_path();
    if !config_path.exists() {
        return Ok(OpenClawConfig::default());
    }
    let contents = std::fs::read_to_string(&config_path)?;
    let config: OpenClawConfig = serde_json::from_str(&contents)?;
    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_is_valid() {
        let config = OpenClawConfig::default();
        assert!(config.channels.whatsapp.is_none());
        assert!(config.models.default.is_none());
        assert!(config.gateway.port.is_none());
    }

    #[test]
    fn parses_json_config() {
        let json = r#"{
            "channels": {
                "whatsapp": {
                    "phone": "+1234567890",
                    "allow_from": ["+1234567890"]
                }
            },
            "models": {
                "default": "claude-opus-4-6"
            },
            "gateway": {
                "port": 3000
            }
        }"#;
        let config: OpenClawConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.gateway.port, Some(3000));
        assert_eq!(config.models.default.as_deref(), Some("claude-opus-4-6"));
    }

    #[test]
    fn load_config_returns_default_when_missing() {
        // Set a temp dir so config file won't exist
        std::env::set_var("OPENCLAW_STATE_DIR", "/tmp/rustyclaw-test-nonexistent");
        let config = load_config().unwrap();
        assert!(config.channels.whatsapp.is_none());
        std::env::remove_var("OPENCLAW_STATE_DIR");
    }
}
