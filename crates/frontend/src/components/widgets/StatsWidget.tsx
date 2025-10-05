'use client';

import React, { useEffect, useState } from 'react';
import { api, StatsData } from '@/lib/api';
import { Card, CardHeader, CardBody } from '@/components/ui/Card';

export function StatsWidget({ className = '' }: { className?: string }) {
  const [stats, setStats] = useState<StatsData | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    const fetchStats = async () => {
      const response = await api.getStats();
      if (response.success) {
        setStats(response.data);
      }
      setLoading(false);
    };

    fetchStats();
    const interval = setInterval(fetchStats, 30000);

    return () => clearInterval(interval);
  }, []);

  if (loading) {
    return (
      <Card className={className}>
        <CardBody>
          <div className="animate-pulse space-y-3">
            <div className="h-4 bg-gray-300 dark:bg-gray-600 rounded w-32"></div>
            <div className="h-4 bg-gray-300 dark:bg-gray-600 rounded w-24"></div>
          </div>
        </CardBody>
      </Card>
    );
  }

  const formatUptime = (seconds: number) => {
    const hours = Math.floor(seconds / 3600);
    const minutes = Math.floor((seconds % 3600) / 60);
    return `${hours}h ${minutes}m`;
  };

  return (
    <Card className={className}>
      <CardHeader>
        <h3 className="text-lg font-semibold text-gray-900 dark:text-white">
          Oracle Statistics
        </h3>
      </CardHeader>
      <CardBody>
        <div className="grid grid-cols-2 gap-4">
          <div className="bg-gradient-to-br from-purple-50 to-blue-50 dark:from-purple-900/20 dark:to-blue-900/20 p-4 rounded-lg">
            <p className="text-sm text-gray-600 dark:text-gray-400">Crypto Symbols</p>
            <p className="text-2xl font-bold text-purple-600 dark:text-purple-400 mt-1">
              {stats?.total_crypto_symbols || 0}
            </p>
          </div>
          <div className="bg-gradient-to-br from-green-50 to-teal-50 dark:from-green-900/20 dark:to-teal-900/20 p-4 rounded-lg">
            <p className="text-sm text-gray-600 dark:text-gray-400">Stock Symbols</p>
            <p className="text-2xl font-bold text-green-600 dark:text-green-400 mt-1">
              {stats?.total_stock_symbols || 0}
            </p>
          </div>
          <div className="col-span-2 bg-gradient-to-br from-orange-50 to-red-50 dark:from-orange-900/20 dark:to-red-900/20 p-4 rounded-lg">
            <p className="text-sm text-gray-600 dark:text-gray-400">System Uptime</p>
            <p className="text-xl font-bold text-orange-600 dark:text-orange-400 mt-1">
              {stats ? formatUptime(stats.uptime_seconds) : '0h 0m'}
            </p>
          </div>
        </div>
      </CardBody>
    </Card>
  );
}
