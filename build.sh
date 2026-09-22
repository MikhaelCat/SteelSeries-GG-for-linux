#!/bin/bash
# Build script for ssgg (SteelSeries GG for Linux)

set -e

echo "=========================================="
echo "Building SteelSeries GG for Linux"
echo "=========================================="

# Detect available optimization tools
if command -v cargo-install &> /dev/null; then
    if ! cargo install sccache --list 2>/dev/null | grep -q sccache; then
        echo "Installing sccache for faster builds..."
        # Uncomment when ready: cargo install sccache
        # cargo install sccache
        echo "sccache installation commented out - add cargo install sccache line in build.rs"
    fi
fi

if command -v lld &> /dev/null || command -v clang &> /dev/null; then
    echo "Using system linker (lld/clang)"
    export RUSTFLAGS="-C link-arg=-fuse-ld=lld"
fi

# Parse arguments
FEATURES=""
PROFILE="release"

for arg in "$@"; do
    case $arg in
        debug|dev)
            PROFILE="debug"
            shift
            ;;
        --features)
            FEATURES="$2"
            shift 2
            ;;
        --all-features)
            FEATURES="--all-features"
            shift
            ;;
        *)
            ;;
    esac
done

echo "Profile: $PROFILE"
if [ -n "$FEATURES" ]; then
    echo "Features: $FEATURES"
fi

# Set Cargo flags based on feature configuration
CARGO_FLAGS=""
if [ -n "$FEATURES" ] && [ "$FEATURES" != "--all-features" ]; then
    CARGO_FLAGS="--features $FEATURES"
elif [ "$FEATURES" == "--all-features" ]; then
    CARGO_FLAGS="--all-features"
fi

echo ""
echo "Building..."
cargo build --$PROFILE $CARGO_FLAGS

echo ""
echo "Build successful!"
echo "Binary location: target/$PROFILE/ssgg"
echo ""
echo "To run:"
echo "  ./target/$PROFILE/ssgg devices          # List devices"
echo "  sudo cp assets/99-steelseries.rules /etc/udev/rules.d/"
echo "  sudo udevadm control --reload-rules"
echo "  sudo usermod -aG input \$USER"
echo ""
