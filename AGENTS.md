# AGENTS.md

## Project Overview

nexus is an LLM API gateway platform (similar to OpenRouter) that aggregates multiple LLM providers behind a unified subscription-based API.

## Architecture

**Stack:**
- `app/client/` — React Native (cross-platform: iOS, Android, Windows, macOS, Linux, Web)
- `app/admin/` — React + Vite + TypeScript admin dashboard
- `service/api/` — Rust Axum API gateway (ports 443/8080)
- `service/auth/` — Rust authentication service (JWT, API key generation/validation)
- `service/router/` — Rust routing engine (strategy pattern for provider selection)
- `service/billing/` — Rust subscription/billing service
- `service/models/` — Rust shared data models
- `service/db/` — Rust database layer (PostgreSQL + Redis + ClickHouse)
- `service/adapters/` — Python FastAPI provider adapters (OpenAI, Anthropic, Google, DeepSeek)

**Database:** PostgreSQL + Redis + ClickHouse (usage logging)

## API Conventions

- OpenAI-compatible endpoints: `/v1/chat/completions`, `/v1/chat/stream`, `/v1/completions`, `/v1/embeddings`, `/v1/models`
- Auth via `Authorization: Bearer <API_KEY>` header (SHA256-hashed keys stored in DB)
- Route strategy header: `X-Route-Strategy: cheapest | fastest | quality | balanced`
- Model specifier format: `provider/model-name` (e.g., `anthropic/claude-3-5-sonnet`)
- Adapter service runs on port 50051 and communicates via HTTP (FastAPI), not gRPC

## Key Implementation Notes

- API key storage: bcrypt password hashing, SHA256 key hashing (never plaintext)
- Adapter endpoints: `/v1/chat/completions`, `/v1/chat/stream`, `/v1/embeddings`, `/health`, `/providers`
- Admin dashboard at port 3000 (dev) / served via nginx in prod, API gateway at ports 443/8080
- **Dockerfile status:** Only `service/adapters/Dockerfile` is implemented; `service/Dockerfile`, `app/admin/Dockerfile`, and `app/client/Dockerfile` are empty placeholders

## Development Commands

```bash
# Rust services (workspace at service/)
cd service && cargo build --release

# Python adapters
cd service/adapters && pip install -r requirements.txt && python -m adapter.main

# React Native client
cd app/client && npm install && npm start

# Admin dashboard
cd app/admin && npm install && npm run dev

# Docker Compose (full stack)
docker-compose up -d
```

## Required Tool Versions

- Rust 1.75+
- Python 3.11+
- Node.js 18+
- PostgreSQL 16+
- Redis 7+

## Important Files

- `docs/ARCHITECTURE.md` — Detailed architecture documentation (830 lines, Chinese)
- `README.md` — Product overview and quick start
- `docker-compose.yml` — Service orchestration (API:8080, adapters:50051, postgres:5432, redis:6379, admin:3000)
