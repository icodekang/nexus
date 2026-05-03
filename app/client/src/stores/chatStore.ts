/**
 * @file Chat Store - 搜索状态管理
 * 管理多模型搜索的状态和结果
 */

import { create } from 'zustand';
import type { ModelResult } from '../api/client';

/**
 * SearchResult - 搜索结果
 */
export interface SearchResult {
  id: string;
  query: string;
  results: ModelResult[];
  judgeModel: string;
  totalLatency: number;
  timestamp: number;
  selectionCategory: string;
  selectedModels: string[];
  hasScoring: boolean;
}

/**
 * ChatState - 搜索状态
 */
interface ChatState {
  // 搜索状态
  isSearching: boolean;
  searchQuery: string;
  currentResult: SearchResult | null;
  searchHistory: SearchResult[];
  selectedModel: string | null;

  // Actions
  setSearching: (loading: boolean) => void;
  setSearchQuery: (query: string) => void;
  setSearchResult: (result: SearchResult) => void;
  clearResult: () => void;
  addToHistory: (result: SearchResult) => void;
  setSelectedModel: (model: string | null) => void;
}

export const useChatStore = create<ChatState>((set) => ({
  isSearching: false,
  searchQuery: '',
  currentResult: null,
  searchHistory: [],
  selectedModel: null,

  setSearching: (loading) => set({ isSearching: loading }),
  setSearchQuery: (query) => set({ searchQuery: query }),

  setSearchResult: (result) => set({
    currentResult: result,
    searchQuery: result.query,
  }),

  clearResult: () => set({
    currentResult: null,
    searchQuery: '',
  }),

  addToHistory: (result) => set((state) => ({
    searchHistory: [result, ...state.searchHistory].slice(0, 20),
  })),

  setSelectedModel: (model) => set({ selectedModel: model }),
}));
