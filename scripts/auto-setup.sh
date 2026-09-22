#!/bin/bash
# SteelSeries GG - Automated Daemon Setup Script
# Configures and launches everything with minimal user intervention

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
LOG_FILE="/tmp/ssgg-setup-$(date +%Y%m%d-%H%M%S).log"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
BLUE='\033[0;34m'
NC='\033[0m'

info() { echo -e "${BLUE}[INFO]${NC} $*"; }
success() { echo -e "${GREEN}[✓]${NC} $*"; }
warning() { echo -e "${YELLOW}[!]${NC} $*"; }
error() { echo -e "${RED}[✗]${NC} $*" >&2; }

log() { echo "[$(date '+%H:%M:%S')] $*" | tee -a "$LOG_FILE"; }

# Check if running as root (shouldn't be for user services)
if [ "$EUID" -eq 0 ]; then
    error "This script should NOT run as root. Running as regular user."
    exit 1
fi

echo "=============================================="
echo "  SteelSeries GG - Automatic Setup"
echo "=============================================="
echo ""

# Step 1: Install udev rules
info "Installing udev rules..."
if sudo cp "$SCRIPT_DIR/assets/99-steelseries.rules" /etc/udev/rules.d/; then
    sudo udevadm control --reload-rules
    sudo udevadm trigger
    success "udev rules installed successfully"
else
    warning "Failed to install udev rules manually"
fi

# Step 2: Add user to input group
info "Checking input group membership..."
if groups | grep -q '\binput\b'; then
    success "User already in input group"
else
    info "Adding user to input group..."
    sudo usermod -aG input $USER
    warning "Log out and log back in for group changes to take effect"
fi

# Step 3: Build binary if not present
info "Checking for existing binary..."
if [ -f target/release/ssgg ]; then
    success "Binary already built, skipping compilation"
else
    info "Building binary from source..."
    if command -v cargo &> /dev/null; then
        cd "$SCRIPT_DIR"
        cargo build --release --quiet || {
            error "Build failed. Installing dependencies..."
            ./setup-deps.sh || {
                error "Dependency installation failed. Please run ./setup-deps.sh manually"
                exit 1
            }
            cargo build --release --quiet
        }
        success "Build completed"
    else
        warning "Cargo not found. Binary must be installed separately."
    fi
fi

# Step 4: Install binary to system
info "Installing binary to /usr/local/bin..."
if sudo cp "$SCRIPT_DIR/target/release/ssgg" /usr/local/bin/ 2>/dev/null; then
    sudo chmod 755 /usr/local/bin/ssgg
    success "Binary installed to /usr/local/bin/ssgg"
else
    warning "Could not install to /usr/local/bin. Using path instead"
    export PATH="$SCRIPT_DIR/target/release:$PATH"
    echo "Export PATH=\"$SCRIPT_DIR/target/release:\$PATH\" >> ~/.bashrc" >> "$LOG_FILE"
fi

# Step 5: Setup systemd user service
info "Setting up systemd user service..."
mkdir -p "$HOME/.config/systemd/user/"
sudo cp "$SCRIPT_DIR/assets/ssgg.service" "$HOME/.config/systemd/user/"
sudo chown $USER:"$SUDO_USER" "$HOME/.config/systemd/user/ssgg.service" 2>/dev/null || \
    cp "$SCRIPT_DIR/assets/ssgg.service" "$HOME/.config/systemd/user/"

# Reload systemd daemon
if command -v systemctl &> /dev/null; then
    systemctl --user daemon-reload
    success "Systemd service configured"
fi

# Step 6: Enable and start service
info "Enabling and starting ssgg daemon..."
if command -v systemctl &> /dev/null; then
    systemctl --user enable --now ssgg 2>&1 || {
        warning "Could not auto-start service. Starting manually..."
        systemctl --user start ssgg
    }
    success "Daemon started and enabled for autostart"
else
    warning "systemctl not available. Service won't auto-start"
fi

# Step 7: Verify service is running
info "Verifying daemon status..."
sleep 2
if systemctl --user is-active ssgg 2>/dev/null; then
    success "ssgg daemon is running"
    journalctl --user -u ssgg -n 10 --no-pager || true
else
    error "Daemon failed to start. Check logs:"
    journalctl --user -u ssgg -n 30 --no-pager
    exit 1
fi

# Step 8: Detect display and prepare for GUI
info "Detecting display server..."
export DISPLAY=${DISPLAY:-":0"}
XAUTHORITY=${XAUTHORITY:-"$HOME/.Xauthority"}

if [ -n "$DISPLAY" ] && [ -n "$XAUTHORITY" ] && [ -f "$XAUTHORITY" ]; then
    success "Display detected: $DISPLAY"
    
    # Create GUI launcher script
    cat > "$HOME/.local/bin/ssgg-gui" << EOF
#!/bin/bash
export DISPLAY=$DISPLAY
export XAUTHORITY=$XAUTHORITY
exec ssgg gui &
EOF
    mkdir -p "$HOME/.local/bin"
    chmod +x "$HOME/.local/bin/ssgg-gui"
    success "GUI launcher created: ~/.local/bin/ssgg-gui"
    
    info "Optional: Launch GUI with 'ssgg-gui' or from applications menu"
else
    warning "No display detected. GUI will need manual launch after X11/Wayland starts."
fi

# Step 9: Create convenience aliases
info "Creating shell alias..."
alias_config="$HOME/.bash_aliases"
if ! grep -q "ssgg" "$alias_config" 2>/dev/null; then
    cat >> "$alias_config" << EOF

# SteelSeries GG aliases
alias ssgg="ssgg"
alias ssgg-status="systemctl --user status ssgg"
alias ssgg-restart="systemctl --user restart ssgg"
alias ssgg-gui="~/.local/bin/ssgg-gui"
EOF
    success "Shell aliases created in ~/.bash_aliases"
fi

# Step 10: Final status
echo ""
echo "=============================================="
echo "  Setup Complete!"
echo "=============================================="
echo ""
success "✅ SteelSeries GG daemon is installed and running"
echo ""
echo "📋 Quick Commands:"
echo "   ssgg devices          # List connected devices"
echo "   ssgg rgb color --red 255 --green 0 --blue 0  # Set RGB color"
echo "   systemctl --user status ssgg    # Check daemon status"
echo "   ssgg-gui              # Launch GUI (when display available)"
echo ""
echo "🔒 Security:"
echo "   scripts/security-audit.sh    # Run security checks"
echo ""
echo "📖 Documentation:"
echo "   INSTALL.md                     # Detailed installation guide"
echo "   README.md                      # Project documentation"
echo ""
echo "⚠️  IMPORTANT: Log out and log back in for udev rules to take full effect"
echo ""
