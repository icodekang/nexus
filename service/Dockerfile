# syntax = docker/dockerfile:1.4
# =============================================================================
# Nexus API Gateway - Rust Multi-stage Build
# =============================================================================
# BuildKit cache mount (--mount=type=cache):
#   Persists /root/.cargo/registry and /app/target across builds on disk.
#   Incremental compilation works automatically — only changed modules recompile.
#   The cache survives even when source layers change.
# Requires: DOCKER_BUILDKIT=1 (see docker-compose.yml header comment).
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

# Copy everything. BuildKit's cache mount for /app/target persists across builds
# regardless of layer changes, so moving COPY before the build does NOT break
# incremental compilation for subsequent runs.
COPY . .

# Single build pass. The cache mount preserves previously compiled dependencies.
# First build: ~80% time on deps, ~20% on incremental code changes.
# Subsequent builds: only changed modules recompile (seconds).
RUN --mount=type=cache,target=/root/.cargo/registry \
    --mount=type=cache,target=/app/target \
    cargo build --release --package nexus-service

# -----------------------------------------------------------------------------
# Stage 2: Runtime
# -----------------------------------------------------------------------------
FROM debian:bookworm-slim AS runtime

WORKDIR /app

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    curl \
    && rm -rf /var/lib/apt/lists/*

RUN groupadd --gid 1000 nexus && \
    useradd --uid 1000 --gid nexus --shell /bin/bash nexus

COPY --from=builder /app/target/release/nexus-service /app/nexus-service

RUN chown -R nexus:nexus /app
USER nexus

EXPOSE 8080

ENV RUST_LOG=info
ENV PORT=8080
ENV DATABASE_URL=postgres://nexus:password@localhost:5432/nexus
ENV REDIS_URL=redis://localhost:6379
ENV JWT_SECRET=change-me-in-production

HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

CMD ["./nexus-service"]
