const API_BASE = import.meta.env.VITE_API_BASE || 'http://localhost:8080';

async function request<T>(path: string, options: RequestInit = {}): Promise<T> {
  const token = localStorage.getItem('nexus_token');
  const headers: Record<string, string> = {
    'Content-Type': 'application/json',
    ...(options.headers as Record<string, string> || {}),
  };
  if (token) {
    headers['Authorization'] = `Bearer ${token}`;
  }

  const res = await fetch(`${API_BASE}${path}`, { ...options, headers });

  if (!res.ok) {
    const err = await res.json().catch(() => ({ error: { message: 'Request failed' } }));
    throw new Error(err.error?.message || 'Request failed');
  }

  return res.json();
}

export interface Model {
  id: string;
  name: string;
  provider: string;
  provider_name: string;
  context_window: number;
  capabilities: string[];
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
  token_quota: number | null;
  quota_used_percent: number;
  usage_by_provider: Array<{ provider: string; requests: number; input_tokens: number; output_tokens: number }>;
  usage_by_model: Array<{ model: string; provider: string; requests: number; input_tokens: number; output_tokens: number }>;
}

export interface SubscriptionInfo {
  subscription_plan: string;
  subscription_start: string | null;
  subscription_end: string | null;
  is_active: boolean;
}

export async function login(email: string, password: string) {
  return request<{ token: string; user: { id: string; email: string; subscription_plan: string } }>('/v1/auth/login', {
    method: 'POST',
    body: JSON.stringify({ email, password }),
  });
}

export async function register(email: string, password: string) {
  return request<{ token: string; user: { id: string; email: string; subscription_plan: string } }>('/v1/auth/register', {
    method: 'POST',
    body: JSON.stringify({ email, password }),
  });
}

export async function fetchModels(provider?: string) {
  const query = provider ? `?provider=${provider}` : '';
  return request<{ data: Model[] }>(`/v1/models${query}`);
}

export async function sendChat(model: string, messages: ChatMessage[]) {
  return request<{
    id: string;
    choices: Array<{ message: { role: string; content: string } }>;
    usage: { prompt_tokens: number; completion_tokens: number; total_tokens: number };
  }>('/v1/chat/completions', {
    method: 'POST',
    body: JSON.stringify({ model, messages, stream: false }),
  });
}

export async function* streamChat(model: string, messages: ChatMessage[]): AsyncGenerator<string> {
  const token = localStorage.getItem('nexus_token');
  const headers: Record<string, string> = {
    'Content-Type': 'application/json',
  };
  if (token) headers['Authorization'] = `Bearer ${token}`;

  const res = await fetch(`${API_BASE}/v1/chat/completions?stream=true`, {
    method: 'POST',
    headers,
    body: JSON.stringify({ model, messages, stream: true }),
  });

  if (!res.ok || !res.body) {
    throw new Error('Stream request failed');
  }

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
        } catch { /* skip malformed chunks */ }
      }
    }
  }
}

export async function fetchSubscription() {
  return request<SubscriptionInfo>('/v1/me/subscription');
}

export async function fetchUsage() {
  return request<UsageData>('/v1/me/usage');
}

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
  return request<{ deleted: boolean }>(`/v1/me/keys/${keyId}`, {
    method: 'DELETE',
  });
}

export async function subscribeToPlan(plan: string) {
  return request<{ message: string; plan: string; subscription_start: string; subscription_end: string }>('/v1/me/subscription', {
    method: 'POST',
    body: JSON.stringify({ plan }),
  });
}
