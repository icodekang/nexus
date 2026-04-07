# NovaChat

**A unified LLM API gateway inspired by OpenRouter, with a beautiful chat interface inspired by DeepSeek.**

## Features

- 🔥 **Unified API** - One endpoint to access 100+ LLMs from OpenAI, Anthropic, Google, DeepSeek, and more
- 💰 **Credits System** - Pay per token, like OpenRouter
- 🤖 **Smart Routing** - Automatically select the best provider based on price, latency, or quality
- 💬 **Beautiful UI** - Clean, modern chat interface
- 🖥️ **Cross-Platform** - Windows, macOS, Linux, iOS, Android, and Web
- 🔒 **Secure** - API key management with secure storage

## Architecture

```
┌──────────────────────────────────────────────────────────────────┐
│                         Clients                                   │
│   Windows App · macOS App · Linux App · iOS · Android · Web      │
└────────────────────────────┬─────────────────────────────────────┘
                             │ HTTPS / WSS
                             ▼
┌──────────────────────────────────────────────────────────────────┐
│                    Rust Core Services                             │
│   API Gateway (axum) · Auth Service · Router Core · Billing      │
└────────────────────────────┬─────────────────────────────────────┘
                             │ gRPC
                             ▼
┌──────────────────────────────────────────────────────────────────┐
│              Python Adapter Service                               │
│   OpenAI · Anthropic · Google · DeepSeek · Mistral · Cohere     │
└────────────────────────────┬─────────────────────────────────────┘
                             │
                             ▼
┌──────────────────────────────────────────────────────────────────┐
│              External LLM Provider APIs                           │
└──────────────────────────────────────────────────────────────────┘
```

## Tech Stack

| Layer | Technology |
|------|------------|
| Core Language | Rust |
| HTTP Framework | Axum |
| Database | PostgreSQL + Redis |
| Provider Adapters | Python + FastAPI |
| Desktop App | Tauri 2.0 + React |
| Mobile App | React Native |
| Admin Dashboard | React + Vite |

## Quick Start

### Prerequisites

- Rust 1.75+
- Python 3.11+
- PostgreSQL 16+
- Redis 7+

### Development

1. Clone and install dependencies:

```bash
# Clone the repository
git clone https://github.com/your-org/nova-chat.git
cd nova-chat

# Install Rust dependencies
cargo build

# Install Python dependencies
cd adapters
pip install -r requirements.txt
cd ..
```

2. Set up environment variables:

```bash
cp .env.example .env
# Edit .env with your API keys
```

3. Start services with Docker Compose:

```bash
docker-compose up -d
```

4. Access the applications:

- API Gateway: https://localhost:443
- Admin Dashboard: http://localhost:3000
- Adapter Service: http://localhost:50051

## Project Structure

```
nova-chat/
├── crates/                    # Rust core services
│   ├── api-gateway/           # HTTP API server
│   ├── router-core/           # Smart routing engine
│   ├── auth-service/          # Authentication
│   ├── billing/              # Credits & billing
│   ├── models/               # Shared data models
│   └── db/                   # Database layer
├── adapters/                  # Python provider adapters
│   └── adapter/
│       ├── openai.py
│       ├── anthropic.py
│       ├── google.py
│       └── deepseek.py
├── clients/                  # Client applications
│   ├── nova-chat/           # Desktop app (Tauri)
│   └── nova-mobile/         # Mobile app (React Native)
├── admin-dashboard/          # Admin web dashboard
└── migrations/               # SQL migrations
```

## API Reference

### Chat Completions

```bash
curl -X POST https://api.novachat.com/v1/chat/completions \
  -H "Authorization: Bearer sk-nova-xxxx" \
  -H "Content-Type: application/json" \
  -d '{
    "model": "gpt-4o",
    "messages": [{"role": "user", "content": "Hello!"}]
  }'
```

### Model Selection

Use provider/model format to select a specific provider:

```json
{
  "model": "anthropic/claude-3-5-sonnet"
}
```

Or let the router select automatically:

```json
{
  "model": "gpt-4o"
}
```

## Configuration

### Routing Strategies

- `cheapest` - Select the lowest price provider
- `fastest` - Select the lowest latency provider
- `quality` - Select the highest quality (largest context window)
- `balanced` - Balanced score of all factors

Set via header:
```
X-Route-Strategy: cheapest
```

## License

MIT
