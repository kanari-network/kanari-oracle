// API Service for Kanari Oracle
const API_BASE_URL = process.env.NEXT_PUBLIC_API_URL || 'http://localhost:3000';

export interface ApiResponse<T> {
  success: boolean;
  data: T;
  error: string | null;
}

export interface TokenResponse {
  token: string;
  expires_at: string;
}

export interface UserProfile {
  id: number;
  username: string;
  email: string | null;
  created_at: string;
}

export interface PriceData {
  symbol: string;
  price: number;
  last_update: string;
  asset_type: string;
}

export interface TokenData {
  token: string;
  expires_at: string;
}

export interface StatsData {
  total_crypto_symbols: number;
  total_stock_symbols: number;
  last_crypto_update: string | null;
  last_stock_update: string | null;
  uptime_seconds: number;
}

export interface SymbolsData {
  crypto: string[];
  stocks: string[];
}

class KanariAPI {
  private token: string | null = null;

  constructor() {
    if (typeof window !== 'undefined') {
      this.token = localStorage.getItem('kanari_token');
    }
  }

  setToken(token: string) {
    this.token = token;
    if (typeof window !== 'undefined') {
      localStorage.setItem('kanari_token', token);
    }
  }

  clearToken() {
    this.token = null;
    if (typeof window !== 'undefined') {
      localStorage.removeItem('kanari_token');
    }
  }

  getToken() {
    return this.token;
  }

  private async request<T>(
    endpoint: string,
    options: RequestInit = {}
  ): Promise<ApiResponse<T>> {
    const headers: Record<string, string> = {
      'Content-Type': 'application/json',
    };

    if (options.headers) {
      Object.assign(headers, options.headers);
    }

    if (this.token && !endpoint.includes('/register') && !endpoint.includes('/login')) {
      headers['Authorization'] = `Bearer ${this.token}`;
    }

    try {
      const response = await fetch(`${API_BASE_URL}${endpoint}`, {
        ...options,
        headers,
      });

      const data = await response.json();
      return data;
    } catch (error) {
      return {
        success: false,
        data: null as any,
        error: error instanceof Error ? error.message : 'Network error',
      };
    }
  }

  // Auth endpoints
  async register(username: string, password: string, email?: string) {
    const response = await this.request<TokenResponse>('/users/register', {
      method: 'POST',
      body: JSON.stringify({ username, password, owner_email: email }),
    });

    if (response.success && response.data) {
      this.setToken(response.data.token);
    }

    return response;
  }

  async login(username: string, password: string) {
    const response = await this.request<TokenResponse>('/users/login', {
      method: 'POST',
      body: JSON.stringify({ username, password }),
    });

    if (response.success && response.data) {
      this.setToken(response.data.token);
    }

    return response;
  }

  async getProfile() {
    return this.request<UserProfile>('/users/profile');
  }

  async changePassword(currentPassword: string, newPassword: string) {
    return this.request<string>('/users/change-password', {
      method: 'POST',
      body: JSON.stringify({
        current_password: currentPassword,
        new_password: newPassword,
      }),
    });
  }

  async changeEmail(currentPassword: string, newEmail: string | null) {
    return this.request<string>('/users/change-email', {
      method: 'POST',
      body: JSON.stringify({
        current_password: currentPassword,
        new_email: newEmail,
      }),
    });
  }

  async deleteAccount(password: string) {
    return this.request<string>('/users/delete', {
      method: 'POST',
      body: JSON.stringify({ password }),
    });
  }

  // Token management
  async listTokens() {
    return this.request<{ tokens: TokenData[] }>('/users/tokens');
  }

  async createToken(label?: string) {
    return this.request<TokenResponse>('/users/tokens', {
      method: 'POST',
      body: JSON.stringify(label ? { label } : {}),
    });
  }

  async revokeToken(token: string) {
    return this.request<string>('/users/tokens/revoke', {
      method: 'POST',
      body: JSON.stringify({ token }),
    });
  }

  // Price endpoints
  async getPrice(assetType: 'crypto' | 'stock', symbol: string) {
    return this.request<PriceData>(`/price/${assetType}/${symbol}`);
  }

  async getAllPrices(assetType: 'crypto' | 'stock') {
    return this.request<PriceData[]>(`/prices/${assetType}`);
  }

  async getSymbols(assetType?: 'crypto' | 'stock') {
    const query = assetType ? `?asset_type=${assetType}` : '';
    return this.request<SymbolsData>(`/symbols${query}`);
  }

  async getStats() {
    return this.request<StatsData>('/stats');
  }

  async forceUpdate(assetType: 'crypto' | 'stock' | 'all' = 'all') {
    return this.request<string>(`/update/${assetType}`, {
      method: 'POST',
    });
  }

  async getHealth() {
    return this.request<{
      status: string;
      last_update: string | null;
      total_symbols: number;
    }>('/health');
  }
}

export const api = new KanariAPI();
