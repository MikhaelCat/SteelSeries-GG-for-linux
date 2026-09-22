// RGB Lighting Control Module
// Handles keyboard, mouse, and headset RGB lighting with various effects

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

/// RGB color representation using HSV for better effect generation
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct RgbColor {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

impl Default for RgbColor {
    fn default() -> Self {
        Self {
            red: 0,
            green: 0,
            blue: 0,
        }
    }
}

/// HSV color representation
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HsvColor {
    pub hue: u16,       // 0-359 degrees
    pub saturation: u8, // 0-255
    pub value: u8,      // 0-255
}

impl RgbColor {
    /// Create from hex string (e.g., "#FF5500")
    pub fn from_hex(hex: &str) -> Result<Self, ColorParseError> {
        if !hex.starts_with('#') {
            return Err(ColorParseError::InvalidFormat);
        }

        let hex = &hex[1..];
        if hex.len() != 6 {
            return Err(ColorParseError::InvalidLength);
        }

        let r = u8::from_str_radix(&hex[0..2], 16).map_err(|_| ColorParseError::InvalidHex)?;
        let g = u8::from_str_radix(&hex[2..4], 16).map_err(|_| ColorParseError::InvalidHex)?;
        let b = u8::from_str_radix(&hex[4..6], 16).map_err(|_| ColorParseError::InvalidHex)?;

        Ok(Self {
            red: r,
            green: g,
            blue: b,
        })
    }

    /// Create from named color
    pub fn from_name(name: &str) -> Option<Self> {
        match name.to_lowercase().as_str() {
            "red" => Some(RgbColor {
                red: 255,
                green: 0,
                blue: 0,
            }),
            "green" => Some(RgbColor {
                red: 0,
                green: 255,
                blue: 0,
            }),
            "blue" => Some(RgbColor {
                red: 0,
                green: 0,
                blue: 255,
            }),
            "cyan" => Some(RgbColor {
                red: 0,
                green: 255,
                blue: 255,
            }),
            "magenta" => Some(RgbColor {
                red: 255,
                green: 0,
                blue: 255,
            }),
            "yellow" => Some(RgbColor {
                red: 255,
                green: 255,
                blue: 0,
            }),
            "white" => Some(RgbColor {
                red: 255,
                green: 255,
                blue: 255,
            }),
            "black" => Some(RgbColor {
                red: 0,
                green: 0,
                blue: 0,
            }),
            _ => None,
        }
    }

    /// Convert to HSV format
    pub fn to_hsv(self) -> HsvColor {
        let r = self.red as f32 / 255.0;
        let g = self.green as f32 / 255.0;
        let b = self.blue as f32 / 255.0;

        let max = r.max(g).max(b);
        let min = r.min(g).min(b);
        let delta = max - min;

        let hue = if delta == 0.0 {
            0.0
        } else if max == r {
            60.0 * (((g - b) / delta) % 6.0)
        } else if max == g {
            60.0 * (((b - r) / delta) + 2.0)
        } else {
            60.0 * (((r - g) / delta) + 4.0)
        };

        let saturation = if max == 0.0 { 0.0 } else { delta / max };
        let value = max;

        HsvColor {
            hue: (hue % 360.0) as u16,
            saturation: (saturation * 255.0) as u8,
            value: (value * 255.0) as u8,
        }
    }

    /// Parse RGB tuple string "(255, 0, 0)"
    pub fn from_tuple(s: &str) -> Option<Self> {
        s.trim_matches(|c| c == '(' || c == ')')
            .split(',')
            .map(|s| s.trim().parse::<u8>().ok())
            .collect::<Option<Vec<u8>>>()
            .and_then(|v| {
                if v.len() == 3 {
                    Some(RgbColor {
                        red: v[0],
                        green: v[1],
                        blue: v[2],
                    })
                } else {
                    None
                }
            })
    }
}

impl std::fmt::Display for RgbColor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "#{:02X}{:02X}{:02X}", self.red, self.green, self.blue)
    }
}

#[derive(Error, Debug)]
pub enum ColorParseError {
    #[error("Invalid hex format")]
    InvalidFormat,
    #[error("Invalid length")]
    InvalidLength,
    #[error("Invalid hex characters")]
    InvalidHex,
}

/// Available RGB lighting effects
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RgbEffect {
    Static,
    Breathing,
    Spectrum,
    Wave(WaveDirection),
    Reactive,
    Gradient(GradientType),
    Custom,
    Off,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WaveDirection {
    LeftToRight,
    RightToLeft,
    TopToBottom,
    BottomToTop,
    CenterToEdges,
    EdgesToCenter,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GradientType {
    Horizontal,
    Vertical,
    Diagonal,
    Radial,
    Circular,
}

/// Lighting zone on a device
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LightingZone {
    /// Zone identifier
    pub id: u32,

    /// Zone name (human readable)
    pub name: String,

    /// Number of LEDs in this zone (for keyboards: key count)
    pub led_count: u32,

    /// Current effect applied to this zone
    pub current_effect: RgbEffect,

    /// Current color for static/single-color effects
    pub current_color: RgbColor,

    /// Brightness level (0-100)
    pub brightness: u8,

    /// Animation speed multiplier (1-10)
    pub speed: u8,
}

/// RGB controller trait for different device types
pub trait RgbController: Send {
    /// Apply color to specific zones
    fn set_zone_color(&mut self, zone_id: u32, color: RgbColor) -> Result<(), RgbError>;

    /// Apply an effect to a zone
    fn apply_effect(&mut self, zone_id: u32, effect: &RgbEffect) -> Result<(), RgbError>;

    /// Set brightness for all zones
    fn set_global_brightness(&mut self, brightness: u8) -> Result<(), RgbError>;

    /// Get current brightness
    fn get_brightness(&self) -> Result<u8, RgbError>;

    /// Get list of supported effects
    fn supported_effects(&self) -> Vec<RgbEffect>;

    /// Refresh current state
    fn refresh_state(&mut self) -> Result<(), RgbError>;
}

/// Base RGB controller implementation
pub struct RgbControllerImpl {
    /// Device identifier
    device_id: String,

    /// All lighting zones
    zones: HashMap<u32, LightingZone>,

    /// Global brightness
    global_brightness: u8,

    /// Effect timing
    effect_timing: EffectTiming,
}

#[derive(Debug, Clone)]
struct EffectTiming {
    /// Frame interval in milliseconds
    frame_interval_ms: u64,

    /// Current frame number
    current_frame: u64,
}

impl RgbControllerImpl {
    pub fn new(device_id: String) -> Self {
        Self {
            device_id,
            zones: HashMap::new(),
            global_brightness: 100,
            effect_timing: EffectTiming {
                frame_interval_ms: 50, // 20 FPS base
                current_frame: 0,
            },
        }
    }

    /// Add a lighting zone
    pub fn add_zone(&mut self, zone: LightingZone) {
        self.zones.insert(zone.id, zone);
    }

    /// Remove a lighting zone
    pub fn remove_zone(&mut self, zone_id: u32) {
        self.zones.remove(&zone_id);
    }

    /// Calculate effect frame for given time
    fn calculate_effect_frame(&mut self, effect: &RgbEffect) -> u64 {
        self.effect_timing.current_frame += 1;
        match effect {
            RgbEffect::Wave(_) | RgbEffect::Reactive | RgbEffect::Breathing => {
                self.effect_timing.current_frame
            }
            _ => 0,
        }
    }

    /// Generate spectrum color based on time
    fn spectrum_color(frame: u64) -> RgbColor {
        let hue = (frame % 360) as f32;
        hsv_to_rgb(hue as u16, 255, 255)
    }

    /// Generate wave pattern across positions
    fn wave_pattern(pos: u32, total: u32, direction: WaveDirection, frame: u64) -> RgbColor {
        let wave_width = 10;
        let wave_pos = ((frame as i64 / 2) % (total as i64 * 2)) as u32;

        let distance = if pos <= wave_pos {
            pos.wrapping_sub(wave_pos.saturating_sub(wave_width))
        } else {
            pos.saturating_sub(wave_pos.saturating_add(wave_width))
        };

        if distance < wave_width {
            RgbColor {
                red: 255,
                green: 0,
                blue: 255, // Magenta wave
            }
        } else {
            RgbColor {
                red: 0,
                green: 0,
                blue: 0,
            }
        }
    }

    /// Generate reactive color for key press
    fn reactive_color() -> RgbColor {
        RgbColor {
            red: 0,
            green: 255,
            blue: 0, // Green reactive
        }
    }
}

impl RgbController for RgbControllerImpl {
    fn set_zone_color(&mut self, zone_id: u32, color: RgbColor) -> Result<(), RgbError> {
        if let Some(zone) = self.zones.get_mut(&zone_id) {
            zone.current_color = color;
            zone.current_effect = RgbEffect::Static;
        }
        Ok(())
    }

    fn apply_effect(&mut self, zone_id: u32, effect: &RgbEffect) -> Result<(), RgbError> {
        if let Some(zone) = self.zones.get_mut(&zone_id) {
            zone.current_effect = effect.clone();

            // Initialize default color for effect
            if zone.current_color.red == 0
                && zone.current_color.green == 0
                && zone.current_color.blue == 0
            {
                zone.current_color = RgbColor::from_name("cyan").unwrap_or(RgbColor {
                    red: 0,
                    green: 255,
                    blue: 255,
                });
            }
        }
        Ok(())
    }

    fn set_global_brightness(&mut self, brightness: u8) -> Result<(), RgbError> {
        if brightness <= 100 {
            self.global_brightness = brightness;
            Ok(())
        } else {
            Err(RgbError::InvalidParameter(format!(
                "Brightness {} out of range",
                brightness
            )))
        }
    }

    fn get_brightness(&self) -> Result<u8, RgbError> {
        Ok(self.global_brightness)
    }

    fn supported_effects(&self) -> Vec<RgbEffect> {
        vec![
            RgbEffect::Static,
            RgbEffect::Breathing,
            RgbEffect::Spectrum,
            RgbEffect::Wave(WaveDirection::LeftToRight),
            RgbEffect::Reactive,
            RgbEffect::Gradient(GradientType::Horizontal),
            RgbEffect::Off,
        ]
    }

    fn refresh_state(&mut self) -> Result<(), RgbError> {
        Ok(())
    }
}

/// Convert HSV to RGB
fn hsv_to_rgb(hue: u16, saturation: u8, value: u8) -> RgbColor {
    let h = hue as f32 / 60.0;
    let s = saturation as f32 / 255.0;
    let v = value as f32 / 255.0;

    let i = h as i32;
    let f = h - i as f32;
    let p = v * (1.0 - s);
    let q = v * (1.0 - s * f);
    let t = v * (1.0 - s * (1.0 - f));

    match i {
        0 => RgbColor {
            red: (v * 255.0) as u8,
            green: (t * 255.0) as u8,
            blue: (p * 255.0) as u8,
        },
        1 => RgbColor {
            red: (q * 255.0) as u8,
            green: (v * 255.0) as u8,
            blue: (p * 255.0) as u8,
        },
        2 => RgbColor {
            red: (p * 255.0) as u8,
            green: (v * 255.0) as u8,
            blue: (t * 255.0) as u8,
        },
        3 => RgbColor {
            red: (p * 255.0) as u8,
            green: (q * 255.0) as u8,
            blue: (v * 255.0) as u8,
        },
        4 => RgbColor {
            red: (t * 255.0) as u8,
            green: (p * 255.0) as u8,
            blue: (v * 255.0) as u8,
        },
        _ => RgbColor {
            red: (v * 255.0) as u8,
            green: (p * 255.0) as u8,
            blue: (q * 255.0) as u8,
        },
    }
}

/// Custom error types for RGB operations
#[derive(Error, Debug)]
pub enum RgbError {
    #[error("Device not found: {0}")]
    DeviceNotFound(String),

    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),

    #[error("HID communication failed: {0}")]
    CommunicationError(String),

    #[error("Invalid color: {0}")]
    InvalidColor(String),

    #[error("Not supported: {0}")]
    NotSupported(String),
}

pub type RgbResult<T> = Result<T, RgbError>;
