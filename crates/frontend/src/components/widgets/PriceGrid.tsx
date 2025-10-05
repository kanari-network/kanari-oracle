'use client';

import React, { useEffect, useState } from 'react';
import { api, PriceData } from '@/lib/api';
import { Card, CardBody } from '@/components/ui/Card';

interface PriceGridProps {
  assetType: 'crypto' | 'stock';
  className?: string;
}

export function PriceGrid({ assetType, className = '' }: PriceGridProps) {
  const [prices, setPrices] = useState<PriceData[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const fetchPrices = async () => {
      try {
        const response = await api.getAllPrices(assetType);
        if (response.success) {
          setPrices(response.data);
          setError(null);
        } else {
          setError(response.error || 'Failed to fetch prices');
        }
      } catch (err) {
        setError('Network error');
      } finally {
        setLoading(false);
      }
    };

    fetchPrices();
    const interval = setInterval(fetchPrices, 30000); // Update every 30 seconds

    return () => clearInterval(interval);
  }, [assetType]);

  if (loading) {
    return (
      <div className={`grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-4 ${className}`}>
        {[...Array(8)].map((_, i) => (
          <Card key={i}>
            <CardBody>
              <div className="animate-pulse">
                <div className="h-4 bg-gray-300 dark:bg-gray-600 rounded w-24 mb-2"></div>
                <div className="h-8 bg-gray-300 dark:bg-gray-600 rounded w-32"></div>
              </div>
            </CardBody>
          </Card>
        ))}
      </div>
    );
  }

  if (error) {
    return (
      <Card className={className}>
        <CardBody>
          <p className="text-red-500 text-center">{error}</p>
        </CardBody>
      </Card>
    );
  }

  return (
    <div className={`grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-4 ${className}`}>
      {prices.map((price) => (
        <Card key={price.symbol} hover>
          <CardBody>
            <div className="flex items-center justify-between">
              <div className="flex-1">
                <p className="text-sm text-gray-600 dark:text-gray-400 font-medium uppercase">
                  {price.symbol}
                </p>
                <p className="text-xl font-bold text-gray-900 dark:text-white mt-1">
                  ${price.price.toLocaleString(undefined, { 
                    minimumFractionDigits: 2, 
                    maximumFractionDigits: price.price < 1 ? 6 : 2 
                  })}
                </p>
                <p className="text-xs text-gray-500 dark:text-gray-500 mt-1">
                  {new Date(price.last_update).toLocaleTimeString()}
                </p>
              </div>
              <div className="text-2xl opacity-50">
                {assetType === 'crypto' ? '₿' : '📈'}
              </div>
            </div>
          </CardBody>
        </Card>
      ))}
    </div>
  );
}
