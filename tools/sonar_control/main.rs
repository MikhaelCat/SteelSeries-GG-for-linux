// Sonar Control CLI Tool (Experimental)
// Requires 'sonar' feature flag enabled

use clap::{Parser, Subcommand};
use std::collections::HashMap;

#[derive(Parser)]
#[command(name = "sonar_control")]
#[command(about = "SteelSeries Sonar HTTP API control utility", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Base URL of Sonar HTTP server
    #[arg(short, long, default_value_t = String::from("http://127.0.0.1:26301"))]
    base_url: String,
}

#[derive(Subcommand)]
enum Commands {
    /// Query status information
    Status,

    /// Discover Sonar server port dynamically
    Discover,

    /// List connected audio devices
    Devices,

    /// Get or set volume levels
    Volume {
        /// Audio channel
        channel: String,

        /// Volume level (0-100) - optional, if not provided shows current value
        level: Option<u8>,
    },

    /// Manage modes
    Mode {
        #[command(subcommand)]
        action: ModeAction,
    },

    /// Manage streamer settings
    Streamer {
        #[command(subcommand)]
        action: StreamerAction,
    },

    /// View available configurations
    Configs,
}

#[derive(Subcommand)]
enum ModeAction {
    /// Get current mode
    Get,

    /// Set mode
    Set {
        mode: String,
    },
}

#[derive(Subcommand)]
enum StreamerAction {
    /// Configure streaming
    Streaming {
        source: String,
        channels: HashMap<String, u8>, // channel -> level
    },

    /// Configure monitoring
    Monitoring {
        channels: HashMap<String, u8>, // channel -> level
    },

    /// Get current streaming config
    GetStreaming,

    /// Get current monitoring config
    GetMonitoring,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Status => {
            println!("Sonar Status endpoint: {}", cli.base_url);
            // Would fetch /status in production
            Ok(())
        }
        Commands::Discover => {
            eprintln!("Sonar discovery is still being implemented");
            Ok(())
        }
        Commands::Devices => {
            eprintln!("Device listing requires active Sonar server connection");
            Ok(())
        }
        Commands::Volume { channel, level } => {
            if let Some(val) = level {
                println!("Setting {} volume to {}%", channel, val);
            } else {
                println!("Current {} volume (not yet implemented)", channel);
            }
            Ok(())
        }
        Commands::Mode { action } => {
            match action {
                ModeAction::Get => println!("Getting mode..."),
                ModeAction::Set { mode } => println!("Setting mode to: {}", mode),
            }
            Ok(())
        }
        Commands::Streamer { action } => {
            eprintln!("Streamer configuration not yet implemented");
            Ok(())
        }
        Commands::Configs => {
            println!("Available configurations (placeholder)");
            Ok(())
        }
    }
}
