# Demo Examples for Kanari Oracle

This file contains various usage examples for the Kanari Oracle system.

## Basic Usage Examples

### 1. Get Current Price for Individual Assets

```bash
# Get Bitcoin price (crypto)
cargo run -- price BTC --asset-type crypto

# Get Apple stock price
cargo run -- price AAPL --asset-type stock

# Get Tesla stock price
cargo run -- price TSLA --asset-type stock

# Get Ethereum price
cargo run -- price ETH --asset-type crypto
```

### 2. List Available Assets

```bash
# List all supported assets
cargo run -- list

# List only cryptocurrencies
cargo run -- list --asset-type crypto

# List only stocks
cargo run -- list --asset-type stock
```

### 3. Start Oracle Service

```bash
# Start with default settings (30-second updates)
cargo run -- start

# Start with 60-second intervals
cargo run -- start --interval 60

# Start with custom configuration file
cargo run -- start --config custom-config.json --interval 45
```

## Configuration Examples

### Sample API Configuration

Create a `config.json` file with your API keys:

```json
{
  "crypto": {
    "coingecko_api_key": "your_coingecko_api_key_here",
    "binance_api_key": "your_binance_api_key_here",
    "binance_secret_key": "your_binance_secret_key_here",
    "default_vs_currency": "usd",
    "symbols": [
      "bitcoin",
      "ethereum",
      "binancecoin",
      "cardano",
      "solana",
      "polkadot",
      "chainlink",
      "litecoin"
    ]
  },
  "stocks": {
    "alpha_vantage_api_key": "your_alpha_vantage_api_key_here",
    "finnhub_api_key": "your_finnhub_api_key_here",
    "symbols": [
      "AAPL",
      "GOOGL",
      "MSFT",
      "AMZN",
      "TSLA",
      "META",
      "NVDA",
      "NFLX"
    ]
  },
  "general": {
    "request_timeout": 30,
    "max_retries": 3,
    "retry_delay": 1000,
    "enable_logging": true
  }
}
```

### Minimal Configuration (No API Keys)

The system works without API keys using free endpoints:

```json
{
  "crypto": {
    "coingecko_api_key": null,
    "binance_api_key": null,
    "binance_secret_key": null,
    "coingecko_api_key": null,
    "default_vs_currency": "usd",
    "symbols": ["bitcoin", "ethereum", "cardano"]
  },
  "stocks": {
    "alpha_vantage_api_key": null,
    "finnhub_api_key": null,
    "symbols": ["AAPL", "GOOGL", "TSLA"]
  },
  "general": {
    "request_timeout": 30,
    "max_retries": 3,
    "retry_delay": 1000,
    "enable_logging": true
  }
}
```

## Expected Output Examples

### Single Price Query

```
$ cargo run -- price AAPL --asset-type stock
Current price for AAPL: $255.74
Last updated: 2025-10-01 14:38:41 UTC
```

### Oracle Service Output

```
$ cargo run -- start --interval 30
Starting Kanari Oracle Service...
Oracle initialized successfully
Update interval: 30 seconds

=== Current Prices (Last updated: 2025-10-01 14:40:15 UTC) ===

--- Cryptocurrencies ---
Symbol   Price ($)    24h Change   Change %   Source    
----------------------------------------------------------------------
BTC      43250.50     1250.30      2.98%      coingecko 
ETH      3420.75      -45.20       -1.30%     coingecko 
BNB      425.80       8.95         2.15%      coingecko 

--- Stocks ---
Symbol   Price ($)    Change       Change %   Source    
----------------------------------------------------------------------
AAPL     255.74       2.35         0.93%      yahoo_finance
GOOGL    142.80       -1.20        -0.83%     yahoo_finance
TSLA     245.60       8.25         3.48%      yahoo_finance

Updated 6 price feeds
```

### List Assets Output

```
$ cargo run -- list
Available Cryptocurrencies:
  BTC (crypto)
  ETH (crypto)
  BNB (crypto)
  ADA (crypto)
  SOL (crypto)
  DOT (crypto)
  LINK (crypto)
  LTC (crypto)
  AVAX (crypto)
  MATIC (crypto)

Available Stocks:
  AAPL (stock)
  GOOGL (stock)
  MSFT (stock)
  AMZN (stock)
  TSLA (stock)
  META (stock)
  NVDA (stock)
  NFLX (stock)
  AMD (stock)
  INTC (stock)
```

## API Sources Used

### Cryptocurrencies

- **Primary**: CoinGecko API (comprehensive data, rate limited without API key)
- **Fallback**: Binance API (basic price data, higher rate limits)

### Stocks

- **Primary**: Alpha Vantage (with API key)
- **Secondary**: Finnhub (with API key)  
- **Fallback**: Yahoo Finance (free, no API key required)

## Error Handling Examples

The oracle handles various error scenarios gracefully:

1. **Network Issues**: Automatic retry with exponential backoff
2. **API Rate Limits**: Fallback to alternative data sources
3. **Invalid Symbols**: Clear error messages
4. **Configuration Errors**: Validation and helpful suggestions

## Performance Tips

1. **Use API Keys**: For better rate limits and more reliable service
2. **Adjust Intervals**: Balance between real-time data and API usage
3. **Select Symbols**: Configure only needed assets to reduce API calls
4. **Monitor Logs**: Enable logging to troubleshoot issues

## Troubleshooting

### Common Issues

1. **403 Forbidden**: Usually indicates rate limiting, try:
   - Adding API keys to configuration
   - Increasing update intervals
   - Using fallback APIs

2. **Network Timeouts**: Check:
   - Internet connection
   - Firewall settings
   - API service status

3. **Invalid Price Data**: Verify:
   - Symbol spelling
   - Asset type (crypto vs stock)
   - API service availability
