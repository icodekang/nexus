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

# Configure cargo mirror (USTC sparse index)
RUN mkdir -p ~/.cargo && \
    printf '%s\n' \
        '[source.crates-io]' \
        'replace-with = "ustc"' \
        '' \
        '[source.ustc]' \
        'registry = "sparse+https://mirrors.ustc.edu.cn/crates.io-index/"' \
        > ~/.cargo/config.toml

# Copy everything
COPY . .

# Build release binary
RUN cargo build --release --package nexus-service

# Copy binary to a stable path so runtime stage can find it
RUN cp /app/target/release/nexus-service /app/nexus-service-binary

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

COPY --from=builder /app/nexus-service-binary /app/nexus-service

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
