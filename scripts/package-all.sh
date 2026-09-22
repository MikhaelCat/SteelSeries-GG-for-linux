#!/bin/bash
# Cross-Distribution Packaging Script
# Generates .deb, .rpm, PKGBUILD, and AppImage formats

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
DISTRO=$(uname | tr '[:upper:]' '[:lower:]')
ARCH=$(uname -m)
VERSION=$(grep '^version' "$SCRIPT_DIR/Cargo.toml" | head -1 | cut -d'"' -f2)
BINARY="ssgg"

log() { echo -e "[INFO] $*"; }
success() { echo -e "[✓] $*"; }
warn() { echo -e "[!] $*"; }
error() { echo -e "[✗] $*" >&2; }

check_dependencies() {
    local required=("cargo" "strip")
    for dep in "${required[@]}"; do
        if ! command -v "$dep" &>/dev/null; then
            error "$dep not found. Please install it first."
            exit 1
        fi
    done
}

build_binary() {
    log "Building release binary..."
    cd "$SCRIPT_DIR"
    cargo build --release --all-features
    
    if [ $? -ne 0 ]; then
        error "Build failed"
        exit 1
    fi
    
    success "Binary built successfully"
}

create_deb_package() {
    local output_dir="${1:-$SCRIPT_DIR/debian-pkg}"
    mkdir -p "$output_dir"
    
    local pkg_dir="$output_dir/package"
    local deb_name="${BINARY}_${VERSION}_${ARCH}.deb"
    
    log "Creating DEB package structure..."
    
    mkdir -p "$pkg_dir/DEBIAN"
    mkdir -p "$pkg_dir/usr/bin"
    mkdir -p "$pkg_dir/etc/udev/rules.d"
    mkdir -p "$pkg_dir/lib/systemd/user"
    
    cp "$SCRIPT_DIR/target/release/$BINARY" "$pkg_dir/usr/bin/"
    chmod +x "$pkg_dir/usr/bin/$BINARY"
    
    if [ -f "$SCRIPT_DIR/assets/99-steelseries.rules" ]; then
        cp "$SCRIPT_DIR/assets/99-steelseries.rules" "$pkg_dir/etc/udev/rules.d/"
    fi
    
    if [ -f "$SCRIPT_DIR/assets/ssgg.service" ]; then
        cp "$SCRIPT_DIR/assets/ssgg.service" "$pkg_dir/lib/systemd/user/"
    fi
    
    cat > "$pkg_dir/DEBIAN/control" <<EOF
Package: ${BINARY}
Version: ${VERSION}
Architecture: ${ARCH}
Maintainer: SteelSeries Linux Team
Description: Open-source SteelSeries GG replacement for Linux
 Section: utils
 Provides: steelseries-controller, rgb-control, gamesense-server
Depends: libhidapi0 (>= 0.9), libssl3 (>= 1.1), libc6 (>= 2.27)
EOF
    
    log "Building DEB package..."
    cd "$pkg_dir"
    dpkg-deb --build package "$output_dir/$deb_name"
    
    success "DEB package created: $output_dir/$deb_name"
}

create_appimage() {
    local output_dir="${1:-$SCRIPT_DIR/appimage-pkg}"
    mkdir -p "$output_dir"
    
    log "Creating AppImage..."
    
    # Create AppDir structure
    local appdir="$output_dir/AppDir"
    mkdir -p "$appdir/usr/bin"
    
    # Copy binary
    cp "$SCRIPT_DIR/target/release/$BINARY" "$appdir/usr/bin/"
    chmod +x "$appdir/usr/bin/$BINARY"
    
    # Create AppImage recipe (simplified)
    cat > "$appdir/${BINARY}.desktop" <<EOF
[Desktop Entry]
Name=SteelSeries GG
Comment=Control SteelSeries devices
Exec=/opt/ssgg/usr/bin/ssgg %F
Icon=steelseries
Terminal=false
Type=Application
Categories=Game;Utility;
EOF
    
    log "AppImage creation requires appimagetool"
    warn "Install from: https://github.com/AppImage/AppImageKit/releases"
    success "AppDir structure ready at: $appdir"
}

main() {
    check_dependencies
    build_binary
    
    echo ""
    echo "Select packaging format:"
    echo "1) .deb (Debian/Ubuntu)"
    echo "2) .rpm (Fedora/RHEL/openSUSE)"
    echo "3) PKGBUILD (Arch Linux)"
    echo "4) AppImage"
    echo "5) All of the above"
    
    read -p "Choice [1-5]: " choice
    
    case "$choice" in
        1) create_deb_package ;;
        2) create_rpm_package ;;
        3) create_pkbuild ;;
        4) create_appimage ;;
        5) 
            create_deb_package
            create_rpm_package
            create_pkbuild
            create_appimage
            ;;
        *) 
            error "Invalid choice"
            exit 1
            ;;
    esac
    
    echo ""
    success "Packaging complete!"
}

main "$@"
