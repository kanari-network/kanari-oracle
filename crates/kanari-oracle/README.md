# Kanari Oracle - Real-time Cryptocurrency & Stock Price Oracle

🚀 A robust, real-time price oracle system for fetching cryptocurrency and stock prices from multiple data sources.

## Features

- **Multi-Source Price Fetching**: CoinGecko, Binance, Alpha Vantage, Finnhub, Yahoo Finance
- **Real-time Updates**: Configurable update intervals
- **Fallback System**: Automatic fallback to alternative APIs when primary sources fail
- **Comprehensive Data**: Prices, 24h changes, volume, market cap
- **Error Handling**: Robust retry mechanisms and error recovery
- **CLI Interface**: Easy-to-use command line interface
- **Configurable**: JSON-based configuration system

## Supported Assets

### Cryptocurrencies
- Bitcoin (BTC)
- Ethereum (ETH)
- Binance Coin (BNB)
- Cardano (ADA)
- Solana (SOL)
- Polkadot (DOT)
- Chainlink (LINK)
- Litecoin (LTC)
- Avalanche (AVAX)
- Polygon (MATIC)

### Stocks
- Apple (AAPL)
- Google (GOOGL)
- Microsoft (MSFT)
- Amazon (AMZN)
- Tesla (TSLA)
- Meta (META)
- NVIDIA (NVDA)
- Netflix (NFLX)
- AMD (AMD)
- Intel (INTC)

## Installation

1. **Clone the repository:**
```bash
git clone <repository-url>
cd kanari-oracle
```

2. **Build the project:**
```bash
cargo build --release
```

3. **Run the oracle:**
```bash
cargo run -- --help
```

## Usage

### 1. Start Oracle Service
Start the continuous price monitoring service:

```bash
# Start with default settings (30-second intervals)
cargo run -- start

# Start with custom interval (60 seconds)
cargo run -- start --interval 60

# Start with custom config file
cargo run -- start --config my-config.json
```

### 2. Get Single Price
Fetch current price for a specific asset:

```bash
# Get Bitcoin price
cargo run -- price BTC --asset-type crypto

# Get Apple stock price
cargo run -- price AAPL --asset-type stock
```

### 3. List Available Symbols
View all supported assets:

```bash
# List all assets
cargo run -- list

# List only cryptocurrencies
cargo run -- list --asset-type crypto

# List only stocks
cargo run -- list --asset-type stock
```

## Configuration

On first run, a `config.json` file will be created with default settings. You can edit this file to add API keys and customize behavior:

```json
{
  "crypto": {
    "coingecko_api_key": null,
    "binance_api_key": null,
    "binance_secret_key": null,
    "default_vs_currency": "usd",
    "symbols": [
      "bitcoin",
      "ethereum",
      "binancecoin",
      "cardano",
      "solana"
    ]
  },
  "stocks": {
    "alpha_vantage_api_key": null,
    "finnhub_api_key": null,
    "symbols": [
      "AAPL",
      "GOOGL",
      "MSFT",
      "AMZN",
      "TSLA"
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

### API Keys (Optional but Recommended)

While the oracle works without API keys using free endpoints, adding API keys provides:
- Higher rate limits
- More reliable service
- Additional data points

#### Getting API Keys:

1. **CoinGecko**: [Get API Key](https://www.coingecko.com/api)
2. **Alpha Vantage**: [Get Free API Key](https://www.alphavantage.co/support/#api-key)
3. **Finnhub**: [Get Free API Key](https://finnhub.io/register)
4. **Binance**: [Get API Key](https://www.binance.com/en/my/settings/api-management)

## Example Output

```
=== Current Prices (Last updated: 2025-10-01 10:30:45 UTC) ===

--- Cryptocurrencies ---
Symbol   Price ($)    24h Change   Change %   Source    
----------------------------------------------------------------------
BTC      43250.50     1250.30      2.98%      coingecko 
ETH      3420.75      -45.20       -1.30%     coingecko 
BNB      425.80       8.95         2.15%      coingecko 
ADA      0.45         0.02         4.65%      coingecko 
SOL      95.30        -2.10        -2.15%     coingecko 

--- Stocks ---
Symbol   Price ($)    Change       Change %   Source    
----------------------------------------------------------------------
AAPL     175.25       2.35         1.36%      yahoo_finance
GOOGL    142.80       -1.20        -0.83%     yahoo_finance
MSFT     420.15       5.80         1.40%      yahoo_finance
AMZN     145.90       -0.95        -0.65%     yahoo_finance
TSLA     245.60       8.25         3.48%      yahoo_finance
```

## Architecture

The oracle system consists of several key components:

### Core Components

1. **Oracle Engine** (`src/oracle.rs`)
   - Central coordination system
   - Price feed management
   - Cache management

2. **Price Fetchers** (`src/fetchers/`)
   - `crypto.rs`: CoinGecko, Binance integration
   - `stock.rs`: Alpha Vantage, Finnhub, Yahoo Finance integration

3. **Data Models** (`src/models.rs`)
   - Price data structures
   - API response models

4. **Configuration** (`src/config.rs`)
   - JSON-based configuration
   - API key management

5. **Error Handling** (`src/errors.rs`)
   - Comprehensive error types
   - Retry mechanisms

### Data Flow

1. **Configuration Loading**: Load settings and API keys
2. **Fetcher Initialization**: Initialize price fetchers for each source
3. **Price Fetching**: Retrieve prices from multiple APIs
4. **Data Processing**: Parse and normalize price data
5. **Cache Update**: Update internal price cache
6. **Output**: Display formatted price information

## Error Handling

The oracle includes robust error handling:

- **Network Failures**: Automatic retries with exponential backoff
- **API Failures**: Fallback to alternative data sources
- **Rate Limiting**: Respect API rate limits
- **Data Validation**: Validate price data before processing

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests if applicable
5. Submit a pull request

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Disclaimer

This software is for educational and informational purposes only. Price data is provided by third-party APIs and may not be suitable for trading or investment decisions. Always verify prices through official sources before making financial decisions.

## Support

For issues, questions, or contributions, please open an issue on the GitHub repository.