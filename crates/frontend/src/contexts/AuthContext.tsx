'use client';

import React, { createContext, useContext, useState, useEffect } from 'react';
import { api, UserProfile } from '@/lib/api';

interface AuthContextType {
  user: UserProfile | null;
  token: string | null;
  loading: boolean;
  login: (username: string, password: string) => Promise<{ success: boolean; error?: string }>;
  register: (username: string, password: string, email?: string) => Promise<{ success: boolean; error?: string }>;
  logout: () => void;
  refreshProfile: () => Promise<void>;
}

const AuthContext = createContext<AuthContextType | undefined>(undefined);

export function AuthProvider({ children }: { children: React.ReactNode }) {
  const [user, setUser] = useState<UserProfile | null>(null);
  const [token, setToken] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    const initAuth = async () => {
      const storedToken = api.getToken();
      if (storedToken) {
        setToken(storedToken);
        await loadProfile();
      }
      setLoading(false);
    };

    initAuth();
  }, []);

  const loadProfile = async () => {
    try {
      const response = await api.getProfile();
      if (response.success) {
        setUser(response.data);
      } else {
        // Token might be expired
        api.clearToken();
        setToken(null);
        setUser(null);
      }
    } catch (error) {
      console.error('Failed to load profile:', error);
    }
  };

  const login = async (username: string, password: string) => {
    try {
      const response = await api.login(username, password);
      if (response.success && response.data) {
        setToken(response.data.token);
        await loadProfile();
        return { success: true };
      }
      return { success: false, error: response.error || 'Login failed' };
    } catch (error) {
      return { success: false, error: 'Network error' };
    }
  };

  const register = async (username: string, password: string, email?: string) => {
    try {
      const response = await api.register(username, password, email);
      if (response.success && response.data) {
        setToken(response.data.token);
        await loadProfile();
        return { success: true };
      }
      return { success: false, error: response.error || 'Registration failed' };
    } catch (error) {
      return { success: false, error: 'Network error' };
    }
  };

  const logout = () => {
    api.clearToken();
    setToken(null);
    setUser(null);
  };

  const refreshProfile = async () => {
    await loadProfile();
  };

  return (
    <AuthContext.Provider value={{ user, token, loading, login, register, logout, refreshProfile }}>
      {children}
    </AuthContext.Provider>
  );
}

export function useAuth() {
  const context = useContext(AuthContext);
  if (context === undefined) {
    throw new Error('useAuth must be used within an AuthProvider');
  }
  return context;
}
