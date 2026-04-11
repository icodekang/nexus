import { create } from 'zustand';
import { fetchModels, type Model } from '../api/client';

interface ModelState {
  models: Model[];
  isLoading: boolean;
  loaded: boolean;
  loadModels: () => Promise<void>;
  getModelsByProvider: () => Record<string, Model[]>;
}

const PROVIDER_ORDER = ['openai', 'anthropic', 'google', 'deepseek'];

export const useModelState = create<ModelState>((set, get) => ({
  models: [],
  isLoading: false,
  loaded: false,

  loadModels: async () => {
    if (get().isLoading) return;
    set({ isLoading: true });
    try {
      const res = await fetchModels();
      const sorted = res.data.sort((a, b) => {
        const ai = PROVIDER_ORDER.indexOf(a.provider);
        const bi = PROVIDER_ORDER.indexOf(b.provider);
        return (ai === -1 ? 99 : ai) - (bi === -1 ? 99 : bi);
      });
      set({ models: sorted, loaded: true });
    } catch (e) {
      console.error('Failed to load models:', e);
    } finally {
      set({ isLoading: false });
    }
  },

  getModelsByProvider: () => {
    const { models } = get();
    const grouped: Record<string, Model[]> = {};
    for (const m of models) {
      if (!grouped[m.provider]) grouped[m.provider] = [];
      grouped[m.provider].push(m);
    }
    return grouped;
  },
}));
