#!/bin/bash
# SteelSeries GG - GUI Launcher
# Automatically connects to systemd daemon with display detection

set -euo pipefail

SCRIPT_NAME="SteelSeries GG"
DAEMON_CHECK_DELAY=2000  # Delay before checking daemon (ms)

# Get environment variables
export DISPLAY=${DISPLAY:-":0"}
XAUTHORITY=${XAUTHORITY:-"$HOME/.Xauthority"}

# Function to show notification
show_notification() {
    local title="$1"
    local message="$2"
    local icon="${3:-'dialog-information'}"
    
    # Try different notification methods
    if command -v notify-send &> /dev/null; then
        notify-send -u normal -i "$icon" "$title" "$message"
    elif command -v dbus-send &> /dev/null && [ -n "$DBUS_SESSION_BUS_ADDRESS" ]; then
        dbus-send --print-reply --dest=org.freedesktop.Notifications \
            /org/freedesktop/Notifications \
            org.freedesktop.notifications.Notify \
            string:"$SCRIPT_NAME" \
            uint32:0 \
            string:"$icon" \
            string:"$title" \
            string:"$message" \
            array:string: \
            struct:int32:-1
    fi
}

# Function to check if daemon is running
check_daemon() {
    if command -v systemctl &> /dev/null; then
        if systemctl --user is-active --quiet ssgg 2>&1; then
            return 0
        fi
    fi
    
    # Fallback: check if process is running
    if pgrep -x "ssgg" > /dev/null 2>&1; then
        return 0
    fi
    
    return 1
}

# Function to check if GUI binary exists
gui_binary_exists() {
    # Check in PATH
    if command -v ssgg-gui-local &> /dev/null; then
        return 0
    fi
    
    # Check in project target directory
    local project_dir="$(dirname "$(dirname "$(readlink -f "$BASH_SOURCE")")")"
    if [ -f "$project_dir/target/release/ssgg-gui" ]; then
        return 0
    fi
    
    return 1
}

echo "=============================================="
echo "  $SCRIPT_NAME - GUI Launcher"
echo "=============================================="
echo ""

# Wait for daemon to be ready
info "Checking daemon status..."
sleep $(($DAEMON_CHECK_DELAY / 1000))

if ! check_daemon; then
    error "❌ Daemon not running!"
    echo ""
    info "Starting daemon..."
    
    if command -v systemctl &> /dev/null; then
        systemctl --user start ssgg 2>&1 || {
            error "Failed to start daemon. Please run:"
            echo "  sudo ./scripts/auto-setup.sh"
            exit 1
        }
        sleep 2
    else
        # Try direct execution
        local project_dir="$(dirname "$(dirname "$(readlink -f "$BASH_SOURCE")")")"
        if [ -f "$project_dir/target/release/ssgg" ]; then
            cd "$project_dir"
            target/release/ssgg daemon &
            sleep 3
        else
            error "Binary not found. Run auto-setup.sh first"
            exit 1
        fi
    fi
    
    success "✅ Daemon started"
fi

# Display system information
echo ""
info "System Information:"
echo "  Display: $DISPLAY"
echo "  XAuthority: ${XAUTHORITY:-not set}"

# List detected devices
echo ""
info "Connected SteelSeries Devices:"
if command -v ssgg &> /dev/null; then
    ssgg devices 2>/dev/null | head -20 || echo "  Unable to detect devices"
else
    echo "  CLI tool not installed yet"
fi

# Launch GUI (this would typically be a GTK4 application)
echo ""
success "🎯 Ready to launch GUI interface"
echo ""
echo "Options:"
echo "  1. Use the applications menu and search 'SteelSeries GG'"
echo "  2. Run 'ssgg-gui' from terminal"
echo "  3. Control RGB from command line:"
echo "     ssgg rgb color --red 255 --green 0 --blue 0"
echo ""

# Show desktop notification
show_notification \
    "SteelSeries GG Ready" \
    "Daemon is running. You can now control your SteelSeries devices." \
    "preferences-system"

echo ""
success "✨ $SCRIPT_NAME initialization complete!"
echo ""
info "Press Ctrl+C to exit (daemon will continue running)"

# Keep process alive briefly to show notification, then exit gracefully
sleep 5
