// SteelSeries GG for Linux - Main Library
// Core functionality and exports

pub mod device;
pub mod rgb;
pub mod mouse;
pub mod gamesense;
pub mod audio;
pub mod config;
pub mod protocol;
pub mod effects;
pub mod util;

/// Re-export commonly used types
pub use device::{Device, DeviceType, DeviceManager};
pub use rgb::{RgbController, RgbEffect, RgbColor};
pub use mouse::{MouseTracker, MouseHandler};
pub use gamesense::{GameSenseServer, GameSenseState};
pub use config::{Config, ConfigManager, Profile};
