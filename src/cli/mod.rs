pub mod parse_bytes;
pub mod parse_duration;

use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "rustyclaw", version, about = "High-performance AI assistant gateway")]
pub struct Cli {
    /// Enable verbose output
    #[arg(short, long, global = true)]
    pub verbose: bool,

    /// Auto-approve all prompts
    #[arg(short, long, global = true)]
    pub yes: bool,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Start the gateway server
    Gateway {
        #[command(subcommand)]
        action: GatewayAction,
    },
    /// Show version information
    Version,
    /// Run the onboarding wizard
    Onboard,
    /// Manage configuration
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },
}

#[derive(Subcommand, Debug)]
pub enum GatewayAction {
    /// Start the gateway
    Start {
        /// Port override
        #[arg(short, long)]
        port: Option<u16>,
    },
    /// Stop the gateway
    Stop,
    /// Restart the gateway
    Restart,
    /// Show gateway status
    Status,
}

#[derive(Subcommand, Debug)]
pub enum ConfigAction {
    /// Show current configuration
    Show,
    /// Edit configuration
    Edit,
    /// Validate configuration
    Validate,
    /// Get a specific config value
    Get {
        /// Config key path (dot-separated)
        key: String,
    },
}

/// Run the CLI application.
pub fn run() {
    let cli = Cli::parse();

    // Initialize logging
    crate::logging::init_logging(cli.verbose);

    match cli.command {
        Some(Commands::Version) => {
            println!("rustyclaw {}", crate::VERSION);
        }
        Some(Commands::Gateway { action }) => {
            match action {
                GatewayAction::Start { port } => {
                    let rt = tokio::runtime::Runtime::new().unwrap();
                    rt.block_on(async {
                        let mut config = crate::config::load_config()
                            .unwrap_or_else(|e| {
                                eprintln!("Failed to load config: {}", e);
                                std::process::exit(1);
                            });

                        // Apply port override
                        if let Some(p) = port {
                            if let Some(ref mut gw) = config.gateway {
                                gw.port = Some(p);
                            } else {
                                config.gateway = Some(crate::config::GatewayConfig {
                                    port: Some(p),
                                    ..Default::default()
                                });
                            }
                        }

                        if let Err(e) = crate::gateway::start_gateway(config).await {
                            eprintln!("Gateway error: {}", e);
                            std::process::exit(1);
                        }
                    });
                }
                GatewayAction::Stop => {
                    println!("Sending stop signal to gateway...");
                    // In a full implementation, would send signal via PID file or HTTP
                }
                GatewayAction::Restart => {
                    println!("Restarting gateway...");
                }
                GatewayAction::Status => {
                    let rt = tokio::runtime::Runtime::new().unwrap();
                    rt.block_on(async {
                        match check_gateway_status().await {
                            Ok(status) => println!("{}", status),
                            Err(e) => {
                                eprintln!("Gateway not reachable: {}", e);
                                std::process::exit(1);
                            }
                        }
                    });
                }
            }
        }
        Some(Commands::Onboard) => {
            println!("Onboarding wizard: not implemented yet");
        }
        Some(Commands::Config { action }) => {
            match action {
                ConfigAction::Show => {
                    match crate::config::load_config() {
                        Ok(config) => {
                            let json = serde_json::to_string_pretty(&config).unwrap();
                            println!("{}", json);
                        }
                        Err(e) => eprintln!("Error loading config: {}", e),
                    }
                }
                ConfigAction::Validate => {
                    match crate::config::load_config() {
                        Ok(config) => {
                            println!("✓ Config is valid");
                            if let Some(model) = config.primary_model() {
                                println!("  Model: {}", model);
                            }
                            if let Some(ws) = config.workspace_dir() {
                                println!("  Workspace: {}", ws);
                            }
                        }
                        Err(e) => {
                            eprintln!("✗ Config validation failed: {}", e);
                            std::process::exit(1);
                        }
                    }
                }
                ConfigAction::Edit => {
                    let config_path = crate::config::resolve_config_path();
                    let editor = std::env::var("EDITOR").unwrap_or_else(|_| "vi".to_string());
                    let status = std::process::Command::new(&editor)
                        .arg(&config_path)
                        .status();
                    match status {
                        Ok(s) if s.success() => println!("Config saved."),
                        _ => eprintln!("Editor exited with error"),
                    }
                }
                ConfigAction::Get { key } => {
                    match crate::config::load_config() {
                        Ok(config) => {
                            let json = serde_json::to_value(&config).unwrap();
                            let parts: Vec<&str> = key.split('.').collect();
                            let mut current = &json;
                            for part in &parts {
                                current = match current.get(part)
                                    .or_else(|| {
                                        // Try camelCase
                                        let camel = to_camel_case(part);
                                        current.get(&camel)
                                    }) {
                                    Some(v) => v,
                                    None => {
                                        eprintln!("Key not found: {}", key);
                                        std::process::exit(1);
                                    }
                                };
                            }
                            println!("{}", serde_json::to_string_pretty(current).unwrap());
                        }
                        Err(e) => eprintln!("Error: {}", e),
                    }
                }
            }
        }
        None => {
            println!("rustyclaw {} — run with --help for usage", crate::VERSION);
        }
    }
}

fn to_camel_case(s: &str) -> String {
    let parts: Vec<&str> = s.split('_').collect();
    if parts.len() <= 1 {
        return s.to_string();
    }
    let mut result = parts[0].to_string();
    for part in &parts[1..] {
        let mut chars = part.chars();
        if let Some(first) = chars.next() {
            result.push(first.to_uppercase().next().unwrap());
            result.extend(chars);
        }
    }
    result
}

async fn check_gateway_status() -> Result<String, Box<dyn std::error::Error>> {
    let config = crate::config::load_config()?;
    let port = crate::config::resolve_gateway_port(&config);
    let url = format!("http://127.0.0.1:{}/health", port);

    let client = reqwest::Client::new();
    let resp = client.get(&url)
        .timeout(std::time::Duration::from_secs(5))
        .send()
        .await?;

    let body: serde_json::Value = resp.json().await?;
    Ok(serde_json::to_string_pretty(&body)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn to_camel_case_works() {
        assert_eq!(to_camel_case("hello_world"), "helloWorld");
        assert_eq!(to_camel_case("simple"), "simple");
        assert_eq!(to_camel_case("a_b_c"), "aBC");
    }
}
