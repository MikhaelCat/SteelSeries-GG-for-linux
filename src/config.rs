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

impl Config {
    pub fn load_from_home() -> Result<Self, ConfigError> {
        // Try loading from default home location
        let config = ConfigManager::load_from_home()?;
        Ok(config)
    }
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
    
    #[error("Configuration not found")]
    ConfigNotFound,
    
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
        // Try loading from default home location
        let config = ConfigManager::load_from_home()?;
        Ok(config)
    }

    /// List available profiles
    pub fn list_profiles(&self) -> ConfigResult<Vec<String>> {
        // List profile files in data directory
        let mut profiles = Vec::new();
        if self.data_dir.exists() {
            for entry in fs::read_dir(&self.data_dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.extension().map_or(false, |ext| ext == "json") {
                    if let Some(filename) = path.file_stem() {
                        profiles.push(filename.to_string_lossy().to_string());
                    }
                }
            }
        }
        Ok(profiles)
    }

    /// Save profile
    pub fn save_profile(&self, name: &str, data: &serde_json::Value) -> ConfigResult<()> {
        let profile_path = self.data_dir.join(format!("{}.json", name));
        let content = serde_json::to_string_pretty(data)?;
        fs::write(profile_path, content)?;
        Ok(())
    }

    /// Load profile
    pub fn load_profile(&self, name: &str) -> ConfigResult<serde_json::Value> {
        let profile_path = self.data_dir.join(format!("{}.json", name));
        let content = fs::read_to_string(profile_path)?;
        let data: serde_json::Value = serde_json::from_str(&content)?;
        Ok(data)
    }

    /// Delete profile
    pub fn delete_profile(&self, name: &str) -> ConfigResult<()> {
        let profile_path = self.data_dir.join(format!("{}.json", name));
        fs::remove_file(profile_path)?;
        Ok(())
    }
}

// TODO: Complete cleanup of old config functions
