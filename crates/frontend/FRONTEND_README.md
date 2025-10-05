# Kanari Oracle Frontend

A beautiful, modern web interface for the Kanari Oracle API built with Next.js 15, React 19, and Tailwind CSS 4.

## Features

- 🎨 **Modern UI** - Beautiful gradient designs with dark mode support
- 🔐 **Authentication** - Secure user registration and login
- 📊 **Real-time Prices** - Live cryptocurrency and stock price feeds
- 🔑 **Token Management** - Create and manage API tokens
- 👤 **Profile Management** - Change password and manage account
- 📱 **Responsive** - Works perfectly on all devices
- ⚡ **Fast** - Built with Next.js 15 Turbopack for blazing-fast performance

## Tech Stack

- **Framework**: Next.js 15.5.4 with Turbopack
- **React**: 19.1.0
- **Styling**: Tailwind CSS 4
- **TypeScript**: Full type safety
- **API Integration**: Custom REST API client

## Getting Started

### Prerequisites

- Node.js 18+ or Bun
- Kanari Oracle API server running (default: http://localhost:3000)

### Installation

1. Navigate to the frontend directory:
```bash
cd crates/frontend
```

2. Install dependencies:
```bash
bun install
# or
npm install
```

3. Create environment file:
```bash
cp .env.example .env.local
```

4. Update `.env.local` with your API URL:
```
NEXT_PUBLIC_API_URL=http://localhost:3000
```

### Development

Start the development server:

```bash
bun dev
# or
npm run dev
```

Open [http://localhost:3001](http://localhost:3001) in your browser.

### Build for Production

```bash
bun run build
bun start
# or
npm run build
npm start
```

## Project Structure

```
src/
├── app/                    # Next.js app router pages
│   ├── dashboard/         # Dashboard page
│   ├── login/             # Login page
│   ├── register/          # Registration page
│   ├── prices/            # Price feed page
│   ├── profile/           # Profile settings page
│   ├── tokens/            # Token management page
│   ├── layout.tsx         # Root layout with AuthProvider
│   └── page.tsx           # Home/landing page
├── components/
│   ├── ui/                # Reusable UI components
│   │   ├── Button.tsx     # Button component
│   │   ├── Card.tsx       # Card components
│   │   └── Input.tsx      # Input component
│   ├── widgets/           # Feature widgets
│   │   ├── PriceWidget.tsx    # Single price display
│   │   ├── PriceGrid.tsx      # Price grid
│   │   ├── StatsWidget.tsx    # Statistics widget
│   │   └── TokenManager.tsx   # Token management widget
│   └── Navbar.tsx         # Navigation bar
├── contexts/
│   └── AuthContext.tsx    # Authentication context
└── lib/
    └── api.ts             # API client
```

## Pages

### Public Pages

- **Home (/)** - Landing page with features and CTA
- **Login (/login)** - User login
- **Register (/register)** - New user registration

### Protected Pages (Require Authentication)

- **Dashboard (/dashboard)** - Overview with stats and featured prices
- **Prices (/prices)** - Full price feed for crypto and stocks
- **Tokens (/tokens)** - API token management
- **Profile (/profile)** - Account settings and password management

## Components

### UI Components

All components support dark mode and are fully responsive.

#### Button
```tsx
import { Button } from '@/components/ui/Button';

<Button variant="primary" size="lg" loading={false}>
  Click Me
</Button>
```

Variants: `primary`, `secondary`, `danger`, `ghost`
Sizes: `sm`, `md`, `lg`

#### Input
```tsx
import { Input } from '@/components/ui/Input';

<Input
  label="Email"
  type="email"
  placeholder="Enter email"
  error="Invalid email"
  icon={<span>📧</span>}
/>
```

#### Card
```tsx
import { Card, CardHeader, CardBody, CardFooter } from '@/components/ui/Card';

<Card hover>
  <CardHeader>Title</CardHeader>
  <CardBody>Content</CardBody>
  <CardFooter>Footer</CardFooter>
</Card>
```

### Widgets

#### PriceWidget
Display a single asset price with auto-refresh (30s):
```tsx
import { PriceWidget } from '@/components/widgets/PriceWidget';

<PriceWidget assetType="crypto" symbol="bitcoin" />
```

#### PriceGrid
Display all prices in a grid:
```tsx
import { PriceGrid } from '@/components/widgets/PriceGrid';

<PriceGrid assetType="crypto" />
```

#### StatsWidget
Display Oracle statistics:
```tsx
import { StatsWidget } from '@/components/widgets/StatsWidget';

<StatsWidget />
```

#### TokenManager
Complete token management interface:
```tsx
import { TokenManager } from '@/components/widgets/TokenManager';

<TokenManager />
```

## API Integration

The frontend uses a custom API client (`src/lib/api.ts`) that handles:

- Token storage in localStorage
- Automatic token injection in headers
- Request/response error handling
- TypeScript types for all endpoints

### Using the API Client

```tsx
import { api } from '@/lib/api';

// Login
const response = await api.login(username, password);

// Get prices
const prices = await api.getAllPrices('crypto');

// Create token
const token = await api.createToken('my-token');
```

## Authentication

The app uses a React Context (`AuthContext`) for global authentication state:

```tsx
import { useAuth } from '@/contexts/AuthContext';

function MyComponent() {
  const { user, login, logout, loading } = useAuth();
  
  if (loading) return <div>Loading...</div>;
  if (!user) return <div>Not logged in</div>;
  
  return <div>Welcome {user.username}</div>;
}
```

## Styling

The app uses Tailwind CSS 4 with:

- Custom color palette (purple/blue/pink gradients)
- Dark mode support
- Responsive breakpoints
- Custom animations
- Glass morphism effects

### Color Scheme

- Primary: Purple (#9333ea) to Blue (#2563eb)
- Secondary: Gray scale
- Danger: Red (#dc2626) to Pink (#db2777)

## Performance

- **Turbopack**: Next.js 15's new bundler for faster builds
- **React 19**: Latest React with improved performance
- **Code Splitting**: Automatic code splitting per page
- **Image Optimization**: Next.js automatic image optimization
- **Caching**: API responses cached in memory

## Browser Support

- Chrome/Edge (latest)
- Firefox (latest)
- Safari (latest)
- Mobile browsers (iOS Safari, Chrome Mobile)

## Deployment

### Vercel (Recommended)

```bash
bun run build
# Deploy to Vercel
```

### Docker

```dockerfile
FROM node:18-alpine
WORKDIR /app
COPY package.json bun.lock ./
RUN npm install
COPY . .
RUN npm run build
CMD ["npm", "start"]
```

### Environment Variables

Required for production:
- `NEXT_PUBLIC_API_URL`: Your API server URL

## Development Tips

1. **Hot Reload**: Turbopack provides instant hot reload
2. **Type Safety**: Use TypeScript types from `src/lib/api.ts`
3. **Dark Mode**: Test both light and dark themes
4. **Responsive**: Test on mobile, tablet, and desktop
5. **Error Handling**: All API calls include error handling

## Troubleshooting

### API Connection Issues

If you can't connect to the API:

1. Check that the API server is running on port 3000
2. Verify `NEXT_PUBLIC_API_URL` in `.env.local`
3. Check browser console for CORS errors

### Build Errors

```bash
# Clear cache and rebuild
rm -rf .next
bun run build
```

### Token Issues

If authentication stops working:

1. Clear localStorage in browser DevTools
2. Login again to get a new token
3. Check token expiration (30 days)

## Contributing

1. Follow the existing code style
2. Use TypeScript for type safety
3. Test on multiple screen sizes
4. Test both light and dark modes
5. Ensure accessibility

## License

Same as parent Kanari Oracle project

## Support

For issues or questions, check the main Kanari Oracle documentation.
