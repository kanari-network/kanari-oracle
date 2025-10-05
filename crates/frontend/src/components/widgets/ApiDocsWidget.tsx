'use client';

import React, { useState } from 'react';
import { Card, CardHeader, CardBody } from '@/components/ui/Card';
import { Button } from '@/components/ui/Button';

interface ApiEndpoint {
  method: string;
  endpoint: string;
  description: string;
  auth: boolean;
  example: string;
  body?: string;
}

const endpoints: ApiEndpoint[] = [
  {
    method: 'POST',
    endpoint: '/users/register',
    description: 'Register a new user account',
    auth: false,
    body: '{"username":"","password":"","owner_email":""}',
    example: 'curl -X POST http://localhost:3000/users/register -H "Content-Type: application/json" -d \'{"username":"alice","password":"secret123","owner_email":"alice@example.com"}\'',
  },
  {
    method: 'POST',
    endpoint: '/users/login',
    description: 'Login with existing credentials',
    auth: false,
    body: '{"username":"","password":""}',
    example: 'curl -X POST http://localhost:3000/users/login -H "Content-Type: application/json" -d \'{"username":"alice","password":"secret123"}\'',
  },
  {
    method: 'GET',
    endpoint: '/users/profile',
    description: 'Get current user profile',
    auth: true,
    example: 'curl -H "Authorization: Bearer YOUR_TOKEN_HERE" http://localhost:3000/users/profile',
  },
  {
    method: 'GET',
    endpoint: '/users/tokens',
    description: 'List all API tokens',
    auth: true,
    example: 'curl -H "Authorization: Bearer YOUR_TOKEN_HERE" http://localhost:3000/users/tokens',
  },
  {
    method: 'POST',
    endpoint: '/users/tokens',
    description: 'Create a new API token',
    auth: true,
    body: '{"label":"automation-key-1"}',
    example: 'curl -X POST http://localhost:3000/users/tokens -H "Content-Type: application/json" -H "Authorization: Bearer YOUR_TOKEN_HERE" -d \'{"label":"automation-key-1"}\'',
  },
  {
    method: 'POST',
    endpoint: '/users/tokens/revoke',
    description: 'Revoke an API token',
    auth: true,
    body: '{"token":"token-to-revoke"}',
    example: 'curl -X POST http://localhost:3000/users/tokens/revoke -H "Content-Type: application/json" -H "Authorization: Bearer YOUR_TOKEN_HERE" -d \'{"token":"token-value"}\'',
  },
  {
    method: 'POST',
    endpoint: '/users/change-password',
    description: 'Change account password',
    auth: true,
    body: '{"current_password":"","new_password":""}',
    example: 'curl -X POST http://localhost:3000/users/change-password -H "Content-Type: application/json" -H "Authorization: Bearer YOUR_TOKEN_HERE" -d \'{"current_password":"old","new_password":"new"}\'',
  },
  {
    method: 'POST',
    endpoint: '/users/delete',
    description: 'Delete user account permanently',
    auth: true,
    body: '{"password":""}',
    example: 'curl -X POST http://localhost:3000/users/delete -H "Content-Type: application/json" -H "Authorization: Bearer YOUR_TOKEN_HERE" -d \'{"password":"your_password"}\'',
  },
  {
    method: 'GET',
    endpoint: '/price/{type}/{symbol}',
    description: 'Get specific asset price',
    auth: true,
    example: 'curl -H "Authorization: Bearer YOUR_TOKEN_HERE" http://localhost:3000/price/crypto/bitcoin',
  },
  {
    method: 'GET',
    endpoint: '/prices/{type}',
    description: 'Get all prices by asset type',
    auth: true,
    example: 'curl -H "Authorization: Bearer YOUR_TOKEN_HERE" http://localhost:3000/prices/crypto',
  },
  {
    method: 'GET',
    endpoint: '/symbols',
    description: 'List available symbols',
    auth: true,
    example: 'curl -H "Authorization: Bearer YOUR_TOKEN_HERE" http://localhost:3000/symbols',
  },
  {
    method: 'GET',
    endpoint: '/stats',
    description: 'Get Oracle statistics',
    auth: true,
    example: 'curl -H "Authorization: Bearer YOUR_TOKEN_HERE" http://localhost:3000/stats',
  },
  {
    method: 'POST',
    endpoint: '/update/{type}',
    description: 'Force price update',
    auth: true,
    example: 'curl -X POST -H "Authorization: Bearer YOUR_TOKEN_HERE" http://localhost:3000/update/all',
  },
  {
    method: 'GET',
    endpoint: '/health',
    description: 'Check API health status',
    auth: false,
    example: 'curl http://localhost:3000/health',
  },
];

export function ApiDocsWidget({ className = '' }: { className?: string }) {
  const [copiedIndex, setCopiedIndex] = useState<number | null>(null);

  const copyToClipboard = (text: string, index: number) => {
    navigator.clipboard.writeText(text);
    setCopiedIndex(index);
    setTimeout(() => setCopiedIndex(null), 2000);
  };

  const getMethodColor = (method: string) => {
    switch (method) {
      case 'GET':
        return 'bg-green-100 text-green-700 dark:bg-green-900/30 dark:text-green-400';
      case 'POST':
        return 'bg-blue-100 text-blue-700 dark:bg-blue-900/30 dark:text-blue-400';
      case 'PUT':
        return 'bg-yellow-100 text-yellow-700 dark:bg-yellow-900/30 dark:text-yellow-400';
      case 'DELETE':
        return 'bg-red-100 text-red-700 dark:bg-red-900/30 dark:text-red-400';
      default:
        return 'bg-gray-100 text-gray-700 dark:bg-gray-800 dark:text-gray-400';
    }
  };

  return (
    <Card className={className}>
      <CardHeader>
        <h3 className="text-2xl font-bold text-gray-900 dark:text-white">
          API Reference
        </h3>
        <p className="text-gray-600 dark:text-gray-400 mt-1">
          Quick reference for all available endpoints
        </p>
      </CardHeader>
      <CardBody>
        <div className="space-y-4">
          {endpoints.map((endpoint, index) => (
            <div
              key={index}
              className="border border-gray-200 dark:border-gray-700 rounded-lg p-4 hover:border-purple-300 dark:hover:border-purple-700 transition-colors"
            >
              <div className="flex items-start gap-3 mb-2">
                <span
                  className={`px-2 py-1 rounded text-xs font-bold ${getMethodColor(
                    endpoint.method
                  )}`}
                >
                  {endpoint.method}
                </span>
                <div className="flex-1">
                  <code className="text-sm font-mono text-gray-900 dark:text-white">
                    {endpoint.endpoint}
                  </code>
                  {endpoint.auth && (
                    <span className="ml-2 text-xs bg-purple-100 text-purple-700 dark:bg-purple-900/30 dark:text-purple-400 px-2 py-0.5 rounded">
                      🔐 Auth Required
                    </span>
                  )}
                </div>
              </div>
              <p className="text-sm text-gray-600 dark:text-gray-400 mb-3">
                {endpoint.description}
              </p>
              {endpoint.body && (
                <div className="mb-3">
                  <p className="text-xs text-gray-500 dark:text-gray-500 mb-1">
                    Request Body:
                  </p>
                  <pre className="bg-gray-50 dark:bg-gray-900 p-2 rounded text-xs overflow-x-auto">
                    <code className="text-gray-800 dark:text-gray-300">
                      {endpoint.body}
                    </code>
                  </pre>
                </div>
              )}
              <div className="relative">
                <p className="text-xs text-gray-500 dark:text-gray-500 mb-1">
                  Example:
                </p>
                <div className="flex items-start gap-2">
                  <pre className="flex-1 bg-gray-50 dark:bg-gray-900 p-2 rounded text-xs overflow-x-auto">
                    <code className="text-gray-800 dark:text-gray-300">
                      {endpoint.example}
                    </code>
                  </pre>
                  <Button
                    size="sm"
                    variant="ghost"
                    onClick={() => copyToClipboard(endpoint.example, index)}
                    className="shrink-0"
                  >
                    {copiedIndex === index ? '✓' : '📋'}
                  </Button>
                </div>
              </div>
            </div>
          ))}
        </div>
      </CardBody>
    </Card>
  );
}
