// Configuration Management Module
// Handles profiles, settings, and persistence

use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufReader, BufWriter};
use std::path::{Path, PathBuf};
use thiserror::Error;

/// Global configuration structure
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Config {
    /// GameSense settings
    pub gamesense: GamesenseConfig,
    
    /// Audio settings (if audio feature enabled)
    #[cfg(feature = "audio")]
    pub audio: AudioConfig,
    
    /// General settings
    pub general: GeneralConfig,
    
    /// Custom effect definitions
    pub custom_effects: HashMap<String, CustomEffect>,
}

impl ToString for Config {
    fn to_string(&self) -> String {
        format!("Config version: {}", self.general.default_profile)
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GamesenseConfig {
    pub enabled: bool,
    pub bind_address: String,
    pub port: u16,
}

#[cfg(feature = "audio")]
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AudioConfig {
    pub master_volume: i32,
    pub game_volume: i32,
    pub chat_volume: i32,
    pub chat_mix_balance: i32, // -100 (full game) to 100 (full chat)
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GeneralConfig {
    pub default_profile: String,
    pub auto_start_daemon: bool,
    pub debug_mode: bool,
}

/// Custom lighting effect definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomEffect {
    pub id: String,
    pub name: String,
    pub effect_type: String, // "wave", "gradient", etc.
    pub parameters: serde_json::Value,
    pub timing: EffectTiming,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EffectTiming {
    pub duration_ms: u64,
    pub frame_interval_ms: u64,
    pub loop_count: usize, // 0 for infinite
}

impl Default for EffectTiming {
    fn default() -> Self {
        Self {
            duration_ms: 10000,
            frame_interval_ms: 50,
            loop_count: 0,
        }
    }
}

/// Profile represents complete device configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Profile {
    /// Profile name
    pub name: String,
    
    /// Description
    pub description: Option<String>,
    
    /// Active devices in this profile
    pub devices: DeviceProfileMap,
    
    /// Whether this profile is active
    pub is_active: bool,
    
    /// Metadata
    pub created_at: String,
    pub modified_at: String,
}

pub type DeviceProfileMap = HashMap<String, DeviceProfile>;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DeviceProfile {
    /// RGB configuration
    pub rgb: RgbDeviceConfig,
    
    /// DPI configuration (for mice)
    pub dpi: Option<DpiDeviceConfig>,
    
    /// Actuation points (for keyboards with OmniPoint)
    pub actuation: Option<HashMap<char, f32>>,
    
    /// Macro bindings
    pub macros: HashMap<String, MacroBinding>,
    
    /// Button remapping
    pub button_map: HashMap<u32, u32>,
    
    /// Additional device-specific settings
    pub extra: serde_json::Value,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RgbDeviceConfig {
    pub effect: String,
    pub color: String,
    pub brightness: u8,
    pub speed: u8,
    pub zones: HashMap<u32, ZoneRgbConfig>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ZoneRgbConfig {
    pub effect: String,
    pub color: String,
    pub brightness: u8,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DpiDeviceConfig {
    pub stages: Vec<DpiStageConfig>,
    pub active_stage: usize,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DpiStageConfig {
    pub dpi: u32,
    pub polling_rate: u32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MacroBinding {
    pub key_sequence: Vec<String>,
    pub delay_ms: u64,
    pub repeat_count: u32,
}

/// Custom error types
#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Configuration directory not found")]
    DirectoryNotFound,
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    
    #[error("Profile not found: {0}")]
    ProfileNotFound(String),
    
    #[error("Invalid profile format: {0}")]
    InvalidProfile(String),
    
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
}

pub type ConfigResult<T> = Result<T, ConfigError>;

/// Configuration manager
pub struct ConfigManager {
    config_dir: PathBuf,
    data_dir: PathBuf,
    current_config: Config,
}

impl ConfigManager {
    /// Create new ConfigManager
    pub fn new() -> ConfigResult<Self> {
        // Get project directories
        let dirs = ProjectDirs::from(
            "com",
            "steelseries-linux",
            "ssgg",
        ).ok_or(ConfigError::DirectoryNotFound)?;
        
        let config_dir = dirs.config_dir().to_path_buf();
        let data_dir = dirs.data_dir().to_path_buf();
        
        // Create directories if they don't exist
        fs::create_dir_all(&config_dir)?;
        fs::create_dir_all(&data_dir)?;
        
        Ok(Self {
            config_dir,
            data_dir,
            current_config: Config::default(),
        })
    }

    /// Load configuration from home directory
    pub fn load_from_home() -> ConfigResult<Config> {
        let config_path = HomeConfigPath::new()?;
        
        if config_path.exists() {
            let file = File::open(&config_path)?;
            let reader = BufReader::new(file);
            let config: Config = serde_json::from_reader(reader)?;
            Ok(config)
        } else {
            Ok(Config::default())
        }
    }

    /// Save global configuration
    pub fn save_global_config(&self, config: &Config) -> ConfigResult<()> {
        let config_path = self.config_dir.join("config.toml");
        let content = toml::to_string_pretty(config).unwrap_or_else(|_| config.to_string());
        fs::write(&config_path, content)?;
        Ok(())
    }

    /// List available profiles
    pub fn list_profiles(&self) -> ConfigResult<Vec<String>> {
        let profiles_dir = self.data_dir.join("profiles");
        
        if !profiles_dir.exists() {
            return Ok(vec![]);
        }
        
        let mut profiles = Vec::new();
        for entry in fs::read_dir(&profiles_dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.extension() == Some(std::ffi::OsStr::new("json")) {
                if let Some(filename) = path.file_stem() {
                    if let Some(name) = filename.to_str() {
                        profiles.push(name.to_string());
                    }
                }
            }
        }
        
        Ok(profiles)
    }

    /// Save a profile
    pub fn save_profile(&self, name: &str, profile: &Profile) -> ConfigResult<()> {
        let profile_path = self.profiles_path(name);
        
        // Ensure parent directory exists
        if let Some(parent) = profile_path.parent() {
            fs::create_dir_all(parent)?;
        }
        
        let file = File::create(&profile_path)?;
        let writer = BufWriter::new(file);
        serde_json::to_writer_pretty(writer, profile)?;
        
        Ok(())
    }

    /// Load a profile by name
    pub fn load_profile(&self, name: &str) -> ConfigResult<Profile> {
        let profile_path = self.profiles_path(name);
        
        if !profile_path.exists() {
            Err(ConfigError::ProfileNotFound(name.to_string()))?
        }
        
        let file = File::open(&profile_path)?;
        let reader = BufReader::new(file);
        let profile: Profile = serde_json::from_reader(reader)?;
        
        Ok(profile)
    }

    /// Delete a profile
    pub fn delete_profile(&self, name: &str) -> ConfigResult<()> {
        let profile_path = self.profiles_path(name);
        
        if profile_path.exists() {
            fs::remove_file(&profile_path)?;
        }
        
        Ok(())
    }

    /// Get path to profile file
    fn profiles_path(&self, name: &str) -> PathBuf {
        self.data_dir.join("profiles").join(format!("{}.json", name))
    }

    /// Export profile as JSON string
    pub fn export_profile(&self, name: &str) -> ConfigResult<String> {
        let profile = self.load_profile(name)?;
        serde_json::to_string_pretty(&profile).map_err(Into::into)
    }

    /// Import profile from JSON string
    pub fn import_profile(&self, name: &str, json: &str) -> ConfigResult<()> {
        let profile: Profile = serde_json::from_str(json)?;
        self.save_profile(name, &profile)
    }

    /// Set default profile
    pub fn set_default_profile(&mut self, name: &str) -> ConfigResult<()> {
        self.current_config.general.default_profile = name.to_string();
        self.save_global_config(&self.current_config)
    }

    /// Get default profile name
    pub fn get_default_profile(&self) -> &str {
        &self.current_config.general.default_profile
    }
}

/// Helper struct for home directory config path
struct HomeConfigPath {
    path: PathBuf,
}

impl AsRef<std::path::Path> for HomeConfigPath {
    fn as_ref(&self) -> &std::path::Path {
        &self.path
    }
}

impl HomeConfigPath {
    fn new() -> ConfigResult<Self> {
        match std::env::var("HOME") {
            Ok(home) => Ok(Self {
                path: PathBuf::from(home).join(".config").join("ssgg").join("config.toml"),
            }),
            Err(_) => Err(ConfigError::DirectoryNotFound),
        }
    }

    fn exists(&self) -> bool {
        self.path.exists()
    }

    fn open(&self) -> std::io::Result<File> {
        File::open(&self.path)
    }
}

/// Utility function to serialize datetime
pub fn format_datetime(dt: &chrono::DateTime<chrono::Utc>) -> String {
    dt.format("%Y-%m-%dT%H:%M:%SZ").to_string()
}

/// Create timestamp
pub fn now_timestamp() -> String {
    chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string()
}
