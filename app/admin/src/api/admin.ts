/**
 * @file Admin 管理后台 API 客户端
 * 提供管理员操作后端服务的 HTTP 请求封装
 * 基于 REST API 与后端通信，自动处理 JWT 认证
 */

const API_BASE = import.meta.env.VITE_API_BASE || 'http://localhost:8080';

/**
 * request - 通用 HTTP 请求封装
 * @description 发送带认证的 JSON 请求，自动附加 Authorization header
 * @param path - API 路径（相对于 API_BASE）
 * @param options - Fetch 请求选项
 * @returns 解析后的 JSON 响应数据
 */
async function request<T>(path: string, options: RequestInit = {}): Promise<T> {
  const token = localStorage.getItem('nexus_admin_token');
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

// ============ Auth 认证相关 ============

/**
 * AuthResponse - 管理员登录响应
 * @description 包含 JWT token 和管理员用户信息
 */
export interface AuthResponse {
  token: string;
  user: {
    id: string;
    email: string;
    phone: string | null;
    subscription_plan: string;
    is_admin: boolean;
  };
}

/**
 * login - 管理员登录
 * @param email - 管理员邮箱
 * @param password - 密码
 * @returns 包含 token 和用户信息的响应
 */
export async function login(email: string, password: string): Promise<AuthResponse> {
  return request<AuthResponse>('/v1/auth/admin-login', {
    method: 'POST',
    body: JSON.stringify({ email, password }),
  });
}

// ============ Dashboard 仪表盘 ============

/**
 * DashboardStats - 仪表盘统计数据
 * @description 平台关键指标汇总
 */
export interface DashboardStats {
  total_users: number;
  active_subscriptions: number;
  total_revenue: number;
  api_calls_today: number;
}

/**
 * fetchDashboardStats - 获取仪表盘统计数据
 * @returns 平台用户数、订阅数、收入和 API 调用统计
 */
export async function fetchDashboardStats(): Promise<DashboardStats> {
  return request<DashboardStats>('/admin/dashboard/stats');
}

// ============ Users 用户管理 ============

/**
 * AdminUser - 管理员视图中的用户信息
 * @description 包含用户的完整信息，用于管理后台用户列表
 */
export interface AdminUser {
  id: string;
  email: string;
  phone: string | null;
  subscription_plan: string;
  is_admin: boolean;
  is_active: boolean;
  created_at: string;
  updated_at: string;
}

/**
 * UsersResponse - 用户列表响应
 * @description 包含用户数据数组和分页信息
 */
export interface UsersResponse {
  data: AdminUser[];
  total: number;
  page: number;
  per_page: number;
}

/**
 * fetchUsers - 获取用户列表
 * @param page - 页码，默认 1
 * @param perPage - 每页数量，默认 20
 * @param search - 搜索关键词
 * @returns 用户列表和分页信息
 */
export async function fetchUsers(page = 1, perPage = 20, search = ''): Promise<UsersResponse> {
  const params = new URLSearchParams({ page: String(page), per_page: String(perPage) });
  if (search) params.set('search', search);
  return request<UsersResponse>(`/admin/users?${params}`);
}

/**
 * updateUser - 更新用户信息
 * @param id - 用户 ID
 * @param data - 要更新的字段（手机号、订阅套餐）
 * @returns 更新后的用户信息
 */
export async function updateUser(id: string, data: { phone?: string; subscription_plan?: string }): Promise<AdminUser> {
  return request<AdminUser>(`/admin/users/${id}`, {
    method: 'PUT',
    body: JSON.stringify(data),
  });
}

// ============ Providers LLM 服务商管理 ============

/**
 * AdminProvider - LLM 服务商配置信息
 * @description 包含服务商的基本配置和状态
 */
export interface AdminProvider {
  id: string;
  name: string;
  slug: string;
  logo_url: string | null;
  api_base_url: string;
  is_active: boolean;
  priority: number;
  created_at: string;
}

/**
 * ProvidersResponse - 服务商列表响应
 */
export interface ProvidersResponse {
  data: AdminProvider[];
}

/**
 * fetchProviders - 获取所有 LLM 服务商列表
 * @returns 服务商配置列表
 */
export async function fetchProviders(): Promise<ProvidersResponse> {
  return request<ProvidersResponse>('/admin/providers');
}

/**
 * createProvider - 创建新的 LLM 服务商
 * @param data - 服务商配置（名称、slug、API 基础 URL、优先级）
 * @returns 创建的服务商信息
 */
export async function createProvider(data: { name: string; slug: string; api_base_url?: string; priority?: number; is_active?: boolean }): Promise<AdminProvider> {
  return request<AdminProvider>('/admin/providers', {
    method: 'POST',
    body: JSON.stringify(data),
  });
}

/**
 * updateProvider - 更新服务商配置
 * @param id - 服务商 ID
 * @param data - 要更新的字段
 * @returns 更新是否成功
 */
export async function updateProvider(id: string, data: { name?: string; slug?: string; api_base_url?: string; is_active?: boolean; priority?: number }): Promise<{ updated: boolean }> {
  return request<{ updated: boolean }>(`/admin/providers/${id}`, {
    method: 'PUT',
    body: JSON.stringify(data),
  });
}

/**
 * deleteProvider - 删除服务商
 * @param id - 服务商 ID
 * @returns 删除是否成功
 */
export async function deleteProvider(id: string): Promise<{ deleted: boolean }> {
  return request<{ deleted: boolean }>(`/admin/providers/${id}`, {
    method: 'DELETE',
  });
}

// ============ Models 模型管理 ============

/**
 * AdminModel - 模型配置信息
 * @description 包含模型的配置、能力描述和状态
 */
export interface AdminModel {
  id: string;
  provider_id: string;
  name: string;
  slug: string;
  model_id: string;
  mode: string;
  context_window: number;
  capabilities: string[];
  is_active: boolean;
  created_at: string;
}

/**
 * ModelsResponse - 模型列表响应
 */
export interface ModelsResponse {
  data: AdminModel[];
}

/**
 * fetchModels - 获取所有模型列表
 * @returns 模型配置列表
 */
export async function fetchModels(): Promise<ModelsResponse> {
  return request<ModelsResponse>('/admin/models');
}

/**
 * createModel - 创建新模型
 * @param data - 模型配置（服务商 ID、名称、slug、模型 ID 等）
 * @returns 创建的模型信息
 */
export async function createModel(data: { provider_id: string; name: string; slug: string; model_id: string; mode?: string; context_window?: number; capabilities?: string[] }): Promise<AdminModel> {
  return request<AdminModel>('/admin/models', {
    method: 'POST',
    body: JSON.stringify(data),
  });
}

/**
 * updateModel - 更新模型配置
 * @param id - 模型 ID
 * @param data - 要更新的字段
 * @returns 更新是否成功
 */
export async function updateModel(id: string, data: { name?: string; slug?: string; model_id?: string; provider_id?: string; context_window?: number; capabilities?: string[]; is_active?: boolean }): Promise<{ updated: boolean }> {
  return request<{ updated: boolean }>(`/admin/models/${id}`, {
    method: 'PUT',
    body: JSON.stringify(data),
  });
}

/**
 * deleteModel - 删除模型
 * @param id - 模型 ID
 * @returns 删除是否成功
 */
export async function deleteModel(id: string): Promise<{ deleted: boolean }> {
  return request<{ deleted: boolean }>(`/admin/models/${id}`, {
    method: 'DELETE',
  });
}

// ============ Provider Keys 服务商密钥管理 ============

/**
 * ProviderKey - 服务商 API 密钥信息
 * @description 包含密钥的掩码信息、状态和优先级
 */
export interface ProviderKey {
  id: string;
  provider_slug: string;
  api_key_masked: string;
  api_key_preview: string;
  base_url: string;
  is_active: boolean;
  priority: number;
  created_at: string;
  updated_at: string;
}

/**
 * ProviderKeysResponse - 密钥列表响应
 */
export interface ProviderKeysResponse {
  data: ProviderKey[];
}

/**
 * fetchProviderKeys - 获取所有服务商密钥列表
 * @returns 密钥列表
 */
export async function fetchProviderKeys(): Promise<ProviderKeysResponse> {
  return request<ProviderKeysResponse>('/admin/provider-keys');
}

/**
 * createProviderKey - 创建新的服务商密钥
 * @param data - 密钥配置（服务商 slug、API 密钥、基础 URL、优先级）
 * @returns 创建的密钥信息
 */
export async function createProviderKey(data: { provider_slug: string; api_key: string; base_url?: string; priority?: number }): Promise<ProviderKey> {
  return request<ProviderKey>('/admin/provider-keys', {
    method: 'POST',
    body: JSON.stringify(data),
  });
}

/**
 * updateProviderKey - 更新服务商密钥
 * @param id - 密钥 ID
 * @param data - 要更新的字段
 * @returns 更新是否成功
 */
export async function updateProviderKey(id: string, data: { api_key?: string; base_url?: string; is_active?: boolean; priority?: number }): Promise<{ updated: boolean }> {
  return request<{ updated: boolean }>(`/admin/provider-keys/${id}`, {
    method: 'PUT',
    body: JSON.stringify(data),
  });
}

/**
 * deleteProviderKey - 删除服务商密钥
 * @param id - 密钥 ID
 * @returns 删除是否成功
 */
export async function deleteProviderKey(id: string): Promise<{ deleted: boolean }> {
  return request<{ deleted: boolean }>(`/admin/provider-keys/${id}`, {
    method: 'DELETE',
  });
}

/**
 * testProviderKey - 测试服务商密钥连接
 * @param id - 密钥 ID
 * @returns 测试是否成功及消息
 */
export async function testProviderKey(id: string): Promise<{ success: boolean; message: string }> {
  return request<{ success: boolean; message: string }>(`/admin/provider-keys/${id}/test`, {
    method: 'POST',
  });
}

// ============ Transactions 交易记录 ============

/**
 * AdminTransaction - 交易记录信息
 * @description 包含交易的金额、类型、状态等信息
 */
export interface AdminTransaction {
  id: string;
  user_id: string;
  user_email: string;
  transaction_type: string;
  amount: number;
  plan: string | null;
  status: string;
  description: string | null;
  created_at: string;
}

/**
 * TransactionsResponse - 交易列表响应
 * @description 包含交易数据数组和分页信息
 */
export interface TransactionsResponse {
  data: AdminTransaction[];
  total: number;
  page: number;
  per_page: number;
}

/**
 * fetchTransactions - 获取交易记录列表
 * @param page - 页码，默认 1
 * @param perPage - 每页数量，默认 20
 * @param type - 交易类型过滤（purchase/refund/renewal）
 * @param status - 交易状态过滤（completed/refunded/pending）
 * @returns 交易列表和分页信息
 */
export async function fetchTransactions(page = 1, perPage = 20, type = '', status = ''): Promise<TransactionsResponse> {
  const params = new URLSearchParams({ page: String(page), per_page: String(perPage) });
  if (type) params.set('type', type);
  if (status) params.set('status', status);
  return request<TransactionsResponse>(`/admin/transactions?${params}`);
}

// ============ Browser Accounts 浏览器账号（ZeroToken）==========

/**
 * BrowserAccount - 浏览器账号信息
 * @description 包含第三方平台（Claude/ChatGPT）认证账号的状态和使用统计
 */
export interface BrowserAccount {
  id: string;
  provider: string;
  email: string | null;
  status: 'pending' | 'active' | 'expired' | 'error';
  request_count: number;
  last_used_at: string | null;
  created_at: string;
}

/**
 * QrCodeData - 二维码数据
 * @description 用于 ZeroToken 认证的二维码和会话信息
 */
export interface QrCodeData {
  session_id: string;
  qr_code_data: string;  // base64 PNG
  code: string;
  expires_at: string;
  auth_url: string;
}

/**
 * LoginUrlResponse - 登录 URL 响应
 * @description 无头浏览器登录流程返回的 URL 信息
 */
export interface LoginUrlResponse {
  account_id: string;
  login_url: string;
  code: string | null;
  expires_at: string | null;
  waiting: boolean;
}

/**
 * fetchBrowserAccounts - 获取所有浏览器账号列表
 * @returns 浏览器账号列表
 */
export async function fetchBrowserAccounts(): Promise<{ data: BrowserAccount[] }> {
  return request<{ data: BrowserAccount[] }>('/admin/accounts');
}

/**
 * createBrowserAccount - 创建新的浏览器账号会话
 * @param provider - 服务商名称（如 claude、chatgpt）
 * @returns 创建的浏览器账号信息
 */
export async function createBrowserAccount(provider: string): Promise<BrowserAccount> {
  return request<BrowserAccount>('/admin/accounts', {
    method: 'POST',
    body: JSON.stringify({ provider }),
  });
}

/**
 * deleteBrowserAccount - 删除浏览器账号
 * @param id - 账号 ID
 * @returns 删除是否成功
 */
export async function deleteBrowserAccount(id: string): Promise<{ deleted: boolean }> {
  return request<{ deleted: boolean }>(`/admin/accounts/${id}`, {
    method: 'DELETE',
  });
}

/**
 * generateQrCode - 为浏览器账号生成认证二维码
 * @param accountId - 账号 ID
 * @returns 二维码数据（base64 PNG）和会话信息
 */
export async function generateQrCode(accountId: string): Promise<QrCodeData> {
  return request<QrCodeData>(`/admin/accounts/${accountId}/qrcode`);
}

/**
 * startLogin - 启动无头浏览器登录流程
 * @param accountId - 账号 ID
 * @returns 登录 URL 和会话信息
 */
export async function startLogin(accountId: string): Promise<LoginUrlResponse> {
  return request<LoginUrlResponse>(`/admin/accounts/${accountId}/start-login`, {
    method: 'POST',
    body: JSON.stringify({ use_headless: true }),
  });
}

/**
 * getLoginUrl - 获取当前登录页面 URL
 * @param accountId - 账号 ID
 * @returns 当前登录 URL 和状态
 */
export async function getLoginUrl(accountId: string): Promise<LoginUrlResponse> {
  return request<LoginUrlResponse>(`/admin/accounts/${accountId}/login-url`);
}

/**
 * completeBrowserAuth - 完成浏览器认证流程
 * @param code - 认证码
 * @param sessionId - 会话 ID
 * @param sessionData - 会话数据
 * @param email - 可选的邮箱（用于账号绑定）
 * @returns 是否成功
 */
export async function completeBrowserAuth(code: string, sessionId: string, sessionData: string, email?: string): Promise<{ success: boolean }> {
  return request<{ success: boolean }>('/admin/accounts/complete-login', {
    method: 'POST',
    body: JSON.stringify({ code, session_id: sessionId, session_data: sessionData, email }),
  });
}
