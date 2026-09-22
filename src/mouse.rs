// Mouse Tracking Module
// Low-latency mouse movement capture and visualization

use hidapi::HidDevice;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use thiserror::Error;

/// Mouse sensor data structure
#[derive(Debug, Clone, Copy)]
pub struct SensorData {
    /// X-axis movement (in DPI units)
    pub dx: i32,
    
    /// Y-axis movement (in DPI units)
    pub dy: i32,
    
    /// Wheel movement (+1 scroll up, -1 scroll down)
    pub wheel: i16,
    
    /// Left button pressed (0 or 1)
    pub left_button: bool,
    
    /// Right button pressed (0 or 1)
    pub right_button: bool,
    
    /// Middle button pressed (0 or 1)
    pub middle_button: bool,
    
    /// Button 4 (back)
    pub button_4: bool,
    
    /// Button 5 (forward)
    pub button_5: bool,
}

impl Default for SensorData {
    fn default() -> Self {
        Self {
            dx: 0,
            dy: 0,
            wheel: 0,
            left_button: false,
            right_button: false,
            middle_button: false,
            button_4: false,
            button_5: false,
        }
    }
}

/// DPI configuration for mouse stages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DpiStage {
    /// Stage name
    pub name: String,
    
    /// DPI value
    pub dpi: u32,
    
    /// Polling rate in Hz
    pub polling_rate: u32, // typically 125, 500, 1000
    
    /// Acceleration enabled?
    pub acceleration: bool,
}

/// Active DPI stages for a mouse
#[derive(Debug, Clone)]
pub struct DpiConfig {
    /// Available stages
    pub stages: Vec<DpiStage>,
    
    /// Currently active stage index
    pub active_stage: usize,
    
    /// Quick-toggle stages (usually 2-3 common values)
    pub quick_toggle_stages: Vec<usize>,
}

impl Default for DpiConfig {
    fn default() -> Self {
        Self {
            stages: vec![
                DpiStage { name: "Low".into(), dpi: 800, polling_rate: 125, acceleration: false },
                DpiStage { name: "Medium".into(), dpi: 1600, polling_rate: 500, acceleration: false },
                DpiStage { name: "High".into(), dpi: 3200, polling_rate: 1000, acceleration: false },
                DpiStage { name: "Ultra".into(), dpi: 6400, polling_rate: 1000, acceleration: true },
            ],
            active_stage: 2,
            quick_toggle_stages: vec![1, 2],
        }
    }
}

/// Real-time mouse tracking overlay state
#[derive(Debug, Clone)]
pub struct OverlayState {
    /// Current raw sensor data
    pub sensor_data: SensorData,
    
    /// Current DPI
    pub current_dpi: u32,
    
    /// Total distance moved
    pub distance_moved_mm: f64,
    
    /// Last motion timestamp
    pub last_motion_time: Instant,
    
    /// Movement speed (mm/s)
    pub speed_mmps: f64,
    
    /// Frames since last update
    pub frames_since_last_update: u32,
}

impl Default for OverlayState {
    fn default() -> Self {
        Self {
            sensor_data: SensorData::default(),
            current_dpi: 1600,
            distance_moved_mm: 0.0,
            last_motion_time: Instant::now(),
            speed_mmps: 0.0,
            frames_since_last_update: 0,
        }
    }
}

/// Mouse handler for reading sensor data
pub struct MouseHandler {
    /// HID device connection
    hid_device: Option<HidDevice>,
    
    /// Device information
    device_id: String,
    
    /// DPI configuration
    dpi_config: DpiConfig,
    
    /// Sensor report format (varies by device)
    report_format: ReportFormat,
}

/// Report format specification for different mice
#[derive(Debug, Clone)]
enum ReportFormat {
    /// Standard HID mouse report
    StandardHid {
        report_size: usize,
        x_offset: usize,
        y_offset: usize,
        wheel_offset: usize,
        buttons_offset: usize,
    },
    
    /// Custom SteelSeries extended report
    ExtendedHid {
        report_size: usize,
        x_byte_start: usize,
        y_byte_start: usize,
        wheel_byte_start: usize,
        flags_byte_start: usize,
        flags_mask: u8,
    },
    
    /// USB interrupt endpoint specific
    InterruptEndpoint {
        endpoint: u8,
        interval_ms: u8,
        buffer_size: usize,
    },
}

/// Custom error types
#[derive(Error, Debug)]
pub enum MouseError {
    #[error("Device not found")]
    DeviceNotFound,
    
    #[error("HID communication failed: {0}")]
    Communication(String),
    
    #[error("Invalid report format: {0}")]
    InvalidReportFormat(String),
    
    #[error("Failed to parse sensor data: {0}")]
    ParseError(String),
    
    #[error("DPI out of range: {0}")]
    DpiOutOfRange(u32),
}

pub type MouseResult<T> = Result<T, MouseError>;

/// Implementation of MouseHandler
impl MouseHandler {
    pub fn new(device_id: String) -> Self {
        Self {
            hid_device: None,
            device_id,
            dpi_config: DpiConfig::default(),
            report_format: ReportFormat::StandardHid {
                report_size: 8,
                x_offset: 0,
                y_offset: 1,
                wheel_offset: 2,
                buttons_offset: 3,
            },
        }
    }

    /// Connect to mouse device
    pub fn connect(&mut self, hid_api: &hidapi::HidApi) -> MouseResult<()> {
        // This would need actual device path from DeviceManager
        // For now, placeholder implementation
        Ok(())
    }

    /// Read sensor data from mouse
    pub fn read_sensor_data(&mut self) -> MouseResult<SensorData> {
        if let Some(ref mut hid) = self.hid_device {
            let mut report = [0u8; 64];
            let bytes_read = hid.read_timeout(&mut report, 10)?;
            
            if bytes_read > 0 {
                return Ok(self.parse_sensor_report(&report));
            } else {
                Err(MouseError::Communication("Timeout reading sensor".to_string()))?
            }
        } else {
            Err(MouseError::DeviceNotFound)?
        }
    }

    /// Parse HID report into sensor data
    fn parse_sensor_report(&self, report: &[u8]) -> SensorData {
        match &self.report_format {
            ReportFormat::StandardHid { 
                x_offset, 
                y_offset, 
                wheel_offset, 
                buttons_offset 
            } => {
                SensorData {
                    dx: report[*x_offset] as i8 as i32,
                    dy: report[*y_offset] as i8 as i32,
                    wheel: ((report[*wheel_offset] as i16) >> 1) as i16,
                    left_button: report[*buttons_offset] & 0x01 != 0,
                    right_button: report[*buttons_offset] & 0x02 != 0,
                    middle_button: report[*buttons_offset] & 0x04 != 0,
                    button_4: report[*buttons_offset] & 0x08 != 0,
                    button_5: report[*buttons_offset] & 0x10 != 0,
                }
            }
            _ => SensorData::default(),
        }
    }

    /// Switch to specific DPI stage
    pub fn switch_dpi_stage(&mut self, stage_index: usize) -> MouseResult<()> {
        if stage_index < self.dpi_config.stages.len() {
            self.dpi_config.active_stage = stage_index;
            Ok(())
        } else {
            Err(MouseError::DpiOutOfRange(stage_index as u32))
        }
    }

    /// Get current DPI value
    pub fn current_dpi(&self) -> u32 {
        self.dpi_config.stages[self.dpi_config.active_stage].dpi
    }

    /// Get all DPI stages
    pub fn dpi_stages(&self) -> &[DpiStage] {
        &self.dpi_config.stages
    }

    /// Calculate pixel movement from raw sensor data
    pub fn calculate_pixel_movement(&self, sensor: &SensorData) -> (i32, i32) {
        let dpi = self.current_dpi() as f64;
        let pixels_per_mm = dpi / 25.4; // Convert DPI to pixels per mm
        
        // Raw sensor values are in counts at current DPI
        // For display purposes, normalize to show movement regardless of DPI
        (sensor.dx, sensor.dy)
    }

    /// Create overlay data structure
    pub fn create_overlay_state(&mut self) -> OverlayState {
        let state = OverlayState::default();
        
        if let Some(stage) = self.dpi_config.stages.get(self.dpi_config.active_stage) {
            state.current_dpi = stage.dpi;
        }
        
        state
    }
}

/// Overlay renderer for mouse tracking visualization
pub struct OverlayRenderer {
    /// Display backend (X11/Wayland)
    backend: BackendType,
    
    /// Window dimensions
    width: u32,
    height: u32,
    
    /// Overlay visibility
    visible: bool,
    
    /// Rendering frame counter
    frame_counter: u64,
}

#[derive(Debug, Clone)]
enum BackendType {
    X11,
    Wayland,
    Offscreen,
}

impl OverlayRenderer {
    pub fn new() -> Self {
        // Detect backend automatically
        let backend = if std::env::var("WAYLAND_DISPLAY").is_ok() {
            BackendType::Wayland
        } else if std::env::var("DISPLAY").is_ok() {
            BackendType::X11
        } else {
            BackendType::Offscreen
        };
        
        Self {
            backend,
            width: 800,
            height: 600,
            visible: false,
            frame_counter: 0,
        }
    }

    /// Show overlay
    pub fn show(&mut self) {
        self.visible = true;
    }

    /// Hide overlay
    pub fn hide(&mut self) {
        self.visible = false;
    }

    /// Render overlay state
    pub fn render(&self, state: &OverlayState) {
        if !self.visible {
            return;
        }

        self.frame_counter += 1;

        // In a real implementation, this would use X11/Wayland APIs
        // to draw semi-transparent overlay showing:
        // - Current DPI value
        // - Movement arrow indicators
        // - Speed graph
        // - Distance moved counter
        
        tracing::debug!(
            "Frame {}, DPI: {} | dx: {}, dy: {} | Speed: {:.2} mm/s",
            self.frame_counter,
            state.current_dpi,
            state.sensor_data.dx,
            state.sensor_data.dy,
            state.speed_mmps
        );
    }

    /// Set overlay position
    pub fn set_position(&mut self, x: i32, y: i32) {
        // Position handling
    }

    /// Set overlay size
    pub fn set_size(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
    }
}

/// Background worker for continuous sensor reading
pub struct MouseTracker {
    handler: MouseHandler,
    renderer: OverlayRenderer,
    running: bool,
    control_tx: Option<std::sync::mpsc::Sender<TrackerControl>>,
}

/// Control messages for tracker
pub enum TrackerControl {
    Start,
    Stop,
    ToggleVisibility,
    NextDpi,
    PreviousDpi,
    SetDpi(usize),
}

impl MouseTracker {
    pub fn new(device_id: String) -> Self {
        Self {
            handler: MouseHandler::new(device_id),
            renderer: OverlayRenderer::new(),
            running: false,
            control_tx: None,
        }
    }

    /// Start the tracker
    pub fn start(&mut self) -> MouseResult<()> {
        if self.running {
            return Ok(());
        }

        // Initialize device
        // self.handler.connect(...)?; // Would use passed HidApi instance
        
        self.renderer.show();
        self.running = true;
        
        Ok(())
    }

    /// Stop the tracker
    pub fn stop(&mut self) {
        self.running = false;
        self.renderer.hide();
    }

    /// Toggle visibility
    pub fn toggle_visibility(&mut self) {
        self.renderer.visible = !self.renderer.visible;
    }

    /// Check if currently tracking
    pub fn is_active(&self) -> bool {
        self.running
    }

    /// Run main loop
    pub fn run(&mut self, hid_api: &hidapi::HidApi) -> MouseResult<()> {
        self.start()?;
        
        let mut refresh_timer = Instant::now();
        
        while self.running {
            // Handle control messages
            // ...

            // Read sensor data every ~16ms (60 FPS target)
            if refresh_timer.elapsed() >= Duration::from_millis(16) {
                let sensor_data = self.handler.read_sensor_data().unwrap_or_default();
                
                // Update overlay state
                // ...
                
                // Render overlay
                self.renderer.render(&OverlayState::default());
                
                refresh_timer = Instant::now();
            }
            
            // Sleep briefly to avoid busy-waiting
            std::thread::sleep(Duration::from_millis(1));
        }
        
        Ok(())
    }
}
