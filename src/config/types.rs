use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Root OpenClaw configuration — matches the real openclaw.json format.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct OpenClawConfig {
    pub meta: Option<MetaConfig>,
    pub auth: Option<AuthConfig>,
    pub env: Option<EnvConfig>,
    pub wizard: Option<WizardConfig>,
    pub agents: Option<AgentsConfig>,
    pub models: Option<ModelsConfig>,
    pub messages: Option<MessagesConfig>,
    pub commands: Option<CommandsConfig>,
    pub channels: Option<ChannelsConfig>,
    pub gateway: Option<GatewayConfig>,
    pub skills: Option<SkillsConfig>,
    pub plugins: Option<PluginsConfig>,
    pub cron: Option<CronConfig>,
    pub memory: Option<MemoryConfig>,
    pub tools: Option<ToolsConfig>,
    pub hooks: Option<HooksConfig>,
    pub browser: Option<BrowserConfig>,
    pub session: Option<SessionConfig>,
    pub broadcast: Option<BroadcastConfig>,
    pub discovery: Option<DiscoveryConfig>,
    #[serde(rename = "nodeHost")]
    pub node_host: Option<NodeHostConfig>,
    pub ui: Option<UiConfig>,
    pub logging: Option<LoggingConfig>,
    pub approvals: Option<ApprovalsConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct MetaConfig {
    pub last_touched_version: Option<String>,
    pub last_touched_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct AuthConfig {
    pub profiles: Option<HashMap<String, AuthProfile>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct AuthProfile {
    pub provider: Option<String>,
    pub mode: Option<String>,
    pub api_key: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct EnvConfig {
    pub vars: Option<HashMap<String, String>>,
    pub shell_env: Option<ShellEnvConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ShellEnvConfig {
    pub enabled: Option<bool>,
    pub timeout_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct WizardConfig {
    pub last_run_at: Option<String>,
    pub last_run_version: Option<String>,
    pub last_run_command: Option<String>,
    pub last_run_mode: Option<String>,
}

// ── Agents ──

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct AgentsConfig {
    pub defaults: Option<AgentDefaults>,
    pub list: Option<Vec<AgentEntry>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct AgentDefaults {
    pub model: Option<AgentModelConfig>,
    pub models: Option<HashMap<String, ModelAliasEntry>>,
    pub workspace: Option<String>,
    pub memory_search: Option<MemorySearchConfig>,
    pub compaction: Option<CompactionConfig>,
    pub heartbeat: Option<HeartbeatConfig>,
    pub max_concurrent: Option<u32>,
    pub subagents: Option<SubagentsConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct AgentModelConfig {
    pub primary: Option<String>,
    pub thinking: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ModelAliasEntry {
    pub alias: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct MemorySearchConfig {
    pub enabled: Option<bool>,
    pub sources: Option<Vec<String>>,
    pub extra_paths: Option<Vec<String>>,
    pub provider: Option<String>,
    pub sync: Option<MemorySyncConfig>,
    pub query: Option<MemoryQueryConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct MemorySyncConfig {
    pub on_session_start: Option<bool>,
    pub on_search: Option<bool>,
    pub watch: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct MemoryQueryConfig {
    pub hybrid: Option<HybridConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct HybridConfig {
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct CompactionConfig {
    pub mode: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct HeartbeatConfig {
    pub every: Option<String>,
    pub active_hours: Option<ActiveHoursConfig>,
    pub target: Option<String>,
    pub to: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ActiveHoursConfig {
    pub start: Option<String>,
    pub end: Option<String>,
    pub timezone: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct SubagentsConfig {
    pub max_concurrent: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct AgentEntry {
    pub id: Option<String>,
    pub name: Option<String>,
    pub workspace: Option<String>,
    pub agent_dir: Option<String>,
    pub model: Option<String>,
    pub group_chat: Option<GroupChatConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct GroupChatConfig {
    pub mention_patterns: Option<Vec<String>>,
    pub history_limit: Option<u32>,
}

// ── Models ──

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ModelsConfig {
    pub default: Option<String>,
    pub providers: Option<HashMap<String, ProviderModelConfig>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ProviderModelConfig {
    pub base_url: Option<String>,
    pub api: Option<String>,
    pub models: Option<Vec<ModelDefinition>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ModelDefinition {
    pub id: Option<String>,
    pub name: Option<String>,
    pub context_window: Option<u64>,
    pub max_tokens: Option<u64>,
}

// ── Messages ──

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct MessagesConfig {
    pub ack_reaction_scope: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct CommandsConfig {
    pub native: Option<String>,
    pub native_skills: Option<String>,
    pub restart: Option<bool>,
}

// ── Channels ──

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ChannelsConfig {
    pub whatsapp: Option<WhatsAppConfig>,
    pub telegram: Option<TelegramConfig>,
    pub discord: Option<DiscordConfig>,
    pub slack: Option<SlackConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct WhatsAppConfig {
    pub dm_policy: Option<String>,
    pub self_chat_mode: Option<bool>,
    pub allow_from: Option<Vec<String>>,
    pub group_policy: Option<String>,
    pub groups: Option<HashMap<String, WhatsAppGroupConfig>>,
    pub debounce_ms: Option<u64>,
    pub media_max_mb: Option<u32>,
    pub phone: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct WhatsAppGroupConfig {
    pub require_mention: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct TelegramConfig {
    pub dm_policy: Option<String>,
    pub bot_token: Option<String>,
    pub allow_from: Option<Vec<String>>,
    pub group_policy: Option<String>,
    pub stream_mode: Option<String>,
    pub link_preview: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct DiscordConfig {
    pub bot_token: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct SlackConfig {
    pub bot_token: Option<String>,
}

// ── Gateway ──

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct GatewayConfig {
    pub port: Option<u16>,
    pub mode: Option<String>,
    pub bind: Option<String>,
    pub custom_bind_host: Option<String>,
    pub auth: Option<GatewayAuthConfig>,
    pub tailscale: Option<TailscaleConfig>,
    pub remote: Option<RemoteConfig>,
    pub tls: Option<TlsConfig>,
    pub reload: Option<ReloadConfig>,
    pub http: Option<HttpConfig>,
    pub nodes: Option<GatewayNodesConfig>,
    pub trusted_proxies: Option<Vec<String>>,
    pub control_ui: Option<ControlUiConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct GatewayAuthConfig {
    pub mode: Option<String>,
    pub token: Option<String>,
    pub password: Option<String>,
    pub allow_tailscale: Option<bool>,
    pub rate_limit: Option<RateLimitConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct RateLimitConfig {
    pub max_attempts: Option<u32>,
    pub window_ms: Option<u64>,
    pub lockout_ms: Option<u64>,
    pub exempt_loopback: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct TailscaleConfig {
    pub mode: Option<String>,
    pub reset_on_exit: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct RemoteConfig {
    pub url: Option<String>,
    pub transport: Option<String>,
    pub token: Option<String>,
    pub password: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct TlsConfig {
    pub enabled: Option<bool>,
    pub cert_path: Option<String>,
    pub key_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ReloadConfig {
    pub mode: Option<String>,
    pub debounce_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct HttpConfig {
    pub endpoints: Option<HttpEndpointsConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct HttpEndpointsConfig {
    pub chat_completions: Option<EndpointToggle>,
    pub responses: Option<EndpointToggle>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct EndpointToggle {
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct GatewayNodesConfig {
    pub allow_commands: Option<Vec<String>>,
    pub deny_commands: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ControlUiConfig {
    pub enabled: Option<bool>,
    pub base_path: Option<String>,
}

// ── Skills / Plugins ──

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct SkillsConfig {
    pub install: Option<SkillsInstallConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct SkillsInstallConfig {
    pub node_manager: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct PluginsConfig {
    pub entries: Option<HashMap<String, PluginEntry>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct PluginEntry {
    pub enabled: Option<bool>,
}

// ── Cron ──

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct CronConfig {
    pub jobs: Option<Vec<CronJobConfig>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct CronJobConfig {
    pub id: Option<String>,
    pub name: Option<String>,
    pub schedule: Option<String>,
    pub enabled: Option<bool>,
    pub kind: Option<String>,
    pub prompt: Option<String>,
    pub session_target: Option<String>,
    pub channel: Option<String>,
    pub to: Option<String>,
}

// ── Memory ──

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct MemoryConfig {
    pub enabled: Option<bool>,
    pub provider: Option<String>,
    pub embedding_model: Option<String>,
}

// ── Tools ──

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ToolsConfig {
    pub deny: Option<Vec<String>>,
    pub allow: Option<Vec<String>>,
    pub also_allow: Option<Vec<String>>,
}

// ── Hooks ──

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct HooksConfig {
    pub module: Option<String>,
    pub paths: Option<Vec<String>>,
}

// ── Browser ──

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct BrowserConfig {
    pub enabled: Option<bool>,
    pub headless: Option<bool>,
}

// ── Session ──

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct SessionConfig {
    pub max_history: Option<u32>,
    pub ttl_hours: Option<u32>,
}

// ── Broadcast ──

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct BroadcastConfig {
    pub enabled: Option<bool>,
}

// ── Discovery ──

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct DiscoveryConfig {
    pub mdns: Option<MdnsConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct MdnsConfig {
    pub mode: Option<String>,
}

// ── Node Host ──

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct NodeHostConfig {
    pub enabled: Option<bool>,
}

// ── UI ──

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct UiConfig {
    pub seam_color: Option<String>,
    pub assistant: Option<AssistantUiConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct AssistantUiConfig {
    pub name: Option<String>,
    pub avatar: Option<String>,
}

// ── Logging ──

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct LoggingConfig {
    pub level: Option<String>,
    pub file: Option<String>,
}

// ── Approvals ──

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ApprovalsConfig {
    pub mode: Option<String>,
}

impl OpenClawConfig {
    /// Get the primary model string (e.g., "anthropic/claude-opus-4-6").
    pub fn primary_model(&self) -> Option<&str> {
        self.agents.as_ref()
            .and_then(|a| a.defaults.as_ref())
            .and_then(|d| d.model.as_ref())
            .and_then(|m| m.primary.as_deref())
    }

    /// Get the workspace directory.
    pub fn workspace_dir(&self) -> Option<&str> {
        self.agents.as_ref()
            .and_then(|a| a.defaults.as_ref())
            .and_then(|d| d.workspace.as_deref())
    }

    /// Parse provider/model from a model string like "anthropic/claude-opus-4-6".
    pub fn parse_model_id(model_str: &str) -> (String, String) {
        if let Some(idx) = model_str.find('/') {
            (model_str[..idx].to_string(), model_str[idx + 1..].to_string())
        } else {
            ("anthropic".to_string(), model_str.to_string())
        }
    }

    /// Check if a plugin is enabled.
    pub fn is_plugin_enabled(&self, name: &str) -> bool {
        self.plugins.as_ref()
            .and_then(|p| p.entries.as_ref())
            .and_then(|e| e.get(name))
            .and_then(|entry| entry.enabled)
            .unwrap_or(false)
    }
}
