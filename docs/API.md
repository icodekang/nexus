# Nexus API Reference

> Unified LLM API Gateway — call OpenAI, Anthropic, Google, DeepSeek models through a single API with one API key.
>
> **SDK 透明兼容**：安装官方 `openai` / `anthropic` Python 库，只需改 `base_url` 和 `api_key`，无需修改任何业务代码。

---

## Table of Contents

1. [Overview](#1-overview)
2. [Authentication](#2-authentication)
3. [User Management](#3-user-management)
4. [API Key Management](#4-api-key-management)
5. [Models](#5-models)
6. [Chat Completions](#6-chat-completions)
7. [Anthropic Messages](#7-anthropic-messages)
8. [Streaming](#8-streaming)
9. [Subscription & Usage](#9-subscription--usage)
10. [Rate Limits](#10-rate-limits)
11. [Error Codes](#11-error-codes)
12. [Python SDK Examples](#12-python-sdk-examples)

---

## 1. Overview

**Base URL**

```
http://localhost:8080        # Local development
https://nexus.com         # Production
```

**Request format**

All request bodies must be JSON (`Content-Type: application/json`).

**Response format**

All responses are JSON. Successful responses include the requested data. Errors include a machine-readable `code` and human-readable `message`.

---

## 2. Authentication

Nexus accepts two types of credentials:

| Type | Header | Use case |
|------|--------|----------|
| API Key | `Authorization: Bearer sk-nexus-xxxx` | Production — call LLM endpoints |
| JWT Token | `Authorization: Bearer eyJhbG...` | Login / management endpoints |

### Register

```
POST /v1/auth/register
```

**Request**

```json
{
  "email": "user@example.com",
  "password": "your-password",
  "phone": "+8613800138000"
}
```

**Response** `201 Created`

```json
{
  "user": {
    "id": "uuid",
    "email": "user@example.com",
    "phone": "+8613800138000",
    "subscription_plan": "none",
    "is_admin": false
  },
  "token": "eyJhbG..."
}
```

---

### Login

```
POST /v1/auth/login
```

**Request**

```json
{
  "email": "user@example.com",
  "password": "your-password"
}
```

**Response** `200 OK`

```json
{
  "user": { "id": "...", "email": "...", "subscription_plan": "monthly", "is_admin": false },
  "token": "eyJhbG..."
}
```

---

### SMS Authentication

Send verification code:

```
POST /v1/auth/send-sms
```

```json
{ "phone": "+8613800138000" }
```

Verify and auto-register/login:

```
POST /v1/auth/verify-sms
```

```json
{ "phone": "+8613800138000", "code": "123456" }
```

---

## 3. User Management

### Get My Profile & Subscription

```
GET /v1/me/subscription
Authorization: Bearer <token>
```

**Response**

```json
{
  "user_id": "uuid",
  "email": "user@example.com",
  "subscription_plan": "monthly",
  "subscription_start": "2026-04-01T00:00:00Z",
  "subscription_end": "2026-05-01T00:00:00Z",
  "is_active": true
}
```

---

### Subscribe / Upgrade

```
POST /v1/me/subscription
Authorization: Bearer <token>
```

```json
{ "plan": "monthly" }
```

Valid plans: `monthly`, `yearly`, `team`, `enterprise`

---

### Get Subscription Plans

```
GET /v1/me/subscription/plans
```

```json
{
  "plans": [
    { "id": "monthly", "name": "Monthly", "price": 19.9, "token_quota": 10000000 },
    { "id": "yearly",  "name": "Yearly",  "price": 199,   "token_quota": 150000000 },
    { "id": "team",    "name": "Team",    "price": 99,    "token_quota": 50000000 },
    { "id": "enterprise","name": "Enterprise","price": 0,   "token_quota": 9223372036854775807 }
  ]
}
```

---

### Usage Statistics

```
GET /v1/me/usage
Authorization: Bearer <token>
```

```json
{
  "period_start": "2026-04-01T00:00:00Z",
  "period_end":   "2026-05-01T00:00:00Z",
  "total_requests": 142,
  "total_input_tokens":  35000,
  "total_output_tokens": 88000,
  "total_tokens":        123000,
  "token_quota":         10000000,
  "quota_used_percent":  1.23,
  "avg_latency_ms":      320,
  "usage_by_provider": [
    { "provider": "openai",   "requests": 100, "input_tokens": 25000, "output_tokens": 60000 },
    { "provider": "anthropic","requests":  42, "input_tokens": 10000, "output_tokens": 28000 }
  ]
}
```

---

## 4. API Key Management

API keys are used to authenticate LLM API calls. Store the plain key safely — it is only returned **once** at creation time.

### Create API Key

```
POST /v1/me/keys
Authorization: Bearer <token>
```

```json
{
  "name": "production-key"
}
```

**Response** `201 Created`

```json
{
  "id": "uuid-of-the-key",
  "key": "sk-nexus-xxxxxxxxxxxxxxxx",
  "name": "production-key",
  "created_at": "2026-04-14T12:00:00Z"
}
```

> **Save `key` now — it will never be shown again.**

---

### List API Keys

```
GET /v1/me/keys
Authorization: Bearer <token>
```

```json
{
  "data": [
    {
      "id": "uuid",
      "name": "production-key",
      "key_prefix": "sk-nexus-xxxx",
      "is_active": true,
      "last_used_at": "2026-04-14T12:30:00Z",
      "created_at": "2026-04-14T12:00:00Z"
    }
  ]
}
```

---

### Delete API Key

```
DELETE /v1/me/keys/:key_id
Authorization: Bearer <token>
```

```json
{ "deleted": true }
```

---

## 5. Models

### List Available Models

```
GET /v1/models
Authorization: Bearer <api-key>
```

Returns all models across all providers. Example response:

```json
{
  "data": [
    {
      "id": "gpt-4o",
      "object": "model",
      "created": 1713123123,
      "owned_by": "openai",
      "permission": [],
      "root": "gpt-4o",
      "parent": null
    },
    {
      "id": "anthropic/claude-3-5-sonnet-20241022",
      "object": "model",
      "owned_by": "anthropic",
      "description": "Most intelligent model for complex tasks"
    }
  ]
}
```

---

## 6. Chat Completions

### OpenAI-Compatible Endpoint

```
POST /v1/openai/chat/completions
Authorization: Bearer <api-key>
```

**Request**

```json
{
  "model": "gpt-4o",
  "messages": [
    { "role": "system", "content": "You are a helpful assistant." },
    { "role": "user", "content": "Explain quantum entanglement in simple terms." }
  ],
  "temperature": 0.7,
  "max_tokens": 500,
  "stream": false,
  "top_p": 1.0,
  "stop": ["."]
}
```

**Parameters**

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `model` | string | Yes | — | Model slug or `provider/model` (e.g. `anthropic/claude-3-5-sonnet`) |
| `messages` | array | Yes | — | Array of `{role, content}` objects |
| `temperature` | float | No | 0.7 | Sampling temperature (0.0–2.0) |
| `max_tokens` | integer | No | — | Maximum tokens to generate |
| `stream` | boolean | No | false | Enable SSE streaming |
| `top_p` | float | No | 1.0 | Nucleus sampling |
| `stop` | array | No | — | Stop sequences |

**Response** `200 OK`

```json
{
  "id": "chatcmpl-xxx",
  "object": "chat.completion",
  "created": 1713123456,
  "model": "gpt-4o",
  "choices": [
    {
      "index": 0,
      "message": {
        "role": "assistant",
        "content": "Quantum entanglement is when two particles..."
      },
      "finish_reason": "stop"
    }
  ],
  "usage": {
    "prompt_tokens": 42,
    "completion_tokens": 128,
    "total_tokens": 170
  }
}
```

**Specifying a provider explicitly**

```json
{ "model": "anthropic/claude-3-5-sonnet-20241022" }
```

---

### Internal Chat Endpoint

```
POST /v1/chat/completions
Authorization: Bearer <api-key>
```

Same request/response format as above. Routes through the internal router with configurable strategy via header:

```
X-Route-Strategy: balanced   # cheapest | fastest | quality | balanced
```

---

## 7. Anthropic Messages

### Anthropic-Compatible Endpoint

```
POST /v1/anthropic/messages
Authorization: Bearer <api-key>
```

**Request**

```json
{
  "model": "claude-3-5-sonnet-20241022",
  "messages": [
    { "role": "user", "content": "What is the capital of France?" }
  ],
  "max_tokens": 256,
  "temperature": 1.0,
  "stream": false,
  "system": "You are a helpful geography tutor."
}
```

> **Note:** `max_tokens` is **required** for Anthropic requests.

**Response** `200 OK`

```json
{
  "id": "msg_xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx",
  "type": "message",
  "role": "assistant",
  "content": [
    { "type": "text", "text": "The capital of France is Paris." }
  ],
  "model": "claude-3-5-sonnet-20241022",
  "stop_reason": "end_turn",
  "stop_sequence": null,
  "usage": {
    "input_tokens":  32,
    "output_tokens": 18
  }
}
```

---

## 8. Streaming

Pass `"stream": true` in the request body. Both endpoints return **Server-Sent Events (SSE)**.

### OpenAI Streaming

Each chunk:

```
event: message
data: {"id":"chatcmpl-xxx","object":"chat.completion.chunk","model":"gpt-4o","delta":{"content":"Quantum"},"finish_reason":null}
```

Final chunk (`finish_reason` not null) contains the full usage.

### Anthropic Streaming

Events follow the Anthropic SSE protocol:

```
event: message_start
data: {"type":"message_start","message":{"id":"msg_xxx","type":"message","role":"assistant","content":[],"model":"claude-3-5-sonnet"}}

event: content_block_start
data: {"type":"content_block_start","index":0,"content_block":{"type":"text","text":""}}

event: content_block_delta
data: {"type":"content_block_delta","index":0,"delta":{"type":"text_delta","text":"Paris"}}

event: message_delta
data: {"type":"message_delta","delta":{"stop_reason":"end_turn"},"usage":{"output_tokens":18}}

event: message_stop
data: {"type":"message_stop"}
```

---

## 9. Subscription & Usage

Nexus uses a subscription model. All LLM calls are included in the subscription fee.

| Plan | Monthly Price | Token Quota | RPM Limit |
|------|-------------|-------------|-----------|
| Free | $0 | 100,000/mo | 10 |
| Monthly | $19.9 | 10,000,000/mo | 60 |
| Yearly | $199 | 150,000,000/mo | 120 |
| Team | $99 | 50,000,000/mo | 300 |
| Enterprise | Custom | Unlimited | 1000 |

> **Quota enforcement:** When token quota is exceeded, API returns `400` with message `Token quota exceeded. Used X / Y.`

---

## 10. Rate Limits

Rate limits are enforced per-user per-minute (RPM) based on subscription plan.

**Response headers on every LLM call:**

```
X-RateLimit-Limit: 60
X-RateLimit-Remaining: 58
X-RateLimit-Reset: 1713123456
```

When exceeded, the API returns `429 Too Many Requests`.

---

## 11. Error Codes

| HTTP Status | Code | Description |
|-------------|------|-------------|
| `400` | `INVALID_REQUEST` | Malformed request body or parameter |
| `400` | `MODEL_NOT_FOUND` | Unknown model slug |
| `400` | `Token quota exceeded` | Monthly token quota used up |
| `401` | `INVALID_CREDENTIALS` | Bad email/password |
| `401` | `INVALID_API_KEY` | API key not found or inactive |
| `403` | `FORBIDDEN` | Admin access required |
| `403` | `SUBSCRIPTION_EXPIRED` | Subscription has expired |
| `429` | `RATE_LIMIT_EXCEEDED` | RPM limit hit |
| `429` | `SMS_RATE_LIMIT_EXCEEDED` | Too many SMS requests |
| `429` | `PROVIDER_ERROR` | All upstream providers failed |
| `500` | `INTERNAL_ERROR` | Server-side error |

**Error response shape:**

```json
{
  "error": {
    "code": "INVALID_API_KEY",
    "message": "API key is invalid or has been revoked.",
    "status": 401
  }
}
```

---

## 12. Python SDK Examples

### Prerequisites

```bash
pip install requests
```

---

### 12.1 Initialize Client

```python
import requests

BASE_URL = "http://localhost:8080"  # Production: https://nexus.com
API_KEY = "sk-nexus-xxxxxxxxxxxxxxxx"  # Your API key from /v1/me/keys
```

---

### 12.2 Register & Login

```python
import requests

BASE_URL = "http://localhost:8080"

# ── Register ──────────────────────────────────────────────
resp = requests.post(
    f"{BASE_URL}/v1/auth/register",
    json={
        "email": "alice@example.com",
        "password": "StrongPass123!",
        "phone": "+8613800138000",
    },
)
resp.raise_for_status()
data = resp.json()
jwt_token = data["token"]
print(f"Registered! User ID: {data['user']['id']}")

# ── Login ─────────────────────────────────────────────────
resp = requests.post(
    f"{BASE_URL}/v1/auth/login",
    json={"email": "alice@example.com", "password": "StrongPass123!"},
)
resp.raise_for_status()
jwt_token = resp.json()["token"]
```

---

### 12.3 Create an API Key

```python
import requests

BASE_URL = "http://localhost:8080"
jwt_token = "eyJhbG..."  # From login

resp = requests.post(
    f"{BASE_URL}/v1/me/keys",
    headers={"Authorization": f"Bearer {jwt_token}"},
    json={"name": "production"},
)
resp.raise_for_status()
key_data = resp.json()
api_key = key_data["key"]
print(f"API Key (save this!): {api_key}")
# {'id': 'uuid', 'key': 'sk-nexus-...', 'name': 'production', 'created_at': '...'}
```

---

### 12.4 List Available Models

```python
import requests

resp = requests.get(
    "http://localhost:8080/v1/models",
    headers={"Authorization": f"Bearer {api_key}"},
)
resp.raise_for_status()
models = resp.json()["data"]
for m in models:
    print(m["id"])
```

---

### 12.5 OpenAI-Compatible Chat (Non-Streaming)

```python
import requests

response = requests.post(
    f"{BASE_URL}/v1/openai/chat/completions",
    headers={"Authorization": f"Bearer {api_key}"},
    json={
        "model": "gpt-4o",
        "messages": [
            {"role": "system", "content": "You are a helpful assistant."},
            {"role": "user", "content": "What is 2 + 2?"},
        ],
        "temperature": 0.7,
        "max_tokens": 100,
    },
)
response.raise_for_status()
result = response.json()

print(result["choices"][0]["message"]["content"])
print(f"Tokens used: {result['usage']['total_tokens']}")
```

---

### 12.6 OpenAI-Compatible Chat (Streaming with SSE)

```python
import requests
import json

response = requests.post(
    f"{BASE_URL}/v1/openai/chat/completions",
    headers={"Authorization": f"Bearer {api_key}"},
    json={
        "model": "gpt-4o",
        "messages": [{"role": "user", "content": "Count to 5."}],
        "stream": True,
    },
    stream=True,
)
response.raise_for_status()

for line in response.iter_lines():
    if not line:
        continue
    line = line.decode("utf-8")
    if line.startswith("data: "):
        data = line[6:]  # strip "data: "
        if data == "[DONE]":
            break
        chunk = json.loads(data)
        delta = chunk.get("choices", [{}])[0].get("delta", {})
        if "content" in delta:
            print(delta["content"], end="", flush=True)
print()
```

---

### 12.7 Anthropic Messages (Non-Streaming)

```python
import requests

response = requests.post(
    f"{BASE_URL}/v1/anthropic/messages",
    headers={"Authorization": f"Bearer {api_key}"},
    json={
        "model": "claude-3-5-sonnet-20241022",
        "messages": [
            {"role": "user", "content": "What is the speed of light?"}
        ],
        "max_tokens": 256,  # Required for Anthropic
        "temperature": 1.0,
        "system": "You are a physics tutor.",
    },
)
response.raise_for_status()
result = response.json()

text = result["content"][0]["text"]
print(text)
print(f"Input tokens: {result['usage']['input_tokens']}")
print(f"Output tokens: {result['usage']['output_tokens']}")
```

---

### 12.8 Anthropic Streaming

```python
import requests

response = requests.post(
    f"{BASE_URL}/v1/anthropic/messages",
    headers={"Authorization": f"Bearer {api_key}"},
    json={
        "model": "claude-3-5-sonnet-20241022",
        "messages": [{"role": "user", "content": "Explain photosynthesis in one sentence."}],
        "max_tokens": 200,
        "stream": True,
    },
    stream=True,
)
response.raise_for_status()

for line in response.iter_lines():
    if not line:
        continue
    line = line.decode("utf-8")
    if line.startswith("event: "):
        event_type = line[7:]
    elif line.startswith("data: "):
        data = json.loads(line[6:])
        if event_type == "content_block_delta":
            print(data["delta"]["text"], end="", flush=True)
        elif event_type == "message_stop":
            print()
```

---

### 12.9 Specify Provider Explicitly

```python
# Route to a specific provider
response = requests.post(
    f"{BASE_URL}/v1/openai/chat/completions",
    headers={"Authorization": f"Bearer {api_key}"},
    json={
        # "provider/model" format routes to that provider
        "model": "anthropic/claude-3-5-sonnet-20241022",
        "messages": [{"role": "user", "content": "Hello!"}],
    },
)
```

---

### 12.10 Check Subscription & Usage

```python
import requests

# Get subscription info
resp = requests.get(
    f"{BASE_URL}/v1/me/subscription",
    headers={"Authorization": f"Bearer {jwt_token}"},
)
sub = resp.json()
print(f"Plan: {sub['subscription_plan']}, Active: {sub['is_active']}")

# Get usage stats
resp = requests.get(
    f"{BASE_URL}/v1/me/usage",
    headers={"Authorization": f"Bearer {jwt_token}"},
)
usage = resp.json()
print(f"Total tokens this period: {usage['total_tokens']}")
print(f"Quota used: {usage['quota_used_percent']:.2f}%")

# List API keys
resp = requests.get(
    f"{BASE_URL}/v1/me/keys",
    headers={"Authorization": f"Bearer {jwt_token}"},
)
keys = resp.json()["data"]
for k in keys:
    print(f"  {k['key_prefix']}... | {k['name']} | active={k['is_active']}")
```

---

### 12.11 Complete Usage Flow (One Script)

```python
#!/usr/bin/env python3
"""
Nexus API — quick integration demo.
Run after: docker-compose up -d
"""
import requests

BASE = "http://localhost:8080"
EMAIL = "demo@example.com"
PASSWORD = "DemoPass123!"
KEY_NAME = "demo-key"


def main():
    # 1. Register
    print("1. Registering user...")
    r = requests.post(f"{BASE}/v1/auth/register", json={
        "email": EMAIL, "password": PASSWORD
    })
    if r.status_code == 400 and "already exists" in r.text:
        print("   User exists, logging in instead.")
    r.raise_for_status()
    jwt = r.json()["token"]
    print(f"   OK — JWT obtained")

    # 2. Subscribe (monthly)
    print("2. Subscribing to Monthly plan...")
    r = requests.post(
        f"{BASE}/v1/me/subscription",
        headers={"Authorization": f"Bearer {jwt}"},
        json={"plan": "monthly"},
    )
    r.raise_for_status()
    print(f"   OK — {r.json()}")

    # 3. Create API key
    print("3. Creating API key...")
    r = requests.post(
        f"{BASE}/v1/me/keys",
        headers={"Authorization": f"Bearer {jwt}"},
        json={"name": KEY_NAME},
    )
    r.raise_for_status()
    api_key = r.json()["key"]
    print(f"   OK — sk-nexus-{'*' * 20}... (saved to variable)")

    # 4. Chat (OpenAI-compatible)
    print("4. Sending chat request via OpenAI endpoint...")
    r = requests.post(
        f"{BASE}/v1/openai/chat/completions",
        headers={"Authorization": f"Bearer {api_key}"},
        json={
            "model": "gpt-4o",
            "messages": [
                {"role": "user", "content": "Give me a one-sentence summary of Rust."}
            ],
            "max_tokens": 50,
        },
    )
    r.raise_for_status()
    reply = r.json()["choices"][0]["message"]["content"]
    print(f"   Response: {reply}")

    # 5. Chat (Anthropic-compatible)
    print("5. Sending chat request via Anthropic endpoint...")
    r = requests.post(
        f"{BASE}/v1/anthropic/messages",
        headers={"Authorization": f"Bearer {api_key}"},
        json={
            "model": "claude-3-5-sonnet-20241022",
            "messages": [
                {"role": "user", "content": "Give me a one-sentence summary of Go."}
            ],
            "max_tokens": 50,
        },
    )
    r.raise_for_status()
    reply = r.json()["content"][0]["text"]
    print(f"   Response: {reply}")

    print("\nAll done!")


if __name__ == "__main__":
    main()
```

**Expected output:**

```
1. Registering user...
   OK — JWT obtained
2. Subscribing to Monthly plan...
   OK — {'message': 'Subscription updated successfully', ...}
3. Creating API key...
   OK — sk-nexus-********************... (saved to variable)
4. Sending chat request via OpenAI endpoint...
   Response: Rust is a systems programming language focused on safety, speed, and concurrency.
5. Sending chat request via Anthropic endpoint...
   Response: Go (or Golang) is a statically typed, compiled language designed at Google for simplicity and efficient concurrency.
```

---

## Appendix A: OpenAI Python SDK（官方库透明兼容）

Nexus 对 OpenAI Python SDK 完全透明兼容。只需将 `base_url` 指向 Nexus，无需修改任何业务代码。

### 安装

```bash
pip install openai
```

### 方式一：代码中直接配置

```python
from openai import OpenAI

client = OpenAI(
    api_key="sk-nexus-xxxxxxxxxxxxxxxx",       # 你在 Nexus 创建的 API Key
    base_url="https://nexus.com/v1/openai" # Nexus OpenAI 兼容端点
)

# 以下所有调用均为标准 OpenAI SDK 用法，完全不动
chat = client.chat.completions.create(
    model="gpt-4o",
    messages=[{"role": "user", "content": "Hello!"}],
)
print(chat.choices[0].message.content)
```

### 方式二：通过环境变量配置（推荐）

SDK 会自动读取 `OPENAI_API_KEY` 和 `OPENAI_BASE_URL` 环境变量：

```bash
export OPENAI_API_KEY="sk-nexus-xxxxxxxxxxxxxxxx"
export OPENAI_BASE_URL="https://nexus.com/v1/openai"
```

```python
from openai import OpenAI

client = OpenAI()  # 自动读取环境变量，无需传参

chat = client.chat.completions.create(
    model="gpt-4o",
    messages=[{"role": "user", "content": "Explain quantum computing in one sentence."}],
)
print(chat.choices[0].message.content)
```

### 流式输出

```python
stream = client.chat.completions.create(
    model="gpt-4o",
    messages=[{"role": "user", "content": "Write a haiku about coding."}],
    stream=True,
)
for chunk in stream:
    if chunk.choices[0].delta.content:
        print(chunk.choices[0].delta.content, end="", flush=True)
```

### 支持的全部模型

在 `model` 参数中传入任意 Nexus 支持的模型名即可，例如：

```python
# OpenAI 模型
client.chat.completions.create(model="gpt-4o", ...)
client.chat.completions.create(model="gpt-4o-mini", ...)

# 跨提供商路由（Nexus 自动选择最优 Provider）
client.chat.completions.create(model="claude-3-5-sonnet-20241022", ...)
client.chat.completions.create(model="gemini-pro", ...)
client.chat.completions.create(model="deepseek-chat", ...)

# 明确指定 Provider
client.chat.completions.create(model="anthropic/claude-3-5-sonnet-20241022", ...)
```

---

## Appendix B: Anthropic Python SDK（官方库透明兼容）

Nexus 对 Anthropic Python SDK 完全透明兼容。SDK 会发送 `x-api-key` 头，Nexus 中间件已做兼容处理。

### 安装

```bash
pip install anthropic
```

### 方式一：代码中直接配置

```python
import anthropic

client = anthropic.Anthropic(
    api_key="sk-nexus-xxxxxxxxxxxxxxxx",        # 你在 Nexus 创建的 API Key
    base_url="https://nexus.com/v1/anthropic" # Nexus Anthropic 兼容端点
)

message = client.messages.create(
    model="claude-3-5-sonnet-20241022",
    max_tokens=256,       # Anthropic 必须指定
    messages=[
        {"role": "user", "content": "What is the meaning of life?"}
    ],
)
print(message.content[0].text)
```

### 方式二：通过环境变量配置（推荐）

```bash
export ANTHROPIC_API_KEY="sk-nexus-xxxxxxxxxxxxxxxx"
export ANTHROPIC_BASE_URL="https://nexus.com/v1/anthropic"
```

```python
import anthropic

client = anthropic.Anthropic()  # 自动读取环境变量

message = client.messages.create(
    model="claude-3-5-sonnet-20241022",
    max_tokens=256,
    messages=[{"role": "user", "content": "Hello, Claude!"}],
)
print(message.content[0].text)
```

### 流式输出

```python
with client.messages.stream(
    model="claude-3-5-sonnet-20241022",
    max_tokens=256,
    messages=[{"role": "user", "content": "Count to 5."}],
) as stream:
    for text in stream.text_stream:
        print(text, end="", flush=True)
```

---

## Appendix C: 与 LangChain / CrewAI / AutoGen 集成

由于 Nexus 完全兼容 OpenAI 和 Anthropic SDK，任何基于这些 SDK 的上层框架均可无缝接入。

### LangChain + Nexus（OpenAI 模型）

```python
from langchain_openai import ChatOpenAI

llm = ChatOpenAI(
    model="gpt-4o",
    openai_api_key="sk-nexus-xxxxxxxxxxxxxxxx",
    openai_api_base="https://nexus.com/v1/openai",  # 关键配置
)
response = llm.invoke("What is 1+1?")
print(response.content)
```

### LangChain + Nexus（Anthropic 模型）

```python
from langchain_anthropic import ChatAnthropic

llm = ChatAnthropic(
    model="claude-3-5-sonnet-20241022",
    anthropic_api_key="sk-nexus-xxxxxxxxxxxxxxxx",
    anthropic_api_url="https://nexus.com/v1/anthropic",  # 关键配置
)
response = llm.invoke("What is 1+1?")
print(response.content)
```

### CrewAI 示例

```python
from crewai import Agent, Task, Crew
from langchain_openai import ChatOpenAI

llm = ChatOpenAI(
    model="gpt-4o",
    openai_api_key="sk-nexus-xxxxxxxxxxxxxxxx",
    openai_api_base="https://nexus.com/v1/openai",
)

researcher = Agent(
    role="Researcher",
    goal="Research AI trends",
    llm=llm,
)

crew = Crew(agents=[researcher], tasks=[...])
crew.kickoff()
```

### AutoGen 示例

```python
from autogen import AssistantAgent, UserProxyAgent
from autogen.agentchat.contrib.multimodal_conversable_agent import MultimodalConversableAgent

llm_config = {
    "model": "gpt-4o",
    "api_key": "sk-nexus-xxxxxxxxxxxxxxxx",
    "base_url": "https://nexus.com/v1/openai",
}

assistant = AssistantAgent("assistant", llm_config=llm_config)
user_proxy = UserProxyAgent("user", human_input_mode="NEVER")

user_proxy.initiate_chat(assistant, message="Write a short poem about the sea.")
```

---

## Appendix D: 与 Dify / NextChat 等第三方工具集成

Nexus 可以作为 OpenAI API 的自定义 Endpoint 接入大量第三方 AI 工具：

| 工具 | 配置方式 |
|------|---------|
| **Dify** | 设置 `API Key` 为 Nexus Key，`endpoint` 为 `https://nexus.com/v1/openai` |
| **NextChat (ChatGPT Next Web)** | 自定义 API Endpoint 填入 `https://nexus.com/v1/openai`，API Key 填入 Nexus Key |
| **LobeChat** | 同 NextChat 配置方式 |
| **AnythingLLM** | 模型 Provider 选择 OpenAI Compatible，填入 Nexus base URL 和 API Key |
| **Open WebUI** | 设置 `OPENAI_API_BASE_URL` 为 `https://nexus.com/v1/openai` |
| **RAG 应用（LlamaIndex）** | `Settings.llm = OpenAI(key, base_url)` |
| **Django / FastAPI 后端** | 同 LangChain 方式配置 LLM |

> **接入 Dify 示例：**
> 1. 进入 Dify → 设置 → 模型供应商 → OpenAI
> 2. `API Base` 填入：`https://nexus.com/v1/openai`
> 3. `API Key` 填入：`sk-nexus-xxxxxxxxxxxxxxxx`
> 4. 保存后即可在 Dify 中使用全部 Nexus 模型

---

*Document version: v1.0 — Nexus API Reference*
*Last updated: 2026-04-14*
