import { createContext, useContext, useState, ReactNode, useCallback } from 'react';

interface AuthContextType {
  isAuthenticated: boolean;
  login: (email: string, password: string) => Promise<boolean>;
  logout: () => void;
}

const AuthContext = createContext<AuthContextType | null>(null);

export function AuthProvider({ children }: { children: ReactNode }) {
  const [isAuthenticated, setIsAuthenticated] = useState(() => {
    return localStorage.getItem('nexus_admin_auth') === 'true';
  });

  const login = useCallback(async (email: string, password: string): Promise<boolean> => {
    const validEmail = 'admin@nexus.io';
    const validPassword = 'admin123';

    // Simulate network delay
    await new Promise((resolve) => setTimeout(resolve, 300));

    if (email === validEmail && password === validPassword) {
      localStorage.setItem('nexus_admin_auth', 'true');
      setIsAuthenticated(true);
      return true;
    }
    throw new Error('Invalid credentials');
  }, []);

  const logout = useCallback(() => {
    localStorage.removeItem('nexus_admin_auth');
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
