/**
 * @file Client 端 API 客户端
 * 提供客户端用户操作后端服务的 HTTP 请求封装
 * 基于 REST API 与后端通信，自动处理 JWT 认证
 */

const API_BASE: string = import.meta.env.VITE_API_BASE ?? '';

export class ApiError extends Error {
  code: string;
  constructor(message: string, code: string) {
    super(message);
    this.name = 'ApiError';
    this.code = code;
  }
}

async function request<T>(path: string, options: RequestInit = {}): Promise<T> {
  const token = localStorage.getItem('nexus_token');
  const headers: Record<string, string> = {
    'Content-Type': 'application/json',
    ...(options.headers as Record<string, string> || {}),
  };
  if (token) {
    headers['Authorization'] = `Bearer ${token}`;
  }

  let res: Response;
  try {
    res = await fetch(`${API_BASE}${path}`, { ...options, headers });
  } catch {
    throw new ApiError('Network error', 'network_error');
  }

  if (!res.ok) {
    const err = await res.json().catch(() => ({ error: { message: 'Request failed', code: 'request_failed' } }));
    const message = err.error?.message || 'Request failed';
    const code = err.error?.code || 'request_failed';
    throw new ApiError(message, code);
  }

  return res.json();
}

// ─── Types ─────────────────────────────────────────────────────────

export interface Model {
  id: string;
  name: string;
  provider: string;
  provider_name: string;
  context_window: number;
  capabilities: string[];
  description: string | null;
  is_key_configured?: boolean;
}

export interface ChatMessage {
  role: 'user' | 'assistant' | 'system';
  content: string;
}

export interface ApiKey {
  id: string;
  name: string | null;
  key_prefix: string;
  is_active: boolean;
  last_used_at: string | null;
  created_at: string;
}

export interface UsageData {
  period_start: string;
  period_end: string;
  total_requests: number;
  total_input_tokens: number;
  total_output_tokens: number;
  total_tokens: number;
  balance: string;
  total_consumed: string;
  avg_latency_ms: number;
  usage_by_provider: Array<{ provider: string; requests: number; input_tokens: number; output_tokens: number }>;
  usage_by_model: Array<{ model: string; provider: string; requests: number; input_tokens: number; output_tokens: number }>;
}

export interface BalanceData {
  balance: string;
  total_purchased: string;
  total_consumed: string;
}

export interface ChargeItem {
  id: string;
  model: string;
  provider: string;
  input_tokens: number;
  output_tokens: number;
  total_cost: string;
  is_free: boolean;
  key_source: string;
  created_at: string;
}

export interface ChargesResponse {
  data: ChargeItem[];
  page: number;
  per_page: number;
}

export interface TokenPackage {
  id: string;
  name: string;
  credits: string;
  price: string;
  bonus_credits: string;
}

export interface ProviderKeyItem {
  id: string;
  provider_slug: string;
  name: string | null;
  api_key_prefix: string;
  is_active: boolean;
  priority_level: string;
  sort_order: number;
  always_use: boolean;
  created_at: string;
}

// ─── Auth ──────────────────────────────────────────────────────────

export async function login(email: string, password: string) {
  return request<{ token: string; user: { id: string; email: string; is_admin: boolean } }>('/v1/auth/login', {
    method: 'POST',
    body: JSON.stringify({ email, password }),
  });
}

export async function register(email: string, password: string) {
  return request<{ token: string; user: { id: string; email: string; is_admin: boolean } }>('/v1/auth/register', {
    method: 'POST',
    body: JSON.stringify({ email, password }),
  });
}

export async function sendSmsCode(phone: string) {
  return request<{ message: string; seconds_valid: number }>('/v1/auth/send-sms', {
    method: 'POST',
    body: JSON.stringify({ phone }),
  });
}

export async function verifySmsCode(phone: string, code: string) {
  return request<{ token: string; user: { id: string; phone: string; is_admin: boolean } }>('/v1/auth/verify-sms', {
    method: 'POST',
    body: JSON.stringify({ phone, code }),
  });
}

// ─── Models / Chat ──────────────────────────────────────────────────

export async function fetchModels(provider?: string) {
  const query = provider ? `?provider=${provider}` : '';
  return request<{ data: Model[] }>(`/v1/models${query}`);
}

export async function* streamChat(model: string, messages: ChatMessage[], sessionId?: string): AsyncGenerator<string> {
  const token = localStorage.getItem('nexus_token');
  const headers: Record<string, string> = { 'Content-Type': 'application/json' };
  if (token) headers['Authorization'] = `Bearer ${token}`;
  if (sessionId) headers['x-session-id'] = sessionId;

  const res = await fetch(`${API_BASE}/v1/chat/completions?stream=true`, {
    method: 'POST',
    headers,
    body: JSON.stringify({ model, messages, stream: true }),
  });

  if (!res.ok || !res.body) throw new ApiError('Stream request failed', 'stream_failed');

  const reader = res.body.getReader();
  const decoder = new TextDecoder();
  let buffer = '';

  while (true) {
    const { done, value } = await reader.read();
    if (done) break;
    buffer += decoder.decode(value, { stream: true });
    const lines = buffer.split('\n');
    buffer = lines.pop() || '';
    for (const line of lines) {
      if (line.startsWith('data: ')) {
        const data = line.slice(6).trim();
        if (data === '[DONE]') return;
        try {
          const parsed = JSON.parse(data);
          const content = parsed.choices?.[0]?.delta?.content;
          if (content) yield content;
        } catch { /* skip */ }
      }
    }
  }
}

export interface DailyUsageItem {
  day: string;
  input_tokens: number;
  output_tokens: number;
  total_cost: string;
}

export interface DailyModelUsage {
  day: string;
  models: Array<{ model: string; cost: string }>;
  total_cost: string;
}

// ─── Balance & Packages ──────────────────────────────────────────────

export async function fetchBalance() {
  return request<BalanceData>('/v1/me/balance');
}

export async function fetchUsage() {
  return request<UsageData>('/v1/me/usage');
}

export async function fetchDailyUsage() {
  return request<{ data: DailyUsageItem[] }>('/v1/me/usage/daily');
}

export async function fetchDailyModelUsage() {
  return request<{ data: DailyModelUsage[] }>('/v1/me/usage/daily/by-model');
}

export async function fetchCharges(page = 1, perPage = 20) {
  return request<ChargesResponse>(`/v1/me/charges?page=${page}&per_page=${perPage}`);
}

export async function fetchPackages() {
  return request<{ packages: TokenPackage[] }>('/v1/me/packages');
}

export async function purchasePackage(packageId: string) {
  return request<{ message: string; credits_added: string; balance: string }>('/v1/me/purchase', {
    method: 'POST',
    body: JSON.stringify({ package_id: packageId }),
  });
}

export async function recharge(amount: number) {
  return request<{ message: string; amount: number; balance: string }>('/v1/me/recharge', {
    method: 'POST',
    body: JSON.stringify({ amount }),
  });
}

export interface TransactionItem {
  id: string;
  type: string;
  amount: number;
  plan: string | null;
  status: string;
  description: string | null;
  created_at: string;
}

export async function fetchTransactions() {
  return request<{ data: TransactionItem[] }>('/v1/me/transactions');
}

// ─── API Keys ────────────────────────────────────────────────────────

export async function fetchApiKeys() {
  return request<{ data: ApiKey[] }>('/v1/me/keys');
}

export async function createApiKey(name: string) {
  return request<{ id: string; key: string; name: string; created_at: string }>('/v1/me/keys', {
    method: 'POST',
    body: JSON.stringify({ name }),
  });
}

export async function deleteApiKey(keyId: string) {
  return request<{ deleted: boolean }>(`/v1/me/keys/${keyId}`, { method: 'DELETE' });
}

// ─── BYOK Provider Keys ──────────────────────────────────────────────

export async function fetchProviderKeys() {
  return request<{ data: ProviderKeyItem[] }>('/v1/me/provider-keys');
}

export async function createProviderKey(data: {
  provider_slug: string;
  api_key: string;
  name?: string;
  base_url?: string;
  priority_level?: string;
  always_use?: boolean;
}) {
  return request<{ id: string; provider_slug: string; name: string | null; created_at: string }>('/v1/me/provider-keys', {
    method: 'POST',
    body: JSON.stringify(data),
  });
}

export async function updateProviderKey(keyId: string, data: {
  name?: string;
  is_active?: boolean;
  priority_level?: string;
  sort_order?: number;
  always_use?: boolean;
}) {
  return request<{ updated: boolean }>(`/v1/me/provider-keys/${keyId}`, {
    method: 'PUT',
    body: JSON.stringify(data),
  });
}

export async function deleteProviderKey(keyId: string) {
  return request<{ deleted: boolean }>(`/v1/me/provider-keys/${keyId}`, { method: 'DELETE' });
}
