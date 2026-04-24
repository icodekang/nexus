# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project overview

nexus is a unified LLM API gateway (OpenRouter-style) with subscription billing, multi-provider routing, and cross-platform clients.

- `nexus/app/client/` — React Native mobile app (Chat, ModelSelect, Profile screens)
- `nexus/app/admin/` — React + Vite admin dashboard
- `nexus/service/` — Rust backend: API gateway, auth, router, billing, shared models, db layer
- `nexus/service/adapters/` — Python FastAPI adapters (OpenAI, Anthropic, Google, DeepSeek)
- `nexus/infra/` — Kubernetes configs + Terraform

## Development

```bash
# Rust backend
cd service && cargo build --release

# Python adapters
cd service/adapters && pip install -r requirements.txt

# Full stack (requires Docker)
docker-compose up -d

# Run Rust tests
cd service && cargo test --workspace
```

## Architecture

```
Client (React Native / React) ──HTTPS──▶  API Gateway (Rust + Axum)
                                            ├─ auth      (JWT + API Key)
                                            ├─ router    (strategy: cheapest/fastest/balanced)
                                            ├─ billing   (subscriptions)
                                            ├─ models    (shared types)
                                            └─ db        (PostgreSQL + Redis)
                                                             │
                                               gRPC / HTTP ─┘
                                                             ▼
                                                  Python Adapters → upstream LLM providers
```

API is OpenAI-compatible: `POST /v1/chat/completions`, `GET /v1/models`, etc.

## Key constraints

- API keys stored as SHA256 hashes (never plaintext)
- Provider adapter protocol differences: Anthropic uses `/v1/messages` (not `/v1/chat/completions`) and a separate `max_tokens` field
- Routing strategies: `cheapest`, `fastest`, `quality`, `balanced` (configurable via `X-Route-Strategy` header)
- Subscription plans: `monthly` / `yearly` / `team` with states `active` / `expired` / `cancelled`

## Working agreement

- Keep shared defaults in `.claw.json`; reserve `.claw/settings.local.json` for machine-local overrides.
- Do not overwrite existing `CLAUDE.md` content automatically; update it intentionally when repo workflows change.
