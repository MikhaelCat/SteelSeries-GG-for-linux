# Pull Request Guidelines

Thank you for contributing to SteelSeries GG for Linux! Please follow these guidelines:

## Development Workflow

### 1. Before You Start

```bash
# Ensure you have the right Rust version
rustup show

# Install development dependencies
sudo apt install libhidapi-dev pkg-config build-essential libssl-dev  # Debian/Ubuntu
sudo dnf install rust cargo hidapi-devel pkgconf-pkg-config gcc       # Fedora
sudo pacman -S rust hidapi base-devel                                  # Arch

# Clone and setup
git clone https://github.com/MikhaelCat/SteelSeries-GG-for-linux.git
cd SteelSeries-GG-for-linux
cargo fetch
```

### 2. Branch Naming

Use conventional naming:
- `feature/add-apex-pro-tkl-support`
- `fix/rgb-effect-memory-leak`
- `docs/update-installation-guide`
- `refactor/device-detection-layer`

### 3. Making Changes

**Code Style:**
```bash
# Format code
cargo fmt

# Check clippy (must be clean)
cargo clippy --all-targets --all-features -- -D warnings
```

**Testing:**
```bash
# Run tests
cargo test --all-features

# Integration testing with real hardware
cargo test -- --test-threads=1

# Security audit
./scripts/security-audit.sh
```

**Build Verification:**
```bash
# Debug build
cargo build --all-features

# Release build
cargo build --release --all-features

# Verify binary size
stat -c%s target/release/ssgg
```

### 4. Commit Messages

Follow [Conventional Commits](https://www.conventionalcommits.org/):

```
type(scope): description

[optional body]

[optional footer/breaking change note]
```

Valid types:
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation only
- `style`: Formatting, missing semi-colons
- `refactor`: Code restructure
- `test`: Test additions/fixes
- `chore`: Build/process changes

Examples:
```
feat(keyboard): add Apex Pro TKL 2023 RGB control

Implemented full RGB zone configuration for Apex Pro TKL 2023
Supports all standard effects and custom per-key lighting patterns.

BREAKING CHANGE: Requires udev rules update after upgrade
```

### 5. Documentation

Update these if relevant:
- [ ] README.md (user-facing features)
- [ ] INSTALL.md (installation changes)
- [ ] ARCHITECTURE.md (technical architecture)
- [ ] CHANGELOG.md (user-visible changes)
- [ ] Source code doc comments (public APIs)

### 6. Checklist

Before submitting PR:
- [ ] Code compiles without warnings (`cargo build`)
- [ ] All tests pass (`cargo test`)
- [ ] Clippy is clean (`cargo clippy`)
- [ ] Code formatted (`cargo fmt --check`)
- [ ] Documentation updated
- [ ] No new memory leaks (Valgrind/ASan)
- [ ] Performance impact assessed
- [ ] Hardware compatibility verified

### 7. Review Process

All PRs require:
- ✅ Minimum one reviewer approval
- ✅ CI pipeline passes (build + tests)
- ✅ Conventional commits followed
- ✅ Documentation completeness checked

Review timeframes:
- Bug fixes: 24 hours
- Features: 72 hours
- Documentation: 48 hours

## Code Quality Standards

### Memory Safety
- Zero unsafe blocks unless absolutely necessary
- Use `Option`/`Result` for error handling
- Avoid raw pointers in public APIs
- Document any FFI interactions

### Performance
- Keep daemon CPU usage <5% idle
- Memory usage <64MB baseline
- RGB latency <1ms target
- Startup time <2 seconds

### Compatibility
- Test on at least 2 distributions
- Support MSRV (Minimum Supported Rust Version)
- Handle both USB and wireless receivers
- Graceful degradation if optional features unavailable

## Examples of Good PRs

Look at recent merges for reference:
- Clean, focused scope
- Comprehensive testing
- Clear documentation
- Professional commit messages

---

For questions, open a Discussion or email: dev@steelseries-linux.dev
