import { create } from 'zustand';
import AsyncStorage from '@react-native-async-storage/async-storage';

export interface User {
  id: string;
  email: string;
  phone?: string;
  subscriptionPlan: string;
  subscriptionEnd?: string;
}

export interface ApiKey {
  id: string;
  name?: string;
  keyPrefix: string;
  isActive: boolean;
  createdAt: string;
}

interface UserState {
  user: User | null;
  apiKeys: ApiKey[];
  isLoggedIn: boolean;
  isLoading: boolean;
  token: string | null;
  
  // Actions
  setUser: (user: User | null) => void;
  setToken: (token: string | null) => void;
  setApiKeys: (keys: ApiKey[]) => void;
  setLoading: (loading: boolean) => void;
  login: (email: string, password: string) => Promise<void>;
  logout: () => Promise<void>;
  loadStoredAuth: () => Promise<void>;
}

export const useUserStore = create<UserState>((set, get) => ({
  user: null,
  apiKeys: [],
  isLoggedIn: false,
  isLoading: true,
  token: null,

  setUser: (user: User | null) => {
    set({ user, isLoggedIn: !!user });
  },

  setToken: (token: string | null) => {
    set({ token });
    if (token) {
      AsyncStorage.setItem('auth_token', token);
    } else {
      AsyncStorage.removeItem('auth_token');
    }
  },

  setApiKeys: (keys: ApiKey[]) => {
    set({ apiKeys: keys });
  },

  setLoading: (loading: boolean) => {
    set({ isLoading: loading });
  },

  login: async (email: string, password: string) => {
    set({ isLoading: true });
    try {
      const response = await fetch('http://localhost:8080/v1/auth/login', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({ email, password }),
      });
      
      if (!response.ok) {
        throw new Error('Login failed');
      }
      
      const data = await response.json();
      set({ 
        user: data.user, 
        token: data.token,
        isLoggedIn: true,
        isLoading: false,
      });
      
      await AsyncStorage.setItem('auth_token', data.token);
    } catch (error) {
      set({ isLoading: false });
      throw error;
    }
  },

  logout: async () => {
    set({ 
      user: null, 
      apiKeys: [], 
      isLoggedIn: false, 
      token: null 
    });
    await AsyncStorage.removeItem('auth_token');
  },

  loadStoredAuth: async () => {
    try {
      const token = await AsyncStorage.getItem('auth_token');
      if (token) {
        set({ token, isLoading: true });
        
        // In a real app, we would validate the token and fetch user data
        // For now, just set the loading to false
        set({ isLoading: false });
      } else {
        set({ isLoading: false });
      }
    } catch (error) {
      set({ isLoading: false });
    }
  },
}));
