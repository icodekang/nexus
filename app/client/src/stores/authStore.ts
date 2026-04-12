import { create } from 'zustand';

interface AuthState {
  token: string | null;
  user: { id: string; phone?: string; email?: string; subscription_plan: string } | null;
  isAuthenticated: boolean;

  login: (token: string, user: { id: string; phone?: string; email?: string; subscription_plan: string }) => void;
  logout: () => void;
  loadFromStorage: () => void;
}

export const useAuthStore = create<AuthState>((set) => ({
  token: null,
  user: null,
  isAuthenticated: false,

  login: (token, user) => {
    localStorage.setItem('nexus_token', token);
    localStorage.setItem('nexus_user', JSON.stringify(user));
    set({ token, user, isAuthenticated: true });
  },

  logout: () => {
    localStorage.removeItem('nexus_token');
    localStorage.removeItem('nexus_user');
    set({ token: null, user: null, isAuthenticated: false });
  },

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
