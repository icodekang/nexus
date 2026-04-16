/**
 * @file Auth Store - 用户认证状态管理
 * 管理用户的登录状态、JWT token 和用户信息
 * 使用 Zustand 进行状态管理，数据持久化到 localStorage
 */

import { create } from 'zustand';

/**
 * AuthState - 认证状态接口
 * @description 包含认证相关的所有状态和操作
 */
interface AuthState {
  token: string | null;      // JWT token
  user: { id: string; phone?: string; email?: string; subscription_plan: string } | null;  // 用户信息
  isAuthenticated: boolean;  // 是否已认证

  // Actions
  login: (token: string, user: { id: string; phone?: string; email?: string; subscription_plan: string }) => void;  // 登录
  logout: () => void;        // 登出
  loadFromStorage: () => void;  // 从 localStorage 恢复状态
}

/**
 * useAuthStore - 认证状态管理 Hook
 * @description 包含状态：
 * - token: JWT 令牌
 * - user: 用户信息对象
 * - isAuthenticated: 认证状态标志
 *
 * 包含操作：
 * - login(): 执行登录，存储 token 和用户信息到 localStorage
 * - logout(): 执行登出，清除 localStorage 和状态
 * - loadFromStorage(): 页面加载时从 localStorage 恢复认证状态
 */
export const useAuthStore = create<AuthState>((set) => ({
  token: null,
  user: null,
  isAuthenticated: false,

  /**
   * login - 用户登录
   * @param token - JWT token
   * @param user - 用户信息
   * 将 token 和用户信息同时存储到 localStorage 和 state
   */
  login: (token, user) => {
    localStorage.setItem('nexus_token', token);
    localStorage.setItem('nexus_user', JSON.stringify(user));
    set({ token, user, isAuthenticated: true });
  },

  /**
   * logout - 用户登出
   * 清除 localStorage 中的 token 和用户信息，重置状态
   */
  logout: () => {
    localStorage.removeItem('nexus_token');
    localStorage.removeItem('nexus_user');
    set({ token: null, user: null, isAuthenticated: false });
  },

  /**
   * loadFromStorage - 从 localStorage 恢复认证状态
   * 页面刷新后尝试恢复用户的登录状态
   * 如果 token 或用户信息格式错误，会清除并重置状态
   */
  loadFromStorage: () => {
    const token = localStorage.getItem('nexus_token');
    const userStr = localStorage.getItem('nexus_user');
    if (token && userStr) {
      try {
        const user = JSON.parse(userStr);
        set({ token, user, isAuthenticated: true });
      } catch {
        localStorage.removeItem('nexus_token');
        localStorage.removeItem('nexus_user');
      }
    }
  },
}));
