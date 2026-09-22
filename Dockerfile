# SteelSeries GG for Linux - Build Container
# Multi-stage build with caching for faster CI/CD

FROM rust:1.97.1-slim-bullseye AS base
LABEL maintainer="SteelSeries Linux Team"
LABEL description="Build environment for SteelSeries GG for Linux"

# Install system dependencies
RUN apt-get update && apt-get install -y \
    libhidapi-dev \
    pkg-config \
    build-essential \
    libssl-dev \
    libpulse-dev \
    lld \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Set working directory
WORKDIR /build

# Copy manifest files first for better caching
COPY Cargo.toml Cargo.lock ./

# Stage 2: Dependencies cache
FROM base AS deps
RUN cargo fetch

# Stage 3: Build
FROM deps AS builder
COPY . .

# Use sccache if available for faster builds
ARG USE_SCCACHE=false
ENV CARGO_INCREMENTAL=0
ENV RUSTFLAGS="-C link-arg=-fuse-ld=lld"

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/build/target \
    if [ "$USE_SCCACHE" = "true" ]; then \
        cargo build --release --all-features; \
    else \
        cargo build --release --all-features; \
    fi

# Stage 4: Runtime image
FROM debian:bullseye-slim AS runtime

LABEL version="0.1.0"
LABEL release-notes="https://github.com/MikhaelCat/SteelSeries-GG-for-linux/releases"

# Install minimal runtime dependencies
RUN apt-get update && apt-get install -y \
    libhidapi0 \
    libssl3 \
    libpulse0 \
    && rm -rf /var/lib/apt/lists/*

# Copy binary from builder
COPY --from=builder /build/target/release/ssgg /usr/local/bin/ssgg
COPY assets/99-steelseries.rules /etc/udev/rules.d/99-steelseries.rules
COPY assets/ssgg.service /lib/systemd/system/ssgg.service

# Create user for running the daemon
RUN groupadd --system ssgg && useradd --system --no-create-home --gid ssgg ssgg

# Set proper permissions
RUN chmod +x /usr/local/bin/ssgg && \
    chmod 644 /etc/udev/rules.d/99-steelseries.rules && \
    chmod 644 /lib/systemd/system/ssgg.service

# Create necessary directories
RUN mkdir -p /home/ssgg/.config/ssgg /var/log/ssgg && \
    chown -R ssgg:ssgg /home/ssgg/.config/ssgg /var/log/ssgg

# Security hardening
USER ssgg

# Configure PID directory
ENV XDG_CONFIG_HOME=/home/ssgg/.config/ssgg
ENV XDG_DATA_HOME=/home/ssgg/.local/share/ssgg
ENV HOME=/home/ssgg

EXPOSE 27301

HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD pgrep -f ssgg || exit 1

CMD ["ssgg", "daemon"]

# Debug stage - full toolchain
FROM builder AS debug
RUN cargo install cargo-gcov cargo-tarpaulin cargo-machete

ENTRYPOINT ["bash"]
