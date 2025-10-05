'use client';

import React, { useEffect } from 'react';
import { useRouter } from 'next/navigation';
import { useAuth } from '@/contexts/AuthContext';
import { Card, CardHeader, CardBody } from '@/components/ui/Card';
import { PriceWidget } from '@/components/widgets/PriceWidget';
import { StatsWidget } from '@/components/widgets/StatsWidget';
import { Button } from '@/components/ui/Button';
import Link from 'next/link';

export default function DashboardPage() {
  const { user, loading } = useAuth();
  const router = useRouter();

  useEffect(() => {
    if (!loading && !user) {
      router.push('/login');
    }
  }, [user, loading, router]);

  if (loading) {
    return (
      <div className="min-h-[calc(100vh-4rem)] flex items-center justify-center">
        <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-purple-600"></div>
      </div>
    );
  }

  if (!user) {
    return null;
  }

  return (
    <div className="min-h-[calc(100vh-4rem)] p-4 sm:p-6 lg:p-8">
      <div className="max-w-7xl mx-auto">
        {/* Header */}
        <div className="mb-8">
          <h1 className="text-3xl sm:text-4xl font-bold text-gray-900 dark:text-white mb-2">
            Welcome back, {user.username}! 👋
          </h1>
          <p className="text-gray-600 dark:text-gray-400">
            Here&apos;s your Oracle dashboard
          </p>
        </div>

        {/* Stats Section */}
        <div className="mb-8">
          <StatsWidget />
        </div>

        {/* Quick Actions */}
        <Card className="mb-8">
          <CardHeader>
            <h2 className="text-xl font-semibold text-gray-900 dark:text-white">
              Quick Actions
            </h2>
          </CardHeader>
          <CardBody>
            <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
              <Link href="/prices?type=crypto" className="block">
                <Button variant="secondary" className="w-full">
                  <span className="mr-2">₿</span> View Crypto Prices
                </Button>
              </Link>
              <Link href="/prices?type=stock" className="block">
                <Button variant="secondary" className="w-full">
                  <span className="mr-2">📈</span> View Stock Prices
                </Button>
              </Link>
              <Link href="/tokens" className="block">
                <Button variant="secondary" className="w-full">
                  <span className="mr-2">🔑</span> Manage Tokens
                </Button>
              </Link>
              <Link href="/profile" className="block">
                <Button variant="secondary" className="w-full">
                  <span className="mr-2">👤</span> Profile Settings
                </Button>
              </Link>
            </div>
          </CardBody>
        </Card>

        {/* Featured Prices */}
        <div>
          <h2 className="text-2xl font-bold text-gray-900 dark:text-white mb-4">
            Featured Prices
          </h2>
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
            <PriceWidget assetType="crypto" symbol="bitcoin" />
            <PriceWidget assetType="crypto" symbol="ethereum" />
            <PriceWidget assetType="stock" symbol="AAPL" />
            <PriceWidget assetType="stock" symbol="TSLA" />
          </div>
        </div>
      </div>
    </div>
  );
}
