/**
 * @file Model Store - 模型状态管理
 * 管理可用模型的列表、加载状态和分组
 * 使用 Zustand 进行状态管理
 */

import { create } from 'zustand';
import { fetchModels, type Model } from '../api/client';

/**
 * ModelState - 模型状态接口
 * @description 包含模型相关的所有状态和操作
 */
interface ModelState {
  models: Model[];         // 模型列表
  isLoading: boolean;      // 是否正在加载
  loaded: boolean;         // 是否已加载过（防止重复加载）

  // Actions
  loadModels: () => Promise<void>;                                // 加载模型列表
  getModelsByProvider: () => Record<string, Model[]>;             // 按服务商分组获取模型
}

// 服务商排序优先级
const PROVIDER_ORDER = ['openai', 'anthropic', 'google', 'deepseek'];

/**
 * useModelState - 模型状态管理 Hook
 * @description 管理可用模型的状态
 *
 * 状态说明：
 * - models: 从后端获取的可用模型列表
 * - isLoading: 是否正在加载模型数据
 * - loaded: 是否已成功加载过（防止重复加载）
 *
 * 核心功能：
 * - loadModels(): 从 API 加载模型列表，按服务商优先级排序
 * - getModelsByProvider(): 将模型按服务商分组返回
 */
export const useModelState = create<ModelState>((set, get) => ({
  models: [],
  isLoading: false,
  loaded: false,

  /**
   * loadModels - 加载模型列表
   * @description 从 API 获取可用模型，按服务商优先级排序
   * 加载完成后设置 loaded=true，防止重复加载
   */
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

  /**
   * getModelsByProvider - 按服务商分组获取模型
   * @returns 以服务商名为 key 的模型分组对象
   * @example { openai: [Model, Model], anthropic: [Model] }
   */
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
