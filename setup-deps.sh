#!/usr/bin/env bash
# Cross-Platform Dependency Installer for SteelSeries GG for Linux
# Detects Linux distribution and installs required dependencies automatically
# Supports: Debian/Ubuntu, Fedora/RHEL, Arch Linux, openSUSE, and more

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
LOG_FILE="/tmp/ssgg-dep-install-$(date +%Y%m%d-%H%M%S).log"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

log() {
    echo -e "${BLUE}[$(date '+%H:%M:%S')]${NC} $*" | tee -a "$LOG_FILE"
}

success() {
    echo -e "${GREEN}[✓]${NC} $*"
}

warn() {
    echo -e "${YELLOW}[!]${NC} $*"
}

error() {
    echo -e "${RED}[✗]${NC} $*" >&2
}

detect_distro() {
    if [[ -f /etc/os-release ]]; then
        . /etc/os-release
        echo "$ID"
    elif command -v lsb_release >/dev/null; then
        lsb_release -is | tr '[:upper:]' '[:lower:]'
    else
        echo "unknown"
    fi
}

get_codename() {
    if [[ -f /etc/os-release ]]; then
        . /etc/os-release
        echo "${VERSION_CODENAME:-}"
    else
        echo ""
    fi
}

check_root() {
    if [[ $EUID -ne 0 ]]; then
        error "This script must be run as root (use sudo)"
        exit 1
    fi
}

install_ubuntu_debian() {
    log "Installing dependencies for Ubuntu/Debian..."
    
    apt-get update || {
        error "Failed to update apt cache"
        exit 1
    }
    
    DEBIAN_PACKAGES=(
        "build-essential"
        "pkg-config"
        "libssl-dev"
        "libhidapi-dev"
    )
    
    # Add audio support packages
    if [[ "$INSTALL_AUDIO" == true ]]; then
        DEBIAN_PACKAGES+=("libpulse-dev")
    fi
    
    install_packages "apt" "${DEBIAN_PACKAGES[@]}"
}

install_fedora_rhel() {
    log "Installing dependencies for Fedora/RHEL..."
    
    dnf install -y \
        gcc-c++ \
        pkgconfig \
        openssl-devel \
        hidapi-devel \
        || yum install -y \
            gcc-c++ \
            libpkgconf \
            openssl-devel \
            hidapi-devel
    
    # Audio support
    if [[ "$INSTALL_AUDIO" == true ]]; then
        dnf install -y pulseaudio-libs-devel \
            || yum install -y pulseaudio-libs-devel
    fi
    
    success "Dependencies installed via DNF/YUM"
}

install_arch() {
    log "Installing dependencies for Arch Linux..."
    
    pacman -Syu --noconfirm \
        base-devel \
        pkgconf \
        openssl \
        hidapi \
        || {
        warn "pacman failed, trying refresh first..."
        pacman -Sy --noconfirm
        pacman -S --noconfirm \
            base-devel \
            pkgconf \
            openssl \
            hidapi
    }
    
    if [[ "$INSTALL_AUDIO" == true ]]; then
        pacman -S --noconfirm \
            pulseaudio-standalone \
            libpulse
    fi
    
    success "Dependencies installed via Pacman"
}

install_opensuse() {
    log "Installing dependencies for openSUSE..."
    
    zypper install -y \
        gcc-c++ \
        make \
        pkg-config \
        libopenssl-devel \
        libhidapi-devel
    
    if [[ "$INSTALL_AUDIO" == true ]]; then
        zypper install -y \
            pulseaudio-devel \
            libpulse-simple0
    fi
    
    success "Dependencies installed via Zypper"
}

install_suse() {
    log "Installing dependencies for SUSE..."
    
    zypper install -y \
        gcc-c++ \
        make \
        pkg-config \
        libopenssl-devel \
        libhidapi-devel \
        || rpm -Uvh \
            http://download.opensuse.org/repositories/openSUSE:/Factory/x86_64/openssl-devel-*-1.x86_64.rpm
    
    success "Dependencies installed via Zypper/RPM"
}

install_alpine() {
    log "Installing dependencies for Alpine Linux..."
    
    apk add \
        alpine-base \
        build-base \
        openssl-dev \
        hidapi-dev \
        pkgconf
    
    if [[ "$INSTALL_AUDIO" == true ]]; then
        apk add \
            pipewire-dev \
            pulseaudio-utils
    fi
    
    success "Dependencies installed via Apk"
}

install_manjaro() {
    # Manjaro is Arch-based, use arch function
    install_arch
}

install_packman() {
    # Packman repository for openSUSE
    warn "Installing Packman repository..."
    zypper ar -cfp 99 https://ftp.gwdg.de/pub/linux/misc/packman/suse/openSUSE_Tumbleweed/ packman
    zypper modifyrepo --disable packman
    zypper dist-removal --auto --allow-vendor-change || true
    success "Packman repository added"
}

check_dependency_installed() {
    local package=$1
    local checker=$2
    
    if eval "$checker"; then
        return 0
    else
        return 1
    fi
}

install_packages_apt() {
    local packages=("$@")
    
    # Check which ones are not installed
    local missing=()
    for pkg in "${packages[@]}"; do
        if ! dpkg -l | grep -q "^ii.*$pkg"; then
            missing+=("$pkg")
        fi
    done
    
    if [[ ${#missing[@]} -gt 0 ]]; then
        apt-get install -y "${missing[@]}" || {
            error "Failed to install: ${missing[*]}"
            exit 1
        }
        success "Installed: ${missing[*]}"
    else
        log "All packages already installed"
    fi
}

install_packages_dnf() {
    local packages=("$@")
    
    for pkg in "${packages[@]}"; do
        if ! rpm -q "$pkg" &>/dev/null; then
            dnf install -y "$pkg" || yum install -y "$pkg"
            success "Installed: $pkg"
        fi
    done
}

install_packages_pacman() {
    local packages=("$@")
    
    for pkg in "${packages[@]}"; do
        if ! pacman -Q "$pkg" &>/dev/null; then
            pacman -S --noconfirm "$pkg"
            success "Installed: $pkg"
        fi
    done
}

install_packages_zypper() {
    local packages=("$@")
    
    for pkg in "${packages[@]}"; do
        if ! rpm -q "$pkg" &>/dev/null; then
            zypper install -y "$pkg"
            success "Installed: $pkg"
        fi
    done
}

main() {
    echo -e "${BLUE}╔═══════════════════════════════════════════════════╗${NC}"
    echo -e "${BLUE}║   SteelSeries GG - Cross-Platform Dependency      ║${NC}"
    echo -e "${BLUE}║     Auto-Detection and Installation Script       ║${NC}"
    echo -e "${BLUE}╚═══════════════════════════════════════════════════╝${NC}"
    echo ""
    
    # Parse arguments
    INSTALL_AUDIO=false
    for arg in "$@"; do
        case $arg in
            --audio)
                INSTALL_AUDIO=true
                ;;
            --skip-root)
                shift # skip root check
                ;;
            *)
                # ignore unknown args
                ;;
        esac
    done
    
    log "Running from directory: $(pwd)"
    log "Log file: $LOG_FILE"
    
    # Detect distribution
    DISTRO=$(detect_distro)
    CODENAME=$(get_codename)
    
    echo -e "${GREEN}Detected Distribution:${NC} $DISTRO"
    if [[ -n "$CODENAME" ]]; then
        echo -e "${GREEN}Codename:${NC} $CODENAME"
    fi
    echo ""
    
    # Install based on distribution family
    case "$DISTRO" in
        ubuntu|debian|linuxmint|pop|elementary)
            check_root
            install_ubuntu_debian
            ;;
        fedora|rhel|centos|almalinux|rocky)
            check_root
            install_fedora_rhel
            ;;
        arch|manjaro|garuda|endeavouros)
            # Can optionally avoid root check for pacman
            if [[ "$1" != "--skip-root" ]]; then
                check_root
            fi
            install_arch
            ;;
        opensuse*|sles)
            check_root
            install_opensuse
            ;;
        suse)
            check_root
            install_suse
            ;;
        alpine)
            check_root
            install_alpine
            ;;
        *)
            warn "Unknown distribution: $DISTRO"
            warn "Please manually install required dependencies:"
            cat <<EOF
Required dependencies for all distributions:
  - C compiler and build tools (gcc, g++, make)
  - pkg-config
  - OpenSSL development headers (libssl-dev or openssl-devel)
  - HIDAPI development headers (libhidapi-dev or hidapi-devel)
  
Optional for audio features:
  - PulseAudio/PipeWire development headers
  
See individual distribution documentation for package names.
EOF
            exit 1
            ;;
    esac
    
    echo ""
    echo -e "${GREEN}==============================================${NC}"
    echo -e "${GREEN}Installation complete!${NC}"
    echo -e "${GREEN}==============================================${NC}"
    echo ""
    echo "Next steps:"
    echo "1. Verify Rust toolchain is installed:"
    echo "   rustc --version && cargo --version"
    echo ""
    echo "2. Build the project:"
    echo "   cd steelseries-gg-linux"
    echo "   cargo build --release"
    echo ""
    echo "3. Run tests:"
    echo "   cargo test"
    echo ""
    
    log "Dependency installation completed successfully"
}

main "$@"
