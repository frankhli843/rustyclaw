use crate::provider::types::{Message, MessageRole, MessageContent, ContentBlock};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// A conversation session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    pub key: String,
    pub agent_id: String,
    pub channel: String,
    pub messages: Vec<Message>,
    pub system_prompt: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub metadata: HashMap<String, String>,
    pub context_files: Vec<ContextFile>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextFile {
    pub path: String,
    pub content: String,
    pub label: String,
}

impl Session {
    pub fn new(key: &str, agent_id: &str, channel: &str) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            key: key.to_string(),
            agent_id: agent_id.to_string(),
            channel: channel.to_string(),
            messages: Vec::new(),
            system_prompt: None,
            created_at: now,
            updated_at: now,
            metadata: HashMap::new(),
            context_files: Vec::new(),
        }
    }

    pub fn add_user_message(&mut self, text: &str) {
        self.messages.push(Message {
            role: MessageRole::User,
            content: MessageContent::Text(text.to_string()),
        });
        self.updated_at = Utc::now();
    }

    pub fn add_assistant_message(&mut self, text: &str) {
        self.messages.push(Message {
            role: MessageRole::Assistant,
            content: MessageContent::Text(text.to_string()),
        });
        self.updated_at = Utc::now();
    }

    pub fn add_tool_result(&mut self, tool_use_id: &str, content: &str, is_error: bool) {
        self.messages.push(Message {
            role: MessageRole::User,
            content: MessageContent::Blocks(vec![ContentBlock::ToolResult {
                tool_use_id: tool_use_id.to_string(),
                content: content.to_string(),
                is_error: if is_error { Some(true) } else { None },
            }]),
        });
        self.updated_at = Utc::now();
    }

    pub fn add_assistant_tool_use(&mut self, blocks: Vec<ContentBlock>) {
        self.messages.push(Message {
            role: MessageRole::Assistant,
            content: MessageContent::Blocks(blocks),
        });
        self.updated_at = Utc::now();
    }

    /// Get the last assistant text response.
    pub fn last_assistant_text(&self) -> Option<String> {
        self.messages.iter().rev().find_map(|m| {
            if m.role == MessageRole::Assistant {
                Some(m.content.to_text())
            } else {
                None
            }
        })
    }

    /// Count total messages.
    pub fn message_count(&self) -> usize {
        self.messages.len()
    }

    /// Approximate token count (rough: 4 chars ≈ 1 token).
    pub fn approximate_tokens(&self) -> usize {
        let mut total = 0;
        if let Some(sp) = &self.system_prompt {
            total += sp.len() / 4;
        }
        for msg in &self.messages {
            total += msg.content.to_text().len() / 4;
        }
        total
    }
}

/// Session key format: "agent:<agent_id>:<channel>:<chat_id>"
pub fn build_session_key(agent_id: &str, channel: &str, chat_id: &str) -> String {
    format!("agent:{}:{}:{}", agent_id, channel, chat_id)
}

/// Session manager — stores active sessions in memory.
#[derive(Clone)]
pub struct SessionManager {
    sessions: Arc<RwLock<HashMap<String, Session>>>,
    max_sessions: usize,
}

impl SessionManager {
    pub fn new(max_sessions: usize) -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            max_sessions,
        }
    }

    /// Get or create a session for the given key.
    pub async fn get_or_create(&self, key: &str, agent_id: &str, channel: &str) -> Session {
        {
            let sessions = self.sessions.read().await;
            if let Some(session) = sessions.get(key) {
                return session.clone();
            }
        }

        let session = Session::new(key, agent_id, channel);
        let mut sessions = self.sessions.write().await;

        // Evict oldest if at capacity
        if sessions.len() >= self.max_sessions {
            if let Some(oldest_key) = sessions.iter()
                .min_by_key(|(_, s)| s.updated_at)
                .map(|(k, _)| k.clone())
            {
                sessions.remove(&oldest_key);
            }
        }

        sessions.insert(key.to_string(), session.clone());
        session
    }

    /// Update a session.
    pub async fn update(&self, session: &Session) {
        let mut sessions = self.sessions.write().await;
        sessions.insert(session.key.clone(), session.clone());
    }

    /// Get a session by key.
    pub async fn get(&self, key: &str) -> Option<Session> {
        let sessions = self.sessions.read().await;
        sessions.get(key).cloned()
    }

    /// Remove a session.
    pub async fn remove(&self, key: &str) -> Option<Session> {
        let mut sessions = self.sessions.write().await;
        sessions.remove(key)
    }

    /// List all session keys.
    pub async fn list_keys(&self) -> Vec<String> {
        let sessions = self.sessions.read().await;
        sessions.keys().cloned().collect()
    }

    /// Count active sessions.
    pub async fn count(&self) -> usize {
        let sessions = self.sessions.read().await;
        sessions.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn session_creation() {
        let session = Session::new("test-key", "main", "whatsapp");
        assert_eq!(session.key, "test-key");
        assert_eq!(session.agent_id, "main");
        assert_eq!(session.channel, "whatsapp");
        assert_eq!(session.message_count(), 0);
    }

    #[test]
    fn session_messages() {
        let mut session = Session::new("k", "main", "wa");
        session.add_user_message("hello");
        session.add_assistant_message("hi there");
        assert_eq!(session.message_count(), 2);
        assert_eq!(session.last_assistant_text(), Some("hi there".into()));
    }

    #[test]
    fn session_tool_result() {
        let mut session = Session::new("k", "main", "wa");
        session.add_tool_result("tu_1", "file contents", false);
        assert_eq!(session.message_count(), 1);
    }

    #[test]
    fn build_session_key_format() {
        let key = build_session_key("main", "whatsapp", "123@g.us");
        assert_eq!(key, "agent:main:whatsapp:123@g.us");
    }

    #[test]
    fn approximate_tokens() {
        let mut session = Session::new("k", "main", "wa");
        session.system_prompt = Some("a".repeat(400));
        session.add_user_message(&"b".repeat(400));
        // 400/4 + 400/4 = 200
        assert_eq!(session.approximate_tokens(), 200);
    }

    #[tokio::test]
    async fn session_manager_get_or_create() {
        let mgr = SessionManager::new(100);
        let s1 = mgr.get_or_create("k1", "main", "wa").await;
        assert_eq!(s1.key, "k1");
        let s2 = mgr.get_or_create("k1", "main", "wa").await;
        assert_eq!(s1.id, s2.id); // Same session
    }

    #[tokio::test]
    async fn session_manager_eviction() {
        let mgr = SessionManager::new(2);
        mgr.get_or_create("k1", "main", "wa").await;
        mgr.get_or_create("k2", "main", "wa").await;
        mgr.get_or_create("k3", "main", "wa").await;
        assert_eq!(mgr.count().await, 2);
    }

    #[tokio::test]
    async fn session_manager_update_and_get() {
        let mgr = SessionManager::new(100);
        let mut s = mgr.get_or_create("k1", "main", "wa").await;
        s.add_user_message("hello");
        mgr.update(&s).await;
        let s2 = mgr.get("k1").await.unwrap();
        assert_eq!(s2.message_count(), 1);
    }

    #[tokio::test]
    async fn session_manager_remove() {
        let mgr = SessionManager::new(100);
        mgr.get_or_create("k1", "main", "wa").await;
        assert_eq!(mgr.count().await, 1);
        mgr.remove("k1").await;
        assert_eq!(mgr.count().await, 0);
    }
}
