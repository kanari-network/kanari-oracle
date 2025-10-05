'use client';

import React, { useEffect, useState } from 'react';
import { useRouter, useSearchParams } from 'next/navigation';
import { useAuth } from '@/contexts/AuthContext';
import { Card, CardHeader, CardBody } from '@/components/ui/Card';
import { Button } from '@/components/ui/Button';
import { PriceGrid } from '@/components/widgets/PriceGrid';

export default function PricesPage() {
  const { user, loading } = useAuth();
  const router = useRouter();
  const searchParams = useSearchParams();
  const [assetType, setAssetType] = useState<'crypto' | 'stock'>(
    (searchParams.get('type') as 'crypto' | 'stock') || 'crypto'
  );

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
          <h1 className="text-3xl sm:text-4xl font-bold text-gray-900 dark:text-white mb-4">
            Price Feed
          </h1>

          {/* Asset Type Toggle */}
          <div className="flex gap-2">
            <Button
              variant={assetType === 'crypto' ? 'primary' : 'secondary'}
              onClick={() => setAssetType('crypto')}
            >
              <span className="mr-2">₿</span> Cryptocurrency
            </Button>
            <Button
              variant={assetType === 'stock' ? 'primary' : 'secondary'}
              onClick={() => setAssetType('stock')}
            >
              <span className="mr-2">📈</span> Stocks
            </Button>
          </div>
        </div>

        {/* Price Grid */}
        <PriceGrid assetType={assetType} />
      </div>
    </div>
  );
}
