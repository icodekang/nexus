// Shared types for NovaChat clients

export interface Message {
  role: 'user' | 'assistant' | 'system';
  content: string;
  name?: string;
}

export interface ChatRequest {
  model: string;
  messages: Message[];
  temperature?: number;
  max_tokens?: number;
  stream?: boolean;
}

export interface ChatResponse {
  id: string;
  object: string;
  created: number;
  model: string;
  choices: Choice[];
  usage: Usage;
}

export interface Choice {
  index: number;
  message: Message;
  finish_reason?: string;
}

export interface Usage {
  prompt_tokens: number;
  completion_tokens: number;
  total_tokens: number;
}

export interface Model {
  id: string;
  name: string;
  provider: string;
  providerName: string;
  priceInput: number;
  priceOutput: number;
  contextWindow: number;
  capabilities: string[];
}

export interface Provider {
  id: string;
  name: string;
  slug: string;
  logoUrl?: string;
}

export interface User {
  id: string;
  email: string;
  credits: number;
}

export interface ApiKey {
  id: string;
  name: string;
  key: string;
  createdAt: string;
}

export interface Conversation {
  id: string;
  title: string;
  model: string;
  createdAt: string;
  updatedAt: string;
}
