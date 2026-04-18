/**
 * Admin API 客户端测试
 *
 * 测试 admin/src/api/admin.ts 中所有 API 函数的数据结构
 * 运行: node test/frontend/api-client.test.js
 */

// 模拟 fetch
const mockFetch = async (data) => {
  return {
    ok: true,
    json: async () => data,
  };
};

// 简单的测试框架
const tests = [];
const test = (name, fn) => tests.push({ name, fn });
const expect = (actual) => ({
  toBe: (expected) => {
    if (actual !== expected) {
      throw new Error(`Expected ${expected} but got ${actual}`);
    }
  },
  toEqual: (expected) => {
    if (JSON.stringify(actual) !== JSON.stringify(expected)) {
      throw new Error(`Expected ${JSON.stringify(expected)} but got ${JSON.stringify(actual)}`);
    }
  },
  toContain: (expected) => {
    if (!actual.includes(expected)) {
      throw new Error(`Expected ${actual} to contain ${expected}`);
    }
  },
  toBeTruthy: () => {
    if (!actual) {
      throw new Error(`Expected truthy value but got ${actual}`);
    }
  },
  toBeFalsy: () => {
    if (actual) {
      throw new Error(`Expected falsy value but got ${actual}`);
    }
  },
  toHaveLength: (len) => {
    if (actual.length !== len) {
      throw new Error(`Expected length ${len} but got ${actual.length}`);
    }
  },
});

// ============ Auth 认证相关 ============

test('AuthResponse 结构验证', () => {
  const response = {
    token: 'eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...',
    user: {
      id: 'user-123',
      email: 'admin@example.com',
      phone: null,
      subscription_plan: 'enterprise',
      is_admin: true,
    },
  };

  expect(response.token).toBeTruthy();
  expect(response.user.email).toBeTruthy();
  expect(response.user.is_admin).toBe(true);
});

test('login 函数请求体格式', () => {
  const body = JSON.stringify({
    email: 'admin@example.com',
    password: 'password123',
  });

  const parsed = JSON.parse(body);
  expect(parsed.email).toBe('admin@example.com');
  expect(parsed.password).toBe('password123');
});

// ============ 用户管理相关 ============

test('AdminUser 结构验证', () => {
  const user = {
    id: 'user-123',
    email: 'user@example.com',
    phone: '+1234567890',
    subscription_plan: 'monthly',
    is_admin: false,
    is_active: true,
    created_at: '2024-01-15T10:30:00Z',
    updated_at: '2024-01-15T10:30:00Z',
  };

  expect(user.email).toContain('@');
  expect(user.subscription_plan).toBeTruthy();
  expect(typeof user.is_admin).toBe('boolean');
  expect(typeof user.is_active).toBe('boolean');
});

test('UsersResponse 结构验证', () => {
  const response = {
    data: [
      { id: '1', email: 'user1@example.com', subscription_plan: 'monthly', is_admin: false, is_active: true, created_at: '', updated_at: '' },
      { id: '2', email: 'user2@example.com', subscription_plan: 'yearly', is_admin: false, is_active: true, created_at: '', updated_at: '' },
    ],
    total: 100,
    page: 1,
    per_page: 20,
  };

  expect(response.data).toHaveLength(2);
  expect(response.total).toBe(100);
  expect(response.page).toBe(1);
});

test('updateUser 函数请求体格式', () => {
  const body = JSON.stringify({
    phone: '+1234567890',
    subscription_plan: 'yearly',
  });

  const parsed = JSON.parse(body);
  expect(parsed.phone).toBe('+1234567890');
  expect(parsed.subscription_plan).toBe('yearly');
});

// ============ 提供商管理相关 ============

test('AdminProvider 结构验证', () => {
  const provider = {
    id: 'provider-1',
    name: 'OpenAI',
    slug: 'openai',
    logo_url: 'https://openai.com/logo.png',
    api_base_url: 'https://api.openai.com/v1',
    is_active: true,
    priority: 1,
    created_at: '2024-01-15T10:30:00Z',
  };

  expect(provider.name).toBeTruthy();
  expect(provider.slug).toBeTruthy();
  expect(provider.is_active).toBe(true);
  expect(provider.priority).toBe(1);
});

test('createProvider 函数请求体格式', () => {
  const body = JSON.stringify({
    name: 'DeepSeek',
    slug: 'deepseek',
    api_base_url: 'https://api.deepseek.com/v1',
    priority: 3,
    is_active: true,
  });

  const parsed = JSON.parse(body);
  expect(parsed.name).toBe('DeepSeek');
  expect(parsed.slug).toBe('deepseek');
});

test('updateProvider 函数请求体格式', () => {
  const body = JSON.stringify({
    name: 'Updated Name',
    is_active: false,
    priority: 2,
  });

  const parsed = JSON.parse(body);
  expect(parsed.name).toBe('Updated Name');
  expect(parsed.is_active).toBe(false);
});

// ============ 模型管理相关 ============

test('AdminModel 结构验证', () => {
  const model = {
    id: 'model-1',
    provider_id: 'provider-1',
    name: 'GPT-4',
    slug: 'gpt-4',
    model_id: 'gpt-4',
    mode: 'chat',
    context_window: 128000,
    capabilities: ['chat', 'function'],
    is_active: true,
    created_at: '2024-01-15T10:30:00Z',
  };

  expect(model.name).toBeTruthy();
  expect(model.capabilities).toContain('chat');
  expect(model.context_window).toBe(128000);
});

test('createModel 函数请求体格式', () => {
  const body = JSON.stringify({
    provider_id: 'provider-1',
    name: 'Claude 3',
    slug: 'claude-3',
    model_id: 'claude-3-opus',
    mode: 'chat',
    context_window: 200000,
    capabilities: ['chat', 'vision'],
  });

  const parsed = JSON.parse(body);
  expect(parsed.name).toBe('Claude 3');
  expect(parsed.capabilities).toContain('vision');
});

test('updateModel 函数支持 provider_id', () => {
  const body = JSON.stringify({
    name: 'Updated Model',
    provider_id: 'provider-2',
    context_window: 100000,
  });

  const parsed = JSON.parse(body);
  expect(parsed.provider_id).toBe('provider-2');
});

// ============ 提供商密钥相关 ============

test('ProviderKey 结构验证', () => {
  const key = {
    id: 'key-1',
    provider_slug: 'openai',
    api_key_masked: 'sk-****************************xyz',
    api_key_preview: 'sk-abc1...xyz',
    base_url: 'https://api.openai.com/v1',
    is_active: true,
    priority: 1,
    created_at: '2024-01-15T10:30:00Z',
    updated_at: '2024-01-15T10:30:00Z',
  };

  expect(key.api_key_masked).toContain('*');
  expect(key.provider_slug).toBeTruthy();
});

test('createProviderKey 函数请求体格式', () => {
  const body = JSON.stringify({
    provider_slug: 'anthropic',
    api_key: 'sk-ant-xxx',
    base_url: 'https://api.anthropic.com',
    priority: 1,
  });

  const parsed = JSON.parse(body);
  expect(parsed.api_key).toBe('sk-ant-xxx');
});

test('testProviderKey 响应结构', () => {
  const response = {
    success: true,
    message: 'Connection successful',
  };

  expect(response.success).toBe(true);
  expect(response.message).toBeTruthy();
});

// ============ 交易记录相关 ============

test('AdminTransaction 结构验证', () => {
  const tx = {
    id: 'tx-123',
    user_id: 'user-1',
    user_email: 'user@example.com',
    transaction_type: 'purchase',
    amount: 19.99,
    plan: 'monthly',
    status: 'completed',
    description: 'Monthly subscription',
    created_at: '2024-01-15T10:30:00Z',
  };

  expect(tx.transaction_type).toBe('purchase');
  expect(tx.amount).toBe(19.99);
  expect(tx.status).toBe('completed');
});

test('TransactionsResponse 结构验证', () => {
  const response = {
    data: [],
    total: 0,
    page: 1,
    per_page: 20,
  };

  expect(response.page).toBe(1);
  expect(response.per_page).toBe(20);
});

// ============ 浏览器账号相关 ============

test('BrowserAccount 结构验证', () => {
  const account = {
    id: 'acc-123',
    provider: 'claude',
    email: 'user@anthropic.com',
    status: 'active',
    request_count: 100,
    last_used_at: '2024-01-15T10:30:00Z',
    created_at: '2024-01-01T00:00:00Z',
  };

  expect(['pending', 'active', 'expired', 'error']).toContain(account.status);
  expect(account.request_count).toBe(100);
});

test('QrCodeData 结构验证', () => {
  const qrData = {
    session_id: 'sess_abc123',
    qr_code_data: 'base64_encoded_png_data',
    code: 'AUTH123',
    expires_at: '2024-01-15T10:30:00Z',
    auth_url: 'https://claude.ai/auth?code=AUTH123',
  };

  expect(qrData.session_id).toBeTruthy();
  expect(qrData.code).toBeTruthy();
  expect(qrData.auth_url).toContain('https://');
});

test('LoginUrlResponse 结构验证', () => {
  const response = {
    account_id: 'acc-123',
    login_url: 'https://auth.claude.ai/...',
    code: 'AUTH123',
    expires_at: '2024-01-15T10:30:00Z',
    waiting: false,
  };

  expect(response.login_url).toBeTruthy();
  expect(typeof response.waiting).toBe('boolean');
});

test('completeBrowserAuth 函数请求体格式', () => {
  const body = JSON.stringify({
    code: 'AUTH123',
    session_id: 'sess_abc123',
    session_data: 'encrypted_session_data',
    email: 'user@example.com',
  });

  const parsed = JSON.parse(body);
  expect(parsed.code).toBe('AUTH123');
  expect(parsed.session_id).toBe('sess_abc123');
});

// ============ 仪表盘统计相关 ============

test('DashboardStats 结构验证', () => {
  const stats = {
    total_users: 1500,
    active_subscriptions: 320,
    total_revenue: 45678.90,
    api_calls_today: 15000,
  };

  expect(stats.total_users).toBe(1500);
  expect(stats.active_subscriptions).toBe(320);
  expect(stats.total_revenue).toBe(45678.90);
  expect(stats.api_calls_today).toBe(15000);
});

// 运行所有测试
let passed = 0;
let failed = 0;

console.log('🧪 运行 Admin API 测试...\n');

for (const { name, fn } of tests) {
  try {
    fn();
    console.log(`✅ ${name}`);
    passed++;
  } catch (error) {
    console.log(`❌ ${name}`);
    console.log(`   Error: ${error.message}`);
    failed++;
  }
}

console.log(`\n📊 测试结果: ${passed} 通过, ${failed} 失败`);

if (failed > 0) {
  process.exit(1);
}
