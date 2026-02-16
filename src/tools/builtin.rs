use crate::provider::types::ToolDefinition;

/// All built-in tool definitions (matching OpenClaw's core tools).
pub fn all_builtin_tools() -> Vec<ToolDefinition> {
    vec![
        read_tool(),
        write_tool(),
        edit_tool(),
        exec_tool(),
        web_search_tool(),
        web_fetch_tool(),
        memory_search_tool(),
        message_tool(),
    ]
}

fn read_tool() -> ToolDefinition {
    ToolDefinition {
        name: "Read".into(),
        description: "Read the contents of a file.".into(),
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "file_path": { "type": "string", "description": "Path to the file to read" },
                "offset": { "type": "number", "description": "Line number to start reading from (1-indexed)" },
                "limit": { "type": "number", "description": "Maximum number of lines to read" }
            },
            "required": ["file_path"]
        }),
    }
}

fn write_tool() -> ToolDefinition {
    ToolDefinition {
        name: "Write".into(),
        description: "Write content to a file. Creates the file if it doesn't exist, overwrites if it does.".into(),
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "file_path": { "type": "string", "description": "Path to the file to write" },
                "content": { "type": "string", "description": "Content to write to the file" }
            },
            "required": ["file_path", "content"]
        }),
    }
}

fn edit_tool() -> ToolDefinition {
    ToolDefinition {
        name: "Edit".into(),
        description: "Edit a file by replacing exact text.".into(),
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "file_path": { "type": "string", "description": "Path to the file to edit" },
                "old_string": { "type": "string", "description": "Exact text to find and replace" },
                "new_string": { "type": "string", "description": "New text to replace with" }
            },
            "required": ["file_path", "old_string", "new_string"]
        }),
    }
}

fn exec_tool() -> ToolDefinition {
    ToolDefinition {
        name: "exec".into(),
        description: "Execute shell commands.".into(),
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "command": { "type": "string", "description": "Shell command to execute" },
                "workdir": { "type": "string", "description": "Working directory" },
                "timeout": { "type": "number", "description": "Timeout in seconds" }
            },
            "required": ["command"]
        }),
    }
}

fn web_search_tool() -> ToolDefinition {
    ToolDefinition {
        name: "web_search".into(),
        description: "Search the web.".into(),
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "query": { "type": "string", "description": "Search query" },
                "count": { "type": "number", "description": "Number of results" }
            },
            "required": ["query"]
        }),
    }
}

fn web_fetch_tool() -> ToolDefinition {
    ToolDefinition {
        name: "web_fetch".into(),
        description: "Fetch and extract readable content from a URL.".into(),
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "url": { "type": "string", "description": "URL to fetch" },
                "extractMode": { "type": "string", "enum": ["markdown", "text"] }
            },
            "required": ["url"]
        }),
    }
}

fn memory_search_tool() -> ToolDefinition {
    ToolDefinition {
        name: "memory_search".into(),
        description: "Search memory and knowledge files.".into(),
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "query": { "type": "string", "description": "Search query" },
                "limit": { "type": "number", "description": "Max results" }
            },
            "required": ["query"]
        }),
    }
}

fn message_tool() -> ToolDefinition {
    ToolDefinition {
        name: "message".into(),
        description: "Send messages via channel plugins.".into(),
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "action": { "type": "string", "enum": ["send", "react", "poll"] },
                "target": { "type": "string", "description": "Target channel/user" },
                "message": { "type": "string", "description": "Message text" }
            },
            "required": ["action"]
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_builtins_have_names() {
        let tools = all_builtin_tools();
        assert!(tools.len() >= 8);
        for tool in &tools {
            assert!(!tool.name.is_empty());
            assert!(!tool.description.is_empty());
        }
    }

    #[test]
    fn tool_names_are_unique() {
        let tools = all_builtin_tools();
        let names: std::collections::HashSet<_> = tools.iter().map(|t| &t.name).collect();
        assert_eq!(names.len(), tools.len());
    }
}
