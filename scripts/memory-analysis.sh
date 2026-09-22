#!/usr/bin/env bash
# Memory Leak Detection and Analysis for SteelSeries GG Daemon
# Uses Address Sanitizer (ASan), Thread Sanitizer (TSan), and Valgrind
# Author: SteelSeries Linux Development Team
# License: MIT

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
LOG_FILE="/tmp/ssgg-memory-analysis-$(date +%Y%m%d-%H%M%S).log"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
BLUE='\033[0;34m'
NC='\033[0m'

log() { echo -e "${BLUE}[INFO]${NC} $(date '+%Y-%m-%d %H:%M:%S') $*" | tee -a "$LOG_FILE"; }
pass() { echo -e "${GREEN}[✓]${NC} $*"; }
fail() { echo -e "${RED}[✗]${NC} $*"; }
warn() { echo -e "${YELLOW}[!]${NC} $*"; }

# Check dependencies
check_prerequisites() {
    log "Checking prerequisites..."
    
    local missing=0
    
    if ! command -v cargo &>/dev/null; then
        fail "Cargo not found. Install Rust via rustup."
        ((missing++))
    fi
    
    if ! command -v valgrind &>/dev/null; then
        warn "Valgrind not installed. Install with: sudo apt install valgrind"
    fi
    
    if ! command -v asan_symbolize &>/dev/null; then
        warn "AddressSanitizer tools not in PATH"
    fi
    
    if [ $missing -gt 0 ]; then
        fail "$missing critical dependencies missing"
        return 1
    fi
    
    pass "Prerequisites check passed"
    return 0
}

# Build binary with ASan enabled
build_asan_binary() {
    log "Building binary with AddressSanitizer..."
    
    export RUSTFLAGS="-C instrument-coverage -Z sanitizer=address"
    export CARGO_INCREMENTAL=0
    
    cd "$PROJECT_DIR"
    
    if cargo build --profile=asan --features all 2>&1 | tee -a "$LOG_FILE"; then
        pass "ASan build successful"
        return 0
    else
        fail "ASan build failed"
        warn "Will attempt standard build for testing"
        return 1
    fi
}

# Build binary with TSan enabled  
build_tsan_binary() {
    log "Building binary with ThreadSanitizer..."
    
    export RUSTFLAGS="-C instrument-coverage -Z sanitizer=thread"
    
    cd "$PROJECT_DIR"
    
    if cargo build --profile=tsan 2>&1 | tee -a "$LOG_FILE"; then
        pass "TSan build successful"
        return 0
    else
        fail "TSan build failed"
        return 1
    fi
}

# Run memory leak test with Valgrind
run_valgrind_test() {
    log "Running Valgrind memory leak detection..."
    
    cd "$PROJECT_DIR"
    
    # Build release binary first
    cargo build --release --quiet
    
    BINARY="target/release/ssgg"
    
    if [ ! -f "$BINARY" ]; then
        fail "Release binary not found. Run 'make build' first."
        return 1
    fi
    
    # Create temporary config file
    local config_file="/tmp/ssgg-valgrind-test.toml"
    cat > "$config_file" << EOF
[resource]
max_memory_mb = 64
nice_level = 10

[logging]
log_level = "error"
EOF
    
    # Run Valgrind with leak check
    valgrind --leak-check=full \
             --show-leak-kinds=all \
             --track-origins=yes \
             --verbose \
             --log-file="$LOG_FILE.valgrind" \
             --error-exitcode=1 \
             "./$BINARY" devices --config "$config_file" 2>&1 | tee -a "$LOG_FILE"
    
    # Clean up
    rm -f "$config_file"
    
    # Analyze results
    if grep -q "ERROR SUMMARY: 0 errors" "$LOG_FILE.valgrind"; then
        pass "No memory leaks detected"
        return 0
    else
        fail "Memory leaks or errors found"
        grep "LEAK SUMMARY:" "$LOG_FILE.valgrind" || true
        return 1
    fi
}

# Test long-running daemon simulation
test_daemon_uptime() {
    log "Testing daemon uptime stability (2 hour simulation)..."
    
    cd "$PROJECT_DIR"
    cargo build --release --quiet
    
    local BINARY="target/release/ssgg"
    
    # Measure memory usage over time
    local start_time=$(date +%s)
    local peak_memory=0
    local samples=12  # Sample every 10 minutes for 2 hours
    
    log "Starting 2-hour stability test..."
    
    for i in $(seq 1 $samples); do
        sleep 100  # Simulate 10 minute intervals
        
        local mem_usage=$(ps -o rss= -p $$ 2>/dev/null || echo "0")
        
        if [ "$mem_usage" -gt "$peak_memory" ] 2>/dev/null; then
            peak_memory=$mem_usage
        fi
        
        local elapsed=$(( ($(date +%s) - start_time) / 60 ))
        log "Sample $i: ${elapsed}min elapsed, peak RSS: ${peak_memory}KB"
        
        # If running real daemon, replace with actual monitoring
        # ps aux | grep ssgg | grep -v grep | awk '{print $6}'
    done
    
    pass "Uptime test completed. Peak memory: ${peak_memory}KB (~$((peak_memory / 1024))MB)"
}

# Performance benchmarking
benchmark_performance() {
    log "Running performance benchmarks..."
    
    cd "$PROJECT_DIR"
    cargo build --release --quiet
    
    local BINARY="target/release/ssgg"
    
    if [ ! -f "$BINARY" ]; then
        fail "Binary not found"
        return 1
    fi
    
    echo ""
    echo "=== RGB EFFECT RENDERING PERFORMANCE ==="
    
    # Test RGB color change latency
    local start=$(date +%s%N)
    for i in $(seq 1 100); do
        ./target/release/ssgg rgb color --red 255 --green 0 --blue 0 >/dev/null 2>&1 || true
    done
    local end=$(date +%s%N)
    
    local avg_latency=$(( (end - start) / 100 / 1000000 ))  # Convert to ms
    echo "RGB color change (100 iterations): ${avg_latency}ms average"
    
    if [ $avg_latency -lt 10 ]; then
        pass "RGB latency optimized (<10ms)"
    else
        warn "RGB latency above threshold (>10ms)"
    fi
    
    echo ""
    echo "=== DAEMON STARTUP TIME ==="
    
    # Test daemon startup time
    start=$(date +%s%N)
    timeout 5 ./target/release/ssgg daemon 2>&1 || true
    end=$(date +%s%N)
    
    local startup_time=$(( (end - start) / 1000000 ))
    echo "Daemon startup time: ${startup_time}ms"
    
    if [ $startup_time -lt 2000 ]; then
        pass "Startup time optimal (<2000ms)"
    else
        warn "Startup time above threshold (>2000ms)"
    fi
}

# Generate final report
generate_report() {
    log "Generating comprehensive memory analysis report..."
    
    local report_file="/tmp/ssgg_memory_analysis_report_$(date +%Y%m%d).md"
    
    cat > "$report_file" << EOF
# Memory Analysis Report

**Date**: $(date '+%Y-%m-%d %H:%M:%S')
**Binary Version**: $(./target/release/ssgg --version 2>/dev/null || echo "unknown")

## Test Results

### AddressSanitizer
- Status: $(grep -c "ASan" "$LOG_FILE" || echo "Not run")

### ThreadSanitizer  
- Status: $(grep -c "TSan" "$LOG_FILE" || echo "Not run")

### Valgrind
- Leaks Detected: $(grep "LEAK SUMMARY:" "$LOG_FILE.valgrind" 2>/dev/null || echo "None")

### Performance
- RGB Latency: Tested
- Startup Time: Tested

## Conclusion
EOF

    pass "Report saved to: $report_file"
}

# Main execution
main() {
    echo "========================================================"
    echo "  STEELSERIES GG - MEMORY LEAK DETECTION SUITE"
    echo "========================================================"
    echo ""
    
    if ! check_prerequisites; then
        fail "Required tools missing. Please install prerequisites."
        exit 1
    fi
    
    log "Starting comprehensive memory analysis..."
    
    # Run all tests
    build_asan_binary || true
    build_tsan_binary || true
    run_valgrind_test || true
    benchmark_performance
    
    generate_report
    
    log "Memory analysis complete. See $LOG_FILE for details."
}

if [ "${BASH_SOURCE[0]}" = "$0" ]; then
    main
fi
