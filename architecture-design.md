# SteelSeries GG Native Linux - Architecture Design Document

## Executive Summary

This document outlines the architecture for a native Linux implementation of SteelSeries GG software, providing full hardware support for SteelSeries devices including RGB lighting control, mouse tracking, device management, and GameSense integration.

## 1. System Overview

### 1.1 Goals
- **Complete Hardware Support**: Full parity with Windows SteelSeries GG functionality
- **Native Linux Integration**: Leverage Linux subsystems (udev, HIDAPI, D-Bus)
- **Modern UI/UX**: Preserve SteelSeries design language using GTK4/Rust or Qt6
- **Daemon Architecture**: Background service with CLI and GUI interfaces
- **Cross-Device Support**: Keyboards, mice, headsets, and accessories

### 1.2 Non-Goals
- Wine compatibility layer reliance
- Proprietary firmware dumping
- Real-time game telemetry injection (GameSense client only)

## 2. Architecture Layers

```
┌─────────────────────────────────────────────────────────────┐
│                     User Interface Layer                      │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────────┐   │
│  │  GTK4/Qt6    │  │   CLI/TUI    │  │   System Tray    │   │
│  │  Main App    │  │  ssgg-cli    │  │   Indicator      │   │
│  └──────────────┘  └──────────────┘  └──────────────────┘   │
├─────────────────────────────────────────────────────────────┤
│                    Service Bus Layer                          │
│  ┌────────────────────────────────────────────────────────┐  │
│  │          D-Bus / IPC Communication Layer               │  │
│  └────────────────────────────────────────────────────────┘  │
├─────────────────────────────────────────────────────────────┤
│                     Core Daemon Layer                         │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌────────────────┐  │
│  │ Device   │ │ RGB      │ │ Mouse    │ │ GameSense      │  │
│  │ Manager  │ │ Engine   │ │ Tracker  │ │ Server         │  │
│  └──────────┘ └──────────┘ └──────────┘ └────────────────┘  │
│  ┌──────────┐ ┌──────────┐ ┌────────────────────────────────┐│
│  │ Profile  │ │ Audio    │ │ Configuration Management       ││
│  │ System   │ │ Mixer    │ │                                ││
│  └──────────┘ └──────────┘ └────────────────────────────────┘│
├─────────────────────────────────────────────────────────────┤
│                   Hardware Abstraction Layer                  │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌────────────────┐  │
│  │ udev     │ │ HIDAPI   │ │ evdev    │ │ libusb         │  │
│  │ Monitor  │ │ Handler  │ │ Parser   │ │ Backend        │  │
│  └──────────┘ └──────────┘ └──────────┘ └────────────────┘  │
├─────────────────────────────────────────────────────────────┤
│                   Device Protocol Layer                       │
│  ┌────────────────────────────────────────────────────────┐  │
│  │ Keyboard Protocols │ Mouse Protocols │ Headset Protocols│  │
│  │ (RGB Zones, OLEDS) │ (Sensors, RGB)  │ (Audio, RGB)     │  │
│  └────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
```

## 3. Component Breakdown

### 3.1 User Interface Layer

#### 3.1.1 Main Application (GTK4-based)
- **Technology**: Rust + GTK4 + Adwaita + CSS styling
- **Responsibilities**:
  - Device selection and management interface
  - RGB lighting effects configuration
  - Profile management and switching
  - GameSense overlay configuration
  - Settings and preferences

#### 3.1.2 CLI Tool (ssgg-cli)
- **Technology**: Rust + Clap
- **Commands**:
  - `devices` - List connected SteelSeries devices
  - `rgb` - Control lighting (color, effect, brightness)
  - `profile` - Save/load configurations
  - `mouse` - Mouse tracking controls
  - `gamesense` - Start/stop GameSense server
  - `daemon` - Manage background service

#### 3.1.3 System Tray Indicator
- **Technology**:libappindicator orayat indicator
- **Functionality**: Quick access to common functions

### 3.2 Core Daemon (sssgd)

#### 3.2.1 Device Manager
- **Module**: `device_manager.rs`
- **Responsibilities**:
  - Enumerate all SteelSeries devices via udev/HIDAPI
  - Maintain device state and connections
  - Handle device hotplug/unplug events
  - Query device capabilities and firmware version

#### 3.2.2 RGB Engine
- **Modules**:
  - `rgb_controller.rs` - Base RGB controller trait
  - `keyboard_rgb.rs` - Keyboard-specific RGB handling
  - `mouse_rgb.rs` - Mouse RGB handling
  - `headset_rgb.rs` - Headset RGB handling
  - `effects.rs` - Lighting effects library
- **Effects Supported**:
  - Static color
  - Breathing
  - Spectrum rainbow
  - Wave (multi-directional)
  - Reactive (key press activation)
  - Gradient
  - Custom per-zone mapping

#### 3.2.3 Mouse Tracker
- **Modules**:
  - `mouse_handler.rs` - HID sensor reading
  - `tracker_engine.rs` - Tracking visualization
  - `overlay_renderer.rs` - X11/Wayland overlay
- **Features**:
  - DPI display and adjustment
  - Sensor polling rate monitoring
  - Real-time movement visualization
  - Profile-based DPI switching

#### 3.2.4 GameSense Server
- **Technology**: Axum HTTP server
- **Port**: 27301 (default)
- **API Endpoints**:
  - `/state` - Current device state
  - `/battery` - Battery levels
  - `/volume` - Audio levels
  - `/temperature` - Device temperature
- **Compatibility**: Full SteelSeries GameSense protocol

#### 3.2.5 Audio Mixer
- **Backends**: PulseAudio, PipeWire
- **Features**:
  - Per-channel volume (Game, Chat, Master)
  - Mute controls
  - Chat Mix balance
  - Sonar API integration

### 3.3 Hardware Abstraction Layer (HAL)

#### 3.3.1 Udev Monitor
- **Technology**: rustudev or uevent
- **Functionality**:
  - Detect device plug/unplug events
  - Filter SteelSeries devices by VID/PID
  - Set up proper permissions (input group)

#### 3.3.2 HIDAPI Handler
- **Crate**: hidapi v2.6.6+
- **Operations**:
  - Open HID devices
  - Send/receive reports
  - Read interrupt endpoints
  - Query device descriptors

#### 3.3.3 EVDEV Parser
- **Use Case**: Alternative input path for certain devices
- **Features**:
  - Raw event parsing
  - Key code translation

### 3.4 Device Protocol Layer

#### 3.4.1 Keyboard Protocols
- **Supported Models**:
  - Apex Pro series (2019, 2023)
  - Apex 3, 5, 7 series
  - RLX, Stracato
- **Protocol Components**:
  - LED control reports (per-key and per-zone)
  - Actuation point adjustment (OmniPoint)
  - Macro storage
  - Profile switching

#### 3.4.2 Mouse Protocols
- **Supported Models**:
  - Rival series
  - Aerox series
  - Iron Wolf
  - Sparrow
- **Protocol Components**:
  - RGB zone control
  - DPI stages and adjustment
  - OLED display commands (if applicable)
  - Button remapping
  - Sensor calibration

#### 3.4.3 Headset Protocols
- **Supported Models**:
  - Arctis 1, 5, 7, 9 series
  - Arctis Pro / Pro Wireless
  - Arctis Nova Pro series
  - One Wireless
- **Protocol Components**:
  - Speaker driver control
  - RGB lighting (if applicable)
  - Microphone mute
  - Chat mix parameters

## 4. Data Flow

### 4.1 RGB Effect Application Flow
```
User (GUI) → CLI command → Daemon (D-Bus)
                        ↓
                RGB Engine
                        ↓
                Effect Calculator
                        ↓
                Device Protocol Encoder
                        ↓
                HIDAPI → Hardware
```

### 4.2 Mouse Tracking Flow
```
Hardware (HID Sensor) → HIDAPI Reader
                        ↓
                Tracker Engine (raw data)
                        ↓
                DPI Calculation
                        ↓
                Overlay Renderer (X11/Wayland)
                        ↓
                Display Update (60fps target)
```

### 4.3 GameSense State Flow
```
Games (HTTP POST) → GameSense Server
                       ↓
                State Processor
                       ↓
                Device Controller → LED/Audio updates
                       ↓
                Real-time feedback
```

## 5. Configuration Management

### 5.1 File Structure
```
~/.config/ssgg/
├── config.toml           # Global daemon configuration
├── profiles/
│   ├── default.json      # Default profile
│   └── gaming.json       # User-created profiles
├── effects/
│   └── custom_effect_1.json  # Custom lighting effects
└── logs/
    └── ssgd.log          # Daemon logs
```

### 5.2 Profile Format (JSON)
```json
{
  "name": "Gaming Profile",
  "devices": {
    "keyboard": {
      "rgb_mode": "wave",
      "rgb_color": "#00FF00",
      "brightness": 80,
      "actuation_points": {
        "w": 1.5,
        "a": 1.5,
        "s": 2.0,
        "d": 1.5
      }
    },
    "mouse": {
      "dpi_stages": [400, 800, 1600, 3200],
      "active_stage": 2,
      "rgb_enabled": true,
      "rgb_effect": "breathing"
    }
  }
}
```

## 6. Build System

### 6.1 Technology Stack
- **Language**: Rust (MSRV 1.97.1)
- **Build Tool**: Cargo
- **UI Framework**: GTK4 (with gtk4-rs bindings)
- **Dependencies**: See README.md for full list

### 6.2 Optional Features
```toml
[features]
default = []
audio = ["libpulse-binding"]       # Audio mixing
sonar = ["reqwest"]                # Sonar API integration
experimental-apex-2023 = []        # Experimental 2023 Apex support
wayland-overlay = []              # Wayland screen capture for tracker
```

### 6.3 Compilation Targets
```bash
# Basic build
cargo build --release

# With audio support
cargo build --release --features audio

# With all features
cargo build --release --all-features
```

## 7. Installation & Deployment

### 7.1 System Requirements
- Linux kernel 4.15+
- Rust 1.97.1+
- udev
- LIBUSB-1.0 (optional, for some mice)
- PulseAudio or PipeWire (for audio features)

### 7.2 Installation Steps
```bash
# Install udev rules
sudo cp assets/99-steelseries.rules /etc/udev/rules.d/
sudo udevadm control --reload-rules
sudo udevadm trigger

# Add user to input group
sudo usermod -aG input $USER

# Build and install
cargo build --release
sudo cp target/release/ssgg /usr/local/bin/
sudo cp assets/sssgd.service ~/.config/systemd/user/
systemctl --user daemon-reload
systemctl --user enable --now ssgg.service
```

### 7.3 Permissions Required
```
/dev/hidraw* - read/write (via udev rules, group: input)
/sys/bus/input - read (for device info)
Network port 27301 - local binding (127.0.0.1)
```

## 8. Security Considerations

### 8.1 Attack Surface
- **HIDAPI Access**: Sandboxed to SteelSeries devices only
- **Network**: Localhost-only binding, no external exposure
- **Filesystem**: User-owned configuration only
- **Permissions**: Minimum required (input group for HID access)

### 8.2 Mitigations
- Strict VID/PID filtering
- Capability-based device access
- Input sanitization for all commands
- Signed profile files (optional future feature)

## 9. Performance Targets

### 9.1 Latency Requirements
- **RGB Effect Updates**: <16ms (60fps visual)
- **Mouse Tracking Overlay**: <16ms refresh
- **GameSense Response**: <100ms from game event
- **Profile Switching**: <500ms apply time

### 9.2 Resource Usage
- **Memory**: <50MB idle, <100MB peak
- **CPU**: <2% single core idle, <10% during effect changes
- **Startup Time**: <2 seconds to ready state

## 10. Testing Strategy

### 10.1 Unit Tests
- Protocol encoding/decoding
- Effect calculations
- Profile serialization

### 10.2 Integration Tests
- Device enumeration
- Command execution
- D-Bus communication

### 10.3 Hardware Testing
- All supported keyboard models
- All supported mouse models
- All supported headset models
- Mixed device scenarios

## 11. Future Extensions

### 11.1 Planned Features
- **Mobile Companion App** (optional)
- **Cloud Profile Sync**
- **Advanced Macros with Scripting**
- **AI-powered Sound Profiles** (Sonar enhancement)
- **Wheel Support** (Stratus controllers)

### 11.2 Research Areas
- Firmware update mechanism (official SDK not available)
- Reverse-engineered proprietary protocols
- Wayland-native overlay implementation
- Kernel driver integration proposals

## 12. References & Resources

### 12.1 Existing Projects
- [steelseriesgg-rs](https://github.com/Ven0m0/steelseriesgg-rs) - Base reference
- [apex-tux](https://github.com/sfholmes/apex-tux) - Keyboard inspiration
- [apex7tkl_linux](https://github.com/Grz3r/apex7tkl_linux) - RGB patterns
- [chameth reverse engineering](https://chameth.com/reverse-engineering-arctis-pro-wireless-headset/)

### 12.2 APIs & Standards
- [HIDAPI Documentation](https://libhidapi.github.io/hidapi/)
- [SteelSeries GameSense API](https://www.steelseries.com/gamesense)
- [LibUSB Specification](http://libusb.info/)
- [udev Rules Reference](https://www.freedesktop.org/software/systemman/manpages/udev.5.html)

---

**Version**: 1.0  
**Last Updated**: September 22, 2026  
**Author**: Qoder Development Team
