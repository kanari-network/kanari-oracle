'use client';

import React, { useEffect, useState } from 'react';
import { api } from '@/lib/api';

interface HealthStatus {
  status: string;
  last_update: string | null;
  total_symbols: number;
}

export function HealthIndicator({ className = '' }: { className?: string }) {
  const [health, setHealth] = useState<HealthStatus | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    const checkHealth = async () => {
      try {
        const response = await api.getHealth();
        if (response.success) {
          setHealth(response.data);
        }
      } catch (error) {
        console.error('Health check failed:', error);
      } finally {
        setLoading(false);
      }
    };

    checkHealth();
    const interval = setInterval(checkHealth, 30000); // Check every 30 seconds

    return () => clearInterval(interval);
  }, []);

  if (loading) {
    return (
      <div className={`inline-flex items-center gap-2 ${className}`}>
        <div className="w-2 h-2 bg-gray-400 rounded-full animate-pulse"></div>
        <span className="text-xs text-gray-500">Checking...</span>
      </div>
    );
  }

  const isHealthy = health?.status === 'healthy';

  return (
    <div className={`inline-flex items-center gap-2 ${className}`}>
      <div
        className={`w-2 h-2 rounded-full ${
          isHealthy
            ? 'bg-green-500 animate-pulse'
            : 'bg-red-500'
        }`}
      ></div>
      <span className={`text-xs ${
        isHealthy
          ? 'text-green-600 dark:text-green-400'
          : 'text-red-600 dark:text-red-400'
      }`}>
        {isHealthy ? 'API Online' : 'API Offline'}
      </span>
    </div>
  );
}
