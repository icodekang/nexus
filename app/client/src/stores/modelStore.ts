import { create } from 'zustand';

export interface Model {
  id: string;
  name: string;
  provider: string;
  providerName: string;
  contextWindow: number;
  capabilities: string[];
}

interface ModelState {
  models: Model[];
  selectedModel: Model | null;
  selectedProvider: string | null;
  isLoading: boolean;
  error: string | null;
  
  // Actions
  setModels: (models: Model[]) => void;
  selectModel: (model: Model) => void;
  selectProvider: (provider: string | null) => void;
  setLoading: (loading: boolean) => void;
  setError: (error: string | null) => void;
  getFilteredModels: () => Model[];
}

export const useModelStore = create<ModelState>((set, get) => ({
  models: [],
  selectedModel: null,
  selectedProvider: null,
  isLoading: false,
  error: null,

  setModels: (models: Model[]) => {
    // Sort models: recommended first, then by name
    const sorted = [...models].sort((a, b) => {
      // Provider order for "recommended"
      const providerOrder = ['openai', 'anthropic', 'google', 'deepseek'];
      const aIndex = providerOrder.indexOf(a.provider);
      const bIndex = providerOrder.indexOf(b.provider);
      
      if (aIndex !== -1 && bIndex !== -1) {
        return aIndex - bIndex;
      }
      if (aIndex !== -1) return -1;
      if (bIndex !== -1) return 1;
      return a.name.localeCompare(b.name);
    });
    
    set({ models: sorted });
    
    // Auto-select first model if none selected
    if (!get().selectedModel && sorted.length > 0) {
      set({ selectedModel: sorted[0] });
    }
  },

  selectModel: (model: Model) => {
    set({ selectedModel: model });
  },

  selectProvider: (provider: string | null) => {
    set({ selectedProvider: provider });
  },

  setLoading: (loading: boolean) => {
    set({ isLoading: loading });
  },

  setError: (error: string | null) => {
    set({ error });
  },

  getFilteredModels: () => {
    const { models, selectedProvider } = get();
    if (!selectedProvider) return models;
    return models.filter((m) => m.provider === selectedProvider);
  },
}));

// Provider metadata
export const PROVIDERS = [
  { id: 'openai', name: 'OpenAI', logo: '🤖', color: '#10A37F' },
  { id: 'anthropic', name: 'Anthropic', logo: '🧠', color: '#FF6B35' },
  { id: 'google', name: 'Google', logo: '🔵', color: '#4285F4' },
  { id: 'deepseek', name: 'DeepSeek', logo: '🔴', color: '#DC2626' },
];

export const getProviderInfo = (providerId: string) => {
  return PROVIDERS.find((p) => p.id === providerId) || PROVIDERS[0];
};
