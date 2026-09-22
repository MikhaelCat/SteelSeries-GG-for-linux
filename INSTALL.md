# Complete Installation Guide - All Major Linux Distributions

## Quick Start (Recommended)

After installing the daemon via your package manager of choice, the GUI will automatically connect and allow keyboard RGB control without additional configuration.

```bash
# Auto-install script (works on ALL distributions)
wget https://github.com/MikhaelCat/SteelSeries-GG-for-linux/releases/latest/download/install.sh
chmod +x install.sh
sudo ./install.sh
systemctl --user enable --now ssgg
ssgg-gui &
```

---

## Platform-Specific Installation

### Debian 11+ / Ubuntu 20.04+

#### Method 1: APT Package (Recommended)

```bash
# Add repository GPG key
wget -qO - https://github.com/MikhaelCat/SteelSeries-GG-for-linux/releases/latest/download/ssgg.gpg.key | sudo gpg --dearmor -o /usr/share/keyrings/ssgg-archive-keyring.gpg

echo "deb [signed-by=/usr/share/keyrings/ssgg-archive-keyring.gpg] https://packages.steelseries-linux.dev/debian stable main" | sudo tee /etc/apt/sources.list.d/steelseries.list

sudo apt update
sudo apt install ssgg
```

#### Method 2: Manual DEB Installation

```bash
wget https://github.com/MikhaelCat/SteelSeries-GG-for-linux/releases/latest/download/ssgg_0.1.0_amd64.deb
sudo dpkg -i ssgg_0.1.0_amd64.deb
sudo systemctl --user enable --now ssgg
ssgg-gui &
```

---

### Fedora 38+ / RHEL/CentOS 8+

#### Method 1: DNF Package

```bash
# Install repository
sudo wget -P /etc/yum.repos.d/ \
  https://github.com/MikhaelCat/SteelSeries-GG-for-linux/releases/latest/download/ssgg.repo

sudo dnf makecache
sudo dnf install ssgg
```

#### Method 2: RPM Installation

```bash
wget https://github.com/MikhaelCat/SteelSeries-GG-for-linux/releases/latest/download/ssgg-0.1.0.x86_64.rpm
sudo dnf install ssgg-0.1.0.x86_64.rpm
sudo systemctl --user enable --now ssgg
ssgg-gui &
```

---

### Arch Linux / Manjaro

#### Method 1: AUR Package (Recommended)

```bash
# Using an AUR helper (yay, paru, etc.)
yay -S ssgg-git

# Or manually from AUR
git clone https://aur.archlinux.org/ssgg-git.git
cd ssgg-git
makepkg -si
systemctl --user enable --now ssgg
ssgg-gui &
```

#### Method 2: From Source

```bash
sudo pacman -S base-devel rust cargo
cargo install --path .
rustc build.rs

# Setup systemd service
cp assets/ssgg.service ~/.config/systemd/user/
systemctl --user daemon-reload
systemctl --user enable --now ssgg
ssgg-gui &
```

---

### openSUSE Tumbleweed / Leap 15.4+

```bash
# Install repository
sudo zypper ar https://download.opensuse.org/repositories/home:ssgg/openSUSE_Tumbleweed/home:ssgg.repo
sudo rpm --import https://download.opensuse.org/repositories/home:ssgg/openSUSE_Tumbleweed/repodata/repomd.xml.key

sudo zypper refresh
sudo zypper install ssgg
```

---

### Alpine Linux 3.17+

```bash
# Add repositories
sudo echo "http://dl-cdn.alpinelinux.org/alpine/edge/community" >> /etc/apk/repositories

sudo apk add rust cargo musl-dev libusb-compat hidapi

cargo install --path .

# Create systemd user directory
mkdir -p ~/.config/systemd/user
cp assets/ssgg.service ~/.config/systemd/user/
systemctl --user daemon-reload
systemctl --user enable --now ssgg
ssgg-gui &
```

---

## Building from Source (All Platforms)

### Prerequisites

#### Debian/Ubuntu
```bash
sudo apt update
sudo apt install -y rustc cargo pkg-config libhidapi-dev libssl-dev libpulse-dev
```

#### Fedora/RHEL
```bash
sudo dnf group install "Development Tools"
sudo dnf install -y rust cargo pkgconfig hidapi-devel openssl-devel pulseaudio-libs-devel
```

#### Arch Linux
```bash
sudo pacman -Syu rust cargo pkgconf hidapi openssl pulseaudio
```

#### openSUSE
```bash
sudo zypper install rust-default cargo pkg-config libhidapi libopenssl3 pulseaudio-lib
```

#### Alpine
```bash
sudo apk add rust cargo pkgconf hidapi-dev openssl-dev pulseaudio-dev
```

### Build Commands

```bash
# Clone repository
git clone https://github.com/MikhaelCat/SteelSeries-GG-for-linux.git
cd SteelSeries-GG-for-linux

# Check dependencies
./setup-deps.sh

# Build optimized release binary
make build

# Run tests
make test

# Install
sudo make install
```

---

## Post-Installation Steps

### 1. Udev Rules (Device Permissions)

```bash
# Copy udev rules
sudo cp assets/99-steelseries.rules /etc/udev/rules.d/
sudo udevadm control --reload-rules
sudo udevadm trigger

# Add user to input group
sudo usermod -aG input $USER
```

**Note:** Logout/login required for group changes to take effect.

### 2. Verify Daemon is Running

```bash
systemctl --user status ssgg
journalctl --user -u ssgg -f
```

### 3. Test CLI Interface

```bash
# List connected devices
ssgg devices --detailed

# Show RGB color
ssgg rgb color --red 255 --green 0 --blue 0

# Get device profile
ssgg profile list
```

### 4. Launch GUI

```bash
# Option 1: From command line (if GUI support enabled)
ssgg-gui &

# Option 2: Menu application
Find "SteelSeries GG" in your applications menu
```

---

## Troubleshooting

### Device Not Detected

```bash
# Check if device is accessible
ls -la /dev/hidraw*
sudo usermod -aG input $USER
```

### Port Already in Use

```bash
# GameSense port conflict (default: 27301)
ssgg daemon --game-port 27302
```

### Permission Denied

```bash
# Fix permissions
sudo chown -R $USER:$USER ~/.config/ssgg
sudo chmod 755 ~/.config/ssgg
```

### Audio Features Not Working

```bash
# Ensure PulseAudio/PipeWire is running
systemctl --user --status pipewire
```

---

## Uninstallation

### Remove Package

#### Debian/Ubuntu
```bash
sudo apt remove ssgg
sudo apt autoremove
```

#### Fedora/RHEL
```bash
sudo dnf remove ssgg
```

#### Arch Linux
```bash
sudo pacman -R ssgg
```

### Cleanup Configuration

```bash
rm -rf ~/.config/ssgg
rm -rf ~/.local/share/ssgg
systemctl --user disable --now ssgg
rm ~/.config/systemd/user/ssgg.service
systemctl --user daemon-reload
```

---

## Security Considerations

The systemd service includes:
- ✅ Memory limit: 128MB maximum
- ✅ CPU usage capped at 50%
- ✅ Kernel protection enabled
- ✅ No new privileges
- ✅ Restricted network access
- ✅ Private temporary directory

For security audit results, run:
```bash
scripts/security-audit.sh
```

---

## Updates

### Automatic (APT/DNF)

```bash
sudo apt update && sudo apt upgrade ssgg
# or
sudo dnf update ssgg
```

### Manual

```bash
# Download latest release
wget https://github.com/MikhaelCat/SteelSeries-GG-for-linux/releases/latest/download/ssgg-*

# Install
sudo dpkg -i ssgg_*  # Debian/Ubuntu
sudo dnf install ssgg-*  # Fedora/RHEL
```

---

## Support

- 📖 **Documentation:** https://github.com/MikhaelCat/SteelSeries-GG-for-linux/wiki
- 🐛 **Issues:** https://github.com/MikhaelCat/SteelSeries-GG-for-linux/issues
- 💬 **Discussions:** https://github.com/MikhaelCat/SteelSeries-GG-for-linux/discussions
- 🔒 **Security Reports:** security@steelseries-linux.dev
