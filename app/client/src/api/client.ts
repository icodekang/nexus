import { useUserStore } from '../stores/userStore';
import { Model } from '../stores/modelStore';

const API_BASE_URL = 'http://localhost:8080/v1';

interface RequestOptions {
  method?: string;
  body?: any;
}

async function request<T>(endpoint: string, options: RequestOptions = {}): Promise<T> {
  const { token } = useUserStore.getState();
  
  const headers: Record<string, string> = {
    'Content-Type': 'application/json',
  };
  
  if (token) {
    headers['Authorization'] = `Bearer ${token}`;
  }
  
  const response = await fetch(`${API_BASE_URL}${endpoint}`, {
    method: options.method || 'GET',
    headers,
    body: options.body ? JSON.stringify(options.body) : undefined,
  });
  
  if (!response.ok) {
    const error = await response.json().catch(() => ({ error: { message: 'Request failed' } }));
    throw new Error(error.error?.message || 'Request failed');
  }
  
  return response.json();
}

// Models API
export async function fetchModels(): Promise<{ data: Model[] }> {
  return request<{ data: Model[] }>('/models');
}

// Chat API
export interface ChatMessage {
  role: 'user' | 'assistant' | 'system';
  content: string;
}

export interface ChatRequest {
  model: string;
  messages: ChatMessage[];
  temperature?: number;
  max_tokens?: number;
  stream?: boolean;
}

export interface ChatResponse {
  id: string;
  model: string;
  choices: {
    index: number;
    message: ChatMessage;
    finish_reason?: string;
  }[];
  usage: {
    prompt_tokens: number;
    completion_tokens: number;
    total_tokens: number;
  };
}

export async function sendChat(request: ChatRequest): Promise<ChatResponse> {
  return request<ChatResponse>('/chat/completions', {
    method: 'POST',
    body: request,
  });
}

export async function* streamChat(
  request: ChatRequest,
  onChunk: (chunk: string) => void
): AsyncGenerator<string> {
  const { token } = useUserStore.getState();
  
  const headers: Record<string, string> = {
    'Content-Type': 'application/json',
  };
  
  if (token) {
    headers['Authorization'] = `Bearer ${token}`;
  }
  
  const response = await fetch(`${API_BASE_URL}/chat/completions?stream=true`, {
    method: 'POST',
    headers,
    body: JSON.stringify({ ...request, stream: true }),
  });
  
  if (!response.ok) {
    throw new Error('Chat request failed');
  }
  
  const reader = response.body?.getReader();
  const decoder = new TextDecoder();
  
  if (!reader) {
    throw new Error('No response body');
  }
  
  let buffer = '';
  
  while (true) {
    const { done, value } = await reader.read();
    
    if (done) break;
    
    buffer += decoder.decode(value, { stream: true });
    
    const lines = buffer.split('\n');
    buffer = lines.pop() || '';
    
    for (const line of lines) {
      if (line.startsWith('data: ')) {
        const data = line.slice(6);
        
        if (data === '[DONE]') {
          return;
        }
        
        try {
          const parsed = JSON.parse(data);
          const delta = parsed.choices?.[0]?.delta?.content;
          if (delta) {
            onChunk(delta);
            yield delta;
          }
        } catch (e) {
          // Ignore parse errors for incomplete chunks
        }
      }
    }
  }
}

// Subscription API
export interface Subscription {
  user_id: string;
  email: string;
  phone?: string;
  subscription_plan: string;
  subscription_start?: string;
  subscription_end?: string;
  is_active: boolean;
}

export async function fetchSubscription(): Promise<Subscription> {
  return request<Subscription>('/me/subscription');
}

// Usage API
export interface UsageStats {
  total_requests: number;
  total_input_tokens: number;
  total_output_tokens: number;
  usage_by_model: Record<string, any>;
  usage_by_provider: Record<string, any>;
}

export async function fetchUsage(): Promise<UsageStats> {
  return request<UsageStats>('/me/usage');
}

// API Keys
export interface ApiKeyResponse {
  id: string;
  key?: string;
  name?: string;
  key_prefix: string;
  is_active: boolean;
  created_at: string;
}

export interface CreateApiKeyRequest {
  name?: string;
}

export async function fetchApiKeys(): Promise<{ data: ApiKeyResponse[] }> {
  return request<{ data: ApiKeyResponse[] }>('/me/keys');
}

export async function createApiKey(request: CreateApiKeyRequest): Promise<ApiKeyResponse> {
  return request<ApiKeyResponse>('/me/keys', {
    method: 'POST',
    body: request,
  });
}

export async function deleteApiKey(keyId: string): Promise<{ deleted: boolean }> {
  return request<{ deleted: boolean }>(`/me/keys/${keyId}`, {
    method: 'DELETE',
  });
}
