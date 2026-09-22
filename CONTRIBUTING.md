# Contributing to SteelSeries GG for Linux

Thank you for your interest in contributing! This guide will help you get started.

## 🌟 How to Contribute

### Where to Start

1. **Good First Issues** - Look for issues labeled "good first issue" on GitHub
2. **Documentation** - Improve README, architecture docs, or code comments
3. **Testing** - Run integration tests with actual hardware
4. **Code Quality** - Fix clippy warnings, improve formatting

### Areas of Focus

- **Hardware Support**: Add new device PIDs and protocol implementations
- **RGB Effects**: Implement new lighting animations
- **UI/UX**: Build GTK4/GTK3 graphical interface
- **Audio Features**: Enhance Sonar and audio mixer support
- **Performance**: Optimize latency-sensitive operations
- **Security**: Audit HID access patterns and network binding

## 🏗️ Development Workflow

### 1. Fork and Clone

```bash
git clone https://github.com/YOUR_USERNAME/SteelSeries-GG-for-linux.git
cd SteelSeries-GG-for-linux
git remote add upstream https://github.com/MikhaelCat/SteelSeries-GG-for-linux.git
```

### 2. Setup Development Environment

```bash
# Install development dependencies
sudo apt-get install \
    libhidapi-dev \
    pkg-config \
    build-essential \
    libssl-dev \
    libpulse-dev \
    lld

# Verify setup
cargo --version
rustc --version
```

### 3. Create Feature Branch

```bash
git checkout -b feature/description-of-feature
```

### 4. Make Changes

Follow these guidelines:

- **Code Style**: Run `cargo fmt` before committing
- **Clippy**: No warnings allowed (`cargo clippy`)
- **Tests**: Add tests for new functionality
- **Documentation**: Update docs when adding features

### 5. Run Pre-commit Checks

```bash
# Option 1: Use pre-commit hook (automated)
git commit

# Option 2: Manual checks
make check
```

### 6. Push and Create Pull Request

```bash
git push origin feature/description-of-feature

# Then create PR on GitHub
```

## 📝 Code Standards

### Rust Coding Standards

```rust
// Follow rustfmt defaults (4 spaces, 100 char line length)
// Document all public APIs
/// This is documented how the function works
pub fn example_function() -> Result<(), Error> {
    // Implementation...
}

// Use descriptive names
let mouse_sensor_data = read_sensor_data(); // ✅
let data = read(); // ❌ Too vague

// Handle errors properly
if let Some(value) = maybe_value {
    Ok(value)
} else {
    Err(Error::NotFound)
}
```

### Documentation Requirements

- All public functions must have doc comments
- New effects/features need examples
- Protocol specifications should include references

### Commit Messages

```bash
# Format: type(scope): description

feat(keyboard): add per-key RGB actuation control
fix(mouse): correct DPI stage switching logic
docs(readme): update installation instructions
test(rgB): add spectrum effect unit tests
refactor(protocol): extract common HID command builder
```

## 🔧 Technical Guidelines

### Adding Device Support

1. Find device VID/PID:
   ```bash
   lsusb | grep -i steelseries
   # Output example: Bus 001 Device 005: ID 1246:XXXX SteelSeries XXX
   ```

2. Add PID to `src/device.rs`:
   ```rust
   fn is_steelseries_device(pid: u16) -> bool {
       matches!(pid,
           // ... existing PIDs ...
           0xXXXX, // Your new device
       )
   }
   ```

3. Test enumeration:
   ```bash
   ./target/release/ssgg devices --detailed
   ```

### Implementing New RGB Effect

```rust
// In src/effects.rs
pub struct MyCoolEffect {
    config: EffectConfig,
    hue_offset: u16,
}

impl LightingEffect for MyCoolEffect {
    fn name(&self) -> &str {
        "MyCoolEffect"
    }
    
    fn get_zone_color(&self, ctx: &EffectContext) -> RgbColor {
        // Calculate color based on zone and time
    }
    
    fn update(&mut self, delta_ms: u64) {
        // Update state between frames
    }
}
```

### Handling HID Communications

```rust
// Always handle timeouts
let mut report = [0u8; 64];
let bytes_read = hid.read_timeout(&mut report, 10)?;

// Validate packet structure
validate_packet_header(&report)?;

// Send safely
send_command(device, &command_buffer)?;
```

## 🧪 Testing

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_rgb_conversion() {
        let rgb = RgbColor::from_hex("#FF0000").unwrap();
        assert_eq!(rgb.red, 255);
        assert_eq!(rgb.green, 0);
        assert_eq!(rgb.blue, 0);
    }
}
```

### Integration Tests (Requires Hardware)

```bash
# Run with physical device attached
cargo test -- --test-threads=1 --features audio
```

### Performance Benchmarks

Create benchmarks in `benches/` directory:

```rust
#[bench]
fn bench_rgb_effect(b: &mut Bencher) {
    b.iter(|| {
        // Benchmark RGB calculation
    })
}
```

## 🐛 Reporting Bugs

When reporting bugs, include:

1. **Environment**: OS version, kernel version, Rust version
2. **Device Info**: VID/PID, model number, firmware version
3. **Steps to Reproduce**: Detailed reproduction steps
4. **Expected Behavior**: What should happen
5. **Actual Behavior**: What actually happened
6. **Logs**: Debug output (`RUST_LOG=debug`)

Example:

```markdown
**Bug Report: Keyboard not detected**

- **OS**: Ubuntu 22.04 LTS
- **Kernel**: 5.19.0-generic
- **Rust**: 1.71.0
- **Device**: SteelSeries Apex Pro (VID: 0x1246, PID: 0x0503)

**Steps:**
1. Connect keyboard via USB
2. Run `ssgg devices`
3. No devices shown

**Expected:** Device appears in list
**Actual:** "No SteelSeries devices detected."

**Debug Log:**
```
$ RUST_LOG=debug ssgg devices
ERROR Failed to enumerate devices: Permission denied
```
```

## 💡 Feature Requests

For new features:

1. Check if similar feature already exists
2. Search existing issues to avoid duplicates
3. Provide detailed use case and implementation ideas
4. Consider performance implications

Good feature requests include:

```markdown
**Feature: Visual Keyboard Hotspot Overlay**

**Problem**: Users can't see which keys are being pressed during macros.

**Proposed Solution**: Draw semi-transparent key highlight overlay during macro playback.

**Implementation Ideas**:
- Use X11/Wayland window layering
- Store key press timing from HID events
- Render glow effect around active keys
- Configurable opacity and duration

**Benefits**: Better macro debugging, improved UX for visual learners
```

## 🔄 Review Process

1. CI pipeline runs automatically on push/PR
2. Maintainer reviews code quality
3. Discuss any concerns or suggestions
4. Address review feedback
5. Merge when approved (2 approvals minimum)

### Code Review Checklist

- [ ] Code follows project style
- [ ] No compiler warnings
- [ ] Clippy passes without warnings
- [ ] Tests added for new functionality
- [ ] Documentation updated
- [ ] Security considerations addressed
- [ ] Backwards compatibility maintained

## 🎯 Current Priorities

Help us focus on what matters most right now:

- **High Priority** 🔴
  - GTK4 GUI implementation
  - Complete protocol documentation
  - Performance optimization for RGB effects
  
- **Medium Priority** 🟡
  - Additional device testing
  - Wayland overlay support
  - Mobile companion app design
  
- **Nice to Have** 🟢
  - Alternative UI themes
  - Extra RGB effects
  - Translation/localization

## 🤝 Community Guidelines

- Be respectful and inclusive
- Help newcomers learn
- Share knowledge in PRs and issues
- Keep discussions focused on technical merit

## 📜 License

By contributing, you agree that your contributions will be licensed under the MIT License.

---

Questions? Join our [Discord](https://discord.gg/example) or open a discussion on GitHub.
