'use client';

import React, { useEffect, useState } from 'react';
import { api, TokenData } from '@/lib/api';
import { Card, CardHeader, CardBody } from '@/components/ui/Card';
import { Button } from '@/components/ui/Button';
import { Input } from '@/components/ui/Input';

export function TokenManager({ className = '' }: { className?: string }) {
  const [tokens, setTokens] = useState<TokenData[]>([]);
  const [loading, setLoading] = useState(true);
  const [creating, setCreating] = useState(false);
  const [label, setLabel] = useState('');
  const [showCreateForm, setShowCreateForm] = useState(false);
  const [copiedToken, setCopiedToken] = useState<string | null>(null);

  const fetchTokens = async () => {
    const response = await api.listTokens();
    if (response.success) {
      setTokens(response.data.tokens);
    }
    setLoading(false);
  };

  useEffect(() => {
    fetchTokens();
  }, []);

  const handleCreateToken = async () => {
    setCreating(true);
    const response = await api.createToken(label || undefined);
    if (response.success) {
      setLabel('');
      setShowCreateForm(false);
      await fetchTokens();
      // Show the new token
      alert(`New token created:\n${response.data.token}\n\nSave this token securely. You won't be able to see it again.`);
    } else {
      alert(`Failed to create token: ${response.error}`);
    }
    setCreating(false);
  };

  const handleRevokeToken = async (token: string) => {
    if (!confirm('Are you sure you want to revoke this token?')) return;

    const response = await api.revokeToken(token);
    if (response.success) {
      await fetchTokens();
    } else {
      alert(`Failed to revoke token: ${response.error}`);
    }
  };

  const copyToClipboard = (token: string) => {
    navigator.clipboard.writeText(token);
    setCopiedToken(token);
    setTimeout(() => setCopiedToken(null), 2000);
  };

  if (loading) {
    return (
      <Card className={className}>
        <CardBody>
          <div className="animate-pulse space-y-3">
            <div className="h-4 bg-gray-300 dark:bg-gray-600 rounded w-32"></div>
            <div className="h-4 bg-gray-300 dark:bg-gray-600 rounded w-full"></div>
          </div>
        </CardBody>
      </Card>
    );
  }

  return (
    <Card className={className}>
      <CardHeader>
        <div className="flex items-center justify-between">
          <h3 className="text-lg font-semibold text-gray-900 dark:text-white">
            API Tokens
          </h3>
          <Button
            size="sm"
            onClick={() => setShowCreateForm(!showCreateForm)}
          >
            {showCreateForm ? 'Cancel' : '+ New Token'}
          </Button>
        </div>
      </CardHeader>
      <CardBody>
        {showCreateForm && (
          <div className="mb-4 p-4 bg-gray-50 dark:bg-gray-900 rounded-lg">
            <Input
              label="Label (optional)"
              placeholder="e.g., automation-key-1"
              value={label}
              onChange={(e) => setLabel(e.target.value)}
              className="mb-3"
            />
            <Button
              onClick={handleCreateToken}
              loading={creating}
              className="w-full"
            >
              Create Token
            </Button>
          </div>
        )}

        <div className="space-y-3">
          {tokens.length === 0 ? (
            <p className="text-gray-500 dark:text-gray-400 text-center py-4">
              No tokens found. Create one to get started.
            </p>
          ) : (
            tokens.map((token, index) => (
              <div
                key={index}
                className="flex items-center justify-between p-3 bg-gray-50 dark:bg-gray-900 rounded-lg"
              >
                <div className="flex-1 min-w-0 mr-3">
                  <div className="flex items-center gap-2">
                    <p className="text-sm font-mono text-gray-900 dark:text-white truncate">
                      {token.token.substring(0, 20)}...
                    </p>
                    <button
                      onClick={() => copyToClipboard(token.token)}
                      className="text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200"
                      title="Copy token"
                    >
                      {copiedToken === token.token ? '✓' : '📋'}
                    </button>
                  </div>
                  <p className="text-xs text-gray-500 dark:text-gray-400 mt-1">
                    Expires: {new Date(token.expires_at).toLocaleDateString()}
                  </p>
                </div>
                <Button
                  variant="danger"
                  size="sm"
                  onClick={() => handleRevokeToken(token.token)}
                >
                  Revoke
                </Button>
              </div>
            ))
          )}
        </div>
      </CardBody>
    </Card>
  );
}
