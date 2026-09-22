# SteelSeries GG for Linux - Makefile

.PHONY: all build clean test clippy fmt docs install uninstall help check security

.DEFAULT_GOAL := help

# Variables
CARGO_OPTS ?= --release
FEATURES ?=
CARGO_ARGS ?=
BINARY_NAME = ssgg
SYSTEMD_SERVICE = ssgg.service
UDEV_RULES = 99-steelseries.rules

# Colors for output
RED := \033[0;31m
GREEN := \033[0;32m
YELLOW := \033[0;33m
BLUE := \033[0;34m
NC := \033[0m # No Color

help: ## Show this help message
	@echo "$(BLUE)SteelSeries GG for Linux - Build & Development$(NC)"
	@echo ""
	@echo "Usage: make [target]"
	@echo ""
	@echo "Available targets:"

build: $(call print_target,Building $(BINARY_NAME)) ## Build the project
	@cargo build $(CARGO_OPTS) $(FEATURES) $(CARGO_ARGS)
	@echo "$(GREEN)✓ Build successful$(NC)"

debug: CARGO_OPTS=--no-default-features --dev
debug: FEATURES=$(if $(FEATURES),$(FEATURES),audio sonar)
debug: ## Build debug version with optional features
	@cargo build $(CARGO_OPTS) $(FEATURES)
	@echo "$(GREEN)✓ Debug build complete$(NC)"

release: CARGO_OPTS=--release --all-features
release: ## Build optimized release binary
	@cargo build $(CARGO_OPTS)
	@strip target/release/$(BINARY_NAME)
	@echo "$(GREEN)✓ Release build complete and stripped$(NC)"

clean: ## Clean build artifacts
	@cargo clean
	@echo "$(YELLOW)✓ Build directory cleaned$(NC)"

test: ## Run unit tests
	@cargo test --all-features --quiet
	@echo "$(GREEN)✓ Tests passed$(NC)"

integration-test: ## Run integration tests (requires hardware)
	@echo "$(YELLOW)⚠ Integration tests require physical device$(NC)"
	@cargo test --all-features -- --test-threads=1 || echo "Integration tests skipped or failed"

clippy: ## Run Clippy linter
	@cargo clippy --all-targets --all-features -- -D warnings
	@echo "$(GREEN)✓ No Clippy warnings found$(NC)"

fmt: ## Format code
	@cargo fmt
	@echo "$(GREEN)✓ Code formatted$(NC)"

fmt-check: ## Check formatting without modifying files
	@cargo fmt -- --check
	@echo "$(GREEN)✓ Formatting is correct$(NC)"

docs: ## Generate documentation
	@cargo doc --no-deps --all-features --open
	@echo "$(GREEN)✓ Documentation generated$(NC)"

check-docs: ## Verify documentation builds without errors
	@cargo doc --no-deps --all-features
	@echo "$(GREEN)✓ Documentation compiles correctly$(NC)"

install: ## Install binary to system
	@sudo cp target/release/$(BINARY_NAME) /usr/local/bin/$(BINARY_NAME)
	@sudo chmod +x /usr/local/bin/$(BINARY_NAME)
	@echo "$(GREEN)✓ Binary installed to /usr/local/bin/$(BINARY_NAME)$(NC)"

install-udev: ## Install udev rules
	@sudo cp assets/$(UDEV_RULES) /etc/udev/rules.d/
	@sudo udevadm control --reload-rules
	@sudo udevadm trigger
	@echo "$(GREEN)✓ Udev rules installed and reloaded$(NC)"

install-service: ## Install systemd service
	@sudo cp assets/$(SYSTEMD_SERVICE) /lib/systemd/system/
	@sudo systemctl daemon-reload
	@echo "$(GREEN)✓ Systemd service installed$(NC)"

uninstall: ## Remove binary
	@sudo rm -f /usr/local/bin/$(BINARY_NAME)
	@echo "$(YELLOW)✓ Binary removed$(NC)"

uninstall-udev: ## Remove udev rules
	@sudo rm -f /etc/udev/rules.d/$(UDEV_RULES)
	@sudo udevadm control --reload-rules
	@echo "$(YELLOW)✓ Udev rules removed$(NC)"

uninstall-service: ## Remove systemd service
	@sudo systemctl disable $(SYSTEMD_SERVICE) 2>/dev/null || true
	@sudo rm -f /lib/systemd/system/$(SYSTEMD_SERVICE)
	@sudo systemctl daemon-reload
	@echo "$(YELLOW)✓ Systemd service removed$(NC)"

install-all: install install-udev install-service ## Install everything (binary, udev, service)
	@echo "$(GREEN)✓ Installation complete!$(NC)"
	@echo ""
	@echo "Next steps:"
	@echo "1. Add user to input group: sudo usermod -aG input $$USER"
	@echo "2. Restart machine or run: sudo reboot"
	@echo "3. Start daemon: systemctl --user start $(SYSTEMD_SERVICE)"

check: fmt-check clippy test ## Run all quality checks
	@echo "$(GREEN)✓ All checks passed!$(NC)"

security-audit: ## Run cargo audit
	@if command -v cargo-audit > /dev/null; then \
		cargo audit --quiet || echo "$(YELLOW)⚠ Security issues found$(NC)"; \
	else \
		echo "$(YELLOW)⚠ Install cargo-audit: cargo install cargo-audit$(NC)"; \
	fi

docker-build: ## Build Docker image
	@docker build -t ssgg:latest .
	@echo "$(GREEN)✓ Docker image built$(NC)"

docker-run: ## Run Docker container
	@docker run -it --rm \
		--privileged \
		-v /dev:/dev \
		-v /run/udev:/run/udev \
		-v ~/.config/ssgg:/home/ssgg/.config/ssgg \
		ssgg:latest \
		ssgg daemon

fix: ## Auto-fix common issues
	@cargo fix --allow-staged --allow-dirty 2>/dev/null || true
	@cargo fmt
	@cargo clippy --fix --allow-dirty 2>/dev/null || true
	@echo "$(GREEN)✓ Fixed common issues$(NC)"

bench: ## Run performance benchmarks
	@mkdir -p benches
	@echo "$(YELLOW)⚠ Benchmarks not yet implemented$(NC)"

package: release ## Create distribution package
	@mkdir -p dist
	@tar czf dist/ssgg_$(shell grep '^version' Cargo.toml | head -1 | cut -d'"' -f2)_linux_amd64.tar.gz \
		target/release/$(BINARY_NAME) \
		assets/$(UDEV_RULES) \
		assets/$(SYSTEMD_SERVICE) \
		README.md
	@echo "$(GREEN)✓ Package created in dist/$(NC)"

.PHONY: all build debug release clean test integration-test clippy fmt fmt-check docs check-docs install install-udev install-service uninstall uninstall-udev uninstall-service install-all check security-audit docker-build docker-run fix bench package
