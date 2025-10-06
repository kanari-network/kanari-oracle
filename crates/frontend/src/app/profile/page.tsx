'use client';

import React, { useEffect, useState } from 'react';
import { useRouter } from 'next/navigation';
import { useAuth } from '@/contexts/AuthContext';
import { api } from '@/lib/api';
import { Card, CardHeader, CardBody } from '@/components/ui/Card';
import { Button } from '@/components/ui/Button';
import { Input } from '@/components/ui/Input';

export default function ProfilePage() {
  const { user, loading, logout, refreshProfile } = useAuth();
  const router = useRouter();
  const [currentPassword, setCurrentPassword] = useState('');
  const [newPassword, setNewPassword] = useState('');
  const [confirmPassword, setConfirmPassword] = useState('');
  const [deletePassword, setDeletePassword] = useState('');
  const [error, setError] = useState('');
  const [success, setSuccess] = useState('');
  const [isChangingPassword, setIsChangingPassword] = useState(false);
  const [isDeletingAccount, setIsDeletingAccount] = useState(false);
  const [newEmailValue, setNewEmailValue] = useState(user?.email || '');
  const [isChangingEmail, setIsChangingEmail] = useState(false);

  useEffect(() => {
    if (!loading && !user) {
      router.push('/login');
    }
  }, [user, loading, router]);

  const handleChangePassword = async (e: React.FormEvent) => {
    e.preventDefault();
    setError('');
    setSuccess('');

    if (newPassword !== confirmPassword) {
      setError('New passwords do not match');
      return;
    }

    if (newPassword.length < 6) {
      setError('Password must be at least 6 characters');
      return;
    }

    setIsChangingPassword(true);

    const response = await api.changePassword(currentPassword, newPassword);

    if (response.success) {
      setSuccess('Password changed successfully!');
      setCurrentPassword('');
      setNewPassword('');
      setConfirmPassword('');
    } else {
      setError(response.error || 'Failed to change password');
    }

    setIsChangingPassword(false);
  };

  const handleChangeEmail = async (e: React.FormEvent) => {
    e.preventDefault();
    setError('');
    setSuccess('');

    if (!newEmailValue) {
      setError('Please enter a valid email');
      return;
    }

    setIsChangingEmail(true);

    const response = await api.changeEmail(currentPassword, newEmailValue);

    if (response.success) {
      setSuccess('Email updated successfully!');
      setCurrentPassword('');
      await refreshProfile();
    } else {
      setError(response.error || 'Failed to update email');
    }

    setIsChangingEmail(false);
  };

  const handleDeleteAccount = async (e: React.FormEvent) => {
    e.preventDefault();
    
    if (!confirm('Are you sure you want to delete your account? This action cannot be undone.')) {
      return;
    }

    setError('');
    setIsDeletingAccount(true);

    const response = await api.deleteAccount(deletePassword);

    if (response.success) {
      alert('Account deleted successfully');
      logout();
      router.push('/');
    } else {
      setError(response.error || 'Failed to delete account');
      setIsDeletingAccount(false);
    }
  };

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
      <div className="max-w-4xl mx-auto space-y-8">
        <div>
          <h1 className="text-3xl sm:text-4xl font-bold text-gray-900 dark:text-white mb-2">
            Profile Settings
          </h1>
          <p className="text-gray-600 dark:text-gray-400">
            Manage your account settings and preferences
          </p>
        </div>

        {/* Profile Information */}
        <Card>
          <CardHeader>
            <h2 className="text-xl font-semibold text-gray-900 dark:text-white">
              Profile Information
            </h2>
          </CardHeader>
          <CardBody>
            <div className="space-y-4">
              <div>
                <label className="text-sm font-medium text-gray-600 dark:text-gray-400">
                  Username
                </label>
                <p className="text-lg text-gray-900 dark:text-white mt-1">
                  {user.username}
                </p>
              </div>
              <div>
                <label className="text-sm font-medium text-gray-600 dark:text-gray-400">
                  Email
                </label>
                <p className="text-lg text-gray-900 dark:text-white mt-1">
                  {user.email || 'Not provided'}
                </p>
              </div>
              <div>
                <label className="text-sm font-medium text-gray-600 dark:text-gray-400">
                  Member Since
                </label>
                <p className="text-lg text-gray-900 dark:text-white mt-1">
                  {new Date(user.created_at).toLocaleDateString()}
                </p>
              </div>
            </div>
          </CardBody>
        </Card>

        {/* Change Password */}
        <Card>
          <CardHeader>
            <h2 className="text-xl font-semibold text-gray-900 dark:text-white">
              Change Password
            </h2>
          </CardHeader>
          <CardBody>
            <form onSubmit={handleChangePassword} className="space-y-4">
              {error && (
                <div className="p-3 bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg">
                  <p className="text-sm text-red-600 dark:text-red-400">{error}</p>
                </div>
              )}

              {success && (
                <div className="p-3 bg-green-50 dark:bg-green-900/20 border border-green-200 dark:border-green-800 rounded-lg">
                  <p className="text-sm text-green-600 dark:text-green-400">{success}</p>
                </div>
              )}

              <Input
                label="Current Password"
                type="password"
                placeholder="Enter current password"
                value={currentPassword}
                onChange={(e) => setCurrentPassword(e.target.value)}
                required
              />

              <Input
                label="New Email"
                type="email"
                placeholder="Enter new email"
                value={newEmailValue}
                onChange={(e) => setNewEmailValue(e.target.value)}
                required
              />

              <Input
                label="New Password"
                type="password"
                placeholder="Enter new password"
                value={newPassword}
                onChange={(e) => setNewPassword(e.target.value)}
                required
              />

              <Input
                label="Confirm New Password"
                type="password"
                placeholder="Confirm new password"
                value={confirmPassword}
                onChange={(e) => setConfirmPassword(e.target.value)}
                required
              />

              <Button
                type="submit"
                loading={isChangingPassword}
              >
                Change Password
              </Button>
            </form>
          </CardBody>
        </Card>

        {/* Change Email */}
        <Card>
          <CardHeader>
            <h2 className="text-xl font-semibold text-gray-900 dark:text-white">
              Change Email
            </h2>
          </CardHeader>
          <CardBody>
            <form onSubmit={handleChangeEmail} className="space-y-4">
              {error && (
                <div className="p-3 bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg">
                  <p className="text-sm text-red-600 dark:text-red-400">{error}</p>
                </div>
              )}

              {success && (
                <div className="p-3 bg-green-50 dark:bg-green-900/20 border border-green-200 dark:border-green-800 rounded-lg">
                  <p className="text-sm text-green-600 dark:text-green-400">{success}</p>
                </div>
              )}

              <Input
                label="Current Password"
                type="password"
                placeholder="Enter current password"
                value={currentPassword}
                onChange={(e) => setCurrentPassword(e.target.value)}
                required
              />

              <Input
                label="New Email"
                type="email"
                placeholder="Enter new email"
                value={newEmailValue}
                onChange={(e) => setNewEmailValue(e.target.value)}
                required
              />

              <Button
                type="submit"
                loading={isChangingEmail}
              >
                Change Email
              </Button>
            </form>
          </CardBody>
        </Card>

        {/* Delete Account */}
        <Card className="border-red-200 dark:border-red-800">
          <CardHeader>
            <h2 className="text-xl font-semibold text-red-600 dark:text-red-400">
              Danger Zone
            </h2>
          </CardHeader>
          <CardBody>
            <form onSubmit={handleDeleteAccount} className="space-y-4">
              <p className="text-gray-600 dark:text-gray-400">
                Once you delete your account, there is no going back. Please be certain.
              </p>

              <Input
                label="Confirm Password"
                type="password"
                placeholder="Enter your password to confirm"
                value={deletePassword}
                onChange={(e) => setDeletePassword(e.target.value)}
                required
              />

              <Button
                type="submit"
                variant="danger"
                loading={isDeletingAccount}
              >
                Delete Account
              </Button>
            </form>
          </CardBody>
        </Card>
      </div>
    </div>
  );
}
