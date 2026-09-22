# Cross-Platform Installation Guide for SteelSeries GG for Linux

This guide covers installation across all major Linux distributions with no source code changes required.

## 🌍 Supported Distributions

### Primary Support (Tested & Verified)
- **Debian 11+** (Bullseye) - .deb packages via APT
- **Ubuntu 20.04+** (Focal+) - .deb packages via APT
- **Fedora 38+** - RPM packages via DNF
- **Arch Linux** (Any version) - PKGBUILD via pacman
- **openSUSE Tumbleweed/Leap 15.4+** - RPM packages via Zypper

### Secondary Support (Works but requires manual setup)
- **RHEL/CentOS/Rocky/AlmaLinux** 8+ - RPM packages
- **Alpine Linux** 3.17+ - APK packages (static binary)
- **Manjaro** - Based on Arch, use Arch packages

## 📦 Quick Install Methods

### Method 1: System Package Manager (Recommended)

#### Debian/Ubuntu
```bash
# Option A: Download DEB package
wget https://github.com/Ven0m0/steelseriesgg-rs/releases/latest/download/ssgg_*.deb
sudo dpkg -i ssgg_*.deb
sudo apt-get install -f  # Fix any dependency issues

# Option B: Add our repository (when available)
echo "deb [arch=amd64] https://packages.steelseries-linux.dev/apt/ stable main" | sudo tee /etc/apt/sources.list.d/steelseries.list
curl -fsSL https://packages.steelseries-linux.dev/gpg.key | sudo gpg --dearmor -o /usr/share/keyrings/steelseries.gpg
sudo apt update
sudo apt install ssgg
```

#### Fedora/RHEL/openSUSE
```bash
# Download RPM package
wget https://github.com/Ven0m0/steelseriesgg-rs/releases/latest/download/ssgg-*.rpm
sudo dnf install ./ssgg-*.rpm

# Or use COPR (when available)
sudo dnf copr enable steelseries/steelseries-gg-linux
sudo dnf install ssgg
```

#### Arch Linux
```bash
# From AUR (using yay or paru)
yay -S steelseries-gg-linux

# Or compile from PKGBUILD
git clone https://aur.archlinux.org/steelseries-gg-linux.git
cd steelseries-gg-linux
makepkg -si
```

### Method 2: Direct Build from Source

All distributions require the same basic dependencies:

```bash
# Install prerequisites
./setup-deps.sh

# Build project
cargo build --release
```

See individual distribution sections below for specific commands.

### Method 3: Universal AppImage
```bash
# Download AppImage
wget https://github.com/Ven0m0/steelseriesgg-rs/releases/latest/download/ssgg-*-x86_64.AppImage
chmod +x ssgg-*-x86_64.AppImage
./ssgg-*-x86_64.AppImage devices
```

## 🔧 Distribution-Specific Instructions

### Ubuntu/Debian Family

#### Dependencies
```bash
sudo apt update
sudo apt install -y \
    build-essential \
    pkg-config \
    libssl-dev \
    libhidapi-dev \
    libpulse-dev  # Optional: audio features
```

#### Build
```bash
# Ensure Rust is installed
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Clone and build
git clone https://github.com/Ven0m0/steelseriesgg-rs.git
cd steelseriesgg-rs
cargo build --release --all-features
```

#### Installation
```bash
# Local installation
sudo cp target/release/ssgg /usr/local/bin/
sudo mkdir -p /usr/local/lib/ssgg
sudo cp assets/* /usr/local/lib/ssgg/

# Configure udev rules
sudo cp /usr/local/lib/ssgg/99-steelseries.rules /etc/udev/rules.d/
sudo udevadm control --reload-rules

# Create systemd service
systemctl --user link /usr/local/lib/ssgg/ssgg.service
systemctl --user enable --now ssgg.service
```

#### Package Creation
```bash
# Create .deb package
make deb
# Result: dist/ssgg_*.deb
```

### Fedora/RHEL/openSUSE Family

#### Dependencies
```bash
# Fedora
sudo dnf install -y \
    gcc-c++ \
    pkgconf-pkg-config \
    openssl-devel \
    hidapi-devel \
    pulseaudio-libs-devel  # Optional

# RHEL/CentOS
sudo yum install -y \
    gcc-c++ \
    libpkgconf \
    openssl-devel \
    hidapi-devel

# openSUSE
sudo zypper install -y \
    gcc-c++ \
    make \
    pkg-config \
    libopenssl-devel \
    libhidapi-devel
```

#### Build
```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Build
cargo build --release --all-features
```

#### Installation
```bash
# Copy binary
sudo cp target/release/ssgg /usr/local/bin/
sudo chmod +x /usr/local/bin/ssgg

# Udev rules
sudo cp assets/99-steelseries.rules /etc/udev/rules.d/
sudo udevadm control --reload-rules

# Service
sudo cp assets/ssgg.service /lib/systemd/user/
systemctl --user daemon-reload
systemctl --user enable --now ssgg.service
```

#### Package Creation
```bash
# Create RPM package
make rpm
# Result: rpmbuild/RPMS/x86_64/ssgg-*.rpm
```

### Arch Linux

#### Dependencies
```bash
# Install from repositories
sudo pacman -Syu base-devel rust pkgconf hidapi openssl pulseaudio-standalone
```

#### Build
```bash
# Rust comes with system or install separately
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Build
cargo build --release --all-features
```

#### Installation
```bash
# Make it available system-wide
sudo cp target/release/ssgg /usr/bin/
sudo chmod +x /usr/bin/ssgg

# Use provided PKGBUILD instead (recommended)
cd arch-pkg  # Contains pre-generated PKGBUILD
makepkg -si
```

#### Package Creation
```bash
# Generate PKGBUILD
make pkgsrc
# Then in arch-pkg directory:
makepkg -si
```

### Alpine Linux

#### Dependencies
```bash
# Alpine uses apk package manager
sudo apk add \
    build-base \
    rust \
    cargo \
    musl-dev \
    pkgconf \
    openssl-dev \
    hidapi-dev \
    libusb-dev
```

#### Build (Static Binary Recommended)
```bash
# Build static binary for maximum compatibility
rustup target add x86_64-unknown-linux-musl
cargo build --release --target x86_64-unknown-linux-musl
strip target/x86_64-unknown-linux-musl/release/ssgg
```

#### Installation
```bash
sudo cp target/x86_64-unknown-linux-musl/release/ssgg /usr/local/bin/
```

## 📦 Packaging Targets

### DEB Package (.deb)
Target: Debian, Ubuntu, Linux Mint, Pop!_OS, Elementary OS

```bash
make deb
ls -lh debian-pkg/*.deb
```

Features included:
- ✅ Automatic dependency resolution via DEBIAN/control
- ✅ Post-install scripts for udev/systemd integration
- ✅ Compatible with all Debian-based systems

### RPM Package (.rpm)
Target: Fedora, RHEL, CentOS, Rocky, AlmaLinux, openSUSE

```bash
make rpm
ls -lh rpmbuild/RPMS/*/ssgg-*.rpm
```

Features included:
- ✅ Spec file with proper requirements
- ✅ Systemd unit file support
- ✅ udev rules inclusion

### PKGBUILD
Target: Arch Linux, Manjaro, EndeavourOS

```bash
make pkgsrc
ls -lh arch-pkg/PKGBUILD
```

Features included:
- ✅ AUR-ready metadata
- ✅ Arch-specific optimizations
- ✅ Proper license declaration

### AppImage
Target: Universal (any distro with run-time support)

```bash
make appimage
ls -lh appimage-pkg/*.AppImage
```

Features included:
- ✅ Single portable executable
- ✅ No installation required
- ✅ Works across desktop environments

## 🐳 Docker-Based Builds

For guaranteed reproducible builds across all distributions:

```bash
# Debian/Ubuntu package
docker build -t ssgg-deb-build -f Dockerfile.debian .
docker run --rm -v $(pwd)/debian-pkg:/output ssgg-deb-build

# Fedora/RPM package  
docker build -t ssgg-rpm-build -f Dockerfile.fedora .
docker run --rm -v $(pwd)/rpm-pkg:/output ssgg-rpm-build
```

## 🧪 Testing on Multiple Platforms

Use GitHub Actions CI to verify builds automatically (already configured):
- Push to trigger workflows
- Check `Actions` tab for build status
- Download artifacts from successful runs

Manual testing checklist per platform:
1. `[ ]` Dependencies install without errors
2. `[ ]` Build completes successfully
3. `[ ]` Binary executes correctly
4. `[ ]` Devices detected properly
5. `[ ]` RGB controls work
6. `[ ]` Daemon starts via systemd
7. `[ ]` Package installs cleanly

## ⚠️ Common Issues & Solutions

### Issue: Permission Denied When Accessing HID Devices
```bash
# Solution: Add user to input group
sudo usermod -aG input $USER
# Log out and back in for changes to take effect
```

### Issue: Cargo Build Fails with Missing Libraries
```bash
# Solution: Run dependency installer
./setup-deps.sh
# Or manually install based on your distribution
```

### Issue: Audio Features Not Working
```bash
# Solution: Ensure PulseAudio/PipeWire headers are installed
# Ubuntu/Debian: sudo apt install libpulse-dev
# Fedora: sudo dnf install pulseaudio-libs-devel
# Arch: sudo pacman -S pipewire pulseaudio-standalone
```

### Issue: Binary Won't Start (Missing Shared Libraries)
```bash
# Solution: Build static binary
cargo build --release --target x86_64-unknown-linux-musl
```

## 🔄 Migration from Windows Version

Users migrating from Windows SteelSeries GG should note:

1. Device drivers are now handled by Linux kernel's HID subsystem
2. RGB effects may differ slightly due to hardware limitations
3. GameSense HTTP API is compatible with Windows versions
4. Profiles can be imported/exported via JSON format

## 📋 Verification Checklist

After installation, verify functionality:

```bash
# 1. Check device detection
ssgg devices --detailed

# 2. Test RGB controls
ssgg rgb color red
ssgg rgb effect breathing --color cyan

# 3. Verify daemon is running
systemctl --user status ssgg.service

# 4. Test GameSense server
curl http://localhost:27301/state

# 5. Check configuration files
cat ~/.config/ssgg/config.toml
```

## 🆘 Troubleshooting

### Enable Verbose Logging
```bash
export RUST_LOG=debug,ssgg=trace
ssgg devices --verbose
```

### Debug HID Access
```bash
# Check if devices are visible to kernel
ls -la /dev/hidraw*

# Monitor kernel messages
dmesg -w | grep hid
```

### View Daemon Logs
```bash
journalctl --user -u ssgg.service -f
```

## 📞 Support & Resources

- **Issue Tracker**: https://github.com/Ven0m0/steelseriesgg-rs/issues
- **Documentation**: Full docs at https://github.com/Ven0m0/steelseriesgg-rs/wiki
- **Community**: Join our Discord for real-time help
- **Mailing List**: Subscribe at steelseries-linux.dev/mailman

---

**Last Updated**: September 22, 2026  
**Version**: 0.1.5  
**Maintenance Status**: Active development
