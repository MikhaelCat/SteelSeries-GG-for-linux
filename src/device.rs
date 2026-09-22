// Device Management Module
// Handles SteelSeries device enumeration and management

use hidapi::HidApi;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

/// All supported SteelSeries device types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DeviceType {
    Keyboard,
    Mouse,
    Headset,
    Mousepad,
    Other,
}

impl std::fmt::Display for DeviceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DeviceType::Keyboard => write!(f, "Keyboard"),
            DeviceType::Mouse => write!(f, "Mouse"),
            DeviceType::Headset => write!(f, "Headset"),
            DeviceType::Mousepad => write!(f, "Mousepad"),
            DeviceType::Other => write!(f, "Other"),
        }
    }
}

/// Connection type for devices
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConnectionType {
    WiredUSB,
    Wireless24GHz,
    Bluetooth,
    WirelessDock,
}

/// Represents a connected SteelSeries device
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Device {
    /// Unique device identifier
    pub id: String,
    
    /// Device type
    pub device_type: DeviceType,
    
    /// Manufacturer ID (always 0x1246 for SteelSeries)
    pub vendor_id: u16,
    
    /// Product ID (varies by model)
    pub product_id: u16,
    
    /// Product name/model
    pub model_name: String,
    
    /// Serial number
    pub serial_number: String,
    
    /// Firmware version
    pub firmware_version: String,
    
    /// Current connection method
    pub connection_type: ConnectionType,
    
    /// HID device path
    pub hid_path: String,
    
    /// Is device currently active
    pub is_active: bool,
    
    /// Supported lighting effects
    pub supported_effects: Vec<String>,
    
    /// Number of RGB zones (0 if not applicable)
    pub rgb_zones: u32,
}

impl Device {
    /// Get display string for device type
    pub fn type_str(&self) -> String {
        self.device_type.to_string()
    }
}

/// Custom error types for device operations
#[derive(Error, Debug)]
pub enum DeviceError {
    #[error("Failed to initialize HID API: {0}")]
    HidInit(#[from] hidapi::HidError),
    
    #[error("Device not found: {0}")]
    DeviceNotFound(String),
    
    #[error("Device access denied: {0}")]
    AccessDenied(String),
    
    #[error("HID operation failed: {0}")]
    HidOperation(String),
    
    #[error("Invalid device format: {0}")]
    InvalidFormat(String),
    
    #[error("Firmware query failed: {0}")]
    FirmwareQuery(String),
}

/// Result type for device operations
pub type DeviceResult<T> = Result<T, DeviceError>;

/// Manages all SteelSeries devices on the system
pub struct DeviceManager {
    /// HIDAPI instance
    hid_api: HidApi,
    
    /// Cache of discovered devices
    devices: HashMap<String, Device>,
    
    /// Active device connections
    connections: HashMap<String, hidapi::HidDevice>,
}

impl DeviceManager {
    /// Create new DeviceManager instance
    pub fn new() -> DeviceResult<Self> {
        let hid_api = HidApi::new()?;
        
        Ok(DeviceManager {
            hid_api,
            devices: HashMap::new(),
            connections: HashMap::new(),
        })
    }

    /// Enumerate all connected SteelSeries devices
    pub fn enumerate(&self) -> DeviceResult<Vec<Device>> {
        // Filter by SteelSeries VID (0x1246)
        let mut result = Vec::new();
        
        for device_info in self.hid_api.device_list() {
            if device_info.vendor_id() == 0x1246 {
                // Filter to known SteelSeries PIDs
                if Self::is_steelseries_device(device_info.product_id()) {
                    match self.create_device_from_info(device_info) {
                        Ok(device) => result.push(device),
                        Err(e) => tracing::warn!("Failed to process device {:?}: {}", device_info, e),
                    }
                }
            }
        }
        
        Ok(result)
    }

    /// Check if a product ID belongs to SteelSeries devices we support
    fn is_steelseries_device(pid: u16) -> bool {
        // Known SteelSeries PIDs from research
        matches!(pid,
            // Keyboards
            0x0503 | // Apex Pro
            0x0603 | // Apex 7
            0x1630 | // Apex Pro TKL 2023
            0x1730 | // Apex 3
            0x1A30 | // Apex 5
            0x1B30 | // Apex 7 TKL
            0x2130 | // RK-680 TUX
            0x2131 | // RK-700 TUX
            0x2132 | // RK-800 TUX
            
            // Mice
            0x0610 | // Rival 105
            0x0611 | // Rival 300
            0x0710 | // Rival 5
            0x0910 | // Aerox 9 Wireless
            0x0A10 | // Rival 3
            0x0A11 | // Rival 3 Wireless
            0x0B10 | // Iron Wolf
            0x0C10 | // Sparrow
            0x0D10 | // Rival 300 Wireless
            0x1010 | // Aerox 0 Wireless
            0x1110 | // Rival 5 Wireless
            0x1210 | // Rival 3 Lite
            0x1211 | // Rival 3 Pro
            0x1310 | // Xtkr
            0x1510 | // Iron Wolf Mini
            0x1610 | // Aerox 5 Wireless
            0x1611 | // Aerox 9 Wireless
            
            // Headsets
            0x0020 | // Arctis 1
            0x0021 | // Arctis 1 Wireless
            0x0022 | // Arctis 5
            0x0023 | // Arctis 7
            0x0024 | // Arctis 9
            0x0025 | // Arctis Pro
            0x0026 | // Arctis Pro Wireless
            0x0027 | // Arctis Nova Pro
            0x0028 | // Arctis Nova Pro Wireless
            0x0029 | // Arctis 7 (2019 Edition)
            0x002A | // Arctis 1 Wireless
            0x002B | // Arctis Nova 5
            0x002C | // Arctis Nova 3
            0x002D | // Arctis Nova 1
            0x2290,  // Arctis Nova Pro Omni
        )
    }

    /// Create Device object from HID info
    fn create_device_from_info(&self, device_info: &hidapi::DeviceInfo) -> DeviceResult<Device> {
        let pid = device_info.product_id();
        let vid = device_info.vendor_id();
        
        // Extract model name from product string or use PID-based lookup
        let model_name = Self::model_name_from_pid(pid);
        
        // Get serial number if available
        let serial = device_info
            .serial_number()
            .map(|s| s.to_string())
            .unwrap_or_else(|| "Unknown".to_string());
        
        // Query firmware version
        let firmware = self.query_firmware(device_info)?;
        
        // Determine connection type
        let conn_type = Self::connection_type_from_path(device_info.path());
        
        // Determine device type from model or PID
        let device_type = Self::device_type_from_pid(pid);
        
        // Get supported effects based on device type
        let supported_effects = Self::supported_effects_for_device(&device_type);
        
        // Calculate RGB zones
        let rgb_zones = Self::rgb_zones_for_device(&device_type, pid);
        
        Ok(Device {
            id: format!("{}_{}", vid, pid),
            device_type,
            vendor_id: vid,
            product_id: pid,
            model_name,
            serial_number: serial,
            firmware_version: firmware,
            connection_type: conn_type,
            hid_path: device_info.path().to_string_lossy().to_string(),
            is_active: true,
            supported_effects,
            rgb_zones,
        })
    }

    /// Map PID to human-readable model name
    fn model_name_from_pid(pid: u16) -> String {
        match pid {
            // Keyboards
            0x0503 => "Apex Pro",
            0x0603 => "Apex 7",
            0x1630 => "Apex Pro TKL 2023",
            0x1730 => "Apex 3",
            0x1A30 => "Apex 5",
            0x1B30 => "Apex 7 TKL",
            
            // Mice
            0x0610 => "Rival 105",
            0x0611 => "Rival 300",
            0x0710 => "Rival 5",
            0x0910 => "Aerox 9 Wireless",
            0x0A10 => "Rival 3",
            0x0A11 => "Rival 3 Wireless",
            0x0B10 => "Iron Wolf",
            0x0C10 => "Sparrow",
            0x0D10 => "Rival 300 Wireless",
            0x1010 => "Aerox 0 Wireless",
            0x1110 => "Rival 5 Wireless",
            0x1210 => "Rival 3 Lite",
            0x1211 => "Rival 3 Pro",
            0x1310 => "Xtkr",
            0x1510 => "Iron Wolf Mini",
            0x1610 => "Aerox 5 Wireless",
            
            // Headsets
            0x0020 => "Arctis 1",
            0x0021 => "Arctis 1 Wireless",
            0x0022 => "Arctis 5",
            0x0023 => "Arctis 7",
            0x0024 => "Arctis 9",
            0x0025 => "Arctis Pro",
            0x0026 => "Arctis Pro Wireless",
            0x0027 => "Arctis Nova Pro",
            0x0028 => "Arctis Nova Pro Wireless",
            0x0029 => "Arctis 7 (2019 Edition)",
            0x002B => "Arctis Nova 5",
            0x002C => "Arctis Nova 3",
            0x002D => "Arctis Nova 1",
            0x2290 => "Arctis Nova Pro Omni",
            
            _ => "Unknown Model",
        }.to_string()
    }

    /// Map PID to device type
    fn device_type_from_pid(pid: u16) -> DeviceType {
        if pid < 0x1000 {
            DeviceType::Headset
        } else if pid < 0x2000 {
            DeviceType::Mouse
        } else {
            DeviceType::Keyboard
        }
    }

    /// Determine connection type from device path
    fn connection_type_from_path(path: &std::path::Path) -> ConnectionType {
        // Simple heuristic - wireless devices often have multiple interfaces
        // This should be improved with actual device querying
        ConnectionType::WiredUSB
    }

    /// Query firmware version from device
    fn query_firmware(&self, device_info: &hidapi::DeviceInfo) -> DeviceResult<String> {
        // Try to open device and query firmware
        // Implementation varies by device type
        
        // Fallback for now
        Ok("Querying...".to_string())
    }

    /// Get list of supported effects for device type
    fn supported_effects_for_device(device_type: &DeviceType) -> Vec<String> {
        match device_type {
            DeviceType::Keyboard => vec![
                "Static".to_string(),
                "Breathing".to_string(),
                "Spectrum".to_string(),
                "Wave".to_string(),
                "Reactive".to_string(),
                "Gradient".to_string(),
                "Custom".to_string(),
                "Off".to_string(),
            ],
            DeviceType::Mouse => vec![
                "Static".to_string(),
                "Breathing".to_string(),
                "Spectrum".to_string(),
                "Wave".to_string(),
                "Reactive".to_string(),
                "Off".to_string(),
            ],
            DeviceType::Headset => vec![
                "Static".to_string(),
                "Breathing".to_string(),
                "Off".to_string(),
            ],
            _ => vec![],
        }
    }

    /// Get RGB zone count for device
    fn rgb_zones_for_device(device_type: &DeviceType, pid: u16) -> u32 {
        match device_type {
            DeviceType::Keyboard => {
                // Keyboards typically have per-key RGB (61-104 keys)
                // Or zone-based lighting (4-12 zones)
                61 // Default to full keyboard
            }
            DeviceType::Mouse => {
                // Mice typically have 2-5 RGB zones
                3 // Typical mouse zones
            }
            DeviceType::Headset => {
                // Headsets typically have 1-2 zones (logo, earcup)
                1
            }
            _ => 0,
        }
    }

    /// Connect to a specific device
    pub fn connect(&mut self, device: &Device) -> DeviceResult<()> {
        let hid_device = self.hid_api.open_path(device.hid_path.clone())?;
        self.connections.insert(device.id.clone(), hid_device);
        Ok(())
    }

    /// Disconnect from a device
    pub fn disconnect(&mut self, device_id: &str) -> DeviceResult<()> {
        if let Some(_dev) = self.connections.remove(device_id) {
            Ok(())
        } else {
            Err(DeviceError::DeviceNotFound(device_id.to_string()))
        }
    }
}
