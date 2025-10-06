use crate::errors::{OracleError, Result};
use serde::{Deserialize, Serialize};
use tokio::fs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub crypto: CryptoConfig,
    #[serde(default)]
    pub stocks: StockConfig,
    #[serde(default)]
    pub general: GeneralConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CryptoConfig {
    pub coingecko_api_key: Option<String>,
    pub coinbase_api_key: Option<String>,
    pub binance_api_key: Option<String>,
    #[serde(default = "default_vs_currency")]
    pub default_vs_currency: String,
    #[serde(default)]
    pub symbols: Vec<String>,
}

fn default_vs_currency() -> String {
    "usd".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StockConfig {
    pub alpha_vantage_api_key: Option<String>,
    pub finnhub_api_key: Option<String>,
    #[serde(default)]
    pub symbols: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig {
    #[serde(default = "default_timeout")]
    pub request_timeout: u64,
    #[serde(default = "default_max_retries")]
    pub max_retries: u32,
    #[serde(default = "default_retry_delay")]
    pub retry_delay: u64,
    #[serde(default = "default_enable_logging")]
    pub enable_logging: bool,
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            request_timeout: default_timeout(),
            max_retries: default_max_retries(),
            retry_delay: default_retry_delay(),
            enable_logging: default_enable_logging(),
        }
    }
}

fn default_timeout() -> u64 {
    30
}
fn default_max_retries() -> u32 {
    3
}
fn default_retry_delay() -> u64 {
    1000
}
fn default_enable_logging() -> bool {
    true
}

impl Default for CryptoConfig {
    fn default() -> Self {
        Self {
            coingecko_api_key: None,
            coinbase_api_key: None,
            binance_api_key: None,
            default_vs_currency: default_vs_currency(),
            symbols: Vec::new(),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            crypto: CryptoConfig {
                symbols: vec![
                    "sui".to_string(),         // Sui
                    "bitcoin".to_string(),     // Bitcoin
                    "ethereum".to_string(),    // Ethereum
                    "tether".to_string(),      // Tether
                    "usd-coin".to_string(),    // USD Coin
                    "binancecoin".to_string(), // Binance Coin
                    "ripple".to_string(),      // XRP
                    "apt".to_string(),         // Aptos
                    "btc".to_string(),         // Bitcoin (Binance)
                    "eth".to_string(),         // Ethereum (Binance)
                ],
                ..Default::default()
            },
            stocks: StockConfig {
                symbols: vec![
                    // Top US Stocks by Market Cap
                    "AAPL".to_string(),  // Apple Inc.
                    "MSFT".to_string(),  // Microsoft Corporation
                    "GOOGL".to_string(), // Alphabet Inc. Class A
                    "AMZN".to_string(),  // Amazon.com Inc.
                    "NVDA".to_string(),  // NVIDIA Corporation
                    "TSLA".to_string(),  // Tesla Inc.
                    "META".to_string(),  // Meta Platforms Inc.
                    "AMD".to_string(),  // Advanced Micro Devices Inc.
                    "INTC".to_string(),  // Intel Corporation
                    "NFLX".to_string(),  // Netflix Inc.
                ],
                ..Default::default()
            },
            general: GeneralConfig {
                request_timeout: default_timeout(),
                max_retries: default_max_retries(),
                retry_delay: default_retry_delay(),
                enable_logging: default_enable_logging(),
            },
        }
    }
}

impl Config {
    pub async fn from_file(path: &str) -> Result<Self> {
        // Check if file exists and get metadata with proper error handling
        let metadata = match fs::metadata(path).await {
            Ok(meta) => meta,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                // File doesn't exist, create default config
                let default_config = Self::default();
                let config_json = serde_json::to_string_pretty(&default_config)?;
                fs::write(path, config_json).await?;
                println!("Created default config file at: {}", path);
                println!("Please edit the config file to add your API keys.");
                return Ok(default_config);
            }
            Err(e) => {
                return Err(OracleError::IoOperationFailed(format!(
                    "Failed to check config file metadata '{}': {}",
                    path, e
                )));
            }
        };

        // Verify that the path points to a regular file, not a directory or symlink
        if !metadata.is_file() {
            return Err(OracleError::ConfigError(format!(
                "Config path '{}' is not a regular file (it might be a directory or symlink)",
                path
            )));
        }

        // Read and parse the config file
        let content = fs::read_to_string(path).await.map_err(|e| {
            OracleError::IoOperationFailed(format!("Failed to read config file '{}': {}", path, e))
        })?;

        let config: Config = serde_json::from_str(&content).map_err(|e| {
            OracleError::ConfigError(format!("Failed to parse config file '{}': {}", path, e))
        })?;

        Ok(config)
    }

    pub fn validate(&self) -> Result<()> {
        if self.crypto.symbols.is_empty() && self.stocks.symbols.is_empty() {
            return Err(OracleError::ConfigError(
                "No symbols configured for crypto or stocks".to_string(),
            ));
        }

        if self.general.request_timeout == 0 {
            return Err(OracleError::ConfigError(
                "Request timeout must be greater than 0".to_string(),
            ));
        }

        Ok(())
    }
}
