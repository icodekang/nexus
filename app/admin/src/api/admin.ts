const API_BASE = import.meta.env.VITE_API_BASE || 'http://localhost:8080';

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

// ============ Auth ============

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

export async function login(email: string, password: string): Promise<AuthResponse> {
  return request<AuthResponse>('/v1/auth/admin-login', {
    method: 'POST',
    body: JSON.stringify({ email, password }),
  });
}

// ============ Dashboard ============

export interface DashboardStats {
  total_users: number;
  active_subscriptions: number;
  total_revenue: number;
  api_calls_today: number;
}

export async function fetchDashboardStats(): Promise<DashboardStats> {
  return request<DashboardStats>('/admin/dashboard/stats');
}

// ============ Users ============

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

export interface UsersResponse {
  data: AdminUser[];
  total: number;
  page: number;
  per_page: number;
}

export async function fetchUsers(page = 1, perPage = 20, search = ''): Promise<UsersResponse> {
  const params = new URLSearchParams({ page: String(page), per_page: String(perPage) });
  if (search) params.set('search', search);
  return request<UsersResponse>(`/admin/users?${params}`);
}

export async function updateUser(id: string, data: { phone?: string; subscription_plan?: string }): Promise<AdminUser> {
  return request<AdminUser>(`/admin/users/${id}`, {
    method: 'PUT',
    body: JSON.stringify(data),
  });
}

// ============ Providers ============

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

export interface ProvidersResponse {
  data: AdminProvider[];
}

export async function fetchProviders(): Promise<ProvidersResponse> {
  return request<ProvidersResponse>('/admin/providers');
}

export async function createProvider(data: { name: string; slug: string; api_base_url?: string; priority?: number }): Promise<AdminProvider> {
  return request<AdminProvider>('/admin/providers', {
    method: 'POST',
    body: JSON.stringify(data),
  });
}

export async function updateProvider(id: string, data: { name?: string; slug?: string; api_base_url?: string; is_active?: boolean; priority?: number }): Promise<{ updated: boolean }> {
  return request<{ updated: boolean }>(`/admin/providers/${id}`, {
    method: 'PUT',
    body: JSON.stringify(data),
  });
}

export async function deleteProvider(id: string): Promise<{ deleted: boolean }> {
  return request<{ deleted: boolean }>(`/admin/providers/${id}`, {
    method: 'DELETE',
  });
}

// ============ Models ============

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

export interface ModelsResponse {
  data: AdminModel[];
}

export async function fetchModels(): Promise<ModelsResponse> {
  return request<ModelsResponse>('/admin/models');
}

export async function createModel(data: { provider_id: string; name: string; slug: string; model_id: string; mode?: string; context_window?: number; capabilities?: string[] }): Promise<AdminModel> {
  return request<AdminModel>('/admin/models', {
    method: 'POST',
    body: JSON.stringify(data),
  });
}

export async function updateModel(id: string, data: { name?: string; slug?: string; model_id?: string; context_window?: number; capabilities?: string[]; is_active?: boolean }): Promise<{ updated: boolean }> {
  return request<{ updated: boolean }>(`/admin/models/${id}`, {
    method: 'PUT',
    body: JSON.stringify(data),
  });
}

export async function deleteModel(id: string): Promise<{ deleted: boolean }> {
  return request<{ deleted: boolean }>(`/admin/models/${id}`, {
    method: 'DELETE',
  });
}

// ============ Provider Keys ============

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

export interface ProviderKeysResponse {
  data: ProviderKey[];
}

export async function fetchProviderKeys(): Promise<ProviderKeysResponse> {
  return request<ProviderKeysResponse>('/admin/provider-keys');
}

export async function createProviderKey(data: { provider_slug: string; api_key: string; base_url?: string; priority?: number }): Promise<ProviderKey> {
  return request<ProviderKey>('/admin/provider-keys', {
    method: 'POST',
    body: JSON.stringify(data),
  });
}

export async function updateProviderKey(id: string, data: { api_key?: string; base_url?: string; is_active?: boolean; priority?: number }): Promise<{ updated: boolean }> {
  return request<{ updated: boolean }>(`/admin/provider-keys/${id}`, {
    method: 'PUT',
    body: JSON.stringify(data),
  });
}

export async function deleteProviderKey(id: string): Promise<{ deleted: boolean }> {
  return request<{ deleted: boolean }>(`/admin/provider-keys/${id}`, {
    method: 'DELETE',
  });
}

export async function testProviderKey(id: string): Promise<{ success: boolean; message: string }> {
  return request<{ success: boolean; message: string }>(`/admin/provider-keys/${id}/test`, {
    method: 'POST',
  });
}

// ============ Transactions ============

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

export interface TransactionsResponse {
  data: AdminTransaction[];
  total: number;
  page: number;
  per_page: number;
}

export async function fetchTransactions(page = 1, perPage = 20, type = '', status = ''): Promise<TransactionsResponse> {
  const params = new URLSearchParams({ page: String(page), per_page: String(perPage) });
  if (type) params.set('type', type);
  if (status) params.set('status', status);
  return request<TransactionsResponse>(`/admin/transactions?${params}`);
}
