pub mod executor;
pub mod builtin;

use crate::provider::types::ToolDefinition;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Result of executing a tool.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    pub content: String,
    pub is_error: bool,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Tool registry — stores available tools and their definitions.
#[derive(Clone)]
pub struct ToolRegistry {
    tools: Arc<RwLock<HashMap<String, RegisteredTool>>>,
    deny_list: Vec<String>,
    allow_list: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct RegisteredTool {
    pub definition: ToolDefinition,
    pub category: ToolCategory,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ToolCategory {
    Builtin,
    Skill,
    Plugin,
    Custom,
}

impl ToolRegistry {
    pub fn new() -> Self {
        let registry = Self {
            tools: Arc::new(RwLock::new(HashMap::new())),
            deny_list: Vec::new(),
            allow_list: Vec::new(),
        };
        registry
    }

    /// Create with deny/allow lists from config.
    pub fn with_policy(deny: Vec<String>, allow: Vec<String>) -> Self {
        Self {
            tools: Arc::new(RwLock::new(HashMap::new())),
            deny_list: deny,
            allow_list: allow,
        }
    }

    /// Register a tool.
    pub async fn register(&self, tool: RegisteredTool) {
        let mut tools = self.tools.write().await;
        tools.insert(tool.definition.name.clone(), tool);
    }

    /// Get a tool definition by name.
    pub async fn get(&self, name: &str) -> Option<RegisteredTool> {
        let tools = self.tools.read().await;
        tools.get(name).cloned()
    }

    /// List all tool definitions (respecting deny/allow policy).
    pub async fn list_definitions(&self) -> Vec<ToolDefinition> {
        let tools = self.tools.read().await;
        tools.values()
            .filter(|t| self.is_allowed(&t.definition.name))
            .map(|t| t.definition.clone())
            .collect()
    }

    /// Check if a tool name is allowed by policy.
    pub fn is_allowed(&self, name: &str) -> bool {
        if self.deny_list.contains(&name.to_string()) {
            return self.allow_list.contains(&name.to_string());
        }
        true
    }

    /// Register all builtin tools.
    pub async fn register_builtins(&self) {
        for tool in builtin::all_builtin_tools() {
            self.register(RegisteredTool {
                definition: tool,
                category: ToolCategory::Builtin,
            }).await;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn register_and_get_tool() {
        let registry = ToolRegistry::new();
        registry.register(RegisteredTool {
            definition: ToolDefinition {
                name: "test_tool".into(),
                description: "A test tool".into(),
                input_schema: serde_json::json!({"type": "object"}),
            },
            category: ToolCategory::Custom,
        }).await;

        let tool = registry.get("test_tool").await;
        assert!(tool.is_some());
        assert_eq!(tool.unwrap().definition.name, "test_tool");
    }

    #[tokio::test]
    async fn deny_list() {
        let registry = ToolRegistry::with_policy(
            vec!["dangerous".into()],
            vec![],
        );
        assert!(!registry.is_allowed("dangerous"));
        assert!(registry.is_allowed("safe"));
    }

    #[tokio::test]
    async fn deny_with_allow_override() {
        let registry = ToolRegistry::with_policy(
            vec!["tool_a".into()],
            vec!["tool_a".into()],
        );
        assert!(registry.is_allowed("tool_a"));
    }

    #[tokio::test]
    async fn list_definitions() {
        let registry = ToolRegistry::with_policy(vec!["blocked".into()], vec![]);
        registry.register(RegisteredTool {
            definition: ToolDefinition {
                name: "allowed".into(),
                description: "OK".into(),
                input_schema: serde_json::json!({}),
            },
            category: ToolCategory::Builtin,
        }).await;
        registry.register(RegisteredTool {
            definition: ToolDefinition {
                name: "blocked".into(),
                description: "NO".into(),
                input_schema: serde_json::json!({}),
            },
            category: ToolCategory::Builtin,
        }).await;
        let defs = registry.list_definitions().await;
        assert_eq!(defs.len(), 1);
        assert_eq!(defs[0].name, "allowed");
    }
}
