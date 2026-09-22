// SteelSeries GG for Linux - Main Library
// Core functionality and exports

pub mod audio;
pub mod config;
pub mod device;
pub mod effects;
pub mod gamesense;
pub mod mouse;
pub mod protocol;
pub mod rgb;
pub mod util;

pub use config::{Config, ConfigManager, Profile};
/// Re-export commonly used types
pub use device::{Device, DeviceManager, DeviceType};
pub use gamesense::{GameSenseServer, GameSenseState};
pub use mouse::{MouseHandler, MouseTracker};
pub use rgb::{RgbColor, RgbController, RgbEffect};
