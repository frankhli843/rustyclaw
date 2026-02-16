use std::path::Path;
use walkdir::WalkDir;

/// A memory search result.
#[derive(Debug, Clone)]
pub struct SearchResult {
    pub file_path: String,
    pub content: String,
    pub score: f64,
    pub line_number: Option<usize>,
}

/// Simple text-based memory search (grep-style).
/// In a full implementation, this would use vector embeddings.
pub fn search_memory(
    workspace_dir: &str,
    query: &str,
    limit: usize,
) -> Vec<SearchResult> {
    let mut results = Vec::new();
    let query_lower = query.to_lowercase();
    let query_terms: Vec<&str> = query_lower.split_whitespace().collect();

    if query_terms.is_empty() {
        return results;
    }

    let search_dirs = vec![
        Path::new(workspace_dir).join("memory"),
        Path::new(workspace_dir).join("knowledge"),
    ];

    for search_dir in &search_dirs {
        if !search_dir.exists() {
            continue;
        }

        for entry in WalkDir::new(search_dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
        {
            let path = entry.path();
            let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
            if !matches!(ext, "md" | "txt" | "json" | "yaml" | "yml") {
                continue;
            }

            if let Ok(content) = std::fs::read_to_string(path) {
                let rel_path = path.strip_prefix(workspace_dir)
                    .unwrap_or(path)
                    .to_string_lossy()
                    .to_string();

                // Search by chunks (paragraphs)
                for (chunk_idx, chunk) in content.split("\n\n").enumerate() {
                    let chunk_lower = chunk.to_lowercase();
                    let matching_terms = query_terms.iter()
                        .filter(|term| chunk_lower.contains(*term))
                        .count();

                    if matching_terms > 0 {
                        let score = matching_terms as f64 / query_terms.len() as f64;
                        // Prepend file path to chunk for context (matching frankclaw behavior)
                        let content_with_path = format!("[{}]\n{}", rel_path, chunk.trim());

                        results.push(SearchResult {
                            file_path: rel_path.clone(),
                            content: content_with_path,
                            score,
                            line_number: Some(chunk_idx + 1),
                        });
                    }
                }
            }
        }
    }

    // Sort by score descending
    results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
    results.truncate(limit);
    results
}

/// List workspace context files (AGENTS.md, SOUL.md, etc.).
pub fn list_workspace_context_files(workspace_dir: &str) -> Vec<(String, String)> {
    let context_files = [
        "AGENTS.md",
        "SOUL.md",
        "USER.md",
        "TOOLS.md",
        "MEMORY.md",
        "HEARTBEAT.md",
        "IDENTITY.md",
    ];

    let mut results = Vec::new();
    for filename in &context_files {
        let path = Path::new(workspace_dir).join(filename);
        if path.exists() {
            if let Ok(content) = std::fs::read_to_string(&path) {
                results.push((filename.to_string(), content));
            }
        }
    }
    results
}

/// Load today's memory file.
pub fn load_today_memory(workspace_dir: &str) -> Option<String> {
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    let path = Path::new(workspace_dir).join("memory").join(format!("{}.md", today));
    std::fs::read_to_string(&path).ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn setup_workspace() -> TempDir {
        let dir = TempDir::new().unwrap();
        let memory_dir = dir.path().join("memory");
        std::fs::create_dir_all(&memory_dir).unwrap();

        std::fs::write(
            memory_dir.join("2024-01-01.md"),
            "# January 1\n\nFrank went to the store.\n\nHe bought apples and oranges.\n\nThe weather was cold.",
        ).unwrap();

        let knowledge_dir = dir.path().join("knowledge");
        std::fs::create_dir_all(&knowledge_dir).unwrap();
        std::fs::write(
            knowledge_dir.join("test.md"),
            "# Test Knowledge\n\nRust is a programming language.\n\nIt focuses on safety and performance.",
        ).unwrap();

        // Context files
        std::fs::write(dir.path().join("AGENTS.md"), "# Agent Config\nTest agent").unwrap();

        dir
    }

    #[test]
    fn search_finds_matching_content() {
        let dir = setup_workspace();
        let results = search_memory(dir.path().to_str().unwrap(), "apples oranges", 10);
        assert!(!results.is_empty());
        assert!(results[0].content.contains("apples"));
    }

    #[test]
    fn search_returns_file_path_prefix() {
        let dir = setup_workspace();
        let results = search_memory(dir.path().to_str().unwrap(), "store", 10);
        assert!(!results.is_empty());
        assert!(results[0].content.starts_with("[memory/"));
    }

    #[test]
    fn search_respects_limit() {
        let dir = setup_workspace();
        let results = search_memory(dir.path().to_str().unwrap(), "the", 1);
        assert!(results.len() <= 1);
    }

    #[test]
    fn search_empty_query() {
        let dir = setup_workspace();
        let results = search_memory(dir.path().to_str().unwrap(), "", 10);
        assert!(results.is_empty());
    }

    #[test]
    fn search_no_results() {
        let dir = setup_workspace();
        let results = search_memory(dir.path().to_str().unwrap(), "xyznonexistent", 10);
        assert!(results.is_empty());
    }

    #[test]
    fn list_context_files() {
        let dir = setup_workspace();
        let files = list_workspace_context_files(dir.path().to_str().unwrap());
        assert_eq!(files.len(), 1); // Only AGENTS.md exists
        assert_eq!(files[0].0, "AGENTS.md");
    }

    #[test]
    fn search_knowledge_dir() {
        let dir = setup_workspace();
        let results = search_memory(dir.path().to_str().unwrap(), "Rust programming", 10);
        assert!(!results.is_empty());
        assert!(results[0].file_path.contains("knowledge"));
    }
}
