# =============================================================================
# Nexus API Gateway - Rust Multi-stage Build
# =============================================================================

# -----------------------------------------------------------------------------
# Stage 1: Builder
# -----------------------------------------------------------------------------
FROM rust:1.75-bookworm AS builder

WORKDIR /app

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy Cargo workspace files
COPY service/Cargo.toml service/Cargo.lock* ./
COPY service/rust-toolchain.toml ./

# Create dummy source to cache dependencies
RUN mkdir -p src && echo "fn main() {}" > src/main.rs

# Build dependencies only (cached)
RUN cargo build --release --manifest-path ./Cargo.toml
RUN rm -rf src

# Copy actual source code
COPY service/ ./service/

# Rebuild with actual code
RUN touch service/api/src/main.rs && \
    cargo build --release --manifest-path ./Cargo.toml

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
COPY --from=builder /app/target/release/api /app/nexus-api

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
CMD ["./nexus-api"]