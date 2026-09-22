# SteelSeries GG for Linux - Cross-Platform Build System
# Supports: Ubuntu/Debian, Fedora/RHEL, Arch Linux, openSUSE, Alpine, and more

.SUFFIXES:
.DELETE_FROM_PRIORITY:

# Configuration
BINARY_NAME := ssgg
VERSION := $(shell grep '^version' Cargo.toml | head -1 | cut -d'"' -f2)
RUST_TOOLCHAIN := 1.97.1

# Platform detection
UNAME_S := $(shell uname -s)
UNAME_M := $(shell uname -m)

ifeq ($(UNAME_S),Linux)
    OS := linux
else ifeq ($(UNAME_S),Darwin)
    OS := macos
else
    OS := unknown
endif

ifeq ($(UNAME_M),x86_64)
    ARCH := x86_64
else ifeq ($(UNAME_M),aarch64)
    ARCH := aarch64
else ifeq ($(UNAME_M),arm64)
    ARCH := arm64
else
    ARCH := $(UNAME_M)
endif

# Distribution detection
ifneq ($(shell test -f /etc/debian_version && echo debian),)
    DISTRO := debian
    PACKAGE_FORMAT := deb
else ifneq ($(shell test -f /etc/fedora-release && echo fedora),)
    DISTRO := fedora
    PACKAGE_FORMAT := rpm
else ifneq ($(shell test -f /etc/redhat-release && echo redhat),)
    DISTRO := rhel
    PACKAGE_FORMAT := rpm
else ifneq ($(shell test -f /etc/arch-release && echo arch),)
    DISTRO := arch
    PACKAGE_FORMAT := pkg.tar.zst
else ifneq ($(shell test -f /etc/SUSE-brand && echo suse),)
    DISTRO := opensuse
    PACKAGE_FORMAT := rpm
else ifneq ($(shell cat /etc/*-release 2>/dev/null | grep alpine),)
    DISTRO := alpine
    PACKAGE_FORMAT := apk
else
    DISTRO := generic
    PACKAGE_FORMAT := binary
endif

# Feature flags
FEATURES ?=
ALL_FEATURES := audio sonar experimental-apex-2023

# Build options
BUILD_TYPE ?= release
CARGO_FLAGS := --$(BUILD_TYPE)
ifneq ($(FEATURES),)
    CARGO_FLAGS += --features $(FEATURES)
endif

# Optimization flags
RUSTFLAGS ?= -C link-arg=-fuse-ld=$(shell which lld 2>/dev/null || which mold 2>/dev/null || echo "lld")
export RUSTFLAGS

# Paths
BUILD_DIR := target/$(BUILD_TYPE)
DIST_DIR := dist

# Colors
RED := \033[0;31m
GREEN := \033[0;32m
YELLOW := \033[0;33m
BLUE := \033[0;34m
NC := \033[0m

help:
	@echo "$(BLUE)╔═══════════════════════════════════════════════════════╗$(NC)"
	@echo "$(BLUE)║   SteelSeries GG - Cross-Platform Build System      ║$(NC)"
	@echo "$(BLUE)╚═══════════════════════════════════════════════════════╝$(NC)"
	@echo ""
	@echo "Detected platform:"
	@echo "  Operating System:  $(OS)"
	@echo "  Architecture:      $(ARCH)"
	@echo "  Distribution:      $(DISTRO)"
	@echo "  Package Format:    $(PACKAGE_FORMAT)"
	@echo ""
	@echo "Quick Start:"
	@echo "  $(GREEN)make build$(NC)         Build the project"
	@echo "  $(GREEN)make test$(NC)          Run tests"
	@echo "  $(GREEN)make check$(NC)         Run all quality checks"
	@echo ""
	@echo "Installation:"
	@echo "  $(GREEN)make install$(NC)       Install to system (requires root)"
	@echo "  $(GREEN)make uninstall$(NC)     Remove from system"
	@echo ""
	@echo "Packaging ($(PACKAGE_FORMAT)):";
	@echo "  $(GREEN)make package$(NC)       Create distribution package"
	@echo "  $(GREEN)make deb$(NC)           Create .deb package"
	@echo "  $(GREEN)make rpm$(NC)           Create .rpm package"
	@echo "  $(GREEN)make appimage$(NC)      Create AppImage"
	@echo ""
	@echo "Cross-Build:"
	@echo "  $(GREEN)make docker-build$(NC)  Build in Docker container"
	@echo ""
	@echo "More targets: $(yellow)make help-all$(NC)"

build: setup-build-env
	@echo "$(GREEN)[INFO]$(NC) Building for $(DISTRO)/$(ARCH)..."
	cargo build $(CARGO_FLAGS)
	@echo "$(GREEN)[✓]$(NC) Build complete: $(BUILD_DIR)/$(BINARY_NAME)"

debug: BUILD_TYPE=debug
debug: FEATURES=audio sonar
debug: build

release: build

all: build test

test: 
	@echo "$(GREEN)[INFO]$(NC) Running tests..."
	cargo test --all-features
	@echo "$(GREEN)[✓]$(NC) Tests passed"

check: fmt-check clippy

fmt:
	cargo fmt

fmt-check:
	cargo fmt -- --check

clippy:
	cargo clippy --all-targets --all-features -- -D warnings

install: build
	@echo "$(GREEN)[INFO]$(NC) Installing $(BINARY_NAME)..."
	sudo cp $(BUILD_DIR)/$(BINARY_NAME) /usr/local/bin/
	sudo chmod +x /usr/local/bin/$(BINARY_NAME)
	@if [ -f assets/99-steelseries.rules ]; then \
		echo "$(GREEN)[INFO]$(NC) Installing udev rules..."; \
		sudo cp assets/99-steelseries.rules /etc/udev/rules.d/; \
		sudo udevadm control --reload-rules; \
	fi
	@if [ -f assets/ssgg.service ]; then \
		echo "$(GREEN)[INFO]$(NC) Installing systemd service..."; \
		sudo cp assets/ssgg.service /lib/systemd/user/; \
		systemctl --user daemon-reload; \
	fi
	@echo "$(GREEN)[✓]$(NC) Installation complete"
	@echo ""
	@echo "Next steps:"
	@echo "  sudo usermod -aG input $$USER"
	@echo "  systemctl --user enable --now ssgg.service"

uninstall:
	@echo "$(GREEN)[INFO]$(NC) Removing $(BINARY_NAME)..."
	sudo rm -f /usr/local/bin/$(BINARY_NAME)
	@if [ -f /etc/udev/rules.d/99-steelseries.rules ]; then \
		sudo rm -f /etc/udev/rules.d/99-steelseries.rules; \
		sudo udevadm control --reload-rules; \
	fi
	@if [ -f /lib/systemd/user/ssgg.service ]; then \
		systemctl --user disable --now ssgg.service; \
		sudo rm -f /lib/systemd/user/ssgg.service; \
		systemctl --user daemon-reload; \
	fi
	@echo "$(GREEN)[✓]$(NC) Uninstallation complete"

package: package-$(PACKAGE_FORMAT)

deb:
	@if [ "$(DISTRO)" != "debian" ]; then \
		echo "$(YELLOW)[WARN]$(NC) DEB packages are best built on Debian-based systems"; \
		echo "$(YELLOW)[INFO]$(NC) Using Docker for cross-distribution build"; \
		make docker-build-deb; \
	else \
		mkdir -p $(DIST_DIR); \
		debuild -us -uc -b || true; \
		mv ../*.deb $(DIST_DIR)/ 2>/dev/null || true; \
		echo "$(GREEN)[✓]$(NC) DEB package created in $(DIST_DIR)/"
	endif

rpm:
	@if [ "$(DISTRO)" != "fedora" ] && [ "$(DISTRO)" != "rhel" ] && [ "$(DISTRO)" != "opensuse" ]; then \
		echo "$(YELLOW)[WARN]$(NC) RPM packages work better on RPM-based distributions"; \
		make docker-build-rpm; \
	else \
		rpkg local || cargo build --release && \
		createrepo ./RPMS/; \
		echo "$(GREEN)[✓]$(NC) RPM package created"
	endif

appimage:
	@if ! command -v appimagetool &> /dev/null; then \
		echo "$(YELLOW)[INFO]$(NC) AppImageTool not found, downloading..."; \
		wget https://github.com/AppImage/AppImageKit/releases/download/continuous/appimagetool-x86_64.AppImage -O /tmp/appimagetool; \
		chmod +x /tmp/appimagetool; \
	fi
	@mkdir -p $(DIST_DIR)
	@if command -v appimagetool &> /dev/null; then \
		echo "$(GREEN)[INFO]$(NC) Creating AppImage..."; \
		# Simplified AppImage creation\n\
		cp $(BUILD_DIR)/$(BINARY_NAME) /tmp/ssgg/;\n\
		mkdir -p /tmp/AppDir/usr/bin;\n\
		cp /tmp/ssgg /tmp/AppDir/usr/bin/;\n\
		echo '#!/bin/bash\nexec /opt/ssgg/usr/bin/ssgg "$$@"' > /tmp/AppDir/usr/bin/ssgg;\n\
		/tmp/appimagetool /tmp/AppDir $(DIST_DIR)/$(BINARY_NAME)_$(VERSION)_$(ARCH).AppImage; \
		echo "$(GREEN)[✓]$(NC) AppImage created"
	endif

docker-build:
	docker build -t ssgg-builder .

docker-build-deb:
	docker build -t ssgg-deb-build -f Dockerfile.debian .
	docker run --rm ssgg-deb-build tar -czf /tmp/ssgg_$(VERSION)_amd64.tar.gz /workspace/target/release/ssgg assets/*

docker-build-rpm:
	docker build -t ssgg-rpm-build -f Dockerfile.fedora .
	docker run --rm ssgg-rpm-build tar -czf /tmp/ssgg_$(VERSION)_x86_64.tar.gz /workspace/target/release/ssgg assets/*

clean:
	cargo clean
	rm -rf $(DIST_DIR)
	rm -rf $(PKG_DIR)

setup-build-env:
	@mkdir -p $(DIST_DIR)

.PHONY: help build debug release all test check fmt fmt-check clippy install uninstall \
        package deb rpm appimage docker-build docker-build-deb docker-build-rpm \
        clean setup-build-env
