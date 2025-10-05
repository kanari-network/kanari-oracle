'use client';

import React from 'react';
import { ApiDocsWidget } from '@/components/widgets/ApiDocsWidget';

export default function ApiDocsPage() {
  return (
    <div className="min-h-[calc(100vh-4rem)] p-4 sm:p-6 lg:p-8">
      <div className="max-w-6xl mx-auto">
        <div className="mb-8">
          <h1 className="text-3xl sm:text-4xl font-bold text-gray-900 dark:text-white mb-2">
            API Documentation
          </h1>
          <p className="text-gray-600 dark:text-gray-400">
            Complete reference for the Kanari Oracle REST API
          </p>
        </div>

        <ApiDocsWidget />
      </div>
    </div>
  );
}
