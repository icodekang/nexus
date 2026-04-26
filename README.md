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

### Docker 部署

```bash
# 启动所有服务（含 PostgreSQL + Redis）
docker-compose up -d
```

### 本地部署

自动安装依赖（Rust、Node.js、PostgreSQL、Redis），无需预先手动安装。

```bash
# 1. 一键部署（检测/安装依赖 → 生成配置 → 建库 → 编译后端 → 安装前端依赖 → 启动服务）
./scripts/setup.sh

# 或使用根目录入口
./deploy.sh

# 或只启动后端
./scripts/start.sh --no-frontend

# 启动前重新构建
./scripts/start.sh --build

# 3. 停止所有服务
./scripts/stop.sh
```

启动后:
- API Gateway: http://localhost:8080
- Admin 面板: http://localhost:3000
- Client 客户端: http://localhost:3001
- 默认管理员: admin@nexus.io / admin123

配置文件为 `.env.local`，可自定义数据库连接、端口、API Keys 等。

## Project Structure

```
nexus/
├── deploy.sh              # 一键部署入口
├── app/
│   ├── client/            # React client
│   └── admin/             # React admin dashboard
├── service/
│   ├── api/               # API Gateway (Rust)
│   ├── auth/              # Authentication (Rust)
│   ├── router/            # Router engine (Rust)
│   ├── billing/           # Billing service (Rust)
│   ├── models/            # Shared models (Rust)
│   ├── db/                # Database layer (Rust)
│   └── adapters/          # LLM provider adapters
├── scripts/
│   ├── setup.sh           # 一键部署（环境检测 → 安装依赖 → 配置 → 建库 → 编译 → 启动）
│   ├── start.sh           # 启动所有服务
│   └── stop.sh            # 停止所有服务
├── infra/
│   ├── k8s/               # Kubernetes configs
│   └── terraform/         # Terraform configs
└── docs/                  # Documentation
```

## Documentation

- [Architecture Documentation](./docs/ARCHITECTURE.md)
- [API Reference](./docs/API.md) — full endpoint reference + Python examples
- [Deployment Guide](./docs/DEPLOYMENT.md) (TODO)

## License

MIT
