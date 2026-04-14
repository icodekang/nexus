# nexus

> A unified LLM API gateway platform with subscription-based access to multiple providers.

## Project Status

**Development in Progress** - All core modules have been implemented.

## Modules Completed

### Backend (Rust)
- ✅ `service/models` - Shared data models (User, Provider, LlmModel, Subscription, ApiLog)
- ✅ `service/db` - Database layer (PostgreSQL + Redis) with migrations
- ✅ `service/auth` - Authentication (JWT, API Key, Password hashing)
- ✅ `service/api` - API Gateway (axum) with all endpoints
- ✅ `service/router` - Router engine with provider selection strategies
- ✅ `service/billing` - Billing and subscription management

### Python Adapters
- ✅ `service/adapters` - OpenAI, Anthropic, Google, DeepSeek adapters

### Frontend
- ✅ `app/client` - React Native client with Chat, ModelSelect, Profile screens
- ✅ `app/admin` - Admin dashboard with Dashboard, Users, Providers, Models, Transactions pages

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                        Clients                                │
│   app/client (React Native) · app/admin (React Dashboard)   │
└──────────────────────────┬──────────────────────────────────┘
                           │ HTTPS
                           ▼
┌──────────────────────────────────────────────────────────────┐
│                    service/ (Rust)                            │
│   api · auth · router · billing · models · db               │
└──────────────────────────┬───────────────────────────────────┘
                           │ gRPC
                           ▼
┌──────────────────────────────────────────────────────────────┐
│              service/adapters/ (Python)                      │
│           OpenAI · Anthropic · Google · DeepSeek             │
└──────────────────────────────────────────────────────────────┘
```

## Quick Start

```bash
# Start all services
docker-compose up -d

# Or for development
cd service && cargo build --release
cd service/adapters && pip install -r requirements.txt
```

## Project Structure

```
nexus/
├── app/
│   ├── client/          # React Native mobile app
│   └── admin/            # React admin dashboard
├── service/
│   ├── api/             # API Gateway (Rust)
│   ├── auth/            # Authentication (Rust)
│   ├── router/          # Router engine (Rust)
│   ├── billing/         # Billing service (Rust)
│   ├── models/          # Shared models (Rust)
│   ├── db/              # Database layer (Rust)
│   └── adapters/         # Python LLM adapters
├── infra/
│   ├── k8s/             # Kubernetes configs
│   └── terraform/       # Terraform configs
└── docs/               # Documentation
```

## Documentation

- [Architecture Documentation](./docs/ARCHITECTURE.md)
- [API Reference](./docs/API.md) — full endpoint reference + Python examples
- [Deployment Guide](./docs/DEPLOYMENT.md) (TODO)

## License

MIT
