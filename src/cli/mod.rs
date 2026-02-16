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
    Start,
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
}

/// Run the CLI application.
pub fn run() {
    let cli = Cli::parse();

    if cli.verbose {
        eprintln!("[rustyclaw] Verbose mode enabled");
    }

    match cli.command {
        Some(Commands::Version) => {
            println!("rustyclaw {}", crate::VERSION);
        }
        Some(Commands::Gateway { action }) => {
            match action {
                GatewayAction::Start => println!("Starting gateway..."),
                GatewayAction::Stop => println!("Stopping gateway..."),
                GatewayAction::Restart => println!("Restarting gateway..."),
                GatewayAction::Status => println!("Gateway status: not implemented yet"),
            }
        }
        Some(Commands::Onboard) => {
            println!("Onboarding wizard: not implemented yet");
        }
        Some(Commands::Config { action }) => {
            match action {
                ConfigAction::Show => println!("Config show: not implemented yet"),
                ConfigAction::Edit => println!("Config edit: not implemented yet"),
                ConfigAction::Validate => println!("Config validate: not implemented yet"),
            }
        }
        None => {
            println!("rustyclaw {} — run with --help for usage", crate::VERSION);
        }
    }
}
