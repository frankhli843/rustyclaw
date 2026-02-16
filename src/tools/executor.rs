use super::ToolResult;
use std::collections::HashMap;
use std::path::Path;
use std::process::Stdio;
use tokio::process::Command;
use tracing::debug;

/// Execute a tool call by name with given input.
pub async fn execute_tool(
    name: &str,
    input: &serde_json::Value,
    workspace_dir: &str,
) -> ToolResult {
    match name {
        "Read" => execute_read(input, workspace_dir).await,
        "Write" => execute_write(input, workspace_dir).await,
        "Edit" => execute_edit(input, workspace_dir).await,
        "exec" => execute_exec(input, workspace_dir).await,
        _ => ToolResult {
            content: format!("Unknown tool: {}", name),
            is_error: true,
            metadata: HashMap::new(),
        },
    }
}

async fn execute_read(input: &serde_json::Value, workspace_dir: &str) -> ToolResult {
    let file_path = input.get("file_path")
        .or_else(|| input.get("path"))
        .and_then(|v| v.as_str())
        .unwrap_or("");

    if file_path.is_empty() {
        return ToolResult {
            content: "file_path is required".into(),
            is_error: true,
            metadata: HashMap::new(),
        };
    }

    let resolved = resolve_path(file_path, workspace_dir);
    let offset = input.get("offset").and_then(|v| v.as_u64()).unwrap_or(1) as usize;
    let limit = input.get("limit").and_then(|v| v.as_u64()).unwrap_or(2000) as usize;

    match tokio::fs::read_to_string(&resolved).await {
        Ok(content) => {
            let lines: Vec<&str> = content.lines().collect();
            let start = (offset.saturating_sub(1)).min(lines.len());
            let end = (start + limit).min(lines.len());
            let selected = lines[start..end].join("\n");

            let mut metadata = HashMap::new();
            metadata.insert("total_lines".into(), serde_json::json!(lines.len()));
            metadata.insert("returned_lines".into(), serde_json::json!(end - start));

            ToolResult {
                content: selected,
                is_error: false,
                metadata,
            }
        }
        Err(e) => ToolResult {
            content: format!("Error reading {}: {}", file_path, e),
            is_error: true,
            metadata: HashMap::new(),
        },
    }
}

async fn execute_write(input: &serde_json::Value, workspace_dir: &str) -> ToolResult {
    let file_path = input.get("file_path")
        .or_else(|| input.get("path"))
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let content = input.get("content").and_then(|v| v.as_str()).unwrap_or("");

    if file_path.is_empty() {
        return ToolResult {
            content: "file_path is required".into(),
            is_error: true,
            metadata: HashMap::new(),
        };
    }

    let resolved = resolve_path(file_path, workspace_dir);

    // Create parent directories
    if let Some(parent) = Path::new(&resolved).parent() {
        if let Err(e) = tokio::fs::create_dir_all(parent).await {
            return ToolResult {
                content: format!("Error creating directories: {}", e),
                is_error: true,
                metadata: HashMap::new(),
            };
        }
    }

    match tokio::fs::write(&resolved, content).await {
        Ok(_) => ToolResult {
            content: format!("Successfully wrote {} bytes to {}", content.len(), file_path),
            is_error: false,
            metadata: HashMap::new(),
        },
        Err(e) => ToolResult {
            content: format!("Error writing {}: {}", file_path, e),
            is_error: true,
            metadata: HashMap::new(),
        },
    }
}

async fn execute_edit(input: &serde_json::Value, workspace_dir: &str) -> ToolResult {
    let file_path = input.get("file_path")
        .or_else(|| input.get("path"))
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let old_string = input.get("old_string")
        .or_else(|| input.get("oldText"))
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let new_string = input.get("new_string")
        .or_else(|| input.get("newText"))
        .and_then(|v| v.as_str())
        .unwrap_or("");

    if file_path.is_empty() || old_string.is_empty() {
        return ToolResult {
            content: "file_path and old_string are required".into(),
            is_error: true,
            metadata: HashMap::new(),
        };
    }

    let resolved = resolve_path(file_path, workspace_dir);

    match tokio::fs::read_to_string(&resolved).await {
        Ok(content) => {
            if !content.contains(old_string) {
                return ToolResult {
                    content: "old_string not found in file".into(),
                    is_error: true,
                    metadata: HashMap::new(),
                };
            }
            let new_content = content.replacen(old_string, new_string, 1);
            match tokio::fs::write(&resolved, &new_content).await {
                Ok(_) => ToolResult {
                    content: format!("Successfully edited {}", file_path),
                    is_error: false,
                    metadata: HashMap::new(),
                },
                Err(e) => ToolResult {
                    content: format!("Error writing {}: {}", file_path, e),
                    is_error: true,
                    metadata: HashMap::new(),
                },
            }
        }
        Err(e) => ToolResult {
            content: format!("Error reading {}: {}", file_path, e),
            is_error: true,
            metadata: HashMap::new(),
        },
    }
}

async fn execute_exec(input: &serde_json::Value, workspace_dir: &str) -> ToolResult {
    let command = input.get("command").and_then(|v| v.as_str()).unwrap_or("");
    let workdir = input.get("workdir").and_then(|v| v.as_str()).unwrap_or(workspace_dir);
    let timeout_secs = input.get("timeout").and_then(|v| v.as_u64()).unwrap_or(30);

    if command.is_empty() {
        return ToolResult {
            content: "command is required".into(),
            is_error: true,
            metadata: HashMap::new(),
        };
    }

    debug!("Executing: {} in {}", command, workdir);

    let result = tokio::time::timeout(
        std::time::Duration::from_secs(timeout_secs),
        Command::new("bash")
            .arg("-c")
            .arg(command)
            .current_dir(workdir)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
    ).await;

    match result {
        Ok(Ok(output)) => {
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            let exit_code = output.status.code().unwrap_or(-1);

            let content = if stderr.is_empty() {
                stdout
            } else if stdout.is_empty() {
                stderr
            } else {
                format!("{}\n{}", stdout, stderr)
            };

            let is_error = exit_code != 0;
            let mut metadata = HashMap::new();
            metadata.insert("exit_code".into(), serde_json::json!(exit_code));

            ToolResult { content, is_error, metadata }
        }
        Ok(Err(e)) => ToolResult {
            content: format!("Error executing command: {}", e),
            is_error: true,
            metadata: HashMap::new(),
        },
        Err(_) => ToolResult {
            content: format!("Command timed out after {}s", timeout_secs),
            is_error: true,
            metadata: HashMap::new(),
        },
    }
}

fn resolve_path(path: &str, workspace_dir: &str) -> String {
    if Path::new(path).is_absolute() {
        path.to_string()
    } else {
        Path::new(workspace_dir).join(path).to_string_lossy().to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn execute_read_file() {
        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("test.txt");
        std::fs::write(&file_path, "line1\nline2\nline3").unwrap();

        let input = serde_json::json!({
            "file_path": file_path.to_str().unwrap()
        });
        let result = execute_read(&input, dir.path().to_str().unwrap()).await;
        assert!(!result.is_error);
        assert!(result.content.contains("line1"));
    }

    #[tokio::test]
    async fn execute_read_with_offset_limit() {
        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("test.txt");
        std::fs::write(&file_path, "line1\nline2\nline3\nline4").unwrap();

        let input = serde_json::json!({
            "file_path": file_path.to_str().unwrap(),
            "offset": 2,
            "limit": 2
        });
        let result = execute_read(&input, dir.path().to_str().unwrap()).await;
        assert!(!result.is_error);
        assert!(result.content.contains("line2"));
        assert!(result.content.contains("line3"));
        assert!(!result.content.contains("line1"));
    }

    #[tokio::test]
    async fn execute_write_file() {
        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("output.txt");

        let input = serde_json::json!({
            "file_path": file_path.to_str().unwrap(),
            "content": "hello world"
        });
        let result = execute_write(&input, dir.path().to_str().unwrap()).await;
        assert!(!result.is_error);
        assert_eq!(std::fs::read_to_string(&file_path).unwrap(), "hello world");
    }

    #[tokio::test]
    async fn execute_write_creates_dirs() {
        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("sub/dir/file.txt");

        let input = serde_json::json!({
            "file_path": file_path.to_str().unwrap(),
            "content": "nested"
        });
        let result = execute_write(&input, dir.path().to_str().unwrap()).await;
        assert!(!result.is_error);
    }

    #[tokio::test]
    async fn execute_edit_file() {
        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("edit.txt");
        std::fs::write(&file_path, "hello world").unwrap();

        let input = serde_json::json!({
            "file_path": file_path.to_str().unwrap(),
            "old_string": "world",
            "new_string": "rust"
        });
        let result = execute_edit(&input, dir.path().to_str().unwrap()).await;
        assert!(!result.is_error);
        assert_eq!(std::fs::read_to_string(&file_path).unwrap(), "hello rust");
    }

    #[tokio::test]
    async fn execute_edit_not_found() {
        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("edit.txt");
        std::fs::write(&file_path, "hello world").unwrap();

        let input = serde_json::json!({
            "file_path": file_path.to_str().unwrap(),
            "old_string": "nonexistent",
            "new_string": "replacement"
        });
        let result = execute_edit(&input, dir.path().to_str().unwrap()).await;
        assert!(result.is_error);
    }

    #[tokio::test]
    async fn execute_exec_command() {
        let dir = tempfile::tempdir().unwrap();
        let input = serde_json::json!({
            "command": "echo hello"
        });
        let result = execute_exec(&input, dir.path().to_str().unwrap()).await;
        assert!(!result.is_error);
        assert!(result.content.contains("hello"));
    }

    #[tokio::test]
    async fn execute_exec_failure() {
        let dir = tempfile::tempdir().unwrap();
        let input = serde_json::json!({
            "command": "false"
        });
        let result = execute_exec(&input, dir.path().to_str().unwrap()).await;
        assert!(result.is_error);
    }

    #[tokio::test]
    async fn execute_unknown_tool() {
        let result = execute_tool("nonexistent", &serde_json::json!({}), "/tmp").await;
        assert!(result.is_error);
        assert!(result.content.contains("Unknown tool"));
    }

    #[test]
    fn resolve_path_absolute() {
        assert_eq!(resolve_path("/tmp/file.txt", "/workspace"), "/tmp/file.txt");
    }

    #[test]
    fn resolve_path_relative() {
        let p = resolve_path("file.txt", "/workspace");
        assert_eq!(p, "/workspace/file.txt");
    }
}
