'use client';

import React, { useEffect, useState } from 'react';
import { api, PriceData } from '@/lib/api';
import { Card, CardHeader, CardBody } from '@/components/ui/Card';

interface PriceWidgetProps {
  assetType: 'crypto' | 'stock';
  symbol: string;
  className?: string;
}

export function PriceWidget({ assetType, symbol, className = '' }: PriceWidgetProps) {
  const [price, setPrice] = useState<PriceData | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const fetchPrice = async () => {
      try {
        const response = await api.getPrice(assetType, symbol);
        if (response.success) {
          setPrice(response.data);
          setError(null);
        } else {
          setError(response.error || 'Failed to fetch price');
        }
      } catch (err) {
        setError('Network error');
      } finally {
        setLoading(false);
      }
    };

    fetchPrice();
    const interval = setInterval(fetchPrice, 30000); // Update every 30 seconds

    return () => clearInterval(interval);
  }, [assetType, symbol]);

  if (loading) {
    return (
      <Card className={className}>
        <CardBody>
          <div className="animate-pulse">
            <div className="h-4 bg-gray-300 dark:bg-gray-600 rounded w-24 mb-2"></div>
            <div className="h-8 bg-gray-300 dark:bg-gray-600 rounded w-32"></div>
          </div>
        </CardBody>
      </Card>
    );
  }

  if (error) {
    return (
      <Card className={className}>
        <CardBody>
          <p className="text-red-500 text-sm">{error}</p>
        </CardBody>
      </Card>
    );
  }

  return (
    <Card hover className={className}>
      <CardBody>
        <div className="flex items-center justify-between">
          <div>
            <p className="text-sm text-gray-600 dark:text-gray-400 font-medium uppercase">
              {price?.symbol}
            </p>
            <p className="text-2xl font-bold text-gray-900 dark:text-white mt-1">
              ${price?.price.toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 })}
            </p>
            <p className="text-xs text-gray-500 dark:text-gray-500 mt-1">
              {price?.last_update && new Date(price.last_update).toLocaleTimeString()}
            </p>
          </div>
          <div className="text-3xl">
            {assetType === 'crypto' ? '₿' : '📈'}
          </div>
        </div>
      </CardBody>
    </Card>
  );
}
