# 🦀 RustyClaw — High-Performance AI Assistant Gateway

A Rust implementation of the [OpenClaw](https://github.com/openclaw/openclaw) personal AI assistant gateway, forked from [frankclaw](https://github.com/frankhli843/frankclaw).

## Why Rust?

- **⚡ Performance** — Zero-cost abstractions, no garbage collector, minimal memory footprint
- **🔒 Safety** — Memory safety guaranteed at compile time, no null pointer exceptions
- **🦀 Reliability** — Rich type system catches bugs before runtime
- **📦 Single Binary** — No Node.js runtime dependency, just one static binary

## Features

Ported from frankclaw/OpenClaw:

- **Gateway Server** — axum-based HTTP server with REST + WebSocket (JSON-RPC), token auth, CORS
- **Anthropic Provider** — Claude Messages API with streaming SSE, tool_use, thinking blocks
- **Session Management** — In-memory sessions with LRU eviction, message history, context injection
- **Channel Plugins** — WhatsApp with allowFrom, groupPolicy, requireMention, debounce
- **Tool System** — Registry with deny/allow policy, builtin tools (Read/Write/Edit/exec)
- **Cron System** — Job scheduling with interval + cron expressions, async tick loop
- **Memory Search** — Text search across memory/ and knowledge/ directories
- **Core Utilities** — E.164 normalization, WhatsApp JID conversion, path resolution, UTF-16 safe string ops
- **Markdown → WhatsApp** — Converts standard Markdown to WhatsApp-compatible formatting
- **Security** — Constant-time secret comparison, injection detection, homoglyph normalization
- **CLI** — Clap-based CLI with gateway start/stop/status, config show/validate/edit
- **Config** — Full OpenClaw config parsing (agents, models, channels, cron, memory, tools, hooks)

## Install

### Pre-built binaries

Download the latest release for your platform from [GitHub Releases](https://github.com/frankhli843/rustyclaw/releases):

| Platform | Binary |
|----------|--------|
| Linux x86_64 | `rustyclaw-linux-x86_64.tar.gz` |
| Linux aarch64 (Raspberry Pi) | `rustyclaw-linux-aarch64.tar.gz` |
| macOS x86_64 | `rustyclaw-macos-x86_64.tar.gz` |
| macOS Apple Silicon | `rustyclaw-macos-aarch64.tar.gz` |

```bash
# Example: download and install on Linux
tar xzf rustyclaw-linux-x86_64.tar.gz
sudo mv rustyclaw /usr/local/bin/
```

### From source

```bash
cargo install --path .

# Or just build
cargo build --release
```

## Usage

```bash
rustyclaw --help
rustyclaw version
rustyclaw gateway start
rustyclaw gateway status
rustyclaw config show
```

## Development

```bash
# Run tests
cargo test

# Build release
cargo build --release

# Run with verbose output
cargo run -- --verbose
```

## Architecture

```
src/
├── cli/              # CLI (clap), parse_duration, parse_bytes
├── config/           # Configuration loading and full type definitions
├── provider/         # Anthropic Claude API provider with streaming
├── gateway/          # axum HTTP server, WebSocket, auth middleware
├── session/          # Session management with LRU eviction
├── channel/          # Channel plugins (WhatsApp)
├── tools/            # Tool registry and builtin executors
├── cron_system/      # Cron job scheduling and execution
├── memory/           # Memory/knowledge file search
├── markdown/         # Markdown conversion (WhatsApp formatting)
├── security/         # Secret comparison, external content protection
├── polls.rs          # Poll input normalization
├── utils.rs          # Core utilities (E.164, JID, paths, UTF-16)
├── version.rs        # Version from Cargo.toml
├── lib.rs            # Library root
└── main.rs           # Binary entry point
```

## Test Coverage

154 tests covering:
- Path normalization and resolution
- WhatsApp number/JID conversion
- Markdown to WhatsApp conversion
- Poll validation
- Duration and byte size parsing
- Security: constant-time comparison, injection detection, content wrapping
- Configuration parsing

## License

MIT — same as OpenClaw.
