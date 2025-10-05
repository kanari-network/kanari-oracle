# Kanari Oracle UI - Quick Start Guide

## 🎨 สวยล้ำ Modern UI for Kanari Oracle API

A beautiful, gradient-rich, dark-mode-enabled frontend for your Kanari Oracle API.

## ✨ Features Implemented

### 🔐 Authentication Pages
- **Registration** (`/register`) - Beautiful signup form with validation
- **Login** (`/login`) - Sleek login interface with error handling
- **Profile** (`/profile`) - Manage account, change password, delete account

### 📊 Dashboard & Data
- **Dashboard** (`/dashboard`) - Overview with stats and featured prices
- **Prices** (`/prices`) - Real-time price grid for crypto & stocks
- **Tokens** (`/tokens`) - Full API token management interface
- **API Docs** (`/api-docs`) - Interactive API reference

### 🎁 Reusable Widgets

All widgets are in `src/components/widgets/`:

1. **PriceWidget** - Single price display with auto-refresh
2. **PriceGrid** - Grid of all prices by type
3. **StatsWidget** - Oracle statistics dashboard
4. **TokenManager** - Complete token CRUD interface
5. **ApiDocsWidget** - Interactive API documentation
6. **HealthIndicator** - Real-time API status

### 🧩 UI Components

Located in `src/components/ui/`:

1. **Button** - 4 variants (primary, secondary, danger, ghost)
2. **Input** - With label, error, and icon support
3. **Card** - Header, Body, Footer sections

## 🚀 Getting Started

### 1. Install Dependencies

```powershell
cd crates/frontend
bun install
```

### 2. Configure Environment

Copy `.env.example` to `.env.local`:

```powershell
Copy-Item .env.example .env.local
```

Edit `.env.local`:
```env
NEXT_PUBLIC_API_URL=http://localhost:3000
```

### 3. Start Development Server

```powershell
bun dev
```

Open http://localhost:3001

### 4. Start API Server

In another terminal:

```powershell
cd ../..
cargo run -- server
```

## 📱 Pages Overview

| Route | Description | Auth Required |
|-------|-------------|---------------|
| `/` | Landing page with features | ❌ |
| `/register` | Create new account | ❌ |
| `/login` | Login to account | ❌ |
| `/dashboard` | Main dashboard | ✅ |
| `/prices` | Price feed (crypto/stocks) | ✅ |
| `/tokens` | Manage API tokens | ✅ |
| `/api-docs` | API documentation | ❌ |
| `/profile` | Account settings | ✅ |

## 🎨 Design Features

### Color Palette
- **Primary Gradient**: Purple (#9333ea) → Blue (#2563eb)
- **Secondary Gradient**: Pink → Red for danger actions
- **Background**: Subtle gray gradients
- **Dark Mode**: Full support with proper contrast

### Visual Elements
- 🌈 Gradient backgrounds and buttons
- 💎 Glass-morphism effects
- 🎯 Smooth animations and transitions
- 📱 Fully responsive design
- 🌙 Beautiful dark mode

## 🔧 Using Components

### Buttons

```tsx
import { Button } from '@/components/ui';

// Primary gradient button
<Button variant="primary" size="lg">
  Get Started
</Button>

// Danger button
<Button variant="danger" onClick={handleDelete}>
  Delete Account
</Button>

// With loading state
<Button loading={isLoading}>
  Submit
</Button>
```

### Inputs

```tsx
import { Input } from '@/components/ui';

<Input
  label="Username"
  type="text"
  placeholder="Enter username"
  icon={<span>👤</span>}
  error={errors.username}
  value={username}
  onChange={(e) => setUsername(e.target.value)}
/>
```

### Cards

```tsx
import { Card, CardHeader, CardBody } from '@/components/ui';

<Card hover>
  <CardHeader>
    <h2>Title</h2>
  </CardHeader>
  <CardBody>
    <p>Content goes here</p>
  </CardBody>
</Card>
```

### Price Widget

```tsx
import { PriceWidget } from '@/components/widgets';

// Show Bitcoin price
<PriceWidget assetType="crypto" symbol="bitcoin" />

// Show Apple stock
<PriceWidget assetType="stock" symbol="AAPL" />
```

### Price Grid

```tsx
import { PriceGrid } from '@/components/widgets';

// Show all crypto prices
<PriceGrid assetType="crypto" />

// Show all stock prices
<PriceGrid assetType="stock" />
```

## 🎯 API Integration

The frontend uses a custom API client (`src/lib/api.ts`):

```tsx
import { api } from '@/lib/api';

// Register
const result = await api.register('username', 'password', 'email@example.com');

// Login
const result = await api.login('username', 'password');

// Get prices
const prices = await api.getAllPrices('crypto');

// Create token
const token = await api.createToken('my-automation-key');

// Get profile
const profile = await api.getProfile();
```

## 🔐 Authentication

Use the Auth Context:

```tsx
import { useAuth } from '@/contexts/AuthContext';

function MyComponent() {
  const { user, login, logout, loading } = useAuth();

  if (loading) return <div>Loading...</div>;
  if (!user) return <div>Please login</div>;

  return <div>Welcome {user.username}!</div>;
}
```

## 🎨 Styling Tips

### Gradients
```tsx
// Purple to Blue
className="bg-gradient-to-r from-purple-600 to-blue-600"

// With transparency
className="bg-gradient-to-br from-purple-500/10 via-blue-500/10 to-pink-500/10"
```

### Dark Mode
```tsx
// Always specify both light and dark variants
className="bg-white dark:bg-gray-800 text-gray-900 dark:text-white"
```

### Hover Effects
```tsx
className="transition-all duration-200 hover:scale-105 hover:shadow-xl"
```

## 📦 Building for Production

```powershell
# Build
bun run build

# Start production server
bun start
```

## 🎯 Testing the UI

1. **Register a new user**
   - Go to `/register`
   - Fill in username, email, password
   - Click "Create Account"

2. **View Dashboard**
   - Automatic redirect after registration
   - See stats, featured prices, quick actions

3. **Manage Tokens**
   - Click "Manage Tokens" or go to `/tokens`
   - Create new token with optional label
   - Copy token for API usage
   - Revoke old tokens

4. **View Prices**
   - Go to `/prices`
   - Toggle between Crypto and Stocks
   - Auto-refresh every 30 seconds

5. **Change Password**
   - Go to `/profile`
   - Scroll to "Change Password"
   - Enter current and new password

## 🐛 Troubleshooting

### API Connection Issues
```powershell
# Check if API is running
curl http://localhost:3000/health

# Verify environment variable
echo $env:NEXT_PUBLIC_API_URL
```

### Token Issues
- Clear browser localStorage
- Login again to get fresh token
- Check token expiration (30 days)

### Build Errors
```powershell
# Clear cache
Remove-Item -Recurse -Force .next

# Reinstall dependencies
Remove-Item -Recurse -Force node_modules
bun install
```

## 🎨 Customization

### Change Colors

Edit Tailwind config or use inline classes:

```tsx
// Change primary gradient
<Button className="bg-gradient-to-r from-pink-600 to-purple-600">
  Custom Color
</Button>
```

### Add New Widgets

1. Create in `src/components/widgets/YourWidget.tsx`
2. Export from `src/components/widgets/index.ts`
3. Use anywhere: `import { YourWidget } from '@/components/widgets'`

## 🌟 Best Features

1. **Auto-refresh** - All price data updates every 30s
2. **Dark Mode** - Beautiful dark theme throughout
3. **Responsive** - Perfect on mobile, tablet, desktop
4. **Type-safe** - Full TypeScript coverage
5. **Error Handling** - Graceful error messages
6. **Loading States** - Skeleton loaders and spinners
7. **Copy to Clipboard** - Quick copy for tokens and code
8. **Health Indicator** - Real-time API status in navbar

## 📚 Learn More

- Full API Documentation: `/api-docs` page
- Component examples: Check each page's source code
- Widget customization: See widget source files

## 🎉 You're All Set!

Enjoy your beautiful, modern, สวยล้ำ UI for Kanari Oracle! 🚀✨
