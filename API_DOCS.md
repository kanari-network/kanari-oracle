# Kanari Oracle API Documentation

## Overview

Kanari Oracle provides real-time cryptocurrency and stock price data through secure HTTP API endpoints. This API is designed for web3 applications, trading bots, and financial services that need reliable and authenticated price feeds.

**Key Features:**

- Real-time crypto and stock price data
- User authentication with secure token management
- Background price updates every 30 seconds
- Multiple data sources with fallback mechanisms
- CORS support for web applications
- Rate limiting and error handling

## Quick Start

### 1. Starting the API Server

```bash
# Start the API server on default port 3000
cargo run -- server

# Start on custom port with custom config
cargo run -- server --port 8080 --config custom-config.json --interval 60
```

### 2. Database Setup

Ensure PostgreSQL is running and create a `.env` file in the project root:

```env
DATABASE_URL="postgresql://username:password@localhost:5432/kanari_db"
```

The API will automatically create required tables on first startup.

### 3. Base URL

```
http://localhost:3000
```

## Authentication

Most API endpoints require authentication using API tokens. You need to register a user account and obtain an API token first.

### User Registration

**POST** `/users/register`

Create a new user account and receive an API token.

**Request Body:**

```json
{
  "username": "your_username",
  "password": "secure_password",
  "owner_email": "user@example.com"  // optional
}
```

**Example:**

```bash
curl -X POST http://localhost:3000/users/register \
  -H "Content-Type: application/json" \
  -d '{"username":"alice","password":"secret123","owner_email":"alice@example.com"}'
```

**Response:**

```json
{
  "success": true,
  "data": {
    "token": "a868de0d-bcf8-4c9d-ba03-bf6b1d861f9a",
    "expires_at": "2025-11-02T14:30:00Z"
  },
  "error": null
}
```

### User Login

**POST** `/users/login`

Login with existing credentials to get a new API token.

**Request Body:**

```json
{
  "username": "your_username",
  "password": "your_password"
}
```

**Example:**

```bash
curl -X POST http://localhost:3000/users/login \
  -H "Content-Type: application/json" \
  -d '{"username":"alice","password":"secret123"}'
```

**PowerShell Example:**

```powershell
$body = @{ username="alice"; password="secret123" } | ConvertTo-Json
Invoke-RestMethod -Uri "http://localhost:3000/users/login" -Method Post -Body $body -ContentType "application/json"
```

**Response:**

```json
{
  "success": true,
  "data": {
    "token": "new-token-uuid-here",
    "expires_at": "2025-11-02T14:30:00Z"
  },
  "error": null
}
```

### Get User Profile

**GET** `/users/profile`

Get the current user's profile information.

**Authentication:**

- Send your API token in the Authorization header: `Authorization: Bearer <YOUR_TOKEN_HERE>`

**Example:**

```bash
curl -H "Authorization: Bearer YOUR_TOKEN_HERE" "http://localhost:3000/users/profile"
```

**PowerShell Example:**

```powershell
$headers = @{ Authorization = "Bearer YOUR_TOKEN_HERE" }
Invoke-RestMethod -Uri "http://localhost:3000/users/profile" -Headers $headers
```

**Response:**

```json
{
  "success": true,
  "data": {
    "id": 1,
    "username": "alice",
    "email": "alice@example.com",
    "created_at": "2025-10-03T14:30:00Z"
  },
  "error": null
}
```

### List All Users

**GET** `/users/list`

Get a list of all registered users (admin function).

**Authentication:**

- Requires an admin user and the Authorization header: `Authorization: Bearer <YOUR_TOKEN_HERE>`

**Example:**

```bash
curl -H "Authorization: Bearer YOUR_TOKEN_HERE" "http://localhost:3000/users/list"
```

**PowerShell Example:**

```powershell
$headers = @{ Authorization = "Bearer YOUR_TOKEN_HERE" }
Invoke-RestMethod -Uri "http://localhost:3000/users/list" -Headers $headers
```

**Response:**

```json
{
  "success": true,
  "data": {
    "users": [
      {
        "id": 1,
        "username": "alice",
        "email": "alice@example.com",
        "created_at": "2025-10-03T14:30:00Z"
      },
      {
        "id": 2,
        "username": "bob",
        "email": "bob@example.com",
        "created_at": "2025-10-03T15:00:00Z"
      }
    ],
    "total_count": 2
  },
  "error": null
}
```

### Change Password

**POST** `/users/change-password`

Change the current user's password. Requires current password confirmation.

**Authentication:**

- Send your API token in the Authorization header: `Authorization: Bearer <YOUR_TOKEN_HERE>`

**Request Body:**

```json
{
  "current_password": "current_password",
  "new_password": "new_secure_password"
}
```

**Example:**

```bash
curl -X POST "http://localhost:3000/users/change-password" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer YOUR_TOKEN_HERE" \
  -d '{"current_password":"old_password","new_password":"new_secure_password"}'
```

**PowerShell Example:**

```powershell
$headers = @{ Authorization = "Bearer YOUR_TOKEN_HERE" }
$body = @{ current_password="old_password"; new_password="new_secure_password" } | ConvertTo-Json
Invoke-RestMethod -Uri "http://localhost:3000/users/change-password" -Method Post -Body $body -Headers $headers -ContentType "application/json"
```

**Response:**

```json
{
  "success": true,
  "data": "Password changed successfully",
  "error": null
}
```

**Security Notes:**

- Requires current password verification
- New password is hashed using Argon2id
- All existing tokens remain valid after password change
- Use strong passwords for better security

### Delete User Account

**POST** `/users/delete`

Delete the current user's account permanently. Requires password confirmation.

**Authentication:**

- Send your API token in the Authorization header: `Authorization: Bearer <YOUR_TOKEN_HERE>`

**Request Body:**

```json
{
  "password": "current_password"
}
```

**Example:**

```bash
curl -X POST "http://localhost:3000/users/delete" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer YOUR_TOKEN_HERE" \
  -d '{"password":"your_password"}'
```

**PowerShell Example:**

```powershell
$headers = @{ Authorization = "Bearer YOUR_TOKEN_HERE" }
$body = @{ password="your_password" } | ConvertTo-Json
Invoke-RestMethod -Uri "http://localhost:3000/users/delete" -Method Post -Body $body -Headers $headers -ContentType "application/json"
```

**Response:**

```json
{
  "success": true,
  "data": "Account deleted successfully",
  "error": null
}
```

**⚠️ Warning:** This action is permanent and will delete:

- User account and profile
- All associated API tokens
- Cannot be undone

### Using API Tokens

Include your token as a query parameter in all authenticated requests:

```
GET /price/crypto/bitcoin?token=YOUR_TOKEN_HERE
```

**Token Details:**

- Tokens expire after 30 days
- Each login/registration generates a new token
- Store tokens securely and refresh before expiration

## API Endpoints

### 1. Health Check (Public)

**GET** `/health`

Returns the current status of the Oracle service. No authentication required.

**Example:**

```bash
curl http://localhost:3000/health
```

**Response:**

```json
{
  "success": true,
  "data": {
    "status": "healthy",
    "last_update": "2025-10-03T14:52:59Z",
    "total_symbols": 56
  },
  "error": null
}
```

### 2. Get Specific Price (Authenticated)

**GET** `/price/{asset_type}/{symbol}`

Get the current price for a specific symbol.

**Parameters:**

- `asset_type`: "crypto" or "stock"
- `symbol`: Symbol name (e.g., "bitcoin" for crypto, "AAPL" for stocks)
- `token`: Your API token (query parameter)

**Examples:**

```bash
# Get Bitcoin price
curl -H "Authorization: Bearer YOUR_TOKEN_HERE" "http://localhost:3000/price/crypto/bitcoin"

# Get Apple stock price
curl -H "Authorization: Bearer YOUR_TOKEN_HERE" "http://localhost:3000/price/stock/AAPL"
```

**Response:**

```json
{
  "success": true,
  "data": {
    "symbol": "BITCOIN",
    "price": 120916.00,
    "timestamp": "2025-10-03T14:52:59Z",
    "asset_type": "crypto"
  },
  "error": null
}
```

### 3. Get All Prices by Type (Authenticated)

**GET** `/prices/{asset_type}`

Get all current prices for a specific asset type.

**Parameters:**

- `asset_type`: "crypto" or "stock"
- `token`: Your API token (query parameter)

**Examples:**

```bash
# Get all crypto prices
curl -H "Authorization: Bearer YOUR_TOKEN_HERE" "http://localhost:3000/prices/crypto"

# Get all stock prices
curl -H "Authorization: Bearer YOUR_TOKEN_HERE" "http://localhost:3000/prices/stock"
```

**Response:**

```json
{
  "success": true,
  "data": [
    {
      "symbol": "bitcoin",
      "price": 120916.00,
      "timestamp": "2025-10-03T14:52:59Z",
      "asset_type": "crypto"
    },
    {
      "symbol": "ethereum", 
      "price": 4483.96,
      "timestamp": "2025-10-03T14:52:59Z",
      "asset_type": "crypto"
    }
  ],
  "error": null
}
```

### 4. List Available Symbols (Authenticated)

**GET** `/symbols?asset_type={type}`

List all available symbols for trading.

**Query Parameters:**

- `token`: Your API token (required)
- `asset_type` (optional): "crypto", "stock", or omit for all

**Examples:**

```bash
# Get all symbols
curl -H "Authorization: Bearer YOUR_TOKEN_HERE" "http://localhost:3000/symbols"

# Get only crypto symbols
curl -H "Authorization: Bearer YOUR_TOKEN_HERE" "http://localhost:3000/symbols?asset_type=crypto"

# Get only stock symbols
curl -H "Authorization: Bearer YOUR_TOKEN_HERE" "http://localhost:3000/symbols?asset_type=stock"
```

**Response:**

```json
{
  "success": true,
  "data": {
    "crypto": ["bitcoin", "ethereum", "binancecoin", "ripple", "sui", "tether", "usd-coin"],
    "stocks": ["AAPL", "GOOGL", "MSFT", "TSLA", "NVDA", "META", "AMZN"]
  },
  "error": null
}
```

### 5. Get Oracle Statistics (Authenticated)

**GET** `/stats`

Get detailed statistics about the Oracle service.

**Example:**

```bash
curl -H "Authorization: Bearer YOUR_TOKEN_HERE" "http://localhost:3000/stats"
```

**Response:**

```json
{
  "success": true,
  "data": {
    "total_crypto_symbols": 7,
    "total_stock_symbols": 49,
    "last_update": "2025-10-03T14:52:59Z",
    "avg_crypto_price": 18782.02,
    "avg_stock_price": 285.67,
    "uptime_seconds": 0
  },
  "error": null
}
```

### 6. Force Update Prices (Authenticated)

**POST** `/update/{asset_type}`

Force an immediate update of price data.

**Parameters:**

- `asset_type`: "crypto", "stock", or "all"
- `token`: Your API token (query parameter)

**Examples:**

```bash
# Update crypto prices
curl -X POST -H "Authorization: Bearer YOUR_TOKEN_HERE" "http://localhost:3000/update/crypto"

# Update stock prices
curl -X POST -H "Authorization: Bearer YOUR_TOKEN_HERE" "http://localhost:3000/update/stock"

# Update all prices
curl -X POST -H "Authorization: Bearer YOUR_TOKEN_HERE" "http://localhost:3000/update/all"
```

**Response:**

```json
{
  "success": true,
  "data": "Updated 56 price feeds",
  "error": null
}
```

## SDK Examples & Integration

### Complete Workflow Example

```bash
# 1. Register a new user
curl -X POST http://localhost:3000/users/register \
  -H "Content-Type: application/json" \
  -d '{"username":"trader1","password":"secure123","owner_email":"trader@example.com"}'

# Response: {"success":true,"data":{"token":"abc-123-def","expires_at":"2025-11-02T..."}}

# 2. Use the token to get prices
curl "http://localhost:3000/price/crypto/bitcoin?token=abc-123-def"
curl "http://localhost:3000/prices/crypto?token=abc-123-def"
curl "http://localhost:3000/symbols?token=abc-123-def"
```

### JavaScript/TypeScript SDK

```javascript
class KanariOracle {
  constructor(baseUrl = 'http://localhost:3000') {
    this.baseUrl = baseUrl;
    this.token = null;
  }

  async register(username, password, email = null) {
    const response = await fetch(`${this.baseUrl}/users/register`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ username, password, owner_email: email })
    });
    const data = await response.json();
    if (data.success) {
      this.token = data.data.token;
      return data.data;
    }
    throw new Error(data.error || 'Registration failed');
  }

  async login(username, password) {
    const response = await fetch(`${this.baseUrl}/users/login`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ username, password })
    });
    const data = await response.json();
    if (data.success) {
      this.token = data.data.token;
      return data.data;
    }
    throw new Error(data.error || 'Login failed');
  }

  async getCryptoPrice(symbol) {
    if (!this.token) throw new Error('Not authenticated. Call login() or register() first.');
    const response = await fetch(`${this.baseUrl}/price/crypto/${symbol}`, {
      headers: { 'Authorization': `Bearer ${this.token}` }
    });
    const data = await response.json();
    return data.success ? data.data : null;
  }

  async getStockPrice(symbol) {
    if (!this.token) throw new Error('Not authenticated. Call login() or register() first.');
    const response = await fetch(`${this.baseUrl}/price/stock/${symbol}`, {
      headers: { 'Authorization': `Bearer ${this.token}` }
    });
    const data = await response.json();
    return data.success ? data.data : null;
  }

  async getAllCryptoPrices() {
    if (!this.token) throw new Error('Not authenticated. Call login() or register() first.');
    const response = await fetch(`${this.baseUrl}/prices/crypto`, {
      headers: { 'Authorization': `Bearer ${this.token}` }
    });
    const data = await response.json();
    return data.success ? data.data : [];
  }

  async getAllStockPrices() {
    if (!this.token) throw new Error('Not authenticated. Call login() or register() first.');
    const response = await fetch(`${this.baseUrl}/prices/stock`, {
      headers: { 'Authorization': `Bearer ${this.token}` }
    });
    const data = await response.json();
    return data.success ? data.data : [];
  }

  async getSymbols(assetType = null) {
    if (!this.token) throw new Error('Not authenticated. Call login() or register() first.');
    const url = assetType 
      ? `${this.baseUrl}/symbols?asset_type=${assetType}`
      : `${this.baseUrl}/symbols`;
    const response = await fetch(url, { headers: { 'Authorization': `Bearer ${this.token}` } });
    const data = await response.json();
    return data.success ? data.data : null;
  }

  async forceUpdate(assetType = 'all') {
    if (!this.token) throw new Error('Not authenticated. Call login() or register() first.');
    const response = await fetch(`${this.baseUrl}/update/${assetType}`, {
      method: 'POST',
      headers: { 'Authorization': `Bearer ${this.token}` }
    });
    const data = await response.json();
    return data.success;
  }

  async getStats() {
    if (!this.token) throw new Error('Not authenticated. Call login() or register() first.');
  const response = await fetch(`${this.baseUrl}/stats`, { headers: { 'Authorization': `Bearer ${this.token}` } });
    const data = await response.json();
    return data.success ? data.data : null;
  }

  async getUserProfile() {
    if (!this.token) throw new Error('Not authenticated. Call login() or register() first.');
  const response = await fetch(`${this.baseUrl}/users/profile`, { headers: { 'Authorization': `Bearer ${this.token}` } });
    const data = await response.json();
    return data.success ? data.data : null;
  }

  async listAllUsers() {
    if (!this.token) throw new Error('Not authenticated. Call login() or register() first.');
  const response = await fetch(`${this.baseUrl}/users/list`, { headers: { 'Authorization': `Bearer ${this.token}` } });
    const data = await response.json();
    return data.success ? data.data : null;
  }

  async changePassword(currentPassword, newPassword) {
    if (!this.token) throw new Error('Not authenticated. Call login() or register() first.');
    const response = await fetch(`${this.baseUrl}/users/change-password`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json', 'Authorization': `Bearer ${this.token}` },
      body: JSON.stringify({ 
        current_password: currentPassword, 
        new_password: newPassword 
      })
    });
    const data = await response.json();
    return data.success ? data.data : null;
  }

  async deleteAccount(password) {
    if (!this.token) throw new Error('Not authenticated. Call login() or register() first.');
    const response = await fetch(`${this.baseUrl}/users/delete`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json', 'Authorization': `Bearer ${this.token}` },
      body: JSON.stringify({ password })
    });
    const data = await response.json();
    if (data.success) {
      this.token = null; // Clear token since account is deleted
    }
    return data.success ? data.data : null;
  }
}

// Usage Example
async function example() {
  const oracle = new KanariOracle();
  
  try {
    // Register or login
    await oracle.register('trader1', 'secure123', 'trader@example.com');
    // Or: await oracle.login('existing_user', 'password');
    
    // Get individual prices
    const btcPrice = await oracle.getCryptoPrice('bitcoin');
    console.log(`BTC Price: $${btcPrice.price}`);
    
    // Get all crypto prices
    const allCrypto = await oracle.getAllCryptoPrices();
    console.log(`Found ${allCrypto.length} crypto prices`);
    
    // Get symbols
    const symbols = await oracle.getSymbols();
    console.log(`Crypto symbols: ${symbols.crypto.join(', ')}`);
    
    // Force update
    const updated = await oracle.forceUpdate('crypto');
    console.log(`Update successful: ${updated}`);
    
  } catch (error) {
    console.error('API Error:', error.message);
  }
}
```

### Python SDK

```python
import requests
from typing import Optional, Dict, List, Any

class KanariOracle:
    def __init__(self, base_url: str = 'http://localhost:3000'):
        self.base_url = base_url
        self.token: Optional[str] = None

    def register(self, username: str, password: str, email: Optional[str] = None) -> Dict[str, Any]:
        """Register a new user and get API token"""
        payload = {"username": username, "password": password}
        if email:
            payload["owner_email"] = email
            
        response = requests.post(
            f'{self.base_url}/users/register',
            json=payload,
            headers={'Content-Type': 'application/json'}
        )
        data = response.json()
        if data['success']:
            self.token = data['data']['token']
            return data['data']
        raise Exception(data.get('error', 'Registration failed'))

    def login(self, username: str, password: str) -> Dict[str, Any]:
        """Login with existing credentials"""
        response = requests.post(
            f'{self.base_url}/users/login',
            json={"username": username, "password": password},
            headers={'Content-Type': 'application/json'}
        )
        data = response.json()
        if data['success']:
            self.token = data['data']['token']
            return data['data']
        raise Exception(data.get('error', 'Login failed'))

    def _check_auth(self):
        if not self.token:
            raise Exception('Not authenticated. Call login() or register() first.')

    def get_crypto_price(self, symbol: str) -> Optional[Dict[str, Any]]:
        """Get current price for a cryptocurrency"""
        self._check_auth()
  response = requests.get(f'{self.base_url}/price/crypto/{symbol}', headers={'Authorization': f'Bearer {self.token}'})
        data = response.json()
        return data['data'] if data['success'] else None

    def get_stock_price(self, symbol: str) -> Optional[Dict[str, Any]]:
        """Get current price for a stock"""
        self._check_auth()
  response = requests.get(f'{self.base_url}/price/stock/{symbol}', headers={'Authorization': f'Bearer {self.token}'})
        data = response.json()
        return data['data'] if data['success'] else None

    def get_all_crypto_prices(self) -> List[Dict[str, Any]]:
        """Get all cryptocurrency prices"""
        self._check_auth()
  response = requests.get(f'{self.base_url}/prices/crypto', headers={'Authorization': f'Bearer {self.token}'})
        data = response.json()
        return data['data'] if data['success'] else []

    def get_all_stock_prices(self) -> List[Dict[str, Any]]:
        """Get all stock prices"""
        self._check_auth()
  response = requests.get(f'{self.base_url}/prices/stock', headers={'Authorization': f'Bearer {self.token}'})
        data = response.json()
        return data['data'] if data['success'] else []

    def get_symbols(self, asset_type: Optional[str] = None) -> Optional[Dict[str, List[str]]]:
        """Get available symbols"""
        self._check_auth()
    url = f'{self.base_url}/symbols'
    if asset_type:
      url += f'?asset_type={asset_type}'
    response = requests.get(url, headers={'Authorization': f'Bearer {self.token}'})
        data = response.json()
        return data['data'] if data['success'] else None

    def force_update(self, asset_type: str = 'all') -> bool:
        """Force immediate price update"""
        self._check_auth()
  response = requests.post(f'{self.base_url}/update/{asset_type}', headers={'Authorization': f'Bearer {self.token}'})
        data = response.json()
        return data['success']

    def get_stats(self) -> Optional[Dict[str, Any]]:
        """Get oracle statistics"""
        self._check_auth()
  response = requests.get(f'{self.base_url}/stats', headers={'Authorization': f'Bearer {self.token}'})
        data = response.json()
        return data['data'] if data['success'] else None

    def get_user_profile(self) -> Optional[Dict[str, Any]]:
        """Get current user profile"""
        self._check_auth()
  response = requests.get(f'{self.base_url}/users/profile', headers={'Authorization': f'Bearer {self.token}'})
        data = response.json()
        return data['data'] if data['success'] else None

    def list_all_users(self) -> Optional[Dict[str, Any]]:
        """List all users (admin function)"""
        self._check_auth()
  response = requests.get(f'{self.base_url}/users/list', headers={'Authorization': f'Bearer {self.token}'})
        data = response.json()
        return data['data'] if data['success'] else None

    def change_password(self, current_password: str, new_password: str) -> bool:
        """Change current user's password"""
        self._check_auth()
    response = requests.post(
      f'{self.base_url}/users/change-password',
      json={
        "current_password": current_password,
        "new_password": new_password
      },
      headers={'Content-Type': 'application/json', 'Authorization': f'Bearer {self.token}'}
    )
        data = response.json()
        return data['success']

    def delete_account(self, password: str) -> bool:
        """Delete current user account permanently"""
        self._check_auth()
    response = requests.post(
      f'{self.base_url}/users/delete',
      json={"password": password},
      headers={'Content-Type': 'application/json', 'Authorization': f'Bearer {self.token}'}
    )
        data = response.json()
        if data['success']:
            self.token = None  # Clear token since account is deleted
        return data['success']

# Usage Example
def main():
    oracle = KanariOracle()
    
    try:
        # Register or login
        auth_data = oracle.register('trader1', 'secure123', 'trader@example.com')
        print(f"Authenticated. Token expires: {auth_data['expires_at']}")
        
        # Get individual prices
        btc_price = oracle.get_crypto_price('bitcoin')
        if btc_price:
            print(f"BTC Price: ${btc_price['price']:,.2f}")
        
        # Get all crypto prices
        all_crypto = oracle.get_all_crypto_prices()
        print(f"Found {len(all_crypto)} crypto prices")
        
        # Get symbols
        symbols = oracle.get_symbols()
        if symbols:
            print(f"Available crypto: {', '.join(symbols['crypto'])}")
            print(f"Available stocks: {', '.join(symbols['stocks'][:5])}...")  # Show first 5
        
        # Get stats
        stats = oracle.get_stats()
        if stats:
            print(f"Total symbols: {stats['total_crypto_symbols']} crypto, {stats['total_stock_symbols']} stocks")
            
    except Exception as e:
        print(f"Error: {e}")

if __name__ == "__main__":
    main()
```

### Rust SDK

```rust
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenResponse {
    pub token: String,
    pub expires_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PriceData {
    pub symbol: String,
    pub price: f64,
    pub timestamp: String,
    pub asset_type: String,
}

pub struct KanariOracle {
    client: Client,
    base_url: String,
    token: Option<String>,
}

impl KanariOracle {
    pub fn new(base_url: Option<String>) -> Self {
        Self {
            client: Client::new(),
            base_url: base_url.unwrap_or_else(|| "http://localhost:3000".to_string()),
            token: None,
        }
    }

    pub async fn register(&mut self, username: &str, password: &str, email: Option<&str>) 
        -> Result<TokenResponse, Box<dyn std::error::Error>> {
        let mut payload = serde_json::json!({
            "username": username,
            "password": password
        });
        
        if let Some(email) = email {
            payload["owner_email"] = serde_json::Value::String(email.to_string());
        }

        let response: ApiResponse<TokenResponse> = self.client
            .post(&format!("{}/users/register", self.base_url))
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await?
            .json()
            .await?;

        if response.success {
            let token_data = response.data.unwrap();
            self.token = Some(token_data.token.clone());
            Ok(token_data)
        } else {
            Err(response.error.unwrap_or_else(|| "Registration failed".to_string()).into())
        }
    }

    pub async fn login(&mut self, username: &str, password: &str) 
        -> Result<TokenResponse, Box<dyn std::error::Error>> {
        let payload = serde_json::json!({
            "username": username,
            "password": password
        });

        let response: ApiResponse<TokenResponse> = self.client
            .post(&format!("{}/users/login", self.base_url))
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await?
            .json()
            .await?;

        if response.success {
            let token_data = response.data.unwrap();
            self.token = Some(token_data.token.clone());
            Ok(token_data)
        } else {
            Err(response.error.unwrap_or_else(|| "Login failed".to_string()).into())
        }
    }

    fn check_auth(&self) -> Result<&str, Box<dyn std::error::Error>> {
        self.token.as_ref()
            .ok_or_else(|| "Not authenticated. Call login() or register() first.".into())
    }

    pub async fn get_crypto_price(&self, symbol: &str) -> Result<Option<PriceData>, Box<dyn std::error::Error>> {
        let token = self.check_auth()?;
  let url = format!("{}/price/crypto/{}", self.base_url, symbol);
        
    let response: ApiResponse<PriceData> = self.client
      .get(&url)
      .header("Authorization", format!("Bearer {}", token))
      .send()
      .await?
      .json()
      .await?;

        Ok(response.data)
    }

    pub async fn get_stock_price(&self, symbol: &str) -> Result<Option<PriceData>, Box<dyn std::error::Error>> {
        let token = self.check_auth()?;
  let url = format!("{}/price/stock/{}", self.base_url, symbol);
        
    let response: ApiResponse<PriceData> = self.client
      .get(&url)
      .header("Authorization", format!("Bearer {}", token))
      .send()
      .await?
      .json()
      .await?;

        Ok(response.data)
    }

    pub async fn change_password(&self, current_password: &str, new_password: &str) -> Result<bool, Box<dyn std::error::Error>> {
        let token = self.check_auth()?;
  let url = format!("{}/users/change-password", self.base_url);
        
        #[derive(Serialize)]
        struct ChangePasswordPayload<'a> {
            current_password: &'a str,
            new_password: &'a str,
        }

        let payload = ChangePasswordPayload {
            current_password,
            new_password,
        };

    let response: ApiResponse<String> = self.client
      .post(&url)
      .header("Authorization", format!("Bearer {}", token))
      .json(&payload)
      .send()
      .await?
      .json()
      .await?;

        Ok(response.success)
    }

    pub async fn force_update(&self, asset_type: &str) -> Result<bool, Box<dyn std::error::Error>> {
        let token = self.check_auth()?;
  let url = format!("{}/update/{}", self.base_url, asset_type);
        
    let response: ApiResponse<String> = self.client
      .post(&url)
      .header("Authorization", format!("Bearer {}", token))
      .send()
      .await?
      .json()
      .await?;

        Ok(response.success)
    }
}

// Usage example
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut oracle = KanariOracle::new(None);
    
    // Register or login
    let auth_data = oracle.register("trader1", "secure123", Some("trader@example.com")).await?;
    println!("Authenticated. Token expires: {}", auth_data.expires_at);
    
    // Get Bitcoin price
    if let Some(btc_price) = oracle.get_crypto_price("bitcoin").await? {
        println!("BTC Price: ${:.2}", btc_price.price);
    }
    
    // Force update
    let updated = oracle.force_update("crypto").await?;
    println!("Update successful: {}", updated);
    
    Ok(())
}
```

## Error Handling

### Error Response Format

When an error occurs, the API returns a standardized error response:

```json
{
  "success": false,
  "data": null,
  "error": "Detailed error description here"
}
```

### Common Error Types

**Authentication Errors:**

- `"Missing token query parameter"` - Token not provided
- `"Invalid or expired token"` - Token is invalid or has expired
- `"Invalid username or password"` - Login credentials incorrect

**Validation Errors:**

- `"Invalid asset type. Use 'crypto' or 'stock'"` - Wrong asset type specified
- `"Symbol 'XYZ' not configured for crypto"` - Symbol not available

**Database Errors:**

- `"error returned from database: relation \"users\" does not exist"` - Database not initialized
- `"error returned from database: password authentication failed"` - Database connection issues

**Rate Limiting:**

- Price fetching may fail if external APIs are rate limited; the system uses fallback APIs automatically

### HTTP Status Codes

- **200 OK**: Request processed (check `success` field in response body)
- **500 Internal Server Error**: Unexpected server error

## Configuration

### Environment Variables

Create a `.env` file in the project root:

```env
DATABASE_URL="postgresql://username:password@localhost:5432/kanari_db"
```

### API Configuration File

The API server uses the same configuration file as the CLI tool for external API keys:

```json
{
  "crypto": {
    "coingecko_api_key": "your_coingecko_api_key",
    "binance_api_key": "your_binance_api_key", 
    "binance_secret_key": "your_binance_secret_key",
    "symbols": ["bitcoin", "ethereum", "binancecoin", "ripple"]
  },
  "stocks": {
    "alpha_vantage_api_key": "your_alpha_vantage_key",
    "finnhub_api_key": "your_finnhub_key",
    "symbols": ["AAPL", "GOOGL", "MSFT", "TSLA"]
  },
  "general": {
    "request_timeout": 30,
    "max_retries": 3,
    "retry_delay": 1000,
    "enable_logging": true
  }
}
```

## System Features

### Security

- **Password Hashing**: Uses Argon2id for secure password storage
- **Token Management**: JWT-like tokens with expiration (30 days)
- **Database Security**: PostgreSQL with prepared statements (SQL injection protection)
- **CORS Support**: Configurable cross-origin resource sharing

### Performance & Reliability

- **Background Updates**: Automatic price updates every 30 seconds
- **Connection Pooling**: Database connection pooling with configurable limits
- **Fallback APIs**: Multiple data sources with automatic fallback
- **Caching**: In-memory price caching for fast response times
- **Concurrent Updates**: Async/await architecture for high performance

### Rate Limiting

The Oracle respects upstream API rate limits and implements fallback mechanisms:

**Cryptocurrency APIs:**

- **CoinGecko**: 10-50 calls/minute (free), higher limits with API key
- **Binance**: 1200 requests per minute

**Stock APIs:**

- **Alpha Vantage**: 5 calls per minute (free tier), 500/minute (premium)
- **Finnhub**: 60 calls per minute (free tier), higher limits with API key
- **Yahoo Finance**: Used as fallback (no API key required)

### Database Schema

The API automatically creates these tables on startup:

```sql
-- Users table
CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    username VARCHAR(255) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    email VARCHAR(255),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- API tokens table  
CREATE TABLE api_tokens (
    id SERIAL PRIMARY KEY,
    token VARCHAR(255) UNIQUE NOT NULL,
    owner VARCHAR(255) NOT NULL,
    expires_at TIMESTAMP NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    FOREIGN KEY (owner) REFERENCES users(username) ON DELETE CASCADE
);
```

## Deployment

### Production Setup

1. **Database**: Set up PostgreSQL with proper credentials
2. **Environment**: Configure `.env` with production database URL
3. **API Keys**: Add external API keys to `config.json` for better rate limits
4. **Monitoring**: Enable logging and monitor error rates
5. **Security**: Use HTTPS, secure database access, and rotate tokens regularly

### Docker Deployment

```dockerfile
FROM rust:1.70 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/kanari /usr/local/bin/
CMD ["kanari", "server"]
```

### Health Monitoring

Monitor the `/health` endpoint for service availability:

```bash
# Check if API is responding
curl -f http://localhost:3000/health || echo "API is down"
```

## Support & Troubleshooting

### Common Issues

1. **Database Connection Failed**
   - Verify PostgreSQL is running
   - Check DATABASE_URL in `.env` file
   - Ensure database user has proper permissions

2. **Token Expired**
   - Tokens expire after 30 days
   - Login again to get a new token
   - Implement token refresh in your client

3. **Rate Limiting**
   - Add API keys to `config.json` for higher limits
   - Implement exponential backoff in your client
   - Monitor external API status pages

4. **Price Data Missing**
   - Check if symbols are configured in `config.json`
   - Verify external API availability
   - Force update using `/update/all` endpoint

### API Status & Updates

- Price data is updated every 30 seconds automatically
- Check the `last_update` field in responses for data freshness
- Use `/health` endpoint to verify service status
- Monitor logs for API errors and rate limiting issues

For technical support or bug reports, check the project's GitHub repository.
