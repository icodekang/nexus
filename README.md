# nexus

> A unified LLM API gateway platform with subscription-based access to multiple providers.

## Project Status

**Development in Progress** — All core modules have been implemented.

## Modules Completed

### Backend (Rust)
- ✅ `service/models` — Shared data models (User, Provider, LlmModel, Subscription, ApiLog)
- ✅ `service/db` — Database layer (PostgreSQL + Redis) with migrations
- ✅ `service/auth` — Authentication (JWT, API Key, Password hashing)
- ✅ `service/api` — API Gateway (axum) with all endpoints
- ✅ `service/router` — Router engine with pressure-equilibrium key scheduling
- ✅ `service/billing` — Billing and subscription management
- ✅ `service/adapters` — Provider clients (HTTP + headless-browser) for OpenAI, Anthropic, DeepSeek, and more

### Frontend
- ✅ `app/client` — React + Vite web client with Chat, ModelSelect, Profile screens
- ✅ `app/admin` — React + Vite admin dashboard with Dashboard, Users, Providers, Models, Transactions pages

> **Note:** `app/client` is a **web SPA**, not a React Native mobile app. It runs in the browser and communicates with the API Gateway over HTTP.

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                        Clients                                │
│   app/client (React + Vite web) · app/admin (React)         │
└──────────────────────────┬──────────────────────────────────┘
                           │ HTTPS
                           ▼
┌──────────────────────────────────────────────────────────────┐
│                    service/ (Rust monolith)                   │
│   api · auth · router · billing · models · db · adapters    │
└──────────────────────────────────────────────────────────────┘
                           │ HTTP
                           ▼
┌──────────────────────────────────────────────────────────────┐
│                    Provider APIs                               │
│           OpenAI · Anthropic · Google · DeepSeek              │
└──────────────────────────────────────────────────────────────┘
```

The `service/adapters` directory is a Rust crate (`provider_client`) that calls upstream providers directly over HTTP. There is no separate Python service or gRPC layer.

## Quick Start

### Docker

```bash
# Start all services (PostgreSQL + Redis + API + Admin + Client)
docker-compose up -d
```

### Local development

```bash
# One-shot setup (detect env → install deps → configure → migrate → build → start)
./scripts/setup.sh

# Or use the top-level entrypoint
./deploy.sh

# Start backend only
./scripts/start.sh --no-frontend

# Rebuild before start
./scripts/start.sh --build

# Stop everything
./scripts/stop.sh
```

After startup:
- API Gateway: http://localhost:8080
- Admin Panel: http://localhost:3000
- Web Client (SPA): http://localhost:3001
- Default admin: admin@nexus.dev / admin123

## Project Structure

```
nexus/
├── deploy.sh              # One-shot deploy entrypoint
├── app/
│   ├── client/            # React + Vite web client (not React Native)
│   └── admin/             # React + Vite admin dashboard
├── service/
│   ├── api/               # API Gateway (Rust — axum)
│   ├── auth/              # Authentication (Rust)
│   ├── router/            # Router engine (Rust) — key scheduling & fallback
│   ├── billing/           # Billing service (Rust)
│   ├── models/            # Shared data models (Rust)
│   ├── db/                # Database layer (Rust) — PostgreSQL + Redis
│   └── adapters/          # Provider clients (Rust crate: `provider_client`)
├── scripts/
│   ├── setup.sh           # One-shot setup (env → deps → config → migrate → build → start)
│   ├── start.sh           # Start all services
│   └── stop.sh            # Stop all services
├── infra/
│   ├── k8s/               # Kubernetes configs
│   └── terraform/         # Terraform configs
└── docs/                  # Documentation
```

## Documentation

- [Architecture Documentation](./docs/ARCHITECTURE.md)
- [API Reference](./docs/API.md) — full endpoint reference + Python examples

## License

MIT
