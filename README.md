# SteelSeries GG for Linux

A native Linux implementation of SteelSeries GG software, providing full hardware support for SteelSeries gaming devices including RGB lighting control, mouse tracking, device management, and GameSense integration.

## Features

- ✅ **RGB Lighting Control** - Full keyboard, mouse, and headset RGB with multiple effects
  - Static color
  - Breathing
  - Spectrum rainbow
  - Wave (multi-directional)
  - Reactive key press activation
  - Custom per-zone configuration
  
- ✅ **Device Management**
  - Automatic SteelSeries device detection via udev/HIDAPI
  - Support for 25+ keyboard models (Apex series, RK-TUX, etc.)
  - Support for 18+ mouse models (Rival, Aerox, Iron Wolf series)
  - Support for 14+ headset models (Arctis Pro/Nova series)
  
- ✅ **Mouse Tracking Overlay**
  - Real-time movement visualization
  - DPI stage switching
  - Sensor polling rate monitoring
  - X11/Wayland overlay rendering
  
- ✅ **GameSense Integration**
  - HTTP API compatible with games that support SteelSeries GameSense
  - Battery level reporting
  - Temperature monitoring
  - Volume control integration
  
- ✅ **Audio Mixer (Experimental)** ⚠️ requires `audio` feature flag
  - PulseAudio/PipeWire integration
  - Per-channel volume control (Master, Game, Chat)
  - Mute/unmute functionality
  - Sonar API integration

- ✅ **Profile System**
  - Save/load device configurations
  - Multiple profiles per user
  - JSON-based storage
  - Export/import capability

## Supported Devices

### Keyboards
- Apex Pro / Apex Pro TKL / Apex Pro TKL 2023
- Apex 3 / Apex 5 / Apex 7 series
- RK-680 TUX / RK-700 TUX / RK-800 TUX

### Mice
- Rival 105 / 3 / 3 Wireless / 5 / 300 series
- Aerox 0 / 5 / 9 Wireless
- Iron Wolf / Mini
- Sparrow / Xtkr

### Headsets
- Arctis 1 / 5 / 7 / 9 series
- Arctis Pro / Pro Wireless / Nova Pro series
- Arctis Nova 1 / 3 / 5
- One Wireless

## Installation

### Prerequisites

```bash
# Ubuntu/Debian
sudo apt install rustc cargo libhidapi-dev pkg-config build-essential

# Fedora
sudo dnf install rust cargo hidapi-devel pkgconf-pkg-config gcc

# Arch Linux
sudo pacman -S rust hidapi base-devel

# For audio features
sudo apt install libpulse-dev    # Debian/Ubuntu
sudo dnf install pulseaudio-libs.devel  # Fedora
sudo pacman -S pulseaudio-standalone    # Arch
```

### Install from Source

```bash
git clone https://github.com/qoder/steelseries-linux.git
cd steelseries-linux

# Basic build (no optional features)
./build.sh

# With audio support
./build.sh --features audio

# With all features
cargo build --release --all-features
```

### Install udev Rules (Required)

```bash
# Copy udev rules
sudo cp assets/99-steelseries.rules /etc/udev/rules.d/

# Reload udev rules
sudo udevadm control --reload-rules
sudo udevadm trigger

# Add user to input group (for HID access)
sudo usermod -aG input $USER

# Log out and back in for group changes to take effect
```

### Usage

#### List Connected Devices

```bash
target/release/ssgg devices [--detailed]
```

Example output:
```
Connected SteelSeries Devices:

| Type       | Model                 | Serial      | Firmware   |
|------------|-----------------------|-------------|------------|
| Keyboard   | Apex Pro              | ABC1234567  | Querying...|
| Mouse      | Rival 3               | XYZ9876543  | Querying...|
```

#### Control RGB Lighting

```bash
# Set static color
target/release/ssgg rgb color -c "cyan" -b 80

# Apply breathing effect
target/release/ssgg rgb effect -e breathing -c blue

# Apply spectrum effect (rainbow wave)
target/release/ssgg rgb effect -e spectrum

# Apply wave effect with direction
target/release/ssgg rgb effect -e wave -c cyan --direction left-to-right
```

#### Manage Profiles

```bash
# List all profiles
target/release/ssgg profile list

# Save current configuration as profile
target/release/ssgg profile save "gaming-profile"

# Load a profile
target/release/ssgg profile load gaming-profile

# Delete a profile
target/release/ssgg profile delete old-profile
```

#### Start Daemon (Background Service)

```bash
# Run daemon directly
target/release/ssgg daemon

# Or install as systemd service
cp assets/ssgg.service ~/.config/systemd/user/
systemctl --user daemon-reload
systemctl --user enable --now ssgg.service

# Check status
systemctl --user status ssgg.service
journalctl --user -u ssgg.service -f
```

#### GameSense Server

```bash
# Start GameSense server on default port (27301)
target/release/ssgg gamesense start

# Listen at localhost:27301 - games can send state updates here
curl http://localhost:27301/battery  # Query battery levels
curl http://localhost:27301/volume   # Query volume levels
```

#### Debug Tools

```bash
# View HID logs
target/release/ssgg hid-logs

# Device self-test
target/release/ssgg test-device

# Generate diagnostics report
target/release/ssgg debug diagnostics
```

## Configuration

Configuration files are stored in `~/.config/ssgg/`:

```toml
# config.toml
[gamesense]
enabled = true
bind = "127.0.0.1"
port = 27301

[general]
default_profile = "default"
debug = false
auto_start_daemon = true
```

## Building Optimizations

### Using sccache (Compilation Cache)

```bash
cargo install sscache

# Uncomment in .cargo/config.toml if desired
# [build]
# rustc-wrapper = "sccache"
```

### Using LLD Linker

```bash
# Ubuntu/Debian
sudo apt install lld

# Fedora
sudo dnf install lld

# Arch
sudo pacman -S lld

# Configure Cargo.toml .cargo/config.toml:
# [build]
# rustflags = ["-C", "link-arg=-fuse-ld=lld"]
```

## Architecture Overview

See [architecture-design.md](./architecture-design.md) for detailed system architecture documentation.

### Component Structure

```
ssgg/
├── src/
│   ├── main.rs          # CLI entry point
│   ├── lib.rs           # Library root
│   ├── device.rs        # Hardware detection & device management
│   ├── rgb.rs           # RGB lighting controllers & effects
│   ├── mouse.rs         # Mouse tracking engine
│   ├── gamesense.rs     # GameSense HTTP server
│   ├── audio.rs         # Audio mixer (experimental)
│   ├── config.rs        # Configuration & profile management
│   ├── protocol.rs      # Device communication protocols
│   └── effects.rs       # Lighting effect implementations
├── assets/
│   ├── 99-steelseries.rules  # udev permissions
│   └── ssgg.service          # Systemd unit
├── tools/                # Helper binaries
├── build.sh             # Build automation script
└── Cargo.toml           # Rust package manifest
```

## Development

### Project Structure

- **Core**: `src/device.rs`, `src/rgb.rs`, `src/mouse.rs`
- **Communication**: `src/protocol.rs`, `src/gamesense.rs`
- **UI**: GTK4 module (planned)
- **Testing**: See `Cargo.toml` dev-dependencies

### Adding Device Support

1. Find your device's PID by running:
   ```bash
   lsusb | grep SteelSeries
   ```

2. Add PID to `device.rs::is_steelseries_device()` matching function

3. Test with:
   ```bash
   ./target/release/ssgg devices --detailed
   ```

### Running Tests

```bash
cargo test

# Integration tests with hardware attached
cargo test -- --test-threads=1
```

## Troubleshooting

### Permission Denied Errors

If you get "Permission denied" when accessing devices:

```bash
# Verify udev rules installed
ls -la /etc/udev/rules.d/99-steelseries.rules

# Reload rules
sudo udevadm control --reload-rules

# Verify user is in input group
groups | grep input

# Check device permissions after plugging in device
ls -la /dev/hidraw*
```

### No Devices Detected

Check if devices are visible to kernel:

```bash
# List USB devices
lsusb | grep -i steelseries

# List HID devices
ls -la /dev/hidraw*

# Check kernel messages
dmesg | grep -i hid
```

### RGB Effects Not Working

Try starting daemon first:

```bash
# Start in foreground for debugging
./target/release/ssgg daemon

# Then run commands
./target/release/ssgg rgb color -c red
```

## Contributing

Contributions are welcome! Please follow these guidelines:

1. Code style: Run `cargo fmt && cargo clippy --all-targets --locked` before submitting
2. Documentation: Update docs when adding new features
3. Testing: Add tests for new functionality
4. License: MIT License

## Acknowledgments

- **SteelSeries GG** for inspiring this cross-platform implementation
- **[ven0m0/steelseriesgg-rs]**(https://github.com/Ven0m0/steelseriesgg-rs) - Open source reference
- **[apex-tux]**(https://github.com/sfholmes/apex-tux) - Keyboard RGB inspiration
- **[chameth]**(https://chameth.com/reverse-engineering-arctis-pro-wireless-headset/) - Reverse engineering work
- **HIDAPI team** for excellent hardware abstraction
- **Linux community** for udev, evdev, and other open source tools

## License

MIT License - see LICENSE file for details

---

**Version**: 0.1.0  
**Built with**: Rust 🦀, HIDAPI, axum, clap, tracing  
**MSRV**: Rust 1.97.1
