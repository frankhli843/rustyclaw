pub mod types;

use std::path::PathBuf;
use crate::utils::resolve_config_dir;
pub use types::*;

/// Resolve the path to the config file.
pub fn resolve_config_path() -> PathBuf {
    let dir = resolve_config_dir();
    let json_path = dir.join("openclaw.json");
    if json_path.exists() {
        return json_path;
    }
    let yaml_path = dir.join("openclaw.yaml");
    if yaml_path.exists() {
        return yaml_path;
    }
    let yml_path = dir.join("openclaw.yml");
    if yml_path.exists() {
        return yml_path;
    }
    // Default to JSON
    json_path
}

/// Load configuration from the default config path.
pub fn load_config() -> Result<OpenClawConfig, Box<dyn std::error::Error>> {
    let config_path = resolve_config_path();
    load_config_from_path(&config_path)
}

/// Load configuration from a specific path.
pub fn load_config_from_path(path: &PathBuf) -> Result<OpenClawConfig, Box<dyn std::error::Error>> {
    if !path.exists() {
        return Ok(OpenClawConfig::default());
    }
    let contents = std::fs::read_to_string(path)?;
    let contents = substitute_env_vars(&contents);

    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("json");
    let config: OpenClawConfig = match ext {
        "yaml" | "yml" => serde_yaml::from_str(&contents)?,
        _ => serde_json::from_str(&contents)?,
    };
    Ok(config)
}

/// Simple ${ENV_VAR} substitution in config strings.
fn substitute_env_vars(input: &str) -> String {
    let re = regex::Regex::new(r"\$\{([A-Za-z_][A-Za-z0-9_]*)\}").unwrap();
    re.replace_all(input, |caps: &regex::Captures| {
        let var_name = &caps[1];
        std::env::var(var_name).unwrap_or_default()
    }).into_owned()
}

/// Resolve the gateway port from config, with default.
pub fn resolve_gateway_port(config: &OpenClawConfig) -> u16 {
    config.gateway.as_ref()
        .and_then(|g| g.port)
        .unwrap_or(18789)
}

/// Resolve the gateway bind address from config.
pub fn resolve_gateway_bind(config: &OpenClawConfig) -> String {
    let bind_mode = config.gateway.as_ref()
        .and_then(|g| g.bind.as_deref())
        .unwrap_or("loopback");

    match bind_mode {
        "lan" => "0.0.0.0".to_string(),
        "loopback" => "127.0.0.1".to_string(),
        "auto" => "127.0.0.1".to_string(),
        _ => "127.0.0.1".to_string(),
    }
}

/// Get the gateway auth token from config.
pub fn resolve_gateway_auth_token(config: &OpenClawConfig) -> Option<String> {
    config.gateway.as_ref()
        .and_then(|g| g.auth.as_ref())
        .and_then(|a| a.token.clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_is_valid() {
        let config = OpenClawConfig::default();
        assert!(config.channels.is_none());
        assert!(config.gateway.is_none());
    }

    #[test]
    fn parses_json_config() {
        let json = r#"{
            "channels": {
                "whatsapp": {
                    "allowFrom": ["+1234567890"],
                    "dmPolicy": "disabled"
                }
            },
            "gateway": {
                "port": 3000,
                "bind": "loopback",
                "auth": {
                    "mode": "token",
                    "token": "test-token"
                }
            }
        }"#;
        let config: OpenClawConfig = serde_json::from_str(json).unwrap();
        assert_eq!(resolve_gateway_port(&config), 3000);
        assert_eq!(resolve_gateway_auth_token(&config), Some("test-token".to_string()));
    }

    #[test]
    fn parses_full_config() {
        let json = r#"{
            "meta": { "lastTouchedVersion": "2026.2.15" },
            "agents": {
                "defaults": {
                    "model": { "primary": "anthropic/claude-opus-4-6" },
                    "workspace": "/tmp/workspace"
                }
            },
            "channels": {
                "whatsapp": {
                    "dmPolicy": "disabled",
                    "allowFrom": ["+1555"],
                    "groupPolicy": "open",
                    "groups": {
                        "*": { "requireMention": false }
                    }
                }
            },
            "gateway": {
                "port": 18789,
                "mode": "local",
                "bind": "loopback",
                "auth": { "mode": "token", "token": "abc123" }
            },
            "plugins": {
                "entries": {
                    "whatsapp": { "enabled": true }
                }
            }
        }"#;
        let config: OpenClawConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.meta.as_ref().unwrap().last_touched_version.as_deref(), Some("2026.2.15"));
        let agents = config.agents.as_ref().unwrap();
        let defaults = agents.defaults.as_ref().unwrap();
        assert_eq!(defaults.workspace.as_deref(), Some("/tmp/workspace"));
    }

    #[test]
    fn env_var_substitution() {
        std::env::set_var("RUSTYCLAW_TEST_VAR", "hello");
        let result = substitute_env_vars("value: ${RUSTYCLAW_TEST_VAR}");
        assert_eq!(result, "value: hello");
        std::env::remove_var("RUSTYCLAW_TEST_VAR");
    }

    #[test]
    fn resolve_port_default() {
        let config = OpenClawConfig::default();
        assert_eq!(resolve_gateway_port(&config), 18789);
    }

    #[test]
    fn resolve_bind_modes() {
        let mk = |bind: &str| -> OpenClawConfig {
            serde_json::from_str(&format!(r#"{{"gateway":{{"bind":"{}"}}}}"#, bind)).unwrap()
        };
        assert_eq!(resolve_gateway_bind(&mk("lan")), "0.0.0.0");
        assert_eq!(resolve_gateway_bind(&mk("loopback")), "127.0.0.1");
    }

    #[test]
    fn load_config_returns_default_when_missing() {
        std::env::set_var("OPENCLAW_STATE_DIR", "/tmp/rustyclaw-test-nonexistent-new");
        let config = load_config().unwrap();
        assert!(config.channels.is_none());
        std::env::remove_var("OPENCLAW_STATE_DIR");
    }
}
