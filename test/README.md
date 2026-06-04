# Nexus 项目测试

本目录包含项目的所有单元测试和集成测试。

## 测试结构

```
test/
├── unit/
│   ├── models/              # Rust 模型单元测试
│   │   ├── user_test.rs
│   │   ├── provider_test.rs
│   │   ├── model_test.rs
│   │   ├── subscription_test.rs
│   │   └── mod.rs
│   ├── auth/               # Rust 认证模块测试
│   │   ├── password_test.rs
│   │   ├── jwt_test.rs
│   │   └── mod.rs
│   ├── api/                # API 数据结构测试
│   │   └── mod.rs
│   ├── adapters/          # 适配器模块测试
│   │   └── mod.rs
│   └── frontend/          # 前端测试
│       ├── api-client.test.js   # Admin API 测试
│       ├── client-api.test.js    # Client API 测试
│       ├── components.test.js    # Admin 组件测试
│       └── client-pages.test.js  # Client 页面测试
├── integration/
│   ├── backend_api_test.rs       # 后端 API 数据结构测试
│   ├── client_api_test.rs        # 客户端 API 数据结构测试
│   ├── sdk_test.py               # OpenAI/Anthropic SDK 集成测试
│   └── requirements.txt          # Python SDK 依赖
└── README.md
```

## 运行测试

### Rust 测试

```bash
cd service
cargo test
```

### 前端测试

```bash
# 运行所有前端测试
node test/unit/frontend/api-client.test.js
node test/unit/frontend/client-api.test.js
node test/unit/frontend/components.test.js
node test/unit/frontend/client-pages.test.js

# 或运行所有前端测试
for f in test/unit/frontend/*.test.js; do node "$f"; done
```

### OpenAI / Anthropic SDK 集成测试

测试通过 Nexus API Key 使用官方 OpenAI Python SDK 和 Anthropic Python SDK 访问大模型。

**前置条件：**
- Nexus 后端服务正在运行
- 已配置 Provider API Key（OPENAI_API_KEY / ANTHROPIC_API_KEY）
- 数据库中已注册模型

```bash
# 安装 Python 依赖
pip install -r test/integration/requirements.txt

# 运行 SDK 集成测试（需要 Nexus 服务运行）
python test/integration/sdk_test.py

# 自定义 Nexus 服务器地址
NEXUS_BASE_URL=http://localhost:8080 python test/integration/sdk_test.py

# 使用自定义测试账号
NEXUS_BASE_URL=http://localhost:8080 \
NEXUS_TEST_EMAIL=admin@nexus.dev \
NEXUS_TEST_PASSWORD=admin123 \
python test/integration/sdk_test.py

# 运行指定测试
python test/integration/sdk_test.py -k "OpenAI SDK"

# 列出所有测试用例
python test/integration/sdk_test.py --list
```

**测试内容：**
- API Key 创建、列表、验证
- OpenAI SDK 非流式 / 流式 Chat Completion
- OpenAI SDK 多轮对话、System Message
- Anthropic SDK 非流式 / 流式 Messages
- Anthropic SDK System Prompt、多轮对话
- Anthropic SDK 响应结构验证
- 跨 SDK 同模型访问

## 测试覆盖

### Rust 后端 (service/)

- [x] `models/` - 用户、提供商、模型、订阅数据模型
- [x] `auth/` - 密码哈希、JWT 认证
- [x] `api/` - API 请求/响应数据结构
- [x] `adapters/` - LLM 适配器配置和类型

### Admin 前端 (app/admin/)

- [x] API 客户端函数测试
- [x] 组件逻辑测试 (颜色映射、格式化函数)
- [x] 分页逻辑测试
- [x] i18n 翻译键测试
- [x] 浏览器账号状态测试
- [x] SSE 事件解析测试

### Client 前端 (app/client/)

- [x] API 客户端函数测试
- [x] 订阅套餐映射测试
- [x] 使用量统计格式化测试
- [x] 聊天消息测试
- [x] 流式响应解析测试
- [x] 登录表单验证测试
- [x] 路由状态测试

## 测试原则

1. **全面性** - 覆盖所有功能，不能遗漏任何小功能
2. **可执行性** - 测试必须能够实际运行
3. **独立性** - 每个测试用例应该独立运行
4. **可读性** - 测试名称清晰，描述测试目的
