use super::{ChannelError, ChannelPlugin, IncomingMessage, OutgoingMessage};
use crate::config::WhatsAppConfig;
use async_trait::async_trait;
use tracing::debug;

/// WhatsApp channel plugin.
/// Communicates via the OpenClaw WebSocket protocol to the WhatsApp bridge.
pub struct WhatsAppPlugin {
    config: WhatsAppConfig,
    connected: bool,
}

impl WhatsAppPlugin {
    pub fn new(config: WhatsAppConfig) -> Self {
        Self {
            config,
            connected: false,
        }
    }

    /// Check if a sender is allowed by the allowFrom list.
    pub fn is_sender_allowed(&self, from: &str) -> bool {
        match &self.config.allow_from {
            None => false,
            Some(allow_list) => {
                allow_list.iter().any(|allowed| {
                    if allowed == "*" {
                        return true;
                    }
                    let normalized_from = crate::utils::normalize_e164(from);
                    let normalized_allowed = crate::utils::normalize_e164(allowed);
                    normalized_from == normalized_allowed
                })
            }
        }
    }

    /// Check if a group message requires a mention based on group config.
    pub fn requires_mention(&self, group_id: &str) -> bool {
        if let Some(groups) = &self.config.groups {
            // Check specific group config first
            if let Some(group_config) = groups.get(group_id) {
                return group_config.require_mention.unwrap_or(true);
            }
            // Check wildcard
            if let Some(wildcard) = groups.get("*") {
                return wildcard.require_mention.unwrap_or(true);
            }
        }
        true // Default: require mention
    }

    /// Check if a message should be processed.
    pub fn should_process(&self, msg: &IncomingMessage) -> bool {
        // Check sender allowlist
        if !self.is_sender_allowed(&msg.from) {
            // For groups, check group policy
            if msg.is_group {
                let policy = self.config.group_policy.as_deref().unwrap_or("closed");
                if policy != "open" {
                    return false;
                }
            } else {
                let dm_policy = self.config.dm_policy.as_deref().unwrap_or("disabled");
                return dm_policy != "disabled";
            }
        }

        // For groups, check mention requirement
        if msg.is_group {
            if self.requires_mention(&msg.chat_id) && !msg.mentions_bot {
                return false;
            }
        }

        true
    }

    /// Get the debounce delay in milliseconds.
    pub fn debounce_ms(&self) -> u64 {
        self.config.debounce_ms.unwrap_or(2000)
    }
}

#[async_trait]
impl ChannelPlugin for WhatsAppPlugin {
    fn name(&self) -> &str {
        "whatsapp"
    }

    async fn send(&self, message: &OutgoingMessage) -> Result<(), ChannelError> {
        if !self.connected {
            return Err(ChannelError::NotConnected);
        }
        // In a real implementation, this would send via the WhatsApp bridge WS connection
        debug!("WhatsApp send to {}: {}", message.to, message.text);
        Ok(())
    }

    async fn react(&self, chat_id: &str, message_id: &str, emoji: &str) -> Result<(), ChannelError> {
        if !self.connected {
            return Err(ChannelError::NotConnected);
        }
        debug!("WhatsApp react {} on {} in {}", emoji, message_id, chat_id);
        Ok(())
    }

    fn is_connected(&self) -> bool {
        self.connected
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn make_config() -> WhatsAppConfig {
        WhatsAppConfig {
            dm_policy: Some("disabled".into()),
            self_chat_mode: Some(false),
            allow_from: Some(vec!["+16478023321".into()]),
            group_policy: Some("open".into()),
            groups: Some({
                let mut m = HashMap::new();
                m.insert("*".into(), crate::config::WhatsAppGroupConfig {
                    require_mention: Some(false),
                });
                m
            }),
            debounce_ms: Some(30000),
            media_max_mb: Some(50),
            phone: None,
        }
    }

    #[test]
    fn sender_allowed() {
        let plugin = WhatsAppPlugin::new(make_config());
        assert!(plugin.is_sender_allowed("+16478023321"));
        assert!(!plugin.is_sender_allowed("+1999999999"));
    }

    #[test]
    fn sender_wildcard() {
        let mut config = make_config();
        config.allow_from = Some(vec!["*".into()]);
        let plugin = WhatsAppPlugin::new(config);
        assert!(plugin.is_sender_allowed("+anything"));
    }

    #[test]
    fn requires_mention_default() {
        let plugin = WhatsAppPlugin::new(make_config());
        // Wildcard says false
        assert!(!plugin.requires_mention("somegroup@g.us"));
    }

    #[test]
    fn should_process_group_open() {
        let plugin = WhatsAppPlugin::new(make_config());
        let msg = IncomingMessage {
            id: "1".into(),
            channel: "whatsapp".into(),
            from: "+1999999999".into(), // Not in allowFrom
            chat_id: "group@g.us".into(),
            text: "hello".into(),
            timestamp: 0,
            is_group: true,
            mentions_bot: false,
            reply_to: None,
            media: None,
        };
        // Group policy is open, require_mention is false
        assert!(plugin.should_process(&msg));
    }

    #[test]
    fn should_process_dm_disabled() {
        let plugin = WhatsAppPlugin::new(make_config());
        let msg = IncomingMessage {
            id: "1".into(),
            channel: "whatsapp".into(),
            from: "+1999999999".into(),
            chat_id: "+1999999999".into(),
            text: "hello".into(),
            timestamp: 0,
            is_group: false,
            mentions_bot: false,
            reply_to: None,
            media: None,
        };
        // DM policy is disabled, sender not in allowFrom
        assert!(!plugin.should_process(&msg));
    }

    #[test]
    fn debounce_from_config() {
        let plugin = WhatsAppPlugin::new(make_config());
        assert_eq!(plugin.debounce_ms(), 30000);
    }
}
