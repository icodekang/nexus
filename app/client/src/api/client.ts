/**
 * @file Client 端 API 客户端
 * 提供客户端用户操作后端服务的 HTTP 请求封装
 * 基于 REST API 与后端通信，自动处理 JWT 认证
 */

const API_BASE: string = import.meta.env.VITE_API_BASE ?? '';

/**
 * ApiError - API 错误类
 * @description 扩展 Error 类，包含错误码用于 i18n 映射
 */
export class ApiError extends Error {
  code: string;
  constructor(message: string, code: string) {
    super(message);
    this.name = 'ApiError';
    this.code = code;
  }
}

/**
 * request - 通用 HTTP 请求封装
 * @description 发送带认证的 JSON 请求，自动附加 Authorization header
 * @param path - API 路径（相对于 API_BASE）
 * @param options - Fetch 请求选项
 * @returns 解析后的 JSON 响应数据
 */
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

/**
 * Model - 模型信息
 * @description 包含模型的基本信息和能力描述
 */
export interface Model {
  id: string;
  name: string;
  provider: string;
  provider_name: string;
  context_window: number;
  capabilities: string[];
}

/**
 * ChatMessage - 聊天消息
 * @description 对话中的单条消息
 */
export interface ChatMessage {
  role: 'user' | 'assistant' | 'system';
  content: string;
}

/**
 * ApiKey - API 密钥信息
 * @description 用户的 API 密钥（显示前缀，完整密钥仅创建时返回）
 */
export interface ApiKey {
  id: string;
  name: string | null;
  key_prefix: string;
  is_active: boolean;
  last_used_at: string | null;
  created_at: string;
}

/**
 * UsageData - 使用量统计数据
 * @description 包含周期内的 token 使用量和配额信息
 */
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

/**
 * SubscriptionInfo - 订阅信息
 * @description 用户的当前订阅状态和计划
 */
export interface SubscriptionInfo {
  subscription_plan: string;
  subscription_start: string | null;
  subscription_end: string | null;
  is_active: boolean;
}

/**
 * login - 邮箱登录
 * @param email - 邮箱地址
 * @param password - 密码
 * @returns token 和用户信息
 */
export async function login(email: string, password: string) {
  return request<{ token: string; user: { id: string; email: string; subscription_plan: string } }>('/v1/auth/login', {
    method: 'POST',
    body: JSON.stringify({ email, password }),
  });
}

/**
 * register - 用户注册
 * @param email - 邮箱地址
 * @param password - 密码
 * @returns token 和用户信息
 */
export async function register(email: string, password: string) {
  return request<{ token: string; user: { id: string; email: string; subscription_plan: string } }>('/v1/auth/register', {
    method: 'POST',
    body: JSON.stringify({ email, password }),
  });
}

/**
 * sendSmsCode - 发送短信验证码
 * @param phone - 手机号码
 * @returns 发送成功消息和验证码有效期
 */
export async function sendSmsCode(phone: string) {
  return request<{ message: string; seconds_valid: number }>('/v1/auth/send-sms', {
    method: 'POST',
    body: JSON.stringify({ phone }),
  });
}

/**
 * verifySmsCode - 验证短信验证码并登录
 * @param phone - 手机号码
 * @param code - 验证码
 * @returns token 和用户信息
 */
export async function verifySmsCode(phone: string, code: string) {
  return request<{ token: string; user: { id: string; phone: string; subscription_plan: string } }>('/v1/auth/verify-sms', {
    method: 'POST',
    body: JSON.stringify({ phone, code }),
  });
}

/**
 * fetchModels - 获取可用模型列表
 * @param provider - 可选，按服务商过滤
 * @returns 模型列表
 */
export async function fetchModels(provider?: string) {
  const query = provider ? `?provider=${provider}` : '';
  return request<{ data: Model[] }>(`/v1/models${query}`);
}

/**
 * sendChat - 发送聊天消息（非流式）
 * @param model - 模型 ID
 * @param messages - 消息历史
 * @param sessionId - 可选会话 ID
 * @returns AI 响应消息和使用量统计
 */
export async function sendChat(model: string, messages: ChatMessage[], sessionId?: string) {
  const headers: Record<string, string> = {};
  if (sessionId) headers['x-session-id'] = sessionId;
  return request<{
    id: string;
    choices: Array<{ message: { role: string; content: string } }>;
    usage: { prompt_tokens: number; completion_tokens: number; total_tokens: number };
  }>('/v1/chat/completions', {
    method: 'POST',
    headers,
    body: JSON.stringify({ model, messages, stream: false }),
  });
}

/**
 * streamChat - 发送聊天消息（流式）
 * @description 使用 Server-Sent Events (SSE) 流式接收 AI 响应
 * @param model - 模型 ID
 * @param messages - 消息历史
 * @param sessionId - 可选会话 ID
 * @yields 逐步返回 AI 回复内容
 */
export async function* streamChat(model: string, messages: ChatMessage[], sessionId?: string): AsyncGenerator<string> {
  const token = localStorage.getItem('nexus_token');
  const headers: Record<string, string> = {
    'Content-Type': 'application/json',
  };
  if (token) headers['Authorization'] = `Bearer ${token}`;
  if (sessionId) headers['x-session-id'] = sessionId;

  const res = await fetch(`${API_BASE}/v1/chat/completions?stream=true`, {
    method: 'POST',
    headers,
    body: JSON.stringify({ model, messages, stream: true }),
  });

  if (!res.ok || !res.body) {
    throw new ApiError('Stream request failed', 'stream_failed');
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

/**
 * fetchSubscription - 获取当前订阅信息
 * @returns 订阅计划和状态
 */
export async function fetchSubscription() {
  return request<SubscriptionInfo>('/v1/me/subscription');
}

/**
 * PlanInfo - 套餐信息
 */
export interface PlanInfo {
  plan: string;
  name: string;
  price_monthly: number;
  price_yearly: number;
  price_team_monthly: number;
  features: string[];
}

/**
 * fetchPlans - 获取可用订阅套餐列表
 * @returns 套餐列表
 */
export async function fetchPlans() {
  return request<{ plans: PlanInfo[] }>('/v1/me/subscription/plans');
}

/**
 * fetchUsage - 获取使用量统计
 * @returns 周期内的 token 使用量和配额信息
 */
export async function fetchUsage() {
  return request<UsageData>('/v1/me/usage');
}

/**
 * fetchApiKeys - 获取用户的 API 密钥列表
 * @returns API 密钥列表
 */
export async function fetchApiKeys() {
  return request<{ data: ApiKey[] }>('/v1/me/keys');
}

/**
 * createApiKey - 创建新的 API 密钥
 * @param name - 密钥名称（如：生产环境、开发环境）
 * @returns 新创建的密钥（完整密钥仅在此返回一次）
 */
export async function createApiKey(name: string) {
  return request<{ id: string; key: string; name: string; created_at: string }>('/v1/me/keys', {
    method: 'POST',
    body: JSON.stringify({ name }),
  });
}

/**
 * deleteApiKey - 删除 API 密钥
 * @param keyId - 密钥 ID
 * @returns 是否删除成功
 */
export async function deleteApiKey(keyId: string) {
  return request<{ deleted: boolean }>(`/v1/me/keys/${keyId}`, {
    method: 'DELETE',
  });
}

/**
 * subscribeToPlan - 订阅套餐
 * @param plan - 套餐名称（如：monthly、yearly）
 * @returns 订阅结果和新的订阅周期信息
 */
export async function subscribeToPlan(plan: string) {
  return request<{ message: string; plan: string; subscription_start: string; subscription_end: string }>('/v1/me/subscription', {
    method: 'POST',
    body: JSON.stringify({ plan }),
  });
}
