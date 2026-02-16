pub mod whatsapp;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// An incoming message from a channel.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncomingMessage {
    pub id: String,
    pub channel: String,
    pub from: String,
    pub chat_id: String,
    pub text: String,
    pub timestamp: u64,
    pub is_group: bool,
    pub mentions_bot: bool,
    pub reply_to: Option<String>,
    pub media: Option<MediaAttachment>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaAttachment {
    pub media_type: String,
    pub url: Option<String>,
    pub data: Option<String>,
    pub filename: Option<String>,
}

/// An outgoing message to a channel.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutgoingMessage {
    pub channel: String,
    pub to: String,
    pub text: String,
    pub reply_to: Option<String>,
    pub media: Option<MediaAttachment>,
}

/// Channel plugin trait.
#[async_trait]
pub trait ChannelPlugin: Send + Sync {
    /// Channel name (e.g., "whatsapp", "telegram").
    fn name(&self) -> &str;

    /// Send a message.
    async fn send(&self, message: &OutgoingMessage) -> Result<(), ChannelError>;

    /// React to a message with an emoji.
    async fn react(&self, chat_id: &str, message_id: &str, emoji: &str) -> Result<(), ChannelError>;

    /// Check if the plugin is connected/ready.
    fn is_connected(&self) -> bool;
}

#[derive(Debug, thiserror::Error)]
pub enum ChannelError {
    #[error("Not connected")]
    NotConnected,
    #[error("Send failed: {0}")]
    SendFailed(String),
    #[error("Channel error: {0}")]
    Other(String),
}

/// Channel manager — routes messages to the appropriate channel plugin.
pub struct ChannelManager {
    plugins: Vec<Box<dyn ChannelPlugin>>,
}

impl ChannelManager {
    pub fn new() -> Self {
        Self { plugins: Vec::new() }
    }

    pub fn register(&mut self, plugin: Box<dyn ChannelPlugin>) {
        self.plugins.push(plugin);
    }

    pub fn get(&self, channel: &str) -> Option<&dyn ChannelPlugin> {
        self.plugins.iter().find(|p| p.name() == channel).map(|p| p.as_ref())
    }

    pub async fn send(&self, message: &OutgoingMessage) -> Result<(), ChannelError> {
        let plugin = self.get(&message.channel)
            .ok_or_else(|| ChannelError::Other(format!("No plugin for channel: {}", message.channel)))?;
        plugin.send(message).await
    }

    pub fn list_channels(&self) -> Vec<&str> {
        self.plugins.iter().map(|p| p.name()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn incoming_message_fields() {
        let msg = IncomingMessage {
            id: "1".into(),
            channel: "whatsapp".into(),
            from: "+1555".into(),
            chat_id: "group@g.us".into(),
            text: "hello".into(),
            timestamp: 1234567890,
            is_group: true,
            mentions_bot: false,
            reply_to: None,
            media: None,
        };
        assert!(msg.is_group);
        assert!(!msg.mentions_bot);
    }

    #[test]
    fn channel_manager_empty() {
        let mgr = ChannelManager::new();
        assert!(mgr.list_channels().is_empty());
        assert!(mgr.get("whatsapp").is_none());
    }
}
