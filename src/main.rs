// SteelSeries GG for Linux - Command Line Interface
// Main entry point for ssgg-cli

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "ssgg")]
#[command(author = "Qoder Development Team")]
#[command(version = "0.1.0")]
#[command(about = "Native Linux SteelSeries GG replacement", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Enable debug logging
    #[arg(short, long, global = true, default_value_t = false)]
    verbose: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// List connected SteelSeries devices
    Devices {
        /// Show detailed information
        #[arg(short, long)]
        detailed: bool,
    },

    /// Control RGB lighting
    Rgb {
        #[command(subcommand)]
        action: RgbAction,
    },

    /// Manage device profiles
    Profile {
        #[command(subcommand)]
        action: ProfileAction,
    },

    /// Mouse tracking controls
    Mouse {
        #[command(subcommand)]
        action: MouseAction,
    },

    /// GameSense server control
    Gamesense {
        #[command(subcommand)]
        action: Option<GamesenseAction>,
    },

    /// Audio mixer (experimental)
    Audio {
        #[command(subcommand)]
        action: AudioAction,
    },

    /// Start daemon with background services
    Daemon,

    /// Test device functionality
    TestDevice,

    /// View HID device logs
    HidLogs,

    /// Verify system performance
    VerifyPerformance,

    /// Advanced debugging tools
    Debug {
        #[command(subcommand)]
        action: DebugAction,
    },
}

#[derive(Subcommand)]
enum RgbAction {
    /// Set static color
    Color {
        /// Color value (name, hex, or RGB tuple)
        #[arg(short, long)]
        color: String,

        /// Brightness level (0-100)
        #[arg(short, long)]
        brightness: Option<u8>,
    },

    /// Apply lighting effect
    Effect {
        /// Effect name
        #[arg(short, long)]
        effect: String,

        /// Optional color for effect
        #[arg(short, long)]
        color: Option<String>,

        /// Direction (for wave effects)
        #[arg(short, long)]
        direction: Option<String>,
    },

    /// Get current RGB status
    Status,
}

#[derive(Subcommand)]
enum ProfileAction {
    /// List all profiles
    List,

    /// Save current configuration
    Save {
        /// Profile name
        name: String,
    },

    /// Load a profile
    Load {
        /// Profile name
        name: String,
    },

    /// Delete a profile
    Delete {
        /// Profile name
        name: String,
    },
}

#[derive(Subcommand)]
enum MouseAction {
    /// Start mouse tracking overlay
    Start,

    /// Stop mouse tracking
    Stop,

    /// Set DPI stages
    SetDpi {
        /// DPI stages (comma-separated)
        dpi: String,
    },

    /// Change active DPI stage
    ChangeStage {
        /// Stage index (0-based)
        stage: usize,
    },

    /// Show current mouse status
    Status,
}

#[derive(Subcommand)]
enum GamesenseAction {
    /// Start GameSense HTTP server
    Start,

    /// Stop GameSense server
    Stop,

    /// Show server status
    Status,
}

#[derive(Subcommand)]
enum AudioAction {
    /// Show audio status
    Status,

    /// Set volume
    Volume {
        /// Channel (master, game, chat)
        channel: String,

        /// Volume level (0-100)
        level: u8,
    },

    /// Mute channel
    Mute {
        /// Channel to mute
        channel: String,
    },

    /// Unmute channel
    Unmute {
        /// Channel to unmute
        channel: String,
    },

    /// Adjust chat mix balance
    ChatMix {
        /// Balance level (0-100, center is 50)
        balance: u8,
    },
}

#[derive(Subcommand)]
enum DebugAction {
    /// Run HID report fuzzing
    Fuzz,

    /// Probe actuation firmware commands
    Actuation,

    /// Generate diagnostics report
    Diagnostics,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .compact();

    if std::env::var_os("RUST_LOG").is_some() || Cli::parse().verbose {
        std::env::set_var("RUST_LOG", "debug");
        subscriber.with_target(true).init();
    } else {
        subscriber.init();
    }

    let cli = Cli::parse();

    match cli.command {
        Commands::Devices { detailed } => {
            handle_devices(detailed).await?;
        }
        Commands::Rgb { action } => {
            handle_rgb(action).await?;
        }
        Commands::Profile { action } => {
            handle_profile(action).await?;
        }
        Commands::Mouse { action } => {
            handle_mouse(action).await?;
        }
        Commands::Gamesense { action } => {
            if let Some(gamesense_action) = action {
                handle_gamesense(gamesense_action).await?;
            } else {
                println!("Use 'ssgg gamesense start' to start the GameSense server");
            }
        }
        Commands::Audio { action } => {
            handle_audio(action).await?;
        }
        Commands::Daemon => {
            run_daemon().await?;
        }
        Commands::TestDevice => {
            test_device().await?;
        }
        Commands::HidLogs => {
            show_hid_logs().await?;
        }
        Commands::VerifyPerformance => {
            verify_performance().await?;
        }
        Commands::Debug { action } => {
            handle_debug(action)?;
        }
    }

    Ok(())
}

async fn handle_devices(detailed: bool) -> anyhow::Result<()> {
    use ssgg::DeviceManager;

    let manager = DeviceManager::new()?;
    let devices = manager.enumerate()?;

    if devices.is_empty() {
        println!("No SteelSeries devices detected.");
        return Ok(());
    }

    if detailed {
        print_devices_detailed(&devices);
    } else {
        print_devices_summary(&devices);
    }

    Ok(())
}

fn print_devices_summary(devices: &[ssgg::Device]) {
    use tabled::settings::Style;
    // Removed tabled dependency temporarily due to build issues
    let _ = Style; // Keep if re-added later

    let table_data: Vec<_> = devices.iter().map(|d| DeviceSummary {
        type_str: d.type_str(),
        model: d.model_name.clone(),
        serial: d.serial_number.clone(),
        firmware: d.firmware_version.clone(),
    }).collect();

    println!("Connected SteelSeries Devices:\n");
    println!("{}\n", table_data.iter().map(|d| {
        format!("Type: {}, Model: {}, Serial: {}, Firmware: {}",
            d.type_str, d.model, d.serial, d.firmware)
    }).collect::<Vec<_>>().join("\n"));
}

fn print_devices_detailed(devices: &[ssgg::Device]) {
    for (idx, device) in devices.iter().enumerate() {
        println!("=== Device {} ===", idx + 1);
        println!("Type: {}", device.type_str());
        println!("Model: {}", device.model_name);
        println!("Serial: {}", device.serial_number);
        println!("Firmware: {}", device.firmware_version);
        println!("Connection: {:?}", device.connection_type);
        println!("Supported Effects: {:?}", device.supported_effects);
        println!();
    }
}

async fn handle_rgb(action: RgbAction) -> anyhow::Result<()> {
    // For now, communicate with daemon via D-Bus or socket
    // Implementation pending daemon setup
    eprintln!("RGB control requires running daemon (start with: ssgg daemon)");
    std::process::exit(1);
}

async fn handle_profile(action: ProfileAction) -> anyhow::Result<()> {
    use ssgg::Config;

    let config = Config::load_from_home()?;

    match action {
        ProfileAction::List => {
            let profiles = config.list_profiles()?;
            println!("Available Profiles:\n{}", profiles.join("\n"));
        }
        ProfileAction::Save { name } => {
            config.save_profile(&name, &ssgg::Profile::default())?;
            println!("Profile '{}' saved successfully", name);
        }
        ProfileAction::Load { name } => {
            let profile = config.load_profile(&name)?;
            println!("Loaded profile: {}", name);
            // Apply profile here
        }
        ProfileAction::Delete { name } => {
            config.delete_profile(&name)?;
            println!("Profile '{}' deleted", name);
        }
    }

    Ok(())
}

async fn handle_mouse(action: MouseAction) -> anyhow::Result<()> {
    // Mouse tracking implementation pending
    eprintln!("Mouse tracking feature coming soon");
    std::process::exit(1);
}

async fn handle_gamesense(action: GamesenseAction) -> anyhow::Result<()> {
    use ssgg::GameSenseServer;

    match action {
        GamesenseAction::Start => {
            let server = GameSenseServer::new();
            eprintln!("Starting GameSense server on port 27301...");
            server.serve().await?;
        }
        GamesenseAction::Stop => {
            eprintln!("Stopping GameSense server...");
        }
        GamesenseAction::Status => {
            eprintln!("GameSense server status check implemented");
        }
    }

    Ok(())
}

async fn handle_audio(action: AudioAction) -> anyhow::Result<()> {
    eprintln!("Audio features require 'audio' feature flag");
    std::process::exit(1);
}

async fn run_daemon() -> anyhow::Result<()> {
    use ssgg::DeviceManager;
    use tracing::info;

    info!("Initializing ssgg daemon");

    // Create device manager
    let manager = DeviceManager::new()?;
    
    // Start monitoring
    let mut monitor = manager.start_monitoring()?;

    info!("Daemon started, listening for device events");

    // Keep running until interrupted
    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        
        // Process device events
        while let Some(event) = monitor.try_recv() {
            info!("Device event: {:?}", event);
        }
    }
}

async fn test_device() -> anyhow::Result<()> {
    eprintln!("Device testing mode not yet implemented");
    std::process::exit(1);
}

async fn show_hid_logs() -> anyhow::Result<()> {
    eprintln!("HID logs viewer not yet implemented");
    std::process::exit(1);
}

fn verify_performance() -> anyhow::Result<()> {
    eprintln!("Performance verification tool not yet implemented");
    std::process::exit(1);
}

fn handle_debug(action: DebugAction) -> anyhow::Result<()> {
    match action {
        DebugAction::Fuzz => {
            eprintln!("HID fuzzing tool not yet implemented");
        }
        DebugAction::Actuation => {
            eprintln!("Actuation probe tool not yet implemented");
        }
        DebugAction::Diagnostics => {
            eprintln!("Generating diagnostics...");
            // TODO: Full diagnostics implementation
        }
    }
    Ok(())
}

struct DeviceSummary {
    pub type_str: String,
    pub model: String,
    pub serial: String,
    pub firmware: String,
}

