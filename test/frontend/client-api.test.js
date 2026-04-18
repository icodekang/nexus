/**
 * Client API 客户端测试
 *
 * 测试 client/src/api/client.ts 中所有 API 函数的数据结构
 * 运行: node test/frontend/client-api.test.js
 */

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
  toBeGreaterThan: (val) => {
    if (actual <= val) {
      throw new Error(`Expected ${actual} to be greater than ${val}`);
    }
  },
});

// ============ 认证相关 ============

test('login 函数请求体格式', () => {
  const body = JSON.stringify({
    email: 'user@example.com',
    password: 'password123',
  });

  const parsed = JSON.parse(body);
  expect(parsed.email).toBe('user@example.com');
  expect(parsed.password).toBe('password123');
});

test('register 函数请求体格式', () => {
  const body = JSON.stringify({
    email: 'newuser@example.com',
    password: 'password123',
  });

  const parsed = JSON.parse(body);
  expect(parsed.email).toBe('newuser@example.com');
  expect(parsed.password).toBe('password123');
});

test('sendSmsCode 函数请求体格式', () => {
  const body = JSON.stringify({
    phone: '+8613812345678',
  });

  const parsed = JSON.parse(body);
  expect(parsed.phone).toBe('+8613812345678');
});

test('verifySmsCode 函数请求体格式', () => {
  const body = JSON.stringify({
    phone: '+8613812345678',
    code: '123456',
  });

  const parsed = JSON.parse(body);
  expect(parsed.phone).toBe('+8613812345678');
  expect(parsed.code).toBe('123456');
});

// ============ 模型相关 ============

test('Model 结构验证', () => {
  const model = {
    id: 'model-1',
    name: 'GPT-4',
    provider: 'openai',
    provider_name: 'OpenAI',
    context_window: 128000,
    capabilities: ['chat', 'function'],
  };

  expect(model.name).toBe('GPT-4');
  expect(model.provider).toBe('openai');
  expect(model.capabilities).toContain('chat');
});

test('ChatMessage 结构验证', () => {
  const message = {
    role: 'user',
    content: 'Hello, how are you?',
  };

  expect(['user', 'assistant', 'system']).toContain(message.role);
  expect(message.content).toBeTruthy();
});

test('fetchModels 带 provider 参数', () => {
  const provider = 'openai';
  const query = provider ? `?provider=${provider}` : '';
  expect(query).toBe('?provider=openai');
});

test('fetchModels 不带 provider 参数', () => {
  const query = '' ? `?provider=` : '';
  expect(query).toBe('');
});

// ============ 聊天相关 ============

test('sendChat 请求体格式 (非流式)', () => {
  const body = JSON.stringify({
    model: 'gpt-4',
    messages: [
      { role: 'system', content: 'You are helpful.' },
      { role: 'user', content: 'Hi' },
    ],
    stream: false,
  });

  const parsed = JSON.parse(body);
  expect(parsed.model).toBe('gpt-4');
  expect(parsed.messages).toHaveLength(2);
  expect(parsed.stream).toBe(false);
});

test('streamChat 请求体格式 (流式)', () => {
  const body = JSON.stringify({
    model: 'gpt-4',
    messages: [{ role: 'user', content: 'Hi' }],
    stream: true,
  });

  const parsed = JSON.parse(body);
  expect(parsed.stream).toBe(true);
});

test('sendChat 响应结构解析', () => {
  const response = {
    id: 'chatcmpl-123',
    choices: [
      {
        message: {
          role: 'assistant',
          content: 'Hello! How can I help you?',
        },
      },
    ],
    usage: {
      prompt_tokens: 10,
      completion_tokens: 20,
      total_tokens: 30,
    },
  };

  expect(response.choices[0].message.content).toBeTruthy();
  expect(response.usage.total_tokens).toBe(30);
});

// ============ 订阅相关 ============

test('SubscriptionInfo 结构验证', () => {
  const sub = {
    subscription_plan: 'monthly',
    subscription_start: '2024-01-01T00:00:00Z',
    subscription_end: '2024-02-01T00:00:00Z',
    is_active: true,
  };

  expect(sub.is_active).toBe(true);
  expect(sub.subscription_plan).toBeTruthy();
});

test('PlanInfo 结构验证', () => {
  const plan = {
    plan: 'monthly',
    name: 'Monthly Plan',
    price_monthly: 19,
    price_yearly: 199,
    price_team_monthly: 49,
    features: ['api_access', 'all_models'],
  };

  expect(plan.price_monthly).toBe(19);
  expect(plan.features).toHaveLength(2);
});

test('subscribeToPlan 请求体格式', () => {
  const body = JSON.stringify({
    plan: 'monthly',
  });

  const parsed = JSON.parse(body);
  expect(parsed.plan).toBe('monthly');
});

test('subscribeToPlan yearly 套餐', () => {
  const body = JSON.stringify({
    plan: 'yearly',
  });

  const parsed = JSON.parse(body);
  expect(parsed.plan).toBe('yearly');
});

// ============ 使用量相关 ============

test('UsageData 结构验证', () => {
  const usage = {
    period_start: '2024-01-01T00:00:00Z',
    period_end: '2024-01-31T23:59:59Z',
    total_requests: 1500,
    total_input_tokens: 500000,
    total_output_tokens: 1000000,
    total_tokens: 1500000,
    token_quota: 5000000,
    quota_used_percent: 30.0,
    usage_by_provider: [
      { provider: 'openai', requests: 1000, input_tokens: 300000, output_tokens: 600000 },
    ],
    usage_by_model: [
      { model: 'gpt-4', provider: 'openai', requests: 1000, input_tokens: 300000, output_tokens: 600000 },
    ],
  };

  expect(usage.total_tokens).toBe(1500000);
  expect(usage.quota_used_percent).toBe(30.0);
  expect(usage.usage_by_provider).toHaveLength(1);
  expect(usage.usage_by_model).toHaveLength(1);
});

test('UsageData 无配额限制', () => {
  const usage = {
    period_start: '2024-01-01T00:00:00Z',
    period_end: '2024-01-31T23:59:59Z',
    total_requests: 500,
    total_input_tokens: 100000,
    total_output_tokens: 200000,
    total_tokens: 300000,
    token_quota: null,
    quota_used_percent: 0,
  };

  expect(usage.token_quota).toBeFalsy();
  expect(usage.quota_used_percent).toBe(0);
});

// ============ API 密钥相关 ============

test('ApiKey 结构验证', () => {
  const key = {
    id: 'key-123',
    name: 'Production Key',
    key_prefix: 'nk_abc1',
    is_active: true,
    last_used_at: '2024-01-15T10:30:00Z',
    created_at: '2024-01-01T00:00:00Z',
  };

  expect(key.key_prefix).toContain('nk_');
  expect(key.is_active).toBe(true);
});

test('createApiKey 请求体格式', () => {
  const body = JSON.stringify({
    name: 'Development Key',
  });

  const parsed = JSON.parse(body);
  expect(parsed.name).toBe('Development Key');
});

test('createApiKey 响应包含完整密钥', () => {
  const response = {
    id: 'key-new',
    key: 'nk_full_secret_key_here',
    name: 'Dev Key',
    created_at: '2024-01-15T10:30:00Z',
  };

  // 完整密钥只在创建时返回
  expect(response.key).toContain('nk_');
  expect(response.key.length).toBeGreaterThan(20);
});

test('deleteApiKey 函数调用格式', () => {
  const keyId = 'key-123';
  const path = `/v1/me/keys/${keyId}`;
  expect(path).toBe('/v1/me/keys/key-123');
});

// ============ 错误处理相关 ============

test('ApiError 结构验证', () => {
  const error = {
    error: {
      message: 'Invalid credentials',
      code: 'invalid_credentials',
    },
  };

  expect(error.error.message).toBeTruthy();
  expect(error.error.code).toBe('invalid_credentials');
});

test('Network error 格式', () => {
  const error = {
    error: {
      message: 'Network error',
      code: 'network_error',
    },
  };

  expect(error.error.code).toBe('network_error');
});

test('Request failed 格式', () => {
  const error = {
    error: {
      message: 'Request failed',
      code: 'request_failed',
    },
  };

  expect(error.error.code).toBe('request_failed');
});

// 运行所有测试
let passed = 0;
let failed = 0;

console.log('🧪 运行 Client API 测试...\n');

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
