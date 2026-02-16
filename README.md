# 🦀 RustyClaw — High-Performance AI Assistant Gateway

A Rust implementation of the [OpenClaw](https://github.com/openclaw/openclaw) personal AI assistant gateway, forked from [frankclaw](https://github.com/frankhli843/frankclaw).

## Why Rust?

- **⚡ Performance** — Zero-cost abstractions, no garbage collector, minimal memory footprint
- **🔒 Safety** — Memory safety guaranteed at compile time, no null pointer exceptions
- **🦀 Reliability** — Rich type system catches bugs before runtime
- **📦 Single Binary** — No Node.js runtime dependency, just one static binary

## Features

Ported from frankclaw/OpenClaw:

- **Core utilities** — E.164 normalization, WhatsApp JID conversion, path resolution, UTF-16 safe string operations
- **Markdown → WhatsApp** — Converts standard Markdown to WhatsApp-compatible formatting
- **Poll management** — Poll input normalization and validation
- **Security** — Constant-time secret comparison, external content wrapping with injection detection, homoglyph normalization
- **CLI** — Clap-based CLI with gateway, config, and onboard subcommands
- **Config** — JSON configuration loading with serde
- **Duration/byte parsing** — Human-friendly duration (10s, 1m, 2h) and byte size (10kb, 1mb) parsing

## Install

```bash
# From source
cargo install --path .

# Or build
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
├── config/           # Configuration loading and types
├── markdown/         # Markdown conversion (WhatsApp formatting)
├── security/         # Secret comparison, external content protection
├── polls.rs          # Poll input normalization
├── utils.rs          # Core utilities (E.164, JID, paths, UTF-16)
├── version.rs        # Version from Cargo.toml
├── lib.rs            # Library root
└── main.rs           # Binary entry point
```

## Test Coverage

72 tests ported from the frankclaw TypeScript test suite covering:
- Path normalization and resolution
- WhatsApp number/JID conversion
- Markdown to WhatsApp conversion
- Poll validation
- Duration and byte size parsing
- Security: constant-time comparison, injection detection, content wrapping
- Configuration parsing

## License

MIT — same as OpenClaw.
