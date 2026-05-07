# nexus 系统架构设计文档

> 本文档描述 nexus 的技术架构，基于 OpenRouter 商业模式，使用 Rust 作为核心服务语言。
>
> **最后更新: 2026-05-08**

---

## 1. 产品定位与商业模式

### 1.1 产品定位

nexus 是一个统一的 LLM API 网关平台，类似 OpenRouter：

- **聚合多提供商**：通过统一 API 访问多 LLM 模型
- **智能路由**：压力均衡 Key 调度 + 会话亲和性 + 自动故障转移
- **订阅收费**：用户按月/年订阅，无限使用 API
- **Web 客户端**：React + Vite 单页应用

### 1.2 商业模式（OpenRouter 模式）

```
用户订阅 → 支付月/年费 → 无限使用 API → 平台赚取差价
```

### 1.3 盈利模型

```
收入 = 用户订阅费 (月/年)
成本 = Provider API 费用 (按调用量)
利润 = 固定订阅费 - Provider API 费用
```

**订阅方案示例：**

| 方案 | 价格 | 适用场景 |
|------|------|----------|
| ZeroToken | ¥10/月 | 浏览器模拟访问，无需 API Key |
| 月付 | $19.9/月 | 个人用户，轻度使用 |
| 年付 | $199/年 ($16.6/月) | 长期用户，节省 17% |
| 团队 | $99/月 | 小团队，5 个席位 |
| 企业 | 定制报价 | 大客户，私有部署 |

---

## 2. 系统架构图

```
┌─────────────────────────────────────────────────────────────┐
│                        Clients                                │
│   app/client (React + Vite web SPA)                          │
│   app/admin  (React + Vite dashboard)                       │
└──────────────────────────┬──────────────────────────────────┘
                           │ HTTPS
                           ▼
┌──────────────────────────────────────────────────────────────┐
│                 service/ (Rust monolith)                      │
│                                                              │
│   api · auth · router · billing · models · db · adapters    │
│                                                              │
│   ┌──────────┐  ┌──────────┐  ┌──────────┐                 │
│   │   Auth    │  │  Router  │  │ Billing  │                 │
│   │ service/  │  │ service/ │  │ service/ │                 │
│   │   auth/   │  │  router/ │  │ billing/ │                 │
│   └──────────┘  └──────────┘  └──────────┘                 │
│                                                              │
│   ┌──────────┐  ┌──────────┐  ┌──────────┐                 │
│   │  Models   │  │    DB    │  │ Adapters │                 │
│   │ service/  │  │ service/ │  │ service/ │                 │
│   │  models/  │  │   db/    │  │adapters/ │                 │
│   └──────────┘  └──────────┘  └──────────┘                 │
│                                                              │
│   PostgreSQL · Redis                                         │
└──────────────────────────┬───────────────────────────────────┘
                           │ HTTP
                           ▼
┌──────────────────────────────────────────────────────────────┐
│                Upstream Provider APIs                         │
│        OpenAI · Anthropic · Google · DeepSeek                │
└──────────────────────────────────────────────────────────────┘
```

> **注意：** `service/adapters/` 是 Rust crate（`provider_client`），通过 HTTP 直接调用上游 API。
> 不存在独立的 Python 服务或 gRPC 层。所有 Provider 调用均在 Rust 进程内完成。

---

## 3. 目录结构

```
nexus/
├── app/                    # 前端应用
│   ├── client/             # React + Vite web 客户端（非 React Native）
│   │   └── src/
│   │       ├── pages/      # ChatScreen, ModelSelectScreen, SettingsScreen, …
│   │       ├── components/ # ChatBubble, ChatInput, ModelCard, Sidebar, …
│   │       ├── stores/     # Zustand: chatStore, modelStore, userStore
│   │       └── api/        # API 客户端
│   │
│   └── admin/              # React + Vite 管理后台
│       └── src/
│           ├── pages/      # Dashboard, Users, Providers, Models, Transactions, …
│           ├── components/ # Layout, DataTable, Charts
│           └── api/        # Admin API 客户端
│
├── service/                # 后端核心 (Rust workspace)
│   ├── Cargo.toml          # Workspace 根
│   ├── main.rs             # 入口 — 启动 axum HTTP 服务
│   ├── Dockerfile
│   │
│   ├── api/                # API 网关 (Rust — axum 0.7)
│   │   └── src/
│   │       ├── routes/     # v1/ (chat, openai, anthropic), auth, me, admin
│   │       ├── middleware/ # auth (JWT + API Key), require_admin
│   │       ├── state.rs    # AppState
│   │       └── error.rs    # ApiError
│   │
│   ├── auth/               # 认证 (Rust) — keygen, validator, jwt, password
│   │
│   ├── router/             # 路由引擎 (Rust) — key_scheduler (压力均衡), selector
│   │
│   ├── billing/            # 计费 (Rust) — subscription, plans
│   │
│   ├── models/             # 共享数据模型 (Rust) — user, provider, model, …
│   │
│   ├── db/                 # 数据库层 (Rust)
│   │   ├── src/            # postgres, redis, migrations
│   │   └── migrations/     # 001–017 SQL 迁移
│   │
│   └── adapters/           # Provider 客户端 (Rust crate: `provider_client`)
│       └── src/            # client (HTTP), browser_emulator, tool_calling, providers
│
├── docs/                   # 文档
│   ├── ARCHITECTURE.md
│   └── API.md
│
├── infra/                  # 部署
│   ├── k8s/
│   └── terraform/
│
├── docker-compose.yml
├── deploy.sh
└── scripts/                # setup.sh, start.sh, stop.sh
```

---

## 4. 模块职责说明

### 4.1 app/client/ — Web 客户端

**技术选型**：React + Vite + TypeScript (Web SPA)

| 特性 | 说明 |
|------|------|
| 跨平台 | 所有支持浏览器的设备（桌面 + 移动 Web） |
| 部署 | Nginx 静态文件服务 |
| 状态管理 | Zustand |
| 国际化 | 内置 i18n 支持 |

### 4.2 app/admin/ — 管理后台

**技术选型**：React + Vite + TypeScript

| 功能 | 说明 |
|------|------|
| 用户管理 | 列表、搜索、禁用、删除 |
| Provider 配置 | 接入/下线 Provider，调整优先级 |
| 模型管理 | 定价、上下文窗口、能力标签 |
| 交易记录 | 订阅购买、续费流水 |
| 实时监控 | QPS、延迟、错误率仪表盘 |

### 4.3 service/api/ — API 网关

**技术选型**：Rust + Axum

```
请求流程：
1. TLS 终止（由反向代理处理）
2. CORS 处理
3. 解析 Authorization Header (Bearer Token 或 x-api-key)
4. Rate Limit 检查 (Redis Sorted Set)
5. 认证中间件：API Key → JWT 回退
6. 订阅检查 + Token 配额检查
7. Key 调度器选择最优 API Key（压力均衡 + 会话亲和性）
8. Provider Client 直接 HTTP 调用上游 API
9. 响应格式转换（OpenAI / Anthropic 兼容）
10. 记录 API Log + 更新 Key 统计
11. 返回客户端
```

### 4.4 service/auth/ — 认证服务

```
用户注册 → bcrypt(密码) → PostgreSQL
用户登录 → 验证密码 → 签发 JWT
创建 Key → UUID → SHA256 哈希 → PostgreSQL
验证 Key → 哈希比对 → 通过/拒绝
JWT 登出 → Token 加入 Redis 黑名单
短信验证 → 阿里云 SMS → Redis 存储验证码
```

### 4.5 service/router/ — 路由引擎

核心算法：**压力均衡 Key 调度器**

```
特性：
- 每个 API Key 有独立的压力值（基于利用率和延迟）
- 压力越低（越空闲）的 Key 优先接收请求
- 会话亲和性：同一会话始终路由到同一 Key（10 分钟 TTL）
- TTL 过期后优先恢复之前的 Key 绑定
- Key 连续 3 次失败后标记为 unhealthy，5 分钟后自动恢复
- 压力随时间衰减，允许自然重新平衡
- 加权随机选择 top-3 最低压力 Key，避免惊群效应
```

### 4.6 service/billing/ — 计费服务

```
用户订阅 → 记录订阅方案和到期日
API 调用 → 检查订阅状态 (有效/到期/过期) → 放行或拒绝
Token 配额 → 按计费周期累计 → 超配额返回错误
```

**订阅方案：**
- ZeroToken：¥10/月，100K tokens/月，使用浏览器模拟器
- Monthly：$19.9/月，2M tokens/月
- Yearly：$199/年，2M tokens/月
- Team：$99/月，10M tokens/月
- Enterprise：定制，无限

### 4.7 service/adapters/ — Provider 客户端

**技术选型**：Rust (`provider_client` crate)

```rust
// 统一 trait
#[async_trait]
pub trait ProviderClient: Send + Sync {
    async fn chat(&self, request: ChatRequest) -> Result<ChatResponse, ProviderError>;
    async fn chat_stream(&self, request: ChatRequest) -> Result<Vec<ChatChunk>, ProviderError>;
    async fn embeddings(&self, request: EmbeddingsRequest) -> Result<EmbeddingsResponse, ProviderError>;
}
```

**两种客户端实现：**

| 客户端 | 用途 | 认证方式 |
|--------|------|---------|
| `HttpProviderClient` | 直接 HTTP 调用上游 API | API Key（Bearer / x-api-key / QueryKey） |
| `BrowserEmulator` | headless Chrome 模拟网页版 | 浏览器 Cookie/Session（ZeroToken） |

**协议适配：**
- OpenAI / DeepSeek：标准 OpenAI Chat Completions 格式
- Anthropic：自动转换 messages + system prompt，支持 Anthropic SSE 流式协议
- Google：自动转换为 Gemini contents/parts 格式，Google SSE 流式解析
- 自定义 Provider：通过 `CUSTOM_PROVIDERS` 环境变量动态注册

---

## 5. 数据库设计

### 5.1 ER 图

```
┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│   users     │────▶│  api_keys   │     │  providers  │
│             │     │             │     │             │
│ id          │     │ id          │     │ id          │
│ email       │     │ user_id     │────▶│ name        │
│ phone       │     │ key_hash    │     │ slug        │
│ password    │     │ is_active   │     │ is_active   │
│ subscription│     │ created_at  │     │ priority    │
│ plan        │     └─────────────┘     └──────┬──────┘
│ end_at      │          ┌─────────────┐       │
│ created_at  │          │  api_logs   │◀──────┘
└──────┬──────┘          │             │
       │                 │ id          │     ┌─────────────┐
       │                 │ user_id     │────▶│   models    │◀┐
       │                 │ provider_id │     │ id          │ │
       │                 │ model_id    │     │ provider_id │─┘
       │                 │ input_tokens│     │ name        │
       └────────────────▶│ output_tokens│    │ context_win │
                         │ latency_ms  │     └─────────────┘
                         │ created_at  │
                         └─────────────┘
```

### 5.2 表结构

| 表名 | 说明 |
|------|------|
| `users` | 用户账户、手机号、密码哈希、订阅方案、到期时间 |
| `api_keys` | 用户 API Key（SHA256 哈希存储） |
| `providers` | LLM 提供商（OpenAI、Anthropic、Google、DeepSeek） |
| `provider_keys` | 平台持有的 Provider API Key（加密存储） |
| `models` | 模型定义、上下文窗口、能力标签 |
| `api_logs` | 每次 API 调用记录（token 数量、延迟） |
| `subscriptions` | 订阅记录（方案、状态、起止时间） |
| `transactions` | 交易记录（购买、续费、退款） |
| `browser_accounts` | ZeroToken 浏览器账户（加密会话数据） |

---

## 6. API 设计

详见 [API.md](./API.md)。

### 6.1 核心端点

| 方法 | 路径 | 说明 |
|------|------|------|
| `POST` | `/v1/chat/completions` | 统一聊天（流式 + 非流式） |
| `POST` | `/v1/chat/batch` | 多模型并行查询 |
| `POST` | `/v1/chat/batch/judge` | LLM-as-Judge 评分 |
| `POST` | `/v1/embeddings` | 向量嵌入 |
| `GET` | `/v1/models` | 模型列表 |
| `POST` | `/v1/openai/chat/completions` | OpenAI SDK 兼容 |
| `POST` | `/v1/anthropic/messages` | Anthropic SDK 兼容 |
| `POST` | `/v1/auth/register` | 注册 |
| `POST` | `/v1/auth/login` | 登录 |
| `GET/POST` | `/v1/me/*` | 订阅、用量、Key 管理 |
| `*` | `/admin/*` | 管理后台 API |

---

## 7. 部署架构

### 7.1 Docker Compose（开发/单机）

```yaml
services:
  nexus-service:  # Rust API Gateway（单体，包含所有后端逻辑）
  nexus-admin:    # React 管理后台 (Nginx)
  nexus-client:   # React Web 客户端 (Nginx)
  postgres:       # PostgreSQL 16
  redis:          # Redis 7
```

详见项目根目录 `docker-compose.yml`。

### 7.2 Kubernetes 生产架构

```
┌─────────────────────────────────────────────────────────────┐
│                    Kubernetes Cluster                          │
│                                                              │
│  ┌─────────────┐  ┌─────────────┐                           │
│  │ nexus-service│  │ nexus-admin │                           │
│  │  (3 pods)   │  │  (2 pods)   │                           │
│  └──────┬──────┘  └──────┬──────┘                           │
│         │                │                                   │
│  ┌──────┴────────────────┴──────┐                           │
│  │   LoadBalancer / Ingress      │                           │
│  └──────────────────────────────┘                           │
│                                                              │
│  ┌──────────┐  ┌──────────┐                                 │
│  │postgres  │  │  redis   │                                 │
│  │(RDS)     │  │(ElastiCache)│                              │
│  └──────────┘  └──────────┘                                 │
└─────────────────────────────────────────────────────────────┘
```

> Kubernetes 部署配置位于 `infra/k8s/`，Terraform 基础设施代码位于 `infra/terraform/`。

---

## 8. 技术栈总结

| 层级 | 技术 | 位置 |
|------|------|------|
| **用户端** | React + Vite + TypeScript | `app/client/` |
| **管理后台** | React + Vite + TypeScript | `app/admin/` |
| **API 网关** | Rust + Axum 0.7 | `service/api/` |
| **认证服务** | Rust (JWT + bcrypt) | `service/auth/` |
| **路由引擎** | Rust (压力均衡 Key 调度) | `service/router/` |
| **计费服务** | Rust | `service/billing/` |
| **共享模型** | Rust | `service/models/` |
| **数据库层** | PostgreSQL 16 + Redis 7 | `service/db/` |
| **Provider 客户端** | Rust (HTTP + headless Chrome) | `service/adapters/` |
| **基础设施** | Docker Compose + Kubernetes + Terraform | `infra/` |

---

## 9. 开发优先级

| 阶段 | 内容 | 状态 |
|------|------|------|
| **Phase 1** | 核心服务（API + Auth + Router + Billing + DB） | ✅ 已完成 |
| **Phase 2** | Provider 客户端（OpenAI + Anthropic + Google + DeepSeek） | ✅ 已完成 |
| **Phase 3** | Web 客户端（聊天 + 模型选择） | ✅ 已完成 |
| **Phase 4** | 管理后台 | ✅ 已完成 |
| **Phase 5** | CI/CD、监控、测试完善 | 🔄 进行中 |

---

## 10. 安全考虑

| 安全点 | 实现方式 |
|--------|---------|
| **用户 API Key 存储** | SHA256 哈希，不存明文 |
| **Provider API Key 存储** | AES-256-GCM 加密 + Base64 编码 |
| **传输安全** | HTTPS/TLS 1.3（由反向代理处理） |
| **Rate Limit** | Redis Sorted Set 分布式计数 |
| **SQL 注入** | sqlx 预处理查询 |
| **Key 泄露** | 支持 Key 撤销和重新生成 |
| **JWT 登出** | Token 加入 Redis 黑名单 |
| **密码存储** | bcrypt 哈希 |

---

*文档版本: v2.0*
*最后更新: 2026-05-08*
