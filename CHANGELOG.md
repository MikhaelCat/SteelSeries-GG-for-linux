# Changelog

All notable changes to this project will be documented in this file. The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Hardware testing suite for all SteelSeries devices (Python3 HIDAPI-based)
- Memory leak detection with ASan, TSan, and Valgrind integration
- Performance benchmarks for RGB latency optimization (<1ms target)
- Long-running daemon stability testing framework (24+ hours)

### Changed
- Updated dependency versions for improved compatibility
- Optimized systemd service resource limits (MemoryMax=128MB, CPUQuota=50%)
- Enhanced udev rules for broader device support

### Fixed
- Resolved workspace member configuration errors in Cargo.toml
- Corrected libusb crate name from `libusb-1.0` to `libusb-sys`
- Fixed non-existent GTK CSS procedure dependency
- Removed unused benchmark configurations

### Security
- Implemented comprehensive security audit automation
- Added memory protection via AddressSanitizer
- Enhanced systemd hardening with kernel protection flags

---

## [0.1.0] - 2026-09-22

### Initial Release

#### Core Features
- **Device Management**: Automatic SteelSeries device detection via HIDAPI/udev
  - Support for 25+ keyboard models (Apex series, RK-TUX)
  - Support for 18+ mouse models (Rival, Aerox, Iron Wolf)
  - Support for 14+ headset models (Arctis Pro/Nova series)

- **RGB Lighting Control**: Full per-device RGB management
  - Static color effects
  - Breathing animations
  - Spectrum rainbow wave
  - Multi-directional wave patterns
  - Per-zone customization

- **Mouse Tracking Engine**: Real-time sensor data capture
  - Polling rate adjustment (125Hz - 8000Hz)
  - DPI stage switching
  - X11/Wayland overlay rendering
  - Low-latency event processing

- **GameSense Integration**: HTTP API compatible with game telemetry
  - Battery level reporting
  - Temperature monitoring
  - Volume control integration
  - Device activity status

- **Audio Mixer (Experimental)**: PulseAudio/PipeWire integration
  - Per-channel volume (Master, Game, Chat)
  - Mute/unmute functionality
  - Sonar API compatibility

- **Profile System**: Configuration persistence and portability
  - JSON-based profile storage
  - Multiple profiles per user
  - Import/export capability
  - Device-specific settings

#### Build & Deployment
- Cross-platform Linux build system (Debian, Fedora, Arch, openSUSE, Alpine)
- Automated CI/CD with GitHub Actions
- Docker containerization for reproducible builds
- Package generation (.deb, .rpm, PKGBUILD, AppImage)

#### Infrastructure
- Systemd user service with automatic restart
- Udev rules for device permissions
- Comprehensive documentation (README, INSTALL, ARCHITECTURE)
- Automated security audit scripts
- Resource optimization configuration

#### Dependencies
- Rust 1.97.1 (MSRV)
- HIDAPI for hardware access
- Tokio for async runtime
- Axum for HTTP server (GameSense)
- Clap for CLI parsing
- Libpulse-binding for audio (optional)

#### Known Issues
- GTK4 GUI implementation pending (Phase 2)
- Sonar firmware updates not yet supported
- Limited test coverage for edge cases

---

## [0.0.1] - Pre-release

### Development Preview
- Basic HIDAPI device enumeration
- Prototype RGB color setting
- Initial CLI structure
