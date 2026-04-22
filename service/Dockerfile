# =============================================================================
# Nexus API Gateway - Rust Multi-stage Build
# =============================================================================

# -----------------------------------------------------------------------------
# Stage 1: Builder
# -----------------------------------------------------------------------------
FROM rust:1.94-bookworm AS builder

WORKDIR /app

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Configure cargo mirror (USTC, tsinghua backup)
RUN mkdir -p ~/.cargo && \
    echo '[source.crates-io]' >> ~/.cargo/config.toml && \
    echo 'replace-with = "ustc"' >> ~/.cargo/config.toml && \
    echo '' >> ~/.cargo/config.toml && \
    echo '[source.ustc]' >> ~/.cargo/config.toml && \
    echo 'registry = "sparse+https://mirrors.ustc.edu.cn/crates.io-index/"' >> ~/.cargo/config.toml

# Copy Cargo workspace files
COPY Cargo.toml Cargo.lock* ./

# Create dummy workspace member crates for dependency caching
RUN for member in models db auth api router billing adapters; do \
        mkdir -p $member/src && \
        echo '[package]' > $member/Cargo.toml && \
        echo "name = \"$member\"" >> $member/Cargo.toml && \
        echo 'version = "0.1.0"' >> $member/Cargo.toml && \
        echo 'edition = "2021"' >> $member/Cargo.toml && \
        echo "fn main() {}" > $member/src/main.rs && \
        echo "pub fn dummy() {}" > $member/src/lib.rs; \
    done && \
    echo 'fn main() {}' > main.rs

# Build dependencies only (cached)
RUN cargo build --release
RUN for member in models db auth api router billing adapters; do rm -rf $member/src; done && rm -f main.rs

# Copy actual source code
COPY . .

# Rebuild with actual code
RUN touch main.rs && \
    cargo build --release

# -----------------------------------------------------------------------------
# Stage 2: Runtime
# -----------------------------------------------------------------------------
FROM debian:bookworm-slim AS runtime

WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN groupadd --gid 1000 nexus && \
    useradd --uid 1000 --gid nexus --shell /bin/bash nexus

# Copy binary from builder
COPY --from=builder /app/target/release/nexus-service /app/nexus-service

# Set ownership
RUN chown -R nexus:nexus /app

# Switch to non-root user
USER nexus

# Expose port
EXPOSE 8080

# Environment defaults (should be overridden via docker-compose)
ENV RUST_LOG=info
ENV PORT=8080
ENV DATABASE_URL=postgres://nexus:password@localhost:5432/nexus
ENV REDIS_URL=redis://localhost:6379
ENV JWT_SECRET=change-me-in-production

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

# Run
CMD ["./nexus-service"]
