# Kanari Oracle API Documentation

## Overview

Kanari Oracle provides real-time cryptocurrency and stock price data through HTTP API endpoints. This API is designed for web3 applications that need reliable price feeds.

## Starting the API Server

```bash
# Start the API server on default port 3000
cargo run -- server

# Start on custom port with custom config
cargo run -- server --port 8080 --config custom-config.json --interval 60
```

## Base URL

```
http://localhost:3000
```

## API Endpoints

### 1. Health Check

**GET** `/health`

Returns the current status of the Oracle service.

**Response:**

```json
{
  "success": true,
  "data": {
    "status": "healthy",
    "last_update": "2025-10-02T10:30:00Z",
    "total_symbols": 19
  },
  "error": null
}
```

### 2. Get Specific Price

**GET** `/price/{asset_type}/{symbol}`

Get the current price for a specific symbol.

**Parameters:**

- `asset_type`: "crypto" or "stock"
- `symbol`: Symbol name (e.g., "bitcoin" for crypto, "AAPL" for stocks)

**Examples:**

```bash
# Get Bitcoin price
curl http://localhost:3000/price/crypto/bitcoin

# Get Apple stock price
curl http://localhost:3000/price/stock/AAPL
```

**Response:**

```json
{
  "success": true,
  "data": {
    "symbol": "BITCOIN",
    "price": 43250.50,
    "timestamp": "2025-10-02T10:30:00Z",
    "asset_type": "crypto"
  },
  "error": null
}
```

### 3. Get All Prices by Type

**GET** `/prices/{asset_type}`

Get all current prices for a specific asset type.

**Parameters:**

- `asset_type`: "crypto" or "stock"

**Examples:**

```bash
# Get all crypto prices
curl http://localhost:3000/prices/crypto

# Get all stock prices
curl http://localhost:3000/prices/stock
```

**Response:**

```json
{
  "success": true,
  "data": [
    {
      "symbol": "bitcoin",
      "price": 43250.50,
      "timestamp": "2025-10-02T10:30:00Z",
      "asset_type": "crypto"
    },
    {
      "symbol": "ethereum",
      "price": 2650.75,
      "timestamp": "2025-10-02T10:30:00Z",
      "asset_type": "crypto"
    }
  ],
  "error": null
}
```

### 4. List Available Symbols

**GET** `/symbols?asset_type={type}`

List all available symbols for trading.

**Query Parameters:**

- `asset_type` (optional): "crypto", "stock", or omit for all

**Examples:**

```bash
# Get all symbols
curl http://localhost:3000/symbols

# Get only crypto symbols
curl http://localhost:3000/symbols?asset_type=crypto

# Get only stock symbols
curl http://localhost:3000/symbols?asset_type=stock
```

**Response:**

```json
{
  "success": true,
  "data": {
    "crypto": ["bitcoin", "ethereum", "binancecoin", "cardano"],
    "stocks": ["AAPL", "GOOGL", "MSFT", "TSLA"]
  },
  "error": null
}
```

### 5. Get Oracle Statistics

**GET** `/stats`

Get detailed statistics about the Oracle service.

**Response:**

```json
{
  "success": true,
  "data": {
    "total_crypto_symbols": 9,
    "total_stock_symbols": 10,
    "last_update": "2025-10-02T10:30:00Z",
    "avg_crypto_price": 15420.30,
    "avg_stock_price": 285.67,
    "uptime_seconds": 0
  },
  "error": null
}
```

### 6. Force Update Prices

**POST** `/update/{asset_type}`

Force an immediate update of price data.

**Parameters:**

- `asset_type`: "crypto", "stock", or "all"

**Examples:**

```bash
# Update crypto prices
curl -X POST http://localhost:3000/update/crypto

# Update stock prices
curl -X POST http://localhost:3000/update/stock

# Update all prices
curl -X POST http://localhost:3000/update/all
```

**Response:**

```json
{
  "success": true,
  "data": "Updated 9 price feeds",
  "error": null
}
```

## Web3 Integration Examples

### JavaScript/TypeScript

```javascript
class KanariOracle {
  constructor(baseUrl = 'http://localhost:3000') {
    this.baseUrl = baseUrl;
  }

  async getCryptoPrice(symbol) {
    const response = await fetch(`${this.baseUrl}/price/crypto/${symbol}`);
    const data = await response.json();
    return data.success ? data.data : null;
  }

  async getStockPrice(symbol) {
    const response = await fetch(`${this.baseUrl}/price/stock/${symbol}`);
    const data = await response.json();
    return data.success ? data.data : null;
  }

  async getAllCryptoPrices() {
    const response = await fetch(`${this.baseUrl}/prices/crypto`);
    const data = await response.json();
    return data.success ? data.data : [];
  }

  async forceUpdate(assetType = 'all') {
    const response = await fetch(`${this.baseUrl}/update/${assetType}`, {
      method: 'POST'
    });
    const data = await response.json();
    return data.success;
  }
}

// Usage
const oracle = new KanariOracle();
const btcPrice = await oracle.getCryptoPrice('bitcoin');
console.log(`BTC Price: $${btcPrice.price}`);
```

### Python

```python
import requests

class KanariOracle:
    def __init__(self, base_url='http://localhost:3000'):
        self.base_url = base_url

    def get_crypto_price(self, symbol):
        response = requests.get(f'{self.base_url}/price/crypto/{symbol}')
        data = response.json()
        return data['data'] if data['success'] else None

    def get_stock_price(self, symbol):
        response = requests.get(f'{self.base_url}/price/stock/{symbol}')
        data = response.json()
        return data['data'] if data['success'] else None

    def get_all_crypto_prices(self):
        response = requests.get(f'{self.base_url}/prices/crypto')
        data = response.json()
        return data['data'] if data['success'] else []

    def force_update(self, asset_type='all'):
        response = requests.post(f'{self.base_url}/update/{asset_type}')
        data = response.json()
        return data['success']

# Usage
oracle = KanariOracle()
btc_price = oracle.get_crypto_price('bitcoin')
print(f"BTC Price: ${btc_price['price']}")
```

### Rust

```rust
use reqwest::Client;
use serde_json::Value;

pub struct KanariOracle {
    client: Client,
    base_url: String,
}

impl KanariOracle {
    pub fn new(base_url: Option<String>) -> Self {
        Self {
            client: Client::new(),
            base_url: base_url.unwrap_or_else(|| "http://localhost:3000".to_string()),
        }
    }

    pub async fn get_crypto_price(&self, symbol: &str) -> Result<Value, reqwest::Error> {
        let url = format!("{}/price/crypto/{}", self.base_url, symbol);
        let response: Value = self.client.get(&url).send().await?.json().await?;
        Ok(response)
    }

    pub async fn get_stock_price(&self, symbol: &str) -> Result<Value, reqwest::Error> {
        let url = format!("{}/price/stock/{}", self.base_url, symbol);
        let response: Value = self.client.get(&url).send().await?.json().await?;
        Ok(response)
    }

    pub async fn force_update(&self, asset_type: &str) -> Result<Value, reqwest::Error> {
        let url = format!("{}/update/{}", self.base_url, asset_type);
        let response: Value = self.client.post(&url).send().await?.json().await?;
        Ok(response)
    }
}
```

## Error Responses

When an error occurs, the API returns:

```json
{
  "success": false,
  "data": null,
  "error": "Error description here"
}
```

Common error status codes:

- `200 OK`: Success (check `success` field in response)
- `500 Internal Server Error`: Server error

## Configuration

The API server uses the same configuration file as the CLI tool. You can specify API keys for better rate limits:

```json
{
  "crypto": {
    "coingecko_api_key": "your_coingecko_api_key",
    "binance_api_key": "your_binance_api_key",
    "binance_secret_key": "your_binance_secret_key"
  },
  "stocks": {
    "alpha_vantage_api_key": "your_alpha_vantage_key",
    "finnhub_api_key": "your_finnhub_key"
  }
}
```

## CORS Support

The API includes CORS headers to allow cross-origin requests from web browsers.

## Rate Limiting

The Oracle respects upstream API rate limits:

- **CoinGecko**: Without API key: 10-50 calls/minute, With API key: higher limits
- **Binance**: 1200 requests per minute
- **Alpha Vantage**: 5 calls per minute (free tier)
- **Finnhub**: 60 calls per minute (free tier)

## Background Updates

When running the API server, price data is automatically updated in the background at the specified interval (default: 30 seconds).
