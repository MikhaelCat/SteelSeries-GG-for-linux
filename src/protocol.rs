// Device Protocol Layer
// Handles low-level communication with SteelSeries devices

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

/// HID Report types
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ReportType {
    Input,
    Output,
    Feature,
}

/// HID report data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HidReport {
    pub report_id: u8,
    pub report_type: ReportType,
    pub data: Vec<u8>,
}

impl HidReport {
    /// Create new input report
    pub fn new_input(data: Vec<u8>) -> Self {
        Self {
            report_id: 0,
            report_type: ReportType::Input,
            data,
        }
    }

    /// Create new output report
    pub fn new_output(data: Vec<u8>) -> Self {
        Self {
            report_id: 0,
            report_type: ReportType::Output,
            data,
        }
    }

    /// Create new feature report
    pub fn new_feature(data: Vec<u8>) -> Self {
        Self {
            report_id: 0,
            report_type: ReportType::Feature,
            data,
        }
    }

    /// Set report ID
    pub fn with_report_id(mut self, id: u8) -> Self {
        self.report_id = id;
        self
    }
}

/// Keyboard protocols
pub mod keyboard {
    use serde::{Deserialize, Serialize};
    use std::collections::HashMap;

    /// Key code mapping (HID usage page 0x07)
    #[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
    #[repr(u8)]
    pub enum KeyCode {
        ErrorNone = 0,
        ErrorRollover = 1,
        PstFail = 2,
        ErrorUndefined = 3,
        A = 4,
        B = 5,
        C = 6,
        D = 7,
        E = 8,
        F = 9,
        G = 10,
        H = 11,
        I = 12,
        J = 13,
        K = 14,
        L = 15,
        M = 16,
        N = 17,
        O = 18,
        P = 19,
        Q = 20,
        R = 21,
        S = 22,
        T = 23,
        U = 24,
        V = 25,
        W = 26,
        X = 27,
        Y = 28,
        Z = 29,
        Number1 = 30,
        Number2 = 31,
        Number3 = 32,
        Number4 = 33,
        Number5 = 34,
        Number6 = 35,
        Number7 = 36,
        Number8 = 37,
        Number9 = 38,
        Number0 = 39,
        Enter = 40,
        Escape = 41,
        Backspace = 42,
        Tab = 43,
        Spacebar = 44,
        Minus = 45,
        Equals = 46,
        BracketL = 47,
        BracketR = 48,
        Backslash = 49,
        NonUsHash = 50,
        Semicolon = 51,
        Apostrophe = 52,
        Grave = 53,
        Comma = 54,
        Period = 55,
        Slash = 56,
        CapsLock = 57,
        F1 = 58,
        F2 = 59,
        F3 = 60,
        F4 = 61,
        F5 = 62,
        F6 = 63,
        F7 = 64,
        F8 = 65,
        F9 = 66,
        F10 = 67,
        F11 = 68,
        F12 = 69,
        PrintScreen = 70,
        ScrollLock = 71,
        Pause = 72,
        Insert = 73,
        Home = 74,
        PageUp = 75,
        Delete = 76,
        End = 77,
        PageDown = 78,
        RightArrow = 79,
        LeftArrow = 80,
        DownArrow = 81,
        UpArrow = 82,
    }

    /// OmniPoint actuation profile
    #[derive(Debug, Clone, Default, Serialize, Deserialize)]
    pub struct ActuationProfile {
        /// Actuation point for each key (in mm)
        pub actuation_points: HashMap<char, f32>, // 'a'-'z', '0'-'9', etc.
        
        /// Switch type (mechanical properties)
        pub switch_type: Option<String>,
        
        /// Macro bindings
        pub macros: HashMap<u32, Vec<KeyCode>>,
    }

    /// Keyboard RGB command builder
    pub struct RgbCommandBuilder {
        command_id: u8,
        zone_id: u32,
        effect: String,
        color: Option<[u8; 3]>,
        brightness: Option<u8>,
        speed: Option<u8>,
    }

    impl RgbCommandBuilder {
        pub fn new(command_id: u8) -> Self {
            Self {
                command_id,
                zone_id: 0,
                effect: "static".to_string(),
                color: None,
                brightness: None,
                speed: None,
            }
        }

        pub fn zone(mut self, zone_id: u32) -> Self {
            self.zone_id = zone_id;
            self
        }

        pub fn effect(mut self, effect: &str) -> Self {
            self.effect = effect.to_string();
            self
        }

        pub fn color(mut self, r: u8, g: u8, b: u8) -> Self {
            self.color = Some([r, g, b]);
            self
        }

        pub fn brightness(mut self, level: u8) -> Self {
            if level <= 100 {
                self.brightness = Some(level);
            }
            self
        }

        pub fn speed(mut self, level: u8) -> Self {
            self.speed = Some(level);
            self
        }

        pub fn build(&self) -> Vec<u8> {
            // Build LED control packet
            let mut packet = vec![self.command_id];
            
            // Zone encoding
            packet.extend_from_slice(&(self.zone_id as u32).to_le_bytes());
            
            // Effect encoding
            packet.push(effect_to_code(&self.effect));
            
            // Color
            if let Some(color) = self.color {
                packet.extend_from_slice(&color);
            }
            
            // Brightness
            if let Some(brightness) = self.brightness {
                packet.push((brightness * 255 / 100) as u8);
            }
            
            // Speed
            if let Some(speed) = self.speed {
                packet.push(speed);
            }
            
            packet
        }
    }

    fn effect_to_code(effect: &str) -> u8 {
        match effect.to_lowercase().as_str() {
            "static" => 0,
            "breathing" => 1,
            "spectrum" => 2,
            "wave" => 3,
            "reactive" => 4,
            "gradient" => 5,
            "off" => 7,
            _ => 0,
        }
    }

    /// Firmware command types
    pub enum FirmwareCommand {
        ReadVersion,
        WriteConfig,
        ReadConfig,
        FactoryReset,
        UpdateFirmware,
    }
}

/// Mouse protocols
pub mod mouse {
    use serde::{Deserialize, Serialize};

    /// DPI sensor reading mode
    #[derive(Debug, Clone, Copy, Default)]
    pub struct SensorReading {
        pub x: i32,
        pub y: i32,
        pub wheel: i16,
        pub buttons: u8,
        pub additional_buttons: Option<u8>,
        pub timestamp: u64,
    }

    /// Mouse polling rate options
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum PollingRate {
        Hz125 = 125,
        Hz250 = 250,
        Hz500 = 500,
        Hz1000 = 1000,
        Hz2000 = 2000,
        Hz4000 = 4000,
        Hz8000 = 8000,
    }

    impl From<PollingRate> for u32 {
        fn from(rate: PollingRate) -> Self {
            rate as u32
        }
    }

    /// DPI stage configuration packet builder
    pub struct DpiConfigBuilder {
        stages: [u32; 5],
        active_stage: usize,
        default_polling_rate: PollingRate,
    }

    impl Default for DpiConfigBuilder {
        fn default() -> Self {
            Self {
                stages: [800, 1600, 3200, 6400, 12000],
                active_stage: 2,
                default_polling_rate: PollingRate::Hz1000,
            }
        }
    }

    impl DpiConfigBuilder {
        pub fn stage(mut self, index: usize, dpi: u32) -> Self {
            if index < 5 {
                self.stages[index] = dpi;
            }
            self
        }

        pub fn active_stage(mut self, index: usize) -> Self {
            if index < 5 {
                self.active_stage = index;
            }
            self
        }

        pub fn polling_rate(mut self, rate: PollingRate) -> Self {
            self.default_polling_rate = rate;
            self
        }

        pub fn build(&self) -> Vec<u8> {
            let mut packet = vec![0x02]; // DPI config command
            
            // Each DPI value encoded as 2 bytes (divided by 100)
            for dpi in self.stages.iter() {
                let dpi_byte1 = ((dpi / 100) % 256) as u8;
                let dpi_byte2 = ((dpi / 100) / 256) as u8;
                packet.extend_from_slice(&[dpi_byte1, dpi_byte2]);
            }
            
            // Active stage (0-4)
            packet.push(self.active_stage as u8);
            
            // Polling rate encoding
            let rate_byte = match self.default_polling_rate {
                PollingRate::Hz125 => 0,
                PollingRate::Hz250 => 1,
                PollingRate::Hz500 => 2,
                PollingRate::Hz1000 => 3,
                PollingRate::Hz2000 => 4,
                PollingRate::Hz4000 => 5,
                PollingRate::Hz8000 => 6,
            };
            packet.push(rate_byte);
            
            packet
        }
    }

    /// OLED display commands for mice
    pub mod oled {
        // Test imports are fine

        /// Screen resolution (varies by device)
        #[derive(Debug, Clone, Copy, Default)]
        pub struct ScreenResolution {
            pub width: u16,
            pub height: u16,
        }

        /// OLED packet types
        pub enum OledPacket {
            ClearScreen,
            DrawPixel { x: u8, y: u8, state: bool },
            DrawLine { x1: u8, y1: u8, x2: u8, y2: u8, state: bool },
            DrawImage { bitmap: Vec<u8>, offset: u32 },
            RefreshScreen,
        }

        impl OledPacket {
            pub fn build(&self) -> Vec<u8> {
                match self {
                    OledPacket::ClearScreen => vec![0x10, 0x00],
                    OledPacket::DrawPixel { x, y, state } => {
                        vec![0x11, *x, *y, if *state { 1 } else { 0 }]
                    }
                    OledPacket::DrawLine { x1, y1, x2, y2, state } => {
                        vec![0x12, *x1, *y1, *x2, *y2, if *state { 1 } else { 0 }]
                    }
                    OledPacket::DrawImage { bitmap, offset } => {
                        let mut packet = vec![0x13, (*offset & 0xFF) as u8, ((offset >> 8) & 0xFF) as u8];
                        packet.extend_from_slice(bitmap);
                        packet
                    }
                    OledPacket::RefreshScreen => vec![0x1F],
                }
            }
        }
    }
}

/// Headset protocols
pub mod headset {
    use serde::{Deserialize, Serialize};

    /// Audio channel types
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum AudioChannel {
        Master = 0,
        Game = 1,
        Chat = 2,
        Stream1 = 3,
        Stream2 = 4,
        Microphone = 5,
    }

    /// Microphone settings
    #[derive(Debug, Clone, Default, Serialize, Deserialize)]
    pub struct MicSettings {
        pub muted: bool,
        pub volume: u8,
        pub noise_cancellation: bool,
    }

    /// Equalizer preset
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum EqPreset {
        Flat,
        BassBoost,
        TrebleBoost,
        VocalBoost,
        Gaming,
        Music,
    }

    /// Audio packet builder
    pub struct AudioControlBuilder {
        channel: AudioChannel,
        volume: Option<u8>,
        mute: Option<bool>,
    }

    impl AudioControlBuilder {
        pub fn new(channel: AudioChannel) -> Self {
            Self {
                channel,
                volume: None,
                mute: None,
            }
        }

        pub fn volume(mut self, level: u8) -> Self {
            self.volume = Some(level);
            self
        }

        pub fn mute(mut self, is_muted: bool) -> Self {
            self.mute = Some(is_muted);
            self
        }

        pub fn build(&self) -> Vec<u8> {
            let mut packet = vec![0x20]; // Audio control command
            
            packet.push(self.channel as u8);
            
            if let Some(vol) = self.volume {
                packet.push(0x01); // Volume flag
                packet.push(vol);
            }
            
            if let Some(muted) = self.mute {
                packet.push(0x02); // Mute flag
                packet.push(if muted { 1 } else { 0 });
            }
            
            packet
        }
    }
}

/// Custom error types
#[derive(Error, Debug)]
pub enum ProtocolError {
    #[error("Invalid report format: {0}")]
    InvalidReport(String),
    
    #[error("Command not supported: {0}")]
    CommandNotSupported(String),
    
    #[error("Device communication failed: {0}")]
    Communication(String),
    
    #[error("Firmware error: {0}")]
    FirmwareError(String),
    
    #[error("Checksum mismatch")]
    ChecksumMismatch,
}

pub type ProtocolResult<T> = Result<T, ProtocolError>;
