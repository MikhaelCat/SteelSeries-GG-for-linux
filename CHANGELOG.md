# Changelog

All notable changes to SteelSeries GG for Linux will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Initial project structure and architecture design
- Device management system with HIDAPI integration
- RGB lighting control module for keyboards, mice, and headsets
- Mouse tracking engine with low-latency capture
- GameSense HTTP API server implementation
- Audio mixer (experimental - requires `audio` feature flag)
- Profile management system with JSON serialization
- Effect rendering pipeline for RGB animations
- CLI interface with subcommands
- Daemon mode with systemd service support
- Comprehensive CI/CD pipeline with GitHub Actions
- Docker containerization for builds
- Makefile for build automation

### Implemented Features
✅ **Hardware Support**
- Keyboard detection (Apex series, RK-TUX series)
- Mouse detection (Rival, Aerox, Iron Wolf series)
- Headset detection (Arctis Pro/Nova series)
- Per-key RGB lighting controls
- Zone-based lighting effects
- DPI configuration for mice

✅ **RGB Effects**
- Static color mode
- Breathing effect
- Spectrum rainbow wave
- Multi-directional wave effect
- Reactive key press activation
- Gradient patterns
- Custom per-zone configuration

✅ **System Integration**
- udev rules for device permissions
- Systemd user service (ssgg.service)
- D-Bus communication (planned)
- Configuration persistence (~/.config/ssgg/)

### Changed
- No breaking changes

### Security
- Input validation on all command-line arguments
- Scoped PID access (0x1246 only)
- Localhost-only binding for GameSense server
- Minimal required permissions (input group)

## [0.1.0] - 2026-09-22

### Added - Initial Release

#### Core Functionality
- Device enumeration and management (`src/device.rs`)
  - HIDAPI integration
  - SteelSeries VID/PID filtering
  - Firmware version querying
  - Connection type detection
  
- RGB Lighting Controller (`src/rgb.rs`)
  - HSV/RGB color conversion
  - Effect calculation engine
  - Zone mapping for devices
  - Brightness scaling
  
- Mouse Tracking Engine (`src/mouse.rs`)
  - Sensor data parsing
  - Overlay renderer (X11/Wayland backend)
  - DPI stage switching
  - Movement visualization
  
- Protocol Layer (`src/protocol.rs`)
  - Keyboard protocol implementations
  - Mouse sensor protocols
  - OLED display commands
  - Audio control packets

#### Infrastructure
- Build System (`Cargo.toml`, `build.sh`)
- CI/CD Pipelines (`.github/workflows/`)
- Docker Containerization (`Dockerfile`)
- Development Tooling (`Makefile`, `.vscode/`)

#### Command Line Interface
- Device listing (`ssgg devices`)
- RGB control (`ssgg rgb`)
- Profile management (`ssgg profile`)
- Mouse tracking controls (`ssgg mouse`)
- GameServer startup (`ssgg gamesense start`)
- Daemon mode (`ssgg daemon`)

#### Documentation
- README.md with installation instructions
- Architecture design document
- Contributor guidelines
- Code comments and inline documentation

#### Testing
- Unit tests for utility functions
- Color conversion tests
- Protocol encoding tests
- Integration test framework setup

### Known Limitations
⚠️ Full per-key keyboard RGB control not yet verified on hardware
⚠️ Mouse sensor overlay rendering requires X11/Wayland libraries
⚠️ Audio features require PulseAudio/PipeWire development packages
⚠️ Some device PIDs pending hardware confirmation

## Future Roadmap

### Version 0.2.0 (Planned)
- GTK4/GTK3 GUI implementation
- Full per-key actuation point adjustment (OmniPoint keyboards)
- Complete OLED display support for mice
- Sonar audio processor integration
- Cloud profile synchronization

### Version 0.3.0 (Planned)
- Mobile companion app (optional)
- Advanced macro programming with scripting
- AI-powered sound profiles (enhanced Sonar)
- Wheel controller support
- Cross-platform firmware updates

---

For more details on each component, see individual source files or [ARCHITECTURE.md](./architecture-design.md).
