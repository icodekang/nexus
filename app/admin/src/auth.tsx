import { createContext, useContext, useState, ReactNode, useCallback } from 'react';
import { login as apiLogin } from './api/admin';

interface AuthContextType {
  isAuthenticated: boolean;
  login: (email: string, password: string) => Promise<boolean>;
  logout: () => void;
}

const AuthContext = createContext<AuthContextType | null>(null);

export function AuthProvider({ children }: { children: ReactNode }) {
  const [isAuthenticated, setIsAuthenticated] = useState(() => {
    return !!localStorage.getItem('nexus_admin_token');
  });

  const login = useCallback(async (email: string, password: string): Promise<boolean> => {
    const res = await apiLogin(email, password);
    localStorage.setItem('nexus_admin_token', res.token);
    localStorage.setItem('nexus_admin_user', JSON.stringify(res.user));
    setIsAuthenticated(true);
    return true;
  }, []);

  const logout = useCallback(() => {
    localStorage.removeItem('nexus_admin_token');
    localStorage.removeItem('nexus_admin_user');
    setIsAuthenticated(false);
  }, []);

  return (
    <AuthContext.Provider value={{ isAuthenticated, login, logout }}>
      {children}
    </AuthContext.Provider>
  );
}

export function useAuth() {
  const context = useContext(AuthContext);
  if (!context) {
    throw new Error('useAuth must be used within AuthProvider');
  }
  return context;
}
