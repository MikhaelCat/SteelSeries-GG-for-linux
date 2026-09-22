#!/bin/bash
# Comprehensive Security Audit Script for SteelSeries GG
# Performs automated security checks on dependencies, code, and configuration

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
LOG_FILE="/tmp/ssgg-security-audit-$(date +%Y%m%d-%H%M%S).log"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
BLUE='\033[0;34m'
NC='\033[0m'

log() { echo -e "${BLUE}[$(date '+%H:%M:%S')]${NC} $*" | tee -a "$LOG_FILE"; }
success() { echo -e "${GREEN}[✓]${NC} $*"; }
warn() { echo -e "${YELLOW}[!]${NC} $*"; }
error() { echo -e "${RED}[✗]${NC} $*" >&2; }

audit_count=0
passed=0
failed=0
warnings=0

check() {
    ((audit_count++))
    log "Checking: $1..."
}

pass() {
    ((passed++))
    success "$1"
}

fail() {
    ((failed++))
    error "$1"
}

warning() {
    ((warnings++))
    warn "$1"
}

echo "=========================================="
echo "  SteelSeries GG - Security Audit Report"
echo "=========================================="
echo ""

# Check 1: Cargo Audit
check "Running cargo-audit for vulnerability scanning..."
if command -v cargo-audit &> /dev/null; then
    if cargo audit --quiet 2>&1; then
        pass "No known vulnerabilities found in dependencies"
    else
        fail "Vulnerabilities detected in dependencies!"
        cargo audit || true
    fi
else
    warning "cargo-audit not installed (install with: cargo install cargo-audit)"
fi

# Check 2: Clippy Warnings
check "Running cargo clippy for code quality issues..."
if cargo clippy --all-targets --all-features -- -D warnings 2>&1 | grep -q "warning:"; then
    warn "Clippy warnings detected (review output above)"
else
    pass "No clippy warnings found"
fi

# Check 3: Dependency Updates
check "Checking for outdated dependencies..."
if command -v cargo-outdated &> /dev/null; then
    outdated=$(cargo outdated --root-deps-only 2>/dev/null | wc -l)
    if [ "$outdated" -gt 0 ]; then
        warn "$outdated outdated dependencies found"
    else
        pass "All dependencies are up-to-date"
    fi
else
    warning "cargo-outdated not installed (use: cargo update)"
fi

# Check 4: Binary Size Optimization
check "Analyzing binary size..."
BINARY_SIZE=$(stat -c%s target/release/ssgg 2>/dev/null || echo "0")
if [ "$BINARY_SIZE" -gt 50000000 ]; then  # 50MB threshold
    fail "Binary size too large: $BINARY_SIZE bytes"
else
    pass "Binary size optimal: $BINARY_SIZE bytes"
fi

# Check 5: Secret Scanning
check "Scanning for hardcoded secrets..."
SECRETS=$(grep -rE "(password|secret|api_key|token)\s*=\s*['\"]?[A-Za-z0-9]{8,}" src/ 2>/dev/null | wc -l)
if [ "$SECRETS" -gt 0 ]; then
    fail "Potential hardcoded secrets found!"
    grep -rE "(password|secret|api_key|token)\s*=\s*['\"]?[A-Za-z0-9]{8,}" src/ || true
else
    pass "No hardcoded secrets detected"
fi

# Check 6: File Permissions
check "Verifying file permissions..."
PERMISSIONS_ISSUES=0

# Check executable bit on scripts
for script in setup-deps.sh build.sh scripts/*.sh; do
    if [ -f "$script" ] && [ ! -x "$script" ]; then
        ((PERMISSIONS_ISSUES++))
    fi
done

# Check sensitive files
for secret_file in assets/99-steelseries.rules assets/ssgg.service; do
    if [ -f "$secret_file" ]; then
        perms=$(stat -c%a "$secret_file")
        if [ "$perms" != "644" ] && [ "$perms" != "600" ]; then
            ((PERMISSIONS_ISSUES++))
        fi
    fi
done

if [ "$PERMISSIONS_ISSUES" -eq 0 ]; then
    pass "All file permissions correct"
else
    fail "$PERMISSIONS_ISSUES permission issues found"
fi

# Check 7: Systemd Service Hardening
check "Analyzing systemd service hardening..."
if grep -q "ProtectSystem=strict" assets/ssgg.service; then
    pass "Systemd service has full hardening enabled"
else
    warning "Consider adding ProtectSystem=strict to systemd service"
fi

if grep -q "NoNewPrivileges=true" assets/ssgg.service; then
    pass "NoNewPrivileges protection enabled"
else
    warning "Add NoNewPrivileges=true to systemd service"
fi

# Check 8: Memory Sanitizer Check
check "Checking for memory safety practices..."
if grep -r "unsafe" src/ 2>/dev/null | grep -v "// unsafe" | wc -l | grep -q "[1-9]"; then
    warning "Unsafe code blocks found (review carefully)"
    grep -r "unsafe" src/ | head -5
else
    pass "No unsafe code blocks detected"
fi

# Summary
echo ""
echo "=========================================="
echo "  Security Audit Summary"
echo "=========================================="
echo -e "Passed:   ${GREEN}$passed${NC}"
echo -e "Failed:   ${RED}$failed${NC}"
echo -e "Warnings: ${YELLOW}$warnings${NC}"
echo -e "Total:    $audit_count tests"
echo ""

if [ "$failed" -eq 0 ]; then
    echo -e "${GREEN}✅ All critical security checks passed!${NC}"
    exit 0
else
    echo -e "${RED}❌ $failed security issue(s) require attention!${NC}"
    echo "See logs at: $LOG_FILE"
    exit 1
fi
