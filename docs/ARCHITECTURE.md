# nexus 系统架构设计文档

> 本文档描述 nexus 的技术架构，基于 OpenRouter 商业模式，使用 Rust 作为核心服务语言。

---

## 1. 产品定位与商业模式

### 1.1 产品定位

nexus 是一个统一的 LLM API 网关平台，类似 OpenRouter：

- **聚合多提供商**：通过统一 API 访问 100+ LLM 模型
- **智能路由**：自动选择最优 Provider（价格/延迟/质量）
- **订阅收费**：用户按月/年订阅，无限使用 API
- **跨平台客户端**：支持 Windows、macOS、Linux、iOS、Android、Web

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
| 月付 | $19.9/月 | 个人用户，轻度使用 |
| 年付 | $199/年 ($16.6/月) | 长期用户，节省 17% |
| 团队 | $99/月 | 小团队，5 个席位 |
| 企业 | 定制报价 | 大客户，私有部署 |

---

## 2. 系统架构图

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              Clients                                          │
│                                                                              │
│   ┌─────────────────────────────────────────────────────────────────────┐   │
│   │                      app/client/ (跨平台框架)                          │   │
│   │                                                                       │   │
│   │   ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌─────────┐              │   │
│   │   │ Windows │  │  macOS  │  │   iOS   │  │ Android │              │   │
│   │   │  App    │  │   App   │  │   App   │  │   App   │              │   │
│   │   └─────────┘  └─────────┘  └─────────┘  └─────────┘              │   │
│   │                                                                       │   │
│   │   ┌─────────────────────────────────────────────────────┐           │   │
│   │   │                    Web Client (SPA)                  │           │   │
│   │   └─────────────────────────────────────────────────────┘           │   │
│   └─────────────────────────────────────────────────────────────────────┘   │
│                                                                              │
│   ┌─────────────────────────────────────────────────────────────────────┐   │
│   │                      app/admin/ (管理后台)                              │   │
│   │                                                                       │   │
│   │   React + Vite 后台管理界面                                            │   │
│   │   用户管理 · Provider 配置 · 交易记录 · 系统监控                        │   │
│   └─────────────────────────────────────────────────────────────────────┘   │
│                                                                              │
└───────────────────────────────────┼───────────────────────────────────────────┘
                                    │ HTTPS / WSS
                                    ▼
┌───────────────────────────────────────────────────────────────────────────────┐
│                         service/ (后端 - Rust + Python)                        │
│                                                                                │
│   ┌───────────────────────────────────────────────────────────────────────┐  │
│   │                          service/api/ (API Gateway)                      │  │
│   │                          Rust · Axum · :443/:8080                        │  │
│   │                                                                        │  │
│   │   POST /v1/chat/completions    GET /v1/models    GET /v1/me/balance   │  │
│   │                                                                        │  │
│   │   ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐             │  │
│   │   │   Auth    │  │  Router  │  │ Billing  │  │  Models  │             │  │
│   │   │ service/  │  │ service/ │  │ service/ │  │ service/ │             │  │
│   │   │   auth/   │  │  router/ │  │ billing/ │  │  models/ │             │  │
│   │   └──────────┘  └──────────┘  └──────────┘  └──────────┘             │  │
│   └─────────────────────────────────────┬───────────────────────────────┘  │
│                                         │                                     │
│   ┌─────────────────────────────────────┴───────────────────────────────┐  │
│   │                          service/db/                                  │  │
│   │                     PostgreSQL · Redis · ClickHouse                    │  │
│   └─────────────────────────────────────────────────────────────────────┘  │
│                                                                                │
│                                         │ gRPC / HTTP                          │
└─────────────────────────────────────────┼─────────────────────────────────────┘
                                          │
          ┌───────────────────────────────┼───────────────────────────────┐
          │                               ▼                               │
          │  ┌─────────────────────────────────────────────────────────┐ │
          │  │              service/adapters/ (Python)                  │ │
          │  │                                                          │ │
          │  │   ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌────────┐ │ │
          │  │   │  OpenAI  │  │Anthropic │  │  Google  │  │DeepSeek│ │ │
          │  │   │ Adapter  │  │ Adapter  │  │ Adapter  │  │Adapter │ │ │
          │  │   └──────────┘  └──────────┘  └──────────┘  └────────┘ │ │
          │  └─────────────────────────────────────────────────────────┘ │
          └──────────────────────────────────────────────────────────────┘
```

---

## 3. 目录结构

```
nexus/
│
├── app/                              # 客户端应用
│   │
│   ├── client/                       # 🌟 用户端（跨平台框架）
│   │   │                               # 使用 React Native
│   │   │
│   │   ├── src/                      # React Native 代码
│   │   │   ├── pages/
│   │   │   │   ├── ChatScreen.tsx         # 主聊天页面
│   │   │   │   ├── ModelSelectScreen.tsx  # 模型选择 ⭐
│   │   │   │   ├── HistoryScreen.tsx      # 历史记录
│   │   │   │   └── SettingsScreen.tsx     # 设置
│   │   │   │
│   │   │   ├── components/
│   │   │   │   ├── ChatBubble.tsx        # 消息气泡
│   │   │   │   ├── ChatInput.tsx        # 输入框
│   │   │   │   ├── ModelCard.tsx        # 模型卡片
│   │   │   │   ├── ProviderFilter.tsx   # Provider 筛选
│   │   │   │   ├── CreditBadge.tsx      # 余额显示
│   │   │   └── Sidebar.tsx            # 侧边导航 (可隐藏)
│   │   │   │
│   │   │   ├── stores/               # 状态管理 (Zustand)
│   │   │   │   ├── chatStore.ts
│   │   │   │   ├── modelStore.ts
│   │   │   │   └── userStore.ts
│   │   │   │
│   │   │   └── api/
│   │   │       └── client.ts         # API 客户端
│   │   │
│   │   ├── ios/                     # iOS 原生配置
│   │   └── android/                  # Android 原生配置
│   │
│   └── admin/                        # 🌟 管理后台
│       ├── src/                      # React 前端
│       │   ├── pages/
│       │   │   ├── Dashboard.tsx      # 仪表盘
│       │   │   ├── Users.tsx          # 用户管理
│       │   │   ├── Providers.tsx      # Provider 管理
│       │   │   ├── Models.tsx         # 模型管理
│       │   │   ├── Transactions.tsx   # 交易记录
│       │   │   └── Settings.tsx       # 系统设置
│       │   │
│       │   └── components/
│       │       ├── Layout.tsx
│       │       ├── DataTable.tsx
│       │       └── Charts.tsx
│       │
│       ├── Dockerfile
│       └── nginx.conf
│
├── service/                          # 🌟 后端核心服务 (Rust + Python)
│   │
│   ├── Cargo.toml                    # Rust workspace 入口
│   ├── rust-toolchain.toml
│   ├── Dockerfile                    # API Gateway 镜像
│   │
│   ├── api/                         # 🌟 API 网关 (Rust Axum)
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── main.rs              # 入口
│   │       │
│   │       ├── routes/              # HTTP 路由
│   │       │   ├── mod.rs
│   │       │   ├── v1/
│   │       │   │   ├── mod.rs
│   │       │   │   ├── chat.rs       # POST /v1/chat/completions
│   │       │   │   ├── models.rs     # GET /v1/models
│   │       │   │   └── embeddings.rs # POST /v1/embeddings
│   │       │   │,
│   │       │   ├── auth.rs          # /v1/auth/*
│   │       │   └── me.rs            # /v1/me/*
│   │       │
│   │       ├── middleware/           # 中间件
│   │       │   ├── mod.rs
│   │       │   ├── auth.rs          # API Key 验证
│   │       │   ├── ratelimit.rs     # 限流
│   │       │   └── logging.rs       # 请求日志
│   │       │
│   │       └── error.rs              # 统一错误处理
│   │
│   ├── auth/                        # 🌟 认证服务 (Rust)
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── keygen.rs            # API Key 生成
│   │       ├── validator.rs         # Key 验证
│   │       └── jwt.rs               # JWT 处理
│   │
│   ├── router/                      # 🌟 路由引擎 (Rust)
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── selector.rs          # Provider 选择
│   │       ├── strategy/            # 路由策略
│   │       │   ├── mod.rs
│   │       │   ├── cheapest.rs      # 最便宜
│   │       │   ├── fastest.rs       # 最低延迟
│   │       │   ├── quality.rs       # 最高质量
│   │       │   └── balanced.rs      # 综合评分
│   │       │
│   │       ├── context.rs          # 路由上下文
│   │       └── fallback.rs          # 降级策略
│   │
│   ├── billing/                     # 🌟 计费服务 (Rust)
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── subscription.rs     # 订阅管理
│   │       ├── plans.rs           # 订阅方案
│   │       └── invoice.rs         # 账单生成
│   │
│   ├── models/                     # 🌟 共享数据模型 (Rust)
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── llm.rs              # LLM 模型定义
│   │       ├── provider.rs         # Provider 定义
│   │       ├── user.rs             # 用户模型
│   │       └── usage.rs            # 用量记录
│   │
│   ├── db/                         # 🌟 数据库层
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── postgres.rs
│   │       └── redis.rs
│   │
│   ├── adapters/                   # 🌟 Python Provider 适配器
│   │   ├── Dockerfile
│   │   ├── requirements.txt
│   │   └── adapter/
│   │       ├── main.py             # FastAPI 入口
│   │       ├── base.py            # 抽象基类
│   │       ├── registry.py        # Provider 注册表
│   │       ├── openai.py          # OpenAI 适配
│   │       ├── anthropic.py       # Anthropic 适配
│   │       ├── google.py          # Google Gemini 适配
│   │       └── deepseek.py        # DeepSeek 适配
│   │
│   └── db/
│       └── migrations/              # SQL 迁移
│           ├── 001_create_users.sql
│           ├── 002_create_api_keys.sql
│           ├── 003_create_providers.sql
│           ├── 004_create_models.sql
│           ├── 005_create_usage_logs.sql
│           └── 006_create_transactions.sql
│
├── docs/                           # 文档
│   ├── ARCHITECTURE.md             # 本文档
│   ├── API.md                      # API 参考（含 Python 示例）
│   └── DEPLOYMENT.md               # 部署指南
│
├── infra/                          # Kubernetes 部署配置
│   ├── k8s/
│   │   ├── api-gateway.yaml
│   │   ├── adapter.yaml
│   │   ├── postgres.yaml
│   │   ├── redis.yaml
│   │   └── admin.yaml
│   └── terraform/
│       ├── main.tf
│       └── variables.tf
│
└── docker-compose.yml               # 本地开发 / 生产部署
```

---

## 4. 模块职责说明

### 4.1 app/client/ — 跨平台客户端

**技术选型**：React Native

| 优势 | 说明 |
|------|------|
| 安装包小 | ~10MB vs Electron ~150MB |
| 原生性能 | 直接调用系统 API |
| 代码共享 | Windows/macOS/Linux 共用 95% 代码 |
| 安全 | Key 存储在系统 Keychain |

**跨平台支持**：
- Windows 10/11
- macOS 11+
- Linux (Debian/Ubuntu/Fedora)
- iOS / Android (通过 React Native)

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
1. TLS 终止
2. CORS 处理
3. 解析 Authorization Header (Bearer Token)
4. Rate Limit 检查 (Redis)
5. 路由到对应 Handler
6. 调用 Router 选择 Provider
7. 调用 Billing 预扣费
8. gRPC 转发给 Adapter
9. 响应转换
10. 返回客户端
```

### 4.4 service/auth/ — 认证服务

```
用户注册 → bcrypt(密码) → PostgreSQL
用户登录 → 验证密码 → 签发 JWT
创建 Key → UUID → SHA256 哈希 → PostgreSQL
验证 Key → 哈希比对 → 通过/拒绝
```

### 4.5 service/router/ — 路由引擎

```rust
// 路由策略
pub enum RouteStrategy {
    Cheapest,   // 按价格排序
    Fastest,    // 按延迟排序
    Quality,    // 按上下文窗口排序
    Balanced,   // 0.4*price + 0.4*latency + 0.2*quality
}

// 选择流程
async fn select_provider(model: &str, strategy: RouteStrategy) -> Provider {
    // 1. 查数据库：哪些 Provider 支持该模型
    // 2. 按策略排序
    // 3. 尝试第一个，失败则降级到下一个
}
```

### 4.6 service/billing/ — 计费服务

```
用户订阅 → 记录订阅方案和到期日 → Transaction 记录
API 调用 → 检查订阅状态 (有效/到期/过期) → 放行或拒绝
到期提醒 → 发送通知 → 引导续费
```

**订阅管理：**
- 支持月付、年付、团队等订阅方案
- 订阅状态：active / expired / cancelled
- 到期前 7 天提醒，到期后限制访问

### 4.7 service/adapters/ — Provider 适配器

**技术选型**：Python + FastAPI

```python
# 统一接口
class BaseAdapter:
    async def chat(self, request) -> ChatResponse
    async def chat_stream(self, request) -> AsyncIterator[ChatChunk]
    async def embeddings(self, request) -> EmbeddingsResponse

# Anthropic 协议差异处理
class AnthropicAdapter(BaseAdapter):
    # Anthropic 用 /v1/messages 而非 /v1/chat/completions
    # Anthropic 用 max_tokens 而非 max_tokens
    # Anthropic system prompt 单独字段
```

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
│ subscription│                                         │
│ end_at      │          ┌─────────────┐                │
│ created_at  │          │  api_logs   │◀───────────────┘
└──────┬──────┘          │             │
       │          │  id           │     ┌─────────────┐
       │          │  user_id      │────▶│   models    │◀┐
       │          │  key_id       │     │             │ │
       │          │  provider_id  │     │ id          │ │
       │          │  model_id     │     │ provider_id │─┘
       │          │  input_tokens │     │ name        │
       │          │  output_tokens│     │ context_win │
       └─────────▶│  latency_ms   │     └─────────────┘
                  │  created_at   │
                  └─────────────┘

       ┌─────────────┐
       │subscriptions│
       │             │
       │ id          │────▶ users
       │ user_id     │
       │ plan        │     (monthly / yearly / team)
       │ status      │     (active / expired / cancelled)
       │ start_at    │
       │ end_at      │
       │ created_at  │
       └─────────────┘
```

### 5.2 表结构

| 表名 | 说明 |
|------|------|
| `users` | 用户账户、手机号、密码哈希、订阅方案、到期时间 |
| `api_keys` | 用户 API Key（SHA256 哈希存储） |
| `providers` | LLM 提供商（OpenAI、Anthropic 等） |
| `models` | 模型定义、上下文窗口、能力标签 |
| `api_logs` | 每次 API 调用记录（token 数量、延迟） |
| `subscriptions` | 订阅记录（方案、状态、起止时间） |

---

## 6. API 设计

### 6.1 端点列表

| 方法 | 路径 | 说明 |
|------|------|------|
| `POST` | `/v1/chat/completions` | 聊天补全（支持流式） |
| `POST` | `/v1/completions` | 文本补全 |
| `POST` | `/v1/embeddings` | 向量嵌入 |
| `GET` | `/v1/models` | 模型列表 |
| `POST` | `/v1/auth/register` | 用户注册 |
| `POST` | `/v1/auth/login` | 用户登录 |
| `GET` | `/v1/me/subscription` | 订阅信息 |
| `POST` | `/v1/me/subscription` | 购买/续费订阅 |
| `GET` | `/v1/me/usage` | 用量统计 |
| `POST` | `/v1/me/keys` | 创建 API Key |
| `GET` | `/v1/me/keys` | 列出 Key |
| `DELETE` | `/v1/me/keys/:id` | 删除 Key |

### 6.2 请求示例

```bash
# 聊天请求
curl -X POST https://api.nexus.com/v1/chat/completions \
  -H "Authorization: Bearer sk-nova-xxxx" \
  -H "Content-Type: application/json" \
  -d '{
    "model": "gpt-4o",
    "messages": [{"role": "user", "content": "解释 Rust 的生命周期"}],
    "temperature": 0.7,
    "max_tokens": 1000
  }'

# 指定 Provider
{
  "model": "anthropic/claude-3-5-sonnet"
}

# 流式响应
{
  "model": "gpt-4o",
  "messages": [...],
  "stream": true
}
```

### 6.3 路由策略 Header

```
X-Route-Strategy: cheapest | fastest | quality | balanced
```

---

## 7. 客户端界面设计

> 设计理念：简洁、现代、移动优先，参考 ChatGPT / DeepSeek / Claude App 的优秀交互体验。

### 7.1 设计原则

| 原则 | 说明 |
|------|------|
| **移动优先** | 单手操作，侧边导航（手机隐藏），关键操作在拇指区 |
| **信息分层** | 重要信息（余额、模型）突出，次要信息折叠 |
| **流畅交互** | 消息发送流畅，打字时即时预览响应 |
| **视觉舒适** | 护眼配色，高对比度文字，适当留白 |

### 7.2 整体布局

```
┌─────────────────────────────────────────────────────────────┐
│  ☰  nexus                    [企业版 ▼] [⚙️] [👤]       │
│  [当前模型: GPT-4o ▼]                                       │
├────────┬────────────────────────────────────────────────────┤
│        │                                                    │
│        │                    聊天消息区域                      │
│  💬 聊天 │                    (上滑查看历史)                    │
│        │                                                    │
│  🤖 模型 │  ┌─────────────────────────────────────┐         │
│        │  │ 你好，我想了解 Rust 异步编程...       │         │
│  👤 我的 │  └─────────────────────────────────────┘         │
│        │                      用户消息 (右对齐, 蓝色气泡)        │
│        │                                                    │
│        │  ┌─────────────────────────────────────┐         │
│        │  │ Rust 异步编程主要使用 tokio 运行时..│         │
│        │  └─────────────────────────────────────┘         │
│        │              AI 助手消息 (左对齐, 灰色气泡)             │
│        │                                                    │
│        │  ┌─────────────────────────────────────┐         │
│        │  │ 正在思考...                         │         │
│        │  └─────────────────────────────────────┘         │
│        │              (流式输出时显示动画)                  │
│        │                                                    │
├────────┴────────────────────────────────────────────────────┤
│  [📎] [            请输入消息...         ] [⬆️ 发送]       │
│        附件按钮     输入框 (支持多行)      发送按钮            │
└─────────────────────────────────────────────────────────────┘

侧边栏行为：
┌─────────────────────────────────────────────────────────────┐
│  设备      │ 默认状态   │ 收起方式                           │
│------------│------------│------------------------------------│
│  手机端    │ 隐藏       │ 点击 ☰ 或左滑呼出                     │
│  电脑端    │ 显示       │ 可手动收起，点击外部自动收起            │
└─────────────────────────────────────────────────────────────┘
```

### 7.3 聊天页面

**顶部导航栏：**
- 左：App Logo + 名称
- 中：当前模型选择器（点击弹出模型选择）
- 右：订阅方案 + 设置图标 + 用户头像

**消息区域：**
- 用户消息：蓝色气泡，右对齐
- AI 消息：灰色气泡，左对齐，带 Avatar
- 流式输出：逐字显示，打字机效果
- 代码块：语法高亮，复制按钮
- 图片：支持展示（如果是 vision 模型）

**输入区域：**
- 附件按钮（图片、文件）
- 多行输入框（自动扩展）
- 发送按钮（可上传后激活）
- 长按消息：复制、重新生成、删除

### 7.4 模型选择页面

```
┌─────────────────────────────────────────────────────────────┐
│  ← 返回                    选择模型                            │
├─────────────────────────────────────────────────────────────┤
│  🔍 搜索模型...                                           │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  推荐                                                        │
│  ┌─────────────────────────────────────────────────────────┐ │
│  │ ⭐ GPT-4o                                     OpenAI    │ │
│  │    $2.50 / $10.00 per 1M · 128K context               │ │
│  │    [vision] [function_call]               [已选择 ✓]   │ │
│  └─────────────────────────────────────────────────────────┘ │
│                                                             │
│  OpenAI                                                     │
│  ┌─────────────────────────────────────────────────────────┐ │
│  │ 🤖 GPT-4o Mini                                   $0.15  │ │
│  │    最便宜 · 128K context · ⚡ 速度最快                 │ │
│  └─────────────────────────────────────────────────────────┘ │
│  ┌─────────────────────────────────────────────────────────┐ │
│  │ 🤖 GPT-4 Turbo                                 $10.00  │ │
│  │    高速 · 128K context                              │ │
│  └─────────────────────────────────────────────────────────┘ │
│                                                             │
│  Anthropic                                                  │
│  ┌─────────────────────────────────────────────────────────┐ │
│  │ 🧠 Claude 3.5 Sonnet                           $3.00   │ │
│  │    最高质量 · 200K context · 🌟 推荐                  │ │
│  └─────────────────────────────────────────────────────────┘ │
│  ┌─────────────────────────────────────────────────────────┐ │
│  │ 🧠 Claude 3 Opus                               $15.00  │ │
│  │    旗舰级 · 200K context                             │ │
│  └─────────────────────────────────────────────────────────┘ │
│                                                             │
│  DeepSeek                                                  │
│  ┌─────────────────────────────────────────────────────────┐ │
│  │ 🔴 DeepSeek V3                                 $0.07   │ │
│  │    💰 最低价格 · 64K context                          │ │
│  └─────────────────────────────────────────────────────────┘ │
│                                                             │
├─────────────────────────────────────────────────────────────┤
│  公司筛选: [全部] [OpenAI] [Anthropic] [Google] [DeepSeek]  │
└─────────────────────────────────────────────────────────────┘
```

**模型卡片信息层次：**
| 信息 | 样式 |
|------|------|
| 模型名称 + Provider Logo | 大字，加粗 |
| 输入/输出价格 | 醒目颜色（价格敏感） |
| 核心能力标签 | pills 样式（vision, function_call 等） |
| 特色说明 | 小字灰色（最便宜、最快、最高质量） |

### 7.5 我的页面（个人中心）

```
┌─────────────────────────────────────────────────────────────┐
│                          我的                                 │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│    ┌─────────────────────────────────────────────────┐     │
│    │              👤 用户头像                         │     │
│    │           user@example.com                      │     │
│    │         +86 138****8888                          │     │
│    └─────────────────────────────────────────────────┘     │
│                                                             │
│    ┌─────────────────────────────────────────────────┐     │
│    │  💰 我的订阅                                    │     │
│    │  ─────────────────────────────────────────────  │     │
│    │  企业版 · 到期: 2025-12-31        [续费] [升级] │     │
│    │  本月用量: 125,000 tokens                       │     │
│    └─────────────────────────────────────────────────┘     │
│                                                             │
│    ┌─────────────────────────────────────────────────┐     │
│    │  🔑 API Keys                        [创建新 Key] │     │
│    │  ─────────────────────────────────────────────  │     │
│    │  sk-nova-xxxx...8f3a    [复制] [删除]          │     │
│    │  sk-nova-xxxx...1b2c    [复制] [删除]          │     │
│    └─────────────────────────────────────────────────┘     │
│                                                             │
│    ┌─────────────────────────────────────────────────┐     │
│    │  📊 用量统计                                    │     │
│    │  ─────────────────────────────────────────────  │     │
│    │  本月: 125,000 tokens                           │     │
│    │  上月: 340,000 tokens                           │     │
│    └─────────────────────────────────────────────────┘     │
│                                                             │
│    ┌─────────────────────────────────────────────────┐     │
│    │  ⚙️ 设置                                        │     │
│    │  · 通知设置                                      │     │
│    │  · 隐私设置                                      │     │
│    │  · 主题模式 (浅色/深色/自动)                     │     │
│    │  · 关于我们                                      │     │
│    └─────────────────────────────────────────────────┘     │
│                                                             │
│    [退出登录]                                               │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

### 7.6 设计规范

**配色方案：**
| 用途 | 浅色模式 | 深色模式 |
|------|----------|----------|
| 背景 | #FFFFFF | #1A1A1A |
| 次背景 | #F5F5F7 | #2D2D2D |
| 主色调 | #10A37F (品牌绿) | #10A37F |
| 用户气泡 | #10A37F | #10A37F |
| AI 气泡 | #F5F5F7 | #2D2D2D |
| 文字主色 | #1D1D1F | #FFFFFF |
| 文字次色 | #86868B | #AEAEB2 |

**字体：**
- 主字体：系统默认 (San Francisco / Roboto)
- 代码：Monospace (SF Mono / Roboto Mono)

**间距：**
- 页面边距：16px
- 卡片间距：12px
- 元素内边距：12px-16px
- 圆角：12px (卡片) / 20px (气泡) / 24px (输入框)

**动效：**
- 页面切换：300ms ease-out
- 按钮点击：scale(0.98) + 100ms
- 消息出现：fade-in + slide-up 200ms
- 流式输出：逐字显示，无延迟

---

## 8. 部署架构

### 8.1 Docker Compose

```yaml
services:
  # API 网关 (Rust)
  api-gateway:
    build: ./service
    ports:
      - "443:443"
      - "8080:8080"
    environment:
      - DATABASE_URL=postgres://nova:password@postgres:5432/nova
      - REDIS_URL=redis://redis:6379
      - ADAPTER_URL=http://adapter:50051
    depends_on:
      - postgres
      - redis
      - adapter

  # Python 适配器
  adapter:
    build: ./service/adapters
    ports:
      - "50051:50051"

  # PostgreSQL
  postgres:
    image: postgres:16
    volumes:
      - pgdata:/var/lib/postgresql/data
      - ./service/db/migrations:/docker-entrypoint-initdb.d

  # Redis
  redis:
    image: redis:7-alpine

  # 管理后台
  admin:
    build: ./app/admin
    ports:
      - "3000:80"

volumes:
  pgdata:
```

> **Kubernetes 部署配置位于 `infra/k8s/`，Terraform 基础设施代码位于 `infra/terraform/`**

### 8.2 Kubernetes 生产架构

```
┌─────────────────────────────────────────────────────────────┐
│                    Kubernetes Cluster                          │
│                                                              │
│  ┌──────────────────────────────────────────────────────┐  │
│  │               app/client/ (React Native)               │  │
│  │           Windows · macOS · Linux · iOS · Android      │  │
│  └──────────────────────────────────────────────────────┘  │
│                                                              │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐         │
│  │ api-gateway │  │   adapter   │  │    admin    │         │
│  │  (3 pods)   │  │  (3 pods)   │  │  (2 pods)   │         │
│  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘         │
│         │                │                │                  │
│  ┌──────┴────────────────┴────────────────┴──────┐          │
│  │         LoadBalancer (Cloudflare + AWS)        │          │
│  └───────────────────────────────────────────────┘          │
│                                                              │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐                   │
│  │postgres  │  │  redis   │  │clickhouse│                   │
│  │(RDS)     │  │(ElastiCache)│ │(Managed) │                   │
│  └──────────┘  └──────────┘  └──────────┘                   │
└──────────────────────────────────────────────────────────────┘
```

---

## 9. 技术栈总结

| 层级 | 技术 | 位置 |
|------|------|------|
| **用户端** | React Native | `app/client/` |
| **管理后台** | React + Vite | `app/admin/` |
| **API 网关** | Rust + Axum | `service/api/` |
| **认证服务** | Rust | `service/auth/` |
| **路由引擎** | Rust | `service/router/` |
| **计费服务** | Rust | `service/billing/` |
| **共享模型** | Rust | `service/models/` |
| **数据库层** | PostgreSQL + Redis | `service/db/` |
| **Provider 适配** | Python + FastAPI | `service/adapters/` |
| **基础设施** | Kubernetes + Terraform | `infra/` |

---

## 10. 开发优先级

| 阶段 | 内容 | 目录 | 预计时间 |
|------|------|------|---------|
| **Phase 1** | 核心服务（API + Auth + Router + Billing + DB） | `service/` | 4-6 周 |
| **Phase 2** | Python Adapter（OpenAI + Anthropic + Google + DeepSeek） | `service/adapters/` | 1-2 周 |
| **Phase 3** | 用户端 React Native App（聊天 + 模型选择） | `app/client/` | 3-4 周 |
| **Phase 4** | 管理后台 React App | `app/admin/` | 2-3 周 |
| **Phase 5** | 监控、CI/CD | infra/ | 1-2 周 |

---

## 11. 安全考虑

| 安全点 | 实现方式 |
|--------|---------|
| **API Key 存储** | SHA256 哈希，不存明文 |
| **传输安全** | HTTPS/TLS 1.3 |
| **Rate Limit** | Redis 分布式计数 |
| **SQL 注入** | sqlx 预处理查询 |
| **Key 泄露** | 支持 Key 撤销 |
| **客户端 Key** | 存储在系统 Keychain |

---

*文档版本: v1.0*
*最后更新: 2026-04-08*
